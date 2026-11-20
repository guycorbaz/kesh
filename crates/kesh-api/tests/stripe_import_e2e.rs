//! Story 40-7b (Epic 40, phase 1) — le compte Stripe et l'import de son
//! rapport, par les routes : réglages Stripe d'un compte bancaire dont le
//! compte de versement (AC-2), refus aux autres chemins (AC-3), route d'import
//! (AC-4), ventilation et transactions de versement (AC-5), doublons et compte
//! rendu (AC-6), tenue de l'import PayPal par le cœur commun (AC-7).
//!
//! Fixtures fictives seulement : le CSV de 40-7a
//! (`kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv`), des
//! comptes « Stripe boutique » sans existence réelle.

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
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use reqwest::multipart;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const FIXTURE: &[u8] =
    include_bytes!("../../kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv");
const SANS_SOURCE: &[u8] = include_bytes!(
    "../../kesh-import/tests/fixtures/stripe-itemized-payout-sans-source-fictif.csv"
);
const STRIPE: &str = "Stripe boutique";

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
/// Stripe, comptes de frais (charge CHF, charge EUR, produit, charge non
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

    /// Le solde Stripe lié à `ledger`, créé par la route ; rend le corps.
    async fn solde_avec(&self, ledger: i64, fee: Option<i64>, payout: Option<i64>) -> Value {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "Stripe",
                    "journalAccountId": ledger,
                    "stripe": { "account": STRIPE, "feeAccountId": fee, "payoutBankAccountId": payout },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde Stripe : {b}");
        b
    }

    /// Un compte bancaire de destination de versement, écrit en base : sans
    /// prestataire (avec IBAN) ou d'un prestataire donné (Wise, PayPal : sans
    /// IBAN) ; rend son identifiant.
    async fn destination(&self, ledger: i64, provider: Option<&str>) -> i64 {
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id, \
             provider, provider_account) VALUES (?, 'Destination', ?, FALSE, ?, ?, ?)",
        )
        .bind(self.seeded.company_id)
        .bind(provider.is_none().then_some("CH9300762011623852957"))
        .bind(ledger)
        .bind(provider)
        .bind(provider.map(|_| "Libellé destination"))
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// Les trois soldes EUR, USD, CHF ; rend leurs identifiants.
    async fn trois_soldes(&self) -> (i64, i64, i64) {
        let e = self.solde_avec(self.eur, Some(self.frais_eur), None).await;
        let u = self.solde_avec(self.usd, None, None).await;
        let c = self.solde_avec(self.chf, Some(self.frais_chf), None).await;
        (
            e["id"].as_i64().unwrap(),
            u["id"].as_i64().unwrap(),
            c["id"].as_i64().unwrap(),
        )
    }

    /// `POST /api/v1/stripe-imports`, avec le jeton donné.
    async fn importer_avec(
        &self,
        jeton: &str,
        fichier: &[u8],
        compte: Option<&str>,
    ) -> (u16, Value) {
        let mut form = multipart::Form::new().part(
            "file",
            multipart::Part::bytes(fichier.to_vec())
                .file_name("payouts.csv")
                .mime_str("text/csv")
                .unwrap(),
        );
        if let Some(c) = compte {
            form = form.text("account", c.to_string());
        }
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/stripe-imports", self.app.base_url))
            .bearer_auth(jeton)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn importer(&self, fichier: &[u8]) -> (u16, Value) {
        self.importer_avec(&self.jwt, fichier, Some(STRIPE)).await
    }

    async fn ecrit(&self) -> (i64, i64, i64) {
        (
            self.count("SELECT COUNT(*) FROM bank_imports").await,
            self.count("SELECT COUNT(*) FROM bank_transactions").await,
            self.count(
                "SELECT COUNT(*) FROM audit_log WHERE action IN ('bank_import.created', 'stripe_import.created')",
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

/// Le rapport d'une seule devise : les lignes de l'en-tête et celles de `devise`.
fn fichier_d_une_devise(devise: &str) -> Vec<u8> {
    let texte = std::str::from_utf8(FIXTURE).unwrap();
    let mut lignes = texte.lines();
    let mut sortie = vec![lignes.next().unwrap().to_string()];
    sortie.extend(
        lignes
            .filter(|l| l.split(',').nth(5) == Some(devise))
            .map(str::to_string),
    );
    (sortie.join("\n") + "\n").into_bytes()
}

fn par_devise(b: &Value, devise: &str) -> Value {
    b["currencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == devise)
        .cloned()
        .unwrap()
}

// ============================================================
// AC-4 à AC-6 — l'import
// ============================================================

/// AC-4, AC-5, AC-6 — le rapport de 40-7a se ventile en trois entrées (CHF, EUR,
/// USD), chaque ligne en brut et frais, chaque versement en une transaction ;
/// relu, il est refusé ; le rapport d'une devise puis celui de toutes les
/// devises ne doublent rien ; la somme de chaque compte est nulle (versements
/// à Σ net ≠ 0 compris).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_ventile_par_devise_puis_doublons(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (e, u, c) = ctx.trois_soldes().await;

    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["account"], STRIPE);
    for (devise, id, inserted) in [("CHF", c, 8), ("EUR", e, 7), ("USD", u, 5)] {
        let r = par_devise(&b, devise);
        assert_eq!(r["bankAccountId"], id, "{devise}");
        assert_eq!(r["inserted"], inserted, "{devise}");
        assert_eq!(r["duplicates"], 0, "{devise}");
        assert!(r["importId"].is_i64(), "{devise}");
    }
    assert_eq!(
        b["payouts"],
        json!([
            { "payoutId": "po_TEST0001", "currency": "CHF", "lines": 4, "net": "125.15" },
            { "payoutId": "po_TEST0002", "currency": "EUR", "lines": 3, "net": "270.30" },
            { "payoutId": "po_TEST0003", "currency": "USD", "lines": 1, "net": "38.54" },
            { "payoutId": "po_TEST0004", "currency": "USD", "lines": 1, "net": "-38.54" },
        ])
    );
    assert_eq!(ctx.ecrit().await, (3, 20, 4));

    // La somme des transactions de chaque compte est nulle : le brut moins les
    // frais de chaque ligne vaut son net, le versement reprend la somme des nets.
    for id in [e, u, c] {
        let somme: Decimal = sqlx::query_scalar(
            "SELECT SUM(amount) FROM bank_transactions WHERE bank_account_id = ?",
        )
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(somme, dec!(0), "compte {id}");
    }
    // Brut : la référence est l'objet lié, le versement et la catégorie suivent.
    let (provider_type, reference, versement, valeur, details): (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<NaiveDate>,
        String,
    ) = sqlx::query_as(
        "SELECT provider_type, reference, end_to_end_id, value_date, details FROM bank_transactions \
         WHERE transaction_id = 'txn_TEST0001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(provider_type.as_deref(), Some("charge"));
    assert_eq!(reference.as_deref(), Some("ch_TEST0001"));
    assert_eq!(versement.as_deref(), Some("po_TEST0001"));
    assert_eq!(valeur, NaiveDate::from_ymd_opt(2020, 2, 3));
    assert_eq!(
        details,
        "Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF BRUT - txn_TEST0001 (2020-01-29 10:15:04- Vente fictive A) - po_TEST0001"
    );
    // Frais : opposé du champ `fee` ; frais rendus d'un remboursement : positifs.
    let frais: Decimal = sqlx::query_scalar(
        "SELECT amount FROM bank_transactions WHERE transaction_id = 'txn_TEST0001:FEE'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frais, dec!(-3.20));
    let frais_rendus: Decimal = sqlx::query_scalar(
        "SELECT amount FROM bank_transactions WHERE transaction_id = 'txn_TEST0003:FEE'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(frais_rendus, dec!(0.60));
    // Une ligne sans frais ni référence n'a pas de transaction de frais.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM bank_transactions WHERE transaction_id = 'txn_TEST0004:FEE'"
        )
        .await,
        0
    );
    let (versement_montant, date, type_): (Decimal, NaiveDate, Option<String>) = sqlx::query_as(
        "SELECT amount, booking_date, provider_type FROM bank_transactions WHERE transaction_id = 'po_TEST0001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(versement_montant, dec!(-125.15));
    assert_eq!(date, NaiveDate::from_ymd_opt(2020, 2, 3).unwrap());
    assert_eq!(type_.as_deref(), Some("payout"));
    // Le retour d'un versement (Σ net négatif) entre au solde.
    let retour: Decimal = sqlx::query_scalar(
        "SELECT amount FROM bank_transactions WHERE transaction_id = 'po_TEST0004'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(retour, dec!(38.54));
    // L'entrée d'import : source CSV, période, soldes absents.
    let (format, du, au, ouverture, cloture, nombre): (
        String,
        NaiveDate,
        NaiveDate,
        Option<Decimal>,
        Option<Decimal>,
        i32,
    ) = sqlx::query_as(
        "SELECT source_format, period_from, period_to, opening_balance, closing_balance, transaction_count \
         FROM bank_imports WHERE bank_account_id = ?",
    )
    .bind(c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(format, "CSV");
    assert_eq!(du, NaiveDate::from_ymd_opt(2020, 1, 29).unwrap());
    assert_eq!(au, NaiveDate::from_ymd_opt(2020, 2, 3).unwrap());
    assert_eq!((ouverture, cloture, nombre), (None, None, 8));
    // La trace de l'import.
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'stripe_import.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["provider"], "stripe");
    assert_eq!(details["provider_account"], STRIPE);
    assert_eq!(details["filename"], "payouts.csv");
    assert_eq!(details["payouts"].as_array().unwrap().len(), 4);

    // Relu tel quel : refusé, rien d'écrit.
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 422, "{b}");
    assert_eq!(code(&b), "BANK_IMPORT_DUPLICATE_FILE");
    assert_eq!(ctx.ecrit().await, (3, 20, 4));
}

/// AC-6 — un rapport par devise, puis le rapport toutes devises de la même
/// période : chaque transaction connue est ignorée et comptée, aucune n'est
/// doublée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_rapport_par_devise_puis_tous_ne_doublent_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.trois_soldes().await;

    for (devise, inserted) in [("chf", 8), ("eur", 7)] {
        let (s, b) = ctx.importer(&fichier_d_une_devise(devise)).await;
        assert_eq!(s, 201, "{devise} : {b}");
        assert_eq!(
            par_devise(&b, &devise.to_uppercase())["inserted"],
            inserted,
            "{devise}"
        );
    }
    assert_eq!(ctx.ecrit().await, (2, 15, 4));
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{b}");
    let chf = par_devise(&b, "CHF");
    assert_eq!(
        (
            chf["inserted"].clone(),
            chf["duplicates"].clone(),
            chf["importId"].clone()
        ),
        (json!(0), json!(8), Value::Null)
    );
    let eur = par_devise(&b, "EUR");
    assert_eq!(
        (eur["inserted"].clone(), eur["duplicates"].clone()),
        (json!(0), json!(7))
    );
    let usd = par_devise(&b, "USD");
    assert_eq!(
        (usd["inserted"].clone(), usd["duplicates"].clone()),
        (json!(5), json!(0))
    );
    // Aucune transaction doublée : les identifiants sont uniques par compte.
    assert_eq!(ctx.ecrit().await, (3, 20, 6));
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM (SELECT bank_account_id, transaction_id FROM bank_transactions \
             GROUP BY bank_account_id, transaction_id HAVING COUNT(*) > 1) d"
        )
        .await,
        0
    );
}

/// AC-4 — une devise sans compte refuse tout l'import en la nommant ; deux
/// comptes pour une devise et un compte clos aussi ; rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refuse_devise_manquante_doublon_de_devise_et_compte_clos(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e = ctx.solde_avec(ctx.eur, None, None).await["id"]
        .as_i64()
        .unwrap();
    ctx.solde_avec(ctx.usd, None, None).await;

    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 422, "{b}");
    assert_eq!(code(&b), "PROVIDER_CURRENCY_ACCOUNT_MISSING");
    assert_eq!(b["error"]["details"]["currencies"], json!(["CHF"]));
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    ctx.solde_avec(ctx.chf, None, None).await;
    let doublon: u64 = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, provider, provider_account) \
         VALUES (?, 'Stripe bis', FALSE, ?, 'stripe', 'stripe BOUTIQUE')",
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

/// AC-4 — ordre des refus de tête : champ manquant (400), compte Stripe
/// inconnu (404 — un compte PayPal du même libellé n'est pas un compte Stripe),
/// fichier illisible (422 `STRIPE_CSV_INVALID`, motif du lecteur), objet lié
/// absent (la marche à suivre dans Stripe), identifiant trop long ; rôle
/// Consultation et clé `read` refusés (403) ; clé `read-write` admise, la
/// trace porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_refus_de_tete_roles_et_cles(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.trois_soldes().await;
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, provider, provider_account) \
         VALUES (?, 'PayPal', FALSE, ?, 'paypal', 'PayPal www')",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.eur2)
    .execute(&pool)
    .await
    .unwrap();

    let (s, b) = ctx.importer_avec(&ctx.jwt, FIXTURE, None).await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
    for inconnu in ["Stripe inconnu", "PayPal www"] {
        let (s, b) = ctx.importer_avec(&ctx.jwt, FIXTURE, Some(inconnu)).await;
        assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND"), "{b}");
        assert_eq!(b["error"]["details"]["account"], inconnu);
    }
    let (s, b) = ctx
        .importer(b"automatic_payout_id,currency\npo_X,chf\n")
        .await;
    assert_eq!((s, code(&b)), (422, "STRIPE_CSV_INVALID"), "{b}");
    assert_eq!(
        b["error"]["details"]["reason"],
        "STRIPE_CSV_MISSING_COLUMNS"
    );
    assert!(b["error"]["details"]["columns"].as_array().unwrap().len() > 3);
    // Sans `source_id` ni `charge_id` : la marche à suivre.
    let (s, b) = ctx.importer(SANS_SOURCE).await;
    assert_eq!((s, code(&b)), (422, "STRIPE_CSV_INVALID"), "{b}");
    assert_eq!(
        b["error"]["details"]["columns"],
        json!(["source_id|charge_id"])
    );
    let aide = b["error"]["details"]["hint"].as_str().unwrap();
    assert!(aide.contains("source_id"), "{aide}");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
    // Un identifiant plus long que sa colonne (frais `:FEE` compris) : 422 et
    // erreur de ligne, non une erreur de base.
    let long = std::str::from_utf8(FIXTURE)
        .unwrap()
        .replacen("txn_TEST0001", &"a".repeat(252), 1);
    let (s, b) = ctx.importer(long.as_bytes()).await;
    assert_eq!((s, code(&b)), (422, "STRIPE_CSV_INVALID"), "{b}");
    assert_eq!(b["error"]["details"]["reason"], "STRIPE_CSV_LINES");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));

    let consultation = forge_jwt(
        ctx.seeded.admin_user_id,
        "Consultation",
        ctx.seeded.company_id,
    );
    let (s, _) = ctx
        .importer_avec(&consultation, FIXTURE, Some(STRIPE))
        .await;
    assert_eq!(s, 403);
    let (_, lecture) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "ro", "read")
            .await;
    let (s, _) = ctx.importer_avec(&lecture, FIXTURE, Some(STRIPE)).await;
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
        .importer_avec(&ecriture, FIXTURE, Some("stripe BOUTIQUE"))
        .await;
    assert_eq!(s, 201, "{b}");
    let (actor_type, key): (String, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log WHERE action = 'stripe_import.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), key), ("api_key", Some(cle_id)));
}

