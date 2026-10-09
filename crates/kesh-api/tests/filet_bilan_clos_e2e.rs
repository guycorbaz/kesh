//! Tests E2E — Story 15-12b (#543) : **le filet sous un bilan clos**, par les
//! routes.
//!
//! L'état fautif « exercice ouvert suivi d'un exercice clos » n'est plus
//! atteignable par l'API depuis la Story 15-12a : il est posé **par SQL direct**
//! (connexion distincte, validée), comme dans les tests de la 15-8a.
//!
//! Couvre (AC 20) : la saisie manuelle (`POST /journal-entries`, `400` et corps
//! complet, message élargi à la saisie) et la contre-passation **par la route**
//! (datée du jour : l'exercice qui couvre le jour est suivi d'un exercice clos).
//! Chaque test compte les écritures avant et après — un refus qui laisse une
//! trace partielle n'est pas un refus. Les autres familles sont couvertes côté
//! base (`kesh-db/tests/filet_bilan_clos.rs`) et l'acceptation par lot dans
//! `reconciliation_e2e.rs` / `reconciliation_rules_e2e.rs`.

mod common;

use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewAccount, NewFiscalYear, NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::{accounts, fiscal_years, journal_entries};
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

async fn login(app: &TestApp, user: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": user, "password": password }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

/// Company + admin + exercice OUVERT couvrant **aujourd'hui**.
///
/// ⚠️ L'exercice couvre le jour courant et non une date figée : la
/// contre-passation porte la date du **jour** (D4), un exercice de 2026 en dur
/// ferait donc échouer la suite en 2027 — un test qui pourrit sans qu'on l'ait
/// touché.
async fn setup(pool: &MySqlPool) -> (TestApp, String, i64, i64) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;

    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();

    let today = Utc::now().date_naive();
    let fy = fiscal_years::create(
        pool,
        1,
        NewFiscalYear {
            company_id,
            name: format!("Exercice {}", today.format("%Y")),
            start_date: NaiveDate::from_ymd_opt(
                today.format("%Y").to_string().parse().unwrap(),
                1,
                1,
            )
            .unwrap(),
            end_date: NaiveDate::from_ymd_opt(
                today.format("%Y").to_string().parse().unwrap(),
                12,
                31,
            )
            .unwrap(),
        },
    )
    .await
    .expect("exercice");

    (app, token, company_id, fy.id)
}

async fn make_account(pool: &MySqlPool, company_id: i64, number: &str, ty: AccountType) -> i64 {
    accounts::create(
        pool,
        1,
        NewAccount {
            company_id,
            number: number.to_string(),
            name: format!("Compte {number}"),
            account_type: ty,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .expect("compte")
    .id
}

fn annee_courante() -> i32 {
    Utc::now()
        .date_naive()
        .format("%Y")
        .to_string()
        .parse()
        .unwrap()
}

/// Un exercice annuel au statut voulu, posé par SQL (la création refuserait un
/// exercice antérieur à un exercice clos ; seul l'état importe ici).
async fn exercice_sql(pool: &MySqlPool, company_id: i64, annee: i32, status: &str) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(format!("Exercice {annee}"))
    .bind(NaiveDate::from_ymd_opt(annee, 1, 1).unwrap())
    .bind(NaiveDate::from_ymd_opt(annee, 12, 31).unwrap())
    .bind(status)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn ecritures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le corps complet d'un `400 LATER_FISCAL_YEAR_CLOSED` : code, message élargi
/// à la saisie (Story 15-12b), `details` exacts — et rien d'autre.
fn assert_corps_complet(body: &Value, fy_id: i64, nom: &str) {
    let error = body["error"].as_object().expect("objet error");
    let mut cles: Vec<&str> = error.keys().map(String::as_str).collect();
    cles.sort_unstable();
    assert_eq!(cles, vec!["code", "details", "message"], "{body}");
    assert_eq!(error["code"], "LATER_FISCAL_YEAR_CLOSED", "{body}");
    assert_eq!(
        error["details"],
        json!({ "fiscalYearId": fy_id, "fiscalYearName": nom }),
        "{body}"
    );
    let msg = error["message"].as_str().unwrap_or_default();
    assert!(
        msg.contains(nom),
        "le message nomme l'exercice clos : {body}"
    );
    assert!(
        msg.contains("ne peut être enregistrée, modifiée ni supprimée"),
        "le message couvre la saisie : {body}"
    );
    assert!(
        msg.contains("en commençant par le plus récent"),
        "le message porte la marche à suivre : {body}"
    );
}

/// AC 8 · AC 20 — `POST /journal-entries` dans l'exercice du jour, suivi de
/// deux exercices clos : `400 LATER_FISCAL_YEAR_CLOSED` nommant le **plus
/// proche**, corps complet, aucune écriture.
///
/// ⛔ Tue la mutation (v) — retirer le filet de `create_in_tx_inner`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_saisie_manuelle_sous_un_posterieur_clos_rend_400(pool: MySqlPool) {
    let (app, token, company_id, _fy) = setup(&pool).await;
    let d = make_account(&pool, company_id, "1000", AccountType::Asset).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let annee = annee_courante();
    let n1 = exercice_sql(&pool, company_id, annee + 1, "Closed").await;
    let _n2 = exercice_sql(&pool, company_id, annee + 2, "Closed").await;
    let avant = ecritures(&pool, company_id).await;

    let resp = app
        .client
        .post(app.url("/api/v1/journal-entries"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "entryDate": Utc::now().date_naive(),
            "journal": "OD",
            "description": "Saisie sous un bilan clos",
            "lines": [
                { "accountId": d, "debit": "100.00", "credit": "0" },
                { "accountId": c, "debit": "0", "credit": "100.00" },
            ],
        }))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_corps_complet(&body, n1, &format!("Exercice {}", annee + 1));
    assert_eq!(
        ecritures(&pool, company_id).await,
        avant,
        "aucune écriture créée"
    );
}

/// AC 20 — la contre-passation **par la route** (`POST
/// /journal-entries/{id}/reverse`), datée du jour : un exercice postérieur clos,
/// commençant le lendemain de la fin de l'exercice du jour, fait de l'exercice
/// du jour l'exercice fautif (patron de la 15-8b). `400`, aucune écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_contre_passation_par_la_route_sous_un_posterieur_clos_rend_400(pool: MySqlPool) {
    let (app, token, company_id, fy_id) = setup(&pool).await;
    let d = make_account(&pool, company_id, "1000", AccountType::Asset).await;
    let c = make_account(&pool, company_id, "1020", AccountType::Asset).await;
    let origine = journal_entries::create(
        &pool,
        fy_id,
        1,
        NewJournalEntry {
            company_id,
            entry_date: Utc::now().date_naive(),
            journal: Journal::OD,
            description: "Écriture à corriger".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: d,
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: c,
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("origine")
    .entry
    .id;
    let annee = annee_courante();
    let n1 = exercice_sql(&pool, company_id, annee + 1, "Closed").await;
    let avant = ecritures(&pool, company_id).await;

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/journal-entries/{origine}/reverse")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_corps_complet(&body, n1, &format!("Exercice {}", annee + 1));
    assert_eq!(
        ecritures(&pool, company_id).await,
        avant,
        "aucune écriture créée"
    );
}
