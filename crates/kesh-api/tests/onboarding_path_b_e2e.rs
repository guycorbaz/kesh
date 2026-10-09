//! Tests E2E API — Onboarding Chemin B (Story 2.3).

mod common;

use std::sync::Arc;

use chrono::TimeDelta;
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use serde_json::json;
use sqlx::MySqlPool;
use std::net::SocketAddr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

/// Helper : advance through shared steps (language + mode) to step=2
async fn advance_to_step_2(app: &TestApp, token: &str) {
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();
}

// --- Tests ---

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn start_production_advances_to_step_3(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isDemo"], false);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn org_type_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Invalid" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn coordinates_validates_ide(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    // Advance to step 5
    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Pme" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // Invalid IDE
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Test SA", "address": {"street":"Rue Test","building":"1","postalCode":"1000","city":"Lausanne","country":"CH"}, "ideNumber": "INVALID" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn full_path_b_flow(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    // Step 2→3: start production
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Step 3→4: org type
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Independant" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Step 4→5: accounting language
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "DE" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Step 5→6: coordinates
    let resp = app.client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Ma Société SA", "address": {"street":"Rue Test","building":"1","postalCode":"1000","city":"Lausanne","country":"CH"}, "ideNumber": "CHE-109.322.551" }))
        .send().await.unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 6);

    // Step 6→7: bank account
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/bank-account"))
        .header("Authorization", auth(&token))
        .json(&json!({ "bankName": "UBS", "iban": "CH93 0076 2011 6238 5295 7", "qrIban": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 7);
    assert_eq!(body["isDemo"], false);

    // Story 15-7a1 (test 8) — finalisation, puis état après `finalize`. Seul test
    // qui traverse les quatre fonctions du socle par la route de production :
    // `bulk_create_from_chart` (accounting-language), `upsert_primary`
    // (bank-account), `insert_with_defaults_in_tx` et
    // `seed_default_swiss_rates_in_tx` (finalize).
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/finalize"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 8);

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let categories: Vec<String> =
        sqlx::query_scalar("SELECT category FROM vat_rates WHERE company_id = ? ORDER BY id")
            .bind(company_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(categories, ["normal", "special", "reduced", "exempt"]);
    let settings_rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM company_invoice_settings WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(settings_rows, 1, "une ligne de réglages de facturation");
    // `user.created` : bootstrap (`ensure_admin_user`), avant toute route ; puis
    // la séquence de l'onboarding (Story 15-7a2, AC 2). Société de
    // `create_test_company` (non provisoire, `Independant`, FR/FR) : la langue
    // FR et le type `Independant` ne changent rien, d'où l'absence de
    // `company.updated` pour `language` et `org-type` ; les coordonnées ne
    // portent pas la levée du drapeau provisoire.
    let actions = common::audit_sequence(&pool).await;
    assert_eq!(
        actions,
        [
            "user.created",
            "installation.step_completed",
            "installation.ui_mode_changed",
            "installation.step_completed",
            "installation.step_completed",
            "installation.step_completed",
            "company.updated",
            "account.chart_loaded",
            "installation.step_completed",
            "company.updated",
            "installation.step_completed",
            "bank_account.created",
            "installation.step_completed",
            "company_invoice_settings.created",
            "vat_rate.created",
            "vat_rate.created",
            "vat_rate.created",
            "vat_rate.created",
            "fiscal_year.created",
            "installation.step_completed",
        ]
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn skip_bank_advances_to_step_7(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    // Advance to step 6
    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Association" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Mon Asso", "address": {"street":"Rue Test","building":"1","postalCode":"1000","city":"Lausanne","country":"CH"}, "ideNumber": null }))
        .send()
        .await
        .unwrap();

    // Skip bank
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/skip-bank"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 7);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_account_validates_iban(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    advance_to_step_2(&app, &token).await;

    // Advance to step 6
    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Pme" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Test", "address": {"street":"Rue Test","building":"1","postalCode":"1000","city":"Lausanne","country":"CH"}, "ideNumber": null }))
        .send()
        .await
        .unwrap();

    // Invalid IBAN
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/bank-account"))
        .header("Authorization", auth(&token))
        .json(&json!({ "bankName": "UBS", "iban": "INVALID", "qrIban": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Story v011-2 (Issue #120) — fresh install : pas de `create_test_company`, le
/// bootstrap crée lui-même la company stub (`is_stub=TRUE`). `GET /state` expose
/// `isStub=true` à step 0 (AC #8-9, #14e), et le flag retombe à `false` une fois
/// les coordonnées renseignées (AC #10, #15).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fresh_install_stub_exposed_then_cleared_by_coordinates(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    // PAS de create_test_company : le bootstrap doit créer la company stub.
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // GET /state : step 0 + isStub true (company issue du bootstrap stub).
    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0);
    assert_eq!(body["isStub"], true);

    // Avance jusqu'aux coordonnées (step 5→6).
    advance_to_step_2(&app, &token).await;
    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(&token))
        .json(&json!({ "orgType": "Pme" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // set_coordinates → isStub doit retomber à false.
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Vraie Société SA", "address": {"street":"Rue Test","building":"1","postalCode":"1000","city":"Lausanne","country":"CH"}, "ideNumber": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 6);
    assert_eq!(body["isStub"], false);

    // Persistance : GET /state reflète isStub=false.
    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["isStub"], false);
}