// ============================================================
// AC-2 — réglages Stripe d'un compte bancaire
// ============================================================

/// AC-2 — création d'un solde Stripe : compte de versement admis (banque
/// ordinaire, compte Wise) et facultatif ; chaque refus de la garde du compte
/// de versement (`not_found`, `archived`, `closed`, `stripe`, `paypal`,
/// `currency`) ; `payoutBankAccountId` refusé pour PayPal ; `paypal` et
/// `stripe` ensemble refusés. Aucun compte créé par un refus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creer_un_solde_stripe_et_ses_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.seeded.company_id;
    let chf_banque = compte(&pool, c, "1034", "Asset", None, true).await;
    let usd2 = compte(&pool, c, "1035", "Asset", Some("USD"), true).await;
    let banque = ctx.destination(chf_banque, None).await;
    let wise_eur = ctx.destination(ctx.eur2, Some("wise")).await;
    let wise_usd = ctx.destination(usd2, Some("wise")).await;

    // CHF → banque ordinaire ; la réponse et la trace portent le compte.
    let b = ctx
        .solde_avec(ctx.chf, Some(ctx.frais_chf), Some(banque))
        .await;
    assert_eq!(b["provider"], "stripe");
    assert_eq!(b["paypal"], Value::Null);
    assert_eq!(
        b["stripe"],
        json!({ "account": STRIPE, "feeAccountId": ctx.frais_chf, "payoutBankAccountId": banque })
    );
    let stripe_chf = b["id"].as_i64().unwrap();
    let trace: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.created' AND entity_id = ?",
    )
    .bind(stripe_chf)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(trace["payout_bank_account_id"], banque);
    assert_eq!(trace["provider"], "stripe");

    // Chaque refus du compte de versement, pour un solde EUR.
    let comptes = ctx.count("SELECT COUNT(*) FROM bank_accounts").await;
    let paypal_eur = ctx.destination(ctx.eur2, Some("paypal")).await;
    let archive = ctx.destination(ctx.eur2, Some("wise")).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let clos = ctx.destination(ctx.eur2, Some("wise")).await;
    sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
        .bind(clos)
        .execute(&pool)
        .await
        .unwrap();
    let comptes = comptes + 3;
    for (cible, raison) in [
        (999_999, "not_found"),
        (archive, "archived"),
        (clos, "closed"),
        (stripe_chf, "stripe"),
        (paypal_eur, "paypal"),
        (banque, "currency"),
    ] {
        let (s, r) = ctx
            .post(
                "/api/v1/bank-accounts",
                json!({ "bankName": "Stripe", "journalAccountId": ctx.eur,
                    "stripe": { "account": STRIPE, "payoutBankAccountId": cible } }),
            )
            .await;
        assert_eq!(
            (s, code(&r)),
            (400, "STRIPE_PAYOUT_ACCOUNT_INVALID"),
            "{raison} : {r}"
        );
        assert_eq!(r["error"]["details"]["reason"], raison);
        assert_eq!(r["error"]["details"]["bankAccountId"], cible);
        assert_eq!(
            ctx.count("SELECT COUNT(*) FROM bank_accounts").await,
            comptes,
            "{raison} : aucun compte créé"
        );
    }
    // Un compte d'une autre société n'existe pas pour cette société.
    // EUR → compte Wise EUR ; USD → compte Wise USD ; sans destination admis.
    ctx.solde_avec(ctx.eur, None, Some(wise_eur)).await;
    ctx.solde_avec(ctx.usd, None, Some(wise_usd)).await;

    // Hors Stripe : `payoutBankAccountId` d'un objet PayPal, et les deux objets.
    let (s, r) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": ctx.eur2,
                "paypal": { "account": "PayPal www", "payoutBankAccountId": wise_eur } }),
        )
        .await;
    assert_eq!((s, code(&r)), (400, "VALIDATION_ERROR"), "{r}");
    let (s, r) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Les deux", "journalAccountId": ctx.eur2,
                "paypal": { "account": "PayPal www" }, "stripe": { "account": "Stripe 2" } }),
        )
        .await;
    assert_eq!((s, code(&r)), (400, "VALIDATION_ERROR"), "{r}");
    // Un compte Stripe sans compte du plan lié : les gardes communes.
    let (s, r) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Stripe", "stripe": { "account": "Stripe 3" } }),
        )
        .await;
    assert_eq!((s, code(&r)), (400, "PROVIDER_LEDGER_REQUIRED"), "{r}");
    // Un second solde de la même devise pour ce compte Stripe : même garde que PayPal.
    let (s, r) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Stripe", "journalAccountId": ctx.chf,
                "stripe": { "account": "stripe BOUTIQUE" } }),
        )
        .await;
    assert_eq!((s, code(&r)), (409, "PROVIDER_CURRENCY_DUPLICATE"), "{r}");
    // Le compte Stripe est un compte « fournisseur » : jamais principal.
    let (s, r) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Stripe", "journalAccountId": ctx.eur2, "isPrimary": true,
                "stripe": { "account": "Stripe 4" } }),
        )
        .await;
    assert_eq!(
        (s, code(&r)),
        (400, "BANK_ACCOUNT_FIELD_NOT_ALLOWED"),
        "{r}"
    );
}

