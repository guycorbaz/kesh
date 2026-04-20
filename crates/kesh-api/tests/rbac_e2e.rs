//! Tests E2E pour le middleware RBAC (story 1.8).
//!
//! Vérifie la hiérarchie de rôles : Consultation < Comptable < Admin.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::routing::get;
use chrono::TimeDelta;
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::middleware::auth::CurrentUser;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::companies;
use serde_json::{Value, json};
use sqlx::MySqlPool;

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

/// Handler de test pour valider le rôle Comptable+.
async fn test_comptable_handler(
    axum::Extension(user): axum::Extension<CurrentUser>,
) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({
        "userId": user.user_id,
        "role": format!("{:?}", user.role),
    })))
}

/// Construit le routeur prod + route de test `_test/comptable` (Comptable+).
async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = std::sync::Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n: i18n.clone(),
    };

    // Route de test protégée par require_comptable_role (inner) + require_auth (outer)
    let test_comptable_router: Router<AppState> = Router::new()
        .route("/api/v1/_test/comptable", get(test_comptable_handler))
        .route_layer(axum::middleware::from_fn(
            kesh_api::middleware::rbac::require_comptable_role,
        ))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            kesh_api::middleware::auth::require_auth,
        ));

    let prod_router = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let app = prod_router.merge(test_comptable_router.with_state(state));

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

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("server not ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

/// Login et retourne le access_token.
async fn login_as(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login failed for {username}");
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}


/// Bootstrap admin, login, create user with role, login as that user.
async fn create_and_login_as(
    app: &TestApp,
    pool: &MySqlPool,
    username: &str,
    role: &str,
) -> String {
    let config = test_config();
    create_test_company(pool).await;
    ensure_admin_user(pool, &config).await.unwrap();
    let admin_token = login_as(app, "admin", TEST_ADMIN_PASSWORD).await;

    // Create user with specified role
    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&admin_token)
        .json(&json!({"username": username, "password": "secure-password-12chars", "role": role}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "create user {username} failed");

    // Login as the new user
    login_as(app, username, "secure-password-12chars").await
}

// === Unauthenticated access → 401 (not 403) ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unauthenticated_request_returns_401_not_403(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    // No bearer token → require_auth layer should return 401 before RBAC layer
    let resp = app
        .client
        .get(app.url("/api/v1/users"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// === T4.2 : AC#1 — Consultation bloqué sur /users/* ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_blocked_get_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;

    let resp = app
        .client
        .get(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_blocked_post_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;

    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .json(&json!({"username": "x", "password": "secure-password-12chars", "role": "Comptable"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_blocked_put_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;

    let resp = app
        .client
        .put(app.url("/api/v1/users/1"))
        .bearer_auth(&token)
        .json(&json!({"role": "Admin", "active": true, "version": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_blocked_disable(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;

    let resp = app
        .client
        .put(app.url("/api/v1/users/1/disable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === T4.3 : AC#2 — Comptable autorisé sur _test/comptable ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn comptable_allowed_on_comptable_route(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "accountant", "Comptable").await;

    let resp = app
        .client
        .get(app.url("/api/v1/_test/comptable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn admin_allowed_on_comptable_route(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let config = test_config();
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &config).await.unwrap();
    let token = login_as(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url("/api/v1/_test/comptable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_blocked_on_comptable_route(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;

    let resp = app
        .client
        .get(app.url("/api/v1/_test/comptable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === T4.4 : AC#3 — Comptable bloqué sur /users/* ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn comptable_blocked_post_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "accountant", "Comptable").await;

    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .json(&json!({"username": "x", "password": "secure-password-12chars", "role": "Consultation"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn comptable_blocked_get_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "accountant", "Comptable").await;

    let resp = app
        .client
        .get(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn comptable_blocked_reset_password(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = create_and_login_as(&app, &pool, "accountant", "Comptable").await;

    let resp = app
        .client
        .put(app.url("/api/v1/users/1/reset-password"))
        .bearer_auth(&token)
        .json(&json!({"newPassword": "new-secure-password-12"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === T4.5 : AC#4 — Admin accès complet ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn admin_can_create_and_list_users(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let config = test_config();
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &config).await.unwrap();
    let token = login_as(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Create
    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .json(&json!({"username": "alice", "password": "secure-password-12chars", "role": "Comptable"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // List
    let resp = app
        .client
        .get(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Comptable route
    let resp = app
        .client
        .get(app.url("/api/v1/_test/comptable"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// === T4.6 : AC#7 — auth/password accessible par tous les rôles ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn change_password_accessible_by_all_roles(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // Test with Consultation role
    let token = create_and_login_as(&app, &pool, "reader", "Consultation").await;
    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .bearer_auth(&token)
        .json(&json!({"currentPassword": "secure-password-12chars", "newPassword": "new-secure-pwd-12ch"}))
        .send()
        .await
        .unwrap();
    // 200 = success (password changed)
    assert_eq!(resp.status(), 200);

    // Test with Comptable role
    let token = create_and_login_as(&app, &pool, "accountant", "Comptable").await;
    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .bearer_auth(&token)
        .json(&json!({"currentPassword": "secure-password-12chars", "newPassword": "new-secure-pwd-12ch"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// === T5.2 : Renforcer assertion optimistic lock 409 ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn optimistic_lock_conflict_returns_correct_error_code(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let config = test_config();
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &config).await.unwrap();
    let token = login_as(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Create user
    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&token)
        .json(&json!({"username": "alice", "password": "secure-password-12chars", "role": "Comptable"}))
        .send()
        .await
        .unwrap();
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    // Send update with wrong version
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Consultation", "active": true, "version": 999}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
}
