//! Story v017-4a (#849) — le cycle brouillon → validée → payée d'une facture
//! fournisseur, **par HTTP** : ce qui ne se voit qu'à travers les routes — le
//! contrat (`status`, `purchaseJournalEntryId` nul et présent, `sourceDocument`
//! enrichi), les rôles et les clés, les codes d'erreur, les règles « document »
//! du `PUT`, le filtre de la liste, l'échéance (#853), et les chemins où un
//! brouillon n'a pas sa place (lots, réconciliation, règlement).
//!
//! Les règles métier sont prouvées au dépôt (`kesh-db/tests/supplier_invoices_draft.rs`).
//!
//! Décor synthétique : `seed_accounting_company`, `fournisseur_rapproche::preparer`.
//!
//! Pré-requis : MariaDB démarré.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{imported_supplier_invoices, users};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

#[path = "../../kesh-db/tests/support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

use fournisseur_rapproche::Fournisseur;

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

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": password }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"]
        .as_str()
        .unwrap_or_else(|| panic!("connexion de {username} : {body}"))
        .to_string()
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
    let admin = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    Ctx {
        seeded,
        f,
        app,
        admin,
    }
}

/// Un Comptable de la société du décor ; rend son jeton.
async fn comptable(pool: &MySqlPool, c: &Ctx) -> String {
    users::create(
        pool,
        NewUser {
            username: "comptable".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id: c.seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    login(&c.app, "comptable", "password123").await
}

/// Une clé d'API `read-write` créée par l'administrateur.
async fn cle_read_write(c: &Ctx) -> String {
    let resp = c
        .app
        .client
        .post(c.app.url("/api/v1/settings/api-keys"))
        .bearer_auth(&c.admin)
        .json(&json!({ "name": "clé v017-4a", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de la clé");
    let body: Value = resp.json().await.unwrap();
    body["key"].as_str().unwrap().to_string()
}

fn corps(c: &Ctx, numero: &str, montant: &str) -> Value {
    json!({
        "contactId": c.f.supplier_id,
        "supplierInvoiceNumber": numero,
        "invoiceDate": "2026-06-15",
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

async fn envoyer(
    c: &Ctx,
    methode: reqwest::Method,
    chemin: &str,
    jeton: &str,
    corps: Option<Value>,
) -> (u16, Value) {
    let mut req = c
        .app
        .client
        .request(methode, c.app.url(chemin))
        .bearer_auth(jeton);
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

async fn post(c: &Ctx, chemin: &str, corps: Value) -> (u16, Value) {
    envoyer(
        c,
        reqwest::Method::POST,
        chemin,
        &c.admin.clone(),
        Some(corps),
    )
    .await
}

async fn creer(c: &Ctx, numero: &str, montant: &str) -> Value {
    let (s, b) = post(c, "/api/v1/supplier-invoices", corps(c, numero, montant)).await;
    assert_eq!(s, 201, "{b}");
    b
}

async fn valider(c: &Ctx, inv: &Value) -> Value {
    let (s, b) = post(
        c,
        &format!("/api/v1/supplier-invoices/{}/validate", inv["id"]),
        json!({ "version": inv["version"] }),
    )
    .await;
    assert_eq!(s, 200, "{b}");
    b
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

/// Un montant de la réponse (chaîne décimale, échelle de la colonne), comparé
/// en valeur : `"25.5000"` vaut `25.50`.
fn montant(v: &Value) -> Decimal {
    v.as_str()
        .unwrap_or_else(|| panic!("montant attendu en chaîne : {v}"))
        .parse()
        .unwrap()
}

async fn nb_ecritures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Un document à traiter, avec ou sans QR (IBAN classique, montant, devise).
async fn document(
    pool: &MySqlPool,
    company_id: i64,
    hash: &str,
    qr: Option<(&str, Decimal, &str)>,
) -> i64 {
    let mut new = NewImportedSupplierInvoice {
        company_id,
        file_hash: hash.into(),
        storage_path: format!("{hash}.png"),
        original_filename: format!("{hash}.png"),
        mime_type: "image/png".into(),
        byte_size: 10,
        creditor_iban: None,
        is_qr_iban: None,
        creditor_address_type: None,
        creditor_name: None,
        creditor_line1: None,
        creditor_line2: None,
        creditor_postal_code: None,
        creditor_town: None,
        creditor_country: None,
        reference_type: None,
        reference_value: None,
        amount: None,
        currency: None,
        unstructured_message: None,
        billing_information: None,
    };
    if let Some((iban, montant, devise)) = qr {
        new.creditor_iban = Some(iban.into());
        new.is_qr_iban = Some(false);
        new.creditor_address_type = Some("K".into());
        new.creditor_name = Some("Fournisseur rapproché SA".into());
        new.creditor_line1 = Some("Rue 1".into());
        new.creditor_line2 = Some("1000 Lausanne".into());
        new.creditor_country = Some("CH".into());
        new.reference_type = Some("NON".into());
        new.amount = Some(montant);
        new.currency = Some(devise.into());
    }
    imported_supplier_invoices::create(pool, &new)
        .await
        .expect("document")
        .id
}

// ---------------------------------------------------------------------------
// AC-4, AC-6 — le contrat de la création
// ---------------------------------------------------------------------------

/// AC-6, AC-4 — `POST /supplier-invoices` rend un **brouillon** : `201`,
/// `status: "draft"`, `purchaseJournalEntryId` **présent et nul**, `version: 1`,
/// aucune écriture ; le `GET` le relit ainsi, `cancellable: false` et
/// `cancelBlockedBy: "SUPPLIER_INVOICE_DRAFT"` (AC-5 (7)).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_returns_a_draft_without_entry(pool: MySqlPool) {
    let c = setup(&pool).await;
    let avant = nb_ecritures(&pool, c.seeded.company_id).await;
    let b = creer(&c, "FF-1", "100.00").await;
    assert_eq!(b["status"], "draft", "{b}");
    assert_eq!(
        b.get("purchaseJournalEntryId"),
        Some(&Value::Null),
        "champ présent et nul : {b}"
    );
    assert_eq!(b["version"], 1);
    assert_eq!(nb_ecritures(&pool, c.seeded.company_id).await, avant);

    let (s, g) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!("/api/v1/supplier-invoices/{}", b["id"]),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200);
    assert_eq!(g["status"], "draft");
    assert_eq!(g.get("purchaseJournalEntryId"), Some(&Value::Null));
    assert_eq!(g["cancellable"], false);
    assert_eq!(g["cancelBlockedBy"], "SUPPLIER_INVOICE_DRAFT", "{g}");
    assert_eq!(g["sourceDocument"], Value::Null);
}

// ---------------------------------------------------------------------------
// AC-10, AC-11 — valider, dévalider
// ---------------------------------------------------------------------------

/// AC-10, AC-11 — les deux routes, leurs réponses et leurs codes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validate_and_unvalidate_routes(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-V", "40.00").await;
    let id = b["id"].clone();
    let v = valider(&c, &b).await;
    assert_eq!(v["status"], "open");
    assert!(v["purchaseJournalEntryId"].is_i64(), "{v}");
    assert_eq!(v["version"], 2);

    // Valider une facture validée : le CODE.
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/validate"),
        json!({ "version": v["version"] }),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "ILLEGAL_STATE_TRANSITION"), "{e}");

    // Dévalider avec une version périmée, puis juste.
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": 1 }),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "OPTIMISTIC_LOCK_CONFLICT"), "{e}");
    let (s, u) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": v["version"] }),
    )
    .await;
    assert_eq!(s, 200, "{u}");
    assert_eq!(u["status"], "draft");
    assert_eq!(u.get("purchaseJournalEntryId"), Some(&Value::Null));
    assert_eq!(u["version"], 3);

    // Dévalider un brouillon.
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": 3 }),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "ILLEGAL_STATE_TRANSITION"), "{e}");

    // Payée : `SUPPLIER_INVOICE_PAID`.
    let v = valider(&c, &u).await;
    let (s, p) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/pay"),
        json!({ "settlementType": "bank_transfer", "bankAccountId": c.f.bank_account_id,
                "paymentDate": "2026-06-20" }),
    )
    .await;
    assert_eq!(s, 200, "{p}");
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": p["version"] }),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_PAID"), "{e} ({v})");
}

// ---------------------------------------------------------------------------
// AC-8 — `PUT`
// ---------------------------------------------------------------------------

/// AC-8 — le `PUT` d'un brouillon : tout change, relecture complète ; refus par
/// leur **code** (M8) : facture validée, version périmée, facture inconnue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_replaces_a_draft(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-P", "10.00").await;
    let id = b["id"].clone();
    let mut nouveau = corps(&c, "FF-P2", "25.50");
    nouveau["invoiceDate"] = json!("2026-06-18");
    nouveau["dueDate"] = json!("2026-07-18");
    nouveau["version"] = b["version"].clone();
    let (s, p) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{id}"),
        &c.admin,
        Some(nouveau.clone()),
    )
    .await;
    assert_eq!(s, 200, "{p}");
    assert_eq!(p["supplierInvoiceNumber"], "FF-P2");
    assert_eq!(p["invoiceDate"], "2026-06-18");
    assert_eq!(p["dueDate"], "2026-07-18");
    assert_eq!(montant(&p["totalAmount"]), dec!(25.50));
    assert_eq!(p["version"], 2);
    assert_eq!(p["status"], "draft");

    // Périmée.
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{id}"),
        &c.admin,
        Some(nouveau.clone()),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "OPTIMISTIC_LOCK_CONFLICT"), "{e}");

    // Validée : le CODE (M8).
    let v = valider(&c, &p).await;
    nouveau["version"] = v["version"].clone();
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{id}"),
        &c.admin,
        Some(nouveau.clone()),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "ILLEGAL_STATE_TRANSITION"), "M8 : {e}");

    // Inconnue.
    let (s, _) = envoyer(
        &c,
        reqwest::Method::PUT,
        "/api/v1/supplier-invoices/999999",
        &c.admin,
        Some(nouveau),
    )
    .await;
    assert_eq!(s, 404);
}

