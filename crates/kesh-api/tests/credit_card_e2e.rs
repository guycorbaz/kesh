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
    for cible in [Value::Null, json!(ctx.actif)] {
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
