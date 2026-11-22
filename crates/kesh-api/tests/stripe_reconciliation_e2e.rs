//! Story 40-7c (Epic 40, phase 1) — la revue et l'acceptation Stripe, par les
//! routes `GET /api/v1/stripe/review` et `POST /api/v1/stripe/accept`, sur le
//! moteur commun des prestataires (`routes::provider_accept`).
//!
//! Montage : le CSV fictif de 40-7a importé par la route de 40-7b dans trois
//! soldes (CHF, EUR, USD) d'un compte « Stripe boutique » ; écritures de vente
//! posées par `POST /journal-entries` (comme le script des boutiques) ; cours
//! mensuels posés en base ; comptes de versement : la banque principale (CHF)
//! et un compte Wise EUR fictif écrit en base (la route n'admet pas encore de
//! compte Wise ; la réception s'y insère par le dépôt). Fixtures fictives
//! seulement.
//!
//! ⚠️ Harnais recopié (`test_config`, `spawn_app`, `forge_jwt`), comme dans
//! chacun des fichiers E2E de ce répertoire (choix C70).

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use common::capture_rejeu::CaptureRejeu;
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{BankImportSourceFormat, NewBankImport, NewBankTransaction};
use kesh_db::repositories::bank_imports;
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
    include_bytes!("../../kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv");
const STRIPE: &str = "Stripe boutique";

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

/// Société multi-devises, trois soldes Stripe (CHF avec frais CHF et
/// versement vers la banque principale ; EUR avec frais EUR et versement vers
/// un compte Wise EUR ; USD sans frais ni versement), cours mensuels de
/// janvier à mars 2020, et le fichier de 40-7a importé.
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
    banque: i64,
    wise_eur_ledger: i64,
    ba_banque: i64,
    ba_wise: i64,
    ba_chf: i64,
    ba_eur: i64,
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

fn d(s: &str) -> NaiveDate {
    NaiveDate::from_str(s).unwrap()
}

fn hash64(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    format!("{:0>64x}", h.finish())
}

