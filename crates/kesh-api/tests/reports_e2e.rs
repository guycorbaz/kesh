//! Tests E2E HTTP Story 9-1 — Rapports comptables.
//!
//! Couvre les ACs #1-#34 :
//! - GET /api/v1/reports/balance-sheet
//! - GET /api/v1/reports/income-statement
//! - GET /api/v1/reports/trial-balance
//! - GET /api/v1/reports/journals
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).
//! Pattern hérité de `bank_accounts_e2e.rs` (8-5a-zero).
#![allow(clippy::too_many_arguments)]

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry, NewJournalEntryLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Spawn helpers
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

// ============================================================
// Seed helpers (Pass 3 BH3-11 : Equity en Liability, jamais 'Equity' type)
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            address: "Rue Test 1".into(),
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

async fn create_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
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

async fn archive_acc(pool: &MySqlPool, account_id: i64, user_id: i64) {
    accounts::archive(pool, account_id, 1, user_id)
        .await
        .unwrap();
}

async fn post_entry(
    pool: &MySqlPool,
    user_id: i64,
    fiscal_year_id: i64,
    company_id: i64,
    date: NaiveDate,
    journal: Journal,
    description: &str,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        fiscal_year_id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal,
            description: description.into(),
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                },
            ],
        },
    )
    .await
    .unwrap();
}

// ============================================================
// Full seed context (Pass 3 BH3-11 + Pass 2 AA2-12 — seed structure obligatoire)
// ============================================================

struct Ctx {
    company_id: i64,
    user_id: i64,
    fy_id: i64,
    acc_1000_asset: i64,
    #[allow(dead_code)]
    acc_2000_liab: i64,
    acc_3000_capital: i64,
    acc_4000_revenue: i64,
    acc_5000_expense: i64,
    acc_1090_archived: i64,
    jwt: String,
}

