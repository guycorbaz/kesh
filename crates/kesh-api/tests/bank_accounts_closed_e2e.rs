//! Tests E2E HTTP du compte bancaire **clos** — Story 28-4a (#642).
//!
//! Couvre AC2 à AC6-bis : les lectures (`GET /bank-accounts` et
//! `includeClosed`, `/companies/current`, l'historique des imports, les
//! propositions), la clôture et la réouverture (effets, ordre des refus,
//! rôles et clés d'API), les refus du compte sur lui-même (`PUT`, `PATCH`,
//! `DELETE`), et **une ligne par route d'écriture** de la table de l'AC6
//! (T-a27, T-a28, T-a29) : le refus, l'absence d'écriture et de trace, puis le
//! même geste sur le compte **rouvert**, qui passe — c'est ce qui prouve que le
//! refus vient de la clôture et non d'une autre garde.
//!
//! ⚠️ **Montage** (choix C-28-4-26) : l'état qui précède le geste refusé
//! (import, règlement, paiement, lot) se pose par les **fonctions du dépôt**
//! que les routes appellent elles-mêmes, ou par les routes quand c'est simple ;
//! le geste refusé, la clôture et la réouverture passent par HTTP. Les lignes
//! dont l'état contredit un invariant (une transaction `pending` ou un lot
//! `generated` sur un compte clos : I1, I2) **cassent l'invariant à dessein**
//! — `closed_on` posé par SQL direct, comme l'aurait fait la fenêtre de #651
//! jusqu'à la v0.13.3 —, et rouvrent par la route.
//!
//! Pré-requis : MariaDB démarré.
#![allow(clippy::too_many_arguments)]

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
use kesh_db::entities::{
    BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction, NewSupplierInvoice,
    NewSupplierInvoiceLine,
};
use kesh_db::repositories::{bank_accounts, bank_imports, contacts, supplier_invoices};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

/// L'IBAN du relevé CAMT de démonstration (`v04_minimal.xml`).
const IBAN_RELEVE: &str = "CH4431999123000889012";

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

// ============================================================
// Contexte et montages
// ============================================================

/// Une société de `seed_accounting_company`, onboarding terminé, arrondi à
/// 5 centimes désactivé, un compte 1020 *Banque* (le 1100 du seed est la
/// créance, qu'un règlement ne peut pas viser), le compte créanciers 2000
/// désigné ; le serveur et un JWT `Admin`.
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
    let ledger: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable) \
         VALUES (?, '1020', 'Banque', 'Asset', 1, 1) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
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
        ledger,
    }
}

impl Ctx {
    fn company_id(&self) -> i64 {
        self.seeded.company_id
    }

    /// Une requête HTTP sous le JWT `Admin` : `(statut, corps)`.
    async fn req(
        &self,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        self.req_as(&self.jwt, methode, chemin, corps).await
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
        self.req(reqwest::Method::POST, chemin, Some(corps)).await
    }

    async fn get(&self, chemin: &str) -> (u16, Value) {
        self.req(reqwest::Method::GET, chemin, None).await
    }

    /// Un compte bancaire **non principal** (sauf `principal`), lié au 1020.
    async fn compte(&self, iban: &str, principal: bool) -> i64 {
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
            .bind(self.ledger)
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    async fn version(&self, id: i64) -> i64 {
        sqlx::query_scalar("SELECT version FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn closed_on(&self, id: i64) -> Option<NaiveDate> {
        sqlx::query_scalar("SELECT closed_on FROM bank_accounts WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    /// Clôture par la route (`closedOn` = 2026-01-31) ; exige `200`.
    async fn clore(&self, id: i64) {
        let v = self.version(id).await;
        let (s, b) = self
            .post(
                &format!("/api/v1/bank-accounts/{id}/close"),
                json!({ "version": v, "closedOn": "2026-01-31" }),
            )
            .await;
        assert_eq!(s, 200, "clôture : {b}");
    }

    /// Réouverture par la route ; exige `200`.
    async fn rouvrir(&self, id: i64) {
        let v = self.version(id).await;
        let (s, b) = self
            .post(
                &format!("/api/v1/bank-accounts/{id}/reopen"),
                json!({ "version": v }),
            )
            .await;
        assert_eq!(s, 200, "réouverture : {b}");
    }

    /// Clôture par **SQL direct** — invariant I1 ou I2 cassé à dessein (montage).
    async fn clore_sql(&self, id: i64) {
        sqlx::query("UPDATE bank_accounts SET closed_on = '2026-01-31' WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
    }

    async fn compter(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql)
            .fetch_one(&self.pool)
            .await
            .unwrap_or_else(|e| panic!("{sql} : {e}"))
    }

    async fn traces(&self) -> i64 {
        common::audit_count(&self.pool).await
    }

    /// Un import d'une transaction `pending` de `montant`, datée du jour, par
    /// le dépôt (la fonction que la route appelle) : `(import, transaction)`.
    async fn import_pending(&self, bank: i64, montant: &str, cle: &str) -> (i64, i64) {
        use std::str::FromStr;
        let jour = Utc::now().date_naive();
        let mut tx = self.pool.begin().await.unwrap();
        let (header, txs) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.company_id(),
                bank_account_id: bank,
                filename: "releve.xml".into(),
                file_hash: format!("{cle:0>64}"),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some(format!("CLOS-{cle}")),
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
                amount: rust_decimal::Decimal::from_str(montant).unwrap(),
                currency: "CHF".into(),
                reference: Some(format!("REF-{cle}")),
                details: "Mouvement".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: None,
            }],
        )
        .await
        .expect("import");
        tx.commit().await.unwrap();
        (header.id, txs[0].id)
    }

    /// Rapprochement manuel par la route, contre le compte de charge 4000.
    async fn rapprocher(&self, bank: i64, transaction: i64) {
        let (s, b) = self
            .post(
                "/api/v1/reconciliation/manual",
                json!({
                    "bankAccountId": bank,
                    "bankTransactionId": transaction,
                    "counterpartyAccountId": self.seeded.accounts["4000"],
                    "description": "Frais",
                }),
            )
            .await;
        assert_eq!(s, 200, "rapprochement manuel : {b}");
    }

    /// Une facture client validée par les routes (100.00 HT, 8,1 % → 108.10).
    async fn facture_validee(&self) -> i64 {
        let client = self.contact("Client", false).await;
        let (s, b) = self
            .post(
                "/api/v1/invoices",
                json!({
                    "contactId": client,
                    "date": Utc::now().date_naive().to_string(),
                    "lines": [
                        {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
                    ]
                }),
            )
            .await;
        assert_eq!(s, 201, "facture : {b}");
        let id = b["id"].as_i64().unwrap();
        let (s, b) = self
            .req(
                reqwest::Method::POST,
                &format!("/api/v1/invoices/{id}/validate"),
                None,
            )
            .await;
        assert_eq!(s, 200, "validation : {b}");
        id
    }

    async fn contact(&self, nom: &str, fournisseur: bool) -> i64 {
        contacts::create(
            &self.pool,
            self.seeded.admin_user_id,
            NewContact {
                company_id: self.company_id(),
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

    /// Une facture fournisseur ouverte de 100.00 (TVA 0), IBAN créancier posé
    /// (payable par lot).
    async fn facture_fournisseur(&self) -> i64 {
        let fournisseur = self.contact("Fournisseur SA", true).await;
        let id = supplier_invoices::create(
            &self.pool,
            NewSupplierInvoice {
                company_id: self.company_id(),
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
                    expense_account_id: self.seeded.accounts["4000"],
                }],
            },
            self.seeded.admin_user_id,
        )
        .await
        .expect("facture fournisseur")
        .invoice
        .id;
        sqlx::query("UPDATE supplier_invoices SET creditor_iban = ? WHERE id = ?")
            .bind("CH5604835012345678009")
            .bind(id)
            .execute(&self.pool)
            .await
            .unwrap();
        id
    }

    /// Paie une facture fournisseur par virement, par la route ; rend le statut.
    async fn payer_fournisseur(&self, id: i64, bank: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/supplier-invoices/{id}/pay"),
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": bank,
                "paymentDate": "2026-06-20"
            }),
        )
        .await
    }

