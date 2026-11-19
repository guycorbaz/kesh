//! Story 48-12a (#975) — le refus des factures fournisseur en double, **par
//! HTTP** : le contrat (`409 SUPPLIER_INVOICE_DUPLICATE` et
//! `SUPPLIER_INVOICE_PROBABLE_DUPLICATE`, `details : { supplierInvoiceId,
//! internalNumber, status, confirmable }`, `confirmNotDuplicate`), ses chemins
//! (création, `PUT`, complétion, dévalidation, clé `read-write`), les messages,
//! et la place du verrou du fournisseur dans les routes (AC-8, tests pilotés).
//!
//! Les règles sont prouvées au dépôt (`kesh-db/tests/supplier_invoice_duplicate.rs`
//! et `supplier_invoice_duplicate_lock.rs`).
//!
//! Décor synthétique : `seed_accounting_company`, `fournisseur_rapproche::preparer`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::repositories::supplier_invoices::{self, DuplicatePolicy};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, seed_accounting_company,
};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

#[path = "../../kesh-db/tests/support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use fournisseur_rapproche::{Fournisseur, document};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";
const BORNE: Duration = Duration::from_secs(10);

/// Le texte de la règle 1 (fr-CH), tel que le catalogue le porte.
const TEXTE_REGLE_1: &str = "Une facture de ce fournisseur porte déjà ce numéro pour la même année. Si c’est la même facture, ne l’enregistrez pas une seconde fois. Si le fournisseur a recommencé sa numérotation en cours d’année, ajoutez un caractère au numéro, par exemple « 15-b ».";
const TEXTE_REGLE_2: &str = "Une facture de ce fournisseur porte déjà la même date et le même montant. Si c’est la même facture, ne l’enregistrez pas une seconde fois ; sinon, confirmez que ce n’est pas un doublon.";
const TEXTE_DEVALIDATION: &str = "Une facture équivalente non annulée existe. Repasser celle-ci en brouillon en ferait un doublon : ne le faites que pour la retirer ou la corriger ensuite.";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
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

struct Ctx {
    seeded: SeededCompany,
    f: Fournisseur,
    app: TestApp,
    admin: String,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    let f = fournisseur_rapproche::preparer(pool, &seeded).await;
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/login", app.base_url))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let admin = body["accessToken"].as_str().expect("jeton").to_string();
    Ctx {
        seeded,
        f,
        app,
        admin,
    }
}

