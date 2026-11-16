//! Story 33-6 (#450) — la réponse d'un règlement dit ce qui a été enregistré,
//! sans seconde lecture après le commit.
//!
//! ⚠️ **Le 404 d'origine n'est pas reproductible ici** : sa fenêtre tient entre
//! deux requêtes SQL, et aucun crochet de test n'existe en production pour la
//! forcer (C-e334-10). La garantie repose donc sur deux preuves : **AC-3**, la
//! réponse égale champ à champ un `GET` qui la suit, et **AC-2**, une garde de
//! structure, lue par `syn` : rien ne s'attend après l'écriture dans les trois
//! handlers, ni après le commit dans les fonctions qui écrivent.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewInvoice, NewInvoiceLine, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::{invoices, journal_entries};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

#[path = "common/autour_appel.rs"]
mod autour_appel;

/// Story 36-4 (#1131) — de `tests/common/`, le relevé « toutes traces » d'un geste
/// et la clé par HTTP.
mod common {
    pub mod audit_traces;
}

use autour_appel::{AroundCall, around};
use common::audit_traces::{self, assert_toutes_a_l_utilisateur, assert_toutes_a_la_cle};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
    caisse: i64,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn post(&self, path: &str, body: Option<Value>) -> Value {
        self.post_as(&self.token, path, body).await
    }

    /// Le `POST` de [`Self::post`] sous un autre jeton — une clé d'API (Story
    /// 36-4, #1131).
    async fn post_as(&self, token: &str, path: &str, body: Option<Value>) -> Value {
        let mut req = self.client.post(self.url(path)).bearer_auth(token);
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        assert_eq!(status, 200, "{path} : {body}");
        body
    }

    async fn get(&self, path: &str) -> Value {
        let resp = self
            .client
            .get(self.url(path))
            .bearer_auth(&self.token)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "{path}");
        resp.json().await.unwrap()
    }

    /// Règle `amount` à la version que la facture porte au moment de l'appel
    /// (Story 34-5, #951 : la version est exigée).
    async fn settle(&self, id: i64, amount: &str) -> Value {
        let version = self.get(&format!("/api/v1/invoices/{id}")).await["version"].clone();
        let (status, body) = self.settle_with(id, amount, Some(version)).await;
        assert_eq!(status, 200, "règlement : {body}");
        body
    }

    /// Le `POST` d'un règlement tel quel, `version` comprise ou omise, et sa
    /// réponse brute : statut et corps (Story 34-5).
    async fn settle_with(&self, id: i64, amount: &str, version: Option<Value>) -> (u16, Value) {
        settle_request(
            &self.client,
            &self.url(&format!("/api/v1/invoices/{id}/settlements")),
            &self.token,
            self.caisse,
            amount,
            version,
        )
        .await
    }

    /// ⛔ AC-3 : la facture rendue par le geste égale **entièrement** celle qu'un
    /// `GET` lit juste après — `version`, `paidAt`, `amountSettled`, `amountDue`,
    /// `updatedAt`, lignes comprises. Une version périmée ferait échouer le geste
    /// suivant en 409 ; un réglé d'avant le geste mentirait à l'écran.
    async fn assert_response_is_what_was_written(&self, id: i64, response: &Value) {
        let read = self.get(&format!("/api/v1/invoices/{id}")).await;
        assert_eq!(
            &response["invoice"], &read,
            "la réponse doit égaler le GET qui la suit"
        );
        assert!(
            !read["lines"].as_array().unwrap().is_empty(),
            "montage : la facture porte des lignes"
        );
    }
}