    /// Règle une facture client par virement (108.10), par la route.
    async fn regler(&self, invoice: i64, bank: i64) -> (u16, Value) {
        self.post(
            &format!("/api/v1/invoices/{invoice}/settlements"),
            json!({
                "settlementType": "bank_transfer",
                "bankAccountId": bank,
                "amount": "108.10",
                "settledOn": Utc::now().date_naive().to_string(),
            }),
        )
        .await
    }
}

/// Le refus `409 BANK_ACCOUNT_CLOSED` de l'AC6, avec `details.bankAccountId`
/// et le message traduit.
fn assert_clos(status: u16, corps: &Value, bank: i64) {
    assert_eq!(status, 409, "{corps}");
    assert_eq!(corps["error"]["code"], "BANK_ACCOUNT_CLOSED", "{corps}");
    assert_eq!(corps["error"]["details"]["bankAccountId"], bank, "{corps}");
    let msg = corps["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("lecture seule") && msg.contains("rouvrez"),
        "{msg}"
    );
}

/// Un instantané des tables qu'un geste refusé ne doit pas toucher.
/// Les traces d'audit en DERNIER (index 9). Revue P1 (E-6) : les rejets
/// (`auto_match_rejected_at`, seul effet de `reject` et d'`unreject`) et les
/// lots confirmés y sont entrés.
async fn etat(ctx: &Ctx) -> [i64; 10] {
    [
        ctx.compter(
            "SELECT COUNT(*) FROM bank_transactions WHERE auto_match_rejected_at IS NOT NULL",
        )
        .await,
        ctx.compter("SELECT COUNT(*) FROM payment_batches WHERE status = 'confirmed'")
            .await,
        ctx.compter("SELECT COUNT(*) FROM bank_imports").await,
        ctx.compter("SELECT COUNT(*) FROM bank_transactions").await,
        ctx.compter("SELECT COUNT(*) FROM bank_transactions WHERE status = 'reconciled'")
            .await,
        ctx.compter("SELECT COUNT(*) FROM journal_entries").await,
        ctx.compter("SELECT COUNT(*) FROM invoice_settlements")
            .await,
        ctx.compter("SELECT COUNT(*) FROM payment_batches").await,
        ctx.compter("SELECT COUNT(*) FROM supplier_invoices WHERE status = 'paid'")
            .await,
        ctx.traces().await,
    ]
}

// ============================================================
// AC2 — les lectures
// ============================================================

/// T-a6, T-a7 (AC2) — `GET /bank-accounts` écarte le compte clos par défaut,
/// `includeClosed=true` le rend (`closedOn` sérialisé `AAAA-MM-JJ`), les deux
/// drapeaux rendent tout ; `/companies/current` l'écarte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_lectures_ecartent_ou_rendent_le_compte_clos(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let ouvert = ctx.compte("CH9300762011623852957", true).await;
    let clos = ctx.compte(IBAN_RELEVE, false).await;
    ctx.clore(clos).await;
    let archive = ctx.compte("CH5604835012345678009", false).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();

    let ids = |b: &Value| -> Vec<i64> {
        let mut v: Vec<i64> = b
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["id"].as_i64().unwrap())
            .collect();
        v.sort_unstable();
        v
    };
    let (s, b) = ctx.get("/api/v1/bank-accounts").await;
    assert_eq!(s, 200);
    assert_eq!(ids(&b), vec![ouvert], "défaut : ni archivés ni clos");
    assert!(b[0]["closedOn"].is_null());

    let (_, b) = ctx.get("/api/v1/bank-accounts?includeClosed=true").await;
    assert_eq!(ids(&b), vec![ouvert, clos]);
    let ligne = b
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["id"] == clos)
        .unwrap();
    assert_eq!(ligne["closedOn"], "2026-01-31");

    let (_, b) = ctx
        .get("/api/v1/bank-accounts?includeArchived=true&includeClosed=true")
        .await;
    assert_eq!(ids(&b), vec![ouvert, clos, archive]);

    let (s, b) = ctx.get("/api/v1/companies/current").await;
    assert_eq!(s, 200);
    let courants: Vec<i64> = b["bankAccounts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["id"].as_i64().unwrap())
        .collect();
    assert_eq!(
        courants,
        vec![ouvert],
        "T-a7 : le compte clos sort des sélecteurs"
    );
}

/// T-a36, T-a37, T-a38 (AC2) — les lectures inchangées : l'historique des
/// imports et le détail d'un import d'un compte clos (`200`), et les
/// propositions d'un compte clos (`200`, liste vide par I1 ; tue M34).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_lectures_d_un_compte_clos_restent_ouvertes(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let clos = ctx.compte(IBAN_RELEVE, false).await;
    let (import, transaction) = ctx.import_pending(clos, "-15.00", "lect").await;
    ctx.rapprocher(clos, transaction).await;
    ctx.clore(clos).await;

    let (s, b) = ctx.get("/api/v1/bank-imports").await;
    assert_eq!(s, 200, "T-a36 : {b}");
    assert!(
        b["items"]
            .as_array()
            .unwrap()
            .iter()
            .any(|i| i["id"] == import),
        "T-a36 : {b}"
    );
    let (s, b) = ctx.get(&format!("/api/v1/bank-imports/{import}")).await;
    assert_eq!(s, 200, "T-a37 : {b}");
    let (s, b) = ctx
        .get(&format!(
            "/api/v1/reconciliation/proposals?bankAccountId={clos}"
        ))
        .await;
    assert_eq!(s, 200, "T-a38 : {b}");
}

// ============================================================
// AC3, AC4 — clore et rouvrir
// ============================================================

/// T-a12, T-a13, T-a14 (AC3) — clore : `200`, `closedOn` explicite ou date du
/// jour UTC, `version + 1`, rien d'autre ; trace `bank_account.closed`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn clore_un_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let b_ = ctx.compte("CH5604835012345678009", false).await;
    let v = ctx.version(a).await;

    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{a}/close"),
            json!({ "version": v, "closedOn": "2026-01-31" }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["closedOn"], "2026-01-31");
    assert_eq!(b["version"], v + 1);
    assert_eq!(b["archived"], false);
    assert_eq!(b["journalAccountId"], ctx.ledger, "le lien reste");
    let details = common::audit_details(&pool, "bank_account", a, "bank_account.closed")
        .await
        .expect("details");
    assert_eq!(details["bank_account_id"], a);
    assert_eq!(details["closed_on"], "2026-01-31");
    assert!(details["before"]["closed_on"].is_null());
    assert_eq!(details["before"]["version"], v);
    assert_eq!(details["after"]["closed_on"], "2026-01-31");
    assert_eq!(details["after"]["version"], v + 1);

    // T-a13 — sans `closedOn` : la date du jour UTC.
    let avant = Utc::now().date_naive();
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{b_}/close"),
            json!({ "version": ctx.version(b_).await }),
        )
        .await;
    let apres = Utc::now().date_naive();
    assert_eq!(s, 200, "{b}");
    let jour: NaiveDate = b["closedOn"].as_str().unwrap().parse().unwrap();
    assert!(jour == avant || jour == apres, "{jour}");
}

