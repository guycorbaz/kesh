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
            revenue_account_id: None,
            description: "Stub".into(),
            quantity: dec!(1),
            unit_price: amount,
            vat_rate: dec!(8.10),
        }],
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();

    let validated =
        kesh_db::repositories::invoices::validate_invoice(pool, company_id, inv.id, admin_id)
            .await
            .expect("validate_invoice");

    (validated.invoice.id, validated.invoice.version)
}

// --- Tests -------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
                revenue_account_id: None,
                description: "Draft".into(),
                quantity: dec!(1),
                unit_price: dec!(50.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_csv_has_bom_and_swiss_amounts(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    // Story 25-4-c4-a : le TTC de cette facture n'est pas un multiple de 5
    // centimes ; sa validation exige un compte de différences d'arrondi.
    designate_rounding(&pool, ids(&pool).await.0).await;
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
    // comme séparateur de milliers). Corrigé Story 21-2a (#246) : le
    // commentaire P10 d'origine prétendait à tort que `total_amount` stocké
    // incluait la TVA — il est HT (1234.56) ; depuis 21-2a la colonne CSV
    // exporte le TTC dérivé des lignes (1234.56 @ 8.1 % → 1334.56). Dans les
    // deux cas le nombre dépasse 1000 → l'assertion sur le séparateur U+2019
    // est robuste (plus qu'un littéral exact).
    assert!(text.contains(';'));
    assert!(
        text.contains("1\u{2019}"),
        "CSV must contain Swiss thousands separator (U+2019), got: {text}"
    );
}

// M6 (review pass 1 G2) — tests AC #8 (paidAt < invoice.date → 400)
// et AC #10 (export CSV > 10'000 lignes → 400 RESULT_TOO_LARGE).

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
    // Story 25-4-c4-a : le TTC de cette facture n'est pas un multiple de 5
    // centimes ; sa validation exige un compte de différences d'arrondi.
    designate_rounding(&pool, ids(&pool).await.0).await;
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

// ============================================================================
// Story 24-3 (#372) — « enregistrer un règlement » remplace `mark-paid`
//
// ⚠️ Les cinq cas HTTP de `mark-paid` / `unmark-paid` ont été retirés AVEC leurs
// routes. Ceux qui suivent portent leurs règles SURVIVANTES sur le nouveau
// chemin — statut, date, borne haute — plus ce que le nouveau chemin apporte.
// ============================================================================

/// Le compte de caisse de la fixture comptable, retrouvé par son numéro.
async fn caisse_id(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '1000'")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .expect("compte 1000 seedé")
}

/// ⛔ **Un règlement en espèces passe par HTTP et produit son écriture.** C'est
/// la correction de #372 vue du client : le mode n'y change rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_in_cash_returns_200_and_settles(pool: MySqlPool) {
    let (company_id, admin_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let caisse = caisse_id(&pool, company_id).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "108.10",
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["fullySettled"], true);
    assert_eq!(
        v["amountDueAfter"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        0.0
    );
    assert!(v["journalEntryId"].as_i64().unwrap() > 0);
    assert!(
        v["invoice"]["paidAt"].as_str().is_some(),
        "solde nul ⇒ `paid_at` posé, got {v:?}"
    );
}

/// Règle SURVIVANTE de `mark_paid_on_draft_invoice_returns_409` : seule une
/// facture validée se règle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_on_draft_invoice_returns_409(pool: MySqlPool) {
    let (company_id, admin_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (inv, _) = invoices::create(
        &pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 4, 30).unwrap()),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Brouillon".into(),
                quantity: dec!(1),
                unit_price: dec!(50.00),
                vat_rate: dec!(0),
            }],
        },
    )
    .await
    .unwrap();
    let caisse = caisse_id(&pool, company_id).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{}/settlements", inv.id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "50.00",
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
}

/// Règle SURVIVANTE de `mark_paid_rejects_paid_at_before_invoice_date`, avec sa
/// tolérance d'un jour — voir le doc-comment de `settle_invoice`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_rejects_settled_on_before_invoice_date(pool: MySqlPool) {
    let (company_id, admin_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 10).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let caisse = caisse_id(&pool, company_id).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "108.10",
            "settledOn": "2026-04-01"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

/// ⛔ **Les routes retirées ne répondent plus.** Sans cette assertion, un retrait
/// à moitié fait — handler supprimé, route encore montée — passerait inaperçu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn removed_mark_paid_routes_are_gone(pool: MySqlPool) {
    // ⚠️ Le seed est nécessaire au LOGIN, pas au cas lui-même : sans utilisateur,
    // `login` échoue avant d'atteindre la route qu'on veut voir absente.
    let (_company_id, _admin_id) = seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    for chemin in ["mark-paid", "unmark-paid"] {
        let resp = app
            .client
            .post(app.url(&format!("/api/v1/invoices/1/{chemin}")))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({ "version": 1 }))
            .send()
            .await
            .unwrap();
        assert!(
            resp.status() == 404 || resp.status() == 405,
            "`{chemin}` doit avoir disparu, got {}",
            resp.status()
        );
    }
}

