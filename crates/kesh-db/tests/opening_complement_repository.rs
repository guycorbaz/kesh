//! Story 25-7 (#445) — complément des soldes de départ, côté base.
//!
//! Couvre `opening_complement::create_opening_complement` (date, contrepartie,
//! refus, et les entrelacements qui prouvent la sérialisation) et
//! `opening_complement::complement_status`. ⚠️ Chaque entrelacement nomme la
//! mutation qu'il tue.

use chrono::{Datelike, NaiveDate, Utc};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, JournalEntryWithLines, NewAccount, NewJournalEntry, NewJournalEntryLine,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::opening_complement::{
    self, ComplementLine, OpeningComplementRefusal as R,
};
use kesh_db::repositories::{accounts, contacts, invoices, journal_entries};
use kesh_db::test_fixtures::{attendre_une_requete_en_cours, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

/// Société de test : les comptes du preset (1000, 1100, 2000, 3000, 4000), un
/// compte de report 2970 de rôle `RetainedEarnings`, les exercices demandés, et
/// une écriture d'ouverture 1000 / 2970 datée du premier jour du premier.
struct Co {
    company_id: i64,
    user_id: i64,
    /// Exercices, dans l'ordre des dates.
    fy: Vec<i64>,
    acc: std::collections::HashMap<&'static str, i64>,
}

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn year_span(y: i32) -> (NaiveDate, NaiveDate) {
    (d(y, 1, 1), d(y, 12, 31))
}

async fn setup(pool: &MySqlPool, fiscal_years: &[(NaiveDate, NaiveDate)]) -> Co {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let company_id = seeded.company_id;
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ?")
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
    let mut fy = Vec::new();
    for (i, (start, end)) in fiscal_years.iter().enumerate() {
        let id = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, ?, ?, ?, 'Open')",
        )
        .bind(company_id)
        .bind(format!("FY{i}"))
        .bind(start)
        .bind(end)
        .execute(pool)
        .await
        .unwrap()
        .last_insert_id() as i64;
        fy.push(id);
    }
    let mut acc = seeded.accounts.clone();
    let retained = accounts::create(
        pool,
        seeded.admin_user_id,
        NewAccount {
            company_id,
            number: "2970".into(),
            name: "Bénéfice reporté".into(),
            account_type: AccountType::Liability,
            parent_id: None,
            role: Some(AccountRole::RetainedEarnings),
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    acc.insert("2970", retained);
    let co = Co {
        company_id,
        user_id: seeded.admin_user_id,
        fy,
        acc,
    };
    let (start, _) = fiscal_years[0];
    post(
        pool,
        &co,
        co.fy[0],
        start,
        &[("1000", dec!(1000), dec!(0)), ("2970", dec!(0), dec!(1000))],
    )
    .await;
    co
}

fn entry(co: &Co, date: NaiveDate, lines: &[(&str, Decimal, Decimal)]) -> NewJournalEntry {
    NewJournalEntry {
        company_id: co.company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "test".into(),
        project_id: None,
        lines: lines
            .iter()
            .map(|(n, debit, credit)| NewJournalEntryLine {
                account_id: co.acc[n],
                debit: *debit,
                credit: *credit,
                project_id: None,
            })
            .collect(),
    }
}

async fn post(
    pool: &MySqlPool,
    co: &Co,
    fy: i64,
    date: NaiveDate,
    lines: &[(&str, Decimal, Decimal)],
) -> JournalEntryWithLines {
    journal_entries::create(pool, fy, co.user_id, entry(co, date, lines))
        .await
        .expect("écriture")
}

fn debit(co: &Co, n: &str, amount: Decimal) -> ComplementLine {
    ComplementLine {
        account_id: co.acc[n],
        debit: amount,
        credit: Decimal::ZERO,
    }
}

fn credit(co: &Co, n: &str, amount: Decimal) -> ComplementLine {
    ComplementLine {
        account_id: co.acc[n],
        debit: Decimal::ZERO,
        credit: amount,
    }
}

async fn complete(
    pool: &MySqlPool,
    co: &Co,
    lines: &[ComplementLine],
    today: NaiveDate,
) -> Result<JournalEntryWithLines, DbError> {
    opening_complement::create_opening_complement(
        pool,
        co.company_id,
        co.user_id,
        lines,
        "Complément des soldes de départ".into(),
        today,
    )
    .await
}

fn refusal(r: Result<JournalEntryWithLines, DbError>) -> (R, Option<i64>) {
    match r {
        Err(DbError::OpeningComplementRefused {
            reason, account_id, ..
        }) => (reason, account_id),
        other => panic!("attendu un refus de complément, obtenu {other:?}"),
    }
}

/// Lignes (compte → (débit, crédit)) d'une écriture.
fn lines_of(e: &JournalEntryWithLines, co: &Co) -> Vec<(&'static str, Decimal, Decimal)> {
    let name = |id: i64| *co.acc.iter().find(|(_, v)| **v == id).unwrap().0;
    let mut v: Vec<_> = e
        .lines
        .iter()
        .map(|l| {
            (
                name(l.account_id),
                l.debit.normalize(),
                l.credit.normalize(),
            )
        })
        .collect();
    v.sort();
    v
}

async fn set_status(pool: &MySqlPool, fy: i64, status: &str) {
    sqlx::query("UPDATE fiscal_years SET status = ? WHERE id = ?")
        .bind(status)
        .bind(fy)
        .execute(pool)
        .await
        .unwrap();
}

async fn lock_books_raw(pool: &MySqlPool, co: &Co, through: NaiveDate) {
    // Directement : `companies::lock_books` refuse une borne >= aujourd'hui,
    // et c'est précisément le cas défensif que l'on veut atteindre.
    sqlx::query("UPDATE companies SET books_locked_through = ? WHERE id = ?")
        .bind(through)
        .bind(co.company_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn count_entries(pool: &MySqlPool, co: &Co, description: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entries WHERE company_id = ? AND description = ?",
    )
    .bind(co.company_id)
    .bind(description)
    .fetch_one(pool)
    .await
    .unwrap()
}

// ============================================================
// Contrepartie
// ============================================================

/// Un actif oublié → au crédit du report ; un passif → au débit.
/// Tue la mutation « sens de la contrepartie inversé ».
#[sqlx::test(migrations = "./test-schema")]
async fn la_contrepartie_va_au_report_dans_le_bon_sens(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let today = d(2026, 6, 1);

    let e = complete(&pool, &co, &[debit(&co, "1100", dec!(250))], today)
        .await
        .unwrap();
    assert_eq!(
        e.entry.entry_date,
        d(2026, 1, 1),
        "branche (a) : date de l'ouverture"
    );
    assert_eq!(e.entry.journal, Journal::OD);
    assert_eq!(
        lines_of(&e, &co),
        vec![("1100", dec!(250), dec!(0)), ("2970", dec!(0), dec!(250))]
    );

    // Un passif oublié : 2000 Capital n'a pas de mouvement dans ce montage.
    let e = complete(&pool, &co, &[credit(&co, "2000", dec!(80.5))], today)
        .await
        .unwrap();
    assert_eq!(
        lines_of(&e, &co),
        vec![("2000", dec!(0), dec!(80.5)), ("2970", dec!(80.5), dec!(0))]
    );
}

/// Deux comptes qui se compensent : aucune contrepartie (une ligne à zéro
/// violerait `chk_jel_debit_credit_exclusive`).
#[sqlx::test(migrations = "./test-schema")]
async fn un_ecart_nul_n_a_pas_de_contrepartie(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let e = complete(
        &pool,
        &co,
        &[debit(&co, "1100", dec!(40)), credit(&co, "2000", dec!(40))],
        d(2026, 6, 1),
    )
    .await
    .unwrap();
    assert_eq!(
        lines_of(&e, &co),
        vec![("1100", dec!(40), dec!(0)), ("2000", dec!(0), dec!(40))]
    );
}

// ============================================================
// Date — arbitrage 2
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn branche_b_premier_exercice_clos_date_du_jour(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2025), year_span(2026)]).await;
    set_status(&pool, co.fy[0], "Closed").await;
    let e = complete(&pool, &co, &[debit(&co, "1100", dec!(1))], d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(
        (e.entry.entry_date, e.entry.fiscal_year_id),
        (d(2026, 6, 1), co.fy[1])
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn branche_b_premier_jour_verrouille_date_du_jour(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    lock_books_raw(&pool, &co, d(2026, 3, 31)).await;
    let e = complete(&pool, &co, &[debit(&co, "1100", dec!(1))], d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(e.entry.entry_date, d(2026, 6, 1));
}

#[sqlx::test(migrations = "./test-schema")]
async fn branche_c_aucun_exercice_ouvert_ne_couvre_le_jour(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2025)]).await;
    set_status(&pool, co.fy[0], "Closed").await;
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(1))], d(2026, 6, 1)).await),
        (R::NoOpenFiscalYear, None)
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn branche_c_aujourd_hui_verrouille(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    lock_books_raw(&pool, &co, d(2026, 6, 1)).await;
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(1))], d(2026, 6, 1)).await),
        (R::DateLocked, None)
    );
}

