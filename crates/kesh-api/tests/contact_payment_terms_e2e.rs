//! End-to-end HTTP tests — Story 21-1 (#245) : conditions de paiement
//! structurées (jours) sur le contact → échéance de facture pré-calculée.
//!
//! Couvre les ACs 25 de la spec :
//! - days + label localisé dans la réponse contact (FR, DE, cas 0 jour) ;
//! - bornes 0..365 (400 hors bornes) ;
//! - défauts à la création de facture (due_date = date + jours, payment_terms
//!   = libellé auto langue contact) — valeurs explicites jamais écrasées ;
//! - contact sans jours → comportement historique (due_date = date) ;
//! - validation `due_date >= date` (400) à la création ;
//! - règle legacy à l'update : paire (date, due_date) inchangée → accepté,
//!   paire modifiée avec due_date < date → 400.
//!
//! Harnais calqué sur `kf004_no_op_e2e.rs` (spawn_app + login + seed company).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{users, vat_rates};
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-password".to_string(),
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
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");
    let body: serde_json::Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
}

async fn create_company_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
) -> i64 {
    let hash = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

/// Société seedée (exercice + comptes + settings facturation + taux TVA) —
/// nécessaire pour créer des factures via l'API.
async fn create_seeded_company(pool: &MySqlPool) -> i64 {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('PTD Test Co', 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("fiscal_year insert");

    let mut accounts = std::collections::HashMap::new();
    for (code, name, account_type) in
        &[("1100", "Créances", "Asset"), ("3000", "Ventes", "Revenue")]
    {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("account insert");
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, default_sales_journal) \
         VALUES (?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .execute(pool)
    .await
    .expect("company_invoice_settings insert");

    vat_rates::seed_default_swiss_rates(pool, company_id)
        .await
        .expect("vat_rates seed");

    company_id
}

/// POST /api/v1/contacts minimal (Entreprise cliente) avec délai optionnel.
async fn create_contact(
    app: &TestApp,
    token: &str,
    name: &str,
    days: Option<i32>,
    language: Option<&str>,
) -> serde_json::Value {
    let mut body = json!({
        "contactType": "Entreprise",
        "name": name,
        "isClient": true,
        "isSupplier": false,
    });
    if let Some(d) = days {
        body["defaultPaymentTermsDays"] = json!(d);
    }
    if let Some(l) = language {
        body["language"] = json!(l);
    }
    let resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("contact create request");
    assert_eq!(resp.status(), 201, "contact create should return 201");
    resp.json().await.expect("contact json")
}

fn invoice_payload(contact_id: i64, date: &str) -> serde_json::Value {
    json!({
        "contactId": contact_id,
        "date": date,
        "lines": [
            {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
        ]
    })
}

// ---------------------------------------------------------------------------
// Contact : days + label localisé dans la réponse
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_with_days_returns_label_fr_and_days(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_user", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_user", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Client FR", Some(30), None).await;
    assert_eq!(contact["defaultPaymentTermsDays"], 30);
    assert_eq!(
        contact["defaultPaymentTermsLabel"], "Payable à 30 jours net",
        "libellé FR attendu (langue contact absente → fallback instance FR)"
    );

    // Le label doit aussi être présent sur l'endpoint LIST (ContactPicker).
    let list: serde_json::Value = app
        .client
        .get(app.url("/api/v1/contacts?search=PTD%20Client%20FR"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list request")
        .json()
        .await
        .expect("list json");
    let item = list["items"]
        .as_array()
        .and_then(|a| a.iter().find(|c| c["name"] == "PTD Client FR"))
        .expect("contact présent dans la liste")
        .clone();
    assert_eq!(item["defaultPaymentTermsLabel"], "Payable à 30 jours net");
    assert_eq!(item["defaultPaymentTermsDays"], 30);

    // GET détail : idem.
    let id = contact["id"].as_i64().unwrap();
    let detail: serde_json::Value = app
        .client
        .get(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get request")
        .json()
        .await
        .expect("get json");
    assert_eq!(detail["defaultPaymentTermsLabel"], "Payable à 30 jours net");

    // Review Pass 1 AA-1 : le label est posé aussi sur UPDATE et ARCHIVE
    // (les 2 handlers corrigés par validate Pass 1 V1-2 — verrou anti-régression).
    let updated: serde_json::Value = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "PTD Client FR",
            "isClient": true,
            "isSupplier": false,
            "defaultPaymentTermsDays": 45,
            "version": detail["version"],
        }))
        .send()
        .await
        .expect("update request")
        .json()
        .await
        .expect("update json");
    assert_eq!(updated["defaultPaymentTermsDays"], 45);
    assert_eq!(
        updated["defaultPaymentTermsLabel"],
        "Payable à 45 jours net"
    );

    let archived: serde_json::Value = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}/archive")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": updated["version"] }))
        .send()
        .await
        .expect("archive request")
        .json()
        .await
        .expect("archive json");
    assert_eq!(
        archived["defaultPaymentTermsLabel"],
        "Payable à 45 jours net"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_language_de_gets_de_label_and_zero_days_immediate(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_de", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_de", "password-12345").await;

    let de = create_contact(&app, &token, "PTD Kunde DE", Some(30), Some("DE")).await;
    assert_eq!(
        de["defaultPaymentTermsLabel"], "Zahlbar innert 30 Tagen",
        "libellé DE attendu (langue du CONTACT, pas la locale de la requête)"
    );

    let comptant = create_contact(&app, &token, "PTD Comptant", Some(0), None).await;
    assert_eq!(comptant["defaultPaymentTermsLabel"], "Payable au comptant");

    // Review Pass 1 ECH-1 : singulier (le libellé part sur le PDF client).
    let un_jour = create_contact(&app, &token, "PTD Un Jour", Some(1), None).await;
    assert_eq!(un_jour["defaultPaymentTermsLabel"], "Payable à 1 jour net");
    let un_tag = create_contact(&app, &token, "PTD Ein Tag", Some(1), Some("DE")).await;
    assert_eq!(un_tag["defaultPaymentTermsLabel"], "Zahlbar innert 1 Tag");

    let sans = create_contact(&app, &token, "PTD Sans Délai", None, None).await;
    assert!(sans["defaultPaymentTermsDays"].is_null());
    assert!(sans["defaultPaymentTermsLabel"].is_null());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_days_out_of_bounds_returns_400(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_bounds", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_bounds", "password-12345").await;

    for bad in [-1, 366] {
        let resp = app
            .client
            .post(app.url("/api/v1/contacts"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({
                "contactType": "Entreprise",
                "name": format!("PTD Bad {bad}"),
                "isClient": true,
                "isSupplier": false,
                "defaultPaymentTermsDays": bad,
            }))
            .send()
            .await
            .expect("request");
        assert_eq!(resp.status(), 400, "days={bad} doit être rejeté en 400");
    }
}

// ---------------------------------------------------------------------------
// Facture : défauts due_date + payment_terms depuis le contact
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_defaults_from_contact_days(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_inv", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_inv", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Client 30j", Some(30), None).await;
    let contact_id = contact["id"].as_i64().unwrap();

    // Sans dueDate ni paymentTerms → date + 30 et libellé auto FR.
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&invoice_payload(contact_id, "2026-07-01"))
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("invoice json");
    assert_eq!(invoice["dueDate"], "2026-07-31", "date + 30 jours attendu");
    assert_eq!(invoice["paymentTerms"], "Payable à 30 jours net");

    // Valeurs explicites → jamais écrasées.
    let mut explicit = invoice_payload(contact_id, "2026-07-01");
    explicit["dueDate"] = json!("2026-07-10");
    explicit["paymentTerms"] = json!("10 jours (négocié)");
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&explicit)
        .send()
        .await
        .expect("invoice create explicit");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("invoice json");
    assert_eq!(invoice["dueDate"], "2026-07-10");
    assert_eq!(invoice["paymentTerms"], "10 jours (négocié)");

    // Review Pass 1 ECH-2 — combinaisons mixtes :
    // (a) dueDate explicite + paymentTerms absent → PAS de libellé auto
    // (sinon « Payable à 30 jours net » incohérent avec une échéance à 9 j).
    let mut mixed_a = invoice_payload(contact_id, "2026-07-01");
    mixed_a["dueDate"] = json!("2026-07-10");
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&mixed_a)
        .send()
        .await
        .expect("invoice mixed a");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(invoice["dueDate"], "2026-07-10");
    assert!(
        invoice["paymentTerms"].is_null(),
        "dueDate explicite ⇒ pas de libellé auto (cohérence PDF)"
    );

    // (b) paymentTerms explicite + dueDate absente → échéance calculée,
    // libellé de l'utilisateur conservé.
    let mut mixed_b = invoice_payload(contact_id, "2026-07-01");
    mixed_b["paymentTerms"] = json!("selon contrat");
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&mixed_b)
        .send()
        .await
        .expect("invoice mixed b");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(invoice["dueDate"], "2026-07-31");
    assert_eq!(invoice["paymentTerms"], "selon contrat");
}

