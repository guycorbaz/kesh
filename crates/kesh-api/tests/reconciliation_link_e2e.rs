//! Tests E2E HTTP — rattacher une transaction bancaire au règlement déjà saisi
//! depuis la fiche de la facture (Story 29-8, #670).
//!
//! Couvre l'AC3 et la table T1 à T20 de l'AC8 de la fiche
//! `29-8-transaction-reglee-ailleurs.md`, plus T17-bis (revue de code P1, E1) :
//! `GET /api/v1/reconciliation/transactions/{id}/link-candidates` et
//! `POST /api/v1/reconciliation/transactions/{id}/link`.
//!
//! ⚠️ **Montage** (choix C-29-8-30) : comme `bank_accounts_closed_e2e.rs`
//! (C-28-4-26), la facture est validée et réglée **par les routes** (le
//! règlement par `POST /invoices/{id}/settlements`, c'est-à-dire depuis la
//! fiche) ; la transaction bancaire est posée par **la fonction du dépôt que la
//! route d'import appelle** (`bank_imports::create_with_transactions`), et non
//! par un fichier CAMT : le geste éprouvé est le rattachement, pas l'analyse
//! d'un relevé. Les états qui contredisent un invariant (compte clos portant
//! une transaction en attente, exercice clos, compte archivé) sont **forgés**
//! par SQL, et chaque test le dit.
//!
//! Pré-requis : MariaDB démarré.
#![allow(clippy::too_many_arguments)]

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::{
    BankImportSourceFormat, Language, NewBankAccount, NewBankImport, NewBankTransaction,
    NewCompany, NewUser, OrgType, Role,
};
use kesh_db::repositories::{bank_accounts, bank_imports, companies, contacts, users};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

const IBAN_A: &str = "CH4431999123000889012";
const IBAN_B: &str = "CH9300762011623852957";
const IBAN_C: &str = "CH5604835012345678009";

// ============================================================
// Serveur et jetons
// ============================================================

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

fn d(s: &str) -> NaiveDate {
    NaiveDate::from_str(s).unwrap()
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

// ============================================================
// Contexte et montages
// ============================================================

/// Une société de `seed_accounting_company` (exercice 2020-2030 ouvert),
/// onboarding terminé, arrondi à 5 centimes désactivé, un compte 1020
/// *Banque* auquel les comptes bancaires sont liés ; le serveur et un JWT
/// `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    ledger: i64,
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let ledger = compte_du_grand_livre(pool, seeded.company_id, "1020", "Banque").await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    Ctx {
        pool: pool.clone(),
        app,
        jwt,
        seeded,
        ledger,
    }
}

async fn compte_du_grand_livre(pool: &MySqlPool, company: i64, numero: &str, nom: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, ?, ?, 'Asset', 1, 1) RETURNING id",
    )
    .bind(company)
    .bind(numero)
    .bind(nom)
    .fetch_one(pool)
    .await
    .unwrap()
}

impl Ctx {
    fn company_id(&self) -> i64 {
        self.seeded.company_id
    }