/// T-a15 à T-a21 (AC3) — les refus de la clôture, dans l'ordre ; aucun
/// n'écrit ni ne trace.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_refus_de_la_cloture_dans_l_ordre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let principal = ctx.compte("CH9300762011623852957", true).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let traces = ctx.traces().await;
    let clore = |id: i64, corps: Value| {
        let chemin = format!("/api/v1/bank-accounts/{id}/close");
        let ctx = &ctx;
        async move { ctx.post(&chemin, corps).await }
    };

    // T-a15 — forme.
    let (s, _) = clore(a, json!({ "version": 0 })).await;
    assert_eq!(s, 400);
    let (s, _) = clore(a, json!({ "version": 1, "closedOn": "2026-13-01" })).await;
    assert_eq!(s, 400);
    let (s, _) = clore(a, json!({})).await;
    assert_eq!(s, 400);
    // Revue P1 (B-1) — année hors du domaine de `DATE` (1000..=9999), avec une
    // version VALIDE : sans la borne, la date partirait à la base (500).
    let v = ctx.version(a).await;
    for date in ["+10000-01-01", "0999-12-31"] {
        let (s, b) = clore(a, json!({ "version": v, "closedOn": date })).await;
        assert_eq!(
            (s, b["error"]["code"].clone()),
            (400, json!("VALIDATION_ERROR")),
            "{date} : {b}"
        );
    }
    // Revue P1 (E-5) — la forme passe avant l'existence.
    let (s, _) = clore(999_999, json!({ "version": 0 })).await;
    assert_eq!(s, 400, "forme avant 404");

    // T-a16 — archivé, autre société.
    let archive = ctx.compte("CH5604835012345678009", false).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = clore(archive, json!({ "version": ctx.version(archive).await })).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (404, json!("BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND"))
    );
    let autre_societe: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre SA', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let etranger: i64 = sqlx::query_scalar(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary) \
         VALUES (?, 'UBS', 'CH9300762011623852957', FALSE) RETURNING id",
    )
    .bind(autre_societe)
    .fetch_one(&pool)
    .await
    .unwrap();
    let (s, _) = clore(etranger, json!({ "version": 1 })).await;
    assert_eq!(s, 404);

    // T-a17 — version périmée.
    let (s, b) = clore(a, json!({ "version": ctx.version(a).await + 5 })).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("OPTIMISTIC_LOCK_CONFLICT"))
    );

    // T-a19 — principal, et l'ORDRE : principal ET pending → le principal.
    ctx.import_pending(principal, "-1.00", "prin").await;
    let (s, b) = clore(
        principal,
        json!({ "version": ctx.version(principal).await }),
    )
    .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (412, json!("BANK_ACCOUNT_CANNOT_CLOSE_PRIMARY")),
        "{b}"
    );
    assert!(
        b["error"]["message"]
            .as_str()
            .unwrap()
            .contains("QR-facture")
    );

    // T-a20 — pending, avec son décompte (une rejetée compte aussi) ; et
    // l'ORDRE (revue P1, E-5) : pending ET lot `generated` → le pending.
    ctx.import_pending(a, "-2.00", "pend").await;
    sqlx::query(
        "INSERT INTO payment_batches (company_id, bank_account_id, status, \
         requested_execution_date, msg_id, payment_info_id) \
         VALUES (?, ?, 'generated', '2026-02-01', 'M', 'P')",
    )
    .bind(ctx.company_id())
    .bind(a)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE bank_account_id = ?",
    )
    .bind(a)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = clore(a, json!({ "version": ctx.version(a).await })).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (412, json!("BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS")),
        "{b}"
    );
    assert_eq!(b["error"]["details"]["pendingCount"], 1);

    // T-a21 — lot `generated` (posé plus haut), une fois les pending levées.
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled' WHERE bank_account_id = ?")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = clore(a, json!({ "version": ctx.version(a).await })).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (412, json!("BANK_ACCOUNT_HAS_OPEN_PAYMENT_BATCHES")),
        "{b}"
    );
    assert_eq!(b["error"]["details"]["batchCount"], 1);

    assert_eq!(ctx.closed_on(a).await, None);
    assert_eq!(ctx.closed_on(principal).await, None);
    assert_eq!(ctx.traces().await, traces, "aucun refus ne trace");

    // T-a18 — déjà clos (après avoir levé le lot).
    sqlx::query("UPDATE payment_batches SET status = 'cancelled'")
        .execute(&pool)
        .await
        .unwrap();
    ctx.clore(a).await;
    let (s, b) = clore(a, json!({ "version": ctx.version(a).await })).await;
    assert_clos(s, &b, a);
    // Revue P1 (E-5) — la version passe avant « déjà clos ».
    let (s, b) = clore(a, json!({ "version": ctx.version(a).await + 5 })).await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("OPTIMISTIC_LOCK_CONFLICT")),
        "{b}"
    );
}

/// T-a22 (AC3) — rôles et clés : Consultation `403`, clé `read` `403
/// API_KEY_READ_ONLY`, clé `read-write` `200`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn roles_et_cles_d_api(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let lecteur: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES ('lecteur', ?, 'Consultation', TRUE, ?) RETURNING id",
    )
    .bind(kesh_db::test_fixtures::ADMIN_PASSWORD_HASH)
    .bind(ctx.company_id())
    .fetch_one(&pool)
    .await
    .unwrap();
    let jeton = forge_jwt(lecteur, "Consultation", ctx.company_id());
    let corps = || json!({ "version": 1 });
    let chemin = format!("/api/v1/bank-accounts/{a}/close");
    let (s, _) = ctx
        .req_as(&jeton, reqwest::Method::POST, &chemin, Some(corps()))
        .await;
    assert_eq!(s, 403);

    let (_, lecture) =
        common::create_key_via_http(&ctx.app.client, &ctx.app.base_url, &ctx.jwt, "r", "read")
            .await;
    let (s, b) = ctx
        .req_as(&lecture, reqwest::Method::POST, &chemin, Some(corps()))
        .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (403, json!("API_KEY_READ_ONLY"))
    );

    let (_, ecriture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let (s, b) = ctx
        .req_as(&ecriture, reqwest::Method::POST, &chemin, Some(corps()))
        .await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx
        .req_as(
            &ecriture,
            reqwest::Method::POST,
            &format!("/api/v1/bank-accounts/{a}/reopen"),
            Some(json!({ "version": 2 })),
        )
        .await;
    assert_eq!(s, 200, "{b}");
}

/// T-a23, T-a24, T-a25 (AC4) — rouvrir : `200`, trace qui garde la date levée ;
/// rouvrir un compte ouvert → `409 BANK_ACCOUNT_NOT_CLOSED` ; le compte rouvert
/// n'est pas principal.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rouvrir_un_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    ctx.clore(a).await;
    let v = ctx.version(a).await;
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{a}/reopen"),
            json!({ "version": v }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert!(b["closedOn"].is_null());
    assert_eq!(b["version"], v + 1);
    assert_eq!(b["isPrimary"], false, "T-a25");
    let details = common::audit_details(&pool, "bank_account", a, "bank_account.reopened")
        .await
        .expect("details");
    assert_eq!(details["before"]["closed_on"], "2026-01-31");
    assert!(details["after"]["closed_on"].is_null());

    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{a}/reopen"),
            json!({ "version": v + 1 }),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("BANK_ACCOUNT_NOT_CLOSED"))
    );
    let (s, b) = ctx
        .post(
            &format!("/api/v1/bank-accounts/{a}/reopen"),
            json!({ "version": v + 7 }),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (409, json!("OPTIMISTIC_LOCK_CONFLICT"))
    );
}

