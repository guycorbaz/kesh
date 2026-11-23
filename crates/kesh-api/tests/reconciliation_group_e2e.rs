//! Tests E2E HTTP — Story 43-2 (#823) : un règlement pour plusieurs factures.
//!
//! - `GET /api/v1/reconciliation/transactions/{id}/group-candidates` (AC5) ;
//! - `POST /api/v1/reconciliation/accept`, type `invoice_group` (AC1, AC2,
//!   AC4, AC6, AC8) ;
//! - la vue et l'annulation d'un groupe (AC7) ; l'accès (AC10).
//!
//! **Montage** : les factures clients sont validées **à la main** (patron de
//! `kesh-db/tests/grouped_settlements.rs` : écriture de vente `D 1100 / C 3000`
//! posée par le dépôt), pour des montants à quatre décimales que la route de
//! création arrondirait ; les transactions sont posées par la fonction du dépôt
//! que la route d'import appelle. Décor synthétique. Pré-requis : MariaDB.
#![allow(clippy::too_many_arguments)]

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction, NewInvoice,
    NewInvoiceLine, NewJournalEntry, NewJournalEntryLine,
};
use kesh_db::repositories::{bank_accounts, bank_imports, invoices, journal_entries};
use kesh_db::test_fixtures::{
    SeededCompany, designate_rounding_account, mark_onboarding_complete, seed_accounting_company,
};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

const IBAN_A: &str = "CH4431999123000889012";
const IBAN_B: &str = "CH9300762011623852957";

// ============================================================
// Serveur et jetons
// ============================================================

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

fn d(s: &str) -> NaiveDate {
    NaiveDate::from_str(s).unwrap()
}

fn dec(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

/// Un nombre de `details_json`, qu'il soit écrit en chaîne ou en nombre.
fn num(v: &Value) -> Decimal {
    match v {
        Value::String(s) => dec(s),
        Value::Number(n) => dec(&n.to_string()),
        other => panic!("nombre attendu, reçu {other}"),
    }
}

fn assert_refus(status: u16, corps: &Value, attendu: u16, code: &str) {
    assert_eq!(status, attendu, "{corps}");
    assert_eq!(corps["error"]["code"], code, "{corps}");
}

// ============================================================
// Contexte et montages
// ============================================================

/// La société du seed (exercice 2020-2030), onboarding terminé ; un compte
/// 1020 *Banque* et un compte bancaire lié ; un compte d'écart 6900 ; le
/// serveur et un JWT `Admin`.
struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
    ledger: i64,
    bank: i64,
    frais: i64,
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    let ledger = compte(pool, seeded.company_id, "1020", "Banque", "Asset").await;
    let frais = compte(pool, seeded.company_id, "6900", "Frais divers", "Expense").await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    let mut ctx = Ctx {
        pool: pool.clone(),
        app,
        jwt,
        seeded,
        ledger,
        bank: 0,
        frais,
    };
    ctx.bank = ctx.compte_bancaire(IBAN_A, ledger).await;
    ctx
}

async fn compte(pool: &MySqlPool, company: i64, numero: &str, nom: &str, genre: &str) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, ?, ?, ?, 1, 1) RETURNING id",
    )
    .bind(company)
    .bind(numero)
    .bind(nom)
    .bind(genre)
    .fetch_one(pool)
    .await
    .unwrap()
}

impl Ctx {
    fn company_id(&self) -> i64 {
        self.seeded.company_id
    }

    fn url(&self, chemin: &str) -> String {
        format!("{}{}", self.app.base_url, chemin)
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
            .request(methode, self.url(chemin))
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

    async fn compte_bancaire(&self, iban: &str, ledger: i64) -> i64 {
        let id = bank_accounts::create(
            &self.pool,
            NewBankAccount {
                company_id: self.company_id(),
                bank_name: "UBS".into(),
                iban: iban.into(),
                qr_iban: None,
                is_primary: false,
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

    async fn contact(&self, nom: &str) -> i64 {
        sqlx::query_scalar(
            "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
             VALUES (?, 'Entreprise', ?, TRUE, TRUE) RETURNING id",
        )
        .bind(self.company_id())
        .bind(nom)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }

    /// Une facture client validée de `montant` (quatre décimales admises),
    /// datée `date`, du client `client` : écriture de vente `D 1100 / C 3000`.
    async fn facture_de(&self, client: &str, numero: &str, montant: &str, date: &str) -> i64 {
        self.facture_projet(client, numero, montant, date, None)
            .await
    }

    async fn facture(&self, numero: &str, montant: &str, date: &str) -> i64 {
        self.facture_de(&format!("Client {numero}"), numero, montant, date)
            .await
    }

    async fn facture_projet(
        &self,
        client: &str,
        numero: &str,
        montant: &str,
        date: &str,
        project_id: Option<i64>,
    ) -> i64 {
        self.facture_sur(
            client,
            numero,
            montant,
            date,
            project_id,
            self.seeded.accounts["1100"],
        )
        .await
    }

    /// Une facture validée dont l'écriture de vente débite `receivable`.
    async fn facture_sur(
        &self,
        client: &str,
        numero: &str,
        montant: &str,
        date: &str,
        project_id: Option<i64>,
        receivable: i64,
    ) -> i64 {
        let s = &self.seeded;
        let contact_id = self.contact(client).await;
        let amount = dec(montant);
        let inv = invoices::create(
            &self.pool,
            s.admin_user_id,
            NewInvoice {
                company_id: s.company_id,
                contact_id,
                date: d(date),
                due_date: Some(d(date)),
                payment_terms: None,
                project_id,
                lines: vec![NewInvoiceLine {
                    description: "Prestation".into(),
                    quantity: Decimal::ONE,
                    unit_price: amount,
                    vat_rate: Decimal::ZERO,
                    revenue_account_id: Some(s.accounts["3000"]),
                }],
            },
        )
        .await
        .expect("facture")
        .0;
        let je = journal_entries::create(
            &self.pool,
            s.fiscal_year_id,
            Acteur::utilisateur(s.admin_user_id),
            NewJournalEntry {
                company_id: s.company_id,
                entry_date: d(date),
                journal: Journal::Ventes,
                description: "Vente".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: receivable,
                        debit: amount,
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: s.accounts["3000"],
                        debit: Decimal::ZERO,
                        credit: amount,
                        project_id: None,
                        foreign: None,
                    },
                ],
            },
        )
        .await
        .expect("écriture de vente");
        sqlx::query(
            "UPDATE invoices SET status = 'validated', journal_entry_id = ?, invoice_number = ? \
             WHERE id = ?",
        )
        .bind(je.entry.id)
        .bind(numero)
        .bind(inv.id)
        .execute(&self.pool)
        .await
        .unwrap();
        inv.id
    }

    /// Une facture fournisseur `open` (posée en SQL : la recherche ne lit que
    /// ses colonnes).
    async fn facture_fournisseur(
        &self,
        fournisseur: &str,
        interne: &str,
        numero: &str,
        reference: &str,
        montant: &str,
        date: &str,
    ) -> i64 {
        let contact_id = self.contact(fournisseur).await;
        let entry: i64 =
            sqlx::query_scalar("SELECT MIN(id) FROM journal_entries WHERE company_id = ?")
                .bind(self.company_id())
                .fetch_one(&self.pool)
                .await
                .unwrap();
        sqlx::query(
            "INSERT INTO supplier_invoices (company_id, contact_id, internal_number, \
             supplier_invoice_number, payment_reference, status, invoice_date, due_date, \
             total_amount, purchase_journal_entry_id) \
             VALUES (?, ?, ?, ?, ?, 'open', ?, ?, ?, ?)",
        )
        .bind(self.company_id())
        .bind(contact_id)
        .bind(interne)
        .bind(numero)
        .bind(reference)
        .bind(d(date))
        .bind(d(date))
        .bind(dec(montant))
        .bind(entry)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    /// Une transaction `pending` de `montant` en `devise` sur `bank`.
    async fn transaction_sur(&self, bank: i64, montant: &str, devise: &str, le: &str) -> i64 {
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
                statement_id: Some(format!("GRP-{cle}")),
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
                details: "Virement groupé".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Banque payeuse".into()),
                counterparty_email: None,
                provider_type: None,
            }],
        )
        .await
        .expect("import");
        tx.commit().await.unwrap();
        txs[0].id
    }

    async fn transaction(&self, montant: &str, le: &str) -> i64 {
        self.transaction_sur(self.bank, montant, "CHF", le).await
    }

    async fn candidats(&self, tx: i64, query: &str) -> (u16, Value) {
        self.get(&format!(
            "/api/v1/reconciliation/transactions/{tx}/group-candidates{query}"
        ))
        .await
    }

    async fn ids_candidats(&self, tx: i64, query: &str) -> Vec<i64> {
        let (s, b) = self.candidats(tx, query).await;
        assert_eq!(s, 200, "candidats : {b}");
        b["candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["id"].as_i64().unwrap())
            .collect()
    }

    async fn accepter(&self, proposals: Value) -> (u16, Value) {
        self.post(
            "/api/v1/reconciliation/accept",
            json!({ "bankAccountId": self.bank, "proposals": proposals }),
        )
        .await
    }

    /// Un groupe seul : `(accepté, refusé)`, l'un des deux `Null`.
    async fn grouper(&self, tx: i64, ids: &[i64], compte: Option<i64>) -> (Value, Value) {
        let mut p = json!({ "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": ids });
        if let Some(c) = compte {
            p["differenceAccountId"] = json!(c);
        }
        let (s, b) = self.accepter(json!([p])).await;
        assert_eq!(s, 200, "{b}");
        (b["accepted"][0].clone(), b["failed"][0].clone())
    }

    async fn plafond(&self, valeur: &str) {
        sqlx::query(
            "UPDATE company_invoice_settings SET group_settlement_max_difference = ? \
             WHERE company_id = ?",
        )
        .bind(dec(valeur))
        .bind(self.company_id())
        .execute(&self.pool)
        .await
        .unwrap();
    }

    async fn scalaire(&self, sql: &str, id: i64) -> i64 {
        sqlx::query_scalar(sql)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("{sql} : {e}"))
    }

