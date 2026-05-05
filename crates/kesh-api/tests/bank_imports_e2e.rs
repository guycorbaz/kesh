//! Tests E2E HTTP pour Story 8-1b — import bancaire CAMT.053 (T6.9).
//!
//! 14 tests : happy path, multi-stmt scoping, RBAC, dedup, balance check,
//! currency, audit log, multi-tenant list/detail, parse error.
//!
//! Fixtures lues directement depuis `crates/kesh-import/tests/fixtures/`
//! via `env!("CARGO_MANIFEST_DIR")` + chemin relatif (M2 validate Pass 2).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{Language, NewBankAccount, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{bank_accounts, companies, users};
use reqwest::multipart;
use serde_json::Value;
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

async fn create_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id
}

fn fixture_path(name: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../kesh-import/tests/fixtures/camt053")
        .join(name)
}

fn read_fixture(name: &str) -> Vec<u8> {
    std::fs::read(fixture_path(name)).expect("fixture lecture OK")
}

fn build_multipart(file_bytes: Vec<u8>, filename: &str, bank_account_id: i64) -> multipart::Form {
    multipart::Form::new()
        .text("bankAccountId", bank_account_id.to_string())
        .part(
            "file",
            multipart::Part::bytes(file_bytes)
                .file_name(filename.to_string())
                .mime_str("application/xml")
                .unwrap(),
        )
}

fn build_multipart_with_confirm(
    file_bytes: Vec<u8>,
    filename: &str,
    bank_account_id: i64,
) -> multipart::Form {
    build_multipart(file_bytes, filename, bank_account_id).text("confirmBalanceMismatch", "true")
}

struct TestSetup {
    user_id: i64,
    #[allow(dead_code)]
    company_id: i64,
    bank_account_id: i64,
    jwt: String,
}

async fn setup(pool: &MySqlPool) -> TestSetup {
    let company_id = create_company(pool, "Acme").await;
    let user_id = create_user(pool, "alice", Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, "CH4431999123000889012").await;
    let jwt = forge_jwt(user_id, "Comptable", company_id);
    TestSetup {
        user_id,
        company_id,
        bank_account_id,
        jwt,
    }
}

