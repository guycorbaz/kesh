//! Story 38-11c2 (Epic 38, phase 1) — la revue et l'acceptation PayPal, par les
//! routes `GET /api/v1/paypal/review` et `POST /api/v1/paypal/accept`, sur le
//! moteur commun des prestataires (`routes::provider_accept`).
//!
//! Montage : le fichier fictif de 38-11a importé par la route de 38-11b dans
//! trois soldes (EUR, USD, CHF) d'un compte « PayPal www » ; écritures de vente
//! posées par `POST /journal-entries` (comme le script des boutiques) ; cours
//! mensuels posés en base. Fixtures fictives seulement.
//!
//! ⚠️ Harnais recopié (`test_config`, `spawn_app`, `forge_jwt`), comme dans
//! chacun des fichiers E2E de ce répertoire (choix C70).

mod common;

use std::collections::HashMap;
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
const FIXTURE: &[u8] =
    include_bytes!("../../kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv");
const PAYPAL: &str = "PayPal www";

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

/// Société multi-devises, trois soldes PayPal (EUR avec compte de frais EUR,
/// USD sans compte de frais, CHF avec compte de frais CHF), compte bancaire
/// principal, cours mensuels de janvier à mars 2020, une règle, et le fichier
/// de 38-11a importé.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    chf: i64,
    eur: i64,
    usd: i64,
    frais_chf: i64,
    frais_eur: i64,
    produit: i64,
    charge: i64,
    banque: i64,
    transit: i64,
    ba_eur: i64,
    ba_usd: i64,
    ba_chf: i64,
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

async fn cours(pool: &MySqlPool, company_id: i64, devise: &str, mois: &str, valeur: &str) {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, ?, 'month', ?, ?, 'afc')",
    )
    .bind(company_id)
    .bind(devise)
    .bind(mois)
    .bind(valeur)
    .execute(pool)
    .await
    .unwrap();
}

