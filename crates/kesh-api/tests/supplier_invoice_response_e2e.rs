//! Story 34-8 (#938) — la réponse des neuf gestes sur une facture fournisseur dit
//! ce qui a été écrit, sans relecture après le commit.
//!
//! ⚠️ **Le 404 ou le 500 d'origine n'est pas reproductible ici** : sa fenêtre tient
//! entre deux requêtes SQL, et aucun crochet de test n'existe en production pour
//! la forcer (C-e334-10, repris par la fiche). La garantie repose sur deux preuves,
//! comme à la 33-6 : **AC-4**, la réponse égale champ à champ un `GET` qui la suit,
//! et **AC-3**, une garde de structure lue par `syn` — rien ne s'attend après
//! l'écriture dans les neuf handlers, et les neuf fonctions qui écrivent terminent
//! leur transaction par `commit_with_view`, qui lit la vue avant de valider.
//!
//! Décor synthétique : `seed_accounting_company`, `fournisseur_rapproche`.
//! Pré-requis : MariaDB démarré.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

#[path = "common/autour_appel.rs"]
mod autour_appel;
#[path = "../../kesh-db/tests/support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use autour_appel::{AroundCall, around, fn_calls};
use fournisseur_rapproche::{Fournisseur, document};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

// ---------------------------------------------------------------------------
// Montage HTTP (patron des tests `*_e2e.rs` : chaque binaire est un crate)
// ---------------------------------------------------------------------------

struct Ctx {
    seeded: SeededCompany,
    f: Fournisseur,
    base_url: String,
    client: reqwest::Client,
    token: String,
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

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
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
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = body["accessToken"].as_str().unwrap().to_string();
    Ctx {
        seeded,
        f,
        base_url,
        client,
        token,
    }
}

impl Ctx {
    async fn send(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> Value {
        let mut req = self
            .client
            .request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.token);
        if let Some(b) = body {
            req = req.json(&b);
        }
        let resp = req.send().await.unwrap();
        let status = resp.status();
        let body: Value = resp.json().await.unwrap();
        assert!(status.is_success(), "{path} : {status} {body}");
        body
    }

    async fn post(&self, path: &str, body: Value) -> Value {
        self.send(reqwest::Method::POST, path, Some(body)).await
    }

    /// Une facture fournisseur **brouillon** de `montant` (sans TVA), créée par la
    /// route ; rend sa réponse.
    async fn draft(&self, number: &str, montant: &str) -> Value {
        self.post("/api/v1/supplier-invoices", self.body(number, montant))
            .await
    }

    fn body(&self, number: &str, montant: &str) -> Value {
        json!({
            "contactId": self.f.supplier_id,
            "supplierInvoiceNumber": number,
            "invoiceDate": "2026-06-15",
            "creditorIban": fournisseur_rapproche::IBAN_CREANCIER,
            "lines": [{
                "description": "Fourniture",
                "quantity": "1",
                "unitPrice": montant,
                "vatRate": "0",
                "expenseAccountId": self.seeded.accounts["4000"],
            }],
        })
    }

    /// Une facture **validée** : brouillon, puis `…/validate`.
    async fn open(&self, number: &str) -> Value {
        let d = self.draft(number, "100.00").await;
        self.post(
            &format!("/api/v1/supplier-invoices/{}/validate", d["id"]),
            json!({ "version": d["version"] }),
        )
        .await
    }

    /// Une facture **payée** par virement.
    async fn paid(&self, number: &str) -> Value {
        let o = self.open(number).await;
        self.post(
            &format!("/api/v1/supplier-invoices/{}/pay", o["id"]),
            self.pay_body(),
        )
        .await
    }

    fn pay_body(&self) -> Value {
        json!({ "settlementType": "bank_transfer", "bankAccountId": self.f.bank_account_id,
                "paymentDate": "2026-06-20" })
    }

    /// AC-4 — la réponse d'un geste égale, **champ à champ**, le `GET` qui la
    /// suit : `status`, `version`, `paidAt`, `settlementJournalEntryId`,
    /// `internalNumber`, lignes, `settlementCancellable`, `cancellable` et leurs
    /// motifs, `lastConfirmedBatch`, `sourceDocument` — et tous les autres.
    async fn assert_response_is_what_was_written(&self, response: &Value) {
        let id = response["id"].as_i64().expect("identifiant rendu");
        let read = self
            .send(
                reqwest::Method::GET,
                &format!("/api/v1/supplier-invoices/{id}"),
                None,
            )
            .await;
        assert_eq!(response, &read, "la réponse doit égaler le GET qui la suit");
        // Montage : les champs calculés le sont, et la facture porte des lignes.
        assert!(read["settlementCancellable"].is_boolean(), "{read}");
        assert!(read["cancellable"].is_boolean(), "{read}");
        assert!(read.get("sourceDocument").is_some(), "{read}");
        assert!(read.get("lastConfirmedBatch").is_some(), "{read}");
        assert!(!read["lines"].as_array().unwrap().is_empty(), "{read}");
    }
}

