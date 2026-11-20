//! Story 38-11b (Epic 38, phase 1) — le compte PayPal et l'import de son CSV,
//! par les routes : réglages PayPal d'un compte bancaire (AC-2), IBAN
//! facultatif, compte du plan en devise et refus aux autres chemins (AC-3),
//! route d'import (AC-4), ventilation (AC-5), doublons et compte rendu (AC-6),
//! verrous de plusieurs comptes (`with_account_locks`).
//!
//! Fixtures fictives seulement : le CSV de 38-11a
//! (`kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv`), des
//! comptes « PayPal www » sans existence réelle.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::repositories::{contacts, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, disable_rounding_to_5_centimes,
    mark_onboarding_complete, seed_accounting_company,
};
use reqwest::multipart;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const FIXTURE: &[u8] =
    include_bytes!("../../kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv");
const PAYPAL: &str = "PayPal www";

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

/// Une société multi-devises : comptes du plan CHF, EUR et USD pour les soldes
/// PayPal, comptes de frais (charge CHF, charge EUR, produit, charge non
/// imputable) ; le serveur et un JWT `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    chf: i64,
    eur: i64,
    usd: i64,
    eur2: i64,
    frais_chf: i64,
    frais_eur: i64,
    frais_usd: i64,
    produit: i64,
    regroupement: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
    postable: bool,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, currency) \
         VALUES (?, ?, ?, ?, 1, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(postable)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let c = seeded.company_id;
    let gain = compte(pool, c, "6950", "Revenue", None, true).await;
    let perte = compte(pool, c, "6960", "Expense", None, true).await;
    let transit = compte(pool, c, "1090", "Asset", None, true).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(transit)
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone()).await,
        jwt: forge_jwt(seeded.admin_user_id, "Admin", c),
        chf: compte(pool, c, "1030", "Asset", None, true).await,
        eur: compte(pool, c, "1031", "Asset", Some("EUR"), true).await,
        usd: compte(pool, c, "1032", "Asset", Some("USD"), true).await,
        eur2: compte(pool, c, "1033", "Asset", Some("EUR"), true).await,
        frais_chf: compte(pool, c, "6940", "Expense", None, true).await,
        frais_eur: compte(pool, c, "6941", "Expense", Some("EUR"), true).await,
        frais_usd: compte(pool, c, "6942", "Expense", Some("USD"), true).await,
        produit: compte(pool, c, "3400", "Revenue", None, true).await,
        regroupement: compte(pool, c, "6900", "Expense", None, false).await,
        seeded,
    }
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
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

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    /// Le solde PayPal lié à `ledger`, créé par la route ; rend le corps.
    async fn solde(&self, ledger: i64, fee: Option<i64>) -> Value {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "PayPal",
                    "journalAccountId": ledger,
                    "paypal": { "account": PAYPAL, "feeAccountId": fee },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde PayPal : {b}");
        b
    }

    /// Les trois soldes EUR, USD, CHF ; rend leurs identifiants.
    async fn trois_soldes(&self) -> (i64, i64, i64) {
        let e = self.solde(self.eur, Some(self.frais_eur)).await;
        let u = self.solde(self.usd, None).await;
        let c = self.solde(self.chf, Some(self.frais_chf)).await;
        (
            e["id"].as_i64().unwrap(),
            u["id"].as_i64().unwrap(),
            c["id"].as_i64().unwrap(),
        )
    }

    /// `POST /api/v1/paypal-imports`, avec le jeton donné.
    async fn importer_avec(
        &self,
        jeton: &str,
        fichier: &[u8],
        compte: Option<&str>,
    ) -> (u16, Value) {
        let mut form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(fichier.to_vec())
                .file_name("Download.CSV")
                .mime_str("text/csv")
                .unwrap(),
        );
        if let Some(c) = compte {
            form = form.text("account", c.to_string());
        }
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/paypal-imports", self.app.base_url))
            .bearer_auth(jeton)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn importer(&self, fichier: &[u8]) -> (u16, Value) {
        self.importer_avec(&self.jwt, fichier, Some(PAYPAL)).await
    }

    async fn ecrit(&self) -> (i64, i64, i64) {
        (
            self.count("SELECT COUNT(*) FROM bank_imports").await,
            self.count("SELECT COUNT(*) FROM bank_transactions").await,
            self.count(
                "SELECT COUNT(*) FROM audit_log WHERE action IN ('bank_import.created', 'paypal_import.created')",
            )
            .await,
        )
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

/// Le fichier de 38-11a avec deux lignes de plus (identifiants neufs), dont une
/// à frais : le relevé suivant, qui chevauche le premier.
fn fichier_chevauchant() -> Vec<u8> {
    let mut f = FIXTURE.to_vec();
    if !f.ends_with(b"\n") {
        f.extend_from_slice(b"\r\n");
    }
    let ligne = |date: &str, id: &str, gross: &str, fee: &str, net: &str, balance: &str| {
        format!(
            "\"{date}\",\"10:00:00\",\"CET\",\"Fanny Fictive\",\"Website Payment\",\"Completed\",\"EUR\",\"{gross}\",\"{fee}\",\"{net}\",\"fanny@example.invalid\",\"compte@example.invalid\",\"{id}\",\"\",\"\",\"Commande 2001\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"{balance}\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"Credit\"\r\n"
        )
    };
    f.extend_from_slice(
        ligne(
            "05/05/2020",
            "0TEST0000000000FQ",
            "12.00",
            "-0.40",
            "11.60",
            "17.60",
        )
        .as_bytes(),
    );
    f.extend_from_slice(
        ligne(
            "06/05/2020",
            "0TEST0000000000GR",
            "8.00",
            "0.00",
            "8.00",
            "25.60",
        )
        .as_bytes(),
    );
    f
}

// ============================================================
// AC-4 à AC-6 — l'import
// ============================================================

/// AC-4, AC-5, AC-6 — le fichier de 38-11a se ventile en trois entrées ;
/// relu, il est refusé ; un relevé qui chevauche s'importe, ses lignes déjà
/// connues en doublons.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_ventile_par_devise_puis_doublons(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (e, u, c) = ctx.trois_soldes().await;

    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["account"], PAYPAL);
    let par_devise = |b: &Value, devise: &str| {
        b["currencies"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["currency"] == devise)
            .cloned()
            .unwrap()
    };
    for (devise, id, inserted) in [("EUR", e, 9), ("USD", u, 3), ("CHF", c, 3)] {
        let r = par_devise(&b, devise);
        assert_eq!(r["bankAccountId"], id, "{devise}");
        assert_eq!(r["inserted"], inserted, "{devise}");
        assert_eq!(r["duplicates"], 0, "{devise}");
        assert!(r["importId"].is_i64(), "{devise}");
    }
    assert_eq!(b["skipped"]["memo"], 1);
    assert_eq!(b["skipped"]["neutralPairs"], 1);
    assert_eq!(
        b["skipped"]["notCompleted"],
        json!([
            { "transactionId": "0TEST0000000000CM", "type": "Website Payment", "status": "Pending" },
            { "transactionId": "0TEST0000000000DN", "type": "General Payment", "status": "Denied" }
        ])
    );
    assert_eq!(ctx.ecrit().await, (3, 15, 4));

    // La somme de chaque solde est celle des `Net` de sa devise (frais positifs
    // du remboursement compris) : EUR 96.80 − 50.50 − 40 + 28.74 − 29.04.
    for (id, attendu) in [(e, dec!(6.00)), (u, dec!(63.04)), (c, dec!(194.56))] {
        let somme: Decimal = sqlx::query_scalar(
            "SELECT SUM(amount) FROM bank_transactions WHERE bank_account_id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(somme, attendu, "compte {id}");
    }
    // Les colonnes neuves relues : le type PayPal et l'adresse de la contrepartie.
    let (provider_type, email, reference, currency): (
        Option<String>,
        Option<String>,
        Option<String>,
        String,
    ) = sqlx::query_as(
        "SELECT provider_type, counterparty_email, reference, currency FROM bank_transactions \
             WHERE transaction_id = '0TEST00000000001A'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(provider_type.as_deref(), Some("Website Payment"));
    assert_eq!(email.as_deref(), Some("alice@example.invalid"));
    assert_eq!(reference.as_deref(), Some("TXN 0TEST00000000001A"));
    assert_eq!(currency, "EUR");
    let frais: Decimal = sqlx::query_scalar(
        "SELECT amount FROM bank_transactions WHERE transaction_id = '0TEST0000000000BL:FEE'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frais, dec!(0.96), "frais positifs d'un remboursement");
    // L'entrée d'import : source CSV, période et solde de clôture de la devise.
    let (format, du, au, cloture, nombre): (String, NaiveDate, NaiveDate, Option<Decimal>, i32) =
        sqlx::query_as(
            "SELECT source_format, period_from, period_to, closing_balance, transaction_count \
             FROM bank_imports WHERE bank_account_id = ?",
        )
        .bind(e)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(format, "CSV");
    assert_eq!(du, NaiveDate::from_ymd_opt(2020, 1, 3).unwrap());
    assert_eq!(au, NaiveDate::from_ymd_opt(2020, 3, 30).unwrap());
    assert_eq!(cloture, Some(dec!(6.00)));
    assert_eq!(nombre, 9);
    // La trace de l'import.
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'paypal_import.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["provider_account"], PAYPAL);
    assert_eq!(details["filename"], "Download.CSV");
    assert_eq!(details["skipped"]["neutralPairs"], 1);

    // Relu tel quel : refusé, rien d'écrit.
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 422, "{b}");
    assert_eq!(code(&b), "BANK_IMPORT_DUPLICATE_FILE");
    assert_eq!(ctx.ecrit().await, (3, 15, 4));

    // Le relevé suivant, qui chevauche : deux lignes neuves (trois
    // transactions), le reste en doublons ; USD et CHF sans entrée neuve.
    let (s, b) = ctx.importer(&fichier_chevauchant()).await;
    assert_eq!(s, 201, "{b}");
    let r = par_devise(&b, "EUR");
    assert_eq!(
        (r["inserted"].clone(), r["duplicates"].clone()),
        (json!(3), json!(9))
    );
    let r = par_devise(&b, "USD");
    assert_eq!(r["importId"], Value::Null);
    assert_eq!(
        (r["inserted"].clone(), r["duplicates"].clone()),
        (json!(0), json!(3))
    );
    assert_eq!(ctx.ecrit().await, (4, 18, 6));
}

/// AC-4 — une devise sans compte refuse tout l'import en la nommant ; deux
/// comptes pour une devise (la course de l'AC-2) et un compte clos aussi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refuse_devise_manquante_doublon_de_devise_et_compte_clos(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e = ctx.solde(ctx.eur, None).await["id"].as_i64().unwrap();
    ctx.solde(ctx.usd, None).await;

    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 422, "{b}");
    assert_eq!(code(&b), "PROVIDER_CURRENCY_ACCOUNT_MISSING");
    assert_eq!(b["error"]["details"]["currencies"], json!(["CHF"]));
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // La CHF, puis un second solde EUR écrit en base (la garde de la route
    // n'est pas sérialisée contre un écrivain futur) : refus d'ambiguïté.
    ctx.solde(ctx.chf, None).await;
    let doublon: u64 = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, provider, provider_account) \
         VALUES (?, 'PayPal bis', FALSE, ?, 'paypal', 'paypal WWW')",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.eur2)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id();
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "PROVIDER_CURRENCY_DUPLICATE");
    assert_eq!(b["error"]["details"]["currency"], "EUR");
    assert_eq!(b["error"]["details"]["bankAccountId"], doublon as i64);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(doublon)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
        .bind(e)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "BANK_ACCOUNT_CLOSED");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-4 — ordre des refus de tête : champ manquant (400), compte PayPal
