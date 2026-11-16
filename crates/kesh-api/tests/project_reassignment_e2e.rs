//! Story 36-16a (#1167) — changer le compte analytique d'une facture client,
//! même payée, par `POST /api/v1/invoices/{id}/reassign-project`.
//!
//! ⛔ **Les états se construisent par les routes** (création, validation,
//! règlement, encaissement par rapprochement, solde du reste, avoir, annulation
//! d'un règlement, verrou des livres) ou par les fonctions de dépôt qu'elles
//! appellent (relevé bancaire). Trois exceptions, écrites au registre
//! (C-36-16a-2) : la configuration (contact, compte bancaire et son compte au
//! grand livre), la clôture d'un exercice (passée en SQL, comme les tests de
//! concurrence de la Story 15-8a : la route exige un exercice suivant et un
//! bilan) et la ligne « divergente » d'AC11, qu'aucune route ne sait produire
//! (donnée historique).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{Duration, NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{
    BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction,
};
use kesh_db::repositories::{bank_accounts, bank_imports};
use kesh_db::test_fixtures::{SeededCompany, designate_rounding_account, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

mod common {
    pub mod audit_traces;
    pub mod jour;
}

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

/// Le contexte d'un test : le serveur, le jeton de l'Administrateur, la société
/// semée, un client, deux comptes analytiques P et Q, un compte bancaire relié au
/// grand livre.
struct Ctx {
    base_url: String,
    client: reqwest::Client,
    admin: String,
    seeded: SeededCompany,
    contact_id: i64,
    p: i64,
    q: i64,
    bank_account_id: i64,
}

impl Ctx {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// Un `POST` sous `token`, rendu brut : statut et corps.
    async fn post_as(&self, token: &str, path: &str, body: Value) -> (u16, Value) {
        let resp = self
            .client
            .post(self.url(path))
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        let text = resp.text().await.unwrap();
        (
            status,
            serde_json::from_str(&text).unwrap_or(Value::String(text)),
        )
    }

    /// Un `POST` de l'Administrateur qui doit réussir.
    async fn post_ok(&self, path: &str, body: Value) -> Value {
        let (status, body) = self.post_as(&self.admin, path, body).await;
        assert!(status == 200 || status == 201, "{path} : {status} {body}");
        body
    }

    async fn get(&self, path: &str) -> Value {
        let resp = self
            .client
            .get(self.url(path))
            .bearer_auth(&self.admin)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "{path}");
        resp.json().await.unwrap()
    }

    async fn invoice(&self, id: i64) -> Value {
        self.get(&format!("/api/v1/invoices/{id}")).await
    }

    async fn version(&self, id: i64) -> i64 {
        self.invoice(id).await["version"].as_i64().unwrap()
    }

    /// Une facture brouillon de 100.— sans TVA, au compte analytique `project`.
    async fn draft(&self, project: Option<i64>, date: NaiveDate) -> i64 {
        let body = self
            .post_ok(
                "/api/v1/invoices",
                json!({
                    "contactId": self.contact_id,
                    "date": date,
                    "projectId": project,
                    "lines": [{
                        "description": "Prestation",
                        "quantity": "1",
                        "unitPrice": "100.00",
                        "vatRate": "0",
                    }],
                }),
            )
            .await;
        body["id"].as_i64().unwrap()
    }

    /// Une facture validée de 100.—, datée de `date`, au compte `project`.
    async fn validated(&self, project: Option<i64>, date: NaiveDate) -> i64 {
        let id = self.draft(project, date).await;
        self.post_ok(&format!("/api/v1/invoices/{id}/validate"), json!({}))
            .await;
        id
    }

    async fn settle(&self, id: i64, amount: &str, date: NaiveDate) {
        let version = self.version(id).await;
        self.post_ok(
            &format!("/api/v1/invoices/{id}/settlements"),
            json!({
                "settlementType": "internal_account",
                "accountId": self.seeded.accounts["1000"],
                "amount": amount,
                "settledOn": date,
                "version": version,
            }),
        )
        .await;
    }

    /// Une transaction bancaire créditée de `amount`, au relevé du compte
    /// bancaire (dépôt de l'import), référencée au numéro de la facture.
    async fn bank_transaction(
        &self,
        pool: &MySqlPool,
        id: i64,
        amount: Decimal,
        date: NaiveDate,
    ) -> i64 {
        let number = self.invoice(id).await["invoiceNumber"]
            .as_str()
            .unwrap()
            .to_string();
        let mut tx = pool.begin().await.unwrap();
        let (_, inserted) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id: self.seeded.company_id,
                bank_account_id: self.bank_account_id,
                filename: "releve.xml".into(),
                file_hash: format!("{:0>64}", format!("{id}{}", date.format("%Y%m%d"))),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some(format!("STMT-{id}")),
                period_from: date,
                period_to: date,
                opening_balance: Some(dec!(0)),
                closing_balance: Some(amount),
                transaction_count: 1,
                imported_by_user_id: self.seeded.admin_user_id,
            },
            vec![NewBankTransaction {
                company_id: self.seeded.company_id,
                bank_account_id: self.bank_account_id,
                booking_date: date,
                value_date: Some(date),
                amount,
                currency: "CHF".into(),
                reference: Some(number),
                details: "Paiement client".into(),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: Some("Client 36-16a".into()),
            }],
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();
        inserted[0].id
    }

    fn accept_body(&self, tx_id: i64, invoice_id: i64) -> Value {
        json!({
            "bankAccountId": self.bank_account_id,
            "proposals": [{ "type": "invoice", "bankTransactionId": tx_id, "invoiceId": invoice_id }],
        })
    }

    /// Encaisse `amount` par rapprochement (`POST /reconciliation/accept`).
    async fn accept(&self, pool: &MySqlPool, id: i64, amount: Decimal, date: NaiveDate) {
        let tx_id = self.bank_transaction(pool, id, amount, date).await;
        let body = self
            .post_ok("/api/v1/reconciliation/accept", self.accept_body(tx_id, id))
            .await;
        assert_eq!(
            body["accepted"].as_array().map(Vec::len),
            Some(1),
            "rapprochement : {body}"
        );
    }

    /// Solde le reste d'arrondi (`POST /write-off`, nature `rounding`).
    async fn write_off(&self, id: i64, date: NaiveDate) {
        let version = self.version(id).await;
        self.post_ok(
            &format!("/api/v1/invoices/{id}/write-off"),
            json!({ "nature": "rounding", "settledOn": date, "version": version }),
        )
        .await;
    }

    async fn credit_note(&self, id: i64, date: NaiveDate) {
        self.post_ok(
            "/api/v1/credit-notes",
            json!({ "invoiceId": id, "date": date }),
        )
        .await;
    }

    /// Le geste, brut.
    async fn reassign(
        &self,
        token: &str,
        id: i64,
        project: Option<i64>,
        version: i64,
        acknowledge: Option<bool>,
    ) -> (u16, Value) {
        let mut body = json!({ "projectId": project, "version": version });
        if let Some(ack) = acknowledge {
            body["acknowledgeClosedPeriods"] = json!(ack);
        }
        self.post_as(
            token,
            &format!("/api/v1/invoices/{id}/reassign-project"),
            body,
        )
        .await
    }

    /// Le geste de l'Administrateur, à la version courante, sans accusé.
    async fn reassign_now(&self, id: i64, project: Option<i64>) -> (u16, Value) {
        let version = self.version(id).await;
        self.reassign(&self.admin, id, project, version, None).await
    }

    /// Un utilisateur de `role` et son jeton de session.
    async fn user(&self, pool: &MySqlPool, username: &str, role: &str) -> String {
        sqlx::query(
            "INSERT INTO users (username, password_hash, role, active, company_id) \
             SELECT ?, password_hash, ?, TRUE, company_id FROM users WHERE username = 'admin'",
        )
        .bind(username)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
        login(&self.client, &self.base_url, username).await
    }
}

