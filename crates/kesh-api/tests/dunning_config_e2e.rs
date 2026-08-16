//! E2E des routes de configuration des rappels (Story 21-3, #231) — RBAC, IDOR,
//! verrou optimiste, seed lazy au premier GET.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, users};
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

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// GET settings sur une company fraîche → seed lazy : 3 niveaux + grâce 5.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_settings_seeds_defaults(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Seed Co").await;
    let admin = create_user(&pool, "admin1", Role::Admin, cid).await;
    let token = forge_jwt(admin, "Admin", cid);

    let res = app
        .client
        .get(app.url("/api/v1/company/dunning-settings"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["gracePeriodDays"], 5);
    assert!(!body["seededAt"].is_null());

    // Les 3 niveaux par défaut sont visibles.
    let levels: Vec<Value> = app
        .client
        .get(app.url("/api/v1/dunning-levels"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(levels.len(), 3);
}

/// Créer un niveau : Admin OK, Comptable → 403.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_level_rbac(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "RBAC Co").await;
    let admin = create_user(&pool, "admin1", Role::Admin, cid).await;
    let comptable = create_user(&pool, "compt1", Role::Comptable, cid).await;

    let payload = json!({ "delayDays": 7, "feeAmount": "15.00" });

    let admin_res = app
        .client
        .post(app.url("/api/v1/dunning-levels"))
        .bearer_auth(forge_jwt(admin, "Admin", cid))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(admin_res.status(), 201);

    let comptable_res = app
        .client
        .post(app.url("/api/v1/dunning-levels"))
        .bearer_auth(forge_jwt(comptable, "Comptable", cid))
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(comptable_res.status(), 403);
}

/// PUT settings avec version périmée → 409.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn update_settings_stale_version_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid = create_company(&pool, "Conflict Co").await;
    let admin = create_user(&pool, "admin1", Role::Admin, cid).await;
    let token = forge_jwt(admin, "Admin", cid);

    // 1er GET seede + pose version.
    app.client
        .get(app.url("/api/v1/company/dunning-settings"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .put(app.url("/api/v1/company/dunning-settings"))
        .bearer_auth(&token)
        .json(&json!({ "gracePeriodDays": 10, "version": 999 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
}

/// IDOR : modifier un niveau d'une autre company → 404 (jamais 403, anti-énumération).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn update_level_cross_tenant_404(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let cid_a = create_company(&pool, "Co A").await;
    let cid_b = create_company(&pool, "Co B").await;
    let admin_a = create_user(&pool, "admin_a", Role::Admin, cid_a).await;
    let admin_b = create_user(&pool, "admin_b", Role::Admin, cid_b).await;

    // A crée un niveau.
    let created: Value = app
        .client
        .post(app.url("/api/v1/dunning-levels"))
        .bearer_auth(forge_jwt(admin_a, "Admin", cid_a))
        .json(&json!({ "delayDays": 7, "feeAmount": "15.00" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let level_id = created["id"].as_i64().unwrap();

    // B tente de le modifier → 404 (scopé company_id, pas 403).
    let res = app
        .client
        .put(app.url(&format!("/api/v1/dunning-levels/{level_id}")))
        .bearer_auth(forge_jwt(admin_b, "Admin", cid_b))
        .json(&json!({ "delayDays": 99, "feeAmount": "0.00", "version": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}