    /// Une empreinte de ce que le geste écrit.
    async fn empreinte(&self) -> Vec<i64> {
        let mut out = Vec::new();
        for sql in [
            "SELECT COUNT(*) FROM journal_entries",
            "SELECT COUNT(*) FROM journal_entry_lines",
            "SELECT COUNT(*) FROM invoice_settlements",
            "SELECT COUNT(*) FROM audit_log",
            "SELECT CAST(COALESCE(SUM(version), 0) AS SIGNED) FROM invoices",
            "SELECT COUNT(*) FROM bank_transactions WHERE status = 'reconciled'",
        ] {
            out.push(sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap());
        }
        out
    }

    /// Les montants écrits sur `account` dans l'écriture `entry` :
    /// `crédit − débit`.
    async fn solde_ligne(&self, entry: i64, account: i64) -> Option<Decimal> {
        sqlx::query_scalar::<_, Option<Decimal>>(
            "SELECT SUM(credit - debit) FROM journal_entry_lines \
             WHERE entry_id = ? AND account_id = ?",
        )
        .bind(entry)
        .bind(account)
        .fetch_one(&self.pool)
        .await
        .unwrap()
    }
}

/// L'empreinte de fichier d'un import (64 caractères hexadécimaux).
fn hash64(seed: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    format!("{:0>64x}", h.finish())
}

// ============================================================
// AC5 — la recherche
// ============================================================

/// AC5 — un crédit : les factures clients de **tous** les clients, triées par
/// l'ordre d'imputation, montants au centime et bruts ; la borne de date
/// (D + 1 rendue, D + 2 non) ; aucune facture fournisseur ; le réglage du
/// plafond à deux décimales ; le montant signé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_recherche_d_un_credit_rend_les_factures_de_tous_les_clients(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx
        .facture_de("Alpha Conseil", "F-2026-0012", "500.0050", "2026-06-03")
        .await;
    let b = ctx
        .facture_de("Beta Services", "F-2026-0013", "300.00", "2026-06-01")
        .await;
    let limite = ctx.facture("F-LIM", "50.00", "2026-06-21").await;
    let _apres = ctx.facture("F-APRES", "50.00", "2026-06-22").await;
    let _fourn = ctx
        .facture_fournisseur("Gamma", "FF-1", "G-1", "R", "80.00", "2026-06-01")
        .await;
    let tx = ctx.transaction("1250.00", "2026-06-20").await;

    let (s, body) = ctx.candidats(tx, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["direction"], "client");
    assert_eq!(body["transactionAmount"], "1250.00");
    assert_eq!(body["maxDifference"], "5.00");
    assert_eq!(body["hasMore"], false);
    let c = body["candidates"].as_array().unwrap();
    let ids: Vec<i64> = c.iter().map(|x| x["id"].as_i64().unwrap()).collect();
    assert_eq!(ids, vec![b, a, limite], "ordre d'imputation");
    let alpha = &c[1];
    assert_eq!(alpha["kind"], "invoice");
    assert_eq!(alpha["number"], "F-2026-0012");
    assert_eq!(alpha["contactName"], "Alpha Conseil");
    assert_eq!(alpha["date"], "2026-06-03");
    assert_eq!(alpha["dueDate"], "2026-06-03");
    assert_eq!(alpha["totalTtc"], "500.01");
    assert_eq!(alpha["amountDue"], "500.01");
    assert_eq!(alpha["amountDueRaw"], "500.0050");

    // Le réglage du plafond, lu tel quel.
    ctx.plafond("1.0000").await;
    let (_, body) = ctx.candidats(tx, "").await;
    assert_eq!(body["maxDifference"], "1.00");
}

/// AC5, D4 — un débit : les factures fournisseur ouvertes, aucune facture
/// client ; le montant signé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_recherche_d_un_debit_rend_les_factures_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let _client = ctx.facture("F-1", "80.00", "2026-06-01").await;
    let g = ctx
        .facture_fournisseur(
            "Gamma Fournitures",
            "FF-2026-001",
            "G-77",
            "RF18539007547034",
            "120.00",
            "2026-06-01",
        )
        .await;
    let tx = ctx.transaction("-200.00", "2026-06-20").await;
    let (s, body) = ctx.candidats(tx, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["direction"], "supplier");
    assert_eq!(body["transactionAmount"], "-200.00");
    let c = body["candidates"].as_array().unwrap();
    assert_eq!(c.len(), 1, "{body}");
    assert_eq!(c[0]["kind"], "supplier_invoice");
    assert_eq!(c[0]["id"], g);
    assert_eq!(c[0]["internalNumber"], "FF-2026-001");
    assert_eq!(c[0]["supplierInvoiceNumber"], "G-77");
    assert_eq!(c[0]["paymentReference"], "RF18539007547034");
    assert_eq!(c[0]["contactName"], "Gamma Fournitures");
    assert_eq!(c[0]["amountDue"], "120.00");
    assert_eq!(c[0]["amountDueRaw"], "120.0000");
    assert!(c[0].get("totalTtc").is_none());
    assert_eq!(ctx.ids_candidats(tx, "?q=G-77").await, vec![g]);
}

/// AC1, D4 — le pré-vol du type fournisseur : identifiant `≤ 0` dans
/// `supplierInvoiceIds`, `differenceAccountId` `≤ 0` → `400` au lot.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_pre_vol_du_type_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let tx = ctx.transaction("-30.00", "2026-06-20").await;
    for p in [
        json!({ "type": "supplier_invoice_group", "bankTransactionId": tx,
                "supplierInvoiceIds": [3, 0] }),
        json!({ "type": "supplier_invoice_group", "bankTransactionId": tx,
                "supplierInvoiceIds": [3, 4], "differenceAccountId": -1 }),
        json!({ "type": "supplier_invoice_group", "bankTransactionId": tx, "invoiceIds": [3, 4] }),
    ] {
        let (s, b) = ctx.accepter(json!([p])).await;
        assert_refus(s, &b, 400, "VALIDATION_ERROR");
    }
}

/// AC5 — `q`, `exclude`, `limit` et `hasMore` ; paramètres invalides en 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn q_exclude_et_limit(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx
        .facture_de("Alpha Conseil", "F-1", "10.00", "2026-06-01")
        .await;
    let b = ctx.facture_de("BL SA", "F-2", "10.00", "2026-06-02").await;
    let c = ctx
        .facture_de("Gamma", "X-2026-0099", "10.00", "2026-06-03")
        .await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    assert_eq!(ctx.ids_candidats(tx, "?q=Alph").await, vec![a]);
    assert_eq!(ctx.ids_candidats(tx, "?q=BL%20SA").await, vec![b]);
    assert_eq!(ctx.ids_candidats(tx, "?q=0099").await, vec![c]);
    assert_eq!(ctx.ids_candidats(tx, "?q=%25").await, Vec::<i64>::new());
    assert_eq!(
        ctx.ids_candidats(tx, &format!("?exclude={a},{c}")).await,
        vec![b]
    );
    let (s, body) = ctx.candidats(tx, "?limit=2").await;
    assert_eq!(s, 200);
    assert_eq!(body["candidates"].as_array().unwrap().len(), 2);
    assert_eq!(body["hasMore"], true, "la troncature a coupé");
    let (_, body) = ctx.candidats(tx, "?limit=3").await;
    assert_eq!(body["hasMore"], false);
    let (_, body) = ctx.candidats(tx, "?limit=0").await;
    assert_eq!(body["candidates"].as_array().unwrap().len(), 3, "défaut");
    let (s, body) = ctx.candidats(tx, "?exclude=1,x").await;
    assert_refus(s, &body, 400, "VALIDATION_ERROR");
    let many: Vec<String> = (1..=51).map(|i| i.to_string()).collect();
    let (s, body) = ctx
        .candidats(tx, &format!("?exclude={}", many.join(",")))
        .await;
    assert_refus(s, &body, 400, "VALIDATION_ERROR");
}

/// AC5 — un montant nul ou une devise autre que CHF : `200`,
/// `direction: null`, liste vide ; une transaction rejetée est servie.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn eur_nul_et_rejetee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let f = ctx.facture("F-1", "10.00", "2026-06-01").await;
    for (montant, devise) in [("10.00", "EUR"), ("0.00", "CHF")] {
        let tx = ctx
            .transaction_sur(ctx.bank, montant, devise, "2026-06-20")
            .await;
        let (s, body) = ctx.candidats(tx, "").await;
        assert_eq!(s, 200, "{body}");
        assert_eq!(body["direction"], Value::Null, "{montant} {devise}");
        assert_eq!(body["candidates"], json!([]));
        assert_eq!(body["hasMore"], false);
    }
    let tx = ctx.transaction("10.00", "2026-06-20").await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/reject",
            json!({ "bankAccountId": ctx.bank, "bankTransactionIds": [tx] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(ctx.ids_candidats(tx, "").await, vec![f]);
}

/// AC5 — les refus, ceux de `get_link_candidates` : transaction inexistante,
/// compte archivé, clos, non lié, transaction rapprochée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_refus_de_la_recherche(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx.candidats(i64::MAX, "").await;
    assert_refus(s, &b, 404, "NOT_FOUND");

    let f1 = ctx.facture("F-1", "10.00", "2026-06-01").await;
    let f2 = ctx.facture("F-2", "20.00", "2026-06-01").await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &[f1, f2], None).await;
    assert!(ko.is_null(), "{ko}");
    assert!(ok.is_object());
    let (s, b) = ctx.candidats(tx, "").await;
    assert_refus(s, &b, 404, "RECONCILIATION_TRANSACTION_NOT_PENDING");

    let pending = ctx.transaction("30.00", "2026-06-20").await;
    sqlx::query("UPDATE bank_accounts SET journal_account_id = NULL WHERE id = ?")
        .bind(ctx.bank)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.candidats(pending, "").await;
    assert_refus(s, &b, 412, "BANK_ACCOUNT_NOT_CONFIGURED");
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, closed_on = '2026-12-31' WHERE id = ?",
    )
    .bind(ctx.ledger)
    .bind(ctx.bank)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = ctx.candidats(pending, "").await;
    assert_refus(s, &b, 409, "BANK_ACCOUNT_CLOSED");
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(ctx.bank)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx.candidats(pending, "").await;
    assert_refus(s, &b, 404, "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");
}