/// AC-8, AC-7 — un brouillon **né de la complétion** d'un document avec QR : au
/// `PUT`, les coordonnées du corps sont refusées (`QR_FIELDS_FROM_DOCUMENT`), le
/// total doit égaler le montant du QR (`AMOUNT_MISMATCH`), et un `PUT` juste
/// garde les coordonnées du QR (M13). La complétion pose `link_origin =
/// 'completion'` et n'écrit aucune écriture (M11).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_of_a_draft_born_from_a_qr_document_keeps_the_qr_rules(pool: MySqlPool) {
    let c = setup(&pool).await;
    let doc = document(
        &pool,
        c.seeded.company_id,
        "qr1",
        Some(("CH9300762011623852957", dec!(100.00), "CHF")),
    )
    .await;
    let avant = nb_ecritures(&pool, c.seeded.company_id).await;
    let mut body = corps(&c, "FF-QR", "100.00");
    body.as_object_mut().unwrap().remove("creditorIban");
    let (s, b) = post(
        &c,
        &format!("/api/v1/imported-supplier-invoices/{doc}/complete"),
        body.clone(),
    )
    .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["status"], "draft", "M11 : {b}");
    assert_eq!(nb_ecritures(&pool, c.seeded.company_id).await, avant, "M11");
    let origin: Option<String> =
        sqlx::query_scalar("SELECT link_origin FROM imported_supplier_invoices WHERE id = ?")
            .bind(doc)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(origin.as_deref(), Some("completion"));
    assert_eq!(b["sourceDocument"]["linkOrigin"], "completion", "{b}");
    assert_eq!(b["sourceDocument"]["importedSupplierInvoiceId"], doc);
    assert_eq!(
        b["sourceDocument"]["qr"]["creditorIban"],
        "CH9300762011623852957"
    );
    assert_eq!(montant(&b["sourceDocument"]["qr"]["amount"]), dec!(100.00));
    assert!(b["sourceDocument"].get("storagePath").is_none());

    let id = b["id"].clone();
    let chemin = format!("/api/v1/supplier-invoices/{id}");
    let mut avec_coord = corps(&c, "FF-QR", "100.00");
    avec_coord["version"] = b["version"].clone();
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &chemin,
        &c.admin,
        Some(avec_coord),
    )
    .await;
    assert_eq!((s, code(&e)), (400, "QR_FIELDS_FROM_DOCUMENT"), "M13 : {e}");

    let mut faux_total = body.clone();
    faux_total["lines"][0]["unitPrice"] = json!("90.00");
    faux_total["version"] = b["version"].clone();
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &chemin,
        &c.admin,
        Some(faux_total),
    )
    .await;
    assert_eq!((s, code(&e)), (400, "AMOUNT_MISMATCH"), "M13 : {e}");

    let mut juste = body;
    juste["supplierInvoiceNumber"] = json!("FF-QR-2");
    juste["version"] = b["version"].clone();
    let (s, p) = envoyer(&c, reqwest::Method::PUT, &chemin, &c.admin, Some(juste)).await;
    assert_eq!(s, 200, "{p}");
    assert_eq!(p["creditorIban"], "CH9300762011623852957");
    assert_eq!(montant(&p["expectedPaymentAmount"]), dec!(100.00));
}

/// AC-8 — **le cas de Guy** : une facture validée, un document joint après
/// coup (devise ≠ CHF, montant ≠ total, coordonnées différentes), dévalidée,
/// éditée, revalidée — aucune règle de la complétion ne s'impose (M20-ter).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_of_a_draft_with_an_attached_document_imposes_nothing(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-J", "80.00").await;
    let v = valider(&c, &b).await;
    let id = v["id"].clone();
    let doc = document(
        &pool,
        c.seeded.company_id,
        "att1",
        Some(("CH5604835012345678009", dec!(999.00), "EUR")),
    )
    .await;
    let (s, a) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/source-document"),
        json!({ "importedSupplierInvoiceId": doc }),
    )
    .await;
    assert_eq!(s, 200, "{a}");
    assert_eq!(a["sourceDocument"]["linkOrigin"], "attachment", "{a}");
    let (s, u) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": a["version"] }),
    )
    .await;
    assert_eq!(s, 200, "{u}");
    let mut nouveau = corps(&c, "FF-J", "82.00");
    nouveau["creditorIban"] = json!("CH9300762011623852957");
    nouveau["version"] = u["version"].clone();
    let (s, p) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{id}"),
        &c.admin,
        Some(nouveau),
    )
    .await;
    assert_eq!(s, 200, "M20-ter : {p}");
    assert_eq!(p["creditorIban"], "CH9300762011623852957");
    assert_eq!(montant(&p["totalAmount"]), dec!(82.00));
    let r = valider(&c, &p).await;
    assert_eq!(r["status"], "open");
}

// ---------------------------------------------------------------------------
// AC-9 — `DELETE`
// ---------------------------------------------------------------------------

