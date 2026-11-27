//! Story 39-11c (Epic 39, phase 1) — la revue et l'acceptation Wise, par les
//! routes `GET /api/v1/wise/review` et `POST /api/v1/wise/accept`, sur le
//! moteur commun des prestataires (`routes::provider_accept`).
//!
//! Montage : les relevés fictifs de 39-11a (CHF, EUR, USD) importés par la
//! route de 39-11b dans trois soldes d'un compte « Wise Pro » ; cours mensuels
//! de 2020 posés en base ; l'autre côté d'un virement ou d'un versement Stripe
//! posé en base (écriture par `POST /journal-entries`, transaction d'un autre
//! compte bancaire rapprochée d'elle). Les paiements par carte convertis
//! (amendement A-1) sur des relevés écrits ici, montants des fixtures de
//! conversion de carte. Fixtures fictives seulement.
//!
//! ⚠️ Harnais recopié (`test_config`, `spawn_app`, `forge_jwt`), comme dans
//! chacun des fichiers E2E de ce répertoire (choix C70).

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use common::capture_rejeu::CaptureRejeu;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{
    SeededCompany, attendre_une_requete_en_cours, disable_rounding_to_5_centimes,
    mark_onboarding_complete, seed_accounting_company,
};
use reqwest::multipart;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::{MySql, MySqlPool, Transaction};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const CHF: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-chf-fictif.csv");
const EUR: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-eur-fictif.csv");
const USD: &[u8] = include_bytes!("../../kesh-import/tests/fixtures/wise-releve-usd-fictif.csv");
const WISE: &str = "Wise Pro";
const HEADER: &str = "TransferWise ID,Date,Date Time,Amount,Currency,Description,Payment Reference,Running Balance,Exchange From,Exchange To,Exchange Rate,Payer Name,Payee Name,Payee Account Number,Merchant,Card Last Four Digits,Card Holder Full Name,Attachment,Note,Total fees,Exchange To Amount,Transaction Type,Transaction Details Type";

#[derive(Clone)]
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

/// Société multi-devises (comptes de gain, de perte et de conversion), trois
/// soldes Wise (CHF, EUR, USD) avec un compte de frais en CHF, compte bancaire
/// principal et un compte bancaire « Stripe » ordinaire, cours mensuels de
/// janvier à mai 2020.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    chf: i64,
    eur: i64,
    usd: i64,
    frais: i64,
    charge: i64,
    banque: i64,
    stripe: i64,
    transit: i64,
    perte: i64,
    ba_banque: i64,
    ba_stripe: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, currency) \
         VALUES (?, ?, ?, ?, 1, 1, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn compte_bancaire(pool: &MySqlPool, c: i64, nom: &str, iban: &str, ledger: i64) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(c)
    .bind(nom)
    .bind(iban)
    .bind(nom == "Banque")
    .bind(ledger)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn monter_sans_import(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let c = seeded.company_id;
    let gain = compte(pool, c, "6950", "Revenue", None).await;
    let perte = compte(pool, c, "6960", "Expense", None).await;
    let transit = compte(pool, c, "1090", "Asset", None).await;
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
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(seeded.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    for mois in [
        "2020-01-01",
        "2020-02-01",
        "2020-03-01",
        "2020-04-01",
        "2020-05-01",
    ] {
        for (devise, valeur) in [("EUR", "0.95"), ("USD", "0.90")] {
            sqlx::query(
                "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
                 VALUES (?, ?, 'month', ?, ?, 'afc')",
            )
            .bind(c)
            .bind(devise)
            .bind(mois)
            .bind(valeur)
            .execute(pool)
            .await
            .unwrap();
        }
    }
    let banque = compte(pool, c, "1020", "Asset", None).await;
    let stripe = compte(pool, c, "1105", "Asset", None).await;
    let ba_banque = compte_bancaire(pool, c, "Banque", "CH9300762011623852957", banque).await;
    let ba_stripe = compte_bancaire(pool, c, "Stripe", "CH5604835012345678009", stripe).await;
    let ctx = Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone()).await,
        jwt: forge_jwt(seeded.admin_user_id, "Admin", c),
        chf: compte(pool, c, "1030", "Asset", None).await,
        eur: compte(pool, c, "1031", "Asset", Some("EUR")).await,
        usd: compte(pool, c, "1032", "Asset", Some("USD")).await,
        frais: compte(pool, c, "6940", "Expense", None).await,
        charge: compte(pool, c, "6500", "Expense", None).await,
        banque,
        stripe,
        transit,
        perte,
        ba_banque,
        ba_stripe,
        seeded,
    };
    for ledger in [ctx.chf, ctx.eur, ctx.usd] {
        ctx.solde(ledger, Some(ctx.frais)).await;
    }
    ctx
}

/// Le montage, plus les trois relevés de 39-11a importés.
async fn monter(pool: &MySqlPool) -> Ctx {
    let ctx = monter_sans_import(pool).await;
    let (s, b) = ctx
        .importer(&[("chf.csv", CHF), ("eur.csv", EUR), ("usd.csv", USD)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    ctx
}

impl Ctx {
    async fn req(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
        jeton: &str,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, format!("{}{chemin}", self.app.base_url))
            .bearer_auth(jeton);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req(reqwest::Method::POST, chemin, Some(corps), &self.jwt)
            .await
    }

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    async fn solde(&self, ledger: i64, fee: Option<i64>) -> i64 {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "Wise",
                    "journalAccountId": ledger,
                    "wise": { "account": WISE, "feeAccountId": fee },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde Wise : {b}");
        b["id"].as_i64().unwrap()
    }

    async fn importer(&self, fichiers: &[(&str, &[u8])]) -> (u16, Value) {
        let mut form = multipart::Form::new().text("account", WISE.to_string());
        for (nom, octets) in fichiers {
            form = form.part(
                "file",
                multipart::Part::bytes(octets.to_vec())
                    .file_name(nom.to_string())
                    .mime_str("text/csv")
                    .unwrap(),
            );
        }
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/wise-imports", self.app.base_url))
            .bearer_auth(&self.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn regle(&self, libelle: &str, genre: &str, valeur: &str, filtre: Option<&str>) -> i64 {
        self.regle_vers(libelle, genre, valeur, filtre, self.charge)
            .await
    }

    /// Une règle de contrepartie `contrepartie` (la charge pour [`Ctx::regle`]).
    async fn regle_vers(
        &self,
        libelle: &str,
        genre: &str,
        valeur: &str,
        filtre: Option<&str>,
        contrepartie: i64,
    ) -> i64 {
        sqlx::query(
            "INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, \
             counterparty_account_id, provider_type) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(self.seeded.company_id)
        .bind(libelle)
        .bind(genre)
        .bind(valeur)
        .bind(contrepartie)
        .bind(filtre)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn revue(&self) -> Value {
        let (s, b) = self
            .req(
                reqwest::Method::GET,
                &format!(
                    "/api/v1/wise/review?account=Wise%20Pro&fiscalYearId={}",
                    self.seeded.fiscal_year_id
                ),
                None,
                &self.jwt,
            )
            .await;
        assert_eq!(s, 200, "revue : {b}");
        b
    }

    async fn accepter(&self, propositions: Vec<Value>) -> (u16, Value) {
        self.post(
            "/api/v1/wise/accept",
            json!({ "account": WISE, "proposals": propositions }),
        )
        .await
    }

    /// La transaction d'identifiant Wise `wise` sur le compte bancaire lié à
    /// `ledger`.
    async fn tx(&self, wise: &str, ledger: i64) -> i64 {
        sqlx::query_scalar(
            "SELECT bt.id FROM bank_transactions bt \
             JOIN bank_accounts ba ON ba.id = bt.bank_account_id \
             WHERE bt.transaction_id = ? AND ba.journal_account_id = ?",
        )
        .bind(wise)
        .bind(ledger)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn etat(&self, id: i64) -> (String, Option<i64>) {
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Mouvement CHF (débit − crédit) et en devise d'un compte du plan.
    async fn mouvement(&self, compte: i64) -> (Decimal, Decimal) {
        sqlx::query_as(
            "SELECT COALESCE(SUM(debit) - SUM(credit), 0), \
                    COALESCE(SUM(CASE WHEN debit > 0 THEN COALESCE(amount_currency, 0) \
                                      ELSE -COALESCE(amount_currency, 0) END), 0) \
             FROM journal_entry_lines WHERE account_id = ?",
        )
        .bind(compte)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    async fn lignes(&self, ecriture: i64) -> Vec<Ligne> {
        sqlx::query_as(
            "SELECT account_id, debit, credit, amount_currency, exchange_rate \
             FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
        )
        .bind(ecriture)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    /// Une écriture du journal Banque à deux lignes (débit `debit`, crédit
    /// `credit`, `chf`), par la route.
    async fn ecriture(&self, date: &str, libelle: &str, debit: i64, credit: i64, chf: &str) -> i64 {
        let (s, b) = self
            .post(
                "/api/v1/journal-entries",
                json!({
                    "entryDate": date,
                    "journal": "Banque",
                    "description": libelle,
                    "lines": [
                        { "accountId": debit, "debit": chf, "credit": "0" },
                        { "accountId": credit, "debit": "0", "credit": chf },
                    ],
                }),
            )
            .await;
        assert_eq!(s, 201, "écriture : {b}");
        b["id"].as_i64().unwrap()
    }

    /// L'autre côté déjà passé : une transaction du compte bancaire
    /// `compte_bancaire`, de `montant`, rapprochée de l'écriture `ecriture`
    /// (posée en base, import compris).
    async fn autre_cote(&self, compte_bancaire: i64, date: &str, montant: &str, ecriture: i64) {
        self.transaction_bancaire(compte_bancaire, date, montant, Some(ecriture))
            .await;
    }

    /// Une transaction CHF du compte bancaire `compte_bancaire` (import
    /// compris), posée en base : rapprochée de `ecriture`, ou en attente.
    async fn transaction_bancaire(
        &self,
        compte_bancaire: i64,
        date: &str,
        montant: &str,
        ecriture: Option<i64>,
    ) -> i64 {
        let c = self.seeded.company_id;
        let hash = format!(
            "{:064}",
            self.count("SELECT COUNT(*) + 1 FROM bank_imports").await
        );
        let import = sqlx::query(
            "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
             source_format, period_from, period_to, transaction_count, imported_by_user_id) \
             VALUES (?, ?, 'autre.xml', ?, 'CAMT053_V04', ?, ?, 1, ?)",
        )
        .bind(c)
        .bind(compte_bancaire)
        .bind(hash)
        .bind(date)
        .bind(date)
        .bind(self.seeded.admin_user_id)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id();
        sqlx::query(
            "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
             amount, currency, details, status, matched_entry_id) \
             VALUES (?, ?, ?, ?, ?, 'CHF', 'Autre côté', ?, ?)",
        )
        .bind(c)
        .bind(import)
        .bind(compte_bancaire)
        .bind(date)
        .bind(montant)
        .bind(if ecriture.is_some() {
            "reconciled"
        } else {
            "pending"
        })
        .bind(ecriture)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// `POST /reconciliation/transactions/{id}/link-entry`.
    async fn rattacher(&self, id: i64, ecriture: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/reconciliation/transactions/{id}/link-entry"),
            json!({ "journalEntryId": ecriture }),
        )
        .await
    }

    /// `POST /reconciliation/transactions/{id}/cancel`, qui doit passer.
    async fn annuler(&self, id: i64) {
        let (s, b) = self
            .post(
                &format!("/api/v1/reconciliation/transactions/{id}/cancel"),
                json!({}),
            )
            .await;
        assert_eq!(s, 200, "annulation : {b}");
    }

    /// Les identifiants des écritures, dans l'ordre.
    async fn ecritures(&self) -> Vec<i64> {
        sqlx::query_scalar("SELECT id FROM journal_entries ORDER BY id")
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }
}

/// Une ligne lue : compte, débit, crédit, montant en devise, cours.
type Ligne = (i64, Decimal, Decimal, Option<Decimal>, Option<Decimal>);

/// La proposition de la revue qui porte la transaction `id`.
fn proposition(revue: &Value, id: i64) -> Value {
    revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["transaction"]["id"] == id)
        .cloned()
        .unwrap_or_else(|| panic!("aucune proposition pour {id} : {revue}"))
}

/// `(motif ou genre, identifiant Wise)` de chaque élément d'une liste, triés.
fn codes(liste: &Value, champ: &str) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = liste
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f[champ].as_str().unwrap().to_string(),
                f["transaction"]["transactionId"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            )
        })
        .collect();
    v.sort();
    v
}

fn paire(a: &str, b: &str) -> (String, String) {
    (a.to_string(), b.to_string())
}

// ============================================================
// AC-1 à AC-7 — la revue des relevés de 39-11a, puis tout accepter
// ============================================================

/// La revue classe chaque ligne ; l'acceptation du tout écrit au bon CHF,
/// rapproche ; la somme des écritures de chaque solde entièrement rapproché
/// égale la variation de son solde Wise ; la seconde revue ne propose plus
/// rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn revue_puis_acceptation_du_tout(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let boutique = ctx
        .regle(
            "Boutique",
            "counterparty_contains",
            "Boutique Exemple",
            Some("CARD"),
        )
        .await;
    ctx.regle(
        "Service",
        "counterparty_contains",
        "Service En Ligne",
        Some("CARD"),
    )
    .await;
    ctx.regle("Atelier", "iban_exact", "CH5604835012345678009", None)
        .await;
    // Versement Stripe du 10 janvier, déjà passé (C-40-K5) : écriture débit
    // Wise CHF / crédit Stripe, rapprochée de la transaction du compte Stripe.
    let versement = ctx
        .ecriture(
            "2020-01-10",
            "Versement Stripe",
            ctx.chf,
            ctx.stripe,
            "2000.00",
        )
        .await;
    ctx.autre_cote(ctx.ba_stripe, "2020-01-10", "-2000.00", versement)
        .await;
    // Virement du 12 février vers la banque, passé depuis la banque le 13 ;
    // une seconde candidate à 13 jours ne compte pas.
    let virement = ctx
        .ecriture(
            "2020-02-13",
            "Virement depuis Wise",
            ctx.banque,
            ctx.chf,
            "300.00",
        )
        .await;
    ctx.autre_cote(ctx.ba_banque, "2020-02-13", "300.00", virement)
        .await;
    let lointain = ctx
        .ecriture(
            "2020-02-25",
            "Virement lointain",
            ctx.banque,
            ctx.chf,
            "300.00",
        )
        .await;
    ctx.autre_cote(ctx.ba_banque, "2020-02-25", "300.00", lointain)
        .await;
    // Virement du 18 février : deux candidates à moins de 5 jours.
    for date in ["2020-02-17", "2020-02-19"] {
        let e = ctx
            .ecriture(date, "Virement ambigu", ctx.banque, ctx.chf, "80.00")
            .await;
        ctx.autre_cote(ctx.ba_banque, date, "80.00", e).await;
    }

    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let revue = ctx.revue().await;
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant,
        "la revue n'écrit rien"
    );
    assert_eq!(
        codes(&revue["proposals"], "type"),
        vec![
            paire("conversion", "BALANCE-90000003"),
            paire("conversion", "BALANCE-90000010"),
            paire("conversion", "CONVERSION_ORDER-90000011"),
            paire("fee", "FEE-BALANCE-90000003"),
            paire("fee", "FEE-BALANCE-90000010"),
            paire("fee", "FEE-CONVERSION_ORDER-90000011"),
            paire("fee", "FEE-TRANSFER-90000004"),
            paire("link_entry", "BALANCE-90000001"),
            paire("link_entry", "TRANSFER-90000004"),
            paire("rule", "CARD-90000007"),
            paire("rule", "CARD-90000012"),
            paire("rule", "TRANSFER-90000005"),
        ],
        "{revue}"
    );
    assert_eq!(
        codes(&revue["toReview"], "code"),
        vec![
            paire("ambiguous_link", "TRANSFER-90000006"),
            paire("no_rule", "CARD-90000002"),
            paire("stripe_payout_unmatched", "BALANCE-90000008"),
        ],
        "{revue}"
    );
    assert_eq!(revue["gaps"], json!([]), "{revue}");
    assert_eq!(
        revue["counts"],
        json!({ "proposals": 12, "toReview": 3, "gaps": 0 })
    );
    let stripe_tx = ctx.tx("BALANCE-90000001", ctx.chf).await;
    let p = proposition(&revue, stripe_tx);
    assert_eq!(p["journalEntryId"], versement);
    assert_eq!(p["entry"]["journalEntryId"], versement);
    let ambigu = revue["toReview"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["code"] == "ambiguous_link")
        .unwrap();
    assert_eq!(ambigu["entries"].as_array().unwrap().len(), 2);
    let carte = ctx.tx("CARD-90000007", ctx.chf).await;
    assert_eq!(proposition(&revue, carte)["ruleId"], boutique);

    let toutes: Vec<Value> = revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let mut p = p.clone();
            let o = p.as_object_mut().unwrap();
            for k in ["transaction", "otherTransaction", "entry"] {
                o.remove(k);
            }
            p
        })
        .collect();
    let (s, b) = ctx.accepter(toutes).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"], json!([]), "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 12, "{b}");

    // Conversion CHF → EUR : une écriture, le CHF de Wise, deux transactions.
    let conv_chf = ctx.tx("BALANCE-90000003", ctx.chf).await;
    let conv_eur = ctx.tx("BALANCE-90000003", ctx.eur).await;
    let (st, e1) = ctx.etat(conv_chf).await;
    assert_eq!(st, "reconciled");
    assert_eq!(ctx.etat(conv_eur).await, ("reconciled".into(), e1));
    let lignes = ctx.lignes(e1.unwrap()).await;
    assert_eq!(lignes.len(), 2);
    assert!(lignes.contains(&(ctx.chf, dec!(0), dec!(500.00), None, None)));
    assert!(
        lignes
            .iter()
            .any(|l| l.0 == ctx.eur && l.1 == dec!(500.00) && l.3 == Some(dec!(462.30)))
    );
    // EUR → USD : deux écritures, même CHF, compte de conversion soldé.
    let (_, ea) = ctx.etat(ctx.tx("BALANCE-90000010", ctx.eur).await).await;
    let (_, eb) = ctx.etat(ctx.tx("BALANCE-90000010", ctx.usd).await).await;
    assert_ne!(ea, eb);
    assert_eq!(ctx.mouvement(ctx.transit).await.0, dec!(0));
    // Les soldes entièrement rapprochés : EUR (160.00) et USD (317.01).
    assert_eq!(ctx.mouvement(ctx.eur).await.1, dec!(160.00));
    assert_eq!(ctx.mouvement(ctx.usd).await.1, dec!(317.01));
    // CHF : la clôture (2433.55) moins les lignes restées en attente
    // (−45.90, −80.00, +1500.00), soit 1059.45 ; s'y ajoutent les trois
    // écritures de virement posées par le montage et restées sans
    // transaction Wise (−300.00, −80.00, −80.00).
    assert_eq!(ctx.mouvement(ctx.chf).await.0 + dec!(460.00), dec!(1059.45));
    // Le virement et le versement : rattachés, aucune écriture neuve.
    assert_eq!(
        ctx.etat(ctx.tx("TRANSFER-90000004", ctx.chf).await).await,
        ("reconciled".into(), Some(virement))
    );
    assert_eq!(
        ctx.etat(stripe_tx).await,
        ("reconciled".into(), Some(versement))
    );
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted' \
             AND JSON_UNQUOTE(JSON_EXTRACT(details_json, '$.provider')) = 'wise'"
        )
        .await,
        12
    );

    let revue = ctx.revue().await;
    assert_eq!(revue["proposals"], json!([]), "{revue}");
    assert_eq!(revue["counts"]["toReview"], 3);
}

