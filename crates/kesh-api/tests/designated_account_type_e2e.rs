//! Story 35-3 (#537) — un compte désigné aux réglages de facturation et retypé
//! depuis ne reçoit plus d'écriture : la validation d'une facture et celle d'une
//! facture fournisseur rendent **400 `DESIGNATED_ACCOUNT_WRONG_TYPE`**, qui nomme le
//! compte, son réglage, son type et le type attendu (AC7) ; et la page des réglages
//! juge le type par la même constante que la validation, en mots (AC9, D4, D6).
//!
//! Ces tests montent l'application : le catalogue fr-CH y est chargé, si bien que
//! les messages s'assertent ici, et nulle part dans un test unitaire.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::AccountType;
use kesh_db::repositories::company_invoice_settings::{
    CLAIM_SETTING_ACCOUNT_TYPES, VAT_SETTING_ACCOUNT_TYPES,
};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";
const SETTINGS: &str = "/api/v1/company/invoice-settings";

/// La phrase d'action, dernière du message (D2).
const ACTION: &str = "Rien n’a été enregistré. Un administrateur doit corriger le réglage concerné — désigner un compte du bon type, ou rendre au compte son type d’origine —, puis la validation pourra être relancée telle quelle.";

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

    /// Retype un compte par `PUT /api/v1/accounts/{id}` — le chemin réel
    /// (`accounts::update`) —, rôle retiré, confirmation du retypage donnée.
    async fn retype(&self, account_id: i64, account_type: AccountType) {
        let list = self.get("/api/v1/accounts").await;
        let a = list
            .as_array()
            .expect("liste des comptes")
            .iter()
            .find(|a| a["id"] == account_id)
            .expect("compte de la société")
            .clone();
        let (status, body) = self
            .send(
                reqwest::Method::PUT,
                &format!("/api/v1/accounts/{account_id}"),
                json!({
                    "name": a["name"],
                    "accountType": account_type.as_str(),
                    "role": null,
                    "postable": a["postable"],
                    "version": a["version"],
                    "confirmRetype": true,
                }),
            )
            .await;
        assert_eq!(status, 200, "retypage : {body}");
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

/// Le montage `seed_accounting_company` (créance `1100` actif, TVA due `2000`
/// passif, exercice 2020-2030), l'arrondi à 5 centimes désactivé, et un contact
/// client et fournisseur.
async fn setup(pool: &MySqlPool) -> (TestApp, SeededCompany, i64) {
    let seeded = seed_accounting_company(pool).await.unwrap();
    kesh_db::test_fixtures::disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .unwrap();
    let app = spawn_app(pool).await;
    let (status, contact) = app
        .send(
            reqwest::Method::POST,
            "/api/v1/contacts",
            json!({
                "contactType": "Entreprise",
                "name": "Tiers 35-3 SA",
                "isClient": true,
                "isSupplier": true,
            }),
        )
        .await;
    assert_eq!(status, 201, "contact : {contact}");
    let contact_id = contact["id"].as_i64().unwrap();
    (app, seeded, contact_id)
}

async fn new_account(pool: &MySqlPool, company_id: i64, number: &str, kind: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, ?, 'Compte 35-3', ?) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(kind)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Une facture brouillon de 100.00 HT à 8.1 % (TVA due écrite).
async fn draft_invoice(app: &TestApp, contact_id: i64) -> i64 {
    let (status, inv) = app
        .send(
            reqwest::Method::POST,
            "/api/v1/invoices",
            json!({
                "contactId": contact_id,
                "date": "2026-06-15",
                "paymentTerms": null,
                "lines": [{
                    "description": "Prestation",
                    "quantity": "1",
                    "unitPrice": "100.00",
                    "vatRate": "8.10",
                }],
            }),
        )
        .await;
    assert_eq!(status, 201, "facture : {inv}");
    inv["id"].as_i64().unwrap()
}

/// `(numéro, réglage, attendu, réel)` de `details.rejected`, dans l'ordre.
fn rejected(body: &Value) -> Vec<(String, String, String, String)> {
    body["error"]["details"]["rejected"]
        .as_array()
        .unwrap_or_else(|| panic!("details.rejected attendu : {body}"))
        .iter()
        .map(|r| {
            assert!(r["accountId"].is_i64(), "accountId : {r}");
            (
                r["accountNumber"].as_str().unwrap().to_string(),
                r["setting"].as_str().unwrap().to_string(),
                r["expectedType"].as_str().unwrap().to_string(),
                r["actualType"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn r(n: &str, s: &str, e: &str, a: &str) -> (String, String, String, String) {
    (n.into(), s.into(), e.into(), a.into())
}

/// AC7 — la créance retypée en passif : 400, le code, les `details`, et le
/// message fr-CH entier — numéro, réglage en mots, *Administration →
/// Facturation*, types en mots, phrase d'action.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac7_sale_validation_names_the_retyped_receivable(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let id = draft_invoice(&app, contact).await;
    app.retype(seeded.accounts["1100"], AccountType::Liability)
        .await;

    let (status, body) = app
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/validate"),
            json!({}),
        )
        .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "DESIGNATED_ACCOUNT_WRONG_TYPE");
    assert_eq!(
        rejected(&body),
        vec![r("1100", "receivable", "Asset", "Liability")]
    );
    assert_eq!(
        body["error"]["message"],
        format!(
            "Le compte 1100, désigné comme compte créance client dans Administration → Facturation, est un compte de passif ; ce réglage demande un compte d’actif. {ACTION}"
        )
    );
    let inv = app.get(&format!("/api/v1/invoices/{id}")).await;
    assert_eq!(inv["status"], "draft", "la facture reste brouillon");
}

/// AC7 — deux comptes aux types attendus différents (créance en passif, TVA due
/// en actif) : deux phrases justes, par numéro, une seule phrase d'action.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac7_sale_validation_names_two_accounts_in_one_refusal(pool: MySqlPool) {
    let (app, seeded, contact) = setup(&pool).await;
    let id = draft_invoice(&app, contact).await;
    app.retype(seeded.accounts["1100"], AccountType::Liability)
        .await;
    app.retype(seeded.accounts["2000"], AccountType::Asset)
        .await;

    let (status, body) = app
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{id}/validate"),
            json!({}),
        )
        .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "DESIGNATED_ACCOUNT_WRONG_TYPE");
    assert_eq!(
        rejected(&body),
        vec![
            r("1100", "receivable", "Asset", "Liability"),
            r("2000", "vatPayable", "Liability", "Asset"),
        ]
    );
    assert_eq!(
        body["error"]["message"],
        format!(
            "Le compte 1100, désigné comme compte créance client dans Administration → Facturation, est un compte de passif ; ce réglage demande un compte d’actif. Le compte 2000, désigné comme compte TVA due dans Administration → Facturation, est un compte d’actif ; ce réglage demande un compte de passif. {ACTION}"
        )
    );
}