// =========================================================================
// Tests T6.9 — 14 cas (F1+F3+F4 validate Pass 1 + O3 validate Pass 3)
// =========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_creates_rows_atomically(pool: MySqlPool) {
    // AC #1, #3a, #17 — happy path
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let xml = read_fixture("v04_minimal.xml");
    let form = build_multipart(xml, "v04_minimal.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    assert!(import_id > 0);
    assert_eq!(body["bankAccountId"].as_i64().unwrap(), s.bank_account_id);
    assert_eq!(body["transactionCount"].as_i64().unwrap(), 1);

    // Vérifier persistance.
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);

    // AC #3a (review code Pass 1 H3) : chaque ligne `bank_transactions`
    // doit avoir `bank_account_id = selected_id`. Sans cette assertion
    // explicite, une régression sur la FK ne serait pas détectée.
    //
    // Review code Pass 2 L3 : ce test crée 1 seule transaction donc
    // `LIMIT 1` est exact. **Si un futur test ajoute > 1 transaction
    // pour le même import**, remplacer par :
    //   COUNT(DISTINCT bank_account_id) FROM bank_transactions WHERE import_id = ?
    // = 1 + assertion `bank_account_id = ?` sur cette unique valeur.
    let ba_id: i64 = sqlx::query_scalar(
        "SELECT bank_account_id FROM bank_transactions WHERE import_id = ? LIMIT 1",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        ba_id, s.bank_account_id,
        "AC #3a : chaque transaction doit porter bank_account_id du compte cible"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_preview_returns_ignored_statements_for_multi_stmt_file(pool: MySqlPool) {
    // AC #3b — F1 validate Pass 1
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_multi_stmt.xml");
    let form = build_multipart(xml, "v04_multi_stmt.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports/preview"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let ignored = body["ignoredStatements"].as_array().unwrap();
    assert_eq!(
        ignored.len(),
        1,
        "1 statement ignoré (l'autre matche le bankAccountId)"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_when_no_stmt_matches_selected_account(pool: MySqlPool) {
    // AC #3c — F4 validate Pass 1
    let company_id = create_company(&pool, "Acme").await;
    let user_id = create_user(&pool, "alice", Role::Comptable, company_id).await;
    // bank_account avec un IBAN qui n'existe pas dans v04_minimal.xml
    let bank_id = create_bank_account(&pool, company_id, "CH5604835012345678009").await;
    let jwt = forge_jwt(user_id, "Comptable", company_id);
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    let form = build_multipart(xml, "v04_minimal.xml", bank_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_NO_MATCHING_STATEMENT");
    assert!(body["error"]["details"]["foundIbans"].is_array());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_when_role_consultation(pool: MySqlPool) {
    // AC #12 — RBAC
    let company_id = create_company(&pool, "Acme").await;
    let user_id = create_user(&pool, "alice", Role::Consultation, company_id).await;
    let bank_id = create_bank_account(&pool, company_id, "CH4431999123000889012").await;
    let jwt = forge_jwt(user_id, "Consultation", company_id);
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    let form = build_multipart(xml, "v04_minimal.xml", bank_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_payload_too_large(pool: MySqlPool) {
    // AC #13 — payload limit
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    // Génère un buffer 11 MiB > limite 10 MiB par défaut.
    let big = vec![b'<'; 11 * 1024 * 1024];
    let form = build_multipart(big, "huge.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 413);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_balance_mismatch_without_confirm(pool: MySqlPool) {
    // AC #14
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_balance_mismatch.xml");
    let form = build_multipart(xml, "v04_balance_mismatch.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_BALANCE_MISMATCH");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_accepts_balance_mismatch_with_confirm(pool: MySqlPool) {
    // AC #14 — confirm bypass + audit log spécial
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let xml = read_fixture("v04_balance_mismatch.xml");
    let form = build_multipart_with_confirm(xml, "v04_balance_mismatch.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    // Vérifier action audit_log spéciale.
    let action: String = sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE entity_type = 'bank_imports' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(action, "bank_import.created_with_balance_mismatch");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_eur_currency(pool: MySqlPool) {
    // AC #15 — devise v0.1 CHF only
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_eur_currency.xml");
    let form = build_multipart(xml, "v04_eur_currency.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_UNSUPPORTED_CURRENCY");
    assert_eq!(body["error"]["details"]["currency"], "EUR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_duplicate_file_within_company(pool: MySqlPool) {
    // AC #16
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");

    // Premier import → 201
    let form1 = build_multipart(xml.clone(), "v04_minimal.xml", s.bank_account_id);
    let resp1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form1)
        .send()
        .await
        .unwrap();
    assert_eq!(resp1.status(), 201);

    // Second import même hash → 409
    let form2 = build_multipart(xml, "v04_minimal.xml", s.bank_account_id);
    let resp2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form2)
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 409);
    let body: Value = resp2.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_DUPLICATE_FILE");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_allows_same_file_across_companies(pool: MySqlPool) {
    // AC #16 multi-tenant safety
    let company_a = create_company(&pool, "CompanyA").await;
    let company_b = create_company(&pool, "CompanyB").await;
    let user_a = create_user(&pool, "alice", Role::Comptable, company_a).await;
    let user_b = create_user(&pool, "bob", Role::Comptable, company_b).await;
    let bank_a = create_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_bank_account(&pool, company_b, "CH4431999123000889012").await;
    let jwt_a = forge_jwt(user_a, "Comptable", company_a);
    let jwt_b = forge_jwt(user_b, "Comptable", company_b);
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    for (jwt, bank_id) in [(jwt_a, bank_a), (jwt_b, bank_b)] {
        let form = build_multipart(xml.clone(), "v04_minimal.xml", bank_id);
        let resp = app
            .client
            .post(app.url("/api/v1/bank-imports"))
            .bearer_auth(&jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201, "même hash, company différente → OK");
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_audit_log_contains_correct_entry(pool: MySqlPool) {
    // AC #18 — F3 validate Pass 1
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let xml = read_fixture("v04_minimal.xml");
    let form = build_multipart(xml, "v04_minimal.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();

    let row: (String, String, i64, i64, Option<Value>) = sqlx::query_as(
        "SELECT action, entity_type, entity_id, user_id, details_json \
         FROM audit_log WHERE entity_type = 'bank_imports' AND entity_id = ?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "bank_import.created");
    assert_eq!(row.1, "bank_imports");
    assert_eq!(row.2, import_id);
    assert_eq!(row.3, s.user_id);
    let details = row.4.unwrap();
    assert!(details.get("filename").is_some());
    assert!(details.get("transaction_count").is_some());
    assert!(details.get("source_format").is_some());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_imports_lists_only_own_company(pool: MySqlPool) {
    // AC #10 — multi-tenant scoping GET
    let company_a = create_company(&pool, "CompanyA").await;
    let company_b = create_company(&pool, "CompanyB").await;
    let user_a = create_user(&pool, "alice", Role::Comptable, company_a).await;
    let user_b = create_user(&pool, "bob", Role::Comptable, company_b).await;
    // Même IBAN sur les deux companies (autorisé multi-tenant) pour que la
    // fixture v04_minimal.xml matche les deux imports.
    let bank_a = create_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_bank_account(&pool, company_b, "CH4431999123000889012").await;
    let jwt_a = forge_jwt(user_a, "Comptable", company_a);
    let jwt_b = forge_jwt(user_b, "Comptable", company_b);
    let app = spawn_app(pool).await;

    // Les deux companies importent la même fixture (hashes distincts par
    // tenant car uq_bank_imports_company_hash est composite).
    let xml = read_fixture("v04_minimal.xml");
    for (jwt, bank_id) in [(&jwt_a, bank_a), (&jwt_b, bank_b)] {
        let form = build_multipart(xml.clone(), "v04_minimal.xml", bank_id);
        let resp = app
            .client
            .post(app.url("/api/v1/bank-imports"))
            .bearer_auth(jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201);
    }

    // GET company_a — doit voir 1 import (le sien).
    let resp = app
        .client
        .get(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt_a)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_import_returns_404_for_other_company_id(pool: MySqlPool) {
    // AC #10 — IDOR cross-tenant 404 (KF-002)
    let company_a = create_company(&pool, "CompanyA").await;
    let company_b = create_company(&pool, "CompanyB").await;
    let user_a = create_user(&pool, "alice", Role::Comptable, company_a).await;
    let user_b = create_user(&pool, "bob", Role::Comptable, company_b).await;
    let bank_a = create_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let jwt_a = forge_jwt(user_a, "Comptable", company_a);
    let jwt_b = forge_jwt(user_b, "Comptable", company_b);
    let app = spawn_app(pool).await;

    // company_a importe.
    let xml = read_fixture("v04_minimal.xml");
    let form = build_multipart(xml, "v04_minimal.xml", bank_a);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt_a)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();

    // company_b tente d'accéder au détail → 404 (jamais 403, KF-002)
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/bank-imports/{import_id}")))
        .bearer_auth(&jwt_b)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_truncated_xml(pool: MySqlPool) {
    // O3 validate Pass 3 — mapping CamtError::MalformedXml → 400
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_truncated.xml");
    let form = build_multipart(xml, "v04_truncated.xml", s.bank_account_id);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_MALFORMED_XML");
    assert_eq!(body["error"]["details"]["kind"], "MALFORMED_XML");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_when_bank_account_belongs_to_other_company(pool: MySqlPool) {
    // 14e test : IDOR sur bankAccountId du payload — un user company_A ne
    // peut pas faire passer le bankAccountId d'une company_B (T6.3 / Pass 1 H5).
    let company_a = create_company(&pool, "CompanyA").await;
    let company_b = create_company(&pool, "CompanyB").await;
    let user_a = create_user(&pool, "alice", Role::Comptable, company_a).await;
    let _user_b = create_user(&pool, "bob", Role::Comptable, company_b).await;
    let _bank_a = create_bank_account(&pool, company_a, "CH4431999123000889012").await;
    let bank_b = create_bank_account(&pool, company_b, "CH9300762011623852957").await;
    let jwt_a = forge_jwt(user_a, "Comptable", company_a);
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    // user_a tente d'importer en passant bank_b (autre company)
    let form = build_multipart(xml, "v04_minimal.xml", bank_b);
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt_a)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        404,
        "IDOR : 404 (jamais 403, anti-énumération)"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
}