// ============================================================
// AC5 — le compte clos refuse ses propres modifications
// ============================================================

/// T-a26 (AC5) — `PUT`, `PATCH` (non identique ET identique), `DELETE` (sans
/// ET avec transactions) sur un compte clos → `409 BANK_ACCOUNT_CLOSED` ; et
/// `PUT` `isPrimary: true` sur un clos laisse le principal principal. Aucune
/// trace. Tue M21, M22, M41, M42, M43.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_compte_clos_refuse_ses_modifications(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let principal = ctx.compte("CH9300762011623852957", true).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    ctx.clore(a).await;
    let v = ctx.version(a).await;
    let traces = ctx.traces().await;
    let chemin = format!("/api/v1/bank-accounts/{a}");

    let put = |is_primary: bool| {
        json!({
            "bankName": "UBS modifiée",
            "iban": IBAN_RELEVE,
            "isPrimary": is_primary,
            "journalAccountId": ctx.ledger,
            "version": v,
        })
    };
    let (s, b) = ctx
        .req(reqwest::Method::PUT, &chemin, Some(put(false)))
        .await;
    assert_clos(s, &b, a);
    let (s, b) = ctx
        .req(reqwest::Method::PUT, &chemin, Some(put(true)))
        .await;
    assert_clos(s, &b, a);
    let principal_reste: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(principal)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        principal_reste,
        "le démotage est défait par l'abandon de la transaction"
    );

    // PATCH non identique (délier), puis identique (no-op) : 409 dans les deux cas.
    let (s, b) = ctx
        .req(
            reqwest::Method::PATCH,
            &chemin,
            Some(json!({ "journalAccountId": null, "version": v })),
        )
        .await;
    assert_clos(s, &b, a);
    let (s, b) = ctx
        .req(
            reqwest::Method::PATCH,
            &chemin,
            Some(json!({ "journalAccountId": ctx.ledger, "version": v })),
        )
        .await;
    assert_clos(s, &b, a);

    // DELETE sans transaction.
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &chemin,
            Some(json!({ "version": v })),
        )
        .await;
    assert_clos(s, &b, a);
    // DELETE avec transactions : 409, non le 412 qui proposerait la clôture.
    ctx.rouvrir(a).await;
    ctx.import_pending(a, "-3.00", "del").await;
    ctx.clore_sql(a).await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &chemin,
            Some(json!({ "version": ctx.version(a).await })),
        )
        .await;
    assert_clos(s, &b, a);

    assert_eq!(
        ctx.traces().await,
        traces + 1,
        "seule la réouverture a tracé"
    );
    let (bank_name, archived): (String, bool) =
        sqlx::query_as("SELECT bank_name, archived FROM bank_accounts WHERE id = ?")
            .bind(a)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(bank_name, "UBS");
    assert!(!archived);
}

/// AC9 — le `412` de l'archivage d'un compte qui porte des transactions garde
/// la voie de la 27-1 et propose la clôture ; le principal s'archive quand
/// tous les autres comptes sont clos (T-a8 par l'API).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_archivage_propose_la_cloture(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let principal = ctx.compte("CH9300762011623852957", true).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    ctx.import_pending(a, "-3.00", "arch").await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-accounts/{a}"),
            Some(json!({ "version": ctx.version(a).await })),
        )
        .await;
    assert_eq!(s, 412, "{b}");
    let msg = b["error"]["message"].as_str().unwrap();
    assert!(msg.contains("peuvent être supprimés"), "{msg}");
    assert!(msg.contains("clore le compte"), "{msg}");

    // Le principal : refusé tant que `a` est ouvert, permis une fois `a` clos.
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-accounts/{principal}"),
            Some(json!({ "version": ctx.version(principal).await })),
        )
        .await;
    assert_eq!(s, 412, "{b}");
    assert!(
        b["error"]["message"]
            .as_str()
            .unwrap()
            .contains("ni archivé ni clos")
    );
    sqlx::query("UPDATE bank_transactions SET status = 'reconciled'")
        .execute(&pool)
        .await
        .unwrap();
    ctx.clore(a).await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-accounts/{principal}"),
            Some(json!({ "version": ctx.version(principal).await })),
        )
        .await;
    assert_eq!(
        s, 200,
        "le principal s'archive quand les autres sont clos : {b}"
    );
}

// ============================================================
// AC6 — T-a27 : une ligne par route d'écriture
// ============================================================

fn multipart(bank: i64) -> reqwest::multipart::Form {
    let xml = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-import/tests/fixtures/camt053/v04_minimal.xml"),
    )
    .expect("fixture");
    reqwest::multipart::Form::new()
        .text("bankAccountId", bank.to_string())
        .part(
            "file",
            reqwest::multipart::Part::bytes(xml)
                .file_name("v04_minimal.xml")
                .mime_str("application/xml")
                .unwrap(),
        )
}

async fn importer_camt(ctx: &Ctx, chemin: &str, bank: i64) -> (u16, Value) {
    let resp = ctx
        .app
        .client
        .post(ctx.app.url(chemin))
        .bearer_auth(&ctx.jwt)
        .multipart(multipart(bank))
        .send()
        .await
        .unwrap();
    let s = resp.status().as_u16();
    (s, resp.json().await.unwrap_or(Value::Null))
}

/// Lignes `preview` et `create` d'import (R6, R7, W22) — tuent M11, M20.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_et_apercu_refuses(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let (s, b) = importer_camt(&ctx, "/api/v1/bank-imports/preview", a).await;
    assert_clos(s, &b, a);
    let (s, b) = importer_camt(&ctx, "/api/v1/bank-imports", a).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);

    ctx.rouvrir(a).await;
    let (s, b) = importer_camt(&ctx, "/api/v1/bank-imports/preview", a).await;
    assert_eq!(s, 200, "aperçu rouvert : {b}");
    let (s, b) = importer_camt(&ctx, "/api/v1/bank-imports", a).await;
    assert_eq!(s, 201, "import rouvert : {b}");
}

/// Lignes W24 (a) et (b) — l'ordre des refus de la suppression d'import
/// (C-28-4-24) : (a) import non rapproché d'un compte clos → `409
/// BANK_ACCOUNT_CLOSED`, import intact ; (b) import rapproché d'un compte clos
/// (construit par l'API) → le refus **du sol**, `409
/// BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS`, et le même sur le compte rouvert.
/// Tuent M12, M54, M55.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn suppression_d_import_et_ordre_des_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // (a)
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (import_a, _) = ctx.import_pending(a, "-5.00", "wa").await;
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-imports/{import_a}"),
            None,
        )
        .await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant, "import intact");
    ctx.rouvrir(a).await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-imports/{import_a}"),
            None,
        )
        .await;
    assert_eq!(s, 204, "rouvert : {b}");

    // (b)
    let c = ctx.compte("CH5604835012345678009", false).await;
    let (import_b, transaction) = ctx.import_pending(c, "-6.00", "wb").await;
    ctx.rapprocher(c, transaction).await;
    ctx.clore(c).await;
    for _ in 0..2 {
        let (s, b) = ctx
            .req(
                reqwest::Method::DELETE,
                &format!("/api/v1/bank-imports/{import_b}"),
                None,
            )
            .await;
        assert_eq!(s, 409, "{b}");
        assert_eq!(
            b["error"]["code"], "BANK_IMPORT_HAS_RECONCILED_TRANSACTIONS",
            "le refus du sol passe avant la clôture : {b}"
        );
        assert_eq!(b["error"]["details"]["reconciledCount"], 1);
        if ctx.closed_on(c).await.is_some() {
            ctx.rouvrir(c).await;
        }
    }
}