// ============================================================
// Refus
// ============================================================

/// Tue la mutation « garde jamais mouvementé retirée ».
#[sqlx::test(migrations = "./test-schema")]
async fn refus_compte_mouvemente(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1000", dec!(5))], d(2026, 6, 1)).await),
        (R::AccountMoved, Some(co.acc["1000"]))
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn refus_par_compte(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let today = d(2026, 6, 1);
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "3000", dec!(5))], today).await),
        (R::NotBalanceAccount, Some(co.acc["3000"]))
    );
    assert_eq!(
        refusal(complete(&pool, &co, &[credit(&co, "2970", dec!(5))], today).await),
        (R::RetainedEarningsLine, Some(co.acc["2970"]))
    );
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(co.acc["1100"])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(5))], today).await),
        (R::AccountInvalid, Some(co.acc["1100"]))
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn refus_compte_d_une_autre_societe(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let foreign = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, '1100', 'Banque', 'Asset')",
    )
    .bind(other_company)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let line = ComplementLine {
        account_id: foreign,
        debit: dec!(5),
        credit: Decimal::ZERO,
    };
    let (reason, account_id) = refusal(complete(&pool, &co, &[line], d(2026, 6, 1)).await);
    assert_eq!((reason, account_id), (R::AccountInvalid, Some(foreign)));

    // ⛔ Le compte étranger n'est jamais VERROUILLÉ (revue de code P1, B-F1 :
    // le `FOR UPDATE` par clé primaire le verrouillait avant que le filtre
    // `company_id` ne l'écarte). Une transaction tient ce compte : le complément
    // doit rendre son refus sans attendre.
    let mut holder = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(foreign)
        .execute(&mut *holder)
        .await
        .unwrap();
    let line = ComplementLine {
        account_id: foreign,
        debit: dec!(5),
        credit: Decimal::ZERO,
    };
    let r = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        complete(&pool, &co, &[line], d(2026, 6, 1)),
    )
    .await
    .expect("le complément ne doit pas attendre le verrou d'un compte étranger");
    assert_eq!(refusal(r).0, R::AccountInvalid);
    holder.rollback().await.unwrap();
}

