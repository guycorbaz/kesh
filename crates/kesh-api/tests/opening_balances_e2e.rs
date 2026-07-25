//! Tests E2E API — Story 14-4 Bilan d'ouverture (soldes de départ).
//!
//! Couvre AC-A/B/D/H : happy path (OD datée `fy_start`, présente au bilan,
//! description en langue comptable de la company), report d'ouverture négatif
//! (perte reportée), toutes les gardes (déséquilibre, < 2 lignes, montant
//! négatif, 0 ligne, compte de résultat, company non-vierge, exercice absent,
//! premier exercice clos, cross-tenant), RBAC POST + GET, les 4 `reason` du
//! `GET /status`, et la course concurrente (P1-M3-BH, deux POST simultanés).
//!
//! Contrat d'assertion (P1-H1-BH) : sur `ALREADY_HAS_ENTRIES` et
//! `non-balance-account`, les tests assertent le **`error.message`** distinct
//! (le `error.code` est partagé — `ILLEGAL_STATE_TRANSITION` /
//! `VALIDATION_ERROR`).

mod common;

use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    AccountRole, NewAccount, NewFiscalYear, NewJournalEntry, NewJournalEntryLine,
};
use kesh_db::repositories::{accounts, fiscal_years, journal_entries};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;
use std::net::SocketAddr;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
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

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

/// Setup minimal : company + admin user + login → token Comptable+ (Admin).
async fn bootstrap_admin(pool: &MySqlPool) -> (TestApp, String) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    (app, token)
}

/// Crée un user `Consultation` et retourne son token (tests RBAC).
async fn create_consultation_user_and_login(app: &TestApp, pool: &MySqlPool) -> String {
    use kesh_db::entities::{NewUser, Role};

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();

    let password_plain = "consultation-test-pw-12345";
    let hash = kesh_api::auth::password::hash_password(password_plain).expect("hash");

    kesh_db::repositories::users::create(
        pool,
        NewUser {
            username: "consultation".into(),
            password_hash: hash,
            role: Role::Consultation,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("create consultation user");

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "consultation", "password": password_plain }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "consultation login should succeed");
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

// ---------------------------------------------------------------------------
// Fixtures DB (via repos, miroir reports_e2e.rs)
// ---------------------------------------------------------------------------

async fn admin_ids(pool: &MySqlPool) -> (i64, i64) {
    let (user_id, company_id): (i64, i64) =
        sqlx::query_as("SELECT id, company_id FROM users WHERE username = 'admin'")
            .fetch_one(pool)
            .await
            .unwrap();
    (user_id, company_id)
}

async fn create_fy(pool: &MySqlPool, user_id: i64, company_id: i64, year: i32) -> i64 {
    fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {year}"),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
    role: Option<AccountRole>,
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
            role,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

/// Seed standard : FY 2026 Open + 1000 (Asset) + 2970 (Liability,
/// RetainedEarnings) + 2000 (Liability sans rôle).
struct Seed {
    user_id: i64,
    company_id: i64,
    fy_id: i64,
    asset: i64,
    retained: i64,
    liability: i64,
}

async fn seed_ready(pool: &MySqlPool) -> Seed {
    let (user_id, company_id) = admin_ids(pool).await;
    let fy_id = create_fy(pool, user_id, company_id, 2026).await;
    let asset = create_acc(
        pool,
        user_id,
        company_id,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;
    let retained = create_acc(
        pool,
        user_id,
        company_id,
        "2970",
        "Report à nouveau",
        AccountType::Liability,
        Some(AccountRole::RetainedEarnings),
    )
    .await;
    let liability = create_acc(
        pool,
        user_id,
        company_id,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;
    Seed {
        user_id,
        company_id,
        fy_id,
        asset,
        retained,
        liability,
    }
}

/// Poste une écriture normale équilibrée via le repo (pour rendre la company
/// non-vierge).
async fn post_normal_entry(pool: &MySqlPool, seed: &Seed, fy_id: i64, date: NaiveDate) {
    journal_entries::create(
        pool,
        fy_id,
        seed.user_id,
        NewJournalEntry {
            company_id: seed.company_id,
            entry_date: date,
            journal: Journal::OD,
            description: "Écriture normale".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: seed.asset,
                    debit: dec!(50),
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: seed.liability,
                    debit: Decimal::ZERO,
                    credit: dec!(50),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
}

fn line(account_id: i64, debit: &str, credit: &str) -> Value {
    json!({ "accountId": account_id, "debit": debit, "credit": credit })
}

// ===========================================================================
// POST — happy paths
// ===========================================================================

/// AC-A : 201, écriture OD datée `fy_start`, description dans la langue
/// comptable de la company (fr-CH), présente au bilan (compte physique 2970
/// itemisé dans equity, ligne calculée `retainedEarnings` = 0).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_happy_path_od_at_fy_start_appears_in_balance_sheet(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "1000.00", "0"),
            line(seed.retained, "0", "1000.00"),
        ]}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 201);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["journal"], "OD", "journal forcé serveur");
    assert_eq!(
        body["entryDate"], "2026-01-01",
        "entry_date = fy_start forcée serveur"
    );
    assert_eq!(body["fiscalYearId"].as_i64(), Some(seed.fy_id));
    // Description rendue en fr-CH (accounting_language de Test Company) —
    // P1-H1 : champ persistant dans la langue comptable de la company.
    assert_eq!(
        body["description"], "Bilan d’ouverture — soldes de départ",
        "description dans la langue comptable de la company"
    );
    assert_eq!(body["lines"].as_array().unwrap().len(), 2);

    // Présence immédiate au bilan (calcul cumulatif, aucune modif balance_sheet).
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            seed.fy_id
        )))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bs: Value = resp.json().await.unwrap();
    let total_assets: Decimal = bs["totalAssets"].as_str().unwrap().parse().unwrap();
    assert_eq!(total_assets, dec!(1000));
    // Compte physique 2970 itemisé dans la section Capitaux propres (14-3c D1).
    let equity = bs["equity"].as_array().unwrap();
    let physical = equity
        .iter()
        .find(|a| a["accountNumber"] == "2970")
        .expect("compte physique 2970 dans equity");
    let physical_balance: Decimal = physical["balance"].as_str().unwrap().parse().unwrap();
    assert_eq!(physical_balance, dec!(1000));
    // Ligne CALCULÉE « Résultat reporté » = 0 (aucun exercice Kesh antérieur ;
    // l'OD datée = fy_start n'alimente jamais `entry_date < fy_start`).
    let retained_calc: Decimal = bs["retainedEarnings"].as_str().unwrap().parse().unwrap();
    assert_eq!(retained_calc, Decimal::ZERO);
    assert_eq!(bs["equationHolds"], true);
}

