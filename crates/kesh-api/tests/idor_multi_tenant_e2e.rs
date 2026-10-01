//! End-to-end HTTP IDOR tests for multi-tenant scoping (Story 6.2).
//!
//! Verifies that HTTP handlers return 404 when users attempt to access resources
//! from other companies. Covers 6 key entities: contacts, products, invoices, accounts, users, companies.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{users, vat_rates};
use kesh_db::test_fixtures::truncate_all;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-password".to_string(),
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
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    // Wait for server to start
    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

/// Login and get access token
async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");

    let body: serde_json::Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
}

/// Create a user in a company (Comptable role by default)
async fn create_company_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
) -> i64 {
    create_company_user_with_role(pool, company_id, username, password, Role::Comptable).await
}

/// Create a user in a company with specified role
async fn create_company_user_with_role(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
    role: Role,
) -> i64 {
    let hash = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

/// Create a company with accounts, fiscal year, and settings (without users)
async fn create_seeded_company(
    pool: &MySqlPool,
) -> (i64, std::collections::HashMap<&'static str, i64>) {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('CI Test Company', 'Test Address 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    // Fiscal year
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("fiscal_year insert");

    // Accounts
    let mut accounts = std::collections::HashMap::new();
    for (code, name, account_type) in &[
        ("1000", "Caisse CI", "Asset"),
        ("1100", "Banque CI", "Asset"),
        ("2000", "Capital CI", "Liability"),
        ("3000", "Ventes CI", "Revenue"),
        ("4000", "Charges CI", "Expense"),
    ] {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("account insert");
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    // Company invoice settings
    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, default_sales_journal) \
         VALUES (?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .execute(pool)
    .await
    .expect("company_invoice_settings insert");

    // Story 7.2 (KF-003) : seed des 4 taux TVA suisses 2024+ pour la company.
    // La nouvelle validation `verify_vat_rates_against_db` exige que la table
    // `vat_rates` contienne le taux passé à `POST /api/v1/products` — sans
    // seed, le test reçoit 400 VALIDATION_ERROR.
    vat_rates::seed_default_swiss_rates(pool, company_id)
        .await
        .expect("vat_rates seed");

    (company_id, accounts)
}

// =========================================================================
// IDOR TESTS — HTTP 404 for cross-company access
// =========================================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_contacts_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    // Setup: two companies with users
    let (company_a_id, _company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create a contact in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Contact A",
            "isClient": false,
            "isSupplier": true,
            "address": "123 A St",
            "email": "a@example.com",
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": "30"
        }))
        .send()
        .await
        .expect("create should succeed");

    assert_eq!(create_resp.status(), 201);
    let contact_data: serde_json::Value = create_resp.json().await.expect("json body");
    let contact_a_id = contact_data["id"].as_i64().expect("id present");

    // Attempt to access contact A as user B (cross-company)
    let get_resp = app
        .client
        .get(app.url(&format!("/api/v1/contacts/{}", contact_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(
        get_resp.status(),
        404,
        "User B cannot access contact from company A"
    );

    // Attempt to archive contact A as user B (cross-company)
    let archive_resp = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{}/archive", contact_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&json!({"version": 0}))
        .send()
        .await
        .expect("archive should succeed");

    assert_eq!(
        archive_resp.status(),
        404,
        "User B cannot archive contact from company A"
    );

    // User A can still access own contact
    let own_access = app
        .client
        .get(app.url(&format!("/api/v1/contacts/{}", contact_a_id)))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(own_access.status(), 200, "User A can access own contact");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_journal_entries_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    // Setup: two companies with users
    let (company_a_id, accounts_a) = create_seeded_company(&pool).await;
    let (company_b_id, _accounts_b) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create a balanced journal entry in company A (debit 1000 / credit 3000).
    let create_resp = app
        .client
        .post(app.url("/api/v1/journal-entries"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "entryDate": "2025-06-15",
            "journal": "Ventes",
            "description": "Écriture IDOR test",
            "lines": [
                { "accountId": accounts_a["1000"], "debit": "100.00", "credit": "0.00" },
                { "accountId": accounts_a["3000"], "debit": "0.00", "credit": "100.00" }
            ]
        }))
        .send()
        .await
        .expect("create should succeed");

    assert_eq!(create_resp.status(), 201);
    let entry_data: serde_json::Value = create_resp.json().await.expect("json body");
    let entry_a_id = entry_data["id"].as_i64().expect("id present");

    // Attempt to access entry A as user B (cross-company) → 404 anti-énumération.
    let get_resp = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{}", entry_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(
        get_resp.status(),
        404,
        "User B cannot access journal entry from company A"
    );

    // Non-existent entry id → 404 (jamais 200 vide).
    let missing_resp = app
        .client
        .get(app.url("/api/v1/journal-entries/99999999"))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(
        missing_resp.status(),
        404,
        "Non-existent journal entry returns 404"
    );

    // User A can access own entry → 200 avec ses lignes.
    let own_access = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{}", entry_a_id)))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(own_access.status(), 200, "User A can access own entry");
    let body: serde_json::Value = own_access.json().await.expect("json body");
    assert_eq!(body["id"].as_i64(), Some(entry_a_id));
    assert_eq!(
        body["lines"].as_array().map(|a| a.len()),
        Some(2),
        "le détail inclut les 2 lignes"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_products_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, _company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create a product in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/products"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "name": "Product A",
            "description": "Test product",
            "unitPrice": "100.00",
            "vatRate": "8.10"
        }))
        .send()
        .await
        .expect("create should succeed");

    let status = create_resp.status();
    let body = create_resp.text().await.expect("body");
    if status != 201 {
        panic!("Create product failed with status {}: {}", status, body);
    }
    let product_data: serde_json::Value = serde_json::from_str(&body).expect("json body");
    let product_a_id = product_data["id"].as_i64().expect("id present");

    // Attempt to access product A as user B (cross-company)
    let get_resp = app
        .client
        .get(app.url(&format!("/api/v1/products/{}", product_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(
        get_resp.status(),
        404,
        "User B cannot access product from company A"
    );

    // User A can access own product
    let own_access = app
        .client
        .get(app.url(&format!("/api/v1/products/{}", product_a_id)))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(own_access.status(), 200, "User A can access own product");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_accounts_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, _company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create an account in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/accounts"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "number": "5000",
            "name": "Test Account",
            "accountType": "Expense",
            "parentId": null
        }))
        .send()
        .await
        .expect("create should succeed");

    assert_eq!(create_resp.status(), 201);
    let account_data: serde_json::Value = create_resp.json().await.expect("json body");
    let account_a_id = account_data["id"].as_i64().expect("id present");

    // Attempt to archive account A as user B (cross-company)
    let archive_resp = app
        .client
        .put(app.url(&format!("/api/v1/accounts/{}/archive", account_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&json!({"version": 0}))
        .send()
        .await
        .expect("archive should succeed");

    assert_eq!(
        archive_resp.status(),
        404,
        "User B cannot archive account from company A"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_invoices_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create an invoice in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "contactId": company_a_accounts["3000"], // Using account as placeholder
            "number": "INV-001",
            "issueDate": "2026-04-18",
            "dueDate": "2026-05-18",
            "lines": [],
            "notes": "Test invoice"
        }))
        .send()
        .await
        .expect("create should succeed");

    if create_resp.status() == 201 {
        let invoice_data: serde_json::Value = create_resp.json().await.expect("json body");
        if let Some(invoice_id) = invoice_data["id"].as_i64() {
            // Attempt to access invoice as user B (cross-company)
            let get_resp = app
                .client
                .get(app.url(&format!("/api/v1/invoices/{}", invoice_id)))
                .header("Authorization", format!("Bearer {}", token_b))
                .send()
                .await
                .expect("get should succeed");

            assert_eq!(
                get_resp.status(),
                404,
                "User B cannot access invoice from company A"
            );

            // User A can access own invoice
            let own_access = app
                .client
                .get(app.url(&format!("/api/v1/invoices/{}", invoice_id)))
                .header("Authorization", format!("Bearer {}", token_a))
                .send()
                .await
                .expect("get should succeed");

            assert_eq!(own_access.status(), 200, "User A can access own invoice");
        }
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_users_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, _company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id =
        create_company_user_with_role(&pool, company_b_id, "bob", "password123", Role::Admin).await;

    let app = spawn_app(pool.clone()).await;
    let token_b = login(&app, "bob", "password123").await;

    // Attempt to disable user A as user B (cross-company) — should return 404
    let disable_resp = app
        .client
        .put(app.url(&format!("/api/v1/users/{}/disable", user_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("disable should succeed");

    assert_eq!(
        disable_resp.status(),
        404,
        "User B cannot disable user from company A"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_companies_current_returns_own_company_only(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, _company_a_accounts) = create_seeded_company(&pool).await;
    let (company_b_id, _company_b_accounts) = create_seeded_company(&pool).await;

    let _user_a_id = create_company_user(&pool, company_a_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // User A access own company — should return 200
    let resp_a = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");
    assert_eq!(resp_a.status(), 200, "User A can access own company");

    // Verify User A gets company A data
    let body_a: serde_json::Value = resp_a.json().await.unwrap();
    assert_eq!(body_a["company"]["id"].as_i64().unwrap(), company_a_id);

    // User B access own company — should return 200
    let resp_b = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .expect("get should succeed");
    assert_eq!(resp_b.status(), 200, "User B can access own company");

    // Verify User B gets company B data (different from A)
    let body_b: serde_json::Value = resp_b.json().await.unwrap();
    assert_eq!(body_b["company"]["id"].as_i64().unwrap(), company_b_id);
    assert_ne!(
        body_b["company"]["id"], body_a["company"]["id"],
        "Users get their own companies"
    );
}

/// Story 18-1a (AC8c) — anti-IDOR sur les nouveaux comptes TVA.
///
/// Le garde `validate_account` de `PUT /company/invoice-settings` rejette un
/// `account_id` **valide mais appartenant à une AUTRE company** (IDOR). C'est un
/// garde-fou distinct de la contrainte FK DB (compte inexistant) couverte au
/// niveau repo (`update_vat_account_foreign_id_rejected_by_fk`) : un compte d'une
/// autre company existe bien dans `accounts` (la FK passe) mais doit être refusé
/// par le check `account.company_id != company_id` du handler route.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn idor_invoice_settings_vat_account_cross_company_rejected(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let (company_a_id, accounts_a) = create_seeded_company(&pool).await;
    let (_company_b_id, accounts_b) = create_seeded_company(&pool).await;

    // `PUT /company/invoice-settings` est Admin-only (cf. lib.rs).
    let _admin_a =
        create_company_user_with_role(&pool, company_a_id, "alice", "password123", Role::Admin)
            .await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;

    // Lire la config courante de A (version + valeurs valides à réémettre).
    let get_resp = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get settings should succeed");
    assert_eq!(get_resp.status(), 200);
    let settings: serde_json::Value = get_resp.json().await.expect("json body");
    let version = settings["version"].as_i64().expect("version present");

    // Compte Liability (2000) de la company B → cible IDOR pour `defaultVatPayableAccountId`
    // (TVA due attend un Liability ; le compte existe mais hors company A).
    let foreign_liability = accounts_b["2000"];

    let put_foreign = app
        .client
        .put(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "invoiceNumberFormat": settings["invoiceNumberFormat"],
            "defaultReceivableAccountId": settings["defaultReceivableAccountId"],
            "defaultRevenueAccountId": settings["defaultRevenueAccountId"],
            "defaultVatPayableAccountId": foreign_liability,
            "defaultVatRecoverableAccountId": null,
            "defaultVatDecompteAccountId": null,
            "defaultSalesJournal": settings["defaultSalesJournal"],
            "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
            "creditNoteNumberFormat": settings["creditNoteNumberFormat"],
            "version": version,
        }))
        .send()
        .await
        .expect("put should succeed");

    assert_eq!(
        put_foreign.status(),
        400,
        "un compte TVA d'une autre company doit être rejeté (anti-IDOR), pas persisté"
    );
    // Épingler la cause exacte : le 400 vient bien de `validate_account` (VALIDATION_ERROR),
    // pas d'un 400 incident (payload malformé, conflit de version) qui masquerait le garde.
    let err_body: serde_json::Value = put_foreign.json().await.expect("error json body");
    assert_eq!(
        err_body["error"]["code"], "VALIDATION_ERROR",
        "le rejet doit être une erreur de validation de compte, pas un autre 400"
    );

    // Contrôle positif : le compte Liability (2000) de A est accepté pour le même champ.
    // (Le PUT précédent ayant échoué, la version n'a pas bougé.)
    let own_liability = accounts_a["2000"];
    let put_own = app
        .client
        .put(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "invoiceNumberFormat": settings["invoiceNumberFormat"],
            "defaultReceivableAccountId": settings["defaultReceivableAccountId"],
            "defaultRevenueAccountId": settings["defaultRevenueAccountId"],
            "defaultVatPayableAccountId": own_liability,
            "defaultVatRecoverableAccountId": null,
            "defaultVatDecompteAccountId": null,
            "defaultSalesJournal": settings["defaultSalesJournal"],
            "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
            "creditNoteNumberFormat": settings["creditNoteNumberFormat"],
            "version": version,
        }))
        .send()
        .await
        .expect("put should succeed");

    assert_eq!(
        put_own.status(),
        200,
        "le compte TVA propre à la company doit être accepté"
    );
}