    async fn req_as(
        &self,
        jeton: &str,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, self.app.url(chemin))
            .bearer_auth(jeton);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req_as(&self.jwt, reqwest::Method::POST, chemin, Some(corps))
            .await
    }

    async fn get(&self, chemin: &str) -> (u16, Value) {
        self.req_as(&self.jwt, reqwest::Method::GET, chemin, None)
            .await
    }

    /// Un compte bancaire (non principal sauf `principal`), lié à `ledger`.
    async fn compte_lie(&self, iban: &str, principal: bool, ledger: i64) -> i64 {
        let id = bank_accounts::create(
            &self.pool,
            NewBankAccount {
                company_id: self.company_id(),
                bank_name: "UBS".into(),
                iban: iban.into(),
                qr_iban: None,
                is_primary: principal,
            },
        )
        .await
        .unwrap()
        .id;
        sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
            .bind(ledger)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    /// Un compte bancaire **non principal** lié au 1020.
    async fn compte(&self, iban: &str) -> i64 {
        self.compte_lie(iban, false, self.ledger).await
    }

    async fn version_compte(&self, id: i64) -> i64 {
        sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn contact(&self, nom: &str) -> i64 {
        contacts::create(
            &self.pool,
            self.seeded.admin_user_id,
            NewContact {
                company_id: self.company_id(),
                contact_type: ContactType::Entreprise,
                name: nom.into(),
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
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

    /// Une facture client validée par les routes (100.00 HT, 8,1 % → 108.10),
    /// datée de `date`.
    async fn facture(&self, date: &str) -> i64 {
        let client = self.contact("Dubarde SàRL").await;
        let (s, b) = self
            .post(
                "/api/v1/invoices",
                json!({
                    "contactId": client,
                    "date": date,
                    "lines": [
                        {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
                    ]
                }),
            )
            .await;
        assert_eq!(s, 201, "facture : {b}");
        let id = b["id"].as_i64().unwrap();
        let (s, b) = self
            .req_as(
                &self.jwt,
                reqwest::Method::POST,
                &format!("/api/v1/invoices/{id}/validate"),
                None,
            )
            .await;
        assert_eq!(s, 200, "validation : {b}");
        id
    }

    /// Règle `facture` par virement sur `bank` **depuis la fiche** :
    /// `(identifiant du règlement, écriture du règlement)`.
    async fn regler(&self, facture: i64, bank: i64, montant: &str, le: &str) -> (i64, i64) {
        let (s, b) = self
            .post(
                &format!("/api/v1/invoices/{facture}/settlements"),
                json!({
                    "settlementType": "bank_transfer",
                    "bankAccountId": bank,
                    "amount": montant,
                    "settledOn": le,
                }),
            )
            .await;
        assert_eq!(s, 200, "règlement : {b}");
        let entry = b["journalEntryId"].as_i64().expect("journalEntryId");
        let sid: i64 =
            sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
                .bind(entry)
                .fetch_one(&self.pool)
                .await
                .unwrap();
        (sid, entry)
    }

    /// Une transaction `pending` de `montant` en `devise`, datée `le` (valeur
    /// et comptabilisation), par le dépôt que la route d'import appelle.
    async fn transaction_en(&self, bank: i64, montant: &str, devise: &str, le: &str) -> i64 {
        let jour = d(le);
        let cle = format!(
            "{bank}-{montant}-{devise}-{le}-{}",
            Utc::now().timestamp_nanos_opt().unwrap()
        );
        let mut tx = self.pool.begin().await.unwrap();
        let (_, txs) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.company_id(),
                bank_account_id: bank,
                filename: "releve.xml".into(),
                file_hash: hash64(&cle),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some(format!("LINK-{cle}")),
                period_from: jour,
                period_to: jour,
                opening_balance: None,
                closing_balance: None,
                transaction_count: 1,
                imported_by_user_id: self.seeded.admin_user_id,
            },
            vec![NewBankTransaction {
                company_id: self.company_id(),
                bank_account_id: bank,
                booking_date: jour,
                value_date: Some(jour),
                amount: dec(montant),
                currency: devise.into(),
                reference: None,
                details: "Virement client".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Dubarde SàRL".into()),
            }],
        )
        .await
        .expect("import");
        tx.commit().await.unwrap();
        txs[0].id
    }

    async fn transaction(&self, bank: i64, montant: &str, le: &str) -> i64 {
        self.transaction_en(bank, montant, "CHF", le).await
    }

    async fn candidats(&self, tx: i64) -> (u16, Value) {
        self.get(&format!(
            "/api/v1/reconciliation/transactions/{tx}/link-candidates"
        ))
        .await
    }

    /// Les identifiants de règlement des candidats, dans l'ordre rendu.
    async fn ids_candidats(&self, tx: i64) -> Vec<i64> {
        let (s, b) = self.candidats(tx).await;
        assert_eq!(s, 200, "candidats : {b}");
        b["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["settlementId"].as_i64().unwrap())
            .collect()
    }

    async fn rattacher(&self, tx: i64, sid: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/reconciliation/transactions/{tx}/link"),
            json!({ "settlementId": sid }),
        )
        .await
    }

    async fn etat(&self, tx: i64) -> (String, Option<i64>) {
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn compter(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql)
            .bind(self.company_id())
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("{sql} : {e}"))
    }

    async fn ecritures(&self) -> i64 {
        self.compter("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .await
    }

    async fn reglements(&self) -> i64 {
        self.compter("SELECT COUNT(*) FROM invoice_settlements WHERE company_id = ?")
            .await
    }

    /// `(reste dû, paid_at)` de la facture, lus par la route de détail.
    async fn reste_et_paye(&self, facture: i64) -> (String, Value) {
        let (s, b) = self.get(&format!("/api/v1/invoices/{facture}")).await;
        assert_eq!(s, 200, "facture : {b}");
        (
            b["amountDue"].as_str().unwrap_or_default().to_string(),
            b["paidAt"].clone(),
        )
    }

    async fn annuler_reglement(&self, facture: i64, sid: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/invoices/{facture}/settlements/{sid}/cancel"),
            json!({}),
        )
        .await
    }

    async fn devalider(&self, facture: i64) -> (u16, Value) {
        let (s, b) = self.get(&format!("/api/v1/invoices/{facture}")).await;
        assert_eq!(s, 200, "{b}");
        let version = b["version"].as_i64().unwrap();
        self.post(
            &format!("/api/v1/invoices/{facture}/unvalidate"),
            json!({ "version": version }),
        )
        .await
    }

    async fn clore_sql(&self, bank: i64) {
        sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
            .bind(bank)
            .execute(&self.pool)
            .await
            .unwrap();
    }
}

fn hash64(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    format!("{:0>64x}", h.finish())
}

fn assert_refus(status: u16, corps: &Value, attendu: u16, code: &str) {
    assert_eq!(status, attendu, "{corps}");
    assert_eq!(corps["error"]["code"], code, "{corps}");
}

fn assert_non_rattachable(status: u16, corps: &Value) {
    assert_refus(status, corps, 409, "RECONCILIATION_SETTLEMENT_NOT_LINKABLE");
}

// ============================================================
// T1 — le défaut de #670
// ============================================================

/// T1 — règlement saisi depuis la fiche, au montant, sur le compte de la
/// transaction ; compte **non principal** dont c'est la seule transaction en
/// attente. Avant : la clôture du compte est refusée. Le rattachement passe la
/// transaction rapprochée de l'écriture du règlement, sans aucune écriture, et
/// la clôture réussit — le défaut de #670.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t1_le_rattachement_debloque_la_cloture_du_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, entry) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-18").await;

    let v = ctx.version_compte(bank).await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{bank}/close"),
            json!({ "version": v, "closedOn": "2026-07-31" }),
        )
        .await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS");

    let (s, b) = ctx.candidats(tx).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["bankTransactionId"], tx);
    let cands = b["candidates"].as_array().unwrap();
    assert_eq!(cands.len(), 1, "{b}");
    let c = &cands[0];
    assert_eq!(c["settlementId"], sid);
    assert_eq!(c["invoiceId"], facture);
    assert_eq!(c["journalEntryId"], entry);
    assert_eq!(c["amount"], "108.10");
    assert_eq!(c["gap"], "0.00");
    assert_eq!(c["daysApart"], 2);
    assert_eq!(c["settledOn"], "2026-06-20");
    assert_eq!(c["contactName"], "Dubarde SàRL");
    assert!(c["invoiceNumber"].is_string(), "{c}");

    let ecritures = ctx.ecritures().await;
    let reglements = ctx.reglements().await;
    let avant = ctx.reste_et_paye(facture).await;

    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["bankTransactionId"], tx);
    assert_eq!(b["journalEntryId"], entry);
    assert_eq!(b["invoiceId"], facture);
    assert_eq!(b["settlementId"], sid);
    assert_eq!(ctx.etat(tx).await, ("reconciled".to_string(), Some(entry)));
    assert_eq!(ctx.ecritures().await, ecritures, "aucune écriture");
    assert_eq!(ctx.reglements().await, reglements, "aucun règlement");
    assert_eq!(ctx.reste_et_paye(facture).await, avant, "facture inchangée");

    let details = common::audit_details(&pool, "bank_transaction", tx, "reconciliation.linked")
        .await
        .expect("trace reconciliation.linked");
    assert_eq!(details["bank_transaction_id"], tx);
    assert_eq!(details["settlement_id"], sid);
    assert_eq!(details["invoice_id"], facture);
    assert_eq!(details["journal_entry_id"], entry);
    assert_eq!(details["amount"], "108.10");
    assert_eq!(details["settlement_amount"], "108.10");
    assert_eq!(details["gap"], "0.00");
    assert_eq!(details["was_previously_rejected"], false);

    let (s, b) = ctx
        .get(&format!("/api/v1/reconciliation/transactions/{tx}"))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["kind"], "invoice_settlement");
    assert_eq!(b["invoiceId"], facture);
    assert_eq!(b["cancellable"], true);

    let v = ctx.version_compte(bank).await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{bank}/close"),
            json!({ "version": v, "closedOn": "2026-07-31" }),
        )
        .await;
    assert_eq!(s, 200, "la clôture réussit après le rattachement : {b}");
}

