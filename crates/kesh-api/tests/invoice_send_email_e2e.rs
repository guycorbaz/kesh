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
//! (429), preview, `PUT /companies/current/email` (Reply-To : happy path +
//! effacement, e-mail invalide 400, RBAC Admin-only 403), contact archivé
//! (400 preview+send), et sémantique `mark_emailed` mid-flight (cancelled →
//! marque quand même, supprimée → NotFound ; review Pass 1 ECH-1/ECH-2).
//!
//! Story 21-5b — envoi de **rappels** (preview / unitaire / lot). Couvre AC 15-16 :
//! preview par niveau (level requis 400, inexistant 422), happy path unitaire
//! (PDF joint + `invoice_reminders` + audit), gardes pré-SMTP (payée 422,
//! suspendue 422, contact sans e-mail 400, contenu vide 422 — chacune vérifiant
//! qu'AUCUN e-mail n'est parti, garantie C1), D18 (saut de niveau 409 / ré-émission
//! autorisée), SMTP down (500 + rien enregistré), SMTP non configuré (412),
//! anti-IDOR cross-tenant (404 unitaire / `INVOICE_NOT_FOUND` en lot), RBAC
//! Consultation (403 sur les 3 routes), pré-check de capacité du lot (429 +
//! `Retry-After` réel) et succès partiel (1 accepted / 2 failed + cap dur 20).

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
use kesh_db::entities::email_template::EmailTemplateType;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{
    bank_accounts, company_dunning_settings, contacts, email_templates, invoices, users,
};
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
            default_payment_terms_days: None,
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
            revenue_account_id: None,
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

/// Dernière entrée d'audit `invoice.emailed` → `details_json` parsé
/// (review Pass 1 AA-1 : l'AC #19 exige to+subject DANS l'audit, pas
/// seulement le comptage).
async fn last_audit_emailed_details(pool: &MySqlPool, invoice_id: i64) -> serde_json::Value {
    let (details,): (Option<serde_json::Value>,) = sqlx::query_as(
        "SELECT details_json FROM audit_log WHERE action = 'invoice.emailed' \
         AND entity_type = 'invoice' AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .unwrap();
    details.expect("details_json présent")
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

    // DB : marquée + audit (avec to+subject dans details — AC #19).
    let (emailed_at, emailed_to) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_some());
    assert_eq!(emailed_to.as_deref(), Some("pia@example.ch"));
    assert_eq!(count_audit_emailed(&pool, invoice_id).await, 1);
    let details = last_audit_emailed_details(&pool, invoice_id).await;
    assert_eq!(details["to"], "pia@example.ch", "audit details.to");
    assert_eq!(details["subject"], subject, "audit details.subject");
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

/// Contact archivé (`active = false`) → 400 CONTACT_ARCHIVED sur preview ET
/// send (review Pass 1 ECH-2 : le carnet d'adresses le considère « à ne plus
/// utiliser » ; son e-mail ne doit plus recevoir de factures).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn archived_contact_returns_400(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, company_id, invoice_id) = seed_sendable(&pool).await;
    sqlx::query("UPDATE contacts SET active = FALSE WHERE company_id = ?")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/email-preview")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "preview refusée");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CONTACT_ARCHIVED");

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "Facture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "envoi refusé");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CONTACT_ARCHIVED");
    assert!(mock.sent_emails().is_empty(), "aucun e-mail parti");
    let (emailed_at, _) = fetch_emailed(&pool, invoice_id).await;
    assert!(emailed_at.is_none(), "facture non marquée");
}

/// `mark_emailed` marque même une facture passée `cancelled` mid-flight
/// (review Pass 1 ECH-1 : l'e-mail est PARTI — un avoir émis entre l'envoi
/// SMTP et le marquage ne doit pas faire perdre la trace emailed_at/audit).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_emailed_survives_cancelled_status(pool: MySqlPool) {
    let (admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();

    let updated = invoices::mark_emailed(
        &pool,
        company_id,
        invoice_id,
        "pia@example.ch",
        "Facture",
        admin_id,
        None,
    )
    .await
    .expect("marquage malgré status cancelled");
    assert!(updated.emailed_at.is_some());
    assert_eq!(count_audit_emailed(&pool, invoice_id).await, 1);
}

