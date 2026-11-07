//! Story 33-5 (#909) — `PUT /api/v1/companies/current/address` : corriger
//! l'adresse de la société après l'assistant.
//!
//! Le défaut de l'issue se joue de bout en bout (AC-3) : une société enregistrée
//! avant la 0.17.4 avec un pays inconnu ne produisait plus aucune QR-facture, et
//! rien ne permettait de corriger l'adresse hors de l'assistant.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use common::{audit_actions, audit_count, audit_details, create_key_via_http};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::bank_account::NewBankAccount;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::user::{NewUser, Role};
use kesh_db::repositories::{bank_accounts, contacts, invoices, users};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";
const ROUTE: &str = "/api/v1/companies/current/address";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn login(&self, username: &str, password: &str) -> String {
        let resp = self
            .client
            .post(self.url("/api/v1/auth/login"))
            .json(&json!({ "username": username, "password": password }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "connexion de {username}");
        let body: Value = resp.json().await.unwrap();
        body["accessToken"].as_str().unwrap().to_string()
    }

    /// La société telle que l'écran la relit (`GET`, non la réponse du `PUT`).
    async fn company(&self, token: &str) -> Value {
        let resp = self
            .client
            .get(self.url("/api/v1/companies/current"))
            .bearer_auth(token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200);
        resp.json::<Value>().await.unwrap()["company"].clone()
    }

    async fn put_address(&self, token: &str, address: Value, version: i64) -> reqwest::Response {
        self.client
            .put(self.url(ROUTE))
            .bearer_auth(token)
            .json(&json!({ "address": address, "version": version }))
            .send()
            .await
            .unwrap()
    }

    async fn get_pdf(&self, token: &str, invoice_id: i64) -> reqwest::Response {
        self.client
            .get(self.url(&format!("/api/v1/invoices/{invoice_id}/pdf")))
            .bearer_auth(token)
            .send()
            .await
            .unwrap()
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        ADMIN_PASSWORD.to_string(),
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

fn valid_address() -> Value {
    json!({
        "street": "Chemin des Essais",
        "building": "7",
        "postalCode": "1800",
        "city": "Vevey",
        "country": "FR",
    })
}

/// Société complète, connexion Admin : `(app, jeton, company_id, admin_id)`.
async fn setup(pool: &MySqlPool) -> (TestApp, String, i64, i64) {
    let seeded = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    let app = spawn_app(pool.clone()).await;
    let token = app.login("admin", ADMIN_PASSWORD).await;
    (app, token, seeded.company_id, seeded.admin_user_id)
}

/// L'état d'une base d'avant la 0.17.4 : aucune route ne permet plus de l'écrire.
async fn force_country(pool: &MySqlPool, company_id: i64, country: &str) {
    sqlx::query("UPDATE companies SET address_country = ? WHERE id = ?")
        .bind(country)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Une facture validée de société complète, prête pour le `GET` du PDF.
async fn seed_validated_invoice(pool: &MySqlPool, company_id: i64, admin_id: i64) -> i64 {
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
    let contact = contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: "Pia Rutschmann".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Marktgasse".into()),
            address_building: Some("28".into()),
            address_postal_code: Some("9400".into()),
            address_city: Some("Rorschach".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30 jours net".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap();
    let (invoice, _) = invoices::create(
        pool,
        admin_id,
        NewInvoice {
            company_id,
            contact_id: contact.id,
            date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
            payment_terms: Some("30 jours net".into()),
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(7.70),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(pool, company_id, invoice.id, admin_id)
        .await
        .expect("validate_invoice");
    invoice.id
}

async fn create_user(pool: &MySqlPool, company_id: i64, name: &str, role: Role) -> String {
    let password = "password-e2e-33-5";
    users::create(
        pool,
        NewUser {
            username: name.into(),
            password_hash: hash_password(password).expect("hash"),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    password.to_string()
}

/// AC-1 — aller-retour relu par `GET` : c'est ce que l'écran consomme, et le
/// seul endroit où se voit un champ oublié du miroir `CompanyJson`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_valid_address_is_written_and_survives_the_round_trip(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();

    let resp = app.put_address(&token, valid_address(), version).await;
    assert_eq!(resp.status(), 200);
    let put: Value = resp.json().await.unwrap();
    assert_eq!(put["version"].as_i64().unwrap(), version + 1);

    let company = app.company(&token).await;
    assert_eq!(company["version"].as_i64().unwrap(), version + 1);
    assert_eq!(company["addressStructured"], valid_address());
    assert_eq!(company["addressCountryKnown"], true);
    let display = company["address"].as_str().unwrap();
    assert!(
        display.contains("Chemin des Essais")
            && display.contains("1800")
            && display.contains("Vevey"),
        "l'adresse d'affichage suit les composants : {display}"
    );
}

/// Les composants sont rognés et le pays mis en majuscules, comme à l'assistant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn components_are_trimmed_and_the_country_uppercased(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    let raw = json!({
        "street": "  Chemin des Essais ",
        "building": " 7 ",
        "postalCode": " 1800 ",
        "city": " Vevey ",
        "country": " fr ",
    });
    assert_eq!(app.put_address(&token, raw, version).await.status(), 200);
    assert_eq!(
        app.company(&token).await["addressStructured"],
        valid_address()
    );
}

/// AC-2 — chaque refus nomme ce qu'il refuse, et rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn invalid_addresses_are_refused_and_nothing_is_written(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();
    let audit_before = audit_count(&pool).await;

    let with = |key: &str, value: &str| {
        let mut address = valid_address();
        address[key] = json!(value);
        address
    };
    let cases = [
        ("pays inconnu XX", with("country", "XX"), "XX"),
        ("pays inconnu SU", with("country", "SU"), "SU"),
        ("pays de trois lettres", with("country", "CHE"), "2 lettres"),
        ("NPA vide", with("postalCode", "  "), "NPA"),
        ("localité vide", with("city", ""), "localité"),
        ("rue trop longue", with("street", &"r".repeat(71)), "rue"),
        (
            "numéro trop long",
            with("building", &"1".repeat(17)),
            "numéro",
        ),
        ("NPA trop long", with("postalCode", &"1".repeat(17)), "NPA"),
        (
            "localité trop longue",
            with("city", &"v".repeat(36)),
            "localité",
        ),
    ];
    for (name, address, marker) in cases {
        let resp = app.put_address(&token, address, version).await;
        assert_eq!(resp.status(), 400, "{name}");
        let body: Value = resp.json().await.unwrap();
        let message = body["error"]["message"].as_str().unwrap();
        assert!(message.contains(marker), "{name} : {message}");
    }

    assert_eq!(app.company(&token).await, before, "rien n'est écrit");
    assert_eq!(audit_count(&pool).await, audit_before, "aucune trace");
}

/// Revue P1 (A-M3, E-M2) — un caractère que la QR-facture refuse est refusé ici :
/// répondre 200 laisserait la QR-facture en panne en annonçant un succès. Le refus
/// nomme le champ et le caractère ; rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_character_the_qr_bill_refuses_is_refused_here(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();
    let audit_before = audit_count(&pool).await;
    let with = |key: &str, value: &str| {
        let mut address = valid_address();
        address[key] = json!(value);
        address
    };
    for (address, field, code) in [
        // ⚠️ Des caractères que SIX exclut aussi : la liste de `kesh-qrbill` va être
        // recalée sur la directive (33-12), et ces tests ne doivent pas l'épingler.
        (with("street", "Rue \u{2603} Cie"), "rue", "U+2603"),
        (with("building", "1\n2"), "numéro", "U+000A"),
        (with("postalCode", "18\u{2603}00"), "NPA", "U+2603"),
        (with("city", "Ve\u{2603}vey"), "localité", "U+2603"),
    ] {
        let resp = app.put_address(&token, address, version).await;
        assert_eq!(resp.status(), 400, "{field}");
        let body: Value = resp.json().await.unwrap();
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            message.contains(field) && message.contains(code),
            "le refus nomme le champ et le caractère : {message}"
        );
    }
    assert_eq!(app.company(&token).await, before, "rien n'est écrit");
    assert_eq!(audit_count(&pool).await, audit_before, "aucune trace");
}

/// Le témoin du refus ci-dessus : ce que la route accepte, la QR-facture le produit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn what_the_route_accepts_the_qr_bill_produces(pool: MySqlPool) {
    let (app, token, company_id, admin_id) = setup(&pool).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, admin_id).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    let address = json!({
        "street": "Chemin de l'Église-Saint-Jürg",
        "building": "7b",
        "postalCode": "CH-1800",
        "city": "Châtel-Saint-Denis",
        "country": "CH",
    });
    assert_eq!(
        app.put_address(&token, address, version).await.status(),
        200
    );
    assert_eq!(app.get_pdf(&token, invoice_id).await.status(), 200);
}

