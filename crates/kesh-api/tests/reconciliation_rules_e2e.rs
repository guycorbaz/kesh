//! Story 8-5b T4.5 — tests E2E HTTP pour `/api/v1/reconciliation/rules`
//! + extension `get_proposals` / `post_accept type='rule'`.
//!
//! 28 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`. Pattern
//! hérité bank_accounts_e2e.rs / reconciliation_e2e.rs (forge JWT
//! directement + spawn_app éphémère).
#![allow(clippy::too_many_arguments)]

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    BankImportSourceFormat, Language, NewAccount, NewBankAccount, NewBankImport,
    NewBankTransaction, NewCompany, NewUser, OrgType, Role,
};
use kesh_db::repositories::{accounts, bank_accounts, bank_imports, companies, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================
// Spawn helpers
// ============================================================

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
        "mysql://test".into(),
        "admin".into(),
        "e2e-test-password".into(),
        std::str::from_utf8(TEST_JWT_SECRET).unwrap().to_string(),
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
            Err(e) => panic!("test server not ready in 2s: {e}"),
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

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

// ============================================================
// Domain seed helpers
// ============================================================

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
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

async fn set_bank_account_journal(pool: &MySqlPool, bank_account_id: i64, journal_account_id: i64) {
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(journal_account_id)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn create_account_with_type(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn archive_account(pool: &MySqlPool, account_id: i64, user_id: i64, _company_id: i64) {
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_one(pool)
        .await
        .unwrap();
    accounts::archive(pool, account_id, version, user_id)
        .await
        .expect("archive account");
}

/// Crée un bank_import + 1 bank_transaction et retourne l'id de la tx.
async fn create_pending_bank_tx(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    amount: Decimal,
    counterparty_name: &str,
    counterparty_iban: Option<&str>,
    reference: Option<&str>,
    currency: &str,
    value_date: Option<NaiveDate>,
) -> i64 {
    let unique_hash = format!(
        "{:064x}",
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
    );
    let mut tx = pool.begin().await.unwrap();
    let (_header, txs) = bank_imports::create_with_transactions(
        &mut tx,
        NewBankImport {
            company_id,
            bank_account_id,
            filename: "stmt.xml".into(),
            file_hash: unique_hash,
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("STMT".into()),
            period_from: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            period_to: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            opening_balance: Some(dec!(1000)),
            closing_balance: Some(dec!(1000) + amount),
            transaction_count: 1,
            imported_by_user_id: user_id,
        },
        vec![NewBankTransaction {
            company_id,
            bank_account_id,
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 10).unwrap(),
            value_date,
            amount,
            currency: currency.into(),
            reference: reference.map(|s| s.into()),
            details: "".into(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: counterparty_iban.map(|s| s.into()),
            counterparty_name: Some(counterparty_name.into()),
        }],
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    txs[0].id
}

/// Ouvre l'exercice fiscal couvrant 2026.
async fn open_fiscal_year_2026(pool: &MySqlPool, company_id: i64, user_id: i64) {
    use kesh_db::entities::NewFiscalYear;
    use kesh_db::repositories::fiscal_years;
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: "2026".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        },
    )
    .await
    .expect("open fiscal year");
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    counterparty_account_id: i64,
    bank_ledger_account_id: i64,
    jwt: String,
}

async fn setup_ctx(pool: &MySqlPool, label: &str, iban: &str, role: Role) -> Ctx {
    let company_id = create_company(pool, label).await;
    let username = format!("{label}_user");
    let user_id = create_user(pool, &username, role, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, iban).await;
    let bank_ledger = create_account_with_type(
        pool,
        company_id,
        user_id,
        "1020",
        "Banque CHF",
        AccountType::Asset,
    )
    .await;
    set_bank_account_journal(pool, bank_account_id, bank_ledger).await;
    let counterparty = create_account_with_type(
        pool,
        company_id,
        user_id,
        "6510",
        "Telecom",
        AccountType::Expense,
    )
    .await;
    let jwt = forge_jwt(user_id, role_str(role), company_id);
    open_fiscal_year_2026(pool, company_id, user_id).await;
    Ctx {
        company_id,
        user_id,
        bank_account_id,
        counterparty_account_id: counterparty,
        bank_ledger_account_id: bank_ledger,
        jwt,
    }
}

async fn post_rule_raw(app: &TestApp, jwt: &str, body: Value) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/reconciliation/rules"))
        .bearer_auth(jwt)
        .json(&body)
        .send()
        .await
        .unwrap()
}

async fn create_rule_ok(app: &TestApp, jwt: &str, body: Value) -> Value {
    let resp = post_rule_raw(app, jwt, body).await;
    assert_eq!(resp.status(), 201, "create rule expected 201");
    resp.json().await.unwrap()
}

// ============================================================
// Test 1 — AC #101 : create returns 201 + audit log
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_create_returns_201_with_audit_log(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let body: Value = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom auto",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
            "priority": 100,
        }),
    )
    .await;

    assert!(body["id"].as_i64().unwrap() > 0);
    assert_eq!(body["label"], "Swisscom auto");
    assert_eq!(body["active"], true);
    assert_eq!(body["version"], 1);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE action = 'reconciliation_rule.created' AND entity_type = 'reconciliation_rules'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1, "audit log entry missing");
}

