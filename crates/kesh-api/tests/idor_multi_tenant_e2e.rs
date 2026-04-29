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
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
    };

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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
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
