//! Tests d'intégration Story 14-4 — bilan d'ouverture (soldes de départ).
//!
//! Couvre les helpers repo (`journal_entries::count_by_company`,
//! `accounts::find_types_by_ids_in_tx`) et la fn dédiée atomique
//! `journal_entries::create_opening_entry` (garde « company vierge » +
//! statut exercice sous `fiscal_years FOR UPDATE`, P1-C1/P3-BH3-1),
//! y compris la course concurrente (P1-M3-BH, miroir
//! `reopen_close_concurrent_is_serialized` de 14-2).

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry,
    NewJournalEntryLine, NewUser, OrgType, Role,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::{
    FY_OPENING_ALREADY_HAS_ENTRIES_KEY, FY_OPENING_FIRST_YEAR_CLOSED_KEY,
};
use kesh_db::repositories::{accounts, audit_log, companies, fiscal_years, journal_entries, users};
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ============================================================
// Seed helpers minimal (miroir report_aggregates.rs)
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "X".into(),
                building: String::new(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$0000000000000000000000000000000000000000000".into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_fy(pool: &MySqlPool, user_id: i64, company_id: i64, name: &str, year: i32) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: name.into(),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
    role: Option<AccountRole>,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            role,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// Écriture d'ouverture standard : Débit actif / Crédit report à-nouveau.
fn opening_entry(
    company_id: i64,
    entry_date: NaiveDate,
    asset_id: i64,
    retained_id: i64,
) -> NewJournalEntry {
    NewJournalEntry {
        company_id,
        entry_date,
        journal: Journal::OD,
        description: "Bilan d'ouverture".into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                account_id: asset_id,
                debit: dec!(1000),
                credit: dec!(0),
                project_id: None,
            },
            NewJournalEntryLine {
                account_id: retained_id,
                debit: dec!(0),
                credit: dec!(1000),
                project_id: None,
            },
        ],
    }
}

// ============================================================
// journal_entries::count_by_company
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn count_by_company_zero_then_n(pool: MySqlPool) {
    let cid = create_company(&pool, "co-count").await;
    let uid = create_user(&pool, "u-count", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0
    );

    let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    journal_entries::create(&pool, fy, uid, opening_entry(cid, date, asset, retained))
        .await
        .unwrap();

    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );

    // Générique `Executor` (P3-ECH-LOW-2) : appelable aussi sur `&mut *tx`.
    let mut tx = pool.begin().await.unwrap();
    assert_eq!(
        journal_entries::count_by_company(&mut *tx, cid)
            .await
            .unwrap(),
        1
    );
    tx.rollback().await.unwrap();
}

/// Le comptage est scopé company : les écritures d'une autre company ne
/// comptent pas.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn count_by_company_is_tenant_scoped(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co-count-a").await;
    let cid_b = create_company(&pool, "co-count-b").await;
    let uid_b = create_user(&pool, "u-count-b", cid_b).await;
    let fy_b = create_fy(&pool, uid_b, cid_b, "FY2026", 2026).await;
    let asset_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;
    let liab_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    journal_entries::create(
        &pool,
        fy_b,
        uid_b,
        opening_entry(cid_b, date, asset_b, liab_b),
    )
    .await
    .unwrap();

    assert_eq!(
        journal_entries::count_by_company(&pool, cid_a)
            .await
            .unwrap(),
        0
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid_b)
            .await
            .unwrap(),
        1
    );
}

// ============================================================
// accounts::find_types_by_ids_in_tx
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_types_by_ids_returns_types_scoped_to_company(pool: MySqlPool) {
    let cid = create_company(&pool, "co-types").await;
    let uid = create_user(&pool, "u-types", cid).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;
    let revenue = create_acc(
        &pool,
        uid,
        cid,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;

    // Compte d'une AUTRE company : absent du résultat (pas d'erreur).
    let cid_other = create_company(&pool, "co-types-other").await;
    let uid_other = create_user(&pool, "u-types-other", cid_other).await;
    let foreign = create_acc(
        &pool,
        uid_other,
        cid_other,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;

    let mut tx = pool.begin().await.unwrap();
    let mut types =
        accounts::find_types_by_ids_in_tx(&mut tx, cid, &[asset, liab, revenue, foreign])
            .await
            .unwrap();
    types.sort();
    tx.rollback().await.unwrap();

    let mut expected = vec![
        (asset, "Asset".to_string()),
        (liab, "Liability".to_string()),
        (revenue, "Revenue".to_string()),
    ];
    expected.sort();
    // `foreign` (autre company) est ABSENT — il retombera dans la garde
    // `InactiveOrInvalidAccounts` de `create_in_tx` (contrat AC-B).
    assert_eq!(types, expected);
}

/// `ids` vide → `Ok(vec![])` sans requête SQL (un `IN ()` vide serait une
/// erreur de syntaxe MariaDB → 500, P3-ECH-LOW-1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_types_by_ids_empty_ids_ok(pool: MySqlPool) {
    let cid = create_company(&pool, "co-types-empty").await;
    let mut tx = pool.begin().await.unwrap();
    let types = accounts::find_types_by_ids_in_tx(&mut tx, cid, &[])
        .await
        .unwrap();
    tx.rollback().await.unwrap();
    assert!(types.is_empty());
}

// ============================================================
// journal_entries::create_opening_entry
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_opening_entry_happy_path_od_with_audit(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open").await;
    let uid = create_user(&pool, "u-open", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let created = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, fy_start, asset, retained),
    )
    .await
    .unwrap();

    assert_eq!(created.entry.journal, Journal::OD);
    assert_eq!(created.entry.entry_date, fy_start);
    assert_eq!(created.entry.fiscal_year_id, fy);
    assert_eq!(created.lines.len(), 2);
    assert_eq!(created.lines[0].debit, dec!(1000));
    assert_eq!(created.lines[1].credit, dec!(1000));

    // Audit `journal_entry.created` présent (posé par create_in_tx).
    let entries = audit_log::find_by_entity(&pool, "journal_entry", created.entry.id, 10)
        .await
        .unwrap();
    assert!(
        entries.iter().any(|e| e.action == "journal_entry.created"),
        "audit journal_entry.created attendu"
    );
}

