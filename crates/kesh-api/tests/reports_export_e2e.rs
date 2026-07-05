//! Tests E2E HTTP Story 9-2a — Export PDF & CSV des 4 rapports comptables.
//!
//! Couvre AC #1, #10, #22-#29, #32(a-h) :
//! - 4 endpoints PDF × 200 binary content (a)
//! - 4 endpoints CSV × 200 text/csv content (b)
//! - format invalid 400 VALIDATION_ERROR (c) — incl. PDF uppercase
//! - multi-tenant 404 (d)
//! - FY out of bounds 400 (e) — avant ou après FY 2020-2030
//! - rapport vide PDF + CSV success path (f) — seed sans écritures
//! - auth 401 (g)
//! - RBAC Consultation 200 PDF + 200 CSV (h)
//!
//! Total : 4 + 4 + 1 + 1 + 1 + 2 + 1 + 2 = **16 tests**.
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).
//! Pattern hérité de `reports_e2e.rs` Story 9-1.
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
use kesh_db::entities::address::StructuredAddress;
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
// Spawn helpers (réutilisés de reports_e2e.rs)
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
// Seed helpers (équivalent setup_full de reports_e2e.rs, simplifié)
// ============================================================

struct Ctx {
    #[allow(dead_code)]
    company_id: i64,
    #[allow(dead_code)]
    user_id: i64,
    fy_id: i64,
    jwt: String,
}

async fn seed_company(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    seed_company_with_name(pool, label, role, &format!("CI {label}")).await
}

async fn seed_company_with_name(pool: &MySqlPool, label: &str, role: Role, name: &str) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: name.into(),
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
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
    .id;

    let user_id = users::create(
        pool,
        NewUser {
            username: format!("{label}_user"),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;

    // FY 2026 (dans la fenêtre 2020-2030 réf. test_fixtures)
    let fy_id = fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: "FY2026".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id;

    let jwt = forge_jwt(user_id, role_str(role), company_id);
    Ctx {
        company_id,
        user_id,
        fy_id,
        jwt,
    }
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

#[allow(clippy::too_many_arguments)]
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

/// Seed identique à reports_e2e.rs setup_full mais centré 9-2a (3 écritures équilibrées).
async fn seed_with_entries(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let ctx = seed_company(pool, label, role).await;
    let acc_asset = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let acc_liab = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "2000",
        "Fournisseurs",
        AccountType::Liability,
    )
    .await;
    let acc_rev = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let acc_exp = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "4000",
        "Achats",
        AccountType::Expense,
    )
    .await;

    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
        Journal::Achats,
        "Achat fournitures",
        acc_exp,
        acc_liab,
        dec!(500.00),
    )
    .await;
    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        Journal::Banque,
        "Encaissement vente",
        acc_asset,
        acc_rev,
        dec!(750.00),
    )
    .await;
    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        Journal::Ventes,
        "Vente services",
        acc_asset,
        acc_rev,
        dec!(1200.00),
    )
    .await;

    ctx
}

// ============================================================
// Assertions helpers
// ============================================================

fn assert_pdf_response(content_type: &str, body: &[u8]) {
    assert_eq!(
        content_type, "application/pdf",
        "Content-Type must be application/pdf"
    );
    assert!(
        body.starts_with(b"%PDF-1."),
        "PDF body must start with %PDF-1. signature, got: {:?}",
        &body.get(..16).unwrap_or(&body[..body.len().min(16)])
    );
}

fn assert_csv_response(content_type: &str, body: &[u8]) {
    assert_eq!(
        content_type, "text/csv; charset=utf-8",
        "Content-Type must be text/csv; charset=utf-8"
    );
    // Pass 1 code-review M2 (BH2-M5) : garde-fou contre un body tronqué — sinon
    // `&body[..3]` panic avec un message peu lisible. Si jamais le backend
    // retournait un body < 3 bytes, on échoue avec un message explicite.
    assert!(
        body.len() >= 3,
        "CSV body must be at least 3 bytes (UTF-8 BOM), got {} bytes",
        body.len()
    );
    assert_eq!(
        &body[..3],
        b"\xef\xbb\xbf",
        "CSV body must start with UTF-8 BOM"
    );
}