// ============================================================
// AC1 — le type et le pré-vol
// ============================================================

/// AC1 — un type inconnu nomme les six types (cinq avant le commit D4) ; un
/// identifiant `≤ 0` dans `invoiceIds` ou un `differenceAccountId` `≤ 0`
/// rendent `400` au lot.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_type_et_le_pre_vol(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (s, b) = ctx
        .accepter(json!([{ "type": "bogus", "bankTransactionId": tx }]))
        .await;
    assert_refus(s, &b, 400, "VALIDATION_ERROR");
    let message = b["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.contains(
            "[\"invoice\", \"split\", \"rule\", \"supplier_invoice\", \"invoice_group\", \"supplier_invoice_group\"]"
        ),
        "{message}"
    );
    for p in [
        json!({ "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [3, 0] }),
        json!({ "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [3, -1] }),
        json!({ "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [3, 4],
                "differenceAccountId": 0 }),
        json!({ "type": "invoice_group", "bankTransactionId": tx }),
    ] {
        let (s, b) = ctx.accepter(json!([p])).await;
        assert_refus(s, &b, 400, "VALIDATION_ERROR");
    }
    // Sonde : la même proposition bien formée passe le pré-vol (refusée au
    // dépôt, en `failed[]`).
    let (s, b) = ctx
        .accepter(json!([{ "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [3] }]))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "RECONCILIATION_GROUP_INVALID");
    assert_eq!(b["failed"][0]["details"]["reason"], "too_few");
}

// ============================================================
// AC2 — le règlement d'un groupe client
// ============================================================

/// AC2, M1 — deux factures de deux clients, au montant exact : l'élément
/// d'`accepted[]` complet ; l'audit écrit par le dépôt, une seule fois.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_groupe_client_se_regle_et_se_trace_une_fois(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("F-1", "500.00", "2026-06-01").await;
    let b = ctx.facture("F-2", "300.00", "2026-06-02").await;
    let tx = ctx.transaction("800.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &[b, a], None).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["bankTransactionId"], tx);
    assert_eq!(ok["invoiceId"], 0);
    assert_eq!(ok["supplierInvoiceId"], Value::Null);
    assert!(ok["journalEntryId"].as_i64().unwrap() > 0);
    assert_eq!(ok["score"]["total"], 1.0);
    assert_eq!(ok["difference"], "0.0000");
    assert_eq!(ok["roundingDifference"], "0.0000");
    assert_eq!(ok["gap"], "0.00");
    assert_eq!(ok["differenceAccountId"], Value::Null);
    assert_eq!(
        ok["settledInvoices"],
        json!([
            { "invoiceId": a, "amount": "500.0000" },
            { "invoiceId": b, "amount": "300.0000" }
        ])
    );
    let accepted: Vec<Value> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'reconciliation.accepted' \
         AND entity_id = ?",
    )
    .bind(tx)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        accepted.len(),
        1,
        "une seule trace : la route n'en écrit pas"
    );
    let t = &accepted[0];
    assert_eq!(t["proposal_type"], "invoice_group");
    assert_eq!(t["batch_size"], 1);
    assert_eq!(num(&t["gap"]), Decimal::ZERO);
    assert_eq!(num(&t["rounding_difference"]), Decimal::ZERO);
    assert_eq!(t["difference_account_id"], Value::Null);
    assert_eq!(t["invoices"].as_array().unwrap().len(), 2);
    let (actor, user, key): (String, i64, Option<i64>) = sqlx::query_as(
        "SELECT actor_type, user_id, actor_api_key_id FROM audit_log \
         WHERE action = 'reconciliation.accepted' AND entity_id = ?",
    )
    .bind(tx)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (actor.as_str(), user, key),
        ("user", ctx.seeded.admin_user_id, None)
    );
    let paid = ctx
        .scalaire(
            "SELECT COUNT(*) FROM audit_log WHERE action = 'invoice.paid' \
             AND entity_id IN (SELECT id FROM invoices WHERE company_id = ?)",
            ctx.company_id(),
        )
        .await;
    assert_eq!(paid, 2);
}

/// AC2, D3 — une clé d'API en écriture : l'audit porte la clé, et la taille du
/// lot reçu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_d_api_est_l_acteur_du_groupe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("F-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("F-2", "20.00", "2026-06-01").await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let other = ctx.transaction("99.00", "2026-06-20").await;
    let (key_id, secret) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "intégration",
        "read-write",
    )
    .await;
    let (s, body) = ctx
        .req_as(
            &secret,
            reqwest::Method::POST,
            "/api/v1/reconciliation/accept",
            Some(json!({ "bankAccountId": ctx.bank, "proposals": [
                { "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [a, b] },
                { "type": "invoice_group", "bankTransactionId": other, "invoiceIds": [a] }
            ] })),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1, "{body}");
    let (actor, key, details): (String, Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, details_json FROM audit_log \
         WHERE action = 'reconciliation.accepted' AND entity_id = ?",
    )
    .bind(tx)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor.as_str(), key), ("api_key", Some(key_id)));
    assert_eq!(details["batch_size"], 2, "taille du lot reçu");
    // A12 — chaque `invoice.paid` porte aussi la clé.
    let paid: Vec<(String, Option<i64>)> = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id FROM audit_log \
         WHERE action = 'invoice.paid' AND entity_id IN (?, ?)",
    )
    .bind(a)
    .bind(b)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        paid,
        vec![
            ("api_key".to_string(), Some(key_id)),
            ("api_key".to_string(), Some(key_id))
        ]
    );
}

// ============================================================
// AC4 — le lot, l'ordre des refus, la concurrence
// ============================================================

/// AC4 — un lot mixte : une facture seule, un groupe valide, un groupe refusé ;
/// deux `accepted`, un `failed`, dans l'ordre ; rien d'écrit pour le refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_lot_mixte_garde_l_ordre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let seule = ctx.facture("F-0", "45.00", "2026-06-01").await;
    let a = ctx.facture("F-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("F-2", "20.00", "2026-06-01").await;
    let c = ctx.facture("F-3", "30.00", "2026-06-01").await;
    let t1 = ctx.transaction("45.00", "2026-06-20").await;
    let t2 = ctx.transaction("30.00", "2026-06-20").await;
    let t3 = ctx.transaction("30.00", "2026-06-20").await;
    let (s, body) = ctx
        .accepter(json!([
            { "type": "invoice", "bankTransactionId": t1, "invoiceId": seule },
            { "type": "invoice_group", "bankTransactionId": t2, "invoiceIds": [a, b] },
            { "type": "invoice_group", "bankTransactionId": t3, "invoiceIds": [c, c] }
        ]))
        .await;
    assert_eq!(s, 200, "{body}");
    let acc = body["accepted"].as_array().unwrap();
    assert_eq!(acc.len(), 2, "{body}");
    assert_eq!(acc[0]["bankTransactionId"], t1);
    assert!(
        acc[0].get("settledInvoices").is_none(),
        "champs absents hors groupe"
    );
    assert_eq!(acc[1]["bankTransactionId"], t2);
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["bankTransactionId"], t3);
    assert_eq!(failed[0]["errorCode"], "RECONCILIATION_GROUP_INVALID");
    assert_eq!(failed[0]["details"]["reason"], "duplicate");
    assert_eq!(failed[0]["details"]["invoiceId"], c);
    let paid_c = ctx
        .scalaire(
            "SELECT COUNT(*) FROM invoices WHERE id = ? AND paid_at IS NOT NULL",
            c,
        )
        .await;
    assert_eq!(paid_c, 0);
}