/// `mark_emailed` sur facture supprimée → `DbError::NotFound` (le handler
/// mappe en 409 EMAIL_SENT_INVOICE_GONE + audit best-effort — la fenêtre
/// exacte n'est pas simulable en E2E, le mapping est vérifié au niveau repo).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn mark_emailed_deleted_invoice_returns_not_found(pool: MySqlPool) {
    let (admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    sqlx::query("DELETE FROM invoice_lines WHERE invoice_id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();

    let r = invoices::mark_emailed(
        &pool,
        company_id,
        invoice_id,
        "pia@example.ch",
        "Facture",
        admin_id,
        None,
    )
    .await;
    assert!(
        matches!(r, Err(kesh_db::errors::DbError::NotFound)),
        "facture disparue → NotFound : {r:?}"
    );
}

/// Story 20-4 — `GET /_test/sent-emails` : capture disponible quand le boot
/// est en mode capture (test-mode + MockMailer partagé), 409 sinon.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_sent_emails_endpoint_captures_and_guards(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_, _, invoice_id) = seed_sendable(&pool).await;

    // Spawn dédié : config test-mode (routes /_test montées) + poignée Some.
    let config = test_config().with_test_mode(true);
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
        pool: pool.clone(),
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        rate_limiter_recovery: Arc::new(kesh_api::build_recovery_rate_limiter()),
        rate_limiter_send_email: Arc::new(kesh_api::build_send_email_rate_limiter()),
        i18n,
        users_exist: Arc::new(AtomicBool::new(true)),
        mailer: Arc::new(mock.clone()),
        smtp_ready: true,
        test_mock_mailer: Some(mock.clone()),
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
    let app = TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    };
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Vide au départ (endpoint non authentifié — monté hors require_auth).
    let resp = app
        .client
        .get(app.url("/api/v1/_test/sent-emails"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["emails"].as_array().unwrap().len(), 0);

    // Un envoi → capturé avec le mapping camelCase complet.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/send-email")))
        .bearer_auth(&token)
        .json(&json!({ "subject": "Facture capture", "body": "Bonjour" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = app
        .client
        .get(app.url("/api/v1/_test/sent-emails"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let emails = body["emails"].as_array().unwrap();
    assert_eq!(emails.len(), 1);
    assert_eq!(emails[0]["to"], "pia@example.ch");
    assert_eq!(emails[0]["subject"], "Facture capture");
    assert_eq!(emails[0]["attachmentContentType"], "application/pdf");
    assert!(
        emails[0]["attachmentFilename"]
            .as_str()
            .unwrap()
            .starts_with("facture-"),
        "filename: {}",
        emails[0]["attachmentFilename"]
    );
    assert!(emails[0]["attachmentSize"].as_u64().unwrap() > 1000);

    // Purge au seed : POST /_test/seed vide le buffer.
    let resp = app
        .client
        .post(app.url("/api/v1/_test/seed"))
        .json(&json!({ "preset": "with-company" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "seed");
    let body: serde_json::Value = app
        .client
        .get(app.url("/api/v1/_test/sent-emails"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["emails"].as_array().unwrap().len(), 0, "buffer purgé");
}

/// Story 20-4 — test-mode SANS poignée de capture (boot sans SMTP factice)
/// → 400 VALIDATION_ERROR explicite (déviation documentée vs spec [409]),
/// pas un 200 vide ambigu.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_sent_emails_endpoint_400_without_capture(pool: MySqlPool) {
    let config = test_config().with_test_mode(true);
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
        rate_limiter_send_email: Arc::new(kesh_api::build_send_email_rate_limiter()),
        i18n,
        users_exist: Arc::new(AtomicBool::new(true)),
        mailer: Arc::new(kesh_api::mail::NoopMailer),
        smtp_ready: false,
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
    let resp = reqwest::Client::new()
        .get(format!("http://{addr}/api/v1/_test/sent-emails"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "AppError::Validation → 400 explicite");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("capture d'e-mails indisponible")
    );
}

// --- Story 21-5b : envoi de rappels par e-mail -------------------------------

/// Seed la config dunning (3 niveaux 0/20/40, grâce 5) via le seed lazy repo.
async fn seed_dunning(pool: &MySqlPool, company_id: i64) {
    let mut tx = pool.begin().await.unwrap();
    company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

async fn reminder_count(pool: &MySqlPool, invoice_id: i64) -> i64 {
    let (n,): (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM invoice_reminders WHERE invoice_id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
    n
}

/// Preview d'un rappel niveau 2 : rendu serveur, destinataire verrouillé, frais mentionnés.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reminder_preview_renders_level(pool: MySqlPool) {
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), MockMailer::new(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/invoices/{invoice_id}/reminder-preview?level=2"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["level"], 2);
    assert_eq!(body["to"], "pia@example.ch");
    assert!(!body["subject"].as_str().unwrap().is_empty());
    // Le corps du niveau 2 mentionne les frais de rappel (20.00).
    assert!(
        body["body"].as_str().unwrap().contains("20.00"),
        "corps niveau 2 mentionne reminderFee : {}",
        body["body"]
    );

    // level absent → 400 ; level inexistant → 422.
    let no_level = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/reminder-preview")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(no_level.status(), 400, "level requis");
    let bad_level = app
        .client
        .get(app.url(&format!(
            "/api/v1/invoices/{invoice_id}/reminder-preview?level=99"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(bad_level.status(), 422);
}

/// Envoi unitaire happy path : e-mail capturé (PDF joint, destinataire verrouillé),
/// ligne invoice_reminders channel='email', audit invoice.reminder_sent.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_happy_path(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "Rappel de paiement", "body": "Corps du rappel." }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["levelNumber"], 1);
    assert_eq!(body["channel"], "email");

    // E-mail capturé : destinataire verrouillé + PDF joint.
    let sent = mock.sent_emails();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "pia@example.ch");
    assert!(
        sent[0]
            .attachment_filename
            .as_deref()
            .unwrap()
            .ends_with(".pdf")
    );

    // Ligne invoice_reminders + audit.
    assert_eq!(reminder_count(&pool, invoice_id).await, 1);
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.reminder_sent' AND entity_id = ?",
    )
    .bind(invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 1);
}

/// Gardes unitaire : facture suspendue → 422 DUNNING_PAUSED, AUCUN e-mail parti (C1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_paused_rejected_before_smtp(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    sqlx::query("UPDATE invoices SET dunning_paused_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "DUNNING_PAUSED"
    );
    assert_eq!(
        mock.sent_emails().len(),
        0,
        "aucun e-mail parti (garde avant SMTP)"
    );
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// Garde unitaire : facture payée → 422 INVOICE_ALREADY_PAID, rien n'est parti (C1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_paid_invoice_rejected_before_smtp(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    sqlx::query("UPDATE invoices SET paid_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "INVOICE_ALREADY_PAID"
    );
    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail parti");
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// Garde unitaire : contact sans e-mail → CONTACT_EMAIL_MISSING, rien n'est parti.
///
/// **400** et non 422 : la variante `AppError::ContactEmailMissing` est partagée
/// avec l'envoi de facture Epic 20 (`errors.rs`, cf. `contact_without_email_returns_400`).
/// AC 15 annonce « 422 » — imprécision de la spec, le code fait foi (cf. Change Log).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_contact_without_email_rejected(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        None, // pas d'e-mail
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    seed_dunning(&pool, company_id).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "CONTACT_EMAIL_MISSING"
    );
    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail parti");
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// Garde unitaire : subject/body vides (après trim) → 422, rien n'est parti.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_empty_content_returns_422(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "   ", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 422);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "INVOICE_EMAIL_EMPTY_CONTENT"
    );
    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail parti");
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// D18 : saut de niveau (> prochain) → 409 LEVEL_ALREADY_SENT avant SMTP ;
/// ré-envoi d'un niveau ≤ prochain autorisé (ré-émission volontaire).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_level_skip_rejected_but_resend_allowed(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Aucun rappel encore envoyé → prochain = 1. Viser 3 est un saut.
    let skip = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 3, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(skip.status(), 409, "saut de niveau refusé");
    assert_eq!(
        skip.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "LEVEL_ALREADY_SENT"
    );
    assert_eq!(mock.sent_emails().len(), 0, "saut → rien n'est parti");

    // Niveau 1 → OK, puis re-niveau 1 (≤ prochain) → autorisé (D18).
    for attempt in 1..=2 {
        let resp = app
            .client
            .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
            .bearer_auth(&token)
            .json(&json!({ "levelNumber": 1, "subject": "Rappel", "body": "Corps." }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 201, "envoi niveau 1, tentative {attempt}");
    }
    assert_eq!(
        mock.sent_emails().len(),
        2,
        "ré-émission niveau 1 autorisée"
    );
    assert_eq!(reminder_count(&pool, invoice_id).await, 2);
}

/// SMTP down → 500 et **rien n'est enregistré** : c'est la garantie que le patch
/// CRITICAL C1 devait apporter (ordre « SMTP puis enregistrer »).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_smtp_down_returns_500_and_records_nothing(pool: MySqlPool) {
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), MockMailer::failing(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 500);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "SMTP_SEND_FAILED"
    );
    assert_eq!(
        reminder_count(&pool, invoice_id).await,
        0,
        "e-mail pas parti → aucune trace de rappel"
    );
    let (n,): (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.reminder_sent' AND entity_id = ?",
    )
    .bind(invoice_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(n, 0, "aucun audit d'envoi");
}

/// SMTP non configuré → 412, garde avant tout envoi.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_smtp_not_configured_returns_412(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), false, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 412);
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "SMTP_NOT_CONFIGURED"
    );
    assert_eq!(mock.sent_emails().len(), 0, "rien n'est parti");
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);

    // Le lot applique la même garde globale.
    let batch = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [invoice_id] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 412, "lot : SMTP non configuré → 412 global");
    assert_eq!(mock.sent_emails().len(), 0);
}