async fn login(client: &reqwest::Client, base_url: &str, username: &str) -> String {
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    body["accessToken"]
        .as_str()
        .unwrap_or_else(|| panic!("login {username} : {body}"))
        .to_string()
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.unwrap();
    designate_rounding_account(pool, seeded.company_id)
        .await
        .unwrap();
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
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
    let state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state, "nonexistent-static-dir".to_string());
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
    let base_url = format!("http://{addr}");
    let client = reqwest::Client::new();
    let admin = login(&client, &base_url, "admin").await;

    // Configuration (hors des états du geste) : le client, le compte bancaire
    // et son compte au grand livre.
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client 36-16a', TRUE) RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let bank_account_id = bank_accounts::create(
        pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque test".into(),
            iban: "CH4431999123000889012".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    let ledger: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '1020', 'Banque compte courant', 'Asset') RETURNING id",
    )
    .bind(seeded.company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(ledger)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .unwrap();

    let mut ctx = Ctx {
        base_url,
        client,
        admin,
        seeded,
        contact_id,
        p: 0,
        q: 0,
        bank_account_id,
    };
    ctx.p = project(&ctx, "P").await;
    ctx.q = project(&ctx, "Q").await;
    ctx
}

/// Un compte analytique, par la route.
async fn project(ctx: &Ctx, code: &str) -> i64 {
    ctx.post_ok(
        "/api/v1/projects",
        json!({ "code": code, "name": format!("Compte {code}") }),
    )
    .await["id"]
        .as_i64()
        .unwrap()
}

/// Le jour de la vente : soixante jours avant le jour de l'horloge de test.
fn sale_day() -> NaiveDate {
    common::jour::aujourd_hui_test() - Duration::days(60)
}

/// AC1 — une facture **payée** au compte P : la vente, un règlement manuel de
/// 40.—, un encaissement rapproché de 59.97, le solde du reste d'arrondi.
async fn paid_invoice(pool: &MySqlPool, ctx: &Ctx) -> i64 {
    let d = sale_day();
    let id = ctx.validated(Some(ctx.p), d).await;
    ctx.settle(id, "40.00", d + Duration::days(5)).await;
    ctx.accept(pool, id, dec!(59.97), d + Duration::days(10))
        .await;
    ctx.write_off(id, d + Duration::days(15)).await;
    assert!(
        !ctx.invoice(id).await["paidAt"].is_null(),
        "montage : la facture est payée"
    );
    id
}

