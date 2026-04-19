//! Tests d'intégration E2E pour la gestion des utilisateurs (story 1.7).
//!
//! Pattern identique à `auth_e2e.rs` : `spawn_app` sur port éphémère,
//! DB fraîche via `#[sqlx::test]`.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
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

fn test_config_with_min_password(min_len: u32) -> Config {
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
        min_len,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    spawn_app_with_config(pool, test_config()).await
}

async fn spawn_app_with_config(pool: MySqlPool, config: Config) -> TestApp {
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

async fn create_test_company(pool: &MySqlPool) {
    companies::create(
        pool,
        NewCompany {
            name: "Test Company".into(),
            address: "Test Address".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("create test company");
}

/// Bootstrappe l'admin puis retourne un access_token Admin.
async fn login_admin(app: &TestApp, pool: &MySqlPool) -> String {
    let config = test_config();
    create_test_company(pool).await;
    ensure_admin_user(pool, &config).await.expect("bootstrap");

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "admin", "password": TEST_ADMIN_PASSWORD}))
        .send()
        .await
        .expect("login");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Crée un utilisateur via l'API et retourne le body JSON.
async fn create_user_api(
    app: &TestApp,
    token: &str,
    username: &str,
    password: &str,
    role: &str,
) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/users"))
        .bearer_auth(token)
        .json(&json!({"username": username, "password": password, "role": role}))
        .send()
        .await
        .expect("create user")
}

/// Login avec un username/password donné.
async fn login_as(app: &TestApp, username: &str, password: &str) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login")
}