/// Anti-IDOR : facture d'une autre company → 404 (unitaire), `INVOICE_NOT_FOUND`
/// per-facture (lot) — même code que « absente », pas de fuite d'existence.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_cross_tenant_is_invisible(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;

    // 2e company + son admin (pattern IDOR des e2e existants).
    let other_company = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Other Co', 'X 1\n1000 L', 'Pme', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    seed_dunning(&pool, other_company).await;
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

    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin2", "password-admin2-e2e").await;

    // Unitaire → 404.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "facture d'une autre company invisible");

    // Lot → failed[] INVOICE_NOT_FOUND, HTTP 200 (succès partiel = succès HTTP).
    let batch = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [invoice_id] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 200);
    let body: serde_json::Value = batch.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 0);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["invoiceId"], invoice_id);
    assert_eq!(
        failed[0]["errorCode"], "INVOICE_NOT_FOUND",
        "cross-tenant indiscernable d'une facture absente"
    );
    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail cross-tenant");
}

/// RBAC : Consultation → 403 sur les 3 routes rappels (comptable_routes).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_consultation_role_returns_403(pool: MySqlPool) {
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
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

    let app = spawn_app(pool.clone(), MockMailer::new(), true, 20).await;
    let token = login(&app, "consult", "password-consult-e2e").await;

    let preview = app
        .client
        .get(app.url(&format!(
            "/api/v1/invoices/{invoice_id}/reminder-preview?level=1"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(preview.status(), 403, "preview Comptable+");

    let unit = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(unit.status(), 403, "envoi unitaire Comptable+");

    let batch = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [invoice_id] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 403, "envoi lot Comptable+");
}

/// Contenu trop long → 400 AVANT le SMTP, rien n'est parti (review Pass 3).
///
/// Sans cette garde, `invoice_reminders.subject`/`body` (colonnes TEXT) rejetaient
/// l'INSERT en MariaDB 1406 **après** l'envoi : l'e-mail partait chez le débiteur,
/// aucune trace n'était écrite, et chaque re-essai le renvoyait à l'identique.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_oversized_content_rejected_before_smtp(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Corps > 10 000 caractères.
    let huge_body = "a".repeat(10_001);
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "Rappel", "body": huge_body }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "corps trop long refusé");

    // Objet > 500 caractères.
    let huge_subject = "s".repeat(501);
    let subj = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": huge_subject, "body": "Corps." }))
        .send()
        .await
        .unwrap();
    assert_eq!(subj.status(), 400, "objet trop long refusé");

    assert_eq!(
        mock.sent_emails().len(),
        0,
        "garde AVANT le SMTP — aucun e-mail parti"
    );
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);

    // La borne haute reste acceptée (pas de régression sur un contenu légitime).
    let ok = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "Rappel", "body": "b".repeat(10_000) }))
        .send()
        .await
        .unwrap();
    assert_eq!(ok.status(), 201, "10 000 caractères pile → accepté");
    assert_eq!(reminder_count(&pool, invoice_id).await, 1);
}

