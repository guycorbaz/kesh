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
use kesh_db::entities::audit_log::Acteur;
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

#[path = "support/jeu_pme.rs"]
mod commun_pme;
use commun_pme::JeuPme;

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
            first_name: None,
            last_name: None,
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
            role: None,
            postable: true,
            currency: None,
            own_fx_accounts: None,
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
        Acteur::utilisateur(user_id),
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
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                    foreign: None,
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
        body.get(..16).unwrap_or(&body[..body.len().min(16)])
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

// ============================================================
// Story 48-11 (#1245) — les montants d'un rapport s'arrondissent au centime
// ============================================================

/// Une écriture à `1234.0050` (fraction de centime, saisie admise à quatre
/// décimales) : le CSV de la balance porte `1234.01` (demi-centime loin de
/// zéro, comme l'écran), non `1234.00` (troncature de `{:.2}`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_trial_balance_csv_arrondit_la_fraction_de_centime(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_arr_csv", Role::Comptable).await;
    let banque = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let ventes = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    post_entry(
        &pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        Journal::Banque,
        "Fraction de centime",
        banque,
        ventes,
        dec!(1234.0050),
    )
    .await;
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
    let body = String::from_utf8(resp.bytes().await.unwrap().to_vec()).unwrap();
    assert!(
        body.contains("1234.01"),
        "le CSV doit porter 1234.01, lu : {body}"
    );
    assert!(
        !body.contains("1234.00"),
        "le CSV ne doit plus tronquer 1234.0050 en 1234.00, lu : {body}"
    );
}

/// Même écriture, en PDF : le texte se lit dans le corps non compressé
/// (`printpdf` ne compresse pas sous `debug_assertions`) ; la police
/// intégrée écrit un octet WinAnsi par caractère, en `Tj` hexadécimal.
/// `1'234.01` = `31273233342e3031`, `1'234.00` = `31273233342e3030`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_trial_balance_pdf_arrondit_la_fraction_de_centime(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_arr_pdf", Role::Comptable).await;
    let banque = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let ventes = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    post_entry(
        &pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        Journal::Banque,
        "Fraction de centime",
        banque,
        ventes,
        dec!(1234.0050),
    )
    .await;
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
    let body = resp.bytes().await.unwrap();
    // Le `Tj` est écrit en hexadécimal ASCII dans le flux : on cherche la
    // chaîne hexadécimale **écrite en texte** (minuscules), non les octets.
    let text = String::from_utf8_lossy(&body).to_lowercase();
    assert!(
        text.contains("31273233342e3031"),
        "le PDF doit porter 1'234.01 (hex 31273233342e3031)"
    );
    assert!(
        !text.contains("31273233342e3030"),
        "le PDF ne doit pas porter 1'234.00 (hex 31273233342e3030)"
    );
}

// ============================================================
// Story 46-3 — exports du bilan et du compte de résultat par groupes
// ============================================================