/// Seed minimal Pass 3 BH3-11 :
/// - 5 comptes : 1000 (Asset), 2000 (Liability), 3000 (Liability — fonds propres),
///   4000 (Revenue), 5000 (Expense). PAS de type Equity (CHECK constraint).
/// - 1 compte archivé 1090 (Asset, active=false) avec écriture.
/// - 3 écritures équilibrées dans des journaux différents (Achats, Banque, OD).
async fn setup_full(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let company_id = create_company(pool, label).await;
    let user_id = create_user(pool, &format!("{label}_user"), role, company_id).await;
    let fy_id = create_fy(
        pool,
        user_id,
        company_id,
        "FY2026",
        NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        NaiveDate::from_ymd_opt(2027, 6, 30).unwrap(),
    )
    .await;

    let acc_1000_asset = create_acc(
        pool,
        user_id,
        company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let acc_2000_liab = create_acc(
        pool,
        user_id,
        company_id,
        "2000",
        "Fournisseurs",
        AccountType::Liability,
    )
    .await;
    let acc_3000_capital = create_acc(
        pool,
        user_id,
        company_id,
        "3000",
        "Capital social",
        AccountType::Liability,
    )
    .await;
    let acc_4000_revenue = create_acc(
        pool,
        user_id,
        company_id,
        "4000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let acc_5000_expense = create_acc(
        pool,
        user_id,
        company_id,
        "5000",
        "Achats",
        AccountType::Expense,
    )
    .await;
    let acc_1090_archived = create_acc(
        pool,
        user_id,
        company_id,
        "1090",
        "Stock obsolète",
        AccountType::Asset,
    )
    .await;

    // 3 écritures équilibrées multi-journaux (Pass 3 BH3-11 énumération explicite)
    // E1 Achats 2026-09-15 : débit 5000=1000, crédit 2000=1000
    post_entry(
        pool,
        user_id,
        fy_id,
        company_id,
        NaiveDate::from_ymd_opt(2026, 9, 15).unwrap(),
        Journal::Achats,
        "Facture fournisseur",
        acc_5000_expense,
        acc_2000_liab,
        dec!(1000.00),
    )
    .await;
    // E2 Banque 2026-12-31 : débit 1000=500, crédit 4000=500
    post_entry(
        pool,
        user_id,
        fy_id,
        company_id,
        NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        Journal::Banque,
        "Encaissement vente",
        acc_1000_asset,
        acc_4000_revenue,
        dec!(500.00),
    )
    .await;
    // E3 OD 2027-03-15 : débit 1090=250 (sur compte qu'on va archiver), crédit 2000=250
    post_entry(
        pool,
        user_id,
        fy_id,
        company_id,
        NaiveDate::from_ymd_opt(2027, 3, 15).unwrap(),
        Journal::OD,
        "Régularisation OD",
        acc_1090_archived,
        acc_2000_liab,
        dec!(250.00),
    )
    .await;

    // Archive compte 1090 APRÈS écriture
    archive_acc(pool, acc_1090_archived, user_id).await;

    let jwt = forge_jwt(user_id, role_str(role), company_id);

    Ctx {
        company_id,
        user_id,
        fy_id,
        acc_1000_asset,
        acc_2000_liab,
        acc_3000_capital,
        acc_4000_revenue,
        acc_5000_expense,
        acc_1090_archived,
        jwt,
    }
}

// ============================================================
// AC #1 — Balance sheet equation holds
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn balance_sheet_returns_balanced_assets_liabilities(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_a", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["equationHolds"], true);

    // Code review Pass 1 patch P14 — asserts numériques exacts au lieu de juste
    // déléguer la correction au flag `equationHolds`. Seed setup_full (Pass 3 BH3-11) :
    //   Assets : 1000 (E2 débit 500) + 1090 archivé (E3 débit 250) = 750
    //   Liabilities : 2000 (E1 crédit 1000 + E3 crédit 250) = 1250 (3000 sans écriture, exclu via HAVING != 0)
    //   equity_result = total_revenues 500 − total_expenses 1000 = -500
    //   Équation : 750 == 1250 + (-500) = 750 ✓
    let total_assets: Decimal = body["totalAssets"].as_str().unwrap().parse().unwrap();
    let total_liabilities: Decimal = body["totalLiabilities"].as_str().unwrap().parse().unwrap();
    let equity_result: Decimal = body["equityResult"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        total_assets,
        dec!(750),
        "totalAssets attendu = 1000:500 + 1090:250 = 750"
    );
    assert_eq!(
        total_liabilities,
        dec!(1250),
        "totalLiabilities attendu = 2000:1250 (3000 exclu HAVING)"
    );
    assert_eq!(
        equity_result,
        dec!(-500),
        "equity_result attendu = 500 − 1000 = -500"
    );
    assert_eq!(
        total_assets,
        total_liabilities + equity_result,
        "équation manuelle indépendante du flag serveur"
    );
}

// ============================================================
// AC #2 — Balance sheet ordering by account_number
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn balance_sheet_orders_accounts_by_number(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let assets = body["assets"].as_array().unwrap();
    // Code review Pass 1 patch P12 — `assert!` (au lieu du `if` qui skippait
    // silencieusement) : seed setup_full produit 1000:500 + 1090:250 = 2 assets,
    // donc l'assertion d'ordering EST exécutable et doit l'être.
    assert!(
        assets.len() >= 2,
        "seed setup_full doit produire au moins 2 assets (1000 + 1090 archivé avec écriture); \
         si la précondition change, le test ordering devient vacuous → corriger le seed"
    );
    let nums: Vec<&str> = assets
        .iter()
        .map(|a| a["accountNumber"].as_str().unwrap())
        .collect();
    let mut sorted = nums.clone();
    sorted.sort();
    assert_eq!(nums, sorted, "assets should be sorted by accountNumber ASC");
}

// ============================================================
// AC #3 — Income statement net_result + ordering
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn income_statement_computes_net_result_and_orders(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_c", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/income-statement?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let total_revenues: Decimal = body["totalRevenues"].as_str().unwrap().parse().unwrap();
    let total_expenses: Decimal = body["totalExpenses"].as_str().unwrap().parse().unwrap();
    let net_result: Decimal = body["netResult"].as_str().unwrap().parse().unwrap();
    // Code review Pass 1 patch P13 — asserts numériques exacts (au lieu de la
    // tautologie `net == rev - exp` qui passe pour 0=0-0). Seed setup_full :
    //   E2 Banque 2026-12-31 : crédit 4000 (Revenue) = 500
    //   E1 Achats 2026-09-15 : débit  5000 (Expense) = 1000
    //   → netResult = 500 - 1000 = -500
    assert_eq!(
        total_revenues,
        dec!(500),
        "totalRevenues attendu = 4000:500"
    );
    assert_eq!(
        total_expenses,
        dec!(1000),
        "totalExpenses attendu = 5000:1000"
    );
    assert_eq!(
        net_result,
        dec!(-500),
        "netResult attendu = 500 - 1000 = -500"
    );
    assert_eq!(net_result, total_revenues - total_expenses);
}

// ============================================================
// AC #4 — Trial balance debit == credit
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn trial_balance_total_debit_equals_total_credit(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_d", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["balanced"], true);
}

// ============================================================
// AC #5 — Archived account with entries appears
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn trial_balance_includes_archived_with_entries(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_e", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    let archived = rows
        .iter()
        .find(|r| r["accountId"].as_i64() == Some(ctx.acc_1090_archived));
    assert!(archived.is_some(), "compte 1090 archivé doit apparaître");
    assert_eq!(archived.unwrap()["active"], false);
}

// ============================================================
// AC #7 — Journals returns 5 sections in fixed order
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn journals_returns_five_sections_in_fixed_order(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_f", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let journals = body["journals"].as_array().unwrap();
    assert_eq!(journals.len(), 5);
    let names: Vec<&str> = journals
        .iter()
        .map(|j| j["journal"].as_str().unwrap())
        .collect();
    assert_eq!(names, vec!["Achats", "Ventes", "Banque", "Caisse", "OD"]);
    // Ventes et Caisse sont vides dans ce seed
    let ventes = journals.iter().find(|j| j["journal"] == "Ventes").unwrap();
    assert_eq!(ventes["entries"].as_array().unwrap().len(), 0);
}

// ============================================================
// AC #8 — Journal filter Achats returns one section
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn journals_filter_achats_returns_one_section(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_g", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals?fiscalYearId={}&journal=Achats",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let journals = body["journals"].as_array().unwrap();
    assert_eq!(journals.len(), 1);
    assert_eq!(journals[0]["journal"], "Achats");
}

// ============================================================
// AC #11 + #31 — Default period = full fiscal year, period in response
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn default_period_uses_fiscal_year_full_range_all_endpoints(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_h", Role::Comptable).await;
    let app = spawn_app(pool).await;

    for endpoint in &[
        "balance-sheet",
        "income-statement",
        "trial-balance",
        "journals",
    ] {
        let resp = app
            .client
            .get(app.url(&format!(
                "/api/v1/reports/{endpoint}?fiscalYearId={}",
                ctx.fy_id
            )))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "endpoint: {endpoint}");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["period"]["fiscalYearId"], ctx.fy_id,
            "endpoint: {endpoint}"
        );
        assert_eq!(
            body["period"]["startDate"], "2026-07-01",
            "endpoint: {endpoint}"
        );
        assert_eq!(
            body["period"]["endDate"], "2027-06-30",
            "endpoint: {endpoint}"
        );
    }
}