// ---------------------------------------------------------------------------
// AC-4 — un test par geste
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn update_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let d = c.draft("FF-PUT", "10.00").await;
    let mut body = c.body("FF-PUT-2", "25.50");
    body["version"] = d["version"].clone();
    let r = c
        .send(
            reqwest::Method::PUT,
            &format!("/api/v1/supplier-invoices/{}", d["id"]),
            Some(body),
        )
        .await;
    assert_eq!(r["supplierInvoiceNumber"], "FF-PUT-2", "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validate_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let r = c.open("FF-VAL").await;
    assert_eq!(r["status"], "open", "{r}");
    assert!(r["internalNumber"].is_string(), "numéro posé : {r}");
    c.assert_response_is_what_was_written(&r).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unvalidate_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let o = c.open("FF-UNV").await;
    let r = c
        .post(
            &format!("/api/v1/supplier-invoices/{}/unvalidate", o["id"]),
            json!({ "version": o["version"] }),
        )
        .await;
    assert_eq!(r["status"], "draft", "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pay_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let r = c.paid("FF-PAY").await;
    assert_eq!(r["status"], "paid", "{r}");
    assert!(r["settlementJournalEntryId"].is_i64(), "{r}");
    assert!(r["paidAt"].is_string(), "{r}");
    assert_eq!(r["settlementCancellable"], true, "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let o = c.open("FF-CAN").await;
    let r = c
        .post(
            &format!("/api/v1/supplier-invoices/{}/cancel", o["id"]),
            json!({}),
        )
        .await;
    assert_eq!(r["status"], "cancelled", "{r}");
    assert_eq!(r["cancellable"], false, "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn settlement_cancel_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let p = c.paid("FF-SCN").await;
    let r = c
        .post(
            &format!("/api/v1/supplier-invoices/{}/settlement/cancel", p["id"]),
            json!({}),
        )
        .await;
    assert!(r["reversalJournalEntryId"].is_i64(), "{r}");
    let invoice = &r["invoice"];
    assert_eq!(invoice["status"], "open", "{r}");
    assert_eq!(invoice["settlementCancellable"], false, "{r}");
    assert_eq!(
        invoice["settlementCancelBlockedBy"], "SUPPLIER_INVOICE_NOT_PAID",
        "{r}"
    );
    c.assert_response_is_what_was_written(invoice).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complete_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let doc = document(&pool, c.seeded.company_id, "e348-complete", None).await;
    let r = c
        .post(
            &format!("/api/v1/imported-supplier-invoices/{doc}/complete"),
            c.body("FF-CPL", "40.00"),
        )
        .await;
    assert_eq!(r["status"], "draft", "{r}");
    assert_eq!(r["sourceDocument"]["importedSupplierInvoiceId"], doc, "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

impl Ctx {
    /// Une facture validée et un document à traiter ; rend le chemin du document de la
    /// facture et l'identifiant du document.
    async fn invoice_and_document(&self, pool: &MySqlPool, hash: &str) -> (String, i64) {
        let o = self.open("FF-DOC").await;
        let doc = document(pool, self.seeded.company_id, hash, None).await;
        (
            format!("/api/v1/supplier-invoices/{}/source-document", o["id"]),
            doc,
        )
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn attach_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (path, doc) = c.invoice_and_document(&pool, "e348-attach").await;
    let r = c
        .post(&path, json!({ "importedSupplierInvoiceId": doc }))
        .await;
    assert_eq!(r["sourceDocument"]["importedSupplierInvoiceId"], doc, "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

/// Revue P1 (A-3) : séparé du rattachement, pour qu'un rouge de l'un ne masque pas
/// l'autre. Le rattachement du montage passe par la route, contrôlé par son statut seul.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn detach_returns_what_was_written(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (path, doc) = c.invoice_and_document(&pool, "e348-detach").await;
    c.post(&path, json!({ "importedSupplierInvoiceId": doc }))
        .await;
    let r = c.send(reqwest::Method::DELETE, &path, None).await;
    assert_eq!(r["sourceDocument"], Value::Null, "{r}");
    c.assert_response_is_what_was_written(&r).await;
}

// ---------------------------------------------------------------------------
// AC-3 — rien ne s'attend après l'écriture ; la vue est lue avant le commit
// ---------------------------------------------------------------------------

const HANDLERS: &str = include_str!("../src/routes/supplier_invoices.rs");
const DOCUMENTS: &str = include_str!("../src/routes/imported_supplier_invoices.rs");
const WRITES: &str = include_str!("../../kesh-db/src/repositories/supplier_invoices.rs");

/// Les neuf handlers et la fonction qui écrit pour chacun.
const NINE_HANDLERS: [(&str, &str, &str); 9] = [
    (
        HANDLERS,
        "update_supplier_invoice",
        "update_supplier_invoice_once",
    ),
    (HANDLERS, "validate_supplier_invoice", "validate"),
    (HANDLERS, "unvalidate_supplier_invoice", "unvalidate"),
    (HANDLERS, "pay_supplier_invoice", "pay"),
    (HANDLERS, "cancel_supplier_invoice", "cancel"),
    (
        HANDLERS,
        "cancel_supplier_invoice_settlement",
        "cancel_settlement",
    ),
    (DOCUMENTS, "complete_import", "complete_import_once"),
    (
        DOCUMENTS,
        "attach_source_document",
        "attach_source_document_once",
    ),
    (
        DOCUMENTS,
        "detach_source_document",
        "detach_source_document_once",
    ),
];

/// Les neuf fonctions qui tiennent la transaction : les cinq enveloppes de
/// `kesh-db` et les quatre tentatives `_once` des routes.
const NINE_WRITERS: [(&str, &str); 9] = [
    (WRITES, "validate"),
    (WRITES, "unvalidate"),
    (WRITES, "pay"),
    (WRITES, "cancel"),
    (WRITES, "cancel_settlement"),
    (HANDLERS, "update_supplier_invoice_once"),
    (DOCUMENTS, "complete_import_once"),
    (DOCUMENTS, "attach_source_document_once"),
    (DOCUMENTS, "detach_source_document_once"),
];

/// La règle d'une fonction qui écrit : elle termine sa transaction par
/// `commit_with_view`, **jamais** par `commit` (`tx.commit()`), et rien ne
/// s'attend après. « Rien après » seul serait trivialement vrai de l'appel final :
/// c'est l'absence de `commit` qui mord. `Err` dit ce qui manque.
fn ends_with_the_view(source: &str, function: &str) -> Result<(), String> {
    if !fn_calls(source, function, "commit_with_view") {
        return Err(format!("{function} n'appelle pas commit_with_view"));
    }
    if fn_calls(source, function, "commit") {
        return Err(format!("{function} valide sa transaction par commit"));
    }
    let after = around(source, function, "commit_with_view", None).awaits_after;
    if after != 0 {
        return Err(format!(
            "{function} attend {after} fois après commit_with_view"
        ));
    }
    Ok(())
}

/// Les handlers ne relisent rien : après l'écriture, plus aucune attente.
#[test]
fn the_nine_handlers_wait_for_nothing_after_the_write() {
    let late: Vec<_> = NINE_HANDLERS
        .iter()
        .filter(|(source, handler, write)| around(source, handler, write, None).awaits_after != 0)
        .map(|(_, handler, _)| *handler)
        .collect();
    assert!(
        late.is_empty(),
        "attendent encore après l'écriture : {late:?}"
    );
}

/// Les cinq enveloppes et les quatre `_once` terminent par `commit_with_view`.
#[test]
fn the_nine_writers_end_with_commit_with_view() {
    let wrong: Vec<_> = NINE_WRITERS
        .iter()
        .filter_map(|(source, function)| ends_with_the_view(source, function).err())
        .collect();
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// « Pas de commit sans réponse » : `commit_with_view` lit la vue avant le
/// commit, et rien ne s'attend après lui.
#[test]
fn commit_with_view_reads_before_the_commit_and_nothing_waits_after_it() {
    assert_eq!(
        around(
            WRITES,
            "commit_with_view",
            "commit",
            Some("get_settlement_view_in_tx")
        ),
        AroundCall {
            awaits_after: 0,
            read_before: true,
        }
    );
}

/// ⛔ La sonde positive passe par le même découpage que la garde : sans elle,
/// une lecture cassée la rendrait verte à vide.
#[test]
fn the_guard_sees_a_reread_and_a_direct_commit() {
    // Une relecture remise après l'écriture, dans un handler factice.
    let handler = "async fn h() {
        let company = get_company_for(&u, &state.pool).await?;
        retry_on_deadlock(\"x\", || supplier_invoices::pay(&state.pool, company.id, id)).await?;
        Ok(Json(SupplierInvoiceResponse::load_with_settlement_cancellation(
            &state.pool, company.id, id).await?))
    }";
    assert_eq!(around(handler, "h", "pay", None).awaits_after, 1);

    // Une enveloppe factice qui valide par `tx.commit()`.
    let direct = "async fn w() {
        let mut tx = pool.begin().await?;
        let view = get_settlement_view_in_tx(&mut tx, c, id).await?;
        let done = pay_in_tx(&mut tx, c, id).await?;
        tx.commit().await?;
        Ok(view)
    }";
    assert_eq!(
        ends_with_the_view(direct, "w"),
        Err("w n'appelle pas commit_with_view".into())
    );
    let both = direct.replace(
        "tx.commit().await?;",
        "commit_with_view(tx, c, Ok(done), |_| id).await?; tx.commit().await?;",
    );
    assert_eq!(
        ends_with_the_view(&both, "w"),
        Err("w valide sa transaction par commit".into())
    );
    let good = "async fn w() {
        let mut tx = pool.begin().await?;
        let result = pay_in_tx(&mut tx, c, id).await;
        let (view, _) = commit_with_view(tx, c, result, |_| id).await?;
        Ok(view)
    }";
    assert_eq!(ends_with_the_view(good, "w"), Ok(()));
    let late = good.replace("Ok(view)", "relire(id).await?; Ok(view)");
    assert_eq!(
        ends_with_the_view(&late, "w"),
        Err("w attend 1 fois après commit_with_view".into())
    );
}
