//! Tests sqlx Story 9-1 — agrégats SQL des rapports comptables.
//!
//! Niveau repository : teste directement `kesh_report::generate_*` sans HTTP,
//! pour valider les patterns SQL (COALESCE, période bounds, fy isolation,
//! multi-tenant, exercice non-calendaire).
//!
//! Complémentaire de `crates/kesh-api/tests/reports_e2e.rs` qui teste le
//! end-to-end HTTP (parsing JWT, response shape, codes erreur).
//!
//! Pattern : `#[sqlx::test]` avec migrator. Seed inline minimal.
#![allow(clippy::too_many_arguments)]

use chrono::NaiveDate;
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry, NewJournalEntryLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use kesh_report::ReportPeriod;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use sqlx::MySqlPool;

// ============================================================
// Seed helpers minimal
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

async fn create_fy(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    name: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: name.into(),
            start_date: start,
            end_date: end,
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
        },
    )
    .await
    .unwrap()
    .id
}

async fn post_entry(
    pool: &MySqlPool,
    user_id: i64,
    fy_id: i64,
    company_id: i64,
    date: NaiveDate,
    journal: Journal,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        fy_id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal,
            description: "test".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
}

// ============================================================
// Tests 1-7
// ============================================================

/// Test 1 : balance_sheet sur 0 écriture → COALESCE retourne 0, équation tient
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn balance_sheet_empty_returns_zero_with_equation(pool: MySqlPool) {
    let cid = create_company(&pool, "co1").await;
    let uid = create_user(&pool, "u1", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(bs.total_assets, Decimal::ZERO);
    assert_eq!(bs.total_liabilities, Decimal::ZERO);
    assert_eq!(bs.equity_result, Decimal::ZERO);
    assert!(bs.equation_holds);
}

/// Test 2 : fiscal_year non-calendaire (juillet → juin), isolation correcte
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fiscal_year_non_calendar_isolation(pool: MySqlPool) {
    let cid = create_company(&pool, "co2").await;
    let uid = create_user(&pool, "u2", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2027, 6, 30).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "Ventes", AccountType::Revenue).await;
    // Écriture au milieu (calendrier qui chevauche 2 années civiles)
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2027, 1, 15).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(500),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    assert!(tb.balanced);
    assert_eq!(tb.total_debit, dec!(500));
}

/// Test 3 : multi-tenant strict cross-company aggregation = 0
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cross_tenant_aggregation_returns_zero(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co3a").await;
    let cid_b = create_company(&pool, "co3b").await;
    let uid_a = create_user(&pool, "ua", cid_a).await;
    let uid_b = create_user(&pool, "ub", cid_b).await;
    let fy_a = create_fy(
        &pool,
        uid_a,
        cid_a,
        "FYA",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let fy_b = create_fy(
        &pool,
        uid_b,
        cid_b,
        "FYB",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset_b = create_acc(&pool, uid_b, cid_b, "1000", "Banque B", AccountType::Asset).await;
    let revenue_b = create_acc(
        &pool,
        uid_b,
        cid_b,
        "4000",
        "Ventes B",
        AccountType::Revenue,
    )
    .await;
    post_entry(
        &pool,
        uid_b,
        fy_b,
        cid_b,
        NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
        Journal::Banque,
        asset_b,
        revenue_b,
        dec!(1234),
    )
    .await;

    // User A fait trial_balance sur fy_a (sans écritures, pas d'accounts non plus)
    let period = ReportPeriod::resolve(&pool, cid_a, fy_a, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid_a, &period)
        .await
        .unwrap();
    assert_eq!(tb.total_debit, Decimal::ZERO);
    assert_eq!(tb.total_credit, Decimal::ZERO);
}

/// Test 4 : période bounds inclusives (start et end exactement sur fy bornes)
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn period_bounds_inclusive(pool: MySqlPool) {
    let cid = create_company(&pool, "co4").await;
    let uid = create_user(&pool, "u4", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "B", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "V", AccountType::Revenue).await;
    // Écriture pile sur le 1er jour
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(100),
    )
    .await;
    // Écriture pile sur le dernier jour
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(200),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    // Les 2 écritures sont incluses → total 300 débit + 300 crédit
    assert_eq!(tb.total_debit, dec!(300));
    assert_eq!(tb.total_credit, dec!(300));
}

/// Test 5 : partial period exclut les écritures hors bornes
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn partial_period_excludes_outside_entries(pool: MySqlPool) {
    let cid = create_company(&pool, "co5").await;
    let uid = create_user(&pool, "u5", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "B", AccountType::Asset).await;
    let revenue = create_acc(&pool, uid, cid, "4000", "V", AccountType::Revenue).await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(100),
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        Journal::Banque,
        asset,
        revenue,
        dec!(200),
    )
    .await;

    // Période Q2 (avril-juin) : exclut les 2 écritures
    let period = ReportPeriod::resolve(
        &pool,
        cid,
        fy,
        Some(NaiveDate::from_ymd_opt(2026, 4, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap()),
    )
    .await
    .unwrap();
    let tb = kesh_report::generate_trial_balance(&pool, cid, &period)
        .await
        .unwrap();
    assert_eq!(tb.total_debit, Decimal::ZERO);
}

/// Test 6 : balance_sheet exclut compte 2979 (Pass 3 ECH3-01)
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn balance_sheet_excludes_2979_from_liabilities(pool: MySqlPool) {
    let cid = create_company(&pool, "co6").await;
    let uid = create_user(&pool, "u6", cid).await;
    let fy = create_fy(
        &pool,
        uid,
        cid,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;
    let asset = create_acc(&pool, uid, cid, "1000", "Banque", AccountType::Asset).await;
    // Compte 2979 sémantiquement equity, stocké Liability
    let acc_2979 = create_acc(
        &pool,
        uid,
        cid,
        "2979",
        "Résultat de l'exercice",
        AccountType::Liability,
    )
    .await;
    post_entry(
        &pool,
        uid,
        fy,
        cid,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        Journal::OD,
        asset,
        acc_2979,
        dec!(500),
    )
    .await;

    let period = ReportPeriod::resolve(&pool, cid, fy, None, None)
        .await
        .unwrap();
    let bs = kesh_report::generate_balance_sheet(&pool, cid, &period)
        .await
        .unwrap();
    // Le compte 2979 ne doit PAS apparaître dans liabilities
    let found_2979 = bs.liabilities.iter().any(|a| a.account_number == "2979");
    assert!(
        !found_2979,
        "Compte 2979 doit être exclu de total_liabilities (Pass 3 ECH3-01)"
    );
}

/// Test 7 : ReportPeriod::resolve cross-tenant returns FiscalYearNotFound
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn report_period_resolve_cross_tenant_returns_fy_not_found(pool: MySqlPool) {
    let cid_a = create_company(&pool, "co7a").await;
    let cid_b = create_company(&pool, "co7b").await;
    let uid_b = create_user(&pool, "u7b", cid_b).await;
    let fy_b = create_fy(
        &pool,
        uid_b,
        cid_b,
        "FY",
        NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    )
    .await;

    // Tenter de résoudre fy_b depuis company_a
    let err = ReportPeriod::resolve(&pool, cid_a, fy_b, None, None)
        .await
        .unwrap_err();
    assert!(
        matches!(err, kesh_report::ReportError::FiscalYearNotFound { .. }),
        "expected FiscalYearNotFound on cross-tenant fy lookup, got: {err:?}"
    );
}
