//! Story 34-11a2 (#972) — la carte de crédit par les routes : création (AC-1,
//! AC-2), garde du numéro complet (AC-3, assistant compris), modification (AC-4),
//! retypage d'un compte lié et règlement client refusé (AC-5), traces (AC-6).
//!
//! Fixtures synthétiques seulement. Les numéros de carte complets employés sont
//! les numéros de **test publics** des réseaux (`4111 1111 1111 1111`) ; les
//! quatre chiffres valent `4821` ou `7360`, choisis pour ne coïncider avec
//! aucun identifiant des montages.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewBankAccount, NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::repositories::{bank_accounts, contacts, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

/// Une société de `seed_accounting_company`, onboarding terminé ; un actif 1020,
/// deux passifs 2030 et 2031 (dettes de carte), le compte créanciers 2000
/// désigné ; le serveur et un JWT `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    actif: i64,
    passif: i64,
    passif2: i64,
}

async fn account(pool: &MySqlPool, company_id: i64, number: &str, t: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, ?, ?, ?, 1, 1) RETURNING id",
    )
    .bind(company_id)
    .bind(number)
    .bind(format!("Compte {number}"))
    .bind(t)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let actif = account(pool, seeded.company_id, "1020", "Asset").await;
    let passif = account(pool, seeded.company_id, "2030", "Liability").await;
    let passif2 = account(pool, seeded.company_id, "2031", "Liability").await;
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    Ctx {
        pool: pool.clone(),
        app,
        jwt,
        seeded,
        actif,
        passif,
        passif2,
    }
}

impl Ctx {
    async fn req(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, format!("{}{chemin}", self.app.base_url))
            .bearer_auth(&self.jwt);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::POST, chemin, Some(corps)).await
    }

    async fn put(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::PUT, chemin, Some(corps)).await
    }

    async fn patch(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::PATCH, chemin, Some(corps)).await
    }

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql)
            .bind(self.seeded.company_id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn bank_accounts(&self) -> i64 {
        self.count("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
            .await
    }

    async fn traces(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Crée une carte liée à `ledger` par la route ; rend le corps.
    async fn carte(&self, ledger: i64) -> Value {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "accountKind": "credit_card",
                    "bankName": "Visa",
                    "description": "Personnel",
                    "cardLast4": "4821",
                    "journalAccountId": ledger,
                }),
            )
            .await;
        assert_eq!(s, 201, "carte : {b}");
        b
    }

    /// Un compte bancaire lié à l'actif 1020, écrit par le dépôt.
    async fn compte_bancaire(&self) -> i64 {
        let id = bank_accounts::create(
            &self.pool,
            NewBankAccount {
                company_id: self.seeded.company_id,
                bank_name: "UBS".into(),
                iban: "CH9300762011623852957".into(),
                qr_iban: None,
                is_primary: false,
            },
        )
        .await
        .unwrap()
        .id;
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(self.actif)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    async fn last_audit(&self, action: &str, entity_id: i64) -> Value {
        sqlx::query_scalar(
            "SELECT details_json FROM audit_log WHERE action = ? AND entity_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(action)
        .bind(entity_id)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn contact(&self, nom: &str, fournisseur: bool) -> i64 {
        contacts::create(
            &self.pool,
            self.seeded.admin_user_id,
            NewContact {
                company_id: self.seeded.company_id,
                contact_type: ContactType::Entreprise,
                name: nom.into(),
                first_name: None,
                last_name: None,
                is_client: !fournisseur,
                is_supplier: fournisseur,
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
        .id
    }
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

// ============================================================
// AC-1, AC-6 — créer une carte
// ============================================================

/// AC-1 — une carte se crée sans IBAN, non principale, liée à un passif ; AC-6 :
/// la trace porte le type et la présence des quatre chiffres, jamais les
/// chiffres.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creer_une_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let b = ctx.carte(ctx.passif).await;
    assert_eq!(b["accountKind"], "credit_card");
    assert_eq!(b["iban"], Value::Null);
    assert_eq!(b["qrIban"], Value::Null);
    assert_eq!(b["isPrimary"], false);
    assert_eq!(b["cardLast4"], "4821");
    assert_eq!(b["journalAccountId"], ctx.passif);

    let id = b["id"].as_i64().unwrap();
    let details = ctx.last_audit("bank_account.created", id).await;
    assert_eq!(details["account_kind"], "credit_card");
    assert_eq!(details["card_last4_present"], true);
    assert_eq!(details["iban_present"], false);
    assert!(
        !details.to_string().contains("4821"),
        "les chiffres ne vont pas dans la trace : {details}"
    );
}

/// AC-2 (C-e341-42) — `cardLast4` `null` ou vide vaut absent : une carte sans
/// chiffres est acceptée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn carte_sans_quatre_chiffres(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    for vide in [Value::Null, json!(""), json!("  ")] {
        let (s, b) = ctx
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "accountKind": "credit_card",
                    "bankName": "Mastercard",
                    "cardLast4": vide,
                    "journalAccountId": ctx.passif,
                }),
            )
            .await;
        assert_eq!(s, 201, "{vide} : {b}");
        assert_eq!(b["cardLast4"], Value::Null);
    }
}