/// #216 — régression : `PUT /company/invoice-settings` SANS `creditNoteNumberFormat`
/// (cas du formulaire frontend actuel) doit renvoyer **200** (pas 422) et **préserver**
/// le format d'avoir existant. Avant le fix, `credit_note_number_format` était requis
/// côté DTO → serde échouait → 422 → aucun réglage sauvable depuis l'UI.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_update_without_credit_note_format_preserves_and_returns_200(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    // Config courante (version + format d'avoir existant à préserver).
    let get_resp = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get");
    let settings: serde_json::Value = get_resp.json().await.expect("json");
    let version = settings["version"].as_i64().expect("version");
    let existing_cn_format = settings["creditNoteNumberFormat"]
        .as_str()
        .expect("creditNoteNumberFormat présent au GET")
        .to_string();

    // PUT SANS `creditNoteNumberFormat` (comme le fait le formulaire frontend).
    let put = app
        .client
        .put(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "invoiceNumberFormat": settings["invoiceNumberFormat"],
            "defaultReceivableAccountId": accounts["1100"],
            "defaultRevenueAccountId": accounts["3000"],
            "defaultVatPayableAccountId": accounts["2000"],
            "defaultVatRecoverableAccountId": null,
            "defaultVatDecompteAccountId": null,
            "defaultSalesJournal": settings["defaultSalesJournal"],
            "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
            // PAS de creditNoteNumberFormat
            "version": version,
        }))
        .send()
        .await
        .expect("put");
    assert_eq!(
        put.status(),
        200,
        "PUT sans creditNoteNumberFormat doit passer (200), pas 422"
    );

    // Le format d'avoir existant doit être préservé.
    let after: serde_json::Value = put.json().await.expect("json");
    assert_eq!(
        after["creditNoteNumberFormat"], existing_cn_format,
        "le format d'avoir doit être préservé quand le champ est absent"
    );
}

