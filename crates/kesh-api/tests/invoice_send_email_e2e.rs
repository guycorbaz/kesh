//! Story 20-3b1 — tests E2E pour l'envoi d'une facture validée par e-mail.
//!
//! `GET /api/v1/invoices/{id}/email-preview` + `POST /api/v1/invoices/{id}/send-email`.
//! MockMailer injecté via AppState littéral (pattern `password_recovery_e2e`),
//! seed facture validée + banque primary + adresses via le flow normal
//! (pattern `invoice_pdf_e2e`). Couvre : happy path (mock capture to/subject
//! rendu/attachment + `emailed_at`/`emailed_to` DB + audit `invoice.emailed`),
//! langue contact DE (preview + PDF à l'envoi), renvoi, contact sans e-mail
//! (400), facture draft (400), IDOR (404), Consultation (403), objet vide
//! (422), SMTP down (500 + non marquée), SMTP non configuré (412), rate-limit
//! (429), preview, et `PUT /companies/current/email` (Reply-To : happy path +
//! effacement, e-mail invalide 400, RBAC Admin-only 403).

use std::net::SocketAddr;
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
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, contacts, invoices, users};
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

/// Options du spawn : mailer injecté, gate SMTP, seuil rate-limit send-email
/// (pattern seuils injectables de `password_recovery_e2e::spawn_app`).
async fn spawn_app(
    pool: MySqlPool,
    mailer: MockMailer,
    smtp_ready: bool,
    send_email_max: u32,
) -> TestApp {
    let config = test_config();
    let rate_limiter = RateLimiter::new(&config);
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
        rate_limiter_recovery: Arc::new(kesh_api::build_recovery_rate_limiter()),
        rate_limiter_send_email: Arc::new(RateLimiter::with_thresholds(
            send_email_max,
            Duration::from_secs(15 * 60),
            Duration::from_secs(15 * 60),
        )),
        i18n,
        users_exist: Arc::new(AtomicBool::new(true)),
        mailer: Arc::new(mailer),
        smtp_ready,
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

/// Contact PDF-ready (adresse structurée complète) avec e-mail, langue et
/// civilité paramétrables.
#[allow(clippy::too_many_arguments)]
async fn seed_contact(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    email: Option<&str>,
    language: Option<Language>,
    salutation: Salutation,
    last_name: Option<&str>,
) -> i64 {
    let contact = contacts::create(
        pool,
        user_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: Some("Pia".into()),
            last_name: last_name.map(String::from),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: email.map(String::from),
            phone: None,
            ide_number: None,
            default_payment_terms: None,
            language,
            salutation,
        },
    )
    .await
    .unwrap();
    contact.id
}

async fn seed_primary_bank(pool: &MySqlPool, company_id: i64) {
    bank_accounts::upsert_primary(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
}

async fn seed_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    user_id: i64,
) -> i64 {
    let new = NewInvoice {
        company_id,
        contact_id,
        date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
        payment_terms: Some("30 jours net".into()),
        lines: vec![NewInvoiceLine {
            description: "Prestation".into(),
            quantity: dec!(1),
            unit_price: dec!(100.00),
            vat_rate: dec!(8.10),
        }],
        project_id: None,
    };
    let (invoice, _lines) = invoices::create(pool, user_id, new).await.unwrap();
    invoices::validate_invoice(pool, company_id, invoice.id, user_id)
        .await
        .expect("validate_invoice");
    invoice.id
}

/// Seed complet PDF-ready + envoi-ready : contact avec e-mail, banque
/// primary, facture validée. Retourne (admin_id, company_id, invoice_id).
async fn seed_sendable(pool: &MySqlPool) -> (i64, i64, i64) {
    let (admin_id, company_id) = seed_base(pool).await;
    let contact_id = seed_contact(
        pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    seed_primary_bank(pool, company_id).await;
    let invoice_id = seed_validated_invoice(pool, company_id, contact_id, admin_id).await;
    (admin_id, company_id, invoice_id)
}

async fn fetch_emailed(pool: &MySqlPool, invoice_id: i64) -> (Option<String>, Option<String>) {
    let row: (Option<chrono::NaiveDateTime>, Option<String>) =
        sqlx::query_as("SELECT emailed_at, emailed_to FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
    (row.0.map(|d| d.to_string()), row.1)
}

async fn count_audit_emailed(pool: &MySqlPool, invoice_id: i64) -> i64 {
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.emailed' \
         AND entity_type = 'invoice' AND entity_id = ?",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .unwrap();
    n
}

// --- Tests --------------------------------------------------------------

/// Happy path : preview pré-remplie (défaut FR rendu) puis envoi → 200,
/// mock capture to/subject/body/attachment/display-name, DB marquée, audit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_email_happy_path(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Preview : to = e-mail contact, subject/body rendus (template défaut FR).
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/email-preview")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let preview: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(preview["to"], "pia@example.ch");
    assert_eq!(preview["language"], "FR");
    let subject = preview["subject"].as_str().unwrap().to_string();
    let body = preview["body"].as_str().unwrap().to_string();
    assert!(
        !subject.contains('{') && !body.contains('{'),
        "toutes les variables doivent être substituées dans les défauts : {subject} / {body}"
    );
    assert!(
        body.contains("Chère Madame Rutschmann"),
        "salutation genrée FR attendue, corps : {body}"
    );

    // Envoi avec le texte de la preview (léger édit du corps).
    let edited_body = format!("{body}\nPS: merci !");
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": subject, "body": edited_body }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let inv: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(inv["emailedTo"], "pia@example.ch");
    assert!(inv["emailedAt"].is_string(), "emailedAt posé");

    // Mock : message complet capturé.
    let sent = mock.sent_emails();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "pia@example.ch");
    assert_eq!(sent[0].subject, subject);
    assert!(sent[0].body.ends_with("PS: merci !"), "corps édité envoyé");
    assert_eq!(
        sent[0].from_display_name.as_deref(),
        Some("CI Test Company"),
        "display-name = nom société"
    );
    let filename = sent[0].attachment_filename.as_deref().unwrap();
    assert!(
        filename.starts_with("facture-") && filename.ends_with(".pdf"),
        "attachment nommé facture-*.pdf : {filename}"
    );
    assert_eq!(
        sent[0].attachment_content_type.as_deref(),
        Some("application/pdf")
    );
    assert!(sent[0].attachment_size > 1000, "PDF non vide");
    // companies.email non renseigné dans le seed → Reply-To omis.
    assert_eq!(sent[0].reply_to, None);

    // DB : marquée + audit.
    let (emailed_at, emailed_to) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_some());
    assert_eq!(emailed_to.as_deref(), Some("pia@example.ch"));
    assert_eq!(count_audit_emailed(&pool, invoice_id).await, 1);
}

