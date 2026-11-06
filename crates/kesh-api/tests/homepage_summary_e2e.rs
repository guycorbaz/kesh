//! Story 33-3a (#918) — `GET /api/v1/reports/homepage-summary` : les chiffres
//! des graphiques de l'accueil, à la frontière HTTP.
//!
//! Le montage part de la date du jour : l'exercice ouvert commence au 1er du
//! mois, trois mois avant aujourd'hui, pour qu'il y ait toujours des mois
//! écoulés et que rien ne rougisse en janvier ; `N−1` le précède immédiatement.
//! Décor synthétique.
#![allow(clippy::too_many_arguments)]

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{Datelike, Months, NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    ApiKeyScope, Language, NewAccount, NewApiKey, NewCompany, NewCreditNote, NewFiscalYear,
    NewJournalEntry, NewJournalEntryLine, NewSupplierInvoice, NewSupplierInvoiceLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{
    accounts, api_keys, companies, contacts, credit_notes, fiscal_years, invoices, journal_entries,
    supplier_invoices, users,
};
use kesh_db::test_fixtures::seed_accounting_company;
use kesh_report::homepage_summary;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use sqlx::MySqlPool;

#[path = "../../kesh-db/tests/support/fournisseur_rapproche.rs"]
mod fournisseur_rapproche;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ROUTE: &str = "/api/v1/reports/homepage-summary";

// ============================================================
// Serveur
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// La synthèse, `200` exigé.
    async fn summary(&self, jwt: &str) -> Value {
        let resp = self
            .client
            .get(self.url(ROUTE))
            .bearer_auth(jwt)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "route de la synthèse");
        resp.json().await.unwrap()
    }

    /// Les deux totaux du compte de résultat d'un exercice.
    async fn income_totals(&self, jwt: &str, fy_id: i64) -> (Decimal, Decimal) {
        let body: Value = self
            .client
            .get(self.url(&format!(
                "/api/v1/reports/income-statement?fiscalYearId={fy_id}"
            )))
            .bearer_auth(jwt)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        (
            dec_of(&body["totalRevenues"]),
            dec_of(&body["totalExpenses"]),
        )
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
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
// Lecture de la réponse
// ============================================================

/// Un montant rendu : une chaîne présente, relue en `Decimal`.
fn dec_of(v: &Value) -> Decimal {
    v.as_str()
        .unwrap_or_else(|| panic!("montant attendu en chaîne, reçu {v}"))
        .parse()
        .unwrap_or_else(|e| panic!("{v} n'est pas un décimal : {e}"))
}

fn month_at(body: &Value, i: usize) -> (Decimal, Decimal) {
    let m = &body["months"][i];
    (dec_of(&m["revenues"]), dec_of(&m["expenses"]))
}

/// L'indice du mois civil de `d` dans `months`.
fn month_index(body: &Value, d: NaiveDate) -> usize {
    let key = format!("{:04}-{:02}", d.year(), d.month());
    body["months"]
        .as_array()
        .unwrap()
        .iter()
        .position(|m| m["month"] == key.as_str())
        .unwrap_or_else(|| panic!("mois {key} absent de {body}"))
}

fn post_numbers(body: &Value) -> Vec<String> {
    body["expensePosts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["number"].as_str().unwrap().to_string())
        .collect()
}

fn post_amounts(body: &Value) -> Vec<Decimal> {
    body["expensePosts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| dec_of(&p["amount"]))
        .collect()
}

fn post_shares(body: &Value) -> Vec<String> {
    body["expensePosts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["share"].as_str().unwrap().to_string())
        .collect()
}

/// `Σ months` égale les deux totaux, et les totaux égalent le compte de résultat
/// (AC5, C-e332-10).
async fn assert_invariant(app: &TestApp, jwt: &str, body: &Value) {
    let (mut rev, mut exp) = (Decimal::ZERO, Decimal::ZERO);
    for i in 0..body["months"].as_array().unwrap().len() {
        let (r, e) = month_at(body, i);
        rev += r;
        exp += e;
    }
    assert_eq!(rev, dec_of(&body["totalRevenues"]), "Σ revenus : {body}");
    assert_eq!(exp, dec_of(&body["totalExpenses"]), "Σ dépenses : {body}");
    let fy_id = body["fiscalYear"]["id"].as_i64().unwrap();
    let (is_rev, is_exp) = app.income_totals(jwt, fy_id).await;
    assert_eq!(rev, is_rev, "revenus ≠ compte de résultat : {body}");
    assert_eq!(exp, is_exp, "dépenses ≠ compte de résultat : {body}");
}

// ============================================================
// Montage
// ============================================================

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

/// Le 1er du mois, trois mois avant aujourd'hui : le début de l'exercice ouvert.
fn year_start() -> NaiveDate {
    today()
        .with_day(1)
        .unwrap()
        .checked_sub_months(Months::new(3))
        .unwrap()
}

/// Le 1er du `k`-ième mois de l'exercice ouvert.
fn month_k(k: u32) -> NaiveDate {
    year_start().checked_add_months(Months::new(k)).unwrap()
}

fn last_day_before(d: NaiveDate) -> NaiveDate {
    d.pred_opt().unwrap()
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    fy_id: i64,
    bank: i64,
    capital: i64,
    revenue: i64,
    expense: i64,
    jwt: String,
}

async fn new_company(pool: &MySqlPool, label: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: label.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
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
    .id
}

async fn new_user(pool: &MySqlPool, label: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: label.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn new_fy(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    name: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: name.into(),
            start_date: start,
            end_date: end,
        },
    )
    .await
    .unwrap()
    .id
}

async fn new_account(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// Une écriture à deux lignes, `id` rendu.
async fn post(
    pool: &MySqlPool,
    user_id: i64,
    fy_id: i64,
    company_id: i64,
    date: NaiveDate,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) -> i64 {
    journal_entries::create(
        pool,
        fy_id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Écriture de test".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap()
    .entry
    .id
}

/// Une société, un administrateur, l'exercice ouvert de douze mois, quatre comptes.
async fn setup(pool: &MySqlPool, label: &str) -> Ctx {
    let company_id = new_company(pool, label).await;
    let user_id = new_user(pool, &format!("{label}_admin"), Role::Admin, company_id).await;
    let start = year_start();
    let end = last_day_before(start.checked_add_months(Months::new(12)).unwrap());
    let fy_id = new_fy(pool, user_id, company_id, "N", start, end).await;
    let bank = new_account(
        pool,
        user_id,
        company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let capital = new_account(
        pool,
        user_id,
        company_id,
        "2800",
        "Capital",
        AccountType::Liability,
    )
    .await;
    let revenue = new_account(
        pool,
        user_id,
        company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let expense = new_account(
        pool,
        user_id,
        company_id,
        "6000",
        "Loyer",
        AccountType::Expense,
    )
    .await;
    Ctx {
        company_id,
        user_id,
        fy_id,
        bank,
        capital,
        revenue,
        expense,
        jwt: forge_jwt(user_id, "Admin", company_id),
    }
}

impl Ctx {
    /// Une charge sur `account`, au débit, dans l'exercice ouvert.
    async fn charge(
        &self,
        pool: &MySqlPool,
        date: NaiveDate,
        account: i64,
        amount: Decimal,
    ) -> i64 {
        post(
            pool,
            self.user_id,
            self.fy_id,
            self.company_id,
            date,
            account,
            self.bank,
            amount,
        )
        .await
    }

    /// Un compte de charges de plus.
    async fn expense_account(&self, pool: &MySqlPool, number: &str) -> i64 {
        new_account(
            pool,
            self.user_id,
            self.company_id,
            number,
            &format!("Charge {number}"),
            AccountType::Expense,
        )
        .await
    }

    /// `N−1`, ouvert, immédiatement avant l'exercice du montage.
    async fn previous_year(&self, pool: &MySqlPool) -> i64 {
        let start = year_start().checked_sub_months(Months::new(12)).unwrap();
        new_fy(
            pool,
            self.user_id,
            self.company_id,
            "N-1",
            start,
            last_day_before(year_start()),
        )
        .await
    }
}

// ============================================================
// AC1 à AC3 — l'exercice et ses mois
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn summary_of_an_empty_open_year(pool: MySqlPool) {
    let c = setup(&pool, "vide").await;
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(b["state"], "open", "{b}");
    assert_eq!(b["asOf"], today().to_string());
    assert_eq!(b["fiscalYear"]["id"], c.fy_id);
    assert_eq!(b["fiscalYear"]["name"], "N");
    assert_eq!(b["fiscalYear"]["startDate"], year_start().to_string());
    let months = b["months"].as_array().unwrap();
    assert_eq!(months.len(), 12, "{b}");
    for m in months {
        assert_eq!(m["revenues"], "0", "{m}");
        assert_eq!(m["expenses"], "0", "{m}");
    }
    assert_eq!(b["totalRevenues"], "0");
    assert_eq!(b["totalExpenses"], "0");
    assert_eq!(b["expensePosts"], serde_json::json!([]));
    assert_eq!(b["otherExpenses"], Value::Null);
    assert_eq!(b["expensePostsBase"], "0");
    assert_eq!(b["creditBalanceExpenseAccounts"], 0);
}

/// La forme exacte de l'état « aucun exercice ouvert », pour une date donnée.
fn assert_no_open_year(b: &Value) {
    assert_eq!(b["state"], "noOpenFiscalYear", "{b}");
    assert_eq!(b["asOf"], today().to_string());
    assert_eq!(b["fiscalYear"], Value::Null, "{b}");
    assert_eq!(b["months"], serde_json::json!([]));
    assert_eq!(b["totalRevenues"], "0");
    assert_eq!(b["totalExpenses"], "0");
    assert_eq!(b["expensePosts"], serde_json::json!([]));
    assert_eq!(b["otherExpenses"], Value::Null);
    assert_eq!(b["expensePostsBase"], "0");
    assert_eq!(b["creditBalanceExpenseAccounts"], 0);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn no_open_year_in_three_cases(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // (a) Aucun exercice.
    let a = new_company(&pool, "sans").await;
    let ua = new_user(&pool, "sans_admin", Role::Admin, a).await;
    assert_no_open_year(&app.summary(&forge_jwt(ua, "Admin", a)).await);

    // (b) Un exercice couvrant le jour, clos, et l'exercice suivant ouvert.
    let c = setup(&pool, "clos").await;
    let next_start = year_start().checked_add_months(Months::new(12)).unwrap();
    let next_end = last_day_before(next_start.checked_add_months(Months::new(12)).unwrap());
    new_fy(&pool, c.user_id, c.company_id, "N+1", next_start, next_end).await;
    fiscal_years::close(&pool, c.user_id, c.company_id, c.fy_id)
        .await
        .unwrap();
    assert_no_open_year(&app.summary(&c.jwt).await);

    // (c) Seul un exercice ne couvrant pas le jour est ouvert.
    let d = new_company(&pool, "ailleurs").await;
    let ud = new_user(&pool, "ailleurs_admin", Role::Admin, d).await;
    let past_end = last_day_before(year_start());
    let past_start = year_start().checked_sub_months(Months::new(12)).unwrap();
    new_fy(&pool, ud, d, "Passé", past_start, past_end).await;
    assert_no_open_year(&app.summary(&forge_jwt(ud, "Admin", d)).await);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn months_follow_the_fiscal_year_bounds(pool: MySqlPool) {
    // Du 15 d'un mois au 14 du même mois l'an suivant : treize mois civils.
    let company_id = new_company(&pool, "quinze").await;
    let user_id = new_user(&pool, "quinze_admin", Role::Admin, company_id).await;
    let start = year_start().with_day(15).unwrap();
    let end = start
        .checked_add_months(Months::new(12))
        .unwrap()
        .with_day(14)
        .unwrap();
    new_fy(&pool, user_id, company_id, "Décalé", start, end).await;
    let app = spawn_app(pool).await;
    let b = app.summary(&forge_jwt(user_id, "Admin", company_id)).await;

    let months = b["months"].as_array().unwrap();
    assert_eq!(months.len(), 13, "{b}");
    assert_eq!(months[0]["from"], start.to_string(), "borne basse : {b}");
    let first_last = last_day_before(year_start().checked_add_months(Months::new(1)).unwrap());
    assert_eq!(months[0]["to"], first_last.to_string());
    assert_eq!(
        months[12]["from"],
        end.with_day(1).unwrap().to_string(),
        "{b}"
    );
    assert_eq!(months[12]["to"], end.to_string(), "borne haute : {b}");
    assert_eq!(
        months[1]["from"],
        year_start()
            .checked_add_months(Months::new(1))
            .unwrap()
            .to_string()
    );
}

// ============================================================
// AC4 à AC6 — les montants
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn signs_and_only_income_statement_accounts(pool: MySqlPool) {
    let c = setup(&pool, "signes").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    // M0 : un revenu ; M1 : une charge ; à quatre décimales, sans arrondi.
    post(
        &pool,
        u,
        f,
        co,
        month_k(0),
        c.bank,
        c.revenue,
        dec!(1000.1234),
    )
    .await;
    post(
        &pool,
        u,
        f,
        co,
        month_k(1),
        c.expense,
        c.bank,
        dec!(300.5678),
    )
    .await;
    // Une ligne de bilan n'y entre pas.
    post(&pool, u, f, co, month_k(1), c.bank, c.capital, dec!(9999)).await;
    // M2 : une charge au crédit, un revenu au débit : négatifs.
    post(&pool, u, f, co, month_k(2), c.bank, c.expense, dec!(50)).await;
    post(&pool, u, f, co, month_k(2), c.revenue, c.bank, dec!(20)).await;

    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(month_at(&b, 0), (dec!(1000.1234), dec!(0)), "{b}");
    assert_eq!(month_at(&b, 1), (dec!(0), dec!(300.5678)), "{b}");
    assert_eq!(month_at(&b, 2), (dec!(-20), dec!(-50)), "{b}");
    assert_eq!(dec_of(&b["totalRevenues"]), dec!(980.1234));
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(250.5678));
    assert_invariant(&app, &c.jwt, &b).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn totals_equal_the_income_statement(pool: MySqlPool) {
    let c = setup(&pool, "invariant").await;
    let prev = c.previous_year(&pool).await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    let second = c.expense_account(&pool, "6100").await;
    // Des écritures ordinaires.
    post(
        &pool,
        u,
        f,
        co,
        month_k(0),
        c.bank,
        c.revenue,
        dec!(1200.50),
    )
    .await;
    post(&pool, u, f, co, month_k(1), c.expense, c.bank, dec!(400.25)).await;
    // Une paire dans l'exercice.
    let in_year = post(&pool, u, f, co, month_k(0), second, c.bank, dec!(70)).await;
    journal_entries::reverse(&pool, co, in_year, u)
        .await
        .unwrap();
    // Une paire à cheval sur N−1 et N.
    let across = post(
        &pool,
        u,
        prev,
        co,
        last_day_before(year_start()),
        c.expense,
        c.bank,
        dec!(33),
    )
    .await;
    journal_entries::reverse_at_next_fiscal_year_start(&pool, co, across, u)
        .await
        .unwrap();
    // Un montant de revenus au débit, sans `reverses_entry_id`, de la forme d'un
    // avoir (l'avoir réel : `credit_note_stays_counted`).
    post(&pool, u, f, co, month_k(2), c.revenue, c.bank, dec!(15)).await;
    // Une charge au crédit.
    post(&pool, u, f, co, month_k(2), c.bank, second, dec!(8)).await;
    // Les deux anomalies de date que la préparation de clôture signale
    // (`OTHER_YEAR_DATED_INSIDE`, `DATE_OUTSIDE_ITS_YEAR`), posées en SQL : le compte
    // de résultat ne les retient pas, la synthèse non plus.
    let other_year = post(
        &pool,
        u,
        prev,
        co,
        year_start().pred_opt().unwrap(),
        c.expense,
        c.bank,
        dec!(1111),
    )
    .await;
    let outside = post(&pool, u, f, co, month_k(0), c.expense, c.bank, dec!(2222)).await;
    for (id, date) in [
        (other_year, month_k(1)),
        (outside, last_day_before(year_start())),
    ] {
        sqlx::query("UPDATE journal_entries SET entry_date = ? WHERE id = ?")
            .bind(date)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;
    assert_invariant(&app, &c.jwt, &b).await;
    assert_eq!(dec_of(&b["totalRevenues"]), dec!(1185.50), "{b}");
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(359.25), "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn other_company_never_enters(pool: MySqlPool) {
    let a = setup(&pool, "sa").await;
    let z = setup(&pool, "sz").await;
    post(
        &pool,
        a.user_id,
        a.fy_id,
        a.company_id,
        month_k(0),
        a.expense,
        a.bank,
        dec!(10),
    )
    .await;
    post(
        &pool,
        z.user_id,
        z.fy_id,
        z.company_id,
        month_k(0),
        z.expense,
        z.bank,
        dec!(777),
    )
    .await;
    post(
        &pool,
        z.user_id,
        z.fy_id,
        z.company_id,
        month_k(0),
        z.bank,
        z.revenue,
        dec!(888),
    )
    .await;

    // Deux lignes forgées, qu'aucun chemin applicatif n'écrit, pour que chacun des
    // deux filtres de société se voie seul : une écriture de Z sur l'exercice et un
    // compte de A ; une écriture de A sur un compte de Z.
    let forged_z: i64 = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 9001, ?, 'OD', 'Forgée Z')",
    )
    .bind(z.company_id)
    .bind(a.fy_id)
    .bind(month_k(0))
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let forged_a: i64 = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 9002, ?, 'OD', 'Forgée A')",
    )
    .bind(a.company_id)
    .bind(a.fy_id)
    .bind(month_k(0))
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (entry, account, debit) in [(forged_z, a.expense, 1000), (forged_a, z.expense, 2000)] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, 1, ?, 0)",
        )
        .bind(entry)
        .bind(account)
        .bind(debit)
        .execute(&pool)
        .await
        .unwrap();
    }

    let app = spawn_app(pool).await;
    let b = app.summary(&a.jwt).await;
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(10), "{b}");
    assert_eq!(dec_of(&b["totalRevenues"]), dec!(0), "{b}");
    assert_eq!(post_numbers(&b), vec!["6000"], "{b}");
    assert_eq!(post_amounts(&b), vec![dec!(10)], "{b}");
}

// ============================================================
// AC7 à AC10 — contre-passations et avoirs
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reversal_pair_in_the_year_leaves_both_months_untouched(pool: MySqlPool) {
    let c = setup(&pool, "paire").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    let other = c.expense_account(&pool, "6100").await;
    // Ce qui reste, de part et d'autre.
    c.charge(&pool, month_k(0), other, dec!(100)).await;
    post(&pool, u, f, co, month_k(0), c.bank, c.revenue, dec!(40)).await;
    // La contre-passation se date au jour (`ReversalDating::Today`), donc au mois 3.
    let charge = c.charge(&pool, month_k(0), c.expense, dec!(500)).await;
    let income = post(&pool, u, f, co, month_k(0), c.bank, c.revenue, dec!(250)).await;
    let r1 = journal_entries::reverse(&pool, co, charge, u)
        .await
        .unwrap();
    let r2 = journal_entries::reverse(&pool, co, income, u)
        .await
        .unwrap();
    assert_eq!(r1.entry.entry_date, today(), "montage : datée du jour");
    assert_eq!(r2.entry.reverses_entry_id, Some(income), "montage");

    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;
    let reversal_month = month_index(&b, today());
    assert_ne!(reversal_month, 0, "montage : deux mois différents");
    assert_eq!(month_at(&b, 0), (dec!(40), dec!(100)), "mois M : {b}");
    assert_eq!(
        month_at(&b, reversal_month),
        (dec!(0), dec!(0)),
        "mois M+k : {b}"
    );
    assert_eq!(
        post_numbers(&b),
        vec!["6100"],
        "le poste ne bouge pas : {b}"
    );
    assert_eq!(post_amounts(&b), vec![dec!(100)], "{b}");
    assert_eq!(b["creditBalanceExpenseAccounts"], 0, "{b}");
    assert_invariant(&app, &c.jwt, &b).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reversal_across_two_years_counts_on_both_sides(pool: MySqlPool) {
    let c = setup(&pool, "cheval").await;
    let prev = c.previous_year(&pool).await;
    let (u, co) = (c.user_id, c.company_id);
    let origin_date = last_day_before(year_start());
    let origin = post(
        &pool,
        u,
        prev,
        co,
        origin_date,
        c.expense,
        c.bank,
        dec!(400),
    )
    .await;
    let reversal = journal_entries::reverse_at_next_fiscal_year_start(&pool, co, origin, u)
        .await
        .unwrap();
    assert_eq!(reversal.entry.entry_date, year_start(), "montage");
    assert_eq!(reversal.entry.fiscal_year_id, c.fy_id, "montage");

    let app = spawn_app(pool.clone()).await;

    // Lue depuis N : la contre-passation compte, à −400, au premier mois.
    let b = app.summary(&c.jwt).await;
    assert_eq!(b["fiscalYear"]["id"], c.fy_id);
    assert_eq!(month_at(&b, 0), (dec!(0), dec!(-400)), "{b}");
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(-400), "{b}");
    assert_eq!(b["creditBalanceExpenseAccounts"], 1, "{b}");
    assert_invariant(&app, &c.jwt, &b).await;

    // Lue depuis N−1, encore ouvert : l'origine compte, à son mois. La route lit
    // l'horloge : c'est `generate` qui reçoit une date de N−1.
    let read = homepage_summary::generate(&pool, co, origin_date)
        .await
        .unwrap();
    let from_prev = serde_json::to_value(&read).unwrap();
    assert_eq!(from_prev["fiscalYear"]["id"], prev, "{from_prev}");
    let i = month_index(&from_prev, origin_date);
    assert_eq!(month_at(&from_prev, i), (dec!(0), dec!(400)), "{from_prev}");
    assert_eq!(dec_of(&from_prev["totalExpenses"]), dec!(400));
    assert_invariant(&app, &c.jwt, &from_prev).await;
}

/// Revue P1 (B-1, E-1, A-1) — un côté d'une paire daté hors des bornes de son exercice,
/// l'anomalie que signale la préparation de clôture, posée en SQL. Le compte de résultat
/// ne retient que l'autre côté : la synthèse aussi. Écarter la paire sur le seul
/// `fiscal_year_id` rendrait 0 là où le compte de résultat rend ±X.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reversal_pair_with_a_side_dated_outside_the_year_follows_the_income_statement(
    pool: MySqlPool,
) {
    let c = setup(&pool, "paireanormale").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    let outside = last_day_before(year_start());
    // L'origine hors bornes : reste la contre-passation, −500 au mois du jour.
    let charge = c.charge(&pool, month_k(0), c.expense, dec!(500)).await;
    journal_entries::reverse(&pool, co, charge, u)
        .await
        .unwrap();
    // La contre-passation hors bornes : reste l'origine, +300 au mois 0.
    let income = post(&pool, u, f, co, month_k(0), c.bank, c.revenue, dec!(300)).await;
    let reversal = journal_entries::reverse(&pool, co, income, u)
        .await
        .unwrap();
    for id in [charge, reversal.entry.id] {
        sqlx::query("UPDATE journal_entries SET entry_date = ? WHERE id = ?")
            .bind(outside)
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;
    let today_month = month_index(&b, today());
    assert_ne!(today_month, 0, "montage : deux mois différents");
    assert_eq!(month_at(&b, 0), (dec!(300), dec!(0)), "{b}");
    assert_eq!(month_at(&b, today_month), (dec!(0), dec!(-500)), "{b}");
    assert_eq!(b["creditBalanceExpenseAccounts"], 1, "{b}");
    assert_invariant(&app, &c.jwt, &b).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cancelled_supplier_invoice_disappears(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let f = fournisseur_rapproche::preparer(&pool, &seeded).await;
    let validated = supplier_invoices::create_validated(
        &pool,
        NewSupplierInvoice {
            company_id: seeded.company_id,
            contact_id: f.supplier_id,
            supplier_invoice_number: Some("F-33".into()),
            invoice_date: today(),
            due_date: None,
            creditor_iban: Some(fournisseur_rapproche::IBAN_CREANCIER.into()),
            creditor_qr_iban: None,
            payment_reference: None,
            expected_payment_amount: None,
            project_id: None,
            lines: vec![NewSupplierInvoiceLine {
                description: "Fourniture".into(),
                quantity: dec!(1),
                unit_price: dec!(120),
                vat_rate: dec!(0),
                expense_account_id: seeded.accounts["4000"],
            }],
        },
        seeded.admin_user_id,
    )
    .await
    .unwrap();
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    let app = spawn_app(pool.clone()).await;

    let before = app.summary(&jwt).await;
    assert_eq!(
        dec_of(&before["totalExpenses"]),
        dec!(120),
        "montage : la facture compte : {before}"
    );

    supplier_invoices::cancel(
        &pool,
        seeded.company_id,
        validated.invoice.id,
        seeded.admin_user_id,
    )
    .await
    .unwrap();

    let after = app.summary(&jwt).await;
    assert_eq!(
        month_at(&after, month_index(&after, today())),
        (dec!(0), dec!(0)),
        "{after}"
    );
    assert_eq!(dec_of(&after["totalExpenses"]), dec!(0), "{after}");
    assert_eq!(after["expensePosts"], serde_json::json!([]), "{after}");
    assert_eq!(after["creditBalanceExpenseAccounts"], 0, "{after}");
    assert_invariant(&app, &jwt, &after).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_note_stays_counted(pool: MySqlPool) {
    let seeded = seed_accounting_company(&pool).await.unwrap();
    let (admin, company) = (seeded.admin_user_id, seeded.company_id);
    let contact = contacts::create(
        &pool,
        admin,
        NewContact {
            company_id: company,
            contact_type: ContactType::Personne,
            name: "Client X".into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
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
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap()
    .id;
    // La facture le mois dernier, l'avoir aujourd'hui : un mois à revenus négatifs.
    let invoice_date = today()
        .with_day(1)
        .unwrap()
        .checked_sub_months(Months::new(1))
        .unwrap();
    let (draft, _) = invoices::create(
        &pool,
        admin,
        NewInvoice {
            company_id: company,
            contact_id: contact,
            date: invoice_date,
            due_date: Some(invoice_date),
            payment_terms: None,
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation".into(),
                quantity: dec!(1),
                unit_price: dec!(100),
                vat_rate: dec!(0),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(&pool, company, draft.id, admin)
        .await
        .unwrap();
    let note = credit_notes::create_credit_note(
        &pool,
        NewCreditNote {
            company_id: company,
            invoice_id: draft.id,
            date: today(),
        },
        admin,
    )
    .await
    .unwrap();
    let note_entry: Option<i64> =
        sqlx::query_scalar("SELECT reverses_entry_id FROM journal_entries WHERE id = ?")
            .bind(note.credit_note.journal_entry_id.unwrap())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(
        note_entry, None,
        "montage : l'avoir n'est pas une contre-passation"
    );

    let jwt = forge_jwt(admin, "Admin", company);
    let app = spawn_app(pool).await;
    let b = app.summary(&jwt).await;
    assert_eq!(
        month_at(&b, month_index(&b, invoice_date)).0,
        dec!(100),
        "{b}"
    );
    assert_eq!(month_at(&b, month_index(&b, today())).0, dec!(-100), "{b}");
    assert_eq!(dec_of(&b["totalRevenues"]), dec!(0), "{b}");
    assert_invariant(&app, &jwt, &b).await;
}

// ============================================================
// AC11 — la clôture
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn closing_a_year_writes_no_entry_and_changes_nothing(pool: MySqlPool) {
    let c = setup(&pool, "cloture").await;
    let prev = c.previous_year(&pool).await;
    let (u, co) = (c.user_id, c.company_id);
    post(
        &pool,
        u,
        prev,
        co,
        last_day_before(year_start()),
        c.expense,
        c.bank,
        dec!(45),
    )
    .await;
    c.charge(&pool, month_k(1), c.expense, dec!(60)).await;

    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM journal_entries")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let app = spawn_app(pool.clone()).await;
    let body_before = app
        .client
        .get(app.url(ROUTE))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let before = count().await;

    fiscal_years::close(&pool, u, co, prev).await.unwrap();

    const WHY: &str = "Une clôture écrit désormais : Guy a décidé d'écarter les écritures de \
                       clôture des graphiques (#918, C-e332-8), et cette synthèse ne sait pas les \
                       reconnaître. À traiter ici.";
    assert_eq!(count().await, before, "{WHY}");
    let body_after = app
        .client
        .get(app.url(ROUTE))
        .bearer_auth(&c.jwt)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    assert_eq!(body_after, body_before, "{WHY}");
    assert!(body_before.contains("\"state\":\"open\""), "{body_before}");
}

// ============================================================
// AC12 à AC16 — les postes
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn top_five_and_others(pool: MySqlPool) {
    let c = setup(&pool, "cinq").await;
    // Sept comptes de soldes distincts, créés dans le désordre.
    for (number, amount) in [
        ("6300", dec!(30)),
        ("6100", dec!(70)),
        ("6600", dec!(10)),
        ("6200", dec!(50)),
        ("6400", dec!(40)),
        ("6500", dec!(20)),
        ("6050", dec!(60)),
    ] {
        let account = c.expense_account(&pool, number).await;
        c.charge(&pool, month_k(0), account, amount).await;
    }
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(
        post_numbers(&b),
        vec!["6100", "6050", "6200", "6400", "6300"],
        "{b}"
    );
    assert_eq!(
        post_amounts(&b),
        vec![dec!(70), dec!(60), dec!(50), dec!(40), dec!(30)],
        "{b}"
    );
    assert_eq!(dec_of(&b["otherExpenses"]["amount"]), dec!(30), "{b}");
    assert_eq!(b["otherExpenses"]["accountCount"], 2, "{b}");
    assert_eq!(dec_of(&b["expensePostsBase"]), dec!(280), "{b}");
    assert_eq!(b["expensePosts"][0]["name"], "Charge 6100");
    assert!(b["expensePosts"][0]["accountId"].is_i64(), "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn boundaries_five_six_and_ties(pool: MySqlPool) {
    let c = setup(&pool, "bornes").await;
    let app = spawn_app(pool.clone()).await;
    // Cinq comptes à égalité, créés dans l'ordre inverse des numéros.
    for number in ["6500", "6400", "6300", "6200", "6100"] {
        let account = c.expense_account(&pool, number).await;
        c.charge(&pool, month_k(0), account, dec!(10)).await;
    }
    let five = app.summary(&c.jwt).await;
    assert_eq!(
        post_numbers(&five),
        vec!["6100", "6200", "6300", "6400", "6500"],
        "à égalité, par numéro : {five}"
    );
    assert_eq!(five["otherExpenses"], Value::Null, "{five}");

    // Un sixième : « Autres » porte un compte, le dernier des numéros.
    let account = c.expense_account(&pool, "6600").await;
    c.charge(&pool, month_k(0), account, dec!(10)).await;
    let six = app.summary(&c.jwt).await;
    assert_eq!(post_numbers(&six).len(), 5, "{six}");
    assert_eq!(six["otherExpenses"]["accountCount"], 1, "{six}");
    assert_eq!(dec_of(&six["otherExpenses"]["amount"]), dec!(10), "{six}");
    assert_eq!(six["otherExpenses"]["share"], "16.7", "{six}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn credit_balance_expense_accounts_are_left_out(pool: MySqlPool) {
    let c = setup(&pool, "crediteur").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    // Un compte à +100, un à −300 : le total est négatif, le poste reste.
    let credit = c.expense_account(&pool, "6100").await;
    c.charge(&pool, month_k(0), c.expense, dec!(100)).await;
    post(&pool, u, f, co, month_k(0), c.bank, credit, dec!(300)).await;
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(dec_of(&b["totalExpenses"]), dec!(-200), "{b}");
    assert_eq!(post_numbers(&b), vec!["6000"], "{b}");
    assert_eq!(b["expensePosts"][0]["share"], "100.0", "{b}");
    assert_eq!(b["otherExpenses"], Value::Null, "{b}");
    assert_eq!(dec_of(&b["expensePostsBase"]), dec!(100), "{b}");
    assert_eq!(b["creditBalanceExpenseAccounts"], 1, "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn no_positive_balance_leaves_an_empty_pie(pool: MySqlPool) {
    let c = setup(&pool, "toutcredit").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    post(&pool, u, f, co, month_k(0), c.bank, c.expense, dec!(300)).await;
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(b["expensePosts"], serde_json::json!([]), "{b}");
    assert_eq!(b["expensePostsBase"], "0", "{b}");
    assert_eq!(b["creditBalanceExpenseAccounts"], 1, "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn zero_balance_account_is_counted_nowhere(pool: MySqlPool) {
    let c = setup(&pool, "nul").await;
    let (u, f, co) = (c.user_id, c.fy_id, c.company_id);
    // Deux lignes qui se compensent sans former une paire.
    let zero = c.expense_account(&pool, "6100").await;
    c.charge(&pool, month_k(0), zero, dec!(75)).await;
    post(&pool, u, f, co, month_k(1), c.bank, zero, dec!(75)).await;
    c.charge(&pool, month_k(0), c.expense, dec!(10)).await;
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;

    assert_eq!(post_numbers(&b), vec!["6000"], "{b}");
    assert_eq!(b["otherExpenses"], Value::Null, "{b}");
    assert_eq!(
        b["creditBalanceExpenseAccounts"], 0,
        "un compte à solde nul n'est compté nulle part : {b}"
    );
    assert_eq!(dec_of(&b["expensePostsBase"]), dec!(10), "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn shares_are_rounded_once(pool: MySqlPool) {
    let c = setup(&pool, "parts").await;
    let second = c.expense_account(&pool, "6100").await;
    c.charge(&pool, month_k(0), c.expense, dec!(200)).await;
    c.charge(&pool, month_k(0), second, dec!(100)).await;
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;
    assert_eq!(post_shares(&b), vec!["66.7", "33.3"], "{b}");
    assert_eq!(dec_of(&b["expensePostsBase"]), dec!(300), "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archived_account_stays_a_post(pool: MySqlPool) {
    let c = setup(&pool, "archive").await;
    let archived = c.expense_account(&pool, "6100").await;
    c.charge(&pool, month_k(0), archived, dec!(90)).await;
    accounts::archive(&pool, archived, 1, c.user_id)
        .await
        .unwrap();
    let app = spawn_app(pool).await;
    let b = app.summary(&c.jwt).await;
    assert_eq!(post_numbers(&b), vec!["6100"], "{b}");
    assert_eq!(b["expensePosts"][0]["name"], "Charge 6100", "{b}");
    assert_invariant(&app, &c.jwt, &b).await;
}

// ============================================================
// AC17, AC19 — les droits, l'absence d'écriture
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn any_role_reads_without_token_refused(pool: MySqlPool) {
    let c = setup(&pool, "roles").await;
    c.charge(&pool, month_k(0), c.expense, dec!(5)).await;
    let consult = new_user(&pool, "roles_consult", Role::Consultation, c.company_id).await;
    let compta = new_user(&pool, "roles_compta", Role::Comptable, c.company_id).await;
    let app = spawn_app(pool).await;

    let anonymous = app.client.get(app.url(ROUTE)).send().await.unwrap();
    assert_eq!(anonymous.status(), 401);

    for jwt in [
        forge_jwt(consult, "Consultation", c.company_id),
        forge_jwt(compta, "Comptable", c.company_id),
        c.jwt.clone(),
    ] {
        let b = app.summary(&jwt).await;
        assert_eq!(dec_of(&b["totalExpenses"]), dec!(5), "{b}");
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn api_key_reads(pool: MySqlPool) {
    let c = setup(&pool, "cle").await;
    c.charge(&pool, month_k(0), c.expense, dec!(6)).await;
    let (token, key_hash) = kesh_api::auth::api_key::generate_pat();
    let mut tx = pool.begin().await.unwrap();
    api_keys::create_in_tx(
        &mut tx,
        NewApiKey {
            company_id: c.company_id,
            created_by_user_id: c.user_id,
            name: "lecture".into(),
            key_hash,
            scope: ApiKeyScope::Read,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let app = spawn_app(pool).await;
    let b = app.summary(&token).await;
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(6), "{b}");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reading_writes_nothing(pool: MySqlPool) {
    let c = setup(&pool, "trace").await;
    c.charge(&pool, month_k(0), c.expense, dec!(7)).await;
    let count = || async {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM audit_log")
            .fetch_one(&pool)
            .await
            .unwrap()
    };
    let app = spawn_app(pool.clone()).await;
    let before = count().await;
    let b = app.summary(&c.jwt).await;
    app.summary(&c.jwt).await;
    assert_eq!(dec_of(&b["totalExpenses"]), dec!(7), "contrôle : {b}");
    assert_eq!(count().await, before, "la lecture n'écrit aucune trace");
}