/// AC2 — une facture **annulée par avoir** au compte P.
async fn cancelled_invoice(ctx: &Ctx) -> i64 {
    let d = sale_day();
    let id = ctx.validated(Some(ctx.p), d).await;
    ctx.credit_note(id, d + Duration::days(5)).await;
    assert_eq!(ctx.invoice(id).await["status"], "cancelled", "montage");
    id
}

// ---------------------------------------------------------------------------
// Lectures
// ---------------------------------------------------------------------------

/// Les écritures que la facture relie aujourd'hui : vente, règlements (solde
/// compris), avoir — triées.
async fn linked_entries(pool: &MySqlPool, id: i64) -> Vec<i64> {
    let mut ids: Vec<i64> = sqlx::query_scalar(
        "SELECT journal_entry_id FROM invoices WHERE id = ? AND journal_entry_id IS NOT NULL \
         UNION SELECT journal_entry_id FROM invoice_settlements WHERE invoice_id = ? \
         UNION SELECT journal_entry_id FROM credit_notes \
               WHERE invoice_id = ? AND journal_entry_id IS NOT NULL",
    )
    .bind(id)
    .bind(id)
    .bind(id)
    .fetch_all(pool)
    .await
    .unwrap();
    ids.sort_unstable();
    ids
}

/// Le compte analytique de chaque ligne des écritures `ids`.
async fn line_projects(pool: &MySqlPool, ids: &[i64]) -> Vec<Option<i64>> {
    let mut out = Vec::new();
    for id in ids {
        let rows: Vec<Option<i64>> = sqlx::query_scalar(
            "SELECT project_id FROM journal_entry_lines WHERE entry_id = ? ORDER BY id",
        )
        .bind(id)
        .fetch_all(pool)
        .await
        .unwrap();
        assert!(!rows.is_empty(), "l'écriture {id} a des lignes");
        out.extend(rows);
    }
    out
}

async fn entry_versions(pool: &MySqlPool, ids: &[i64]) -> Vec<i32> {
    let mut out = Vec::new();
    for id in ids {
        out.push(
            sqlx::query_scalar("SELECT version FROM journal_entries WHERE id = ?")
                .bind(id)
                .fetch_one(pool)
                .await
                .unwrap(),
        );
    }
    out
}

fn ids_of(v: &Value) -> Vec<i64> {
    let mut ids: Vec<i64> = v
        .as_array()
        .unwrap_or_else(|| panic!("tableau attendu : {v}"))
        .iter()
        .map(|x| x.as_i64().unwrap())
        .collect();
    ids.sort_unstable();
    ids
}

/// AC3 — tout ce que le geste ne doit pas toucher, hors `project_id`, `version`
/// et `updated_at` : écritures, lignes (lettrage compris), règlements,
/// rapprochements, factures.
async fn snapshot(pool: &MySqlPool, company_id: i64) -> Vec<String> {
    let mut out: Vec<String> = sqlx::query_scalar(
        "SELECT CONCAT_WS('|', id, fiscal_year_id, entry_number, entry_date, journal, description, \
                IFNULL(reverses_entry_id, '-')) \
         FROM journal_entries WHERE company_id = ? ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .unwrap();
    out.extend(
        sqlx::query_scalar::<_, String>(
            "SELECT CONCAT_WS('|', l.id, l.entry_id, l.account_id, l.line_order, l.debit, l.credit, \
                    IFNULL(l.lettering_key, '-'), IFNULL(l.lettering_origin, '-')) \
             FROM journal_entry_lines l JOIN journal_entries e ON e.id = l.entry_id \
             WHERE e.company_id = ? ORDER BY l.id",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .unwrap(),
    );
    out.extend(
        sqlx::query_scalar::<_, String>(
            "SELECT CONCAT_WS('|', id, invoice_id, journal_entry_id, amount, settled_on, \
                    settlement_type, IFNULL(write_off_nature, '-')) \
             FROM invoice_settlements WHERE company_id = ? ORDER BY id",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .unwrap(),
    );
    out.extend(
        sqlx::query_scalar::<_, String>(
            "SELECT CONCAT_WS('|', id, status, IFNULL(matched_entry_id, '-')) \
             FROM bank_transactions WHERE company_id = ? ORDER BY id",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .unwrap(),
    );
    out.extend(
        sqlx::query_scalar::<_, String>(
            "SELECT CONCAT_WS('|', id, status, IFNULL(invoice_number, '-'), date, \
                    IFNULL(paid_at, '-'), IFNULL(journal_entry_id, '-'), total_amount) \
             FROM invoices WHERE company_id = ? ORDER BY id",
        )
        .bind(company_id)
        .fetch_all(pool)
        .await
        .unwrap(),
    );
    out
}

async fn audits(pool: &MySqlPool, action: &str) -> Vec<(i64, String, Option<i64>, Value)> {
    let rows: Vec<(i64, String, Option<i64>, Option<Value>)> = sqlx::query_as(
        "SELECT entity_id, actor_type, actor_api_key_id, details_json FROM audit_log \
         WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|(id, t, k, d)| (id, t, k, d.unwrap_or(Value::Null)))
        .collect()
}

/// Passe l'exercice semé à `Closed` (simulé : cf. l'en-tête du fichier).
async fn close_fiscal_year(pool: &MySqlPool, ctx: &Ctx) {
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn lock_books(ctx: &Ctx, through: NaiveDate) {
    ctx.post_ok(
        "/api/v1/companies/current/books-lock",
        json!({ "through": through }),
    )
    .await;
}

// ---------------------------------------------------------------------------
// T6 — l'invariant D3 sur les créations automatiques
// ---------------------------------------------------------------------------

/// T6 (D3) — les écritures que Kesh crée pour une facture portent le compte de
/// la facture sur **chaque** ligne : vente, règlement, rapprochement et solde
/// d'une payée ; vente et avoir d'une annulée. C'est l'invariant que le geste
/// prolonge (toutes les lignes prennent le compte visé). Sonde : `project_id:
/// None` à la création de l'avoir rougit ce test.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn automatic_entries_carry_the_invoice_account_on_every_line(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    for id in [
        paid_invoice(&pool, &ctx).await,
        cancelled_invoice(&ctx).await,
    ] {
        let entries = linked_entries(&pool, id).await;
        let projects = line_projects(&pool, &entries).await;
        assert!(
            projects.iter().all(|p| *p == Some(ctx.p)),
            "facture {id} : {projects:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// T1 — l'ensemble des écritures
// ---------------------------------------------------------------------------

/// T1a (AC1) — une facture payée : la facture et chaque ligne de la vente, des
/// règlements (le rapproché compris) et du solde passent à Q ; la réponse liste
/// ces écritures, et rien d'autre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_paid_invoice_moves_with_all_its_entries(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let entries = linked_entries(&pool, id).await;
    assert_eq!(entries.len(), 4, "montage : vente, deux règlements, solde");
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.p))
    );

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(body["invoice"]["projectId"], ctx.q);
    assert_eq!(ids_of(&body["journalEntryIds"]), entries);
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.q)),
        "toutes les lignes passent à Q"
    );
    assert_eq!(
        body["invoice"],
        ctx.invoice(id).await,
        "la réponse égale le GET qui la suit"
    );
}