/// Revue P1 (A-L7, E-L4) — les bornes SIX acceptées, multioctets compris : le compte
/// est en caractères, non en octets.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_exact_length_limits_are_accepted(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    let address = json!({
        "street": "é".repeat(70),
        "building": "é".repeat(16),
        "postalCode": "é".repeat(16),
        "city": "é".repeat(35),
        "country": "CH",
    });
    let resp = app.put_address(&token, address.clone(), version).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(app.company(&token).await["addressStructured"], address);
}

/// Revue P1 (B L-4, A-L7) — l'ordre : la saisie se juge avant la version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_invalid_address_at_a_stale_version_is_a_400(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    let mut address = valid_address();
    address["country"] = json!("XX");
    let resp = app.put_address(&token, address, version - 1).await;
    assert_eq!(resp.status(), 400);
}

/// Revue P1 (E-L3) — `ß` ne se change pas en `SS`, le Soudan du Sud.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_non_ascii_country_is_not_uppercased_into_a_known_code(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    let mut address = valid_address();
    address["country"] = json!("ß");
    let resp = app.put_address(&token, address, version).await;
    assert_eq!(resp.status(), 400);
}

/// Revue P1 (A-L6) — un pays en minuscules, que le PDF refuse, reçoit l'avis.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_lowercase_stored_country_is_not_known(pool: MySqlPool) {
    let (app, token, company_id, admin_id) = setup(&pool).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, admin_id).await;
    force_country(&pool, company_id, "ch").await;
    let company = app.company(&token).await;
    assert_eq!(company["addressCountryKnown"], false, "{company}");
    assert_eq!(
        app.get_pdf(&token, invoice_id).await.status(),
        400,
        "le PDF le refuse bien : l'avis dit vrai"
    );
}

