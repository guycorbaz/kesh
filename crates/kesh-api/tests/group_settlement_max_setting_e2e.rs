//! Story 43-7 (#823) — le plafond de l'écart d'un règlement groupé, réglé par
//! l'administrateur : `GET`/`PUT /company/invoice-settings`
//! (`groupSettlementMaxDifference`).
//!
//! - **AC1** : `GET` le rend en chaîne à deux décimales (5.00 par défaut ; une
//!   valeur à quatre décimales posée par SQL s'affiche arrondie au centime) ; le
//!   `PUT` l'accepte facultatif (absent : préservé), et **un `PUT` qui ne change que
//!   le plafond l'écrit** (relu, version incrémentée) — c'est le test qui rougit si
//!   le champ manque à `is_no_op_change` ;
//! - **AC2** : bornes 0, 0.01, 100.00 acceptées ; 100.01, −0.01 et 1.005 refusés en
//!   400 `VALIDATION_ERROR` qui nomme le champ ;
//! - **AC3** : l'audit `company_invoice_settings.updated` porte `{before, after}` et
//!   l'acteur utilisateur ;
//! - Administrateur seul : un Comptable reçoit 403, rien n'est écrit.
//!
//! Pré-requis : MariaDB démarré localement.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;
use std::str::FromStr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";
const SETTINGS: &str = "/api/v1/company/invoice-settings";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get(&self, path: &str) -> Value {
        let resp = self
            .client
            .get(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success(), "GET {path} : {}", resp.status());
        resp.json().await.unwrap()
    }

    async fn send(&self, method: reqwest::Method, path: &str, body: Value) -> (u16, Value) {
        let resp = self
            .client
            .request(method, self.url(path))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap();
        (
            status,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
    }

    /// Le corps d'un `PUT` des réglages tel que les lit `GET`, sans le format
    /// d'avoir (le client qui ne l'envoie pas, #216) ; l'appelant y ajoute ce
    /// qu'il change.
    async fn settings_body(&self) -> Value {
        let s = self.get(SETTINGS).await;
        json!({
            "invoiceNumberFormat": s["invoiceNumberFormat"],
            "defaultReceivableAccountId": s["defaultReceivableAccountId"],
            "defaultRevenueAccountId": s["defaultRevenueAccountId"],
            "defaultVatPayableAccountId": s["defaultVatPayableAccountId"],
            "defaultVatRecoverableAccountId": s["defaultVatRecoverableAccountId"],
            "defaultVatDecompteAccountId": s["defaultVatDecompteAccountId"],
            "defaultSalesJournal": s["defaultSalesJournal"],
            "journalEntryDescriptionTemplate": s["journalEntryDescriptionTemplate"],
            "defaultPayableAccountId": s["defaultPayableAccountId"],
            "version": s["version"],
        })
    }

    async fn put_settings(&self, body: Value) -> (u16, Value) {
        self.send(reqwest::Method::PUT, SETTINGS, body).await
    }
}

async fn login(base_url: &str, client: &reqwest::Client) -> String {
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn spawn_app(pool: &MySqlPool) -> TestApp {
    let config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
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
    let state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
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
    let base_url = format!("http://{addr}");
    let client = reqwest::Client::new();
    let token = login(&base_url, &client).await;
    TestApp {
        base_url,
        client,
        token,
    }
}

/// Le montage `seed_accounting_company` et les réglages créés.
async fn setup(pool: &MySqlPool) -> (TestApp, SeededCompany) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    kesh_db::repositories::company_invoice_settings::get_or_create_default(pool, seeded.company_id)
        .await
        .unwrap();
    (spawn_app(pool).await, seeded)
}

