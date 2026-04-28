//! Tests E2E API — Story 3.7 Gestion des exercices comptables.

mod common;

use std::sync::Arc;

use chrono::{Datelike, TimeDelta};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::repositories::audit_log;
use serde_json::json;
use sqlx::MySqlPool;
use std::net::SocketAddr;

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
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

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

/// Setup minimal : company + admin user + login → token Comptable+ (Admin).
async fn bootstrap_admin(pool: &MySqlPool) -> (TestApp, String) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    (app, token)
}

/// Crée un user `Consultation` (lecture seule, pas Comptable+) pour les tests RBAC.
async fn create_consultation_user_and_login(app: &TestApp, pool: &MySqlPool) -> String {
    use kesh_db::entities::{NewUser, Role};

    // Le mot de passe doit être hashé Argon2id par auth::login. Pour simplifier,
    // on insère un hash factice et on bypass via JWT direct. Plus simple :
    // utiliser le module auth comme bootstrap mais avec rôle Consultation.
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();

    // Hash réel d'un mot de passe connu.
    let password_plain = "consultation-test-pw-12345";
    let hash = kesh_api::auth::password::hash_password(password_plain).expect("hash");

    kesh_db::repositories::users::create(
        pool,
        NewUser {
            username: "consultation".into(),
            password_hash: hash,
            role: Role::Consultation,
            active: true,
            company_id,
        },
    )
    .await
    .expect("create consultation user");

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "consultation", "password": password_plain }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "consultation login should succeed");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

