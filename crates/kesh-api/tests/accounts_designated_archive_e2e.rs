//! Story 33-8 (#486) — un compte désigné dans les paramètres de facturation ne
//! s'archive plus sans avertissement ; et un compte déjà archivé n'y verrouille
//! plus les réglages (la seconde face de l'issue).

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use common::{audit_actions, audit_details};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::entities::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::company_invoice_settings::DESIGNATED_ACCOUNT_COLUMNS;
use kesh_db::repositories::{invoices, users};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

/// La table de l'AC-1 : la colonne, et le code stable que l'API en rend.
const TEN_SETTINGS: [(&str, &str); 10] = [
    ("default_receivable_account_id", "receivable"),
    ("default_revenue_account_id", "revenue"),
    ("default_vat_payable_account_id", "vatPayable"),
    ("default_vat_recoverable_account_id", "vatRecoverable"),
    ("default_vat_decompte_account_id", "vatDecompte"),
    ("default_payable_account_id", "payable"),
    ("default_rounding_account_id", "rounding"),
    ("default_discount_account_id", "discount"),
    ("default_bank_fees_account_id", "bankFees"),
    ("default_bad_debt_account_id", "badDebt"),
];

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn archive(&self, id: i64, body: Value) -> reqwest::Response {
        self.client
            .put(self.url(&format!("/api/v1/accounts/{id}/archive")))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap()
    }
}

async fn login(base_url: &str, client: &reqwest::Client, name: &str, password: &str) -> String {
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": name, "password": password }))
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
    let token = login(&base_url, &client, "admin", ADMIN_PASSWORD).await;
    TestApp {
        base_url,
        client,
        token,
    }
}

async fn setup(pool: &MySqlPool) -> (TestApp, SeededCompany) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    // Le montage désigne d'office des comptes du seed : les tests partent de zéro
    // et désignent ce qu'ils veulent éprouver.
    clear_designations(pool, seeded.company_id).await;
    (spawn_app(pool).await, seeded)
}

async fn clear_designations(pool: &MySqlPool, company_id: i64) {
    let set = TEN_SETTINGS
        .iter()
        .map(|(c, _)| format!("{c} = NULL"))
        .collect::<Vec<_>>()
        .join(", ");
    sqlx::query(&format!(
        "UPDATE company_invoice_settings SET {set} WHERE company_id = ?"
    ))
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn designate(pool: &MySqlPool, company_id: i64, column: &str, account_id: i64) {
    sqlx::query(&format!(
        "UPDATE company_invoice_settings SET {column} = ? WHERE company_id = ?"
    ))
    .bind(account_id)
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Un compte neuf de la société, de type et de numéro donnés.
async fn new_account(pool: &MySqlPool, company_id: i64, number: &str, kind: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Compte 33-8', ?) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(kind)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn account_state(pool: &MySqlPool, id: i64) -> (bool, i32) {
    sqlx::query_as("SELECT active, version FROM accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn settings_of(body: &Value) -> Vec<String> {
    body["error"]["details"]["settings"]
        .as_array()
        .unwrap_or_else(|| panic!("details.settings attendu : {body}"))
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect()
}

/// AC-1 — chacun des dix réglages : 409, le code exact, le compte intact, aucune trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn each_of_the_ten_settings_stops_the_archive(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    for (i, (column, code)) in TEN_SETTINGS.iter().enumerate() {
        let id = new_account(&pool, seeded.company_id, &format!("95{i:02}"), "Expense").await;
        designate(&pool, seeded.company_id, column, id).await;

        let resp = app.archive(id, json!({ "version": 1 })).await;
        assert_eq!(resp.status(), 409, "{column}");
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["error"]["code"], "ACCOUNT_DESIGNATED_IN_SETTINGS");
        assert_eq!(settings_of(&body), vec![code.to_string()], "{column}");
        assert!(
            !body["error"]["message"].as_str().unwrap().is_empty(),
            "le message est traduit"
        );

        assert_eq!(account_state(&pool, id).await, (true, 1), "{column}");
        assert!(
            audit_actions(&pool, "account", id).await.is_empty(),
            "{column} : aucune trace"
        );
    }
}