async fn cle_read_write(c: &Ctx) -> String {
    let (s, b) = envoyer(
        c,
        reqwest::Method::POST,
        "/api/v1/settings/api-keys",
        &c.admin,
        Some(json!({ "name": "clé 48-12a", "scope": "read-write" })),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    b["key"].as_str().unwrap().to_string()
}

fn corps(c: &Ctx, numero: Option<&str>, date: &str, montant: &str) -> Value {
    json!({
        "contactId": c.f.supplier_id,
        "supplierInvoiceNumber": numero,
        "invoiceDate": date,
        "creditorIban": fournisseur_rapproche::IBAN_CREANCIER,
        "lines": [{
            "description": "Fourniture",
            "quantity": "1",
            "unitPrice": montant,
            "vatRate": "0",
            "expenseAccountId": c.seeded.accounts["4000"],
        }],
    })
}

async fn envoyer_url(
    client: &reqwest::Client,
    url: String,
    methode: reqwest::Method,
    jeton: &str,
    corps: Option<Value>,
) -> (u16, Value) {
    let mut req = client.request(methode, url).bearer_auth(jeton);
    if let Some(b) = corps {
        req = req.json(&b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let text = resp.text().await.unwrap();
    let body = if text.is_empty() {
        Value::Null
    } else {
        serde_json::from_str(&text).unwrap_or(Value::String(text))
    };
    (status, body)
}

async fn envoyer(
    c: &Ctx,
    methode: reqwest::Method,
    chemin: &str,
    jeton: &str,
    corps: Option<Value>,
) -> (u16, Value) {
    envoyer_url(
        &c.app.client,
        format!("{}{}", c.app.base_url, chemin),
        methode,
        jeton,
        corps,
    )
    .await
}

async fn post(c: &Ctx, chemin: &str, corps: Value) -> (u16, Value) {
    envoyer(c, reqwest::Method::POST, chemin, &c.admin, Some(corps)).await
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

fn statut_document(pool: &MySqlPool, doc: i64) -> impl std::future::Future<Output = String> + '_ {
    async move {
        sqlx::query_scalar("SELECT status FROM imported_supplier_invoices WHERE id = ?")
            .bind(doc)
            .fetch_one(pool)
            .await
            .unwrap()
    }
}

// ---------------------------------------------------------------------------
// Le contrat (AC-1, AC-3, AC-10, AC-12)
// ---------------------------------------------------------------------------

/// `POST` deux fois → `201` puis `409 SUPPLIER_INVOICE_DUPLICATE`, `details` et
/// `confirmable: false`, message « si c'est la même facture » puis « 15-b » ;
/// `confirmNotDuplicate` sans effet. Règle 2 : `409` confirmable, puis `201`
/// avec `confirmNotDuplicate: true`. Tue « champ `confirmNotDuplicate` ignoré » et
/// « message de la règle 1 sans « si c'est la même facture » ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_twice_is_refused_with_details_and_message(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, a) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("FA-1"), "2026-06-15", "10"),
    )
    .await;
    assert_eq!(s, 201, "{a}");
    for confirm in [false, true] {
        let mut b = corps(&c, Some("fa 1"), "2026-11-15", "99");
        b["supplierInvoiceNumber"] = json!(" FA-1 ");
        b["confirmNotDuplicate"] = json!(confirm);
        let (s, e) = post(&c, "/api/v1/supplier-invoices", b).await;
        assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
        assert_eq!(
            e["error"]["details"],
            json!({
                "supplierInvoiceId": a["id"],
                "internalNumber": null,
                "status": "draft",
                "confirmable": false,
            })
        );
        assert_eq!(e["error"]["message"], TEXTE_REGLE_1);
    }

    // Règle 2.
    let (s, p) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, None, "2026-07-01", "50"),
    )
    .await;
    assert_eq!(s, 201, "{p}");
    let (s, e) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, None, "2026-07-01", "50.00"),
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_PROBABLE_DUPLICATE"),
        "{e}"
    );
    assert_eq!(e["error"]["details"]["confirmable"], true);
    assert_eq!(e["error"]["details"]["supplierInvoiceId"], p["id"]);
    assert_eq!(e["error"]["message"], TEXTE_REGLE_2);
    let mut b = corps(&c, None, "2026-07-01", "50");
    b["confirmNotDuplicate"] = json!(true);
    let (s, q) = post(&c, "/api/v1/supplier-invoices", b).await;
    assert_eq!(s, 201, "{q}");
    let rule: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'supplier_invoice.duplicate_confirmed' AND entity_id = ?",
    )
    .bind(q["id"].as_i64().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(rule["rule"], 2);
}

/// `PUT` et clé `read-write` : la même règle, par chaque chemin (AC-10).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_and_api_key_paths_apply_the_rule(pool: MySqlPool) {
    let c = setup(&pool).await;
    let cle = cle_read_write(&c).await;
    let (s, a) = envoyer(
        &c,
        reqwest::Method::POST,
        "/api/v1/supplier-invoices",
        &cle,
        Some(corps(&c, Some("K-1"), "2026-02-02", "10")),
    )
    .await;
    assert_eq!(s, 201, "{a}");
    let (s, e) = envoyer(
        &c,
        reqwest::Method::POST,
        "/api/v1/supplier-invoices",
        &cle,
        Some(corps(&c, Some("K-1"), "2026-02-03", "11")),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
    let (s, b) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("K-2"), "2026-02-02", "10"),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    let mut put = corps(&c, Some("K-1"), "2026-02-02", "10");
    put["version"] = b["version"].clone();
    for jeton in [&c.admin, &cle] {
        let (s, e) = envoyer(
            &c,
            reqwest::Method::PUT,
            &format!("/api/v1/supplier-invoices/{}", b["id"]),
            jeton,
            Some(put.clone()),
        )
        .await;
        assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
        assert_eq!(e["error"]["details"]["supplierInvoiceId"], a["id"]);
    }
    // `PUT` sans changement des quatre éléments : passe.
    let mut put = corps(&c, Some("K-2"), "2026-02-02", "10");
    put["version"] = b["version"].clone();
    put["dueDate"] = json!("2026-03-03");
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{}", b["id"]),
        &cle,
        Some(put),
    )
    .await;
    assert_eq!(s, 200, "{e}");
}