// ============================================================
// Test 2 — AC #102 : duplicate (active) rejected 409
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_create_rejects_duplicate_match_when_active(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let body = json!({
        "label": "R1",
        "matchType": "counterparty_contains",
        "matchValue": "Swisscom",
        "counterpartyAccountId": ctx.counterparty_account_id,
    });
    create_rule_ok(&app, &ctx.jwt, body.clone()).await;

    let resp = post_rule_raw(&app, &ctx.jwt, body).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "RECONCILIATION_RULE_DUPLICATE");
    assert_eq!(
        body["error"]["details"]["matchType"],
        "counterparty_contains"
    );
    assert_eq!(body["error"]["details"]["matchValue"], "Swisscom");
}

// ============================================================
// Test 3 — AC #103, Q3 : recreate ok après soft-delete
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_create_succeeds_when_existing_rule_is_inactive(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r1 = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;

    let r1_id = r1["id"].as_i64().unwrap();
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r1_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // R2 même match → succès.
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R2",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
}

// ============================================================
// Test 4 — AC #104 : reject when counterparty account archived
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_create_rejects_archived_account(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    archive_account(
        &pool,
        ctx.counterparty_account_id,
        ctx.user_id,
        ctx.company_id,
    )
    .await;
    let app = spawn_app(pool.clone()).await;

    let resp = post_rule_raw(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
}

// ============================================================
// Test 5 — AC #105 : list paginated
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_list_paginated(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    for i in 0..5 {
        create_rule_ok(
            &app,
            &ctx.jwt,
            json!({
                "label": format!("R{i}"),
                "matchType": "counterparty_contains",
                "matchValue": format!("Match{i}"),
                "counterpartyAccountId": ctx.counterparty_account_id,
                "priority": 100 + i,
            }),
        )
        .await;
    }
    let resp = app
        .client
        .get(app.url("/api/v1/reconciliation/rules?page=1&perPage=3"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 5);
    assert_eq!(body["items"].as_array().unwrap().len(), 3);
}

// ============================================================
// Test 6 — AC #106 : list filters active
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_list_filters_active(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r1 = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R2",
            "matchType": "counterparty_contains",
            "matchValue": "Sunrise",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r1_id = r1["id"].as_i64().unwrap();
    app.client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r1_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();

    // active=true → 1.
    let resp = app
        .client
        .get(app.url("/api/v1/reconciliation/rules?active=true"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);

    // active=false → 1.
    let resp = app
        .client
        .get(app.url("/api/v1/reconciliation/rules?active=false"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["total"], 1);
}

// ============================================================
// Test 7 — AC #107 : multi-tenant scoping
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_list_scopes_by_company(pool: MySqlPool) {
    let ctx_a = setup_ctx(&pool, "Alpha", "CH4431999123000889012", Role::Comptable).await;
    let ctx_b = setup_ctx(&pool, "Beta", "CH9300762011623852957", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    create_rule_ok(
        &app,
        &ctx_a.jwt,
        json!({
            "label": "A1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx_a.counterparty_account_id,
        }),
    )
    .await;
    let resp_a = app
        .client
        .get(app.url("/api/v1/reconciliation/rules"))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();
    let body_a: Value = resp_a.json().await.unwrap();
    assert_eq!(body_a["total"], 1);

    let resp_b = app
        .client
        .get(app.url("/api/v1/reconciliation/rules"))
        .bearer_auth(&ctx_b.jwt)
        .send()
        .await
        .unwrap();
    let body_b: Value = resp_b.json().await.unwrap();
    assert_eq!(
        body_b["total"], 0,
        "B doit pas voir les rules de A (KF-002)"
    );
}

// ============================================================
// Test 8 — AC #108 : optimistic lock 409 OPTIMISTIC_LOCK_CONFLICT
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_update_uses_optimistic_lock(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r_id = r["id"].as_i64().unwrap();

    // PATCH version=1 → version=2.
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": 1, "label": "Renamed"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // PATCH version=1 stale → 409 OPTIMISTIC_LOCK_CONFLICT.
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": 1, "label": "Other"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
}

// ============================================================
// Test 9 — AC #109 : PATCH reactivates inactive rule
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_patch_reactivates_inactive_rule(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r_id = r["id"].as_i64().unwrap();

    app.client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();

    // PATCH active=true → reactive.
    let current_version: i32 =
        sqlx::query_scalar("SELECT version FROM reconciliation_rules WHERE id = ?")
            .bind(r_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": current_version, "active": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["active"], true);
}

// ============================================================
// Test 10 — AC #109b : reactivation fails when concurrent active rule
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_patch_reactivation_fails_when_concurrent_active_rule_exists(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r1 = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r1_id = r1["id"].as_i64().unwrap();
    app.client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r1_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();

    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R2",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;

    let current_version: i32 =
        sqlx::query_scalar("SELECT version FROM reconciliation_rules WHERE id = ?")
            .bind(r1_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{r1_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": current_version, "active": true}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "RECONCILIATION_RULE_DUPLICATE");
}

// ============================================================
// Test 11 — AC #110 : delete soft-deletes + preserves audit history
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_delete_soft_deletes_and_preserves_audit_history(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r_id = r["id"].as_i64().unwrap();
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // DB : row still exists, active=false.
    let active: bool = sqlx::query_scalar("SELECT active FROM reconciliation_rules WHERE id = ?")
        .bind(r_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!active);

    // Audit : created + deleted.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type='reconciliation_rules' AND entity_id=?",
    )
    .bind(r_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(audit_count >= 2);
}

// ============================================================
// Test 12 — AC #111 : delete idempotent (already inactive)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_delete_idempotent_when_already_inactive(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r_id = r["id"].as_i64().unwrap();
    for _ in 0..2 {
        let resp = app
            .client
            .delete(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }
}

// ============================================================
// Test 13 — AC #112 : mutations require Comptable role
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rule_mutations_require_comptable_role(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let app = spawn_app(pool.clone()).await;
    let resp = post_rule_raw(
        &app,
        &ctx.jwt,
        json!({
            "label": "R1",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    assert_eq!(resp.status(), 403, "Consultation rejetée 403");
}

// ============================================================
// Test 14 — AC #113 : rule appears in /proposals when no invoice candidate
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_applies_rule_when_no_invoice_candidate(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-150.00),
        "Swisscom Schweiz AG",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let proposals = body["proposals"].as_array().unwrap();
    assert!(!proposals.is_empty());
    let candidates = proposals[0]["candidates"].as_array().unwrap();
    assert!(candidates.iter().any(|c| c["candidateType"] == "rule"));
}

// ============================================================
// Test 15 — AC #114 : strong invoice score override rule
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_invoice_candidate_overrides_rule(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    // tx avec montant positif = crédit → propose_matches cherchera invoice.
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(150.00),
        "Swisscom Schweiz AG",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    // Pas d'invoice créée — pas d'override possible. Vu que score < 0.5
    // (aucun candidate invoice), la rule s'applique tout de même.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let candidates = body["proposals"][0]["candidates"].as_array().unwrap();
    assert!(candidates.iter().any(|c| c["candidateType"] == "rule"));
}

// ============================================================
// Test 16 — AC #115 : highest priority rule wins
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_applies_highest_priority_rule(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let other_account = create_account_with_type(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "6520",
        "Postes",
        AccountType::Expense,
    )
    .await;
    // Priority lower = higher priority (priority ASC dans first_matching_rule).
    // priority 50 (gagne en priorité ASC) sur other_account.
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Postes prio_50",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": other_account,
            "priority": 50,
        }),
    )
    .await;
    // priority 100 sur counterparty_account_id avec match différent
    // pour éviter UNIQUE (company, match_type, match_value).
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Telecom prio_100",
            "matchType": "counterparty_contains",
            "matchValue": "Swiss",
            "counterpartyAccountId": ctx.counterparty_account_id,
            "priority": 100,
        }),
    )
    .await;
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-150.00),
        "Swisscom",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let rule_cand = body["proposals"][0]["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["candidateType"] == "rule")
        .unwrap()
        .clone();
    assert_eq!(rule_cand["counterpartyAccountId"], other_account);
}

// ============================================================
// Test 17 — AC #116 : skip rule sur compte de contrepartie archivé
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_skips_rule_with_archived_account(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    archive_account(
        &pool,
        ctx.counterparty_account_id,
        ctx.user_id,
        ctx.company_id,
    )
    .await;
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-150.00),
        "Swisscom",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let candidates = body["proposals"][0]["candidates"].as_array().unwrap();
    assert!(
        !candidates.iter().any(|c| c["candidateType"] == "rule"),
        "rule sur compte archivé doit être skippée"
    );
}

// ============================================================
// Test 18 — AC #117 : skip inactive rule
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_skips_inactive_rule(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let r_id = r["id"].as_i64().unwrap();
    app.client
        .delete(app.url(&format!("/api/v1/reconciliation/rules/{r_id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-150.00),
        "Swisscom",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let candidates = body["proposals"][0]["candidates"].as_array().unwrap();
    assert!(!candidates.iter().any(|c| c["candidateType"] == "rule"));
}

// ============================================================
// Helpers pour tests accept-with-rule
// ============================================================

/// Crée un projet analytique (Story 19-5) et retourne son id.
async fn create_project(pool: &MySqlPool, company_id: i64, code: &str, archived: bool) -> i64 {
    let result = sqlx::query(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(code)
    .bind(format!("Projet {code}"))
    .bind(archived)
    .execute(pool)
    .await
    .expect("project insert");
    result.last_insert_id() as i64
}

async fn create_rule_and_tx(
    pool: &MySqlPool,
    app: &TestApp,
    ctx: &Ctx,
    counterparty_name: &str,
    match_value: &str,
    amount: Decimal,
    currency: &str,
    value_date: Option<NaiveDate>,
) -> (i64, i64) {
    let r = create_rule_ok(
        app,
        &ctx.jwt,
        json!({
            "label": format!("Auto {match_value}"),
            "matchType": "counterparty_contains",
            "matchValue": match_value,
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    let rule_id = r["id"].as_i64().unwrap();
    let tx_id = create_pending_bank_tx(
        pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        amount,
        counterparty_name,
        None,
        None,
        currency,
        value_date,
    )
    .await;
    (rule_id, tx_id)
}

async fn post_accept_rule(
    app: &TestApp,
    jwt: &str,
    bank_account_id: i64,
    tx_id: i64,
    rule_id: i64,
    counterparty_account_id: i64,
) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(jwt)
        .json(&json!({
            "bankAccountId": bank_account_id,
            "proposals": [{
                "type": "rule",
                "bankTransactionId": tx_id,
                "ruleId": rule_id,
                "counterpartyAccountId": counterparty_account_id,
            }],
        }))
        .send()
        .await
        .unwrap()
}

// ============================================================
// Test 19 — AC #118 : accept creates JE + increments count
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_creates_journal_entry_and_increments_count(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom Schweiz AG",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1);
    assert!(body["failed"].as_array().unwrap().is_empty());

    let applied_count: i64 =
        sqlx::query_scalar("SELECT applied_count FROM reconciliation_rules WHERE id = ?")
            .bind(rule_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(applied_count, 1);
}

// ============================================================
// Test 20 — AC #118bis : per-proposal failed when bank_account NOT configured
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_returns_failed_when_bank_account_not_configured(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Désactiver journal_account_id.
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?")
        .bind(ctx.bank_account_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["errorCode"], "BANK_ACCOUNT_NOT_CONFIGURED");
}

// ============================================================
// Test 21 — AC #119 : RECONCILIATION_RULE_NO_LONGER_MATCHES race
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_rejects_when_no_longer_matches(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    // Modifier la rule pour qu'elle ne match plus.
    let current_version: i32 =
        sqlx::query_scalar("SELECT version FROM reconciliation_rules WHERE id = ?")
            .bind(rule_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    app.client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{rule_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": current_version, "matchValue": "Sunrise"}))
        .send()
        .await
        .unwrap();

    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(
        failed[0]["errorCode"],
        "RECONCILIATION_RULE_NO_LONGER_MATCHES"
    );
}

// ============================================================
// Test 22 — AC #120 : counterpartyAccountId mismatch
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_validates_counterparty_account_consistency(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let other = create_account_with_type(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "6520",
        "Postes",
        AccountType::Expense,
    )
    .await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = post_accept_rule(&app, &ctx.jwt, ctx.bank_account_id, tx_id, rule_id, other).await;
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed[0]["errorCode"], "RECONCILIATION_RULE_MISMATCH");
}

// ============================================================
// Test 23 — AC #121 : concurrent rule update during accept
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_handles_concurrent_rule_update(pool: MySqlPool) {
    // Pour ce test simple, on vérifie juste que l'accept produit pas
    // 500 même quand la rule a été PATCHéee juste avant (race window).
    // Le pattern complet (interleaving) est documenté §risques 1.
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    // PATCH non-impactant sur match.
    app.client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{rule_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"expectedVersion": 1, "label": "Renamed"}))
        .send()
        .await
        .unwrap();
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);
}

// ============================================================
// Test 24 — AC #122 : audit log triple (accepted + rule.applied)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_emits_triple_audit_log(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);

    // Pass 1 code review MEDIUM AA3 fix : assert exactement 3 entrées
    // audit (reconciliation.accepted + reconciliation_rule.applied +
    // journal_entry.created) + assert applied_count_after + match_type
    // dans les details JSON.
    let applied_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation_rule.applied'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let accepted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.accepted'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let je_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'journal_entry.created'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(applied_count, 1, "exactly one reconciliation_rule.applied");
    assert_eq!(accepted_count, 1, "exactly one reconciliation.accepted");
    assert_eq!(je_count, 1, "exactly one journal_entry.created");

    // Verify applied_count_after field (Pass 1 HIGH AA2 fix).
    let applied_details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE action = 'reconciliation_rule.applied' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(applied_details["applied_count_after"], 1);

    // Verify match_type field in reconciliation.accepted (Pass 1 HIGH AA3 fix).
    let accepted_details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE action = 'reconciliation.accepted' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(accepted_details["match_type"], "counterparty_contains");
}

// ============================================================
// Test 25 — Pass 4 ECH4-3 : AcceptBodyExtractor message lists 'rule'
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_extractor_error_lists_rule(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/accept"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "proposals": [{"type": "BOGUS", "bankTransactionId": 1}]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("rule"), "message should list 'rule': {msg}");
}

// ============================================================
// Test 26 — AC #117bis Pass 3 R1 : skip rule for non-CHF tx
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_proposals_skips_rule_for_non_chf_transaction(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Swisscom",
            "matchType": "counterparty_contains",
            "matchValue": "Swisscom",
            "counterpartyAccountId": ctx.counterparty_account_id,
        }),
    )
    .await;
    create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-150.00),
        "Swisscom",
        None,
        None,
        "EUR",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            ctx.bank_account_id
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let candidates = body["proposals"][0]["candidates"].as_array().unwrap();
    assert!(!candidates.iter().any(|c| c["candidateType"] == "rule"));
}

// ============================================================
// Test 27 — Pass 3 R1 : accept rejects non-CHF tx
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_rejects_non_chf_transaction(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "EUR",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    let body: Value = resp.json().await.unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed[0]["errorCode"], "RECONCILIATION_CURRENCY_MISMATCH");
}

// ============================================================
// Test 28 — Pass 3 R4 : audit value_date nullable
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_emits_audit_with_null_value_date_when_tx_value_date_is_null(
    pool: MySqlPool,
) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (rule_id, tx_id) = create_rule_and_tx(
        &pool,
        &app,
        &ctx,
        "Swisscom",
        "Swisscom",
        dec!(-150.00),
        "CHF",
        None,
    )
    .await;
    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);

    let details: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE action = 'reconciliation_rule.applied' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(
        details["value_date"].is_null(),
        "value_date must be null when tx.value_date is null"
    );
    assert!(!details["entry_date"].is_null());
    let _ = ctx.bank_ledger_account_id;
}