/// Conversion à une écriture : annuler un côté ne retire que son lien ;
/// l'écriture reste, l'autre côté reste rapproché ; le côté annulé se
/// rattache à nouveau à la même écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_annulation_d_un_cote_puis_rattachement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let chf = ctx.tx("BALANCE-90000003", ctx.chf).await;
    let eur = ctx.tx("BALANCE-90000003", ctx.eur).await;
    let revue = ctx.revue().await;
    let mut conv = proposition(&revue, chf);
    conv.as_object_mut().unwrap().remove("transaction");
    conv.as_object_mut().unwrap().remove("otherTransaction");
    let (s, b) = ctx.accepter(vec![conv]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();

    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{chf}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(chf).await, ("pending".into(), None));
    assert_eq!(ctx.etat(eur).await, ("reconciled".into(), Some(ecriture)));
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {ecriture}"
        ))
        .await,
        0,
        "aucune contre-passation"
    );
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{chf}/link-entry"),
            json!({ "journalEntryId": ecriture }),
        )
        .await;
    assert_eq!(s, 200, "le côté annulé se rattache à nouveau : {b}");
    assert_eq!(ctx.etat(chf).await, ("reconciled".into(), Some(ecriture)));
}

// ============================================================
// Amendement A-1 à A-3 — paiement par carte converti, écriture groupée
// ============================================================

/// La partielle « USD, CHF » : CHF −86.60 (solde vidé), USD −13.85 converti
/// pour 11.35 CHF, frais USD −0.04 (CSV « avec frais indiqués séparément »).
fn releves_partielle() -> (Vec<u8>, Vec<u8>) {
    let chf = format!(
        "{HEADER}\nCARD-90000020,10-03-2020,10-03-2020 12:00:00.000,-86.60,CHF,Carte Boutique Exemple,,0.00,,,,,,,Boutique Exemple,,,,,,,DEBIT,CARD\n"
    );
    (chf.into_bytes(), releve_usd_carte("10-03-2020"))
}

/// La part USD du paiement `CARD-90000020` (USD −13.85 converti pour 11.35
/// CHF, frais USD −0.04), datée `date` (`JJ-MM-AAAA`) : seule, une conversion
/// totale.
fn releve_usd_carte(date: &str) -> Vec<u8> {
    format!(
        "{HEADER}\nCARD-90000020,{date},{date} 12:00:00.000,-13.85,USD,Carte Boutique Exemple,,0.00,USD,CHF,0.81950,,,,Boutique Exemple,,,,,0.04,11.35,DEBIT,CARD\n\
         FEE-CARD-90000020,{date},{date} 11:59:59.999,-0.04,USD,Frais de conversion,,13.85,,,,,,,,,,,,,,DEBIT,CARD\n"
    )
    .into_bytes()
}

/// Les deux exports USD d'un paiement coupé par la borne de leurs périodes :
/// la carte `CARD-90000020` passée le 10 mars à minuit pile, son frais
/// `FEE-CARD-90000020` une milliseconde avant, le 9 mars. L'export qui finit
/// le 9 mars ne porte que le frais ; celui qui commence le 10 porte la carte
/// sans son frais, et — export « avec frais séparés » — un virement envoyé
/// dont le frais est sa propre ligne `FEE-`. C'est la seule coupure qu'un
/// export de Wise produise (ses périodes sont des jours), et la seule que la
/// garde de la Story 39-11e reçoive (`check_fees_separated`, `kesh-import`) :
/// une ligne à frais sans sa `FEE-`, dans un fichier qui en porte. Rend
/// `(frais, carte)`.
fn releves_usd_coupes() -> (Vec<u8>, Vec<u8>) {
    let frais = format!(
        "{HEADER}\nFEE-CARD-90000020,09-03-2020,09-03-2020 23:59:59.999,-0.04,USD,Frais de conversion,,13.85,,,,,,,,,,,,,,DEBIT,CARD\n"
    );
    let carte = format!(
        "{HEADER}\nTRANSFER-70000032,10-03-2020,10-03-2020 11:00:00.000,-5.00,USD,Virement envoyé,,14.50,,,,,Fournisseur Fictif,,,,,,,0.50,,DEBIT,TRANSFER\n\
         FEE-TRANSFER-70000032,10-03-2020,10-03-2020 10:59:59.999,-0.50,USD,Frais de virement,,19.50,,,,,,,,,,,,,,DEBIT,TRANSFER\n\
         TRANSFER-70000031,10-03-2020,10-03-2020 09:00:00.000,20.00,USD,Virement reçu,,20.00,,,,Client Fictif,,,,,,,,,,CREDIT,TRANSFER\n\
         CARD-90000020,10-03-2020,10-03-2020 00:00:00.000,-13.85,USD,Carte Boutique Exemple,,0.00,USD,CHF,0.81950,,,,Boutique Exemple,,,,,0.04,11.35,DEBIT,CARD\n"
    );
    (frais.into_bytes(), carte.into_bytes())
}

/// Un relevé d'une devise fait de virements reçus `(identifiant, date,
/// montant, solde après)` — des lignes sans rapport avec le paiement par
/// carte, qui fixent la période et les soldes du relevé.
fn releve_virements(devise: &str, lignes: &[(&str, &str, &str, &str)]) -> Vec<u8> {
    let mut csv = format!("{HEADER}\n");
    for (id, date, montant, solde) in lignes {
        csv.push_str(&format!(
            "{id},{date},{date} 09:00:00.000,{montant},{devise},Virement reçu,,{solde},,,,Client Fictif,,,,,,,,,,CREDIT,TRANSFER\n"
        ));
    }
    csv.into_bytes()
}

/// La règle « Boutique » des paiements par carte.
async fn regle_boutique(ctx: &Ctx) -> i64 {
    ctx.regle(
        "Boutique",
        "counterparty_contains",
        "Boutique Exemple",
        Some("CARD"),
    )
    .await
}

/// Les éléments de `liste` dont le motif commence par `card_group`.
fn de_groupe(liste: &Value) -> Vec<Value> {
    liste
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| {
            f["code"].as_str().unwrap_or("").starts_with("card_group") || f["type"] == "card_group"
        })
        .cloned()
        .collect()
}