/// inconnu (404), fichier illisible (422 `PAYPAL_CSV_INVALID`, motif du
/// lecteur) ; rôle Consultation et clé `read` refusés (403) ; clé `read-write`
/// admise, la trace porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refus_de_tete_roles_et_cles(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.trois_soldes().await;

    let (s, b) = ctx.importer_avec(&ctx.jwt, FIXTURE, None).await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = ctx
        .importer_avec(&ctx.jwt, FIXTURE, Some("PayPal inconnu"))
        .await;
    assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND"), "{b}");
    assert_eq!(b["error"]["details"]["account"], "PayPal inconnu");
    let (s, b) = ctx
        .importer(b"\"Date\",\"Name\"\r\n\"01/01/2020\",\"x\"\r\n")
        .await;
    assert_eq!((s, code(&b)), (422, "PAYPAL_CSV_INVALID"), "{b}");
    assert_eq!(
        b["error"]["details"]["reason"],
        "PAYPAL_CSV_MISSING_COLUMNS"
    );
    assert!(b["error"]["details"]["columns"].as_array().unwrap().len() > 3);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
    // Revue P1 (L2) — un identifiant de transaction plus long que sa colonne
    // (frais `:FEE` compris) : 422 et erreur de ligne, non une erreur de base.
    let long =
        std::str::from_utf8(FIXTURE)
            .unwrap()
            .replacen("0TEST00000000001A", &"A".repeat(252), 1);
    let (s, b) = ctx.importer(long.as_bytes()).await;
    assert_eq!((s, code(&b)), (422, "PAYPAL_CSV_INVALID"), "{b}");
    assert_eq!(b["error"]["details"]["reason"], "PAYPAL_CSV_LINES");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // Le libellé se compare sans casse.
    let consultation = forge_jwt(
        ctx.seeded.admin_user_id,
        "Consultation",
        ctx.seeded.company_id,
    );
    let (s, _) = ctx
        .importer_avec(&consultation, FIXTURE, Some(PAYPAL))
        .await;
    assert_eq!(s, 403);
    let (_, lecture) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "ro", "read")
            .await;
    let (s, _) = ctx.importer_avec(&lecture, FIXTURE, Some(PAYPAL)).await;
    assert_eq!(s, 403);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    let (cle_id, ecriture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let (s, b) = ctx
        .importer_avec(&ecriture, FIXTURE, Some("paypal WWW"))
        .await;
    assert_eq!(s, 201, "{b}");
    let (actor_type, key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log WHERE action = 'paypal_import.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), key), ("api_key", Some(cle_id)));
}

/// Revue P1 (L1) — un `PUT` qui retire les réglages PayPal du solde EUR
/// (encore sans transaction : la garde 5 passe), non commité pendant que
/// l'import a lu les soldes et pris ses verrous nommés : l'import relit sous
/// ses verrous, attend le commit, voit le compte devenu ordinaire et refuse
/// (`422 PROVIDER_CURRENCY_ACCOUNT_MISSING`, `EUR`), rien d'écrit. Le `PUT`
/// est simulé par sa ligne tenue en exclusif (son `FOR UPDATE`). Tue la
/// relecture sous les verrous (l'import écrirait sur le compte ordinaire).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_relit_les_soldes_sous_ses_verrous(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (e, _, _) = ctx.trois_soldes().await;

    let mut put = pool.begin().await.unwrap();
    sqlx::query(
        "UPDATE bank_accounts SET provider = NULL, provider_account = NULL, \
         fee_account_id = NULL, iban = 'CH9300762011623852957' WHERE id = ?",
    )
    .bind(e)
    .execute(&mut *put)
    .await
    .unwrap();

    let task = {
        let client = ctx.app.client.clone();
        let url = format!("{}/api/v1/paypal-imports", ctx.app.base_url);
        let jwt = ctx.jwt.clone();
        tokio::spawn(async move {
            let form = multipart::Form::new()
                .part(
                    "file",
                    multipart::Part::bytes(FIXTURE.to_vec())
                        .file_name("Download.CSV")
                        .mime_str("text/csv")
                        .unwrap(),
                )
                .text("account", PAYPAL);
            let resp = client
                .post(url)
                .bearer_auth(jwt)
                .multipart(form)
                .send()
                .await
                .unwrap();
            let status = resp.status().as_u16();
            (status, resp.json::<Value>().await.unwrap_or(Value::Null))
        })
    };
    assert!(
        attendre_une_requete_en_cours(
            &pool,
            &["provider_account = ?", "LOCK IN SHARE MODE"],
            || { task.is_finished() }
        )
        .await,
        "l'import doit relire les soldes sous ses verrous, et attendre le PUT"
    );
    put.commit().await.unwrap();

    let (s, b) = task.await.unwrap();
    assert_eq!(
        (s, code(&b)),
        (422, "PROVIDER_CURRENCY_ACCOUNT_MISSING"),
        "{b}"
    );
    assert_eq!(b["error"]["details"]["currencies"], json!(["EUR"]));
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-5 — un verrou tenu sur le second compte (par identifiant) : `409
/// RECONCILIATION_ACCOUNT_LOCKED`, rien d'écrit, et le premier verrou rendu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_verrou_tenu_sur_le_second_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (e, u, c) = ctx.trois_soldes().await;
    let mut ids = [e, u, c];
    ids.sort();
    let base: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let nom = |id: i64| format!("reconcile:{base}:{}:{id}", ctx.seeded.company_id);
    let mut tenant = pool.acquire().await.unwrap();
    let pris: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
        .bind(nom(ids[1]))
        .fetch_one(&mut *tenant)
        .await
        .unwrap();
    assert_eq!(pris, Some(1));

    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(code(&b), "RECONCILIATION_ACCOUNT_LOCKED");
    assert_eq!(b["error"]["details"]["bankAccountId"], ids[1]);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
    let libre: Option<i32> = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
        .bind(nom(ids[0]))
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(libre, Some(1), "le premier verrou est rendu");

    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(nom(ids[1]))
        .fetch_one(&mut *tenant)
        .await
        .unwrap();
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{b}");
}