// ============================================================
// AC-2 — refus à la création, sans écriture
// ============================================================

/// AC-2 — chaque refus d'une carte rend `400` et n'écrit rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_a_la_creation_d_une_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let base = json!({
        "accountKind": "credit_card",
        "bankName": "Visa",
        "journalAccountId": ctx.passif,
    });
    let with = |k: &str, v: Value| {
        let mut b = base.clone();
        b[k] = v;
        b
    };
    let without_link = {
        let mut b = base.clone();
        b.as_object_mut().unwrap().remove("journalAccountId");
        b
    };
    let cas: Vec<(Value, &str, Option<&str>)> = vec![
        (
            with("iban", json!("CH9300762011623852957")),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
            Some("iban"),
        ),
        (
            with("qrIban", json!("CH4431999123000889012")),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
            Some("qrIban"),
        ),
        (
            with("isPrimary", json!(true)),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
            Some("isPrimary"),
        ),
        (without_link, "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY", None),
        (
            with("journalAccountId", Value::Null),
            "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY",
            None,
        ),
        (
            with("journalAccountId", json!(ctx.actif)),
            "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY",
            None,
        ),
        (with("cardLast4", json!("12345")), "VALIDATION_ERROR", None),
        (with("cardLast4", json!("12a4")), "VALIDATION_ERROR", None),
        (with("cardLast4", json!("123")), "VALIDATION_ERROR", None),
        (
            with("bankName", json!("Visa 4111 1111 1111 1111")),
            "CARD_NUMBER_NOT_ALLOWED",
            Some("bankName"),
        ),
        (
            with("description", json!("perso 4111111111111111")),
            "CARD_NUMBER_NOT_ALLOWED",
            Some("description"),
        ),
    ];
    let (avant, traces_avant) = (ctx.bank_accounts().await, ctx.traces().await);
    for (corps, attendu, champ) in cas {
        let (s, b) = ctx.post("/api/v1/bank-accounts", corps.clone()).await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(code(&b), attendu, "{corps} : {b}");
        if let Some(champ) = champ {
            assert_eq!(b["error"]["details"]["field"], champ, "{corps}");
        }
    }
    assert_eq!(ctx.bank_accounts().await, avant, "rien d'écrit");
    assert_eq!(ctx.traces().await, traces_avant, "aucune trace");
}