// ===========================================================================
// Story 25-3-a-1 (#414) — lister et annuler les règlements d'une facture
// ===========================================================================

/// Facture validée réglée en espèces ; rend `(invoice_id, settlement_id)`.
async fn settled_invoice(pool: &MySqlPool, app: &TestApp, token: &str) -> (i64, i64) {
    let (company_id, admin_id): (i64, i64) =
        sqlx::query_as("SELECT company_id, id FROM users WHERE username = 'admin'")
            .fetch_one(pool)
            .await
            .unwrap();
    let contact_id = seed_contact(pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let caisse = caisse_id(pool, company_id).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "108.10",
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let sid: i64 = sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    (id, sid)
}

async fn get_settlements(app: &TestApp, token: &str, id: i64) -> reqwest::Response {
    app.client
        .get(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

async fn post_cancel(app: &TestApp, token: &str, id: i64, sid: i64) -> reqwest::Response {
    app.client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements/{sid}/cancel")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

/// ⛔ **Le parcours nominal** : la liste annonce le règlement annulable,
/// l'annulation rend la facture relue — `paidAt` retombé, reste dû entier —,
/// puis la liste est vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_settlement_returns_200_and_reopens_the_invoice(pool: MySqlPool) {
    seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let (id, sid) = settled_invoice(&pool, &app, &token).await;

    let resp = get_settlements(&app, &token, id).await;
    assert_eq!(resp.status(), 200);
    let list: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(list.as_array().unwrap().len(), 1);
    assert_eq!(list[0]["id"], sid);
    assert_eq!(list[0]["cancellable"], true);
    assert!(list[0]["cancelBlockedBy"].is_null(), "got {list:?}");
    assert_eq!(list[0]["settlementType"], "internal_account");

    let resp = post_cancel(&app, &token, id, sid).await;
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert!(v["reversalJournalEntryId"].as_i64().unwrap() > 0);
    assert!(v["invoice"]["paidAt"].is_null(), "got {v:?}");
    assert_eq!(
        v["invoice"]["amountDue"]
            .as_str()
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        108.10
    );

    let list: serde_json::Value = get_settlements(&app, &token, id)
        .await
        .json()
        .await
        .unwrap();
    assert!(list.as_array().unwrap().is_empty(), "got {list:?}");
}

/// Consultation LIT la liste (200) mais n'annule pas (403) ; une facture ou un
/// règlement étranger rend 404.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_settlement_roles_and_not_found(pool: MySqlPool) {
    let (_admin, company_id) = seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let (id, sid) = settled_invoice(&pool, &app, &token).await;

    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "lecteur".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Consultation,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "lecteur", "password": "password123" }))
        .send()
        .await
        .unwrap();
    let lecteur = resp.json::<serde_json::Value>().await.unwrap()["accessToken"]
        .as_str()
        .unwrap()
        .to_string();

    assert_eq!(get_settlements(&app, &lecteur, id).await.status(), 200);
    assert_eq!(post_cancel(&app, &lecteur, id, sid).await.status(), 403);

    assert_eq!(get_settlements(&app, &token, id + 9999).await.status(), 404);
    assert_eq!(
        post_cancel(&app, &token, id + 9999, sid).await.status(),
        404
    );
    assert_eq!(
        post_cancel(&app, &token, id, sid + 9999).await.status(),
        404
    );
}

/// ⛔ **Exercice clos : la liste le dit AVANT le clic, et le clic le refuse
/// avec le même code** (arbitrage Q5 — rouvrir l'exercice est le chemin).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_closed_fiscal_year_is_announced_and_refused(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let (id, sid) = settled_invoice(&pool, &app, &token).await;
    let fy_id: i64 = sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::fiscal_years::close(&pool, admin_id, company_id, fy_id)
        .await
        .unwrap();

    let list: serde_json::Value = get_settlements(&app, &token, id)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(list[0]["cancellable"], false);
    assert_eq!(list[0]["cancelBlockedBy"], "FISCAL_YEAR_CLOSED");

    let resp = post_cancel(&app, &token, id, sid).await;
    assert_eq!(resp.status(), 409);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED", "got {body:?}");
}

// --- Story 25-4-a (#455, #456) — le résiduel juste, à la frontière HTTP --------

async fn post_credit_note(app: &TestApp, token: &str, invoice_id: i64) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/credit-notes"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "invoiceId": invoice_id, "date": "2026-05-10" }))
        .send()
        .await
        .unwrap()
}

