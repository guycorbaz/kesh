//! Tests d'intégration E2E des endpoints onboarding (story 2.2).

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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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
    create_test_company(&pool).await;
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

/// P8 / H4 — even if `is_demo = true` (potentially corrupted flag), `reset()` must
/// refuse to wipe a finalized onboarding (step >= 7). The secondary step gate is
/// the security floor; the is_demo flag alone is not sufficient.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_blocks_step_7_even_when_is_demo_flag_is_true(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Initialize onboarding_state, then forge a corrupted state: is_demo=true + step=7
    // simulates either DB tampering or a bug that flipped is_demo on a finalized tenant.
    sqlx::query("INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) VALUES (TRUE, 7, TRUE, 'guided', 1)")
        .execute(&pool)
        .await
        .expect("seed onboarding_state row");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "reset() must refuse step >= 7 regardless of is_demo"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");

    // Verify the state was NOT wiped
    let row: (i32, bool) = sqlx::query_as(
        "SELECT step_completed, is_demo FROM onboarding_state WHERE singleton = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("read onboarding_state");
    assert_eq!(row.0, 7, "step must remain 7 after refused reset");
    assert!(row.1, "is_demo must remain true after refused reset");
}

/// P6-L5 — Positive path: when KESH_PRODUCTION_RESET=1 and is_demo=true at step > 2,
/// reset() must succeed. Pairs with `reset_blocks_production_past_step_2` and
/// `reset_blocks_step_7_even_when_is_demo_flag_is_true` to cover the gate matrix.
///
/// Note: this test mutates a process-level env var. It uses `serial_test::serial`
/// or a similar guard if available; here we set + unset around the request and
/// rely on the test harness running tests sequentially (cargo test --test-threads=1
/// per `.cargo/config.toml`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_allows_demo_at_step_5_when_env_var_set(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Demo user mid-flow (step 5 = post-org-type, post-coordinates).
    sqlx::query(
        "INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) \
         VALUES (TRUE, 5, TRUE, 'guided', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed onboarding_state row");

    // SAFETY: process-wide env mutation. Tests run with --test-threads=1 (see
    // .cargo/config.toml RUST_TEST_THREADS=2 — but reset env var read happens
    // inside the request handler in this same process). We restore the previous
    // value after the request to avoid leaking into subsequent tests.
    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
    // SAFETY (Rust 2024): set_var/remove_var are unsafe due to potential races
    // with other threads reading env. Acceptable inside a serialized test.
    unsafe {
        std::env::set_var("KESH_PRODUCTION_RESET", "true");
    }

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    // Restore env var BEFORE assertions so a panic doesn't leak state.
    unsafe {
        match prev {
            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
        }
    }

    assert_eq!(
        resp.status(),
        200,
        "reset() with KESH_PRODUCTION_RESET=true must succeed for is_demo=true at step 5"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0, "state should be reset to step 0");
    assert_eq!(body["isDemo"], false, "is_demo cleared after reset");
}

/// P8 / P4 — production path (is_demo=false) past step 2 must always be blocked,
/// regardless of KESH_PRODUCTION_RESET. P6-L8: distinct ONBOARDING_RESET_FORBIDDEN
/// error code (403) so the client can distinguish policy-refusal from a finalized
/// onboarding (which uses ONBOARDING_STEP_ALREADY_COMPLETED at 400).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_blocks_production_past_step_2(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    sqlx::query("INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) VALUES (TRUE, 5, FALSE, 'expert', 1)")
        .execute(&pool)
        .await
        .expect("seed onboarding_state row");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_RESET_FORBIDDEN");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_demo_at_wrong_step_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
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