/// Garde « company vierge » (P3-BH3-1) : une écriture existante dans un
/// AUTRE exercice bloque aussi la génération (company-wide, pas
/// premier-exercice-seulement) — aucune écriture ni audit supplémentaire.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_opening_entry_rejected_if_company_has_entries_in_any_fy(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-busy").await;
    let uid = create_user(&pool, "u-open-busy", cid).await;
    let fy_first = create_fy(&pool, uid, cid, "FY2025", 2025).await;
    let fy_later = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    // Écriture normale dans l'exercice POSTÉRIEUR (pas le premier).
    let d_later = NaiveDate::from_ymd_opt(2026, 6, 1).unwrap();
    journal_entries::create(
        &pool,
        fy_later,
        uid,
        opening_entry(cid, d_later, asset, liab),
    )
    .await
    .unwrap();

    let fy_start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy_first,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(&err, DbError::Invariant(k) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "attendu Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY), obtenu {err:?}"
    );
    // Aucune 2e écriture insérée.
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );
}

/// Premier exercice `Closed` → `Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)`
/// (re-check sous le lock — le code 14-4 distinct, pas le générique
/// `FiscalYearClosed`, L5).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_opening_entry_rejected_if_first_year_closed(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-closed").await;
    let uid = create_user(&pool, "u-open-closed", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    fiscal_years::close(&pool, uid, cid, fy).await.unwrap();

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        fy,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(&err, DbError::Invariant(k) if k == FY_OPENING_FIRST_YEAR_CLOSED_KEY),
        "attendu Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY), obtenu {err:?}"
    );
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        0
    );
}

/// Exercice inexistant (ou d'une autre company) → `NotFound`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_opening_entry_not_found_for_missing_fy(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-404").await;
    let uid = create_user(&pool, "u-open-404", cid).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let liab = create_acc(
        &pool,
        uid,
        cid,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let err = journal_entries::create_opening_entry(
        &pool,
        cid,
        999_999,
        uid,
        opening_entry(cid, fy_start, asset, liab),
    )
    .await
    .unwrap_err();

    assert!(
        matches!(err, DbError::NotFound),
        "attendu NotFound, obtenu {err:?}"
    );
}

/// Course concurrente (P1-M3-BH, miroir `reopen_close_concurrent_is_serialized`
/// de 14-2) : deux générations simultanées sur la même company vierge — le
/// `fiscal_years FOR UPDATE` + garde `count_by_company` sous le lock
/// sérialisent : exactement UNE écriture créée, l'autre reçoit
/// `ALREADY_HAS_ENTRIES`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_opening_entry_concurrent_generation_is_serialized(pool: MySqlPool) {
    let cid = create_company(&pool, "co-open-race").await;
    let uid = create_user(&pool, "u-open-race", cid).await;
    let fy = create_fy(&pool, uid, cid, "FY2026", 2026).await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset, None).await;
    let retained = create_acc(
        &pool,
        uid,
        cid,
        "2970",
        "Report",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;

    let fy_start = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let p1 = pool.clone();
    let p2 = pool.clone();
    let new1 = opening_entry(cid, fy_start, asset, retained);
    let new2 = opening_entry(cid, fy_start, asset, retained);

    let t1 = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p1, cid, fy, uid, new1).await
    });
    let t2 = tokio::spawn(async move {
        journal_entries::create_opening_entry(&p2, cid, fy, uid, new2).await
    });

    let (r1, r2) = tokio::join!(t1, t2);
    let r1 = r1.expect("task 1 should not panic");
    let r2 = r2.expect("task 2 should not panic");

    let ok_count = [&r1, &r2].iter().filter(|r| r.is_ok()).count();
    assert_eq!(
        ok_count, 1,
        "exactement une génération doit réussir: {r1:?} / {r2:?}"
    );
    let loser = if r1.is_err() { r1 } else { r2 };
    assert!(
        matches!(&loser, Err(DbError::Invariant(k)) if k == FY_OPENING_ALREADY_HAS_ENTRIES_KEY),
        "le perdant doit recevoir ALREADY_HAS_ENTRIES, obtenu {loser:?}"
    );

    // État final : UNE seule écriture, pas de doublon.
    assert_eq!(
        journal_entries::count_by_company(&pool, cid).await.unwrap(),
        1
    );
}