async fn monter(pool: &MySqlPool) -> Ctx {
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
    for mois in ["2020-01-01", "2020-02-01", "2020-03-01"] {
        cours(pool, c, "EUR", mois, "0.95").await;
        cours(pool, c, "USD", mois, "0.90").await;
    }
    let banque = compte(pool, c, "1020", "Asset", None).await;
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, 'Banque', 'CH9300762011623852957', TRUE, ?)",
    )
    .bind(c)
    .bind(banque)
    .execute(pool)
    .await
    .unwrap();
    let mut ctx = Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone()).await,
        jwt: forge_jwt(seeded.admin_user_id, "Admin", c),
        chf: compte(pool, c, "1030", "Asset", None).await,
        eur: compte(pool, c, "1031", "Asset", Some("EUR")).await,
        usd: compte(pool, c, "1032", "Asset", Some("USD")).await,
        frais_chf: compte(pool, c, "6940", "Expense", None).await,
        frais_eur: compte(pool, c, "6941", "Expense", Some("EUR")).await,
        produit: compte(pool, c, "3400", "Revenue", None).await,
        charge: compte(pool, c, "6500", "Expense", None).await,
        banque,
        transit,
        seeded,
        ba_eur: 0,
        ba_usd: 0,
        ba_chf: 0,
    };
    sqlx::query(
        "INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, \
         counterparty_account_id) VALUES (?, 'Licence', 'counterparty_contains', \
         'Fournisseur Exemple', ?)",
    )
    .bind(c)
    .bind(ctx.charge)
    .execute(pool)
    .await
    .unwrap();
    ctx.ba_eur = ctx.solde(ctx.eur, Some(ctx.frais_eur)).await;
    ctx.ba_usd = ctx.solde(ctx.usd, None).await;
    ctx.ba_chf = ctx.solde(ctx.chf, Some(ctx.frais_chf)).await;
    let (s, b) = ctx.importer(FIXTURE).await;
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
                    "bankName": "PayPal",
                    "journalAccountId": ledger,
                    "paypal": { "account": PAYPAL, "feeAccountId": fee },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde PayPal : {b}");
        b["id"].as_i64().unwrap()
    }

    async fn importer(&self, fichier: &[u8]) -> (u16, Value) {
        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(fichier.to_vec())
                    .file_name("Download.CSV")
                    .mime_str("text/csv")
                    .unwrap(),
            )
            .text("account", PAYPAL.to_string());
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/paypal-imports", self.app.base_url))
            .bearer_auth(&self.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    /// Une écriture de vente de la boutique, par `POST /journal-entries` :
    /// débit du compte PayPal `ledger` (en devise si `devise` porte montant et
    /// cours), crédit du produit.
    async fn vente(
        &self,
        date: &str,
        libelle: &str,
        ledger: i64,
        chf: &str,
        devise: Option<(&str, &str)>,
    ) -> i64 {
        let mut ligne = json!({ "accountId": ledger, "debit": chf, "credit": "0" });
        if let Some((montant, cours)) = devise {
            ligne["foreign"] = json!({ "amount": montant, "rate": cours, "rateSource": "manual" });
        }
        let (s, b) = self
            .post(
                "/api/v1/journal-entries",
                json!({
                    "entryDate": date,
                    "journal": "Ventes",
                    "description": libelle,
                    "lines": [ligne, { "accountId": self.produit, "debit": "0", "credit": chf }],
                }),
            )
            .await;
        assert_eq!(s, 201, "vente : {b}");
        b["id"].as_i64().unwrap()
    }

    /// Les ventes des boutiques : trois concordantes (EUR, USD, CHF), une au
    /// mauvais montant (AK), une sans encaissement (ZZ).
    async fn ventes(&self) -> HashMap<&'static str, i64> {
        let mut v = HashMap::new();
        v.insert(
            "1A",
            self.vente(
                "2020-01-03",
                "Commande WWW-1001 · TXN 0TEST00000000001A",
                self.eur,
                "95.00",
                Some(("100", "0.95")),
            )
            .await,
        );
        v.insert(
            "2B",
            self.vente(
                "2020-01-15",
                "Abonnement TXN 0TEST00000000002B",
                self.usd,
                "18.00",
                Some(("20", "0.90")),
            )
            .await,
        );
        v.insert(
            "6F",
            self.vente(
                "2020-02-14",
                "Commande 1002 transaction: 0test00000000006f",
                self.chf,
                "1234.56",
                None,
            )
            .await,
        );
        v.insert(
            "AK",
            self.vente(
                "2020-03-25",
                "Commande 1003 TXN 0TEST0000000000AK",
                self.eur,
                "27.55",
                Some(("29", "0.95")),
            )
            .await,
        );
        v.insert(
            "ZZ",
            self.vente(
                "2020-03-26",
                "Commande 1009 TXN 0TEST0000000000ZZ",
                self.eur,
                "9.50",
                Some(("10", "0.95")),
            )
            .await,
        );
        v
    }

    async fn revue(&self) -> Value {
        let (s, b) = self
            .req(
                reqwest::Method::GET,
                &format!(
                    "/api/v1/paypal/review?account=PayPal%20www&fiscalYearId={}",
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
            "/api/v1/paypal/accept",
            json!({ "account": PAYPAL, "proposals": propositions }),
        )
        .await
    }

    async fn tx(&self, transaction_id: &str) -> i64 {
        sqlx::query_scalar("SELECT id FROM bank_transactions WHERE transaction_id = ?")
            .bind(transaction_id)
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

    async fn ecritures(&self) -> i64 {
        self.count("SELECT COUNT(*) FROM journal_entries").await
    }
}

/// Une ligne lue : compte, débit, crédit, montant en devise, provenance.
type LigneFx = (i64, Decimal, Decimal, Option<Decimal>, Option<String>);
/// Une ligne lue : compte, débit, crédit, montant en devise, cours, provenance.
type LigneTaux = (
    i64,
    Decimal,
    Decimal,
    Option<Decimal>,
    Option<Decimal>,
    Option<String>,
);

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

fn codes(liste: &Value) -> Vec<(String, Option<String>)> {
    liste
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["code"].as_str().unwrap().to_string(),
                f["transaction"]["transactionId"]
                    .as_str()
                    .map(str::to_string),
            )
        })
        .collect()
}

// ============================================================
// AC-5, AC-6, AC-7, AC-8 — la revue, puis tout accepter
// ============================================================