/// AC-2 (validation P4, F3, F4 ; M10, M10b) — un compte **bancaire** sans IBAN,
/// type absent ou explicite, rend `400` (non `500` du `CHECK`) ; `cardLast4`
/// fourni rend `400 BANK_ACCOUNT_FIELD_NOT_ALLOWED` ; `null` ou `""` passent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_bancaire_sans_iban_ou_avec_quatre_chiffres(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let avant = ctx.bank_accounts().await;
    for corps in [
        json!({ "bankName": "UBS" }),
        json!({ "bankName": "UBS", "iban": null }),
        json!({ "bankName": "UBS", "accountKind": "bank" }),
        json!({ "bankName": "UBS", "accountKind": "bank", "iban": null }),
    ] {
        let (s, b) = ctx.post("/api/v1/bank-accounts", corps.clone()).await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(code(&b), "VALIDATION_ERROR", "{corps} : {b}");
        // Revue P2 (B4) — le motif est celui d'un IBAN vide (`kesh-core`), même
        // code qu'en 0.17.5 ; le message, lui, n'est plus celui de l'extracteur.
        let message = b["error"]["message"].as_str().unwrap_or("");
        assert!(message.contains("IBAN invalide"), "{corps} : {b}");
    }
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "cardLast4": "4821" }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "BANK_ACCOUNT_FIELD_NOT_ALLOWED");
    assert_eq!(b["error"]["details"]["field"], "cardLast4");
    assert_eq!(ctx.bank_accounts().await, avant, "rien d'écrit");

    for vide in [Value::Null, json!("")] {
        let (s, b) = ctx
            .post(
                "/api/v1/bank-accounts",
                json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "cardLast4": vide }),
            )
            .await;
        assert_eq!(s, 201, "{vide} : {b}");
        assert_eq!(b["accountKind"], "bank");
    }
}

/// AC-3 — la garde vaut pour les deux types : un compte bancaire dont la
/// description porte un numéro complet est refusé ; un IBAN imprimé dans la
/// description passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn garde_du_numero_sur_un_compte_bancaire(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "description": "Carte 5555-5555-5555-4444" }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CARD_NUMBER_NOT_ALLOWED");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "description": "CH95 0076 2008 0636 0837 7" }),
        )
        .await;
    assert_eq!(s, 201, "un IBAN imprimé n'est pas un numéro de carte : {b}");
}

/// AC-3 (M1b) — la garde atteint l'assistant (`POST /onboarding/bank-account`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn garde_du_numero_a_l_assistant(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/onboarding/bank-account",
            json!({ "bankName": "Visa 4111 1111 1111 1111", "iban": "CH9300762011623852957" }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CARD_NUMBER_NOT_ALLOWED");
    assert_eq!(b["error"]["details"]["field"], "bankName");
}

// ============================================================
// AC-4 — modifier une carte
// ============================================================