/// Le `POST` d'un règlement par la caisse, hors de `TestApp` pour partir d'une
/// tâche à part (AC-3 de la 34-5) : rend le statut et le corps, sans rien
/// affirmer. Sans `version`, le corps l'omet.
async fn settle_request(
    client: &reqwest::Client,
    url: &str,
    token: &str,
    caisse: i64,
    amount: &str,
    version: Option<Value>,
) -> (u16, Value) {
    let mut body = json!({
        "settlementType": "internal_account",
        "accountId": caisse,
        "amount": amount,
        "settledOn": "2026-04-15",
    });
    if let Some(version) = version {
        body["version"] = version;
    }
    let resp = client
        .post(url)
        .bearer_auth(token)
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

async fn spawn_app(pool: &MySqlPool, seeded: &SeededCompany) -> TestApp {
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
    let login: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    TestApp {
        base_url,
        client,
        token: login["accessToken"].as_str().unwrap().to_string(),
        caisse: seeded.accounts["1000"],
    }
}

/// Une facture validée de `amount`, sans TVA, avec son écriture de vente posée à
/// la main (patron de `kesh-db/tests/invoice_settlement.rs`) : c'est ce qui permet
/// un total brut hors centime, celui de l'écart d'arrondi.
async fn validated_invoice(pool: &MySqlPool, seeded: &SeededCompany, amount: Decimal) -> i64 {
    let date = NaiveDate::from_ymd_opt(2026, 4, 1).unwrap();
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client 33-6', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let invoice = invoices::create(
        pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id,
            date,
            due_date: Some(date),
            payment_terms: None,
            project_id: None,
            lines: vec![NewInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: amount,
                vat_rate: dec!(0),
                revenue_account_id: Some(seeded.accounts["3000"]),
            }],
        },
    )
    .await
    .unwrap()
    .0;
    let entry = journal_entries::create(
        pool,
        seeded.fiscal_year_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(seeded.admin_user_id),
        NewJournalEntry {
            company_id: seeded.company_id,
            entry_date: date,
            journal: Journal::Ventes,
            description: "Vente".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seeded.accounts["1100"],
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                    foreign: None,
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["3000"],
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                    foreign: None,
                },
            ],
        },
    )
    .await
    .unwrap();
    sqlx::query("UPDATE invoices SET status = 'validated', journal_entry_id = ? WHERE id = ?")
        .bind(entry.entry.id)
        .bind(invoice.id)
        .execute(pool)
        .await
        .unwrap();
    invoice.id
}

async fn setup(pool: &MySqlPool, amount: Decimal) -> (TestApp, i64) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    let id = validated_invoice(pool, &seeded, amount).await;
    (spawn_app(pool, &seeded).await, id)
}