/// La revue liste ce qu'il faut et n'écrit rien ; l'acceptation du tout écrit
/// au bon CHF, rapproche, trace ; la seconde revue ne propose plus rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn revue_puis_acceptation_du_tout(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ventes = ctx.ventes().await;
    let ecritures_avant = ctx.ecritures().await;
    let audit_avant = ctx.count("SELECT COUNT(*) FROM audit_log").await;

    let revue = ctx.revue().await;
    assert_eq!(revue["account"], PAYPAL);
    assert_eq!(revue["fiscalYearId"], ctx.seeded.fiscal_year_id);
    assert_eq!(
        ctx.ecritures().await,
        ecritures_avant,
        "la revue n'écrit rien"
    );
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log").await,
        audit_avant,
        "la revue ne trace rien"
    );
    let genres: Vec<(String, i64)> = revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["type"].as_str().unwrap().to_string(),
                p["transaction"]["id"].as_i64().unwrap(),
            )
        })
        .collect();
    assert_eq!(genres.len(), 11, "{revue}");
    assert_eq!(revue["counts"]["proposals"], 11);
    let t = |id: &'static str| ctx.tx(id);
    let attendu = [
        ("link_entry", t("0TEST00000000001A").await),
        ("fee", t("0TEST00000000001A:FEE").await),
        ("link_entry", t("0TEST00000000002B").await),
        ("rule", t("0TEST00000000003C").await),
        ("fee", t("0TEST00000000003C:FEE").await),
        ("conversion", t("0TEST00000000004D").await),
        ("link_entry", t("0TEST00000000006F").await),
        ("fee", t("0TEST00000000006F:FEE").await),
        ("withdrawal", t("0TEST00000000007G").await),
        ("fee", t("0TEST0000000000AK:FEE").await),
        ("fee", t("0TEST0000000000BL:FEE").await),
    ];
    for (genre, id) in attendu {
        assert!(
            genres.contains(&(genre.to_string(), id)),
            "{genre} {id} attendu : {revue}"
        );
    }
    // Rattachement à la vente de même identifiant ; aucun code TVA nulle part.
    let lien = proposition(&revue, t("0TEST00000000001A").await);
    assert_eq!(lien["journalEntryId"], ventes["1A"]);
    assert_eq!(lien["entry"]["ids"], json!(["0TEST00000000001A"]));
    assert!(!revue.to_string().contains("vat"), "aucun code TVA");
    // La conversion EUR → USD, nommée par sa moitié débitée.
    let conv = proposition(&revue, t("0TEST00000000004D").await);
    assert_eq!(conv["creditTransactionId"], t("0TEST00000000005E").await);
    assert_eq!(conv["otherTransaction"]["currency"], "USD");
    // Le retrait CHF, contrepartie préremplie au compte du plan du principal.
    let retrait = proposition(&revue, t("0TEST00000000007G").await);
    assert_eq!(retrait["counterpartyAccountId"], ctx.banque);
    // Montants en chaînes décimales.
    assert_eq!(lien["transaction"]["amount"], "100.00");
    // Écarts et lignes à examiner.
    assert_eq!(
        codes(&revue["gaps"]),
        vec![
            (
                "amount_or_currency_mismatch".to_string(),
                Some("0TEST0000000000AK".to_string())
            ),
            ("sale_without_receipt".to_string(), None),
        ]
    );
    let ecart = &revue["gaps"][0]["details"];
    assert_eq!(ecart["expectedAmount"], "30.00");
    assert_eq!(ecart["foundAmount"], "29.00");
    assert_eq!(
        revue["gaps"][1]["entries"][0]["journalEntryId"],
        ventes["ZZ"]
    );
    assert_eq!(
        codes(&revue["toReview"]),
        vec![
            (
                "fee_account_missing".to_string(),
                Some("0TEST00000000002B:FEE".to_string())
            ),
            ("no_rule".to_string(), Some("0TEST0000000000BL".to_string())),
        ]
    );
    // Story 38-11d (revue P1, M5) — la transaction d'une ligne « à examiner » porte
    // l'e-mail de la contrepartie (colonne importée), pour écrire une règle à filtre.
    let sans_regle = revue["toReview"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["code"] == "no_rule")
        .unwrap();
    assert_eq!(
        sans_regle["transaction"]["counterpartyEmail"], "daniel@example.invalid",
        "{sans_regle}"
    );
    assert!(
        revue["toReview"][0]["transaction"]
            .as_object()
            .unwrap()
            .contains_key("counterpartyEmail")
    );

    // Tout accepter.
    let propositions: Vec<Value> = revue["proposals"].as_array().unwrap().clone();
    let (s, b) = ctx.accepter(propositions).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"], json!([]), "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 11, "{b}");
    for (_, id) in &genres {
        assert_eq!(ctx.etat(*id).await.0, "reconciled", "transaction {id}");
    }
    // Le rattachement n'écrit rien : la transaction pointe la vente.
    assert_eq!(
        ctx.etat(t("0TEST00000000001A").await).await.1,
        Some(ventes["1A"])
    );
    // Frais EUR 3.20 au cours de janvier (0.95) : 3.04 CHF, des deux côtés en EUR.
    let frais = ctx.etat(t("0TEST00000000001A:FEE").await).await.1.unwrap();
    let lignes: Vec<LigneFx> = sqlx::query_as(
        "SELECT account_id, debit, credit, amount_currency, rate_source FROM journal_entry_lines \
         WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(frais)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lignes,
        vec![
            (
                ctx.eur,
                dec!(0),
                dec!(3.04),
                Some(dec!(3.20)),
                Some("afc".into())
            ),
            (
                ctx.frais_eur,
                dec!(3.04),
                dec!(0),
                Some(dec!(3.20)),
                Some("afc".into())
            ),
        ]
    );
    let (journal, libelle): (String, String) =
        sqlx::query_as("SELECT journal, description FROM journal_entries WHERE id = ?")
            .bind(frais)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(journal, "Banque");
    assert_eq!(libelle, "Frais PayPal EUR · TXN 0TEST00000000001A");
    // Frais positifs (remboursement) : crédit des frais, débit du solde.
    let rendu = ctx.etat(t("0TEST0000000000BL:FEE").await).await.1.unwrap();
    let (debit_solde,): (Decimal,) = sqlx::query_as(
        "SELECT debit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(rendu)
    .bind(ctx.eur)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(debit_solde, dec!(0.91), "0.96 EUR × 0.95");
    // La conversion EUR → USD : deux écritures de même CHF (40 × 0.95 = 38.00),
    // compte de conversion soldé ; chaque transaction sur l'écriture de son compte.
    let e_src = ctx.etat(t("0TEST00000000004D").await).await.1.unwrap();
    let e_dst = ctx.etat(t("0TEST00000000005E").await).await.1.unwrap();
    assert_ne!(e_src, e_dst);
    assert_eq!(
        ctx.mouvement(ctx.transit).await.0,
        dec!(0),
        "conversion soldée"
    );
    let (chf_src,): (Decimal,) = sqlx::query_as(
        "SELECT credit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(e_src)
    .bind(ctx.eur)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (chf_dst,): (Decimal,) = sqlx::query_as(
        "SELECT debit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(e_dst)
    .bind(ctx.usd)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((chf_src, chf_dst), (dec!(38.00), dec!(38.00)));
    // Le retrait : débit de la banque, crédit du solde CHF.
    let r = ctx.etat(t("0TEST00000000007G").await).await.1.unwrap();
    let (banque_debit,): (Decimal,) = sqlx::query_as(
        "SELECT debit FROM journal_entry_lines WHERE entry_id = ? AND account_id = ?",
    )
    .bind(r)
    .bind(ctx.banque)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(banque_debit, dec!(1000.00));
    // La règle : compteur et trace.
    assert_eq!(
        ctx.count("SELECT applied_count FROM reconciliation_rules")
            .await,
        1
    );
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'reconciliation_rule.applied'")
            .await,
        1
    );
    // Solde USD en devise = somme des transactions USD rapprochées (vente 20,
    // conversion 44.12 ; les frais 2B:FEE, sans compte, restent en attente).
    assert_eq!(ctx.mouvement(ctx.usd).await.1, dec!(64.12));
    // Une trace par proposition acceptée, sous l'acteur.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted' \
             AND actor_type = 'user'"
        )
        .await,
        11
    );

    // Seconde revue : plus rien à proposer, écarts et lignes à examiner inchangés.
    let seconde = ctx.revue().await;
    assert_eq!(seconde["proposals"], json!([]), "{seconde}");
    assert_eq!(codes(&seconde["gaps"]), codes(&revue["gaps"]));
    assert_eq!(codes(&seconde["toReview"]), codes(&revue["toReview"]));
}