/// AC-9 — Administrateur seul : un Comptable → `403`, une clé `read-write` →
/// `403 API_KEY_ADMIN_FORBIDDEN` ; l'Administrateur → `204` ; une facture
/// validée → `409 SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_route_is_admin_only(pool: MySqlPool) {
    let c = setup(&pool).await;
    let jeton_comptable = comptable(&pool, &c).await;
    let cle = cle_read_write(&c).await;
    let b = creer(&c, "FF-DEL", "10.00").await;
    let chemin = format!("/api/v1/supplier-invoices/{}", b["id"]);

    let (s, _) = envoyer(&c, reqwest::Method::DELETE, &chemin, &jeton_comptable, None).await;
    assert_eq!(s, 403, "Comptable");
    let (s, e) = envoyer(&c, reqwest::Method::DELETE, &chemin, &cle, None).await;
    assert_eq!((s, code(&e)), (403, "API_KEY_ADMIN_FORBIDDEN"), "{e}");
    let (s, _) = envoyer(&c, reqwest::Method::DELETE, &chemin, &c.admin, None).await;
    assert_eq!(s, 204);
    let (s, _) = envoyer(&c, reqwest::Method::GET, &chemin, &c.admin, None).await;
    assert_eq!(s, 404);

    let o = creer(&c, "FF-DEL-O", "10.00").await;
    let o = valider(&c, &o).await;
    let (s, e) = envoyer(
        &c,
        reqwest::Method::DELETE,
        &format!("/api/v1/supplier-invoices/{}", o["id"]),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_MUST_BE_UNVALIDATED_FIRST"),
        "{e}"
    );
}

// ---------------------------------------------------------------------------
// AC-13, AC-5 — ce qu'un brouillon ne fait pas
// ---------------------------------------------------------------------------

/// AC-13, AC-5 (4) à (6) — un brouillon ne s'annule pas, ne se paie pas, n'a pas
/// de règlement à annuler, n'entre pas dans un lot (`failed[]`, jamais `500`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_draft_is_neither_cancelled_paid_nor_batched(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-N", "60.00").await;
    let id = b["id"].clone();
    let avant = nb_ecritures(&pool, c.seeded.company_id).await;

    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_DRAFT"), "M18 : {e}");
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/pay"),
        json!({ "settlementType": "bank_transfer", "bankAccountId": c.f.bank_account_id,
                "paymentDate": "2026-06-20" }),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "ILLEGAL_STATE_TRANSITION"), "{e}");
    let (s, e) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
        json!({}),
    )
    .await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_NOT_PAID"), "{e}");
    let (s, lot) = post(
        &c,
        "/api/v1/payment-batches",
        json!({ "bankAccountId": c.f.bank_account_id, "requestedExecutionDate": "2026-06-25",
                "supplierInvoiceIds": [id] }),
    )
    .await;
    assert!(s == 200 || s == 201, "jamais un 500 : {s} {lot}");
    let failed = lot["failed"].as_array().expect("failed[]");
    assert_eq!(failed.len(), 1, "{lot}");
    assert_eq!(failed[0]["errorCode"], "SUPPLIER_INVOICE_NOT_OPEN", "{lot}");
    assert_eq!(nb_ecritures(&pool, c.seeded.company_id).await, avant);
}

/// AC-5 (1) à (3) — la réconciliation : un brouillon au montant exact d'un débit
/// n'est pas proposé (M12a) ; il ne masque pas la règle qui couvre le débit
/// (M12c) ; l'accepter rend `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, motif
/// `supplier_invoice_not_open`, dans `failed[]`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reconciliation_ignores_drafts(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-R", "100.00").await;
    let tx = fournisseur_rapproche::transaction_en_attente(
        &pool,
        &c.seeded,
        c.f.bank_account_id,
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        None,
        dec!(-100.00),
        None,
        Some("Fournisseur rapproché SA"),
    )
    .await;
    let tx_voisin = fournisseur_rapproche::transaction_en_attente(
        &pool,
        &c.seeded,
        c.f.bank_account_id,
        NaiveDate::from_ymd_opt(2026, 6, 21).unwrap(),
        None,
        dec!(-100.03),
        None,
        Some("Fournisseur rapproché SA"),
    )
    .await;
    let (s, _) = post(
        &c,
        "/api/v1/reconciliation/rules",
        json!({ "label": "Fournisseur", "matchType": "counterparty_contains",
                "matchValue": "rapproché", "counterpartyAccountId": c.seeded.accounts["4000"] }),
    )
    .await;
    assert_eq!(s, 201);
    let (s, props) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!(
            "/api/v1/reconciliation/proposals?bankAccountId={}",
            c.f.bank_account_id
        ),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200);
    for p in props["proposals"].as_array().unwrap() {
        let cands = p["candidates"].as_array().unwrap();
        assert!(
            cands.iter().all(|k| k["supplierInvoiceId"] != b["id"]),
            "M12a : un brouillon n'est jamais proposé : {p}"
        );
        if p["bankTransactionId"] == tx_voisin {
            assert!(
                cands.iter().any(|k| k["candidateType"] == "rule"),
                "M12c : un brouillon ne masque pas la règle : {p}"
            );
        }
    }
    let (s, r) = post(
        &c,
        "/api/v1/reconciliation/accept",
        json!({ "bankAccountId": c.f.bank_account_id, "proposals": [{
            "type": "supplier_invoice", "bankTransactionId": tx, "supplierInvoiceId": b["id"] }] }),
    )
    .await;
    assert_eq!(s, 200, "{r}");
    assert_eq!(
        r["failed"][0]["errorCode"], "RECONCILIATION_INVOICE_NOT_ELIGIBLE",
        "{r}"
    );
    assert_eq!(
        r["failed"][0]["details"]["reason"], "supplier_invoice_not_open",
        "{r}"
    );
}

// ---------------------------------------------------------------------------
// AC-14 — #853, l'échéance
// ---------------------------------------------------------------------------

/// AC-14 — une échéance antérieure à la date est refusée à la création, au
/// `PUT` et à la complétion (document sans QR et avec QR), `details` aux dates
/// ISO ; une échéance égale passe ; le refus passe après l'échelle des lignes et
/// avant les coordonnées (M14).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn due_date_before_invoice_date_is_refused(pool: MySqlPool) {
    let c = setup(&pool).await;
    let mut avant = corps(&c, "FF-E", "10.00");
    avant["dueDate"] = json!("2026-06-14");
    let (s, e) = post(&c, "/api/v1/supplier-invoices", avant.clone()).await;
    assert_eq!(
        (s, code(&e)),
        (400, "SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE"),
        "M14 : {e}"
    );
    assert_eq!(e["error"]["details"]["invoiceDate"], "2026-06-15");
    assert_eq!(e["error"]["details"]["dueDate"], "2026-06-14");

    // Après l'échelle des lignes…
    let mut echelle = avant.clone();
    echelle["lines"][0]["quantity"] = json!("1.00001");
    let (s, e) = post(&c, "/api/v1/supplier-invoices", echelle).await;
    assert_eq!((s, code(&e)), (400, "VALIDATION_ERROR"), "{e}");
    // … avant les coordonnées.
    let mut coord = avant.clone();
    coord["creditorIban"] = json!("CH00");
    let (s, e) = post(&c, "/api/v1/supplier-invoices", coord).await;
    assert_eq!(
        (s, code(&e)),
        (400, "SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE"),
        "{e}"
    );

    let mut egale = corps(&c, "FF-E2", "10.00");
    egale["dueDate"] = json!("2026-06-15");
    let (s, b) = post(&c, "/api/v1/supplier-invoices", egale).await;
    assert_eq!(s, 201, "{b}");

    let mut put = avant.clone();
    put["version"] = b["version"].clone();
    let (s, e) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{}", b["id"]),
        &c.admin,
        Some(put),
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (400, "SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE"),
        "{e}"
    );

    let sans_qr = document(&pool, c.seeded.company_id, "e1", None).await;
    let (s, e) = post(
        &c,
        &format!("/api/v1/imported-supplier-invoices/{sans_qr}/complete"),
        avant.clone(),
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (400, "SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE"),
        "{e}"
    );
    let avec_qr = document(
        &pool,
        c.seeded.company_id,
        "e2",
        Some(("CH9300762011623852957", dec!(10.00), "CHF")),
    )
    .await;
    let mut sans_coord = avant;
    sans_coord.as_object_mut().unwrap().remove("creditorIban");
    let (s, e) = post(
        &c,
        &format!("/api/v1/imported-supplier-invoices/{avec_qr}/complete"),
        sans_coord,
    )
    .await;
    assert_eq!(
        (s, code(&e)),
        (400, "SUPPLIER_INVOICE_DUE_BEFORE_INVOICE_DATE"),
        "{e}"
    );
}