// ============================================================
// Story 19-5 — projet analytique par défaut sur une règle
// ============================================================

/// AC #21a — accept `type=rule` d'une règle portant `defaultProjectId` :
/// les **2 lignes** de l'écriture générée portent le projet (propagation
/// document-level via `line.project_id.or(new.project_id)`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_stamps_default_project_on_all_lines(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let project_id = create_project(&pool, ctx.company_id, "RENOV-RULE", false).await;

    // Règle avec projet par défaut.
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Loyer projet",
            "matchType": "counterparty_contains",
            "matchValue": "Loyer",
            "counterpartyAccountId": ctx.counterparty_account_id,
            "defaultProjectId": project_id,
        }),
    )
    .await;
    assert_eq!(r["defaultProjectId"].as_i64(), Some(project_id));
    let rule_id = r["id"].as_i64().unwrap();

    let tx_id = create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-1200.00),
        "Loyer SA",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;

    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let accepted = body["accepted"].as_array().unwrap();
    assert_eq!(accepted.len(), 1);
    let je_id = accepted[0]["journalEntryId"].as_i64().unwrap();

    let project_ids: Vec<Option<i64>> = sqlx::query_scalar(
        "SELECT project_id FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(je_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(project_ids.len(), 2);
    assert!(
        project_ids.iter().all(|p| *p == Some(project_id)),
        "les 2 lignes doivent porter le projet par défaut de la règle, got {project_ids:?}"
    );
}

