//! Tests d'intégration E2E de l'internationalisation (story 2.1).
//!
//! Note: tests run serially because they share a global i18n state
//! (kesh_api::errors::init_error_i18n sets a static ONCE_LOCK).
//! Running tests in parallel would cause race conditions.

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
use std::sync::OnceLock;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// Mutex to synchronize i18n initialization across concurrent tests
static I18N_INIT_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

async fn get_i18n_lock() -> tokio::sync::MutexGuard<'static, ()> {
    I18N_INIT_LOCK
        .get_or_init(|| tokio::sync::Mutex::new(()))
        .lock()
        .await
}

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

/// Les quatre catalogues réels (`crates/kesh-i18n/locales`), tels que l'application
/// de test les charge — et tels que les tests les relisent pour calculer l'attendu.
fn catalogues() -> kesh_i18n::I18nBundle {
    kesh_i18n::I18nBundle::load(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("kesh-i18n/locales")
            .as_path(),
    )
    .expect("load test i18n")
}

async fn spawn_app_with_locale(pool: MySqlPool, locale: kesh_i18n::Locale) -> TestApp {
    let config = test_config().with_locale(locale);
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(catalogues());

    // Init global i18n pour les messages d'erreur
    // Note: must be called within a get_i18n_lock guard from the test
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn i18n_messages_endpoint_returns_locale_and_messages(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn error_messages_are_in_french_by_default(pool: MySqlPool) {
    let _guard = get_i18n_lock().await;
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn error_messages_in_german_when_locale_de(pool: MySqlPool) {
    let _guard = get_i18n_lock().await;
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

/// Story 32-1d (#711, L-B7) — le refus d'un réglage de sauvegarde hors domaine
/// (`PUT /api/v1/admin/backup-schedule`, fréquence 7 h) parle la langue de
/// l'instance : le message égale le rendu `de-CH` de
/// `admin-backup-schedule-error-out-of-domain`, et diffère du rendu `fr-CH` —
/// sans quoi un repli français resté dans la route passerait pour juste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn backup_schedule_out_of_domain_refusal_in_german(pool: MySqlPool) {
    const CLE: &str = "admin-backup-schedule-error-out-of-domain";
    let _guard = get_i18n_lock().await;
    let app = spawn_app_with_locale(pool.clone(), kesh_i18n::Locale::DeCh).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let reglage: serde_json::Value = app
        .client
        .get(app.url("/api/v1/admin/backup-schedule"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get backup-schedule")
        .json()
        .await
        .expect("json");
    let version = reglage["version"].clone();
    assert!(version.is_i64(), "version absente : {reglage}");

    let resp = app
        .client
        .put(app.url("/api/v1/admin/backup-schedule"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "enabled": true, "intervalHours": 7, "retentionCount": 5, "version": version
        }))
        .send()
        .await
        .expect("put backup-schedule");
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let message = body["error"]["message"].as_str().unwrap();

    let cat = catalogues();
    let rendu = |locale: kesh_i18n::Locale| {
        cat.format(&locale, CLE, None)
            .replace(['\u{2068}', '\u{2069}'], "")
    };
    let de = rendu(kesh_i18n::Locale::DeCh);
    let fr = rendu(kesh_i18n::Locale::FrCh);
    assert_ne!(de, CLE, "de-CH : {CLE} absente du catalogue");
    assert_ne!(de, fr, "de-CH : {CLE} retombe sur le français");
    assert_eq!(message, de);
}

/// Story 32-5c (refs #723, AC-c4) — le refus d'un réseau à bits d'hôte
/// (`PUT /api/v1/admin/mcp/settings`, `10.1.2.3/8`) parle la langue de
/// l'instance, avec ses deux paramètres : l'entrée et la forme attendue
/// (`10.0.0.0/8`), sans marques d'isolation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mcp_settings_host_bits_refusal_in_german(pool: MySqlPool) {
    const CLE: &str = "admin-mcp-settings-error-network-host-bits";
    let _guard = get_i18n_lock().await;
    let app = spawn_app_with_locale(pool.clone(), kesh_i18n::Locale::DeCh).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .put(app.url("/api/v1/admin/mcp/settings"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "enabled": true, "addressFilterEnabled": true,
            "allowedNetworks": ["10.1.2.3/8"], "trustedProxies": [], "allowedOrigins": [],
            "version": 0
        }))
        .send()
        .await
        .expect("put mcp/settings");
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let message = body["error"]["message"].as_str().unwrap();

    let cat = catalogues();
    let mut args = kesh_i18n::FluentArgs::new();
    args.set("entry", "10.1.2.3/8");
    args.set("expected", "10.0.0.0/8");
    let rendu = |locale: kesh_i18n::Locale| {
        cat.format(&locale, CLE, Some(&args))
            .replace(['\u{2068}', '\u{2069}'], "")
    };
    let de = rendu(kesh_i18n::Locale::DeCh);
    let fr = rendu(kesh_i18n::Locale::FrCh);
    assert_ne!(de, CLE, "de-CH : {CLE} absente du catalogue");
    assert_ne!(de, fr, "de-CH : {CLE} retombe sur le français");
    assert!(de.contains("10.0.0.0/8"), "forme attendue absente : {de}");
    assert_eq!(message, de);

    let lignes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM mcp_server_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(lignes, 0, "un refus n'écrit rien");
}