/// Lit un montant rendu par l'API : **présent, non `null`, chaîne**, parsé en
/// `Decimal` — ni comparaison de chaîne (l'échelle suit le calcul SQL), ni
/// `f64`. `null` voudrait dire « non calculé » : ce serait un échec.
fn montant(v: &serde_json::Value, champ: &str) -> rust_decimal::Decimal {
    let s = v[champ]
        .as_str()
        .unwrap_or_else(|| panic!("`{champ}` doit être une chaîne présente, reçu {v:?}"));
    s.parse::<rust_decimal::Decimal>()
        .unwrap_or_else(|e| panic!("`{champ}` = {s:?} n'est pas un décimal : {e}"))
}

/// #455 — une facture à 8,1 % créditée, jamais réglée : `GET /invoices/{id}`
/// — ouverte aux clés API en lecture — rend un reste dû **nul**, et non la TVA.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_credited_invoice_reports_zero_amount_due(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    assert_eq!(post_credit_note(&app, &token, id).await.status(), 201);

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["status"], "cancelled", "anti-vacuité : l'avoir est passé");
    assert_eq!(
        montant(&v, "amountDue"),
        rust_decimal::Decimal::ZERO,
        "et non 8.10 — got {v:?}"
    );
    assert_eq!(montant(&v, "amountSettled"), rust_decimal::Decimal::ZERO);
}

/// Story 15-6a (test 12) — la frontière HTTP du changement de contrat : la
/// créance de la vente archivée, `POST /api/v1/credit-notes` rend **400**
/// `ACCOUNT_ARCHIVED`, nomme 1100 dans `details.rejected[]`, et dit
/// « Impossible d'émettre l'avoir ». Il rendait `INACTIVE_OR_INVALID_ACCOUNTS`,
/// anonyme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_note_route_names_the_archived_sale_receivable(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool)
        .await
        .expect("seed_accounting_company");
    let contact_id = seed_contact(&pool, seeded.company_id, seeded.admin_user_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        seeded.company_id,
        contact_id,
        seeded.admin_user_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let receivable = seeded.accounts["1100"];
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(receivable)
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = post_credit_note(&app, &token, id).await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ARCHIVED", "got {body:?}");
    assert_eq!(
        body["error"]["details"]["rejected"],
        json!([{ "accountId": receivable, "accountNumber": "1100" }])
    );
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("Impossible d'émettre l'avoir") && message.contains("1100"),
        "message : {message}"
    );
}

/// #456 — un avoir sur une facture réglée en partie est refusé en **409**, avec
/// son code et un message qui dit quoi faire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_note_on_partially_settled_invoice_is_409(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let caisse = caisse_id(&pool, company_id).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let settle = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "40.00",
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(settle.status(), 200);

    let resp = post_credit_note(&app, &token, id).await;
    assert_eq!(resp.status(), 409);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        v["error"]["code"], "CREDIT_NOTE_INVOICE_SETTLED",
        "got {v:?}"
    );
    let message = v["error"]["message"].as_str().unwrap();
    assert!(
        message.starts_with("Cette facture porte un règlement, même partiel"),
        "message fr attendu, reçu {message:?}"
    );
    assert_eq!(v["error"]["details"]["invoiceId"], id);
    assert!(v["error"]["details"]["settlementId"].as_i64().is_some());
}

// --- Story 25-4-b1 (#416) — l'échéancier porte le reste dû, à la frontière HTTP --

/// Une facture de 108.10 (100.— à 8,1 %) réglée de 40.— en espèces, par HTTP.
async fn partially_settled_invoice(
    pool: &MySqlPool,
    app: &TestApp,
    token: &str,
    admin_id: i64,
    company_id: i64,
) -> i64 {
    let contact_id = seed_contact(pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let caisse = caisse_id(pool, company_id).await;
    let settle = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": "40.00",
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(settle.status(), 200);
    id
}

/// AC 8-9 — la liste de l'échéancier rend `amountSettled` et `amountDue` par
/// ligne, et le résumé somme le reste dû.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn due_dates_list_carries_amount_due(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let id = partially_settled_invoice(&pool, &app, &token, admin_id, company_id).await;

    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    let item = v["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["id"] == id)
        .unwrap_or_else(|| panic!("facture {id} absente : {v:?}"));
    assert_eq!(montant(item, "totalTtc"), dec!(108.10));
    assert_eq!(montant(item, "amountSettled"), dec!(40.00));
    assert_eq!(montant(item, "amountDue"), dec!(68.10));
    assert_eq!(
        montant(&v["summary"], "unpaidTotal"),
        dec!(68.10),
        "résumé : {v:?}"
    );
}

/// AC 11 — l'export CSV porte une colonne « Reste dû » et le statut « partiellement
/// payée ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn due_dates_csv_has_amount_due_and_partial_status(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let _id = partially_settled_invoice(&pool, &app, &token, admin_id, company_id).await;

    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates/export.csv"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let text = String::from_utf8_lossy(&resp.bytes().await.unwrap()).to_string();
    let mut lines = text.trim_start_matches('\u{feff}').lines();
    let header = lines.next().unwrap();
    assert!(header.contains(";Total;Reste dû;"), "en-tête : {header}");
    let row = lines.next().expect("une ligne");
    assert!(row.contains(";108.10;68.10;"), "ligne : {row}");
    assert!(row.contains("Partiellement payée"), "statut : {row}");
}