/// AC-14 — une facture existante dont l'échéance précède la date reste lisible
/// et payable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_existing_invoice_due_before_its_date_stays_payable(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-ANC", "30.00").await;
    let v = valider(&c, &b).await;
    sqlx::query("UPDATE supplier_invoices SET due_date = '2026-06-01' WHERE id = ?")
        .bind(v["id"].as_i64().unwrap())
        .execute(&pool)
        .await
        .unwrap();
    let (s, g) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!("/api/v1/supplier-invoices/{}", v["id"]),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200);
    assert_eq!(g["dueDate"], "2026-06-01");
    let (s, p) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{}/pay", v["id"]),
        json!({ "settlementType": "bank_transfer", "bankAccountId": c.f.bank_account_id,
                "paymentDate": "2026-06-20" }),
    )
    .await;
    assert_eq!(s, 200, "{p}");
    assert_eq!(p["status"], "paid");
}

// ---------------------------------------------------------------------------
// AC-15 — le filtre de la liste
// ---------------------------------------------------------------------------

/// AC-15 — `?status=` filtre, `total` compte le filtre ; sans paramètre, tout ;
/// valeur inconnue ou vide → `400 VALIDATION_ERROR` (M17).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn list_filters_by_status(pool: MySqlPool) {
    let c = setup(&pool).await;
    creer(&c, "FF-L1", "10.00").await;
    let o = creer(&c, "FF-L2", "11.00").await;
    valider(&c, &o).await;
    let x = creer(&c, "FF-L3", "12.00").await;
    let x = valider(&c, &x).await;
    let (s, _) = post(
        &c,
        &format!("/api/v1/supplier-invoices/{}/cancel", x["id"]),
        json!({}),
    )
    .await;
    assert_eq!(s, 200);
    let lire = |q: &'static str| {
        let c = &c;
        async move {
            envoyer(
                c,
                reqwest::Method::GET,
                &format!("/api/v1/supplier-invoices{q}"),
                &c.admin,
                None,
            )
            .await
        }
    };
    let (s, l) = lire("").await;
    assert_eq!((s, l["total"].clone()), (200, json!(3)));
    let (s, l) = lire("?status=draft").await;
    assert_eq!((s, l["total"].clone()), (200, json!(1)), "M17 : {l}");
    assert_eq!(l["items"][0]["status"], "draft");
    let (s, l) = lire("?status=draft,open").await;
    assert_eq!((s, l["total"].clone()), (200, json!(2)), "{l}");
    let (s, e) = lire("?status=ouverte").await;
    assert_eq!((s, code(&e)), (400, "VALIDATION_ERROR"), "{e}");
    let (s, e) = lire("?status=").await;
    assert_eq!((s, code(&e)), (400, "VALIDATION_ERROR"), "{e}");
}

// ---------------------------------------------------------------------------
// AC-4 — l'export et les fournisseurs proposés
// ---------------------------------------------------------------------------

/// AC-4 — l'export global d'une base qui porte un brouillon s'écrit ; AC-5 (8)
/// — les fournisseurs proposés par l'IBAN comptent le brouillon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_and_supplier_candidates_with_a_draft(pool: MySqlPool) {
    let c = setup(&pool).await;
    creer(&c, "FF-X", "10.00").await;
    let resp = c
        .app
        .client
        .get(c.app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&c.admin)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "l'export s'écrit avec un brouillon");
    let doc = document(
        &pool,
        c.seeded.company_id,
        "cand",
        Some((fournisseur_rapproche::IBAN_CREANCIER, dec!(10.00), "CHF")),
    )
    .await;
    let (s, k) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!("/api/v1/imported-supplier-invoices/{doc}/supplier-candidates"),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200);
    assert_eq!(k["candidates"][0]["id"], c.f.supplier_id, "{k}");
}

/// AC-8, limite L7 — un document lié **avant la 0.17.0** (`link_origin` nul)
/// n'impose rien au `PUT`, même s'il porte un QR : la facture dévalidée se
/// modifie comme une saisie.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_of_a_draft_with_a_pre_017_link_imposes_nothing(pool: MySqlPool) {
    let c = setup(&pool).await;
    let doc = document(
        &pool,
        c.seeded.company_id,
        "l7",
        Some(("CH9300762011623852957", dec!(100.00), "CHF")),
    )
    .await;
    let mut body = corps(&c, "FF-L7", "100.00");
    body.as_object_mut().unwrap().remove("creditorIban");
    let (s, b) = post(
        &c,
        &format!("/api/v1/imported-supplier-invoices/{doc}/complete"),
        body.clone(),
    )
    .await;
    assert_eq!(s, 200, "{b}");
    sqlx::query("UPDATE imported_supplier_invoices SET link_origin = NULL WHERE id = ?")
        .bind(doc)
        .execute(&pool)
        .await
        .unwrap();
    let mut libre = corps(&c, "FF-L7", "120.00");
    libre["version"] = b["version"].clone();
    let (s, p) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{}", b["id"]),
        &c.admin,
        Some(libre),
    )
    .await;
    assert_eq!(s, 200, "L7 : {p}");
    assert_eq!(p["creditorIban"], fournisseur_rapproche::IBAN_CREANCIER);
    assert_eq!(p["sourceDocument"]["linkOrigin"], Value::Null, "{p}");
}

/// AC-17 — garde de table seulement : un brouillon créé par l'API porte le statut
/// `draft`, et ce code figure parmi les statuts connus de l'outil MCP. Ce test
/// n'appelle **pas** l'outil : la lecture d'un brouillon par l'outil (`statut:
/// "draft"`, `ecritureAchatId` présent et nul) est prouvée par
/// `mcp_outils_e2e.rs::un_brouillon_lu_par_les_outils` (revue P1, A-2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_mcp_service_reads_a_draft(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "FF-MCP", "10.00").await;
    let statut: String = sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(b["id"].as_i64().unwrap())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "draft");
    assert!(
        kesh_api::mcp::outils::commun::STATUTS_FACTURE_FOURNISSEUR
            .iter()
            .any(|(code, _)| *code == "draft"),
        "le statut `draft` est connu de l'outil MCP"
    );
}