/// AC-4 (M3) — `PUT` sans `accountKind` : nom, description, quatre chiffres et
/// lien (passif) changent ; `cardLast4` absent les efface ; la carte reste une
/// carte. AC-6 : instantanés de la trace sans les chiffres.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modifier_une_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();

    let (s, b) = ctx
        .put(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({
                "bankName": "Visa Gold",
                "description": "Professionnel",
                "cardLast4": "7360",
                "journalAccountId": ctx.passif2,
                "version": carte["version"],
            }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accountKind"], "credit_card");
    assert_eq!(b["bankName"], "Visa Gold");
    assert_eq!(b["description"], "Professionnel");
    assert_eq!(b["cardLast4"], "7360");
    assert_eq!(b["iban"], Value::Null);
    assert_eq!(b["journalAccountId"], ctx.passif2);

    let details = ctx.last_audit("bank_account.updated", id).await;
    assert_eq!(details["before"]["account_kind"], "credit_card");
    assert_eq!(details["after"]["card_last4_present"], true);
    let texte = details.to_string();
    assert!(
        !texte.contains("7360") && !texte.contains("4821"),
        "{texte}"
    );

    let (s, b) = ctx
        .put(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({ "bankName": "Visa Gold", "journalAccountId": ctx.passif2, "version": b["version"] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["cardLast4"], Value::Null, "absents : effacés");
    assert_eq!(b["description"], Value::Null);
}

/// AC-4 (C-e341-30, C-e341-43) — refus du `PUT` d'une carte, sans écriture ; un
/// id inconnu ou archivé rend `404` avant tout refus de forme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_du_put_d_une_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    let v = carte["version"].clone();
    let base = json!({ "bankName": "Visa", "journalAccountId": ctx.passif, "version": v });
    let with = |k: &str, val: Value| {
        let mut b = base.clone();
        b[k] = val;
        b
    };
    let cas = [
        (
            with("accountKind", json!("bank")),
            "BANK_ACCOUNT_KIND_IMMUTABLE",
        ),
        (
            with("iban", json!("CH9300762011623852957")),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
        ),
        (
            with("qrIban", json!("CH4431999123000889012")),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
        ),
        (
            with("isPrimary", json!(true)),
            "CREDIT_CARD_FIELD_NOT_ALLOWED",
        ),
        (
            with("journalAccountId", json!(ctx.actif)),
            "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY",
        ),
        (
            with("journalAccountId", Value::Null),
            "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY",
        ),
    ];
    for (corps, attendu) in cas {
        let (s, b) = ctx
            .put(&format!("/api/v1/bank-accounts/{id}"), corps.clone())
            .await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(code(&b), attendu, "{corps} : {b}");
    }
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(json!(version), v, "rien d'écrit");

    // Inconnu, puis archivé : 404, et non 400 de l'IBAN absent.
    let (s, b) = ctx
        .put(
            "/api/v1/bank-accounts/999999",
            json!({ "bankName": "X", "version": 1 }),
        )
        .await;
    assert_eq!(s, 404, "{b}");
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .put(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({ "bankName": "X", "version": v }),
        )
        .await;
    assert_eq!(s, 404, "{b}");
}

/// AC-4 (M10, M10b) — `PUT` d'un compte bancaire sans IBAN ou avec quatre
/// chiffres : `400`, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_du_put_d_un_compte_bancaire(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let id = ctx.compte_bancaire().await;
    for (corps, attendu) in [
        (
            json!({ "bankName": "UBS", "version": 1 }),
            "VALIDATION_ERROR",
        ),
        (
            json!({ "bankName": "UBS", "iban": null, "version": 1 }),
            "VALIDATION_ERROR",
        ),
        (
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "cardLast4": "4821", "version": 1 }),
            "BANK_ACCOUNT_FIELD_NOT_ALLOWED",
        ),
    ] {
        let (s, b) = ctx
            .put(&format!("/api/v1/bank-accounts/{id}"), corps.clone())
            .await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(code(&b), attendu, "{corps} : {b}");
    }
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(version, 1, "rien d'écrit");
}