/// T1b (AC2) — une facture annulée par avoir : la facture, la vente et l'avoir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_cancelled_invoice_moves_with_its_sale_and_credit_note(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = cancelled_invoice(&ctx).await;
    let entries = linked_entries(&pool, id).await;
    assert_eq!(entries.len(), 2, "montage : vente et avoir");

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(ids_of(&body["journalEntryIds"]), entries);
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.q))
    );
    assert_eq!(ctx.invoice(id).await["projectId"], ctx.q);
}

// ---------------------------------------------------------------------------
// T2, T3 — rien d'autre ; les versions
// ---------------------------------------------------------------------------

/// T2 (AC3) — rien ne change hors `project_id`, `version` et `updated_at`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nothing_else_changes(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let before = snapshot(&pool, ctx.seeded.company_id).await;
    assert!(
        before.iter().any(|r| r.contains("|document")),
        "montage : le lettrage de la pièce est posé"
    );

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_eq!(status, 200, "{body}");

    assert_eq!(snapshot(&pool, ctx.seeded.company_id).await, before);
}

/// T3 (AC4) — la facture +1, chaque écriture touchée +1, celles d'une autre
/// facture inchangées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn versions_move_by_one_and_only_for_what_changes(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let other = ctx.validated(Some(ctx.p), sale_day()).await;
    let entries = linked_entries(&pool, id).await;
    let others = linked_entries(&pool, other).await;
    let before = entry_versions(&pool, &entries).await;
    let others_before = entry_versions(&pool, &others).await;
    let version = ctx.version(id).await;

    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, None)
        .await;
    assert_eq!(status, 200, "{body}");

    assert_eq!(body["invoice"]["version"].as_i64().unwrap(), version + 1);
    assert_eq!(
        entry_versions(&pool, &entries).await,
        before.iter().map(|v| v + 1).collect::<Vec<_>>()
    );
    assert_eq!(entry_versions(&pool, &others).await, others_before);
}

// ---------------------------------------------------------------------------
// T4 — l'encaissement concurrent
// ---------------------------------------------------------------------------