/// AC4 — sens et montant nul (sonde de C-43-15), ordre du pré-vol, et les
/// bornes du nombre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sens_montant_nul_et_ordre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("F-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("F-2", "20.00", "2026-06-01").await;
    let debit = ctx.transaction("-30.00", "2026-06-20").await;
    let (_, ko) = ctx.grouper(debit, &[a, b], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_TRANSACTION_NOT_CREDIT");
    assert_eq!(ko["details"]["reason"], "debit_transaction");
    let zero = ctx.transaction("0.00", "2026-06-20").await;
    let (_, ko) = ctx.grouper(zero, &[a, b], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_TRANSACTION_NOT_CREDIT");
    assert_eq!(ko["details"]["reason"], "zero_amount_transaction");
    // Transaction inexistante et une seule facture : le pré-vol par lot d'abord.
    let (_, ko) = ctx.grouper(i64::MAX, &[a], None).await;
    assert_eq!(ko["errorCode"], "BANK_TRANSACTION_NOT_FOUND");
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (_, ko) = ctx.grouper(tx, &[a], None).await;
    assert_eq!(ko["details"]["reason"], "too_few");
    let many: Vec<i64> = (1..=51).collect();
    let (_, ko) = ctx.grouper(tx, &many, None).await;
    assert_eq!(ko["details"]["reason"], "too_many");
    // Facture introuvable.
    let (_, ko) = ctx.grouper(tx, &[a, 999_999_999], None).await;
    assert_eq!(ko["errorCode"], "INVOICE_NOT_FOUND");
    assert_eq!(ko["details"]["invoiceId"], 999_999_999);
    // La sonde : la même transaction règle le groupe valide.
    let (ok, ko) = ctx.grouper(tx, &[a, b], None).await;
    assert!(ko.is_null(), "{ko}");
    assert!(ok.is_object());
}

/// AC4 — deux lots concurrents, sur deux comptes bancaires, dont les groupes
/// partagent une facture : l'un aboutit, l'autre refuse la facture ou est
/// rejoué ; jamais un `500`, jamais deux règlements.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deux_lots_concurrents_ne_reglent_pas_deux_fois(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ledger_b = compte(&pool, ctx.company_id(), "1021", "Banque B", "Asset").await;
    let bank_b = ctx.compte_bancaire(IBAN_B, ledger_b).await;
    let shared = ctx.facture("F-S", "10.00", "2026-06-01").await;
    let a = ctx.facture("F-A", "20.00", "2026-06-01").await;
    let b = ctx.facture("F-B", "20.00", "2026-06-01").await;
    let t1 = ctx.transaction("30.00", "2026-06-20").await;
    let t2 = ctx
        .transaction_sur(bank_b, "30.00", "CHF", "2026-06-20")
        .await;
    let un = ctx.post(
        "/api/v1/reconciliation/accept",
        json!({ "bankAccountId": ctx.bank, "proposals": [
            { "type": "invoice_group", "bankTransactionId": t1, "invoiceIds": [shared, a] }
        ] }),
    );
    let deux = ctx.post(
        "/api/v1/reconciliation/accept",
        json!({ "bankAccountId": bank_b, "proposals": [
            { "type": "invoice_group", "bankTransactionId": t2, "invoiceIds": [shared, b] }
        ] }),
    );
    let ((s1, b1), (s2, b2)) = tokio::join!(un, deux);
    assert_eq!((s1, s2), (200, 200), "{b1} {b2}");
    let accepted =
        b1["accepted"].as_array().unwrap().len() + b2["accepted"].as_array().unwrap().len();
    assert_eq!(accepted, 1, "{b1} {b2}");
    let failed = if b1["failed"].as_array().unwrap().is_empty() {
        &b2["failed"][0]
    } else {
        &b1["failed"][0]
    };
    assert_eq!(
        failed["errorCode"], "RECONCILIATION_INVOICE_NOT_ELIGIBLE",
        "{failed}"
    );
    let reason = failed["details"]["reason"].as_str().unwrap();
    assert!(
        ["race_during_update", "invoice_already_paid", "nothing_due"].contains(&reason),
        "{failed}"
    );
    assert_eq!(failed["details"]["invoiceId"], shared);
    let settled = ctx
        .scalaire(
            "SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?",
            shared,
        )
        .await;
    assert_eq!(settled, 1);
}

// ============================================================
// AC6 — la différence
// ============================================================

/// AC6, D8 bis — le tableau de la décision, un cas par ligne : `difference`
/// (`d`), `gap` (`e`), `roundingDifference` (`r`) et les lignes écrites sur le
/// compte choisi et sur le compte d'arrondi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_tableau_de_la_decision(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let arrondi = designate_rounding_account(&pool, ctx.company_id())
        .await
        .unwrap();
    // (factures, payé, d, e, r)
    let cas: [(&[&str], &str, &str, &str, &str); 6] = [
        (&["10.0050", "10.0050"], "20.01", "0.0000", "0.00", "0.0000"),
        (&["10.0050", "10.0050"], "20.02", "0.0100", "0.01", "0.0000"),
        (
            &["500.00", "300.00"],
            "799.99",
            "-0.0100",
            "-0.01",
            "0.0000",
        ),
        (
            &["10.0050", "10.0050", "10.0050"],
            "30.06",
            "0.0450",
            "0.05",
            "-0.0050",
        ),
        (
            &["10.0049", "10.0000"],
            "20.00",
            "-0.0049",
            "0.00",
            "-0.0049",
        ),
        (
            &["10.0050", "10.0000"],
            "20.00",
            "-0.0050",
            "-0.01",
            "0.0050",
        ),
    ];
    for (n, (factures, paye, dd, e, r)) in cas.iter().enumerate() {
        let mut ids = Vec::new();
        for (k, m) in factures.iter().enumerate() {
            ids.push(ctx.facture(&format!("T{n}-{k}"), m, "2026-06-01").await);
        }
        let tx = ctx.transaction(paye, "2026-06-20").await;
        let needs_account = *e != "0.00";
        let (ok, ko) = ctx
            .grouper(tx, &ids, needs_account.then_some(ctx.frais))
            .await;
        assert!(ko.is_null(), "cas {n} : {ko}");
        assert_eq!(ok["difference"], *dd, "cas {n}");
        assert_eq!(ok["gap"], *e, "cas {n}");
        assert_eq!(ok["roundingDifference"], *r, "cas {n}");
        assert_eq!(
            dec(ok["difference"].as_str().unwrap()),
            dec(ok["roundingDifference"].as_str().unwrap()) + dec(ok["gap"].as_str().unwrap()),
            "invariant, cas {n}"
        );
        let entry = ok["journalEntryId"].as_i64().unwrap();
        // Client : `e > 0` et `r > 0` au crédit.
        let sur_frais = ctx.solde_ligne(entry, ctx.frais).await;
        let sur_arrondi = ctx.solde_ligne(entry, arrondi).await;
        assert_eq!(
            sur_frais,
            needs_account.then(|| dec(e)),
            "ligne d'écart, cas {n}"
        );
        assert_eq!(
            sur_arrondi,
            (*r != "0.0000").then(|| dec(r)),
            "ligne d'arrondi, cas {n}"
        );
        if needs_account {
            assert_eq!(ok["differenceAccountId"], ctx.frais, "cas {n}");
        } else {
            assert_eq!(ok["differenceAccountId"], Value::Null, "cas {n}");
        }
    }
}

/// AC6 — un écart sans compte est refusé ; avec un compte, moins-payé et
/// trop-perçu sont admis ; au-delà du réglage, refusé avec le réglage ; les
/// montants soldés au brut.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_ecart_dans_les_deux_sens(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let mk = async |n: &str| {
        (
            ctx.facture(&format!("{n}-1"), "500.00", "2026-06-01").await,
            ctx.facture(&format!("{n}-2"), "300.00", "2026-06-01").await,
        )
    };
    let (a, b) = mk("M").await;
    let tx = ctx.transaction("799.60", "2026-06-20").await;
    let empreinte = ctx.empreinte().await;
    let (_, ko) = ctx.grouper(tx, &[a, b], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_GROUP_INVALID");
    assert_eq!(ko["details"]["reason"], "difference_account_required");
    assert_eq!(ko["details"]["gap"], "-0.40");
    assert_eq!(ctx.empreinte().await, empreinte, "rien d'écrit");
    let (ok, _) = ctx.grouper(tx, &[a, b], Some(ctx.frais)).await;
    assert_eq!(ok["gap"], "-0.40");
    assert_eq!(
        ctx.solde_ligne(ok["journalEntryId"].as_i64().unwrap(), ctx.frais)
            .await,
        Some(dec("-0.40")),
        "moins-payé au débit du compte choisi"
    );
    assert_eq!(ok["settledInvoices"][0]["amount"], "500.0000");

    let (a, b) = mk("P").await;
    let tx = ctx.transaction("800.40", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &[a, b], Some(ctx.frais)).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "0.40");

    ctx.plafond("1.00").await;
    let (a, b) = mk("X").await;
    let tx = ctx.transaction("801.01", "2026-06-20").await;
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(ctx.frais)).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE");
    assert_eq!(ko["details"]["maxDifference"], "1.00");
    assert_eq!(ko["details"]["gap"], "1.01");
    assert_eq!(ko["details"]["transactionAmount"], "801.01");
    assert_eq!(ko["details"]["sumApplied"], "800.0000");
    let (ok, _) = {
        let tx = ctx.transaction("801.00", "2026-06-20").await;
        ctx.grouper(tx, &[a, b], Some(ctx.frais)).await
    };
    assert_eq!(ok["gap"], "1.00", "au plafond exactement : admis");
}