/// AC-4 (M4) — `PATCH` qui délie une carte ou la relie à un actif : `400`, rien
/// d'écrit ; vers un autre passif : accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn patch_d_une_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    // Revue P1 (B1 = E1, M12) — une charge, un produit, le compte débiteurs
    // (actif désigné) et un actif non imputable rendent le code de la carte,
    // comme au `POST` et au `PUT`, avant les refus de la transaction.
    let non_imputable = account(&ctx.pool, ctx.seeded.company_id, "1090", "Asset").await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(non_imputable)
        .execute(&ctx.pool)
        .await
        .unwrap();
    for cible in [
        Value::Null,
        json!(ctx.actif),
        json!(ctx.seeded.accounts["4000"]),
        json!(ctx.seeded.accounts["3000"]),
        json!(ctx.seeded.accounts["1100"]),
        json!(non_imputable),
    ] {
        let (s, b) = ctx
            .patch(
                &format!("/api/v1/bank-accounts/{id}"),
                json!({ "journalAccountId": cible, "version": carte["version"] }),
            )
            .await;
        assert_eq!(s, 400, "{cible} : {b}");
        assert_eq!(code(&b), "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY", "{b}");
    }
    let lie: Option<i64> =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(lie, Some(ctx.passif), "rien d'écrit");
    let (s, b) = ctx
        .patch(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({ "journalAccountId": ctx.passif2, "version": carte["version"] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["journalAccountId"], ctx.passif2);
}

// ============================================================
// AC-5 — concurrence : le type relu sous verrou par les routes (revue P1, B2)
// ============================================================

/// Attend (dix secondes au plus) qu'une autre connexion de la base de test
/// exécute une requête qui répond à `motif` (`LIKE`) — bloquée sur un verrou,
/// pour une requête qui finirait en microsecondes. Sonde `PROCESSLIST` (sans le
/// privilège `PROCESS`, `INNODB_TRX` est fermé) ; le texte montre les `?`.
async fn attendre_requete_en_cours(pool: &MySqlPool, motif: &str) {
    for _ in 0..200 {
        let n: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM information_schema.PROCESSLIST \
             WHERE DB = DATABASE() AND ID <> CONNECTION_ID() AND INFO LIKE ?",
        )
        .bind(motif)
        .fetch_one(pool)
        .await
        .unwrap();
        if n > 0 {
            return;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
    panic!("aucune requête en cours ne répond à {motif:?}");
}

/// La relecture du type par une route, vue bloquée derrière le retypage.
const RELECTURE_DU_TYPE: &str = "%account_type, active FROM accounts%LOCK IN SHARE MODE%";

/// Un retypage **tenu ouvert** : `passif` pris `FOR UPDATE` et passé en charge,
/// sans commit. Une route qui le lit sur le pool y voit encore un passif.
async fn retypage_tenu(pool: &MySqlPool, passif: i64) -> sqlx::Transaction<'static, sqlx::MySql> {
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM accounts WHERE id = ? FOR UPDATE")
        .bind(passif)
        .execute(&mut *tx)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET account_type = 'Expense' WHERE id = ?")
        .bind(passif)
        .execute(&mut *tx)
        .await
        .unwrap();
    tx
}

impl Ctx {
    /// Lance une requête en tâche (le serveur la traite pendant que le test
    /// tient le retypage).
    fn lancer(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Value,
    ) -> tokio::task::JoinHandle<(u16, Value)> {
        let r = self
            .app
            .client
            .request(methode, format!("{}{chemin}", self.app.base_url))
            .bearer_auth(&self.jwt)
            .json(&corps);
        tokio::spawn(async move {
            let resp = r.send().await.unwrap();
            let s = resp.status().as_u16();
            (s, resp.json().await.unwrap_or(Value::Null))
        })
    }
}

/// AC-5 (C-e341-40 ; revue P1, B2 = E2, A12 ; M11) — création d'une carte
/// pendant qu'un retypage du passif est tenu : le pré-vol lit encore un passif,
/// la transaction **attend** la relecture du type (vue bloquée), puis le
/// retypage commité, la voit en charge et refuse. Sans la relecture
/// (`recheck_ledger_type_in_tx` retiré de `create_once`), la sonde ne voit
/// jamais la relecture et le test rougit — et la carte serait liée à une charge.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creation_croisee_avec_un_retypage(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let tenu = retypage_tenu(&pool, ctx.passif).await;
    let creation = ctx.lancer(
        reqwest::Method::POST,
        "/api/v1/bank-accounts",
        json!({ "accountKind": "credit_card", "bankName": "Visa", "journalAccountId": ctx.passif }),
    );
    attendre_requete_en_cours(&pool, RELECTURE_DU_TYPE).await;
    tenu.commit().await.unwrap();
    let (s, b) = creation.await.unwrap();
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY", "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ? AND account_kind = 'credit_card'")
            .await,
        0,
        "aucune carte liée à une charge"
    );
}

