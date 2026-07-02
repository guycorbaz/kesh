//! Story 19-1 — tests d'intégration CRUD des projets analytiques.
//!
//! Couvre la pile complète (route → repo → DB) : CRUD, arbre 2 niveaux + garde de
//! hiérarchie, unicité `code`, verrou optimiste, archivage + filtre, auth.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::seed_accounting_company;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

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
    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
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
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn create_project(app: &TestApp, token: &str, body: serde_json::Value) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/projects"))
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn crud_tree_2_levels_and_hierarchy_guard(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    // Racine.
    let resp = create_project(
        &app,
        &token,
        json!({ "code": "RENOV", "name": "Rénovation chalet" }),
    )
    .await;
    assert_eq!(resp.status(), 201);
    let root: serde_json::Value = resp.json().await.unwrap();
    let root_id = root["id"].as_i64().unwrap();
    assert!(root["parentId"].is_null());
    assert_eq!(root["archived"], false);

    // Sous-projet.
    let resp = create_project(
        &app,
        &token,
        json!({ "code": "RENOV-TOIT", "name": "Toiture", "parentId": root_id }),
    )
    .await;
    assert_eq!(resp.status(), 201);
    let sub: serde_json::Value = resp.json().await.unwrap();
    let sub_id = sub["id"].as_i64().unwrap();
    assert_eq!(sub["parentId"], root_id);

    // Garde 2 niveaux : sous-projet d'un sous-projet → 400.
    let resp = create_project(
        &app,
        &token,
        json!({ "code": "RENOV-TOIT-X", "name": "X", "parentId": sub_id }),
    )
    .await;
    assert_eq!(resp.status(), 400);

    // Liste : racine avant sous-projet (tri arbre).
    let list: serde_json::Value = app
        .client
        .get(app.url("/api/v1/projects"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let arr = list.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], root_id);
    assert_eq!(arr[1]["id"], sub_id);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn duplicate_code_is_rejected(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let r1 = create_project(&app, &token, json!({ "code": "DUP", "name": "A" })).await;
    assert_eq!(r1.status(), 201);
    let r2 = create_project(&app, &token, json!({ "code": "DUP", "name": "B" })).await;
    // UniqueConstraintViolation → 4xx (jamais 500).
    assert!(
        r2.status().is_client_error(),
        "attendu 4xx, eu {}",
        r2.status()
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_uses_optimistic_lock(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let created: serde_json::Value =
        create_project(&app, &token, json!({ "code": "P", "name": "Nom 1" }))
            .await
            .json()
            .await
            .unwrap();
    let id = created["id"].as_i64().unwrap();
    let version = created["version"].as_i64().unwrap();

    // Update avec la bonne version → 200.
    let ok = app
        .client
        .put(app.url(&format!("/api/v1/projects/{id}")))
        .bearer_auth(&token)
        .json(&json!({ "code": "P", "name": "Nom 2", "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 200);

    // Re-update avec la version périmée → 409.
    let stale = app
        .client
        .put(app.url(&format!("/api/v1/projects/{id}")))
        .bearer_auth(&token)
        .json(&json!({ "code": "P", "name": "Nom 3", "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), 409);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn archive_hides_from_default_list(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let created: serde_json::Value = create_project(
        &app,
        &token,
        json!({ "code": "ARCH", "name": "À archiver" }),
    )
    .await
    .json()
    .await
    .unwrap();
    let id = created["id"].as_i64().unwrap();
    let version = created["version"].as_i64().unwrap();

    let arch = app
        .client
        .post(app.url(&format!("/api/v1/projects/{id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(arch.status(), 200);

    // Liste par défaut : exclut l'archivé.
    let list: serde_json::Value = app
        .client
        .get(app.url("/api/v1/projects"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list.as_array().unwrap().len(), 0);

    // includeArchived=true : présent.
    let list_all: serde_json::Value = app
        .client
        .get(app.url("/api/v1/projects?includeArchived=true"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_all.as_array().unwrap().len(), 1);
    assert_eq!(list_all[0]["archived"], true);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn requires_authentication(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/projects"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    let resp = create_project(&app, "", json!({ "code": "X", "name": "X" })).await;
    assert_eq!(resp.status(), 401);
}
