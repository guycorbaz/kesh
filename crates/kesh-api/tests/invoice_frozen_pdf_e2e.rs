//! Story 25-6-b (#387) — le PDF d'une facture est figé : une pièce émise ne
//! change plus.
//!
//! Chaque test aurait échoué avant le patch (AC 7) : deux rendus de la même
//! facture n'avaient pas les mêmes octets (date de création et `/ID` du PDF),
//! le téléchargement suivait la langue de l'installation, et une modification
//! du client changeait la pièce déjà émise.
//!
//! ⚠️ Chaque test a **son propre** `KESH_DOCUMENTS_DIR` (patron
//! `inbox_import_e2e`) : plusieurs suppriment ou altèrent un fichier figé, ce
//! qui ne doit jamais toucher le répertoire partagé d'un autre test.

mod common;

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::mail::MockMailer;
use kesh_api::middleware::rate_limit::RateLimiter;
use kesh_api::routes::issued_invoice_pdf::{self, IssuedPdf, PdfContext, PoseOutcome};
use kesh_api::{AppState, build_router};
use kesh_db::entities::Language;
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::credit_note::NewCreditNote;
use kesh_db::entities::invoice::{Invoice, NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, companies, contacts, credit_notes, invoices, users};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use serde_json::json;
use sha2::{Digest, Sha256};
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