/// AC-5 (revue P1, B2, A12 ; M11b) — même course au `PUT` d'une carte qui
/// change de passif.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_croise_avec_un_retypage(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    let tenu = retypage_tenu(&pool, ctx.passif2).await;
    let put = ctx.lancer(
        reqwest::Method::PUT,
        &format!("/api/v1/bank-accounts/{id}"),
        json!({ "bankName": "Visa", "journalAccountId": ctx.passif2, "version": carte["version"] }),
    );
    attendre_requete_en_cours(&pool, RELECTURE_DU_TYPE).await;
    tenu.commit().await.unwrap();
    let (s, b) = put.await.unwrap();
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY", "{b}");
    let lie: Option<i64> =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(lie, Some(ctx.passif), "rien d'écrit");
}

/// AC-5 (revue P1, A12 ; M4) — même course au `PATCH` : le pré-vol (type de la
/// carte, B1) voit un passif, la transaction relit le type sous verrou et le
/// contrôle de la carte, sous le `FOR UPDATE` de la ligne, refuse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn patch_croise_avec_un_retypage(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    let tenu = retypage_tenu(&pool, ctx.passif2).await;
    let patch = ctx.lancer(
        reqwest::Method::PATCH,
        &format!("/api/v1/bank-accounts/{id}"),
        json!({ "journalAccountId": ctx.passif2, "version": carte["version"] }),
    );
    attendre_requete_en_cours(&pool, RELECTURE_DU_TYPE).await;
    tenu.commit().await.unwrap();
    let (s, b) = patch.await.unwrap();
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CREDIT_CARD_LEDGER_MUST_BE_LIABILITY", "{b}");
    let lie: Option<i64> =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(lie, Some(ctx.passif), "rien d'écrit");
}

/// Revue P1 (E5) — un numéro complet collé dans le champ des quatre chiffres :
/// `400 CARD_NUMBER_NOT_ALLOWED`, `details.field = "cardLast4"`, rien d'écrit,
/// et le numéro n'est pas repris dans la réponse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn numero_complet_dans_les_quatre_chiffres(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let avant = ctx.bank_accounts().await;
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({
                "accountKind": "credit_card",
                "bankName": "Visa",
                "cardLast4": "4111 1111 1111 1111",
                "journalAccountId": ctx.passif,
            }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "CARD_NUMBER_NOT_ALLOWED");
    assert_eq!(b["error"]["details"]["field"], "cardLast4");
    assert!(!b.to_string().contains("1111"), "{b}");
    assert_eq!(ctx.bank_accounts().await, avant);
}

/// Revue P1 (B7 = E4) — un IBAN valide écrit « CH 48… » d'un bloc dans une
/// description, dont les chiffres passeraient pour un Visa : accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_iban_dans_la_description_n_est_pas_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "description": "ex CH 4826751125328704655" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
}

/// Revue P1 (E7) — le refus du retypage d'un compte lié passe **avant** la garde
/// des écritures : un passif mouvementé, retypé sans confirmation, rend
/// `ACCOUNT_LINKED_TO_BANK_ACCOUNT`, non `ACCOUNT_HAS_ENTRIES`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retypage_refuse_avant_la_garde_des_ecritures(pool: MySqlPool) {
    use kesh_db::entities::journal_entry::Journal;
    use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
    let ctx = monter(&pool).await;
    ctx.carte(ctx.passif).await;
    let date = NaiveDate::from_ymd_opt(2026, 3, 1).unwrap();
    kesh_db::repositories::journal_entries::create(
        &ctx.pool,
        ctx.seeded.fiscal_year_id,
        ctx.seeded.admin_user_id,
        NewJournalEntry {
            company_id: ctx.seeded.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Achat par carte".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: ctx.seeded.accounts["4000"],
                    debit: dec!(50.00),
                    credit: rust_decimal::Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: ctx.passif,
                    debit: rust_decimal::Decimal::ZERO,
                    credit: dec!(50.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
    let (s, b) = ctx
        .put(
            &format!("/api/v1/accounts/{}", ctx.passif),
            json!({ "name": "Compte 2030", "accountType": "Expense", "role": null, "postable": true, "version": 1 }),
        )
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "ACCOUNT_LINKED_TO_BANK_ACCOUNT", "{b}");
}

// ============================================================
// AC-5 — retypage et règlements
// ============================================================

/// AC-5 — retyper `Expense` le passif d'une carte par la route du plan :
/// `409 ACCOUNT_LINKED_TO_BANK_ACCOUNT`, qui nomme la carte ; rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn retypage_refuse_par_la_route(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let (s, b) = ctx
        .put(
            &format!("/api/v1/accounts/{}", ctx.passif),
            json!({
                "name": "Compte 2030",
                "accountType": "Expense",
                "role": null,
                "postable": true,
                "confirmAccountRetype": true,
                "version": 1,
            }),
        )
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "ACCOUNT_LINKED_TO_BANK_ACCOUNT");
    assert_eq!(b["error"]["details"]["bankAccountId"], carte["id"]);
    assert_eq!(b["error"]["details"]["bankAccountName"], "Visa — Personnel");
    let t: String = sqlx::query_scalar("SELECT account_type FROM accounts WHERE id = ?")
        .bind(ctx.passif)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(t, "Liability");
}