// ============================================================
// AC-8 — conversion CHF ↔ EUR en une écriture, annulation d'un côté
// ============================================================

/// Un relevé de plus : une conversion CHF → EUR (chaque moitié pointe l'autre).
fn fichier_conversion_chf() -> Vec<u8> {
    let entete = std::str::from_utf8(FIXTURE)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let ligne = |id: &str, devise: &str, montant: &str, reference: &str, sens: &str| {
        format!(
            "\"14/03/2020\",\"10:00:00\",\"CET\",\"\",\"User Initiated Currency Conversion\",\"Completed\",\"{devise}\",\"{montant}\",\"0.00\",\"{montant}\",\"\",\"\",\"{id}\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"{reference}\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"\",\"{sens}\"\r\n"
        )
    };
    let mut f = format!("{entete}\r\n");
    f.push_str(&ligne(
        "0TESTCONVCHF00001",
        "CHF",
        "-93.00",
        "0TESTCONVEUR00001",
        "Debit",
    ));
    f.push_str(&ligne(
        "0TESTCONVEUR00001",
        "EUR",
        "100.00",
        "0TESTCONVCHF00001",
        "Credit",
    ));
    f.into_bytes()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_chf_en_une_ecriture_puis_annulation_d_un_cote(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx.importer(&fichier_conversion_chf()).await;
    assert_eq!(s, 201, "{b}");
    let chf = ctx.tx("0TESTCONVCHF00001").await;
    let eur = ctx.tx("0TESTCONVEUR00001").await;

    let revue = ctx.revue().await;
    let conv = proposition(&revue, chf);
    assert_eq!(conv["type"], "conversion");
    assert_eq!(conv["creditTransactionId"], eur);
    let (s, b) = ctx.accepter(vec![conv]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"], json!([]), "{b}");
    assert_eq!(b["accepted"][0]["bankTransactionIds"], json!([chf, eur]));
    let ecriture = b["accepted"][0]["journalEntryIds"][0].as_i64().unwrap();
    assert_eq!(
        b["accepted"][0]["journalEntryIds"],
        json!([ecriture]),
        "une écriture"
    );
    assert_eq!(ctx.etat(chf).await, ("reconciled".into(), Some(ecriture)));
    assert_eq!(ctx.etat(eur).await, ("reconciled".into(), Some(ecriture)));
    // Le CHF réel fait foi : 93.00 des deux côtés, 100 EUR au cours déduit.
    let lignes: Vec<LigneTaux> = sqlx::query_as(
        "SELECT account_id, debit, credit, amount_currency, exchange_rate, rate_source \
             FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(ecriture)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        lignes,
        vec![
            (
                ctx.eur,
                dec!(93.00),
                dec!(0),
                Some(dec!(100.00)),
                Some(dec!(0.93)),
                Some("manual".into())
            ),
            (ctx.chf, dec!(0), dec!(93.00), None, None, None),
        ]
    );
    // Annuler un côté ne retire que son lien : l'écriture reste, l'autre côté
    // reste rapproché.
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{eur}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(eur).await, ("pending".into(), None));
    assert_eq!(ctx.etat(chf).await, ("reconciled".into(), Some(ecriture)));
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {ecriture}"
        ))
        .await,
        0,
        "aucune contre-passation"
    );
}