/// T2 — une transaction rejetée est candidate et se rattache ; le marqueur de
/// rejet est remis à zéro et la trace le dit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t2_une_transaction_rejetee_se_rattache(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-18").await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/reject",
            json!({ "bankAccountId": bank, "bankTransactionIds": [tx] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["rejected"].as_array().unwrap().len(), 1, "{b}");

    assert_eq!(ctx.ids_candidats(tx).await, vec![sid]);
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_eq!(s, 200, "{b}");
    let rejete: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT auto_match_rejected_at FROM bank_transactions WHERE id = ?")
            .bind(tx)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rejete, None, "marqueur de rejet remis à zéro");
    let details = common::audit_details(&pool, "bank_transaction", tx, "reconciliation.linked")
        .await
        .unwrap();
    assert_eq!(details["was_previously_rejected"], true);
}

/// T3 — le cas de la 29-6 : la transaction dépasse le règlement de 3 centimes.
/// Candidat à écart « 0.03 », à deux décimales, à l'API comme à l'audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t3_un_ecart_de_trois_centimes_se_rattache_et_se_dit(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "100.00", "2026-06-20").await;
    let tx = ctx.transaction(bank, "100.03", "2026-06-20").await;
    let (s, b) = ctx.candidats(tx).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["candidates"][0]["settlementId"], sid);
    assert_eq!(b["candidates"][0]["gap"], "0.03");
    assert_eq!(b["candidates"][0]["amount"], "100.00");
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_eq!(s, 200, "{b}");
    let details = common::audit_details(&pool, "bank_transaction", tx, "reconciliation.linked")
        .await
        .unwrap();
    assert_eq!(details["gap"], "0.03");
    assert_eq!(details["amount"], "100.03");
    assert_eq!(details["settlement_amount"], "100.00");
}

/// T4 — la borne haute de l'écart : 0.05 est candidat, 0.06 ne l'est pas et
/// son rattachement est refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t4_la_borne_haute_de_l_ecart(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let f1 = ctx.facture("2026-06-01").await;
    let f2 = ctx.facture("2026-06-01").await;
    let (s5, _) = ctx.regler(f1, bank, "100.00", "2026-06-20").await;
    let (s6, _) = ctx.regler(f2, bank, "90.00", "2026-06-20").await;
    let t5 = ctx.transaction(bank, "100.05", "2026-06-20").await;
    let t6 = ctx.transaction(bank, "90.06", "2026-06-20").await;
    assert_eq!(ctx.ids_candidats(t5).await, vec![s5]);
    assert!(ctx.ids_candidats(t6).await.is_empty());
    let (s, b) = ctx.rattacher(t6, s6).await;
    assert_non_rattachable(s, &b);
    assert_eq!(ctx.etat(t6).await, ("pending".to_string(), None));
    let (s, b) = ctx.rattacher(t5, s5).await;
    assert_eq!(s, 200, "{b}");
}

/// T5 — un règlement qui dépasse la transaction (écart négatif) n'est jamais
/// candidat : ce serait de l'argent aux livres que la banque n'a pas reçu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t5_un_reglement_superieur_a_la_transaction_est_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "100.01", "2026-06-20").await;
    let tx = ctx.transaction(bank, "100.00", "2026-06-20").await;
    assert!(ctx.ids_candidats(tx).await.is_empty());
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_non_rattachable(s, &b);
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
}

/// T6 — règlement sur un **autre** compte bancaire lié au **même** compte du
/// grand livre (partage admis) : seul le filtre de compte bancaire l'écarte
/// (tue M1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t6_un_reglement_sur_un_autre_compte_bancaire_est_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank_a = ctx.compte(IBAN_A).await;
    let bank_b = ctx.compte(IBAN_B).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank_b, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank_a, "108.10", "2026-06-20").await;
    assert!(ctx.ids_candidats(tx).await.is_empty());
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_non_rattachable(s, &b);
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
}