fn decimal(value: &Value) -> Decimal {
    value
        .as_str()
        .unwrap_or_else(|| panic!("montant attendu : {value}"))
        .parse()
        .unwrap()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_partial_settlement_returns_what_was_written(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    let before = app.get(&format!("/api/v1/invoices/{id}")).await;

    let response = app.settle(id, "40.00").await;

    app.assert_response_is_what_was_written(id, &response).await;
    let invoice = &response["invoice"];
    assert_eq!(decimal(&invoice["amountSettled"]), dec!(40.00));
    assert_eq!(decimal(&invoice["amountDue"]), dec!(60.00));
    assert_eq!(
        invoice["version"].as_i64().unwrap(),
        before["version"].as_i64().unwrap() + 1,
        "la version rendue est celle d'après le geste"
    );
    assert!(invoice["paidAt"].is_null());
    assert_eq!(response["fullySettled"], false);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_full_settlement_returns_what_was_written(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    app.settle(id, "40.00").await;

    let response = app.settle(id, "60.00").await;

    app.assert_response_is_what_was_written(id, &response).await;
    let invoice = &response["invoice"];
    assert_eq!(decimal(&invoice["amountSettled"]), dec!(100.00));
    assert_eq!(decimal(&invoice["amountDue"]), dec!(0));
    assert!(!invoice["paidAt"].is_null(), "soldée, donc payée");
    assert_eq!(response["fullySettled"], true);
}

/// Un paiement au centime d'un reste brut hors centime : l'écart passe au compte
/// d'arrondi, et le réglé rendu est le brut, non le montant payé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_settlement_with_a_rounding_gap_returns_what_was_written(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    designate_rounding_account(&pool, seeded.company_id)
        .await
        .unwrap();
    let id = validated_invoice(&pool, &seeded, dec!(10.0050)).await;
    let app = spawn_app(&pool, &seeded).await;

    let response = app.settle(id, "10.01").await;

    app.assert_response_is_what_was_written(id, &response).await;
    assert_eq!(response["fullySettled"], true);
    assert_eq!(
        decimal(&response["invoice"]["amountSettled"]),
        dec!(10.0050)
    );
    // Le montage exerce bien la branche d'écart : l'écriture porte trois lignes.
    let lines: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?")
            .bind(response["journalEntryId"].as_i64().unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(lines, 3, "caisse, créance, arrondi");
}

/// Une facture de 100.00 réglée de 90.00, puis **soldée** de son reste (10.00,
/// escompte) : rend l'application, la facture et la réponse du solde. Extrait de
/// `a_write_off_returns_what_was_written` par la Story 36-4 (#1131), qui
/// annule ce solde par une clé.
async fn setup_written_off(pool: &MySqlPool) -> (TestApp, i64, Value) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["4000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let id = validated_invoice(pool, &seeded, dec!(100.00)).await;
    let app = spawn_app(pool, &seeded).await;
    let settled = app.settle(id, "90.00").await;

    let response = app
        .post(
            &format!("/api/v1/invoices/{id}/write-off"),
            Some(json!({
                "nature": "discount",
                "settledOn": "2026-04-16",
                "version": settled["invoice"]["version"],
            })),
        )
        .await;
    (app, id, response)
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_write_off_returns_what_was_written(pool: MySqlPool) {
    let (app, id, response) = setup_written_off(&pool).await;

    app.assert_response_is_what_was_written(id, &response).await;
    let invoice = &response["invoice"];
    assert_eq!(decimal(&response["amount"]), dec!(10.00));
    assert_eq!(
        decimal(&invoice["amountDue"]),
        dec!(0),
        "le reste est soldé"
    );
    assert_eq!(decimal(&invoice["amountSettled"]), dec!(100.00));
    assert!(!invoice["paidAt"].is_null());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_cancellation_returns_what_was_written(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    let settled = app.settle(id, "100.00").await;
    assert!(!settled["invoice"]["paidAt"].is_null(), "montage");
    let list = app.get(&format!("/api/v1/invoices/{id}/settlements")).await;
    let settlement_id = list.as_array().unwrap()[0]["id"].as_i64().unwrap();
    assert!(
        audit_traces::lignes_lettrees_document(&pool).await > 0,
        "montage : le lettrage de la pièce est posé"
    );
    let avant = audit_traces::max_audit_id(&pool).await;

    let response = app
        .post(
            &format!("/api/v1/invoices/{id}/settlements/{settlement_id}/cancel"),
            None,
        )
        .await;

    app.assert_response_is_what_was_written(id, &response).await;
    let invoice = &response["invoice"];
    assert_eq!(decimal(&invoice["amountSettled"]), dec!(0));
    assert_eq!(decimal(&invoice["amountDue"]), dec!(100.00));
    assert!(invoice["paidAt"].is_null(), "la facture est rouverte");
    assert_eq!(
        invoice["version"].as_i64().unwrap(),
        settled["invoice"]["version"].as_i64().unwrap() + 1
    );

    // Story 36-4 (#1131, AC4) — par session, rien ne change : toutes les traces
    // du geste sont à l'utilisateur, sans clé.
    assert_toutes_a_l_utilisateur(
        &audit_traces::audit_depuis(&pool, avant).await,
        audit_traces::user_id_de(&pool, "admin").await,
        INVOICE_SETTLEMENT_CANCEL_ACTIONS,
    );
}

/// Ce qu'écrit au journal d'audit l'annulation d'un règlement client en caisse
/// qui avait soldé la facture — relevé sur ce montage (Story 36-4, #1131) : le
/// délettrage de la pièce, la contre-passation (l'inverse, le marquage de
/// l'écriture, et un lettrage `reversal` par ligne libre d'un compte lettrable —
/// ici deux : la caisse `1000` et la créance, dont le lettrage de la pièce vient
/// d'être défait), le geste.
const INVOICE_SETTLEMENT_CANCEL_ACTIONS: &[&str] = &[
    "lettering.removed",
    "journal_entry.created",
    "journal_entry.reversed",
    "lettering.created",
    "lettering.created",
    "invoice.settlement_cancelled",
];

/// Ce qu'écrit l'annulation du **solde** de `setup_written_off` — relevé sur ce
/// montage (Story 36-4, #1131) : mêmes étapes, un seul lettrage `reversal`, la
/// contrepartie du solde (escompte, `4000`) n'étant pas lettrable.
const WRITE_OFF_CANCEL_ACTIONS: &[&str] = &[
    "lettering.removed",
    "journal_entry.created",
    "journal_entry.reversed",
    "lettering.created",
    "invoice.settlement_cancelled",
];

/// L'annulation, par la clé `secret`, de la ligne de règlement de la facture
/// `id` dont le type est `settlement_type`, après avoir relevé que la pièce est
/// lettrée (`piece_lettree`) ou ne l'est pas (règlement partiel) ; rend
/// l'identifiant d'audit d'avant le geste (Story 36-4, #1131).
async fn cancel_by_key(
    app: &TestApp,
    pool: &MySqlPool,
    secret: &str,
    id: i64,
    settlement_type: &str,
    piece_lettree: bool,
) -> i64 {
    let list = app.get(&format!("/api/v1/invoices/{id}/settlements")).await;
    let settlement_id = list
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["settlementType"] == settlement_type)
        .unwrap_or_else(|| panic!("montage : une ligne {settlement_type} dans {list}"))["id"]
        .as_i64()
        .unwrap();
    assert_eq!(
        audit_traces::lignes_lettrees_document(pool).await > 0,
        piece_lettree,
        "montage : le lettrage de la pièce (le délettrage n'est exercé que s'il est posé)"
    );
    let avant = audit_traces::max_audit_id(pool).await;
    app.post_as(
        secret,
        &format!("/api/v1/invoices/{id}/settlements/{settlement_id}/cancel"),
        None,
    )
    .await;
    avant
}

/// ⛔ Story 36-4 (#1131, AC1) — **l'annulation d'un règlement client par une clé
/// d'API est attribuée à la clé**, toutes traces comprises : le geste, la
/// contre-passation et le délettrage de la pièce. Avant la story, chacune était
/// au nom du créateur de la clé (`actor_type = 'user'`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_cancels_an_invoice_settlement_and_is_named_everywhere(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    app.settle(id, "100.00").await;
    let (cle, secret) = audit_traces::create_key_via_http(
        &app.client,
        &app.base_url,
        &app.token,
        "rw",
        "read-write",
    )
    .await;

    let avant = cancel_by_key(&app, &pool, &secret, id, "internal_account", true).await;

    assert_toutes_a_la_cle(
        &audit_traces::audit_depuis(&pool, avant).await,
        cle,
        audit_traces::createur_de_la_cle(&pool, cle).await,
        INVOICE_SETTLEMENT_CANCEL_ACTIONS,
    );
}

/// ⛔ Story 36-4 (#1131, AC1, cas du solde) — la même route annule un **solde**
/// (`settlementType = write_off`) : ses traces portent la clé de même. Le chemin
/// des traces ne branche pas sur le type ; ce test tient la promesse de la note
/// de version (« un règlement ou un solde »).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_cancels_a_write_off_and_is_named_everywhere(pool: MySqlPool) {
    let (app, id, _) = setup_written_off(&pool).await;
    let (cle, secret) = audit_traces::create_key_via_http(
        &app.client,
        &app.base_url,
        &app.token,
        "rw",
        "read-write",
    )
    .await;

    let avant = cancel_by_key(&app, &pool, &secret, id, "write_off", true).await;

    assert_toutes_a_la_cle(
        &audit_traces::audit_depuis(&pool, avant).await,
        cle,
        audit_traces::createur_de_la_cle(&pool, cle).await,
        WRITE_OFF_CANCEL_ACTIONS,
    );
}

/// Ce qu'écrit l'annulation d'un règlement **partiel** en caisse (40.00 sur
/// 100.00) : aucun lettrage de pièce, donc aucun `lettering.removed` ; la
/// contre-passation lettre les deux lignes libres (caisse `1000`, créance) —
/// relevé sur ce montage (Story 36-4, revue P1 E2).
const PARTIAL_SETTLEMENT_CANCEL_ACTIONS: &[&str] = &[
    "journal_entry.created",
    "journal_entry.reversed",
    "lettering.created",
    "lettering.created",
    "invoice.settlement_cancelled",
];

/// ⛔ Story 36-4 (#1131, revue P1 E2) — l'annulation par clé d'un règlement
/// **partiel**, sans lettrage de pièce à défaire : ses traces portent la clé, et
/// la liste ne contient pas `lettering.removed`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_api_key_cancels_a_partial_settlement_and_is_named_everywhere(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    app.settle(id, "40.00").await;
    let (cle, secret) = audit_traces::create_key_via_http(
        &app.client,
        &app.base_url,
        &app.token,
        "rw",
        "read-write",
    )
    .await;

    let avant = cancel_by_key(&app, &pool, &secret, id, "internal_account", false).await;

    assert_toutes_a_la_cle(
        &audit_traces::audit_depuis(&pool, avant).await,
        cle,
        audit_traces::createur_de_la_cle(&pool, cle).await,
        PARTIAL_SETTLEMENT_CANCEL_ACTIONS,
    );
}