async fn monter_partielle(pool: &MySqlPool) -> (Ctx, i64, [i64; 3]) {
    let ctx = monter_sans_import(pool).await;
    let (chf, usd) = releves_partielle();
    let (s, b) = ctx.importer(&[("chf.csv", &chf), ("usd.csv", &usd)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    // L'ordre de la revue : la ligne principale (en CHF), puis par identifiant
    // (les frais, une milliseconde avant leur ligne, sont importés d'abord).
    let mut autres = [
        ctx.tx("CARD-90000020", ctx.usd).await,
        ctx.tx("FEE-CARD-90000020", ctx.usd).await,
    ];
    autres.sort_unstable();
    let ids = [ctx.tx("CARD-90000020", ctx.chf).await, autres[0], autres[1]];
    (ctx, regle, ids)
}

fn groupe(ids: &[i64], regle: i64, contrepartie: i64) -> Value {
    json!({
        "type": "card_group",
        "bankTransactionIds": ids,
        "ruleId": regle,
        "counterpartyAccountId": contrepartie,
    })
}

/// Une écriture : la charge au montant du marchand (97.95), chaque solde
/// crédité de sa part (CHF 86.60 ; USD 13.89 au cours du mois, 12.50 CHF), le
/// frais (0.04 USD, 0.04 CHF) au compte de frais du solde, la différence (le
/// change seul, 1.11) en perte de change ; les trois transactions rapprochées.
/// Annuler une ligne contre-passe l'écriture et remet les trois en attente ;
/// la proposition reparaît (rien de perdu).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_en_une_ecriture_puis_annulation(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let revue = ctx.revue().await;
    assert_eq!(revue["counts"]["proposals"], 1, "{revue}");
    let p = &revue["proposals"][0];
    assert_eq!(p["type"], "card_group");
    assert_eq!(p["bankTransactionIds"], json!(ids));
    assert_eq!(p["ruleId"], regle);
    assert_eq!(p["counterpartyAccountId"], ctx.charge);
    assert_eq!(revue["toReview"], json!([]), "{revue}");

    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(b["accepted"][0]["type"], "card_group");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("reconciled".into(), Some(ecriture)));
    }
    assert_eq!(
        ctx.lignes(ecriture).await,
        vec![
            (ctx.charge, dec!(97.95), dec!(0), None, None),
            (ctx.chf, dec!(0), dec!(86.60), None, None),
            (
                ctx.usd,
                dec!(0),
                dec!(12.50),
                Some(dec!(13.89)),
                Some(dec!(0.90))
            ),
            (ctx.frais, dec!(0.04), dec!(0), None, None),
            (ctx.perte, dec!(1.11), dec!(0), None, None),
        ]
    );
    assert_eq!(
        ctx.count(&format!(
            "SELECT applied_count FROM reconciliation_rules WHERE id = {regle}"
        ))
        .await,
        1
    );

    // Annuler la ligne de frais : contre-passation, les trois en attente.
    let frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    // La vue ne l'annonce pas bloquée par les autres lignes du paiement.
    let (s, vue) = ctx
        .req(
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{frais}"),
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["cancelBlockedBy"], Value::Null, "{vue}");
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{frais}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("pending".into(), None), "{id}");
    }
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {ecriture}"
        ))
        .await,
        1
    );
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation.cancelled' \
             AND details_json LIKE '%groupedWith%'"
        )
        .await,
        2,
        "une trace par ligne remise en attente avec elle"
    );
    let revue = ctx.revue().await;
    assert_eq!(revue["proposals"][0]["type"], "card_group", "{revue}");
}

/// Story 53-6a (#1674, AC-2, AC-3, AC-4, AC-7, R6) — **le paiement Wise groupé
/// d'un exercice antérieur encore ouvert** : l'écriture du groupe est du
/// 10 mars 2020 ; l'exercice du montage (2020-2030) est coupé au 31 décembre
/// 2020 et un exercice suivant couvre le jour (état forgé, dit tel : l'écriture
/// reste dans le premier).
///
/// - **clos** : le geste est refusé `409 FISCAL_YEAR_CLOSED`, rien n'est écrit,
///   les trois transactions restent rapprochées ;
/// - **rouvert** : une seule inverse, datée de l'écriture (10 mars 2020), dans
///   l'exercice de l'écriture ; rien dans l'exercice du jour ; les trois
///   transactions reviennent en attente.
///
/// La réouverture est posée par le statut (SQL) : la clôture réelle exigerait la
/// réévaluation des soldes en devises du montage, qui n'est pas le sujet. Tue M1
/// pour le groupe Wise (le bras `Entry` daté du jour au lieu de l'écriture).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    let premier = ctx.seeded.fiscal_year_id;
    sqlx::query("UPDATE fiscal_years SET end_date = '2020-12-31' WHERE id = ?")
        .bind(premier)
        .execute(&pool)
        .await
        .unwrap();
    let suivant: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice suivant', '2021-01-01', '2030-12-31', 'Open') RETURNING id",
    )
    .bind(ctx.seeded.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    let chemin = format!("/api/v1/reconciliation/transactions/{frais}/cancel");

    // AC-4 — l'exercice de l'écriture est clos : refusé, rien n'est écrit.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(premier)
        .execute(&pool)
        .await
        .unwrap();
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["code"], "FISCAL_YEAR_CLOSED", "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant,
        "rien d'écrit"
    );
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("reconciled".into(), Some(ecriture)));
    }

    // AC-3 — l'exercice est rouvert : l'inverse y est datée de l'écriture.
    sqlx::query("UPDATE fiscal_years SET status = 'Open' WHERE id = ?")
        .bind(premier)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_eq!(s, 200, "{b}");
    let inverses: Vec<(chrono::NaiveDate, i64)> = sqlx::query_as(
        "SELECT entry_date, fiscal_year_id FROM journal_entries WHERE reverses_entry_id = ?",
    )
    .bind(ecriture)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        inverses,
        vec![(
            chrono::NaiveDate::from_ymd_opt(2020, 3, 10).unwrap(),
            premier
        )],
        "une seule inverse, datée de l'écriture, dans son exercice"
    );
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE fiscal_year_id = {suivant}"
        ))
        .await,
        0,
        "rien dans l'exercice du jour"
    );
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("pending".into(), None), "{id}");
    }
}

/// Amendement A-3 (décision (c)) : la proposition qu'on n'accepte pas se
/// traite à la main par l'API, en plusieurs appels, sans être perdue — une
/// écriture par la route des écritures, puis le rattachement ; le reste suit
/// la revue (règle, frais).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_traitee_a_la_main_par_l_api(pool: MySqlPool) {
    let (ctx, _, ids) = monter_partielle(&pool).await;
    let (chf, usd, frais) = (
        ids[0],
        ctx.tx("CARD-90000020", ctx.usd).await,
        ctx.tx("FEE-CARD-90000020", ctx.usd).await,
    );
    // La ligne USD convertie : écriture en devise à la main, puis rattachement.
    let (s, b) = ctx
        .post(
            "/api/v1/journal-entries",
            json!({
                "entryDate": "2020-03-10",
                "journal": "Banque",
                "description": "Boutique, part USD, à la main",
                "lines": [
                    { "accountId": ctx.charge, "debit": "11.35", "credit": "0" },
                    { "accountId": ctx.usd, "debit": "0", "credit": "11.35",
                      "foreign": { "amount": "13.85", "rate": "0.81949", "rateSource": "manual" } },
                ],
            }),
        )
        .await;
    assert_eq!(s, 201, "écriture à la main : {b}");
    let manuelle = b["id"].as_i64().unwrap();
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{usd}/link-entry"),
            json!({ "journalEntryId": manuelle }),
        )
        .await;
    assert_eq!(s, 200, "rattachement : {b}");
    // Le paiement n'est plus converti en attente : ses lignes reviennent au
    // classement ligne à ligne, chacune proposée.
    let revue = ctx.revue().await;
    assert_eq!(
        codes(&revue["proposals"], "type"),
        vec![
            paire("fee", "FEE-CARD-90000020"),
            paire("rule", "CARD-90000020")
        ],
        "{revue}"
    );
    let propositions: Vec<Value> = revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            let mut p = p.clone();
            p.as_object_mut().unwrap().remove("transaction");
            p
        })
        .collect();
    let (s, b) = ctx.accepter(propositions).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    for id in [chf, usd, frais] {
        assert_eq!(ctx.etat(id).await.0, "reconciled");
    }
}

/// Rejugement : une ligne rapprochée entre la revue et l'acceptation rend le
/// groupe périmé ; une contrepartie en devise est refusée ; sans compte de
/// perte de change, `FX_ACCOUNT_INVALID`. Rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_perimee_ou_refusee(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    // Groupe incomplet — les deux lignes de carte sans la ligne de frais, que
    // le jugement seul accepterait : périmé (l'ensemble relu diffère).
    let carte_usd = ctx.tx("CARD-90000020", ctx.usd).await;
    let (s, b) = ctx
        .accepter(vec![groupe(&[ids[0], carte_usd], regle, ctx.charge)])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(b["failed"][0]["details"]["reason"], "card_group_changed");
    assert_eq!(b["failed"][0]["bankTransactionId"], ids[0]);
    // Autre contrepartie que celle de la règle : périmée.
    let (_, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.frais)]).await;
    assert_eq!(
        b["failed"][0]["details"]["reason"],
        "rule_no_longer_matches"
    );
    // Sans compte de perte de change.
    sqlx::query("UPDATE companies SET fx_loss_account_id = NULL WHERE id = ?")
        .bind(ctx.seeded.company_id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let (_, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(b["failed"][0]["errorCode"], "FX_ACCOUNT_INVALID", "{b}");
    assert_eq!(b["failed"][0]["details"]["field"], "fxLossAccountId");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant,
        "rien d'écrit"
    );
    for id in ids {
        assert_eq!(ctx.etat(id).await.0, "pending");
    }
    // Sans règle : à examiner, motif propre.
    sqlx::query("UPDATE reconciliation_rules SET active = FALSE WHERE id = ?")
        .bind(regle)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let revue = ctx.revue().await;
    assert_eq!(
        codes(&revue["toReview"], "code"),
        vec![paire("card_group_no_rule", "CARD-90000020")],
        "{revue}"
    );
    assert_eq!(revue["toReview"][0]["details"]["merchantAmount"], "97.95");
}

/// Revue M1 de la 39-11c : une partielle dont **un seul** solde est importé
/// (ici l'USD, la ligne en CHF manque) ressemble à une conversion totale ; elle
/// n'est ni proposée ni acceptée tant que le relevé du solde en CHF qui couvre
/// la date n'est pas importé — sinon deux écritures. Le relevé CHF arrivé
/// ensuite, la proposition paraît et l'acceptation passe **une** écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_dont_un_seul_solde_est_importe(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (chf, usd) = releves_partielle();
    let (s, b) = ctx.importer(&[("usd.csv", &usd)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(revue["counts"]["proposals"], 0, "{revue}");
    assert_eq!(
        codes(&revue["toReview"], "code"),
        vec![paire("card_group_statements_missing", "CARD-90000020")],
        "{revue}"
    );
    let manquants = &revue["toReview"][0]["details"]["missingBankAccountIds"];
    assert_eq!(manquants.as_array().map(Vec::len), Some(1), "{revue}");
    // Forgée, la proposition est refusée à l'acceptation, rien d'écrit.
    let carte = ctx.tx("CARD-90000020", ctx.usd).await;
    let frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    let mut ids = [carte, frais];
    ids.sort_unstable();
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(
        b["failed"][0]["details"]["reason"],
        "card_group_statements_missing"
    );
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant
    );
    // Le relevé du solde en CHF arrive : une proposition, une écriture.
    let (s, b) = ctx.importer(&[("chf.csv", &chf)]).await;
    assert_eq!(s, 201, "import : {b}");
    let revue = ctx.revue().await;
    assert_eq!(revue["counts"]["proposals"], 1, "{revue}");
    assert_eq!(revue["toReview"], json!([]), "{revue}");
    let tous = revue["proposals"][0]["bankTransactionIds"].clone();
    let ids: Vec<i64> = serde_json::from_value(tous).unwrap();
    assert_eq!(ids.len(), 3);
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant + 1,
        "une seule écriture"
    );
}