/// Review Pass 1 BH-1 : `date + délai` proche de `NaiveDate::MAX` ne doit PAS
/// paniquer (chrono `Add` panique sur overflow) → 400 propre.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_date_overflow_returns_400_not_panic(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_ovf", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_ovf", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Overflow", Some(365), None).await;
    let contact_id = contact["id"].as_i64().unwrap();

    // NaiveDate::MAX = +262142-12-31 ; MAX - 100 jours + 365 jours → overflow.
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&invoice_payload(contact_id, "+262142-10-01"))
        .send()
        .await
        .expect("invoice overflow");
    assert_eq!(
        resp.status(),
        400,
        "overflow chrono → 400 Validation, jamais de panic/500"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_defaults_use_contact_language_for_label(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_inv_de", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_inv_de", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Kunde 30T", Some(30), Some("DE")).await;
    let contact_id = contact["id"].as_i64().unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&invoice_payload(contact_id, "2026-07-01"))
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("invoice json");
    assert_eq!(invoice["paymentTerms"], "Zahlbar innert 30 Tagen");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_without_contact_days_keeps_historical_default(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_hist", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_hist", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Sans Jours", None, None).await;
    let contact_id = contact["id"].as_i64().unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&invoice_payload(contact_id, "2026-07-01"))
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("invoice json");
    assert_eq!(
        invoice["dueDate"], "2026-07-01",
        "défaut historique due_date = date (Review P6) conservé"
    );
    assert!(invoice["paymentTerms"].is_null());
}

