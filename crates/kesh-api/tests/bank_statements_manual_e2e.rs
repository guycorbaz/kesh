//! Tests E2E HTTP — Story v016-2a (#816) : saisir un relevé bancaire à la main.
//!
//! `POST /api/v1/bank-imports/manual/preview` et `POST /api/v1/bank-imports/manual`
//! (AC-9 à AC-13, AC-16 à AC-18). Montage sur le squash
//! (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`). Données
//! **synthétiques** uniquement.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    ContactType, Language, NewBankAccount, NewCompany, NewContact, NewUser, OrgType, Role,
};
use kesh_db::repositories::{bank_accounts, companies, contacts as contacts_repo, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const IBAN: &str = "CH4431999123000889012";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

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
        "e2e-test-admin-password".to_string(),
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
            Err(e) => panic!("test server did not become ready within 2s: {e}"),
        }
    }
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

async fn create_account(pool: &MySqlPool, company_id: i64, number: &str, kind: &str) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, ?, ?, ?, 1, 1)",
    )
    .bind(company_id)
    .bind(number)
    .bind(format!("Compte {number}"))
    .bind(kind)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    bank_ledger_account_id: i64,
    receivable_account_id: i64,
    revenue_account_id: i64,
    expense_account_id: i64,
    fiscal_year_id: i64,
    jwt: String,
}

/// Société, utilisateur Comptable, comptes 1100/1020/3000/6810, compte
/// bancaire lié au 1020, exercice 2026 ouvert.
async fn setup(pool: &MySqlPool) -> Ctx {
    let company_id = create_company(pool, "Acme").await;
    let user_id = create_user(pool, "alice", Role::Comptable, company_id).await;
    let receivable_account_id = create_account(pool, company_id, "1100", "Asset").await;
    let bank_ledger_account_id = create_account(pool, company_id, "1020", "Asset").await;
    let revenue_account_id = create_account(pool, company_id, "3000", "Revenue").await;
    let expense_account_id = create_account(pool, company_id, "6810", "Expense").await;
    let bank_account_id = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "Banque Exemple".into(),
            iban: IBAN.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(bank_ledger_account_id)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .unwrap();
    let fiscal_year_id = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    Ctx {
        company_id,
        user_id,
        bank_account_id,
        bank_ledger_account_id,
        receivable_account_id,
        revenue_account_id,
        expense_account_id,
        fiscal_year_id,
        jwt: forge_jwt(user_id, "Comptable", company_id),
    }
}

/// Le relevé de septembre : 1000.00 − 320.25 + 50.00 = 729.75.
fn body(bank_account_id: i64) -> Value {
    json!({
        "bankAccountId": bank_account_id,
        "periodFrom": "2026-09-01",
        "periodTo": "2026-09-30",
        "statementNumber": "9/2026",
        "openingBalance": "1000.00",
        "closingBalance": "729.75",
        "lines": [
            { "bookingDate": "2026-09-03", "valueDate": null, "details": "Prélèvement assurance",
              "counterpartyName": "Assurance Exemple SA", "reference": null,
              "amount": "320.25", "direction": "debit" },
            { "bookingDate": "2026-09-15", "valueDate": "2026-09-15", "details": "Virement reçu",
              "counterpartyName": null, "reference": "F-2026-0042",
              "amount": "50.00", "direction": "credit" }
        ]
    })
}

/// Le relevé d'octobre qui suit celui de septembre : 729.75 + 100 = 829.75.
fn october(bank_account_id: i64, opening: &str, closing: &str) -> Value {
    json!({
        "bankAccountId": bank_account_id,
        "periodFrom": "2026-10-01",
        "periodTo": "2026-10-31",
        "openingBalance": opening,
        "closingBalance": closing,
        "lines": [
            { "bookingDate": "2026-10-10", "details": "Virement reçu", "amount": "100.00",
              "direction": "credit" }
        ]
    })
}

async fn post(app: &TestApp, path: &str, token: &str, body: &Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url(path))
        .bearer_auth(token)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn create(app: &TestApp, token: &str, body: &Value) -> (u16, Value) {
    post(app, "/api/v1/bank-imports/manual", token, body).await
}

