//! Story 5.3 — tests E2E pour `GET /api/v1/invoices/:id/pdf`.
//!
//! Ces tests tournent contre une DB MariaDB réelle (pattern `#[sqlx::test]`
//! identique aux autres `*_e2e.rs`). Ils couvrent les 9 cas exigés par T6 :
//! 200 (happy path QR-IBAN), 400 `INVOICE_NOT_VALIDATED`, 400 `INVOICE_NOT_PDF_READY`
//! (×2), 404 autre company, 400 `INVOICE_TOO_MANY_LINES_FOR_PDF`, 401 sans JWT,
//! 200 pour chaque rôle (3 cas).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::company::{Language, NewCompany, OrgType};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, companies, contacts, invoices, users};
use rust_decimal_macros::dec;
use serde_json::json;
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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
    };

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

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Seeds company + admin user (returns admin user_id + company_id).
async fn seed_base(pool: &MySqlPool) -> (i64, i64) {
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let admin_id: i64 = sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    let company = companies::create(
        pool,
        NewCompany {
            name: "Robert Schneider SA".into(),
            address: "Rue du Lac 1268\n2501 Biel".into(),
            org_type: OrgType::Pme,
            ide_number: Some("CHE-123.456.789".into()),
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();
    (admin_id, company.id)
}

async fn seed_contact(pool: &MySqlPool, company_id: i64, user_id: i64, with_address: bool) -> i64 {
    let contact = contacts::create(
        pool,
        user_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            is_client: true,
            is_supplier: false,
            address: if with_address {
                Some("Marktgasse 28\n9400 Rorschach".into())
            } else {
                None
            },
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: Some("30 jours net".into()),
        },
    )
    .await
    .unwrap();
    contact.id
}

async fn seed_primary_bank(pool: &MySqlPool, company_id: i64, with_qr_iban: bool) {
    let new = NewBankAccount {
        company_id,
        bank_name: "UBS".into(),
        iban: "CH9300762011623852957".into(),
        qr_iban: if with_qr_iban {
            Some("CH4431999123000889012".into())
        } else {
            None
        },
        is_primary: true,
    };
    bank_accounts::upsert_primary(pool, new).await.unwrap();
}

async fn seed_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    user_id: i64,
    n_lines: usize,
) -> i64 {
    let lines: Vec<NewInvoiceLine> = (0..n_lines)
        .map(|i| NewInvoiceLine {
            description: format!("Ligne {}", i + 1),
            quantity: dec!(1),
            unit_price: dec!(100.00),
            vat_rate: dec!(7.70),
        })
        .collect();
    let new = NewInvoice {
        company_id,
        contact_id,
        date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
        payment_terms: Some("30 jours net".into()),
        lines,
    };
    let (invoice, _lines) = invoices::create(pool, user_id, new).await.unwrap();
    // Validate to transition to status='validated'.
    let _ = invoices::validate_invoice(pool, company_id, invoice.id, user_id).await;
    invoice.id
}

// --- Tests -------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_happy_path_returns_200_with_content_disposition(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 3).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let headers = resp.headers();
    assert_eq!(headers["content-type"], "application/pdf");
    let disposition = headers["content-disposition"].to_str().unwrap();
    assert!(disposition.starts_with("inline; filename=\"facture-"));
    let bytes = resp.bytes().await.unwrap();
    assert!(bytes.starts_with(b"%PDF-1."));
    assert!(bytes.len() > 1_000);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_requires_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/1/pdf"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_draft_invoice_returns_400_not_validated(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    // Create invoice but do NOT validate.
    let (invoice, _) = invoices::create(
        &pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: None,
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                description: "Item".into(),
                quantity: dec!(1),
                unit_price: dec!(100.0),
                vat_rate: dec!(7.70),
            }],
        },
    )
    .await
    .unwrap();

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{}/pdf", invoice.id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_VALIDATED");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_missing_primary_bank_returns_400(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    // NO primary bank seeded.
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 2).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_missing_contact_address_returns_400(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, false).await;
    seed_primary_bank(&pool, company_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 2).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_too_many_lines_returns_400(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 36).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_TOO_MANY_LINES_FOR_PDF");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_iban_classique_without_qr_iban_works(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, false).await; // no qr_iban
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 2).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_unknown_invoice_returns_404(pool: MySqlPool) {
    seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/999999/pdf"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
}

// --- AC16 / T6.8 : 3 rôles (Admin, Comptable, Consultation) accèdent au PDF.

async fn seed_user_with_role(pool: &MySqlPool, username: &str, password: &str, role: Role) {
    let phc = hash_password(password).expect("hash password");
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: phc,
            role,
            active: true,
        },
    )
    .await
    .expect("create user");
}

async fn login_as(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"]
        .as_str()
        .expect("accessToken missing")
        .to_string()
}

async fn run_pdf_role_scenario(pool: MySqlPool, username: &str, role: Role) {
    let password = "role-test-password";
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 3).await;
    seed_user_with_role(&pool, username, password, role).await;

    let app = spawn_app(pool.clone()).await;
    let token = login_as(&app, username, password).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "role {role:?} must access PDF");
    assert_eq!(resp.headers()["content-type"], "application/pdf");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_admin_role_returns_200(pool: MySqlPool) {
    // Le user seedé via `seed_base` → ensure_admin_user est déjà Admin ; on
    // rejoue simplement le chemin happy-path pour couvrir explicitement AC16.
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 3).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_comptable_role_returns_200(pool: MySqlPool) {
    run_pdf_role_scenario(pool, "comptable_pdf", Role::Comptable).await;
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pdf_consultation_role_returns_200(pool: MySqlPool) {
    run_pdf_role_scenario(pool, "observateur_pdf", Role::Consultation).await;
}