// ===========================================================================
// CREATE — happy path + multi-tenant defense + erreurs
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_happy_path(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["id"].as_i64().is_some());
    assert_eq!(body["name"], "Exercice 2027");
    assert_eq!(body["status"], "Open");
    assert_eq!(body["startDate"], "2027-01-01");
    assert_eq!(body["endDate"], "2027-12-31");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_injected_company_id_ignored(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    // Récupérer le company_id de l'admin pour comparer.
    let admin_company_id: i64 =
        sqlx::query_scalar("SELECT company_id FROM users WHERE username = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31",
            // Tentative d'injection : doit être ignoré par le backend.
            "companyId": 999
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["companyId"], admin_company_id);

    // Code Review Pass 1 F13 — vérification DB directe en plus du body HTTP :
    // s'assurer que le `company_id` stocké correspond à celui du JWT, pas au
    // 999 injecté dans le payload.
    let new_id: i64 = body["id"].as_i64().expect("id in response");
    let stored_company_id: i64 =
        sqlx::query_scalar("SELECT company_id FROM fiscal_years WHERE id = ?")
            .bind(new_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        stored_company_id, admin_company_id,
        "stored company_id must equal JWT company_id, not payload-injected value"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_overlap(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    // Pré-insérer Exercice 2027 (Jan-Dec).
    app.client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();

    // Tentative Mid 2027 → overlap.
    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Mid 2027",
            "startDate": "2027-07-01",
            "endDate": "2028-06-30"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("chevauche") || msg.contains("overlap"),
        "expected overlap message, got: {msg}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_duplicate_name(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    app.client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2028-01-01",
            "endDate": "2028-12-31"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("nom") || msg.contains("name") || msg.contains("existe"),
        "expected duplicate-name message, got: {msg}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_dates_invalid(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Bad",
            "startDate": "2027-12-31",
            "endDate": "2027-01-01"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

// ===========================================================================
// LIST + GET
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_empty(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_array());
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_populated_desc_order(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    for year in [2025, 2026, 2027] {
        app.client
            .post(app.url("/api/v1/fiscal-years"))
            .header("Authorization", auth(&token))
            .json(&json!({
                "name": format!("Exercice {year}"),
                "startDate": format!("{year}-01-01"),
                "endDate": format!("{year}-12-31"),
            }))
            .send()
            .await
            .unwrap();
    }

    let resp = app
        .client
        .get(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    // DESC: le plus récent en tête.
    assert_eq!(arr[0]["name"], "Exercice 2027");
    assert_eq!(arr[1]["name"], "Exercice 2026");
    assert_eq!(arr[2]["name"], "Exercice 2025");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_by_id_happy(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();
    let created: serde_json::Value = create.json().await.unwrap();
    let id = created["id"].as_i64().unwrap();

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/fiscal-years/{id}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "Exercice 2027");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_by_id_missing(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/fiscal-years/9999"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_by_id_other_company_returns_404(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    // Créer une 2e company + un fiscal_year direct DB pour cette company.
    use kesh_db::entities::{Language, NewCompany, NewFiscalYear, OrgType};
    let other_company = kesh_db::repositories::companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
            address: "Other Address".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();

    let other_fy = kesh_db::repositories::fiscal_years::create_for_seed(
        &pool,
        NewFiscalYear {
            company_id: other_company.id,
            name: "Other FY 2027".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/fiscal-years/{}", other_fy.id)))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "anti-énumération : 404 et pas 403");
}

// ===========================================================================
// UPDATE NAME
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_name_happy(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/fiscal-years/{id}")))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "FY 2027" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["name"], "FY 2027");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_name_duplicate(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    app.client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();

    let create_2028: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2028",
            "startDate": "2028-01-01",
            "endDate": "2028-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id_2028 = create_2028["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/fiscal-years/{id_2028}")))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Exercice 2027" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_name_other_company_returns_404(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    use kesh_db::entities::{Language, NewCompany, NewFiscalYear, OrgType};
    let other_company = kesh_db::repositories::companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
            address: "Other Address".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();

    let other_fy = kesh_db::repositories::fiscal_years::create_for_seed(
        &pool,
        NewFiscalYear {
            company_id: other_company.id,
            name: "Other FY 2027".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/fiscal-years/{}", other_fy.id)))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "Hijacked" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_name_empty(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/fiscal-years/{id}")))
        .header("Authorization", auth(&token))
        .json(&json!({ "name": "   " }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

// ===========================================================================
// CLOSE
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_happy(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/fiscal-years/{id}/close")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["status"], "Closed");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_already_closed(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    // Premier close OK.
    app.client
        .post(app.url(&format!("/api/v1/fiscal-years/{id}/close")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    // Second close → 409 ILLEGAL_STATE_TRANSITION.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/fiscal-years/{id}/close")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_other_company_returns_404(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    use kesh_db::entities::{Language, NewCompany, NewFiscalYear, OrgType};
    let other_company = kesh_db::repositories::companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
            address: "Other Address".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();

    let other_fy = kesh_db::repositories::fiscal_years::create_for_seed(
        &pool,
        NewFiscalYear {
            company_id: other_company.id,
            name: "Other FY 2027".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/fiscal-years/{}/close", other_fy.id)))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// ===========================================================================
// RBAC — Consultation et auth
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rbac_post_create_consultation_returns_403(pool: MySqlPool) {
    let (app, _admin_token) = bootstrap_admin(&pool).await;
    let consultation_token = create_consultation_user_and_login(&app, &pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&consultation_token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rbac_put_update_consultation_returns_403(pool: MySqlPool) {
    let (app, admin_token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&admin_token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let consultation_token = create_consultation_user_and_login(&app, &pool).await;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/fiscal-years/{id}")))
        .header("Authorization", auth(&consultation_token))
        .json(&json!({ "name": "Hijacked" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rbac_post_close_consultation_returns_403(pool: MySqlPool) {
    let (app, admin_token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&admin_token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let consultation_token = create_consultation_user_and_login(&app, &pool).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/fiscal-years/{id}/close")))
        .header("Authorization", auth(&consultation_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rbac_get_list_no_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/fiscal-years"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ===========================================================================
// AC #23 — DELETE non supporté
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_fiscal_year_returns_405(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/fiscal-years/{id}")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 405);
}

// ===========================================================================
// AUDIT LOG — création scopée company
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_writes_audit_log(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let create: serde_json::Value = app
        .client
        .post(app.url("/api/v1/fiscal-years"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "name": "Exercice 2027",
            "startDate": "2027-01-01",
            "endDate": "2027-12-31"
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = create["id"].as_i64().unwrap();

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", id, 10)
        .await
        .unwrap();
    assert!(entries.iter().any(|e| e.action == "fiscal_year.created"));
}

// ===========================================================================
// PATH B FINALIZE — auto-create fiscal_year
// ===========================================================================

async fn advance_to_step_2_path_b(app: &TestApp, token: &str) {
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();
}

async fn run_path_b_until_finalize(app: &TestApp, token: &str) {
    advance_to_step_2_path_b(app, token).await;
    app.client
        .post(app.url("/api/v1/onboarding/start-production"))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/org-type"))
        .header("Authorization", auth(token))
        .json(&json!({ "orgType": "Pme" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/accounting-language"))
        .header("Authorization", auth(token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/coordinates"))
        .header("Authorization", auth(token))
        .json(&json!({
            "name": "Ma SARL",
            "address": "Rue Test 1, 1000 Lausanne",
            "ideNumber": "CHE-109.322.551"
        }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/skip-bank"))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
}

/// Pré-requis Path B finalize : `insert_with_defaults_in_tx` exige les
/// comptes 1100 et 3000 pour pré-remplir les `company_invoice_settings`.
/// On les insère directement en SQL pour éviter de dépendre du chart loader
/// (la route `accounts::create` requiert un user_id et passe par audit log
/// — superflu pour un setup de test).
async fn seed_minimal_chart(pool: &MySqlPool, company_id: i64) {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active) \
         VALUES (?, '1100', 'Créances clients', 'Asset', TRUE)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("seed account 1100");
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active) \
         VALUES (?, '3000', 'Ventes', 'Revenue', TRUE)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("seed account 3000");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn path_b_finalize_creates_fiscal_year(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    seed_minimal_chart(&pool, company_id).await;

    run_path_b_until_finalize(&app, &token).await;
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/finalize"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Vérifier qu'un fiscal_year a été créé pour l'année courante.
    let list = kesh_db::repositories::fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    let year = chrono::Utc::now()
        .naive_utc()
        .date()
        .format("%Y")
        .to_string();
    assert_eq!(list[0].name, format!("Exercice {year}"));

    // AC #18 — audit log présent avec user_id = admin.
    let entries = audit_log::find_by_entity(&pool, "fiscal_year", list[0].id, 10)
        .await
        .unwrap();
    let admin_user_id: i64 =
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'admin' LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    let create_entry = entries
        .iter()
        .find(|e| e.action == "fiscal_year.created")
        .expect("audit entry present");
    assert_eq!(create_entry.user_id, admin_user_id);
}

/// Story 7.2 (KF-003) — vérifie que `finalize` Path B seed les 4 taux TVA
/// suisses 2024+ pour la nouvelle company, dans la même tx que le reste du
/// finalize. Test exécute le full Path B + finalize, puis lit `vat_rates`
/// directement et via `GET /api/v1/vat-rates`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn path_b_finalize_seeds_vat_rates(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    seed_minimal_chart(&pool, company_id).await;

    run_path_b_until_finalize(&app, &token).await;
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/finalize"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 4 vat_rates en DB (DB-side check).
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vat_rates WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 4, "Path B finalize should seed 4 vat_rates");

    // Et l'endpoint REST renvoie la même liste.
    let resp = app
        .client
        .get(app.url("/api/v1/vat-rates"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 4);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn path_b_finalize_idempotent_with_existing_fiscal_year(pool: MySqlPool) {
    use kesh_db::entities::NewFiscalYear;

    let (app, token) = bootstrap_admin(&pool).await;

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    seed_minimal_chart(&pool, company_id).await;
    let admin_user_id: i64 =
        sqlx::query_scalar("SELECT id FROM users WHERE username = 'admin' LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();

    // Code Review Pass 1 F18 — pré-insérer via `create()` (qui audit) au lieu
    // de `create_for_seed()` (qui n'audit pas). De cette façon, la vérification
    // qu'« aucun audit fiscal_year.created n'a été ajouté » devient vraiment
    // significative — on s'assure qu'il n'y a qu'**une seule** entrée audit
    // existante (pré-insertion) après le finalize idempotent, pas zéro et pas
    // deux.
    let preexisting = kesh_db::repositories::fiscal_years::create(
        &pool,
        admin_user_id,
        NewFiscalYear {
            company_id,
            name: "Pre-existing".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    // Vérification baseline : 1 audit fiscal_year.created issu de `create()`.
    let baseline_audit = audit_log::find_by_entity(&pool, "fiscal_year", preexisting.id, 10)
        .await
        .unwrap();
    let baseline_created_count = baseline_audit
        .iter()
        .filter(|e| e.action == "fiscal_year.created")
        .count();
    assert_eq!(
        baseline_created_count, 1,
        "1 baseline audit entry from create()"
    );

    run_path_b_until_finalize(&app, &token).await;
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/finalize"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let list = kesh_db::repositories::fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1, "no duplicate created");
    assert_eq!(list[0].name, "Pre-existing");

    // Le finalize idempotent ne doit AUCUNEMENT toucher au compteur d'audit :
    // toujours 1 entrée fiscal_year.created (celle de la pré-insertion), pas 2.
    let after_audit = audit_log::find_by_entity(&pool, "fiscal_year", preexisting.id, 10)
        .await
        .unwrap();
    let after_created_count = after_audit
        .iter()
        .filter(|e| e.action == "fiscal_year.created")
        .count();
    assert_eq!(
        after_created_count, 1,
        "finalize must be idempotent on audit log (still 1 fiscal_year.created entry, not 2)"
    );
}

/// Code Review Pass 1 F9 — AC #14 : couvre la non-régression du flow démo
/// (Path A) après le refactor T1.9 (`kesh_seed::seed_demo` migré vers
/// `create_for_seed`). On vérifie :
/// - Le fiscal_year est bien créé (`status='Open'`, dates 1er janvier-31 décembre).
/// - Le nom est bien `Exercice {YYYY}` où YYYY = année courante.
/// - **Aucun audit log fiscal_year.created** n'est inséré (cohérent avec
///   `create_for_seed` qui n'audit pas).
///
/// Pattern aligné sur `onboarding_e2e::full_onboarding_flow_demo_path` :
/// language → mode=guided → seed-demo (3 calls successifs).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn demo_path_creates_fiscal_year(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;

    // Avancer jusqu'au step demo : language → mode=guided → seed-demo.
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "language step should succeed");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "mode step should succeed");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "seed-demo should succeed");

    // Vérifier le fiscal_year créé par seed_demo.
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    let list = kesh_db::repositories::fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(
        list.len(),
        1,
        "exactly one fiscal_year created by seed_demo"
    );
    let fy = &list[0];
    let current_year = chrono::Utc::now().naive_utc().date().year();
    assert_eq!(fy.name, format!("Exercice {current_year}"));
    assert_eq!(fy.start_date.year(), current_year);
    assert_eq!(fy.start_date.month(), 1);
    assert_eq!(fy.start_date.day(), 1);
    assert_eq!(fy.end_date.year(), current_year);
    assert_eq!(fy.end_date.month(), 12);
    assert_eq!(fy.end_date.day(), 31);
    assert_eq!(
        fy.status,
        kesh_db::entities::FiscalYearStatus::Open,
        "seed_demo creates an Open fiscal year"
    );

    // AC #14 — pas d'entrée audit fiscal_year.created (seed contexte système).
    let entries = audit_log::find_by_entity(&pool, "fiscal_year", fy.id, 10)
        .await
        .unwrap();
    assert!(
        !entries.iter().any(|e| e.action == "fiscal_year.created"),
        "create_for_seed must NOT write an audit log entry (system seed context)"
    );
}

// ---------------------------------------------------------------------------
// Code Review Pass 1 F2 — multi-tenant defense in depth pour update_name + close
// ---------------------------------------------------------------------------

async fn create_other_company(pool: &MySqlPool) -> i64 {
    use kesh_db::entities::{Language, NewCompany, OrgType};
    kesh_db::repositories::companies::create(
        pool,
        NewCompany {
            name: "Other Co".into(),
            address: "Other Street 2".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

/// F2 — `update_name` repo refuse une mutation cross-tenant même si le
/// pre-check du handler est bypassé (ex. caller direct hors route).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_name_repo_rejects_cross_tenant(pool: MySqlPool) {
    use kesh_db::entities::NewFiscalYear;
    use kesh_db::errors::DbError;

    let (_app, _token) = bootstrap_admin(&pool).await;

    let admin_company_id: i64 =
        sqlx::query_scalar("SELECT company_id FROM users WHERE username = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let admin_user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username = 'admin'")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Créer une 2e company avec son propre fiscal_year.
    let other_company_id = create_other_company(&pool).await;
    let other_fy = kesh_db::repositories::fiscal_years::create_for_seed(
        &pool,
        NewFiscalYear {
            company_id: other_company_id,
            name: "Other FY".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    // Tentative cross-tenant : admin (company 1) tente de renommer le fiscal_year
    // de Other Co (company 2). Le repo doit retourner NotFound, pas autoriser.
    let result = kesh_db::repositories::fiscal_years::update_name(
        &pool,
        admin_user_id,
        admin_company_id,
        other_fy.id,
        "hacked".into(),
    )
    .await;
    assert!(
        matches!(result, Err(DbError::NotFound)),
        "update_name must reject cross-tenant mutation, got {:?}",
        result
    );

    // Vérifier que le nom n'a PAS changé en DB.
    let unchanged = kesh_db::repositories::fiscal_years::find_by_id(&pool, other_fy.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.name, "Other FY", "name must remain unchanged");
}

/// F2 — `close` repo refuse une mutation cross-tenant.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_repo_rejects_cross_tenant(pool: MySqlPool) {
    use kesh_db::entities::{FiscalYearStatus, NewFiscalYear};
    use kesh_db::errors::DbError;

    let (_app, _token) = bootstrap_admin(&pool).await;

    let admin_company_id: i64 =
        sqlx::query_scalar("SELECT company_id FROM users WHERE username = 'admin'")
            .fetch_one(&pool)
            .await
            .unwrap();
    let admin_user_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE username = 'admin'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let other_company_id = create_other_company(&pool).await;
    let other_fy = kesh_db::repositories::fiscal_years::create_for_seed(
        &pool,
        NewFiscalYear {
            company_id: other_company_id,
            name: "Other FY".into(),
            start_date: chrono::NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
            end_date: chrono::NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap();

    let result = kesh_db::repositories::fiscal_years::close(
        &pool,
        admin_user_id,
        admin_company_id,
        other_fy.id,
    )
    .await;
    assert!(
        matches!(result, Err(DbError::NotFound)),
        "close must reject cross-tenant mutation, got {:?}",
        result
    );

    // Vérifier que le fiscal_year est toujours Open en DB.
    let unchanged = kesh_db::repositories::fiscal_years::find_by_id(&pool, other_fy.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.status, FiscalYearStatus::Open);
}