#[sqlx::test(migrations = "./test-schema")]
async fn refus_sans_compte_de_report_ou_non_imputable(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let today = d(2026, 6, 1);
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(co.acc["2970"])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(5))], today).await),
        (R::RetainedEarningsNotPostable, None)
    );
    sqlx::query("UPDATE accounts SET role = NULL WHERE id = ?")
        .bind(co.acc["2970"])
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(5))], today).await),
        (R::NoRetainedEarnings, None)
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn refus_societe_sans_ecriture(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let r = opening_complement::create_opening_complement(
        &pool,
        seeded.company_id,
        seeded.admin_user_id,
        &[ComplementLine {
            account_id: seeded.accounts["1100"],
            debit: dec!(5),
            credit: Decimal::ZERO,
        }],
        "x".into(),
        d(2026, 6, 1),
    )
    .await;
    assert_eq!(refusal(r), (R::NoEntries, None));
}

/// La priorité : `NO_ENTRIES` avant la date, la date avant le report, le
/// report avant les comptes.
#[sqlx::test(migrations = "./test-schema")]
async fn les_refus_suivent_la_priorite(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2025)]).await;
    set_status(&pool, co.fy[0], "Closed").await;
    sqlx::query("UPDATE accounts SET role = NULL WHERE id = ?")
        .bind(co.acc["2970"])
        .execute(&pool)
        .await
        .unwrap();
    // Exercice introuvable ET pas de report ET compte mouvementé : la date parle.
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1000", dec!(5))], d(2026, 6, 1)).await).0,
        R::NoOpenFiscalYear
    );
}