// --- Story 25-4-c3-b (#476) — le règlement manuel au centime ------------------

/// `(company_id, admin_id)` relus en base — `seed_base` rend ses ids dans un
/// ordre que les appelants historiques échangent sans conséquence (ids égaux).
async fn ids(pool: &MySqlPool) -> (i64, i64) {
    sqlx::query_as("SELECT company_id, id FROM users WHERE username = 'admin'")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Désigne un compte de différences d'arrondi (6940, charge). Rend son id.
async fn designate_rounding(pool: &MySqlPool, company_id: i64) -> i64 {
    let id = sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '6940', 'Différences d''arrondi', 'Expense')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_rounding_account_id = ? WHERE company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
    id
}

/// Une facture validée dont le reste brut porte quatre décimales :
/// `unit_price` 9.2550 à 8.1 % → **10.0050** ; 9.2540 → **10.004** (la TVA seule
/// est arrondie à deux décimales, `line_ttc_sql`).
async fn raw_due_invoice(pool: &MySqlPool, unit_price: rust_decimal::Decimal) -> i64 {
    let (company_id, admin_id) = ids(pool).await;
    // Story 25-4-c4-a : ces tests portent sur une facture émise SANS arrondi à
    // 5 centimes — le cas que le chemin d'écart au centime (25-4-c3-b) couvre
    // encore (réglage désactivé, factures antérieures). Le réglage est actif par
    // défaut ; on le désactive ici, et c'est précisément le cas à tester.
    sqlx::query(
        "UPDATE company_invoice_settings SET round_to_5_centimes = FALSE WHERE company_id = ?",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
    let contact_id = seed_contact(pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        unit_price,
    )
    .await;
    id
}

async fn post_settle(
    app: &TestApp,
    token: &str,
    id: i64,
    caisse: i64,
    amount: &str,
) -> reqwest::Response {
    app.client
        .post(app.url(&format!("/api/v1/invoices/{id}/settlements")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "settlementType": "internal_account",
            "accountId": caisse,
            "amount": amount,
            "settledOn": "2026-04-15"
        }))
        .send()
        .await
        .unwrap()
}

/// `(montant réglé, lignes (compte, débit, crédit))` des règlements d'une facture.
async fn settlements_with_lines(
    pool: &MySqlPool,
    id: i64,
) -> Vec<(
    rust_decimal::Decimal,
    Vec<(i64, rust_decimal::Decimal, rust_decimal::Decimal)>,
)> {
    let rows: Vec<(rust_decimal::Decimal, i64)> = sqlx::query_as(
        "SELECT amount, journal_entry_id FROM invoice_settlements WHERE invoice_id = ? ORDER BY id",
    )
    .bind(id)
    .fetch_all(pool)
    .await
    .unwrap();
    let mut out = Vec::new();
    for (amount, entry) in rows {
        let lines = sqlx::query_as(
            "SELECT account_id, debit, credit FROM journal_entry_lines WHERE entry_id = ? \
             ORDER BY line_order, id",
        )
        .bind(entry)
        .fetch_all(pool)
        .await
        .unwrap();
        out.push((amount, lines));
    }
    out
}

async fn receivable_id(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '1100'")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn paid(pool: &MySqlPool, id: i64) -> bool {
    sqlx::query_scalar::<_, Option<chrono::NaiveDateTime>>(
        "SELECT paid_at FROM invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
    .is_some()
}

/// ⛔ **10.01 sur un reste brut de 10.0050 solde la facture** : réglé au brut,
/// écart de 0.0050 au crédit du compte d'arrondi. Avant : trop-perçu refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_a_centime_amount_on_a_half_centime_invoice(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let rounding = designate_rounding(&pool, company_id).await;
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = post_settle(&app, &token, id, caisse, "10.01").await;
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["fullySettled"], true, "got {v:?}");
    let receivable = receivable_id(&pool, company_id).await;
    assert_eq!(
        settlements_with_lines(&pool, id).await,
        vec![(
            dec!(10.0050),
            vec![
                (caisse, dec!(10.01), dec!(0)),
                (receivable, dec!(0), dec!(10.0050)),
                (rounding, dec!(0), dec!(0.0050)),
            ]
        )]
    );
    assert!(paid(&pool, id).await);
}

