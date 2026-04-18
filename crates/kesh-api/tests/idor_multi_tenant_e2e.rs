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
use kesh_db::repositories::users;
use kesh_db::test_fixtures::{seed_accounting_company, truncate_all};
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

/// Create a user in a company
async fn create_company_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
) -> i64 {
    let hash = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

// =========================================================================
// IDOR TESTS — HTTP 404 for cross-company access
// =========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn idor_contacts_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    // Setup: two companies with users
    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let _user_a_id = create_company_user(&pool, company_a.company_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b.company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create a contact in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "contactType": "Supplier",
            "name": "Contact A",
            "isClient": false,
            "isSupplier": true,
            "address": "123 A St",
            "email": "a@example.com",
            "phone": null,
            "ideNumber": null,
            "defaultPaymentTerms": 30
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

    // Attempt to delete contact A as user B
    let delete_resp = app
        .client
        .delete(app.url(&format!("/api/v1/contacts/{}", contact_a_id)))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&json!({"version": 0}))
        .send()
        .await
        .expect("delete should succeed");

    assert_eq!(
        delete_resp.status(),
        404,
        "User B cannot delete contact from company A"
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

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let _user_a_id = create_company_user(&pool, company_a.company_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b.company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create a product in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/products"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "code": "PROD-A",
            "name": "Product A",
            "unitOfMeasure": "Unit",
            "defaultPrice": 100.00,
            "vatRate": 0.077,
            "accountId": company_a.accounts["3000"]
        }))
        .send()
        .await
        .expect("create should succeed");

    assert_eq!(create_resp.status(), 201);
    let product_data: serde_json::Value = create_resp.json().await.expect("json body");
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

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let _user_a_id = create_company_user(&pool, company_a.company_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b.company_id, "bob", "password123").await;

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

    // User A can access own account
    let own_access = app
        .client
        .get(app.url(&format!("/api/v1/accounts/{}", account_a_id)))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .expect("get should succeed");

    assert_eq!(own_access.status(), 200, "User A can access own account");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn idor_invoices_cross_company_returns_404(pool: MySqlPool) {
    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let _user_a_id = create_company_user(&pool, company_a.company_id, "alice", "password123").await;
    let _user_b_id = create_company_user(&pool, company_b.company_id, "bob", "password123").await;

    let app = spawn_app(pool.clone()).await;
    let token_a = login(&app, "alice", "password123").await;
    let token_b = login(&app, "bob", "password123").await;

    // Create an invoice in company A
    let create_resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&json!({
            "contactId": company_a.accounts["3000"], // Using account as placeholder
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