/// `with_account_locks` — ids triés et dédoublonnés, tous rendus après la
/// closure ; un verrou tenu rend `AccountLocked` sans appeler la closure.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn verrous_de_plusieurs_comptes(pool: MySqlPool) {
    use kesh_reconciliation::{ReconciliationError, with_account_locks};
    let base: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&pool)
        .await
        .unwrap();
    let nom = |id: i64| format!("reconcile:{base}:1:{id}");
    let mut tx = pool.begin().await.unwrap();
    let noms: Vec<String> = [10, 20, 30].into_iter().map(nom).collect();
    let vus = with_account_locks(&mut tx, 1, &[30, 10, 20, 10], 1, async |t| {
        let mut tenus = Vec::new();
        for n in &noms {
            let n: Option<i64> = sqlx::query_scalar("SELECT IS_USED_LOCK(?)")
                .bind(n)
                .fetch_one(&mut **t)
                .await
                .map_err(ReconciliationError::Database)?;
            tenus.push(n.is_some());
        }
        Ok(tenus)
    })
    .await
    .unwrap();
    assert_eq!(vus, [true, true, true]);
    for id in [10, 20, 30] {
        let libre: Option<i32> = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
            .bind(nom(id))
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(libre, Some(1), "verrou {id} rendu");
    }

    let mut tenant = pool.acquire().await.unwrap();
    let _: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
        .bind(nom(20))
        .fetch_one(&mut *tenant)
        .await
        .unwrap();
    let mut appele = false;
    let r = with_account_locks(&mut tx, 1, &[30, 20, 10], 1, async |_| {
        appele = true;
        Ok(())
    })
    .await;
    assert!(
        matches!(
            r,
            Err(ReconciliationError::AccountLocked {
                bank_account_id: 20,
                ..
            })
        ),
        "{r:?}"
    );
    assert!(!appele, "la closure n'est pas appelée");
    let libre: Option<i32> = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
        .bind(nom(10))
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(libre, Some(1), "le verrou 10, pris avant, est rendu");
}

