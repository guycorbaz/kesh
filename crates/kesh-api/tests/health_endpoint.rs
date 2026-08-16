//! Tests d'intégration du `GET /health` (Story 10.3).
//!
//! Couvre la nouvelle shape body `{ status, db, version }` introduite par
//! Story 10.3 — consommée par le frontend `apiHealth.pollHealth()` (banner
//! dégradé + auto-recovery) et par le smoke test post-build `release.yml`.
//!
//! Stratégie : `#[sqlx::test(migrations = "../kesh-db/test-schema")]` fournit
//! une base éphémère migrée. Le cas "DB down" est simulé via `pool.close()` —
//! la méthode canonique sqlx 0.8 pour fermer les connexions sans intervention
//! infrastructure (cf. AC #4).

use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use sqlx::MySqlPool;
use std::net::SocketAddr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "health-endpoint-test-password";

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

    // Pass 1 code review F4 : `static_dir` non-existant intentionnel — ce test n'exerce
    // que `GET /health` qui résout via le router Axum avant que le fallback `ServeDir`
    // (servant la SPA statique) ne soit consulté. Un path bidon évite de coupler ce test
    // au build frontend (`frontend/build/index.html`). Cohérent avec le pattern
    // `spa_resilience.rs` qui, lui, utilise `spa_stub_dir()` car il **teste** explicitement
    // le fallback `ServeDir` (GET / → index.html).
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

/// AC #4(a) : pool ouvert, `SELECT 1` réussit → 200 + body `{ status: "ok",
/// db: true, version: <CARGO_PKG_VERSION> }`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn health_endpoint_returns_ok_when_db_up(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/health"))
        .send()
        .await
        .expect("health request");

    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(body["status"], "ok");
    assert_eq!(body["db"], true);
    let version = body["version"].as_str().expect("version is a string");
    assert!(!version.is_empty(), "version should not be empty");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}

/// AC #4(b) : `pool.close()` simule la perte de DB → `SELECT 1` échoue → 503
/// + body `{ status: "degraded", db: false, version: <CARGO_PKG_VERSION> }`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn health_endpoint_returns_degraded_when_db_down(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // Sanity-check: DB up first
    let resp_up = app
        .client
        .get(app.url("/health"))
        .send()
        .await
        .expect("health request (up)");
    assert_eq!(resp_up.status(), 200);

    // Simulate DB failure: close the pool (canonical sqlx 0.8 method).
    pool.close().await;

    let resp_down = app
        .client
        .get(app.url("/health"))
        .send()
        .await
        .expect("health request (down)");

    assert_eq!(resp_down.status(), 503);

    let body: serde_json::Value = resp_down.json().await.expect("json");
    assert_eq!(body["status"], "degraded");
    assert_eq!(body["db"], false);
    let version = body["version"].as_str().expect("version is a string");
    assert!(!version.is_empty(), "version should not be empty");
    assert_eq!(version, env!("CARGO_PKG_VERSION"));
}