/// T7 — un règlement par compte interne n'est pas un virement : jamais
/// candidat (test de comportement ; M2 est équivalente, C-29-8-28).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t7_un_reglement_par_compte_interne_est_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/invoices/{facture}/settlements"),
            json!({
                "settlementType": "internal_account",
                "accountId": ctx.seeded.accounts["1000"],
                "amount": "108.10",
                "settledOn": "2026-06-20",
            }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let entry = b["journalEntryId"].as_i64().unwrap();
    let sid: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
            .bind(entry)
            .fetch_one(&pool)
            .await
            .unwrap();
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;
    assert!(ctx.ids_candidats(tx).await.is_empty());
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_non_rattachable(s, &b);
}

/// T8 — l'écriture d'un règlement déjà rapproché n'est plus candidate :
/// (a) règlement né d'une proposition acceptée ; (b) règlement déjà rattaché à
/// une autre transaction. La seconde transaction reste en attente (tue M3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t8_une_ecriture_deja_rapprochee_n_est_pas_rattachable(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let jour = Utc::now().date_naive().to_string();

    // (a) — un règlement né d'une acceptation.
    let fa = ctx.facture(&jour).await;
    let ta = ctx.transaction(bank, "108.10", &jour).await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/accept",
            json!({
                "bankAccountId": bank,
                "proposals": [{ "type": "invoice", "bankTransactionId": ta, "invoiceId": fa }]
            }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    let entry_a = ctx.etat(ta).await.1.expect("lien de l'acceptation");
    let sa: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE journal_entry_id = ?")
            .bind(entry_a)
            .fetch_one(&pool)
            .await
            .unwrap();
    let ta2 = ctx.transaction(bank, "108.10", &jour).await;
    assert!(!ctx.ids_candidats(ta2).await.contains(&sa));
    let (s, b) = ctx.rattacher(ta2, sa).await;
    assert_non_rattachable(s, &b);
    assert_eq!(ctx.etat(ta2).await, ("pending".to_string(), None));

    // (b) — un règlement rattaché à une première transaction.
    let fb = ctx.facture("2026-06-01").await;
    let (sb, entry_b) = ctx.regler(fb, bank, "50.00", "2026-06-20").await;
    let t1 = ctx.transaction(bank, "50.00", "2026-06-20").await;
    let t2 = ctx.transaction(bank, "50.00", "2026-06-21").await;
    let (s, b) = ctx.rattacher(t1, sb).await;
    assert_eq!(s, 200, "{b}");
    assert!(ctx.ids_candidats(t2).await.is_empty());
    let (s, b) = ctx.rattacher(t2, sb).await;
    assert_non_rattachable(s, &b);
    assert_eq!(ctx.etat(t2).await, ("pending".to_string(), None));
    assert_eq!(
        ctx.etat(t1).await,
        ("reconciled".to_string(), Some(entry_b))
    );
}

/// T9 — une transaction de débit, nulle ou hors CHF : aucun candidat, et le
/// rattachement est refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t9_debit_nul_ou_hors_chf_ne_se_rattachent_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    for (montant, devise) in [("-108.10", "CHF"), ("0.00", "CHF"), ("108.10", "EUR")] {
        let tx = ctx
            .transaction_en(bank, montant, devise, "2026-06-20")
            .await;
        let (s, b) = ctx.candidats(tx).await;
        assert_eq!(s, 200, "{montant} {devise} : {b}");
        assert_eq!(b["candidates"], json!([]), "{montant} {devise}");
        let (s, b) = ctx.rattacher(tx, sid).await;
        assert_non_rattachable(s, &b);
        assert_eq!(b["error"]["details"]["bankTransactionId"], tx);
        assert_eq!(b["error"]["details"]["settlementId"], sid);
        assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
    }
}

/// T10 — une transaction déjà rapprochée : `404
/// RECONCILIATION_TRANSACTION_NOT_PENDING`, au `GET` comme au `POST`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t10_une_transaction_rapprochee_est_refusee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let f1 = ctx.facture("2026-06-01").await;
    let f2 = ctx.facture("2026-06-01").await;
    let (s1, _) = ctx.regler(f1, bank, "108.10", "2026-06-20").await;
    let (s2, _) = ctx.regler(f2, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;
    let (s, b) = ctx.rattacher(tx, s1).await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.candidats(tx).await;
    assert_refus(s, &b, 404, "RECONCILIATION_TRANSACTION_NOT_PENDING");
    let (s, b) = ctx.rattacher(tx, s2).await;
    assert_refus(s, &b, 404, "RECONCILIATION_TRANSACTION_NOT_PENDING");
}