// ============================================================
// AC-2, AC-3 — les réglages d'un compte PayPal
// ============================================================

/// AC-2, AC-3 — un solde PayPal EUR se crée sans IBAN, lié à un compte du plan
/// EUR ; la réponse porte `provider` et `paypal` ; les refus de forme ; le même
/// compte du plan sans `paypal` reste refusé (`ACCOUNT_IN_FOREIGN_CURRENCY`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creer_un_solde_paypal_et_ses_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let b = ctx.solde(ctx.eur, Some(ctx.frais_eur)).await;
    assert_eq!(b["provider"], "paypal");
    assert_eq!(
        b["paypal"],
        json!({ "account": PAYPAL, "feeAccountId": ctx.frais_eur })
    );
    assert_eq!(b["iban"], Value::Null);
    assert_eq!(b["accountKind"], "bank");
    let id = b["id"].as_i64().unwrap();
    let trace: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.created' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(trace["provider"], "paypal");
    assert_eq!(trace["provider_account"], PAYPAL);
    assert_eq!(trace["fee_account_id"], ctx.frais_eur);

    // La liste porte les champs, un compte ordinaire les porte à `null`.
    let (s, liste) = ctx
        .req(reqwest::Method::GET, "/api/v1/bank-accounts", None)
        .await;
    assert_eq!(s, 200);
    let item = liste
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == id)
        .unwrap()
        .clone();
    assert_eq!(item["provider"], "paypal");

    let avant = ctx.count("SELECT COUNT(*) FROM bank_accounts").await;
    let base = |paypal: Value| json!({ "bankName": "PayPal", "journalAccountId": ctx.usd, "paypal": paypal });
    let (s, b) = ctx
        .post("/api/v1/bank-accounts", base(json!({ "account": "   " })))
        .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            base(json!({ "account": "x".repeat(121) })),
        )
        .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            base(json!({ "account": "4111 1111 1111 1111" })),
        )
        .await;
    assert_eq!((s, code(&b)), (400, "CARD_NUMBER_NOT_ALLOWED"), "{b}");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "paypal": { "account": PAYPAL } }),
        )
        .await;
    assert_eq!((s, code(&b)), (400, "PROVIDER_LEDGER_REQUIRED"), "{b}");
    // Un second solde EUR du même compte PayPal (libellé sans casse).
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": ctx.eur2,
        "paypal": { "account": "paypal WWW" } }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "PROVIDER_CURRENCY_DUPLICATE"), "{b}");
    assert_eq!(b["error"]["details"]["bankAccountId"], id);
    assert_eq!(b["error"]["details"]["currency"], "EUR");
    // Les quatre motifs du compte de frais.
    for (fee, motif) in [
        (999_999, "not_found"),
        (ctx.regroupement, "not_postable"),
        (ctx.produit, "not_expense"),
        (ctx.frais_eur, "currency"),
    ] {
        let (s, b) = ctx
            .post(
                "/api/v1/bank-accounts",
                base(json!({ "account": PAYPAL, "feeAccountId": fee })),
            )
            .await;
        assert_eq!(
            (s, code(&b)),
            (400, "PROVIDER_FEE_ACCOUNT_INVALID"),
            "{motif} : {b}"
        );
        assert_eq!(b["error"]["details"]["reason"], motif);
    }
    // QR-IBAN, principal, carte.
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": ctx.usd,
        "qrIban": "CH4431999123000889012", "paypal": { "account": PAYPAL } }),
        )
        .await;
    assert_eq!(
        (s, code(&b)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{b}"
    );
    assert_eq!(b["error"]["details"]["field"], "qrIban");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": ctx.usd,
        "isPrimary": true, "paypal": { "account": PAYPAL } }),
        )
        .await;
    assert_eq!(
        (s, code(&b)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{b}"
    );
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Visa", "accountKind": "credit_card",
        "journalAccountId": ctx.seeded.accounts["2000"], "paypal": { "account": PAYPAL } }),
        )
        .await;
    assert_eq!((s, code(&b)), (400, "CREDIT_CARD_FIELD_NOT_ALLOWED"), "{b}");
    // Sans `paypal`, un compte du plan en devise reste refusé (37-1).
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "UBS", "iban": "CH9300762011623852957",
        "journalAccountId": ctx.usd }),
        )
        .await;
    assert_eq!((s, code(&b)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM bank_accounts").await,
        avant,
        "rien d'écrit"
    );

    // Un solde USD avec un compte de frais USD, et un IBAN donné (contrôlé).
    let (s, b) = ctx.post("/api/v1/bank-accounts", json!({ "bankName": "PayPal", "journalAccountId": ctx.usd,
        "iban": "CH93 0076 2011 6238 5295 7", "paypal": { "account": PAYPAL, "feeAccountId": ctx.frais_usd } }))
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["iban"], "CH9300762011623852957");
}