/// Lot : panne SMTP → `SMTP_SEND_FAILED` per-facture dans un HTTP 200, sans tuer
/// le lot ni escalader (review Pass 4 — ce chemin, ajouté en Pass 3 avec son log
/// `error!`, n'avait aucun test).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_batch_smtp_failure_is_per_invoice(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    seed_dunning(&pool, company_id).await;
    let a = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let b = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    let app = spawn_app(pool.clone(), MockMailer::failing(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [a, b] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "panne SMTP → per-facture, pas d'escalade"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 0);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2, "les 2 factures échouent individuellement");
    for f in failed {
        assert_eq!(f["errorCode"], "SMTP_SEND_FAILED");
    }
    // Aucun e-mail parti → aucune trace, cohérent avec l'unitaire.
    assert_eq!(reminder_count(&pool, a).await, 0);
    assert_eq!(reminder_count(&pool, b).await, 0);
}

/// Lot : template surdimensionné → `REMINDER_CONTENT_TOO_LONG` per-facture AVANT le
/// SMTP (review Pass 4 — chemin ajouté en Pass 3 sans test). Atteignable car
/// `email_templates` ne borne pas la longueur à l'enregistrement.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_batch_oversized_template_rejected_before_smtp(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;

    // Override de template niveau 1 avec un corps > REMINDER_BODY_MAX (10 000).
    email_templates::upsert_override(
        &pool,
        company_id,
        EmailTemplateType::InvoiceReminder,
        Language::Fr,
        1,
        None,
        admin_id,
        None,
        "Rappel".to_string(),
        "x".repeat(10_001),
    )
    .await
    .expect("upsert override");

    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [invoice_id] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 0);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["errorCode"], "REMINDER_CONTENT_TOO_LONG");
    assert_eq!(
        failed[0]["details"]["bodyMax"], 10_000,
        "details porte la borne dépassée"
    );
    assert_eq!(
        mock.sent_emails().len(),
        0,
        "garde AVANT le SMTP — rien n'est parti"
    );
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// Lot plus grand que le quota d'envoi → 422 (et non un 429 perpétuel) : aucune
/// attente ne peut le rendre acceptable (review Pass 3).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_batch_over_send_quota_returns_422(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    seed_dunning(&pool, company_id).await;
    let a = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let b = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let c = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    // Quota d'envoi = 2, lot de 3 → impossible par construction, fenêtre vierge.
    let app = spawn_app(pool.clone(), mock.clone(), true, 2).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [a, b, c] }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        422,
        "lot > quota → 422, pas un 429 perpétuel"
    );
    assert_eq!(
        resp.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "BATCH_EXCEEDS_SEND_QUOTA"
    );
    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail parti");
}

