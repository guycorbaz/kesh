//! Story 19-1 — tests d'intégration CRUD des projets analytiques.
//!
//! Couvre la pile complète (route → repo → DB) : CRUD, arbre 2 niveaux + garde de
//! hiérarchie, unicité `code`, verrou optimiste, archivage + filtre, auth.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
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

async fn login_as(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Crée un user d'un rôle donné dans une company (pour tester le gate RBAC).
async fn create_user(pool: &MySqlPool, company_id: i64, username: &str, role: Role) {
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("create user");
}

/// Crée une 2e company minimale (pour tester l'isolation cross-company). Retourne son id.
async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    let res = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, 'Adresse\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .bind(name)
    .execute(pool)
    .await
    .expect("company insert");
    res.last_insert_id() as i64
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
    // UniqueConstraintViolation → 409 Conflict (jamais 500).
    assert_eq!(r2.status(), 409);
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mutation_requires_comptable_role(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    // Un user Consultation dans la même company.
    create_user(&pool, seeded.company_id, "lecteur", Role::Consultation).await;
    let app = spawn_app(pool).await;
    let token = login_as(&app, "lecteur", "password123").await;

    // Lecture autorisée (tout rôle authentifié).
    let read = app
        .client
        .get(app.url("/api/v1/projects"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(read.status(), 200);

    // Mutation refusée (Comptable+ requis) → 403.
    let create = create_project(&app, &token, json!({ "code": "X", "name": "X" })).await;
    assert_eq!(create.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cross_company_project_is_not_found(pool: MySqlPool) {
    // Company A (seed) crée un projet ; company B ne doit pas y accéder (IDOR).
    seed_accounting_company(&pool).await.expect("seed");
    let company_b = create_company(&pool, "Company B").await;
    create_user(&pool, company_b, "userb", Role::Comptable).await;
    let app = spawn_app(pool).await;

    let token_a = login(&app).await;
    let created: serde_json::Value = create_project(
        &app,
        &token_a,
        json!({ "code": "SECRET", "name": "Projet A" }),
    )
    .await
    .json()
    .await
    .unwrap();
    let id_a = created["id"].as_i64().unwrap();

    let token_b = login_as(&app, "userb", "password123").await;
    // GET cross-company → 404 (pas de fuite d'existence).
    let get_b = app
        .client
        .get(app.url(&format!("/api/v1/projects/{id_a}")))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(get_b.status(), 404);
    // PUT cross-company → 404.
    let put_b = app
        .client
        .put(app.url(&format!("/api/v1/projects/{id_a}")))
        .bearer_auth(&token_b)
        .json(&json!({ "code": "SECRET", "name": "Hack", "version": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(put_b.status(), 404);
    // La company B ne voit pas le projet de A dans sa liste.
    let list_b: serde_json::Value = app
        .client
        .get(app.url("/api/v1/projects"))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(list_b.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn too_long_code_returns_400_not_500(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let long_code = "A".repeat(33); // > VARCHAR(32)
    let resp = create_project(&app, &token, json!({ "code": long_code, "name": "X" })).await;
    assert_eq!(resp.status(), 400, "attendu 400, eu {}", resp.status());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cannot_archive_root_with_active_children(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let root: serde_json::Value = create_project(&app, &token, json!({ "code": "R", "name": "R" }))
        .await
        .json()
        .await
        .unwrap();
    let root_id = root["id"].as_i64().unwrap();
    create_project(
        &app,
        &token,
        json!({ "code": "R-SUB", "name": "Sub", "parentId": root_id }),
    )
    .await;

    // Archiver la racine alors qu'un sous-projet actif existe → 400.
    let arch = app
        .client
        .post(app.url(&format!("/api/v1/projects/{root_id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": root["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(arch.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cannot_parent_under_archived_root(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    // Racine sans enfant → archivable.
    let root: serde_json::Value =
        create_project(&app, &token, json!({ "code": "AR", "name": "AR" }))
            .await
            .json()
            .await
            .unwrap();
    let root_id = root["id"].as_i64().unwrap();
    let arch = app
        .client
        .post(app.url(&format!("/api/v1/projects/{root_id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": root["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(arch.status(), 200);

    // Créer un sous-projet sous une racine archivée → 400.
    let sub = create_project(
        &app,
        &token,
        json!({ "code": "AR-SUB", "name": "Sub", "parentId": root_id }),
    )
    .await;
    assert_eq!(sub.status(), 400);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cannot_unarchive_sub_under_archived_parent(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    // Racine + sous-projet actifs.
    let root: serde_json::Value = create_project(&app, &token, json!({ "code": "U", "name": "U" }))
        .await
        .json()
        .await
        .unwrap();
    let root_id = root["id"].as_i64().unwrap();
    let sub: serde_json::Value = create_project(
        &app,
        &token,
        json!({ "code": "U-SUB", "name": "Sub", "parentId": root_id }),
    )
    .await
    .json()
    .await
    .unwrap();
    let sub_id = sub["id"].as_i64().unwrap();

    // Archiver le sous-projet d'abord (autorisé), puis la racine (plus d'enfant actif).
    let arch_sub: serde_json::Value = app
        .client
        .post(app.url(&format!("/api/v1/projects/{sub_id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": sub["version"] }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let sub_v = arch_sub["version"].as_i64().unwrap();
    let arch_root = app
        .client
        .post(app.url(&format!("/api/v1/projects/{root_id}/archive")))
        .bearer_auth(&token)
        .json(&json!({ "version": root["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(arch_root.status(), 200);

    // Désarchiver le sous-projet alors que sa racine est archivée → 400.
    let unarch = app
        .client
        .post(app.url(&format!("/api/v1/projects/{sub_id}/unarchive")))
        .bearer_auth(&token)
        .json(&json!({ "version": sub_v }))
        .send()
        .await
        .unwrap();
    assert_eq!(unarch.status(), 400);
}
