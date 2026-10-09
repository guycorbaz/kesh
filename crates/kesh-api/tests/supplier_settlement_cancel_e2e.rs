//! Annuler le règlement d'une facture fournisseur par l'API — Story 25-3-a-2 (#414) —
//! et annuler la facture elle-même, même payée — Story 25-3-c (#454) ; puis le
//! refus « contrepartie = compte créanciers » du paiement et des lots, par HTTP
//! — Story 15-6b (#474), revue de code P1.
//!
//! ⚠️ Premier fichier de tests HTTP des factures fournisseurs : le montage de
//! l'application est repris de `invoice_echeancier_e2e.rs` (chaque binaire de
//! test est un crate à part). Les règles métier sont prouvées côté dépôt
//! (`kesh-db/tests/supplier_invoices_repository.rs`) ; ici, ce qui ne se voit
//! qu'à travers HTTP : les champs de lecture, les rôles, les codes d'erreur.
//!
//! Pré-requis : MariaDB démarré.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice};
use kesh_db::repositories::{contacts, supplier_invoices};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

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
    body["accessToken"].as_str().unwrap().to_string()
}

/// Une facture fournisseur de 100.00 (TVA 0), ouverte ; rend son id.
async fn open_invoice(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    // Le seed ne pose pas de compte fournisseurs par défaut : montage repris de
    // `kesh-db/tests/supplier_invoices_repository.rs::setup`.
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let supplier = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Fournisseur SA".into(),
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    supplier_invoices::create(
        pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: supplier,
            supplier_invoice_number: Some("FF-1".into()),
            invoice_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            due_date: None,
            creditor_iban: None,
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur")
    .invoice
    .id
}

async fn get(app: &TestApp, token: &str, path: &str) -> reqwest::Response {
    app.client
        .get(app.url(path))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
}

async fn post(app: &TestApp, token: &str, path: &str, body: Value) -> reqwest::Response {
    app.client
        .post(app.url(path))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// ⛔ **Le parcours nominal, par HTTP** : `pay` rend déjà les champs de lecture
/// (l'écran remplace son état par cette réponse) ; l'annulation rend la facture
/// ouverte, colonnes vidées, et l'écriture inverse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pay_then_cancel_the_settlement(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let v: Value = get(&app, &token, &format!("/api/v1/supplier-invoices/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(
        v["settlementCancellable"], false,
        "ouverte : rien à annuler"
    );
    assert_eq!(v["settlementCancelBlockedBy"], "SUPPLIER_INVOICE_NOT_PAID");

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/pay"),
        json!({
            "settlementType": "internal_account",
            "accountId": seeded.accounts["1000"],
            "paymentDate": "2026-06-20"
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["status"], "paid");
    assert_eq!(
        v["settlementCancellable"], true,
        "la réponse de `pay` porte les champs : l'écran remplace son état par elle"
    );
    assert!(v["lastConfirmedBatch"].is_null());

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/settlement/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let v: Value = resp.json().await.unwrap();
    assert!(v["reversalJournalEntryId"].as_i64().unwrap() > 0);
    assert_eq!(v["invoice"]["status"], "open");
    assert!(v["invoice"]["settlementJournalEntryId"].is_null());
    assert!(v["invoice"]["paidAt"].is_null());
    assert_eq!(
        v["invoice"]["settlementCancelBlockedBy"],
        "SUPPLIER_INVOICE_NOT_PAID"
    );
}

/// Rôles et refus : Consultation LIT (200) mais n'annule pas (403) ; une
/// facture étrangère rend 404 ; une facture ouverte, 409 avec son code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_settlement_roles_and_refusals(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "lecteur".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Consultation,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let lecteur = login(&app, "lecteur", "password123").await;

    let path = format!("/api/v1/supplier-invoices/{id}/settlement/cancel");
    let resp = post(&app, &token, &path, json!({})).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "SUPPLIER_INVOICE_NOT_PAID",
        "got {body:?}"
    );

    supplier_invoices::pay(
        &pool,
        seeded.company_id,
        id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    assert_eq!(
        get(&app, &lecteur, &format!("/api/v1/supplier-invoices/{id}"))
            .await
            .status(),
        200
    );
    assert_eq!(post(&app, &lecteur, &path, json!({})).await.status(), 403);
    assert_eq!(
        post(
            &app,
            &token,
            &format!("/api/v1/supplier-invoices/{}/settlement/cancel", id + 9999),
            json!({})
        )
        .await
        .status(),
        404
    );
}

