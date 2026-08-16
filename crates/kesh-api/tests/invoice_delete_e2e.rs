//! #219 — tests E2E pour la suppression définitive d'une facture validée.
//!
//! Focalisé sur la couche HTTP/auth (le repository couvre déjà les gardes
//! métier — payée / exercice clos / créditée : cf. `kesh_db::repositories::
//! invoices::tests::test_delete_validated_*`). Ici on vérifie :
//! - `DELETE /api/v1/invoices/:id` en Admin sur une validée impayée → 204 +
//!   facture ET écriture comptable liée supprimées ;
//! - la réservation **Admin** (401 sans token, 403 en Comptable).
//!
//! Seed via `kesh_db::test_fixtures::seed_accounting_company` + validation via
//! le flow normal `validate_invoice` (aucun UPDATE direct sur `status`).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{contacts, invoices};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

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

    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login {username}");
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn seed_base(pool: &MySqlPool) -> (i64, i64) {
    let seeded = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    (seeded.admin_user_id, seeded.company_id)
}

async fn seed_contact(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Client X".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
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
    .id
}

/// Crée une facture puis la valide via le flow normal. Retourne
/// `(invoice_id, journal_entry_id)`.
async fn create_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    admin_id: i64,
) -> (i64, i64) {
    let new = NewInvoice {
        company_id,
        contact_id,
        date: chrono::Utc::now().date_naive(),
        due_date: None,
        payment_terms: None,
        lines: vec![NewInvoiceLine {
            revenue_account_id: None,
            description: "Stub".into(),
            quantity: dec!(1),
            unit_price: dec!(100.00),
            vat_rate: dec!(8.10),
        }],
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();
    let validated = invoices::validate_invoice(pool, company_id, inv.id, admin_id)
        .await
        .expect("validate_invoice");
    let je_id = validated
        .invoice
        .journal_entry_id
        .expect("facture validée doit avoir une écriture");
    (validated.invoice.id, je_id)
}

// --- Tests -------------------------------------------------------------------

/// Happy path : Admin supprime une facture validée impayée en exercice ouvert
/// → 204 + la facture ET son écriture comptable liée disparaissent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_validated_as_admin_returns_204(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (invoice_id, je_id) =
        create_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/invoices/{invoice_id}")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    let inv: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(inv.is_none(), "la facture doit être supprimée");

    let je: Option<(i64,)> = sqlx::query_as("SELECT id FROM journal_entries WHERE id = ?")
        .bind(je_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(
        je.is_none(),
        "l'écriture comptable liée doit être supprimée"
    );
}

/// Sans token → 401 (couche auth avant RBAC).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_requires_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (invoice_id, _je_id) =
        create_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/invoices/{invoice_id}")))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

/// La suppression est réservée Admin : un Comptable est refusé (403) et la
/// facture reste intacte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_as_comptable_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (invoice_id, _je_id) =
        create_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    // Admin crée un Comptable, qui se connecte.
    let admin_token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .post(app.url("/api/v1/users"))
        .bearer_auth(&admin_token)
        .json(&json!({
            "username": "comptable1",
            "password": "secure-password-12chars",
            "role": "Comptable"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let comptable_token = login(&app, "comptable1", "secure-password-12chars").await;

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/invoices/{invoice_id}")))
        .bearer_auth(&comptable_token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);

    // La facture n'a pas été touchée.
    let inv: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_optional(&pool)
        .await
        .unwrap();
    assert!(inv.is_some(), "facture intacte après refus 403");
}
