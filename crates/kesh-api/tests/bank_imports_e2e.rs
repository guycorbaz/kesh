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
            email: None,
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

    // Story 8-3 T6.6 — l'action audit_log est désormais **canonique
    // unique** `bank_import.created` + `details_json.modifiers` qui
    // discrimine via la liste `["balance_mismatch", ...]` (cf. spec
    // §audit-log-actions). Pattern de migration depuis 8-1b.
    let row: (String, Option<Value>) = sqlx::query_as(
        "SELECT action, details_json FROM audit_log WHERE entity_type = 'bank_imports' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "bank_import.created");
    let details = row.1.expect("details_json present");
    let modifiers = details["modifiers"].as_array().unwrap();
    assert!(
        modifiers
            .iter()
            .any(|m| m == &Value::String("balance_mismatch".to_string())),
        "modifiers={modifiers:?} doit contenir 'balance_mismatch'"
    );
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

// Story 8-3 T1.5 — `post_import_rejects_duplicate_file_within_company` (8-1b)
// supprimé : il assertait l'erreur SQL 1062 (UNIQUE violation), caduque
// post-migration `20260507000001_bank_imports_relax_hash_unique.sql`. Remplacé
// par `post_import_rejects_duplicate_file_without_confirm` (AC #2) +
// `post_import_accepts_duplicate_file_with_confirm` (AC #3) ajoutés en T6.5.

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

// =========================================================================
// Story 8-3 T6.5 — 13 nouveaux tests E2E (FR43 + FR51 + KF #70)
// =========================================================================

fn build_multipart_with_text_fields(
    file_bytes: Vec<u8>,
    filename: &str,
    bank_account_id: i64,
    extra_fields: &[(&str, String)],
) -> multipart::Form {
    let mut form = build_multipart(file_bytes, filename, bank_account_id);
    for (name, value) in extra_fields {
        form = form.text(name.to_string(), value.clone());
    }
    form
}

async fn create_test_bank_profile_csv(pool: &MySqlPool, company_id: i64, user_id: i64) -> i64 {
    let new_profile = kesh_db::entities::NewBankProfile {
        bank_name: "TestBank8-3".into(),
        filename_pattern: None,
        column_mapping_json: r#"{"date":0,"amount":1,"reference":2,"details":3}"#.into(),
        date_format: "%Y-%m-%d".into(),
        decimal_separator: ".".into(),
        field_separator: ";".into(),
        encoding: None,
        header_row_count: 1,
    };
    let mut tx = pool.begin().await.unwrap();
    let p =
        kesh_db::repositories::bank_profiles::create(&mut tx, company_id, &new_profile, user_id)
            .await
            .expect("create bank_profile");
    tx.commit().await.unwrap();
    p.id
}

fn build_csv_multipart(
    bytes: Vec<u8>,
    filename: &str,
    bank_account_id: i64,
    bank_profile_id: i64,
    extra: &[(&str, String)],
) -> multipart::Form {
    let mut form = multipart::Form::new()
        .text("bankAccountId", bank_account_id.to_string())
        .text("bankProfileId", bank_profile_id.to_string())
        .part(
            "file",
            multipart::Part::bytes(bytes)
                .file_name(filename.to_string())
                .mime_str("text/csv")
                .unwrap(),
        );
    for (name, value) in extra {
        form = form.text(name.to_string(), value.clone());
    }
    form
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_preview_returns_duplicate_file_warning(pool: MySqlPool) {
    // AC #1 — fichier déjà importé : preview retourne `warnings.duplicateFile`.
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    // Premier import (réussi)
    let form1 = build_multipart(xml.clone(), "v04_minimal.xml", s.bank_account_id);
    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(form1)
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    // Preview du même fichier → warning duplicateFile présent
    let form2 = build_multipart(xml, "v04_minimal.xml", s.bank_account_id);
    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports/preview"))
        .bearer_auth(&s.jwt)
        .multipart(form2)
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 200);
    let body: Value = r2.json().await.unwrap();
    let dup = &body["warnings"]["duplicateFile"];
    assert!(!dup.is_null(), "duplicateFile attendu");
    assert!(dup["existingImportId"].as_i64().unwrap() > 0);
    assert_eq!(dup["existingFilename"].as_str().unwrap(), "v04_minimal.xml");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_rejects_duplicate_file_without_confirm(pool: MySqlPool) {
    // AC #2 — sans confirmDuplicateFile → 422 BANK_IMPORT_DUPLICATE_FILE.
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    let xml = read_fixture("v04_minimal.xml");
    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(
            xml.clone(),
            "v04_minimal.xml",
            s.bank_account_id,
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(xml, "v04_minimal.xml", s.bank_account_id))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 422);
    let body: Value = r2.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_DUPLICATE_FILE");
    assert!(
        body["error"]["details"]["existingImportId"]
            .as_i64()
            .unwrap()
            > 0
    );
    assert_eq!(
        body["error"]["details"]["existingFilename"]
            .as_str()
            .unwrap(),
        "v04_minimal.xml"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_accepts_duplicate_file_with_confirm(pool: MySqlPool) {
    // AC #3 — avec confirmDuplicateFile=true → 201 + 2 rows distincts.
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let xml = read_fixture("v04_minimal.xml");
    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(
            xml.clone(),
            "v04_minimal.xml",
            s.bank_account_id,
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart_with_text_fields(
            xml,
            "v04_minimal.xml",
            s.bank_account_id,
            &[("confirmDuplicateFile", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 201);

    // Vérifie 2 rows distincts.
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM bank_imports WHERE company_id = ? AND filename = 'v04_minimal.xml'",
    )
    .bind(s.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count, 2);

    // Audit log : action canonique + modifiers contient duplicate_file.
    let row: (String, Option<Value>) = sqlx::query_as(
        "SELECT action, details_json FROM audit_log WHERE entity_type='bank_imports' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "bank_import.created");
    let modifiers = row.1.unwrap()["modifiers"].as_array().cloned().unwrap();
    assert!(modifiers.iter().any(|m| m == "duplicate_file"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_preview_returns_duplicate_lines_warning(pool: MySqlPool) {
    // AC #5 — preview détecte les lignes en doublon (clé composite stable).
    let s = setup(&pool).await;
    let app = spawn_app(pool).await;

    // Premier import : v04_minimal (1 ligne, ref RF18539007547034).
    let xml1 = read_fixture("v04_minimal.xml");
    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(xml1, "v04_minimal.xml", s.bank_account_id))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    // Preview du fichier overlap (hash distinct, 1 ligne en doublon).
    let xml2 = read_fixture("v04_overlap.xml");
    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports/preview"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(xml2, "v04_overlap.xml", s.bank_account_id))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 200);
    let body: Value = r2.json().await.unwrap();
    assert!(
        body["warnings"]["duplicateFile"].is_null(),
        "duplicateFile devrait être null (hashes distincts)"
    );
    let dup_lines = body["warnings"]["duplicateLines"].as_array().unwrap();
    assert_eq!(dup_lines.len(), 1, "1 ligne doublon attendue");
    let new_idx = dup_lines[0]["newIndex"].as_u64().unwrap();
    assert!(new_idx == 0 || new_idx == 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_skips_duplicate_lines_by_default(pool: MySqlPool) {
    // AC #6 — sans confirmDuplicateLines → skip (default).
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(
            read_fixture("v04_minimal.xml"),
            "v04_minimal.xml",
            s.bank_account_id,
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(
            read_fixture("v04_overlap.xml"),
            "v04_overlap.xml",
            s.bank_account_id,
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 201);
    let body: Value = r2.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    // Doit avoir persisté 1 transaction (la nouvelle), pas la doublon.
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 1);

    // Audit modifiers contient duplicate_lines_skipped.
    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let modifiers = details.unwrap()["modifiers"].as_array().cloned().unwrap();
    assert!(modifiers.iter().any(|m| m == "duplicate_lines_skipped"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_force_imports_duplicate_lines(pool: MySqlPool) {
    // AC #7 — confirmDuplicateLines=import → persiste tout.
    let s = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart(
            read_fixture("v04_minimal.xml"),
            "v04_minimal.xml",
            s.bank_account_id,
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_multipart_with_text_fields(
            read_fixture("v04_overlap.xml"),
            "v04_overlap.xml",
            s.bank_account_id,
            &[("confirmDuplicateLines", "import".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 201);
    let body: Value = r2.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 2, "import='import' → toutes les lignes persistées");

    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let modifiers = details.unwrap()["modifiers"].as_array().cloned().unwrap();
    assert!(modifiers.iter().any(|m| m == "duplicate_lines_imported"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_does_not_detect_duplicate_lines_across_tenants(pool: MySqlPool) {
    // AC #10 — multi-tenant safety : transactions cross-company invisibles.
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

    // company_b importe v04_minimal.
    let rb = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&jwt_b)
        .multipart(build_multipart(xml.clone(), "v04_minimal.xml", bank_b))
        .send()
        .await
        .unwrap();
    assert_eq!(rb.status(), 201);

    // company_a importe v04_minimal — pas de doublons côté company_a.
    let ra = app
        .client
        .post(app.url("/api/v1/bank-imports/preview"))
        .bearer_auth(&jwt_a)
        .multipart(build_multipart(xml, "v04_minimal.xml", bank_a))
        .send()
        .await
        .unwrap();
    assert_eq!(ra.status(), 200);
    let body: Value = ra.json().await.unwrap();
    assert!(
        body["warnings"]["duplicateLines"]
            .as_array()
            .unwrap()
            .is_empty(),
        "aucun doublon cross-tenant attendu"
    );
    assert!(
        body["warnings"]["duplicateFile"].is_null(),
        "duplicateFile null cross-tenant (hash existe sur company_b mais filtré pour company_a)"
    );
}

// === CSV path (T6.5#8-13) ===

const CSV_VALID_3_LINES: &str = "date;amount;ref;details\n\
                                 2026-05-15;100.00;REF-1;V1\n\
                                 2026-05-16;200.00;REF-2;V2\n\
                                 2026-05-17;300.00;REF-3;V3\n";

const CSV_PARTIAL_3VALID_3INVALID: &str = "date;amount;ref;details\n\
                                            2026-05-15;100.00;REF-1;V1\n\
                                            INVALID_DATE;200.00;REF-2;V2\n\
                                            2026-05-16;NOT_NUM;REF-3;V3\n\
                                            2026-05-17;300.00;REF-4;V4\n\
                                            2026-05-18;400.00;REF-5;V5\n\
                                            BAD_DATE_2;500.00;REF-6;V6\n";

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_preview_csv_returns_invalid_lines_warning(pool: MySqlPool) {
    // AC #12 — CSV avec lignes invalides : warnings.invalidLines retourné.
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool).await;

    let r = app
        .client
        .post(app.url("/api/v1/bank-imports/preview"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let body: Value = r.json().await.unwrap();
    let inv = &body["warnings"]["invalidLines"];
    assert!(!inv.is_null());
    let lines = inv["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 3);
    assert_eq!(inv["totalErrors"].as_u64().unwrap(), 3);
    assert!(!inv["truncated"].as_bool().unwrap());
    // Les transactions valides sont aussi listées.
    let txs = body["transactions"].as_array().unwrap();
    assert_eq!(txs.len(), 3);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_csv_accepts_partial_with_confirm(pool: MySqlPool) {
    // AC #14 — confirmPartialImport=true persiste les lignes valides.
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool.clone()).await;

    let r = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmPartialImport", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let body: Value = r.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 3, "3 lignes valides persistées");

    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let d = details.unwrap();
    let modifiers = d["modifiers"].as_array().cloned().unwrap();
    assert!(modifiers.iter().any(|m| m == "partial"));
    assert_eq!(d["partial_invalid_lines"].as_u64().unwrap(), 3);
    assert_eq!(d["partial_total_errors"].as_u64().unwrap(), 3);
    assert!(!d["partial_truncated"].as_bool().unwrap());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_csv_accepts_partial_with_truncated_errors(pool: MySqlPool) {
    // AC #15 — 50 valides + 150 invalides → cap 100 + truncated=true.
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool.clone()).await;

    let mut csv = String::from("date;amount;ref;details\n");
    for i in 0..50 {
        csv.push_str(&format!("2026-05-15;{}.00;R{i};Valid\n", i + 1));
    }
    for i in 0..150 {
        csv.push_str(&format!("INVALID_DATE_{i};100.00;R;X\n"));
    }

    let r = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            csv.into_bytes(),
            "stmt_huge.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmPartialImport", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 201);
    let body: Value = r.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
            .bind(import_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 50);

    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let d = details.unwrap();
    assert_eq!(d["partial_invalid_lines"].as_u64().unwrap(), 100);
    assert_eq!(d["partial_total_errors"].as_u64().unwrap(), 150);
    assert!(d["partial_truncated"].as_bool().unwrap());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_csv_rejects_partial_when_zero_valid_lines(pool: MySqlPool) {
    // AC #16 — 0 valides + N invalides → 422 reason="no_valid_lines_to_commit".
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool).await;

    let csv =
        "date;amount;ref;details\nINVALID_A;100;R1;A\nINVALID_B;200;R2;B\nINVALID_C;300;R3;C\n";
    let r = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            csv.as_bytes().to_vec(),
            "all_invalid.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmPartialImport", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 422);
    let body: Value = r.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_CSV_PARTIAL_FAILURE");
    assert_eq!(
        body["error"]["details"]["reason"],
        "no_valid_lines_to_commit"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_csv_combines_three_confirm_flags(pool: MySqlPool) {
    // AC #17 — combinaison duplicate_file + duplicate_lines + partial.
    //
    // M6 (Pass 1 review) — utilise CSV_PARTIAL_3VALID_3INVALID des deux
    // côtés (premier import en partial commit, second en re-upload du même
    // fichier) pour exercer les 3 modifiers simultanément :
    //   - duplicate_file (même hash)
    //   - duplicate_lines_skipped (les 3 valides matchent les transactions
    //     déjà persistées)
    //   - partial (3 lignes invalides parsent comme telles)
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool.clone()).await;

    // Premier import : CSV avec 3 valides + 3 invalides + confirmPartialImport.
    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmPartialImport", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201, "premier import partial commit");

    // Second import : MÊME bytes → même hash. Les 3 valides matchent les
    // transactions persistées en T1 → duplicate_lines_skipped. Le parse
    // CSV produit 3 invalides → partial. confirmDuplicateFile bypass.
    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[
                ("confirmDuplicateFile", "true".into()),
                ("confirmDuplicateLines", "skip".into()),
                ("confirmPartialImport", "true".into()),
            ],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 201);
    let body: Value = r2.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();

    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let detail_obj = details.unwrap();
    let modifiers = detail_obj["modifiers"].as_array().cloned().unwrap();
    let mods_str: Vec<String> = modifiers
        .iter()
        .map(|m| m.as_str().unwrap().to_string())
        .collect();
    assert!(
        mods_str.contains(&"duplicate_file".to_string()),
        "modifiers contains duplicate_file: {mods_str:?}"
    );
    assert!(
        mods_str.contains(&"duplicate_lines_skipped".to_string()),
        "modifiers contains duplicate_lines_skipped: {mods_str:?}"
    );
    assert!(
        mods_str.contains(&"partial".to_string()),
        "modifiers contains partial: {mods_str:?}"
    );
    // M6 — assertion de tri non-trivial : 3 modifiers ordonnés
    // alphabétiquement (cf. spec §audit-log-actions).
    assert_eq!(
        mods_str,
        vec![
            "duplicate_file".to_string(),
            "duplicate_lines_skipped".to_string(),
            "partial".to_string(),
        ],
        "modifiers triés alphabétiquement (3 éléments — tri non-trivial)"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn dedup_handles_2000_existing_under_3s(pool: MySqlPool) {
    // L10 (Pass 1 review) — AC #29 perf smoke E2E HTTP. **Non-bloquant CI** :
    // émet un warning via `eprintln!` si > 3s sans faire échouer le test
    // (la cible 200×2000 décrit la machine de dev nominale ; CI peut être
    // plus lente). Détecte les régressions perf sans flakiness.
    //
    // Volume réduit pour rester raisonnable en CI : 200 existantes × 50
    // nouvelles (vs 200×2000 dans la spec). L'invariant testé est
    // O(N+M) via HashMap — pas O(N×M). Le test unitaire
    // `kesh_core::detect_duplicate_lines_handles_n_to_m_in_o_n_plus_m`
    // couvre déjà 1000×5000 < 50ms côté algo pur.
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;

    // Seed direct SQL : 1 bank_imports row + 200 bank_transactions dans
    // la fenêtre 2026-04-01 .. 2026-04-30 du nouvel import.
    //
    // CI-fix : `file_hash` doit faire 64 chars exactement (CHECK
    // constraint `chk_bank_imports_hash_len` = SHA-256 hex). Seed a
    // sentinel-hash distinct des autres tests (préfixe `dedupperf`).
    let seed_file_hash = format!("dedupperf{}", "0".repeat(64 - 9));
    let imported_at = Utc::now().naive_utc();
    let import_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, statement_id, period_from, period_to, opening_balance, \
         closing_balance, transaction_count, imported_at, imported_by_user_id) \
         VALUES (?, ?, 'seed.csv', ?, 'CSV', NULL, '2026-04-01', \
         '2026-04-30', NULL, NULL, 200, ?, ?) RETURNING id",
    )
    .bind(s.company_id)
    .bind(s.bank_account_id)
    .bind(&seed_file_hash)
    .bind(imported_at)
    .bind(s.user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut qb = sqlx::QueryBuilder::<sqlx::MySql>::new(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, \
         booking_date, value_date, amount, currency, reference, details, end_to_end_id, \
         transaction_id, counterparty_iban, counterparty_name, status) ",
    );
    qb.push_values(0..200_i64, |mut b, i| {
        let day = ((i % 28) + 1) as u32;
        let date = chrono::NaiveDate::from_ymd_opt(2026, 4, day).unwrap();
        let amount = rust_decimal::Decimal::new(100 + i, 2);
        b.push_bind(s.company_id)
            .push_bind(import_id)
            .push_bind(s.bank_account_id)
            .push_bind(date)
            .push_bind(Option::<chrono::NaiveDate>::None)
            .push_bind(amount)
            .push_bind("CHF")
            .push_bind(Some(format!("SEED-REF-{i}")))
            .push_bind("seed".to_string())
            .push_bind(Option::<String>::None)
            .push_bind(Option::<String>::None)
            .push_bind(Option::<String>::None)
            .push_bind(Option::<String>::None)
            // CI-fix : `chk_bank_transactions_status` n'autorise que
            // `'pending'` ou `'reconciled'` (cf. migration 20260504000001).
            .push_bind("pending");
    });
    qb.build().execute(&pool).await.unwrap();

    // CSV de 50 nouvelles transactions hors de la fenêtre seedée pour
    // garantir 0 doublons (test sur le path de chargement+comparaison,
    // pas sur les match).
    let mut csv = String::from("date;amount;ref;details\n");
    for i in 0..50 {
        let day = (i % 28) + 1;
        csv.push_str(&format!(
            "2026-05-{day:02};{}.00;NEW-REF-{i};new\n",
            500 + i
        ));
    }

    let app = spawn_app(pool.clone()).await;
    let start = std::time::Instant::now();
    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            csv.into_bytes(),
            "stress.csv",
            s.bank_account_id,
            profile_id,
            &[],
        ))
        .send()
        .await
        .unwrap();
    let elapsed = start.elapsed();
    assert_eq!(resp.status(), 201, "stress import 201");

    if elapsed.as_secs_f64() > 3.0 {
        eprintln!(
            "[L10 perf warning] dedup_handles_2000_existing_under_3s: {elapsed:?} > 3s — possible regression"
        );
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_with_confirm_duplicate_file_on_fresh_file_no_modifier(pool: MySqlPool) {
    // L7 (Pass 1 review) — defensive client envoie confirmDuplicateFile=true
    // sur un fichier frais (pas de duplicate detected) → 201 sans le
    // modifier `duplicate_file` (le modifier n'est ajouté que si
    // `find_by_company_and_hash` retourne `Some(_)`).
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_VALID_3_LINES.as_bytes().to_vec(),
            "fresh.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmDuplicateFile", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "fresh file accepted");

    let body: Value = resp.json().await.unwrap();
    let import_id = body["id"].as_i64().unwrap();
    let details: Option<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type='bank_imports' AND entity_id=?",
    )
    .bind(import_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mods = details.unwrap()["modifiers"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let mods_str: Vec<String> = mods
        .iter()
        .map(|m| m.as_str().unwrap().to_string())
        .collect();
    assert!(
        !mods_str.contains(&"duplicate_file".to_string()),
        "modifier duplicate_file absent quand pas de duplicate (got {mods_str:?})"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_import_returns_duplicate_file_first_when_no_flags(pool: MySqlPool) {
    // AC #18 — fail-fast applicatif : duplicate_file 422 retourné AVANT
    // le parse CSV (donc pas BANK_CSV_PARTIAL_FAILURE même si le CSV
    // contiendrait des erreurs).
    let s = setup(&pool).await;
    let profile_id = create_test_bank_profile_csv(&pool, s.company_id, s.user_id).await;
    let app = spawn_app(pool).await;

    let r1 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[("confirmPartialImport", "true".into())],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 201);

    // Re-upload SAME CSV sans confirms — duplicate_file détecté en
    // premier (avant le parse).
    let r2 = app
        .client
        .post(app.url("/api/v1/bank-imports"))
        .bearer_auth(&s.jwt)
        .multipart(build_csv_multipart(
            CSV_PARTIAL_3VALID_3INVALID.as_bytes().to_vec(),
            "stmt.csv",
            s.bank_account_id,
            profile_id,
            &[],
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 422);
    let body: Value = r2.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "BANK_IMPORT_DUPLICATE_FILE",
        "duplicate_file détecté AVANT le parse partial"
    );
}