/// Le jeu PME de la Story 46-1 (`support/jeu_pme.rs`) : la société et son
/// exercice 2026 par [`seed_company_with_name`], l'exercice 2025 ici, puis le
/// plan, le compte en dollars et les écritures par le module partagé.
async fn jeu_pme(pool: &MySqlPool) -> JeuPme {
    let ctx = seed_company_with_name(pool, "pme_46_3", Role::Comptable, "Société PME 46-3").await;
    let fy_2025 = fiscal_years::create(
        pool,
        ctx.user_id,
        NewFiscalYear {
            company_id: ctx.company_id,
            name: "FY2025".into(),
            start_date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id;
    commun_pme::peupler(
        pool,
        ctx.company_id,
        ctx.user_id,
        fy_2025,
        ctx.fy_id,
        ctx.jwt,
    )
    .await
}

/// GET d'un export du jeu PME (`route` : `balance-sheet` ou
/// `income-statement`), avec la requête `suite` après `format`.
async fn export_pme(
    app: &TestApp,
    jeu: &JeuPme,
    route: &str,
    fy: i64,
    format: &str,
    suite: &str,
) -> reqwest::Response {
    app.client
        .get(app.url(&format!(
            "/api/v1/reports/{route}/export?fiscalYearId={fy}&format={format}{suite}"
        )))
        .bearer_auth(&jeu.jwt)
        .send()
        .await
        .unwrap()
}

/// Chemin d'une fixture d'export des rapports par groupes (Stories 46-3, 46-6b).
fn fixture_groupes(nom: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/rapports-groupes")
        .join(nom)
}

/// Compare les exports sans paramètre de groupes de `rapports` (`(route,
/// nom)`), pour les deux exercices du jeu PME et les deux formats, à leurs
/// fixtures `export-<nom>-<année>.{csv,pdf.txt}` : CSV **à l'octet**, PDF aux
/// **mêmes lignes de texte** (extracteur partagé `kesh_report::texte_pdf`).
/// `ecrire` réécrit les fixtures — à ne faire qu'au commit de base de la story
/// qui les pose.
async fn comparer_aux_fixtures(
    app: &TestApp,
    jeu: &JeuPme,
    rapports: &[(&str, &str)],
    ecrire: bool,
) {
    for (route, nom) in rapports {
        for (fy, annee) in [(jeu.fy_2025, 2025), (jeu.fy_2026, 2026)] {
            for format in ["csv", "pdf"] {
                let resp = export_pme(app, jeu, route, fy, format, "").await;
                assert_eq!(resp.status(), 200, "{route} {annee} {format}");
                let corps = resp.bytes().await.unwrap().to_vec();
                let (chemin, obtenu) = if format == "csv" {
                    (fixture_groupes(&format!("export-{nom}-{annee}.csv")), corps)
                } else {
                    let lignes = kesh_report::texte_pdf::lignes(&corps);
                    assert!(
                        lignes.len() > 5,
                        "{route} {annee} : PDF illisible ({lignes:?})"
                    );
                    (
                        fixture_groupes(&format!("export-{nom}-{annee}.pdf.txt")),
                        (lignes.join("\n") + "\n").into_bytes(),
                    )
                };
                if ecrire {
                    std::fs::create_dir_all(chemin.parent().unwrap()).unwrap();
                    std::fs::write(&chemin, &obtenu).unwrap();
                }
                let attendu = std::fs::read(&chemin)
                    .unwrap_or_else(|e| panic!("fixture {} illisible : {e}", chemin.display()));
                assert_eq!(
                    String::from_utf8_lossy(&obtenu),
                    String::from_utf8_lossy(&attendu),
                    "{route} {annee} {format} sans paramètre : l'export a changé (NFR-46-1)"
                );
                assert_eq!(obtenu, attendu, "{route} {annee} {format} : octets");
            }
        }
    }
}

/// AC-2 (NFR-46-1) — sans paramètre de groupes, l'export CSV du bilan et du
/// compte de résultat est **identique à l'octet**, et l'export PDF a les
/// **mêmes lignes de texte**, qu'au commit de base de la Story 46-3, fixtures
/// écrites dans `tests/fixtures/rapports-groupes/export-*` avant toute
/// modification du rendu. `KESH_ECRIRE_FIXTURES_46_3=1` les réécrit — à ne
/// faire qu'au commit de base.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn exports_sans_parametre_identiques_a_la_fixture_de_base(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    let ecrire = std::env::var("KESH_ECRIRE_FIXTURES_46_3").as_deref() == Ok("1");
    comparer_aux_fixtures(
        &app,
        &jeu,
        &[("balance-sheet", "bilan"), ("income-statement", "resultat")],
        ecrire,
    )
    .await;
}

/// Story 46-6b, AC-3 (NFR-46-1) — sans paramètre de groupes, l'export de la
/// **balance** est inchangé : CSV à l'octet, PDF aux mêmes lignes de texte,
/// contre les fixtures `export-balance-*` écrites au commit de base de la
/// 46-6b, avant toute modification du rendu. `KESH_ECRIRE_FIXTURES_46_6B=1`
/// les réécrit — à ne faire qu'au commit de base.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_balance_sans_parametre_identique_a_la_fixture_de_base(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    let ecrire = std::env::var("KESH_ECRIRE_FIXTURES_46_6B").as_deref() == Ok("1");
    comparer_aux_fixtures(&app, &jeu, &[("trial-balance", "balance")], ecrire).await;
}