/// AC-2 — modifier un solde Stripe : `stripe` absent = inchangé ; `paypal:
/// null` ne retire pas un compte Stripe ; le compte de versement se pose, se
/// change et se retire ; sur un compte qui porte des transactions le compte de
/// versement se change encore, mais ni le libellé, ni le compte du plan, ni le
/// prestataire, ni le retrait des réglages (409 `PROVIDER_ACCOUNT_IN_USE`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modifier_un_solde_stripe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.seeded.company_id;
    let chf_banque = compte(&pool, c, "1034", "Asset", None, true).await;
    let banque = ctx.destination(chf_banque, None).await;
    let chf_banque2 = compte(&pool, c, "1036", "Asset", None, true).await;
    let banque2 = ctx.destination(chf_banque2, None).await;

    let b = ctx.solde_avec(ctx.chf, None, Some(banque)).await;
    let id = b["id"].as_i64().unwrap();
    let chemin = format!("/api/v1/bank-accounts/{id}");
    let version = |b: &Value| b["version"].as_i64().unwrap();

    // Absent = inchangé.
    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf, "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["stripe"]["account"], STRIPE);
    assert_eq!(b["stripe"]["payoutBankAccountId"], banque);
    assert_eq!(b["bankName"], "Stripe CHF");
    // `paypal: null` ne retire pas un compte Stripe.
    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf, "paypal": null,
                "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["provider"], "stripe");
    // Le compte de versement se change, se retire, se repose.
    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf,
                "stripe": { "account": STRIPE, "payoutBankAccountId": banque2 }, "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["stripe"]["payoutBankAccountId"], banque2);
    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf,
                "stripe": { "account": STRIPE, "payoutBankAccountId": null }, "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["stripe"]["payoutBankAccountId"], Value::Null);
    // Un compte de versement refusé au `PUT` comme au `POST`.
    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf,
                "stripe": { "account": STRIPE, "payoutBankAccountId": id }, "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (400, "STRIPE_PAYOUT_ACCOUNT_INVALID"), "{r}");
    assert_eq!(
        r["error"]["details"]["reason"], "stripe",
        "un compte ne se désigne pas lui-même"
    );
    let (s, b) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf,
                "stripe": { "account": STRIPE, "payoutBankAccountId": banque }, "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "{b}");

    // Un import : le compte porte des transactions.
    ctx.solde_avec(ctx.eur, None, None).await;
    ctx.solde_avec(ctx.usd, None, None).await;
    let (s, r) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{r}");

    for (corps, attendu) in [
        // libellé, retrait, prestataire, compte du plan : refusés.
        (json!({ "stripe": { "account": "Stripe autre" } }), 409),
        (json!({ "stripe": null }), 409),
        (json!({ "paypal": { "account": STRIPE } }), 409),
    ] {
        let mut corps = corps;
        corps["bankName"] = json!("Stripe CHF");
        corps["journalAccountId"] = json!(ctx.chf);
        corps["version"] = json!(version(&b));
        let (s, r) = ctx.put(&chemin, corps.clone()).await;
        assert_eq!(
            (s, code(&r)),
            (attendu, "PROVIDER_ACCOUNT_IN_USE"),
            "{corps} : {r}"
        );
    }
    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": chf_banque2, "version": version(&b) }),
        )
        .await;
    assert_eq!((s, code(&r)), (409, "PROVIDER_ACCOUNT_IN_USE"), "{r}");
    // Le compte de versement et le compte de frais se changent encore.
    let (s, r) = ctx
        .put(
            &chemin,
            json!({ "bankName": "Stripe CHF", "journalAccountId": ctx.chf,
                "stripe": { "account": STRIPE, "feeAccountId": ctx.frais_chf, "payoutBankAccountId": banque2 },
                "version": version(&b) }),
        )
        .await;
    assert_eq!(s, 200, "le compte de versement se change : {r}");
    assert_eq!(r["stripe"]["payoutBankAccountId"], banque2);
    let trace: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.updated' AND entity_id = ? \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(trace["after"]["payout_bank_account_id"], banque2);
    assert_eq!(trace["before"]["payout_bank_account_id"], banque);
}