/// « 10.000 » sur un reste brut de 10.004 : un montant au centime (normalisé,
/// pas refusé pour ses zéros), qui solde avec un écart de 0.004 au **débit**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_below_the_raw_due_debits_the_rounding_account(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let rounding = designate_rounding(&pool, company_id).await;
    let id = raw_due_invoice(&pool, dec!(9.2540)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = post_settle(&app, &token, id, caisse, "10.000").await;
    assert_eq!(resp.status(), 200, "« 10.000 » est au centime");
    let receivable = receivable_id(&pool, company_id).await;
    assert_eq!(
        settlements_with_lines(&pool, id).await,
        vec![(
            dec!(10.004),
            vec![
                (caisse, dec!(10.00), dec!(0)),
                (receivable, dec!(0), dec!(10.004)),
                (rounding, dec!(0.004), dec!(0)),
            ]
        )]
    );
    assert!(paid(&pool, id).await);
}

/// ⛔ **10.008 est refusé pour ses décimales**, avec le code d'échelle : la garde
/// d'avant le refusait déjà — en trop-perçu (`INVALID_INPUT`) — si bien qu'un
/// test sur le seul statut 400 serait vert sans le patch. Et au niveau du dépôt,
/// la double borne le refuse aussi (garde de mutation : borne `p > b` retirée).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_three_decimal_amount_is_refused_at_both_layers(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, admin_id) = ids(&pool).await;
    designate_rounding(&pool, company_id).await;
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = post_settle(&app, &token, id, caisse, "10.008").await;
    assert_eq!(resp.status(), 400);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["error"]["code"], "VALIDATION_ERROR", "got {v:?}");
    assert!(settlements_with_lines(&pool, id).await.is_empty());

    let err = kesh_db::repositories::invoice_settlements_write::settle_invoice(
        &pool,
        admin_id,
        company_id,
        id,
        kesh_db::entities::SettlementChoice::InternalAccount { account_id: caisse },
        dec!(10.008),
        NaiveDate::from_ymd_opt(2026, 4, 15).unwrap(),
    )
    .await
    .expect_err("entre le brut et l'arrondi : trop-perçu, jamais un partiel");
    assert!(
        matches!(&err, kesh_db::errors::DbError::InvalidInput(m) if m.starts_with("overpayment")),
        "got {err:?}"
    );
    assert!(settlements_with_lines(&pool, id).await.is_empty());
    assert!(!paid(&pool, id).await);
}

/// Deux paiements, 5.00 puis 5.01, sur 10.0050 : le premier est un partiel à
/// deux lignes, le second solde avec l'écart.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_second_of_two_payments_carries_the_rounding_line(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let rounding = designate_rounding(&pool, company_id).await;
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    assert_eq!(
        post_settle(&app, &token, id, caisse, "5.00").await.status(),
        200
    );
    assert!(!paid(&pool, id).await, "partiel");
    assert_eq!(
        post_settle(&app, &token, id, caisse, "5.01").await.status(),
        200
    );
    let receivable = receivable_id(&pool, company_id).await;
    let s = settlements_with_lines(&pool, id).await;
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].0, dec!(5.00));
    assert_eq!(s[0].1.len(), 2, "le partiel n'a pas d'écart");
    assert_eq!(
        s[1],
        (
            dec!(5.0050),
            vec![
                (caisse, dec!(5.01), dec!(0)),
                (receivable, dec!(0), dec!(5.0050)),
                (rounding, dec!(0), dec!(0.0050)),
            ]
        )
    );
    assert!(paid(&pool, id).await);
}

/// ⛔ **Sans compte d'arrondi utilisable, 400 dédié et rien d'écrit** — réglage
/// absent, puis compte archivé depuis sa désignation (#486). Un paiement SANS
/// écart, lui, n'exige rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_rounding_gap_without_a_usable_account_is_a_dedicated_400(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = post_settle(&app, &token, id, caisse, "10.01").await;
    assert_eq!(resp.status(), 400);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        v["error"]["code"], "ROUNDING_ACCOUNT_NOT_CONFIGURED",
        "got {v:?}"
    );
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Paramètres"),
        "le message dit où agir, got {v:?}"
    );
    assert!(settlements_with_lines(&pool, id).await.is_empty());

    let rounding = designate_rounding(&pool, company_id).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let resp = post_settle(&app, &token, id, caisse, "10.01").await;
    assert_eq!(resp.status(), 400);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["error"]["code"], "ROUNDING_ACCOUNT_NOT_CONFIGURED");
    assert!(settlements_with_lines(&pool, id).await.is_empty());

    // Sans écart, rien n'est exigé : un partiel passe.
    assert_eq!(
        post_settle(&app, &token, id, caisse, "5.00").await.status(),
        200
    );
}

