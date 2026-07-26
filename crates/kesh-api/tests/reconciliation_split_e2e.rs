//! Tests E2E HTTP pour Story 8-5a-bis FR48 — éclatement de transaction
//! agrégée (POST `/api/v1/reconciliation/split`).
//!
//! 10 tests couvrant les ACs #93-#99 :
//!
//! - Happy paths : #93 (split débit), #94 (split crédit).
//! - Balance + bornes : #95, #96.
//! - 412 BANK_ACCOUNT_NOT_CONFIGURED : #97.
//! - Multi-tenant + RBAC : #98.
//! - Audit log + already reconciled : #99.
//!
//! Pré-requis : MariaDB démarré localement (`KESH_TEST_MODE=true`).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::{AccountType, NewAccount};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    BankImportSourceFormat, Language, NewBankAccount, NewBankImport, NewBankTransaction,
    NewCompany, NewUser, OrgType, Role,
};
use kesh_db::repositories::{accounts, bank_accounts, bank_imports, companies, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
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

// ============================================================
// Domain seed helpers
// ============================================================

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

async fn create_bank_account_no_journal(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
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
    user_id: i64,
    company_id: i64,
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
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

async fn link_bank_account_to_journal(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    journal_account_id: i64,
) {
    let mut tx = pool.begin().await.unwrap();
    let bank_account = bank_accounts::find_by_id_for_company(pool, company_id, bank_account_id)
        .await
        .unwrap()
        .expect("bank_account exists");
    bank_accounts::set_journal_account_id_for_company(
        &mut tx,
        company_id,
        bank_account_id,
        Some(journal_account_id),
        bank_account.version,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

/// Crée un projet analytique (Story 19-5) et retourne son id.
async fn create_project(pool: &MySqlPool, company_id: i64, code: &str) -> i64 {
    let result = sqlx::query(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, ?, ?, FALSE)",
    )
    .bind(company_id)
    .bind(code)
    .bind(format!("Projet {code}"))
    .execute(pool)
    .await
    .expect("project insert");
    result.last_insert_id() as i64
}

async fn insert_fake_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
    let name = format!("FY 2026 c{company_id}");
    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?")
            .bind(company_id)
            .bind(&name)
            .fetch_optional(pool)
            .await
            .expect("fiscal_year lookup");
    if let Some(id) = existing {
        return id;
    }
    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
         created_at, updated_at) \
         VALUES (?, ?, '2026-01-01', '2026-12-31', 'Open', NOW(3), NOW(3))",
    )
    .bind(company_id)
    .bind(&name)
    .execute(pool)
    .await
    .expect("fiscal_year insert");
    result.last_insert_id() as i64
}

fn make_new_import(
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    period_from: NaiveDate,
    period_to: NaiveDate,
) -> NewBankImport {
    NewBankImport {
        company_id,
        bank_account_id,
        filename: "stmt.xml".into(),
        file_hash: file_hash.into(),
        source_format: BankImportSourceFormat::Camt053V04,
        statement_id: Some("STMT-001".into()),
        period_from,
        period_to,
        opening_balance: Some(dec!(1000.00)),
        closing_balance: Some(dec!(1100.00)),
        transaction_count: 1,
        imported_by_user_id: user_id,
    }
}

#[allow(clippy::too_many_arguments)]
fn make_new_tx(
    company_id: i64,
    bank_account_id: i64,
    booking_date: NaiveDate,
    value_date: Option<NaiveDate>,
    amount: Decimal,
    currency: &str,
    reference: &str,
) -> NewBankTransaction {
    NewBankTransaction {
        company_id,
        bank_account_id,
        booking_date,
        value_date,
        amount,
        currency: currency.into(),
        reference: Some(reference.into()),
        details: "Test tx".into(),
        end_to_end_id: None,
        transaction_id: None,
        counterparty_iban: None,
        counterparty_name: None,
    }
}

