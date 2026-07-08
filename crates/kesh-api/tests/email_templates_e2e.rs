//! Tests E2E HTTP pour Story 20-1 (Epic 20 #224) — socle CRUD Admin-only
//! `/api/v1/admin/email-templates`.
//!
//! Couvre : RBAC (Admin seul), round-trip CRUD, zéro-config (AC #16),
//! validation 422 (tokens inconnus), conflit 409 (version stale), restore
//! 204 puis fallback défaut, path params invalides → 400.

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

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
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
            Err(e) => panic!("test server did not become ready within 2s: {e}"),
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

// ---------------------------------------------------------------------------
// RBAC
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn rbac_only_admin_can_list(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "RBAC Co").await;

    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let comptable_id = create_user(&pool, "comptable1", Role::Comptable, company_id).await;
    let consultation_id = create_user(&pool, "consult1", Role::Consultation, company_id).await;

    let admin_token = forge_jwt(admin_id, "Admin", company_id);
    let comptable_token = forge_jwt(comptable_id, "Comptable", company_id);
    let consultation_token = forge_jwt(consultation_id, "Consultation", company_id);

    let admin_res = app
        .client
        .get(app.url("/api/v1/admin/email-templates"))
        .bearer_auth(&admin_token)
        .send()
        .await
        .unwrap();
    assert_eq!(admin_res.status(), 200);

    let comptable_res = app
        .client
        .get(app.url("/api/v1/admin/email-templates"))
        .bearer_auth(&comptable_token)
        .send()
        .await
        .unwrap();
    assert_eq!(comptable_res.status(), 403);

    let consultation_res = app
        .client
        .get(app.url("/api/v1/admin/email-templates"))
        .bearer_auth(&consultation_token)
        .send()
        .await
        .unwrap();
    assert_eq!(consultation_res.status(), 403);
}

// ---------------------------------------------------------------------------
// Zéro-config (AC #16)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn list_returns_four_defaults_for_fresh_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Fresh Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    let res = app
        .client
        .get(app.url("/api/v1/admin/email-templates"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Vec<Value> = res.json().await.unwrap();
    assert_eq!(body.len(), 4);
    assert!(body.iter().all(|t| t["isDefault"].as_bool().unwrap()));
    assert!(body.iter().all(|t| t["version"].is_null()));
}

// ---------------------------------------------------------------------------
// Round-trip CRUD
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn crud_round_trip_create_update_restore(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "CRUD Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    // Création (expectedVersion absent = None).
    let create_res = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({
            "subject": "Facture {invoiceNumber} personnalisée",
            "body": "{salutation}, montant {amount} échéance {dueDate}. {companyName}",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_res.status(), 200);
    let created: Value = create_res.json().await.unwrap();
    assert_eq!(created["version"].as_i64().unwrap(), 1);
    assert!(!created["isDefault"].as_bool().unwrap());

    // GET unique reflète l'override.
    let get_res = app
        .client
        .get(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(get_res.status(), 200);
    let fetched: Value = get_res.json().await.unwrap();
    assert_eq!(
        fetched["subject"].as_str().unwrap(),
        "Facture {invoiceNumber} personnalisée"
    );

    // Modification à la bonne version.
    let update_res = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({
            "subject": "Facture {invoiceNumber} v2",
            "body": "{salutation}, montant {amount}. {companyName}",
            "expectedVersion": 1,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(update_res.status(), 200);
    let updated: Value = update_res.json().await.unwrap();
    assert_eq!(updated["version"].as_i64().unwrap(), 2);

    // Restore default.
    let delete_res = app
        .client
        .delete(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(delete_res.status(), 204);

    let after_restore: Value = app
        .client
        .get(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(after_restore["isDefault"].as_bool().unwrap());

    // Restore idempotent (déjà sur le défaut) → toujours 204.
    let delete_again = app
        .client
        .delete(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(delete_again.status(), 204);
}

// ---------------------------------------------------------------------------
// Validation 422
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn put_rejects_unknown_variables_with_422(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Validation Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    let res = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({
            "subject": "Sujet {unknownToken}",
            "body": "Corps {alsoUnknown}",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "EMAIL_TEMPLATE_UNKNOWN_VARIABLES"
    );
    let unknown: Vec<String> = body["error"]["details"]["unknownVariables"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        unknown,
        vec!["unknownToken".to_string(), "alsoUnknown".to_string()]
    );
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn put_rejects_empty_subject_or_body(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Empty Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    let res = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({ "subject": "   ", "body": "Corps valide" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

// ---------------------------------------------------------------------------
// Conflit 409
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn put_stale_version_returns_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Conflict Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    // Création.
    app.client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({ "subject": "S1", "body": "B1 {amount}" }))
        .send()
        .await
        .unwrap();

    // expectedVersion=None alors qu'une ligne existe déjà → conflit.
    let create_again = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({ "subject": "S2", "body": "B2 {amount}" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create_again.status(), 409);

    // expectedVersion stale (2 au lieu de 1).
    let stale = app
        .client
        .put(app.url("/api/v1/admin/email-templates/invoice_send/FR"))
        .bearer_auth(&token)
        .json(&json!({ "subject": "S3", "body": "B3 {amount}", "expectedVersion": 2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(stale.status(), 409);
}

// ---------------------------------------------------------------------------
// Path params invalides
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn invalid_template_type_or_language_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "Invalid Path Co").await;
    let admin_id = create_user(&pool, "admin1", Role::Admin, company_id).await;
    let token = forge_jwt(admin_id, "Admin", company_id);

    let bad_type = app
        .client
        .get(app.url("/api/v1/admin/email-templates/unknown_type/FR"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(bad_type.status(), 400);

    let bad_language = app
        .client
        .get(app.url("/api/v1/admin/email-templates/invoice_send/fr"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(bad_language.status(), 400);
}