// ============================================================
// AC #12 — Partial period excludes outside entries
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn partial_period_excludes_outside_entries(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_i", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // Période 2026-08-01 → 2026-12-31 : exclut E3 (2027-03-15)
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals?fiscalYearId={}&periodStart=2026-08-01&periodEnd=2026-12-31",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let od = body["journals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|j| j["journal"] == "OD")
        .unwrap();
    assert_eq!(
        od["entries"].as_array().unwrap().len(),
        0,
        "E3 OD 2027-03-15 doit être exclu de la période [2026-08-01;2026-12-31]"
    );
}

// ============================================================
// AC #13 — Period out of FY returns 400 with 4 fields details
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn period_end_out_of_fy_returns_400_with_four_details_fields(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_j", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}&periodEnd=2028-01-15",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "REPORT_PERIOD_OUT_OF_FISCAL_YEAR");
    let details = &body["error"]["details"];
    assert!(details["fyStart"].is_string());
    assert!(details["fyEnd"].is_string());
    assert!(details["requestedStart"].is_string());
    assert!(details["requestedEnd"].is_string());
    assert_eq!(details["requestedEnd"], "2028-01-15");
}

// ============================================================
// AC #15 — Multi-fiscal-years isolation
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn multi_fiscal_years_isolation(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_k", Role::Comptable).await;

    // Créer fy2 calendrier 2025
    let fy2_id = create_fy(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "FY2025",
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
    )
    .await;

    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance?fiscalYearId={fy2_id}"
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    // fy2 sans écritures → balanced=true et toutes les rows ont balance=0
    assert_eq!(body["balanced"], true);
    let rows = body["rows"].as_array().unwrap();
    for r in rows {
        let bal: Decimal = r["balance"].as_str().unwrap().parse().unwrap();
        assert_eq!(
            bal,
            Decimal::ZERO,
            "fy2 row {} doit avoir balance=0",
            r["accountNumber"]
        );
    }
}

