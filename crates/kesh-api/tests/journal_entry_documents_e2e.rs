//! Story 55-3a (#1607) — `GET /api/v1/journal-entries/{id}/documents` : la liste
//! des justificatifs d'une écriture, par la route, avec l'état du PDF d'une
//! facture (`pdfStatus`), ses droits (tout rôle, clés d'API en lecture) et son
//! `404` hors société.
//!
//! La matrice des rattachements (cinq branches, négatifs posés en SQL direct,
//! sociétés) est éprouvée sur la fonction que le service appelle,
//! `kesh_report::documents::documents_of_entry`, dans
//! `crates/kesh-report/tests/documents.rs` ; ici, des données écrites par
//! l'application (validation, gel par `GET /pdf`, avoir).
//!
//! Harnais recopié de `invoice_frozen_pdf_e2e.rs` (un `KESH_DOCUMENTS_DIR` par
//! test : le gel écrit un fichier).

mod common;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::mail::MockMailer;
use kesh_api::middleware::rate_limit::RateLimiter;
use kesh_api::{AppState, build_router};
use kesh_db::entities::Language;
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::credit_note::NewCreditNote;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, contacts, credit_notes, invoices, users};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";
const ROLE_PASSWORD: &str = "role-test-password";

// ============================================================
// Harness
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    #[allow(dead_code)]
    documents: PathBuf,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn unique_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kesh-{prefix}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn load_i18n() -> Arc<kesh_i18n::I18nBundle> {
    Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    )
}