/// Story 39-11d2 (AC-1) — ce que l'écran lit d'un paiement groupé : la
/// proposition porte ses lignes (`lines`, dans l'ordre de
/// `bankTransactionIds`, la ligne CHF en tête) et le montant du marchand
/// (`merchantAmount`) ; un corps d'acceptation qui reprend ces champs
/// d'affichage est accepté (ils sont ignorés), en une écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn revue_du_groupe_avec_ses_lignes(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let revue = ctx.revue().await;
    let p = &revue["proposals"][0];
    assert_eq!(p["type"], "card_group", "{revue}");
    assert_eq!(p["merchantAmount"], "97.95", "{revue}");
    assert_eq!(p["ruleId"], regle, "{revue}");
    let lignes = p["lines"].as_array().expect("lines");
    assert_eq!(lignes.len(), 3, "{revue}");
    let identifiants: Vec<i64> = lignes.iter().map(|l| l["id"].as_i64().unwrap()).collect();
    assert_eq!(identifiants, ids.to_vec(), "l'ordre de bankTransactionIds");
    assert_eq!(lignes[0], p["transaction"], "la principale en tête");
    assert_eq!(lignes[0]["currency"], "CHF");
    assert_eq!(lignes[0]["amount"], "-86.60");
    assert_eq!(lignes[0]["transactionId"], "CARD-90000020");
    let frais = lignes
        .iter()
        .find(|l| l["transactionId"] == "FEE-CARD-90000020")
        .expect("la ligne de frais");
    assert_eq!(
        (&frais["amount"], &frais["currency"]),
        (&json!("-0.04"), &json!("USD"))
    );
    // Le corps tel que la revue le sert, champs d'affichage compris.
    let mut corps = p.clone();
    corps.as_object_mut().unwrap().remove("transaction");
    assert!(corps.get("lines").is_some() && corps.get("merchantAmount").is_some());
    let (s, b) = ctx.accepter(vec![corps]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(b["accepted"][0]["type"], "card_group", "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("reconciled".into(), Some(ecriture)));
    }
}

/// Story 39-11d2 (AC-1) — un `card_group_statements_missing` porte, à la
/// revue, les lignes du paiement (`details.lines`) et la devise de chaque
/// solde manquant (`details.missingCurrencies`, rang pour rang avec
/// `missingBankAccountIds`) ; refusé à l'acceptation, ses `failed[].details`
/// ne portent que les champs du motif : `missingCurrencies`, sans `lines`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn lignes_et_devises_d_un_groupe_a_examiner(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (_, usd) = releves_partielle();
    let (s, b) = ctx.importer(&[("usd.csv", &usd)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(examen[0]["code"], "card_group_statements_missing");
    let details = &examen[0]["details"];
    assert_eq!(details["missingCurrencies"], json!(["CHF"]), "{revue}");
    let solde_chf: i64 =
        sqlx::query_scalar("SELECT id FROM bank_accounts WHERE journal_account_id = ?")
            .bind(ctx.chf)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    assert_eq!(details["missingBankAccountIds"], json!([solde_chf]));
    let lignes: Vec<&str> = details["lines"]
        .as_array()
        .expect("details.lines")
        .iter()
        .map(|l| l["transactionId"].as_str().unwrap())
        .collect();
    assert_eq!(
        lignes,
        vec!["CARD-90000020", "FEE-CARD-90000020"],
        "{revue}"
    );
    let ids: Vec<i64> = serde_json::from_value(details["bankTransactionIds"].clone()).unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    let refus = &b["failed"][0]["details"];
    assert_eq!(refus["reason"], "card_group_statements_missing", "{b}");
    assert_eq!(refus["missingCurrencies"], json!(["CHF"]), "{b}");
    assert_eq!(refus["missingBankAccountIds"], json!([solde_chf]), "{b}");
    assert!(
        refus.get("lines").is_none(),
        "seuls les champs du motif : {b}"
    );
}

/// Revue P2, M1 : le relevé CHF importé **d'abord**, sa ligne `CARD-` est
/// passée seule par la règle (une écriture) ; le relevé USD arrivé ensuite, la
/// part USD du même paiement n'est ni proposée ni acceptée
/// (`card_group_partly_reconciled`, mêmes détails des deux côtés) — jamais une
/// seconde écriture du même paiement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_chf_d_abord_deja_rapprochee(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (chf, usd) = releves_partielle();
    let (s, b) = ctx.importer(&[("chf.csv", &chf)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let ligne_chf = ctx.tx("CARD-90000020", ctx.chf).await;
    let revue = ctx.revue().await;
    let p = proposition(&revue, ligne_chf);
    assert_eq!(p["type"], "rule", "{revue}");
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;

    let (s, b) = ctx.importer(&[("usd.csv", &usd)]).await;
    assert_eq!(s, 201, "import : {b}");
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(examen[0]["code"], "card_group_partly_reconciled", "{revue}");
    assert_eq!(examen[0]["transaction"]["transactionId"], "CARD-90000020");
    let attendus = json!({
        "reconciledBankTransactionIds": [ligne_chf],
        "journalEntryIds": [ecriture],
    });
    for champ in ["reconciledBankTransactionIds", "journalEntryIds"] {
        assert_eq!(examen[0]["details"][champ], attendus[champ], "{revue}");
    }
    // Forgée, la proposition est refusée, avec le même motif et les mêmes
    // détails ; rien d'écrit.
    let mut ids = [
        ctx.tx("CARD-90000020", ctx.usd).await,
        ctx.tx("FEE-CARD-90000020", ctx.usd).await,
    ];
    ids.sort_unstable();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    let refus = &b["failed"][0];
    assert_eq!(refus["errorCode"], "PROVIDER_PROPOSAL_STALE", "{b}");
    assert_eq!(refus["details"]["reason"], "card_group_partly_reconciled");
    for champ in ["reconciledBankTransactionIds", "journalEntryIds"] {
        assert_eq!(refus["details"][champ], attendus[champ], "{b}");
    }
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant,
        "aucune seconde écriture"
    );
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("pending".into(), None));
    }
}

/// Revue P2, M2 : une conversion **totale** (aucune ligne CHF) datée après la
/// dernière ligne du relevé CHF n'est couverte par aucun relevé — « à
/// examiner » ; le relevé CHF suivant importé, dont le solde d'ouverture
/// prolonge la clôture du premier, atteste qu'aucune ligne CHF n'a été
/// sautée : la conversion est proposée, et acceptée en une écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn totale_apres_la_derniere_ligne_chf(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let premier = releve_virements(
        "CHF",
        &[("TRANSFER-70000001", "10-03-2020", "50.00", "50.00")],
    );
    let usd = releve_usd_carte("20-03-2020");
    let (s, b) = ctx
        .importer(&[("chf.csv", &premier), ("usd.csv", &usd)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let ba_chf = ctx.tx("TRANSFER-70000001", ctx.chf).await;
    let ba_chf: i64 =
        sqlx::query_scalar("SELECT bank_account_id FROM bank_transactions WHERE id = ?")
            .bind(ba_chf)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    let revue = ctx.revue().await;
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(
        examen[0]["code"], "card_group_statements_missing",
        "{revue}"
    );
    assert_eq!(
        examen[0]["details"]["missingBankAccountIds"],
        json!([ba_chf])
    );

    let suivant = releve_virements(
        "CHF",
        &[("TRANSFER-70000002", "05-04-2020", "10.00", "60.00")],
    );
    let (s, b) = ctx.importer(&[("chf.csv", &suivant)]).await;
    assert_eq!(s, 201, "import : {b}");
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let groupes = de_groupe(&revue["proposals"]);
    assert_eq!(groupes.len(), 1, "{revue}");
    let ids: Vec<i64> = serde_json::from_value(groupes[0]["bankTransactionIds"].clone()).unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert_eq!(
        ctx.lignes(ecriture).await,
        vec![
            (ctx.charge, dec!(11.35), dec!(0), None, None),
            (
                ctx.usd,
                dec!(0),
                dec!(12.50),
                Some(dec!(13.89)),
                Some(dec!(0.90))
            ),
            (ctx.frais, dec!(0.04), dec!(0), None, None),
            (ctx.perte, dec!(1.11), dec!(0), None, None),
        ]
    );
}

/// Revue P2, M2 : le relevé CHF suivant ne prolonge **pas** le premier (son
/// solde d'ouverture, 60.00, n'est pas la clôture du premier, 50.00) : une
/// ligne CHF a pu être sautée, celle du paiement comprise — la conversion
/// reste « à examiner », et l'acceptation forgée est refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn totale_releve_chf_suivant_discontinu(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let premier = releve_virements(
        "CHF",
        &[("TRANSFER-70000001", "10-03-2020", "50.00", "50.00")],
    );
    let suivant = releve_virements(
        "CHF",
        &[("TRANSFER-70000002", "05-04-2020", "10.00", "70.00")],
    );
    let usd = releve_usd_carte("20-03-2020");
    for f in [&premier, &suivant, &usd] {
        let (s, b) = ctx.importer(&[("releve.csv", f)]).await;
        assert_eq!(s, 201, "import : {b}");
    }
    let regle = regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(
        examen[0]["code"], "card_group_statements_missing",
        "{revue}"
    );
    let mut ids = [
        ctx.tx("CARD-90000020", ctx.usd).await,
        ctx.tx("FEE-CARD-90000020", ctx.usd).await,
    ];
    ids.sort_unstable();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["details"]["reason"], "card_group_statements_missing",
        "{b}"
    );
    assert_eq!(
        b["failed"][0]["details"]["missingBankAccountIds"],
        examen[0]["details"]["missingBankAccountIds"]
    );
}