async fn preview(app: &TestApp, token: &str, body: &Value) -> (u16, Value) {
    post(app, "/api/v1/bank-imports/manual/preview", token, body).await
}

/// `(imports, transactions, traces d'audit)` — ce qu'un refus ne doit pas
/// changer.
async fn counts(pool: &MySqlPool) -> (i64, i64, i64) {
    let imports: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_imports")
        .fetch_one(pool)
        .await
        .unwrap();
    let txs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions")
        .fetch_one(pool)
        .await
        .unwrap();
    (imports, txs, common::audit_count(pool).await)
}

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or("<aucun code>")
}

// ---------------------------------------------------------------------------
// AC-10 — création nominale
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_records_header_lines_and_one_trace(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (status, b) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(status, 201, "{b}");
    let id = b["id"].as_i64().unwrap();
    assert_eq!(b["sourceFormat"], "MANUAL");
    assert_eq!(b["filename"], "saisie-manuelle-2026-09-01-2026-09-30");
    assert_eq!(b["statementId"], "9/2026");
    assert_eq!(b["openingBalance"], "1000.00");
    assert_eq!(b["closingBalance"], "729.75");
    assert_eq!(b["transactionCount"], 2);
    assert_eq!(b["bankAccountId"], c.bank_account_id);
    assert_eq!(b["fileHash"].as_str().unwrap().len(), 64);

    let (source_format, period_from, period_to): (String, NaiveDate, NaiveDate) = sqlx::query_as(
        "SELECT source_format, period_from, period_to FROM bank_imports WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(source_format, "MANUAL");
    assert_eq!(
        (period_from, period_to),
        (
            NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
            NaiveDate::from_ymd_opt(2026, 9, 30).unwrap()
        )
    );

    type Row = (
        Decimal,
        String,
        String,
        String,
        Option<String>,
        Option<String>,
        Option<NaiveDate>,
    );
    let rows: Vec<Row> = sqlx::query_as(
        "SELECT amount, currency, status, details, reference, counterparty_name, value_date \
         FROM bank_transactions WHERE import_id = ? ORDER BY id",
    )
    .bind(id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].0, dec!(-320.25), "débit enregistré négatif");
    assert_eq!(rows[1].0, dec!(50.00), "crédit enregistré positif");
    assert!(rows.iter().all(|r| r.1 == "CHF" && r.2 == "pending"));
    assert_eq!(rows[0].5.as_deref(), Some("Assurance Exemple SA"));
    assert_eq!(rows[1].4.as_deref(), Some("F-2026-0042"));
    assert_eq!(rows[1].6, NaiveDate::from_ymd_opt(2026, 9, 15));

    assert_eq!(
        common::audit_actions(&pool, "bank_imports", id).await,
        vec!["bank_import.entered_manually".to_string()],
        "une seule trace, action propre"
    );
    let d = common::audit_details(&pool, "bank_imports", id, "bank_import.entered_manually")
        .await
        .unwrap();
    assert_eq!(d["filename"], "saisie-manuelle-2026-09-01-2026-09-30");
    assert_eq!(d["transaction_count"], 2);
    assert_eq!(d["source_format"], "MANUAL");
    assert_eq!(d["modifiers"], json!([]));
    assert_eq!(d["period_from"], "2026-09-01");
    assert_eq!(d["period_to"], "2026-09-30");
    assert_eq!(d["opening_balance"], "1000.00");
    assert_eq!(d["closing_balance"], "729.75");
    assert_eq!(d["statement_id"], "9/2026");
    let (actor_type, key, user, _) =
        common::audit_actor(&pool, "bank_imports", id, "bank_import.entered_manually").await;
    assert_eq!((actor_type.as_str(), key, user), ("user", None, c.user_id));
}

/// Un relevé sans ligne (mois sans mouvement) s'enregistre : son solde sert
/// la tuile et K3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_accepts_a_statement_without_lines(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let mut b = body(c.bank_account_id);
    b["lines"] = json!([]);
    b["closingBalance"] = json!("1000.00");
    let (status, r) = create(&app, &c.jwt, &b).await;
    assert_eq!(status, 201, "{r}");
    assert_eq!(r["transactionCount"], 0);
}