// ============================================================
// AC #32(a) — 4 endpoints PDF × 200 binary content
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_balance_sheet_pdf_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_bs_pdf", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let cd = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_pdf_response(&ct, &body);
    assert!(
        cd.contains("attachment"),
        "Content-Disposition must be attachment, got: {cd}"
    );
    assert!(cd.contains(".pdf"), "filename must include .pdf");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_income_statement_pdf_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_is_pdf", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/income-statement/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_pdf_response(&ct, &body);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_trial_balance_pdf_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_tb_pdf", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_pdf_response(&ct, &body);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_journal_report_pdf_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_jr_pdf", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_pdf_response(&ct, &body);
}

// ============================================================
// AC #32(b) — 4 endpoints CSV × 200 text/csv content
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_balance_sheet_csv_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_bs_csv", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let cd = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_csv_response(&ct, &body);
    assert!(cd.contains(".csv"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_income_statement_csv_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_is_csv", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/income-statement/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_csv_response(&ct, &body);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_trial_balance_csv_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_tb_csv", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_csv_response(&ct, &body);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_journal_report_csv_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_jr_csv", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_csv_response(&ct, &body);
}

// ============================================================
// AC #32(c) — format invalid 400 (incl. uppercase PDF Pass 2 ECH2-H1)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_format_invalid_returns_400(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_invalid", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // Multiple invalid cases : missing, empty, uppercase, mixed case, unknown.
    let invalid_formats = [
        "", // ?format= empty
        "PDF", "Csv", "xml", "Pdf",
    ];

    for fmt in &invalid_formats {
        let url = if fmt.is_empty() {
            format!(
                "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=",
                ctx.fy_id
            )
        } else {
            format!(
                "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format={fmt}",
                ctx.fy_id
            )
        };
        let resp = app
            .client
            .get(app.url(&url))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 400, "format={fmt:?} should return 400");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(
            body["error"]["code"], "VALIDATION_ERROR",
            "format={fmt:?} should return VALIDATION_ERROR"
        );
    }

    // Aussi tester format manquant entièrement
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "format absent should return 400");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// ============================================================
// AC #32(d) — multi-tenant 404
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_cross_tenant_returns_404(pool: MySqlPool) {
    let ctx_a = seed_with_entries(&pool, "co_x_a", Role::Comptable).await;
    let ctx_b = seed_with_entries(&pool, "co_x_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // User A tente d'accéder au FY de B
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=pdf",
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

// Pass 1 code-review M1 (BH2-M3 + ECH2-M3) : `export_cross_tenant_returns_404`
// ne couvrait que balance-sheet. Les 3 autres endpoints doivent prouver la
// même protection multi-tenant pour éviter une régression silencieuse si une
// route omet `current_user.company_id` dans `ReportPeriod::resolve`.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_income_statement_cross_tenant_returns_404(pool: MySqlPool) {
    let ctx_a = seed_with_entries(&pool, "co_x_is_a", Role::Comptable).await;
    let ctx_b = seed_with_entries(&pool, "co_x_is_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/income-statement/export?fiscalYearId={}&format=pdf",
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_trial_balance_cross_tenant_returns_404(pool: MySqlPool) {
    let ctx_a = seed_with_entries(&pool, "co_x_tb_a", Role::Comptable).await;
    let ctx_b = seed_with_entries(&pool, "co_x_tb_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance/export?fiscalYearId={}&format=csv",
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_journals_cross_tenant_returns_404(pool: MySqlPool) {
    let ctx_a = seed_with_entries(&pool, "co_x_jr_a", Role::Comptable).await;
    let ctx_b = seed_with_entries(&pool, "co_x_jr_b", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals/export?fiscalYearId={}&format=pdf",
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
// AC #32(e) — FY out of bounds 400 (avant OU après FY 2020-2030)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_period_out_of_fy_returns_400(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_oob", Role::Comptable).await;
    let app = spawn_app(pool).await;

    // Période APRÈS la fin du FY (FY 2026-01-01 → 2026-12-31)
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&periodStart=2027-01-01&periodEnd=2027-12-31&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Période AVANT le début du FY
    let resp2 = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&periodStart=2025-01-01&periodEnd=2025-06-30&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 400);
}

// ============================================================
// AC #32(f) — Empty report PDF + CSV success path
// ============================================================
// Stratégie Pass 2 AA2-H2 : seed `with-company` SANS aucune insertion
// journal_entries + période DANS FY 2026.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_empty_balance_sheet_pdf_success(pool: MySqlPool) {
    // Seed company + fy mais AUCUNE écriture
    let ctx = seed_company(&pool, "co_empty_pdf", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "empty report must still return 200 PDF");
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_pdf_response(&ct, &body);
    // PDF empty doit quand même être un PDF valide (avec empty_message)
    assert!(
        body.len() > 200,
        "even empty PDF should have minimal content"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_empty_journals_csv_success(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_empty_csv", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/journals/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    let body = resp.bytes().await.unwrap();
    assert_csv_response(&ct, &body);
    // Empty CSV : 1 ligne header + 0 data row
    let text = String::from_utf8(body[3..].to_vec()).unwrap();
    let lines: Vec<&str> = text.split("\r\n").filter(|l| !l.is_empty()).collect();
    assert_eq!(lines.len(), 1, "empty CSV must have only header line");
}

// ============================================================
// AC #32(g) — Auth 401 (request GET, pas de Bearer)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_without_auth_returns_401(pool: MySqlPool) {
    let _ctx = seed_with_entries(&pool, "co_unauth", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/reports/balance-sheet/export?fiscalYearId=1&format=pdf"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        401,
        "unauthenticated request must return 401 — if 200 then routes leaked outside authenticated_routes (BH-H1 IDOR risk)"
    );
}

// ============================================================
// AC #32(h) — RBAC Consultation 200 PDF + 200 CSV
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_pdf_consultation_role_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_cons_pdf", Role::Consultation).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "Consultation role must be allowed to export (read-only)"
    );
    let body = resp.bytes().await.unwrap();
    assert!(body.starts_with(b"%PDF-1."));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_csv_consultation_role_returns_200(pool: MySqlPool) {
    let ctx = seed_with_entries(&pool, "co_cons_csv", Role::Consultation).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/trial-balance/export?fiscalYearId={}&format=csv",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    assert_eq!(&body[..3], b"\xef\xbb\xbf");
}

// ============================================================
// T9.4 — Content-Disposition avec company name non-ASCII (Pass 1 ECH-M2)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_content_disposition_handles_non_ascii_company_name(pool: MySqlPool) {
    let ctx = seed_company_with_name(&pool, "co_uml", Role::Comptable, "Müller AG").await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet/export?fiscalYearId={}&format=pdf",
            ctx.fy_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let cd = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .expect("Content-Disposition header must be set (no HeaderValue panic on non-ASCII)")
        .to_str()
        .unwrap();
    // ASCII fallback : `ü` remplacé par `_`
    assert!(cd.contains("muller-ag") || cd.contains("m_ller-ag"));
    // RFC 5987 percent-encoded form (avec tag langue BCP-47 — Pass 1 code-review M14).
    // Le test accepte les deux variantes : `UTF-8''` (sans tag) ou `UTF-8'<lang>'`.
    assert!(
        cd.contains("filename*=UTF-8'") && cd.contains('\''),
        "Content-Disposition must include filename*=UTF-8'…' (RFC 5987), got: {cd}"
    );
}