/// Revue P2 (écart déclaré) : une partielle sur **deux soldes en devise**.
/// Le relevé CHF couvre la date sans ligne du paiement ; la part USD est
/// importée ; le solde EUR n'a qu'un relevé antérieur à la date — rien ne
/// l'atteint : limite écrite, la part USD se propose seule. Le relevé EUR
/// suivant importé, qui ne prolonge pas le précédent, encadre la date sans
/// l'attester : le paiement passe « à examiner » (`missingBankAccountIds` :
/// le solde EUR).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_deux_devises_dont_une_absente(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let chf = releve_virements(
        "CHF",
        &[
            ("TRANSFER-70000011", "05-03-2020", "40.00", "40.00"),
            ("TRANSFER-70000012", "15-03-2020", "10.00", "50.00"),
        ],
    );
    let eur = releve_virements(
        "EUR",
        &[("TRANSFER-70000021", "01-02-2020", "20.00", "20.00")],
    );
    let usd = releve_usd_carte("10-03-2020");
    let (s, b) = ctx
        .importer(&[("chf.csv", &chf), ("eur.csv", &eur), ("usd.csv", &usd)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    assert_eq!(de_groupe(&revue["proposals"]).len(), 1, "limite : {revue}");

    let eur_suivant = releve_virements(
        "EUR",
        &[("TRANSFER-70000022", "01-04-2020", "5.00", "30.00")],
    );
    let (s, b) = ctx.importer(&[("eur.csv", &eur_suivant)]).await;
    assert_eq!(s, 201, "import : {b}");
    let ba_eur: i64 = sqlx::query_scalar(
        "SELECT bank_account_id FROM bank_transactions WHERE transaction_id = 'TRANSFER-70000022'",
    )
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(
        examen[0]["code"], "card_group_statements_missing",
        "{revue}"
    );
    assert_eq!(
        examen[0]["details"]["missingBankAccountIds"],
        json!([ba_eur])
    );
}

/// Le relevé CHF « sans la ligne du paiement » : deux virements qui
/// encadrent le 10-03-2020 (export pris avant le règlement de la carte).
fn releve_chf_sans_la_carte() -> Vec<u8> {
    releve_virements(
        "CHF",
        &[
            ("TRANSFER-70000011", "05-03-2020", "40.00", "40.00"),
            ("TRANSFER-70000012", "15-03-2020", "10.00", "50.00"),
        ],
    )
}

/// Revue P3, M1 (prémisse vérifiée : aucun montant compté deux fois) : le
/// relevé CHF couvre la date **sans** la ligne du paiement ; la part USD forme
/// un groupe, accepté (une écriture, marchand 11.35). Un second export CHF,
/// qui porte la ligne `CARD-` en CHF, est importé ensuite : hors groupe, elle
/// se propose par la règle, au montant de sa seule part (86.60), comme la
/// reprise à la main de l'amendement A-3. Les deux écritures font la charge,
/// les crédits des soldes et la perte de change de l'écriture unique
/// (`partielle_en_une_ecriture_puis_annulation`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_usd_d_abord_puis_ligne_chf(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let usd = releve_usd_carte("10-03-2020");
    let (s, b) = ctx
        .importer(&[("chf.csv", &releve_chf_sans_la_carte()), ("usd.csv", &usd)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    let groupes = de_groupe(&revue["proposals"]);
    assert_eq!(groupes.len(), 1, "{revue}");
    let ids: Vec<i64> = serde_json::from_value(groupes[0]["bankTransactionIds"].clone()).unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");

    let (chf, _) = releves_partielle();
    let (s, b) = ctx.importer(&[("chf2.csv", &chf)]).await;
    assert_eq!(s, 201, "import : {b}");
    let ligne_chf = ctx.tx("CARD-90000020", ctx.chf).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let p = proposition(&revue, ligne_chf);
    assert_eq!(p["type"], "rule", "{revue}");
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(ctx.mouvement(ctx.charge).await.0, dec!(97.95));
    assert_eq!(ctx.mouvement(ctx.chf).await.0, dec!(-86.60));
    assert_eq!(ctx.mouvement(ctx.usd).await, (dec!(-12.50), dec!(-13.89)));
    assert_eq!(ctx.mouvement(ctx.perte).await.0, dec!(1.11));
}

/// Revue P3, L1 : le frais `FEE-CARD` importé seul d'abord (l'export qui finit
/// la veille, `releves_usd_coupes`) et passé au compte de frais ne bloque pas
/// le paiement dont les lignes de carte arrivent ensuite : le groupe (sans le frais) se propose et s'écrit, la part USD
/// créditée de la seule ligne de carte (13.85), le frais n'étant pas compté
/// deux fois.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn frais_d_abord_puis_la_carte(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (chf, _) = releves_partielle();
    let (frais, carte) = releves_usd_coupes();
    let (s, b) = ctx.importer(&[("chf.csv", &chf), ("f.csv", &frais)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let id_frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    let revue = ctx.revue().await;
    let p = proposition(&revue, id_frais);
    assert_eq!(p["type"], "fee", "{revue}");
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;

    let (s, b) = ctx.importer(&[("c.csv", &carte)]).await;
    assert_eq!(s, 201, "import : {b}");
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let groupes = de_groupe(&revue["proposals"]);
    assert_eq!(groupes.len(), 1, "{revue}");
    let ids: Vec<i64> = serde_json::from_value(groupes[0]["bankTransactionIds"].clone()).unwrap();
    assert_eq!(
        ids,
        vec![
            ctx.tx("CARD-90000020", ctx.chf).await,
            ctx.tx("CARD-90000020", ctx.usd).await
        ]
    );
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant + 1
    );
    let lignes = ctx.lignes(ecriture).await;
    assert_eq!(lignes[0], (ctx.charge, dec!(97.95), dec!(0), None, None));
    let usd_credite: Vec<Option<Decimal>> = lignes
        .iter()
        .filter(|l| l.0 == ctx.usd)
        .map(|l| l.3)
        .collect();
    assert_eq!(usd_credite, vec![Some(dec!(13.85))], "{lignes:?}");
}

/// Revue P3, M1/A-3 (le frais arrivé après coup) : le groupe de la part
/// USD passé sans son frais (l'export qui commence à la carte,
/// `releves_usd_coupes`), le frais importé ensuite par l'export de la veille,
/// du même solde, se propose seul (`fee`) et s'accepte : une écriture de frais
/// à part, que le groupe n'a pas comptée. Ce test fige qu'un frais hors groupe
/// n'est pas retiré par un contrôle de ses sœurs ; la réduction du motif 1 à
/// la carte est gardée par `frais_d_abord_puis_la_carte`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn frais_apres_le_groupe_passe_seul(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (frais, carte) = releves_usd_coupes();
    let (s, b) = ctx
        .importer(&[("chf.csv", &releve_chf_sans_la_carte()), ("c.csv", &carte)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    let groupes = de_groupe(&revue["proposals"]);
    assert_eq!(groupes.len(), 1, "{revue}");
    let ids: Vec<i64> = serde_json::from_value(groupes[0]["bankTransactionIds"].clone()).unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");

    let (s, b) = ctx.importer(&[("f.csv", &frais)]).await;
    assert_eq!(s, 201, "import : {b}");
    let id_frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let p = proposition(&revue, id_frais);
    assert_eq!(p["type"], "fee", "{revue}");
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(ctx.etat(id_frais).await.0, "reconciled");
}

/// Revue P3, L2 (`DateCoverage::Gap`, relevé suivant sans solde d'ouverture) :
/// les deux relevés CHF se suivraient, mais le suivant n'a pas de solde
/// d'ouverture — rien n'atteste la continuité : la conversion totale datée
/// entre eux reste « à examiner », le solde CHF manquant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn totale_releve_chf_suivant_sans_solde_d_ouverture(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let premier = releve_virements(
        "CHF",
        &[("TRANSFER-70000001", "10-03-2020", "50.00", "50.00")],
    );
    let suivant = releve_virements(
        "CHF",
        &[("TRANSFER-70000002", "05-04-2020", "10.00", "60.00")],
    );
    let usd = releve_usd_carte("20-03-2020");
    for f in [&premier, &suivant, &usd] {
        let (s, b) = ctx.importer(&[("releve.csv", f)]).await;
        assert_eq!(s, 201, "import : {b}");
    }
    let suivant_id = ctx.tx("TRANSFER-70000002", ctx.chf).await;
    let (import_id, ba_chf): (i64, i64) =
        sqlx::query_as("SELECT import_id, bank_account_id FROM bank_transactions WHERE id = ?")
            .bind(suivant_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    sqlx::query("UPDATE bank_imports SET opening_balance = NULL WHERE id = ?")
        .bind(import_id)
        .execute(&ctx.pool)
        .await
        .unwrap();
    regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(
        examen[0]["code"], "card_group_statements_missing",
        "{revue}"
    );
    assert_eq!(
        examen[0]["details"]["missingBankAccountIds"],
        json!([ba_chf])
    );
}

/// Revue P3, L2 (`DateCoverage::NoPrevious`, solde CHF) : le seul relevé CHF
/// commence après la date du paiement — rien n'atteste ce qui précède : la
/// conversion totale reste « à examiner », le solde CHF manquant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn totale_sans_releve_chf_anterieur(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let chf = releve_virements(
        "CHF",
        &[("TRANSFER-70000002", "05-04-2020", "10.00", "60.00")],
    );
    let usd = releve_usd_carte("20-03-2020");
    let (s, b) = ctx.importer(&[("chf.csv", &chf), ("usd.csv", &usd)]).await;
    assert_eq!(s, 201, "import : {b}");
    let ba_chf: i64 = sqlx::query_scalar(
        "SELECT bank_account_id FROM bank_transactions WHERE transaction_id = 'TRANSFER-70000002'",
    )
    .fetch_one(&ctx.pool)
    .await
    .unwrap();
    regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen = de_groupe(&revue["toReview"]);
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(
        examen[0]["code"], "card_group_statements_missing",
        "{revue}"
    );
    assert_eq!(
        examen[0]["details"]["missingBankAccountIds"],
        json!([ba_chf])
    );
}

/// Revue P3, L2 (`DateCoverage::NoPrevious`, solde en devise) : le seul
/// relevé EUR commence après la date du paiement ; un solde en devise ouvert
/// après la date n'est pas exigé : la partielle se propose.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_solde_en_devise_ouvert_apres(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let eur = releve_virements(
        "EUR",
        &[("TRANSFER-70000022", "01-04-2020", "5.00", "30.00")],
    );
    let (chf, usd) = releves_partielle();
    let (s, b) = ctx
        .importer(&[("chf.csv", &chf), ("eur.csv", &eur), ("usd.csv", &usd)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    regle_boutique(&ctx).await;
    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["toReview"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    assert_eq!(de_groupe(&revue["proposals"]).len(), 1, "{revue}");
}

/// Revue P2, L-d : le prédicat du rang 3 (`settlement_entry_cancel_blocker`)
/// n'exempte une autre ligne du paiement groupé que **rapprochée** : forgée en
/// attente sans perdre son écriture (état qu'aucun chemin ne produit), elle
/// bloque l'annulation (`MATCHED_BANK_TRANSACTION`), à la vue comme au geste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annulation_bloquee_par_une_ligne_liee_non_rapprochee(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    sqlx::query("UPDATE bank_transactions SET status = 'pending' WHERE id = ?")
        .bind(ids[0])
        .execute(&ctx.pool)
        .await
        .unwrap();
    let frais = ctx.tx("FEE-CARD-90000020", ctx.usd).await;
    let (s, vue) = ctx
        .req(
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{frais}"),
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(s, 200, "{vue}");
    assert_eq!(vue["cancelBlockedBy"], "MATCHED_BANK_TRANSACTION", "{vue}");
    assert_eq!(vue["cancelBlockedDocumentId"], ids[0], "{vue}");
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{frais}/cancel"),
            json!({}),
        )
        .await;
    assert_ne!(s, 200, "{b}");
    assert_eq!(ctx.etat(frais).await, ("reconciled".into(), Some(ecriture)));
}

/// Revue L1 : le cours du mois fait gagner du change — 13.89 USD à 0.80 =
/// 11.11 CHF pour un marchand à 11.35 et un frais de 0.04 USD (0.03 CHF, au
/// compte de frais) : la différence (0.27) est **créditée** au compte de gain,
/// et l'écriture est équilibrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_gain_de_change(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    sqlx::query(
        "UPDATE exchange_rates SET rate = 0.80 WHERE company_id = ? AND currency = 'USD' \
         AND rate_date = '2020-03-01'",
    )
    .bind(ctx.seeded.company_id)
    .execute(&ctx.pool)
    .await
    .unwrap();
    let gain: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '6950'")
            .bind(ctx.seeded.company_id)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert_eq!(
        ctx.lignes(ecriture).await,
        vec![
            (ctx.charge, dec!(97.95), dec!(0), None, None),
            (ctx.chf, dec!(0), dec!(86.60), None, None),
            (
                ctx.usd,
                dec!(0),
                dec!(11.11),
                Some(dec!(13.89)),
                Some(dec!(0.80))
            ),
            (ctx.frais, dec!(0.03), dec!(0), None, None),
            (gain, dec!(0), dec!(0.27), None, None),
        ]
    );
}

// ============================================================
// Correctif de la PR de l'Epic 39 — le frais d'un paiement groupé va au
// compte de frais du solde, jamais à la perte de change
// ============================================================

/// Somme des débits et des crédits CHF d'une écriture.
async fn totaux(ctx: &Ctx, ecriture: i64) -> (Decimal, Decimal) {
    sqlx::query_as("SELECT SUM(debit), SUM(credit) FROM journal_entry_lines WHERE entry_id = ?")
        .bind(ecriture)
        .fetch_one(&ctx.pool)
        .await
        .unwrap()
}

/// Le compte de frais d'un solde en devise du compte Wise, tenu dans cette
/// devise : la ligne de frais porte le montant en devise et le cours retenu
/// (comme un frais isolé, `write_provider_entry`) ; la perte de change ne
/// porte que le change (1.11), l'écriture est équilibrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupe_frais_au_compte_de_frais_en_devise(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let frais_usd = compte(
        &ctx.pool,
        ctx.seeded.company_id,
        "6941",
        "Expense",
        Some("USD"),
    )
    .await;
    sqlx::query("UPDATE bank_accounts SET fee_account_id = ? WHERE journal_account_id = ?")
        .bind(frais_usd)
        .bind(ctx.usd)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert_eq!(
        ctx.lignes(ecriture).await,
        vec![
            (ctx.charge, dec!(97.95), dec!(0), None, None),
            (ctx.chf, dec!(0), dec!(86.60), None, None),
            (
                ctx.usd,
                dec!(0),
                dec!(12.50),
                Some(dec!(13.89)),
                Some(dec!(0.90))
            ),
            (
                frais_usd,
                dec!(0.04),
                dec!(0),
                Some(dec!(0.04)),
                Some(dec!(0.90))
            ),
            (ctx.perte, dec!(1.11), dec!(0), None, None),
        ]
    );
    let (debit, credit) = totaux(&ctx, ecriture).await;
    assert_eq!((debit, credit), (dec!(99.10), dec!(99.10)), "équilibrée");
    // Rien au compte de frais en CHF, la perte ne porte pas le frais.
    assert_eq!(ctx.mouvement(ctx.frais).await.0, dec!(0));
    assert_eq!(ctx.mouvement(ctx.perte).await.0, dec!(1.11));
}

/// Le solde qui porte le frais n'a pas de compte de frais : la revue met le
/// groupe « à examiner » sous le motif commun d'un frais sans compte de frais
/// (`fee_account_missing`, le solde nommé), sans proposition ; l'acceptation
/// forgée est refusée en `failed[]` (`PROVIDER_PROPOSAL_STALE`, même motif),
/// HTTP 200, rien d'écrit. Un solde sans frais (CHF) sans compte de frais ne
/// bloque rien : le compte est réglé de nouveau, le groupe s'accepte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn groupe_refuse_sans_compte_de_frais(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let ba_usd: i64 =
        sqlx::query_scalar("SELECT id FROM bank_accounts WHERE journal_account_id = ?")
            .bind(ctx.usd)
            .fetch_one(&ctx.pool)
            .await
            .unwrap();
    let regler = |compte: Option<i64>, ledger: i64| {
        let pool = ctx.pool.clone();
        async move {
            sqlx::query("UPDATE bank_accounts SET fee_account_id = ? WHERE journal_account_id = ?")
                .bind(compte)
                .bind(ledger)
                .execute(&pool)
                .await
                .unwrap();
        }
    };
    regler(None, ctx.usd).await;
    // Le solde CHF, sans frais dans ce paiement, n'a pas non plus de compte
    // de frais : il n'est pas nommé.
    regler(None, ctx.chf).await;

    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let examen: Vec<&Value> = revue["toReview"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["code"] == "fee_account_missing")
        .collect();
    assert_eq!(examen.len(), 1, "{revue}");
    assert_eq!(examen[0]["transaction"]["id"], ids[0], "{revue}");
    assert_eq!(examen[0]["details"]["cardId"], "CARD-90000020");
    assert_eq!(examen[0]["details"]["bankAccountIds"], json!([ba_usd]));
    assert_eq!(examen[0]["details"]["bankTransactionIds"], json!(ids));

    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"], json!([]), "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(b["failed"][0]["bankTransactionId"], ids[0]);
    assert_eq!(b["failed"][0]["details"]["reason"], "fee_account_missing");
    assert_eq!(b["failed"][0]["details"]["bankAccountIds"], json!([ba_usd]));
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant,
        "rien d'écrit"
    );
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("pending".into(), None), "{id}");
    }
    assert_eq!(ctx.mouvement(ctx.perte).await.0, dec!(0));

    // Le compte de frais réglé de nouveau (le seul solde à frais) : accepté.
    regler(Some(ctx.frais), ctx.usd).await;
    let revue = ctx.revue().await;
    assert_eq!(de_groupe(&revue["proposals"]).len(), 1, "{revue}");
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
}