/// Langue contact DE → preview rendue avec le template défaut DE
/// (décision #11 : le corps suit la langue du contact, pas l'instance FR).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn preview_uses_contact_language(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("de@example.ch"),
        Some(Language::De),
        Salutation::Monsieur,
        Some("Muster"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/email-preview")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let preview: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(preview["language"], "DE");
    assert!(
        preview["body"]
            .as_str()
            .unwrap()
            .contains("Sehr geehrter Herr Muster"),
        "salutation DE genrée attendue : {}",
        preview["body"]
    );
}

/// Langue contact DE à l'ENVOI : le PDF joint est rendu via la locale du
/// contact (`Locale::from("DE")`, décision #11) — 200 + attachment non vide
/// prouvent que le chemin de rendu DE aboutit (le texte du PDF n'est pas
/// extractible ici, streams compressés — même limite qu'invoice_pdf_e2e).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_uses_contact_language_for_pdf(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("de@example.ch"),
        Some(Language::De),
        Salutation::Monsieur,
        Some("Muster"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "Rechnung", "body": "Guten Tag" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let sent = mock.sent_emails();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "de@example.ch");
    assert!(
        sent[0].attachment_size > 1000,
        "PDF locale DE rendu et joint (taille {})",
        sent[0].attachment_size
    );
}

/// Renvoi : 2e POST → 200, emailed_at écrasé, 2 entrées d'audit.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn resend_overwrites_and_audits_each_send(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    for _ in 0..2 {
        let resp = app
            .client
            .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
            .bearer_auth(&token)
            .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
    }

    assert_eq!(mock.sent_emails().len(), 2);
    assert_eq!(count_audit_emailed(&pool, invoice_id).await, 2);
    let (emailed_at, _) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_some());
}

/// Contact sans e-mail → 400 CONTACT_EMAIL_MISSING (destinataire verrouillé),
/// et la preview renvoie to=null.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_without_email_returns_400(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        None,
        None,
        Salutation::Neutre,
        None,
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    let app = spawn_app(pool.clone(), MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/email-preview")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let preview: serde_json::Value = resp.json().await.unwrap();
    assert!(preview["to"].is_null(), "preview to=null sans e-mail");

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CONTACT_EMAIL_MISSING");
    let (emailed_at, _) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_none(), "non marquée");
}

/// Facture draft → 400 INVOICE_NOT_VALIDATED (erreur héritée du service PDF).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn draft_invoice_returns_400_not_validated(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Neutre,
        None,
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    // Facture créée mais PAS validée.
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

    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{}/send-email", invoice.id)))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_VALIDATED");
}

/// IDOR : facture d'une autre company → 404 (scoping find_by_id_with_lines).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn idor_other_company_invoice_returns_404(pool: MySqlPool) {
    let (_, _, invoice_id) = seed_sendable(&pool).await;

    // 2e company + admin2 (pattern IDOR des e2e existants).
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Other Co', 'X 1\n1000 L', 'Pme', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let phc = hash_password("password-admin2-e2e").expect("hash");
    users::create(
        &pool,
        NewUser {
            username: "admin2".into(),
            password_hash: phc,
            role: Role::Admin,
            active: true,
            company_id: other_company,
            email: None,
        },
    )
    .await
    .unwrap();

    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "admin2", "password-admin2-e2e").await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "facture d'une autre company invisible");
}