/// T4 (AC5) — une acceptation par rapprochement dont l'instantané précède le
/// geste est refusée (`race_during_update`), sans écriture ; relancée après, elle
/// passe au nouveau compte. Patron de `reconciliation_e2e.rs`
/// (`accept_refuses_when_a_partial_manual_settlement_lands_meanwhile`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_reconciliation_started_before_the_gesture_is_refused(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let d = sale_day();
    let id = ctx.validated(Some(ctx.p), d).await;
    let tx_id = ctx
        .bank_transaction(&pool, id, dec!(100.00), d + Duration::days(10))
        .await;
    let entries_before: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .bind(ctx.seeded.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    // (1) Le verrou de métadonnées sur `contacts`, connexion détachée du pool.
    let mut verrou = pool.acquire().await.unwrap().detach();
    sqlx::query("LOCK TABLES contacts WRITE")
        .execute(&mut verrou)
        .await
        .unwrap();

    // (2) L'acceptation part, lit la facture, et attend sur le contact.
    let client = ctx.client.clone();
    let url = ctx.url("/api/v1/reconciliation/accept");
    let token = ctx.admin.clone();
    let body = ctx.accept_body(tx_id, id);
    let acceptation = tokio::spawn(async move {
        let resp = client
            .post(url)
            .bearer_auth(token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    let bloquee =
        kesh_db::test_fixtures::attendre_une_requete_en_cours(&pool, &["FROM contacts"], || {
            acceptation.is_finished()
        })
        .await;
    assert!(bloquee, "l'acceptation devait attendre sur `contacts`");

    // (3) Le geste, commité pendant l'attente.
    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_eq!(status, 200, "{body}");

    // (4) Relâcher : l'acceptation reprend sur un instantané périmé.
    sqlx::query("UNLOCK TABLES")
        .execute(&mut verrou)
        .await
        .unwrap();
    drop(verrou);
    let (status, body) = acceptation.await.unwrap();
    assert_eq!(status, 200, "{body}");
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1, "{body}");
    assert_eq!(
        failed[0]["errorCode"],
        "RECONCILIATION_INVOICE_NOT_ELIGIBLE"
    );
    assert_eq!(failed[0]["details"]["reason"], "race_during_update");
    let settlements: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM invoice_settlements WHERE invoice_id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(settlements, 0, "aucun règlement écrit");
    let entries_after: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
            .bind(ctx.seeded.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(entries_after, entries_before, "aucune écriture neuve");

    // (5) Relancée, elle passe, au compte Q.
    let body = ctx
        .post_ok("/api/v1/reconciliation/accept", ctx.accept_body(tx_id, id))
        .await;
    assert_eq!(body["accepted"].as_array().map(Vec::len), Some(1), "{body}");
    let entries = linked_entries(&pool, id).await;
    assert_eq!(entries.len(), 2);
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.q))
    );
}

// ---------------------------------------------------------------------------
// T5 — les paires déliées
// ---------------------------------------------------------------------------

/// T5 (AC10) — après l'annulation d'un règlement, ce règlement et sa
/// contre-passation gardent P et leur version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_unlinked_pair_keeps_its_account(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let d = sale_day();
    let id = ctx.validated(Some(ctx.p), d).await;
    ctx.settle(id, "40.00", d + Duration::days(5)).await;
    let (settlement_id, settlement_entry): (i64, i64) =
        sqlx::query_as("SELECT id, journal_entry_id FROM invoice_settlements WHERE invoice_id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    ctx.post_ok(
        &format!("/api/v1/invoices/{id}/settlements/{settlement_id}/cancel"),
        json!({}),
    )
    .await;
    let reversal: i64 =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE reverses_entry_id = ?")
            .bind(settlement_entry)
            .fetch_one(&pool)
            .await
            .unwrap();
    let pair = [settlement_entry, reversal];
    let pair_versions = entry_versions(&pool, &pair).await;

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_eq!(status, 200, "{body}");

    assert_eq!(
        ids_of(&body["journalEntryIds"]),
        linked_entries(&pool, id).await
    );
    assert!(
        line_projects(&pool, &pair)
            .await
            .iter()
            .all(|p| *p == Some(ctx.p))
    );
    assert_eq!(entry_versions(&pool, &pair).await, pair_versions);
}

// ---------------------------------------------------------------------------
// T6 (AC6) — les rapports
// ---------------------------------------------------------------------------

async fn project_return(ctx: &Ctx, project: i64) -> Value {
    ctx.get(&format!(
        "/api/v1/reports/project-return?projectId={project}&mode=fiscal_year&fiscalYearId={}",
        ctx.seeded.fiscal_year_id
    ))
    .await["totals"]
        .clone()
}

/// AC6 — le rendement de Q contient ce que celui de P contenait pour la
/// facture ; celui de P ne le contient plus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reports_follow_the_account(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let p_before = project_return(&ctx, ctx.p).await;
    let q_before = project_return(&ctx, ctx.q).await;
    assert_ne!(p_before, q_before, "montage : P porte la facture, Q rien");

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_eq!(status, 200, "{body}");

    assert_eq!(project_return(&ctx, ctx.q).await, p_before);
    assert_eq!(project_return(&ctx, ctx.p).await, q_before);
}

// ---------------------------------------------------------------------------
// T7 — la période close
// ---------------------------------------------------------------------------

fn assert_refused(status: u16, body: &Value, expected: u16, code: &str) {
    assert_eq!(status, expected, "{body}");
    assert_eq!(body["error"]["code"], code, "{body}");
}

