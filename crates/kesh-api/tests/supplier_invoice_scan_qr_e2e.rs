//! Story 12.4 — tests d'intégration pour `POST /api/v1/supplier-invoices/scan-qr`.
//!
//! Endpoint de pré-remplissage : prend un texte SPC (décodé côté navigateur par
//! jsQR) et retourne les coordonnées de paiement. **Lecture seule** — aucune
//! écriture DB. Ces tests tournent contre MariaDB réelle (`#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`) car
//! l'endpoint est derrière le garde `require_comptable_role` (auth + rôle).
//!
//! Le SPC valide est produit par le générateur `kesh_qrbill::build_payload`
//! (inverse exact du parseur `parse_spc_payload` livré par 12-5a), ce qui garantit
//! un round-trip cohérent sans dépendre d'une fixture binaire.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::seed_accounting_company;
use kesh_qrbill::build_payload;
use kesh_qrbill::types::{Address, AddressType, Currency, QrBillData, Reference};
use kesh_qrbill::validation::build_qrr;
use rust_decimal_macros::dec;
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";
const IBAN_OK: &str = "CH9300762011623852957";
const QR_IBAN_OK: &str = "CH4431999123000889012";

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

/// Construit un payload SPC valide via le générateur (miroir du parseur).
fn spc_payload(iban: &str, reference: Reference) -> String {
    let data = QrBillData {
        creditor_iban: iban.into(),
        creditor: Address {
            address_type: AddressType::Combined,
            name: "Robert Schneider SA".into(),
            line1: "Rue du Lac 1268".into(),
            line2: "2501 Biel".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        },
        ultimate_debtor: Some(Address {
            address_type: AddressType::Combined,
            name: "Pia Rutschmann".into(),
            line1: "Marktgasse 28".into(),
            line2: "9400 Rorschach".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        }),
        amount: Some(dec!(1234.50)),
        currency: Currency::Chf,
        reference,
        unstructured_message: Some("Facture F-2026-0042".into()),
        billing_information: None,
    };
    build_payload(&data).expect("build_payload")
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn scan_qr_qr_iban_with_qrr_returns_coordinates(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let qrr = build_qrr(1, 42).expect("qrr");
    let spc = spc_payload(QR_IBAN_OK, Reference::Qrr(qrr.clone()));

    let resp = app
        .client
        .post(app.url("/api/v1/supplier-invoices/scan-qr"))
        .bearer_auth(&token)
        .json(&json!({ "spcText": spc }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    // QR-IBAN → creditorQrIban (pas creditorIban).
    assert_eq!(body["creditorQrIban"], QR_IBAN_OK);
    assert!(body["creditorIban"].is_null());
    assert_eq!(body["paymentReference"], qrr);
    assert_eq!(body["expectedPaymentAmount"], "1234.50");
    assert_eq!(body["currency"], "CHF");
    assert_eq!(body["creditorName"], "Robert Schneider SA");
    assert_eq!(body["unstructuredMessage"], "Facture F-2026-0042");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn scan_qr_classic_iban_no_reference(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let spc = spc_payload(IBAN_OK, Reference::None);

    let resp = app
        .client
        .post(app.url("/api/v1/supplier-invoices/scan-qr"))
        .bearer_auth(&token)
        .json(&json!({ "spcText": spc }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    // IBAN classique → creditorIban (pas creditorQrIban), pas de référence.
    assert_eq!(body["creditorIban"], IBAN_OK);
    assert!(body["creditorQrIban"].is_null());
    assert!(body["paymentReference"].is_null());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn scan_qr_invalid_payload_returns_400(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;
    let token = login(&app).await;

    let resp = app
        .client
        .post(app.url("/api/v1/supplier-invoices/scan-qr"))
        .bearer_auth(&token)
        .json(&json!({ "spcText": "ceci n'est pas un QR SPC" }))
        .send()
        .await
        .unwrap();
    // Erreur métier 4xx (jamais 500) — AC3.
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn scan_qr_requires_authentication(pool: MySqlPool) {
    seed_accounting_company(&pool).await.expect("seed");
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/supplier-invoices/scan-qr"))
        .json(&json!({ "spcText": "SPC\n0200\n1\n" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}