/// AC-2, AC-3 — `PUT` et `PATCH` d'un solde PayPal : `paypal` absent garde les
/// réglages, `iban: null` admis ; retirer les réglages d'un compte EUR rend
/// `ACCOUNT_IN_FOREIGN_CURRENCY` ; après un import, changer le libellé ou le
/// retirer rend `PROVIDER_ACCOUNT_IN_USE` (le compte de frais se change) ; le
/// `PATCH` est refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modifier_un_solde_paypal(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let b = ctx.solde(ctx.eur, None).await;
    let id = b["id"].as_i64().unwrap();
    let chemin = format!("/api/v1/bank-accounts/{id}");
    let version = |b: &Value| b["version"].as_i64().unwrap();

    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur,
            "iban": null, "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["paypal"]["account"], PAYPAL, "absent = inchangé");
    assert_eq!(b["bankName"], "PayPal EUR");

    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur,
            "iban": "CH9300762011623852957", "paypal": null, "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{r}");

    let (s, r) = ctx
        .req(
            reqwest::Method::PATCH,
            &chemin,
            Some(json!({ "journalAccountId": ctx.eur2, "version": version(&b) })),
        )
        .await;
    assert_eq!(
        (s, code(&r)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{r}"
    );
    assert_eq!(r["error"]["details"]["field"], "journalAccountId");

    // Les autres soldes, puis un import : le compte porte des transactions.
    ctx.solde(ctx.usd, None).await;
    ctx.solde(ctx.chf, None).await;
    let (s, r) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{r}");

    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur,
            "paypal": { "account": "PayPal autre" }, "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (409, "PROVIDER_ACCOUNT_IN_USE"), "{r}");
    // Retirer : le 409 passe avant le refus de devise.
    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur,
            "paypal": null, "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (409, "PROVIDER_ACCOUNT_IN_USE"), "{r}");
    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur2,
            "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (409, "PROVIDER_ACCOUNT_IN_USE"), "{r}");
    let (s, r) = ctx
        .put(&chemin, json!({ "bankName": "PayPal EUR", "journalAccountId": ctx.eur,
            "paypal": { "account": PAYPAL, "feeAccountId": ctx.frais_eur }, "version": version(&b) }))
        .await;
    assert_eq!(s, 200, "le compte de frais se change : {r}");
    assert_eq!(r["paypal"]["feeAccountId"], ctx.frais_eur);
    let trace: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.updated' AND entity_id = ? \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(trace["after"]["fee_account_id"], ctx.frais_eur);
    assert_eq!(trace["before"]["fee_account_id"], Value::Null);
}

