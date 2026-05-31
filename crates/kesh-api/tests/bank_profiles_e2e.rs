//! Tests E2E HTTP pour Story 8-2 T4 — CRUD bank_profiles.
//!
//! 13 tests : create + audit log, validation (XOR, séparateurs, regex,
//! chrono, indices collision), RBAC (Consultation refusée write,
//! authorisée read), multi-tenant (404 cross-tenant), UNIQUE bank_name,
//! delete avec snapshot, update full replacement.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
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
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
        users_exist: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
    };

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
            address: "Rue Test 1".into(),
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
        },
    )
    .await
    .unwrap()
    .id
}

fn valid_profile_payload(bank_name: &str) -> Value {
    json!({
        "bankName": bank_name,
        "filenamePattern": r"^export-\d+\.csv$",
        "columnMapping": {
            "date": 0,
            "amount": 1,
            "reference": 2,
            "details": 3
        },
        "dateFormat": "%Y-%m-%d",
        "decimalSeparator": ".",
        "fieldSeparator": ";",
        "encoding": null,
        "headerRowCount": 1
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_creates_with_audit_log(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 201);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["bankName"].as_str().unwrap(), "UBS");

    let audit_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE entity_type='bank_profiles'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(audit_count, 1);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_xor_violation(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["columnMapping"]["debit_credit_split"] = json!([4, 5]);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    let status = res.status();
    let body: Value = res.json().await.unwrap();
    assert_eq!(status, 422, "expected 422, got {} body={}", status, body);
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "BANK_CSV_PROFILE_INVALID"
    );
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_equal_separators(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["fieldSeparator"] = json!(",");
    payload["decimalSeparator"] = json!(",");

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_column_mapping_collision(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["columnMapping"]["date"] = json!(1);
    payload["columnMapping"]["amount"] = json!(1);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_invalid_chrono_format(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["dateFormat"] = json!("%Q"); // %Q invalide chrono

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_invalid_regex(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["filenamePattern"] = json!("(unclosed");

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_pattern_over_200_chars(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let mut payload = valid_profile_payload("UBS");
    payload["filenamePattern"] = json!("a".repeat(201));

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&payload)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 422);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn post_bank_profile_rejects_consultation_role(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "consult", Role::Consultation, company_id).await;
    let token = forge_jwt(user_id, "Consultation", company_id);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 403);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn get_bank_profile_allowed_for_consultation_role(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let comptable_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let consult_id = create_user(&pool, "consult", Role::Consultation, company_id).await;

    // Crée un profil via comptable
    let comp_token = forge_jwt(comptable_id, "Comptable", company_id);
    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&comp_token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    // Lecture accessible à Consultation
    let consult_token = forge_jwt(consult_id, "Consultation", company_id);
    let res = app
        .client
        .get(app.url(&format!("/api/v1/bank-profiles/{}", id)))
        .bearer_auth(&consult_token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn get_bank_profile_returns_404_for_other_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_a = create_company(&pool, "A").await;
    let company_b = create_company(&pool, "B").await;
    let user_a = create_user(&pool, "a", Role::Comptable, company_a).await;
    let user_b = create_user(&pool, "b", Role::Comptable, company_b).await;

    // company_a crée un profil
    let token_a = forge_jwt(user_a, "Comptable", company_a);
    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token_a)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    // company_b reçoit 404 sur le même id (pas 403)
    let token_b = forge_jwt(user_b, "Comptable", company_b);
    let res = app
        .client
        .get(app.url(&format!("/api/v1/bank-profiles/{}", id)))
        .bearer_auth(&token_b)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn list_bank_profiles_only_returns_own_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_a = create_company(&pool, "A").await;
    let company_b = create_company(&pool, "B").await;
    let user_a = create_user(&pool, "a", Role::Comptable, company_a).await;
    let user_b = create_user(&pool, "b", Role::Comptable, company_b).await;

    let token_a = forge_jwt(user_a, "Comptable", company_a);
    app.client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token_a)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token_a)
        .json(&valid_profile_payload("Raiffeisen"))
        .send()
        .await
        .unwrap();

    let token_b = forge_jwt(user_b, "Comptable", company_b);
    app.client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token_b)
        .json(&valid_profile_payload("PostFinance"))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .get(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token_a)
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["total"].as_i64().unwrap(), 2);
    let names: Vec<String> = body["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["bankName"].as_str().unwrap().to_string())
        .collect();
    assert!(names.contains(&"UBS".to_string()));
    assert!(!names.contains(&"PostFinance".to_string()));
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn duplicate_bank_name_within_company_returns_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    app.client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 409);
    let body: Value = res.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "BANK_CSV_PROFILE_DUPLICATE"
    );
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn put_bank_profile_full_replacement_writes_audit_log(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let mut updated = valid_profile_payload("UBS");
    updated["dateFormat"] = json!("%d.%m.%Y");
    updated["headerRowCount"] = json!(2);

    let res = app
        .client
        .put(app.url(&format!("/api/v1/bank-profiles/{}", id)))
        .bearer_auth(&token)
        .json(&updated)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    assert_eq!(body["dateFormat"].as_str().unwrap(), "%d.%m.%Y");
    assert_eq!(body["headerRowCount"].as_i64().unwrap(), 2);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type='bank_profiles' AND action='bank_profile.updated'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[sqlx::test(migrations = "../kesh-db/migrations")]
async fn delete_bank_profile_writes_audit_log_with_snapshot(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "C1").await;
    let user_id = create_user(&pool, "comp", Role::Comptable, company_id).await;
    let token = forge_jwt(user_id, "Comptable", company_id);

    let res = app
        .client
        .post(app.url("/api/v1/bank-profiles"))
        .bearer_auth(&token)
        .json(&valid_profile_payload("UBS"))
        .send()
        .await
        .unwrap();
    let created: Value = res.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let res = app
        .client
        .delete(app.url(&format!("/api/v1/bank-profiles/{}", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // Snapshot UBS conservé en audit log post-delete
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE entity_type='bank_profiles' AND action='bank_profile.deleted' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["bank_name"].as_str().unwrap(), "UBS");

    // Profil bien supprimé
    let res = app
        .client
        .get(app.url(&format!("/api/v1/bank-profiles/{}", id)))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}