/// T-a28, T-a29 (AC6, R1, R2) — `accept` et `reject` sur un compte clos :
/// `200`, chaque proposition en `failed[]` `BANK_ACCOUNT_CLOSED`, rien de
/// modifié ; rouvert, les deux passent. Tuent M16, M17.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accepter_et_rejeter_sur_un_compte_clos(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t1) = ctx.import_pending(a, "-7.00", "acc").await;
    let (_, t1b) = ctx.import_pending(a, "-5.00", "acc2").await;
    let (_, t2) = ctx.import_pending(a, "-8.00", "rej").await;
    let (_, t2b) = ctx.import_pending(a, "-9.00", "rej2").await;
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    let ventilation = |t: i64, x: &str, y: &str| {
        json!({
            "type": "split",
            "bankTransactionId": t,
            "splits": [
                { "counterpartyAccountId": ctx.seeded.accounts["4000"], "amount": x, "description": "Frais A" },
                { "counterpartyAccountId": ctx.seeded.accounts["4000"], "amount": y, "description": "Frais B" }
            ]
        })
    };
    // Revue P1 (E-4) — DEUX propositions, et deux identifiants NON triés pour le
    // rejet : chacune doit sortir en `failed[]`, dans l'ordre du lot (accept) ou
    // trié (reject).
    let accepter = json!({
        "bankAccountId": a,
        "proposals": [ventilation(t1b, "2.00", "3.00"), ventilation(t1, "3.00", "4.00")]
    });
    let rejeter = json!({ "bankAccountId": a, "bankTransactionIds": [t2b, t2] });

    let (s, b) = ctx
        .post("/api/v1/reconciliation/accept", accepter.clone())
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["accepted"].as_array().unwrap().len(), 0);
    let ids = |v: &Value| -> Vec<i64> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|f| f["bankTransactionId"].as_i64().unwrap())
            .collect()
    };
    assert_eq!(ids(&b["failed"]), vec![t1b, t1], "{b}");
    for f in b["failed"].as_array().unwrap() {
        assert_eq!(f["errorCode"], "BANK_ACCOUNT_CLOSED");
        assert_eq!(f["details"]["bankAccountId"], a);
    }
    let (s, b) = ctx
        .post("/api/v1/reconciliation/reject", rejeter.clone())
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["rejected"].as_array().unwrap().len(), 0);
    assert_eq!(ids(&b["failed"]), vec![t2, t2b], "triés : {b}");
    for f in b["failed"].as_array().unwrap() {
        assert_eq!(f["errorCode"], "BANK_ACCOUNT_CLOSED");
    }
    assert_eq!(etat(&ctx).await, avant);

    ctx.rouvrir(a).await;
    let (s, b) = ctx.post("/api/v1/reconciliation/accept", accepter).await;
    assert_eq!(
        (s, b["accepted"].as_array().unwrap().len()),
        (200, 2),
        "{b}"
    );
    let (s, b) = ctx.post("/api/v1/reconciliation/reject", rejeter).await;
    assert_eq!(
        (s, b["rejected"].as_array().unwrap().len()),
        (200, 2),
        "{b}"
    );
}

/// Lignes `manual` et `split` (R3, R4) — tuent M18.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rapprochement_manuel_et_ventile_refuses(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t1) = ctx.import_pending(a, "-9.00", "man").await;
    let (_, t2) = ctx.import_pending(a, "-10.00", "spl").await;
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    let manuel = json!({
        "bankAccountId": a,
        "bankTransactionId": t1,
        "counterpartyAccountId": ctx.seeded.accounts["4000"],
        "description": "Frais",
    });
    let ventile = json!({
        "bankAccountId": a,
        "bankTransactionId": t2,
        "splits": [
            { "counterpartyAccountId": ctx.seeded.accounts["4000"], "amount": "4.00", "description": "A" },
            { "counterpartyAccountId": ctx.seeded.accounts["4000"], "amount": "6.00", "description": "B" }
        ]
    });
    let (s, b) = ctx
        .post("/api/v1/reconciliation/manual", manuel.clone())
        .await;
    assert_clos(s, &b, a);
    let (s, b) = ctx
        .post("/api/v1/reconciliation/split", ventile.clone())
        .await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);

    ctx.rouvrir(a).await;
    let (s, b) = ctx.post("/api/v1/reconciliation/manual", manuel).await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx.post("/api/v1/reconciliation/split", ventile).await;
    assert_eq!(s, 200, "{b}");
}

/// Ligne `unreject` (R8, W30) — une rejetée est `pending` : montage par SQL
/// direct. Tue M33.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annuler_un_rejet_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t) = ctx.import_pending(a, "-11.00", "unr").await;
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(t)
        .execute(&pool)
        .await
        .unwrap();
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/reconciliation/transactions/{t}/unreject");
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_eq!(s, 204, "{b}");
}

/// Revue P1 (E-1) — **W30 seule**, sans R8 : une clôture commitée ENTRE le
/// pré-vol (R8, compte encore ouvert) et la garde de la transaction.
/// Montage déterministe : une transaction de test tient la ligne de la
/// transaction bancaire ; l'annulation de rejet passe R8, puis attend ce verrou
/// (`find_by_id_for_account_for_update`, vue sur la sonde) ; on clôt le compte
/// par SQL direct (une rejetée est `pending`, I1), on relâche : la garde W30,
/// première lecture non verrouillante de la transaction, voit le compte clos.
/// Le refus sort par `conclude_locked_attempt` en `409 BANK_ACCOUNT_CLOSED`,
/// rien d'écrit. Tue M33-b (W30 retirée), que R8 ne voit pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annuler_un_rejet_pendant_une_cloture_refuse_par_w30(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t) = ctx.import_pending(a, "-11.00", "w30").await;
    sqlx::query("UPDATE bank_transactions SET auto_match_rejected_at = NOW(3) WHERE id = ?")
        .bind(t)
        .execute(&pool)
        .await
        .unwrap();

    let mut tenue = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM bank_transactions WHERE id = ? FOR UPDATE")
        .bind(t)
        .fetch_one(&mut *tenue)
        .await
        .unwrap();

    let url = ctx
        .app
        .url(&format!("/api/v1/reconciliation/transactions/{t}/unreject"));
    let (client, jwt) = (ctx.app.client.clone(), ctx.jwt.clone());
    let requete = tokio::spawn(async move {
        let resp = client.post(url).bearer_auth(jwt).send().await.unwrap();
        let s = resp.status().as_u16();
        (s, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &[
            "FROM bank_transactions",
            "AND bank_account_id = ? AND id = ? FOR UPDATE",
        ],
        || requete.is_finished(),
    )
    .await;
    assert!(
        vue,
        "l'annulation devait attendre le verrou de la ligne : {:?}",
        requete.await
    );
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    tenue.commit().await.unwrap();

    let (s, b) = requete.await.unwrap();
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant, "rien d'écrit");
}

