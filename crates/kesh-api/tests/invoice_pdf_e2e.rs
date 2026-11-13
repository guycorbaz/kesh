//! Story 5.3 — tests E2E pour `GET /api/v1/invoices/:id/pdf`.
//!
//! Ces tests tournent contre une DB MariaDB réelle (pattern `#[sqlx::test]`
//! identique aux autres `*_e2e.rs`). Ils couvrent les 9 cas exigés par T6 :
//! 200 (happy path QR-IBAN), 400 `INVOICE_NOT_VALIDATED`, 400 `INVOICE_NOT_PDF_READY`
//! (×2), 404 autre company, 400 `INVOICE_TOO_MANY_LINES_FOR_PDF`, 401 sans JWT,
//! 200 pour chaque rôle (3 cas).
//!
//! Depuis Story 6.4 : seed via `kesh_db::test_fixtures::seed_accounting_company`
//! + validation via `kesh_db::repositories::invoices::validate_invoice`
//!   (aucun INSERT manuel ni UPDATE direct sur `invoices.status` — KF-001 closed).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::credit_note::NewCreditNote;
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, contacts, credit_notes, invoices, users};
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

async fn seed_contact(pool: &MySqlPool, company_id: i64, user_id: i64, with_address: bool) -> i64 {
    let contact = contacts::create(
        pool,
        user_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: with_address.then(|| "Marktgasse".into()),
            address_building: with_address.then(|| "28".into()),
            address_postal_code: with_address.then(|| "9400".into()),
            address_city: with_address.then(|| "Rorschach".into()),
            address_country: with_address.then(|| "CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30 jours net".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
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

/// Crée une facture `draft` puis la valide via le flow normal
/// (`kesh_db::repositories::invoices::validate_invoice`, équivalent in-process
/// de `POST /api/v1/invoices/:id/validate`). Aucun INSERT manuel ni UPDATE
/// direct sur `invoices.status` — KF-001 closed via Story 6.4.
///
/// Prérequis : `seed_accounting_company` doit avoir été appelé au préalable
/// (fiscal_year Open + company_invoice_settings complet).
async fn seed_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    user_id: i64,
    n_lines: usize,
) -> i64 {
    let lines: Vec<NewInvoiceLine> = (0..n_lines)
        .map(|i| NewInvoiceLine {
            revenue_account_id: None,
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
        project_id: None,
    };
    let (invoice, _lines) = invoices::create(pool, user_id, new).await.unwrap();

    kesh_db::repositories::invoices::validate_invoice(pool, company_id, invoice.id, user_id)
        .await
        .expect("validate_invoice");

    invoice.id
}

// --- Tests -------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
                revenue_account_id: None,
                description: "Item".into(),
                quantity: dec!(1),
                unit_price: dec!(100.0),
                vat_rate: dec!(7.70),
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
        .get(app.url(&format!("/api/v1/invoices/{}/pdf", invoice.id)))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_VALIDATED");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

async fn seed_user_with_role(
    pool: &MySqlPool,
    username: &str,
    password: &str,
    role: Role,
    company_id: i64,
) {
    let phc = hash_password(password).expect("hash password");
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: phc,
            role,
            active: true,
            company_id,
            email: None,
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
    seed_user_with_role(&pool, username, password, role, company_id).await;

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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pdf_comptable_role_returns_200(pool: MySqlPool) {
    run_pdf_role_scenario(pool, "comptable_pdf", Role::Comptable).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pdf_consultation_role_returns_200(pool: MySqlPool) {
    run_pdf_role_scenario(pool, "observateur_pdf", Role::Consultation).await;
}

/// Story 16-3a, passe 6 de revue — **la chaîne complète du refus d'en-tête**,
/// du rendu PDF jusqu'au code d'erreur lu par le frontend.
///
/// ⚠️ Ce test comble un trou de garde, pas un défaut d'état : les littéraux
/// `INVOICE_PDF_HEADER_OVERFLOW` d'`errors.rs` et de la liste blanche de
/// `invoices/[id]/+page.svelte` coïncident aujourd'hui. Ce qui manquait, c'est
/// ce qui les **tient ensemble**. Le seul test du débordement haut vivait dans
/// `kesh-qrbill` et s'arrêtait à `matches!(err, QrBillError::HeaderOverflow(_))`
/// — sans traverser `map_qrbill_error`, ni `IntoResponse`, ni HTTP. Si la chaîne
/// dérivait (renommage du variant, faute de frappe, harmonisation), le frontend
/// retomberait sur `'invoice-pdf-error-generic'`, les quatre traductions
/// redeviendraient mortes, et **le gate resterait vert** : c'est le HIGH de la
/// passe 3, rejoué un cran plus bas.
///
/// Le jumeau `pdf_too_many_lines_returns_400` couvre la garde **basse** depuis
/// l'origine ; celui-ci couvre la garde **haute**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pdf_header_overflow_returns_400_with_its_own_code(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, true).await;
    // 2 lignes seulement : la garde BASSE ne doit pas se déclencher à la place
    // de celle qu'on mesure.
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 2).await;

    // L'émetteur porte ses trois coordonnées ET une adresse à rallonge.
    sqlx::query(
        "UPDATE companies SET address = ?, phone = ?, email = ?, website = ? WHERE id = ?",
    )
    .bind("Ligne 1\nLigne 2\nLigne 3\nLigne 4\nLigne 5\nLigne 6\nLigne 7\nLigne 8\nLigne 9\nLigne 10")
    .bind("+41 21 123 45 67")
    .bind("contact@demo.ch")
    .bind("https://demo.ch")
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();

    // Le destinataire aussi : c'est le cumul des deux blocs qui fait déborder.
    sqlx::query("UPDATE contacts SET address = ? WHERE id = ?")
        .bind("Dest 1\nDest 2\nDest 3\nDest 4\nDest 5\nDest 6\nDest 7\nDest 8\nDest 9\nDest 10")
        .bind(contact_id)
        .execute(&pool)
        .await
        .unwrap();

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "un en-tête qui déborde doit être refusé, pas imprimé par-dessus le tableau"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "INVOICE_PDF_HEADER_OVERFLOW",
        "le code doit être CELUI de la garde haute — c'est le littéral que la \
         liste blanche du frontend traduit ; un code générique rendrait les \
         quatre traductions mortes sans qu'aucun test ne le voie"
    );
}

// --- Story 35-2a (#955, #964) — le refus nomme le champ et le caractère ------

/// La valeur fr-CH d'une clé sans variable, lue au catalogue réel : un message
/// sans argument se compare entier, ce qui prouve qu'il ne porte aucun texte de
/// `kesh-qrbill`.
fn fr(cle: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("kesh-i18n/locales");
    let bundle = kesh_i18n::I18nBundle::load(&dir).expect("catalogue");
    let valeur = bundle.format(&kesh_i18n::Locale::FrCh, cle, None);
    assert_ne!(valeur, cle, "{cle} absente du catalogue fr-CH");
    valeur
}

/// Le montage commun : une facture validée, jamais figée (le téléchargement la
/// rend), un client à l'adresse complète et un compte principal sans QR-IBAN.
async fn montage_refus(pool: &MySqlPool) -> (i64, i64, i64) {
    let (admin_id, company_id) = seed_base(pool).await;
    let contact_id = seed_contact(pool, company_id, admin_id, true).await;
    seed_primary_bank(pool, company_id, false).await;
    let invoice_id = seed_validated_invoice(pool, company_id, contact_id, admin_id, 1).await;
    (company_id, contact_id, invoice_id)
}

/// Télécharge le PDF, exige le 400 `INVOICE_NOT_PDF_READY` sans `details`, et
/// rend le message ; il ne porte ni variable Fluent, ni marque d'isolation, ni
/// nom technique de `kesh-qrbill`.
async fn message_du_refus(pool: &MySqlPool, invoice_id: i64) -> String {
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
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY", "{body}");
    assert!(body["error"]["details"].is_null(), "aucun details : {body}");
    let message = body["error"]["message"].as_str().unwrap().to_string();
    message_propre(&message);
    message
}

/// AC1 : ni `{`, ni `$`, ni « Champ », ni nom technique, ni marque d'isolation.
fn message_propre(message: &str) {
    for interdit in [
        "{",
        "$",
        "Champ",
        "creditor.",
        "debtor.",
        "address.",
        "unstructured_message",
        "billing_information",
        "\u{2068}",
        "\u{2069}",
    ] {
        assert!(
            !message.contains(interdit),
            "« {interdit} » dans le message : {message:?}"
        );
    }
    assert!(!message.trim().is_empty());
}

async fn sql(pool: &MySqlPool, requete: &str, valeur: &str, id: i64) {
    sqlx::query(requete)
        .bind(valeur)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

/// AC1 — la localité du client porte un tiret demi-cadratin : le message nomme
/// le champ, montre le caractère et son point de code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_la_localite_du_client_et_le_caractere(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE contacts SET address_city = ? WHERE id = ?",
        "Ve\u{2013}vey",
        contact_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("localité du client"), "{m}");
    assert!(m.contains('\u{2013}'), "{m}");
    assert!(m.contains("U+2013"), "{m}");
}

/// AC1 — un saut de ligne s'écrit `\n` (deux caractères), jamais tel quel.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_ecrit_un_saut_de_ligne_echappe(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE contacts SET address_building = ? WHERE id = ?",
        "1\n2",
        contact_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("numéro"), "{m}");
    assert!(m.contains("client"), "{m}");
    assert!(m.contains("\\n"), "{m}");
    assert!(!m.contains('\n'), "{m:?}");
    assert!(m.contains("U+000A"), "{m}");
}

