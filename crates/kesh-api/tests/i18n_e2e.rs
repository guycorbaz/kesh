//! Tests d'intégration E2E de l'internationalisation (story 2.1).

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
    spawn_app_with_locale(pool, kesh_i18n::Locale::FrCh).await
}

async fn spawn_app_with_locale(pool: MySqlPool, locale: kesh_i18n::Locale) -> TestApp {
    let config = test_config().with_locale(locale);
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

    // Init global i18n pour les messages d'erreur
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

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .expect("login request");
    let body: serde_json::Value = resp.json().await.expect("login json");
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_string()
}

// --- Tests ---

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn i18n_messages_endpoint_returns_locale_and_messages(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();

    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url("/api/v1/i18n/messages"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("i18n request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["locale"], "fr-CH");
    assert!(body["messages"].is_object());
    assert!(body["messages"]["error-invalid-credentials"].is_string());
    assert!(body["messages"]["error-forbidden"].is_string());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn i18n_messages_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/i18n/messages"))
        .send()
        .await
        .expect("i18n request");

    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn error_messages_are_in_french_by_default(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "wrong", "password": "wrong" }))
        .send()
        .await
        .expect("login request");

    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
    let message = body["error"]["message"].as_str().unwrap();
    assert_eq!(message, "Identifiants invalides");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn error_messages_in_german_when_locale_de(pool: MySqlPool) {
    let app = spawn_app_with_locale(pool.clone(), kesh_i18n::Locale::DeCh).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "wrong", "password": "wrong" }))
        .send()
        .await
        .expect("login request");

    assert_eq!(resp.status(), 401);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
    let message = body["error"]["message"].as_str().unwrap();
    assert_eq!(message, "Ungültige Anmeldedaten");
}