// ============================================================
// AC-3 — les autres chemins refusent un compte PayPal
// ============================================================

/// Revue P1 (M3) — la devise du compte du plan d'un solde PayPal ne change
/// pas, même sans ligne d'écriture : EUR → CHF (devise retirée) et EUR → USD
/// → `409 ACCOUNT_CURRENCY_PROVIDER_LINKED` (`details.bankAccountId`), la
/// devise reste EUR ; un compte EUR sans solde PayPal la change toujours.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_devise_du_compte_du_plan_d_un_solde_paypal_ne_change_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let solde = ctx.solde(ctx.eur, None).await["id"].as_i64().unwrap();
    let modifier = |id: i64, devise: Value| {
        let ctx = &ctx;
        async move {
            let (nom, version): (String, i32) =
                sqlx::query_as("SELECT name, version FROM accounts WHERE id = ?")
                    .bind(id)
                    .fetch_one(&ctx.pool)
                    .await
                    .unwrap();
            ctx.put(
                &format!("/api/v1/accounts/{id}"),
                json!({ "name": nom, "accountType": "Asset", "role": null,
                    "postable": true, "version": version, "currency": devise }),
            )
            .await
        }
    };
    for devise in [Value::Null, json!("USD")] {
        let (s, b) = modifier(ctx.eur, devise.clone()).await;
        assert_eq!(
            (s, code(&b)),
            (409, "ACCOUNT_CURRENCY_PROVIDER_LINKED"),
            "{devise} : {b}"
        );
        assert_eq!(b["error"]["details"]["bankAccountId"], solde);
    }
    let devise: Option<String> = sqlx::query_scalar("SELECT currency FROM accounts WHERE id = ?")
        .bind(ctx.eur)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(devise.as_deref(), Some("EUR"));
    // Témoin : le compte EUR sans solde PayPal change de devise.
    let (s, b) = modifier(ctx.eur2, Value::Null).await;
    assert_eq!(s, 200, "{b}");
}