/// AC-9 — **non-régression de route** (étape 0 de la 4a, validation P5, P2) :
/// le `DELETE` HTTP d'un brouillon aboutit (`204`, jamais `500`) pendant un
/// rattachement concurrent, rejoué à la main jusqu'à la sentinelle puis conclu.
/// Sous l'ordre prescrit, aucun interblocage ne se forme : ce test est vert avec
/// ou sans l'enveloppe de rejeu et **ne prouve rien sur elle** ; le mutant
/// M20-quater est tué au dépôt (`deleting_a_draft_during_an_attachment_does_not_deadlock`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn delete_route_during_an_attachment_ends_in_204(pool: MySqlPool) {
    use kesh_db::entities::imported_supplier_invoice::LinkOrigin;
    use kesh_db::test_fixtures::attendre_une_requete_en_cours;

    let c = setup(&pool).await;
    let company = c.seeded.company_id;
    let b = creer(&c, "FF-RT", "10.00").await;
    let id = b["id"].as_i64().unwrap();
    let doc = document(&pool, company, "rt1", None).await;

    let mut a = pool.begin().await.unwrap();
    for (sql, cle) in [
        (
            "SELECT id FROM imported_supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
            doc,
        ),
        (
            "SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
            id,
        ),
    ] {
        let _: Option<i64> = sqlx::query_scalar(sql)
            .bind(cle)
            .bind(company)
            .fetch_optional(&mut *a)
            .await
            .unwrap();
    }
    let _: Option<i64> = sqlx::query_scalar("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(company)
        .fetch_optional(&mut *a)
        .await
        .unwrap();

    let url = c.app.url(&format!("/api/v1/supplier-invoices/{id}"));
    let client = c.app.client.clone();
    let jeton = c.admin.clone();
    let route = tokio::spawn(async move {
        client
            .delete(url)
            .bearer_auth(jeton)
            .send()
            .await
            .unwrap()
            .status()
            .as_u16()
    });
    let vu = attendre_une_requete_en_cours(
        &pool,
        &["FROM supplier_invoices WHERE id", "FOR UPDATE"],
        || route.is_finished(),
    )
    .await;
    assert!(vu, "le DELETE devait attendre la facture");
    imported_supplier_invoices::mark_completed(&mut a, company, doc, id, LinkOrigin::Attachment)
        .await
        .expect("le rattachement aboutit");
    a.commit().await.unwrap();
    let status = route.await.unwrap();
    assert_eq!(status, 204, "jamais 500");
}

// ---------------------------------------------------------------------------
// Story v0172-annulees (#868) — repasser en brouillon une annulée, par HTTP
// ---------------------------------------------------------------------------

async fn unvalidate_route(c: &Ctx, pool: &MySqlPool, id: i64) -> (u16, Value) {
    let version: i32 = sqlx::query_scalar("SELECT version FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    post(
        c,
        &format!("/api/v1/supplier-invoices/{id}/unvalidate"),
        json!({ "version": version }),
    )
    .await
}

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// Une facture fournisseur validée (montant sans TVA) par le dépôt ; rend son id.
async fn validee_depot(pool: &MySqlPool, c: &Ctx, numero: &str, montant: Decimal) -> (i64, i64) {
    let v = kesh_db::repositories::supplier_invoices::create_validated(
        pool,
        kesh_db::entities::NewSupplierInvoice {
            company_id: c.seeded.company_id,
            contact_id: c.f.supplier_id,
            supplier_invoice_number: Some(numero.into()),
            invoice_date: jour(2026, 6, 15),
            due_date: None,
            creditor_iban: Some(fournisseur_rapproche::IBAN_CREANCIER.into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![kesh_db::entities::NewSupplierInvoiceLine {
                description: "Fourniture".into(),
                quantity: dec!(1),
                unit_price: montant,
                vat_rate: dec!(0),
                expense_account_id: c.seeded.accounts["4000"],
            }],
        },
        c.seeded.admin_user_id,
    )
    .await
    .expect("facture validée");
    (v.invoice.id, v.invoice.purchase_journal_entry_id.unwrap())
}

async fn annuler_depot(pool: &MySqlPool, c: &Ctx, id: i64) {
    kesh_db::repositories::supplier_invoices::cancel(
        pool,
        c.seeded.company_id,
        id,
        c.seeded.admin_user_id,
    )
    .await
    .expect("annulation");
}

/// Les cinq champs neufs de `details` (AC-8), toujours présents.
fn details_neufs(e: &Value) -> [Value; 5] {
    let d = &e["error"]["details"];
    for k in [
        "documentId",
        "documentNumber",
        "batchId",
        "transactionDate",
        "entryId",
        "entryNumber",
        "entryDate",
    ] {
        assert!(d.get(k).is_some(), "details.{k} toujours présent : {e}");
    }
    [
        d["batchId"].clone(),
        d["transactionDate"].clone(),
        d["entryId"].clone(),
        d["entryNumber"].clone(),
        d["entryDate"].clone(),
    ]
}

/// T7 (AC-8) — la route `unvalidate` : 200 sur une annulée (les deux écritures
/// supprimées) ; puis un 409 par variante de `SupplierUnvalidationBlocker`,
/// `details` complets, message fr-CH exact, arguments substitués (catalogue de
/// l'harnais chargé). Décor unique (V5-4) : aucun refus d'exercice ici.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unvalidate_route_redrafts_a_cancelled_invoice_and_names_what_to_undo(pool: MySqlPool) {
    let c = setup(&pool).await;
    let co = c.seeded.company_id;

    // 200 — annulée → brouillon.
    let (id, achat) = validee_depot(&pool, &c, "FF-R7", dec!(20)).await;
    annuler_depot(&pool, &c, id).await;
    let avant = nb_ecritures(&pool, co).await;
    let (s, b) = unvalidate_route(&c, &pool, id).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["status"], "draft");
    assert_eq!(b.get("purchaseJournalEntryId"), Some(&Value::Null));
    assert_eq!(
        nb_ecritures(&pool, co).await,
        avant - 2,
        "achat et contre-passation"
    );
    let _ = achat;

    // SUPPLIER_INVOICE_PAID — règlement non rapproché.
    let (paid, _, _) = fournisseur_rapproche::facture_payee(
        &pool,
        &c.seeded,
        c.f,
        "FF-R7P",
        jour(2026, 6, 15),
        dec!(30),
    )
    .await;
    let (s, e) = unvalidate_route(&c, &pool, paid).await;
    assert_eq!((s, code(&e)), (409, "SUPPLIER_INVOICE_PAID"), "{e}");
    assert_eq!(
        e["error"]["message"],
        "Cette facture fournisseur est payée : annulez d'abord son règlement (bouton « Annuler le règlement » de cette fiche), puis repassez-la en brouillon."
    );
    assert_eq!(
        details_neufs(&e),
        [
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );
    // Story 33-1b (#926) : la forme courte, le numéro interne d'abord.
    assert_eq!(e["error"]["details"]["documentNumber"], "FF-2020-002");

    // SUPPLIER_INVOICE_PAID_RECONCILED.
    let r = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &c.seeded,
        c.f,
        "FF-R7R",
        jour(2026, 6, 17),
        dec!(30),
    )
    .await;
    let (s, e) = unvalidate_route(&c, &pool, r.supplier_invoice_id).await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_PAID_RECONCILED"),
        "{e}"
    );
    assert_eq!(
        e["error"]["message"],
        "Cette facture fournisseur est payée par la transaction bancaire du 17.06.2026 : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — elle redeviendra validée —, puis repassez-la en brouillon."
    );
    assert_eq!(e["error"]["details"]["documentId"], r.bank_transaction_id);
    assert_eq!(
        details_neufs(&e),
        [
            Value::Null,
            json!("2026-06-17"),
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );

    // SUPPLIER_INVOICE_IN_PAYMENT_BATCH.
    let (lotee, _) = validee_depot(&pool, &c, "FF-R7L", dec!(11)).await;
    let outcome = kesh_db::repositories::payment_batches::create_batch(
        &pool,
        kesh_db::entities::NewPaymentBatch {
            company_id: co,
            bank_account_id: c.f.bank_account_id,
            requested_execution_date: jour(2026, 6, 25),
            supplier_invoice_ids: vec![lotee],
        },
        c.seeded.admin_user_id,
    )
    .await
    .expect("lot");
    let lot = outcome.batch.expect("lot créé").batch.id;
    let (s, e) = unvalidate_route(&c, &pool, lotee).await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_IN_PAYMENT_BATCH"),
        "{e}"
    );
    assert_eq!(
        e["error"]["message"],
        format!(
            "Cette facture figure dans le lot de paiement n° {lot}, en cours : annulez d'abord ce lot, dans Paiements fournisseurs, puis repassez-la en brouillon."
        )
    );
    assert_eq!(
        details_neufs(&e),
        [
            json!(lot),
            Value::Null,
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );

    // SUPPLIER_INVOICE_DETACHED_PAYMENT — sans transaction.
    let (detachee, _, reglement) = fournisseur_rapproche::facture_payee(
        &pool,
        &c.seeded,
        c.f,
        "FF-R7D",
        jour(2026, 6, 18),
        dec!(30),
    )
    .await;
    annuler_depot(&pool, &c, detachee).await;
    let numero: i64 = sqlx::query_scalar("SELECT entry_number FROM journal_entries WHERE id = ?")
        .bind(reglement)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (s, e) = unvalidate_route(&c, &pool, detachee).await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_DETACHED_PAYMENT"),
        "{e}"
    );
    assert_eq!(
        e["error"]["message"],
        format!(
            "Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° {numero} du 18.06.2026) reste au grand livre. Si ce paiement n'a pas eu lieu, contre-passez-le d'abord depuis sa fiche, puis repassez la facture en brouillon."
        )
    );
    assert_eq!(
        details_neufs(&e),
        [
            Value::Null,
            Value::Null,
            json!(reglement),
            json!(numero),
            json!("2026-06-18")
        ]
    );

    // SUPPLIER_INVOICE_DETACHED_PAYMENT — paiement rapproché.
    let rr = fournisseur_rapproche::reglement_rapproche(
        &pool,
        &c.seeded,
        c.f,
        "FF-R7DR",
        jour(2026, 6, 19),
        dec!(30),
    )
    .await;
    annuler_depot(&pool, &c, rr.supplier_invoice_id).await;
    let numero: i64 = sqlx::query_scalar("SELECT entry_number FROM journal_entries WHERE id = ?")
        .bind(rr.settlement_entry_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (s, e) = unvalidate_route(&c, &pool, rr.supplier_invoice_id).await;
    assert_eq!(
        (s, code(&e)),
        (409, "SUPPLIER_INVOICE_DETACHED_PAYMENT"),
        "{e}"
    );
    assert_eq!(
        e["error"]["message"],
        format!(
            "Cette facture a été annulée alors qu'elle était payée : son paiement (écriture n° {numero} du 19.06.2026) est rapproché de la transaction bancaire du 19.06.2026. Si ce paiement n'a pas eu lieu, annulez d'abord ce rapprochement, depuis le détail de l'import bancaire — il contre-passera le paiement —, puis repassez la facture en brouillon."
        )
    );
    assert_eq!(
        details_neufs(&e),
        [
            Value::Null,
            json!("2026-06-19"),
            json!(rr.settlement_entry_id),
            json!(numero),
            json!("2026-06-19")
        ]
    );

    // MATCHED_BANK_TRANSACTION — l'achat rapproché.
    let (rapprochee, achat) = validee_depot(&pool, &c, "FF-R7M", dec!(13)).await;
    let tx = fournisseur_rapproche::debit_rapproche(
        &pool,
        &c.seeded,
        c.f,
        achat,
        jour(2026, 6, 21),
        dec!(13),
    )
    .await;
    let (s, e) = unvalidate_route(&c, &pool, rapprochee).await;
    assert_eq!((s, code(&e)), (409, "MATCHED_BANK_TRANSACTION"), "{e}");
    assert_eq!(
        e["error"]["message"],
        "L'écriture d'achat de cette facture est rapprochée de la transaction bancaire du 21.06.2026 : annulez d'abord ce rapprochement, depuis le détail de l'import bancaire, puis repassez la facture en brouillon."
    );
    assert_eq!(e["error"]["details"]["documentId"], tx);
    assert_eq!(
        details_neufs(&e),
        [
            Value::Null,
            json!("2026-06-21"),
            Value::Null,
            Value::Null,
            Value::Null
        ]
    );
}

