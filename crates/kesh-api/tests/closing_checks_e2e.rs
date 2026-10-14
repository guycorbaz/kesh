//! Tests E2E HTTP — Story 29-1a (#232) : `GET /api/v1/fiscal-years/{id}/closing-checks`,
//! les contrôles préalables à la clôture, en lecture seule (T-a11).
//!
//! Couvre : 200 pour tout rôle (Admin, Comptable, Consultation) et pour une
//! clé d'API `read` ; 404 pour un exercice d'une autre société
//! (anti-énumération) ; 401 sans authentification ; la forme JSON (camelCase,
//! codes stables, montants en chaînes).
//!
//! Pré-requis : MariaDB démarré.

mod common;

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

async fn fiscal_year_2026(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn get(app: &TestApp, token: Option<&str>, fy: i64) -> reqwest::Response {
    let mut req = app
        .client
        .get(app.url(&format!("/api/v1/fiscal-years/{fy}/closing-checks")));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    req.send().await.unwrap()
}

/// T-a11 — 200 pour Admin, Comptable, Consultation et une clé `read` ; forme JSON.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn closing_checks_are_served_to_every_role(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company = create_company(&pool, "Acme").await;
    let fy = fiscal_year_2026(&pool, company).await;
    // Un écart réel au relevé : un montant part en chaîne.
    let ledger: i64 = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, '1020', 'Banque', 'Asset')",
    )
    .bind(company)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let ba: i64 = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'UBS', 'CH4431999123000889012', ?)",
    )
    .bind(company)
    .bind(ledger)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    let mut tokens = Vec::new();
    for (role, label) in [
        (Role::Admin, "admin_u"),
        (Role::Comptable, "comptable_u"),
        (Role::Consultation, "consult_u"),
    ] {
        let uid = create_user(&pool, label, role, company).await;
        tokens.push(forge_jwt(uid, role.as_str(), company));
    }
    sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', REPEAT('e', 64), 'CAMT053', '2026-12-01', '2026-12-31', 100.00, 0, \
                 (SELECT id FROM users WHERE company_id = ? ORDER BY id LIMIT 1))",
    )
    .bind(company)
    .bind(ba)
    .bind(company)
    .execute(&pool)
    .await
    .unwrap();
    let (_id, key) =
        common::create_key_via_http(&app.client, &app.base_url, &tokens[1], "ro", "read").await;
    tokens.push(key);

    for token in &tokens {
        let resp = get(&app, Some(token), fy).await;
        assert_eq!(resp.status(), 200);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["fiscalYearId"], fy);
        assert_eq!(body["fiscalYearName"], "Exercice 2026");
        assert_eq!(body["startDate"], "2026-01-01");
        assert_eq!(body["endDate"], "2026-12-31");
        let codes: Vec<&str> = body["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["code"].as_str().unwrap())
            .collect();
        assert_eq!(
            codes,
            vec![
                "EARLIER_FISCAL_YEAR_OPEN",
                "PENDING_BANK_TRANSACTIONS",
                "BANK_STATEMENT_AT_YEAR_END",
                "DRAFT_INVOICES",
                "IMPORTED_SUPPLIER_INVOICES_TO_COMPLETE",
                "INCONSISTENT_ENTRIES",
                "NEXT_FISCAL_YEAR",
            ]
        );
        let k3 = &body["checks"][2];
        assert_eq!(k3["status"], "attention");
        assert_eq!(k3["severity"], "warning");
        let acc = &k3["details"]["accounts"][0];
        assert_eq!(acc["bankAccountId"], ba);
        assert_eq!(acc["reason"], "GAP");
        assert!(acc["gap"].is_string(), "montant en chaîne : {acc}");
        assert!(acc["statementClosingBalance"].is_string());
        assert!(k3["details"]["notCheckable"].is_array());
        let k6 = &body["checks"][5];
        assert!(k6["details"]["countByReason"]["UNBALANCED"].is_number());
    }
}

/// T-a11 — 404 pour l'exercice d'une autre société, 401 sans authentification.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn closing_checks_are_scoped_and_authenticated(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let mine = create_company(&pool, "Acme").await;
    let theirs = create_company(&pool, "Autre").await;
    let their_fy = fiscal_year_2026(&pool, theirs).await;
    let uid = create_user(&pool, "u", Role::Admin, mine).await;
    let token = forge_jwt(uid, Role::Admin.as_str(), mine);

    let resp = get(&app, Some(&token), their_fy).await;
    assert_eq!(resp.status(), 404);
    let resp = get(&app, Some(&token), i64::MAX).await;
    assert_eq!(resp.status(), 404);
    let resp = get(&app, None, their_fy).await;
    assert_eq!(resp.status(), 401);
}