/// Le montage sans import (un test l'importe en deux fichiers).
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
    for mois in ["2020-01-01", "2020-02-01", "2020-03-01"] {
        cours(pool, c, "EUR", mois, "0.95").await;
        cours(pool, c, "USD", mois, "0.90").await;
    }
    let banque = compte(pool, c, "1020", "Asset", None).await;
    let ba_banque = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, 'Banque', 'CH9300762011623852957', TRUE, ?)",
    )
    .bind(c)
    .bind(banque)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let wise_eur_ledger = compte(pool, c, "1025", "Asset", Some("EUR")).await;
    let ba_wise = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, \
         provider, provider_account) VALUES (?, 'Wise', FALSE, ?, 'wise', 'Wise fictif')",
    )
    .bind(c)
    .bind(wise_eur_ledger)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
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
        banque,
        wise_eur_ledger,
        ba_banque,
        ba_wise,
        seeded,
        ba_chf: 0,
        ba_eur: 0,
    };
    ctx.ba_chf = ctx
        .solde(ctx.chf, Some(ctx.frais_chf), Some(ctx.ba_banque))
        .await;
    ctx.ba_eur = ctx
        .solde(ctx.eur, Some(ctx.frais_eur), Some(ctx.ba_wise))
        .await;
    ctx.solde(ctx.usd, None, None).await;
    ctx
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let ctx = monter_sans_import(pool).await;
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

    async fn solde(&self, ledger: i64, fee: Option<i64>, payout: Option<i64>) -> i64 {
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
        b["id"].as_i64().unwrap()
    }

    async fn importer(&self, fichier: &[u8]) -> (u16, Value) {
        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(fichier.to_vec())
                    .file_name("payouts.csv")
                    .mime_str("text/csv")
                    .unwrap(),
            )
            .text("account", STRIPE.to_string());
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/stripe-imports", self.app.base_url))
            .bearer_auth(&self.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    /// Une écriture passée par `POST /journal-entries` : débit `debit`
    /// (en devise si `devise` porte montant et cours), crédit `credit`, au
    /// même CHF.
    async fn ecriture(
        &self,
        date: &str,
        libelle: &str,
        debit: (i64, Option<(&str, &str)>),
        credit: (i64, Option<(&str, &str)>),
        chf: &str,
    ) -> i64 {
        let ligne = |compte: i64, sens_debit: bool, devise: Option<(&str, &str)>| {
            let mut l = if sens_debit {
                json!({ "accountId": compte, "debit": chf, "credit": "0" })
            } else {
                json!({ "accountId": compte, "debit": "0", "credit": chf })
            };
            if let Some((montant, cours)) = devise {
                l["foreign"] = json!({ "amount": montant, "rate": cours, "rateSource": "manual" });
            }
            l
        };
        let (s, b) = self
            .post(
                "/api/v1/journal-entries",
                json!({
                    "entryDate": date,
                    "journal": "Ventes",
                    "description": libelle,
                    "lines": [ligne(debit.0, true, debit.1), ligne(credit.0, false, credit.1)],
                }),
            )
            .await;
        assert_eq!(s, 201, "écriture : {b}");
        b["id"].as_i64().unwrap()
    }

    /// Une vente de la boutique : débit du solde Stripe `ledger`, crédit du
    /// produit.
    async fn vente(
        &self,
        date: &str,
        libelle: &str,
        ledger: i64,
        chf: &str,
        devise: Option<(&str, &str)>,
    ) -> i64 {
        self.ecriture(date, libelle, (ledger, devise), (self.produit, None), chf)
            .await
    }

    /// Les ventes des boutiques : deux concordantes (CHF, EUR), une au mauvais
    /// montant (ch_TEST0002), deux pour la même charge (ch_TEST0004), une sans
    /// charge (ch_TESTSANS) ; ch_TEST0099 n'a pas de vente.
    async fn ventes(&self) -> std::collections::HashMap<&'static str, i64> {
        let mut v = std::collections::HashMap::new();
        v.insert(
            "1",
            self.vente(
                "2020-01-29",
                "Boutique, commande 1001 · TXN ch_TEST0001",
                self.chf,
                "100.00",
                None,
            )
            .await,
        );
        v.insert(
            "2",
            self.vente(
                "2020-01-30",
                "Boutique, commande 1002 (ch_TEST0002)",
                self.chf,
                "55.00",
                None,
            )
            .await,
        );
        v.insert(
            "3",
            self.vente(
                "2020-02-05",
                "Boutique, commande 1003 · TXN ch_TEST0003",
                self.eur,
                "190.00",
                Some(("200", "0.95")),
            )
            .await,
        );
        for (cle, libelle) in [
            ("4a", "Boutique, commande 1004 · ch_TEST0004"),
            ("4b", "Doublon de la commande 1004 · ch_TEST0004"),
        ] {
            v.insert(
                cle,
                self.vente(
                    "2020-02-12",
                    libelle,
                    self.usd,
                    "36.00",
                    Some(("40", "0.90")),
                )
                .await,
            );
        }
        v.insert(
            "sans",
            self.vente(
                "2020-02-13",
                "Boutique, commande 1009 · ch_TESTSANS",
                self.eur,
                "9.50",
                Some(("10", "0.95")),
            )
            .await,
        );
        v
    }

    async fn revue_avec(&self, jeton: &str) -> (u16, Value) {
        self.req(
            reqwest::Method::GET,
            &format!(
                "/api/v1/stripe/review?account=Stripe%20boutique&fiscalYearId={}",
                self.seeded.fiscal_year_id
            ),
            None,
            jeton,
        )
        .await
    }

    async fn revue(&self) -> Value {
        let (s, b) = self.revue_avec(&self.jwt).await;
        assert_eq!(s, 200, "revue : {b}");
        b
    }

    async fn accepter(&self, propositions: Vec<Value>) -> (u16, Value) {
        self.post(
            "/api/v1/stripe/accept",
            json!({ "account": STRIPE, "proposals": propositions }),
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

    async fn ecritures(&self) -> i64 {
        self.count("SELECT COUNT(*) FROM journal_entries").await
    }

    /// Une transaction insérée par le dépôt sur le compte bancaire `bank` (la
    /// réception d'un versement, côté banque ou Wise).
    async fn reception(&self, bank: i64, montant: &str, devise: &str, le: &str) -> i64 {
        let jour = d(le);
        let cle = format!("{bank}-{montant}-{devise}-{le}");
        let mut tx = self.pool.begin().await.unwrap();
        let (_, txs) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.seeded.company_id,
                bank_account_id: bank,
                filename: "reception.xml".into(),
                file_hash: hash64(&cle),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some(format!("RECEPTION-{cle}")),
                period_from: jour,
                period_to: jour,
                opening_balance: None,
                closing_balance: None,
                transaction_count: 1,
                imported_by_user_id: self.seeded.admin_user_id,
            },
            vec![NewBankTransaction {
                company_id: self.seeded.company_id,
                bank_account_id: bank,
                booking_date: jour,
                value_date: Some(jour),
                amount: Decimal::from_str(montant).unwrap(),
                currency: devise.into(),
                reference: None,
                details: "Argent reçu de Stripe".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Stripe".into()),
                counterparty_email: None,
                provider_type: None,
            }],
        )
        .await
        .expect("import de la réception");
        tx.commit().await.unwrap();
        txs[0].id
    }

    async fn rattacher(&self, transaction: i64, ecriture: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/reconciliation/transactions/{transaction}/link-entry"),
            json!({ "journalEntryId": ecriture }),
        )
        .await
    }
}

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