/// Revue P1 (E-M3) — l'ancien texte libre d'une société d'avant les composants, seule
/// source de son adresse, reste à l'audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_overwritten_free_text_stays_in_the_audit(pool: MySqlPool) {
    let (app, token, company_id, _) = setup(&pool).await;
    sqlx::query(
        "UPDATE companies SET address = 'Ancienne adresse libre', address_street = '', \
         address_building = '', address_postal_code = '', address_city = '', \
         address_country = '' WHERE id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();
    let version = app.company(&token).await["version"].as_i64().unwrap();

    assert_eq!(
        app.put_address(&token, valid_address(), version)
            .await
            .status(),
        200
    );

    let details = audit_details(&pool, "company", company_id, "company.updated")
        .await
        .expect("details_json");
    assert_eq!(details["before"]["address"], "Ancienne adresse libre");
    assert!(
        details["after"].get("address").is_none(),
        "le nouveau texte se déduit des composants : {details}"
    );
}

/// AC-2, C-e334-5 — un pays inconnu reste refusé **même inchangé** : c'est la
/// correction elle-même. L'accepter annoncerait un succès en laissant la
/// QR-facture en panne (mutation M1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_unknown_country_already_stored_is_still_refused(pool: MySqlPool) {
    let (app, token, company_id, _) = setup(&pool).await;
    force_country(&pool, company_id, "SU").await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();
    let audit_before = audit_count(&pool).await;

    let mut address = valid_address();
    address["country"] = json!("SU");
    let resp = app.put_address(&token, address, version).await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert!(body["error"]["message"].as_str().unwrap().contains("SU"));

    assert_eq!(app.company(&token).await, before, "version inchangée");
    assert_eq!(audit_count(&pool).await, audit_before);
}

/// AC-3 — le défaut de l'issue, de bout en bout.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn correcting_the_country_restores_the_qr_invoice(pool: MySqlPool) {
    let (app, token, company_id, admin_id) = setup(&pool).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, admin_id).await;
    force_country(&pool, company_id, "SU").await;

    let company = app.company(&token).await;
    assert_eq!(company["addressCountryKnown"], false);
    assert_eq!(company["addressStructured"]["country"], "SU");

    let resp = app.get_pdf(&token, invoice_id).await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVOICE_NOT_PDF_READY");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Pays invalide"),
        "{body}"
    );

    let mut address = valid_address();
    address["country"] = json!("CH");
    let resp = app
        .put_address(&token, address, company["version"].as_i64().unwrap())
        .await;
    assert_eq!(resp.status(), 200);
    assert_eq!(app.company(&token).await["addressCountryKnown"], true);

    let resp = app.get_pdf(&token, invoice_id).await;
    assert_eq!(resp.status(), 200);
    assert!(resp.bytes().await.unwrap().starts_with(b"%PDF-1."));
}

