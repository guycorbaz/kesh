//! Tests E2E HTTP Story 8-5a-base — réconciliation manuelle (FR45).
//!
//! 11 tests couvrant les ACs #83-#92 + L48 zero_amount :
//!
//! 1. happy path crédit positif (+200 → bank débit / contrepartie crédit)
//! 2. happy path débit négatif (-150 → bank crédit / contrepartie débit)
//! 3. 404 unknown bank_account
//! 4. 412 BANK_ACCOUNT_NOT_CONFIGURED (journal_account_id NULL)
//! 5. 404 unknown counterparty
//! 6. 404 archived counterparty
//! 7. 404 already reconciled tx
//! 8. réversibilise auto rejection (was_previously_rejected=true audit)
//! 9. 409 fiscal year closed pré-flight
//! 10. multi-tenant : tx d'autre company → 404
//! 11. 400 zero_amount transaction (L48 / step 4bis)
//!
//! Pré-requis : MariaDB démarré localement.
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
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Spawn helpers (pattern reconciliation_e2e.rs / bank_accounts_e2e.rs)
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

// ============================================================
// Domain seed helpers
// ============================================================

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

async fn create_account(
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

async fn archive_account(pool: &MySqlPool, account_id: i64, user_id: i64) {
    accounts::archive(pool, account_id, 1, user_id)
        .await
        .expect("archive account");
}

/// Crée un fiscal_year `Open` couvrant 2026-01-01 → 2026-12-31.
async fn insert_open_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) \
         VALUES (?, ?, '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(format!("FY 2026 c{company_id}"))
    .execute(pool)
    .await
    .expect("fiscal_year insert");
    result.last_insert_id() as i64
}

/// Crée un fiscal_year `Closed` (pour AC #89).
async fn insert_closed_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) \
         VALUES (?, ?, '2026-01-01', '2026-12-31', 'Closed', NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(format!("FY 2026 c{company_id} closed"))
    .execute(pool)
    .await
    .expect("fiscal_year insert closed");
    result.last_insert_id() as i64
}

fn make_new_import(
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    period: NaiveDate,
) -> NewBankImport {
    NewBankImport {
        company_id,
        bank_account_id,
        filename: "stmt.xml".into(),
        file_hash: file_hash.into(),
        source_format: BankImportSourceFormat::Camt053V04,
        statement_id: Some("STMT-001".into()),
        period_from: period,
        period_to: period,
        opening_balance: Some(dec!(1000.00)),
        closing_balance: Some(dec!(1100.00)),
        transaction_count: 1,
        imported_by_user_id: user_id,
    }
}

fn make_new_tx(
    company_id: i64,
    bank_account_id: i64,
    booking_date: NaiveDate,
    amount: Decimal,
    reference: &str,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date,
        value_date: Some(booking_date),
        amount,
        currency: "CHF".into(),
        reference: Some(reference.into()),
        details: "Test tx".into(),
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name: None,
    }
}

async fn seed_bank_transactions(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    booking_date: NaiveDate,
    new_txs: Vec<NewBankTransaction>,
) -> Vec<i64> {
    let mut tx = pool.begin().await.unwrap();
    let count = new_txs.len() as i32;
    let mut import = make_new_import(
        company_id,
        bank_account_id,
        user_id,
        file_hash,
        booking_date,
    );
    import.transaction_count = count;
    let (_, inserted) = bank_imports::create_with_transactions(&mut tx, import, new_txs)
        .await
        .expect("create_with_transactions");
    tx.commit().await.unwrap();
    inserted.iter().map(|t| t.id).collect()
}

fn unique_hash(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    let nano = chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0);
    nano.hash(&mut h);
    let v = h.finish();
    format!("{:0>64x}", v)
}

/// Lie `bank_account.journal_account_id` → asset 1020 directement
/// (bypass route PATCH /bank-accounts pour simplifier le seed).
async fn link_bank_account_journal(
    pool: &MySqlPool,
    bank_account_id: i64,
    journal_account_id: i64,
) {
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ?",
    )
    .bind(journal_account_id)
    .bind(bank_account_id)
    .execute(pool)
    .await
    .expect("link bank_account journal");
}

