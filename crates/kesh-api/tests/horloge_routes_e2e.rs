//! Story 35-5a0 (#929, #1022, #907), AC2 — l'horloge de l'état atteint les
//! sites : la contre-passation est datée du jour de l'horloge, et le verrou de
//! période refuse une borne égale à ce jour.
//!
//! Montage `seed_accounting_company` : un exercice unique **2020-2030 ouvert**
//! (`kesh-db/src/test_fixtures.rs`), qui couvre le `2025-12-31`, le
//! `2026-01-01` et le `2026-06-15` — la contre-passation reste une écriture,
//! jamais un refus d'exercice. L'écriture contre-passée est datée du
//! `2025-06-01`, antérieure à tous les jours éprouvés.
//!
//! `2025-12-31T23:30:00Z` donne le **1er janvier** : le jour de Zurich, depuis
//! la Story 35-5a (la 35-5a0 y lisait encore le jour UTC, le 31 décembre ; ce
//! test est celui qu'elle a retourné, exception nommée à son AC10).
//!
//! AC2 n'exerce que **deux** des neuf sites ; les sept autres sont tenus par la
//! garde de forme (`date_du_jour_garde.rs`).

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_core::horloge::HorlogeFixe;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{NewJournalEntry, NewJournalEntryLine};
use kesh_db::repositories::journal_entries;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const ADMIN_PASSWORD: &str = "admin123";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    token: String,
}

impl TestApp {
    async fn post(&self, path: &str, body: Value) -> (reqwest::StatusCode, Value) {
        let resp = self
            .client
            .post(format!("{}{path}", self.base_url))
            .bearer_auth(&self.token)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = resp.status();
        (status, resp.json().await.unwrap_or(Value::Null))
    }
}

fn instant(s: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(s).unwrap().with_timezone(&Utc)
}

fn jour(a: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(a, m, j).unwrap()
}

/// Un serveur dont l'horloge est arrêtée sur `maintenant`.
async fn spawn_app(pool: &MySqlPool, maintenant: DateTime<Utc>) -> TestApp {
    let config = Config::from_fields_for_test(
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
    );
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
    let mut state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
    state.horloge = Arc::new(HorlogeFixe(maintenant));
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
    let body: Value = client
        .post(format!("{base_url}/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = body["accessToken"].as_str().unwrap().to_string();
    TestApp {
        base_url,
        client,
        token,
    }
}

/// Une écriture OD de 100.– datée du `2025-06-01`, passée par le dépôt.
async fn ecriture(pool: &MySqlPool, s: &SeededCompany) -> i64 {
    journal_entries::create(
        pool,
        s.fiscal_year_id,
        kesh_db::entities::audit_log::Acteur::utilisateur(s.admin_user_id),
        NewJournalEntry {
            company_id: s.company_id,
            entry_date: jour(2025, 6, 1),
            journal: Journal::OD,
            description: "Écriture à contre-passer".into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: s.accounts["4000"],
                    debit: dec!(100.00),
                    credit: dec!(0),
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: s.accounts["1000"],
                    debit: dec!(0),
                    credit: dec!(100.00),
                    project_id: None,
                },
            ],
        },
    )
    .await
    .expect("écriture d'origine")
    .entry
    .id
}

async fn contre_passer(app: &TestApp, id: i64) -> Value {
    let (status, body) = app
        .post(&format!("/api/v1/journal-entries/{id}/reverse"), json!({}))
        .await;
    assert_eq!(status, 201, "corps : {body}");
    body
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_contre_passation_et_le_verrou_suivent_l_horloge_de_l_etat(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool, instant("2026-06-15T10:00:00Z")).await;

    let origine = ecriture(&pool, &s).await;
    let inverse = contre_passer(&app, origine).await;
    assert_eq!(
        inverse["entryDate"].as_str(),
        Some("2026-06-15"),
        "la contre-passation est datée du jour de l'horloge : {inverse}"
    );

    // Une borne égale au jour de l'horloge est refusée…
    let (status, body) = app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": "2026-06-15" }),
        )
        .await;
    assert_eq!(status, 400, "borne au jour de l'horloge : {body}");
    // … par la garde de date (`booksLockBoundNotPast`, rendu `INVALID_INPUT` avec
    // son message propre), non par un autre refus.
    assert_eq!(body["error"]["code"], "INVALID_INPUT", "{body}");
    assert!(
        body["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("antérieure à aujourd'hui")),
        "message de booksLockBoundNotPast attendu : {body}"
    );
    // … la veille est acceptée.
    let (status, body) = app
        .post(
            "/api/v1/companies/current/books-lock",
            json!({ "through": "2026-06-14" }),
        )
        .await;
    assert_eq!(status, 200, "borne à la veille de l'horloge : {body}");
}

/// 23 h 30 UTC le 31 décembre, 00 h 30 le 1er janvier à Zurich : la
/// contre-passation est datée du **1er janvier** (Story 35-5a ; la 35-5a0 la
/// datait du 31, le jour UTC), dans l'exercice 2020-2030 du montage.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_23h30_utc_le_31_decembre_la_contre_passation_est_au_1er_janvier(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(&pool, instant("2025-12-31T23:30:00Z")).await;

    let origine = ecriture(&pool, &s).await;
    let inverse = contre_passer(&app, origine).await;
    assert_eq!(
        inverse["entryDate"].as_str(),
        Some("2026-01-01"),
        "le jour suisse de l'horloge : {inverse}"
    );
    assert_eq!(
        inverse["fiscalYearId"].as_i64(),
        Some(s.fiscal_year_id),
        "dans l'exercice 2020-2030 du montage : {inverse}"
    );
}