#[allow(clippy::too_many_arguments)]
async fn seed_bank_transactions(
    pool: &MySqlPool,
    company_id: i64,
    bank_account_id: i64,
    user_id: i64,
    file_hash: &str,
    period_from: NaiveDate,
    period_to: NaiveDate,
    new_txs: Vec<NewBankTransaction>,
) -> Vec<i64> {
    let mut tx = pool.begin().await.unwrap();
    let count = new_txs.len() as i32;
    let mut import = make_new_import(
        company_id,
        bank_account_id,
        user_id,
        file_hash,
        period_from,
        period_to,
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

// ============================================================
// Setup combiné (company + user + bank_account + journal + tx)
// ============================================================

struct SplitCtx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    bank_ledger_account_id: i64,
    cp_a_account_id: i64,
    cp_b_account_id: i64,
    cp_c_account_id: i64,
    tx_id: i64,
    jwt: String,
}

/// Setup commun pour les tests split : company + user Comptable +
/// bank_account configuré (journal_account_id 1020 actif) + 3 comptes
/// contreparties classes 5/6/7 actifs + 1 fiscal year ouvert 2026 +
/// 1 bank_transaction pending débit -10700.00.
async fn setup_split_ctx(pool: &MySqlPool, label: &str, iban: &str) -> SplitCtx {
    let company_id = create_company(pool, label).await;
    let user_id = create_user(pool, &format!("{label}_user"), Role::Comptable, company_id).await;
    let bank_account_id = create_bank_account_no_journal(pool, company_id, iban).await;
    // bank ledger 1020 Banque (Asset).
    let bank_ledger_account_id = create_account(
        pool,
        user_id,
        company_id,
        "1020",
        "Banque",
        AccountType::Asset,
    )
    .await;
    // counterparties class 5/6/7.
    let cp_a_account_id = create_account(
        pool,
        user_id,
        company_id,
        "5000",
        "Salaires",
        AccountType::Expense,
    )
    .await;
    let cp_b_account_id = create_account(
        pool,
        user_id,
        company_id,
        "5700",
        "Charges sociales",
        AccountType::Expense,
    )
    .await;
    let cp_c_account_id = create_account(
        pool,
        user_id,
        company_id,
        "6900",
        "Divers",
        AccountType::Expense,
    )
    .await;
    link_bank_account_to_journal(pool, company_id, bank_account_id, bank_ledger_account_id).await;
    let _ = insert_fake_fiscal_year(pool, company_id).await;

    let booking_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let tx_ids = seed_bank_transactions(
        pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash(label),
        booking_date,
        booking_date,
        vec![make_new_tx(
            company_id,
            bank_account_id,
            booking_date,
            Some(booking_date),
            dec!(-10700.00),
            "CHF",
            "SALAIRES-MAY",
        )],
    )
    .await;

    let jwt = forge_jwt(user_id, "Comptable", company_id);

    SplitCtx {
        company_id,
        user_id,
        bank_account_id,
        bank_ledger_account_id,
        cp_a_account_id,
        cp_b_account_id,
        cp_c_account_id,
        tx_id: tx_ids[0],
        jwt,
    }
}

// ============================================================
// Tests
// ============================================================

/// Story 19-5 — split avec un `projectId` **par ligne de ventilation** :
/// chaque ligne de contrepartie porte son propre projet, la ligne banque
/// reste non taguée. Validation projet automatique (create_in_tx per-ligne).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_tags_project_per_line(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_project", "CH1000000000000000009").await;
    let project_a = create_project(&pool, ctx.company_id, "CHALET").await;
    let project_b = create_project(&pool, ctx.company_id, "APPART").await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5000", "description": "Rénovation chalet", "projectId": project_a },
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "4500", "description": "Rénovation appart", "projectId": project_b },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "1200", "description": "Divers non affecté" },
        ],
        "valueDate": "2026-05-31"
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().unwrap();

    // Ligne banque : non taguée.
    let bank_project: Option<i64> = sqlx::query_scalar(
        "SELECT project_id FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(je_id)
    .bind(ctx.bank_ledger_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        bank_project, None,
        "la ligne banque ne doit pas être taguée"
    );

    // Les 2 premières lignes de ventilation portent leur projet respectif.
    let tagged: Vec<(Decimal, Option<i64>)> = sqlx::query_as(
        "SELECT debit, project_id FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id != ? ORDER BY debit DESC",
    )
    .bind(je_id)
    .bind(ctx.bank_ledger_account_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    // débit 5000 → project_a, débit 4500 → project_b, débit 1200 → None.
    assert_eq!(tagged.len(), 3);
    assert_eq!(tagged[0], (dec!(5000.00), Some(project_a)));
    assert_eq!(tagged[1], (dec!(4500.00), Some(project_b)));
    assert_eq!(tagged[2], (dec!(1200.00), None));
}

/// AC #93 — happy path split débit. Tx -10700 → 3 lignes contreparties
/// (5000+4500+1200) en N+1 lignes JE (1 banque crédit 10700 + 3 splits débit).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_creates_journal_entry_with_n_plus_1_lines(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_happy_debit", "CH1000000000000000001").await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5000", "description": "Salaire Alice" },
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "4500", "description": "Salaire Bob" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "1200", "description": "Charges sociales" },
        ],
        "valueDate": "2026-05-31"
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().unwrap();
    assert_eq!(body["bankTransactionId"], ctx.tx_id);

    // Vérifie 4 lignes JE (1 banque + 3 splits).
    let line_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?")
            .bind(je_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(line_count, 4);

    // Banque crédit 10700.
    let bank_credit: Decimal = sqlx::query_scalar(
        "SELECT credit FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ?",
    )
    .bind(je_id)
    .bind(ctx.bank_ledger_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(bank_credit, dec!(10700.00));

    // bank_transactions status = reconciled.
    let status: String = sqlx::query_scalar("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(ctx.tx_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "reconciled");
}

/// AC #94 — happy path split crédit. Tx +5000 → 2 splits crédit (3000+2000).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_creates_journal_entry_for_credit_transaction(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_happy_credit", "CH1000000000000000002").await;

    // Override : crée une nouvelle tx +5000.
    let booking_date = NaiveDate::from_ymd_opt(2026, 5, 28).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        ctx.company_id,
        ctx.bank_account_id,
        ctx.user_id,
        &unique_hash("credit_tx"),
        booking_date,
        booking_date,
        vec![make_new_tx(
            ctx.company_id,
            ctx.bank_account_id,
            booking_date,
            Some(booking_date),
            dec!(5000.00),
            "CHF",
            "ENCAISSEMENT-MAY",
        )],
    )
    .await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": tx_ids[0],
        "splits": [
            { "counterpartyAccountId": ctx.cp_c_account_id, "amount": "3000", "description": "Intérêts" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "2000", "description": "Remboursement" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().unwrap();

    // Banque débit 5000 (entrée cash).
    let bank_debit: Decimal = sqlx::query_scalar(
        "SELECT debit FROM journal_entry_lines \
         WHERE entry_id = ? AND account_id = ?",
    )
    .bind(je_id)
    .bind(ctx.bank_ledger_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(bank_debit, dec!(5000.00));
}

/// AC #95 — split déséquilibré → 400 RECONCILIATION_SPLIT_IMBALANCE
/// avec details { expected, actual, difference }.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_rejects_imbalanced_payload(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_imbalance", "CH1000000000000000003").await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5000.00", "description": "Salaire Alice" },
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "4500.00", "description": "Salaire Bob" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "1000.00", "description": "Charges" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "RECONCILIATION_SPLIT_IMBALANCE");
    assert_eq!(body["error"]["details"]["expected"], "10700.00");
    assert_eq!(body["error"]["details"]["actual"], "10500.00");
    assert_eq!(body["error"]["details"]["difference"], "-200.00");
}

/// AC #96 part 1 — splits.len() < 2 → 400 Validation.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_rejects_single_line_payload(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_single_line", "CH1000000000000000004").await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "10700", "description": "Tout" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// AC #96 part 2 — splits.len() > 50 → 400 Validation.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_rejects_too_many_lines(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_too_many", "CH1000000000000000005").await;

    let splits: Vec<Value> = (0..51)
        .map(|i| {
            serde_json::json!({
                "counterpartyAccountId": ctx.cp_a_account_id,
                "amount": "1",
                "description": format!("Line {i}"),
            })
        })
        .collect();
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": splits,
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// AC #97 — bank_account.journal_account_id NULL → 412 BANK_ACCOUNT_NOT_CONFIGURED.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_rejects_unconfigured_bank_account_with_412(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    // Setup minimal : pas de lien journal_account_id.
    let company_id = create_company(&pool, "split_unconfig_co").await;
    let user_id = create_user(&pool, "split_unconfig_user", Role::Comptable, company_id).await;
    let bank_account_id =
        create_bank_account_no_journal(&pool, company_id, "CH1000000000000000006").await;
    let cp_account = create_account(
        &pool,
        user_id,
        company_id,
        "5000",
        "Salaires",
        AccountType::Expense,
    )
    .await;
    let _ = insert_fake_fiscal_year(&pool, company_id).await;
    let booking_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
    let tx_ids = seed_bank_transactions(
        &pool,
        company_id,
        bank_account_id,
        user_id,
        &unique_hash("unconfig"),
        booking_date,
        booking_date,
        vec![make_new_tx(
            company_id,
            bank_account_id,
            booking_date,
            None,
            dec!(-100.00),
            "CHF",
            "TX1",
        )],
    )
    .await;
    let jwt = forge_jwt(user_id, "Comptable", company_id);

    let body = serde_json::json!({
        "bankAccountId": bank_account_id,
        "bankTransactionId": tx_ids[0],
        "splits": [
            { "counterpartyAccountId": cp_account, "amount": "60", "description": "A" },
            { "counterpartyAccountId": cp_account, "amount": "40", "description": "B" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 412);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_ACCOUNT_NOT_CONFIGURED");
}

/// AC #98 part 1 — counterpartyAccountId cross-tenant → 404 ACCOUNT_NOT_FOUND
/// avec body details.missingAccountIds.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_does_not_leak_cross_tenant_account(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_xtenant_a", "CH1000000000000000007").await;
    // Company B avec un compte chez elle qu'on tente d'utiliser depuis A.
    let company_b = create_company(&pool, "split_xtenant_b_co").await;
    let user_b = create_user(&pool, "split_xtenant_b_user", Role::Comptable, company_b).await;
    let other_account = create_account(
        &pool,
        user_b,
        company_b,
        "5000",
        "Autre",
        AccountType::Expense,
    )
    .await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5350", "description": "Ok" },
            { "counterpartyAccountId": other_account, "amount": "5350", "description": "Cross" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
    let missing = body["error"]["details"]["missingAccountIds"]
        .as_array()
        .expect("missingAccountIds array");
    assert!(missing.iter().any(|v| v.as_i64() == Some(other_account)));
}

/// AC #98 part 2 — Role Consultation → 403 Forbidden.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_requires_comptable_role(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_rbac", "CH1000000000000000008").await;
    let consult_user = create_user(
        &pool,
        "split_rbac_consult",
        Role::Consultation,
        ctx.company_id,
    )
    .await;
    let consult_jwt = forge_jwt(consult_user, "Consultation", ctx.company_id);

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5350", "description": "A" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "5350", "description": "B" },
        ],
    });

    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {consult_jwt}"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