/// Lignes W9 — annuler un rapprochement de type `Entry` (manuel), puis un
/// rapprochement **de facture** (l'étape (6) passerait aussi par W25) ; les
/// deux construits par l'API sur un compte ouvert, puis le compte clos. Tue M9.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annuler_un_rapprochement_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // `Entry`.
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t) = ctx.import_pending(a, "-12.00", "w9e").await;
    ctx.rapprocher(a, t).await;
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/reconciliation/transactions/{t}/cancel");
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_eq!(s, 200, "{b}");

    // Facture.
    let c = ctx.compte("CH5604835012345678009", false).await;
    let facture = ctx.facture_validee().await;
    let (_, t) = ctx.import_pending(c, "108.10", "w9f").await;
    let (s, b) = ctx
        .post(
            "/api/v1/reconciliation/accept",
            json!({
                "bankAccountId": c,
                "proposals": [{ "type": "invoice", "bankTransactionId": t, "invoiceId": facture }]
            }),
        )
        .await;
    assert_eq!(
        (s, b["accepted"].as_array().unwrap().len()),
        (200, 1),
        "{b}"
    );
    ctx.clore(c).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/reconciliation/transactions/{t}/cancel");
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_clos(s, &b, c);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(c).await;
    let (s, b) = ctx.req(reqwest::Method::POST, &chemin, None).await;
    assert_eq!(s, 200, "{b}");
}

/// Lignes W8 (règlement par virement) et W25 (annulation de ce règlement).
/// Tuent M8, M13.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reglement_client_et_son_annulation_refuses(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    // W8
    let facture = ctx.facture_validee().await;
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let (s, b) = ctx.regler(facture, a).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.regler(facture, a).await;
    assert_eq!(s, 200, "{b}");
    // W25
    let sid: i64 = sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
        .bind(facture)
        .fetch_one(&pool)
        .await
        .unwrap();
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/invoices/{facture}/settlements/{sid}/cancel");
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_eq!(s, 200, "{b}");
}

/// Lignes W27 (paiement fournisseur), W29 (annulation du règlement), W28
/// (annulation d'une facture payée par virement). Tuent M14, M15, M40.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn paiement_fournisseur_et_ses_annulations_refuses(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    // W27
    let f = ctx.facture_fournisseur().await;
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_eq!(s, 200, "{b}");
    // W29
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/supplier-invoices/{f}/settlement/cancel");
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_eq!(s, 200, "{b}");
    // W28 — repayer, clore, annuler la facture payée.
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_eq!(s, 200, "{b}");
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/supplier-invoices/{f}/cancel");
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.post(&chemin, json!({})).await;
    assert_eq!(s, 200, "{b}");
}

/// Lignes W17 (création de lot) et W20 (confirmation, lot `generated` sur un
/// compte clos : montage par SQL direct). Tuent M10-bis, M14 par la
/// confirmation.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn lots_de_paiement_refuses(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let f = ctx.facture_fournisseur().await;
    let creer = json!({
        "bankAccountId": a,
        "requestedExecutionDate": "2026-07-01",
        "supplierInvoiceIds": [f]
    });
    // W17
    ctx.clore(a).await;
    let avant = etat(&ctx).await;
    let (s, b) = ctx.post("/api/v1/payment-batches", creer.clone()).await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx.post("/api/v1/payment-batches", creer).await;
    assert_eq!(s, 200, "{b}");
    let lot = b["batch"]["id"].as_i64().expect("lot créé");
    // W20
    ctx.clore_sql(a).await;
    let avant = etat(&ctx).await;
    let chemin = format!("/api/v1/payment-batches/{lot}/confirm");
    let (s, b) = ctx
        .post(&chemin, json!({ "paymentDate": "2026-07-02" }))
        .await;
    assert_clos(s, &b, a);
    assert_eq!(etat(&ctx).await, avant);
    ctx.rouvrir(a).await;
    let (s, b) = ctx
        .post(&chemin, json!({ "paymentDate": "2026-07-02" }))
        .await;
    assert_eq!(s, 200, "{b}");
}

/// W21 (`SortieAutorisee`, C-28-4-23) — l'annulation d'un lot `generated`
/// reste permise sur un compte clos : c'est le geste qui répare ce que la
/// fenêtre de #651 a pu laisser passer jusqu'à la v0.13.3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn annuler_un_lot_reste_permis_sur_un_compte_clos(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let f = ctx.facture_fournisseur().await;
    let (s, b) = ctx
        .post(
            "/api/v1/payment-batches",
            json!({ "bankAccountId": a, "requestedExecutionDate": "2026-07-01", "supplierInvoiceIds": [f] }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let lot = b["batch"]["id"].as_i64().unwrap();
    ctx.clore_sql(a).await;
    let (s, b) = ctx
        .post(&format!("/api/v1/payment-batches/{lot}/cancel"), json!({}))
        .await;
    assert_eq!(s, 200, "{b}");
}

// ============================================================
// AC6-bis — l'archivé refusé aux gestes qui créent
// ============================================================

/// T-a42, T-a43 (AC6-bis) — un règlement de facture et un paiement
/// fournisseur par virement vers un compte **archivé** rendent l'erreur d'un
/// compte inexistant (`404 NOT_FOUND`, code relu au dev, Dev Agent Record), et
/// n'écrivent rien ; clé d'API `read-write` comprise. Tuent M46, M47.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_gestes_qui_creent_refusent_l_archive(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let inexistant = 999_999;
    let facture = ctx.facture_validee().await;
    let f = ctx.facture_fournisseur().await;
    // Le code de référence : un compte inexistant, aujourd'hui.
    let (s_ref, b_ref) = ctx.regler(facture, inexistant).await;
    let (sf_ref, bf_ref) = ctx.payer_fournisseur(f, inexistant).await;
    sqlx::query("UPDATE bank_accounts SET archived = TRUE WHERE id = ?")
        .bind(a)
        .execute(&pool)
        .await
        .unwrap();
    let avant = etat(&ctx).await;

    let (s, b) = ctx.regler(facture, a).await;
    assert_eq!(s, s_ref, "{b}");
    assert_eq!(b["error"]["code"], b_ref["error"]["code"], "{b}");
    assert_eq!((s, b["error"]["code"].clone()), (404, json!("NOT_FOUND")));
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_eq!(s, sf_ref, "{b}");
    assert_eq!(b["error"]["code"], bf_ref["error"]["code"], "{b}");
    assert_eq!((s, b["error"]["code"].clone()), (404, json!("NOT_FOUND")));

    let (_, cle) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let avant_cle = etat(&ctx).await;
    let (s, b) = ctx
        .req_as(
            &cle,
            reqwest::Method::POST,
            &format!("/api/v1/invoices/{facture}/settlements"),
            Some(json!({
                "settlementType": "bank_transfer",
                "bankAccountId": a,
                "amount": "108.10",
                "settledOn": Utc::now().date_naive().to_string(),
            })),
        )
        .await;
    // Revue P1 (A-L6) — le code aussi, pas seulement le statut.
    assert_eq!(
        (s, b["error"]["code"].clone()),
        (404, json!("NOT_FOUND")),
        "{b}"
    );
    assert_eq!(etat(&ctx).await, avant_cle);
    assert_eq!(avant[..9], avant_cle[..9], "aucune écriture métier");
}

// ============================================================
// Story 29-4b (#651, AC7) — la fenêtre des cinq annulations
// ============================================================
//
// ⚠️ **Montage** (choix C-29-4-32) : l'état à annuler se pose par les routes,
// comme plus haut, sur un compte **ouvert** ; l'annulation, elle, s'appelle
// **au dépôt** (la fonction `_in_tx` que la route appelle), dans une
// transaction que le test possède : il en voit l'erreur de dépôt et l'annule
// **explicitement** avant tout comptage. La clôture concurrente est
// `close_in_tx`, au dépôt, sans commit tant que le test ne le décide pas.

