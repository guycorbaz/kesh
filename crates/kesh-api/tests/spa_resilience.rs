//! Tests d'intégration de la résilience SPA + endpoints DB-indépendants
//! quand la DB est perdue (Story 10.3, AC #5).
//!
//! Stratégie : `#[sqlx::test(migrations = "../kesh-db/test-schema")]` fournit
//! une base éphémère migrée. La perte de DB est simulée via `pool.close()` (méthode
//! canonique sqlx 0.8). Les deux tests vérifient que :
//!
//! - (a) Le fallback `ServeDir` (`lib.rs:54+`) continue de servir `index.html`
//!   sur `GET /` même pool fermé — pure I/O fichier, aucun accès DB.
//! - (b) `/api/v1/i18n/messages` reste joignable après `pool.close()`, le
//!   handler ne touchant que `state.config.locale` + bundle Fluent in-memory.
//!   Route JWT-protégée → setup obligatoire avant `pool.close()` :
//!   `ensure_admin_user` + login → JWT, puis fermeture pool, puis GET avec
//!   header `Authorization: Bearer <token>`. Le middleware JWT vérifie la
//!   signature en mémoire, donc le token reste valide DB down.

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
const TEST_ADMIN_PASSWORD: &str = "spa-resilience-test-password";

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

/// Résout le path du fixture SPA stub via `CARGO_MANIFEST_DIR` (pattern projet
/// canonique, 28 occurrences dans `crates/kesh-api/tests/*.rs`). Évite la fragilité
/// d'un chemin relatif au CWD du test (qui diffère entre `cargo test --workspace`
/// cwd=workspace-root et `-p kesh-api` cwd=crate-dir).
fn spa_stub_dir() -> String {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/spa-stub")
        .to_string_lossy()
        .into_owned()
}

async fn spawn_app(pool: MySqlPool, static_dir: String) -> TestApp {
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

    let app = build_router(state, static_dir);
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

/// AC #5(a) : pool fermé, le fallback `ServeDir` continue de servir `index.html`
/// sur `GET /` en pure I/O fichier (200 + Content-Type `text/html`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn spa_index_served_when_db_down(pool: MySqlPool) {
    let app = spawn_app(pool.clone(), spa_stub_dir()).await;

    pool.close().await;

    let resp = app
        .client
        .get(app.url("/"))
        .send()
        .await
        .expect("GET / request");

    assert_eq!(resp.status(), 200);
    let content_type = resp
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        content_type.starts_with("text/html"),
        "expected text/html content-type, got: {content_type}"
    );

    let body = resp.text().await.expect("body text");
    assert!(
        body.contains("kesh-spa-stub"),
        "body should contain SPA stub marker, got: {body}"
    );
}

/// AC #5(b) : route JWT-protégée `/api/v1/i18n/messages` reste joignable après
/// `pool.close()` car le handler ne touche pas la DB (config locale + bundle
/// Fluent in-memory) et le middleware JWT vérifie la signature en mémoire.
///
/// Setup obligatoire : créer admin + login pour obtenir un JWT valide AVANT
/// de fermer le pool. Sinon impossible d'émettre un token (le login fait des
/// requêtes DB).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn i18n_messages_served_when_db_down(pool: MySqlPool) {
    let app = spawn_app(pool.clone(), spa_stub_dir()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();

    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    pool.close().await;

    let resp = app
        .client
        .get(app.url("/api/v1/i18n/messages"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("i18n request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert!(
        body["locale"].is_string(),
        "locale should be a string, got: {body}"
    );
    assert!(
        body["messages"].is_object(),
        "messages should be an object, got: {body}"
    );
    assert!(
        !body["messages"].as_object().unwrap().is_empty(),
        "messages object should not be empty"
    );
}