/// `PUT /company/invoice-settings` depuis la config courante, le compte de
/// différences d'arrondi posé à `rounding` — ou **absent** du corps si `None`.
/// Rend le statut et le corps de la réponse.
async fn put_rounding_account(
    app: &TestApp,
    token: &str,
    rounding: Option<serde_json::Value>,
) -> (u16, serde_json::Value) {
    let settings: serde_json::Value = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get")
        .json()
        .await
        .expect("json");
    let mut body = json!({
        "invoiceNumberFormat": settings["invoiceNumberFormat"],
        "defaultReceivableAccountId": settings["defaultReceivableAccountId"],
        "defaultRevenueAccountId": settings["defaultRevenueAccountId"],
        "defaultVatPayableAccountId": settings["defaultVatPayableAccountId"],
        "defaultVatRecoverableAccountId": settings["defaultVatRecoverableAccountId"],
        "defaultVatDecompteAccountId": settings["defaultVatDecompteAccountId"],
        "defaultSalesJournal": settings["defaultSalesJournal"],
        "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
        "version": settings["version"],
    });
    if let Some(value) = rounding {
        body["defaultRoundingAccountId"] = value;
    }
    let resp = app
        .client
        .put(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("put");
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(serde_json::Value::Null))
}

/// Story 25-4-c3-a1 (#476) — le compte de différences d'arrondi : charge **ou**
/// produit, actif, **imputable**, de la société. Tout autre compte est refusé en
/// 400 ; le refus du compte non imputable est la garde que les écritures
/// automatiques ne portent pas (`enforce_postable = false`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_rounding_account_is_validated(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, accounts) = create_seeded_company(&pool).await;
    let (_other_company, other_accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    // Un compte de charge de regroupement (non imputable), et un archivé.
    let mut extra = std::collections::HashMap::new();
    for (code, account_type, active, postable) in [
        ("6000", "Expense", true, false),
        ("6100", "Expense", false, true),
        ("3900", "Revenue", true, false),
        ("3910", "Revenue", false, true),
    ] {
        let id = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
             VALUES (?, ?, 'Compte test', ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(account_type)
        .bind(active)
        .bind(postable)
        .execute(&pool)
        .await
        .expect("account insert")
        .last_insert_id() as i64;
        extra.insert(code, id);
    }
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    for (label, account, expected) in [
        ("charge imputable", accounts["4000"], 200),
        ("produit imputable", accounts["3000"], 200),
        ("actif", accounts["1000"], 400),
        ("passif", accounts["2000"], 400),
        ("charge non imputable", extra["6000"], 400),
        ("charge archivée", extra["6100"], 400),
        ("produit non imputable", extra["3900"], 400),
        ("produit archivé", extra["3910"], 400),
        ("charge d'une autre société", other_accounts["4000"], 400),
    ] {
        let (status, body) = put_rounding_account(&app, &token, Some(json!(account))).await;
        assert_eq!(status, expected, "{label} : {body}");
        if expected == 200 {
            assert_eq!(body["defaultRoundingAccountId"], account, "{label}");
        }
    }
}

/// Story 25-4-c3-a1 — **absent** du corps, le compte d'arrondi est **préservé**
/// (un client qui ignore encore le champ ne l'efface pas) ; **présent à `null`**,
/// il est effacé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_rounding_account_absent_preserves_null_clears(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let (status, body) = put_rounding_account(&app, &token, Some(json!(accounts["4000"]))).await;
    assert_eq!(status, 200, "{body}");

    let (status, body) = put_rounding_account(&app, &token, None).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["defaultRoundingAccountId"], accounts["4000"],
        "absent du corps : préservé"
    );

    let (status, body) = put_rounding_account(&app, &token, Some(serde_json::Value::Null)).await;
    assert_eq!(status, 200, "{body}");
    assert!(
        body["defaultRoundingAccountId"].is_null(),
        "présent à null : effacé ; corps = {body}"
    );
}

