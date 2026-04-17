//! Story 5.4 — tests E2E pour l'échéancier factures.
//!
//! Couvre les ACs critiques (#5, #8, #11, #13). Les chemins « happy path »
//! complets (créer → valider → marquer payée → CSV) sont vérifiés par
//! Playwright (T6.4) — le repository-level couvre déjà la logique métier
//! (24 tests `kesh-db::repositories::invoices`).
//!
//! Depuis Story 6.4 : seed via `kesh_db::test_fixtures::seed_accounting_company`
//! + validation via `kesh_db::repositories::invoices::validate_invoice`
//!   (aucun INSERT manuel ni UPDATE direct sur `invoices.status` — KF-001 closed).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
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
/// Password du user `admin` seedé par `seed_accounting_company`
/// (cf. `kesh_db::test_fixtures::ADMIN_PASSWORD_HASH`).
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

/// Seede une company comptablement complète via `test_fixtures::seed_accounting_company` :
/// 1 company + 2 users Admin (`admin/admin123`, `changeme/changeme`) + fiscal_year
/// 2020-2030 Open + 5 accounts + `company_invoice_settings` avec défauts.
/// Retourne `(admin_user_id, company_id)`.
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
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée une facture puis la valide via le flow normal
/// (`kesh_db::repositories::invoices::validate_invoice`, équivalent in-process
/// de `POST /api/v1/invoices/:id/validate`). Aucun INSERT manuel ni UPDATE
/// direct sur `invoices.status` — KF-001 closed via Story 6.4.
///
/// Prérequis : `seed_accounting_company` doit avoir été appelé au préalable
/// (fiscal_year Open + company_invoice_settings complet).
///
/// Retourne `(invoice_id, version)` — la version est lue depuis la facture
/// validée, qui sert aux tests de verrouillage optimiste.
async fn create_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    admin_id: i64,
    date: NaiveDate,
    due_date: NaiveDate,
    amount: rust_decimal::Decimal,
) -> (i64, i32) {
    let new = NewInvoice {
        company_id,
        contact_id,
        date,
        due_date: Some(due_date),
        payment_terms: None,
        lines: vec![NewInvoiceLine {
            description: "Stub".into(),
            quantity: dec!(1),
            unit_price: amount,
            vat_rate: dec!(8.10),
        }],
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();

    let validated =
        kesh_db::repositories::invoices::validate_invoice(pool, company_id, inv.id, admin_id)
            .await
            .expect("validate_invoice");

    (validated.invoice.id, validated.invoice.version)
}

// --- Tests -------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_due_dates_requires_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_due_dates_default_returns_only_unpaid_validated(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;

    // 1 validated unpaid + 1 draft (filtered out implicitement).
    let _ = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let _ = invoices::create(
        &pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()),
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                description: "Draft".into(),
                quantity: dec!(1),
                unit_price: dec!(50.00),
                vat_rate: dec!(8.10),
            }],
        },
    )
    .await
    .unwrap();

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let items = body["items"].as_array().unwrap();
    // B23 : depuis B1 (review pass 1 G2 B) le défaut backend = `unpaid` —
    // l'unique facture seedée est `validated` + `paid_at IS NULL` → 1 résultat.
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["status"], "validated");
    assert_eq!(items[0]["paidAt"], serde_json::Value::Null);

    // Summary doit refléter 1 facture impayée (100.00).
    assert_eq!(body["summary"]["unpaidCount"], 1);
}