/// Un montant de `groupedRows` (chaîne décimale exacte) tel que le PDF groupé
/// l'écrit : arrondi au centime, l'équidistant loin de zéro, apostrophes des
/// milliers — écrit ici indépendamment du rendu, pour que le test le contrôle.
fn montant_suisse(brut: &str) -> String {
    let d: Decimal = brut
        .parse()
        .unwrap_or_else(|_| panic!("montant « {brut} »"));
    let mut r = d.round_dp_with_strategy(2, rust_decimal::RoundingStrategy::MidpointAwayFromZero);
    if r.is_zero() {
        r = Decimal::ZERO;
    }
    r.rescale(2);
    let s = r.to_string();
    let (signe, corps) = s.strip_prefix('-').map_or(("", s.as_str()), |c| ("-", c));
    let (entier, decimales) = corps.split_once('.').unwrap();
    let mut groupes: Vec<String> = Vec::new();
    let chiffres: Vec<char> = entier.chars().collect();
    for bloc in chiffres.rchunks(3) {
        groupes.insert(0, bloc.iter().collect());
    }
    format!("{signe}{}.{decimales}", groupes.join("'"))
}

/// La ligne de texte qu'une ligne de `groupedRows` doit donner au PDF, selon
/// la règle de rendu de la 46-1 (AC-3), et son libellé seul.
fn ligne_attendue(l: &Value, pied: bool) -> (String, String) {
    let kind = l["kind"].as_str().unwrap();
    let nom = l["accountName"].as_str().unwrap();
    let numero = l["accountNumber"].as_str().unwrap();
    let (numero, libelle) = match kind {
        "group" | "account" => (Some(numero), nom.to_string()),
        "own" => (Some(numero), format!("{nom} (solde propre)")),
        "subtotal" => (None, format!("Total {nom}")),
        autre => panic!("kind inconnu : {autre}"),
    };
    let montant = !(kind == "group" && l["open"] == true && pied);
    let mut morceaux: Vec<String> = numero.map(String::from).into_iter().collect();
    morceaux.push(libelle.clone());
    if montant {
        if let (Some(cur), Some(fx)) = (l["currency"].as_str(), l["balanceCurrency"].as_str()) {
            morceaux.push(format!("{cur} {}", montant_suisse(fx)));
        }
        morceaux.push(montant_suisse(l["balance"].as_str().unwrap()));
    }
    (morceaux.join(" "), libelle)
}

/// Les lignes du PDF entre la ligne `debut` (exclue) et la première ligne qui
/// commence par `fin` (exclue).
fn lignes_entre(lignes: &[String], debut: &str, fin: &str) -> Vec<String> {
    let a = lignes
        .iter()
        .position(|l| l == debut)
        .unwrap_or_else(|| panic!("« {debut} » absent : {lignes:?}"))
        + 1;
    let b = a + lignes[a..]
        .iter()
        .position(|l| l.starts_with(fin))
        .unwrap_or_else(|| panic!("« {fin} » absent : {lignes:?}"));
    lignes[a..b].to_vec()
}