/// PayPal, Stripe et Wise peuvent partager **un même** compte de frais :
/// aucune garde ne l'interdit (ni contrainte d'unicité, ni CHECK, ni la garde
/// des réglages de prestataire) ; le frais du paiement groupé Wise y va.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn trois_prestataires_un_meme_compte_de_frais(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let c = ctx.seeded.company_id;
    let paypal = compte(&ctx.pool, c, "1033", "Asset", None).await;
    let stripe = compte(&ctx.pool, c, "1034", "Asset", None).await;
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "PayPal", "journalAccountId": paypal,
                    "paypal": { "account": "PayPal www", "feeAccountId": ctx.frais } }),
        )
        .await;
    assert_eq!(s, 201, "solde PayPal : {b}");
    let (s, b) = ctx
        .post(
            "/api/v1/bank-accounts",
            json!({ "bankName": "Stripe", "journalAccountId": stripe,
                    "stripe": { "account": "Stripe boutique", "feeAccountId": ctx.frais } }),
        )
        .await;
    assert_eq!(s, 201, "solde Stripe : {b}");
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(DISTINCT provider) FROM bank_accounts WHERE fee_account_id = {}",
            ctx.frais
        ))
        .await,
        3,
        "PayPal, Stripe et Wise sur le même compte de frais"
    );
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert!(
        ctx.lignes(ecriture)
            .await
            .contains(&(ctx.frais, dec!(0.04), dec!(0), None, None)),
        "le frais au compte de frais partagé"
    );
}

/// Revue L1 : une proposition `rule` forgée sur un versement Stripe est
/// refusée à l'acceptation (`kind_changed`) — le versement ne se rapproche
/// jamais par règle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn regle_forgee_sur_un_versement_stripe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let regle = ctx
        .regle("Stripe", "counterparty_contains", "Stripe", None)
        .await;
    let versement = ctx.tx("BALANCE-90000008", ctx.chf).await;
    let avant = ctx.count("SELECT COUNT(*) FROM journal_entries").await;
    let (s, b) = ctx
        .accepter(vec![json!({
            "type": "rule",
            "bankTransactionId": versement,
            "ruleId": regle,
            "counterpartyAccountId": ctx.charge,
        })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(b["failed"][0]["details"]["reason"], "kind_changed");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM journal_entries").await,
        avant
    );
    assert_eq!(ctx.etat(versement).await.0, "pending");
}

// ============================================================
// AC-6 — écarts
// ============================================================

/// Conversion dont l'autre moitié n'est pas importée, frais sans origine,
/// relevé discontinu (un seul écart, même quand un relevé postérieur chevauche
/// sa fin), relevé chevauchant (aucun).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ecarts(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let (s, b) = ctx.importer(&[("chf.csv", CHF), ("eur.csv", EUR)]).await;
    assert_eq!(s, 201, "{b}");
    // Chevauchant : son premier jour (15 mars) est couvert par le relevé CHF.
    let chevauchant = format!(
        "{HEADER}\nCARD-90000031,20-03-2020,20-03-2020 10:00:00.000,-10.00,CHF,Carte,,2000.00,,,,,,,Kiosque,,,,,,,DEBIT,CARD\n\
         BALANCE-90000008,15-03-2020,15-03-2020 08:00:00.000,1500.00,CHF,Argent reçu de Stripe,,2010.00,,,,Stripe Fictif SA,,,,,,,,,,CREDIT,DEPOSIT\n"
    );
    let (s, b) = ctx.importer(&[("chf2.csv", chevauchant.as_bytes())]).await;
    assert_eq!(s, 201, "{b}");
    // Discontinu (ouverture 1000.00, du 1er au 3 mai), avec des frais sans
    // origine.
    let discontinu = format!(
        "{HEADER}\nCARD-90000042,03-05-2020,03-05-2020 10:00:00.000,-3.00,CHF,Carte,,995.00,,,,,,,Kiosque,,,,,,,DEBIT,CARD\n\
         FEE-TRANSFER-90000040,01-05-2020,01-05-2020 10:00:00.000,-2.00,CHF,Frais,,998.00,,,,,,,,,,,,,,DEBIT,TRANSFER\n"
    );
    let (s, b) = ctx.importer(&[("chf3.csv", discontinu.as_bytes())]).await;
    assert_eq!(s, 201, "{b}");
    // Un relevé postérieur qui chevauche la fin du discontinu (3 mai) : le
    // premier jour du discontinu reste découvert, son écart demeure.
    let posterieur = format!(
        "{HEADER}\nCARD-90000041,04-05-2020,04-05-2020 10:00:00.000,-8.00,CHF,Carte,,987.00,,,,,,,Kiosque,,,,,,,DEBIT,CARD\n\
         CARD-90000042,03-05-2020,03-05-2020 10:00:00.000,-3.00,CHF,Carte,,995.00,,,,,,,Kiosque,,,,,,,DEBIT,CARD\n"
    );
    let (s, b) = ctx.importer(&[("chf4.csv", posterieur.as_bytes())]).await;
    assert_eq!(s, 201, "{b}");

    let revue = ctx.revue().await;
    assert_eq!(
        codes(&revue["gaps"], "code"),
        vec![
            paire("balance_gap", ""),
            paire("conversion_half_missing", "BALANCE-90000010"),
            paire("conversion_half_missing", "CONVERSION_ORDER-90000011"),
            paire("fee_without_origin", "FEE-TRANSFER-90000040"),
        ],
        "{revue}"
    );
    let gap = revue["gaps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["code"] == "balance_gap")
        .unwrap();
    assert_eq!(gap["details"]["date"], "2020-05-01");
    assert_eq!(gap["details"]["currency"], "CHF");
    assert_eq!(gap["details"]["found"], "1000.00");
    // Le versement et la conversion orpheline ne sont jamais des règles.
    assert!(
        !codes(&revue["proposals"], "type")
            .iter()
            .any(|(_, id)| id == "BALANCE-90000008" || id == "BALANCE-90000010")
    );
}

// ============================================================
// AC-1, AC-7 — droits, corps, rejeu
// ============================================================

/// Clé `read` : revue admise, acceptation refusée (403) ; corps vide ou mal
/// formé : 400 ; compte inconnu : 404.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn droits_et_refus_globaux(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/settings/api-keys",
            json!({ "name": "lecture", "scope": "read" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let cle = b["key"].as_str().unwrap().to_string();
    let chemin = format!(
        "/api/v1/wise/review?account=Wise%20Pro&fiscalYearId={}",
        ctx.seeded.fiscal_year_id
    );
    let (s, _) = ctx.req(reqwest::Method::GET, &chemin, None, &cle).await;
    assert_eq!(s, 200);
    let frais = ctx.tx("FEE-TRANSFER-90000004", ctx.chf).await;
    let corps = json!({ "account": WISE, "proposals": [{ "type": "fee", "bankTransactionId": frais, "feeAccountId": ctx.frais }] });
    let (s, _) = ctx
        .req(
            reqwest::Method::POST,
            "/api/v1/wise/accept",
            Some(corps),
            &cle,
        )
        .await;
    assert_eq!(s, 403);
    assert_eq!(ctx.etat(frais).await.0, "pending");
    let (s, _) = ctx.accepter(vec![]).await;
    assert_eq!(s, 400);
    let (s, _) = ctx
        .accepter(vec![json!({ "type": "card_group", "bankTransactionIds": [] , "ruleId": 1, "counterpartyAccountId": 1 })])
        .await;
    assert_eq!(s, 400);
    let (s, _) = ctx
        .post(
            "/api/v1/wise/accept",
            json!({ "account": WISE, "proposals": [{ "type": "inconnu" }] }),
        )
        .await;
    assert_eq!(s, 400);
    let (s, b) = ctx
        .req(
            reqwest::Method::GET,
            &format!(
                "/api/v1/wise/review?account=Inconnu&fiscalYearId={}",
                ctx.seeded.fiscal_year_id
            ),
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(s, 404, "{b}");
    // Un compte PayPal n'est pas un compte Wise.
    let (s, _) = ctx
        .req(
            reqwest::Method::GET,
            "/api/v1/paypal/review?account=Wise%20Pro&fiscalYearId=1",
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(s, 404);
}

/// Une transaction de test **alourdie** (la victime choisie par InnoDB est la
/// plus légère).
async fn transaction_lourde(pool: &MySqlPool) -> Transaction<'static, MySql> {
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lest_rejeu (id INT AUTO_INCREMENT PRIMARY KEY, x INT) \
         ENGINE=InnoDB",
    )
    .execute(pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    for _ in 0..10 {
        sqlx::query("INSERT INTO lest_rejeu (x) SELECT seq FROM seq_1_to_50")
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    tx
}

/// Un interblocage injecté dans l'écriture d'un paiement groupé (patron de
/// `rejeu_interblocage_e2e.rs`) : l'acceptation tient l'exercice et attend la
/// ligne d'une transaction que tient le test, qui demande l'exercice →
/// InnoDB annule l'acceptation, **rejouée** (témoin `wise::accept`), sans
/// ligne dans `failed[]`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_groupe(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let capture = CaptureRejeu::installer();
    let mut lourde = transaction_lourde(&ctx.pool).await;
    sqlx::query("SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE")
        .bind(ids[0])
        .fetch_all(&mut *lourde)
        .await
        .unwrap();
    let app = ctx.app.clone();
    let jwt = ctx.jwt.clone();
    let corps = json!({ "account": WISE, "proposals": [groupe(&ids, regle, ctx.charge)] });
    let route = tokio::spawn(async move {
        let resp = app
            .client
            .post(format!("{}/api/v1/wise/accept", app.base_url))
            .bearer_auth(jwt)
            .json(&corps)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    assert!(
        attendre_une_requete_en_cours(&ctx.pool, &["UPDATE bank_transactions"], || route
            .is_finished())
        .await,
        "l'acceptation devait attendre la ligne tenue"
    );
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(ctx.seeded.fiscal_year_id)
        .fetch_all(&mut *lourde)
        .await
        .expect("la transaction de test doit survivre à l'interblocage");
    lourde.rollback().await.unwrap();
    let (status, corps) = route.await.unwrap();
    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(corps["failed"], json!([]), "{corps}");
    assert_eq!(corps["accepted"].as_array().unwrap().len(), 1, "{corps}");
    capture.exiger_un_rejeu("wise::accept");
    let ecriture = corps["accepted"][0]["journalEntryIds"][0].as_i64();
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("reconciled".into(), ecriture));
    }
}

// ============================================================
// AC-4 — coût de la lecture de l'autre côté (mesure)
// ============================================================

/// 300 virements en attente et 20 000 écritures sur le compte du plan du
/// solde CHF : la revue lit l'autre côté une fois par virement ; sa durée est
/// relevée (`--nocapture`, au Change Log), bornée large ici.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cout_de_la_lecture_de_l_autre_cote(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let c = ctx.seeded.company_id;
    let ba: i64 = sqlx::query_scalar(
        "SELECT id FROM bank_accounts WHERE company_id = ? AND journal_account_id = ?",
    )
    .bind(c)
    .bind(ctx.chf)
    .fetch_one(&pool)
    .await
    .unwrap();
    let import = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'mesure.csv', REPEAT('a', 64), 'CSV', '2020-01-01', '2020-12-31', 300, ?)",
    )
    .bind(c)
    .bind(ba)
    .bind(ctx.seeded.admin_user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id();
    sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details, transaction_id, provider_type) \
         SELECT ?, ?, ?, DATE_ADD('2020-01-01', INTERVAL seq DAY), -(seq + 0.50), 'CHF', \
                'Virement', CONCAT('TRANSFER-', 80000000 + seq), 'TRANSFER' \
         FROM seq_1_to_300",
    )
    .bind(c)
    .bind(import)
    .bind(ba)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) \
         SELECT ?, ?, 100000 + seq, DATE_ADD('2020-01-01', INTERVAL (seq MOD 360) DAY), \
                'Banque', 'Mesure' FROM seq_1_to_20000",
    )
    .bind(c)
    .bind(ctx.seeded.fiscal_year_id)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         SELECT e.id, ?, 1, 0, 10.00 FROM journal_entries e \
         WHERE e.company_id = ? AND e.description = 'Mesure'",
    )
    .bind(ctx.chf)
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         SELECT e.id, ?, 2, 10.00, 0 FROM journal_entries e \
         WHERE e.company_id = ? AND e.description = 'Mesure'",
    )
    .bind(ctx.charge)
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    let debut = std::time::Instant::now();
    let revue = ctx.revue().await;
    let duree = debut.elapsed();
    println!("39-11c mesure : revue de 300 virements sur 20 000 écritures en {duree:?}");
    assert_eq!(revue["counts"]["toReview"], 300, "aucune règle : no_rule");
    assert!(
        duree < std::time::Duration::from_secs(30),
        "revue trop lente : {duree:?}"
    );
}