/// AC-3 — import CAMT et relevé manuel, règlement de facture, paiement de
/// facture fournisseur sur un compte PayPal : `409 BANK_ACCOUNT_IS_PROVIDER`,
/// rien d'écrit ; lot pain.001 depuis un compte PayPal, sans IBAN ou avec :
/// `409 BANK_ACCOUNT_IS_PROVIDER` dès la création (revue P1, M2) ;
/// solde d'ouverture d'un compte PayPal EUR : refus central des lignes en
/// devise, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_autres_chemins_refusent_un_compte_paypal(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let chf = ctx.solde(ctx.chf, None).await["id"].as_i64().unwrap();
    let jour = common::jour::aujourd_hui_test().to_string();

    // Import CAMT (aperçu et création).
    let camt = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-import/tests/fixtures/camt053/v04_characterization.xml"),
    )
    .expect("fixture CAMT");
    for route in ["/api/v1/bank-imports/preview", "/api/v1/bank-imports"] {
        let form = multipart::Form::new()
            .text("bankAccountId", chf.to_string())
            .part(
                "file",
                multipart::Part::bytes(camt.clone())
                    .file_name("releve.xml")
                    .mime_str("application/xml")
                    .unwrap(),
            );
        let resp = ctx
            .app
            .client
            .post(format!("{}{route}", ctx.app.base_url))
            .bearer_auth(&ctx.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let s = resp.status().as_u16();
        let b: Value = resp.json().await.unwrap();
        assert_eq!(
            (s, code(&b)),
            (409, "BANK_ACCOUNT_IS_PROVIDER"),
            "{route} : {b}"
        );
        assert_eq!(b["error"]["details"]["bankAccountId"], chf);
    }
    // Relevé saisi à la main (aperçu et création).
    let releve = json!({
        "bankAccountId": chf, "periodFrom": "2026-09-01", "periodTo": "2026-09-30",
        "openingBalance": "0.00", "closingBalance": "50.00",
        "lines": [{ "bookingDate": "2026-09-15", "valueDate": null, "details": "Vente",
            "counterpartyName": null, "reference": null, "amount": "50.00", "direction": "credit" }]
    });
    for route in [
        "/api/v1/bank-imports/manual/preview",
        "/api/v1/bank-imports/manual",
    ] {
        let (s, b) = ctx.post(route, releve.clone()).await;
        assert_eq!(
            (s, code(&b)),
            (409, "BANK_ACCOUNT_IS_PROVIDER"),
            "{route} : {b}"
        );
    }
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    // Règlement d'une facture client.
    let client = ctx.contact("Client", false).await;
    let (s, b) = ctx
        .post("/api/v1/invoices", json!({ "contactId": client, "date": jour,
            "lines": [{"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}] }))
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
    let ecritures = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/invoices/{invoice}/settlements"),
            json!({
            "settlementType": "bank_transfer", "bankAccountId": chf, "amount": "108.10",
            "settledOn": jour, "version": b["version"] }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        ecritures,
        "règlement : rien d'écrit"
    );

    // Paiement d'une facture fournisseur.
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
    let ecritures = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx
        .post(&format!("/api/v1/supplier-invoices/{ff}/pay"),
            json!({ "settlementType": "bank_transfer", "bankAccountId": chf, "paymentDate": "2026-06-20" }))
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    let ecritures_apres = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    assert_eq!(ecritures_apres, ecritures, "rien d'écrit");

    // Lot pain.001 (revue P1, M2) : refusé dès la création, avec ou sans IBAN
    // (le refus du prestataire passe avant celui de l'IBAN), aucun lot.
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "iban": "CH9300762011623852957",
            "journalAccountId": ctx.eur2, "paypal": { "account": "PayPal pro" } }),
        )
        .await;
    assert_eq!(s, 201, "solde PayPal avec IBAN : {b}");
    let avec_iban = b["id"].as_i64().unwrap();
    for compte in [chf, avec_iban] {
        let (s, b) = ctx
            .post(
                "/api/v1/payment-batches",
                json!({ "bankAccountId": compte,
                "requestedExecutionDate": "2026-06-25", "supplierInvoiceIds": [ff] }),
            )
            .await;
        assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
        assert_eq!(b["error"]["details"]["bankAccountId"], compte);
    }
    assert_eq!(ctx.count("SELECT COUNT(*) FROM payment_batches").await, 0);

    // Solde d'ouverture d'un compte PayPal EUR : refusé par le contrôle central
    // des lignes en devise, ni compte ni écriture. (Le report à nouveau, rôle
    // exigé par le complément, est posé d'abord.)
    sqlx::query("UPDATE accounts SET role = 'RetainedEarnings' WHERE id = ?")
        .bind(ctx.seeded.accounts["2000"])
        .execute(&pool)
        .await
        .unwrap();
    // L'exercice multi-devises : sans lui, le refus serait `MULTI_CURRENCY_DISABLED`.
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE")
        .execute(&pool)
        .await
        .unwrap();
    let comptes = ctx.count("SELECT COUNT(*) FROM bank_accounts").await;
    let (s, b) = ctx
        .post("/api/v1/bank-accounts", json!({ "bankName": "PayPal", "journalAccountId": ctx.eur,
            "paypal": { "account": PAYPAL }, "openingBalance": { "amount": "10.00", "side": "debit" } }))
        .await;
    assert_eq!(code(&b), "ACCOUNT_REQUIRES_CURRENCY", "{s} {b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM bank_accounts").await,
        comptes
    );
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        ecritures
    );
}