/// T11 — transaction ou règlement d'une autre société : `404`, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t11_une_autre_societe_ne_voit_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;

    // Une seconde société, son compte bancaire lié et sa transaction.
    let autre = companies::create(
        &pool,
        NewCompany {
            name: "Autre SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Autre".into(),
                building: "3".into(),
                postal_code: "1200".into(),
                city: "Genève".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    let autre_user = users::create(
        &pool,
        NewUser {
            username: "autre_admin".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Admin,
            active: true,
            company_id: autre,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let autre_jwt = forge_jwt(autre_user, "Admin", autre);
    let autre_ledger = compte_du_grand_livre(&pool, autre, "1020", "Banque").await;
    let autre_bank = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: autre,
            bank_name: "BCV".into(),
            iban: IBAN_C.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(autre_ledger)
        .bind(autre_bank)
        .execute(&pool)
        .await
        .unwrap();
    let mut t = pool.begin().await.unwrap();
    let (_, autres) = bank_imports::create_with_transactions(
        &mut t,
        NewBankImport {
            company_id: autre,
            bank_account_id: autre_bank,
            filename: "autre.xml".into(),
            file_hash: hash64("autre-societe"),
            source_format: BankImportSourceFormat::Camt053V04,
            statement_id: Some("AUTRE".into()),
            period_from: d("2026-06-20"),
            period_to: d("2026-06-20"),
            opening_balance: None,
            closing_balance: None,
            transaction_count: 1,
            imported_by_user_id: autre_user,
        },
        vec![NewBankTransaction {
            company_id: autre,
            bank_account_id: autre_bank,
            booking_date: d("2026-06-20"),
            value_date: Some(d("2026-06-20")),
            amount: dec("108.10"),
            currency: "CHF".into(),
            reference: None,
            details: "Autre".into(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
        }],
    )
    .await
    .unwrap();
    t.commit().await.unwrap();
    let autre_tx = autres[0].id;

    let traces = common::audit_count(&pool).await;
    // La transaction de la première société, vue de l'autre : 404.
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{tx}/link-candidates"),
            None,
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{tx}/link"),
            Some(json!({ "settlementId": sid })),
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    // Le règlement de la première société, depuis une transaction de l'autre : 404.
    let (s, b) = ctx
        .req_as(
            &autre_jwt,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{autre_tx}/link"),
            Some(json!({ "settlementId": sid })),
        )
        .await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    assert_eq!(common::audit_count(&pool).await, traces, "rien d'écrit");
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
    assert_eq!(ctx.etat(autre_tx).await, ("pending".to_string(), None));
}

/// T12 — rôles et clés : Consultation 403 ; clé en lecture seule refusée au
/// `POST` ; clé `read-write` admise et nommée à l'audit. Et le corps sans
/// `settlementId`, ou à `0`, rend `400 VALIDATION_ERROR`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t12_roles_cles_et_corps(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;

    let lecteur = users::create(
        &pool,
        NewUser {
            username: "lecteur".into(),
            password_hash: hash_password("password123").unwrap(),
            role: Role::Consultation,
            active: true,
            company_id: ctx.company_id(),
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let jwt_lecteur = forge_jwt(lecteur, "Consultation", ctx.company_id());
    let (s, _) = ctx
        .req_as(
            &jwt_lecteur,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{tx}/link-candidates"),
            None,
        )
        .await;
    assert_eq!(s, 403);
    let (s, _) = ctx
        .req_as(
            &jwt_lecteur,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{tx}/link"),
            Some(json!({ "settlementId": sid })),
        )
        .await;
    assert_eq!(s, 403);

    for corps in [
        json!({}),
        json!({ "settlementId": 0 }),
        json!({ "settlementId": -3 }),
    ] {
        let (s, b) = ctx
            .post(
                &format!("/api/v1/reconciliation/transactions/{tx}/link"),
                corps.clone(),
            )
            .await;
        assert_refus(s, &b, 400, "VALIDATION_ERROR");
    }

    let (_, cle_ro) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "ro", "read")
            .await;
    let (s, _) = ctx
        .req_as(
            &cle_ro,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{tx}/link"),
            Some(json!({ "settlementId": sid })),
        )
        .await;
    assert_eq!(s, 403);
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
    // La lecture des candidats, elle, est ouverte à la clé en lecture seule.
    let (s, b) = ctx
        .req_as(
            &cle_ro,
            reqwest::Method::GET,
            &format!("/api/v1/reconciliation/transactions/{tx}/link-candidates"),
            None,
        )
        .await;
    assert_eq!(s, 200, "{b}");

    let (cle_id, cle_rw) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let (s, b) = ctx
        .req_as(
            &cle_rw,
            reqwest::Method::POST,
            &format!("/api/v1/reconciliation/transactions/{tx}/link"),
            Some(json!({ "settlementId": sid })),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let (_, api_key, _, _) =
        common::audit_actor(&pool, "bank_transaction", tx, "reconciliation.linked").await;
    assert_eq!(api_key, Some(cle_id), "la clé est nommée à l'audit");
}

/// T13 (a) — compte clos **avant** le geste (état forgé par SQL : I1 cassé à
/// dessein) : le pré-vol rend `409 BANK_ACCOUNT_CLOSED`, au `GET` et au `POST`.
///
/// Et il passe **avant** le refus « rapprochée » (AC1 ; revue de code P1, A2) :
/// sur un compte clos, toute transaction est rapprochée (I1) ; une transaction
/// **rapprochée** d'un compte clos rend donc `409 BANK_ACCOUNT_CLOSED`, et non
/// `404 RECONCILIATION_TRANSACTION_NOT_PENDING`, au `GET` comme au `POST`. Ce
/// témoin rougit si l'ordre du pré-vol et du statut s'inverse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t13a_compte_clos_avant_le_geste(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;
    // Le témoin : une seconde transaction, rapprochée par rattachement avant
    // la clôture.
    let autre = ctx.facture("2026-06-02").await;
    let (sid_r, entry_r) = ctx.regler(autre, bank, "108.10", "2026-06-21").await;
    let rapprochee = ctx.transaction(bank, "108.10", "2026-06-21").await;
    let (s, b) = ctx.rattacher(rapprochee, sid_r).await;
    assert_eq!(s, 200, "{b}");
    ctx.clore_sql(bank).await;
    let (s, b) = ctx.candidats(tx).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
    let (s, b) = ctx.candidats(rapprochee).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    let (s, b) = ctx.rattacher(rapprochee, sid).await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    assert_eq!(
        ctx.etat(rapprochee).await,
        ("reconciled".to_string(), Some(entry_r))
    );
}

/// T13 (b) — **course forgée** : une connexion tient le verrou nommé du compte,
/// le `POST` est vu bloqué dessus, la clôture est posée par SQL, le verrou
/// relâché : la garde **dans la transaction** (`ensure_open_in_tx`, étape 4.3)
/// rend `409 BANK_ACCOUNT_CLOSED` (tue M19).
///
/// ⚠️ **État forgé** : une clôture réelle refuserait (`412`, I1 —
/// `close_in_tx` compte la `pending` sous le verrou du compte) ; le test exerce
/// la garde, non une course atteignable (fiche, T13, validation P3 F4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t13b_compte_clos_pendant_l_attente_du_verrou(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;

    let db_name: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(&pool)
        .await
        .unwrap();
    let nom = format!("reconcile:{db_name}:{}:{bank}", ctx.company_id());
    let mut tenue = pool.acquire().await.unwrap();
    let pris: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 60)")
        .bind(&nom)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    assert_eq!(pris, Some(1));

    let client = ctx.app.client.clone();
    let url = ctx
        .app
        .url(&format!("/api/v1/reconciliation/transactions/{tx}/link"));
    let jwt = ctx.jwt.clone();
    let requete = tokio::spawn(async move {
        let r = client
            .post(url)
            .bearer_auth(jwt)
            .json(&json!({ "settlementId": sid }))
            .send()
            .await
            .unwrap();
        let s = r.status().as_u16();
        (s, r.json::<Value>().await.unwrap_or(Value::Null))
    });
    let vu = kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["GET_LOCK"], || {
        requete.is_finished()
    })
    .await;
    assert!(vu, "le POST doit attendre le verrou du compte");
    ctx.clore_sql(bank).await;
    let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(&nom)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();
    let (s, b) = requete.await.unwrap();
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
}

/// T14 — le tri : écart croissant, puis jours d'écart (valeur absolue)
/// croissants. A écart 0.00 réglé 40 jours avant ; B écart 0.00 réglé 3 jours
/// après ; C écart 0.02 réglé 1 jour après → B, A, C (tue M11 et M18).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t14_le_tri_des_candidats(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-05-01").await;
    let (a, _) = ctx.regler(facture, bank, "30.00", "2026-05-06").await;
    let (b_, _) = ctx.regler(facture, bank, "30.00", "2026-06-18").await;
    let (c, _) = ctx.regler(facture, bank, "29.98", "2026-06-16").await;
    let tx = ctx.transaction(bank, "30.00", "2026-06-15").await;
    let (s, b) = ctx.candidats(tx).await;
    assert_eq!(s, 200, "{b}");
    let cands = b["candidates"].as_array().unwrap();
    let ordre: Vec<i64> = cands
        .iter()
        .map(|c| c["settlementId"].as_i64().unwrap())
        .collect();
    assert_eq!(ordre, vec![b_, a, c], "{b}");
    let jours: Vec<i64> = cands
        .iter()
        .map(|c| c["daysApart"].as_i64().unwrap())
        .collect();
    assert_eq!(jours, vec![3, 40, 1]);
    assert_eq!(cands[2]["gap"], "0.02");
}