/// AC-4 (FR-46-3, FR-46-9, remarque 4 du Project Lead) — le PDF est fidèle à
/// l'écran : avec **les mêmes paramètres** (niveau 2, un groupe ouvert à la
/// main, un fermé, sous-totaux en pied), la suite des lignes de chaque section
/// du PDF est exactement celle de `groupedRows.<section>` du JSON (46-1), selon
/// la règle de rendu ; Capitaux propres : `groupedRows.equity` puis les lignes
/// calculées. Le CSV groupé a ses deux colonnes de plus et une ligne par ligne
/// groupée. Aucun libellé du jeu n'atteint la limite de troncature, ce que le
/// test affirme d'abord : aucun « … » n'entre dans la comparaison.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_pdf_groupe_fidele_au_json(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    let cas = [
        (
            "balance-sheet",
            format!(
                "&detailLevel=2&subtotalPosition=footer&open={}&closed={}",
                jeu.id("10"),
                jeu.id("20")
            ),
            vec![
                ("assets", "Actifs", "Total actifs"),
                ("liabilities", "Passifs", "Total passifs"),
                ("equity", "Capitaux propres", "Total capitaux propres"),
            ],
        ),
        (
            "income-statement",
            format!(
                "&detailLevel=2&subtotalPosition=footer&open={}&closed={}",
                jeu.id("30"),
                jeu.id("5")
            ),
            vec![
                ("revenues", "Produits", "Total produits"),
                ("expenses", "Charges", "Total charges"),
            ],
        ),
    ];
    for (route, suite, sections) in &cas {
        for fy in [jeu.fy_2025, jeu.fy_2026] {
            let json: Value = app
                .client
                .get(app.url(&format!("/api/v1/reports/{route}?fiscalYearId={fy}{suite}")))
                .bearer_auth(&jeu.jwt)
                .send()
                .await
                .unwrap()
                .json()
                .await
                .unwrap();
            assert!(
                !json["grouping"]["open"].as_array().unwrap().is_empty()
                    || !json["grouping"]["closed"].as_array().unwrap().is_empty(),
                "{route} {fy} : aucun écart manuel retenu ({})",
                json["grouping"]
            );
            let resp = export_pme(&app, &jeu, route, fy, "pdf", suite).await;
            assert_eq!(resp.status(), 200, "{route} {fy}");
            let pdf = kesh_report::texte_pdf::lignes(&resp.bytes().await.unwrap());
            assert!(
                pdf.contains(&"Niveau 2, détail ajusté à la main".to_string()),
                "{route} {fy} : en-tête {pdf:?}"
            );
            let mut nb_lignes = 0;
            for (cle, titre, total) in sections {
                let groupees = json["groupedRows"][cle].as_array().unwrap();
                let mut attendu: Vec<String> = Vec::new();
                for l in groupees {
                    let (ligne, libelle) = ligne_attendue(l, true);
                    let depth = l["depth"].as_u64().unwrap() as u32;
                    let limite = kesh_report::pdf::limite_nom_groupe(
                        depth,
                        l["currency"].is_string() && l["balanceCurrency"].is_string(),
                    );
                    assert!(
                        libelle.chars().count() <= limite,
                        "{route} : « {libelle} » dépasse {limite} caractères (jeu à revoir)"
                    );
                    attendu.push(ligne);
                }
                nb_lignes += groupees.len();
                if *cle == "equity" {
                    let report = json["retainedEarnings"].as_str().unwrap();
                    if montant_suisse(report) != "0.00" {
                        attendu.push(format!(
                            "Résultat reporté (calculé) {}",
                            montant_suisse(report)
                        ));
                    }
                    attendu.push(format!(
                        "Résultat de l'exercice {}",
                        montant_suisse(json["equityResult"].as_str().unwrap())
                    ));
                }
                assert_eq!(
                    lignes_entre(&pdf, titre, total),
                    attendu,
                    "{route} {fy} section {cle}"
                );
            }
            assert!(
                nb_lignes >= 3,
                "{route} {fy} : jeu trop maigre ({nb_lignes} lignes)"
            );
            assert!(!pdf.iter().any(|l| l.contains('…')), "{pdf:?}");

            // Le CSV groupé, mêmes paramètres : en-tête à deux colonnes de plus,
            // une ligne par ligne groupée.
            let resp = export_pme(&app, &jeu, route, fy, "csv", suite).await;
            assert_eq!(resp.status(), 200, "{route} {fy} csv");
            let corps = resp.text().await.unwrap();
            let premiere = corps.trim_start_matches('\u{feff}').lines().next().unwrap();
            assert!(
                premiere.starts_with("Section;Niveau;TypeLigne;NumeroCompte;NomCompte;Solde"),
                "{premiere}"
            );
            let lignes_groupees = corps
                .lines()
                .filter(|l| {
                    let c: Vec<&str> = l.split(';').collect();
                    c.len() > 2 && c[1].parse::<u32>().is_ok()
                })
                .count();
            assert_eq!(lignes_groupees, nb_lignes, "{route} {fy} csv");
        }
    }
}