// ============================================================
// Multi-actor setup
// ============================================================

/// Setup complet : company + user + bank_account lié à 1020 + fiscal_year
/// Open + 1 compte de contrepartie 6810 actif.
struct ManualCtx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    bank_ledger_account_id: i64,
    counterparty_account_id: i64,
    jwt: String,
}

async fn setup_full(pool: &MySqlPool, label: &str, iban: &str, role: Role) -> ManualCtx {
    let company_id = create_company(pool, label).await;
    let username = format!("{label}_user");
    let user_id = create_user(pool, &username, role, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, iban).await;
    let bank_ledger_account_id = create_account(
        pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let counterparty_account_id = create_account(
        pool,
        company_id,
        user_id,
        "6810",
        "Frais bancaires",
        AccountType::Expense,
    )
    .await;
    link_bank_account_journal(pool, bank_account_id, bank_ledger_account_id).await;
    insert_open_fiscal_year(pool, company_id).await;
    let role_str = match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    };
    let jwt = forge_jwt(user_id, role_str, company_id);
    ManualCtx {
        company_id,
        user_id,
        bank_account_id,
        bank_ledger_account_id,
        counterparty_account_id,
        jwt,
    }
}

// ============================================================
// Tests : POST /api/v1/reconciliation/manual
// ============================================================

/// AC #83 — happy path **débit négatif** (paiement) : tx pending -150
/// CHF + counterpartyAccountId 6810 → 200 OK + journal_entry à 2 lignes
/// (1020 crédit 150 + 6810 débit 150) + tx reconciled.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_creates_journal_entry_and_marks_transaction_reconciled(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("manual_debit"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(-150.00),
            "TWINT-FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": ctx.counterparty_account_id,
            "description": "Frais TWINT mai",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "happy débit doit retourner 200");
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().expect("journalEntryId");
    assert_eq!(body["bankTransactionId"].as_i64().unwrap(), tx_id);

    // Vérifier l'écriture créée : 2 lignes équilibrées.
    let lines: Vec<(i64, Decimal, Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(je_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lines.len(), 2);
    // Ligne 1 : banque crédit 150.
    assert_eq!(lines[0].0, ctx.bank_ledger_account_id);
    assert_eq!(lines[0].1, dec!(0));
    assert_eq!(lines[0].2, dec!(150.00));
    // Ligne 2 : contrepartie débit 150.
    assert_eq!(lines[1].0, ctx.counterparty_account_id);
    assert_eq!(lines[1].1, dec!(150.00));
    assert_eq!(lines[1].2, dec!(0));

    // Tx reconciled, matched_entry_id, auto_match_rejected_at NULL.
    let (status, matched_entry_id, rej_at): (String, Option<i64>, Option<chrono::NaiveDateTime>) =
        sqlx::query_as(
            "SELECT status, matched_entry_id, auto_match_rejected_at \
             FROM bank_transactions WHERE id = ?",
        )
        .bind(tx_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "reconciled");
    assert_eq!(matched_entry_id, Some(je_id));
    assert!(rej_at.is_none(), "auto_match_rejected_at doit être NULL");
}