/// AC7 (V3-3) — côté achat, créanciers `2000` retypés en actif et TVA
/// récupérable `1171` retypée en passif : le refus se lit par numéro, `1171`
/// puis `2000`, alors que l'ordre des rôles donnerait `2000` puis `1171`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac7_purchase_validation_orders_the_accounts_by_number(pool: MySqlPool) {
    let (app, seeded, supplier) = setup(&pool).await;
    let recoverable = new_account(&pool, seeded.company_id, "1171", "Asset").await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ?, \
         default_vat_recoverable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(recoverable)
    .bind(seeded.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let (status, inv) = app
        .send(
            reqwest::Method::POST,
            "/api/v1/supplier-invoices",
            json!({
                "contactId": supplier,
                "supplierInvoiceNumber": "FF-353-1",
                "invoiceDate": "2026-06-15",
                "dueDate": null,
                "lines": [{
                    "description": "Fourniture",
                    "quantity": "1",
                    "unitPrice": "100.00",
                    "vatRate": "8.10",
                    "expenseAccountId": seeded.accounts["4000"],
                }],
            }),
        )
        .await;
    assert_eq!(status, 201, "brouillon fournisseur : {inv}");
    app.retype(seeded.accounts["2000"], AccountType::Asset)
        .await;
    app.retype(recoverable, AccountType::Liability).await;

    let (status, body) = app
        .send(
            reqwest::Method::POST,
            &format!("/api/v1/supplier-invoices/{}/validate", inv["id"]),
            json!({ "version": inv["version"] }),
        )
        .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "DESIGNATED_ACCOUNT_WRONG_TYPE");
    assert_eq!(
        rejected(&body),
        vec![
            r("1171", "vatRecoverable", "Asset", "Liability"),
            r("2000", "payable", "Liability", "Asset"),
        ]
    );
    assert_eq!(
        body["error"]["message"],
        format!(
            "Le compte 1171, désigné comme compte TVA récupérable dans Administration → Facturation, est un compte de passif ; ce réglage demande un compte d’actif. Le compte 2000, désigné comme compte créanciers dans Administration → Facturation, est un compte d’actif ; ce réglage demande un compte de passif. {ACTION}"
        )
    );
}