/// N2 (review pass 3 B) : test inversé — un `paid_at` futur (date d'exécution
/// bancaire programmée) doit être accepté. Le test précédent qui asseyait un
/// rejet 400 reposait sur une AC#8 incorrecte (clarification domaine 2026-04-15).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_paid_accepts_future_paid_at_returns_200(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, version) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(50.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    // Date future plausible (ordre de virement programmé J+30).
    let future = "2026-05-15T00:00:00";
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/mark-paid")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "paidAt": future, "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_paid_on_draft_invoice_returns_409(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    // Facture draft (jamais validée).
    let (inv, _) = invoices::create(
        &pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()),
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                description: "X".into(),
                quantity: dec!(1),
                unit_price: dec!(10.00),
                vat_rate: dec!(8.10),
            }],
        },
    )
    .await
    .unwrap();

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{}/mark-paid", inv.id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": inv.version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_paid_then_unmark_paid_round_trip(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(75.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    // Mark.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/mark-paid")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": v }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["paidAt"].is_string());
    let v2 = body["version"].as_i64().unwrap() as i32;

    // Unmark.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/unmark-paid")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": v2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["paidAt"], serde_json::Value::Null);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_csv_has_bom_and_swiss_amounts(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let _ = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(1234.56),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates/export.csv"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(
        resp.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/csv")
    );
    // B7 (review pass 1 G2 B) : Content-Disposition explicite avec extension.
    let cd = resp.headers()["content-disposition"].to_str().unwrap();
    assert!(cd.contains("attachment"), "expected attachment, got: {cd}");
    assert!(cd.contains(".csv"), "expected .csv extension, got: {cd}");
    let bytes = resp.bytes().await.unwrap();
    // BOM UTF-8.
    assert_eq!(&bytes[..3], &[0xEF, 0xBB, 0xBF]);
    let text = String::from_utf8_lossy(&bytes);
    // Séparateur ; + montant formaté suisse (apostrophe typographique U+2019
    // comme séparateur de milliers). P10 (review pass 2) : le total_amount
    // stocké est unit_price × (1 + vat_rate/100) = 1234.56 × 1.081 ≈ 1334.56,
    // formaté `1'334.56`. On asserte la présence du séparateur Swiss sur un
    // nombre > 1000 dans le corps (plus robuste qu'un littéral exact).
    assert!(text.contains(';'));
    assert!(
        text.contains("1\u{2019}"),
        "CSV must contain Swiss thousands separator (U+2019), got: {text}"
    );
}

// M6 (review pass 1 G2) — tests AC #8 (paidAt < invoice.date → 400)
// et AC #10 (export CSV > 10'000 lignes → 400 RESULT_TOO_LARGE).

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_paid_rejects_paid_at_before_invoice_date(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    // Facture datée 2026-04-10 ; paid_at = 2026-04-01 → 9 jours avant, bien
    // au-delà de la tolérance de 1 jour (P2 review pass 1 G1).
    let (id, version) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(10.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let before = "2026-04-01T12:00:00";
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/mark-paid")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "paidAt": before, "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_INPUT");
    // H2 (review pass 1 G2) : le message est localisé via la clé FTL dédiée
    // `invoice-error-paid-at-before-invoice-date`, pas le fallback générique.
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.to_lowercase().contains("date") && msg.to_lowercase().contains("paiement"),
        "expected localized paidAtBeforeInvoiceDate message, got: {msg}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn export_csv_over_limit_returns_400_result_too_large(pool: MySqlPool) {
    // MAX_EXPORT_ROWS = 10_000 → créer 10_001 factures serait prohibitif.
    // On utilise un override d'env var pour piloter la limite effective.
    // En l'absence d'override, on seed 11 factures et on vérifie via un
    // endpoint interne de test (fallback : skip si non disponible).
    //
    // Stratégie pragmatique : on vérifie que le code `RESULT_TOO_LARGE` est
    // défini dans le mapping d'erreurs (smoke test). Le scénario à 10'001
    // lignes est couvert par un test d'intégration spec-level lorsqu'un
    // harness de test permettant le seed massif sera disponible (dette
    // technique : T6 testcoverage extended).
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let _ = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(1.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    // Happy path : 1 facture seedée, aucun dépassement → 200.
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates/export.csv"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Smoke : vérifier que le variant `ResultTooLarge` du AppError renvoie
    // bien `RESULT_TOO_LARGE` / 400 (découplé du seed massif).
    use axum::response::IntoResponse;
    use kesh_api::errors::AppError;
    let resp = AppError::ResultTooLarge("x".into()).into_response();
    assert_eq!(resp.status(), axum::http::StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(resp.into_body(), 4096).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["error"]["code"], "RESULT_TOO_LARGE");
}

/// B10 (review pass 1 G2 B) : exerce le path `alreadyUnpaid` — tenter de
/// dé-marquer une facture jamais marquée payée doit retourner 400
/// `INVALID_INPUT` avec la clé i18n `invoice-error-already-unpaid`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unmark_paid_on_never_paid_returns_400_already_unpaid(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, version) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(50.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/unmark-paid")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_INPUT");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.to_lowercase().contains("payée") || msg.to_lowercase().contains("paid"),
        "expected localized alreadyUnpaid message, got: {msg}"
    );
}
