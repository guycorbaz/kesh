//! Story 34-2 (#533) — la portée `SingleBankLedger` du complément des soldes de
//! départ, côté base : le solde d'ouverture d'un compte bancaire passe par le
//! mécanisme du mode *Compléter*, dans la transaction de l'appelant, sur une
//! société **sans écriture** (le cas nominal d'une installation neuve).
//!
//! Les tests de la portée `Screen` restent ceux de
//! `opening_complement_repository.rs`, inchangés (AC-1). Mutations tuées :
//! M1 (portée ignorée) et M7 (« aucun exercice » confondu avec « aucune
//! écriture »).

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::entities::{AccountRole, JournalEntryWithLines, NewAccount};
use kesh_db::errors::DbError;
use kesh_db::repositories::accounts;
use kesh_db::repositories::opening_complement::{
    self, ComplementLine, ComplementScope, OpeningComplementRefusal as R,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

/// Société du preset (exercice 2020-2030 ouvert, aucune écriture), plus un
/// compte de report 2970 et un compte bancaire 1020 neuf. Rend l'id de 1020 et
/// celui de 2970.
async fn setup(pool: &MySqlPool) -> (SeededCompany, i64, i64) {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let mk = |number: &'static str, name: &'static str, ty, role| NewAccount {
        company_id: seeded.company_id,
        number: number.into(),
        name: name.into(),
        account_type: ty,
        parent_id: None,
        role,
        postable: true,
    };
    let retained = accounts::create(
        pool,
        seeded.admin_user_id,
        mk(
            "2970",
            "Bénéfice reporté",
            AccountType::Liability,
            Some(AccountRole::RetainedEarnings),
        ),
    )
    .await
    .unwrap()
    .id;
    let bank = accounts::create(
        pool,
        seeded.admin_user_id,
        mk("1020", "Banque", AccountType::Asset, None),
    )
    .await
    .unwrap()
    .id;
    (seeded, bank, retained)
}

/// Le complément en transaction, conclu par un commit sur succès.
async fn complete_in_tx(
    pool: &MySqlPool,
    seeded: &SeededCompany,
    line: ComplementLine,
    scope: ComplementScope,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.unwrap();
    let account = line.account_id;
    let r = opening_complement::create_opening_complement_in_tx(
        &mut tx,
        seeded.company_id,
        seeded.admin_user_id,
        &[line],
        &[account],
        "Solde d'ouverture — UBS".into(),
        d(2026, 6, 1),
        scope,
    )
    .await;
    match &r {
        Ok(_) => tx.commit().await.unwrap(),
        Err(_) => tx.rollback().await.unwrap(),
    }
    r
}

fn refusal(r: Result<JournalEntryWithLines, DbError>) -> R {
    match r {
        Err(DbError::OpeningComplementRefused { reason, .. }) => reason,
        other => panic!("attendu un refus de complément, obtenu {other:?}"),
    }
}

async fn entries(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// AC-4 — société sans écriture, premier exercice ouvert : la portée
/// `SingleBankLedger` passe l'écriture, datée du premier jour du premier
/// exercice ; contrepartie au report. Tue M1 (portée ignorée : `NoEntries`).
#[sqlx::test(migrations = "./test-schema")]
async fn single_bank_ledger_passe_sur_une_societe_sans_ecriture(pool: MySqlPool) {
    let (seeded, bank, retained) = setup(&pool).await;
    assert_eq!(entries(&pool, seeded.company_id).await, 0);

    let e = complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: dec!(1234.50),
            credit: Decimal::ZERO,
        },
        ComplementScope::SingleBankLedger,
    )
    .await
    .expect("écriture d'ouverture");

    assert_eq!(
        e.entry.entry_date,
        d(2020, 1, 1),
        "premier jour du premier exercice"
    );
    assert_eq!(e.entry.fiscal_year_id, seeded.fiscal_year_id);
    assert_eq!(e.entry.description, "Solde d'ouverture — UBS");
    let mut lines: Vec<_> = e
        .lines
        .iter()
        .map(|l| (l.account_id, l.debit.normalize(), l.credit.normalize()))
        .collect();
    lines.sort();
    let mut attendu = vec![
        (bank, dec!(1234.5), Decimal::ZERO),
        (retained, Decimal::ZERO, dec!(1234.5)),
    ];
    attendu.sort();
    assert_eq!(lines, attendu);
    assert_eq!(entries(&pool, seeded.company_id).await, 1);
}

/// Un solde créditeur (une dette, sens *Dette*) : le compte lié au crédit, le
/// report au débit.
#[sqlx::test(migrations = "./test-schema")]
async fn single_bank_ledger_credit_contrepartie_au_debit(pool: MySqlPool) {
    let (seeded, bank, retained) = setup(&pool).await;
    let e = complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: Decimal::ZERO,
            credit: dec!(80),
        },
        ComplementScope::SingleBankLedger,
    )
    .await
    .unwrap();
    let report = e.lines.iter().find(|l| l.account_id == retained).unwrap();
    assert_eq!(
        (report.debit.normalize(), report.credit.normalize()),
        (dec!(80), Decimal::ZERO)
    );
}

/// AC-1 — la portée `Screen`, appelée par la même fonction, garde son refus
/// `NoEntries` sur une société sans écriture.
#[sqlx::test(migrations = "./test-schema")]
async fn screen_garde_no_entries(pool: MySqlPool) {
    let (seeded, bank, _) = setup(&pool).await;
    let r = complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: dec!(5),
            credit: Decimal::ZERO,
        },
        ComplementScope::Screen,
    )
    .await;
    assert_eq!(refusal(r), R::NoEntries);
    assert_eq!(entries(&pool, seeded.company_id).await, 0);
}

/// AC-4 — sans aucun exercice : `NoOpenFiscalYear`, jamais `NoEntries`, rien
/// d'écrit. Tue M7.
#[sqlx::test(migrations = "./test-schema")]
async fn single_bank_ledger_sans_exercice_refuse_no_open_fiscal_year(pool: MySqlPool) {
    let (seeded, bank, _) = setup(&pool).await;
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ?")
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let r = complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: dec!(5),
            credit: Decimal::ZERO,
        },
        ComplementScope::SingleBankLedger,
    )
    .await;
    assert_eq!(refusal(r), R::NoOpenFiscalYear);
    assert_eq!(entries(&pool, seeded.company_id).await, 0);
}

/// Mêmes refus que *Compléter* : un compte déjà mouvementé est refusé
/// `AccountMoved`, même en portée `SingleBankLedger`.
#[sqlx::test(migrations = "./test-schema")]
async fn single_bank_ledger_refuse_un_compte_mouvemente(pool: MySqlPool) {
    let (seeded, bank, _) = setup(&pool).await;
    complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: dec!(5),
            credit: Decimal::ZERO,
        },
        ComplementScope::SingleBankLedger,
    )
    .await
    .unwrap();
    let r = complete_in_tx(
        &pool,
        &seeded,
        ComplementLine {
            account_id: bank,
            debit: dec!(7),
            credit: Decimal::ZERO,
        },
        ComplementScope::SingleBankLedger,
    )
    .await;
    assert_eq!(refusal(r), R::AccountMoved);
    assert_eq!(entries(&pool, seeded.company_id).await, 1);
}