/// AC1 — un nom de 71 caractères : le champ et la limite de 70.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_le_nom_trop_long_et_sa_limite(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE contacts SET name = ? WHERE id = ?",
        &"A".repeat(71),
        contact_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("nom du client"), "{m}");
    assert!(m.contains("70"), "{m}");
}

/// AC1 — un nom d'entreprise qui ne porte qu'un trait d'union conditionnel est vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_le_nom_vide_de_l_entreprise(pool: MySqlPool) {
    let (company_id, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE companies SET name = ? WHERE id = ?",
        "\u{AD}",
        company_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await.to_lowercase();
    assert!(m.contains("nom de l'entreprise"), "{m}");
    assert!(m.contains("vide"), "{m}");
    // Revue de code P1 (B1) — le nom ne se modifie pas dans Paramètres →
    // Organisation : la phrase ne doit pas y renvoyer.
    assert!(!m.contains("organisation"), "{m}");
}

/// AC1, D9 — un caractère refusé dans la communication vient du numéro de
/// facture, qu'une facture validée ne corrige pas : le message renvoie au format.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_du_numero_de_facture_renvoie_au_format(pool: MySqlPool) {
    let (_, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE invoices SET invoice_number = ? WHERE id = ?",
        "F\u{2013}1",
        invoice_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("numéro de facture"), "{m}");
    assert!(m.contains("U+2013"), "{m}");
    assert!(m.contains("Administration → Facturation"), "{m}");
}