/// La complétion : doublon → `409`, document `to_complete` ; puis le second
/// passage d'un document complété rend `IMPORT_NOT_PENDING_COMPLETION`, jamais
/// un refus de doublon (D4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn completion_refuses_a_duplicate_and_keeps_the_document(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, _) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("D-1"), "2026-05-05", "10"),
    )
    .await;
    assert_eq!(s, 201);
    let doc = document(&pool, c.seeded.company_id, "dup-48-12a", None).await;
    let chemin = format!("/api/v1/imported-supplier-invoices/{doc}/complete");
    let (s, e) = post(&c, &chemin, corps(&c, Some("D-1"), "2026-05-06", "12")).await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
    assert_eq!(statut_document(&pool, doc).await, "to_complete");
    let (s, e) = post(&c, &chemin, corps(&c, Some("D-2"), "2026-05-06", "12")).await;
    assert_eq!(s, 200, "{e}");
    let (s, e) = post(&c, &chemin, corps(&c, Some("D-2"), "2026-05-06", "12")).await;
    assert_eq!((s, code(&e)), (409, "IMPORT_NOT_PENDING_COMPLETION"), "{e}");
}

/// `POST …/unvalidate` d'une annulée en doublon : refus (code de la règle,
/// `confirmable: true`, message de la dévalidation, sans « 15-b »), puis
/// `confirmNotDuplicate: true` → `200`, audit au nom de la clé. Tue « message de
/// création servi à la dévalidation ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unvalidation_of_a_cancelled_duplicate_by_api_key(pool: MySqlPool) {
    let c = setup(&pool).await;
    let cle = cle_read_write(&c).await;
    let (_, a) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("U-1"), "2026-04-04", "10"),
    )
    .await;
    let (s, v) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{}/validate", a["id"]),
        json!({ "version": a["version"] }),
    )
    .await;
    assert_eq!(s, 200, "{v}");
    let (s, x) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{}/cancel", a["id"]),
        json!({}),
    )
    .await;
    assert_eq!(s, 200, "{x}");
    let (s, b) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("U-1"), "2026-04-05", "11"),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    let chemin = format!("/api/v1/supplier-invoices/{}/unvalidate", a["id"]);
    let (s, e) = envoyer(
        &c,
        reqwest::Method::POST,
        &chemin,
        &cle,
        Some(json!({ "version": x["version"] })),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
    assert_eq!(
        e["error"]["details"],
        json!({ "supplierInvoiceId": b["id"], "internalNumber": null, "status": "draft", "confirmable": true })
    );
    assert_eq!(e["error"]["message"], TEXTE_DEVALIDATION);
    assert!(!e["error"]["message"].as_str().unwrap().contains("15-b"));
    let (s, r) = envoyer(
        &c,
        reqwest::Method::POST,
        &chemin,
        &cle,
        Some(json!({ "version": x["version"], "confirmNotDuplicate": true })),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    assert_eq!(r["status"], "draft");
    let cle_audit: Option<i64> = sqlx::query_scalar(
        "SELECT actor_api_key_id FROM audit_log WHERE action = 'supplier_invoice.duplicate_confirmed' AND entity_id = ?",
    )
    .bind(a["id"].as_i64().unwrap())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(cle_audit.is_some(), "la confirmation est au nom de la clé");
}

// ---------------------------------------------------------------------------
// Concurrence pilotée par les routes (AC-8)
// ---------------------------------------------------------------------------

fn facture(c: &Ctx, numero: &str) -> NewSupplierInvoice {
    NewSupplierInvoice {
        company_id: c.seeded.company_id,
        contact_id: c.f.supplier_id,
        supplier_invoice_number: Some(numero.into()),
        invoice_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
        due_date: None,
        creditor_iban: Some(fournisseur_rapproche::IBAN_CREANCIER.into()),
        creditor_qr_iban: None,
        payment_reference: None,
        expected_payment_amount: None,
        project_id: None,
        lines: vec![NewSupplierInvoiceLine {
            description: "Fourniture".into(),
            quantity: dec!(1),
            unit_price: dec!(10),
            vat_rate: dec!(0),
            expense_account_id: c.seeded.accounts["4000"],
        }],
    }
}

/// A : une transaction brute du test, sur le pool de l'application, qui crée la
/// facture `numero` sans commiter.
async fn a_cree<'p>(
    pool: &'p MySqlPool,
    c: &Ctx,
    numero: &str,
) -> sqlx::Transaction<'p, sqlx::MySql> {
    let mut a = pool.begin().await.unwrap();
    supplier_invoices::create_draft_in_tx_with_policy(
        &mut a,
        facture(c, numero),
        c.seeded.admin_user_id,
        DuplicatePolicy::Refuse,
    )
    .await
    .expect("A crée");
    a
}