/// AC-4 — version périmée : 409, sans écriture ni trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_is_refused_without_writing(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();
    let audit_before = audit_count(&pool).await;

    let resp = app.put_address(&token, valid_address(), version - 1).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");

    assert_eq!(app.company(&token).await, before);
    assert_eq!(audit_count(&pool).await, audit_before);
}

/// AC-4 — valeur identique : 200, sans bump de version ni trace (KF-004, M3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_identical_address_is_a_no_op_without_audit(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let version = app.company(&token).await["version"].as_i64().unwrap();
    assert_eq!(
        app.put_address(&token, valid_address(), version)
            .await
            .status(),
        200
    );
    let audit_before = audit_count(&pool).await;

    let resp = app.put_address(&token, valid_address(), version + 1).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["version"].as_i64().unwrap(), version + 1);
    assert_eq!(audit_count(&pool).await, audit_before);
}

/// AC-5 — une société créée avant le 2026-07-05 n'a que le texte libre : la même
/// route renseigne les composants, et le PDF cesse de les réclamer.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_free_text_only_company_gets_its_components(pool: MySqlPool) {
    let (app, token, company_id, admin_id) = setup(&pool).await;
    let invoice_id = seed_validated_invoice(&pool, company_id, admin_id).await;
    sqlx::query(
        "UPDATE companies SET address = 'Ancienne adresse libre', address_street = '', \
         address_building = '', address_postal_code = '', address_city = '', \
         address_country = '' WHERE id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();

    let company = app.company(&token).await;
    assert_eq!(company["address"], "Ancienne adresse libre");
    assert_eq!(
        company["addressCountryKnown"], true,
        "un pays vide n'est pas un pays inconnu"
    );
    let resp = app.get_pdf(&token, invoice_id).await;
    assert_eq!(resp.status(), 400);
    let refused: Value = resp.json().await.unwrap();
    assert_eq!(refused["error"]["code"], "INVOICE_NOT_PDF_READY");
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("adresse de l'entreprise est vide"),
        "{refused}"
    );

    let resp = app
        .put_address(
            &token,
            valid_address(),
            company["version"].as_i64().unwrap(),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let company = app.company(&token).await;
    assert_eq!(company["addressStructured"], valid_address());
    assert!(
        !company["address"]
            .as_str()
            .unwrap()
            .contains("Ancienne adresse"),
        "le texte libre est remplacé"
    );

    let resp = app.get_pdf(&token, invoice_id).await;
    assert_eq!(resp.status(), 200, "le PDF ne réclame plus l'adresse");
}

/// AC-6 — Comptable et Consultation : 403 ; rien n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn only_an_administrator_may_correct_the_address(pool: MySqlPool) {
    let (app, token, company_id, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();

    for (name, role) in [
        ("compta335", Role::Comptable),
        ("consult335", Role::Consultation),
    ] {
        let password = create_user(&pool, company_id, name, role).await;
        let user_token = app.login(name, &password).await;
        let resp = app.put_address(&user_token, valid_address(), version).await;
        assert_eq!(resp.status(), 403, "{name}");
    }
    assert_eq!(app.company(&token).await, before);
}

/// AC-6 — une clé d'API, même en écriture, n'y a pas accès (`require_not_pat`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_may_not_correct_the_address(pool: MySqlPool) {
    let (app, token, _, _) = setup(&pool).await;
    let before = app.company(&token).await;
    let version = before["version"].as_i64().unwrap();
    let (_, key) =
        create_key_via_http(&app.client, &app.base_url, &token, "cle-33-5", "read-write").await;

    let resp = app.put_address(&key, valid_address(), version).await;
    assert_eq!(resp.status(), 403);
    assert_eq!(app.company(&token).await, before);
}