// ============================================================
// AC-7 — rejugement, refus par proposition
// ============================================================

/// Une proposition acceptée deux fois : la seconde est périmée, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn acceptee_deux_fois_la_seconde_perimee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let revue = ctx.revue().await;
    let frais = proposition(&revue, ctx.tx("0TEST00000000001A:FEE").await);
    let (s, b) = ctx.accepter(vec![frais.clone()]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let avant = ctx.ecritures().await;
    let (s, b) = ctx.accepter(vec![frais]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"], json!([]));
    assert_eq!(
        b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
        "{b}"
    );
    assert_eq!(
        b["failed"][0]["details"]["reason"],
        "transaction_not_pending"
    );
    assert_eq!(ctx.ecritures().await, avant, "rien d'écrit");
}

/// La vente rattachée entre la revue et l'acceptation : le rattachement est
/// périmé ; le reste du lot est accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn vente_rattachee_entre_la_revue_et_l_acceptation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ventes = ctx.ventes().await;
    let revue = ctx.revue().await;
    let tx_1a = ctx.tx("0TEST00000000001A").await;
    let lien = proposition(&revue, tx_1a);
    let frais = proposition(&revue, ctx.tx("0TEST00000000001A:FEE").await);
    // Une autre transaction pointe la vente entre-temps.
    sqlx::query(
        "UPDATE bank_transactions SET matched_entry_id = ?, status = 'reconciled' WHERE id = ?",
    )
    .bind(ventes["1A"])
    .bind(ctx.tx("0TEST00000000007G").await)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = ctx.accepter(vec![lien, frais]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    assert_eq!(b["accepted"][0]["type"], "fee");
    assert_eq!(b["failed"][0]["bankTransactionId"], tx_1a);
    assert_eq!(b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE");
    assert_eq!(b["failed"][0]["details"]["reason"], "entry_linked");
    assert_eq!(ctx.etat(tx_1a).await, ("pending".into(), None));
}

/// Cours absent pour le mois : `RATE_UNAVAILABLE` dans `failed[]`, le reste
/// accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cours_absent_dans_failed_le_reste_accepte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query("DELETE FROM exchange_rates WHERE currency = 'EUR' AND rate_date = '2020-03-01'")
        .execute(&pool)
        .await
        .unwrap();
    let revue = ctx.revue().await;
    let mars = ctx.tx("0TEST0000000000AK:FEE").await;
    let janvier = ctx.tx("0TEST00000000001A:FEE").await;
    let (s, b) = ctx
        .accepter(vec![
            proposition(&revue, mars),
            proposition(&revue, janvier),
        ])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["accepted"][0]["bankTransactionIds"],
        json!([janvier]),
        "{b}"
    );
    assert_eq!(
        b["failed"],
        json!([{
            "bankTransactionId": mars,
            "errorCode": "RATE_UNAVAILABLE",
            "details": { "currency": "EUR", "month": "2020-03-01" },
        }])
    );
    assert_eq!(ctx.etat(mars).await.0, "pending");
}