/// Revue P1 (E-11) : deux règlements dont un annulé, reste partiel des deux côtés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_partial_cancellation_returns_what_was_written(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(100.00)).await;
    app.settle(id, "40.00").await;
    app.settle(id, "60.00").await;
    let list = app.get(&format!("/api/v1/invoices/{id}/settlements")).await;
    let last = list.as_array().unwrap().last().unwrap()["id"]
        .as_i64()
        .unwrap();

    let response = app
        .post(
            &format!("/api/v1/invoices/{id}/settlements/{last}/cancel"),
            None,
        )
        .await;

    app.assert_response_is_what_was_written(id, &response).await;
    let invoice = &response["invoice"];
    assert_eq!(decimal(&invoice["amountSettled"]), dec!(40.00));
    assert_eq!(decimal(&invoice["amountDue"]), dec!(60.00));
    assert!(invoice["paidAt"].is_null());
}

// ---------------------------------------------------------------------------
// Story 34-5 (#951) — un second envoi du même règlement n'écrit rien
// ---------------------------------------------------------------------------

/// Ce qu'un règlement laisse en base : lignes `invoice_settlements`, écritures
/// de règlement distinctes, entrées d'audit `invoice.partially_settled`.
async fn settlement_traces(pool: &MySqlPool, id: i64) -> (i64, i64, i64) {
    let rows: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(id)
            .fetch_one(pool)
            .await
            .unwrap();
    let entries: i64 = sqlx::query_scalar(
        "SELECT COUNT(DISTINCT journal_entry_id) FROM invoice_settlements WHERE invoice_id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap();
    let audits: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'invoice' AND entity_id = ? \
         AND action = 'invoice.partially_settled'",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap();
    (rows, entries, audits)
}

/// Le seul règlement écrit, et le reste dû de 200 — ce que l'AC-2 et l'AC-3 exigent
/// après deux envois du même règlement de 100 sur 300.
async fn assert_one_settlement_of_100(app: &TestApp, pool: &MySqlPool, id: i64) {
    assert_eq!(
        settlement_traces(pool, id).await,
        (1, 1, 1),
        "une ligne, une écriture de règlement, une entrée d'audit"
    );
    let entries_in_journal: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entries je \
         JOIN invoice_settlements s ON s.journal_entry_id = je.id WHERE s.invoice_id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap();
    assert_eq!(entries_in_journal, 1, "une seule écriture au journal");
    let read = app.get(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(decimal(&read["amountDue"]), dec!(200.00));
}

/// AC-1 : un corps sans `version` est refusé par l'extracteur, `422`, et rien
/// n'est écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_settlement_without_version_is_refused_by_the_extractor(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(300.00)).await;
    let before = app.get(&format!("/api/v1/invoices/{id}")).await;

    let (status, body) = app.settle_with(id, "100.00", None).await;

    assert_eq!(status, 422, "corps sans version : {body}");
    assert_eq!(settlement_traces(&pool, id).await, (0, 0, 0));
    let after = app.get(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(after["version"], before["version"], "version inchangée");
}

/// ⛔ AC-2, le défaut de #951 : deux envois **identiques**, même version, d'un
/// règlement partiel de 100 sur 300. Le premier rend 200, le second 409, et un
/// seul règlement est écrit. Avant la 34-5, la route ignorait la version et
/// rendait 200 deux fois : deux règlements, deux écritures, un reste dû de 100.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_same_settlement_sent_twice_is_written_once(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(300.00)).await;
    let version = app.get(&format!("/api/v1/invoices/{id}")).await["version"].clone();

    let (first, first_body) = app.settle_with(id, "100.00", Some(version.clone())).await;
    let (second, second_body) = app.settle_with(id, "100.00", Some(version)).await;

    assert_eq!(first, 200, "premier envoi : {first_body}");
    assert_eq!(second, 409, "second envoi, même version : {second_body}");
    assert_eq!(second_body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_one_settlement_of_100(&app, &pool, id).await;
}

/// ⛔ AC-3 : deux envois **simultanés** du même règlement, même version.
///
/// Montage déterministe (patron `victime`, `rejeu_interblocage_e2e.rs`) : une
/// transaction de test tient la facture ; les deux requêtes partent et sont vues
/// **toutes deux** en attente sur le `FOR UPDATE` de l'étape (1) ; la transaction
/// de test est annulée. Une comparaison faite sous le verrou rend un 200 et un
/// 409 ; une comparaison faite avant lui, par une lecture ordinaire (M2), lit deux
/// fois l'ancienne version et rend deux 200 — les montants (100 sur 300) laissent
/// passer le trop-perçu, qui ne peut donc pas masquer le défaut.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn two_simultaneous_identical_settlements_write_one(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(300.00)).await;
    let version = app.get(&format!("/api/v1/invoices/{id}")).await["version"].clone();

    let mut holder = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM invoices WHERE id = ? FOR UPDATE")
        .bind(id)
        .fetch_all(&mut *holder)
        .await
        .unwrap();

    let url = app.url(&format!("/api/v1/invoices/{id}/settlements"));
    let spawn = |v: Value| {
        let (client, url, token, caisse) = (
            app.client.clone(),
            url.clone(),
            app.token.clone(),
            app.caisse,
        );
        tokio::spawn(async move {
            settle_request(&client, &url, &token, caisse, "100.00", Some(v)).await
        })
    };
    let a = spawn(version.clone());
    let b = spawn(version);

    let both_waiting = kesh_db::test_fixtures::attendre_des_requetes_en_cours(
        &pool,
        &["FROM invoices", "journal_entry_id", "FOR UPDATE"],
        2,
        || a.is_finished() || b.is_finished(),
    )
    .await;
    assert!(
        both_waiting,
        "les deux envois devaient attendre le verrou de la facture — le montage ne prouve rien sinon"
    );
    holder.rollback().await.unwrap();

    let (ra, rb) = (a.await.unwrap(), b.await.unwrap());
    let mut statuses = [ra.0, rb.0];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 409], "un 200 et un 409 : {ra:?} / {rb:?}");
    let refused = if ra.0 == 409 { &ra.1 } else { &rb.1 };
    assert_eq!(refused["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_one_settlement_of_100(&app, &pool, id).await;
}

/// Revue P1 (E1-3) : une facture inconnue rend `404`, et non `409`, quelle que soit la
/// version envoyée — un conflit dirait au client que la facture existe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_unknown_invoice_is_not_found_whatever_the_version(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(300.00)).await;
    for version in [0, 1, 99] {
        let (status, body) = app
            .settle_with(id + 9999, "100.00", Some(json!(version)))
            .await;
        assert_eq!(status, 404, "version {version} : {body}");
        assert_eq!(body["error"]["code"], "NOT_FOUND");
    }
    assert_eq!(settlement_traces(&pool, id).await, (0, 0, 0));
}