/// AC-5 (C-e341-41, M7b) — un règlement client sur une carte : `400
/// SETTLEMENT_ACCOUNT_IS_CREDIT_CARD`, rien d'écrit. Un paiement fournisseur
/// depuis la carte passe (D créanciers / C passif de la carte).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reglements_et_carte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let carte_id = carte["id"].as_i64().unwrap();

    // Facture client validée.
    let client = ctx.contact("Client", false).await;
    let (s, b) = ctx
        .post(
            "/api/v1/invoices",
            json!({
                "contactId": client,
                "date": Utc::now().date_naive().to_string(),
                "lines": [{"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}]
            }),
        )
        .await;
    assert_eq!(s, 201, "facture : {b}");
    let invoice = b["id"].as_i64().unwrap();
    let (s, b) = ctx
        .req(
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{invoice}/validate"),
            None,
        )
        .await;
    assert_eq!(s, 200, "validation : {b}");

    let ecritures = ctx
        .count("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/invoices/{invoice}/settlements"),
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": carte_id,
                "amount": "108.10",
                "settledOn": Utc::now().date_naive().to_string(),
            }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "SETTLEMENT_ACCOUNT_IS_CREDIT_CARD");
    assert_eq!(b["error"]["details"]["bankAccountId"], carte_id);
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .await,
        ecritures,
        "rien d'écrit"
    );

    // Facture fournisseur payée depuis la carte : passe.
    let fournisseur = ctx.contact("Fournisseur SA", true).await;
    let ff = supplier_invoices::create_validated(
        &ctx.pool,
        NewSupplierInvoice {
            company_id: ctx.seeded.company_id,
            contact_id: fournisseur,
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
                expense_account_id: ctx.seeded.accounts["4000"],
            }],
        },
        ctx.seeded.admin_user_id,
    )
    .await
    .expect("facture fournisseur")
    .invoice
    .id;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/supplier-invoices/{ff}/pay"),
            json!({ "settlementType": "bank_transfer", "bankAccountId": carte_id, "paymentDate": "2026-06-20" }),
        )
        .await;
    assert_eq!(s, 200, "paiement fournisseur depuis la carte : {b}");
}

// ============================================================
// Revue P2 — branches sans témoin (E3, A8)
// ============================================================