// === T7.2 : Tests création ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_success(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    assert_eq!(resp.status(), 201);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "alice");
    assert_eq!(body["role"], "Comptable");
    assert_eq!(body["active"], true);
    assert!(body["version"].as_i64().is_some());
    assert!(body["id"].as_i64().is_some());
    // password_hash must never appear
    assert!(body.get("passwordHash").is_none());
    assert!(body.get("password_hash").is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_invalid_role_returns_422(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(&app, &token, "bob", "secure-password-12chars", "SuperAdmin").await;
    assert_eq!(resp.status(), 422);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_duplicate_username_returns_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    assert_eq!(resp.status(), 201);

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "another-password-12ch",
        "Consultation",
    )
    .await;
    assert_eq!(resp.status(), 409);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_short_password_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(&app, &token, "alice", "short", "Comptable").await;
    assert_eq!(resp.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_empty_username_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(&app, &token, "", "secure-password-12chars", "Comptable").await;
    assert_eq!(resp.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_whitespace_username_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(&app, &token, "   ", "secure-password-12chars", "Comptable").await;
    assert_eq!(resp.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_user_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create a Comptable user
    let resp = create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    assert_eq!(resp.status(), 201);

    // Login as Comptable
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    // Try to create user with Comptable token
    let resp = create_user_api(
        &app,
        comptable_token,
        "alice",
        "secure-password-12chars",
        "Consultation",
    )
    .await;
    assert_eq!(resp.status(), 403);
}

// === T7.3 : Tests modification ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_change_role(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();
    let version = user["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Consultation", "active": true, "version": version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["role"], "Consultation");
    assert!(body["version"].as_i64().unwrap() > version);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_reactivate(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create + disable
    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/disable", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let disabled: Value = resp.json().await.unwrap();
    let version = disabled["version"].as_i64().unwrap();

    // Reactivate via update
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Comptable", "active": true, "version": version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["active"], true);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_version_conflict_returns_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    // Use wrong version
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Consultation", "active": true, "version": 999}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
}

// === P2 : Tests guards update_user (self-disable, last-admin, demotion) ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_self_disable_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Get admin's own ID
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=1"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let admin_id = list["items"][0]["id"].as_i64().unwrap();
    let version = list["items"][0]["version"].as_i64().unwrap();

    // Try to deactivate self via PUT /users/:id
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", admin_id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Admin", "active": false, "version": version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CANNOT_DISABLE_SELF");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_deactivate_last_admin_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create a second admin, then get their ID
    let resp = create_user_api(&app, &token, "admin2", "secure-password-12chars", "Admin").await;
    let admin2: Value = resp.json().await.unwrap();
    let admin2_id = admin2["id"].as_i64().unwrap();

    // Login as admin2
    let resp = login_as(&app, "admin2", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let admin2_token = body["accessToken"].as_str().unwrap().to_string();

    // Disable admin2 via /disable first (leaves only original admin)
    app.client
        .put(app.url(&format!("/api/v1/users/{}/disable", admin2_id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    // admin2's stale JWT: try to deactivate original admin via PUT /users/:id
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10"))
        .bearer_auth(&admin2_token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let admin1 = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "admin")
        .unwrap();
    let admin1_id = admin1["id"].as_i64().unwrap();
    let admin1_version = admin1["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", admin1_id)))
        .bearer_auth(&admin2_token)
        .json(&json!({"role": "Admin", "active": false, "version": admin1_version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CANNOT_DISABLE_LAST_ADMIN");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_demote_last_admin_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create a second admin
    let resp = create_user_api(&app, &token, "admin2", "secure-password-12chars", "Admin").await;
    let admin2: Value = resp.json().await.unwrap();
    let admin2_id = admin2["id"].as_i64().unwrap();
    let admin2_version = admin2["version"].as_i64().unwrap();

    // Demote admin2 to Comptable (OK — original admin is still Admin)
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", admin2_id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Comptable", "active": true, "version": admin2_version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Now try to demote the last admin (original) — should fail
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let admin1 = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "admin")
        .unwrap();
    let admin1_id = admin1["id"].as_i64().unwrap();
    let admin1_version = admin1["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", admin1_id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Comptable", "active": true, "version": admin1_version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CANNOT_DISABLE_LAST_ADMIN");
}

// === P3 : Test session revocation via update_user deactivation ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_deactivate_revokes_sessions(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create user and get their refresh token
    create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "alice", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let alice_refresh = body["refreshToken"].as_str().unwrap().to_string();

    // Get alice's data
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let alice = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "alice")
        .unwrap();
    let alice_id = alice["id"].as_i64().unwrap();
    let alice_version = alice["version"].as_i64().unwrap();

    // Deactivate alice via PUT /users/:id (not /disable)
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", alice_id)))
        .bearer_auth(&token)
        .json(&json!({"role": "Comptable", "active": false, "version": alice_version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Alice's refresh token should be revoked
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": alice_refresh}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// === T7.4 : Tests liste ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_users_paginated(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create 2 users (admin already exists = 3 total)
    create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    create_user_api(
        &app,
        &token,
        "bob",
        "secure-password-12chars",
        "Consultation",
    )
    .await;

    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10&offset=0"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 3);
    assert_eq!(body["items"].as_array().unwrap().len(), 3);
    assert_eq!(body["offset"], 0);
    assert_eq!(body["limit"], 10);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_users_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/users"))
        .bearer_auth(comptable_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === T7.5 : Tests détail ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_user_success(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/users/{}", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["username"], "alice");
    assert!(body.get("passwordHash").is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_user_not_found_returns_404(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/users/99999"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// === T7.6 : Tests désactivation ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn disable_user_success(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/disable", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["active"], false);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn disable_user_self_disable_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Get admin user ID (the one we logged in as)
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=1&offset=0"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let admin_id = body["items"][0]["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/disable", admin_id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CANNOT_DISABLE_SELF");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn disable_last_admin_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin1_token = login_admin(&app, &pool).await;

    // Create admin2 and get their JWT
    let resp = create_user_api(
        &app,
        &admin1_token,
        "admin2",
        "secure-password-12chars",
        "Admin",
    )
    .await;
    assert_eq!(resp.status(), 201);
    let admin2: Value = resp.json().await.unwrap();
    let admin2_id = admin2["id"].as_i64().unwrap();
    let admin2_version = admin2["version"].as_i64().unwrap();

    let resp = login_as(&app, "admin2", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let admin2_token = body["accessToken"].as_str().unwrap().to_string();

    // admin1 demotes admin2 to Comptable (admin2's JWT still says Admin)
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}", admin2_id)))
        .bearer_auth(&admin1_token)
        .json(&json!({"role": "Comptable", "active": true, "version": admin2_version}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Now only admin1 is Admin (count_active_by_role = 1).
    // admin2 uses their stale JWT (still says Admin) to try to disable admin1.
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10&offset=0"))
        .bearer_auth(&admin2_token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let admin1_id = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "admin")
        .unwrap()["id"]
        .as_i64()
        .unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/disable", admin1_id)))
        .bearer_auth(&admin2_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CANNOT_DISABLE_LAST_ADMIN");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn disable_user_login_impossible_after(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create + disable
    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    app.client
        .put(app.url(&format!("/api/v1/users/{}/disable", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    // Login as disabled user should fail
    let resp = login_as(&app, "alice", "secure-password-12chars").await;
    assert_eq!(resp.status(), 401);
}

// === T7.7 : Tests reset password ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_password_success(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();
    let version_before = user["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/reset-password", id)))
        .bearer_auth(&token)
        .json(&json!({"newPassword": "new-secure-password-12"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["version"].as_i64().unwrap() > version_before);

    // Login with new password should succeed
    let resp = login_as(&app, "alice", "new-secure-password-12").await;
    assert_eq!(resp.status(), 200);

    // Login with old password should fail
    let resp = login_as(&app, "alice", "secure-password-12chars").await;
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_password_short_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    let resp = create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/reset-password", id)))
        .bearer_auth(&token)
        .json(&json!({"newPassword": "short"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_password_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/users/1/reset-password"))
        .bearer_auth(comptable_token)
        .json(&json!({"newPassword": "new-secure-password-12"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === F6 : Tests 403 manquants (update, disable, get non-admin) ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_user_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/users/1"))
        .bearer_auth(comptable_token)
        .json(&json!({"role": "Admin", "active": true, "version": 1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn disable_user_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/users/1/disable"))
        .bearer_auth(comptable_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_user_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    create_user_api(
        &app,
        &token,
        "comptable1",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "comptable1", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let comptable_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/users/1"))
        .bearer_auth(comptable_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// === F7 : Test sessions invalidées après reset_password ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_password_revokes_sessions(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let token = login_admin(&app, &pool).await;

    // Create user and get their refresh_token
    create_user_api(
        &app,
        &token,
        "alice",
        "secure-password-12chars",
        "Comptable",
    )
    .await;
    let resp = login_as(&app, "alice", "secure-password-12chars").await;
    let body: Value = resp.json().await.unwrap();
    let alice_refresh = body["refreshToken"].as_str().unwrap().to_string();

    // Get alice's ID
    let resp = app
        .client
        .get(app.url("/api/v1/users?limit=10&offset=0"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    let list: Value = resp.json().await.unwrap();
    let alice_id = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|u| u["username"] == "alice")
        .unwrap()["id"]
        .as_i64()
        .unwrap();

    // Admin resets alice's password
    app.client
        .put(app.url(&format!("/api/v1/users/{}/reset-password", alice_id)))
        .bearer_auth(&token)
        .json(&json!({"newPassword": "new-secure-password-12"}))
        .send()
        .await
        .unwrap();

    // Alice's old refresh token should be revoked
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": alice_refresh}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// === T7.8 : Test politique configurable ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn configurable_password_policy(pool: MySqlPool) {
    let config = test_config_with_min_password(20);
    // Bootstrap with standard config (admin password fits default min_length=12)
    let bootstrap_config = test_config();
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &bootstrap_config).await.unwrap();

    let app = spawn_app_with_config(pool.clone(), config).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "admin", "password": TEST_ADMIN_PASSWORD}))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let token = body["accessToken"].as_str().unwrap();

    // Create with 15-char password should fail (min is 20)
    let resp = create_user_api(&app, token, "alice", "fifteen-chars!!", "Comptable").await;
    assert_eq!(resp.status(), 400);

    // Reset with 15-char password should also fail
    // First create a user with a long enough password
    let resp = create_user_api(&app, token, "bob", "twenty-char-password!!", "Comptable").await;
    assert_eq!(resp.status(), 201);
    let user: Value = resp.json().await.unwrap();
    let id = user["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/reset-password", id)))
        .bearer_auth(token)
        .json(&json!({"newPassword": "fifteen-chars!!"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);

    // Change password with 15-char password should also fail
    let resp = login_as(&app, "bob", "twenty-char-password!!").await;
    let body: Value = resp.json().await.unwrap();
    let bob_token = body["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .bearer_auth(bob_token)
        .json(
            &json!({"currentPassword": "twenty-char-password!!", "newPassword": "fifteen-chars!!"}),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}