// ---------------------------------------------------------------------------
// Validation due_date >= date
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_due_date_before_date_rejected_on_create(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ptd_val", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_val", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Val", None, None).await;
    let contact_id = contact["id"].as_i64().unwrap();

    let mut payload = invoice_payload(contact_id, "2026-07-01");
    payload["dueDate"] = json!("2026-06-30");
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&payload)
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 400, "due_date < date doit être rejeté");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invoice_legacy_due_date_pair_unchanged_editable(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    let user_id = create_company_user(&pool, company_id, "ptd_leg", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ptd_leg", "password-12345").await;

    let contact = create_contact(&app, &token, "PTD Legacy", None, None).await;
    let contact_id = contact["id"].as_i64().unwrap();

    // Facture legacy avec due_date < date, seedée via le repo (la validation
    // est handler-only — le repo accepte, comme les données pré-21-1).
    let (legacy, _lines) = kesh_db::repositories::invoices::create(
        &pool,
        user_id,
        kesh_db::entities::invoice::NewInvoice {
            company_id,
            contact_id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            due_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap()),
            payment_terms: None,
            project_id: None,
            lines: vec![kesh_db::entities::invoice::NewInvoiceLine {
                revenue_account_id: None,
                description: "Legacy".into(),
                quantity: rust_decimal::Decimal::ONE,
                unit_price: rust_decimal::Decimal::new(10000, 2),
                vat_rate: rust_decimal::Decimal::new(810, 2),
            }],
        },
    )
    .await
    .expect("legacy invoice seed");

    // PUT en gardant la paire (date, due_date) identique, autre champ modifié
    // → accepté (règle legacy AC 17).
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{}", legacy.id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-07-01",
            "dueDate": "2026-06-15",
            "paymentTerms": "corrigé",
            "version": legacy.version,
            "lines": [
                {"description": "Legacy", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
            ]
        }))
        .send()
        .await
        .expect("legacy update");
    assert_eq!(
        resp.status(),
        200,
        "paire (date, due_date) inchangée → l'édition des autres champs passe"
    );
    let updated: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(updated["paymentTerms"], "corrigé");

    // PUT qui MODIFIE la paire vers une autre échéance < date → 400.
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/invoices/{}", legacy.id)))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-07-01",
            "dueDate": "2026-06-01",
            "version": updated["version"],
            "lines": [
                {"description": "Legacy", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
            ]
        }))
        .send()
        .await
        .expect("legacy update bad");
    assert_eq!(
        resp.status(),
        400,
        "paire modifiée avec due_date < date → 400"
    );
}