/// AC-1, AC-8 — les refus des paramètres de groupes de la 46-1, sur les
/// exports du bilan, du compte de résultat et (Story 46-6b, AC-2) de la
/// balance, aux deux formats : `400 VALIDATION_ERROR` qui nomme le paramètre,
/// paramètre répété compris ; `format` est contrôlé d'abord.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_groupes_refus_du_contrat(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    let refus = [
        ("&detailLevel=0", "detailLevel"),
        ("&detailLevel=21", "detailLevel"),
        ("&detailLevel=ALL", "detailLevel"),
        ("&subtotalPosition=header", "subtotalPosition"),
        ("&open=1", "open"),
        ("&detailLevel=2&subtotalPosition=middle", "subtotalPosition"),
        ("&detailLevel=2&open=1,a", "open"),
        ("&detailLevel=2&open=7&closed=3,7", "closed"),
        ("&detailLevel=2&detailLevel=3", "detailLevel"),
        ("&detailLevel=2&open=1&open=2", "open"),
    ];
    // Story 46-6b : la balance suit le même contrat.
    for route in ["balance-sheet", "income-statement", "trial-balance"] {
        for format in ["pdf", "csv"] {
            for (suite, parametre) in refus {
                let resp = export_pme(&app, &jeu, route, jeu.fy_2025, format, suite).await;
                assert_eq!(resp.status(), 400, "{route} {format}{suite}");
                let body: Value = resp.json().await.unwrap();
                assert_eq!(body["error"]["code"], "VALIDATION_ERROR", "{route}{suite}");
                let message = body["error"]["message"].as_str().unwrap_or_default();
                assert!(
                    message.contains(parametre),
                    "{route}{suite} : « {message} »"
                );
            }
        }
        // Le format d'abord : un format invalide l'emporte sur des groupes invalides.
        let resp = export_pme(&app, &jeu, route, jeu.fy_2025, "xls", "&detailLevel=0").await;
        assert_eq!(resp.status(), 400);
        let body: Value = resp.json().await.unwrap();
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("format"),
            "{body}"
        );
    }
}

/// Un montant arrondi au centime, sans apostrophe, tel que le CSV groupé
/// l'écrit — écrit ici indépendamment du rendu.
fn montant_csv(brut: &str) -> String {
    montant_suisse(brut).replace('\'', "")
}

/// Un libellé tronqué comme le PDF le fait au-delà de `max` caractères (« … »
/// final), écrit ici indépendamment du rendu.
fn tronque(texte: &str, max: usize) -> String {
    if texte.chars().count() <= max {
        texte.to_string()
    } else {
        format!("{}…", texte.chars().take(max - 1).collect::<String>())
    }
}

/// Une ligne « solde propre » telle que le PDF l'abrège (Story 46-7) : le nom
/// seul, le suffixe « (solde propre) » restant entier — écrit ici
/// indépendamment du rendu.
fn tronque_propre(nom: &str, max: usize) -> String {
    let suffixe = " (solde propre)";
    let complet = format!("{nom}{suffixe}");
    let place = max.saturating_sub(suffixe.chars().count());
    if complet.chars().count() <= max {
        complet
    } else if place == 0 {
        tronque(&complet, max)
    } else {
        format!("{}{suffixe}", tronque(nom, place))
    }
}

