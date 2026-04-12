//! Tests d'intégration E2E des endpoints onboarding (story 2.2).

use std::sync::Arc;

use chrono::TimeDelta;
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

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
    };

    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
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
        .expect("login request");
    let body: serde_json::Value = resp.json().await.expect("login json");
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_string()
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

// --- Tests ---

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_state_returns_initial_state(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

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
    assert_eq!(body["isDemo"], false);
    assert!(body["uiMode"].is_null());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_state_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_language_advances_to_step_1(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_language_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "XX" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_language_twice_returns_step_already_completed(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // First call succeeds
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // Second call fails
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "DE" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn set_mode_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // First set language
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // Invalid mode
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "invalid" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_onboarding_flow_demo_path(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Step 0 → 1: language
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Step 1 → 2: mode
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 2);
    assert_eq!(body["uiMode"], "guided");

    // Step 2 → 3: seed demo
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isDemo"], true);
    assert_eq!(body["uiMode"], "guided");

    // Verify state persisted
    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isDemo"], true);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_clears_demo_data(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Complete onboarding
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "expert" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    // Reset
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0);
    assert_eq!(body["isDemo"], false);
    assert!(body["uiMode"].is_null());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_demo_at_wrong_step_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Try seed-demo at step 0 (should require step 2)
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");
}