/// AC-6 : la réponse porte la version incrémentée, et un règlement qui la reprend
/// passe — deux acomptes successifs depuis le même écran restent possibles. Le
/// refus du trop-perçu borne toujours la somme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_settlement_at_the_returned_version_is_accepted(pool: MySqlPool) {
    let (app, id) = setup(&pool, dec!(300.00)).await;
    let version = app.get(&format!("/api/v1/invoices/{id}")).await["version"].clone();

    let (s1, r1) = app.settle_with(id, "100.00", Some(version.clone())).await;
    assert_eq!(s1, 200, "{r1}");
    assert_eq!(
        r1["invoice"]["version"].as_i64().unwrap(),
        version.as_i64().unwrap() + 1
    );
    let (s2, r2) = app
        .settle_with(id, "100.00", Some(r1["invoice"]["version"].clone()))
        .await;
    assert_eq!(s2, 200, "second acompte, à la version rendue : {r2}");
    assert_eq!(decimal(&r2["invoice"]["amountDue"]), dec!(100.00));

    let (s3, r3) = app
        .settle_with(id, "150.00", Some(r2["invoice"]["version"].clone()))
        .await;
    assert_eq!(s3, 400, "le trop-perçu reste refusé, version à jour : {r3}");
}

// ---------------------------------------------------------------------------
// AC-2, AC-8 — rien ne s'attend après l'écriture, ni après le commit
// ---------------------------------------------------------------------------