/// Refus de contrepartie et de forme : un retrait vers un compte de charge,
/// une règle dont la contrepartie a changé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn contrepartie_controlee_et_regle_perimee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let retrait = ctx.tx("0TEST00000000007G").await;
    let regle_tx = ctx.tx("0TEST00000000003C").await;
    let rule_id = ctx.count("SELECT id FROM reconciliation_rules").await;
    let (s, b) = ctx
        .accepter(vec![
            json!({ "type": "withdrawal", "bankTransactionId": retrait, "counterpartyAccountId": ctx.charge }),
            json!({ "type": "rule", "bankTransactionId": regle_tx, "ruleId": rule_id, "counterpartyAccountId": ctx.produit }),
        ])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "VALIDATION_ERROR", "{b}");
    assert_eq!(
        b["failed"][0]["details"]["reason"],
        "counterparty_wrong_type"
    );
    assert_eq!(b["failed"][1]["errorCode"], "PROVIDER_PROPOSAL_STALE");
    assert_eq!(
        b["failed"][1]["details"]["reason"],
        "rule_no_longer_matches"
    );
    // La contrepartie ne peut pas être le compte du solde lui-même.
    let (_, b) = ctx
        .accepter(vec![json!({ "type": "withdrawal", "bankTransactionId": retrait, "counterpartyAccountId": ctx.chf })])
        .await;
    assert_eq!(
        b["failed"][0]["details"]["reason"], "counterparty_equals_bank_ledger",
        "{b}"
    );
}

// ============================================================
// AC-6, AC-7 — refus globaux, transaction inconnue, droits
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_globaux_et_par_proposition(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let fy = ctx.seeded.fiscal_year_id;
    // Compte inconnu, exercice inconnu.
    let (s, b) = ctx
        .req(
            reqwest::Method::GET,
            &format!("/api/v1/paypal/review?account=Inconnu&fiscalYearId={fy}"),
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (404, Some("PROVIDER_ACCOUNT_NOT_FOUND"))
    );
    let (s, b) = ctx
        .req(
            reqwest::Method::GET,
            "/api/v1/paypal/review?account=PayPal%20www&fiscalYearId=999999",
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (404, Some("FISCAL_YEAR_NOT_FOUND"))
    );
    let frais = ctx.tx("0TEST00000000001A:FEE").await;
    let corps = |p: Value| json!({ "account": PAYPAL, "proposals": p });
    // Corps vide, doublon (les deux moitiés d'une conversion comptent),
    // retrait sans contrepartie, JSON mal formé : 400.
    for p in [
        json!([]),
        json!([
            { "type": "fee", "bankTransactionId": frais, "feeAccountId": ctx.frais_eur },
            { "type": "conversion", "debitTransactionId": 1, "creditTransactionId": frais },
        ]),
        json!([{ "type": "withdrawal", "bankTransactionId": frais }]),
        json!([{ "type": "inconnu", "bankTransactionId": frais }]),
    ] {
        let (s, b) = ctx.post("/api/v1/paypal/accept", corps(p.clone())).await;
        assert_eq!(s, 400, "{p} : {b}");
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR");
    }
    let (s, b) = ctx
        .post(
            "/api/v1/paypal/accept",
            json!({ "account": "Inconnu", "proposals": [{ "type": "fee", "bankTransactionId": frais, "feeAccountId": 1 }] }),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (404, Some("PROVIDER_ACCOUNT_NOT_FOUND"))
    );
    // Transaction inconnue, ou d'un autre compte : par proposition.
    let (s, b) = ctx
        .accepter(vec![
            json!({ "type": "fee", "bankTransactionId": 99999999, "feeAccountId": ctx.frais_eur }),
        ])
        .await;
    assert_eq!(s, 200);
    assert_eq!(
        b["failed"][0]["errorCode"], "BANK_TRANSACTION_NOT_FOUND",
        "{b}"
    );
}

/// Clé d'API `read` : admise à la revue, refusée à l'acceptation (403).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_read_admise_en_lecture_refusee_a_l_acceptation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/settings/api-keys",
            json!({ "name": "lecture", "scope": "read" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let cle = b["key"].as_str().unwrap().to_string();
    let (s, _) = ctx
        .req(
            reqwest::Method::GET,
            &format!(
                "/api/v1/paypal/review?account=PayPal%20www&fiscalYearId={}",
                ctx.seeded.fiscal_year_id
            ),
            None,
            &cle,
        )
        .await;
    assert_eq!(s, 200);
    let frais = ctx.tx("0TEST00000000001A:FEE").await;
    let (s, _) = ctx
        .req(
            reqwest::Method::POST,
            "/api/v1/paypal/accept",
            Some(json!({ "account": PAYPAL, "proposals": [{ "type": "fee", "bankTransactionId": frais, "feeAccountId": ctx.frais_eur }] })),
            &cle,
        )
        .await;
    assert_eq!(s, 403);
    assert_eq!(ctx.etat(frais).await.0, "pending");
}

/// Une clé `read-write` accepte ; la trace porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_read_write_acceptee_et_tracee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .post(
            "/api/v1/settings/api-keys",
            json!({ "name": "écriture", "scope": "read-write" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    let cle = b["key"].as_str().unwrap().to_string();
    let frais = ctx.tx("0TEST00000000001A:FEE").await;
    let (s, b) = ctx
        .req(
            reqwest::Method::POST,
            "/api/v1/paypal/accept",
            Some(json!({ "account": PAYPAL, "proposals": [{ "type": "fee", "bankTransactionId": frais, "feeAccountId": ctx.frais_eur }] })),
            &cle,
        )
        .await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted' \
             AND actor_type = 'api_key' AND actor_api_key_id IS NOT NULL"
        )
        .await,
        1
    );
}