async fn stored(pool: &MySqlPool, company_id: i64) -> (Decimal, i32) {
    sqlx::query_as(
        "SELECT group_settlement_max_difference, version FROM company_invoice_settings \
         WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_renders_the_ceiling_at_two_decimals(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let s = app.get(SETTINGS).await;
    assert_eq!(s["groupSettlementMaxDifference"], "5.00", "défaut");

    // Quatre décimales par SQL direct (la CHECK les admet) : affichées au centime.
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 2.5049 \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let s = app.get(SETTINGS).await;
    assert_eq!(s["groupSettlementMaxDifference"], "2.50");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_put_that_changes_only_the_ceiling_writes_it(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let mut body = app.settings_body().await;
    let version = body["version"].as_i64().unwrap();
    body["groupSettlementMaxDifference"] = json!("1.00");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["groupSettlementMaxDifference"], "1.00");
    assert_eq!(resp["version"].as_i64().unwrap(), version + 1);
    let (value, db_version) = stored(&pool, seeded.company_id).await;
    assert_eq!(value, dec("1.00"));
    assert_eq!(i64::from(db_version), version + 1);

    // AC3 : l'audit porte l'ancienne et la nouvelle valeur, avec l'acteur utilisateur.
    let (actor_type, user_id, details): (String, i64, serde_json::Value) = sqlx::query_as(
        "SELECT actor_type, user_id, details_json FROM audit_log \
         WHERE action = 'company_invoice_settings.updated' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor_type, "user");
    assert_eq!(user_id, seeded.admin_user_id);
    assert_eq!(
        dec(details["before"]["groupSettlementMaxDifference"]
            .as_str()
            .unwrap()),
        dec("5.00")
    );
    assert_eq!(
        dec(details["after"]["groupSettlementMaxDifference"]
            .as_str()
            .unwrap()),
        dec("1.00")
    );

    // Renvoyer la même valeur : « sans changement », version inchangée.
    let mut again = app.settings_body().await;
    again["groupSettlementMaxDifference"] = json!("1.00");
    let (status, resp) = app.put_settings(again).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(stored(&pool, seeded.company_id).await.1, db_version);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_absent_ceiling_is_preserved(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let mut body = app.settings_body().await;
    body["groupSettlementMaxDifference"] = json!("7.25");
    assert_eq!(app.put_settings(body).await.0, 200);
    // Un client ancien n'envoie pas le champ : il ne l'efface pas.
    let mut old = app.settings_body().await;
    old["journalEntryDescriptionTemplate"] = json!("{INVOICE_NUMBER}");
    let (status, resp) = app.put_settings(old).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["groupSettlementMaxDifference"], "7.25");
    assert_eq!(stored(&pool, seeded.company_id).await.0, dec("7.25"));
}

/// Revue P1 (E1-1, A1-2) : un `PUT` sans le champ sur un plafond stocké à quatre
/// décimales le laisse intact (rien n'est arrondi au centime affiché) ; `null` de
/// même (préservé, non effacé).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_absent_or_null_ceiling_leaves_a_four_decimal_value_intact(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 5.0060 \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    // Absent : un autre réglage change, le plafond reste 5.0060.
    let mut body = app.settings_body().await;
    body["journalEntryDescriptionTemplate"] = json!("{INVOICE_NUMBER}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["groupSettlementMaxDifference"], "5.01", "affichage");
    assert_eq!(stored(&pool, seeded.company_id).await.0, dec("5.0060"));
    // `null` : préservé.
    let mut body = app.settings_body().await;
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    body["groupSettlementMaxDifference"] = Value::Null;
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(stored(&pool, seeded.company_id).await.0, dec("5.0060"));
}

/// Revue P1 (B1-2) : `GET` arrondit le demi-centime en s'éloignant de zéro.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_rounds_a_half_cent_away_from_zero(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    sqlx::query(
        "UPDATE company_invoice_settings SET group_settlement_max_difference = 5.0050 \
         WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(
        app.get(SETTINGS).await["groupSettlementMaxDifference"],
        "5.01"
    );
}

/// Revue P1 (E1-5) : les formes à zéros en trop et le zéro négatif sont acceptés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn trailing_zero_forms_and_negative_zero_are_accepted(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    for (sent, stored_value) in [
        ("5.0000", "5"),
        ("100.0000", "100"),
        ("-0", "0"),
        ("0.00", "0"),
    ] {
        let mut body = app.settings_body().await;
        body["groupSettlementMaxDifference"] = json!(sent);
        let (status, resp) = app.put_settings(body).await;
        assert_eq!(status, 200, "{sent} : {resp}");
        assert_eq!(
            stored(&pool, seeded.company_id).await.0,
            dec(stored_value),
            "{sent}"
        );
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_ceiling_bounds(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    for ok in ["0", "0.01", "100.00", "100", "12.5"] {
        let mut body = app.settings_body().await;
        body["groupSettlementMaxDifference"] = json!(ok);
        let (status, resp) = app.put_settings(body).await;
        assert_eq!(status, 200, "{ok} : {resp}");
        assert_eq!(stored(&pool, seeded.company_id).await.0, dec(ok), "{ok}");
    }
    let before = stored(&pool, seeded.company_id).await;
    for bad in ["100.01", "-0.01", "1.005", "1000", "-5"] {
        let mut body = app.settings_body().await;
        body["groupSettlementMaxDifference"] = json!(bad);
        let (status, resp) = app.put_settings(body).await;
        assert_eq!(status, 400, "{bad} : {resp}");
        assert_eq!(resp["error"]["code"], "VALIDATION_ERROR", "{bad}");
        assert!(
            resp["error"]["message"]
                .as_str()
                .unwrap()
                .contains("Écart admis"),
            "{bad} : le refus nomme le champ : {resp}"
        );
    }
    assert_eq!(stored(&pool, seeded.company_id).await, before, "rien écrit");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_comptable_cannot_set_the_ceiling(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    users::create(
        &pool,
        NewUser {
            username: "comptable".into(),
            password_hash: hash_password("comptable123").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let login: Value = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "comptable", "password": "comptable123" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login["accessToken"].as_str().unwrap().to_string();
    let mut body = app.settings_body().await;
    body["groupSettlementMaxDifference"] = json!("1.00");
    let resp = app
        .client
        .put(app.url(SETTINGS))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 403);
    assert_eq!(stored(&pool, seeded.company_id).await.0, dec("5.00"));
}