/// AC9 (D4) — la page des réglages juge les quatre champs par la constante que
/// lit la validation : un compte du type qu'elle donne passe, un compte de charge
/// est refusé ; le refus du compte débiteurs dit le type attendu EN MOTS (D6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ac9_settings_route_reads_the_same_types_as_the_validation(pool: MySqlPool) {
    let (app, seeded, _) = setup(&pool).await;
    let c = seeded.company_id;
    // Un compte neuf, actif et imputable, par type : chaque champ CHANGE de valeur.
    let mut fresh = std::collections::HashMap::new();
    for (number, kind) in [
        ("1180", AccountType::Asset),
        ("1181", AccountType::Asset),
        ("2080", AccountType::Liability),
        ("2081", AccountType::Liability),
    ] {
        fresh
            .entry(kind.as_str())
            .or_insert_with(Vec::new)
            .push(new_account(&pool, c, number, kind.as_str()).await);
    }
    let expense = new_account(&pool, c, "4080", "Expense").await;
    let fields = [
        (
            "defaultReceivableAccountId",
            CLAIM_SETTING_ACCOUNT_TYPES.receivable,
        ),
        (
            "defaultPayableAccountId",
            CLAIM_SETTING_ACCOUNT_TYPES.payable,
        ),
        (
            "defaultVatPayableAccountId",
            VAT_SETTING_ACCOUNT_TYPES.payable,
        ),
        (
            "defaultVatRecoverableAccountId",
            VAT_SETTING_ACCOUNT_TYPES.recoverable,
        ),
    ];
    for (field, expected) in fields {
        let mut body = app.get(SETTINGS).await;
        body[field] = json!(expense);
        let (status, refused) = app.send(reqwest::Method::PUT, SETTINGS, body).await;
        assert_eq!(
            status, 400,
            "{field} : un compte de charge est refusé : {refused}"
        );

        let candidates = fresh
            .get_mut(expected.as_str())
            .expect("compte du type attendu");
        let account = candidates.pop().expect("un compte neuf par champ");
        let mut body = app.get(SETTINGS).await;
        body[field] = json!(account);
        let (status, saved) = app.send(reqwest::Method::PUT, SETTINGS, body).await;
        assert_eq!(
            status,
            200,
            "{field} : un compte {} passe : {saved}",
            expected.as_str()
        );
    }

    // Le message, en mots, pour le compte débiteurs.
    let mut body = app.get(SETTINGS).await;
    body["defaultReceivableAccountId"] = json!(expense);
    let (status, refused) = app.send(reqwest::Method::PUT, SETTINGS, body).await;
    assert_eq!(status, 400);
    assert_eq!(
        refused["error"]["message"],
        "Compte créance : type de compte incompatible (attendu : actif)"
    );
}
