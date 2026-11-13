//! Story 35-1 (#932, #948) — la numérotation des factures et des avoirs :
//!
//! - **AC2** : `PUT /company/invoice-settings` refuse un format client ou d'avoir
//!   **sans compteur** (`{SEQ}` ou `{SEQ:NN}`) en 400 `VALIDATION_ERROR`, mais
//!   seulement s'il **change** : un format déjà enregistré sans compteur, renvoyé
//!   à l'identique ou omis, ne bloque pas l'enregistrement d'autre chose (D2) ;
//! - **AC3** : un numéro que le format enregistré ne peut pas rendre (plus de 64
//!   caractères, accolade dans le nom de l'exercice) ne finit plus en `500` : la
//!   validation d'une facture et la création d'un avoir rendent **400
//!   `NUMBER_FORMAT_UNRENDERABLE`**, `details { document, cause }`, rien n'est
//!   écrit, le compteur n'avance pas (D4) ;
//! - **AC4** : un numéro déjà attribué (format sans compteur, ou sans exercice sur
//!   deux exercices) rend le même refus, cause `duplicate`, au lieu du générique
//!   `409 RESOURCE_CONFLICT` (D5).
//!
//! Les formats impossibles se posent par SQL direct : la route les refuse
//! désormais, et c'est le cas d'une base enregistrée avant la 0.17.7.
//!
//! Ces tests montent l'application avec le catalogue fr-CH : les messages
//! s'assertent ici. Pré-requis : MariaDB démarré localement.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";
const SETTINGS: &str = "/api/v1/company/invoice-settings";