/// `PUT` par la route : B (requête HTTP changeant le numéro vers celui de A)
/// attend le fournisseur, puis rend `409`. Tue « la route lit le document joint
/// avant `lock_draft_and_supplier_in_tx` » (l'instantané de B précède A : `200`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_route_takes_the_supplier_lock_before_reading_the_document(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (s, b) = post(
        &c,
        "/api/v1/supplier-invoices",
        corps(&c, Some("R-0"), "2026-06-01", "10"),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    let a = a_cree(&pool, &c, "R-1").await;
    let mut put = corps(&c, Some("R-1"), "2026-06-01", "10");
    put["version"] = b["version"].clone();
    let (client, url, jeton) = (
        c.app.client.clone(),
        format!("{}/api/v1/supplier-invoices/{}", c.app.base_url, b["id"]),
        c.admin.clone(),
    );
    let tache = tokio::spawn(async move {
        envoyer_url(&client, url, reqwest::Method::PUT, &jeton, Some(put)).await
    });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM contacts WHERE id", "FOR UPDATE"], || tache
            .is_finished())
        .await,
        "B doit attendre le verrou du fournisseur"
    );
    a.commit().await.unwrap();
    let (s, e) = tokio::time::timeout(BORNE, tache)
        .await
        .expect("B conclut")
        .unwrap();
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
}

/// Complétion par la route : B (complétion d'un document au numéro de A)
/// attend le fournisseur, puis rend `409` ; le document reste `to_complete`.
/// Tue « lecture ordinaire ajoutée avant la création dans la route ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn completion_route_waits_for_the_supplier_then_refuses(pool: MySqlPool) {
    let c = setup(&pool).await;
    let doc = document(&pool, c.seeded.company_id, "pil-48-12a", None).await;
    let a = a_cree(&pool, &c, "R-2").await;
    let (client, url, jeton, b) = (
        c.app.client.clone(),
        format!(
            "{}/api/v1/imported-supplier-invoices/{doc}/complete",
            c.app.base_url
        ),
        c.admin.clone(),
        corps(&c, Some("R-2"), "2026-06-02", "10"),
    );
    let tache = tokio::spawn(async move {
        envoyer_url(&client, url, reqwest::Method::POST, &jeton, Some(b)).await
    });
    assert!(
        attendre_une_requete_en_cours(&pool, &["FROM contacts WHERE id", "FOR UPDATE"], || tache
            .is_finished())
        .await,
        "B doit attendre le verrou du fournisseur"
    );
    a.commit().await.unwrap();
    let (s, e) = tokio::time::timeout(BORNE, tache)
        .await
        .expect("B conclut")
        .unwrap();
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DUPLICATE"), "{e}");
    assert_eq!(statut_document(&pool, doc).await, "to_complete");
}