// Le visiteur `syn` (`AroundCall`, `around`) et ses limites écrites vivent dans
// `common/autour_appel.rs` depuis la Story 34-8 (#938, C-e343-11), qui l'emploie
// aussi pour les factures fournisseur. Les assertions ci-dessous n'ont pas changé.

const HANDLERS: &str = include_str!("../src/routes/invoices.rs");
const WRITES: &str = include_str!("../../kesh-db/src/repositories/invoice_settlements_write.rs");
const SENDS: &str = include_str!("../src/routes/mcp_envois.rs");

/// Les handlers ne relisent rien : après l'écriture, plus aucune attente.
#[test]
fn the_three_handlers_wait_for_nothing_after_the_write() {
    for (handler, write) in [
        ("settle_invoice_handler", "settle_invoice"),
        ("write_off_invoice_handler", "write_off_invoice"),
        ("cancel_invoice_settlement_handler", "cancel_settlement"),
    ] {
        assert_eq!(
            around(HANDLERS, handler, write, None).awaits_after,
            0,
            "{handler} attend encore après l'écriture"
        );
    }
}

/// « Pas de commit sans réponse » : la réponse est lue avant le commit, et rien ne
/// s'attend après lui, aux trois fonctions d'écriture et à la décision d'un envoi.
#[test]
fn the_response_is_read_before_the_commit_and_nothing_waits_after_it() {
    for (source, function, read) in [
        (WRITES, "settle_invoice", "read_after_write_in_tx"),
        (WRITES, "write_off_invoice", "read_after_write_in_tx"),
        (WRITES, "cancel_settlement", "read_after_write_in_tx"),
        (SENDS, "decider_envoi", "trouver_in_tx"),
    ] {
        assert_eq!(
            around(source, function, "commit", Some(read)),
            AroundCall {
                awaits_after: 0,
                read_before: true,
            },
            "{function}"
        );
    }
}