/// Une annulation, appelée au dépôt.
#[derive(Debug, Clone, Copy)]
enum Annulation {
    /// W9 — `reconciliation_cancel::cancel_in_tx`, par la transaction bancaire.
    Rapprochement(i64),
    /// W24 — `bank_imports::delete_unreconciled_in_tx`, par l'import.
    Import(i64),
    /// W25 — `invoice_settlements_write::cancel_settlement_in_tx`.
    Reglement { facture: i64, reglement: i64 },
    /// W28 — `supplier_invoices::cancel_in_tx` (facture payée par virement).
    FactureFournisseur(i64),
    /// W29 — `supplier_invoices::cancel_settlement_in_tx`.
    ReglementFournisseur(i64),
}

impl Annulation {
    async fn executer(
        self,
        tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
        company_id: i64,
        user_id: i64,
    ) -> Result<(), kesh_db::errors::DbError> {
        use kesh_db::repositories::{invoice_settlements_write, reconciliation_cancel};
        match self {
            Annulation::Rapprochement(t) => {
                reconciliation_cancel::cancel_in_tx(tx, company_id, t, user_id, None)
                    .await
                    .map(|_| ())
            }
            Annulation::Import(i) => {
                match bank_imports::delete_unreconciled_in_tx(tx, company_id, i).await? {
                    bank_imports::DeleteBankImportOutcome::Deleted(_) => Ok(()),
                    autre => panic!("montage : suppression d'import — {autre:?}"),
                }
            }
            Annulation::Reglement { facture, reglement } => {
                invoice_settlements_write::cancel_settlement_in_tx(
                    tx, company_id, facture, reglement, user_id,
                )
                .await
                .map(|_| ())
            }
            Annulation::FactureFournisseur(f) => {
                supplier_invoices::cancel_in_tx(tx, company_id, f, user_id)
                    .await
                    .map(|_| ())
            }
            Annulation::ReglementFournisseur(f) => {
                supplier_invoices::cancel_settlement_in_tx(tx, company_id, f, user_id)
                    .await
                    .map(|_| ())
            }
        }
    }
}

/// L'annulation, en tâche, dans sa propre transaction : commit si elle
/// aboutit, `rollback` **explicite** sinon (un abandon par `drop` est
/// asynchrone et laisserait les comptages dépendre de l'ordonnancement).
fn annulation_en_tache(
    ctx: &Ctx,
    annulation: Annulation,
) -> tokio::task::JoinHandle<Result<(), kesh_db::errors::DbError>> {
    let (pool, company_id, user_id) =
        (ctx.pool.clone(), ctx.company_id(), ctx.seeded.admin_user_id);
    tokio::spawn(async move {
        let mut tx = pool.begin().await.unwrap();
        let r = annulation.executer(&mut tx, company_id, user_id).await;
        if r.is_ok() {
            tx.commit().await.unwrap();
        } else {
            tx.rollback().await.unwrap();
        }
        r
    })
}

/// Le motif de la revérification des annulations
/// (`bank_accounts::RECHECK_OPEN_LOCKED_SQL`) dans la liste des processus.
const MOTIFS_REVERIFICATION: [&str; 2] =
    ["SELECT closed_on FROM bank_accounts", "LOCK IN SHARE MODE"];

/// T29b-1, le patron (AC7, « fenêtre ») : (1) une clôture `C` tient la ligne
/// du compte en exclusif, sans commiter ; (2) l'annulation démarre — sa vue
/// naît à la garde non verrouillante de la 28-4, qui voit le compte ouvert ;
/// (3) elle est vue **en attente sur sa revérification**, ou terminée (au
/// sol, elle n'attend pas : la borne de dix secondes n'est pas consommée) ;
/// (4) `C` commite ; (5) l'issue de l'annulation est rendue — sa transaction
/// déjà annulée par la tâche si elle a échoué.
async fn annuler_pendant_une_cloture(
    ctx: &Ctx,
    compte: i64,
    annulation: Annulation,
) -> Result<(), kesh_db::errors::DbError> {
    use kesh_db::repositories::bank_accounts::ClosureOutcome;
    let v = ctx.version(compte).await as i32;
    let mut cloture = ctx.pool.begin().await.unwrap();
    let r = bank_accounts::close_in_tx(
        &mut cloture,
        ctx.company_id(),
        compte,
        v,
        NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
    )
    .await
    .unwrap();
    assert!(
        matches!(r, ClosureOutcome::Done { .. }),
        "montage : la clôture doit passer — {r:?}"
    );
    let tache = annulation_en_tache(ctx, annulation);
    let _attend = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &ctx.pool,
        &MOTIFS_REVERIFICATION,
        || tache.is_finished(),
    )
    .await;
    cloture.commit().await.unwrap();
    tache.await.expect("tâche d'annulation")
}

/// L'issue attendue de T29b-1 : refus `BankAccountClosed` sur `compte`, rien
/// d'écrit (instantané [`etat`] inchangé), le compte clos.
async fn exiger_le_refus(
    ctx: &Ctx,
    compte: i64,
    avant: [i64; 10],
    r: Result<(), kesh_db::errors::DbError>,
    geste: &str,
) {
    assert!(
        matches!(
            r,
            Err(kesh_db::errors::DbError::BankAccountClosed { bank_account_id })
                if bank_account_id == compte
        ),
        "⛔ {geste} commencée avant la clôture devait être refusée — reçu {r:?}"
    );
    assert_eq!(etat(ctx).await, avant, "{geste} : rien d'écrit");
    assert!(ctx.closed_on(compte).await.is_some(), "le compte est clos");
}

/// T29b-1 (W9) — l'annulation d'un rapprochement de type `Entry` (la
/// transaction rapprochée, aucune `pending` : la clôture passe). **Rouge au
/// sol** : l'annulation aboutit et remet une transaction `pending` sur le
/// compte qu'on clôt. Tue M29-8.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_fenetre_de_l_annulation_d_un_rapprochement_est_fermee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t) = ctx.import_pending(a, "-12.00", "f9").await;
    ctx.rapprocher(a, t).await;
    let avant = etat(&ctx).await;
    let r = annuler_pendant_une_cloture(&ctx, a, Annulation::Rapprochement(t)).await;
    exiger_le_refus(&ctx, a, avant, r, "l'annulation du rapprochement").await;
}

/// T29b-1 (W24) — la suppression d'un import **vide** (la seule fenêtre de
/// W24 : un import qui porte des transactions les a `pending`, et la clôture
/// refuse). **Rouge au sol** : l'import disparaît d'un compte clos. Tue
/// M29-9.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_fenetre_de_la_suppression_d_un_import_vide_est_fermee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let import: i64 = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'vide.xml', ?, 'CAMT053_V04', '2026-01-01', '2026-01-31', 0, ?)",
    )
    .bind(ctx.company_id())
    .bind(a)
    .bind(format!("{:0>64}", "f24"))
    .bind(ctx.seeded.admin_user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let avant = etat(&ctx).await;
    let r = annuler_pendant_une_cloture(&ctx, a, Annulation::Import(import)).await;
    exiger_le_refus(&ctx, a, avant, r, "la suppression de l'import vide").await;
}

/// T29b-1 (W25) — l'annulation d'un règlement client par virement. **Rouge au
/// sol** : le règlement est annulé sur un compte clos. Tue M29-10.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_fenetre_de_l_annulation_d_un_reglement_client_est_fermee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let facture = ctx.facture_validee().await;
    let (s, b) = ctx.regler(facture, a).await;
    assert_eq!(s, 200, "règlement : {b}");
    let reglement: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    let avant = etat(&ctx).await;
    let r =
        annuler_pendant_une_cloture(&ctx, a, Annulation::Reglement { facture, reglement }).await;
    exiger_le_refus(&ctx, a, avant, r, "l'annulation du règlement").await;
}

