//! Annuler le règlement d'une facture fournisseur par l'API — Story 25-3-a-2 (#414).
//!
//! ⚠️ Premier fichier de tests HTTP des factures fournisseurs : le montage de
//! l'application est repris de `invoice_echeancier_e2e.rs` (chaque binaire de
//! test est un crate à part). Les règles métier sont prouvées côté dépôt
//! (`kesh-db/tests/supplier_invoices_repository.rs`) ; ici, ce qui ne se voit
//! qu'à travers HTTP : les champs de lecture, les rôles, les codes d'erreur.
//!
//! Pré-requis : MariaDB démarré.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice};
use kesh_db::repositories::{contacts, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
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
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// Une facture fournisseur de 100.00 (TVA 0), ouverte ; rend son id.
async fn open_invoice(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    // Le seed ne pose pas de compte fournisseurs par défaut : montage repris de
    // `kesh-db/tests/supplier_invoices_repository.rs::setup`.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let supplier = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Fournisseur SA".into(),
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
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
    .id;
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: supplier,
            supplier_invoice_number: Some("FF-1".into()),
            invoice_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur")
    .invoice
    .id
}

async fn get(app: &TestApp, token: &str, path: &str) -> reqwest::Response {
    app.client
        .get(app.url(path))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

async fn post(app: &TestApp, token: &str, path: &str, body: Value) -> reqwest::Response {
    app.client
        .post(app.url(path))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// ⛔ **Le parcours nominal, par HTTP** : `pay` rend déjà les champs de lecture
/// (l'écran remplace son état par cette réponse) ; l'annulation rend la facture
/// ouverte, colonnes vidées, et l'écriture inverse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pay_then_cancel_the_settlement(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let v: Value = get(&app, &token, &format!("/api/v1/supplier-invoices/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        v["settlementCancellable"], false,
        "ouverte : rien à annuler"
    );
    assert_eq!(v["settlementCancelBlockedBy"], "SUPPLIER_INVOICE_NOT_PAID");

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/pay"),
        json!({
            "settlementType": "internal_account",
            "accountId": seeded.accounts["1000"],
            "paymentDate": "2026-06-20"
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["status"], "paid");
    assert_eq!(
        v["settlementCancellable"], true,
        "la réponse de `pay` porte les champs : l'écran remplace son état par elle"
    );
    assert!(v["lastConfirmedBatch"].is_null());

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let v: Value = resp.json().await.unwrap();
    assert!(v["reversalJournalEntryId"].as_i64().unwrap() > 0);
    assert_eq!(v["invoice"]["status"], "open");
    assert!(v["invoice"]["settlementJournalEntryId"].is_null());
    assert!(v["invoice"]["paidAt"].is_null());
    assert_eq!(
        v["invoice"]["settlementCancelBlockedBy"],
        "SUPPLIER_INVOICE_NOT_PAID"
    );
}

/// Rôles et refus : Consultation LIT (200) mais n'annule pas (403) ; une
/// facture étrangère rend 404 ; une facture ouverte, 409 avec son code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_settlement_roles_and_refusals(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "lecteur".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Consultation,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let lecteur = login(&app, "lecteur", "password123").await;

    let path = format!("/api/v1/supplier-invoices/{id}/settlement/cancel");
    let resp = post(&app, &token, &path, json!({})).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "SUPPLIER_INVOICE_NOT_PAID",
        "got {body:?}"
    );

    supplier_invoices::pay(
        &pool,
        seeded.company_id,
        id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    assert_eq!(
        get(&app, &lecteur, &format!("/api/v1/supplier-invoices/{id}"))
            .await
            .status(),
        200
    );
    assert_eq!(post(&app, &lecteur, &path, json!({})).await.status(), 403);
    assert_eq!(
        post(
            &app,
            &token,
            &format!("/api/v1/supplier-invoices/{}/settlement/cancel", id + 9999),
            json!({})
        )
        .await
        .status(),
        404
    );
}