/// Story v0174-correctifs-2 (#869, AC-2.2) — le repassage en brouillon d'une
/// facture dont l'achat est daté sous le verrou de période rend `400
/// PERIOD_LOCKED` **avec** `details.lockedThrough` (la borne) et
/// `details.attempted` (la date de l'achat), `AAAA-MM-JJ`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn unvalidate_under_the_period_lock_names_its_bounds_869(pool: MySqlPool) {
    let c = setup(&pool).await;
    let (id, _) = validee_depot(&pool, &c, "FF-869", dec!(30)).await;
    sqlx::query("UPDATE companies SET books_locked_through = '2026-06-30' WHERE id = ?")
        .bind(c.seeded.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let (s, e) = unvalidate_route(&c, &pool, id).await;
    assert_eq!((s, code(&e)), (400, "PERIOD_LOCKED"), "{e}");
    assert_eq!(e["error"]["details"]["lockedThrough"], "2026-06-30", "{e}");
    assert_eq!(e["error"]["details"]["attempted"], "2026-06-15", "{e}");
}

// ---------------------------------------------------------------------------
// Story 33-1a (#926) — le numéro interne, par HTTP
// ---------------------------------------------------------------------------

/// AC17 — `internalNumber` est toujours présent, nul pour un brouillon jamais
/// validé ; `supplierInvoiceNumber` ne change ni de nom ni de valeur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn internal_number_is_in_every_response(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "ABC-1", "10.00").await;
    assert_eq!(b.get("internalNumber"), Some(&Value::Null), "{b}");
    assert_eq!(b["supplierInvoiceNumber"], "ABC-1");

    let v = valider(&c, &b).await;
    assert_eq!(v["internalNumber"], "FF-2020-001", "{v}");
    assert_eq!(v["supplierInvoiceNumber"], "ABC-1");

    let chemin = format!("/api/v1/supplier-invoices/{}", v["id"]);
    let (s, fiche) = envoyer(&c, reqwest::Method::GET, &chemin, &c.admin, None).await;
    assert_eq!(s, 200, "{fiche}");
    assert_eq!(fiche["internalNumber"], "FF-2020-001");

    let (s, liste) = envoyer(
        &c,
        reqwest::Method::GET,
        "/api/v1/supplier-invoices",
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200, "{liste}");
    assert_eq!(
        liste["items"][0]["internalNumber"], "FF-2020-001",
        "{liste}"
    );
    assert_eq!(liste["items"][0]["supplierInvoiceNumber"], "ABC-1");
}