/// AC-A happy path bis (P3-BH3-5) : report d'ouverture NÉGATIF (perte
/// reportée) = **débit** sur le compte de rôle RetainedEarnings. L'équilibre
/// tient (100 débit actifs + 20 débit report = 50 crédit dettes + 70 crédit
/// capital) et l'equity affiche un solde débiteur.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_happy_path_negative_retained_loss(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let capital = create_acc(
        &pool,
        seed.user_id,
        seed.company_id,
        "2800",
        "Capital",
        AccountType::Liability,
        Some(AccountRole::EquityCapital),
    )
    .await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "100.00", "0"),
            line(seed.retained, "20.00", "0"),
            line(seed.liability, "0", "50.00"),
            line(capital, "0", "70.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);

    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/balance-sheet?fiscalYearId={}",
            seed.fy_id
        )))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let bs: Value = resp.json().await.unwrap();
    let equity = bs["equity"].as_array().unwrap();
    let physical = equity
        .iter()
        .find(|a| a["accountNumber"] == "2970")
        .expect("compte 2970 dans equity");
    let physical_balance: Decimal = physical["balance"].as_str().unwrap().parse().unwrap();
    assert_eq!(
        physical_balance,
        dec!(-20),
        "solde equity débiteur (perte reportée)"
    );
    assert_eq!(bs["equationHolds"], true);
}

// ===========================================================================
// POST — gardes de validation (AC-B)
// ===========================================================================

/// Body déséquilibré → 400 `ENTRY_UNBALANCED`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_unbalanced_400_entry_unbalanced(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "1000.00", "0"),
            line(seed.retained, "0", "900.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ENTRY_UNBALANCED");
}

/// Exactement 1 ligne → 400 `VALIDATION_ERROR` `EntryNeedsTwoLines`
/// (P3-ECH-LOW-3 : UNE ligne, pas 2 dont une vide qui donnerait
/// `EntryLineDebitCreditExclusive`). Assert du **message** distinct.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_single_line_400_needs_two_lines(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [ line(seed.asset, "100.00", "0") ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("deux lignes"),
        "message EntryNeedsTwoLines attendu, obtenu: {msg}"
    );
}

/// Montant négatif → 400 `VALIDATION_ERROR` `EntryNegativeAmount` (assert message).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_negative_amount_400(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "-100.00", "0"),
            line(seed.retained, "0", "-100.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("négatif"),
        "message EntryNegativeAmount attendu, obtenu: {msg}"
    );
}