/// L'état K2 (`PENDING_BANK_TRANSACTIONS`) de la préparation de clôture de
/// l'exercice du seed : `(statut, nombre)`.
async fn k2(ctx: &Ctx) -> (String, i64) {
    let (s, b) = ctx
        .get(&format!(
            "/api/v1/fiscal-years/{}/closing-checks",
            ctx.seeded.fiscal_year_id
        ))
        .await;
    assert_eq!(s, 200, "{b}");
    let k2 = b["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "PENDING_BANK_TRANSACTIONS")
        .cloned()
        .expect("K2");
    (
        k2["status"].as_str().unwrap().to_string(),
        k2["count"].as_i64().unwrap(),
    )
}

/// T15 — AC3 : ce que le rattachement débloque et ce qu'il laisse, sur un
/// compte que le test **ne clôt pas**. La facture porte deux règlements par
/// virement sur ce compte : `s`, rattaché, et le témoin `w`, jamais rattaché —
/// il ne diffère de `s` que par le lien. Ordre des assertions écrit par la
/// fiche (validation P2, R4 = F4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t15_ce_que_le_rattachement_debloque_et_laisse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, entry_s) = ctx.regler(facture, bank, "50.00", "2026-06-20").await;
    let (w, _) = ctx.regler(facture, bank, "20.00", "2026-06-21").await;
    let t = ctx.transaction(bank, "50.00", "2026-06-20").await;

    // (1) Préservation, avant : la dévalidation est refusée au motif 1. Elle ne
    // lit pas le lien (son motif 5 vise l'écriture de vente) : elle ne prouve
    // rien du rattachement, et aucune mutation ne la vise.
    let (st, b) = ctx.devalider(facture).await;
    assert_refus(st, &b, 409, "INVOICE_HAS_SETTLEMENTS");
    // (5, avant) K2 de la 29-1a voit la transaction en attente.
    assert_eq!(k2(&ctx).await, ("attention".to_string(), 1));

    // (2) Rattachement.
    let (st, b) = ctx.rattacher(t, s).await;
    assert_eq!(st, 200, "{b}");

    // (3) L'annulation de `s` depuis la fiche est refusée par le socle de la
    // contre-passation ; celle du témoin `w` réussit.
    let (st, b) = ctx.annuler_reglement(facture, s).await;
    assert_refus(st, &b, 409, "MATCHED_BANK_TRANSACTION");
    let (st, b) = ctx.annuler_reglement(facture, w).await;
    assert_eq!(st, 200, "le témoin non rattaché s'annule : {b}");

    // (4) Préservation, après.
    let (st, b) = ctx.devalider(facture).await;
    assert_refus(st, &b, 409, "INVOICE_HAS_SETTLEMENTS");

    // (5) K2 : la transaction rattachée en sort.
    assert_eq!(k2(&ctx).await, ("ok".to_string(), 0));

    // (6) En dernier — il supprime `s` : *Annuler le rapprochement* annule
    // aussi le règlement saisi depuis la fiche.
    let (st, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{t}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
    let reste_s: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE id = ?")
        .bind(s)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(reste_s, 0, "le règlement est retiré");
    let contre: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(entry_s)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(contre, 1, "contre-passation de l'écriture du règlement");
    let (reste, paye) = ctx.reste_et_paye(facture).await;
    assert_eq!(
        dec(&reste),
        dec("108.10"),
        "facture de nouveau à régler en entier"
    );
    assert!(paye.is_null());
}

/// T16 — course réelle entre l'annulation de `s` depuis la fiche et le
/// rattachement de `t` à `s`. Exactement l'un réussit, et l'état final est
/// cohérent. ⚠️ Ne prouve ni l'attente du perdant ni l'ordre d'acquisition
/// (C-29-8-25) : sans chevauchement, la seconde voit la première par simple
/// succession.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t16_course_annulation_et_rattachement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, entry) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let t = ctx.transaction(bank, "108.10", "2026-06-20").await;
    let ((sa, ba), (sl, bl)) = tokio::join!(ctx.annuler_reglement(facture, s), ctx.rattacher(t, s));
    let present: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE id = ?")
        .bind(s)
        .fetch_one(&pool)
        .await
        .unwrap();
    match (sa, sl) {
        (200, _) => {
            assert_refus(sl, &bl, 404, "NOT_FOUND");
            assert_eq!(ctx.etat(t).await, ("pending".to_string(), None));
            assert_eq!(present, 0);
        }
        (_, 200) => {
            assert_refus(sa, &ba, 409, "MATCHED_BANK_TRANSACTION");
            assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(entry)));
            assert_eq!(present, 1);
        }
        _ => panic!("aucun des deux n'a réussi : {sa} {ba} / {sl} {bl}"),
    }
}