/// AC-2 — deux réglages sur le même compte : les deux codes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_settings_on_one_account_are_both_named(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;
    designate(&pool, seeded.company_id, "default_bad_debt_account_id", id).await;

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(settings_of(&body), vec!["discount", "badDebt"]);
}

/// AC-3 — confirmé : archivé, tracé, le réglage reste tel quel ; et l'effet que
/// l'écran annonce est vrai : la validation d'une facture qui l'emploie est refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_confirmed_archive_goes_through_and_the_setting_stays(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let id = seeded.accounts["1100"];
    designate(
        &pool,
        seeded.company_id,
        "default_receivable_account_id",
        id,
    )
    .await;
    let (_, version) = account_state(&pool, id).await;

    let resp = app
        .archive(id, json!({ "version": version, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(resp.json::<Value>().await.unwrap()["active"], false);

    let details = audit_details(&pool, "account", id, "account.archived")
        .await
        .expect("details_json");
    assert_eq!(details["designated_settings"], json!(["receivable"]));
    let kept: Option<i64> = sqlx::query_scalar(
        "SELECT default_receivable_account_id FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(kept, Some(id), "le réglage ne bouge pas");

    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client 33-8', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let date = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let (invoice, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                revenue_account_id: Some(seeded.accounts["3000"]),
            }],
        },
    )
    .await
    .unwrap();
    assert!(
        invoices::validate_invoice(&pool, seeded.company_id, invoice.id, seeded.admin_user_id)
            .await
            .is_err(),
        "ce que dit le dialogue : l'écriture sur ce compte est refusée"
    );
}

/// AC-4 — non désigné : 200 sans confirmation, audit exactement inchangé (M6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_undesignated_account_archives_as_before(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 200);

    let details = audit_details(&pool, "account", id, "account.archived")
        .await
        .expect("details_json");
    assert!(
        details.get("designated_settings").is_none(),
        "le snapshot reste exactement celui d'avant : {details}"
    );
}

/// AC-4 — un sous-compte actif prime sur la désignation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn active_children_are_refused_first(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let parent = new_account(&pool, seeded.company_id, "9500", "Expense").await;
    let child = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    sqlx::query("UPDATE accounts SET parent_id = ? WHERE id = ?")
        .bind(parent)
        .bind(child)
        .execute(&pool)
        .await
        .unwrap();
    designate(
        &pool,
        seeded.company_id,
        "default_discount_account_id",
        parent,
    )
    .await;

    let resp = app
        .archive(parent, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION", "{body}");
    assert_eq!(account_state(&pool, parent).await, (true, 1));
}