// ============================================================
// Règle commune des prestataires — une transaction Wise n'est écrite qu'une
// fois (correctif des prestataires, #1560, et genre de clé de la #1534)
// ============================================================

/// IBAN du compte bancaire principal du montage (`ba_banque`).
const IBAN_BANQUE: &str = "CH9300762011623852957";

/// Une ligne d'un relevé Wise écrit ici : identifiant, date (`JJ-MM-AAAA`),
/// heure, montant, solde après, sens (`DEBIT`, `CREDIT`), contrepartie, IBAN
/// du bénéficiaire, type de détail (`TRANSFER`, `CARD`).
type LigneWise<'a> = (
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
    &'a str,
);

/// Un relevé Wise d'une devise, lignes de la plus récente à la plus ancienne
/// (comme l'export de Wise). Un virement porte sa contrepartie en
/// bénéficiaire (débit) ou en payeur (crédit), une carte en marchand.
fn releve_wise(devise: &str, lignes: &[LigneWise]) -> Vec<u8> {
    let mut csv = format!("{HEADER}\n");
    for (id, date, heure, montant, solde, sens, nom, iban, genre) in lignes {
        let (payeur, beneficiaire, marchand) = match (*genre, *sens) {
            ("CARD", _) => ("", "", *nom),
            (_, "CREDIT") => (*nom, "", ""),
            _ => ("", *nom, ""),
        };
        csv.push_str(&format!(
            "{id},{date},{date} {heure},{montant},{devise},Mouvement {id},,{solde},,,,{payeur},{beneficiaire},{iban},{marchand},,,,,,,{sens},{genre}\n"
        ));
    }
    csv.into_bytes()
}

/// Les identifiants des écritures nouvelles depuis `avant`.
fn nouvelles(apres: &[i64], avant: &[i64]) -> Vec<i64> {
    apres
        .iter()
        .filter(|e| !avant.contains(e))
        .copied()
        .collect()
}

/// L'acceptation d'une proposition `rule` (ancienne ou forgée) de `id` est
/// refusée `PROVIDER_TRANSACTION_ALREADY_POSTED`, en `200`, l'écriture
/// `ecriture` nommée, sans aucune écriture ni trace d'acceptation.
async fn regle_refusee_deja_comptabilisee(ctx: &Ctx, regle: Value, id: i64, ecriture: i64) {
    let avant = ctx.ecritures().await;
    let audit = ctx
        .count("SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted'")
        .await;
    let (s, b) = ctx.accepter(vec![regle]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"], json!([]), "{b}");
    assert_eq!(
        b["failed"],
        json!([{
            "bankTransactionId": id,
            "errorCode": "PROVIDER_TRANSACTION_ALREADY_POSTED",
            "details": { "journalEntryId": ecriture },
        }]),
        "{b}"
    );
    assert_eq!(ctx.ecritures().await, avant, "aucune seconde écriture");
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted'")
            .await,
        audit,
        "aucune trace d'acceptation"
    );
    assert_eq!(ctx.etat(id).await, ("pending".into(), None));
}

/// Le motif « à examiner » de la transaction `id`, s'il y en a un.
fn motif(revue: &Value, id: i64) -> Option<Value> {
    revue["toReview"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["transaction"]["id"] == id)
        .cloned()
}

/// Le scénario du défaut, sur un virement Wise → banque dans la devise
/// `devise` (`reçu` : le montant en CHF que reçoit la banque) :
///
/// 1. le virement `…001` accepté par sa règle (contrepartie : le compte du
///    plan de la banque), une écriture ;
/// 2. la réception de la banque rattachée à cette écriture : rattaché des
///    deux côtés ;
/// 3. le côté Wise détaché (`…/cancel`) : le lien seul est retiré, l'écriture
///    reste, rapprochée de la réception ;
/// 4. la revue propose de **rattacher** le virement à son écriture, jamais
///    une règle ; l'ancienne proposition `rule`, renvoyée, est refusée
///    `PROVIDER_TRANSACTION_ALREADY_POSTED`, sans rien écrire ;
/// 5. la sonde positive : un second virement du même montant et de la même
///    devise (`…002`, deux jours plus tard), que l'écriture ne nomme pas,
///    reste proposé par sa règle — non rattaché à l'écriture du premier — et
///    s'écrit, par une écriture à lui ; un virement du même montant dans
///    l'autre devise (`…003`) aussi ;
/// 6. le rattachement proposé passe, sans nouvelle écriture.
async fn virement_detache_cote_wise(pool: MySqlPool, devise: &str, autre: &str, recu: &str) {
    let ctx = monter_sans_import(&pool).await;
    let n = if devise == "CHF" { "71" } else { "72" };
    let (t1, t2, t3) = (
        format!("TRANSFER-{n}000001"),
        format!("TRANSFER-{n}000002"),
        format!("TRANSFER-{n}000003"),
    );
    let depot = format!("TRANSFER-{n}000000");
    let principal = releve_wise(
        devise,
        &[
            (
                &t2,
                "17-02-2020",
                "10:00:00.000",
                "-100.00",
                "800.00",
                "DEBIT",
                "Moi-même",
                IBAN_BANQUE,
                "TRANSFER",
            ),
            (
                &t1,
                "15-02-2020",
                "10:00:00.000",
                "-100.00",
                "900.00",
                "DEBIT",
                "Moi-même",
                IBAN_BANQUE,
                "TRANSFER",
            ),
            (
                &depot,
                "01-02-2020",
                "09:00:00.000",
                "1000.00",
                "1000.00",
                "CREDIT",
                "Client Fictif",
                "",
                "TRANSFER",
            ),
        ],
    );
    let jumeau = releve_wise(
        autre,
        &[
            (
                &t3,
                "15-02-2020",
                "10:00:00.000",
                "-100.00",
                "900.00",
                "DEBIT",
                "Moi-même",
                IBAN_BANQUE,
                "TRANSFER",
            ),
            (
                "TRANSFER-79000000",
                "01-02-2020",
                "09:00:00.000",
                "1000.00",
                "1000.00",
                "CREDIT",
                "Client Fictif",
                "",
                "TRANSFER",
            ),
        ],
    );
    let (s, b) = ctx
        .importer(&[("principal.csv", &principal), ("jumeau.csv", &jumeau)])
        .await;
    assert_eq!(s, 201, "import : {b}");
    let regle = ctx
        .regle_vers(
            "Vers la banque",
            "iban_exact",
            IBAN_BANQUE,
            Some("TRANSFER"),
            ctx.banque,
        )
        .await;
    let ledger = |d: &str| match d {
        "CHF" => ctx.chf,
        "EUR" => ctx.eur,
        _ => ctx.usd,
    };
    let (v1, v2, v3) = (
        ctx.tx(&t1, ledger(devise)).await,
        ctx.tx(&t2, ledger(devise)).await,
        ctx.tx(&t3, ledger(autre)).await,
    );

    // 1. Accepter le premier virement par sa règle.
    let ancienne = proposition(&ctx.revue().await, v1);
    assert_eq!(
        (ancienne["type"].clone(), ancienne["ruleId"].clone()),
        (json!("rule"), json!(regle))
    );
    let (s, b) = ctx.accepter(vec![ancienne.clone()]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    // 2. La réception de la banque, rattachée à la même écriture.
    let reception = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-02-16", recu, None)
        .await;
    let (s, b) = ctx.rattacher(reception, ecriture).await;
    assert_eq!(s, 200, "réception : {b}");
    // 3. Détacher le côté Wise : le lien seul.
    ctx.annuler(v1).await;
    assert_eq!(ctx.etat(v1).await, ("pending".into(), None));
    assert_eq!(
        ctx.etat(reception).await,
        ("reconciled".into(), Some(ecriture))
    );
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {ecriture}"
        ))
        .await,
        0,
        "l'écriture reste"
    );

    // 4. La revue : rattacher, jamais réécrire.
    let revue = ctx.revue().await;
    let p1 = proposition(&revue, v1);
    assert_eq!(
        (p1["type"].clone(), p1["journalEntryId"].clone()),
        (json!("link_entry"), json!(ecriture)),
        "{revue}"
    );
    regle_refusee_deja_comptabilisee(&ctx, ancienne, v1, ecriture).await;

    // Un rattachement forgé du second virement à l'écriture qui nomme le
    // premier : périmé (`entry_changed`), rien n'est rattaché.
    let (s, b) = ctx
        .accepter(vec![json!({
            "type": "link_entry",
            "bankTransactionId": v2,
            "journalEntryId": ecriture,
        })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(b["failed"][0]["details"]["reason"], "entry_changed", "{b}");
    assert_eq!(ctx.etat(v2).await, ("pending".into(), None));

    // 5. La sonde positive : le second virement, et le jumeau dans l'autre devise.
    let revue = ctx.revue().await;
    for v in [v2, v3] {
        let p = proposition(&revue, v);
        assert_eq!(p["type"], "rule", "{devise} {v} : {revue}");
        assert!(motif(&revue, v).is_none(), "{revue}");
        let avant = ctx.ecritures().await;
        let (s, b) = ctx.accepter(vec![p]).await;
        assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
        let neuves = nouvelles(&ctx.ecritures().await, &avant);
        assert_eq!(neuves.len(), 1, "une écriture à lui");
        assert_ne!(neuves[0], ecriture);
        assert_eq!(ctx.etat(v).await, ("reconciled".into(), Some(neuves[0])));
    }

    // 6. Le rattachement proposé : sans nouvelle écriture.
    let avant = ctx.ecritures().await;
    let p1 = proposition(&ctx.revue().await, v1);
    let (s, b) = ctx.accepter(vec![p1]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(ctx.ecritures().await, avant);
    assert_eq!(ctx.etat(v1).await, ("reconciled".into(), Some(ecriture)));
}

/// [`virement_detache_cote_wise`] en CHF (le jumeau en EUR).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn virement_rattache_des_deux_cotes_detache_cote_wise_chf(pool: MySqlPool) {
    virement_detache_cote_wise(pool, "CHF", "EUR", "100.00").await;
}

/// [`virement_detache_cote_wise`] en EUR (100 EUR au cours de 0.95 : 95.00 CHF
/// reçus ; le jumeau en CHF).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn virement_rattache_des_deux_cotes_detache_cote_wise_eur(pool: MySqlPool) {
    virement_detache_cote_wise(pool, "EUR", "CHF", "95.00").await;
}