// ============================================================
// AC-7 — interblocage dans une proposition : rejeu, jamais dans failed[]
// ============================================================

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

/// Le patron du rejeu (`rejeu_interblocage_e2e.rs`) : la transaction de test
/// tient `tenue` (requête et identifiant), l'acceptation de `proposition` est
/// vue en attente sur `motifs`, la transaction de test demande `fermeture` que
/// l'acceptation tient déjà → InnoDB annule l'acceptation, qui doit être
/// **rejouée** (témoin `paypal::accept`) et réussir, sans ligne dans `failed[]`.
async fn rejeu(
    ctx: &Ctx,
    proposition: Value,
    tenue: (&str, i64),
    motifs: &[&str],
    fermeture: (&str, i64),
) -> Value {
    let capture = CaptureRejeu::installer();
    let mut lourde = transaction_lourde(&ctx.pool).await;
    sqlx::query(tenue.0)
        .bind(tenue.1)
        .fetch_all(&mut *lourde)
        .await
        .unwrap();
    let app = ctx.app.clone();
    let jwt = ctx.jwt.clone();
    let route = tokio::spawn(async move {
        let resp = app
            .client
            .post(format!("{}/api/v1/paypal/accept", app.base_url))
            .bearer_auth(jwt)
            .json(&json!({ "account": PAYPAL, "proposals": [proposition] }))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    assert!(
        attendre_une_requete_en_cours(&ctx.pool, motifs, || route.is_finished()).await,
        "l'acceptation devait attendre sur {motifs:?}"
    );
    sqlx::query(fermeture.0)
        .bind(fermeture.1)
        .fetch_all(&mut *lourde)
        .await
        .expect("la transaction de test doit survivre à l'interblocage");
    lourde.rollback().await.unwrap();
    let (status, corps) = route.await.unwrap();
    assert_eq!(status, 200, "⛔ l'interblocage doit être rejoué : {corps}");
    assert_eq!(
        corps["failed"],
        json!([]),
        "jamais un interblocage dans failed[] : {corps}"
    );
    assert_eq!(corps["accepted"].as_array().unwrap().len(), 1, "{corps}");
    capture.exiger_un_rejeu("paypal::accept");
    corps
}

/// Écrire une écriture, puis rapprocher : l'acceptation tient l'exercice
/// (`FOR UPDATE`) et attend la ligne de la transaction que tient le test.
async fn rejeu_d_une_ecriture(ctx: &Ctx, proposition: Value, transaction: i64) {
    let corps = rejeu(
        ctx,
        proposition,
        (
            "SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE",
            transaction,
        ),
        &["UPDATE bank_transactions"],
        (
            "SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE",
            ctx.seeded.fiscal_year_id,
        ),
    )
    .await;
    let ecriture = corps["accepted"][0]["journalEntryIds"][0].as_i64();
    assert_eq!(ctx.etat(transaction).await, ("reconciled".into(), ecriture));
    assert!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id = {}",
            ecriture.unwrap()
        ))
        .await
            >= 1
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_frais(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let id = ctx.tx("0TEST00000000001A:FEE").await;
    let avant = ctx.ecritures().await;
    rejeu_d_une_ecriture(
        &ctx,
        json!({ "type": "fee", "bankTransactionId": id, "feeAccountId": ctx.frais_eur }),
        id,
    )
    .await;
    assert_eq!(ctx.ecritures().await, avant + 1, "une seule écriture");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_regle(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let id = ctx.tx("0TEST00000000003C").await;
    let rule_id = ctx.count("SELECT id FROM reconciliation_rules").await;
    rejeu_d_une_ecriture(
        &ctx,
        json!({ "type": "rule", "bankTransactionId": id, "ruleId": rule_id, "counterpartyAccountId": ctx.charge }),
        id,
    )
    .await;
    assert_eq!(
        ctx.count("SELECT applied_count FROM reconciliation_rules")
            .await,
        1,
        "compteur incrémenté une fois"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_retrait(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let id = ctx.tx("0TEST00000000007G").await;
    rejeu_d_une_ecriture(
        &ctx,
        json!({ "type": "withdrawal", "bankTransactionId": id, "counterpartyAccountId": ctx.banque }),
        id,
    )
    .await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_conversion_a_deux_ecritures(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let debit = ctx.tx("0TEST00000000004D").await;
    let credit = ctx.tx("0TEST00000000005E").await;
    let avant = ctx.ecritures().await;
    rejeu_d_une_ecriture(
        &ctx,
        json!({ "type": "conversion", "debitTransactionId": debit, "creditTransactionId": credit }),
        debit,
    )
    .await;
    assert_eq!(ctx.ecritures().await, avant + 2, "deux écritures, une fois");
    assert_eq!(ctx.etat(credit).await.0, "reconciled");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_conversion_a_une_ecriture(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx.importer(&fichier_conversion_chf()).await;
    assert_eq!(s, 201, "{b}");
    let debit = ctx.tx("0TESTCONVCHF00001").await;
    let credit = ctx.tx("0TESTCONVEUR00001").await;
    let avant = ctx.ecritures().await;
    rejeu_d_une_ecriture(
        &ctx,
        json!({ "type": "conversion", "debitTransactionId": debit, "creditTransactionId": credit }),
        debit,
    )
    .await;
    assert_eq!(ctx.ecritures().await, avant + 1, "une écriture, une fois");
}

/// Le rattachement : `link_entry_in_tx` tient la transaction (`FOR UPDATE`) et
/// attend l'écriture de vente que tient le test.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_rattachement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ventes = ctx.ventes().await;
    let id = ctx.tx("0TEST00000000001A").await;
    rejeu(
        &ctx,
        json!({ "type": "link_entry", "bankTransactionId": id, "journalEntryId": ventes["1A"] }),
        (
            "SELECT id FROM journal_entries WHERE id = ? FOR UPDATE",
            ventes["1A"],
        ),
        &["journal_entries", "FOR UPDATE"],
        (
            "SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE",
            id,
        ),
    )
    .await;
    assert_eq!(
        ctx.etat(id).await,
        ("reconciled".into(), Some(ventes["1A"]))
    );
}

// ============================================================
// AC-5 — coût de la revue (mesure)
// ============================================================

/// 600 transactions et 600 écritures de vente : la revue tient en une
/// seconde ou deux (mesure au Change Log, `--nocapture`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cout_de_la_revue_600_transactions_600_ecritures(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.seeded.company_id;
    let import: i64 =
        sqlx::query_scalar("SELECT MIN(id) FROM bank_imports WHERE bank_account_id = ?")
            .bind(ctx.ba_eur)
            .fetch_one(&pool)
            .await
            .unwrap();
    let mut tx = pool.begin().await.unwrap();
    for i in 0..600 {
        let id = format!("0PERF{i:012}");
        sqlx::query(
            "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
             amount, currency, details, transaction_id, provider_type) \
             VALUES (?, ?, ?, '2020-06-01', 10.00, 'EUR', '', ?, 'Website Payment')",
        )
        .bind(c)
        .bind(import)
        .bind(ctx.ba_eur)
        .bind(&id)
        .execute(&mut *tx)
        .await
        .unwrap();
        let e = sqlx::query(
            "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
             journal, description) VALUES (?, ?, ?, '2020-06-01', 'Ventes', ?)",
        )
        .bind(c)
        .bind(ctx.seeded.fiscal_year_id)
        .bind(100_000 + i as i64)
        .bind(format!("Vente TXN {id}"))
        .execute(&mut *tx)
        .await
        .unwrap()
        .last_insert_id() as i64;
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit, \
             amount_currency, exchange_rate, rate_source) VALUES \
             (?, ?, 1, 9.50, 0, 10.00, 0.95, 'manual'), (?, ?, 2, 0, 9.50, NULL, NULL, NULL)",
        )
        .bind(e)
        .bind(ctx.eur)
        .bind(e)
        .bind(ctx.produit)
        .execute(&mut *tx)
        .await
        .unwrap();
    }
    tx.commit().await.unwrap();
    let debut = std::time::Instant::now();
    let revue = ctx.revue().await;
    let duree = debut.elapsed();
    eprintln!("revue de 600 transactions et 600 écritures : {duree:?}");
    let liens = revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["type"] == "link_entry")
        .count();
    assert_eq!(liens, 600, "les 600 ventes rattachables");
    assert!(duree < std::time::Duration::from_secs(10), "{duree:?}");
}