/// M22 — un échec de la trace n'écrit rien : un déclencheur fait échouer
/// l'insertion de `bank_import.entered_manually`, et ni l'entête ni les
/// lignes ne restent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_writes_nothing_when_the_trace_fails(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    sqlx::raw_sql(
        "CREATE TRIGGER t_v0162a_fail BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = 'bank_import.entered_manually' THEN \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = 'v016-2a atomicity'; END IF; END",
    )
    .execute(&pool)
    .await
    .unwrap();
    let before = counts(&pool).await;
    let (status, _) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(status, 500);
    assert_eq!(counts(&pool).await, before, "rien d'écrit");
    let (imports, txs, _) = counts(&pool).await;
    assert_eq!((imports, txs), (0, 0));
    sqlx::raw_sql("DROP TRIGGER t_v0162a_fail")
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(status, 201, "sans le déclencheur, la même saisie passe");
}

// ---------------------------------------------------------------------------
// AC-11 — refus, sans écriture
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_refuses_an_unbalanced_statement(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let mut b = body(c.bank_account_id);
    b["closingBalance"] = json!("729.76");
    let before = counts(&pool).await;
    for (path, label) in [
        ("/api/v1/bank-imports/manual", "création"),
        ("/api/v1/bank-imports/manual/preview", "aperçu"),
    ] {
        let (status, r) = post(&app, path, &c.jwt, &b).await;
        assert_eq!(status, 422, "{label} : {r}");
        assert_eq!(code(&r), "BANK_STATEMENT_UNBALANCED");
        assert_eq!(
            r["error"]["details"],
            json!({ "opening": "1000.00", "closing": "729.76",
                    "computedClosing": "729.75", "diff": "0.01" })
        );
        assert!(r["error"]["message"].as_str().unwrap().contains("solde"));
    }
    assert_eq!(counts(&pool).await, before);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_refuses_invalid_strings_with_field_and_reason(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let before = counts(&pool).await;
    type BodyMutation = Box<dyn Fn(&mut Value)>;
    let cases: Vec<(BodyMutation, Value)> = vec![
        (
            Box::new(|b| b["lines"][0]["direction"] = json!("out")),
            json!({ "line": 1, "field": "direction", "reason": "unknown_direction" }),
        ),
        (
            Box::new(|b| {
                b.as_object_mut().unwrap().remove("lines");
            }),
            json!({ "line": null, "field": "lines", "reason": "required" }),
        ),
        (
            Box::new(|b| b["lines"][1]["bookingDate"] = json!("2026-10-01")),
            json!({ "line": 2, "field": "bookingDate", "reason": "out_of_period" }),
        ),
        (
            Box::new(|b| b["lines"][1]["amount"] = json!("abc")),
            json!({ "line": 2, "field": "amount", "reason": "invalid_number" }),
        ),
        (
            Box::new(|b| b["periodFrom"] = json!("")),
            json!({ "line": null, "field": "periodFrom", "reason": "required" }),
        ),
        (
            Box::new(|b| b["periodFrom"] = json!("2026-10-01")),
            json!({ "line": null, "field": "periodTo", "reason": "period_inverted" }),
        ),
        (
            Box::new(|b| b["lines"][0]["details"] = json!("é".repeat(501))),
            json!({ "line": 1, "field": "details", "reason": "too_long" }),
        ),
        (
            Box::new(|b| b["lines"][1]["amount"] = json!("50.001")),
            json!({ "line": 2, "field": "amount", "reason": "too_many_decimals" }),
        ),
    ];
    for (i, (mutate, details)) in cases.into_iter().enumerate() {
        let mut b = body(c.bank_account_id);
        mutate(&mut b);
        for path in [
            "/api/v1/bank-imports/manual",
            "/api/v1/bank-imports/manual/preview",
        ] {
            let (status, r) = post(&app, path, &c.jwt, &b).await;
            assert_eq!(status, 400, "cas {i}, {path} : {r}");
            assert_eq!(code(&r), "BANK_STATEMENT_INVALID", "cas {i}");
            assert_eq!(r["error"]["details"], details, "cas {i}");
        }
    }
    assert_eq!(counts(&pool).await, before, "rien d'écrit");
}

/// Tout rejet du **corps** rend `400 VALIDATION_ERROR` sans `field` ni
/// `reason` (extracteur partagé, C-v016-2-19) — tue M24.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn body_rejections_are_400_validation_error(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let before = counts(&pool).await;
    let url = app.url("/api/v1/bank-imports/manual");

    let mut attempts: Vec<(&str, reqwest::RequestBuilder)> = Vec::new();
    attempts.push((
        "sans Content-Type",
        app.client
            .post(&url)
            .bearer_auth(&c.jwt)
            .body(body(c.bank_account_id).to_string()),
    ));
    attempts.push((
        "JSON illisible",
        app.client
            .post(&url)
            .bearer_auth(&c.jwt)
            .header("Content-Type", "application/json")
            .body("{ pas du json"),
    ));
    // Au-delà de la limite (10 Mio par défaut).
    let mut huge = body(c.bank_account_id);
    huge["lines"][0]["details"] = json!("x".repeat(11 * 1024 * 1024));
    attempts.push((
        "au-delà de la limite",
        app.client.post(&url).bearer_auth(&c.jwt).json(&huge),
    ));
    let mut number = body(c.bank_account_id);
    number["lines"][1]["amount"] = json!(50);
    attempts.push((
        "montant en nombre",
        app.client.post(&url).bearer_auth(&c.jwt).json(&number),
    ));
    let mut not_array = body(c.bank_account_id);
    not_array["lines"] = json!("non");
    attempts.push((
        "lines non tableau",
        app.client.post(&url).bearer_auth(&c.jwt).json(&not_array),
    ));
    let mut no_account = body(c.bank_account_id);
    no_account.as_object_mut().unwrap().remove("bankAccountId");
    attempts.push((
        "bankAccountId absent",
        app.client.post(&url).bearer_auth(&c.jwt).json(&no_account),
    ));
    let mut text_account = body(c.bank_account_id);
    text_account["bankAccountId"] = json!("3");
    attempts.push((
        "bankAccountId non entier",
        app.client
            .post(&url)
            .bearer_auth(&c.jwt)
            .json(&text_account),
    ));

    for (label, req) in attempts {
        let resp = req.send().await.unwrap();
        let status = resp.status().as_u16();
        let r: Value = resp.json().await.unwrap_or(Value::Null);
        assert_eq!(status, 400, "{label} : {r}");
        assert_eq!(code(&r), "VALIDATION_ERROR", "{label}");
        assert!(r["error"]["details"]["field"].is_null(), "{label}");
        assert!(r["error"]["details"]["reason"].is_null(), "{label}");
    }
    assert_eq!(counts(&pool).await, before);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn account_refusals_write_nothing(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    // Compte d'une autre société.
    let other = create_company(&pool, "Autre").await;
    let other_bank = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: other,
            bank_name: "Autre".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    let before = counts(&pool).await;
    for path in [
        "/api/v1/bank-imports/manual",
        "/api/v1/bank-imports/manual/preview",
    ] {
        let (s, r) = post(&app, path, &c.jwt, &body(other_bank)).await;
        assert_eq!(
            (s, code(&r)),
            (404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"),
            "{path}"
        );
        let (s, r) = post(&app, path, &c.jwt, &body(999_999)).await;
        assert_eq!(
            (s, code(&r)),
            (404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"),
            "{path}"
        );
    }
    // Compte clos.
    // (Un compte clos n'est pas principal : `chk_bank_accounts_closed_not_primary`.)
    sqlx::query(
        "UPDATE bank_accounts SET closed_on = '2026-08-31', is_primary = FALSE WHERE id = ?",
    )
    .bind(c.bank_account_id)
    .execute(&pool)
    .await
    .unwrap();
    for path in [
        "/api/v1/bank-imports/manual",
        "/api/v1/bank-imports/manual/preview",
    ] {
        let (s, r) = post(&app, path, &c.jwt, &body(c.bank_account_id)).await;
        assert_eq!((s, code(&r)), (409, "BANK_ACCOUNT_CLOSED"), "{path}");
    }
    // Compte archivé.
    sqlx::query("UPDATE bank_accounts SET closed_on = NULL, archived = TRUE WHERE id = ?")
        .bind(c.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    for path in [
        "/api/v1/bank-imports/manual",
        "/api/v1/bank-imports/manual/preview",
    ] {
        let (s, r) = post(&app, path, &c.jwt, &body(c.bank_account_id)).await;
        assert_eq!(
            (s, code(&r)),
            (404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"),
            "{path}"
        );
    }
    assert_eq!(counts(&pool).await, before);
}

/// Relevé identique : refus sans `confirmDuplicateFile` (tue M21) ; il est
/// aussi doublon ligne à ligne, d'où les deux confirmations.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn duplicate_statement_needs_confirmation(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (s, first) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201);
    let before = counts(&pool).await;

    let (s, r) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!((s, code(&r)), (422, "BANK_IMPORT_DUPLICATE_FILE"), "{r}");
    assert_eq!(r["error"]["details"]["existingImportId"], first["id"]);
    assert_eq!(counts(&pool).await, before);

    // Même relevé, ordre des clés et `"50"` au lieu de `"50.00"` : même empreinte.
    let mut same = body(c.bank_account_id);
    same["lines"][1]["amount"] = json!("50");
    same["statementNumber"] = json!("  9/2026 ");
    let (s, r) = create(&app, &c.jwt, &same).await;
    assert_eq!((s, code(&r)), (422, "BANK_IMPORT_DUPLICATE_FILE"), "{r}");

    let mut b = body(c.bank_account_id);
    b["confirmDuplicateFile"] = json!(true);
    let (s, r) = create(&app, &c.jwt, &b).await;
    assert_eq!(
        (s, code(&r)),
        (422, "BANK_STATEMENT_DUPLICATE_LINES"),
        "{r}"
    );
    assert_eq!(counts(&pool).await, before);

    b["confirmDuplicateLines"] = json!(true);
    let (s, r) = create(&app, &c.jwt, &b).await;
    assert_eq!(s, 201, "{r}");
    assert_eq!(r["transactionCount"], 2, "toutes les lignes enregistrées");
    let id = r["id"].as_i64().unwrap();
    let d = common::audit_details(&pool, "bank_imports", id, "bank_import.entered_manually")
        .await
        .unwrap();
    assert_eq!(
        d["modifiers"],
        json!(["duplicate_file", "duplicate_lines_imported"])
    );
    assert_eq!(d["duplicate_lines_imported"], 2);
}

/// Doublons de lignes : refus avec les rangs (tue M5), jamais ignorés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn duplicate_lines_are_refused_then_all_recorded(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (s, first) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201);
    let existing: i64 = sqlx::query_scalar(
        "SELECT id FROM bank_transactions WHERE import_id = ? AND amount = 50.00",
    )
    .bind(first["id"].as_i64().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    // Un relevé différent (autre numéro, autre période jointive) qui reprend
    // la ligne du 15 septembre en deuxième position.
    let b = json!({
        "bankAccountId": c.bank_account_id,
        "periodFrom": "2026-09-15", "periodTo": "2026-09-30",
        "statementNumber": "9b/2026",
        "openingBalance": "0", "closingBalance": "60.00",
        "lines": [
            { "bookingDate": "2026-09-16", "details": "Autre", "amount": "10.00", "direction": "credit" },
            { "bookingDate": "2026-09-15", "details": "Virement reçu", "reference": "F-2026-0042",
              "amount": "50.00", "direction": "credit" }
        ]
    });
    let before = counts(&pool).await;
    let (s, r) = create(&app, &c.jwt, &b).await;
    assert_eq!(
        (s, code(&r)),
        (422, "BANK_STATEMENT_DUPLICATE_LINES"),
        "{r}"
    );
    assert_eq!(
        r["error"]["details"]["lines"],
        json!([{ "line": 2, "existingTransactionId": existing }])
    );
    assert_eq!(counts(&pool).await, before, "rien d'écrit");

    let mut b = b;
    b["confirmDuplicateLines"] = json!(true);
    let (s, r) = create(&app, &c.jwt, &b).await;
    assert_eq!(s, 201, "{r}");
    assert_eq!(r["transactionCount"], 2);
    let n: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?")
        .bind(r["id"].as_i64().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(n, 2, "la ligne en doublon est enregistrée, non ignorée");
}

/// Écart de solde de départ : refus sans confirmation (tue M6), puis trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn opening_balance_gap_needs_confirmation(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (s, sept) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201);

    // Le solde attendu : aucun écart, aucune confirmation.
    let (s, r) = create(
        &app,
        &c.jwt,
        &october(c.bank_account_id, "729.75", "829.75"),
    )
    .await;
    assert_eq!(s, 201, "{r}");
    let delete = app
        .client
        .delete(app.url(&format!("/api/v1/bank-imports/{}", r["id"])))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(delete.status(), 204);

    let before = counts(&pool).await;
    let gap = october(c.bank_account_id, "700.00", "800.00");
    let (s, r) = create(&app, &c.jwt, &gap).await;
    assert_eq!(
        (s, code(&r)),
        (422, "BANK_STATEMENT_OPENING_BALANCE_GAP"),
        "{r}"
    );
    assert_eq!(
        r["error"]["details"],
        json!({ "expected": "729.75", "entered": "700.00",
                "previousImportId": sept["id"], "previousPeriodTo": "2026-09-30" })
    );
    assert_eq!(counts(&pool).await, before);

    let mut gap = gap;
    gap["confirmOpeningBalanceGap"] = json!(true);
    let (s, r) = create(&app, &c.jwt, &gap).await;
    assert_eq!(s, 201, "{r}");
    let d = common::audit_details(
        &pool,
        "bank_imports",
        r["id"].as_i64().unwrap(),
        "bank_import.entered_manually",
    )
    .await
    .unwrap();
    assert_eq!(d["modifiers"], json!(["opening_balance_gap"]));
    assert_eq!(d["expected_opening_balance"], "729.75");
    assert_eq!(d["statement_id"], Value::Null);
}

// ---------------------------------------------------------------------------
// AC-12 — l'aperçu n'écrit rien et rend ses avertissements
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn preview_writes_nothing_and_warns(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;

    // Sans relevé antérieur : sommes, aucun avertissement.
    let before = counts(&pool).await;
    let (s, r) = preview(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 200, "{r}");
    assert_eq!(r["totalCredits"], "50.00");
    assert_eq!(r["totalDebits"], "320.25");
    assert_eq!(r["computedClosingBalance"], "729.75");
    assert_eq!(r["transactionCount"], 2);
    assert_eq!(
        r["warnings"],
        json!({ "duplicateFile": null, "duplicateLines": [],
                "openingBalanceGap": null, "overlappingStatements": [] })
    );
    assert_eq!(counts(&pool).await, before, "l'aperçu n'écrit rien (M12)");

    // Après la saisie de septembre : le même relevé à l'aperçu porte trois
    // avertissements, l'aperçu d'octobre décalé le quatrième.
    let (_, sept) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    let before = counts(&pool).await;
    let (s, r) = preview(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 200, "{r}");
    let w = &r["warnings"];
    assert_eq!(w["duplicateFile"]["existingImportId"], sept["id"]);
    assert_eq!(w["duplicateLines"].as_array().unwrap().len(), 2);
    assert_eq!(w["duplicateLines"][0]["line"], 1);
    assert_eq!(
        w["overlappingStatements"],
        json!([{ "importId": sept["id"], "periodFrom": "2026-09-01",
                 "periodTo": "2026-09-30", "sourceFormat": "MANUAL" }])
    );
    assert_eq!(w["openingBalanceGap"], Value::Null);

    let (s, r) = preview(
        &app,
        &c.jwt,
        &october(c.bank_account_id, "700.00", "800.00"),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    assert_eq!(
        r["warnings"]["openingBalanceGap"],
        json!({ "expected": "729.75", "entered": "700.00",
                "previousImportId": sept["id"], "previousPeriodTo": "2026-09-30" })
    );
    assert_eq!(counts(&pool).await, before, "l'aperçu n'écrit rien");
}

// ---------------------------------------------------------------------------
// AC-13 — rôles et clés
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn roles_and_api_keys(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let viewer = create_user(&pool, "bob", Role::Consultation, c.company_id).await;
    let viewer_jwt = forge_jwt(viewer, "Consultation", c.company_id);
    let before = counts(&pool).await;
    for path in [
        "/api/v1/bank-imports/manual",
        "/api/v1/bank-imports/manual/preview",
    ] {
        let (s, _) = post(&app, path, &viewer_jwt, &body(c.bank_account_id)).await;
        assert_eq!(s, 403, "Consultation : {path}");
    }
    assert_eq!(counts(&pool).await, before);

    let (_, read_key) =
        common::create_key_via_http(&app.client, &app.base_url, &c.jwt, "lecture", "read").await;
    let (s, r) = create(&app, &read_key, &body(c.bank_account_id)).await;
    assert_eq!((s, code(&r)), (403, "API_KEY_READ_ONLY"), "{r}");

    let (rw_id, rw_key) =
        common::create_key_via_http(&app.client, &app.base_url, &c.jwt, "ecriture", "read-write")
            .await;
    let (s, r) = create(&app, &rw_key, &body(c.bank_account_id)).await;
    assert_eq!(s, 201, "{r}");
    let id = r["id"].as_i64().unwrap();
    let (_, key, _, _) =
        common::audit_actor(&pool, "bank_imports", id, "bank_import.entered_manually").await;
    assert_eq!(key, Some(rw_id), "la trace porte la clé");

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-imports/{id}")))
        .bearer_auth(&rw_key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
    let r: Value = resp.json().await.unwrap();
    assert_eq!(code(&r), "API_KEY_OPERATION_FORBIDDEN");
}

// ---------------------------------------------------------------------------
// AC-17 — suppression et ressaisie
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_then_reenter_without_confirmation(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let (_, r) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    let id = r["id"].as_i64().unwrap();
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-imports/{id}")))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let (imports, txs, _) = counts(&pool).await;
    assert_eq!((imports, txs), (0, 0));
    let d = common::audit_details(&pool, "bank_imports", id, "bank_import.deleted")
        .await
        .unwrap();
    assert_eq!(d["source_format"], "MANUAL");

    let (s, r) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201, "ressaisie sans confirmDuplicateFile : {r}");
}

// ---------------------------------------------------------------------------
// AC-16 — le circuit du rapprochement, sur un relevé MANUAL
// ---------------------------------------------------------------------------

async fn seed_validated_invoice(c: &Ctx, pool: &MySqlPool, amount: Decimal) -> i64 {
    let contact_id = contacts_repo::create(
        pool,
        c.user_id,
        NewContact {
            company_id: c.company_id,
            contact_type: ContactType::Entreprise,
            name: "Client Exemple SA".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    let je_id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description, version, created_at, updated_at) \
         VALUES (?, ?, 1, '2026-08-20', 'Ventes', 'Vente', 1, NOW(3), NOW(3))",
    )
    .bind(c.company_id)
    .bind(c.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (order, account, debit, credit) in [
        (1, c.receivable_account_id, amount, Decimal::ZERO),
        (2, c.revenue_account_id, Decimal::ZERO, amount),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je_id)
        .bind(account)
        .bind(order)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .unwrap();
    }
    let invoice_id = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id, paid_at, version, created_at, updated_at) \
         VALUES (?, ?, 'F-2026-0042', 'validated', '2026-08-20', ?, ?, NULL, 1, NOW(3), NOW(3))",
    )
    .bind(c.company_id)
    .bind(contact_id)
    .bind(amount)
    .bind(je_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, \
         vat_rate, line_total) VALUES (?, 1, 'Ligne', 1, ?, 0, ?)",
    )
    .bind(invoice_id)
    .bind(amount)
    .bind(amount)
    .execute(pool)
    .await
    .unwrap();
    invoice_id
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn manual_statement_lines_reconcile_like_imported_ones(pool: MySqlPool) {
    let c = setup(&pool).await;
    let invoice_id = seed_validated_invoice(&c, &pool, dec!(50.00)).await;
    let app = spawn_app(pool.clone()).await;
    let (s, r) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201, "{r}");
    let import_id = r["id"].as_i64().unwrap();
    let ids: Vec<(i64, Decimal)> =
        sqlx::query_as("SELECT id, amount FROM bank_transactions WHERE import_id = ? ORDER BY id")
            .bind(import_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    let (debit_tx, credit_tx) = (ids[0].0, ids[1].0);

    // Proposition : la facture de 50.00 est candidate de la ligne au crédit.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            c.bank_account_id
        )))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let p: Value = resp.json().await.unwrap();
    let proposal = p["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|x| x["bankTransactionId"] == credit_tx)
        .expect("la ligne saisie au crédit est proposée");
    assert!(
        proposal["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cand| cand["invoiceId"] == invoice_id),
        "la facture est candidate : {proposal}"
    );

    // Acceptation : la transaction passe `reconciled`, la facture est réglée.
    let (s, r) = post(
        &app,
        "/api/v1/reconciliation/accept",
        &c.jwt,
        &json!({ "bankAccountId": c.bank_account_id,
                 "proposals": [{ "type": "invoice", "bankTransactionId": credit_tx,
                                 "invoiceId": invoice_id }] }),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    assert_eq!(r["accepted"].as_array().unwrap().len(), 1, "{r}");
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(credit_tx)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "reconciled");
    let paid: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT paid_at FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(paid.is_some(), "la facture est réglée");

    // Le relevé rapproché ne se supprime pas.
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-imports/{import_id}")))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let r: Value = resp.json().await.unwrap();
    assert_eq!(code(&r), "BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS");

    // Annulation du rapprochement.
    let (s, r) = post(
        &app,
        &format!("/api/v1/reconciliation/transactions/{credit_tx}/cancel"),
        &c.jwt,
        &json!({}),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(credit_tx)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "pending");

    // Rapprochement manuel de la ligne au débit avec un compte de charges.
    let (s, r) = post(
        &app,
        "/api/v1/reconciliation/manual",
        &c.jwt,
        &json!({ "bankAccountId": c.bank_account_id, "bankTransactionId": debit_tx,
                 "counterpartyAccountId": c.expense_account_id,
                 "description": "Assurance septembre" }),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    let (status, matched): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(debit_tx)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(status, "reconciled");
    assert!(matched.is_some());
}

// ---------------------------------------------------------------------------
// AC-18 — le solde du relevé : tuile et contrôle K3
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn manual_statement_feeds_the_tile_and_k3(pool: MySqlPool) {
    let c = setup(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let k3 = |app: &TestApp| {
        app.client
            .get(app.url(&format!(
                "/api/v1/fiscal-years/{}/closing-checks",
                c.fiscal_year_id
            )))
            .bearer_auth(&c.jwt)
            .send()
    };
    // Montage : une écriture sur le compte lié, pour que K3 examine le compte ;
    // sans relevé à solde, il est « non vérifiable » (NO_BALANCE_STATEMENT).
    let je = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 1, '2026-09-10', 'Banque', 'Apport')",
    )
    .bind(c.company_id)
    .bind(c.fiscal_year_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (order, account, debit, credit) in [
        (1, c.bank_ledger_account_id, dec!(729.75), Decimal::ZERO),
        (2, c.revenue_account_id, Decimal::ZERO, dec!(729.75)),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je)
        .bind(account)
        .bind(order)
        .bind(debit)
        .bind(credit)
        .execute(&pool)
        .await
        .unwrap();
    }
    let before: Value = k3(&app).await.unwrap().json().await.unwrap();
    assert!(
        before.to_string().contains("NO_BALANCE_STATEMENT"),
        "montage : sans relevé, NO_BALANCE_STATEMENT — {before}"
    );

    let (s, r) = create(&app, &c.jwt, &body(c.bank_account_id)).await;
    assert_eq!(s, 201, "{r}");

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let accounts: Value = resp.json().await.unwrap();
    let list = accounts
        .as_array()
        .cloned()
        .or_else(|| accounts["items"].as_array().cloned())
        .expect("liste des comptes");
    let a = list
        .iter()
        .find(|a| a["id"] == c.bank_account_id)
        .expect("le compte");
    assert_eq!(a["statementClosingBalance"], "729.75", "{a}");
    assert_eq!(a["statementDate"], "2026-09-30", "{a}");
    assert_eq!(a["statementGap"], Value::Null, "livres = relevé : {a}");

    let after: Value = k3(&app).await.unwrap().json().await.unwrap();
    assert!(
        !after.to_string().contains("NO_BALANCE_STATEMENT"),
        "après la saisie, plus de NO_BALANCE_STATEMENT — {after}"
    );
}
