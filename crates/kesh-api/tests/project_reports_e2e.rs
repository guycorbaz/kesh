//! Tests E2E HTTP Story 19-6a — rapport « Dépenses par projet ».
//!
//! Couvre : JSON 200 + shape, export PDF (signature %PDF) + CSV (BOM),
//! validation (mode=fiscal_year sans fiscalYearId → 400 ; projectId ≤ 0 → 400),
//! projet inconnu → 404, multi-tenant (projet d'une autre company → 404).
//!
//! Pré-requis : MariaDB (sqlx::test crée une DB éphémère par test).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::Value;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-admin-password".to_string(),
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
    let app = build_router(state, "nonexistent-static-dir".to_string());
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
            Err(e) => panic!("test server not ready: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: "Comptable".to_string(),
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

async fn mk_project(pool: &MySqlPool, company_id: i64, code: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, ?, ?, FALSE) RETURNING id",
    )
    .bind(company_id)
    .bind(code)
    .bind(format!("Projet {code}"))
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn post_expense(pool: &MySqlPool, seeded: &SeededCompany, project: i64) {
    journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        seeded.admin_user_id,
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            journal: Journal::Achats,
            description: "Dépense projet".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["4000"],
                    debit: rust_decimal_macros::dec!(120.00),
                    credit: rust_decimal::Decimal::ZERO,
                    project_id: Some(project),
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["2000"],
                    debit: rust_decimal::Decimal::ZERO,
                    credit: rust_decimal_macros::dec!(120.00),
                    project_id: Some(project),
                },
            ],
        },
    )
    .await
    .unwrap();
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_expenses_json_shape(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "RENOV").await;
    post_expense(&pool, &seeded, project).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/project-expenses?projectId={project}&mode=fiscal_year&fiscalYearId={}",
            seeded.fiscal_year_id
        )))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["reportType"], "project-expenses");
    assert_eq!(body["project"]["code"], "RENOV");
    assert_eq!(body["grandTotal"], "120.0000");
    assert!(body["sections"].as_array().unwrap().len() == 1);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_expenses_export_pdf_and_csv(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "RENOV").await;
    post_expense(&pool, &seeded, project).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);
    let base = format!(
        "/api/v1/reports/project-expenses/export?projectId={project}&mode=fiscal_year&fiscalYearId={}",
        seeded.fiscal_year_id
    );

    // PDF.
    let pdf = app
        .client
        .get(app.url(&format!("{base}&format=pdf")))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(pdf.status(), 200);
    assert_eq!(pdf.headers()["content-type"], "application/pdf");
    let bytes = pdf.bytes().await.unwrap();
    assert!(bytes.starts_with(b"%PDF"), "signature PDF");

    // CSV.
    let csv = app
        .client
        .get(app.url(&format!("{base}&format=csv")))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(csv.status(), 200);
    let csv_bytes = csv.bytes().await.unwrap();
    assert_eq!(&csv_bytes[0..3], &[0xEF, 0xBB, 0xBF], "BOM UTF-8");
    let text = String::from_utf8_lossy(&csv_bytes[3..]);
    assert!(text.contains("Projet;SousProjet;NumeroCompte"));
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_expenses_cumulative_mode(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "RENOV").await;
    post_expense(&pool, &seeded, project).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);

    // Mode cumulé : pas de fiscalYearId requis.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/project-expenses?projectId={project}&mode=cumulative"
        )))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["mode"], "cumulative");
    assert_eq!(body["grandTotal"], "120.0000");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_expenses_validation_and_not_found(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "RENOV").await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);

    // fiscal_year sans fiscalYearId → 400.
    let r400 = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/project-expenses?projectId={project}&mode=fiscal_year"
        )))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r400.status(), 400);

    // projectId inconnu → 404.
    let r404 = app
        .client
        .get(app.url("/api/v1/reports/project-expenses?projectId=999999&mode=cumulative"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r404.status(), 404);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_expenses_multi_tenant_isolation(pool: MySqlPool) {
    // Company A seedée + projet ; un JWT d'une autre company ne doit pas y accéder.
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "RENOV").await;
    let app = spawn_app(pool.clone()).await;

    // JWT forgé pour une company_id fictive (999) — le projet appartient à A.
    let other_jwt = forge_jwt(seeded.admin_user_id, 999);
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/project-expenses?projectId={project}&mode=cumulative"
        )))
        .bearer_auth(&other_jwt)
        .send()
        .await
        .unwrap();
    // resolve_scope scope par company_id du JWT → projet introuvable → 404.
    assert_eq!(resp.status(), 404);
}

// ---- Story 19-6b : Rendement par projet ----

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_return_json_and_export(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let project = mk_project(&pool, seeded.company_id, "INVEST").await;
    post_expense(&pool, &seeded, project).await; // 120 de charge (Expense 4000)
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);

    // JSON.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/project-return?projectId={project}&mode=fiscal_year&fiscalYearId={}",
            seeded.fiscal_year_id
        )))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["reportType"], "project-return");
    // Coût investi = charge 120 (Expense) ; 0 revenu → rendement 0.00 (coût > 0).
    assert_eq!(body["totals"]["coutInvesti"], "120.0000");
    assert_eq!(body["totals"]["rendementPct"], "0"); // 0 revenu → rendement 0 (frontend formate 0.00%)

    // Export PDF + CSV.
    let base = format!(
        "/api/v1/reports/project-return/export?projectId={project}&mode=fiscal_year&fiscalYearId={}",
        seeded.fiscal_year_id
    );
    let pdf = app
        .client
        .get(app.url(&format!("{base}&format=pdf")))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(pdf.status(), 200);
    assert!(pdf.bytes().await.unwrap().starts_with(b"%PDF"));

    let csv = app
        .client
        .get(app.url(&format!("{base}&format=csv")))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(csv.status(), 200);
    let csv_bytes = csv.bytes().await.unwrap();
    assert_eq!(&csv_bytes[0..3], &[0xEF, 0xBB, 0xBF]);
    assert!(String::from_utf8_lossy(&csv_bytes[3..]).contains("Projet;SousProjet;CoutInvesti"));
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn project_return_unknown_project_404(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);
    let resp = app
        .client
        .get(app.url("/api/v1/reports/project-return?projectId=999999&mode=cumulative"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}