/// Story 25-4-c3-a1 — un compte d'arrondi devenu **archivé** après sa désignation
/// (l'archivage n'est pas gardé, #486) ne doit pas bloquer l'enregistrement d'un
/// réglage SANS rapport : reconduit tel quel, il n'est pas revalidé. Le changer
/// pour un compte invalide, lui, reste refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_unchanged_archived_rounding_account_does_not_block_other_changes(
    pool: MySqlPool,
) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let (status, body) = put_rounding_account(&app, &token, Some(json!(accounts["4000"]))).await;
    assert_eq!(status, 200, "{body}");
    // Le compte est archivé ailleurs, après coup.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(accounts["4000"])
        .execute(&pool)
        .await
        .expect("archivage");

    // L'écran renvoie la valeur telle qu'il l'a lue : reconduite, elle passe.
    let (status, body) = put_rounding_account(&app, &token, Some(json!(accounts["4000"]))).await;
    assert_eq!(
        status, 200,
        "⛔ un compte d'arrondi reconduit ne doit pas bloquer l'enregistrement : {body}"
    );
    // Désigner un compte invalide, en revanche, reste refusé.
    let (status, body) = put_rounding_account(&app, &token, Some(json!(accounts["1000"]))).await;
    assert_eq!(status, 400, "{body}");
}