/// `{ lines: [] }` → 400 `VALIDATION_ERROR`, PAS 500 (garde `ids.is_empty()`
/// dans `find_types_by_ids_in_tx`, P3-ECH-LOW-1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_empty_lines_400_not_500(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let _seed = seed_ready(&pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Ligne sur un compte Revenue → 400 `VALIDATION_ERROR` avec le **message**
/// `error-opening-balances-non-balance-account` (garde D4 — le code reste
/// `VALIDATION_ERROR`, P3-AA-1/BH3-6).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_revenue_account_400_non_balance_account(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let revenue = create_acc(
        &pool,
        seed.user_id,
        seed.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
        None,
    )
    .await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "100.00", "0"),
            line(revenue, "0", "100.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("comptes de bilan"),
        "message non-balance-account attendu, obtenu: {msg}"
    );
}

/// Company avec ≥1 écriture (posée dans un exercice POSTÉRIEUR, pas le
/// premier — garde company-wide P3-BH3-1) → 409 avec le **message** distinct
/// `already-has-entries` (le code `ILLEGAL_STATE_TRANSITION` est partagé,
/// P1-H1-BH — c'est le message qu'on asserte).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_already_has_entries_409_distinct_message(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let fy_later = create_fy(&pool, seed.user_id, seed.company_id, 2027).await;
    post_normal_entry(
        &pool,
        &seed,
        fy_later,
        NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
    )
    .await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "1000.00", "0"),
            line(seed.retained, "0", "1000.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("contient déjà des écritures"),
        "message distinct already-has-entries attendu, obtenu: {msg}"
    );
}

/// Aucun exercice → 400 `VALIDATION_ERROR` (assert message no-fiscal-year).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_no_fiscal_year_400(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let (user_id, company_id) = admin_ids(&pool).await;
    // Comptes mais AUCUN exercice.
    let asset = create_acc(
        &pool,
        user_id,
        company_id,
        "1000",
        "Banque",
        AccountType::Asset,
        None,
    )
    .await;
    let liab = create_acc(
        &pool,
        user_id,
        company_id,
        "2000",
        "Dettes",
        AccountType::Liability,
        None,
    )
    .await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(asset, "100.00", "0"),
            line(liab, "0", "100.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("Aucun exercice"),
        "message no-fiscal-year attendu, obtenu: {msg}"
    );
}

/// Premier exercice Closed → 400 avec le message distinct 14-4
/// `first-year-closed` — code `VALIDATION_ERROR`, PAS `FISCAL_YEAR_CLOSED`
/// (P3-AA-2 : pré-check handler et re-check sous-lock épinglés au même outcome).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_first_year_closed_400_distinct_message(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    fiscal_years::close(&pool, seed.user_id, seed.company_id, seed.fy_id)
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "100.00", "0"),
            line(seed.retained, "0", "100.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "VALIDATION_ERROR",
        "code VALIDATION_ERROR (pas FISCAL_YEAR_CLOSED), P3-AA-2"
    );
    let msg = body["error"]["message"].as_str().unwrap_or("");
    assert!(
        msg.contains("premier exercice est clôturé"),
        "message distinct first-year-closed attendu, obtenu: {msg}"
    );
}

/// Cross-tenant : une ligne référence un compte d'une AUTRE company →
/// l'id absent du résultat de la garde de type retombe dans `create_in_tx` →
/// 400 `INACTIVE_OR_INVALID_ACCOUNTS` (PAS `non-balance-account`, P3-AA-1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_cross_tenant_account_inactive_or_invalid(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    // Compte Asset d'une autre company (passe la garde de type par absence).
    let other_company = kesh_db::repositories::companies::create(
        &pool,
        kesh_db::entities::NewCompany {
            name: "Autre SA".into(),
            first_name: None,
            last_name: None,
            address_structured: kesh_db::entities::address::StructuredAddress {
                street: "X".into(),
                building: String::new(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: kesh_db::entities::OrgType::Independant,
            accounting_language: kesh_db::entities::Language::Fr,
            instance_language: kesh_db::entities::Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    let other_user = kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "other-admin".into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$YWFhYWFhYWFhYWFhYWFhYQ$0000000000000000000000000000000000000000000".into(),
            role: kesh_db::entities::Role::Admin,
            active: true,
            company_id: other_company,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let foreign_asset = create_acc(
        &pool,
        other_user,
        other_company,
        "1000",
        "Banque étrangère",
        AccountType::Asset,
        None,
    )
    .await;

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&json!({ "lines": [
            line(seed.asset, "100.00", "0"),
            line(foreign_asset, "0", "100.00"),
        ]}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INACTIVE_OR_INVALID_ACCOUNTS");
}

// ===========================================================================
// POST — RBAC
// ===========================================================================

/// Consultation → 403 ; non-auth → 401.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_rbac_consultation_403_unauth_401(pool: MySqlPool) {
    let (app, _token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let consultation_token = create_consultation_user_and_login(&app, &pool).await;

    let body = json!({ "lines": [
        line(seed.asset, "100.00", "0"),
        line(seed.retained, "0", "100.00"),
    ]});

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&consultation_token))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "Consultation → 403");

    let resp = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "non-auth → 401");
}