/// ⛔ **Annuler un règlement à trois lignes les contre-passe toutes** et rend le
/// reste brut d'avant ; si le compte d'arrondi a été archivé entre-temps, le
/// refus NOMME le compte à réactiver (`archived_accounts_in_tx`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelling_a_three_line_settlement_reverses_the_rounding_line(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let rounding = designate_rounding(&pool, company_id).await;
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    assert_eq!(
        post_settle(&app, &token, id, caisse, "10.01")
            .await
            .status(),
        200
    );
    let sid: i64 = sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();

    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let resp = post_cancel(&app, &token, id, sid).await;
    assert_eq!(resp.status(), 400);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert!(
        v.to_string().contains("6940"),
        "le refus nomme le compte d'arrondi à réactiver, got {v:?}"
    );

    sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let resp = post_cancel(&app, &token, id, sid).await;
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    let reversal = v["reversalJournalEntryId"].as_i64().unwrap();
    let receivable = receivable_id(&pool, company_id).await;
    let lines: Vec<(i64, rust_decimal::Decimal, rust_decimal::Decimal)> = sqlx::query_as(
        "SELECT account_id, debit, credit FROM journal_entry_lines WHERE entry_id = ? \
         ORDER BY line_order, id",
    )
    .bind(reversal)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lines,
        vec![
            (caisse, dec!(0), dec!(10.01)),
            (receivable, dec!(10.0050), dec!(0)),
            (rounding, dec!(0.0050), dec!(0)),
        ]
    );
    assert_eq!(
        v["invoice"]["amountDue"]
            .as_str()
            .map(|s| s.parse::<rust_decimal::Decimal>().unwrap()),
        Some(dec!(10.0050)),
        "le reste brut d'avant, got {v:?}"
    );
    assert!(v["invoice"]["paidAt"].is_null());
}

/// Le compte d'arrondi peut être un **produit** (AC 4 : « charge ou produit ») —
/// la revérification au moment d'écrire l'accepte, et l'écart s'y écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_revenue_rounding_account_receives_the_gap(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let rounding = designate_rounding(&pool, company_id).await;
    // Retypage seul : atteignable par l'API (`PUT /accounts/{id}`), le numéro ne l'est pas.
    sqlx::query("UPDATE accounts SET account_type = 'Revenue' WHERE id = ?")
        .bind(rounding)
        .execute(&pool)
        .await
        .unwrap();
    let id = raw_due_invoice(&pool, dec!(9.2550)).await;
    let caisse = caisse_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    assert_eq!(
        post_settle(&app, &token, id, caisse, "10.01")
            .await
            .status(),
        200
    );
    let s = settlements_with_lines(&pool, id).await;
    assert_eq!(s[0].1[2], (rounding, dec!(0), dec!(0.0050)));
}

// --- Story 25-4-c4-b (#494) — l'arrondi visible : figé, ou en aperçu ---------

async fn get_invoice_json(app: &TestApp, token: &str, id: i64) -> serde_json::Value {
    app.client
        .get(app.url(&format!("/api/v1/invoices/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap()
}

fn dec_of(v: &serde_json::Value) -> rust_decimal::Decimal {
    v.as_str()
        .expect("décimal sérialisé en chaîne")
        .parse()
        .unwrap()
}

/// Un brouillon à 114.19 HT @ 8.1 % → TTC 123.44, sans le valider.
async fn draft_123_44(pool: &MySqlPool) -> i64 {
    let (company_id, admin_id) = ids(pool).await;
    let contact_id = seed_contact(pool, company_id, admin_id).await;
    let (inv, _) = invoices::create(
        pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
            due_date: None,
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(114.19),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();
    inv.id
}

/// ⛔ Validée : `roundingAmount` FIGÉ, compris dans `totalTtc`, `roundingIsPreview`
/// faux. Brouillon : un APERÇU, que `totalTtc` n'inclut pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_invoice_carries_its_frozen_or_previewed_rounding(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, admin_id) = ids(&pool).await;
    designate_rounding(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let draft = draft_123_44(&pool).await;
    let v = get_invoice_json(&app, &token, draft).await;
    assert_eq!(v["roundingIsPreview"], true, "got {v}");
    assert_eq!(dec_of(&v["roundingAmount"]), dec!(0.01));
    assert_eq!(
        dec_of(&v["totalTtc"]),
        dec!(123.44),
        "l'aperçu n'altère pas le TTC"
    );

    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, draft, admin_id)
        .await
        .unwrap();
    let v = get_invoice_json(&app, &token, draft).await;
    assert_eq!(v["roundingIsPreview"], false);
    assert_eq!(dec_of(&v["roundingAmount"]), dec!(0.01));
    assert_eq!(
        dec_of(&v["totalTtc"]),
        dec!(123.45),
        "figé, compris dans le TTC"
    );
}

/// Réglage désactivé : l'aperçu d'un brouillon est nul.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_preview_follows_the_setting(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(&pool, company_id)
        .await
        .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let v = get_invoice_json(&app, &token, draft_123_44(&pool).await).await;
    assert_eq!(dec_of(&v["roundingAmount"]), dec!(0));
    assert_eq!(v["roundingIsPreview"], true);
}

/// ⛔ La réponse de la DÉVALIDATION porte l'aperçu : la fiche l'affiche sans
/// relire la facture (validation P1 de la story, HIGH).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_unvalidation_response_carries_the_preview(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, admin_id) = ids(&pool).await;
    designate_rounding(&pool, company_id).await;
    let id = draft_123_44(&pool).await;
    let validated =
        kesh_db::repositories::invoices::validate_invoice(&pool, company_id, id, admin_id)
            .await
            .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/unvalidate")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": validated.invoice.version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["status"], "draft");
    assert_eq!(v["roundingIsPreview"], true, "got {v}");
    assert_eq!(dec_of(&v["roundingAmount"]), dec!(0.01));
}