/// Un nom d'exercice de **50** caractères, la borne de `fiscal_years` (ASCII).
const FY_NAME_50: &str = "Exercice-de-cinquante-caracteres-pour-le-test-3510";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn get(&self, path: &str) -> Value {
        let resp = self
            .client
            .get(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap();
        assert!(resp.status().is_success(), "GET {path} : {}", resp.status());
        resp.json().await.unwrap()
    }

    async fn send(&self, method: reqwest::Method, path: &str, body: Value) -> (u16, Value) {
        let resp = self
            .client
            .request(method, self.url(path))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap();
        (
            status,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
    }

    /// Le corps d'un `PUT` des réglages tel que les lit `GET`, sans le format
    /// d'avoir (le client qui ne l'envoie pas, #216) ; l'appelant y ajoute ce
    /// qu'il change.
    async fn settings_body(&self) -> Value {
        let s = self.get(SETTINGS).await;
        json!({
            "invoiceNumberFormat": s["invoiceNumberFormat"],
            "defaultReceivableAccountId": s["defaultReceivableAccountId"],
            "defaultRevenueAccountId": s["defaultRevenueAccountId"],
            "defaultVatPayableAccountId": s["defaultVatPayableAccountId"],
            "defaultVatRecoverableAccountId": s["defaultVatRecoverableAccountId"],
            "defaultVatDecompteAccountId": s["defaultVatDecompteAccountId"],
            "defaultSalesJournal": s["defaultSalesJournal"],
            "journalEntryDescriptionTemplate": s["journalEntryDescriptionTemplate"],
            "defaultPayableAccountId": s["defaultPayableAccountId"],
            "version": s["version"],
        })
    }

    async fn put_settings(&self, body: Value) -> (u16, Value) {
        self.send(reqwest::Method::PUT, SETTINGS, body).await
    }

    async fn validate(&self, invoice_id: i64) -> (u16, Value) {
        self.send(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{invoice_id}/validate"),
            json!({}),
        )
        .await
    }

    async fn credit_note(&self, invoice_id: i64, date: &str) -> (u16, Value) {
        self.send(
            reqwest::Method::POST,
            "/api/v1/credit-notes",
            json!({ "invoiceId": invoice_id, "date": date }),
        )
        .await
    }
}

async fn login(base_url: &str, client: &reqwest::Client) -> String {
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn spawn_app(pool: &MySqlPool) -> TestApp {
    let config = Config::from_fields_for_test(
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
    );
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
    let state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
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
    let base_url = format!("http://{addr}");
    let client = reqwest::Client::new();
    let token = login(&base_url, &client).await;
    TestApp {
        base_url,
        client,
        token,
    }
}

/// Le montage `seed_accounting_company` (exercice 2020-2030, comptes désignés),
/// l'arrondi à 5 centimes désactivé, et un contact client.
async fn setup(pool: &MySqlPool) -> (TestApp, SeededCompany, i64) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .unwrap();
    // Les réglages de facturation existent avant tout `UPDATE` direct.
    kesh_db::repositories::company_invoice_settings::get_or_create_default(pool, seeded.company_id)
        .await
        .unwrap();
    let app = spawn_app(pool).await;
    let (status, contact) = app
        .send(
            reqwest::Method::POST,
            "/api/v1/contacts",
            json!({
                "contactType": "Entreprise",
                "name": "Client 35-1 SA",
                "isClient": true,
            }),
        )
        .await;
    assert_eq!(status, 201, "contact : {contact}");
    let contact_id = contact["id"].as_i64().unwrap();
    (app, seeded, contact_id)
}

/// Pose un format par SQL direct — le cas d'une base enregistrée avant la
/// 0.17.7, ou d'une écriture hors de la route.
async fn set_format(pool: &MySqlPool, company_id: i64, column: &str, format: &str) {
    let sql = format!("UPDATE company_invoice_settings SET {column} = ? WHERE company_id = ?");
    sqlx::query(&sql)
        .bind(format)
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn stored_format(pool: &MySqlPool, company_id: i64, column: &str) -> String {
    let sql = format!("SELECT {column} FROM company_invoice_settings WHERE company_id = ?");
    sqlx::query_scalar(&sql)
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn rename_fiscal_year(pool: &MySqlPool, fiscal_year_id: i64, name: &str) {
    sqlx::query("UPDATE fiscal_years SET name = ? WHERE id = ?")
        .bind(name)
        .bind(fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Le compteur d'un exercice ; `None` tant qu'aucun numéro n'y a été tiré.
async fn counter(pool: &MySqlPool, table: &str, fiscal_year_id: i64) -> Option<i64> {
    let sql = format!("SELECT next_number FROM {table} WHERE fiscal_year_id = ?");
    sqlx::query_scalar(&sql)
        .bind(fiscal_year_id)
        .fetch_optional(pool)
        .await
        .unwrap()
}

/// Une facture brouillon de 100.00 HT sans TVA, à la date donnée.
async fn draft_invoice(app: &TestApp, contact_id: i64, date: &str) -> i64 {
    let (status, inv) = app
        .send(
            reqwest::Method::POST,
            "/api/v1/invoices",
            json!({
                "contactId": contact_id,
                "date": date,
                "paymentTerms": null,
                "lines": [{
                    "description": "Prestation",
                    "quantity": "1",
                    "unitPrice": "100.00",
                    "vatRate": "0.00",
                }],
            }),
        )
        .await;
    assert_eq!(status, 201, "facture : {inv}");
    inv["id"].as_i64().unwrap()
}

/// Une facture validée (format en place à l'appel) ; rend son id.
async fn validated_invoice(app: &TestApp, contact_id: i64, date: &str) -> i64 {
    let id = draft_invoice(app, contact_id, date).await;
    let (status, body) = app.validate(id).await;
    assert_eq!(status, 200, "validation : {body}");
    id
}

/// Le refus de D4 : 400, le code, `details { document, cause }`, et un message
/// qui nomme le format.
fn assert_unrenderable(status: u16, body: &Value, document: &str, cause: &str, format: &str) {
    assert_eq!(status, 400, "{body}");
    assert_eq!(
        body["error"]["code"], "NUMBER_FORMAT_UNRENDERABLE",
        "{body}"
    );
    assert_eq!(
        body["error"]["details"],
        json!({ "document": document, "cause": cause }),
        "{body}"
    );
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains(&format!("« {format} »")),
        "le message nomme le format : {message}"
    );
}

async fn assert_still_draft(app: &TestApp, invoice_id: i64) {
    let inv = app.get(&format!("/api/v1/invoices/{invoice_id}")).await;
    assert_eq!(inv["status"], "draft", "la facture reste brouillon : {inv}");
    assert!(inv["invoiceNumber"].is_null(), "sans numéro : {inv}");
}

async fn credit_note_count(pool: &MySqlPool, invoice_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM credit_notes WHERE invoice_id = ?")
        .bind(invoice_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// AC2 — la route des réglages
// ---------------------------------------------------------------------------

/// AC2 — un format client changé en `F-{YEAR}` (sans compteur) : 400
/// `VALIDATION_ERROR`, rien d'enregistré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_changed_invoice_format_without_counter_is_refused(pool: MySqlPool) {
    let (app, seeded, _) = setup(&pool).await;
    let mut body = app.settings_body().await;
    body["invoiceNumberFormat"] = json!("F-{YEAR}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 400, "{resp}");
    assert_eq!(resp["error"]["code"], "VALIDATION_ERROR", "{resp}");
    assert!(
        resp["error"]["message"]
            .as_str()
            .unwrap_or_default()
            .contains("compteur"),
        "le refus nomme le compteur : {resp}"
    );
    assert_eq!(
        stored_format(&pool, seeded.company_id, "invoice_number_format").await,
        "F-{YEAR}-{SEQ:04}"
    );
}

/// AC2 — un format d'avoir changé en `AV-{YEAR}` : 400, rien d'enregistré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_changed_credit_note_format_without_counter_is_refused(pool: MySqlPool) {
    let (app, seeded, _) = setup(&pool).await;
    let before = stored_format(&pool, seeded.company_id, "credit_note_number_format").await;
    let mut body = app.settings_body().await;
    body["creditNoteNumberFormat"] = json!("AV-{YEAR}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 400, "{resp}");
    assert_eq!(resp["error"]["code"], "VALIDATION_ERROR", "{resp}");
    assert_eq!(
        stored_format(&pool, seeded.company_id, "credit_note_number_format").await,
        before
    );
}

/// AC2 (D2) — un format d'avoir **enregistré** sans compteur ne bloque pas
/// l'enregistrement d'autre chose : omis (#216) ou renvoyé à l'identique, le
/// `PUT` aboutit ; changé en `AV-{YEAR}-{SEQ}`, il aboutit et l'enregistre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_stored_credit_note_format_without_counter_does_not_block(pool: MySqlPool) {
    let (app, seeded, _) = setup(&pool).await;
    set_format(
        &pool,
        seeded.company_id,
        "credit_note_number_format",
        "AV-{YEAR}",
    )
    .await;

    // Omis : un autre réglage change.
    let mut body = app.settings_body().await;
    body["journalEntryDescriptionTemplate"] = json!("Vente {INVOICE_NUMBER}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "omis : {resp}");

    // Renvoyé à l'identique.
    let mut body = app.settings_body().await;
    body["creditNoteNumberFormat"] = json!("AV-{YEAR}");
    body["journalEntryDescriptionTemplate"] = json!("{YEAR}-{INVOICE_NUMBER}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "à l'identique : {resp}");
    assert_eq!(
        stored_format(&pool, seeded.company_id, "credit_note_number_format").await,
        "AV-{YEAR}"
    );

    // Corrigé.
    let mut body = app.settings_body().await;
    body["creditNoteNumberFormat"] = json!("AV-{YEAR}-{SEQ}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "corrigé : {resp}");
    assert_eq!(
        stored_format(&pool, seeded.company_id, "credit_note_number_format").await,
        "AV-{YEAR}-{SEQ}"
    );
}

/// AC2 (D2) — de même pour le format client, que l'écran envoie toujours : un
/// format enregistré `F-{YEAR}`, renvoyé à l'identique avec un autre réglage
/// modifié, aboutit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac2_stored_invoice_format_without_counter_does_not_block(pool: MySqlPool) {
    let (app, seeded, _) = setup(&pool).await;
    set_format(
        &pool,
        seeded.company_id,
        "invoice_number_format",
        "F-{YEAR}",
    )
    .await;
    let mut body = app.settings_body().await;
    assert_eq!(body["invoiceNumberFormat"], "F-{YEAR}");
    body["journalEntryDescriptionTemplate"] = json!("Vente {INVOICE_NUMBER}");
    let (status, resp) = app.put_settings(body).await;
    assert_eq!(status, 200, "{resp}");
    let s = app.get(SETTINGS).await;
    assert_eq!(
        s["journalEntryDescriptionTemplate"],
        "Vente {INVOICE_NUMBER}"
    );
    assert_eq!(s["invoiceNumberFormat"], "F-{YEAR}");
}

// ---------------------------------------------------------------------------
// AC3 — le 500 devenu refus
// ---------------------------------------------------------------------------

/// AC3 — un exercice nommé sur 50 caractères et `FACT-{FY}-{SEQ:10}` (pire cas
/// jugé 41 ≤ 64, rendu 66 > 64) : 400 `NUMBER_FORMAT_UNRENDERABLE`, cause
/// `too_long` ; la facture reste brouillon, sans numéro ; le compteur n'avance
/// pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_invoice_number_too_long_is_a_typed_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    // Un premier numéro tiré au format par défaut : le compteur existe, à 2.
    validated_invoice(&app, contact, "2026-06-15").await;
    let before = counter(&pool, "invoice_number_sequences", seeded.fiscal_year_id).await;
    assert_eq!(before, Some(2));

    rename_fiscal_year(&pool, seeded.fiscal_year_id, FY_NAME_50).await;
    let format = "FACT-{FY}-{SEQ:10}";
    set_format(&pool, seeded.company_id, "invoice_number_format", format).await;
    let id = draft_invoice(&app, contact, "2026-06-16").await;

    let (status, body) = app.validate(id).await;
    assert_unrenderable(status, &body, "invoice", "too_long", format);
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains(FY_NAME_50),
        "le message nomme l'exercice : {message}"
    );
    assert!(
        message.contains("Administration → Facturation"),
        "le message dit où corriger : {message}"
    );
    assert_still_draft(&app, id).await;
    assert_eq!(
        counter(&pool, "invoice_number_sequences", seeded.fiscal_year_id).await,
        before,
        "le compteur n'a pas avancé"
    );
}

/// AC3 — même refus à la création d'un avoir : `AVOIR-{FY}-{SEQ:10}` sur un
/// exercice de 50 caractères (rendu 67 > 64) ; aucun avoir créé, compteur
/// inchangé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_credit_note_number_too_long_is_a_typed_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let first = validated_invoice(&app, contact, "2026-06-15").await;
    let (status, body) = app.credit_note(first, "2026-06-15").await;
    assert_eq!(status, 201, "premier avoir : {body}");
    let before = counter(&pool, "credit_note_number_sequences", seeded.fiscal_year_id).await;
    assert_eq!(before, Some(2));

    let id = validated_invoice(&app, contact, "2026-06-16").await;
    rename_fiscal_year(&pool, seeded.fiscal_year_id, FY_NAME_50).await;
    let format = "AVOIR-{FY}-{SEQ:10}";
    set_format(
        &pool,
        seeded.company_id,
        "credit_note_number_format",
        format,
    )
    .await;

    let (status, body) = app.credit_note(id, "2026-06-16").await;
    assert_unrenderable(status, &body, "credit_note", "too_long", format);
    assert_eq!(credit_note_count(&pool, id).await, 0, "aucun avoir créé");
    assert_eq!(
        counter(&pool, "credit_note_number_sequences", seeded.fiscal_year_id).await,
        before,
        "le compteur n'a pas avancé"
    );
}

/// AC3 — un nom d'exercice à accolade s'enregistre par la route
/// (`PUT /fiscal-years/{id}` ne le contrôle pas, constaté à l'étape 0) : sous un
/// format qui reprend `{FY}`, la validation rend 400, cause
/// `brace_in_fiscal_year_name`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac3_brace_in_fiscal_year_name_is_a_typed_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let (status, resp) = app
        .send(
            reqwest::Method::PUT,
            &format!("/api/v1/fiscal-years/{}", seeded.fiscal_year_id),
            json!({ "name": "Ex {2026}" }),
        )
        .await;
    assert_eq!(status, 200, "renommage à accolade : {resp}");
    let format = "F-{FY}-{SEQ:04}";
    set_format(&pool, seeded.company_id, "invoice_number_format", format).await;
    let id = draft_invoice(&app, contact, "2026-06-15").await;

    let (status, body) = app.validate(id).await;
    assert_unrenderable(
        status,
        &body,
        "invoice",
        "brace_in_fiscal_year_name",
        format,
    );
    assert_still_draft(&app, id).await;
}

// ---------------------------------------------------------------------------
// AC4 — un numéro déjà attribué (D5)
// ---------------------------------------------------------------------------

/// AC4 — `F-{YEAR}` enregistré : la première facture de l'exercice reçoit
/// `F-2020` (année de début de l'exercice 2020-2030), la seconde rend 400,
/// cause `duplicate`, et reste brouillon. Rouge au départ : `409
/// RESOURCE_CONFLICT`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_invoice_duplicate_number_is_a_typed_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let format = "F-{YEAR}";
    set_format(&pool, seeded.company_id, "invoice_number_format", format).await;
    let first = draft_invoice(&app, contact, "2026-06-15").await;
    let (status, body) = app.validate(first).await;
    assert_eq!(status, 200, "première : {body}");
    assert_eq!(body["invoiceNumber"], "F-2020", "{body}");

    let second = draft_invoice(&app, contact, "2026-06-16").await;
    let (status, body) = app.validate(second).await;
    assert_unrenderable(status, &body, "invoice", "duplicate", format);
    assert_still_draft(&app, second).await;
}

/// AC4 — un avoir sous `AV-{YEAR}` : le premier aboutit, le second rend 400,
/// cause `duplicate` ; aucun second avoir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_credit_note_duplicate_number_is_a_typed_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let a = validated_invoice(&app, contact, "2026-06-15").await;
    let b = validated_invoice(&app, contact, "2026-06-16").await;
    let format = "AV-{YEAR}";
    set_format(
        &pool,
        seeded.company_id,
        "credit_note_number_format",
        format,
    )
    .await;
    let (status, body) = app.credit_note(a, "2026-06-15").await;
    assert_eq!(status, 201, "premier avoir : {body}");

    let (status, body) = app.credit_note(b, "2026-06-16").await;
    assert_unrenderable(status, &body, "credit_note", "duplicate", format);
    assert_eq!(credit_note_count(&pool, b).await, 0, "aucun second avoir");
}

/// AC4 — `FACT{SEQ}`, accepté par D1, sur deux exercices : le compteur repart à
/// 1 au second, dont la première facture reçoit un numéro déjà attribué → 400,
/// cause `duplicate` (#1058).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac4_format_without_fiscal_year_collides_on_second_fiscal_year(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let format = "FACT{SEQ}";
    set_format(&pool, seeded.company_id, "invoice_number_format", format).await;

    let first = draft_invoice(&app, contact, "2025-06-15").await;
    let (status, body) = app.validate(first).await;
    assert_eq!(status, 200, "premier exercice : {body}");
    assert_eq!(body["invoiceNumber"], "FACT1", "{body}");

    let second = draft_invoice(&app, contact, "2026-06-15").await;
    let (status, body) = app.validate(second).await;
    assert_unrenderable(status, &body, "invoice", "duplicate", format);
    assert_still_draft(&app, second).await;
}