/// AC6, D8 bis — l'ordre : le plafond d'abord, puis le compte d'arrondi, puis
/// le compte choisi ; le plafond ne borne que `|e|`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_ordre_des_controles_de_la_difference(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Sans compte d'arrondi désigné (le seed n'en a pas).
    // r ≠ 0, e ≠ 0, sans compte choisi → le compte d'arrondi d'abord.
    let a = ctx.facture("R-1", "10.0050", "2026-06-01").await;
    let b = ctx.facture("R-2", "10.0000", "2026-06-01").await;
    let tx = ctx.transaction("20.00", "2026-06-20").await;
    let (_, ko) = ctx.grouper(tx, &[a, b], None).await;
    assert_eq!(ko["errorCode"], "ROUNDING_ACCOUNT_NOT_CONFIGURED");
    // r = 0, e ≠ 0, sans compte choisi → le compte choisi, jamais l'arrondi.
    let c = ctx.facture("R-3", "10.00", "2026-06-01").await;
    let e = ctx.facture("R-4", "10.00", "2026-06-01").await;
    let tx2 = ctx.transaction("20.01", "2026-06-20").await;
    let (_, ko) = ctx.grouper(tx2, &[c, e], None).await;
    assert_eq!(ko["details"]["reason"], "difference_account_required");
    // Plafond à 0.00 : `e ≠ 0` refusé, avec ou sans compte, avant tout compte.
    ctx.plafond("0.00").await;
    let (_, ko) = ctx.grouper(tx2, &[c, e], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE");
    assert_eq!(ko["details"]["maxDifference"], "0.00");
    let (_, ko) = ctx.grouper(tx2, &[c, e], Some(ctx.frais)).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_GROUP_DIFFERENCE_TOO_LARGE");
    // … mais l'arrondi automatique (`e = 0`) passe, le plafond ne bornant que
    // `|e|`.
    designate_rounding_account(&pool, ctx.company_id())
        .await
        .unwrap();
    let f = ctx.facture("R-5", "10.0049", "2026-06-01").await;
    let g = ctx.facture("R-6", "10.0000", "2026-06-01").await;
    let tx3 = ctx.transaction("20.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx3, &[f, g], None).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "0.00");
    assert_eq!(ok["roundingDifference"], "-0.0049");
}

/// AC6, D9 — les contrôles du compte choisi, dans l'ordre, un test par cause ;
/// les deux causes de `role: "counterparty"` et leur discriminant ; les
/// comptes admis.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_controles_du_compte_choisi(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let pair = async |n: &str| {
        let a = ctx.facture(&format!("{n}-1"), "10.00", "2026-06-01").await;
        let b = ctx.facture(&format!("{n}-2"), "10.00", "2026-06-01").await;
        let tx = ctx.transaction("19.60", "2026-06-20").await;
        (a, b, tx)
    };
    // 1. La banque de la transaction, même non imputable.
    let (a, b, tx) = pair("B").await;
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(ctx.ledger)).await;
    assert_eq!(ko["errorCode"], "VALIDATION_ERROR");
    assert_eq!(ko["details"]["reason"], "counterparty_equals_bank_ledger");
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(ctx.ledger)
        .execute(&pool)
        .await
        .unwrap();
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(ctx.ledger)).await;
    assert_eq!(ko["details"]["reason"], "counterparty_equals_bank_ledger");
    sqlx::query("UPDATE accounts SET postable = TRUE WHERE id = ?")
        .bind(ctx.ledger)
        .execute(&pool)
        .await
        .unwrap();
    // 2. Inexistant, archivé, d'une autre société.
    let archive = compte(&pool, ctx.company_id(), "6901", "Archivé", "Expense").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let other_company: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let etranger = compte(&pool, other_company, "6900", "Frais", "Expense").await;
    for id in [999_999_999, archive, etranger] {
        let (_, ko) = ctx.grouper(tx, &[a, b], Some(id)).await;
        assert_eq!(ko["errorCode"], "ACCOUNT_NOT_FOUND", "{id}");
        assert_eq!(ko["details"]["missingAccountIds"], json!([id]));
    }
    // 3. Non imputable, avec son numéro.
    let titre = compte(&pool, ctx.company_id(), "69", "Titre", "Expense").await;
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(titre)
        .execute(&pool)
        .await
        .unwrap();
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(titre)).await;
    assert_eq!(ko["errorCode"], "ACCOUNT_NOT_POSTABLE");
    assert_eq!(
        ko["details"]["rejected"],
        json!([{ "accountId": titre, "accountNumber": "69" }])
    );
    // 4. Le compte de créance des factures : `counterparty`, sans
    //    `bankAccountId`, `accountId` = le compte envoyé.
    let creance = ctx.seeded.accounts["1100"];
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(creance)).await;
    assert_eq!(ko["errorCode"], "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT");
    assert_eq!(ko["details"]["role"], "counterparty");
    assert_eq!(ko["details"]["accountId"], creance);
    assert!(ko["details"].get("bankAccountId").is_none(), "{ko}");
    // Rien d'écrit par ces refus ; la sonde : le compte 2000 (dette, autre
    // famille) est admis.
    let (ok, ko) = ctx
        .grouper(tx, &[a, b], Some(ctx.seeded.accounts["2000"]))
        .await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "-0.40");
    // Le compte du grand livre d'une AUTRE banque est admis.
    let ledger_b = compte(&pool, ctx.company_id(), "1021", "Banque B", "Asset").await;
    let _bank_b = ctx.compte_bancaire(IBAN_B, ledger_b).await;
    let (a, b, tx) = pair("O").await;
    let (ok, ko) = ctx.grouper(tx, &[a, b], Some(ledger_b)).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["differenceAccountId"], ledger_b);
}

/// AC6, D9 (P4 M1) — la seconde cause de `role: "counterparty"` : le compte de
/// créance d'une facture est le compte de la banque (configuration) → même
/// code, même rôle, **avec** `bankAccountId`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_banque_egale_a_la_creance_porte_bank_account_id(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("C-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("C-2", "20.00", "2026-06-01").await;
    let bank_sur_creance = ctx
        .compte_bancaire(IBAN_B, ctx.seeded.accounts["1100"])
        .await;
    let tx = ctx
        .transaction_sur(bank_sur_creance, "30.00", "CHF", "2026-06-20")
        .await;
    let (s, body) = ctx
        .post(
            "/api/v1/reconciliation/accept",
            json!({ "bankAccountId": bank_sur_creance, "proposals": [
                { "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [a, b] }
            ] }),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    let ko = &body["failed"][0];
    assert_eq!(ko["errorCode"], "SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT");
    assert_eq!(ko["details"]["role"], "counterparty");
    assert_eq!(ko["details"]["bankAccountId"], bank_sur_creance);
    assert_eq!(ko["details"]["accountId"], ctx.seeded.accounts["1100"]);
}

/// AC6 — une facture au projet archivé après coup est acceptée ; un paiement
/// antérieur à une facture est refusé, la facture nommée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn projet_archive_et_paiement_anterieur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let projet: i64 =
        sqlx::query("INSERT INTO projects (company_id, code, name) VALUES (?, 'P', 'Projet')")
            .bind(ctx.company_id())
            .execute(&pool)
            .await
            .unwrap()
            .last_insert_id() as i64;
    let a = ctx
        .facture_projet("Client A", "P-1", "10.00", "2026-06-01", Some(projet))
        .await;
    let b = ctx.facture("P-2", "20.00", "2026-06-01").await;
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(projet)
        .execute(&pool)
        .await
        .unwrap();
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &[a, b], None).await;
    assert!(ko.is_null(), "{ko}");
    assert!(ok.is_object());

    let c = ctx.facture("D-1", "10.00", "2026-06-01").await;
    let tard = ctx.facture("D-2", "20.00", "2026-06-25").await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (_, ko) = ctx.grouper(tx, &[c, tard], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_INVOICE_NOT_ELIGIBLE");
    assert_eq!(ko["details"]["reason"], "payment_date_before_invoice_date");
    assert_eq!(ko["details"]["tolerance_days"], 1);
    assert_eq!(ko["details"]["invoiceId"], tard);
    assert_eq!(ko["details"]["invoiceNumber"], "D-2");
}

// ============================================================
// AC7 — la vue et l'annulation
// ============================================================

async fn vue(ctx: &Ctx, tx: i64) -> Value {
    let (s, b) = ctx
        .get(&format!("/api/v1/reconciliation/transactions/{tx}"))
        .await;
    assert_eq!(s, 200, "{b}");
    b
}

/// AC7 — la vue d'un groupe (`e < 0`), le refus d'un retrait seul, puis
/// l'annulation et sa réponse.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_vue_et_l_annulation_d_un_groupe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("V-1", "500.00", "2026-06-01").await;
    let b = ctx.facture("V-2", "300.00", "2026-06-01").await;
    let tx = ctx.transaction("799.60", "2026-06-20").await;
    let (ok, _) = ctx.grouper(tx, &[a, b], Some(ctx.frais)).await;
    assert_eq!(ok["gap"], "-0.40");
    let v = vue(&ctx, tx).await;
    assert_eq!(v["kind"], "grouped_settlement");
    assert_eq!(v["invoiceId"], Value::Null);
    assert_eq!(v["invoiceNumber"], Value::Null);
    assert_eq!(v["supplierInvoiceId"], Value::Null);
    assert_eq!(
        v["groupedInvoices"],
        json!([
            { "kind": "invoice", "id": a, "number": "V-1", "amount": "500.0000" },
            { "kind": "invoice", "id": b, "number": "V-2", "amount": "300.0000" }
        ])
    );
    assert_eq!(v["gap"], "-0.40");
    assert_eq!(
        v["differenceAccount"],
        json!({ "id": ctx.frais, "number": "6900", "name": "Frais divers" })
    );
    assert_eq!(v["cancellable"], true);
    assert_eq!(v["cancelBlockedInvoice"], Value::Null);

    // Un groupe n'est jamais un retrait seul.
    let empreinte = ctx.empreinte().await;
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{tx}/cancel"),
            json!({ "expectUnlinkOnly": true }),
        )
        .await;
    assert_refus(s, &body, 409, "RECONCILIATION_CANCEL_CHANGED");
    assert_eq!(ctx.empreinte().await, empreinte, "rien d'écrit");

    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{tx}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["kind"], "grouped_settlement");
    assert_eq!(body["invoiceId"], Value::Null);
    assert_eq!(body["supplierInvoiceId"], Value::Null);
    assert!(body["reversalJournalEntryId"].as_i64().is_some());
    assert_eq!(
        body["cancelledInvoices"],
        json!([
            { "kind": "invoice", "id": a, "number": "V-1" },
            { "kind": "invoice", "id": b, "number": "V-2" }
        ])
    );
    let v = vue(&ctx, tx).await;
    assert_eq!(v["kind"], Value::Null);
    assert_eq!(v["groupedInvoices"], json!([]));
    assert_eq!(v["gap"], Value::Null);
    assert_eq!(v["differenceAccount"], Value::Null);
}