/// AC-4 — version périmée, avec confirmation : le 409 de version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_with_confirmation_is_a_version_conflict(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    let resp = app
        .archive(id, json!({ "version": 99, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(account_state(&pool, id).await, (true, 1));
}

/// AC-4, C-e334-29 — version périmée, sans confirmation : la désignation, lue sur
/// l'état courant, parle d'abord ; sa confirmation à la même version reçoit le 409
/// de version. Le compte n'a bougé à aucun des deux (M7).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_without_confirmation_hits_the_designation_first(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    let resp = app.archive(id, json!({ "version": 99 })).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_DESIGNATED_IN_SETTINGS");
    assert_eq!(account_state(&pool, id).await, (true, 1));

    let resp = app
        .archive(id, json!({ "version": 99, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(account_state(&pool, id).await, (true, 1));
}

/// Le refus ne consomme pas la version : la confirmation part de celle de l'écran.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_refusal_does_not_consume_the_version(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    assert_eq!(app.archive(id, json!({ "version": 1 })).await.status(), 409);
    let resp = app
        .archive(id, json!({ "version": 1, "confirmDesignated": true }))
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(account_state(&pool, id).await, (false, 2));
}

/// Le Comptable, qui ne règle pas ces paramètres, n'est pas enfermé : il confirme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_comptable_may_confirm(pool: MySqlPool) {
    let (mut app, seeded) = setup(&pool).await;
    users::create(
        &pool,
        NewUser {
            username: "compta338".into(),
            password_hash: hash_password("password-e2e-33-8").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    app.token = login(&app.base_url, &app.client, "compta338", "password-e2e-33-8").await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    designate(&pool, seeded.company_id, "default_discount_account_id", id).await;

    assert_eq!(app.archive(id, json!({ "version": 1 })).await.status(), 409);
    assert_eq!(
        app.archive(id, json!({ "version": 1, "confirmDesignated": true }))
            .await
            .status(),
        200
    );
}

/// AC-5 — une désignation d'une autre société ne déclenche rien (M2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn another_companys_designation_is_ignored(pool: MySqlPool) {
    let (app, seeded) = setup(&pool).await;
    let other: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre 33-8', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // Le même numéro de compte dans l'autre société, et une désignation forcée en
    // SQL du compte de la première depuis les réglages de la seconde.
    new_account(&pool, other, "9501", "Expense").await;
    let id = new_account(&pool, seeded.company_id, "9501", "Expense").await;
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_discount_account_id) \
         VALUES (?, ?)",
    )
    .bind(other)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();

    let resp = app.archive(id, json!({ "version": 1 })).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// AC-6 — la liste égale les colonnes réelles : un onzième réglage ne se tait pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_inventory_equals_the_real_columns(pool: MySqlPool) {
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(COLUMN_NAME AS CHAR) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'company_invoice_settings' \
         AND COLUMN_NAME LIKE 'default\\_%\\_account\\_id'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        columns.len() >= 10,
        "la requête voit les colonnes : {columns:?}"
    );
    let real: BTreeSet<&str> = columns.iter().map(String::as_str).collect();
    let listed: BTreeSet<&str> = DESIGNATED_ACCOUNT_COLUMNS.iter().map(|(c, _)| *c).collect();
    assert_eq!(listed, real);
    assert_eq!(
        DESIGNATED_ACCOUNT_COLUMNS.to_vec(),
        TEN_SETTINGS.to_vec(),
        "les codes de l'API sont stables"
    );
}

/// AC-7 — la seconde face : un compte archivé par confirmation n'empêche plus
/// d'enregistrer un autre réglage (M5) ; passer **à** un compte archivé reste refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_archived_designation_no_longer_locks_the_settings(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool).await;
    let vat_payable = new_account(&pool, seeded.company_id, "9501", "Liability").await;
    designate(
        &pool,
        seeded.company_id,
        "default_vat_payable_account_id",
        vat_payable,
    )
    .await;
    assert_eq!(
        app.archive(
            vat_payable,
            json!({ "version": 1, "confirmDesignated": true })
        )
        .await
        .status(),
        200
    );

    let path = "/api/v1/company/invoice-settings";
    let current: Value = app
        .client
        .get(app.url(path))
        .bearer_auth(&app.token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(current["defaultVatPayableAccountId"], json!(vat_payable));

    let mut body = current.clone();
    body["invoiceNumberFormat"] = json!("F-{YEAR}-{SEQ:04}");
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let saved: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "{saved}");
    assert_eq!(saved["invoiceNumberFormat"], "F-{YEAR}-{SEQ:04}");

    // Passer À un autre compte archivé reste refusé.
    let other = new_account(&pool, seeded.company_id, "9502", "Liability").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    let mut body = saved.clone();
    body["defaultVatPayableAccountId"] = json!(other);
    let resp = app
        .client
        .put(app.url(path))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let refused: Value = resp.json().await.unwrap();
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("archivé"),
        "{refused}"
    );
}