/// `(motif, transactionId)` d'une liste, en ensemble (l'ordre est celui des
/// dates, contrôlé ailleurs par la revue commune).
fn codes(liste: &Value) -> BTreeSet<(String, Option<String>)> {
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

fn genres(revue: &Value) -> BTreeSet<(String, String)> {
    revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| {
            (
                p["type"].as_str().unwrap().to_string(),
                p["transaction"]["transactionId"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            )
        })
        .collect()
}

fn paire(code: &str, txid: Option<&str>) -> (String, Option<String>) {
    (code.to_string(), txid.map(str::to_string))
}

/// Une ligne lue : compte, débit, crédit, montant en devise, cours.
type Ligne = (i64, Decimal, Decimal, Option<Decimal>, Option<Decimal>);

async fn lignes(pool: &MySqlPool, ecriture: i64) -> Vec<Ligne> {
    sqlx::query_as(
        "SELECT account_id, debit, credit, amount_currency, exchange_rate \
         FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order",
    )
    .bind(ecriture)
    .fetch_all(pool)
    .await
    .unwrap()
}

// ============================================================
// AC-1 à AC-6 — la revue, puis tout accepter
// ============================================================

/// La revue liste ce qu'il faut et n'écrit rien ; l'acceptation du tout
/// rattache les charges, écrit frais et versements au bon CHF, à la date de
/// la transaction, au libellé de l'import ; la seconde revue ne propose plus
/// rien et liste les versements sans réception ; une réception EUR rattachée
/// fait disparaître l'écart, une réception CHF du même nombre est refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn revue_puis_acceptation_du_tout(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ventes = ctx.ventes().await;
    let ecritures_avant = ctx.ecritures().await;
    let audit_avant = ctx.count("SELECT COUNT(*) FROM audit_log").await;

    let revue = ctx.revue().await;
    assert_eq!(revue["account"], STRIPE);
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
    let p = |g: &str, t: &str| (g.to_string(), t.to_string());
    assert_eq!(
        genres(&revue),
        BTreeSet::from([
            p("link_entry", "txn_TEST0001"),
            p("fee", "txn_TEST0001:FEE"),
            p("fee", "txn_TEST0002:FEE"),
            p("fee", "txn_TEST0003:FEE"),
            p("fee", "txn_TEST0004"),
            p("payout", "po_TEST0001"),
            p("link_entry", "txn_TEST0005"),
            p("fee", "txn_TEST0005:FEE"),
            p("fee", "txn_TEST0006:FEE"),
            p("fee", "txn_TEST0007"),
            p("fee", "txn_TEST0007:FEE"),
            p("payout", "po_TEST0002"),
        ]),
        "{revue}"
    );
    assert_eq!(revue["counts"]["proposals"], 12);
    let lien = proposition(&revue, ctx.tx("txn_TEST0005").await);
    assert_eq!(lien["journalEntryId"], ventes["3"]);
    assert_eq!(lien["entry"]["ids"], json!(["ch_TEST0003"]));
    let versement = proposition(&revue, ctx.tx("po_TEST0002").await);
    assert_eq!(versement["payoutBankAccountId"], ctx.ba_wise);
    assert_eq!(versement["payoutAccountLabel"], "Wise");
    assert_eq!(versement["transaction"]["amount"], "-270.30");
    assert!(!revue.to_string().contains("vat"), "aucun code TVA");
    assert_eq!(
        codes(&revue["gaps"]),
        BTreeSet::from([
            paire("amount_or_currency_mismatch", Some("txn_TEST0002")),
            paire("receipt_without_sale", Some("txn_TEST0006")),
            paire("sale_without_receipt", None),
        ]),
        "{revue}"
    );
    assert_eq!(
        codes(&revue["toReview"]),
        BTreeSet::from([
            paire("refund", Some("txn_TEST0003")),
            paire("ambiguous_sale", Some("txn_TEST0008")),
            paire("fee_account_missing", Some("txn_TEST0008:FEE")),
            paire("payout_account_missing", Some("po_TEST0003")),
            paire("other_category", Some("txn_TEST0009")),
            paire("payout_reversal", Some("po_TEST0004")),
        ]),
        "{revue}"
    );

    // Tout accepter, en renvoyant les propositions de la revue telles quelles.
    let propositions: Vec<Value> = revue["proposals"].as_array().unwrap().clone();
    let (s, b) = ctx.accepter(propositions).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"], json!([]), "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 12, "{b}");
    // Les charges pointent leur vente ; aucune écriture pour elles.
    assert_eq!(
        ctx.etat(ctx.tx("txn_TEST0001").await).await,
        ("reconciled".into(), Some(ventes["1"]))
    );
    assert_eq!(
        ctx.etat(ctx.tx("txn_TEST0005").await).await,
        ("reconciled".into(), Some(ventes["3"]))
    );
    assert_eq!(
        ctx.ecritures().await,
        ecritures_avant + 10,
        "dix écritures : huit frais, deux versements"
    );
    // Frais CHF : à la date de la charge (non à sa date de disponibilité), au
    // libellé de la transaction.
    let frais_tx = ctx.tx("txn_TEST0001:FEE").await;
    let frais = ctx.etat(frais_tx).await.1.unwrap();
    assert_eq!(
        lignes(&pool, frais).await,
        vec![
            (ctx.chf, dec!(0), dec!(3.20), None, None),
            (ctx.frais_chf, dec!(3.20), dec!(0), None, None),
        ]
    );
    let (journal, libelle, date): (String, String, NaiveDate) =
        sqlx::query_as("SELECT journal, description, entry_date FROM journal_entries WHERE id = ?")
            .bind(frais)
            .fetch_one(&pool)
            .await
            .unwrap();
    let details: String = sqlx::query_scalar("SELECT details FROM bank_transactions WHERE id = ?")
        .bind(frais_tx)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(journal, "Banque");
    assert_eq!(libelle, details, "le libellé de la transaction");
    assert!(
        libelle.starts_with("Frais Stripe · Encaissement Stripe de 100.00 CHF -3.20=96.80 CHF"),
        "{libelle}"
    );
    assert!(libelle.ends_with(" - po_TEST0001"), "{libelle}");
    assert_eq!(date, d("2020-01-29"), "date de la charge, non 2020-02-03");
    // Frais rendus (remboursement) : débit du solde.
    let rendu = ctx.etat(ctx.tx("txn_TEST0003:FEE").await).await.1.unwrap();
    assert_eq!(
        lignes(&pool, rendu).await[0],
        (ctx.chf, dec!(0.60), dec!(0), None, None)
    );
    // Frais EUR au cours de février : 6.00 EUR, 5.70 CHF, des deux côtés.
    let frais_eur = ctx.etat(ctx.tx("txn_TEST0005:FEE").await).await.1.unwrap();
    assert_eq!(
        lignes(&pool, frais_eur).await,
        vec![
            (
                ctx.eur,
                dec!(0),
                dec!(5.70),
                Some(dec!(6.00)),
                Some(dec!(0.95))
            ),
            (
                ctx.frais_eur,
                dec!(5.70),
                dec!(0),
                Some(dec!(6.00)),
                Some(dec!(0.95))
            ),
        ]
    );
    // Versement EUR : débit Wise EUR, crédit solde Stripe EUR, 270.30 EUR au
    // même cours, même CHF ; daté du versement.
    let po2 = ctx.tx("po_TEST0002").await;
    let e_po2 = ctx.etat(po2).await.1.unwrap();
    let l = lignes(&pool, e_po2).await;
    assert_eq!(l.len(), 2);
    assert_eq!((l[0].0, l[1].0), (ctx.eur, ctx.wise_eur_ledger), "{l:?}");
    assert_eq!(l[0].2, l[1].1, "même CHF");
    assert_eq!(
        (l[0].3, l[0].4, l[1].3, l[1].4),
        (
            Some(dec!(270.30)),
            Some(dec!(0.95)),
            Some(dec!(270.30)),
            Some(dec!(0.95))
        )
    );
    let (libelle, date): (String, NaiveDate) =
        sqlx::query_as("SELECT description, entry_date FROM journal_entries WHERE id = ?")
            .bind(e_po2)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(libelle, "Versement Stripe po_TEST0002 270.30 EUR");
    assert_eq!(date, d("2020-02-10"));
    // Versement CHF : vers la banque principale, 125.15.
    let e_po1 = ctx.etat(ctx.tx("po_TEST0001").await).await.1.unwrap();
    assert_eq!(
        lignes(&pool, e_po1).await,
        vec![
            (ctx.chf, dec!(0), dec!(125.15), None, None),
            (ctx.banque, dec!(125.15), dec!(0), None, None),
        ]
    );
    // Une trace par proposition acceptée, sous l'acteur.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'provider.proposal_accepted' \
             AND actor_type = 'user' AND JSON_VALUE(details_json, '$.provider') = 'stripe'"
        )
        .await,
        12
    );

    // Seconde revue : plus rien à proposer ; les deux versements attendent leur
    // réception.
    let seconde = ctx.revue().await;
    assert_eq!(seconde["proposals"], json!([]), "{seconde}");
    let mut attendu = codes(&revue["gaps"]);
    attendu.insert(paire("payout_without_receipt", Some("po_TEST0001")));
    attendu.insert(paire("payout_without_receipt", Some("po_TEST0002")));
    assert_eq!(codes(&seconde["gaps"]), attendu, "{seconde}");
    assert_eq!(codes(&seconde["toReview"]), codes(&revue["toReview"]));
    let ecart = seconde["gaps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["transaction"]["transactionId"] == "po_TEST0002")
        .unwrap();
    assert_eq!(
        ecart["details"],
        json!({ "payoutId": "po_TEST0002", "journalEntryId": e_po2, "payoutBankAccountId": ctx.ba_wise })
    );

    // AC-1 : une transaction CHF du même nombre ne se rattache pas à l'écriture
    // d'un versement EUR ; la réception Wise EUR, si (autre côté, même devise).
    let chf = ctx
        .reception(ctx.ba_banque, "270.30", "CHF", "2020-02-11")
        .await;
    let (s, b) = ctx.rattacher(chf, e_po2).await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (409, Some("RECONCILIATION_ENTRY_NOT_LINKABLE")),
        "{b}"
    );
    let wise = ctx
        .reception(ctx.ba_wise, "270.30", "EUR", "2020-02-11")
        .await;
    let (s, b) = ctx.rattacher(wise, e_po2).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id = {e_po2}"
        ))
        .await,
        2,
        "deux transactions sur l'écriture du versement"
    );
    let troisieme = ctx.revue().await;
    let sans_reception: Vec<&Value> = troisieme["gaps"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|g| g["code"] == "payout_without_receipt")
        .collect();
    assert_eq!(sans_reception.len(), 1, "{troisieme}");
    assert_eq!(
        sans_reception[0]["transaction"]["transactionId"],
        "po_TEST0001"
    );
}