/// Story 46-6b, AC-2 — l'export de la balance est **fidèle à l'écran** : avec
/// les mêmes paramètres (niveau 2, un groupe ouvert et un fermé à la main,
/// sous-totaux en pied), la suite des lignes du PDF entre l'en-tête des
/// colonnes et le résultat reporté est exactement celle de `groupedRows` du
/// JSON (46-6a), selon la règle de rendu — quatre colonnes arrondies au
/// centime, seconde ligne en devise d'une ligne en devise, libellé tronqué à
/// la colonne de l'intitulé — celles du numéro et de l'intitulé pour un
/// sous-total, sans numéro ([`kesh_report::pdf::limite_nom_groupe_balance`]) ;
/// l'en-tête dit le niveau. Le CSV, mêmes paramètres : `Niveau;TypeLigne` en
/// tête et une ligne par ligne groupée, aux mêmes cellules.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_balance_groupee_fidele_au_json(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    let suite = format!(
        "&detailLevel=2&subtotalPosition=footer&open={}&closed={}",
        jeu.id("10"),
        jeu.id("20")
    );
    let mut vu_devise = false;
    for fy in [jeu.fy_2025, jeu.fy_2026] {
        let json: Value = app
            .client
            .get(app.url(&format!(
                "/api/v1/reports/trial-balance?fiscalYearId={fy}{suite}"
            )))
            .bearer_auth(&jeu.jwt)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert!(
            !json["grouping"]["open"].as_array().unwrap().is_empty()
                && !json["grouping"]["closed"].as_array().unwrap().is_empty(),
            "{fy} : écarts manuels non retenus ({})",
            json["grouping"]
        );
        let groupees = json["groupedRows"].as_array().unwrap();
        assert!(groupees.len() >= 5, "{fy} : jeu trop maigre ({groupees:?})");

        // Le PDF.
        let resp = export_pme(&app, &jeu, "trial-balance", fy, "pdf", &suite).await;
        assert_eq!(resp.status(), 200, "{fy}");
        // Les pieds de page (« page 1 / 2 ») ne sont pas des lignes du rapport.
        let pdf: Vec<String> = kesh_report::texte_pdf::lignes(&resp.bytes().await.unwrap())
            .into_iter()
            .filter(|l| !l.starts_with("page "))
            .collect();
        assert!(
            pdf.contains(&"Niveau 2, détail ajusté à la main".to_string()),
            "{fy} : en-tête {pdf:?}"
        );
        let entete = pdf
            .iter()
            .find(|l| l.starts_with("N° compte Intitulé"))
            .unwrap_or_else(|| panic!("en-tête des colonnes absent : {pdf:?}"))
            .clone();
        let mut attendu: Vec<String> = Vec::new();
        let mut csv_attendu: Vec<String> = Vec::new();
        for l in groupees {
            let kind = l["kind"].as_str().unwrap();
            let nom = l["accountName"].as_str().unwrap();
            let numero = l["accountNumber"].as_str().unwrap();
            let depth = l["depth"].as_u64().unwrap() as u32;
            let libelle = match kind {
                "group" | "account" => nom.to_string(),
                "own" => format!("{nom} (solde propre)"),
                "subtotal" => format!("Total {nom}"),
                autre => panic!("kind inconnu : {autre}"),
            };
            let montant = !(kind == "group" && l["open"] == true);
            let cles = [
                "openingBalance",
                "totalDebit",
                "totalCredit",
                "closingBalance",
            ];
            let mut morceaux: Vec<String> = Vec::new();
            if kind != "subtotal" {
                morceaux.push(numero.to_string());
            }
            let limite = kesh_report::pdf::limite_nom_groupe_balance(depth, kind != "subtotal");
            morceaux.push(if kind == "own" {
                tronque_propre(nom, limite)
            } else {
                tronque(&libelle, limite)
            });
            if montant {
                morceaux.extend(cles.map(|c| montant_suisse(l[c].as_str().unwrap())));
            }
            attendu.push(morceaux.join(" "));
            let cur = l["currency"].as_str().filter(|_| montant);
            if let Some(cur) = cur {
                vu_devise = true;
                let fx = [
                    "openingCurrency",
                    "totalDebitCurrency",
                    "totalCreditCurrency",
                    "closingCurrency",
                ]
                .map(|c| montant_suisse(l[c].as_str().unwrap_or("0")));
                attendu.push(format!(
                    "{cur} ouverture {} · débit {} · crédit {} · clôture {}",
                    fx[0], fx[1], fx[2], fx[3]
                ));
            }
            let mut cellules = vec![
                depth.to_string(),
                type_csv(kind).to_string(),
                numero.to_string(),
                nom.to_string(),
            ];
            cellules.extend(cles.map(|c| {
                if montant {
                    montant_csv(l[c].as_str().unwrap())
                } else {
                    String::new()
                }
            }));
            csv_attendu.push(cellules.join(";"));
        }
        assert_eq!(
            lignes_entre(&pdf, &entete, "Résultat reporté"),
            attendu,
            "{fy} : PDF groupé"
        );

        // Le CSV, mêmes paramètres.
        let resp = export_pme(&app, &jeu, "trial-balance", fy, "csv", &suite).await;
        assert_eq!(resp.status(), 200, "{fy} csv");
        let corps = resp.text().await.unwrap();
        let lignes: Vec<&str> = corps.trim_start_matches('\u{feff}').split("\r\n").collect();
        assert!(
            lignes[0].starts_with(
                "Niveau;TypeLigne;NumeroCompte;NomCompte;SoldeOuverture;TotalDebit;TotalCredit;SoldeCloture"
            ),
            "{}",
            lignes[0]
        );
        let colonnes = lignes[0].split(';').count();
        let corps_groupe: Vec<String> = lignes[1..=groupees.len()]
            .iter()
            .map(|l| l.split(';').take(8).collect::<Vec<_>>().join(";"))
            .collect();
        assert_eq!(corps_groupe, csv_attendu, "{fy} : CSV groupé");
        assert!(
            lignes[groupees.len() + 1].starts_with(";;;Résultat reporté (calculé);"),
            "{fy} : {lignes:?}"
        );
        assert!(
            lignes[groupees.len() + 2].starts_with(";;Total;;;"),
            "{fy} : {lignes:?}"
        );
        assert!(
            lignes[1..=groupees.len() + 2]
                .iter()
                .all(|l| l.split(';').count() == colonnes),
            "{fy} : colonnes inégales"
        );
    }
    assert!(vu_devise, "le jeu PME doit porter une ligne en devise");
}

