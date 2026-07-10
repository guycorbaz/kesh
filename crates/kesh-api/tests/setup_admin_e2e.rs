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

fn test_config_with_max_attempts(max_attempts: u32) -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "test-admin-password-12chars".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        max_attempts,
        TimeDelta::minutes(30),
        12,
    )
}

fn test_config() -> Config {
    // Rate-limit large pour ne pas interférer avec les tests (5 dans la matrice
    // réelle, mais notre suite a besoin de plusieurs POSTs sans déclencher le bloc).
    test_config_with_max_attempts(100)
}

async fn spawn_app(pool: MySqlPool, users_exist_initial: bool) -> TestApp {
    spawn_app_with_config(pool, users_exist_initial, test_config()).await
}

async fn spawn_app_with_config(
    pool: MySqlPool,
    users_exist_initial: bool,
    config: Config,
) -> TestApp {
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
        // Story 17-4c — littéral-exception (E2E setup) : limiter recovery.
        rate_limiter_recovery: Arc::new(kesh_api::build_recovery_rate_limiter()),
        i18n,
        users_exist: users_exist.clone(),
        // Story 17-4b — littéral-exception (users_exist variable) : mailer no-op.
        mailer: Arc::new(kesh_api::mail::NoopMailer),
        // Story 20-3b1 — champs hors scope setup (défauts).
        rate_limiter_send_email: Arc::new(kesh_api::build_send_email_rate_limiter()),
        smtp_ready: false,
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

/// T-A5 (Story 17-4a) — Validation email : email invalide → 400.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_with_invalid_email_returns_400(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool.clone(), false).await;

    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "first-admin",
            "password": "first-admin-password-secure-12",
            "email": "bad",
        }))
        .send()
        .await
        .expect("POST");
    assert_eq!(res.status(), 400);

    // Aucun user créé.
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(count, 0);
}

/// T-A5 (Story 17-4a) — Validation email : email valide → succès + email persisté.
/// La réponse `/setup/admin` (`LoginResponse`) n'expose pas l'email, donc on
/// l'assert directement en base.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_with_valid_email_succeeds(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    let app = spawn_app(pool.clone(), false).await;

    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "first-admin",
            "password": "first-admin-password-secure-12",
            "email": "admin@example.com",
        }))
        .send()
        .await
        .expect("POST");
    assert_eq!(res.status(), 200);

    let email: Option<String> =
        sqlx::query_scalar("SELECT email FROM users WHERE username = 'first-admin'")
            .fetch_one(&pool)
            .await
            .expect("fetch email");
    assert_eq!(email.as_deref(), Some("admin@example.com"));
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

/// AC #22 (CR Pass 1 AUD1-1) — Rate-limit déclenche 429 après N tentatives invalides.
/// Utilise `max_attempts=2` pour atteindre le bloc en 3 POSTs.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn setup_admin_rate_limit_returns_429(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");

    // Rate-limit serré pour ce test : 2 tentatives autorisées, la 3e bloque.
    let app = spawn_app_with_config(pool, false, test_config_with_max_attempts(2)).await;

    // 2 tentatives invalides (password trop court → 400 + record_failed_attempt).
    for i in 0..2 {
        let res = app
            .client
            .post(app.url("/api/v1/setup/admin"))
            .json(&json!({ "username": "admin", "password": "short" }))
            .send()
            .await
            .expect("POST");
        assert_eq!(res.status(), 400, "tentative {}: validation error", i + 1);
    }

    // 3e tentative → 429 (bloquée par rate-limit même avant validation).
    let res = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({
            "username": "admin",
            "password": "valid-password-12-chars",
        }))
        .send()
        .await
        .expect("POST blocked");
    assert_eq!(res.status(), 429, "rate-limit should block");

    // L'header Retry-After doit être présent (cohérent /auth/login).
    let retry_after = res.headers().get("retry-after");
    assert!(retry_after.is_some(), "Retry-After header expected on 429");
}

/// AC #6 / AC1 — Race TOCTOU 2 usernames distincts : **garantie stricte** que le
/// verrou sentinelle (`SELECT _kesh_version id=1 FOR UPDATE`, Story 17-1) crée
/// au plus 1 admin (fermeture issue #133, ex-limitation L1 Story v011-5).
///
/// Deux requêtes concurrentes avec des usernames DIFFÉRENTS (`alice`/`bob`) — donc
/// non bloquées entre elles par la contrainte `UNIQUE username` — se sérialisent
/// sur le verrou sentinelle. Résultat **déterministe** :
/// - exactement **1** utilisateur en base après les deux requêtes ;
/// - l'ensemble des deux status HTTP == `{200, 410}` (un succès, un auto-disable),
///   dans un ordre quelconque (l'ordre alice/bob gagnant reste non déterministe).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn toctou_race_two_distinct_usernames_creates_exactly_one_admin(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    seed_stub_company_only(&pool).await.expect("seed stub");
    // La row sentinelle `_kesh_version (id=1)` est posée par la migration
    // 20260522000001 (`INSERT ... VALUES (1, '0.1.0', '0.1.0')`) appliquée par
    // `kesh_db::MIGRATOR` — indispensable pour que le `FOR UPDATE` verrouille.

    let app = spawn_app(pool.clone(), false).await;
    let base_url = app.base_url.clone();

    // Les deux requêtes partagent le même process backend (même AppState/pool)
    // → la contention DB se produit réellement sur le verrou sentinelle.
    let client_a = app.client.clone();
    let client_b = app.client.clone();
    let url_a = format!("{}/api/v1/setup/admin", base_url);
    let url_b = format!("{}/api/v1/setup/admin", base_url);

    let fut_a = tokio::spawn(async move {
        client_a
            .post(&url_a)
            .json(&json!({
                "username": "alice",
                "password": "alice-pw-12-chars-long",
            }))
            .send()
            .await
    });
    let fut_b = tokio::spawn(async move {
        client_b
            .post(&url_b)
            .json(&json!({
                "username": "bob",
                "password": "bob-pw-12-chars-long",
            }))
            .send()
            .await
    });

    let res_a = fut_a.await.expect("join A").expect("send A");
    let res_b = fut_b.await.expect("join B").expect("send B");

    // Exactement 1 admin : le verrou sentinelle garantit la sérialisation.
    let final_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(
        final_count,
        1,
        "le verrou sentinelle garantit exactement 1 admin (status A={}, B={})",
        res_a.status(),
        res_b.status()
    );

    // Le set des deux status == {200, 410}, ordre indifférent.
    let mut statuses = [res_a.status().as_u16(), res_b.status().as_u16()];
    statuses.sort_unstable();
    assert_eq!(
        statuses,
        [200, 410],
        "un succès (200) et un auto-disable (410), ordre indifférent"
    );
}