/// ⛔ **Story 25-3-c — annuler une facture PAYÉE, par HTTP.** Le GET dit
/// « annulable » ; l'annulation rend la facture relue avec ses champs :
/// `cancelled`, règlement détaché, « déjà annulée » comme motif.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pay_then_cancel_the_invoice(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let v: Value = get(&app, &token, &format!("/api/v1/supplier-invoices/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(v["cancellable"], true, "ouverte : annulable");
    assert!(v["cancelBlockedBy"].is_null());

    supplier_invoices::pay(
        &pool,
        seeded.company_id,
        id,
        SettlementChoice::InternalAccount {
            account_id: seeded.accounts["1000"],
        },
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let v: Value = get(&app, &token, &format!("/api/v1/supplier-invoices/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(v["cancellable"], true, "payée : annulable aussi");

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/cancel"),
        json!({}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["status"], "cancelled");
    assert!(v["settlementJournalEntryId"].is_null(), "règlement détaché");
    assert!(v["paidAt"].is_null());
    assert_eq!(
        v["cancellable"], false,
        "la réponse est relue avec ses champs de lecture"
    );
    assert_eq!(v["cancelBlockedBy"], "SUPPLIER_INVOICE_CANCELLED");
}

/// Story 25-3-c — rôles et refus : Consultation 403 ; facture étrangère 404 ;
/// seconde annulation 409 avec son code ; compte de charge archivé, 400 qui
/// NOMME le compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancel_invoice_roles_and_refusals(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "lecteur".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Consultation,
            active: true,
            company_id: seeded.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let lecteur = login(&app, "lecteur", "password123").await;
    let path = format!("/api/v1/supplier-invoices/{id}/cancel");

    assert_eq!(post(&app, &lecteur, &path, json!({})).await.status(), 403);
    assert_eq!(
        post(
            &app,
            &token,
            &format!("/api/v1/supplier-invoices/{}/cancel", id + 9999),
            json!({})
        )
        .await
        .status(),
        404
    );

    // Compte de charge archivé : le socle refuse, en NOMMANT le compte.
    let charge = seeded.accounts["4000"];
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(charge)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::accounts::archive(&pool, charge, version, seeded.admin_user_id)
        .await
        .unwrap();
    let v: Value = get(&app, &token, &format!("/api/v1/supplier-invoices/{id}"))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(v["cancelBlockedBy"], "ACCOUNT_ARCHIVED");
    assert_eq!(v["cancelBlockedLabel"], "4000");
    let resp = post(&app, &token, &path, json!({})).await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_ARCHIVED", "got {body:?}");

    // Compte réactivé : l'annulation passe ; la seconde est refusée, avec son code.
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(charge)
        .fetch_one(&pool)
        .await
        .unwrap();
    kesh_db::repositories::accounts::reactivate(
        &pool,
        charge,
        version,
        seeded.admin_user_id,
        false,
    )
    .await
    .unwrap();
    assert_eq!(post(&app, &token, &path, json!({})).await.status(), 200);
    let resp = post(&app, &token, &path, json!({})).await;
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "SUPPLIER_INVOICE_CANCELLED",
        "got {body:?}"
    );
}

// ---------------------------------------------------------------------------
// Story 15-6b (#474) — revue de code P1, finding L5 = E3 : le refus
// « contrepartie = compte créanciers » à travers HTTP, sur les trois routes
// que `docs/api-external.md` documente et que seuls le dépôt et le mapping
// `AppError` exerçaient jusque-là.
// ---------------------------------------------------------------------------

/// Un compte bancaire source (IBAN valide), relié au compte du grand livre
/// `ledger` ; rend son id.
async fn bank_linked_to(pool: &MySqlPool, seeded: &SeededCompany, ledger: i64) -> i64 {
    let bank = kesh_db::repositories::bank_accounts::create(
        pool,
        kesh_db::entities::NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque Source".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap();
    relink(pool, bank.id, ledger).await;
    bank.id
}

async fn relink(pool: &MySqlPool, bank_id: i64, ledger: i64) {
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ledger)
        .bind(bank_id)
        .execute(pool)
        .await
        .unwrap();
}