// ============================================================
// Décision (c) du 21.11 — ce qu'on n'accepte pas se traite à la main par l'API
// ============================================================

/// Une proposition qu'on n'accepte pas reste proposée ; une ligne à examiner
/// (remboursement CHF, retour de versement USD) se traite par une écriture
/// passée à la main, en devise, puis le rattachement — et sort de la revue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn non_accepte_reste_propose_et_se_traite_a_la_main(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let revue = ctx.revue().await;
    let n = revue["counts"]["proposals"].as_i64().unwrap();
    assert_eq!(
        n, 10,
        "sans vente : huit frais et deux versements — {revue}"
    );
    // On accepte un seul frais : les autres propositions restent.
    let frais = proposition(&revue, ctx.tx("txn_TEST0001:FEE").await);
    let (s, b) = ctx.accepter(vec![frais]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let seconde = ctx.revue().await;
    assert_eq!(seconde["counts"]["proposals"], n - 1, "{seconde}");

    // Le remboursement CHF : écriture à la main, puis rattachement.
    let remboursement = ctx.tx("txn_TEST0003").await;
    let e = ctx
        .ecriture(
            "2020-01-31",
            "Remboursement de la vente A",
            (ctx.produit, None),
            (ctx.chf, None),
            "20.00",
        )
        .await;
    let (s, b) = ctx.rattacher(remboursement, e).await;
    assert_eq!(s, 200, "{b}");
    // Le retour de versement USD : écriture en devise, puis rattachement.
    let retour = ctx.tx("txn_TEST0009").await;
    let e = ctx
        .ecriture(
            "2020-02-20",
            "Retour du versement po_TEST0003",
            (ctx.produit, None),
            (ctx.usd, Some(("38.54", "0.90"))),
            "34.69",
        )
        .await;
    let (s, b) = ctx.rattacher(retour, e).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(retour).await, ("reconciled".into(), Some(e)));
    let troisieme = ctx.revue().await;
    let restes = codes(&troisieme["toReview"]);
    assert!(
        !restes.contains(&paire("refund", Some("txn_TEST0003"))),
        "{troisieme}"
    );
    assert!(
        !restes.contains(&paire("other_category", Some("txn_TEST0009"))),
        "{troisieme}"
    );
    assert_eq!(troisieme["counts"]["proposals"], n - 1);
}