/// AC14 — ni la création, ni la modification, ni la complétion d'un document
/// n'écrivent le numéro, même si le corps le porte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nobody_writes_the_internal_number_but_the_validation(pool: MySqlPool) {
    let c = setup(&pool).await;
    let mut body = corps(&c, "ABC-1", "10.00");
    body["internalNumber"] = json!("FF-PIRATE");
    let (s, b) = post(&c, "/api/v1/supplier-invoices", body.clone()).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["internalNumber"], Value::Null, "création : {b}");

    let mut modif = body.clone();
    modif["version"] = b["version"].clone();
    let (s, m) = envoyer(
        &c,
        reqwest::Method::PUT,
        &format!("/api/v1/supplier-invoices/{}", b["id"]),
        &c.admin,
        Some(modif),
    )
    .await;
    assert_eq!(s, 200, "{m}");
    assert_eq!(m["internalNumber"], Value::Null, "modification : {m}");

    let doc = document(&pool, c.seeded.company_id, "num-interne", None).await;
    let (s, k) = post(
        &c,
        &format!("/api/v1/imported-supplier-invoices/{doc}/complete"),
        body,
    )
    .await;
    assert_eq!(s, 200, "{k}");
    assert_eq!(k["internalNumber"], Value::Null, "complétion : {k}");

    assert_eq!(valider(&c, &m).await["internalNumber"], "FF-2020-001");
}

/// AC16 — le réglage : au défaut, préservé quand le corps l'omet, modifiable, refusé
/// en 400 hors de la forme `<préfixe>{YEAR}<séparateur>{SEQ}`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_internal_number_format_setting(pool: MySqlPool) {
    let c = setup(&pool).await;
    let chemin = "/api/v1/company/invoice-settings";
    let (s, reglages) = envoyer(&c, reqwest::Method::GET, chemin, &c.admin, None).await;
    assert_eq!(s, 200, "{reglages}");
    assert_eq!(
        reglages["supplierInternalNumberFormat"],
        "FF-{YEAR}-{SEQ:03}"
    );

    // Absent du corps : préservé, sur une valeur qui n'est pas le défaut (M16).
    let mut pose = reglages.clone();
    pose["supplierInternalNumberFormat"] = json!("LF-{YEAR}-{SEQ:02}");
    let (s, pose) = envoyer(&c, reqwest::Method::PUT, chemin, &c.admin, Some(pose)).await;
    assert_eq!(s, 200, "{pose}");
    let mut sans = pose.clone();
    sans.as_object_mut()
        .unwrap()
        .remove("supplierInternalNumberFormat");
    sans["invoiceNumberFormat"] = json!("F-{YEAR}-{SEQ:05}");
    let (s, apres) = envoyer(&c, reqwest::Method::PUT, chemin, &c.admin, Some(sans)).await;
    assert_eq!(s, 200, "{apres}");
    assert_eq!(apres["invoiceNumberFormat"], "F-{YEAR}-{SEQ:05}");
    assert_eq!(apres["supplierInternalNumberFormat"], "LF-{YEAR}-{SEQ:02}");

    for ko in [
        "",
        "FF-{YEAR}",
        "FF-{SEQ:03}",
        "FF-{YEAR}-{SEQ:11}",
        "FF-{FY}-{SEQ}",
        "FF-{FY}-{YEAR}-{SEQ}",
        "{YEAR}-{YEAR}-{SEQ}",
        "{SEQ}-{YEAR}",
        "FF-{YEAR}{SEQ}",
        "FF-{YEAR}-1{SEQ}",
        "FF-{YEAR}-{SEQ}-X",
    ] {
        let mut corps_ko = apres.clone();
        corps_ko["supplierInternalNumberFormat"] = json!(ko);
        let (s, b) = envoyer(&c, reqwest::Method::PUT, chemin, &c.admin, Some(corps_ko)).await;
        assert_eq!(s, 400, "format {ko:?} : {b}");
    }

    let mut bon = apres.clone();
    bon["supplierInternalNumberFormat"] = json!("KF-{YEAR}-{SEQ:04}");
    let (s, change) = envoyer(&c, reqwest::Method::PUT, chemin, &c.admin, Some(bon)).await;
    assert_eq!(s, 200, "{change}");
    assert_eq!(change["supplierInternalNumberFormat"], "KF-{YEAR}-{SEQ:04}");

    let b = creer(&c, "ABC-2", "10.00").await;
    assert_eq!(valider(&c, &b).await["internalNumber"], "KF-2020-0001");

    let audit: serde_json::Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'company_invoice_settings.updated' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit["after"]["supplierInternalNumberFormat"], "KF-{YEAR}-{SEQ:04}",
        "{audit}"
    );
}

/// AC19 — l'export porte le numéro et le format, pas le compteur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_export_carries_the_internal_number(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "ABC-1", "10.00").await;
    valider(&c, &b).await;
    let octets = c
        .app
        .client
        .get(c.app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&c.admin)
        .send()
        .await
        .unwrap()
        .bytes()
        .await
        .unwrap();
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(octets)).expect("ZIP");
    let mut lire = |nom: &str| {
        use std::io::Read;
        let mut texte = String::new();
        archive
            .by_name(nom)
            .unwrap_or_else(|_| panic!("{nom} absent"))
            .read_to_string(&mut texte)
            .unwrap();
        texte
    };
    let factures = lire("supplier_invoices.csv");
    assert!(
        factures
            .lines()
            .next()
            .unwrap()
            .contains("supplier_invoice_number;internal_number;")
            || factures
                .lines()
                .next()
                .unwrap()
                .contains("supplier_invoice_number,internal_number,"),
        "{factures}"
    );
    assert!(factures.contains("FF-2020-001"), "{factures}");
    assert!(
        lire("company_invoice_settings.csv").contains("supplier_internal_number_format"),
        "le format est exporté"
    );
    assert!(
        !lire("fiscal_years.csv").contains("supplier_invoice_next_number"),
        "le compteur ne l'est pas"
    );
}

// ---------------------------------------------------------------------------
// Story 33-1b (#926) — le tri, la recherche, la société, les lots
// ---------------------------------------------------------------------------

async fn liste(c: &Ctx, requete: &str) -> (u16, Value) {
    envoyer(
        c,
        reqwest::Method::GET,
        &format!("/api/v1/supplier-invoices{requete}"),
        &c.admin,
        None,
    )
    .await
}

fn numeros_de(b: &Value) -> Vec<Value> {
    b["items"]
        .as_array()
        .unwrap_or_else(|| panic!("items : {b}"))
        .iter()
        .map(|i| i["internalNumber"].clone())
        .collect()
}

/// Trois factures validées, puis leurs numéros posés en SQL pour exercer le tri : 1000
/// passe après 999 ; plus un brouillon sans numéro.
async fn trois_numeros(pool: &MySqlPool, c: &Ctx) {
    for (saisi, interne) in [
        ("A-1", "FF-2026-1000"),
        ("A-2", "FF-2026-999"),
        ("A-3", "FF-2025-1000"),
    ] {
        let b = creer(c, saisi, "10.00").await;
        let v = valider(c, &b).await;
        sqlx::query("UPDATE supplier_invoices SET internal_number = ? WHERE id = ?")
            .bind(interne)
            .bind(v["id"].as_i64())
            .execute(pool)
            .await
            .unwrap();
    }
    creer(c, "A-4", "10.00").await;
}