// ============================================================
// AC #16 — Cross-tenant fiscal year returns 404
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cross_tenant_fiscal_year_returns_404(pool: MySqlPool) {
    let ctx_a = setup_full(&pool, "co_l_a", Role::Comptable).await;
    let ctx_b = setup_full(&pool, "co_l_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // User A tente d'accéder à fy de B
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx_b.fy_id
        )))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_NOT_FOUND");
}

// ============================================================
// AC #19 — fiscalYearId missing returns 400
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fiscal_year_id_missing_returns_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_m", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/reports/balance-sheet"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    // Pass 3 BH3-12 : Axum Query rejection default = text/plain 400
    assert_eq!(resp.status(), 400);
}

// ============================================================
// AC #20-bis — fiscalYearId <= 0 returns 400 JSON VALIDATION_ERROR
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fiscal_year_id_zero_or_negative_returns_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_n", Role::Comptable).await;
    let app = spawn_app(pool).await;

    for value in &["0", "-1"] {
        let resp = app
            .client
            .get(app.url(&format!(
                "/api/v1/reports/balance-sheet?fiscalYearId={value}"
            )))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "value: {value}");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR", "value: {value}");
    }
}

// ============================================================
// AC #23 + #32 — Consultation role can read reports
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_role_can_read_reports(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_o", Role::Consultation).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ============================================================
// AC #24 — Unauthenticated returns 401
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unauthenticated_returns_401(pool: MySqlPool) {
    let _ctx = setup_full(&pool, "co_p", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/reports/balance-sheet?fiscalYearId=1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ============================================================
// AC #25 — Audit emitted on success with sentinel entity_id
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn report_generated_audit_emitted_on_success(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_q", Role::Comptable).await;
    let pool_clone = pool.clone();
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Vérifier audit row + details_json content (Pass 1 code review patch P15 —
    // AC #25 exige `details_json = { report_type, fiscal_year_id, period_start,
    // period_end, journal_filter }`. Auparavant seuls user/action/entity_* étaient
    // assertés → faux-vert si une régression renommait les clés JSON.)
    //
    // **Migration Story 9-5-2 (Epic 9.5)** : clés JSON migrées camelCase →
    // snake_case (cohérent convention §audit_log JSON keys documentée au-dessus
    // de `emit_report_audit` dans `crates/kesh-api/src/routes/reports.rs`).
    //
    // Colonne `details_json` est de type `JSON` côté MariaDB (stockée comme blob
    // binaire) → fetch en `Vec<u8>` puis parse UTF-8 + JSON.
    let row: (i64, String, String, i64, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT user_id, action, entity_type, entity_id, details_json FROM audit_log \
         WHERE user_id = ? AND action = 'report.generated' \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(ctx.user_id)
    .fetch_one(&pool_clone)
    .await
    .unwrap();
    assert_eq!(row.0, ctx.user_id);
    assert_eq!(row.1, "report.generated");
    assert_eq!(row.2, "report");
    assert_eq!(row.3, 0, "entity_id doit être AUDIT_ENTITY_ID_NONE (0)");

    let details_bytes = row.4.expect("details_json doit être non-NULL");
    let details: Value =
        serde_json::from_slice(&details_bytes).expect("details_json doit être JSON valide");
    assert_eq!(
        details["report_type"], "balance-sheet",
        "AC #25 report_type"
    );
    assert_eq!(
        details["fiscal_year_id"], ctx.fy_id,
        "AC #25 fiscal_year_id"
    );
    assert!(
        details.get("period_start").is_some(),
        "AC #25 period_start présent (peut être null)"
    );
    assert!(
        details.get("period_end").is_some(),
        "AC #25 period_end présent (peut être null)"
    );
    assert!(
        details.get("journal_filter").is_some(),
        "AC #25 journal_filter présent (peut être null)"
    );
}

// ============================================================
// AC #26 — No audit on 400/404
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn report_generated_audit_not_emitted_on_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_r", Role::Comptable).await;
    let pool_clone = pool.clone();
    let app = spawn_app(pool).await;

    // Période hors fy → 400
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}&periodEnd=2099-01-01",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE user_id = ? AND action = 'report.generated'",
    )
    .bind(ctx.user_id)
    .fetch_one(&pool_clone)
    .await
    .unwrap();
    assert_eq!(count, 0, "Aucun audit ne doit être émis sur 400");
}

// ============================================================
// AC #18 — Audit scoped via JOIN users
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn audit_log_scoped_to_company_via_user_join(pool: MySqlPool) {
    let ctx_a = setup_full(&pool, "co_s_a", Role::Comptable).await;
    let ctx_b = setup_full(&pool, "co_s_b", Role::Comptable).await;
    let pool_clone = pool.clone();
    let app = spawn_app(pool).await;

    // User A consulte rapport
    app.client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            ctx_a.fy_id
        )))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();

    // Query JOIN users sur company A
    let count_a: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log al \
         JOIN users u ON al.user_id = u.id \
         WHERE al.action = 'report.generated' AND u.company_id = ?",
    )
    .bind(ctx_a.company_id)
    .fetch_one(&pool_clone)
    .await
    .unwrap();
    assert!(count_a >= 1);

    // Query JOIN users sur company B = 0
    let count_b: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log al \
         JOIN users u ON al.user_id = u.id \
         WHERE al.action = 'report.generated' AND u.company_id = ?",
    )
    .bind(ctx_b.company_id)
    .fetch_one(&pool_clone)
    .await
    .unwrap();
    assert_eq!(count_b, 0, "Audit ne doit pas être visible cross-tenant");
}