/// AC7 — `gap` et `differenceAccount` pour `e > 0` et `e = 0` ; une facture
/// créditée bloque l'annulation et se nomme ; l'annulation d'un rapprochement
/// unitaire rend son `kind` et une liste vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_vue_selon_l_ecart_et_la_facture_qui_bloque(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("W-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("W-2", "20.00", "2026-06-01").await;
    let tx = ctx.transaction("30.40", "2026-06-20").await;
    ctx.grouper(tx, &[a, b], Some(ctx.frais)).await;
    let v = vue(&ctx, tx).await;
    assert_eq!(v["gap"], "0.40");
    assert_eq!(v["differenceAccount"]["id"], ctx.frais);

    let c = ctx.facture("Z-1", "10.00", "2026-06-01").await;
    let e = ctx.facture("Z-2", "20.00", "2026-06-01").await;
    let tx0 = ctx.transaction("30.00", "2026-06-20").await;
    ctx.grouper(tx0, &[c, e], None).await;
    let v = vue(&ctx, tx0).await;
    assert_eq!(v["gap"], "0.00");
    assert_eq!(v["differenceAccount"], Value::Null);

    // État forgé (C-e431a) : une facture du groupe créditée.
    sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
        .bind(e)
        .execute(&pool)
        .await
        .unwrap();
    let v = vue(&ctx, tx0).await;
    assert_eq!(v["cancellable"], false);
    assert_eq!(v["cancelBlockedBy"], "INVOICE_CREDITED");
    assert_eq!(
        v["cancelBlockedInvoice"],
        json!({ "kind": "invoice", "id": e, "number": "Z-2" })
    );
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{tx0}/cancel"),
            json!({}),
        )
        .await;
    assert_refus(s, &body, 409, "INVOICE_CREDITED");

    // Un rapprochement unitaire : `kind` et liste vide.
    let seule = ctx.facture("U-1", "45.00", "2026-06-01").await;
    let t1 = ctx.transaction("45.00", "2026-06-20").await;
    let (s, body) = ctx
        .accepter(json!([{ "type": "invoice", "bankTransactionId": t1, "invoiceId": seule }]))
        .await;
    assert_eq!(s, 200, "{body}");
    let v = vue(&ctx, t1).await;
    assert_eq!(v["groupedInvoices"], json!([]));
    assert_eq!(v["gap"], Value::Null);
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{t1}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["kind"], "invoice_settlement");
    assert_eq!(body["invoiceId"], seule);
    assert_eq!(body["cancelledInvoices"], json!([]));
}

// ============================================================
// AC8 — la garde CHF
// ============================================================

/// AC8 — une transaction EUR est refusée en
/// `RECONCILIATION_CURRENCY_MISMATCH`, rien d'écrit ; la même en CHF passe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_garde_chf(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("E-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("E-2", "20.00", "2026-06-01").await;
    let eur = ctx
        .transaction_sur(ctx.bank, "30.00", "EUR", "2026-06-20")
        .await;
    let empreinte = ctx.empreinte().await;
    let (_, ko) = ctx.grouper(eur, &[a, b], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_CURRENCY_MISMATCH");
    assert_eq!(
        ko["details"],
        json!({ "transactionCurrency": "EUR", "expected": "CHF" })
    );
    assert_eq!(ctx.empreinte().await, empreinte, "rien d'écrit");
    let chf = ctx.transaction("30.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(chf, &[a, b], None).await;
    assert!(ko.is_null(), "{ko}");
    assert!(ok.is_object());
}

// ============================================================
// AC10 — l'accès
// ============================================================

/// AC10 — Comptable admis ; Consultation refusée (`403`) ; clé en lecture :
/// la recherche oui, l'acceptation non.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_acces(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("A-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("A-2", "20.00", "2026-06-01").await;
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let chemin = format!("/api/v1/reconciliation/transactions/{tx}/group-candidates");
    let corps = json!({ "bankAccountId": ctx.bank, "proposals": [
        { "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [a, b] }
    ] });

    let consultation = forge_jwt(ctx.seeded.admin_user_id, "Consultation", ctx.company_id());
    let (s, _) = ctx
        .req_as(&consultation, reqwest::Method::GET, &chemin, None)
        .await;
    assert_eq!(s, 403);
    let (s, _) = ctx
        .req_as(
            &consultation,
            reqwest::Method::POST,
            "/api/v1/reconciliation/accept",
            Some(corps.clone()),
        )
        .await;
    assert_eq!(s, 403);

    let (_, lecture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "lecture",
        "read",
    )
    .await;
    let (s, body) = ctx
        .req_as(&lecture, reqwest::Method::GET, &chemin, None)
        .await;
    assert_eq!(s, 200, "{body}");
    let (s, body) = ctx
        .req_as(
            &lecture,
            reqwest::Method::POST,
            "/api/v1/reconciliation/accept",
            Some(corps.clone()),
        )
        .await;
    assert_refus(s, &body, 403, "API_KEY_READ_ONLY");

    let comptable = forge_jwt(ctx.seeded.admin_user_id, "Comptable", ctx.company_id());
    let (s, body) = ctx
        .req_as(&comptable, reqwest::Method::GET, &chemin, None)
        .await;
    assert_eq!(s, 200, "{body}");
    let (s, body) = ctx
        .req_as(
            &comptable,
            reqwest::Method::POST,
            "/api/v1/reconciliation/accept",
            Some(corps),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1, "{body}");
}

// ============================================================
// AC3, AC6, AC7 — le type fournisseur (D4), sur l'implémentation de la 43-1b
// ============================================================

impl Ctx {
    /// Une facture fournisseur validée avec son écriture d'achat
    /// `D 4000 / C 2000` (la dette lue par le règlement).
    async fn facture_achat(&self, interne: &str, montant: &str, date: &str) -> i64 {
        let s = &self.seeded;
        let contact_id = self.contact(&format!("Fournisseur {interne}")).await;
        let amount = dec(montant);
        let je = journal_entries::create(
            &self.pool,
            s.fiscal_year_id,
            Acteur::utilisateur(s.admin_user_id),
            NewJournalEntry {
                company_id: s.company_id,
                entry_date: d(date),
                journal: Journal::Achats,
                description: "Achat".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: s.accounts["4000"],
                        debit: amount,
                        credit: Decimal::ZERO,
                        project_id: None,
                        foreign: None,
                    },
                    NewJournalEntryLine {
                        account_id: s.accounts["2000"],
                        debit: Decimal::ZERO,
                        credit: amount,
                        project_id: None,
                        foreign: None,
                    },
                ],
            },
        )
        .await
        .expect("écriture d'achat");
        sqlx::query(
            "INSERT INTO supplier_invoices (company_id, contact_id, internal_number, status, \
             invoice_date, due_date, total_amount, purchase_journal_entry_id) \
             VALUES (?, ?, ?, 'open', ?, ?, ?, ?)",
        )
        .bind(s.company_id)
        .bind(contact_id)
        .bind(interne)
        .bind(d(date))
        .bind(d(date))
        .bind(amount)
        .bind(je.entry.id)
        .execute(&self.pool)
        .await
        .unwrap()
        .last_insert_id() as i64
    }

    async fn grouper_fournisseur(
        &self,
        tx: i64,
        ids: &[i64],
        compte: Option<i64>,
    ) -> (Value, Value) {
        let mut p = json!({ "type": "supplier_invoice_group", "bankTransactionId": tx,
                            "supplierInvoiceIds": ids });
        if let Some(c) = compte {
            p["differenceAccountId"] = json!(c);
        }
        let (s, b) = self.accepter(json!([p])).await;
        assert_eq!(s, 200, "{b}");
        (b["accepted"][0].clone(), b["failed"][0].clone())
    }
}

/// AC3 — deux factures fournisseur réglées au montant exact par un débit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_groupe_fournisseur_se_regle(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture_achat("FF-1", "120.00", "2026-06-01").await;
    let b = ctx.facture_achat("FF-2", "80.00", "2026-06-02").await;
    let tx = ctx.transaction("-200.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper_fournisseur(tx, &[a, b], None).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["invoiceId"], 0);
    assert_eq!(ok["supplierInvoiceId"], Value::Null);
    assert_eq!(ok["gap"], "0.00");
    assert_eq!(
        ok["settledInvoices"],
        json!([
            { "supplierInvoiceId": a, "amount": "120.0000" },
            { "supplierInvoiceId": b, "amount": "80.0000" }
        ])
    );
    // P3-6 — ce qui est écrit : les factures payées, l'écriture, la transaction
    // rapprochée (le détail des lignes et du lettrage est prouvé au dépôt par
    // la 43-1b).
    let payees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM supplier_invoices WHERE id IN (?, ?) AND status = 'paid'",
    )
    .bind(a)
    .bind(b)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payees, 2);
    let entry = ok["journalEntryId"].as_i64().unwrap();
    let lignes = ctx
        .scalaire(
            "SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?",
            entry,
        )
        .await;
    assert_eq!(lignes, 3, "banque et deux dettes");
    let (statut, lien): (String, Option<i64>) =
        sqlx::query_as("SELECT status, matched_entry_id FROM bank_transactions WHERE id = ?")
            .bind(tx)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((statut.as_str(), lien), ("reconciled", Some(entry)));
    // Un crédit est refusé (sonde du sens).
    let credit = ctx.transaction("200.00", "2026-06-20").await;
    let c = ctx.facture_achat("FF-3", "120.00", "2026-06-01").await;
    let e = ctx.facture_achat("FF-4", "80.00", "2026-06-01").await;
    let (_, ko) = ctx.grouper_fournisseur(credit, &[c, e], None).await;
    assert_eq!(ko["errorCode"], "RECONCILIATION_TRANSACTION_NOT_DEBIT");
    assert_eq!(ko["details"]["reason"], "credit_transaction");
}