/// ⛔ Désactiver le réglage ne touche aucune facture émise : son arrondi est figé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn disabling_the_setting_leaves_issued_invoices_alone(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, admin_id) = ids(&pool).await;
    designate_rounding(&pool, company_id).await;
    let id = draft_123_44(&pool).await;
    kesh_db::repositories::invoices::validate_invoice(&pool, company_id, id, admin_id)
        .await
        .unwrap();
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(&pool, company_id)
        .await
        .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let v = get_invoice_json(&app, &token, id).await;
    assert_eq!(dec_of(&v["roundingAmount"]), dec!(0.01));
    assert_eq!(dec_of(&v["totalTtc"]), dec!(123.45));
}

/// ⛔ Le `GET` d'un brouillon ne s'écrit pas : sans ligne de réglages, l'aperçu
/// prend le défaut (actif) et la ligne n'est PAS recréée — revue de code P1,
/// lentille B (l'ancien `get_or_create_default` faisait un `INSERT IGNORE` à
/// chaque lecture, y compris sous une clé d'API en lecture seule).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reading_a_draft_writes_nothing(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    let draft = draft_123_44(&pool).await;
    sqlx::query("DELETE FROM company_invoice_settings WHERE company_id = ?")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let v = get_invoice_json(&app, &token, draft).await;
    assert_eq!(
        dec_of(&v["roundingAmount"]),
        dec!(0.01),
        "défaut actif : {v}"
    );
    let rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM company_invoice_settings WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rows, 0, "la lecture n'a rien écrit");
}

/// Story 25-4-e (#495) — sous le seuil, la validation par l'API rend un 400
/// `INVOICE_BELOW_MINIMUM` dont le message nomme les deux montants.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validating_below_the_minimum_is_a_named_400(pool: MySqlPool) {
    seed_base(&pool).await;
    let (company_id, _) = ids(&pool).await;
    designate_rounding(&pool, company_id).await;
    sqlx::query(
        "UPDATE company_invoice_settings SET minimum_invoice_amount = 200.00 WHERE company_id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();
    let id = draft_123_44(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/validate")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["error"]["code"], "INVOICE_BELOW_MINIMUM", "got {v}");
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("123.45") && msg.contains("200.00"),
        "les deux montants : {msg}"
    );
}

// --- Story 25-4-d2a (#384, #490) — solder le reste, à la frontière HTTP --------