/// ⛔ La sonde positive passe par le même découpage que la garde : sans elle, une
/// lecture cassée la rendrait verte à vide.
#[test]
fn the_guard_sees_every_form_of_reread() {
    let write = "async fn f() { let a = g().await?; write(x).await?; REREAD }";
    for (form, reread) in [
        (
            "la relecture d'origine",
            "let b = find(&state.pool, 1).await?;",
        ),
        (
            "le service",
            "let c = factures::detail(&ctx, &state.lecture(), 1).await?;",
        ),
        ("une autre fin", "let d = find(1).await.map_err(boom)?;"),
        ("sans point-virgule", "h().await"),
    ] {
        let source = write.replace("REREAD", reread);
        assert_eq!(
            around(&source, "f", "write", None).awaits_after,
            1,
            "{form}"
        );
    }
    assert_eq!(
        around(&write.replace("REREAD", "Ok(1)"), "f", "write", None).awaits_after,
        0,
        "ce qui précède l'écriture ne compte pas, et le rendu non plus"
    );

    let committed = "async fn f() { read(1).await?; tx.commit().await?; Ok(1) }";
    assert_eq!(
        around(committed, "f", "commit", Some("read")),
        AroundCall {
            awaits_after: 0,
            read_before: true
        }
    );
    let late = "async fn f() { tx.commit().await?; read(1).await?; Ok(1) }";
    assert_eq!(
        around(late, "f", "commit", Some("read")),
        AroundCall {
            awaits_after: 1,
            read_before: false
        }
    );
    // L'autre bras du `match`, celui du `rollback`, n'est pas « après le commit ».
    let arms = "async fn f() { let d = read(1).await; match d {
        Ok(d) => { tx.commit().await?; Ok(d) }
        Err(e) => { tx.rollback().await; Err(e) }
    } }";
    assert_eq!(
        around(arms, "f", "commit", Some("read")),
        AroundCall {
            awaits_after: 0,
            read_before: true
        }
    );
}

/// Revue P2 (P2-1) — l'attente qui n'est pas dans une instruction suivant l'appel :
/// le bras d'un `match` sur l'appel, un `if let`, et la même instruction.
#[test]
fn the_guard_sees_a_wait_in_a_match_arm_or_the_same_statement() {
    for (form, source) in [
        (
            "le bras d'un match sur l'appel",
            "async fn f() { match tx.commit().await {
                Ok(()) => { read(1).await?; Ok(1) }
                Err(e) => Err(e),
            } }",
        ),
        (
            "le bras d'un if let sur l'appel",
            "async fn f() { if let Ok(()) = tx.commit().await { read(1).await } else { Ok(1) } }",
        ),
        (
            "la même instruction",
            "async fn f() { Ok(Json(read(tx.commit().await?).await?)) }",
        ),
    ] {
        assert_eq!(
            around(source, "f", "commit", None).awaits_after,
            1,
            "{form}"
        );
    }
}