/// AC-6 — jamais d'autre société : la route ne suit que le rattachement de
/// l'utilisateur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn another_company_is_never_touched(pool: MySqlPool) {
    let (app, token, company_id, _) = setup(&pool).await;
    common::create_test_company(&pool).await;
    let other: (i64, String, String, i32) = sqlx::query_as(
        "SELECT id, address_city, address_country, version FROM companies WHERE id <> ?",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let version = app.company(&token).await["version"].as_i64().unwrap();
    assert_eq!(
        app.put_address(&token, valid_address(), version)
            .await
            .status(),
        200
    );

    let after: (i64, String, String, i32) = sqlx::query_as(
        "SELECT id, address_city, address_country, version FROM companies WHERE id = ?",
    )
    .bind(other.0)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(after, other);
    assert_eq!(
        app.company(&token).await["addressStructured"]["city"],
        "Vevey"
    );
}

/// AC-7 — `update_in_tx` est un full-replace : tout champ non reporté serait
/// effacé en silence, en 200. Et le sens inverse : les routes voisines ne
/// touchent pas l'adresse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_independant_keeps_what_the_address_route_does_not_touch(pool: MySqlPool) {
    assert_neighbours_preserve(pool, "Independant").await;
}

/// Le même, pour une PME : un `org_type` fixe laisserait l'autre test vert.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_pme_keeps_what_the_address_route_does_not_touch(pool: MySqlPool) {
    assert_neighbours_preserve(pool, "Pme").await;
}

async fn assert_neighbours_preserve(pool: MySqlPool, org_type: &str) {
    let (app, token, company_id, _) = setup(&pool).await;
    sqlx::query(
        "UPDATE companies SET first_name = 'Prénom Essai', last_name = 'Nom Essai', \
         ide_number = 'CHE109322551', email = 'reply@demo.ch', \
         phone = '+41 21 123 45 67', website = 'https://demo.ch', is_stub = TRUE, \
         org_type = ?, accounting_language = 'DE', instance_language = 'IT', \
         books_locked_through = '2020-12-31' WHERE id = ?",
    )
    .bind(org_type)
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();
    let before = app.company(&token).await;
    let snapshot = |pool: MySqlPool| async move {
        let row: (bool,) = sqlx::query_as("SELECT is_stub FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        row.0
    };

    let resp = app
        .put_address(&token, valid_address(), before["version"].as_i64().unwrap())
        .await;
    assert_eq!(resp.status(), 200);
    let after = app.company(&token).await;
    for field in [
        "name",
        "firstName",
        "lastName",
        "ideNumber",
        "orgType",
        "accountingLanguage",
        "instanceLanguage",
        "email",
        "phone",
        "website",
        "booksLockedThrough",
    ] {
        // ⚠️ Une valeur nulle ou égale au défaut comparerait vide à vide : le montage
        // pose chaque champ (revue P1, E-M1).
        assert!(!before[field].is_null(), "montage : {field} non posé");
        assert_eq!(after[field], before[field], "champ {field} perdu");
    }
    assert_eq!(before["orgType"], org_type, "montage");
    assert_eq!(before["accountingLanguage"], "DE", "montage");
    assert_eq!(before["instanceLanguage"], "IT", "montage");
    assert!(snapshot(pool.clone()).await, "is_stub ne se lève pas ici");

    // Le sens inverse : e-mail, puis coordonnées, sur l'adresse corrigée.
    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/email"))
        .bearer_auth(&token)
        .json(&json!({ "email": "autre@demo.ch", "version": after["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let version = resp.json::<Value>().await.unwrap()["version"].clone();
    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .bearer_auth(&token)
        .json(&json!({ "phone": null, "website": null, "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let last = app.company(&token).await;
    assert_eq!(last["addressStructured"], valid_address());
    assert_eq!(last["address"], after["address"]);
}

/// AC-8 — une entrée, bornée aux cinq composants, attribuée à l'Administrateur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_audit_entry_carries_the_five_components_only(pool: MySqlPool) {
    let (app, token, company_id, admin_id) = setup(&pool).await;
    let before = app.company(&token).await;
    assert_eq!(audit_actions(&pool, "company", company_id).await.len(), 0);

    let resp = app
        .put_address(&token, valid_address(), before["version"].as_i64().unwrap())
        .await;
    assert_eq!(resp.status(), 200);

    assert_eq!(
        audit_actions(&pool, "company", company_id).await,
        vec!["company.updated".to_string()]
    );
    let details = audit_details(&pool, "company", company_id, "company.updated")
        .await
        .expect("details_json");
    assert_eq!(
        details,
        json!({ "before": before["addressStructured"], "after": valid_address() })
    );
    let actor = common::audit_actor(&pool, "company", company_id, "company.updated").await;
    assert_eq!(actor.2, admin_id);
}