/// `TypeLigne` du CSV pour un `kind` du JSON.
fn type_csv(kind: &str) -> &'static str {
    match kind {
        "group" => "groupe",
        "account" => "compte",
        "own" => "propre",
        "subtotal" => "soustotal",
        autre => panic!("kind inconnu : {autre}"),
    }
}

/// Story 46-6b, AC-2 — l'export de la balance **lit** désormais les
/// paramètres de groupes : un paramètre répété est refusé (`400
/// VALIDATION_ERROR`, couvert par `export_groupes_refus_du_contrat`), ce qui
/// remplace le test de la 46-3 (AC-8) et de la 46-6a (AC-9) qui l'affirmait
/// **ignoré** ; avec `detailLevel`, le corps diffère de celui d'avant ; sans,
/// il est celui d'avant (`export_balance_sans_parametre_identique_a_la_fixture_de_base`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_balance_lit_les_parametres_de_groupes(pool: MySqlPool) {
    let jeu = jeu_pme(&pool).await;
    let app = spawn_app(pool).await;
    for format in ["csv", "pdf"] {
        let repete = export_pme(
            &app,
            &jeu,
            "trial-balance",
            jeu.fy_2025,
            format,
            "&detailLevel=2&detailLevel=3",
        )
        .await;
        assert_eq!(repete.status(), 400, "{format}");
        let sans = export_pme(&app, &jeu, "trial-balance", jeu.fy_2025, format, "").await;
        assert_eq!(sans.status(), 200);
        let sans = sans.bytes().await.unwrap();
        let avec = export_pme(
            &app,
            &jeu,
            "trial-balance",
            jeu.fy_2025,
            format,
            "&detailLevel=1",
        )
        .await;
        assert_eq!(avec.status(), 200, "{format}");
        let avec = avec.bytes().await.unwrap();
        if format == "csv" {
            assert_ne!(avec, sans);
            assert!(String::from_utf8_lossy(&avec).contains("Niveau;TypeLigne"));
        } else {
            let avec = kesh_report::texte_pdf::lignes(&avec);
            assert_ne!(avec, kesh_report::texte_pdf::lignes(&sans));
            assert!(avec.contains(&"Niveau 1".to_string()), "{avec:?}");
        }
    }
}