/// Story 25-4-c4-b (#494) — `roundTo5Centimes` : lu en `GET`, changé en `PUT`,
/// **préservé** quand il est absent du corps.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_round_to_5_centimes_is_exposed_and_preserved(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let put = |value: Option<bool>| {
        let app = &app;
        let token = token.clone();
        async move {
            let settings: serde_json::Value = app
                .client
                .get(app.url("/api/v1/company/invoice-settings"))
                .header("Authorization", format!("Bearer {token}"))
                .send()
                .await
                .expect("get")
                .json()
                .await
                .expect("json");
            assert!(
                settings["roundTo5Centimes"].is_boolean(),
                "GET : {settings}"
            );
            let mut body = json!({
                "invoiceNumberFormat": settings["invoiceNumberFormat"],
                "defaultReceivableAccountId": settings["defaultReceivableAccountId"],
                "defaultRevenueAccountId": settings["defaultRevenueAccountId"],
                "defaultVatPayableAccountId": settings["defaultVatPayableAccountId"],
                "defaultVatRecoverableAccountId": settings["defaultVatRecoverableAccountId"],
                "defaultVatDecompteAccountId": settings["defaultVatDecompteAccountId"],
                "defaultSalesJournal": settings["defaultSalesJournal"],
                "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
                "version": settings["version"],
            });
            if let Some(v) = value {
                body["roundTo5Centimes"] = json!(v);
            }
            let resp = app
                .client
                .put(app.url("/api/v1/company/invoice-settings"))
                .header("Authorization", format!("Bearer {token}"))
                .json(&body)
                .send()
                .await
                .expect("put");
            let status = resp.status().as_u16();
            let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
            (status, body)
        }
    };

    let (status, body) = put(Some(false)).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["roundTo5Centimes"], false);
    let (status, body) = put(None).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["roundTo5Centimes"], false,
        "absent du corps : préservé"
    );
    let (status, body) = put(Some(true)).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["roundTo5Centimes"], true);
}