/// AC6 — l'écart fournisseur : un débit supérieur aux factures, compte choisi,
/// `gap` positif ; sans compte, refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_ecart_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture_achat("FF-1", "120.00", "2026-06-01").await;
    let b = ctx.facture_achat("FF-2", "80.00", "2026-06-02").await;
    let tx = ctx.transaction("-200.40", "2026-06-20").await;
    let (_, ko) = ctx.grouper_fournisseur(tx, &[a, b], None).await;
    assert_eq!(ko["details"]["reason"], "difference_account_required");
    let (ok, ko) = ctx.grouper_fournisseur(tx, &[a, b], Some(ctx.frais)).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "0.40");
    assert_eq!(ok["differenceAccountId"], ctx.frais);
    // P3-6 — un débit supérieur aux factures : l'écart au **débit** du compte
    // choisi.
    assert_eq!(
        ctx.solde_ligne(ok["journalEntryId"].as_i64().unwrap(), ctx.frais)
            .await,
        Some(dec("-0.40"))
    );
    let payees: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM supplier_invoices WHERE id IN (?, ?) AND status = 'paid'",
    )
    .bind(a)
    .bind(b)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payees, 2);
}

/// Epic 37 — un compte d'écart tenu en devise est refusé
/// (`ACCOUNT_IN_FOREIGN_CURRENCY`, `details { accountId, currency }`), rien
/// d'écrit ; le même compte en CHF passe (sonde).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_d_ecart_en_devise_est_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("D-1", "500.00", "2026-06-01").await;
    let b = ctx.facture("D-2", "300.00", "2026-06-01").await;
    let tx = ctx.transaction("799.60", "2026-06-20").await;
    let en_devise = compte(&pool, ctx.company_id(), "6905", "Écarts EUR", "Expense").await;
    sqlx::query("UPDATE accounts SET currency = 'EUR' WHERE id = ?")
        .bind(en_devise)
        .execute(&pool)
        .await
        .unwrap();
    let empreinte = ctx.empreinte().await;
    let (_, ko) = ctx.grouper(tx, &[a, b], Some(en_devise)).await;
    assert_eq!(ko["errorCode"], "ACCOUNT_IN_FOREIGN_CURRENCY", "{ko}");
    assert_eq!(
        ko["details"],
        json!({ "accountId": en_devise, "currency": "EUR" })
    );
    assert_eq!(ctx.empreinte().await, empreinte, "rien d'écrit");
    // Sonde : le **même** compte, rendu au CHF (aucune ligne ne le vise
    // encore), est admis.
    sqlx::query("UPDATE accounts SET currency = NULL WHERE id = ?")
        .bind(en_devise)
        .execute(&pool)
        .await
        .unwrap();
    let (ok, ko) = ctx.grouper(tx, &[a, b], Some(en_devise)).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "-0.40");
    assert_eq!(ok["differenceAccountId"], en_devise);
}

/// Revue P3, P3-4 (AC5) — un débit dont le montant exact est retenu par une
/// facture ouverte d'un lot `generated` **de ce compte** : le `POST` refuse
/// tout groupe (`RECONCILIATION_SUPPLIER_PAYMENT_HOLD`, garde (d) de
/// l'unitaire) ; la recherche n'offre donc rien et dit pourquoi
/// (`heldByPaymentBatch`). Sans le lot, les mêmes factures sont offertes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_recherche_reflete_la_retenue_d_un_lot(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture_achat("FF-A", "10.00", "2026-06-01").await;
    let b = ctx.facture_achat("FF-B", "20.00", "2026-06-01").await;
    let tx = ctx.transaction("-30.00", "2026-06-20").await;
    let (s, body) = ctx.candidats(tx, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["candidates"].as_array().unwrap().len(), 2, "{body}");
    // P4-2 — la clé est **présente** et vaut `null` (l'index d'une clé absente
    // rendrait aussi `Null`).
    assert_retenue_nulle(&body);

    // Une facture ouverte de 30.00 dans un lot `generated` du compte.
    let retenue = ctx.facture_achat("FF-L", "30.00", "2026-06-01").await;
    let lot = sqlx::query(
        "INSERT INTO payment_batches (company_id, bank_account_id, status, \
         requested_execution_date, msg_id, payment_info_id) \
         VALUES (?, ?, 'generated', '2026-06-19', REPLACE(UUID(), '-', ''), 'P')",
    )
    .bind(ctx.company_id())
    .bind(ctx.bank)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
         end_to_end_id, amount) VALUES (?, ?, 1, 'E2E', 30)",
    )
    .bind(lot)
    .bind(retenue)
    .execute(&pool)
    .await
    .unwrap();
    // Le `POST` refuse le groupe…
    let (_, ko) = ctx.grouper_fournisseur(tx, &[a, b], None).await;
    assert_eq!(
        ko["errorCode"], "RECONCILIATION_SUPPLIER_PAYMENT_HOLD",
        "{ko}"
    );
    // … et la recherche ne l'offre plus, en le disant.
    let (s, body) = ctx.candidats(tx, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["direction"], "supplier", "{body}");
    assert_eq!(body["candidates"], json!([]), "{body}");
    assert_eq!(body["hasMore"], false, "{body}");
    assert_eq!(
        body["heldByPaymentBatch"],
        json!({ "supplierInvoiceId": retenue, "paymentBatchId": lot }),
        "{body}"
    );

    // P4-4 — sondes négatives.
    // (1) Un crédit du même montant : la recherche client n'est pas touchée.
    let c1 = ctx.facture("C-1", "10.00", "2026-06-01").await;
    let c2 = ctx.facture("C-2", "20.00", "2026-06-01").await;
    let credit = ctx.transaction("30.00", "2026-06-20").await;
    let (s, body) = ctx.candidats(credit, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["direction"], "client", "{body}");
    assert_eq!(ctx.ids_candidats(credit, "").await, vec![c1, c2], "{body}");
    assert_retenue_nulle(&body);
    // (2) Le lot sur un **autre** compte bancaire ne retient pas ce débit.
    let autre_ledger = compte(&pool, ctx.company_id(), "1021", "Banque B", "Asset").await;
    let autre_banque = ctx.compte_bancaire(IBAN_B, autre_ledger).await;
    sqlx::query("UPDATE payment_batches SET bank_account_id = ? WHERE id = ?")
        .bind(autre_banque)
        .bind(lot)
        .execute(&pool)
        .await
        .unwrap();
    let (s, body) = ctx.candidats(tx, "").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["candidates"].as_array().unwrap().len(), 2, "{body}");
    assert_retenue_nulle(&body);
    // (3) Le lot rendu au compte puis **confirmé** ne retient plus rien. État
    // forgé : sa facture reste `open` (la confirmation réelle la paierait, ce
    // qui l'exclurait déjà par son statut) — seul le statut du lot change, ce
    // qui isole le critère `pb.status = 'generated'`.
    sqlx::query(
        "UPDATE payment_batches SET bank_account_id = ?, status = 'confirmed' WHERE id = ?",
    )
    .bind(ctx.bank)
    .bind(lot)
    .execute(&pool)
    .await
    .unwrap();
    // Hors de tout lot `generated`, FF-L redevient elle-même candidate.
    assert_eq!(ctx.ids_candidats(tx, "").await, vec![a, b, retenue]);
    let (_, body) = ctx.candidats(tx, "").await;
    assert_retenue_nulle(&body);
}

/// La réponse de la recherche porte `heldByPaymentBatch`, à `null` (P4-2).
fn assert_retenue_nulle(body: &Value) {
    let objet = body.as_object().expect("objet");
    assert!(objet.contains_key("heldByPaymentBatch"), "{body}");
    assert_eq!(objet["heldByPaymentBatch"], Value::Null, "{body}");
}

/// AC7 — la vue et l'annulation d'un groupe fournisseur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_vue_et_l_annulation_d_un_groupe_fournisseur(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture_achat("FF-1", "120.00", "2026-06-01").await;
    let b = ctx.facture_achat("FF-2", "80.00", "2026-06-02").await;
    let tx = ctx.transaction("-199.60", "2026-06-20").await;
    let (ok, _) = ctx.grouper_fournisseur(tx, &[a, b], Some(ctx.frais)).await;
    assert_eq!(ok["gap"], "-0.40");
    let v = vue(&ctx, tx).await;
    assert_eq!(v["kind"], "grouped_settlement");
    assert_eq!(
        v["groupedInvoices"],
        json!([
            { "kind": "supplier_invoice", "id": a, "number": "FF-1", "amount": "120.0000" },
            { "kind": "supplier_invoice", "id": b, "number": "FF-2", "amount": "80.0000" }
        ])
    );
    assert_eq!(v["gap"], "-0.40");
    assert_eq!(v["differenceAccount"]["id"], ctx.frais);
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{tx}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["kind"], "grouped_settlement");
    assert_eq!(
        body["cancelledInvoices"],
        json!([
            { "kind": "supplier_invoice", "id": a, "number": "FF-1" },
            { "kind": "supplier_invoice", "id": b, "number": "FF-2" }
        ])
    );
}

// ============================================================
// Revue P1 — sondes et bords
// ============================================================

/// B2 — le type fournisseur : une proposition bien formée sur une transaction
/// inexistante passe l'extracteur et le pré-vol (`200`,
/// `BANK_TRANSACTION_NOT_FOUND`), ce qu'un type inconnu ne fait pas (`400`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_type_fournisseur_se_desserialise_et_ne_panique_pas(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = ctx
        .accepter(
            json!([{ "type": "supplier_invoice_group", "bankTransactionId": i64::MAX,
                           "supplierInvoiceIds": [3, 4] }]),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["failed"][0]["errorCode"], "BANK_TRANSACTION_NOT_FOUND");
    let (s, b) = ctx
        .accepter(
            json!([{ "type": "supplier_invoice_groupe", "bankTransactionId": i64::MAX,
                           "supplierInvoiceIds": [3, 4] }]),
        )
        .await;
    assert_refus(s, &b, 400, "VALIDATION_ERROR");
}