/// AC3 — tri naturel, sans numéro à la fin dans les deux sens ; `sort` absent : l'ordre
/// d'aujourd'hui ; valeur hors liste : 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_list_sorts_by_internal_number_naturally(pool: MySqlPool) {
    let c = setup(&pool).await;
    trois_numeros(&pool, &c).await;

    let (s, b) = liste(&c, "?sort=internalNumber&order=asc").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        numeros_de(&b),
        [
            json!("FF-2025-1000"),
            json!("FF-2026-999"),
            json!("FF-2026-1000"),
            Value::Null
        ]
    );
    let (_, b) = liste(&c, "?sort=internalNumber&order=desc").await;
    assert_eq!(
        numeros_de(&b),
        [
            json!("FF-2026-1000"),
            json!("FF-2026-999"),
            json!("FF-2025-1000"),
            Value::Null
        ]
    );
    // `order` absent : croissant.
    let (_, b) = liste(&c, "?sort=internalNumber").await;
    assert_eq!(numeros_de(&b)[0], json!("FF-2025-1000"));

    // `sort` absent : `invoice_date DESC, id DESC`, le dernier créé d'abord.
    let (_, b) = liste(&c, "").await;
    assert_eq!(b["items"][0]["supplierInvoiceNumber"], "A-4");
    assert_eq!(b["items"][3]["supplierInvoiceNumber"], "A-1");

    for mauvais in [
        "?sort=contactName",
        "?sort=internal_number",
        "?sort=internalNumber&order=up",
    ] {
        let (s, b) = liste(&c, mauvais).await;
        assert_eq!(s, 400, "{mauvais} : {b}");
    }
}

/// AC4 — `q` trouve le numéro interne, celui du fournisseur et le nom du fournisseur ; `%` et
/// `_` sont échappés ; `total` compte le filtre ; cumulable avec `status` ; blanc : aucun
/// filtre ; au-delà de 128 : 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_list_searches_numbers_and_supplier(pool: MySqlPool) {
    let c = setup(&pool).await;
    trois_numeros(&pool, &c).await;
    let total = |b: &Value| b["total"].as_i64().unwrap();

    let (s, b) = liste(&c, "?q=FF-2026-999").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(numeros_de(&b), [json!("FF-2026-999")]);
    assert_eq!(total(&b), 1, "total compte le filtre");

    let (_, b) = liste(&c, "?q=A-2").await;
    assert_eq!(
        numeros_de(&b),
        [json!("FF-2026-999")],
        "numéro du fournisseur"
    );

    let (_, b) = liste(&c, "?q=rapproch").await;
    assert_eq!(total(&b), 4, "nom du fournisseur");

    let (_, b) = liste(&c, "?q=A-4&status=open").await;
    assert_eq!(total(&b), 0, "cumulable avec le statut");
    let (_, b) = liste(&c, "?q=A-4&status=draft").await;
    assert_eq!(total(&b), 1);

    // `%` et `_` n'ont pas leur sens de joker.
    let (_, b) = liste(&c, "?q=%25").await;
    assert_eq!(total(&b), 0, "% échappé");
    let (_, b) = liste(&c, "?q=A_2").await;
    assert_eq!(total(&b), 0, "_ échappé");

    let (s, b) = liste(&c, "?q=%20%20").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(total(&b), 4, "q blanc : aucun filtre");

    let (s, b) = liste(&c, &format!("?q={}", "x".repeat(129))).await;
    assert_eq!(s, 400, "{b}");
    let (s, _) = liste(&c, &format!("?q={}", "x".repeat(128))).await;
    assert_eq!(s, 200);
}

/// AC5 — `q` et `sort` ne franchissent pas la société.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_search_stays_in_the_company(pool: MySqlPool) {
    let c = setup(&pool).await;
    creer(&c, "CHEZ-A", "10.00").await;
    let autre = kesh_db::test_fixtures::seed_stub_company_only(&pool)
        .await
        .unwrap();
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur de B', TRUE) RETURNING id",
    )
    .bind(autre)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, \
         internal_number, status, invoice_date, total_amount) \
         VALUES (?, ?, 'CHEZ-B', 'FF-2026-777', 'draft', '2026-03-01', 0)",
    )
    .bind(autre)
    .bind(contact)
    .execute(&pool)
    .await
    .unwrap();

    for q in ["CHEZ-B", "FF-2026-777", "Fournisseur%20de%20B"] {
        let (s, b) = liste(&c, &format!("?q={q}")).await;
        assert_eq!(s, 200, "{b}");
        assert_eq!(b["total"], 0, "{q} : l'autre société ne se voit pas ({b})");
    }
    let (_, b) = liste(&c, "?sort=internalNumber").await;
    assert_eq!(b["total"], 1, "{b}");
}

/// AC16 — toutes les réponses d'un lot rendent les numéros de chaque facture ; `#id`
/// n'est plus la seule chose qu'un lot sait de ses factures.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_payment_batch_names_its_invoices(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "ABC-LOT", "10.00").await;
    let v = valider(&c, &b).await;
    let (s, lot) = post(
        &c,
        "/api/v1/payment-batches",
        json!({
            "bankAccountId": c.f.bank_account_id,
            "requestedExecutionDate": "2026-06-20",
            "supplierInvoiceIds": [v["id"]],
        }),
    )
    .await;
    assert_eq!(s, 200, "{lot}");
    let controle = |lot: &Value, contexte: &str| {
        let item = &lot["items"][0];
        assert_eq!(
            item["internalNumber"], v["internalNumber"],
            "{contexte} : {lot}"
        );
        assert_eq!(
            item["supplierInvoiceNumber"], "ABC-LOT",
            "{contexte} : {lot}"
        );
    };
    controle(&lot["batch"], "création");
    let id = lot["batch"]["id"].as_i64().unwrap();
    let (_, detail) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!("/api/v1/payment-batches/{id}"),
        &c.admin,
        None,
    )
    .await;
    controle(&detail, "détail");
    let (s, annule) = post(
        &c,
        &format!("/api/v1/payment-batches/{id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(s, 200, "{annule}");
    controle(&annule, "annulation");

    // La confirmation (revue P1, A-F3) : le lot annulé rend la facture à un second lot.
    let (s, lot) = post(
        &c,
        "/api/v1/payment-batches",
        json!({
            "bankAccountId": c.f.bank_account_id,
            "requestedExecutionDate": "2026-06-20",
            "supplierInvoiceIds": [v["id"]],
        }),
    )
    .await;
    assert_eq!(s, 200, "{lot}");
    let id = lot["batch"]["id"].as_i64().unwrap();
    let (s, confirme) = post(
        &c,
        &format!("/api/v1/payment-batches/{id}/confirm"),
        json!({ "paymentDate": "2026-06-20" }),
    )
    .await;
    assert_eq!(s, 200, "{confirme}");
    controle(&confirme, "confirmation");
}

/// AC14 — postes ouverts : `document.number` garde le numéro du fournisseur, le numéro
/// interne s'ajoute à côté, et `kesh_postes_ouverts` suit au niveau du service.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_open_items_document_carries_both_numbers(pool: MySqlPool) {
    let c = setup(&pool).await;
    let b = creer(&c, "ABC-PO", "10.00").await;
    let v = valider(&c, &b).await;
    let (s, body) = envoyer(
        &c,
        reqwest::Method::GET,
        &format!("/api/v1/accounts/{}/open-items", c.seeded.accounts["2000"]),
        &c.admin,
        None,
    )
    .await;
    assert_eq!(s, 200, "{body}");
    let doc = &body["items"][0]["document"];
    assert_eq!(doc["type"], "supplierInvoice", "{body}");
    assert_eq!(doc["number"], "ABC-PO", "la valeur du champ ne change pas");
    assert_eq!(doc["internalNumber"], v["internalNumber"]);
}