// ============================================================
// AC-4 c — versement importé en deux fichiers
// ============================================================

/// Un versement dont les lignes arrivent par deux fichiers : son montant est
/// fixé au premier, la ligne du second le fait mentir → écart
/// `payout_total_mismatch`, aucune proposition.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn versement_importe_en_deux_fichiers_ecart_de_total(pool: MySqlPool) {
    let ctx = monter_sans_import(&pool).await;
    let entete = std::str::from_utf8(FIXTURE)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let premier = format!(
        "{entete}\npo_DEUX,2020-02-03 00:00:00,txn_D1,2020-01-29 10:00:00,2020-02-03 00:00:00,chf,100.00,3.20,96.80,charge,Vente 1,ch_D1,ch_D1\n"
    );
    let second = format!(
        "{entete}\npo_DEUX,2020-02-03 00:00:00,txn_D2,2020-01-30 10:00:00,2020-02-03 00:00:00,chf,50.00,1.75,48.25,charge,Vente 2,ch_D2,ch_D2\n"
    );
    let (s, b) = ctx.importer(premier.as_bytes()).await;
    assert_eq!(s, 201, "{b}");
    let (s, b) = ctx.importer(second.as_bytes()).await;
    assert_eq!(s, 201, "{b}");
    let revue = ctx.revue().await;
    assert!(
        !genres(&revue).iter().any(|(g, _)| g == "payout"),
        "{revue}"
    );
    let ecart = revue["gaps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|g| g["code"] == "payout_total_mismatch")
        .unwrap_or_else(|| panic!("écart de total attendu : {revue}"));
    assert_eq!(
        ecart["details"],
        json!({ "payoutId": "po_DEUX", "expected": "96.80", "found": "145.05" })
    );
    // L'acceptation forcée du versement est périmée, rien d'écrit.
    let po = ctx.tx("po_DEUX").await;
    let avant = ctx.ecritures().await;
    let (s, b) = ctx
        .accepter(vec![json!({ "type": "payout", "bankTransactionId": po })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE");
    assert_eq!(b["failed"][0]["details"]["reason"], "payout_total_mismatch");
    assert_eq!(ctx.ecritures().await, avant);
}

// ============================================================
// AC-6 — rejugement, refus par proposition
// ============================================================

/// Un versement accepté deux fois : la seconde est périmée, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn versement_accepte_deux_fois_le_second_perime(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let revue = ctx.revue().await;
    let v = proposition(&revue, ctx.tx("po_TEST0002").await);
    let (s, b) = ctx.accepter(vec![v.clone()]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    assert_eq!(b["accepted"][0]["type"], "payout");
    let avant = ctx.ecritures().await;
    let (s, b) = ctx.accepter(vec![v]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"], json!([]));
    assert_eq!(b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE");
    assert_eq!(
        b["failed"][0]["details"]["reason"],
        "transaction_not_pending"
    );
    assert_eq!(ctx.ecritures().await, avant, "rien d'écrit");
}

/// Règle le compte de versement du solde Stripe qui porte le compte du plan
/// `ledger` (SQL direct : le garde de l'API refuserait certains réglages que
/// les tests de refus veulent pourtant atteindre).
async fn regler_versement(ctx: &Ctx, ledger: i64, destination: i64) {
    sqlx::query("UPDATE bank_accounts SET payout_bank_account_id = ? WHERE journal_account_id = ?")
        .bind(destination)
        .bind(ledger)
        .execute(&ctx.pool)
        .await
        .unwrap();
}

/// Un compte bancaire ordinaire de plus, au compte du plan donné (ou sans).
async fn banque_de_plus(ctx: &Ctx, nom: &str, iban: &str, ledger: Option<i64>) -> i64 {
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, ?, ?, FALSE, ?)",
    )
    .bind(ctx.seeded.company_id)
    .bind(nom)
    .bind(iban)
    .bind(ledger)
    .execute(&ctx.pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Un `payout` forcé par un client d'API est rejugé à l'acceptation (revue P1
/// M1) : un retour de versement et un versement de montant positif sont
/// périmés sous leur motif, sans écriture — le sens de l'écriture serait sinon
/// inversé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn versement_a_retour_ou_positif_refuse_a_l_acceptation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Le solde USD n'a pas de compte de versement : on lui en règle un, en
    // USD aussi, pour que seul le rejugement du versement puisse refuser (une
    // destination CHF ferait refuser par `counterparty_wrong_currency`).
    let usd_destination = compte(&pool, ctx.seeded.company_id, "1033", "Asset", Some("USD")).await;
    let ba_usd = banque_de_plus(
        &ctx,
        "Banque USD",
        "CH4431999123000889012",
        Some(usd_destination),
    )
    .await;
    regler_versement(&ctx, ctx.usd, ba_usd).await;
    // Un versement positif (frais seuls : − Σ net > 0) en CHF.
    let entete = std::str::from_utf8(FIXTURE)
        .unwrap()
        .lines()
        .next()
        .unwrap()
        .to_string();
    let positif = format!(
        "{entete}\npo_POS,2020-02-20 00:00:00,txn_POS1,2020-02-18 10:00:00,2020-02-20 00:00:00,chf,-5.00,0.00,-5.00,fee,Frais fictifs seuls,,\n"
    );
    let (s, b) = ctx.importer(positif.as_bytes()).await;
    assert_eq!(s, 201, "{b}");
    let avant = ctx.ecritures().await;
    for (versement, motif) in [
        ("po_TEST0004", "payout_reversal"),
        ("po_POS", "payout_not_negative"),
    ] {
        let id = ctx.tx(versement).await;
        let (s, b) = ctx
            .accepter(vec![json!({ "type": "payout", "bankTransactionId": id })])
            .await;
        assert_eq!(s, 200, "{versement} : {b}");
        assert_eq!(b["accepted"], json!([]), "{versement} : {b}");
        assert_eq!(
            b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE",
            "{versement} : {b}"
        );
        assert_eq!(b["failed"][0]["details"]["reason"], motif, "{b}");
        assert_eq!(ctx.etat(id).await, ("pending".into(), None), "{versement}");
    }
    assert_eq!(ctx.ecritures().await, avant, "rien d'écrit");
}

/// Les refus propres à la destination d'un versement (revue P1 L3) : compte
/// sans compte du plan (`BANK_ACCOUNT_NOT_CONFIGURED`), compte du plan d'un
/// autre type ou d'une autre devise que le solde (`VALIDATION_ERROR`) ; rien
/// d'écrit, le versement reste à traiter.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn destination_de_versement_sans_plan_de_mauvais_type_ou_de_mauvaise_devise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let po = ctx.tx("po_TEST0001").await; // CHF
    let avant = ctx.ecritures().await;

    let sans_plan = banque_de_plus(&ctx, "Sans plan", "CH5604835012345678009", None).await;
    regler_versement(&ctx, ctx.chf, sans_plan).await;
    let (s, b) = ctx
        .accepter(vec![json!({ "type": "payout", "bankTransactionId": po })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "BANK_ACCOUNT_NOT_CONFIGURED",
        "{b}"
    );
    assert_eq!(b["failed"][0]["details"]["bankAccountId"], sans_plan);

    let mauvais_type =
        banque_de_plus(&ctx, "Charge", "CH4431999123000889012", Some(ctx.frais_chf)).await;
    regler_versement(&ctx, ctx.chf, mauvais_type).await;
    let (s, b) = ctx
        .accepter(vec![json!({ "type": "payout", "bankTransactionId": po })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "VALIDATION_ERROR", "{b}");
    assert_eq!(
        b["failed"][0]["details"]["reason"], "counterparty_wrong_type",
        "{b}"
    );

    // Un solde CHF versé sur un compte du plan en EUR.
    regler_versement(&ctx, ctx.chf, ctx.ba_wise).await;
    let (s, b) = ctx
        .accepter(vec![json!({ "type": "payout", "bankTransactionId": po })])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "VALIDATION_ERROR", "{b}");
    assert_eq!(
        b["failed"][0]["details"]["reason"], "counterparty_wrong_currency",
        "{b}"
    );
    assert_eq!(ctx.ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.etat(po).await, ("pending".into(), None));
}

/// La garde `locked_accounts` d'`accept_payout` (revue P1 L1) : une destination
/// que le lot n'a pas verrouillée périme la proposition
/// (`provider_settings_changed`) ; verrouillée, le versement s'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn destination_non_verrouillee_perime_le_versement(pool: MySqlPool) {
    use kesh_api::routes::provider_accept::{AcceptContext, EngineProposal, Refusal};
    use kesh_api::routes::stripe::{OwnProposal, StripeProposal};
    let ctx = monter(&pool).await;
    let c = ctx.seeded.company_id;
    let po = ctx.tx("po_TEST0002").await;
    let proposition = StripeProposal::Own(OwnProposal::Payout {
        bank_transaction_id: po,
        payout_bank_account_id: None,
    });
    for (verrouilles, attendu_perime) in [(vec![], true), (vec![ctx.ba_wise], false)] {
        let mut tx = pool.begin().await.unwrap();
        let balances = kesh_db::repositories::bank_accounts::list_provider_balances(
            &mut *tx,
            c,
            kesh_reconciliation::stripe::SPEC.provider,
            STRIPE,
        )
        .await
        .unwrap();
        let contexte = AcceptContext {
            spec: &kesh_reconciliation::stripe::SPEC,
            company_id: c,
            acteur: kesh_db::entities::audit_log::Acteur {
                user_id: ctx.seeded.admin_user_id,
                api_key_id: None,
            },
            account: STRIPE,
            balances: balances.into_iter().map(|b| (b.account.id, b)).collect(),
            locked_entries: Default::default(),
            locked_accounts: &verrouilles,
        };
        let r = proposition.accept(&mut tx, &contexte).await;
        tx.rollback().await.unwrap();
        match (r, attendu_perime) {
            (Err(Refusal::Failed(f)), true) => {
                assert_eq!(f.error_code, "PROVIDER_PROPOSAL_STALE");
                assert_eq!(f.details.unwrap()["reason"], "provider_settings_changed");
            }
            (Ok(a), false) => assert_eq!(a.kind, "payout"),
            (r, _) => panic!("verrouillés {verrouilles:?} : {r:?}"),
        }
    }
}

/// La vente rattachée entre la revue et l'acceptation : le rattachement de la
/// charge est périmé ; le reste du lot est accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn vente_rattachee_entre_la_revue_et_l_acceptation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ventes = ctx.ventes().await;
    let revue = ctx.revue().await;
    let charge = ctx.tx("txn_TEST0001").await;
    let lien = proposition(&revue, charge);
    let frais = proposition(&revue, ctx.tx("txn_TEST0001:FEE").await);
    sqlx::query(
        "UPDATE bank_transactions SET matched_entry_id = ?, status = 'reconciled' WHERE id = ?",
    )
    .bind(ventes["1"])
    .bind(ctx.tx("txn_TEST0002").await)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = ctx.accepter(vec![lien, frais]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    assert_eq!(b["accepted"][0]["type"], "fee");
    assert_eq!(b["failed"][0]["bankTransactionId"], charge);
    assert_eq!(b["failed"][0]["errorCode"], "PROVIDER_PROPOSAL_STALE");
    assert_eq!(b["failed"][0]["details"]["reason"], "entry_linked");
    assert_eq!(ctx.etat(charge).await, ("pending".into(), None));
}

/// Le compte de versement changé, archivé ou clos entre la revue et
/// l'acceptation : la proposition de la revue est périmée (changé, archivé) ou
/// refusée `BANK_ACCOUNT_CLOSED` (clos) ; rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_de_versement_change_archive_ou_clos_depuis_la_revue(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let revue = ctx.revue().await;
    let po = ctx.tx("po_TEST0002").await;
    let v = proposition(&revue, po);
    let avant = ctx.ecritures().await;
    let raison = |b: &Value| {
        (
            b["failed"][0]["errorCode"]
                .as_str()
                .unwrap_or("")
                .to_string(),
            b["failed"][0]["details"]["reason"]
                .as_str()
                .unwrap_or("")
                .to_string(),
        )
    };

    // Changé : un autre compte Wise EUR.
    let autre = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, is_primary, journal_account_id, \
         provider, provider_account) VALUES (?, 'Wise 2', FALSE, ?, 'wise', 'Wise fictif 2')",
    )
    .bind(ctx.seeded.company_id)
    .bind(ctx.wise_eur_ledger)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let regler = |dest: i64| {
        sqlx::query("UPDATE bank_accounts SET payout_bank_account_id = ? WHERE id = ?")
            .bind(dest)
            .bind(ctx.ba_eur)
            .execute(&pool)
    };
    regler(autre).await.unwrap();
    let (s, b) = ctx.accepter(vec![v.clone()]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        raison(&b),
        (
            "PROVIDER_PROPOSAL_STALE".into(),
            "provider_settings_changed".into()
        ),
        "{b}"
    );
    regler(ctx.ba_wise).await.unwrap();

    // Archivé.
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(ctx.ba_wise)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.accepter(vec![v.clone()]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        raison(&b),
        (
            "PROVIDER_PROPOSAL_STALE".into(),
            "payout_account_archived".into()
        ),
        "{b}"
    );
    // La revue ne le propose plus : compte de versement inutilisable.
    let r = ctx.revue().await;
    assert!(
        codes(&r["toReview"]).contains(&paire("payout_account_missing", Some("po_TEST0002"))),
        "{r}"
    );
    sqlx::query("UPDATE bank_accounts SET archived = FALSE, closed_on = '2020-12-31' WHERE id = ?")
        .bind(ctx.ba_wise)
        .execute(&pool)
        .await
        .unwrap();

    // Clos.
    let (s, b) = ctx.accepter(vec![v]).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "BANK_ACCOUNT_CLOSED", "{b}");
    assert_eq!(b["failed"][0]["details"]["bankAccountId"], ctx.ba_wise);
    assert_eq!(ctx.ecritures().await, avant, "rien d'écrit");
    assert_eq!(ctx.etat(po).await, ("pending".into(), None));
}