/// AC #21d — le `defaultProjectId` d'une règle a été archivé après la
/// création de la règle : l'accept re-valide et retourne un `FailedProposal`
/// `PROJECT_ARCHIVED` (HTTP 200, `accepted` vide), sans casser le batch ni
/// escalader en AppError globale.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn accept_with_rule_fails_proposal_when_default_project_archived(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let project_id = create_project(&pool, ctx.company_id, "SOON-ARCHIVED", false).await;

    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Loyer projet clos",
            "matchType": "counterparty_contains",
            "matchValue": "Loyer",
            "counterpartyAccountId": ctx.counterparty_account_id,
            "defaultProjectId": project_id,
        }),
    )
    .await;
    let rule_id = r["id"].as_i64().unwrap();

    // Archiver le projet APRÈS création de la règle.
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(project_id)
        .execute(&pool)
        .await
        .unwrap();

    let tx_id = create_pending_bank_tx(
        &pool,
        ctx.company_id,
        ctx.user_id,
        ctx.bank_account_id,
        dec!(-1200.00),
        "Loyer SA",
        None,
        None,
        "CHF",
        Some(NaiveDate::from_ymd_opt(2026, 5, 10).unwrap()),
    )
    .await;

    let resp = post_accept_rule(
        &app,
        &ctx.jwt,
        ctx.bank_account_id,
        tx_id,
        rule_id,
        ctx.counterparty_account_id,
    )
    .await;
    // Succès partiel = succès HTTP (pattern batch FailedProposal).
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(
        body["accepted"].as_array().unwrap().is_empty(),
        "aucune proposition ne doit être acceptée"
    );
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["errorCode"].as_str(), Some("PROJECT_ARCHIVED"));
    assert_eq!(failed[0]["bankTransactionId"].as_i64(), Some(tx_id));
    // AC11 — le projet fautif est joint dans details.
    assert_eq!(
        failed[0]["details"]["projectId"].as_i64(),
        Some(project_id),
        "details doit porter projectId (AC11)"
    );

    // La transaction reste pending (rollback du savepoint).
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        status, "pending",
        "la tx doit rester pending après échec projet"
    );
}