/// AC #84 — happy path **crédit positif** (encaissement) : tx pending
/// +200.00 CHF + counterpartyAccountId 7510 (Revenue ici) → 200 OK +
/// journal_entry à 2 lignes (1020 débit 200 + 7510 crédit 200).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_creates_journal_entry_for_negative_amount(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Compte additionnel 7510 Intérêts (Revenue).
    let revenue_id = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "7510",
        "Intérêts bancaires",
        AccountType::Revenue,
    )
    .await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("manual_credit"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(200.00),
            "INTEREST-MAY",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": revenue_id,
            "description": "Intérêts mai",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().unwrap();

    let lines: Vec<(i64, Decimal, Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(je_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(lines.len(), 2);
    // Ligne 1 : banque débit 200 (entrée cash).
    assert_eq!(lines[0].0, ctx.bank_ledger_account_id);
    assert_eq!(lines[0].1, dec!(200.00));
    assert_eq!(lines[0].2, dec!(0));
    // Ligne 2 : revenue crédit 200.
    assert_eq!(lines[1].0, revenue_id);
    assert_eq!(lines[1].1, dec!(0));
    assert_eq!(lines[1].2, dec!(200.00));
}

/// AC #87 (multi-tenant bank_account) — bankAccountId inconnu / d'une
/// autre company → 404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_404_for_unknown_bank_account_id(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    // bank_account_id 99999 n'existe pas.
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": 99999,
            "bankTransactionId": 1,
            "counterpartyAccountId": ctx.counterparty_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
}

/// AC #85 — bank_account.journal_account_id IS NULL → 412
/// BANK_ACCOUNT_NOT_CONFIGURED.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_412_for_bank_account_not_configured(pool: MySqlPool) {
    let company_id = create_company(&pool, "Acme").await;
    let user_id = create_user(&pool, "alice", Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account(&pool, company_id, "CH4431999123000889012").await;
    // PAS de link_bank_account_journal — journal_account_id reste NULL.
    let counterparty_id = create_account(
        &pool,
        company_id,
        user_id,
        "6810",
        "Frais bancaires",
        AccountType::Expense,
    )
    .await;
    insert_open_fiscal_year(&pool, company_id).await;
    let jwt = forge_jwt(user_id, "Comptable", company_id);
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash("412_unconfigured"),
        day,
        vec![make_new_tx(
            company_id,
            bank_account_id,
            day,
            dec!(-50.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&jwt)
        .json(&json!({
            "bankAccountId": bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": counterparty_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 412, "412 Precondition Failed");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_ACCOUNT_NOT_CONFIGURED");
    assert_eq!(
        body["error"]["details"]["bankAccountId"].as_i64().unwrap(),
        bank_account_id
    );
    assert!(body["error"]["details"]["hint"].is_string(), "hint présent");
}

/// AC #86 — counterpartyAccountId inconnu / d'une autre company →
/// 404 ACCOUNT_NOT_FOUND (anti-énumération KF-002).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_404_for_unknown_counterparty_account_id(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("404_counterparty"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(-50.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": 99999,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
}

/// AC #86 (variante) — counterpartyAccountId archivé → 404 ACCOUNT_NOT_FOUND
/// (anti-énumération : pas de leak de l'existence).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_404_for_archived_counterparty_account(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    archive_account(&pool, ctx.counterparty_account_id, ctx.user_id).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("archived"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(-50.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": ctx.counterparty_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
}

/// AC #88 — tx déjà reconciled → 404 RECONCILIATION_TRANSACTION_NOT_PENDING.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_404_for_already_reconciled_transaction(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("already_recon"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(-50.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];
    // Marque la tx comme reconciled directement.
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE id = ?")
        .bind(tx_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": ctx.counterparty_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "RECONCILIATION_TRANSACTION_NOT_PENDING"
    );
}

/// AC #90 — tx avec `auto_match_rejected_at` non NULL → manual réussit
/// ET reset `auto_match_rejected_at = NULL` ET audit log
/// `was_previously_rejected = true` (lève L23 partiellement, F6''' Pass 3 Opus).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_reverses_auto_rejection_when_auto_match_rejected_at_set(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("reverse_reject"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(-75.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];
    // Pose auto_match_rejected_at (simule rejet précédent 8-4).
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(tx_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": ctx.counterparty_account_id,
            "description": "Annulation rejet",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Tx reconciled + auto_match_rejected_at NULL post-update.
    let rej_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT auto_match_rejected_at FROM bank_transactions WHERE id = ?")
            .bind(tx_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        rej_at.is_none(),
        "auto_match_rejected_at doit être NULL après manual reverse"
    );

    // Audit log : details.was_previously_rejected = true (F6''' Pass 3).
    let details_json: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log \
         WHERE action = 'reconciliation.manual_matched' AND entity_id = ?",
    )
    .bind(tx_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details_json["was_previously_rejected"],
        serde_json::Value::Bool(true),
        "audit details.was_previously_rejected doit être true"
    );
    // Vérifier shape complète audit details_json snake_case top-level.
    assert!(details_json["bank_transaction_id"].is_i64());
    assert!(details_json["counterparty_account_id"].is_i64());
    assert!(details_json["journal_entry_id"].is_i64());
    assert!(details_json["amount"].is_string());
    assert!(details_json["description"].is_string());
    assert!(details_json["value_date"].is_string());
}

/// AC #89 — fiscal_year `Closed` → 409 RECONCILIATION_FISCAL_YEAR_CLOSED.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_returns_409_for_fiscal_year_closed_pre_flight(pool: MySqlPool) {
    let company_id = create_company(&pool, "Acme").await;
    let user_id = create_user(&pool, "alice", Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account(&pool, company_id, "CH4431999123000889012").await;
    let bank_ledger_id = create_account(
        &pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let counterparty_id = create_account(
        &pool,
        company_id,
        user_id,
        "6810",
        "Frais bancaires",
        AccountType::Expense,
    )
    .await;
    link_bank_account_journal(&pool, bank_account_id, bank_ledger_id).await;
    // PAS de fiscal_year Open : seul un fiscal_year Closed couvre 2026 →
    // `find_open_covering_date` retourne None → 409.
    insert_closed_fiscal_year(&pool, company_id).await;
    let jwt = forge_jwt(user_id, "Comptable", company_id);

    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash("fy_closed"),
        day,
        vec![make_new_tx(
            company_id,
            bank_account_id,
            day,
            dec!(-50.00),
            "FEES",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&jwt)
        .json(&json!({
            "bankAccountId": bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": counterparty_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "RECONCILIATION_FISCAL_YEAR_CLOSED");
}

/// AC #87 — multi-tenant : tx d'une autre company → 404
/// RECONCILIATION_TRANSACTION_NOT_PENDING (anti-leak KF-002, le helper
/// `find_strictly_pending_by_id_for_account` retourne None via filtre
/// `(company_id, bank_account_id, id)`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_does_not_leak_cross_tenant_transaction(pool: MySqlPool) {
    let ctx_a = setup_full(&pool, "AcmeA", "CH4431999123000889012", Role::Comptable).await;
    // Company B avec sa propre bank_account + tx + bank_ledger 1020 +
    // counterparty 6810.
    let company_b = create_company(&pool, "AcmeB").await;
    let user_b = create_user(&pool, "bob", Role::Comptable, company_b).await;
    let bank_b = create_bank_account(&pool, company_b, "CH9300762011623852957").await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_b_ids = seed_bank_transactions(
        &pool,
        company_b,
        bank_b,
        user_b,
        &unique_hash("tenant_b"),
        day,
        vec![make_new_tx(company_b, bank_b, day, dec!(-100.00), "FEES-B")],
    )
    .await;
    let tx_b_id = tx_b_ids[0];

    let app = spawn_app(pool.clone()).await;
    // user_a tente de réconcilier la tx de company_b en passant son
    // propre bank_account_id (qui est valide pour user_a) — la tx_b_id
    // ne lui appartient pas → 404 RECONCILIATION_TRANSACTION_NOT_PENDING
    // (le helper filtre par `(company_a, bank_a, tx_b_id)` → None).
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx_a.jwt)
        .json(&json!({
            "bankAccountId": ctx_a.bank_account_id,
            "bankTransactionId": tx_b_id,
            "counterpartyAccountId": ctx_a.counterparty_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "anti-leak cross-tenant tx → 404");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "RECONCILIATION_TRANSACTION_NOT_PENDING"
    );
}

/// L48 (F7''' Pass 3 Opus) — tx avec `amount = 0.00` → 400
/// VALIDATION_ERROR avec marqueur "zero_amount_transaction" dans
/// `error.message` (F5'''' Pass 6 Opus shape réelle de
/// `AppError::Validation`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_manual_rejects_zero_amount_transaction(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let day = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("zero_amount"),
        day,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            day,
            dec!(0),
            "ZERO",
        )],
    )
    .await;
    let tx_id = tx_ids[0];

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/manual"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankAccountId": ctx.bank_account_id,
            "bankTransactionId": tx_id,
            "counterpartyAccountId": ctx.counterparty_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("zero_amount_transaction"),
        "marqueur zero_amount_transaction attendu dans message, got: {message}"
    );
}