/// RBAC : Consultation → 403 (route dans comptable_routes).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn consultation_role_returns_403(pool: MySqlPool) {
    let (_, company_id, invoice_id) = {
        let (a, c, i) = seed_sendable(&pool).await;
        (a, c, i)
    };
    let phc = hash_password("password-consult-e2e").expect("hash");
    users::create(
        &pool,
        NewUser {
            username: "consult".into(),
            password_hash: phc,
            role: Role::Consultation,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();

    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "consult", "password-consult-e2e").await;
    for (method, path) in [
        (
            "GET",
            format!("/api/v1/invoices/{invoice_id}/email-preview"),
        ),
        ("POST", format!("/api/v1/invoices/{invoice_id}/send-email")),
    ] {
        let req = match method {
            "GET" => app.client.get(app.url(&path)),
            _ => app
                .client
                .post(app.url(&path))
                .json(&json!({ "subject": "s", "body": "b" })),
        };
        let resp = req.bearer_auth(&token).send().await.unwrap();
        assert_eq!(resp.status(), 403, "{method} {path} en Consultation");
    }
}

/// Objet vide (après trim) → 422 INVOICE_EMAIL_EMPTY_CONTENT.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn empty_subject_returns_422(pool: MySqlPool) {
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "   ", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_EMAIL_EMPTY_CONTENT");
}

/// SMTP down (MockMailer::failing) → 500 SMTP_SEND_FAILED et facture NON
/// marquée (décision #16 : pas de marquage sur échec).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn smtp_failure_returns_500_and_does_not_mark(pool: MySqlPool) {
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool.clone(), MockMailer::failing(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 500);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "SMTP_SEND_FAILED");
    let (emailed_at, emailed_to) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_none() && emailed_to.is_none(), "non marquée");
    assert_eq!(count_audit_emailed(&pool, invoice_id).await, 0);
}

/// SMTP non configuré (smtp_ready=false) → 412 SMTP_NOT_CONFIGURED, aucune
/// capture mock (garde AVANT l'envoi — anti « envoi fantôme » NoopMailer).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn smtp_not_configured_returns_412(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool.clone(), mock.clone(), false, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 412);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "SMTP_NOT_CONFIGURED");
    assert!(mock.sent_emails().is_empty(), "rien n'est parti");
    let (emailed_at, _) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_none(), "non marquée");
}

/// Rate-limit (company, user) : seuil 2 → 3e envoi 429 avec Retry-After.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_email_rate_limited_returns_429(pool: MySqlPool) {
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool, MockMailer::new(), true, 2).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    for i in 1..=2 {
        let resp = app
            .client
            .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
            .bearer_auth(&token)
            .json(&json!({ "subject": "s", "body": "b" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "envoi {i}/2");
    }
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429, "3e envoi bloqué");
    assert!(
        resp.headers().get("retry-after").is_some(),
        "header Retry-After présent"
    );
}

/// 401 sans token (middleware auth en amont du rôle).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_email_requires_auth_returns_401(pool: MySqlPool) {
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .json(&json!({ "subject": "s", "body": "b" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

/// `PUT /companies/current/email` (Admin) : pose l'e-mail société → le
/// Reply-To des envois suivants l'utilise (décision #2 epic-20) ; effacement
/// (`email: null`) → Reply-To de nouveau omis.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn company_email_endpoint_sets_reply_to(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, _, invoice_id) = seed_sendable(&pool).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Version courante via GET /companies/current (verrou optimiste).
    let current: serde_json::Value = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let version = current["company"]["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .bearer_auth(&token)
        .json(&json!({ "email": "info@ci-test.ch", "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let updated: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(updated["email"], "info@ci-test.ch");
    let new_version = updated["version"].as_i64().unwrap();
    assert!(new_version > version, "verrou optimiste incrémenté");

    // L'envoi reprend l'e-mail société en Reply-To.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let sent = mock.sent_emails();
    assert_eq!(
        sent.last().unwrap().reply_to.as_deref(),
        Some("info@ci-test.ch")
    );

    // Effacement : email null → Reply-To omis au prochain envoi.
    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .bearer_auth(&token)
        .json(&json!({ "email": null, "version": new_version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let cleared: serde_json::Value = resp.json().await.unwrap();
    assert!(cleared["email"].is_null());
}

/// E-mail société invalide → 400 VALIDATION_ERROR (aucune écriture).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn company_email_invalid_returns_400(pool: MySqlPool) {
    let (_, _, _) = seed_sendable(&pool).await;
    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .bearer_auth(&token)
        .json(&json!({ "email": "pas-un-email", "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// `PUT /companies/current/email` est Admin-only : Comptable → 403.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn company_email_requires_admin(pool: MySqlPool) {
    let (_, company_id, _) = seed_sendable(&pool).await;
    let phc = hash_password("password-compta-e2e").expect("hash");
    users::create(
        &pool,
        NewUser {
            username: "compta".into(),
            password_hash: phc,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let app = spawn_app(pool, MockMailer::new(), true, 20).await;
    let token = login(&app, "compta", "password-compta-e2e").await;
    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .bearer_auth(&token)
        .json(&json!({ "email": "info@ci-test.ch", "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}