// ============================================================
// Contre-passation et dévalidation (arbitrage 1)
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_contre_passe_reste_mouvemente(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let e = post(
        &pool,
        &co,
        co.fy[0],
        d(2026, 2, 1),
        &[("1100", dec!(9), dec!(0)), ("2000", dec!(0), dec!(9))],
    )
    .await;
    journal_entries::reverse(&pool, co.company_id, e.entry.id, co.user_id)
        .await
        .unwrap();
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(5))], d(2026, 6, 1)).await).0,
        R::AccountMoved
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn un_compte_libere_par_la_devalidation_redevient_completable(pool: MySqlPool) {
    // L'exercice du preset couvre 2020-2030 : la facture de 2026 y tombe.
    let co = setup(&pool, &[(d(2020, 1, 1), d(2030, 12, 31))]).await;
    let contact = contacts::create(
        &pool,
        co.user_id,
        NewContact {
            company_id: co.company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap();
    let (invoice, _) = invoices::create(
        &pool,
        co.user_id,
        NewInvoice {
            company_id: co.company_id,
            contact_id: contact.id,
            date: d(2026, 4, 14),
            due_date: None,
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Conseil".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(&pool, co.company_id, invoice.id, co.user_id)
        .await
        .unwrap();
    // Le compte débiteurs (1100) est mouvementé par la facture, et par elle seule.
    assert_eq!(
        refusal(complete(&pool, &co, &[debit(&co, "1100", dec!(5))], d(2026, 6, 1)).await).0,
        R::AccountMoved
    );
    let version = invoices::find_by_id(&pool, co.company_id, invoice.id)
        .await
        .unwrap()
        .unwrap()
        .version;
    invoices::unvalidate(&pool, co.company_id, invoice.id, co.user_id, version)
        .await
        .unwrap();
    complete(&pool, &co, &[debit(&co, "1100", dec!(5))], d(2026, 6, 1))
        .await
        .expect("1100 redevenu complétable");
}

// ============================================================
// Status
// ============================================================

#[sqlx::test(migrations = "./test-schema")]
async fn le_status_annonce_le_complement(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let s = opening_complement::complement_status(&pool, co.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert!(s.can_complete());
    assert_eq!(s.complete_reason(), "READY");
    let numbers: Vec<_> = s
        .completable_accounts
        .iter()
        .map(|a| a.number.as_str())
        .collect();
    // 1000 et 2970 sont mouvementés (l'ouverture), 3000/4000 sont de résultat.
    assert_eq!(numbers, vec!["1100", "2000"]);
    assert_eq!(s.date.as_ref().unwrap().date, d(2026, 1, 1));
    assert_eq!(s.retained_earnings.as_ref().unwrap().number, "2970");

    // Plus aucun compte complétable.
    complete(
        &pool,
        &co,
        &[debit(&co, "1100", dec!(1)), credit(&co, "2000", dec!(1))],
        d(2026, 6, 1),
    )
    .await
    .unwrap();
    let s = opening_complement::complement_status(&pool, co.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(
        (s.can_complete(), s.complete_reason()),
        (false, "NO_COMPLETABLE_ACCOUNT")
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn le_status_d_une_societe_sans_ecriture(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let s = opening_complement::complement_status(&pool, seeded.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(
        (s.can_complete(), s.complete_reason()),
        (false, "NO_ENTRIES")
    );
}

#[sqlx::test(migrations = "./test-schema")]
async fn le_status_rend_la_raison_prioritaire(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    sqlx::query("UPDATE accounts SET role = NULL WHERE id = ?")
        .bind(co.acc["2970"])
        .execute(&pool)
        .await
        .unwrap();
    let s = opening_complement::complement_status(&pool, co.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(s.complete_reason(), "NO_RETAINED_EARNINGS");
    lock_books_raw(&pool, &co, d(2026, 6, 1)).await;
    let s = opening_complement::complement_status(&pool, co.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(s.complete_reason(), "DATE_LOCKED");
}

/// Les deux raisons que seul le POST exerçait (revue de code P1, A-M1).
#[sqlx::test(migrations = "./test-schema")]
async fn le_status_rend_no_open_fiscal_year_et_report_non_imputable(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(co.acc["2970"])
        .execute(&pool)
        .await
        .unwrap();
    let s = opening_complement::complement_status(&pool, co.company_id, d(2026, 6, 1))
        .await
        .unwrap();
    assert_eq!(
        (s.can_complete(), s.complete_reason()),
        (false, "RETAINED_EARNINGS_NOT_POSTABLE")
    );

    // La date prime sur le report : premier exercice clos, rien d'ouvert pour le jour.
    set_status(&pool, co.fy[0], "Closed").await;
    let s = opening_complement::complement_status(&pool, co.company_id, d(2027, 6, 1))
        .await
        .unwrap();
    assert_eq!(s.complete_reason(), "NO_OPEN_FISCAL_YEAR");
}

// ============================================================
// Entrelacements
// ============================================================

/// (1) Une écriture d'un AUTRE exercice, lignes insérées non validées sur le
/// compte visé : le complément attend sur le verrou des comptes, puis refuse.
///
/// Tue « comptes saisis sans `FOR UPDATE` » (l'instantané s'ouvrirait avant la
/// validation de l'écriture) et « lecture jamais mouvementé avant le verrou ».
#[sqlx::test(migrations = "./test-schema")]
async fn entrelacement_1_ecriture_en_vol_sur_le_compte_vise(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026), year_span(2027)]).await;
    let mut tx = pool.begin().await.unwrap();
    journal_entries::create_in_tx(
        &mut tx,
        co.fy[1],
        co.user_id,
        entry(
            &co,
            d(2027, 2, 1),
            &[("1100", dec!(7), dec!(0)), ("1000", dec!(0), dec!(7))],
        ),
        true,
    )
    .await
    .unwrap();

    let task = {
        let pool = pool.clone();
        let line = debit(&co, "1100", dec!(5));
        let (cid, uid) = (co.company_id, co.user_id);
        tokio::spawn(async move {
            opening_complement::create_opening_complement(
                &pool,
                cid,
                uid,
                &[line],
                "c".into(),
                d(2026, 6, 1),
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts WHERE id IN", "FOR UPDATE"], || task
            .is_finished())
        .await,
        "le complément doit attendre sur le verrou des comptes"
    );
    tx.commit().await.unwrap();
    assert_eq!(refusal(task.await.unwrap()).0, R::AccountMoved);
}

/// (2) Une écriture du MÊME exercice, sans compte visé, qui tient l'exercice :
/// le complément attend sur l'exercice, puis réussit — aucun interblocage.
///
/// Tue « société prise en exclusif » : l'écriture, une fois l'exercice tenu,
/// demande `companies` en partagé pour son en-tête.
#[sqlx::test(migrations = "./test-schema")]
async fn entrelacement_2_ecriture_du_meme_exercice_sans_compte_vise(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(co.fy[0])
        .execute(&mut *tx)
        .await
        .unwrap();

    let task = {
        let pool = pool.clone();
        let line = debit(&co, "1100", dec!(5));
        let (cid, uid) = (co.company_id, co.user_id);
        tokio::spawn(async move {
            opening_complement::create_opening_complement(
                &pool,
                cid,
                uid,
                &[line],
                "c".into(),
                d(2026, 6, 1),
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &[
                "FROM fiscal_years",
                "ORDER BY start_date LIMIT 1 FOR UPDATE"
            ],
            || task.is_finished()
        )
        .await,
        "le complément doit attendre sur l'exercice"
    );
    journal_entries::create_in_tx(
        &mut tx,
        co.fy[0],
        co.user_id,
        entry(
            &co,
            d(2026, 3, 1),
            &[("1000", dec!(3), dec!(0)), ("2970", dec!(0), dec!(3))],
        ),
        true,
    )
    .await
    .expect("l'écriture tenant l'exercice passe : aucun interblocage");
    tx.commit().await.unwrap();
    task.await.unwrap().expect("puis le complément passe");
}

/// (3) Un archivage du compte visé, en vol : le complément attend, puis refuse.
///
/// Tue « comptes saisis sans `FOR UPDATE` » : sans verrou, le complément
/// validerait une ligne sur un compte archivé (la clé étrangère ne contrôle
/// pas `active`).
#[sqlx::test(migrations = "./test-schema")]
async fn entrelacement_3_archivage_en_vol(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(co.acc["1100"])
        .execute(&mut *tx)
        .await
        .unwrap();

    let task = {
        let pool = pool.clone();
        let line = debit(&co, "1100", dec!(5));
        let (cid, uid) = (co.company_id, co.user_id);
        tokio::spawn(async move {
            opening_complement::create_opening_complement(
                &pool,
                cid,
                uid,
                &[line],
                "c".into(),
                d(2026, 6, 1),
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM accounts WHERE id IN", "FOR UPDATE"], || task
            .is_finished())
        .await
    );
    tx.commit().await.unwrap();
    assert_eq!(refusal(task.await.unwrap()).0, R::AccountInvalid);
}

/// (4) Deux compléments du même compte : un seul réussit. Lancés par
/// `tokio::join!`, sans attente observée : une exécution l'un après l'autre
/// satisfait aussi l'assertion (revue de code P2, F-7) — la sérialisation sur le
/// verrou des comptes est prouvée par (1) et (3), pas par celui-ci.
#[sqlx::test(migrations = "./test-schema")]
async fn entrelacement_4_deux_complements_du_meme_compte(pool: MySqlPool) {
    let co = setup(&pool, &[year_span(2026)]).await;
    let today = d(2026, 6, 1);
    let lines = [debit(&co, "1100", dec!(5))];
    let (a, b) = tokio::join!(
        complete(&pool, &co, &lines, today),
        complete(&pool, &co, &lines, today)
    );
    let ok = [a.is_ok(), b.is_ok()].iter().filter(|x| **x).count();
    assert_eq!(ok, 1, "un seul complément réussit : {a:?} / {b:?}");
    let refused = if a.is_err() { a } else { b };
    assert_eq!(refusal(refused).0, R::AccountMoved);
}

/// (5) Deux exercices, une contre-passation d'une écriture de l'exercice du
/// jour en vol. L'ordre d'obtention n'est pas maîtrisé ; l'assertion porte sur
/// l'état final : chaque issue est un succès ou un interblocage, et la base
/// reflète exactement les succès.
///
/// ⚠️ **Ce que ce test ne prouve PAS** (revue de code P2, F-2) : que le cycle se
/// forme, ni qu'un ordre de verrous plutôt qu'un autre est tenu — la base
/// reflète les succès par la seule atomicité des transactions. Il prouve
/// l'absence d'**autre** erreur qu'un interblocage, et aucune mutation de l'ordre
/// des verrous ne le ferait rougir. Le rejeu reste sans test (fiche, Limites).
#[sqlx::test(migrations = "./test-schema")]
async fn entrelacement_5_contre_passation_en_vol(pool: MySqlPool) {
    let y = Utc::now().date_naive().year();
    let co = setup(&pool, &[year_span(y - 1), year_span(y)]).await;
    let today = Utc::now().date_naive();
    let original = post(
        &pool,
        &co,
        co.fy[1],
        d(y, 1, 2),
        &[("1000", dec!(4), dec!(0)), ("2970", dec!(0), dec!(4))],
    )
    .await;

    let mut blocker = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(co.fy[1])
        .execute(&mut *blocker)
        .await
        .unwrap();

    let c = {
        let pool = pool.clone();
        let line = debit(&co, "1100", dec!(5));
        let (cid, uid) = (co.company_id, co.user_id);
        tokio::spawn(async move {
            opening_complement::create_opening_complement(
                &pool,
                cid,
                uid,
                &[line],
                "complement-5".into(),
                today,
            )
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["FROM fiscal_years", "start_date DESC LIMIT 1 FOR UPDATE"],
            || c.is_finished()
        )
        .await
    );
    let r = {
        let pool = pool.clone();
        let (cid, uid, id) = (co.company_id, co.user_id, original.entry.id);
        tokio::spawn(async move { journal_entries::reverse(&pool, cid, id, uid).await })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["JOIN fiscal_years", "FOR UPDATE"], || r
            .is_finished())
        .await
    );
    blocker.rollback().await.unwrap();

    let deadlock = |e: &DbError| kesh_db::retry::is_deadlock_error(e);
    let c = c.await.unwrap();
    let r = r.await.unwrap();
    assert!(
        c.as_ref().is_ok() || c.as_ref().is_err_and(deadlock),
        "complément : {c:?}"
    );
    assert!(
        r.as_ref().is_ok() || r.as_ref().is_err_and(deadlock),
        "contre-passation : {r:?}"
    );
    assert_eq!(
        count_entries(&pool, &co, "complement-5").await,
        i64::from(c.is_ok())
    );
    let reversals: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(original.entry.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reversals, i64::from(r.is_ok()));
}