// ============================================================
// AC-3 — les autres chemins refusent un compte Stripe
// ============================================================

/// AC-3 — un compte Stripe reçoit ni relevé (CAMT ou saisi, aperçu et
/// création), ni règlement de facture client, ni paiement de facture
/// fournisseur, ni lot pain.001, ni solde d'ouverture en devise : les refus sont
/// jugés sur `provider IS NOT NULL`, les mêmes que pour PayPal.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_autres_chemins_refusent_un_compte_stripe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let chf = ctx.solde_avec(ctx.chf, None, None).await["id"]
        .as_i64()
        .unwrap();
    let jour = common::jour::aujourd_hui_test().to_string();

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
            json!({ "settlementType": "bank_transfer", "bankAccountId": chf, "amount": "108.10",
            "settledOn": jour, "version": b["version"] }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        ecritures,
        "règlement : rien d'écrit"
    );

    // Paiement d'une facture fournisseur, puis lot pain.001.
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
        .post(&format!("/api/v1/supplier-invoices/{ff}/pay"),
            json!({ "settlementType": "bank_transfer", "bankAccountId": chf, "paymentDate": "2026-06-20" }))
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    let (s, b) = ctx
        .post(
            "/api/v1/payment-batches",
            json!({ "bankAccountId": chf,
            "requestedExecutionDate": "2026-06-25", "supplierInvoiceIds": [ff] }),
        )
        .await;
    assert_eq!((s, code(&b)), (409, "BANK_ACCOUNT_IS_PROVIDER"), "{b}");
    assert_eq!(ctx.count("SELECT COUNT(*) FROM payment_batches").await, 0);

    // Solde d'ouverture d'un compte Stripe EUR : refusé comme pour PayPal.
    sqlx::query("UPDATE accounts SET role = 'RetainedEarnings' WHERE id = ?")
        .bind(ctx.seeded.accounts["2000"])
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE")
        .execute(&pool)
        .await
        .unwrap();
    let comptes = ctx.count("SELECT COUNT(*) FROM bank_accounts").await;
    let ecritures = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx
        .post("/api/v1/bank-accounts", json!({ "bankName": "Stripe", "journalAccountId": ctx.eur,
            "stripe": { "account": "Stripe EUR" }, "openingBalance": { "amount": "10.00", "side": "debit" } }))
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

// ============================================================
// AC-7 — le cœur commun : l'import PayPal et l'import Stripe se côtoient
// ============================================================

/// AC-7 — un même fichier se range sous chaque prestataire sans heurter l'autre
/// : un compte PayPal et un compte Stripe de même devise vivent ensemble, et
/// l'import de l'un n'écrit rien sur l'autre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn paypal_et_stripe_cohabitent_sur_le_meme_coeur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    ctx.trois_soldes().await;
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, provider, provider_account) \
         VALUES (?, 'PayPal EUR', FALSE, ?, 'paypal', 'PayPal www')",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.eur2)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = ctx.importer(FIXTURE).await;
    assert_eq!(s, 201, "{b}");
    // Aucune transaction sur le compte PayPal.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM bank_transactions t JOIN bank_accounts a ON a.id = t.bank_account_id \
             WHERE a.provider = 'paypal'"
        )
        .await,
        0
    );
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'paypal_import.created'")
            .await,
        0
    );
}