/// Story 25-4-e (#495) — `minimumInvoiceAmount` : posé, **préservé** s'il est
/// absent, **effacé** à `null` ; nul, négatif ou à trois décimales : 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settings_minimum_invoice_amount_is_set_preserved_cleared_and_validated(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");
    let (company_id, _accounts) = create_seeded_company(&pool).await;
    create_company_user_with_role(&pool, company_id, "alice", "password123", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "alice", "password123").await;

    let put = |value: Option<serde_json::Value>| {
        let app = &app;
        let token = token.clone();
        async move {
            let settings: serde_json::Value = app
                .client
                .get(app.url("/api/v1/company/invoice-settings"))
                .header("Authorization", format!("Bearer {token}"))
                .send()
                .await
                .expect("get")
                .json()
                .await
                .expect("json");
            let mut body = json!({
                "invoiceNumberFormat": settings["invoiceNumberFormat"],
                "defaultReceivableAccountId": settings["defaultReceivableAccountId"],
                "defaultRevenueAccountId": settings["defaultRevenueAccountId"],
                "defaultVatPayableAccountId": settings["defaultVatPayableAccountId"],
                "defaultVatRecoverableAccountId": settings["defaultVatRecoverableAccountId"],
                "defaultVatDecompteAccountId": settings["defaultVatDecompteAccountId"],
                "defaultSalesJournal": settings["defaultSalesJournal"],
                "journalEntryDescriptionTemplate": settings["journalEntryDescriptionTemplate"],
                "version": settings["version"],
            });
            if let Some(v) = value {
                body["minimumInvoiceAmount"] = v;
            }
            let resp = app
                .client
                .put(app.url("/api/v1/company/invoice-settings"))
                .header("Authorization", format!("Bearer {token}"))
                .json(&body)
                .send()
                .await
                .expect("put");
            let status = resp.status().as_u16();
            let body: serde_json::Value = resp.json().await.unwrap_or(serde_json::Value::Null);
            (status, body)
        }
    };
    let amount = |b: &serde_json::Value| {
        b["minimumInvoiceAmount"]
            .as_str()
            .map(|s| s.parse::<rust_decimal::Decimal>().unwrap())
    };

    let (status, body) = put(Some(json!("5.00"))).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(amount(&body), Some(rust_decimal_macros::dec!(5.00)));
    let (status, body) = put(None).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        amount(&body),
        Some(rust_decimal_macros::dec!(5.00)),
        "absent : préservé"
    );
    for bad in ["0", "-1.00", "4.005"] {
        let (status, body) = put(Some(json!(bad))).await;
        assert_eq!(status, 400, "{bad} : {body}");
    }
    let (status, body) = put(Some(serde_json::Value::Null)).await;
    assert_eq!(status, 200, "{body}");
    assert!(
        body["minimumInvoiceAmount"].is_null(),
        "null : effacé — {body}"
    );
}