/// T17 — course de double rattachement : deux transactions du même compte, un
/// règlement, deux `POST` lancés ensemble. Exactement un `200`, l'autre `409`,
/// une seule transaction pointe l'écriture. ⚠️ Ne prouve pas l'ordre
/// d'acquisition (C-29-8-25) — renvoyé à la revue de code.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t17_course_de_double_rattachement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, entry) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let t1 = ctx.transaction(bank, "108.10", "2026-06-20").await;
    let t2 = ctx.transaction(bank, "108.10", "2026-06-21").await;
    let ((s1, b1), (s2, b2)) = tokio::join!(ctx.rattacher(t1, s), ctx.rattacher(t2, s));
    let (gagnant, perdant, bp) = match (s1, s2) {
        (200, _) => (t1, t2, (s2, b2)),
        (_, 200) => (t2, t1, (s1, b1)),
        _ => panic!("aucun 200 : {s1} {b1} / {s2} {b2}"),
    };
    assert_non_rattachable(bp.0, &bp.1);
    assert_eq!(
        ctx.etat(gagnant).await,
        ("reconciled".to_string(), Some(entry))
    );
    assert_eq!(ctx.etat(perdant).await, ("pending".to_string(), None));
    let liens: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id = ?")
            .bind(entry)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(liens, 1);
}

/// T17-bis — **entrelacement forcé** du double rattachement (revue de code P1,
/// E1) : une connexion `tenue` joue le premier rattachement — elle prend le
/// verrou de ligne du règlement `s` —, le `POST` de `t2` est **vu en attente**
/// sur ce verrou (sonde `PROCESSLIST`, comme T13 (b) : sans le privilège
/// `PROCESS`, qui ne concerne qu'`INNODB_TRX`), puis `tenue` lie `t1` à
/// l'écriture de `s` et commite. Le `POST` doit rendre `409
/// RECONCILIATION_SETTLEMENT_NOT_LINKABLE` : sa vue naît **après** l'attente
/// (étape 4.3), et l'étape 4.4 voit le lien commité. Un seul lien.
///
/// Rougit sous M10 (sans `FOR UPDATE` sur `s`, le `POST` n'attend jamais : la
/// sonde ne le voit pas, et il lie `t2`) et sous M14 (la règle lue avant
/// l'attente fait naître la vue trop tôt : `200`, deux liens). L'invariant
/// « un règlement ne se rattache qu'une fois » n'a pas de contrainte `UNIQUE` :
/// ce test est sa seule garde dynamique.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t17bis_le_second_rattachement_attend_le_verrou_du_reglement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, entry) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let t1 = ctx.transaction(bank, "108.10", "2026-06-20").await;
    let t2 = ctx.transaction(bank, "108.10", "2026-06-21").await;

    let mut tenue = pool.begin().await.unwrap();
    let pris: Option<i64> =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE id = ? FOR UPDATE")
            .bind(s)
            .fetch_optional(&mut *tenue)
            .await
            .unwrap();
    assert_eq!(pris, Some(s));

    let client = ctx.app.client.clone();
    let url = ctx
        .app
        .url(&format!("/api/v1/reconciliation/transactions/{t2}/link"));
    let jwt = ctx.jwt.clone();
    let requete = tokio::spawn(async move {
        let r = client
            .post(url)
            .bearer_auth(jwt)
            .json(&json!({ "settlementId": s }))
            .send()
            .await
            .unwrap();
        let st = r.status().as_u16();
        (st, r.json::<Value>().await.unwrap_or(Value::Null))
    });
    // Les motifs nomment la requête de l'étape 4.2 (a), seule à joindre les
    // deux ; celle de `tenue` est finie (sa connexion est inactive) et la sonde
    // exclut la sienne.
    let vu = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["invoice_settlements", "FOR UPDATE"],
        || requete.is_finished(),
    )
    .await;
    if !vu {
        let (st, b) = requete.await.unwrap();
        panic!("le POST n'a jamais attendu le verrou du règlement : rendu {st} {b}");
    }
    let fait = sqlx::query(
        "UPDATE bank_transactions SET status = 'reconciled', matched_entry_id = ? WHERE id = ?",
    )
    .bind(entry)
    .bind(t1)
    .execute(&mut *tenue)
    .await
    .unwrap();
    assert_eq!(fait.rows_affected(), 1);
    tenue.commit().await.unwrap();

    let (st, b) = requete.await.unwrap();
    assert_non_rattachable(st, &b);
    assert_eq!(ctx.etat(t2).await, ("pending".to_string(), None));
    let liens: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM bank_transactions WHERE matched_entry_id = ?")
            .bind(entry)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(liens, 1);
}