// ===========================================================================
// GET /status — les 4 reasons + RBAC (P1-M1-AA / P1-L-4)
// ===========================================================================

async fn get_status(app: &TestApp, token: &str) -> Value {
    let resp = app
        .client
        .get(app.url("/api/v1/opening-balances/status"))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

/// `NO_FISCAL_YEAR` : aucun exercice.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn status_no_fiscal_year(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let body = get_status(&app, &token).await;
    assert_eq!(body["reason"], "NO_FISCAL_YEAR");
    assert_eq!(body["canEnter"], false);
    assert!(body["fiscalYear"].is_null());
}

/// `READY` : premier exercice Open + company vierge.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn status_ready(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let body = get_status(&app, &token).await;
    assert_eq!(body["reason"], "READY");
    assert_eq!(body["canEnter"], true);
    assert_eq!(body["fiscalYear"]["id"].as_i64(), Some(seed.fy_id));
    assert_eq!(body["fiscalYear"]["startDate"], "2026-01-01");
    assert_eq!(body["fiscalYear"]["status"], "Open");
}

/// `FIRST_YEAR_CLOSED` : premier exercice clos (prioritaire sur
/// ALREADY_HAS_ENTRIES dans l'ordre d'évaluation).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn status_first_year_closed(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    fiscal_years::close(&pool, seed.user_id, seed.company_id, seed.fy_id)
        .await
        .unwrap();
    let body = get_status(&app, &token).await;
    assert_eq!(body["reason"], "FIRST_YEAR_CLOSED");
    assert_eq!(body["canEnter"], false);
    assert_eq!(body["fiscalYear"]["status"], "Closed");
}

/// `ALREADY_HAS_ENTRIES` : une écriture dans un exercice QUELCONQUE (ici un
/// exercice postérieur, pas le premier — garde company-wide P3-BH3-1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn status_already_has_entries_any_fiscal_year(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;
    let fy_later = create_fy(&pool, seed.user_id, seed.company_id, 2027).await;
    post_normal_entry(
        &pool,
        &seed,
        fy_later,
        NaiveDate::from_ymd_opt(2027, 3, 1).unwrap(),
    )
    .await;

    let body = get_status(&app, &token).await;
    assert_eq!(body["reason"], "ALREADY_HAS_ENTRIES");
    assert_eq!(body["canEnter"], false);
    // Le premier exercice (2026) reste rapporté — c'est lui qui daterait
    // l'écriture, l'écran affiche l'état verrouillé.
    assert_eq!(body["fiscalYear"]["id"].as_i64(), Some(seed.fy_id));
}

/// RBAC `GET /status` (P1-L-4) : Consultation → 403 ; non-auth → 401.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn status_rbac_consultation_403_unauth_401(pool: MySqlPool) {
    let (app, _token) = bootstrap_admin(&pool).await;
    let _seed = seed_ready(&pool).await;
    let consultation_token = create_consultation_user_and_login(&app, &pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/opening-balances/status"))
        .header("Authorization", auth(&consultation_token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "Consultation → 403");

    let resp = app
        .client
        .get(app.url("/api/v1/opening-balances/status"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "non-auth → 401");
}

// ===========================================================================
// Course concurrente (P1-M3-BH) — deux POST simultanés
// ===========================================================================

/// Deux générations HTTP simultanées sur la même company vierge : exactement
/// une réussit (201), l'autre reçoit 409 `ALREADY_HAS_ENTRIES` — état final =
/// UNE écriture (sérialisation par `fiscal_years FOR UPDATE`, miroir
/// `reopen_close_concurrent_is_serialized` 14-2).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_concurrent_generation_only_one_succeeds(pool: MySqlPool) {
    let (app, token) = bootstrap_admin(&pool).await;
    let seed = seed_ready(&pool).await;

    let body = json!({ "lines": [
        line(seed.asset, "1000.00", "0"),
        line(seed.retained, "0", "1000.00"),
    ]});

    let req1 = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&body)
        .send();
    let req2 = app
        .client
        .post(app.url("/api/v1/opening-balances"))
        .header("Authorization", auth(&token))
        .json(&body)
        .send();

    let (r1, r2) = tokio::join!(req1, req2);
    let r1 = r1.unwrap();
    let r2 = r2.unwrap();
    let statuses = [r1.status().as_u16(), r2.status().as_u16()];

    assert_eq!(
        statuses.iter().filter(|s| **s == 201).count(),
        1,
        "exactement un 201, obtenu {statuses:?}"
    );
    assert_eq!(
        statuses.iter().filter(|s| **s == 409).count(),
        1,
        "exactement un 409, obtenu {statuses:?}"
    );

    // État final : UNE seule écriture, pas de doublon.
    let count = journal_entries::count_by_company(&pool, seed.company_id)
        .await
        .unwrap();
    assert_eq!(count, 1);
}