/// Cours absent : les frais et le versement EUR de février en `failed[]`
/// (`RATE_UNAVAILABLE`), le reste (CHF) accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cours_absent_dans_failed_le_reste_accepte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query("DELETE FROM exchange_rates WHERE currency = 'EUR' AND rate_date = '2020-02-01'")
        .execute(&pool)
        .await
        .unwrap();
    let revue = ctx.revue().await;
    let po2 = ctx.tx("po_TEST0002").await;
    let frais_chf = ctx.tx("txn_TEST0001:FEE").await;
    let (s, b) = ctx
        .accepter(vec![
            proposition(&revue, po2),
            proposition(&revue, frais_chf),
        ])
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["accepted"][0]["bankTransactionIds"],
        json!([frais_chf]),
        "{b}"
    );
    assert_eq!(
        b["failed"],
        json!([{
            "bankTransactionId": po2,
            "errorCode": "RATE_UNAVAILABLE",
            "details": { "currency": "EUR", "month": "2020-02-01" },
        }])
    );
    assert_eq!(ctx.etat(po2).await.0, "pending");
}

/// Refus globaux (400, 404) et types non admis pour Stripe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_globaux_et_types_non_admis(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let fy = ctx.seeded.fiscal_year_id;
    let (s, b) = ctx
        .req(
            reqwest::Method::GET,
            &format!("/api/v1/stripe/review?account=Inconnu&fiscalYearId={fy}"),
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
            "/api/v1/stripe/review?account=Stripe%20boutique&fiscalYearId=999999",
            None,
            &ctx.jwt,
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (404, Some("FISCAL_YEAR_NOT_FOUND"))
    );
    let frais = ctx.tx("txn_TEST0001:FEE").await;
    let corps = |p: Value| json!({ "account": STRIPE, "proposals": p });
    for p in [
        json!([]),
        json!([{ "type": "rule", "bankTransactionId": frais, "ruleId": 1, "counterpartyAccountId": ctx.frais_chf }]),
        json!([{ "type": "withdrawal", "bankTransactionId": frais, "counterpartyAccountId": ctx.banque }]),
        json!([{ "type": "payout", "bankTransactionId": 0 }]),
        json!([
            { "type": "fee", "bankTransactionId": frais, "feeAccountId": ctx.frais_chf },
            { "type": "payout", "bankTransactionId": frais },
        ]),
        json!([{ "type": "inconnu", "bankTransactionId": frais }]),
    ] {
        let (s, b) = ctx.post("/api/v1/stripe/accept", corps(p.clone())).await;
        assert_eq!(s, 400, "{p} : {b}");
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR");
    }
    let (s, b) = ctx
        .post(
            "/api/v1/stripe/accept",
            json!({ "account": "Inconnu", "proposals": [{ "type": "payout", "bankTransactionId": frais }] }),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (404, Some("PROVIDER_ACCOUNT_NOT_FOUND"))
    );
    // Un frais proposé en versement : genre rejugé, périmé.
    let (s, b) = ctx
        .accepter(vec![
            json!({ "type": "payout", "bankTransactionId": frais }),
        ])
        .await;
    assert_eq!(s, 200);
    assert_eq!(b["failed"][0]["details"]["reason"], "kind_changed", "{b}");
}