/// T7 (AC7) — exercice clos : un Comptable `403` avec ou sans accusé ; une clé
/// `read-write` d'un Administrateur, avec l'accusé, `403` ; l'Administrateur en
/// session `409` sans accusé, `200` avec ; chaque refus n'écrit rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_closed_fiscal_year_is_for_the_admin_in_session_only(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let comptable = ctx.user(&pool, "comptable", "Comptable").await;
    let (cle, secret) = common::audit_traces::create_key_via_http(
        &ctx.client,
        &ctx.base_url,
        &ctx.admin,
        "rw",
        "read-write",
    )
    .await;
    close_fiscal_year(&pool, &ctx).await;
    let before = snapshot(&pool, ctx.seeded.company_id).await;
    let version = ctx.version(id).await;

    for ack in [None, Some(true)] {
        let (status, body) = ctx
            .reassign(&comptable, id, Some(ctx.q), version, ack)
            .await;
        assert_refused(
            status,
            &body,
            403,
            "PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY",
        );
        assert_eq!(body["error"]["details"]["entryCount"], 4);
    }
    let (status, body) = ctx
        .reassign(&secret, id, Some(ctx.q), version, Some(true))
        .await;
    assert_refused(
        status,
        &body,
        403,
        "PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY",
    );
    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, None)
        .await;
    assert_refused(status, &body, 409, "PROJECT_REASSIGNMENT_CLOSED_PERIOD");
    assert_eq!(
        body["error"]["details"],
        json!({ "entryCount": 4, "fiscalYears": ["Exercice CI 2020-2030"], "lockedThrough": null })
    );
    assert_eq!(snapshot(&pool, ctx.seeded.company_id).await, before);
    assert_eq!(ctx.version(id).await, version, "un refus n'écrit rien");
    assert!(audits(&pool, "invoice.project_reassigned").await.is_empty());
    let _ = cle;

    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, Some(true))
        .await;
    assert_eq!(status, 200, "{body}");
    let traces = audits(&pool, "invoice.project_reassigned").await;
    assert_eq!(traces.len(), 1);
    assert_eq!(traces[0].3["closedPeriodsAcknowledged"], true);
}

/// T7 — verrou des livres au jour exact de la vente (seuil inclusif) : la seule
/// écriture, la vente, est exposée ; la veille, elle ne l'est pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_books_lock_threshold_is_inclusive(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let d = sale_day();
    let id = ctx.validated(Some(ctx.p), d).await;
    let comptable = ctx.user(&pool, "comptable", "Comptable").await;

    lock_books(&ctx, d - Duration::days(1)).await;
    let version = ctx.version(id).await;
    let (status, body) = ctx
        .reassign(&comptable, id, Some(ctx.q), version, None)
        .await;
    assert_eq!(
        status, 200,
        "la veille de la vente : rien d'exposé — {body}"
    );

    lock_books(&ctx, d).await;
    let version = ctx.version(id).await;
    let (status, body) = ctx
        .reassign(&comptable, id, Some(ctx.p), version, None)
        .await;
    assert_refused(
        status,
        &body,
        403,
        "PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY",
    );
    assert_eq!(
        body["error"]["details"],
        json!({ "entryCount": 1, "fiscalYears": [], "lockedThrough": d.to_string() })
    );
}

/// T7 — `entryCount` ne compte que les écritures exposées qui changeraient : le
/// verrou couvre la vente et le premier règlement, pas les suivants.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn entry_count_counts_only_exposed_entries(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    lock_books(&ctx, sale_day() + Duration::days(7)).await;
    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_refused(status, &body, 409, "PROJECT_REASSIGNMENT_CLOSED_PERIOD");
    assert_eq!(body["error"]["details"]["entryCount"], 2, "{body}");
}

/// T7 — **clôture concurrente, déterministe** : une transaction de test tient
/// l'exercice `FOR UPDATE` et l'a passé `Closed` (la forme de l'étape (e) de
/// `fiscal_years::close`) ; le geste, lancé en tâche, attend ce verrou ; au
/// `COMMIT` il lit `Closed` et refuse, sans rien écrire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_concurrent_closing_is_seen_by_the_gesture(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = cancelled_invoice(&ctx).await;
    let version = ctx.version(id).await;
    let before = snapshot(&pool, ctx.seeded.company_id).await;

    let mut cloture = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&mut *cloture)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ? AND status = 'Open'")
        .bind(ctx.seeded.fiscal_year_id)
        .execute(&mut *cloture)
        .await
        .unwrap();

    let client = ctx.client.clone();
    let url = ctx.url(&format!("/api/v1/invoices/{id}/reassign-project"));
    let token = ctx.admin.clone();
    let q = ctx.q;
    let geste = tokio::spawn(async move {
        let resp = client
            .post(url)
            .bearer_auth(token)
            .json(&json!({ "projectId": q, "version": version }))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    let attend = kesh_db::test_fixtures::attendre_une_requete_en_cours(
        &pool,
        &["FROM fiscal_years", "FOR UPDATE"],
        || geste.is_finished(),
    )
    .await;
    cloture.commit().await.unwrap();
    let (status, body) = geste.await.unwrap();
    assert!(
        attend,
        "le geste devait attendre l'exercice — rendu : {status} {body}"
    );
    assert_refused(status, &body, 409, "PROJECT_REASSIGNMENT_CLOSED_PERIOD");
    assert_eq!(snapshot(&pool, ctx.seeded.company_id).await, before);
    assert_eq!(ctx.version(id).await, version);
}

// ---------------------------------------------------------------------------
// T8 — l'audit
// ---------------------------------------------------------------------------

/// T8 (AC9) — par une clé : `invoice.project_reassigned`, `actor_type =
/// 'api_key'`, la clé, et les `details`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_trace_names_the_api_key(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let entries = linked_entries(&pool, id).await;
    let (cle, secret) = common::audit_traces::create_key_via_http(
        &ctx.client,
        &ctx.base_url,
        &ctx.admin,
        "rw",
        "read-write",
    )
    .await;
    let version = ctx.version(id).await;

    let (status, body) = ctx.reassign(&secret, id, Some(ctx.q), version, None).await;
    assert_eq!(status, 200, "{body}");

    let traces = audits(&pool, "invoice.project_reassigned").await;
    assert_eq!(traces.len(), 1);
    let (entity, actor_type, key, details) = &traces[0];
    assert_eq!(*entity, id);
    assert_eq!(actor_type, "api_key");
    assert_eq!(*key, Some(cle));
    assert_eq!(
        *details,
        json!({
            "fromProjectId": ctx.p,
            "toProjectId": ctx.q,
            "journalEntryIds": entries,
            "closedPeriodsAcknowledged": false,
        })
    );
}