/// Le compte de banque 1020, absent du seed.
async fn ledger_1020(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Une facture ouverte payable par virement (IBAN du créancier posé).
async fn open_invoice_with_iban(pool: &MySqlPool, seeded: &SeededCompany) -> i64 {
    let id = open_invoice(pool, seeded).await;
    sqlx::query("UPDATE supplier_invoices SET creditor_iban = ? WHERE id = ?")
        .bind("CH5604835012345678009")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    id
}

async fn status_of(pool: &MySqlPool, id: i64) -> String {
    sqlx::query_scalar("SELECT status FROM supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `POST /supplier-invoices/{id}/pay` par le compte interne 2000 — le compte
/// créanciers de la facture : **400**, code et `details` du refus, message qui
/// nomme le compte ; la facture reste ouverte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pay_through_the_payable_account_itself_is_a_400(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice(&pool, &seeded).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let payable = seeded.accounts["2000"];

    let resp = post(
        &app,
        &token,
        &format!("/api/v1/supplier-invoices/{id}/pay"),
        json!({
            "settlementType": "internal_account",
            "accountId": payable,
            "paymentDate": "2026-06-20"
        }),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        body["error"]["details"],
        json!({
            "accountId": payable,
            "accountNumber": "2000",
            "claim": "payable",
            "role": "counterparty",
        })
    );
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("2000") && msg.contains("compte créanciers de cette facture"),
        "message fr attendu : {msg}"
    );
    assert!(!msg.contains('$') && !msg.contains('\u{2068}'), "{msg}");
    assert_eq!(status_of(&pool, id).await, "open");
}

/// `POST /payment-batches` avec un compte bancaire relié au 2000 : **200**,
/// aucun lot, la facture dans `failed[]` avec le code et les `details` du
/// refus (dont `bankAccountId`) — patron FailedProposal, jamais un 4xx global.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_batch_with_a_bank_linked_to_the_payable_reports_the_invoice_in_failed(
    pool: MySqlPool,
) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice_with_iban(&pool, &seeded).await;
    let payable = seeded.accounts["2000"];
    let bank_id = bank_linked_to(&pool, &seeded, payable).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = post(
        &app,
        &token,
        "/api/v1/payment-batches",
        json!({
            "bankAccountId": bank_id,
            "requestedExecutionDate": "2026-07-01",
            "supplierInvoiceIds": [id]
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["batch"].is_null(), "aucun lot : {body}");
    assert_eq!(body["failed"].as_array().unwrap().len(), 1);
    assert_eq!(body["failed"][0]["supplierInvoiceId"], id);
    assert_eq!(
        body["failed"][0]["errorCode"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    assert_eq!(
        body["failed"][0]["details"],
        json!({
            "bankAccountId": bank_id,
            "accountId": payable,
            "accountNumber": "2000",
            "claim": "payable",
            "role": "counterparty",
        })
    );
}

/// `POST /payment-batches/{id}/confirm` après un relien du compte bancaire au
/// 2000 : **400**, `details` portant `paymentBatchId` et `supplierInvoiceId`,
/// message qui nomme la facture ; le lot reste `generated`, la facture
/// `open`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn confirm_batch_after_a_relink_to_the_payable_is_a_contextual_400(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let id = open_invoice_with_iban(&pool, &seeded).await;
    let l1020 = ledger_1020(&pool, &seeded).await;
    let bank_id = bank_linked_to(&pool, &seeded, l1020).await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let resp = post(
        &app,
        &token,
        "/api/v1/payment-batches",
        json!({
            "bankAccountId": bank_id,
            "requestedExecutionDate": "2026-07-01",
            "supplierInvoiceIds": [id]
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let batch_id = body["batch"]["id"].as_i64().expect("lot créé");

    let payable = seeded.accounts["2000"];
    relink(&pool, bank_id, payable).await;
    let resp = post(
        &app,
        &token,
        &format!("/api/v1/payment-batches/{batch_id}/confirm"),
        json!({ "paymentDate": "2026-07-02" }),
    )
    .await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"],
        "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT"
    );
    let details = &body["error"]["details"];
    assert_eq!(details["paymentBatchId"], batch_id);
    assert_eq!(details["supplierInvoiceId"], id);
    assert_eq!(details["accountId"], payable);
    assert_eq!(details["claim"], "payable");
    assert_eq!(details["role"], "counterparty");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("FF-1") && msg.contains("2000") && msg.contains("lot"),
        "message de lot attendu : {msg}"
    );
    assert!(!msg.contains('$') && !msg.contains('\u{2068}'), "{msg}");
    let status: String = sqlx::query_scalar("SELECT status FROM payment_batches WHERE id = ?")
        .bind(batch_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "generated");
    assert_eq!(status_of(&pool, id).await, "open");
}