/// Clé d'API `read` : admise à la revue, refusée à l'acceptation (403) ; une
/// clé `read-write` accepte, et la trace porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cles_d_api(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let cle = |scope: &'static str| {
        let ctx = &ctx;
        async move {
            let (s, b) = ctx
                .post(
                    "/api/v1/settings/api-keys",
                    json!({ "name": format!("clé {scope}"), "scope": scope }),
                )
                .await;
            assert_eq!(s, 201, "{b}");
            b["key"].as_str().unwrap().to_string()
        }
    };
    let lecture = cle("read").await;
    let (s, _) = ctx.revue_avec(&lecture).await;
    assert_eq!(s, 200);
    let po = ctx.tx("po_TEST0001").await;
    let corps =
        json!({ "account": STRIPE, "proposals": [{ "type": "payout", "bankTransactionId": po }] });
    let (s, _) = ctx
        .req(
            reqwest::Method::POST,
            "/api/v1/stripe/accept",
            Some(corps.clone()),
            &lecture,
        )
        .await;
    assert_eq!(s, 403);
    assert_eq!(ctx.etat(po).await.0, "pending");
    let ecriture = cle("read-write").await;
    let (s, b) = ctx
        .req(
            reqwest::Method::POST,
            "/api/v1/stripe/accept",
            Some(corps),
            &ecriture,
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
// AC-6 — annulation d'un versement avant sa réception
// ============================================================

/// Le geste existant (`…/cancel`) contre-passe l'écriture d'un versement dont
/// la réception n'est pas rattachée ; le versement redevient proposable.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annulation_d_un_versement_avant_reception(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let revue = ctx.revue().await;
    let po = ctx.tx("po_TEST0001").await;
    let (s, b) = ctx.accepter(vec![proposition(&revue, po)]).await;
    assert_eq!((s, b["failed"].clone()), (200, json!([])), "{b}");
    let ecriture = ctx.etat(po).await.1.unwrap();
    let (s, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{po}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.etat(po).await, ("pending".into(), None));
    assert_eq!(
        ctx.count(&format!(
            "SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = {ecriture}"
        ))
        .await,
        1,
        "l'écriture du versement est contre-passée"
    );
    let seconde = ctx.revue().await;
    assert!(
        genres(&seconde).contains(&("payout".to_string(), "po_TEST0001".to_string())),
        "{seconde}"
    );
}