/// T8 — un échec de la trace fait échouer le geste entier : rien n'a changé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_failing_trace_fails_the_whole_gesture(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let entries = linked_entries(&pool, id).await;
    let version = ctx.version(id).await;
    let versions = entry_versions(&pool, &entries).await;
    sqlx::query(
        "CREATE TRIGGER t_36_16a_fail BEFORE INSERT ON audit_log FOR EACH ROW \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '36-16a atomicity'",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, None)
        .await;

    sqlx::query("DROP TRIGGER t_36_16a_fail")
        .execute(&pool)
        .await
        .unwrap();
    assert!(status >= 500, "le geste échoue : {status} {body}");
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.p))
    );
    assert_eq!(entry_versions(&pool, &entries).await, versions);
    let invoice = ctx.invoice(id).await;
    assert_eq!(invoice["projectId"], ctx.p);
    assert_eq!(invoice["version"].as_i64().unwrap(), version);
}

// ---------------------------------------------------------------------------
// T9 — l'ordre des refus (AC8), une paire voisine par test
// ---------------------------------------------------------------------------

/// Rôle avant tout : Consultation et clé `read` → `403` du contrôle de rôle,
/// même sur une facture absente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_role_check_comes_first(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let lecteur = ctx.user(&pool, "lecteur", "Consultation").await;
    let (_, read_key) = common::audit_traces::create_key_via_http(
        &ctx.client,
        &ctx.base_url,
        &ctx.admin,
        "ro",
        "read",
    )
    .await;
    for token in [&lecteur, &read_key] {
        let (status, body) = ctx.reassign(token, 999_999, Some(ctx.q), 1, None).await;
        assert_eq!(status, 403, "{body}");
    }
}

/// Facture absente ou d'une autre société → `404`, avant la version.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_unknown_or_foreign_invoice_is_404_before_the_version(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let (status, body) = ctx
        .reassign(&ctx.admin, 999_999, Some(ctx.q), 0, None)
        .await;
    assert_refused(status, &body, 404, "NOT_FOUND");

    let other_company: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let other_contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Autre client', TRUE) RETURNING id",
    )
    .bind(other_company)
    .fetch_one(&pool)
    .await
    .unwrap();
    let foreign: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, status, date, total_amount, version) \
         VALUES (?, ?, 'draft', ?, 0, 1) RETURNING id",
    )
    .bind(other_company)
    .bind(other_contact)
    .bind(sale_day())
    .fetch_one(&pool)
    .await
    .unwrap();
    let (status, body) = ctx
        .reassign(&ctx.admin, foreign, Some(ctx.q), 1, None)
        .await;
    assert_refused(status, &body, 404, "NOT_FOUND");
}

/// Version périmée → `409 OPTIMISTIC_LOCK_CONFLICT`, **même si le compte ne
/// change pas** (la version précède « rien à faire »).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stale_version_is_refused_even_when_nothing_would_change(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.validated(Some(ctx.p), sale_day()).await;
    let version = ctx.version(id).await;
    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.p), version - 1, None)
        .await;
    assert_refused(status, &body, 409, "OPTIMISTIC_LOCK_CONFLICT");
}

/// Rien à faire avant le compte visé : le compte **actuel** archivé, visé de
/// nouveau, rend `200` (il n'est pas validé). Une ligne divergente (donnée
/// historique, C-36-16a-2) fait passer le geste au-delà de « rien à faire » : il
/// réaligne la ligne sur le compte archivé, sans refus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_current_archived_account_is_not_revalidated(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.validated(Some(ctx.p), sale_day()).await;
    let entries = linked_entries(&pool, id).await;
    sqlx::query(
        "UPDATE journal_entry_lines SET project_id = ? WHERE entry_id = ? ORDER BY id LIMIT 1",
    )
    .bind(ctx.q)
    .bind(entries[0])
    .execute(&pool)
    .await
    .unwrap();
    archive(&ctx, ctx.p).await;
    let (status, body) = ctx.reassign_now(id, Some(ctx.p)).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(ids_of(&body["journalEntryIds"]), entries);
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.p))
    );
}

/// Archive un compte analytique par la route, à sa version courante.
async fn archive(ctx: &Ctx, project: i64) {
    let version = ctx.get(&format!("/api/v1/projects/{project}")).await["version"].clone();
    ctx.post_ok(
        &format!("/api/v1/projects/{project}/archive"),
        json!({ "version": version }),
    )
    .await;
}