/// Revue P2 (E3-1, E3-2) — le `PUT` d'une carte traverse la garde du numéro
/// complet (nom, description, quatre chiffres) ; `"1234 5678"` n'est pas une
/// carte : `VALIDATION_ERROR`, sans écho de la saisie.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_d_une_carte_et_numero_complet(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    let base =
        json!({ "bankName": "Visa", "journalAccountId": ctx.passif, "version": carte["version"] });
    let with = |k: &str, v: Value| {
        let mut b = base.clone();
        b[k] = v;
        b
    };
    for (corps, champ) in [
        (
            with("bankName", json!("Visa 4111 1111 1111 1111")),
            "bankName",
        ),
        (
            with("description", json!("perso 4111111111111111")),
            "description",
        ),
        (with("cardLast4", json!("4111-1111-1111-1111")), "cardLast4"),
    ] {
        let (s, b) = ctx
            .put(&format!("/api/v1/bank-accounts/{id}"), corps.clone())
            .await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(code(&b), "CARD_NUMBER_NOT_ALLOWED", "{corps} : {b}");
        assert_eq!(b["error"]["details"]["field"], champ);
        assert!(!b.to_string().contains("1111"), "{b}");
    }
    let (s, b) = ctx
        .put(
            &format!("/api/v1/bank-accounts/{id}"),
            with("cardLast4", json!("1234 5678")),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "VALIDATION_ERROR");
    assert!(!b.to_string().contains("5678"), "{b}");
    let version: i32 = sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(&ctx.pool)
        .await
        .unwrap();
    assert_eq!(json!(version), carte["version"], "rien d'écrit");
}

/// Revue P2 (E3-3 ; M19) — `PATCH` d'une carte vers son passif courant :
/// no-op (200, version inchangée) ; d'une carte **archivée** vers un actif :
/// `404` (le pré-vol garde la règle d'un compte bancaire, la transaction rend
/// le `404`) ; d'une carte **close** : `409 BANK_ACCOUNT_CLOSED`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn patch_d_une_carte_inchangee_archivee_ou_close(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let carte = ctx.carte(ctx.passif).await;
    let id = carte["id"].as_i64().unwrap();
    let (s, b) = ctx
        .patch(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({ "journalAccountId": ctx.passif, "version": carte["version"] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["version"], carte["version"], "no-op : version inchangée");

    let close = ctx.carte(ctx.passif).await;
    let close_id = close["id"].as_i64().unwrap();
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-06-30' WHERE id = ?")
        .bind(close_id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .patch(
            &format!("/api/v1/bank-accounts/{close_id}"),
            json!({ "journalAccountId": ctx.passif2, "version": close["version"] }),
        )
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "BANK_ACCOUNT_CLOSED", "{b}");

    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .patch(
            &format!("/api/v1/bank-accounts/{id}"),
            json!({ "journalAccountId": ctx.actif, "version": carte["version"] }),
        )
        .await;
    assert_eq!(s, 404, "carte archivée : {b}");
}

/// Revue P2 (E3-4) — une carte vers un passif **non imputable** ou vers le
/// **compte créanciers désigné** : les règles d'un compte bancaire s'appliquent
/// (`ACCOUNT_NOT_POSTABLE`, `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`), rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn carte_vers_un_passif_non_imputable_ou_creanciers(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let non_imputable = account(&ctx.pool, ctx.seeded.company_id, "2090", "Liability").await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(non_imputable)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let avant = ctx.bank_accounts().await;
    for (ledger, attendu) in [
        (non_imputable, "ACCOUNT_NOT_POSTABLE"),
        (
            ctx.seeded.accounts["2000"],
            "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
        ),
    ] {
        let (s, b) = ctx
            .post(
                "/api/v1/bank-accounts",
                json!({ "accountKind": "credit_card", "bankName": "Visa", "journalAccountId": ledger }),
            )
            .await;
        assert_eq!(s, 400, "{b}");
        assert_eq!(code(&b), attendu, "{b}");
    }
    assert_eq!(ctx.bank_accounts().await, avant, "rien d'écrit");
}

/// Revue P2 (A8) — au `PUT`, `version < 1` passe **avant** le `404` d'un id
/// inconnu (ordre écrit au doc-comment, à la note ¹¹ et au CHANGELOG).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_version_zero_avant_le_404(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .put(
            "/api/v1/bank-accounts/999999",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957", "version": 0 }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "VALIDATION_ERROR", "{b}");
}