// ============================================================
// AC-6 — interblocage dans une proposition : rejeu, jamais dans failed[]
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
/// tient la ligne du versement ; l'acceptation, qui a écrit l'écriture (et
/// tient l'exercice), attend la ligne pour la rapprocher ; le test demande
/// l'exercice → InnoDB annule l'acceptation, qui doit être **rejouée** (témoin
/// `stripe::accept`) et réussir, sans ligne dans `failed[]`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn interblocage_rejoue_versement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let po = ctx.tx("po_TEST0002").await;
    let avant = ctx.ecritures().await;
    let capture = CaptureRejeu::installer();
    let mut lourde = transaction_lourde(&ctx.pool).await;
    sqlx::query("SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE")
        .bind(po)
        .fetch_all(&mut *lourde)
        .await
        .unwrap();
    let app = ctx.app.clone();
    let jwt = ctx.jwt.clone();
    let route = tokio::spawn(async move {
        let resp = app
            .client
            .post(format!("{}/api/v1/stripe/accept", app.base_url))
            .bearer_auth(jwt)
            .json(&json!({ "account": STRIPE, "proposals": [{ "type": "payout", "bankTransactionId": po }] }))
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
        "l'acceptation devait attendre la ligne du versement"
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
    capture.exiger_un_rejeu("stripe::accept");
    assert_eq!(ctx.ecritures().await, avant + 1, "une seule écriture");
    assert_eq!(ctx.etat(po).await.0, "reconciled");
}

// ============================================================
// Volume — 600 transactions et 600 écritures de vente
// ============================================================

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
        let charge = format!("ch_PERF{i:08}");
        sqlx::query(
            "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
             amount, currency, details, transaction_id, reference, end_to_end_id, provider_type) \
             VALUES (?, ?, ?, '2020-06-01', 10.00, 'EUR', '', ?, ?, 'po_PERF', 'charge')",
        )
        .bind(c)
        .bind(import)
        .bind(ctx.ba_eur)
        .bind(format!("txn_PERF{i:08}"))
        .bind(&charge)
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
        .bind(format!("Vente boutique · {charge}"))
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
    eprintln!("revue Stripe de 600 transactions et 600 écritures : {duree:?}");
    let liens = revue["proposals"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|p| p["type"] == "link_entry")
        .count();
    assert_eq!(liens, 600, "les 600 ventes rattachables");
    assert!(duree < std::time::Duration::from_secs(10), "{duree:?}");
}