async fn get_pdf(app: &TestApp, token: &str, id: i64) -> reqwest::Response {
    app.client
        .get(app.url(&format!("/api/v1/invoices/{id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

async fn pdf_bytes(app: &TestApp, token: &str, id: i64) -> Vec<u8> {
    let resp = get_pdf(app, token, id).await;
    assert_eq!(resp.status(), 200);
    resp.bytes().await.unwrap().to_vec()
}

async fn error_code(resp: reqwest::Response) -> String {
    let body: serde_json::Value = resp.json().await.unwrap();
    body["error"]["code"]
        .as_str()
        .unwrap_or_default()
        .to_string()
}

async fn invoice(pool: &MySqlPool, s: &Seeded, id: i64) -> Invoice {
    invoices::find_by_id(pool, s.company_id, id)
        .await
        .unwrap()
        .unwrap()
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn frozen_file(app: &TestApp, inv: &Invoice) -> PathBuf {
    app.documents.join(inv.pdf_storage_path.as_deref().unwrap())
}

async fn refreeze(app: &TestApp, token: &str, id: i64) -> reqwest::Response {
    app.client
        .post(app.url(&format!("/api/v1/invoices/{id}/pdf/refreeze")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

// ============================================================
// Le gel
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deux_telechargements_rendent_les_memes_octets(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 2).await;
    let version_avant = invoice(&pool, &s, id).await.version;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let premier = pdf_bytes(&app, &token, id).await;
    let second = pdf_bytes(&app, &token, id).await;
    assert_eq!(
        premier, second,
        "le second téléchargement rend le document figé"
    );

    let inv = invoice(&pool, &s, id).await;
    assert_eq!(inv.pdf_sha256.as_deref(), Some(sha256(&premier).as_str()));
    assert_eq!(std::fs::read(frozen_file(&app, &inv)).unwrap(), premier);
    assert_eq!(inv.version, version_avant, "le gel ne touche pas `version`");
    assert_eq!(
        common::audit_actions(&pool, "invoice", id)
            .await
            .iter()
            .filter(|a| *a == "invoice.pdf_frozen")
            .count(),
        1,
        "un seul gel, une seule trace"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_pdf_est_dans_la_langue_du_client(pool: MySqlPool) {
    let s = seed(&pool, Some(Language::De)).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    pdf_bytes(&app, &token, id).await;
    // L'installation est en français ; le client, en allemand.
    assert_eq!(
        invoice(&pool, &s, id).await.pdf_language.as_deref(),
        Some("DE")
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_email_joint_les_octets_du_telechargement_et_fige(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 2).await;
    let mailer = MockMailer::new();
    let app = spawn_app(pool.clone(), mailer.clone(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/send-email")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    // Le premier envoi a figé.
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_some());

    let piece_jointe = mailer.sent_emails()[0].attachment_bytes.clone();
    assert_eq!(pdf_bytes(&app, &token, id).await, piece_jointe);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modifier_le_client_apres_le_gel_ne_change_pas_le_pdf(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let avant = pdf_bytes(&app, &token, id).await;

    sqlx::query("UPDATE contacts SET address_city = 'St. Gallen', language = 'DE' WHERE id = ?")
        .bind(s.contact_id)
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(pdf_bytes(&app, &token, id).await, avant);
    assert_eq!(
        invoice(&pool, &s, id).await.pdf_language.as_deref(),
        Some("FR")
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_echec_smtp_laisse_la_facture_figee(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::failing(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/send-email")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 500);

    let inv = invoice(&pool, &s, id).await;
    assert!(inv.emailed_at.is_none(), "non marquée envoyée");
    let fige = inv
        .pdf_sha256
        .clone()
        .expect("le gel survit à l'échec SMTP");
    assert_eq!(sha256(&pdf_bytes(&app, &token, id).await), fige);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_facture_a_dix_lignes_ne_fige_rien(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 10).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = get_pdf(&app, &token, id).await;
    assert_eq!(resp.status(), 400);
    assert_eq!(error_code(resp).await, "INVOICE_TOO_MANY_LINES_FOR_PDF");
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_none());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_repertoire_non_inscriptible_repond_500_sans_rien_figer(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    // Un FICHIER à la place du répertoire : `create_dir_all` échoue.
    let blocker = unique_dir("documents-blocked").join("not-a-dir");
    std::fs::write(&blocker, b"x").unwrap();
    let app = spawn_app(pool.clone(), MockMailer::new(), blocker).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    assert_eq!(get_pdf(&app, &token, id).await.status(), 500);
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_none());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_premier_telechargement_d_un_consultation_fige_et_le_nomme(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let lecteur = seed_user(&pool, "lecteur", Role::Consultation, s.company_id).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "lecteur", ROLE_PASSWORD).await;

    pdf_bytes(&app, &token, id).await;
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_some());
    let (actor_type, _, user_id, _) =
        common::audit_actor(&pool, "invoice", id, "invoice.pdf_frozen").await;
    assert_eq!((actor_type.as_str(), user_id), ("user", lecteur));
}

// ============================================================
// L'intégrité
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_fichier_fige_supprime_repond_410(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    let inv = invoice(&pool, &s, id).await;
    std::fs::remove_file(frozen_file(&app, &inv)).unwrap();

    let resp = get_pdf(&app, &token, id).await;
    assert_eq!(resp.status(), 410);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_PDF_GONE");
    // Le message nomme le fichier manquant.
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains(inv.pdf_sha256.as_deref().unwrap()),
        "{body}"
    );
    // Jamais régénéré en silence.
    assert!(!frozen_file(&app, &inv).exists());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_fichier_fige_altere_repond_500_sans_rien_servir(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    let inv = invoice(&pool, &s, id).await;
    std::fs::write(frozen_file(&app, &inv), b"%PDF-1.7 altered").unwrap();

    let resp = get_pdf(&app, &token, id).await;
    assert_eq!(resp.status(), 500);
    assert!(!resp.bytes().await.unwrap().starts_with(b"%PDF"));
}

// ============================================================
// Le cycle de vie
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn devalider_detache_et_revalider_fige_un_nouveau_document(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let ancien = pdf_bytes(&app, &token, id).await;
    let inv = invoice(&pool, &s, id).await;

    invoices::unvalidate(&pool, s.company_id, id, s.admin_id, inv.version)
        .await
        .unwrap();
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_none());
    // Le fichier reste sur le disque.
    assert!(frozen_file(&app, &inv).exists());

    invoices::validate_invoice(&pool, s.company_id, id, s.admin_id)
        .await
        .unwrap();
    let nouveau = pdf_bytes(&app, &token, id).await;
    assert_ne!(sha256(&nouveau), sha256(&ancien), "un nouveau document");
    assert_eq!(
        invoice(&pool, &s, id).await.pdf_sha256.as_deref(),
        Some(sha256(&nouveau).as_str())
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_facture_annulee_garde_son_pdf_fige(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let emis = pdf_bytes(&app, &token, id).await;

    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: id,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .unwrap();
    assert_eq!(invoice(&pool, &s, id).await.status, "cancelled");

    assert_eq!(pdf_bytes(&app, &token, id).await, emis);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_facture_annulee_jamais_rendue_n_a_pas_de_pdf(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: id,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .unwrap();
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = get_pdf(&app, &token, id).await;
    assert_eq!(resp.status(), 400);
    assert_eq!(error_code(resp).await, "INVOICE_NOT_VALIDATED");
}

// ============================================================
// Le service, aux points de course — déterministe
// ============================================================

async fn ctx_parts(
    pool: &MySqlPool,
    s: &Seeded,
) -> (kesh_db::entities::Company, Arc<kesh_i18n::I18nBundle>) {
    let company = companies::find_by_id(pool, s.company_id)
        .await
        .unwrap()
        .unwrap();
    (company, load_i18n())
}

/// Un rendu concurrent a figé avant nous : c'est **son** document qui est
/// servi, jamais le nôtre (mutation tuée : garde `pdf_storage_path IS NULL`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_pose_perdante_sert_le_document_de_l_autre(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let (company, i18n) = ctx_parts(&pool, &s).await;
    let documents = unique_dir("documents");
    let ctx = PdfContext {
        pool: &pool,
        i18n: &i18n,
        documents_dir: &documents,
        company: &company,
        user_id: s.admin_id,
        actor_api_key_id: None,
    };
    let version = invoice(&pool, &s, id).await.version;

    let premier = issued_invoice_pdf::get_or_freeze(&ctx, id).await.unwrap();
    let autre = IssuedPdf {
        bytes: b"%PDF-1.7 autre rendu".to_vec(),
        filename_base: "x".into(),
    };
    match issued_invoice_pdf::pose(&ctx, id, version, autre, Language::Fr)
        .await
        .unwrap()
    {
        PoseOutcome::Adopted(pdf) => assert_eq!(pdf.bytes, premier.bytes),
        other => panic!("attendu Adopted, obtenu {other:?}"),
    }
}

/// La facture a changé pendant le rendu : la pose est refusée et annoncée
/// comme telle, puis le service fige les **nouvelles** données (mutation
/// tuée : garde `version`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_rendu_perime_est_refuse_puis_refait(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let (company, i18n) = ctx_parts(&pool, &s).await;
    let documents = unique_dir("documents");
    let ctx = PdfContext {
        pool: &pool,
        i18n: &i18n,
        documents_dir: &documents,
        company: &company,
        user_id: s.admin_id,
        actor_api_key_id: None,
    };
    let v_rendu = invoice(&pool, &s, id).await.version;
    invoices::unvalidate(&pool, s.company_id, id, s.admin_id, v_rendu)
        .await
        .unwrap();
    invoices::validate_invoice(&pool, s.company_id, id, s.admin_id)
        .await
        .unwrap();

    let rendu = IssuedPdf {
        bytes: b"%PDF-1.7 ancien rendu".to_vec(),
        filename_base: "x".into(),
    };
    let outcome = issued_invoice_pdf::pose(&ctx, id, v_rendu, rendu, Language::Fr)
        .await
        .unwrap();
    assert!(matches!(outcome, PoseOutcome::Changed), "{outcome:?}");
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_none());

    let pdf = issued_invoice_pdf::get_or_freeze(&ctx, id).await.unwrap();
    assert!(pdf.bytes.starts_with(b"%PDF-1."));
    assert_ne!(pdf.bytes, b"%PDF-1.7 ancien rendu".to_vec());
}

/// Dévalidée entre le rendu et la pose : rien n'est figé sur le brouillon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_devalidation_intercalee_ne_laisse_rien_de_fige(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let (company, i18n) = ctx_parts(&pool, &s).await;
    let documents = unique_dir("documents");
    let ctx = PdfContext {
        pool: &pool,
        i18n: &i18n,
        documents_dir: &documents,
        company: &company,
        user_id: s.admin_id,
        actor_api_key_id: None,
    };
    let v = invoice(&pool, &s, id).await.version;
    invoices::unvalidate(&pool, s.company_id, id, s.admin_id, v)
        .await
        .unwrap();

    let rendu = IssuedPdf {
        bytes: b"%PDF-1.7 rendu".to_vec(),
        filename_base: "x".into(),
    };
    let res = issued_invoice_pdf::pose(&ctx, id, v, rendu, Language::Fr).await;
    assert!(matches!(res, Err(AppError::InvoiceNotValidated)), "{res:?}");
    assert!(invoice(&pool, &s, id).await.pdf_storage_path.is_none());
}

/// Isolation : le PDF figé d'une autre société n'est jamais servi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_pdf_fige_d_une_autre_societe_n_est_jamais_servi(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let (company, i18n) = ctx_parts(&pool, &s).await;
    let documents = unique_dir("documents");
    let ctx = PdfContext {
        pool: &pool,
        i18n: &i18n,
        documents_dir: &documents,
        company: &company,
        user_id: s.admin_id,
        actor_api_key_id: None,
    };
    issued_invoice_pdf::get_or_freeze(&ctx, id).await.unwrap();

    let mut autre = company.clone();
    autre.id = company.id + 999;
    let ctx_autre = PdfContext {
        company: &autre,
        ..ctx
    };
    let res = issued_invoice_pdf::get_or_freeze(&ctx_autre, id).await;
    assert!(
        matches!(
            res,
            Err(AppError::Database(kesh_db::errors::DbError::NotFound))
        ),
        "{res:?}"
    );
}

// ============================================================
// Refiger (arbitrage 7)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_apres_un_410_produit_un_nouveau_document_trace(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    let avant = invoice(&pool, &s, id).await;
    std::fs::remove_file(frozen_file(&app, &avant)).unwrap();
    assert_eq!(get_pdf(&app, &token, id).await.status(), 410);

    let resp = refreeze(&app, &token, id).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["pdfFrozenAt"].is_string(), "{body}");

    let apres = invoice(&pool, &s, id).await;
    assert_ne!(apres.pdf_sha256, avant.pdf_sha256);
    assert_eq!(
        sha256(&pdf_bytes(&app, &token, id).await),
        apres.pdf_sha256.clone().unwrap()
    );
    let details = common::audit_details(&pool, "invoice", id, "invoice.pdf_refrozen")
        .await
        .unwrap();
    assert_eq!(details["oldPdfSha256"], avant.pdf_sha256.unwrap());
    assert_eq!(details["pdfSha256"], apres.pdf_sha256.unwrap());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_est_refuse_a_un_comptable(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    seed_user(&pool, "compta", Role::Comptable, s.company_id).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "compta", ROLE_PASSWORD).await;
    assert_eq!(refreeze(&app, &token, id).await.status(), 403);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_une_facture_non_figee_repond_409(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = refreeze(&app, &token, id).await;
    assert_eq!(resp.status(), 409);
    assert_eq!(error_code(resp).await, "INVOICE_PDF_NOT_FROZEN");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_un_fichier_present_repond_409(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    let avant = invoice(&pool, &s, id).await.pdf_sha256;

    let resp = refreeze(&app, &token, id).await;
    assert_eq!(resp.status(), 409);
    assert_eq!(error_code(resp).await, "INVOICE_PDF_PRESENT");
    assert_eq!(invoice(&pool, &s, id).await.pdf_sha256, avant);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_un_fichier_altere_repond_409_integrite(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    let inv = invoice(&pool, &s, id).await;
    std::fs::write(frozen_file(&app, &inv), b"%PDF-1.7 altered").unwrap();

    let resp = refreeze(&app, &token, id).await;
    assert_eq!(resp.status(), 409);
    assert_eq!(error_code(resp).await, "INVOICE_PDF_INTEGRITY");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refiger_une_facture_annulee_est_refuse(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: id,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .unwrap();
    let inv = invoice(&pool, &s, id).await;
    std::fs::remove_file(frozen_file(&app, &inv)).unwrap();

    let resp = refreeze(&app, &token, id).await;
    assert_eq!(resp.status(), 400);
    assert_eq!(error_code(resp).await, "INVOICE_CANCELLED");
}

/// Revue P1 (B1) : le service sert le PDF figé d'une facture annulée — pour le
/// téléchargement. L'envoi, lui, reste réservé à une facture validée : sans sa
/// garde, une facture annulée par un avoir repartait au client par l'API.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_facture_annulee_figee_ne_s_envoie_pas(pool: MySqlPool) {
    let s = seed(&pool, None).await;
    let id = seed_invoice(&pool, &s, 1).await;
    let mailer = MockMailer::new();
    let app = spawn_app(pool.clone(), mailer.clone(), unique_dir("documents")).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    pdf_bytes(&app, &token, id).await;
    credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: s.company_id,
            invoice_id: id,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        s.admin_id,
    )
    .await
    .unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/send-email")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    assert_eq!(error_code(resp).await, "INVOICE_NOT_VALIDATED");
    assert!(mailer.sent_emails().is_empty(), "rien n'est parti");
    assert!(invoice(&pool, &s, id).await.emailed_at.is_none());
}