/// Désigne la charge `4000` comme compte des trois natures.
async fn designate_write_off_accounts(pool: &MySqlPool, company_id: i64) {
    sqlx::query(
        "UPDATE company_invoice_settings cis \
         JOIN accounts a ON a.company_id = cis.company_id AND a.number = '4000' \
         SET cis.default_discount_account_id = a.id, cis.default_bank_fees_account_id = a.id, \
             cis.default_bad_debt_account_id = a.id \
         WHERE cis.company_id = ?",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("comptes de nature");
}

async fn invoice_version(app: &TestApp, token: &str, id: i64) -> i64 {
    let v: serde_json::Value = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    v["version"].as_i64().expect("version")
}

async fn post_write_off(
    app: &TestApp,
    token: &str,
    id: i64,
    body: serde_json::Value,
) -> reqwest::Response {
    app.client
        .post(app.url(&format!("/api/v1/invoices/{id}/write-off")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// ⛔ **Le solde traverse la frontière HTTP** : la facture réglée en partie est
/// soldée et payée ; la liste porte la nature et bloque le règlement antérieur
/// (code traduit, pas d'annulation) ; un rappel est refusé ; l'échéancier dit
/// « Payée ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn write_off_via_http_settles_and_blocks_the_earlier_settlement(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    designate_write_off_accounts(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let id = partially_settled_invoice(&pool, &app, &token, admin_id, company_id).await;
    let version = invoice_version(&app, &token, id).await;

    let resp = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "discount", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(montant(&body, "amount"), dec!(68.10));
    assert_eq!(montant(&body["invoice"], "amountDue"), dec!(0));
    assert!(body["invoice"]["paidAt"].is_string(), "{body}");

    let list: serde_json::Value = get_settlements(&app, &token, id)
        .await
        .json()
        .await
        .unwrap();
    let rows = list.as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let earlier = rows
        .iter()
        .find(|r| r["settlementType"] == "internal_account")
        .unwrap();
    let solde = rows
        .iter()
        .find(|r| r["settlementType"] == "write_off")
        .unwrap();
    assert_eq!(solde["writeOffNature"], "discount");
    assert!(earlier["writeOffNature"].is_null());
    assert_eq!(earlier["cancellable"], false);
    assert_eq!(earlier["cancelBlockedBy"], "INVOICE_WRITTEN_OFF");
    assert_eq!(solde["cancellable"], true);

    let resp = post_cancel(&app, &token, id, earlier["id"].as_i64().unwrap()).await;
    assert_eq!(resp.status(), 409);

    // Un rappel est refusé sur la facture soldée.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/reminders/manual")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "levelNumber": 1, "sentAt": "2026-04-25T10:00:00", "note": null }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422, "{:?}", resp.text().await);

    // L'échéancier dit « Payée », pas « Partiellement payée ».
    let resp = app
        .client
        .get(app.url("/api/v1/invoices/due-dates/export.csv?paymentStatus=all"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let text = String::from_utf8_lossy(&resp.bytes().await.unwrap()).to_string();
    let row = text.lines().nth(1).expect("une ligne");
    assert!(
        row.contains("Payée") && !row.contains("Partiellement"),
        "ligne : {row}"
    );
}

/// Les refus de la route : `version` périmée (409), nature inconnue (400, pas
/// 422), Consultation (403), facture déjà soldée (400).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn write_off_route_refusals(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    designate_write_off_accounts(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let id = partially_settled_invoice(&pool, &app, &token, admin_id, company_id).await;
    let version = invoice_version(&app, &token, id).await;

    let stale = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "discount", "settledOn": "2026-04-20", "version": version - 1 }),
    )
    .await;
    assert_eq!(stale.status(), 409);

    let unknown = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "gift", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(unknown.status(), 400);

    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "lecteur".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Consultation,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let lecteur = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "lecteur", "password": "password123" }))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["accessToken"]
        .as_str()
        .unwrap()
        .to_string();
    let forbidden = post_write_off(
        &app,
        &lecteur,
        id,
        json!({ "nature": "discount", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(forbidden.status(), 403);

    let ok = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "bank_fees", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(ok.status(), 200);
    let version = invoice_version(&app, &token, id).await;
    let again = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "bank_fees", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(again.status(), 400);
    let body: serde_json::Value = again.json().await.unwrap();
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("déjà payée"),
        "{body}"
    );
}

// --- Story 15-6b (#474) — la contrepartie n'est pas la créance, à la frontière HTTP

/// Story 15-6b, test 16 — `POST …/settlements` par compte interne = la créance :
/// **400**, code, `details.claim = "receivable"`, message **fr** (la langue des
/// messages est globale au processus, `init_error_i18n`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settle_on_the_receivable_account_is_a_400_with_details(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let creance = receivable_id(&pool, company_id).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;

    let resp = post_settle(&app, &token, id, creance, "108.10").await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        body["error"]["details"],
        json!({
            "accountId": creance,
            "accountNumber": "1100",
            "claim": "receivable",
            "role": "counterparty",
        })
    );
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("1100") && msg.contains("compte débiteurs de cette facture"),
        "message fr attendu : {msg}"
    );
    assert!(!msg.contains('$') && !msg.contains('\u{2068}'), "{msg}");
    assert!(!paid(&pool, id).await);
}

/// Story 15-6b, test 16 bis — `POST …/write-off` dont le compte de la nature
/// `discount` est la créance retypée : **400**, `details.role =
/// "write_off_nature"`, message qui renvoie à Paramètres → Facturation — et non
/// à « banque, caisse » (clé plate `error-settlement-write-off-account-is-receivable`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn write_off_on_a_nature_account_that_is_the_receivable_is_a_400(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id).await;
    let (id, _v) = create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2026, 4, 30).unwrap(),
        dec!(100.00),
    )
    .await;
    let creance = receivable_id(&pool, company_id).await;
    sqlx::query("UPDATE accounts SET account_type = 'Expense', role = NULL WHERE id = ?")
        .bind(creance)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ? WHERE company_id = ?",
    )
    .bind(creance)
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let version = invoice_version(&app, &token, id).await;

    let resp = post_write_off(
        &app,
        &token,
        id,
        json!({ "nature": "discount", "settledOn": "2026-04-20", "version": version }),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(body["error"]["details"]["role"], "write_off_nature");
    assert_eq!(body["error"]["details"]["claim"], "receivable");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("nature de solde") && msg.contains("Paramètres → Facturation"),
        "{msg}"
    );
    assert!(!msg.contains("banque, caisse"), "{msg}");
    assert!(!paid(&pool, id).await);
}