/// AC4 (#964) — un NPA de l'entreprise qui ne porte qu'U+00AD : l'adresse est
/// dite incomplète, le champ est nommé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_le_npa_vide_de_l_entreprise(pool: MySqlPool) {
    let (company_id, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE companies SET address_postal_code = ? WHERE id = ?",
        "\u{AD}",
        company_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await.to_lowercase();
    assert!(m.contains("adresse de l'entreprise incomplète"), "{m}");
    // Revue de code P1 (B2) — la proposition qui nomme le champ, et non l'autre :
    // la parenthèse « NPA et localité » porte les deux mots.
    assert!(m.contains("le npa est vide"), "{m}");
    assert!(!m.contains("la localité est vide"), "{m}");
}

/// AC4 (#964) — une localité du client qui ne porte qu'U+200B.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_la_localite_vide_du_client(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE contacts SET address_city = ? WHERE id = ?",
        "\u{200B}",
        contact_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await.to_lowercase();
    assert!(m.contains("adresse du client incomplète"), "{m}");
    // Revue de code P1 (B2) — la proposition qui nomme le champ, et non l'autre.
    assert!(m.contains("la localité est vide"), "{m}");
    assert!(!m.contains("le npa est vide"), "{m}");
}

/// AC4 — un client sans aucune adresse garde la phrase de l'adresse requise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_d_un_client_sans_adresse(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, false).await;
    seed_primary_bank(&pool, company_id, false).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 1).await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert_eq!(m, fr("invoice-pdf-error-client-address-required"));
}