/// Compte visé inconnu → `404`, archivé → `409 ILLEGAL_STATE_TRANSITION`, et
/// tous deux **avant** la période close (un Comptable reçoit le `404`, non le
/// `403`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn the_target_account_is_checked_before_the_closed_period(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.validated(Some(ctx.p), sale_day()).await;
    let archived = project(&ctx, "ARCH").await;
    archive(&ctx, archived).await;
    let comptable = ctx.user(&pool, "comptable", "Comptable").await;
    close_fiscal_year(&pool, &ctx).await;
    let version = ctx.version(id).await;

    let (status, body) = ctx
        .reassign(&comptable, id, Some(999_999), version, None)
        .await;
    assert_refused(status, &body, 404, "NOT_FOUND");
    let (status, body) = ctx
        .reassign(&comptable, id, Some(archived), version, None)
        .await;
    assert_refused(status, &body, 409, "ILLEGAL_STATE_TRANSITION");
    let (status, body) = ctx
        .reassign(&comptable, id, Some(ctx.q), version, None)
        .await;
    assert_refused(
        status,
        &body,
        403,
        "PROJECT_REASSIGNMENT_CLOSED_PERIOD_ADMIN_ONLY",
    );
}

// ---------------------------------------------------------------------------
// T10 — rien à faire, ligne divergente, brouillon, accusé en période ouverte
// ---------------------------------------------------------------------------

/// AC11 — facture et lignes déjà à Q : `200`, versions inchangées, aucun audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nothing_to_do_writes_nothing(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let (status, _) = ctx.reassign_now(id, Some(ctx.q)).await;
    assert_eq!(status, 200);
    let entries = linked_entries(&pool, id).await;
    let versions = entry_versions(&pool, &entries).await;
    let version = ctx.version(id).await;
    let traces = audits(&pool, "invoice.project_reassigned").await.len();

    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, None)
        .await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(body["journalEntryIds"], json!([]));
    assert_eq!(body["invoice"]["version"].as_i64().unwrap(), version);
    assert_eq!(entry_versions(&pool, &entries).await, versions);
    assert_eq!(
        audits(&pool, "invoice.project_reassigned").await.len(),
        traces
    );
}

/// AC11 — une facture à Q dont une ligne diverge (donnée historique) est
/// réalignée ; seule l'écriture qui diverge change.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_diverging_line_is_realigned(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let d = sale_day();
    let id = ctx.validated(Some(ctx.q), d).await;
    ctx.settle(id, "40.00", d + Duration::days(5)).await;
    let entries = linked_entries(&pool, id).await;
    let diverging = entries[1];
    sqlx::query(
        "UPDATE journal_entry_lines SET project_id = ? WHERE entry_id = ? ORDER BY id LIMIT 1",
    )
    .bind(ctx.p)
    .bind(diverging)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = ctx.reassign_now(id, Some(ctx.q)).await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(ids_of(&body["journalEntryIds"]), vec![diverging]);
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(|p| *p == Some(ctx.q))
    );
}

/// AC11 — `projectId: null` retire le compte partout.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn null_removes_the_account_everywhere(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = paid_invoice(&pool, &ctx).await;
    let entries = linked_entries(&pool, id).await;
    let (status, body) = ctx.reassign_now(id, None).await;
    assert_eq!(status, 200, "{body}");
    assert!(body["invoice"]["projectId"].is_null());
    assert!(
        line_projects(&pool, &entries)
            .await
            .iter()
            .all(Option::is_none)
    );
}

/// AC11 / D10 — un brouillon : la facture seule, version +1, trace,
/// `journalEntryIds: []`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_draft_changes_alone(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.draft(Some(ctx.p), sale_day()).await;
    let version = ctx.version(id).await;

    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, None)
        .await;

    assert_eq!(status, 200, "{body}");
    assert_eq!(body["journalEntryIds"], json!([]));
    assert_eq!(body["invoice"]["projectId"], ctx.q);
    assert_eq!(body["invoice"]["version"].as_i64().unwrap(), version + 1);
    assert_eq!(audits(&pool, "invoice.project_reassigned").await.len(), 1);
}

/// AC11 — l'accusé envoyé en période ouverte : `200`,
/// `closedPeriodsAcknowledged: false`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn an_acknowledgement_in_an_open_period_is_not_recorded(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.validated(Some(ctx.p), sale_day()).await;
    let version = ctx.version(id).await;
    let (status, body) = ctx
        .reassign(&ctx.admin, id, Some(ctx.q), version, Some(true))
        .await;
    assert_eq!(status, 200, "{body}");
    let traces = audits(&pool, "invoice.project_reassigned").await;
    assert_eq!(traces[0].3["closedPeriodsAcknowledged"], false);
}

/// `projectId` est exigé : un corps qui l'omet ne retire pas le compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_missing_project_id_is_refused(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let id = ctx.validated(Some(ctx.p), sale_day()).await;
    let version = ctx.version(id).await;
    let (status, _) = ctx
        .post_as(
            &ctx.admin,
            &format!("/api/v1/invoices/{id}/reassign-project"),
            json!({ "version": version }),
        )
        .await;
    assert!((400..500).contains(&status), "{status}");
    assert_eq!(ctx.invoice(id).await["projectId"], ctx.p);
}