/// Contact archivé APRÈS création de la facture (la création, elle, refuse un
/// contact archivé — cf. `invoices.rs`) : le rappel est refusé sur les 2 surfaces.
///
/// Le lot renvoie `CONTACT_ARCHIVED` et **non** `CONTACT_EMAIL_MISSING` (review
/// Pass 2) : le contact a bien un e-mail, il est seulement archivé — annoncer
/// « e-mail manquant » enverrait l'utilisateur corriger le mauvais problème.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_archived_contact_reports_archived_not_missing_email(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (_admin_id, company_id, invoice_id) = seed_sendable(&pool).await;
    seed_dunning(&pool, company_id).await;
    // Le contact a un e-mail (seed_sendable) — on l'archive seulement.
    sqlx::query("UPDATE contacts SET active = FALSE WHERE company_id = ?")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    // Unitaire → 400 CONTACT_ARCHIVED (variante partagée Epic 20).
    let unit = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "x", "body": "y" }))
        .send()
        .await
        .unwrap();
    assert_eq!(unit.status(), 400);
    assert_eq!(
        unit.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "CONTACT_ARCHIVED"
    );

    // Lot → failed[] CONTACT_ARCHIVED, HTTP 200.
    let batch = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [invoice_id] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 200);
    let body: serde_json::Value = batch.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 0);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(
        failed[0]["errorCode"], "CONTACT_ARCHIVED",
        "le contact a un e-mail — le code doit dire ARCHIVED, pas EMAIL_MISSING"
    );

    assert_eq!(mock.sent_emails().len(), 0, "aucun e-mail parti");
    assert_eq!(reminder_count(&pool, invoice_id).await, 0);
}