/// AC5 (D7) — le pays de l'entreprise hors de la liste ISO : la partie et le code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_le_pays_de_l_entreprise(pool: MySqlPool) {
    let (company_id, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE companies SET address_country = ? WHERE id = ?",
        "SU",
        company_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("pays de l'entreprise"), "{m}");
    assert!(m.contains("SU"), "{m}");
}

/// AC5 (D7) — le pays du client hors de la liste (posé en SQL : la saisie le
/// refuse depuis la 0.17.4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_nomme_le_pays_du_client(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE contacts SET address_country = ? WHERE id = ?",
        "SU",
        contact_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert!(m.contains("pays du client"), "{m}");
    assert!(m.contains("SU"), "{m}");
}

/// AC6 — le compte principal absent : la phrase du catalogue, entière.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_sans_compte_principal(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 1).await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert_eq!(m, fr("invoice-pdf-error-no-primary-bank"));
}

/// AC6 — un IBAN à la clé de contrôle fausse : la phrase du catalogue, sans le
/// texte de `kesh-qrbill`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_d_un_iban_invalide(pool: MySqlPool) {
    let (company_id, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE bank_accounts SET iban = ? WHERE company_id = ? AND is_primary",
        "CH9300762011623852958",
        company_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert_eq!(m, fr("invoice-pdf-error-iban-invalid"));
}

/// AC6 — un « QR-IBAN » qui n'en est pas un (IID hors de 30000-31999).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_d_un_qr_iban_invalide(pool: MySqlPool) {
    let (company_id, _, invoice_id) = montage_refus(&pool).await;
    sql(
        &pool,
        "UPDATE bank_accounts SET qr_iban = ? WHERE company_id = ? AND is_primary",
        "CH9300762011623852957",
        company_id,
    )
    .await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert_eq!(m, fr("invoice-pdf-error-qr-iban-invalid"));
}

/// Retire un contact malgré les clés étrangères, sur une seule connexion : le
/// seul moyen d'atteindre « contact introuvable », qu'aucune route ne produit.
async fn retirer_le_contact(pool: &MySqlPool, contact_id: i64) {
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("DELETE FROM contacts WHERE id = ?")
        .bind(contact_id)
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
}

/// AC6 — le contact de la facture introuvable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_d_un_contact_introuvable(pool: MySqlPool) {
    let (_, contact_id, invoice_id) = montage_refus(&pool).await;
    retirer_le_contact(&pool, contact_id).await;
    let m = message_du_refus(&pool, invoice_id).await;
    assert_eq!(m, fr("invoice-pdf-error-contact-missing"));
}

/// AC11, D8 — le PDF d'un avoir dont le contact est introuvable : la phrase de
/// l'avoir, et non celle de la facture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_du_pdf_d_un_avoir_sans_contact(pool: MySqlPool) {
    let (admin_id, company_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, true).await;
    seed_primary_bank(&pool, company_id, false).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, contact_id, admin_id, 1).await;
    let avoir = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id,
            invoice_id,
            date: NaiveDate::from_ymd_opt(2026, 4, 20).unwrap(),
        },
        admin_id,
    )
    .await
    .unwrap();
    retirer_le_contact(&pool, contact_id).await;

    let app = spawn_app(pool.clone()).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/credit-notes/{}/pdf",
            avoir.credit_note.id
        )))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY", "{body}");
    let m = body["error"]["message"].as_str().unwrap();
    assert_eq!(m, fr("credit-note-pdf-error-contact-missing"));
    assert_ne!(m, fr("invoice-pdf-error-contact-missing"));
}