/// P2-5, M2 (Epic 37) — un lot mixte de deux groupes clients : le premier
/// réussit, le second rend une `DbError` du dépôt. La réussie est écrite et
/// rendue dans `accepted`, l'autre refusée seule dans `failed`, le lot en `200`.
///
/// ⚠️ **État forgé, inatteignable par l'API** : les factures du second groupe
/// portent leur créance sur un compte passé en devise **après** l'écriture de
/// vente (`UPDATE accounts SET currency`), ce que l'API refuse
/// (`ACCOUNT_CURRENCY_LOCKED`, Epic 37). C'est la seule source de `DbError`
/// déterministe d'un règlement groupé client qui reste : la 43-1b a remplacé le
/// corps provisoire fournisseur, et le compte d'écart en devise est refusé en
/// amont (`ACCOUNT_IN_FOREIGN_CURRENCY`). Le contrôle central de l'écriture
/// refuse alors en `MULTI_CURRENCY_DISABLED` (exercice mono-devise), que la
/// route rend par `fx_failed_proposal`, jamais en `DATABASE_ERROR`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_lot_mixte_garde_une_erreur_de_base_dans_sa_proposition(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let x = ctx.facture("M-1", "10.00", "2026-06-01").await;
    let y = ctx.facture("M-2", "20.00", "2026-06-01").await;
    let credit = ctx.transaction("30.00", "2026-06-20").await;
    let creance_eur = compte(&pool, ctx.company_id(), "1105", "Débiteurs EUR", "Asset").await;
    let a = ctx
        .facture_sur(
            "Client EUR",
            "M-3",
            "10.00",
            "2026-06-01",
            None,
            creance_eur,
        )
        .await;
    let c = ctx
        .facture_sur(
            "Client EUR 2",
            "M-4",
            "20.00",
            "2026-06-01",
            None,
            creance_eur,
        )
        .await;
    sqlx::query("UPDATE accounts SET currency = 'EUR' WHERE id = ?")
        .bind(creance_eur)
        .execute(&pool)
        .await
        .unwrap();
    let tx = ctx.transaction("30.00", "2026-06-20").await;
    let (s, b) = ctx
        .accepter(json!([
            { "type": "invoice_group", "bankTransactionId": credit, "invoiceIds": [x, y] },
            { "type": "invoice_group", "bankTransactionId": tx, "invoiceIds": [a, c] }
        ]))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    assert_eq!(b["accepted"][0]["bankTransactionId"], credit, "{b}");
    assert_eq!(b["failed"].as_array().unwrap().len(), 1, "{b}");
    assert_eq!(b["failed"][0]["bankTransactionId"], tx, "{b}");
    assert_eq!(
        b["failed"][0]["errorCode"], "MULTI_CURRENCY_DISABLED",
        "{b}"
    );
    assert_eq!(
        b["failed"][0]["details"]["fiscalYearId"], ctx.seeded.fiscal_year_id,
        "{b}"
    );
    for id in [x, y] {
        let paid = ctx
            .scalaire(
                "SELECT COUNT(*) FROM invoices WHERE id = ? AND paid_at IS NOT NULL",
                id,
            )
            .await;
        assert_eq!(paid, 1, "la proposition réussie est écrite");
    }
    let (statut,): (String,) = sqlx::query_as("SELECT status FROM bank_transactions WHERE id = ?")
        .bind(tx)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(statut, "pending", "la refusée n'a rien écrit");
}

/// A3 — la route trie par l'ordre d'imputation **puis** tronque : une facture
/// plus récente mais échue plus tôt passe devant, et c'est elle que garde
/// `limit=1`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_ordre_d_imputation_puis_la_troncature(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ancienne = ctx.facture("I-1", "10.00", "2026-06-01").await;
    let recente = ctx.facture("I-2", "10.00", "2026-06-05").await;
    // L'ancienne est à 30 jours, la récente à vue : l'ordre SQL (date, id)
    // donne [ancienne, récente], l'ordre d'imputation [récente, ancienne].
    sqlx::query("UPDATE invoices SET due_date = '2026-07-01' WHERE id = ?")
        .bind(ancienne)
        .execute(&pool)
        .await
        .unwrap();
    let tx = ctx.transaction("20.00", "2026-06-20").await;
    assert_eq!(ctx.ids_candidats(tx, "").await, vec![recente, ancienne]);
    let (s, body) = ctx.candidats(tx, "?limit=1").await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["candidates"][0]["id"], recente);
    assert_eq!(body["candidates"].as_array().unwrap().len(), 1);
    assert_eq!(body["hasMore"], true);
}

/// E3 — cinquante factures exactement : admis.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cinquante_factures_sont_admises(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let mut ids = Vec::new();
    for n in 0..50 {
        ids.push(ctx.facture(&format!("C-{n}"), "1.00", "2026-06-01").await);
    }
    let tx = ctx.transaction("50.00", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &ids, None).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["settledInvoices"].as_array().unwrap().len(), 50);
}

/// E2, A13 — `differenceAccountId` envoyé alors que `e = 0` : ignoré, même
/// s'il est interdit (la banque) ; aucune ligne, `differenceAccountId: null`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_compte_choisi_est_ignore_sans_ecart(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("Z-1", "10.00", "2026-06-01").await;
    let b = ctx.facture("Z-2", "10.01", "2026-06-01").await;
    let tx = ctx.transaction("20.01", "2026-06-20").await;
    let (ok, ko) = ctx.grouper(tx, &[a, b], Some(ctx.ledger)).await;
    assert!(ko.is_null(), "{ko}");
    assert_eq!(ok["gap"], "0.00");
    assert_eq!(ok["differenceAccountId"], Value::Null);
    let entry = ok["journalEntryId"].as_i64().unwrap();
    assert_eq!(
        ctx.solde_ligne(entry, ctx.ledger).await,
        Some(dec("-20.01")),
        "la seule ligne de banque"
    );
    let lignes = ctx
        .scalaire(
            "SELECT COUNT(*) FROM journal_entry_lines WHERE entry_id = ?",
            entry,
        )
        .await;
    assert_eq!(lignes, 3, "banque et deux créances");
}

/// Une autre société, sa facture validée et une transaction sur son compte
/// bancaire (posées en SQL).
async fn autre_societe(ctx: &Ctx) -> (i64, i64) {
    let pool = &ctx.pool;
    let company: i64 = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client étranger', TRUE) RETURNING id",
    )
    .bind(company)
    .fetch_one(pool)
    .await
    .unwrap();
    let entry: i64 = sqlx::query_scalar("SELECT MIN(id) FROM journal_entries")
        .fetch_one(pool)
        .await
        .unwrap();
    let invoice = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, date, \
         total_amount, journal_entry_id) VALUES (?, ?, 'X-1', 'validated', '2026-06-01', 10, ?)",
    )
    .bind(company)
    .bind(contact)
    .bind(entry)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let ledger = compte(pool, company, "1020", "Banque", "Asset").await;
    let bank = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id) \
         VALUES (?, 'Autre banque', 'CH5604835012345678009', ?)",
    )
    .bind(company)
    .bind(ledger)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let import = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'x.xml', SHA2(UUID(), 256), 'camt053', '2026-06-20', '2026-06-20', ?)",
    )
    .bind(company)
    .bind(bank)
    .bind(ctx.seeded.admin_user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let tx = sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details) VALUES (?, ?, ?, '2026-06-20', 10, 'CHF', 'virement')",
    )
    .bind(company)
    .bind(import)
    .bind(bank)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    (invoice, tx)
}

/// E6 (KF-002) — la facture d'une autre société dans un groupe est
/// introuvable, rien d'écrit ; sa transaction est introuvable pour la
/// recherche.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_autre_societe_reste_invisible(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.facture("S-1", "10.00", "2026-06-01").await;
    let (etrangere, sa_transaction) = autre_societe(&ctx).await;
    let tx = ctx.transaction("20.00", "2026-06-20").await;
    let empreinte = ctx.empreinte().await;
    let (_, ko) = ctx.grouper(tx, &[a, etrangere], None).await;
    assert_eq!(ko["errorCode"], "INVOICE_NOT_FOUND", "{ko}");
    assert_eq!(ko["details"]["invoiceId"], etrangere);
    assert_eq!(ctx.empreinte().await, empreinte, "rien d'écrit");
    let (s, b) = ctx.candidats(sa_transaction, "").await;
    assert_refus(s, &b, 404, "NOT_FOUND");
}

/// A6 — le `kind` de l'annulation pour un règlement fournisseur unitaire et
/// pour une écriture (ventilation), `cancelledInvoices` vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_genre_de_l_annulation_hors_groupe(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let si = ctx.facture_achat("FF-U", "120.00", "2026-06-01").await;
    let debit = ctx.transaction("-120.00", "2026-06-20").await;
    let (s, b) = ctx
        .accepter(
            json!([{ "type": "supplier_invoice", "bankTransactionId": debit,
                           "supplierInvoiceId": si }]),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{debit}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["kind"], "supplier_settlement");
    assert_eq!(body["supplierInvoiceId"], si);
    assert_eq!(body["cancelledInvoices"], json!([]));

    let credit = ctx.transaction("30.00", "2026-06-20").await;
    let (s, b) = ctx
        .accepter(
            json!([{ "type": "split", "bankTransactionId": credit, "splits": [
            { "counterpartyAccountId": ctx.frais, "amount": "10.00", "description": "Part A" },
            { "counterpartyAccountId": ctx.seeded.accounts["3000"], "amount": "20.00",
              "description": "Part B" }
        ] }]),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 1, "{b}");
    let (s, body) = ctx
        .post(
            &format!("/api/v1/reconciliation/transactions/{credit}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["kind"], "entry");
    assert_eq!(body["invoiceId"], Value::Null);
    assert_eq!(body["cancelledInvoices"], json!([]));
}