/// `documents` : le `KESH_DOCUMENTS_DIR` de ce test — un chemin de FICHIER
/// le rend non inscriptible.
async fn spawn_app(pool: MySqlPool, mailer: MockMailer, documents: PathBuf) -> TestApp {
    let mut config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
    config.documents_dir = documents.to_string_lossy().to_string();
    let rate_limiter = RateLimiter::new(&config);
    let i18n = load_i18n();
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        rate_limiter_recovery: Arc::new(kesh_api::build_recovery_rate_limiter()),
        rate_limiter_send_email: Arc::new(RateLimiter::with_thresholds(
            100,
            Duration::from_secs(15 * 60),
            Duration::from_secs(15 * 60),
        )),
        i18n,
        users_exist: Arc::new(AtomicBool::new(true)),
        mailer: Arc::new(mailer),
        smtp_ready: true,
        test_mock_mailer: None,
        backup_lock: Arc::new(tokio::sync::Mutex::new(())),
        horloge: kesh_api::horloge::horloge_de_test(),
        // Story 37-2 — source factice, comme `new_for_tests`.
        rate_source: Arc::new(kesh_api::exchange_rates::MockRateSource::new()),
        rate_fetch_lock: Arc::new(tokio::sync::Mutex::new(())),
        rate_fetch_memo: Arc::default(),
        rate_login_trigger: Arc::new(std::sync::Mutex::new(Some(std::time::Instant::now()))),
        test_rate_source: None,
        provider_downloads: Default::default(),
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
        documents,
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

struct Seeded {
    company_id: i64,
    admin_id: i64,
    contact_id: i64,
}

/// Société comptable + banque principale + contact PDF-ready, dans la langue
/// donnée (`None` : celle de l'installation, FR).
async fn seed(pool: &MySqlPool, language: Option<Language>) -> Seeded {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
    let contact = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: Some("Pia".into()),
            last_name: Some("Rutschmann".into()),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: Some("pia@example.ch".into()),
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap();
    Seeded {
        company_id: seeded.company_id,
        admin_id: seeded.admin_user_id,
        contact_id: contact.id,
    }
}

async fn seed_invoice(pool: &MySqlPool, s: &Seeded, n_lines: usize) -> i64 {
    let lines = (0..n_lines)
        .map(|i| NewInvoiceLine {
            revenue_account_id: None,
            line_type: kesh_db::entities::LineType::Priced,
            description: format!("Ligne {}", i + 1),
            quantity: dec!(1),
            unit_price: dec!(100.00),
            vat_rate: dec!(8.10),
        })
        .collect();
    let (invoice, _) = invoices::create(
        pool,
        s.admin_id,
        NewInvoice {
            company_id: s.company_id,
            contact_id: s.contact_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
            payment_terms: Some("30 jours net".into()),
            lines,
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(pool, s.company_id, invoice.id, s.admin_id)
        .await
        .expect("validation");
    invoice.id
}

async fn seed_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash_password(ROLE_PASSWORD).unwrap(),
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

/// L'écriture de vente d'une facture validée.
async fn ecriture_de_la_facture(pool: &MySqlPool, invoice_id: i64) -> i64 {
    sqlx::query_scalar("SELECT journal_entry_id FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn documents(app: &TestApp, token: Option<&str>, entry_id: i64) -> reqwest::Response {
    let mut req = app
        .client
        .get(app.url(&format!("/api/v1/journal-entries/{entry_id}/documents")));
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    req.send().await.unwrap()
}

async fn documents_json(app: &TestApp, token: &str, entry_id: i64) -> Value {
    let resp = documents(app, Some(token), entry_id).await;
    assert_eq!(resp.status(), 200, "liste de l'écriture {entry_id}");
    resp.json().await.unwrap()
}

/// AC-3, AC-10 — une facture validée non figée : `notFrozen` ; après un
/// `GET /pdf`, la même écriture rend `frozen`. La liste ne fige rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_facture_validee_passe_de_not_frozen_a_frozen_apres_le_pdf(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let facture = seed_invoice(&pool, &s, 1).await;
    let ecriture = ecriture_de_la_facture(&pool, facture).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let liste = documents_json(&app, &token, ecriture).await;
    let numero: Option<String> =
        sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        liste,
        json!([{ "kind": "invoice", "id": facture, "number": numero, "pdfStatus": "notFrozen" }]),
        "facture validée non figée, sans `fileName`"
    );
    // Lire la liste deux fois ne fige pas la facture.
    documents_json(&app, &token, ecriture).await;
    let chemin: Option<String> =
        sqlx::query_scalar("SELECT pdf_storage_path FROM invoices WHERE id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(chemin, None, "la liste ne fige rien");

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{facture}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "le PDF fige la facture");
    let liste = documents_json(&app, &token, ecriture).await;
    assert_eq!(liste[0]["pdfStatus"], "frozen");

    // Un avoir annule la facture figée : l'écriture de vente garde la facture
    // (annulée figée), celle de l'avoir porte l'avoir, sans `pdfStatus`.
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: facture,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .unwrap();
    let (avoir, ecriture_avoir): (i64, i64) =
        sqlx::query_as("SELECT id, journal_entry_id FROM credit_notes WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    let liste = documents_json(&app, &token, ecriture).await;
    assert_eq!(liste[0]["id"], facture);
    assert_eq!(liste[0]["pdfStatus"], "frozen", "annulée figée");
    let liste = documents_json(&app, &token, ecriture_avoir).await;
    assert_eq!(liste.as_array().unwrap().len(), 1, "{liste}");
    assert_eq!(liste[0]["kind"], "creditNote");
    assert_eq!(liste[0]["id"], avoir);
    assert!(liste[0].get("pdfStatus").is_none() && liste[0].get("fileName").is_none());

    // Le journal dit la même chose que la liste (même constructeur).
    let fy: i64 = sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ?")
        .bind(s.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let journal: Value = app
        .client
        .get(app.url(&format!("/api/v1/reports/journals?fiscalYearId={fy}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let mut vues = 0;
    for e in journal["journals"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|j| j["entries"].as_array().unwrap())
    {
        let id = e["entryId"].as_i64().unwrap();
        let non_vide = !documents_json(&app, &token, id)
            .await
            .as_array()
            .unwrap()
            .is_empty();
        assert_eq!(e["hasDocuments"] == true, non_vide, "écriture {id}");
        vues += 1;
    }
    assert_eq!(vues, 2, "la vente et l'avoir");
}

/// AC-3 — droits : Consultation `200` ; clé d'API en lecture `200` ; écriture
/// d'une autre société et écriture inconnue `404` (corps d'`AppError`) ; sans
/// session `401`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn droits_de_la_route(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let facture = seed_invoice(&pool, &s, 1).await;
    let ecriture = ecriture_de_la_facture(&pool, facture).await;
    seed_user(&pool, "lecteur", Role::Consultation, s.company_id).await;

    // Société B et son administrateur.
    let b: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société B', 'Rue B 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    seed_user(&pool, "admin_b", Role::Admin, b).await;

    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let admin = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let lecteur = login(&app, "lecteur", ROLE_PASSWORD).await;
    let liste = documents_json(&app, &lecteur, ecriture).await;
    assert_eq!(liste[0]["id"], facture, "Consultation : {liste}");

    let (_, cle) =
        common::audit_traces::create_key_via_http(&app.client, &app.base_url, &admin, "ro", "read")
            .await;
    let liste = documents_json(&app, &cle, ecriture).await;
    assert_eq!(liste[0]["id"], facture, "clé en lecture : {liste}");

    let admin_b = login(&app, "admin_b", ROLE_PASSWORD).await;
    for (qui, token, id) in [
        ("autre société", admin_b.as_str(), ecriture),
        ("écriture inconnue", admin.as_str(), 9_999_999),
    ] {
        let resp = documents(&app, Some(token), id).await;
        assert_eq!(resp.status(), 404, "{qui}");
        let corps: Value = resp.json().await.expect("corps JSON d'AppError");
        assert_eq!(corps["error"]["code"], "NOT_FOUND", "{qui} : {corps}");
    }

    assert_eq!(documents(&app, None, ecriture).await.status(), 401);
}