// ============================================================
// AC #14 — Period inversed returns 400 VALIDATION_ERROR
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn period_inversed_returns_400_validation_code(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_t", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // Hack : pour avoir start > end sans tomber dans le check out-of-fy d'abord,
    // il faut que les 2 dates soient dans le fy mais inversées.
    // FY = 2026-07-01 → 2027-06-30. Test : start=2027-03-01, end=2026-09-01.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}&periodStart=2027-03-01&periodEnd=2026-09-01",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// ============================================================
// Cross-tenant aggregation filter (AC #17 implicite)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cross_tenant_aggregation_filtered_by_company(pool: MySqlPool) {
    let ctx_a = setup_full(&pool, "co_u_a", Role::Comptable).await;
    let _ctx_b = setup_full(&pool, "co_u_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // User A obtient trial_balance pour son fy
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance?fiscalYearId={}",
            ctx_a.fy_id
        )))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    // Aucun row ne doit avoir un accountId qui n'existe pas chez ctx_a
    let expected_account_ids = [
        ctx_a.acc_1000_asset,
        ctx_a.acc_2000_liab,
        ctx_a.acc_3000_capital,
        ctx_a.acc_4000_revenue,
        ctx_a.acc_5000_expense,
        ctx_a.acc_1090_archived,
    ];
    for r in rows {
        let aid = r["accountId"].as_i64().unwrap();
        assert!(
            expected_account_ids.contains(&aid),
            "Found cross-tenant account id: {aid}"
        );
    }
}

// ============================================================
// AC #6 — Archived account WITHOUT entries excluded
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn trial_balance_excludes_archived_without_entries(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_w", Role::Comptable).await;
    // Créer un compte archivé sans écriture
    let archived_no_entries = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1099",
        "Compte archivé sans entrée",
        AccountType::Asset,
    )
    .await;
    archive_acc(&pool, archived_no_entries, ctx.user_id).await;

    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    let found = rows
        .iter()
        .any(|r| r["accountId"].as_i64() == Some(archived_no_entries));
    assert!(!found, "compte 1099 archivé sans écriture doit être exclu");
}