/// T18 — ancien compte lié : le règlement a été passé sur le 1020, puis le
/// compte bancaire a été lié à un autre compte du grand livre. Son mouvement
/// sur le compte lié actuel est nul : il n'est candidat ni pour (a) 0.03 ni
/// pour (b) son propre montant (tue M16).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t18_un_reglement_sur_l_ancien_compte_lie_n_est_pas_candidat(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (s, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let nouveau = compte_du_grand_livre(&pool, ctx.company_id(), "1021", "Banque 2").await;
    let v = ctx.version_compte(bank).await;
    let (st, b) = ctx
        .req_as(
            &ctx.jwt,
            reqwest::Method::PATCH,
            &format!("/api/v1/bank-accounts/{bank}"),
            Some(json!({ "journalAccountId": nouveau, "version": v })),
        )
        .await;
    assert_eq!(st, 200, "{b}");
    for montant in ["0.03", "108.10"] {
        let t = ctx.transaction(bank, montant, "2026-06-20").await;
        assert!(ctx.ids_candidats(t).await.is_empty(), "{montant}");
        let (st, b) = ctx.rattacher(t, s).await;
        assert_non_rattachable(st, &b);
    }
}

/// T19 — exercice clos : un règlement d'un exercice clôturé (statut posé par
/// SQL) se rattache à une transaction de l'exercice suivant ; son *Annuler le
/// rapprochement* est ensuite refusé comme tout rapprochement d'exercice clos.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t19_un_reglement_d_exercice_clos_se_rattache(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // L'exercice du seed (2020-2030) est ramené à 2020-2025 ; 2026-2030 s'ouvre.
    sqlx::query("UPDATE fiscal_years SET end_date = '2025-12-31' WHERE id = ?")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026-2030', '2026-01-01', '2030-12-31', 'Open')",
    )
    .bind(ctx.company_id())
    .execute(&pool)
    .await
    .unwrap();
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2025-12-01").await;
    let (s, entry) = ctx.regler(facture, bank, "108.10", "2025-12-20").await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let t = ctx.transaction(bank, "108.10", "2026-01-05").await;
    assert_eq!(ctx.ids_candidats(t).await, vec![s]);
    let (st, b) = ctx.rattacher(t, s).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(entry)));
    let (st, b) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{t}/cancel"),
            json!({}),
        )
        .await;
    assert_refus(st, &b, 409, "FISCAL_YEAR_CLOSED");
    assert_eq!(ctx.etat(t).await, ("reconciled".to_string(), Some(entry)));
}

/// T20 — compte bancaire archivé (archivage posé par SQL) : la réponse de
/// `post_manual` (`AppError::BankAccountNotFound`), au `GET` et au `POST` ;
/// rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn t20_compte_archive(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    let traces = common::audit_count(&pool).await;
    let (s, b) = ctx.candidats(tx).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
    assert_eq!(common::audit_count(&pool).await, traces);
    assert_eq!(ctx.etat(tx).await, ("pending".to_string(), None));
    // Archivé **et** clos : l'archivage passe avant la clôture (ordre du
    // pré-vol ; revue de code P1, A2).
    ctx.clore_sql(bank).await;
    let (s, b) = ctx.candidats(tx).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
}

/// Le pré-vol rend `412 BANK_ACCOUNT_NOT_CONFIGURED` sur un compte sans compte
/// du grand livre lié, au `GET` comme au `POST` (AC1, ordre des refus).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_non_lie_rend_412(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let bank = ctx.compte(IBAN_A).await;
    let facture = ctx.facture("2026-06-01").await;
    let (sid, _) = ctx.regler(facture, bank, "108.10", "2026-06-20").await;
    let tx = ctx.transaction(bank, "108.10", "2026-06-20").await;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?")
        .bind(bank)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.candidats(tx).await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_NOT_CONFIGURED");
    let (s, b) = ctx.rattacher(tx, sid).await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_NOT_CONFIGURED");
}

/// Une transaction inexistante rend `404 NOT_FOUND` aux deux routes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn transaction_inexistante_rend_404(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx.candidats(i64::MAX).await;
    assert_refus(s, &b, 404, "NOT_FOUND");
    let (s, b) = ctx.rattacher(i64::MAX, 1).await;
    assert_refus(s, &b, 404, "NOT_FOUND");
}