/// L'écriture d'un virement passée **à la main**, hors de la fenêtre du
/// rattachement (dix jours après), rapprochée de la réception de la banque,
/// son côté Wise libre :
///
/// - son libellé ne cite qu'une **carte** (`CARD-…`, un autre genre de clé) :
///   elle ne nomme aucun virement, et la carte ne fait pas d'elle l'écriture
///   d'un autre virement — le repli prudent la retient : le virement est « à
///   examiner » sous `already_posted`, jamais proposé par sa règle ; la règle
///   renvoyée est refusée sans rien écrire ;
/// - la sonde positive : son libellé nomme **un autre virement**
///   (`TRANSFER-73000077`) — elle est l'écriture de celui-là ; le virement est
///   proposé par sa règle et s'écrit, par une écriture à lui.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn virement_ecrit_a_la_main_citant_une_carte_reste_au_repli_prudent(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let releve = releve_wise(
        "CHF",
        &[
            (
                "TRANSFER-73000001",
                "15-02-2020",
                "10:00:00.000",
                "-120.00",
                "880.00",
                "DEBIT",
                "Moi-même",
                IBAN_BANQUE,
                "TRANSFER",
            ),
            (
                "TRANSFER-73000000",
                "01-02-2020",
                "09:00:00.000",
                "1000.00",
                "1000.00",
                "CREDIT",
                "Client Fictif",
                "",
                "TRANSFER",
            ),
        ],
    );
    let (s, b) = ctx.importer(&[("chf.csv", &releve)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = ctx
        .regle_vers(
            "Vers la banque",
            "iban_exact",
            IBAN_BANQUE,
            Some("TRANSFER"),
            ctx.banque,
        )
        .await;
    let v = ctx.tx("TRANSFER-73000001", ctx.chf).await;
    let ecriture = ctx
        .ecriture(
            "2020-02-25",
            "Virement de Wise vers la banque CARD-73000009",
            ctx.banque,
            ctx.chf,
            "120.00",
        )
        .await;
    let reception = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-02-25", "120.00", None)
        .await;
    let (s, b) = ctx.rattacher(reception, ecriture).await;
    assert_eq!(s, 200, "réception : {b}");

    let revue = ctx.revue().await;
    let m = motif(&revue, v).unwrap_or_else(|| panic!("{revue}"));
    assert_eq!(m["code"], "already_posted", "{m}");
    assert_eq!(m["details"], json!({ "journalEntryId": ecriture }));
    assert!(
        !revue["proposals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["transaction"]["id"] == v),
        "{revue}"
    );
    let forgee = json!({
        "type": "rule",
        "bankTransactionId": v,
        "ruleId": regle,
        "counterpartyAccountId": ctx.banque,
    });
    regle_refusee_deja_comptabilisee(&ctx, forgee, v, ecriture).await;

    // La sonde positive : l'écriture nomme un autre virement.
    sqlx::query("UPDATE journal_entries SET description = ? WHERE id = ?")
        .bind("Virement de Wise vers la banque TRANSFER-73000077")
        .bind(ecriture)
        .execute(&ctx.pool)
        .await
        .unwrap();
    let revue = ctx.revue().await;
    assert!(motif(&revue, v).is_none(), "{revue}");
    let p = proposition(&revue, v);
    assert_eq!(p["type"], "rule", "{revue}");
    let avant = ctx.ecritures().await;
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let neuves = nouvelles(&ctx.ecritures().await, &avant);
    assert_eq!(neuves.len(), 1);
    assert_ne!(neuves[0], ecriture);
    assert_eq!(ctx.etat(v).await, ("reconciled".into(), Some(neuves[0])));
}

/// 39-11a2 — un même identifiant au débit puis au crédit (vérification de
/// carte : −5.00 puis +5.00, à quelques secondes). L'écriture passée de la
/// banque vers Wise (+5.00 au solde Wise, la banque −5.00), qui nomme la
/// carte, est l'écriture du **crédit** seul : le crédit est « à examiner »
/// sous `already_posted`, et sa règle renvoyée est refusée sans rien écrire ;
/// le **débit**, de l'autre sens (son mouvement n'égale pas celui de
/// l'écriture), reste proposé par sa règle et s'écrit — la sonde positive.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn meme_identifiant_au_debit_et_au_credit_seul_le_sens_ecrit_est_retenu(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let releve = releve_wise(
        "CHF",
        &[
            (
                "CARD-74000005",
                "01-03-2020",
                "12:00:05.000",
                "5.00",
                "1000.00",
                "CREDIT",
                "Boutique Exemple",
                "",
                "CARD",
            ),
            (
                "CARD-74000005",
                "01-03-2020",
                "12:00:00.000",
                "-5.00",
                "995.00",
                "DEBIT",
                "Boutique Exemple",
                "",
                "CARD",
            ),
            (
                "TRANSFER-74000000",
                "01-02-2020",
                "09:00:00.000",
                "1000.00",
                "1000.00",
                "CREDIT",
                "Client Fictif",
                "",
                "TRANSFER",
            ),
        ],
    );
    let (s, b) = ctx.importer(&[("chf.csv", &releve)]).await;
    assert_eq!(s, 201, "import : {b}");
    let regle = regle_boutique(&ctx).await;
    let ids: Vec<(i64, Decimal)> = sqlx::query_as(
        "SELECT id, amount FROM bank_transactions WHERE transaction_id = 'CARD-74000005' ORDER BY amount",
    )
    .fetch_all(&ctx.pool)
    .await
    .unwrap();
    let [(debit, _), (credit, _)] = ids[..] else {
        panic!("{ids:?}")
    };
    let ecriture = ctx
        .ecriture(
            "2020-03-01",
            "Vérification de carte CARD-74000005",
            ctx.chf,
            ctx.banque,
            "5.00",
        )
        .await;
    let banque = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-03-01", "-5.00", None)
        .await;
    let (s, b) = ctx.rattacher(banque, ecriture).await;
    assert_eq!(s, 200, "banque : {b}");

    let revue = ctx.revue().await;
    let m = motif(&revue, credit).unwrap_or_else(|| panic!("{revue}"));
    assert_eq!(m["code"], "already_posted", "{m}");
    assert_eq!(m["details"], json!({ "journalEntryId": ecriture }));
    let forgee = json!({
        "type": "rule",
        "bankTransactionId": credit,
        "ruleId": regle,
        "counterpartyAccountId": ctx.charge,
    });
    regle_refusee_deja_comptabilisee(&ctx, forgee, credit, ecriture).await;

    // Le débit : proposé, écrit.
    assert!(motif(&revue, debit).is_none(), "{revue}");
    let p = proposition(&revue, debit);
    assert_eq!(p["type"], "rule", "{revue}");
    let avant = ctx.ecritures().await;
    let (s, b) = ctx.accepter(vec![p]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let neuves = nouvelles(&ctx.ecritures().await, &avant);
    assert_eq!(neuves.len(), 1);
    assert_eq!(
        ctx.etat(debit).await,
        ("reconciled".into(), Some(neuves[0]))
    );
}

/// Le paiement par carte converti (la partielle « USD, CHF ») dont la ligne
/// CHF est **déjà comptabilisée** : une écriture passée à la main qui la nomme
/// (`CARD-90000020`), au mouvement de la ligne sur le solde CHF, rapprochée de
/// l'autre côté (une transaction de la banque de sens contraire), son côté
/// Wise libre. La sonde positive d'abord : sans elle, le groupe est proposé.
/// Avec elle, le groupe n'est plus proposé — la ligne est « à examiner » sous
/// `already_posted` — et le groupe renvoyé est refusé
/// `PROVIDER_TRANSACTION_ALREADY_POSTED` (nommé par la ligne principale),
/// sans rien écrire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn partielle_dont_une_ligne_est_deja_comptabilisee(pool: MySqlPool) {
    let (ctx, regle, ids) = monter_partielle(&pool).await;
    let revue = ctx.revue().await;
    assert_eq!(de_groupe(&revue["proposals"]).len(), 1, "{revue}");

    let ecriture = ctx
        .ecriture(
            "2020-03-10",
            "Carte CARD-90000020 réglée depuis la banque",
            ctx.banque,
            ctx.chf,
            "86.60",
        )
        .await;
    let banque = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-03-10", "86.60", None)
        .await;
    let (s, b) = ctx.rattacher(banque, ecriture).await;
    assert_eq!(s, 200, "banque : {b}");

    let revue = ctx.revue().await;
    assert_eq!(
        de_groupe(&revue["proposals"]),
        Vec::<Value>::new(),
        "{revue}"
    );
    let m = motif(&revue, ids[0]).unwrap_or_else(|| panic!("{revue}"));
    assert_eq!(m["code"], "already_posted", "{m}");
    assert_eq!(m["details"], json!({ "journalEntryId": ecriture }));
    let avant = ctx.ecritures().await;
    let (s, b) = ctx.accepter(vec![groupe(&ids, regle, ctx.charge)]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"],
        json!([{
            "bankTransactionId": ids[0],
            "errorCode": "PROVIDER_TRANSACTION_ALREADY_POSTED",
            "details": { "journalEntryId": ecriture },
        }]),
        "{b}"
    );
    assert_eq!(ctx.ecritures().await, avant, "aucune seconde écriture");
    for id in ids {
        assert_eq!(ctx.etat(id).await, ("pending".into(), None));
    }
    // Les autres lignes du paiement restent dans la revue : une part du
    // paiement est déjà passée (revue P1 de la PR, M2).
    let reste = motif(&revue, ids[1]).unwrap_or_else(|| panic!("{revue}"));
    assert_eq!(reste["code"], "card_group_partly_posted", "{reste}");
    assert_eq!(
        reste["details"]["bankTransactionIds"],
        json!([ids[1], ids[2]])
    );
    assert_eq!(
        reste["details"]["postedBankTransactionIds"],
        json!([ids[0]])
    );
    assert_eq!(reste["details"]["journalEntryIds"], json!([ecriture]));
    assert!(
        motif(&revue, ids[2]).is_none(),
        "une ligne par paiement : {revue}"
    );
}

/// Revue P1 de la PR (M1) : une écriture passée à la main qui débite le solde
/// Wise et cite un identifiant Wise, non rapprochée, n'est pas une « vente sans
/// encaissement » — Wise n'a pas de vente ; aucun écart `sale_without_receipt`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ecriture_a_la_main_citant_un_identifiant_n_est_pas_une_vente(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let releve = releve_wise(
        "CHF",
        &[(
            "TRANSFER-75000000",
            "01-02-2020",
            "09:00:00.000",
            "1000.00",
            "1000.00",
            "CREDIT",
            "Client Fictif",
            "",
            "TRANSFER",
        )],
    );
    let (s, b) = ctx.importer(&[("chf.csv", &releve)]).await;
    assert_eq!(s, 201, "import : {b}");
    ctx.ecriture(
        "2020-02-10",
        "Dépôt TRANSFER-75000009",
        ctx.chf,
        ctx.banque,
        "50.00",
    )
    .await;
    let revue = ctx.revue().await;
    assert_eq!(revue["gaps"], json!([]), "{revue}");
}

/// Revue P1 de la PR (L1) : l'écriture qui nomme le virement est hors de la
/// fenêtre (dix jours), une autre, sans identifiant, rapprochée de la banque,
/// est dans la fenêtre. La revue ne propose pas de rattacher le virement à
/// celle-ci — elle peut être l'écriture d'un autre virement — : il est « à
/// examiner » sous `already_posted`, l'écriture qui le nomme désignée. Sonde
/// positive : sans l'écriture qui le nomme, le rattachement est proposé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rattachement_propose_seulement_vers_l_ecriture_qui_nomme(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let releve = releve_wise(
        "CHF",
        &[
            (
                "TRANSFER-76000001",
                "15-02-2020",
                "10:00:00.000",
                "-120.00",
                "880.00",
                "DEBIT",
                "Moi-même",
                IBAN_BANQUE,
                "TRANSFER",
            ),
            (
                "TRANSFER-76000000",
                "01-02-2020",
                "09:00:00.000",
                "1000.00",
                "1000.00",
                "CREDIT",
                "Client Fictif",
                "",
                "TRANSFER",
            ),
        ],
    );
    let (s, b) = ctx.importer(&[("chf.csv", &releve)]).await;
    assert_eq!(s, 201, "import : {b}");
    let v = ctx.tx("TRANSFER-76000001", ctx.chf).await;
    let proche = ctx
        .ecriture(
            "2020-02-16",
            "Virement vers la banque",
            ctx.banque,
            ctx.chf,
            "120.00",
        )
        .await;
    let r1 = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-02-16", "120.00", None)
        .await;
    let (s, b) = ctx.rattacher(r1, proche).await;
    assert_eq!(s, 200, "{b}");
    // Sonde positive : la seule écriture, dans la fenêtre, se propose.
    let p = proposition(&ctx.revue().await, v);
    assert_eq!(
        (p["type"].clone(), p["journalEntryId"].clone()),
        (json!("link_entry"), json!(proche))
    );
    let nommee = ctx
        .ecriture(
            "2020-02-25",
            "Virement vers la banque TRANSFER-76000001",
            ctx.banque,
            ctx.chf,
            "120.00",
        )
        .await;
    let r2 = ctx
        .transaction_bancaire(ctx.ba_banque, "2020-02-25", "120.00", None)
        .await;
    let (s, b) = ctx.rattacher(r2, nommee).await;
    assert_eq!(s, 200, "{b}");
    let revue = ctx.revue().await;
    let m = motif(&revue, v).unwrap_or_else(|| panic!("{revue}"));
    assert_eq!(m["code"], "already_posted", "{m}");
    assert_eq!(m["details"], json!({ "journalEntryId": nommee }));
    assert!(
        !revue["proposals"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["transaction"]["id"] == v),
        "{revue}"
    );
    // Revue P3 de la PR (M-A) : un rattachement forgé vers l'écriture proche,
    // qui n'est pas celle déjà passée du virement, est périmé à l'acceptation
    // (`entry_changed`), rien n'est rattaché ; la sonde positive : le
    // rattachement à l'écriture qui le nomme passe, sans nouvelle écriture.
    let avant = ctx.ecritures().await;
    let (s, b) = ctx
        .accepter(vec![json!({
            "type": "link_entry",
            "bankTransactionId": v,
            "journalEntryId": proche,
        })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(b["failed"][0]["details"]["reason"], "entry_changed", "{b}");
    assert_eq!(ctx.etat(v).await, ("pending".into(), None));
    let (s, b) = ctx
        .accepter(vec![json!({
            "type": "link_entry",
            "bankTransactionId": v,
            "journalEntryId": nommee,
        })])
        .await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(ctx.etat(v).await, ("reconciled".into(), Some(nommee)));
    assert_eq!(ctx.ecritures().await, avant);
}
