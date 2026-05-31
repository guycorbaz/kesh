//! Tests d'intégration `POST /api/v1/setup/admin` (Story v011-5).
//!
//! Couvre :
//! - Happy path : DB vide + 1 stub company → 200 + Set-Cookie + flag mémoire `users_exist=true`.
//! - 410 SETUP_ALREADY_COMPLETE : second appel après création réussie.
//! - 423 SETUP_REQUIRED : `GET /api/v1/auth/me` retourne 423 quand `users_exist=false`.
//! - Validation : password < 12 chars → 400 VALIDATION_ERROR.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{seed_stub_company_only, truncate_all};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    users_exist: Arc<AtomicBool>,
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
        "test-admin-password-12chars".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        // Rate-limit large pour ne pas interférer avec les tests (5 dans la matrice
        // réelle, mais notre suite a besoin de plusieurs POSTs sans déclencher le bloc).
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool, users_exist_initial: bool) -> TestApp {
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

    let users_exist = Arc::new(AtomicBool::new(users_exist_initial));

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
        users_exist: users_exist.clone(),
    };

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

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("client");

    TestApp {
        base_url: format!("http://{}", addr),
        client,
        users_exist,
    }
}

/// AC #11 — Happy path : DB vide (1 stub company seedée) + users_exist=false →
/// POST /setup/admin retourne 200 + Set-Cookie HttpOnly + flag bascule true.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_happy_path_returns_200_and_cookies(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool.clone(), false).await;
    // État initial : users_exist=false (pas d'admin).
    assert!(!app.users_exist.load(Ordering::Acquire));

    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "first-admin",
            "password": "first-admin-password-secure-12",
        }))
        .send()
        .await
        .expect("setup POST");

    assert_eq!(res.status(), 200, "happy path → 200");

    // Vérifier Set-Cookie HttpOnly access + refresh.
    let cookies: Vec<_> = res.headers().get_all("set-cookie").iter().collect();
    let cookie_str: String = cookies
        .iter()
        .map(|h| h.to_str().unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" | ");
    assert!(
        cookie_str.contains("kesh_access_token"),
        "should set kesh_access_token cookie: {cookie_str}"
    );
    assert!(
        cookie_str.contains("kesh_refresh_token"),
        "should set kesh_refresh_token cookie"
    );
    assert!(cookie_str.contains("HttpOnly"), "cookies must be HttpOnly");

    let body: Value = res.json().await.expect("body json");
    assert_eq!(body["username"], "first-admin");
    assert_eq!(body["role"], "Admin");

    // Flag mémoire users_exist basculé à true post-INSERT.
    assert!(
        app.users_exist.load(Ordering::Acquire),
        "users_exist should be true after successful setup"
    );

    // Vérifier DB : 1 admin créé attaché à la company stub.
    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(user_count, 1);
}

/// AC #10 — Deuxième appel → 410 SETUP_ALREADY_COMPLETE.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_returns_410_when_user_already_exists(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool.clone(), false).await;

    // 1er POST (happy path).
    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "admin",
            "password": "first-admin-password-secure-12",
        }))
        .send()
        .await
        .expect("first POST");
    assert_eq!(res.status(), 200);

    // 2e POST → 410.
    let res2 = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "attacker",
            "password": "attacker-tries-too-late-456",
        }))
        .send()
        .await
        .expect("second POST");
    assert_eq!(res2.status(), 410, "second setup → 410 Gone");

    let body: Value = res2.json().await.expect("body");
    assert_eq!(body["error"]["code"], "SETUP_ALREADY_COMPLETE");

    // Pas de 2e user créé.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 1, "no second admin created");
}

/// AC #13 — Route protégée + `users_exist=false` → 423 SETUP_REQUIRED.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_returns_423_when_users_empty(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool, false).await;

    let res = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .expect("GET /me");
    assert_eq!(res.status(), 423, "protected route + no users → 423");

    let body: Value = res.json().await.expect("body");
    assert_eq!(body["error"]["code"], "SETUP_REQUIRED");
}

/// AC #9 — Validation : password < 12 chars → 400 VALIDATION_ERROR.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_returns_400_on_weak_password(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool.clone(), false).await;

    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "admin",
            "password": "tooshort",
        }))
        .send()
        .await
        .expect("POST");
    assert_eq!(res.status(), 400);

    let body: Value = res.json().await.expect("body");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    // Aucun user créé.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

/// AC #9 — Validation : username vide → 400 VALIDATION_ERROR.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_returns_400_on_empty_username(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool, false).await;

    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "   ",
            "password": "valid-password-with-enough-chars",
        }))
        .send()
        .await
        .expect("POST");
    assert_eq!(res.status(), 400);
}

/// AC #14 — Routes publiques exemptes du gate 423.
/// `/health` doit retourner 200 même si `users_exist=false`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn health_endpoint_bypasses_setup_gate(pool: MySqlPool) {
    let app = spawn_app(pool, false).await;

    let res = app
        .client
        .get(app.url("/health"))
        .send()
        .await
        .expect("GET /health");
    // /health peut retourner 200 ou 503 selon DB ; l'important est que ce ne soit PAS 423.
    assert_ne!(res.status(), 423, "/health must not be gated by setup");
}
