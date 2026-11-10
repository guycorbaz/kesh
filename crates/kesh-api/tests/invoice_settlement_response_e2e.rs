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

use autour_appel::{AroundCall, around};

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