/// Pré-check de capacité du lot : lot > slots restants → 429 global AVANT le
/// 1er SMTP (aucun e-mail parti — pas de blocage à mi-course). Vérifie aussi que
/// `Retry-After` reflète la fenêtre réelle et non un `1` codé en dur (review Pass 1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_batch_capacity_precheck_returns_429(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    seed_dunning(&pool, company_id).await;
    let inv_a = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let inv_b = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;

    // Seuil 2 : un envoi unitaire consomme 1 slot → il en reste 1, insuffisant pour un lot de 2.
    let app = spawn_app(pool.clone(), mock.clone(), true, 2).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let warmup = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{inv_a}/reminders/send")))
        .bearer_auth(&token)
        .json(&json!({ "levelNumber": 1, "subject": "Rappel", "body": "Corps." }))
        .send()
        .await
        .unwrap();
    assert_eq!(warmup.status(), 201, "1er envoi consomme un slot");
    assert_eq!(mock.sent_emails().len(), 1);

    let batch = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [inv_a, inv_b] }))
        .send()
        .await
        .unwrap();
    assert_eq!(batch.status(), 429, "lot de 2 > 1 slot restant");
    let retry_after: u64 = batch
        .headers()
        .get("retry-after")
        .expect("header Retry-After présent")
        .to_str()
        .unwrap()
        .parse()
        .expect("Retry-After entier");
    assert!(
        retry_after > 1,
        "Retry-After doit refléter la fenêtre réelle (~900 s), pas un 1 codé en dur — obtenu {retry_after}"
    );
    assert_eq!(
        mock.sent_emails().len(),
        1,
        "pré-check global → aucun e-mail supplémentaire (pas de blocage mi-course)"
    );
}

/// Envoi par lot : succès partiel { accepted, failed } + cap dur.
///
/// AC 16 : 1 OK + 1 payée + 1 sans e-mail → 1 accepted, 2 failed avec les bons codes.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn send_reminder_batch_partial_success(pool: MySqlPool) {
    let mock = MockMailer::new();
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(
        &pool,
        company_id,
        admin_id,
        Some("pia@example.ch"),
        None,
        Salutation::Madame,
        Some("Rutschmann"),
    )
    .await;
    // 2e contact sans e-mail → CONTACT_EMAIL_MISSING per-facture.
    let mute_contact = seed_contact(
        &pool,
        company_id,
        admin_id,
        None,
        None,
        Salutation::Madame,
        Some("Muet"),
    )
    .await;
    seed_primary_bank(&pool, company_id).await;
    seed_dunning(&pool, company_id).await;
    let ok_invoice = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let paid_invoice = seed_validated_invoice(&pool, company_id, contact_id, admin_id).await;
    let mute_invoice = seed_validated_invoice(&pool, company_id, mute_contact, admin_id).await;
    sqlx::query("UPDATE invoices SET paid_at = UTC_TIMESTAMP(6) WHERE id = ?")
        .bind(paid_invoice)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn_app(pool.clone(), mock.clone(), true, 20).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": [ok_invoice, paid_invoice, mute_invoice] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "succès partiel = succès HTTP");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1);
    assert_eq!(body["accepted"][0]["invoiceId"], ok_invoice);
    assert_eq!(body["accepted"][0]["levelNumber"], 1);

    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2, "2 échecs per-facture");
    let code_for = |id: i64| -> String {
        failed
            .iter()
            .find(|f| f["invoiceId"] == id)
            .unwrap_or_else(|| panic!("facture {id} absente de failed[]"))["errorCode"]
            .as_str()
            .unwrap()
            .to_string()
    };
    assert_eq!(code_for(paid_invoice), "INVOICE_ALREADY_PAID");
    assert_eq!(code_for(mute_invoice), "CONTACT_EMAIL_MISSING");
    // `details` fait partie de la signature canonique FailedProposal (Epic 8).
    for f in failed {
        assert!(
            f.get("details").is_some(),
            "champ details présent (null admis) : {f}"
        );
    }
    assert_eq!(
        mock.sent_emails().len(),
        1,
        "un seul e-mail effectivement parti"
    );

    // Cap dur 20 : un lot de 21 → 422.
    let too_many: Vec<i64> = (1..=21).collect();
    let cap = app
        .client
        .post(app.url("/api/v1/dunning/reminders/send-batch"))
        .bearer_auth(&token)
        .json(&json!({ "invoiceIds": too_many }))
        .send()
        .await
        .unwrap();
    assert_eq!(cap.status(), 422);
    assert_eq!(
        cap.json::<serde_json::Value>().await.unwrap()["error"]["code"],
        "BATCH_TOO_LARGE"
    );
}
