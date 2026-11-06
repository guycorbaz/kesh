//! Story 33-6 (#450) — la réponse d'un règlement dit ce qui a été enregistré,
//! sans seconde lecture après le commit.
//!
//! ⚠️ **Le 404 d'origine n'est pas reproductible ici** : sa fenêtre tient entre
//! deux requêtes SQL, et aucun crochet de test n'existe en production pour la
//! forcer (C-e334-10). La garantie repose donc sur deux preuves : **AC-3**, la
//! réponse égale champ à champ un `GET` qui la suit, et **AC-2**, la garde
//! lexicale qui interdit la relecture dans les trois handlers.

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
        let mut req = self.client.post(self.url(path)).bearer_auth(&self.token);
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

    async fn settle(&self, id: i64, amount: &str) -> Value {
        self.post(
            &format!("/api/v1/invoices/{id}/settlements"),
            Some(json!({
                "settlementType": "internal_account",
                "accountId": self.caisse,
                "amount": amount,
                "settledOn": "2026-04-15",
            })),
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
        seeded.admin_user_id,
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
                },
                NewJournalEntryLine {
                    account_id: seeded.accounts["3000"],
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_write_off_returns_what_was_written(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_discount_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["4000"])
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let id = validated_invoice(&pool, &seeded, dec!(100.00)).await;
    let app = spawn_app(&pool, &seeded).await;
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
}

// ---------------------------------------------------------------------------
// AC-2 — la garde lexicale
// ---------------------------------------------------------------------------

/// Le corps d'une fonction de `routes/invoices.rs`, depuis `pub async fn <nom>(`
/// jusqu'à la fonction suivante du fichier.
fn body_of<'a>(source: &'a str, name: &str) -> &'a str {
    let marker = format!("pub async fn {name}(");
    let start = source
        .find(&marker)
        .unwrap_or_else(|| panic!("fonction introuvable : {name}"));
    let rest = &source[start + marker.len()..];
    let end = rest.find("\n}\n").expect("fin de fonction");
    &rest[..end]
}

const FORBIDDEN: [&str; 3] = ["find_by_id_with_lines", "amount_settled(", "amount_due("];

/// Ce qui suit l'écriture ne relit rien : ni la facture, ni les montants, ni le
/// pool. Une relecture y rendrait un 404 ou un 500 sur un règlement déjà commité.
#[test]
fn the_three_handlers_read_nothing_after_the_write() {
    let source = include_str!("../src/routes/invoices.rs");
    for (handler, write_call) in [
        ("settle_invoice_handler", "settle_invoice("),
        ("write_off_invoice_handler", "write_off_invoice("),
        ("cancel_invoice_settlement_handler", "cancel_settlement("),
    ] {
        let body = body_of(source, handler);
        let at = body
            .find(write_call)
            .unwrap_or_else(|| panic!("{handler} : appel d'écriture introuvable"));
        // L'appel d'écriture se referme sur son `retry_on_deadlock` : `.await?;`.
        let after = &body[at + body[at..].find(".await?;").expect("fin de l'appel")..];
        for token in FORBIDDEN.iter().chain(["state.pool"].iter()) {
            assert!(
                !after.contains(token),
                "{handler} relit après le commit : `{token}`"
            );
        }
    }
}

/// ⛔ La sonde positive : sans elle, une extraction cassée rendrait la garde
/// verte à vide. Une relecture écrite dans un corps doit être trouvée par les mêmes
/// jetons, et les corps réels ne doivent pas être tronqués.
#[test]
fn the_guard_sees_a_real_read_in_the_same_file() {
    let source = include_str!("../src/routes/invoices.rs");
    assert!(
        source.contains("find_by_id_with_lines"),
        "sonde : le fichier porte encore des lectures de la facture"
    );
    let probe = "\npub async fn sonde(\n) {\n    let x = 1.await?;\n    find_by_id_with_lines(&state.pool);\n}\n";
    let body = body_of(probe, "sonde");
    assert!(body.contains(FORBIDDEN[0]) && body.contains("state.pool"));
    let lengths: Vec<usize> = [
        "settle_invoice_handler",
        "write_off_invoice_handler",
        "cancel_invoice_settlement_handler",
    ]
    .iter()
    .map(|h| body_of(source, h).len())
    .collect();
    assert!(
        lengths.iter().all(|l| *l > 200),
        "extraction trop courte : {lengths:?}"
    );
}