// ============================================================
// AC #9 — Journals orders entries chronologically
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn journals_orders_entries_chronologically(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_x", Role::Comptable).await;
    // Ajouter 2 écritures Achats avec dates inverses
    post_entry(
        &pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 11, 20).unwrap(),
        Journal::Achats,
        "Achat tardif",
        ctx.acc_5000_expense,
        ctx.acc_2000_liab,
        dec!(100.00),
    )
    .await;
    post_entry(
        &pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 8, 1).unwrap(),
        Journal::Achats,
        "Achat précoce",
        ctx.acc_5000_expense,
        ctx.acc_2000_liab,
        dec!(200.00),
    )
    .await;

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals?fiscalYearId={}&journal=Achats",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let achats = &body["journals"][0];
    let entries = achats["entries"].as_array().unwrap();
    let dates: Vec<&str> = entries
        .iter()
        .map(|e| e["entryDate"].as_str().unwrap())
        .collect();
    let mut sorted = dates.clone();
    sorted.sort();
    assert_eq!(
        dates, sorted,
        "Achats entries doivent être triés ASC par date"
    );
}

// ============================================================
// AC #20 — fiscalYearId malformé (parse error) returns 400
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fiscal_year_id_malformed_returns_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_y", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/reports/balance-sheet?fiscalYearId=abc"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    // Pass 3 BH3-12 : Axum Query rejection = text/plain 400
    assert_eq!(resp.status(), 400);
}

// ============================================================
// AC #21 — Date malformée returns 400
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn date_malformed_returns_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_z", Role::Comptable).await;
    let app = spawn_app(pool).await;

    for date in &["2026/01/15", "2026-02-30"] {
        let resp = app
            .client
            .get(app.url(&format!(
                "/api/v1/reports/balance-sheet?fiscalYearId={}&periodStart={date}",
                ctx.fy_id
            )))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "date: {date}");
    }
}

// ============================================================
// AC #22 — Journal enum invalide returns 400
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn journal_enum_invalid_returns_400(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_aa", Role::Comptable).await;
    let app = spawn_app(pool).await;

    for journal in &["Salaires", "achats"] {
        let resp = app
            .client
            .get(app.url(&format!(
                "/api/v1/reports/journals?fiscalYearId={}&journal={journal}",
                ctx.fy_id
            )))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "journal: {journal}");
    }
}

// ============================================================
// AC #28 / Pass 1 ECH-01 — Empty period returns zero totals + equation holds
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn balance_sheet_empty_period_returns_zero_totals_equation_holds(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_bb", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // Période 1 jour sans écritures (entre 2 écritures réelles)
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}&periodStart=2026-10-01&periodEnd=2026-10-01",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let assets = body["assets"].as_array().unwrap();
    let liabilities = body["liabilities"].as_array().unwrap();
    assert_eq!(assets.len(), 0);
    assert_eq!(liabilities.len(), 0);
    assert_eq!(body["equationHolds"], true);
    let total_assets: Decimal = body["totalAssets"].as_str().unwrap().parse().unwrap();
    let equity_result: Decimal = body["equityResult"].as_str().unwrap().parse().unwrap();
    assert_eq!(total_assets, Decimal::ZERO);
    assert_eq!(equity_result, Decimal::ZERO);
}

// ============================================================
// AC #10 — Journal line ordering preserved
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn journals_preserves_line_order(pool: MySqlPool) {
    let ctx = setup_full(&pool, "co_v", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals?fiscalYearId={}&journal=Achats",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let achats = &body["journals"][0];
    let entries = achats["entries"].as_array().unwrap();
    if !entries.is_empty() {
        let lines = entries[0]["lines"].as_array().unwrap();
        let orders: Vec<i64> = lines
            .iter()
            .map(|l| l["lineOrder"].as_i64().unwrap())
            .collect();
        let mut sorted = orders.clone();
        sorted.sort();
        assert_eq!(orders, sorted);
    }
}