/// T29b-1 (W28) — l'annulation d'une facture fournisseur payée par virement.
/// **Rouge au sol**. Tue M29-11.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_fenetre_de_l_annulation_d_une_facture_fournisseur_payee_est_fermee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let f = ctx.facture_fournisseur().await;
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_eq!(s, 200, "paiement : {b}");
    let avant = etat(&ctx).await;
    let r = annuler_pendant_une_cloture(&ctx, a, Annulation::FactureFournisseur(f)).await;
    exiger_le_refus(&ctx, a, avant, r, "l'annulation de la facture payée").await;
}

/// T29b-1 (W29) — l'annulation du règlement d'une facture fournisseur par
/// virement. **Rouge au sol**. Tue M29-12.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_fenetre_de_l_annulation_d_un_reglement_fournisseur_est_fermee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let f = ctx.facture_fournisseur().await;
    let (s, b) = ctx.payer_fournisseur(f, a).await;
    assert_eq!(s, 200, "paiement : {b}");
    let avant = etat(&ctx).await;
    let r = annuler_pendant_une_cloture(&ctx, a, Annulation::ReglementFournisseur(f)).await;
    exiger_le_refus(&ctx, a, avant, r, "l'annulation du règlement fournisseur").await;
}

/// T29b-2, le patron (AC7, « autre entrelacement ») : l'annulation, appelée
/// dans une transaction de test **non commitée**, a passé sa revérification
/// (elle tient la ligne du compte en partagé) ; la clôture, lancée en tâche,
/// **doit** être vue en attente sur son `FOR UPDATE` ; au commit de
/// l'annulation, la clôture rend son issue.
async fn clore_apres_une_annulation(
    ctx: &Ctx,
    compte: i64,
    annulation: Annulation,
) -> kesh_db::repositories::bank_accounts::ClosureOutcome {
    use kesh_db::repositories::bank_accounts::ClosureOutcome;
    let mut tx = ctx.pool.begin().await.unwrap();
    annulation
        .executer(&mut tx, ctx.company_id(), ctx.seeded.admin_user_id)
        .await
        .expect("l'annulation, sur un compte ouvert, passe");
    let v = ctx.version(compte).await as i32;
    let (pool, company_id) = (ctx.pool.clone(), ctx.company_id());
    let cloture = tokio::spawn(async move {
        let mut c = pool.begin().await.unwrap();
        let r = bank_accounts::close_in_tx(
            &mut c,
            company_id,
            compte,
            v,
            NaiveDate::from_ymd_opt(2026, 1, 31).unwrap(),
        )
        .await;
        if matches!(r, Ok(ClosureOutcome::Done { .. })) {
            c.commit().await.unwrap();
        } else {
            c.rollback().await.unwrap();
        }
        r
    });
    let vue = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &ctx.pool,
        &["FROM bank_accounts", "archived = FALSE FOR UPDATE"],
        || cloture.is_finished(),
    )
    .await;
    assert!(
        vue,
        "⛔ la clôture devait attendre l'annulation, qui tient le compte en partagé — reçu {:?}",
        cloture.await.map(|r| r.map(|_| ()))
    );
    tx.commit().await.unwrap();
    cloture.await.expect("tâche de clôture").expect("clôture")
}

/// T29b-2 (W9) — l'annulation d'un rapprochement passée d'abord : la clôture
/// attend son commit, puis voit la transaction redevenue `pending` et refuse
/// (`412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` à la route). Tue M29-13
/// (revérification non verrouillante : la clôture n'attendrait pas, et
/// rendrait `Done` sur la vue d'avant l'annulation).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cloture_attend_l_annulation_d_un_rapprochement_puis_la_voit(pool: MySqlPool) {
    use kesh_db::repositories::bank_accounts::{ClosureOutcome, ClosureRefusal};
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let (_, t) = ctx.import_pending(a, "-12.00", "e9").await;
    ctx.rapprocher(a, t).await;
    let r = clore_apres_une_annulation(&ctx, a, Annulation::Rapprochement(t)).await;
    assert!(
        matches!(
            r,
            ClosureOutcome::Refused(ClosureRefusal::PendingTransactions { pending_count: 1 })
        ),
        "⛔ la clôture devait voir la transaction redevenue pending — reçu {r:?}"
    );
    assert!(ctx.closed_on(a).await.is_none(), "le compte reste ouvert");
}

/// T29b-2 (W25) — l'annulation d'un règlement passée d'abord : la clôture
/// attend son commit, puis passe — l'annulation a eu lieu sur un compte
/// encore ouvert, l'ordre juste. ⚠️ L'issue seule ne distingue pas une
/// revérification non verrouillante (M29-13 : la clôture, qui ne compte ni
/// règlements ni écritures, rendrait `Done` aussi) ; c'est l'**attente** de la
/// clôture, exigée par le montage, qui la distingue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cloture_attend_l_annulation_d_un_reglement_puis_passe(pool: MySqlPool) {
    use kesh_db::repositories::bank_accounts::ClosureOutcome;
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let facture = ctx.facture_validee().await;
    let (s, b) = ctx.regler(facture, a).await;
    assert_eq!(s, 200, "règlement : {b}");
    let reglement: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    let r = clore_apres_une_annulation(&ctx, a, Annulation::Reglement { facture, reglement }).await;
    assert!(
        matches!(r, ClosureOutcome::Done { .. }),
        "⛔ la clôture devait passer après l'annulation — reçu {r:?}"
    );
    assert_eq!(
        ctx.compter(&format!(
            "SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = {facture}"
        ))
        .await,
        0,
        "le règlement est annulé"
    );
}

/// T29b-5 (AC7, W25) — **une annulation sur un compte archivé passe**,
/// revérification comprise : la revérification des annulations ne lit que
/// `closed_on`, jamais `archived` (un compte archivé ne se rouvre pas ;
/// refuser l'annulation d'un de ses règlements la rendrait définitivement
/// impossible). Montage : règlement par virement vers `A`, puis `A` archivé
/// par la route (l'archivage ne compte que les transactions bancaires).
/// Tue M29-24 (W25 appelle la variante « archivé » : `NotFound`, `404`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_annulation_de_reglement_passe_sur_un_compte_archive(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let a = ctx.compte(IBAN_RELEVE, false).await;
    let facture = ctx.facture_validee().await;
    let (s, b) = ctx.regler(facture, a).await;
    assert_eq!(s, 200, "règlement : {b}");
    let reglement: i64 =
        sqlx::query_scalar("SELECT id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(facture)
            .fetch_one(&pool)
            .await
            .unwrap();
    let v = ctx.version(a).await;
    let (s, b) = ctx
        .req(
            reqwest::Method::DELETE,
            &format!("/api/v1/bank-accounts/{a}"),
            Some(json!({ "version": v })),
        )
        .await;
    assert_eq!(
        (s, b["archived"].clone()),
        (200, json!(true)),
        "archivage : {b}"
    );

    let (s, b) = ctx
        .post(
            &format!("/api/v1/invoices/{facture}/settlements/{reglement}/cancel"),
            json!({}),
        )
        .await;
    assert_eq!(
        s, 200,
        "⛔ l'annulation sur un compte archivé doit passer : {b}"
    );
    assert_eq!(
        ctx.compter(&format!(
            "SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = {facture}"
        ))
        .await,
        0,
        "le règlement est annulé"
    );
}