/// AC #99 part 1 — audit log `reconciliation.split_applied` émis +
/// `journal_entry.created` émis par `journal_entries::create_in_tx`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_emits_audit_log(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_audit", "CH1000000000000000009").await;

    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5000", "description": "Alice" },
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "4500", "description": "Bob" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "1200", "description": "Charges" },
        ],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let je_id = body["journalEntryId"].as_i64().unwrap();

    // 1 audit log reconciliation.split_applied avec snake_case top-level + sub.
    let row: (Value,) = sqlx::query_as(
        "SELECT details_json FROM audit_log \
         WHERE action = 'reconciliation.split_applied' AND entity_id = ? LIMIT 1",
    )
    .bind(ctx.tx_id)
    .fetch_one(&pool)
    .await
    .expect("audit reconciliation.split_applied");
    let details = row.0;
    assert_eq!(details["bank_transaction_id"], ctx.tx_id);
    assert_eq!(details["journal_entry_id"], je_id);
    assert_eq!(details["total_amount"], "10700.00");
    assert_eq!(details["was_previously_rejected"], false);
    let splits_audit = details["splits"].as_array().unwrap();
    assert_eq!(splits_audit.len(), 3);
    assert_eq!(
        splits_audit[0]["counterparty_account_id"],
        ctx.cp_a_account_id
    );
    // P5 Pass 1 code-review — scale 2 normalisé via round_dp(2).
    assert_eq!(splits_audit[0]["amount"], "5000.00");

    // 1 audit log journal_entry.created.
    let je_audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE action = 'journal_entry.created' AND entity_id = ?",
    )
    .bind(je_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(je_audit_count, 1);
}

/// AC #99 part 2 — Tx déjà reconciled → 404 RECONCILIATION_TRANSACTION_NOT_PENDING.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn split_rejects_already_reconciled(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup_split_ctx(&pool, "split_already_reconciled", "CH1000000000000000010").await;

    // Premier split : ok.
    let body = serde_json::json!({
        "bankAccountId": ctx.bank_account_id,
        "bankTransactionId": ctx.tx_id,
        "splits": [
            { "counterpartyAccountId": ctx.cp_a_account_id, "amount": "5350", "description": "A" },
            { "counterpartyAccountId": ctx.cp_b_account_id, "amount": "5350", "description": "B" },
        ],
    });
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Deuxième split sur la même tx → 404 not pending.
    let resp = app
        .client
        .post(app.url("/api/v1/reconciliation/split"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&body)
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