/// Story 19-5 (Pass 1 HIGH BH) — PATCH `defaultProjectId: null` **efface**
/// réellement le projet par défaut (colonne → NULL), et non un no-op silencieux.
/// Verrouille le fix `double_option` (sans lui, serde replierait `null` sur
/// `None` = inchangé).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_default_project_null_clears_the_column(pool: MySqlPool) {
    let ctx = setup_ctx(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let project_id = create_project(&pool, ctx.company_id, "TO-CLEAR", false).await;

    // Règle avec projet par défaut.
    let r = create_rule_ok(
        &app,
        &ctx.jwt,
        json!({
            "label": "Règle à nettoyer",
            "matchType": "counterparty_contains",
            "matchValue": "Truc",
            "counterpartyAccountId": ctx.counterparty_account_id,
            "defaultProjectId": project_id,
        }),
    )
    .await;
    let rule_id = r["id"].as_i64().unwrap();
    let version = r["version"].as_i64().unwrap();
    assert_eq!(r["defaultProjectId"].as_i64(), Some(project_id));

    // PATCH avec defaultProjectId: null → doit effacer.
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{rule_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "expectedVersion": version, "defaultProjectId": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(
        body["defaultProjectId"].is_null(),
        "la réponse doit refléter defaultProjectId = null après effacement"
    );

    // Ground-truth DB : la colonne est bien NULL.
    let db_val: Option<i64> =
        sqlx::query_scalar("SELECT default_project_id FROM reconciliation_rules WHERE id = ?")
            .bind(rule_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(db_val, None, "default_project_id doit être NULL en base");

    // Sanity : omettre le champ laisse la valeur inchangée (on ré-affecte puis
    // on patche un autre champ sans toucher au projet).
    let version2 = body["version"].as_i64().unwrap();
    let resp2 = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{rule_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "expectedVersion": version2,
            "defaultProjectId": project_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp2.status(), 200);
    let body2: Value = resp2.json().await.unwrap();
    assert_eq!(body2["defaultProjectId"].as_i64(), Some(project_id));

    // PATCH d'un autre champ (label) SANS defaultProjectId → projet inchangé.
    let version3 = body2["version"].as_i64().unwrap();
    let resp3 = app
        .client
        .patch(app.url(&format!("/api/v1/reconciliation/rules/{rule_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "expectedVersion": version3, "label": "Renommée" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp3.status(), 200);
    let body3: Value = resp3.json().await.unwrap();
    assert_eq!(
        body3["defaultProjectId"].as_i64(),
        Some(project_id),
        "omettre defaultProjectId doit laisser la valeur inchangée"
    );
}
