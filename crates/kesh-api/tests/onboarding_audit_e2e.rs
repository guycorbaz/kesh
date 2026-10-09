//! La piste de contrôle de l'installation de production — Story 15-7a2 (#434).
//!
//! Les neuf routes de configuration (`language`, `mode`, `start-production`,
//! `org-type`, `accounting-language`, `coordinates`, `bank-account`,
//! `skip-bank`, `finalize`) écrivent leurs entrées d'audit dans la transaction
//! de leur mutation, l'étape `installation.step_completed` en dernier.
//!
//! ⛔ Les assertions portent sur la **séquence exacte** des actions
//! (`ORDER BY id`) et sur le **contenu** des détails, jamais sur un nombre :
//! un compte resterait vert sur une action remplacée par une autre.

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use common::{audit_count, audit_sequence};
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use serde_json::{Value, json};
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const IBAN: &str = "CH93 0076 2011 6238 5295 7";
const IBAN_COMPACT: &str = "CH9300762011623852957";
const OTHER_IBAN: &str = "CH5604835012345678009";

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
    assert_eq!(resp.status(), 200, "connexion de l'administrateur");
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

/// `POST /api/v1/onboarding/<route>` — rend `(statut, corps)`.
async fn post(app: &TestApp, token: &str, route: &str, body: Option<Value>) -> (u16, Value) {
    let mut req = app
        .client
        .post(app.url(&format!("/api/v1/onboarding/{route}")))
        .header("Authorization", format!("Bearer {token}"));
    if let Some(b) = body {
        req = req.json(&b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    let body = resp.json().await.unwrap_or(Value::Null);
    (status, body)
}

fn coordinates_body(name: &str) -> Value {
    json!({
        "name": name,
        "address": {"street": "Rue du Lac", "building": "12", "postalCode": "1003",
                    "city": "Lausanne", "country": "CH"},
        "ideNumber": "CHE-109.322.551",
    })
}

/// Montage commun : base vide, bootstrap (société provisoire `Independant`,
/// FR/FR, administrateur, `user.created`), état d'onboarding initialisé.
async fn bootstrap(pool: &MySqlPool) -> (TestApp, String, i64) {
    let app = spawn_app(pool.clone()).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    kesh_db::repositories::onboarding::init_state(pool)
        .await
        .unwrap();
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    (app, token, company_id)
}

/// Pose l'étape d'onboarding par SQL (montage).
async fn set_step(pool: &MySqlPool, step: i32) {
    sqlx::query("UPDATE onboarding_state SET step_completed = ?")
        .bind(step)
        .execute(pool)
        .await
        .unwrap();
}

async fn max_audit_id(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Les actions écrites après le repère `after_id`, dans l'ordre.
async fn actions_after(pool: &MySqlPool, after_id: i64) -> Vec<String> {
    sqlx::query_scalar("SELECT action FROM audit_log WHERE id > ? ORDER BY id")
        .bind(after_id)
        .fetch_all(pool)
        .await
        .unwrap()
}

/// `(action, details)` de toutes les entrées, dans l'ordre.
async fn entries(pool: &MySqlPool) -> Vec<(String, Value)> {
    let rows: Vec<(String, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT action, details_json FROM audit_log ORDER BY id")
            .fetch_all(pool)
            .await
            .unwrap();
    rows.into_iter()
        .map(|(a, d)| {
            (
                a,
                d.map(|b| serde_json::from_slice(&b).unwrap())
                    .unwrap_or(Value::Null),
            )
        })
        .collect()
}

fn details_of<'a>(all: &'a [(String, Value)], action: &str) -> Vec<&'a Value> {
    all.iter()
        .filter(|(a, _)| a == action)
        .map(|(_, d)| d)
        .collect()
}

async fn company_version(pool: &MySqlPool, id: i64) -> i32 {
    sqlx::query_scalar("SELECT version FROM companies WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn state_row(pool: &MySqlPool) -> (i32, i32) {
    sqlx::query_as("SELECT step_completed, version FROM onboarding_state")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn accounts_count(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Parcours de production de bout en bout, avec ou sans compte bancaire.
async fn production_path(app: &TestApp, token: &str, with_bank: bool) {
    for (route, body) in [
        ("language", Some(json!({ "language": "DE" }))),
        ("mode", Some(json!({ "mode": "guided" }))),
        ("start-production", None),
        ("org-type", Some(json!({ "orgType": "Association" }))),
        ("accounting-language", Some(json!({ "language": "DE" }))),
        ("coordinates", Some(coordinates_body("Club des Amis"))),
    ] {
        let (status, body) = post(app, token, route, body).await;
        assert_eq!(status, 200, "{route} : {body}");
    }
    let (status, body) = if with_bank {
        post(
            app,
            token,
            "bank-account",
            Some(json!({ "bankName": "UBS", "iban": IBAN, "qrIban": null })),
        )
        .await
    } else {
        post(app, token, "skip-bank", None).await
    };
    assert_eq!(status, 200, "compte bancaire : {body}");
    let (status, body) = post(app, token, "finalize", None).await;
    assert_eq!(status, 200, "finalize : {body}");
    assert_eq!(body["stepCompleted"], 8);
}

/// La séquence attendue du parcours de production (AC 2), après `user.created`.
fn expected_path(with_bank: bool) -> Vec<&'static str> {
    let mut v = vec![
        "user.created",
        "company.updated",
        "installation.step_completed",
        "installation.ui_mode_changed",
        "installation.step_completed",
        "installation.step_completed",
        "company.updated",
        "installation.step_completed",
        "company.updated",
        "account.chart_loaded",
        "installation.step_completed",
        "company.updated",
        "installation.step_completed",
    ];
    if with_bank {
        v.push("bank_account.created");
    }
    v.extend([
        "installation.step_completed",
        "company_invoice_settings.created",
        "vat_rate.created",
        "vat_rate.created",
        "vat_rate.created",
        "vat_rate.created",
        "fiscal_year.created",
        "installation.step_completed",
    ]);
    v
}

// --- Test 1 -----------------------------------------------------------------

/// Test 1 — parcours de production complet sur une base vide (AC 1, 2, 3, 4, 5, 9).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn production_path_writes_the_exact_sequence(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    let version_before_coordinates_probe = company_version(&pool, company_id).await;
    assert_eq!(audit_sequence(&pool).await, ["user.created"]);

    production_path(&app, &token, true).await;

    assert_eq!(audit_sequence(&pool).await, expected_path(true));
    let all = entries(&pool).await;

    // Les étapes : from, to, step.
    let steps = details_of(&all, "installation.step_completed");
    let names = [
        "language",
        "mode",
        "start_production",
        "org_type",
        "accounting_language",
        "coordinates",
        "bank_account",
        "finalize",
    ];
    assert_eq!(steps.len(), names.len());
    for (i, (d, name)) in steps.iter().zip(names).enumerate() {
        assert_eq!(
            **d,
            json!({ "from": i, "to": i + 1, "step": name }),
            "étape {i}"
        );
    }
    let step_meta: Vec<(String, i64)> = sqlx::query_as(
        "SELECT entity_type, entity_id FROM audit_log WHERE action = 'installation.step_completed'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        step_meta
            .iter()
            .all(|(t, id)| t == "installation" && *id == 0)
    );

    // Mode : `installation.ui_mode_changed` de `null` à `guided`.
    assert_eq!(
        *details_of(&all, "installation.ui_mode_changed")[0],
        json!({ "before": null, "after": "guided" })
    );

    // Les quatre `company.updated`, réduits aux champs de chaque route.
    let updated = details_of(&all, "company.updated");
    assert_eq!(
        *updated[0],
        json!({ "before": { "instance_language": "FR" }, "after": { "instance_language": "DE" } })
    );
    assert_eq!(
        *updated[1],
        json!({ "before": { "org_type": "Independant" }, "after": { "org_type": "Association" } })
    );
    assert_eq!(
        *updated[2],
        json!({ "before": { "accounting_language": "FR" }, "after": { "accounting_language": "DE" } })
    );
    let coords = updated[3];
    let keys: BTreeSet<&str> = coords["after"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        keys,
        BTreeSet::from([
            "name",
            "first_name",
            "last_name",
            "address_street",
            "address_building",
            "address_postal_code",
            "address_city",
            "address_country",
            "ide_number",
            "is_stub",
            "version",
        ]),
        "seuls les champs que la route écrit, plus version"
    );
    assert_eq!(coords["before"]["is_stub"], true);
    assert_eq!(coords["after"]["is_stub"], false);
    assert_eq!(coords["after"]["name"], "Club des Amis");
    // Société provisoire : coordonnées changées ⇒ version + 2 (AC 3).
    let before_v = coords["before"]["version"].as_i64().unwrap();
    assert_eq!(coords["after"]["version"].as_i64().unwrap(), before_v + 2);
    // Trois `company.updated` avant (langue, type, langue comptable) : +3.
    assert_eq!(before_v, i64::from(version_before_coordinates_probe) + 3);
    let company_meta: Vec<(String, i64)> = sqlx::query_as(
        "SELECT entity_type, entity_id FROM audit_log WHERE action = 'company.updated'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert!(
        company_meta
            .iter()
            .all(|(t, id)| t == "company" && *id == company_id)
    );

    // Plan comptable : une entrée agrégée, count = COUNT(*) = longueur.
    let chart = details_of(&all, "account.chart_loaded")[0];
    let count = accounts_count(&pool, company_id).await;
    assert!(count > 0);
    assert_eq!(chart["count"].as_i64().unwrap(), count);
    assert_eq!(chart["accounts"].as_array().unwrap().len() as i64, count);
    assert_eq!(chart["company_id"], company_id);
    assert_eq!(chart["org_type"], "Association");
    assert_eq!(chart["language"], "DE");

    // Compte bancaire : présence, sans IBAN en clair — nulle part.
    let bank = details_of(&all, "bank_account.created")[0];
    assert_eq!(bank["iban_present"], true);
    assert_eq!(bank["qr_iban_present"], false);
    let raw: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE CAST(details_json AS CHAR) LIKE ? \
         OR CAST(details_json AS CHAR) LIKE ?",
    )
    .bind(format!("%{IBAN_COMPACT}%"))
    .bind(format!("%{IBAN}%"))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(raw, 0, "aucun IBAN en clair dans details_json");

    // Finalisation : réglages créés, quatre taux dans l'ordre du seed, un exercice.
    // Revue P1 (A-2) : l'objet `details` ENTIER, contre la ligne insérée —
    // une clé renommée ou perdue rougit, et pas seulement les trois premières.
    let settings = details_of(&all, "company_invoice_settings.created")[0];
    assert!(settings["default_receivable_account_id"].is_i64());
    assert!(settings["default_revenue_account_id"].is_i64());
    assert_eq!(*settings, settings_row_json(&pool, company_id).await);
    let rates: Vec<Value> = details_of(&all, "vat_rate.created")
        .into_iter()
        .cloned()
        .collect();
    assert_eq!(rates, vat_rate_rows_json(&pool, company_id).await);
    let categories: Vec<&str> = rates
        .iter()
        .map(|d| d["category"].as_str().unwrap())
        .collect();
    assert_eq!(categories, ["normal", "special", "reduced", "exempt"]);
    assert_eq!(details_of(&all, "fiscal_year.created").len(), 1);
}

/// La forme attendue de `company_invoice_settings.created`, relue de la ligne
/// en base (les neuf clés de l'AC 5).
async fn settings_row_json(pool: &MySqlPool, company_id: i64) -> Value {
    type Row = (
        i64,
        String,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
        Option<i64>,
    );
    let r: Row = sqlx::query_as(
        "SELECT company_id, invoice_number_format, default_receivable_account_id, \
         default_revenue_account_id, default_payable_account_id, default_rounding_account_id, \
         default_discount_account_id, default_bank_fees_account_id, default_bad_debt_account_id \
         FROM company_invoice_settings WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    json!({
        "company_id": r.0,
        "invoice_number_format": r.1,
        "default_receivable_account_id": r.2,
        "default_revenue_account_id": r.3,
        "default_payable_account_id": r.4,
        "default_rounding_account_id": r.5,
        "default_discount_account_id": r.6,
        "default_bank_fees_account_id": r.7,
        "default_bad_debt_account_id": r.8,
    })
}

/// La forme attendue de chaque `vat_rate.created` (forme de `routes/vat.rs`),
/// relue des taux en base dans l'ordre d'insertion.
async fn vat_rate_rows_json(pool: &MySqlPool, company_id: i64) -> Vec<Value> {
    let rows: Vec<(
        i64,
        String,
        String,
        chrono::NaiveDate,
        Option<chrono::NaiveDate>,
    )> = sqlx::query_as(
        "SELECT id, category, CAST(rate AS CHAR), valid_from, valid_to \
             FROM vat_rates WHERE company_id = ? ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|(id, category, rate, from, to)| {
            json!({
                "vat_rate_id": id,
                "category": category,
                "rate": rate,
                "valid_from": from,
                "valid_to": to,
            })
        })
        .collect()
}

// --- Test 2 -----------------------------------------------------------------

/// Test 2 — même parcours avec `skip-bank` : l'étape seule (AC 1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn production_path_with_skip_bank(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;
    production_path(&app, &token, false).await;
    assert_eq!(audit_sequence(&pool).await, expected_path(false));
    let all = entries(&pool).await;
    assert!(all.iter().all(|(a, _)| !a.starts_with("bank_account.")));
    assert!(
        details_of(&all, "installation.step_completed")
            .iter()
            .any(|d| d["step"] == "skip_bank")
    );
}

// --- Test 3 -----------------------------------------------------------------

/// Test 3 — `finalize` rejoué à l'étape 8 : aucune entrée nouvelle (AC 1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn finalize_replayed_at_step_8_writes_nothing(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;
    production_path(&app, &token, false).await;
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(&app, &token, "finalize", None).await;
    assert_eq!(status, 200);
    assert_eq!(body["stepCompleted"], 8);
    assert!(actions_after(&pool, marker).await.is_empty());
}

// --- Test 4 -----------------------------------------------------------------

/// Test 4 — `finalize` sur une société déjà dotée de réglages et de deux taux
/// qui heurtent la clé unique des défauts : on trace ce qui a été inséré (AC 5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn finalize_traces_only_what_it_inserted(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    let chart = kesh_core::chart_of_accounts::load_chart("Independant").unwrap();
    kesh_db::repositories::accounts::bulk_create_from_chart(&pool, company_id, &chart, "fr")
        .await
        .unwrap();
    kesh_db::repositories::company_invoice_settings::insert_with_defaults(&pool, company_id)
        .await
        .unwrap();
    for (label, rate) in [("TVA 8.1", "8.10"), ("TVA 2.6", "2.60")] {
        sqlx::query(
            "INSERT INTO vat_rates (company_id, label, rate, valid_from) VALUES (?, ?, ?, '2024-01-01')",
        )
        .bind(company_id)
        .bind(label)
        .bind(rate)
        .execute(&pool)
        .await
        .unwrap();
    }
    set_step(&pool, 7).await;
    let marker = max_audit_id(&pool).await;

    let (status, body) = post(&app, &token, "finalize", None).await;
    assert_eq!(status, 200, "{body}");

    assert_eq!(
        actions_after(&pool, marker).await,
        [
            "vat_rate.created",
            "vat_rate.created",
            "fiscal_year.created",
            "installation.step_completed",
        ]
    );
    let all = entries(&pool).await;
    let categories: Vec<&str> = details_of(&all, "vat_rate.created")
        .iter()
        .map(|d| d["category"].as_str().unwrap())
        .collect();
    assert_eq!(categories, ["special", "exempt"]);
}

// --- Test 5 -----------------------------------------------------------------

/// Test 5 (a) — `language` avec la langue courante : l'étape seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn noop_language_writes_only_the_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    let v = company_version(&pool, company_id).await;
    let marker = max_audit_id(&pool).await;
    let (status, _) = post(&app, &token, "language", Some(json!({ "language": "FR" }))).await;
    assert_eq!(status, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
    assert_eq!(company_version(&pool, company_id).await, v);
}

/// Test 5 (b) — `mode` avec le mode déjà posé : l'étape seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn noop_mode_writes_only_the_step(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;
    sqlx::query("UPDATE onboarding_state SET step_completed = 1, ui_mode = 'guided'")
        .execute(&pool)
        .await
        .unwrap();
    let marker = max_audit_id(&pool).await;
    let (status, _) = post(&app, &token, "mode", Some(json!({ "mode": "guided" }))).await;
    assert_eq!(status, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
}

/// Test 5 (c) — `org-type` égal : l'étape seule, `version` inchangée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn noop_org_type_writes_only_the_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 3).await;
    let v = company_version(&pool, company_id).await;
    let marker = max_audit_id(&pool).await;
    let (status, _) = post(
        &app,
        &token,
        "org-type",
        Some(json!({ "orgType": "Independant" })),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
    assert_eq!(company_version(&pool, company_id).await, v);
}

/// Test 5 (d) — `accounting-language` égale, plan préchargé : l'étape seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn noop_accounting_language_writes_only_the_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    let chart = kesh_core::chart_of_accounts::load_chart("Independant").unwrap();
    kesh_db::repositories::accounts::bulk_create_from_chart(&pool, company_id, &chart, "fr")
        .await
        .unwrap();
    set_step(&pool, 4).await;
    let v = company_version(&pool, company_id).await;
    let marker = max_audit_id(&pool).await;
    let (status, _) = post(
        &app,
        &token,
        "accounting-language",
        Some(json!({ "language": "FR" })),
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
    assert_eq!(company_version(&pool, company_id).await, v);
}

/// Pose sur la société exactement les coordonnées de [`coordinates_body`].
async fn set_identical_coordinates(pool: &MySqlPool, company_id: i64, is_stub: bool) {
    sqlx::query(
        "UPDATE companies SET name = 'Club des Amis', first_name = NULL, last_name = NULL, \
         address = 'Rue du Lac 12, 1003 Lausanne', address_street = 'Rue du Lac', \
         address_building = '12', address_postal_code = '1003', address_city = 'Lausanne', \
         address_country = 'CH', ide_number = 'CHE109322551', is_stub = ? WHERE id = ?",
    )
    .bind(is_stub)
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap();
}

/// Test 5 (e) — coordonnées identiques, société NON provisoire : l'étape seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn noop_coordinates_on_a_real_company_writes_only_the_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_identical_coordinates(&pool, company_id, false).await;
    set_step(&pool, 5).await;
    let v = company_version(&pool, company_id).await;
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(
        &app,
        &token,
        "coordinates",
        Some(coordinates_body("Club des Amis")),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
    assert_eq!(company_version(&pool, company_id).await, v);
}

/// Test 5 (f) — coordonnées identiques, société PROVISOIRE : une entrée dont
/// `before`/`after` ne diffèrent que par `is_stub` et `version`, `version + 1`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn identical_coordinates_on_a_stub_trace_only_the_stub_flag(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_identical_coordinates(&pool, company_id, true).await;
    set_step(&pool, 5).await;
    let v = company_version(&pool, company_id).await;
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(
        &app,
        &token,
        "coordinates",
        Some(coordinates_body("Club des Amis")),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        actions_after(&pool, marker).await,
        ["company.updated", "installation.step_completed"]
    );
    assert_eq!(company_version(&pool, company_id).await, v + 1);
    let all = entries(&pool).await;
    let d = details_of(&all, "company.updated")[0];
    let (before, after) = (
        d["before"].as_object().unwrap(),
        d["after"].as_object().unwrap(),
    );
    let differing: BTreeSet<&str> = before
        .keys()
        .filter(|k| before[*k] != after[*k])
        .map(String::as_str)
        .collect();
    assert_eq!(differing, BTreeSet::from(["is_stub", "version"]));
    assert_eq!(before["is_stub"], true);
    assert_eq!(after["is_stub"], false);
}

// --- Test 6 -----------------------------------------------------------------

/// Test 6 — `bank-account` : création, modification de l'IBAN, puis données
/// identiques (AC 1, 3, 8, 9).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_account_created_updated_unchanged(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;

    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    let body = json!({ "bankName": "UBS", "iban": IBAN, "qrIban": null });
    assert_eq!(post(&app, &token, "bank-account", Some(body)).await.0, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["bank_account.created", "installation.step_completed"]
    );

    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    let body = json!({ "bankName": "UBS", "iban": OTHER_IBAN, "qrIban": null });
    assert_eq!(post(&app, &token, "bank-account", Some(body)).await.0, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["bank_account.updated", "installation.step_completed"]
    );
    let all = entries(&pool).await;
    let d = details_of(&all, "bank_account.updated")[0];
    assert_eq!(d["trigger"], "onboarding");
    assert_eq!(d["iban_changed"], true);
    assert_eq!(d["qr_iban_changed"], false);
    let before_keys: BTreeSet<&str> = d["before"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        before_keys,
        BTreeSet::from([
            "bank_name",
            "iban_present",
            "qr_iban_present",
            "is_primary",
            "journal_account_id",
            "version",
        ])
    );
    let raw = d.to_string();
    assert!(!raw.contains(IBAN_COMPACT) && !raw.contains(OTHER_IBAN));

    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    let body = json!({ "bankName": "UBS", "iban": OTHER_IBAN, "qrIban": null });
    assert_eq!(post(&app, &token, "bank-account", Some(body)).await.0, 200);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["installation.step_completed"]
    );
}

// --- Test 7 -----------------------------------------------------------------

/// Test 7 — `language` sans société en base : `company.created`, dont le
/// `company_id` d'audit est celui de `users.company_id` — l'id MORT du montage
/// (#528, fermé par la 15-7b2) (AC 1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn language_without_company_traces_company_created(pool: MySqlPool) {
    let (app, token, dead_company_id) = bootstrap(&pool).await;
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("DELETE FROM companies")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    let marker = max_audit_id(&pool).await;

    let (status, body) = post(&app, &token, "language", Some(json!({ "language": "IT" }))).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        actions_after(&pool, marker).await,
        ["company.created", "installation.step_completed"]
    );
    let new_id: i64 = sqlx::query_scalar("SELECT id FROM companies")
        .fetch_one(&pool)
        .await
        .unwrap();
    let (entity_type, entity_id, audit_company_id, raw): (String, i64, Option<i64>, Vec<u8>) =
        sqlx::query_as(
            "SELECT entity_type, entity_id, company_id, details_json FROM audit_log \
             WHERE action = 'company.created'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(entity_type, "company");
    assert_eq!(entity_id, new_id);
    assert_ne!(new_id, dead_company_id);
    assert_eq!(
        audit_company_id,
        Some(dead_company_id),
        "#528 : company_id d'audit = users.company_id, l'id mort"
    );
    let details: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        details,
        json!({ "instance_language": "IT", "is_stub": true })
    );
}

// --- Test 8 -----------------------------------------------------------------

/// Test 8 — étapes franchies par jeton d'API : chaque entrée porte
/// `actor_type = 'api_key'` et `actor_api_key_id` (AC 6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn api_key_steps_are_attributed_to_the_key(pool: MySqlPool) {
    let (app, jwt, _) = bootstrap(&pool).await;
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({ "name": "onboarding", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let key_body: Value = resp.json().await.unwrap();
    let key_id = key_body["id"].as_i64().unwrap();
    let key = key_body["key"].as_str().unwrap().to_string();

    let check = |rows: Vec<(String, String, Option<i64>)>, expected: &[&str]| {
        let actions: Vec<&str> = rows.iter().map(|(a, _, _)| a.as_str()).collect();
        assert_eq!(actions, expected);
        for (action, actor_type, actor_key) in &rows {
            assert_eq!(actor_type, "api_key", "{action}");
            assert_eq!(*actor_key, Some(key_id), "{action}");
        }
    };
    let rows_after = |marker: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query_as::<_, (String, String, Option<i64>)>(
                "SELECT action, actor_type, actor_api_key_id FROM audit_log WHERE id > ? ORDER BY id",
            )
            .bind(marker)
            .fetch_all(&pool)
            .await
            .unwrap()
        }
    };

    set_step(&pool, 3).await;
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(
        &app,
        &key,
        "org-type",
        Some(json!({ "orgType": "Association" })),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    check(
        rows_after(marker).await,
        &["company.updated", "installation.step_completed"],
    );

    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(&app, &key, "skip-bank", None).await;
    assert_eq!(status, 200, "{body}");
    check(rows_after(marker).await, &["installation.step_completed"]);
}

// --- Test 9 -----------------------------------------------------------------

const FAIL_TRIGGER: &str = "CREATE TRIGGER t_15_7a_fail BEFORE INSERT ON audit_log FOR EACH ROW \
     SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '15-7a atomicity'";

async fn trigger_present(pool: &MySqlPool) -> bool {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.TRIGGERS \
         WHERE TRIGGER_SCHEMA = DATABASE() AND TRIGGER_NAME = 't_15_7a_fail'",
    )
    .fetch_one(pool)
    .await
    .unwrap();
    n == 1
}

async fn company_snapshot(pool: &MySqlPool, id: i64) -> (String, String, i32, bool) {
    sqlx::query_as("SELECT name, accounting_language, version, is_stub FROM companies WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Test 9 (a) — atomicité de `accounting-language` : l'échec de l'écriture
/// d'audit annule la société, le plan et l'étape (AC 8). Origine prouvée par
/// différence : seul le déclencheur sépare l'échec du succès.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn accounting_language_is_atomic_with_its_trace(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 4).await;
    let company = company_snapshot(&pool, company_id).await;
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    sqlx::raw_sql(FAIL_TRIGGER).execute(&pool).await.unwrap();
    assert!(trigger_present(&pool).await);

    let body = json!({ "language": "DE" });
    let (status, _) = post(&app, &token, "accounting-language", Some(body.clone())).await;
    assert_eq!(status, 500);
    assert_eq!(accounts_count(&pool, company_id).await, 0);
    assert_eq!(company_snapshot(&pool, company_id).await, company);
    assert_eq!(state_row(&pool).await, state);
    assert_eq!(audit_count(&pool).await, audit);

    sqlx::raw_sql("DROP TRIGGER t_15_7a_fail")
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = post(&app, &token, "accounting-language", Some(body)).await;
    assert_eq!(status, 200, "le même montage, sans déclencheur, réussit");
    assert!(accounts_count(&pool, company_id).await > 0);
}

/// Test 9 (b) — atomicité de `coordinates` (AC 8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn coordinates_is_atomic_with_its_trace(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 5).await;
    let company = company_snapshot(&pool, company_id).await;
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    sqlx::raw_sql(FAIL_TRIGGER).execute(&pool).await.unwrap();
    assert!(trigger_present(&pool).await);

    let (status, _) = post(
        &app,
        &token,
        "coordinates",
        Some(coordinates_body("Nouveau Nom SA")),
    )
    .await;
    assert_eq!(status, 500);
    assert_eq!(company_snapshot(&pool, company_id).await, company);
    assert_eq!(state_row(&pool).await, state);
    assert_eq!(audit_count(&pool).await, audit);

    sqlx::raw_sql("DROP TRIGGER t_15_7a_fail")
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = post(
        &app,
        &token,
        "coordinates",
        Some(coordinates_body("Nouveau Nom SA")),
    )
    .await;
    assert_eq!(status, 200, "le même montage, sans déclencheur, réussit");
    assert_eq!(
        company_snapshot(&pool, company_id).await.0,
        "Nouveau Nom SA"
    );
}

/// Test 9 (c) — revue P1 (A-1) : `accounting-language` à langue ÉGALE, plan non
/// chargé. Aucune `company.updated` : la première écriture d'audit est
/// `account.chart_loaded`, APRÈS `bulk_create_from_chart_in_tx`. Son échec doit
/// emporter le plan et l'étape — ce que 9 (a), qui échoue sur `company.updated`
/// avant tout compte, ne pouvait pas montrer.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn chart_loading_is_atomic_with_its_trace(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 4).await;
    let company = company_snapshot(&pool, company_id).await;
    assert_eq!(company.1, "FR", "montage : langue comptable FR");
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    assert_eq!(accounts_count(&pool, company_id).await, 0);
    sqlx::raw_sql(FAIL_TRIGGER).execute(&pool).await.unwrap();
    assert!(trigger_present(&pool).await);

    let body = json!({ "language": "FR" });
    let (status, _) = post(&app, &token, "accounting-language", Some(body.clone())).await;
    assert_eq!(status, 500);
    assert_eq!(accounts_count(&pool, company_id).await, 0, "plan annulé");
    assert_eq!(company_snapshot(&pool, company_id).await, company);
    assert_eq!(state_row(&pool).await, state, "étape annulée");
    assert_eq!(audit_count(&pool).await, audit);

    sqlx::raw_sql("DROP TRIGGER t_15_7a_fail")
        .execute(&pool)
        .await
        .unwrap();
    let marker = max_audit_id(&pool).await;
    let (status, _) = post(&app, &token, "accounting-language", Some(body)).await;
    assert_eq!(status, 200, "le même montage, sans déclencheur, réussit");
    assert!(accounts_count(&pool, company_id).await > 0);
    assert_eq!(
        actions_after(&pool, marker).await,
        ["account.chart_loaded", "installation.step_completed"],
        "la première écriture d'audit de ce montage est bien le plan"
    );
}

/// Test 9 (d) — revue P1 (A-1) : `skip-bank`, sans entrée de domaine. La
/// première écriture d'audit est l'entrée d'étape elle-même, APRÈS
/// `update_step_in_tx` : son échec doit laisser `onboarding_state` intact.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn step_is_atomic_with_its_own_entry(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 6).await;
    let company = company_snapshot(&pool, company_id).await;
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    sqlx::raw_sql(FAIL_TRIGGER).execute(&pool).await.unwrap();
    assert!(trigger_present(&pool).await);

    let (status, _) = post(&app, &token, "skip-bank", None).await;
    assert_eq!(status, 500);
    assert_eq!(state_row(&pool).await, state, "étape et version annulées");
    assert_eq!(company_snapshot(&pool, company_id).await, company);
    assert_eq!(audit_count(&pool).await, audit);

    sqlx::raw_sql("DROP TRIGGER t_15_7a_fail")
        .execute(&pool)
        .await
        .unwrap();
    let (status, _) = post(&app, &token, "skip-bank", None).await;
    assert_eq!(status, 200, "le même montage, sans déclencheur, réussit");
    assert_eq!(state_row(&pool).await, (7, state.1 + 1));
}

// --- Test 10 ----------------------------------------------------------------

/// Test 10 — étape refusée sous verrou : `org-type` à l'étape 4 ⇒ 400, aucune
/// mutation ni entrée (AC 1, 8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn out_of_step_request_is_refused_under_lock(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 4).await;
    let company = company_snapshot(&pool, company_id).await;
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    let (status, body) = post(
        &app,
        &token,
        "org-type",
        Some(json!({ "orgType": "Association" })),
    )
    .await;
    assert_eq!(status, 400);
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");
    assert_eq!(company_snapshot(&pool, company_id).await, company);
    assert_eq!(state_row(&pool).await, state);
    assert_eq!(audit_count(&pool).await, audit);
}

// --- Test 11 ----------------------------------------------------------------

/// Test 11 — deux `accounting-language` concurrentes à l'étape 4 : un 200 et un
/// 400, un seul plan chargé, une seule étape (AC 4, 8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn concurrent_accounting_language_loads_the_chart_once(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    set_step(&pool, 4).await;
    let (_, v) = state_row(&pool).await;
    let app = Arc::new(app);

    let spawn = |app: Arc<TestApp>, token: String| {
        tokio::spawn(async move {
            post(
                &app,
                &token,
                "accounting-language",
                Some(json!({ "language": "DE" })),
            )
            .await
        })
    };
    let a = spawn(app.clone(), token.clone());
    let b = spawn(app.clone(), token.clone());
    let (ra, rb) = (a.await.unwrap(), b.await.unwrap());

    let mut statuses = [ra.0, rb.0];
    statuses.sort_unstable();
    assert_eq!(statuses, [200, 400], "{ra:?} / {rb:?}");
    let refused = if ra.0 == 400 { &ra.1 } else { &rb.1 };
    assert_eq!(
        refused["error"]["code"],
        "ONBOARDING_STEP_ALREADY_COMPLETED"
    );

    let all = entries(&pool).await;
    assert_eq!(details_of(&all, "account.chart_loaded").len(), 1);
    assert_eq!(
        details_of(&all, "installation.step_completed")
            .iter()
            .filter(|d| d["step"] == "accounting_language")
            .count(),
        1
    );
    assert_eq!(state_row(&pool).await, (5, v + 1));
    let chart = kesh_core::chart_of_accounts::load_chart("Independant").unwrap();
    assert_eq!(
        accounts_count(&pool, company_id).await,
        chart.len() as i64,
        "le plan, une seule fois"
    );
}

// --- Test 12 ----------------------------------------------------------------

/// Test 12 — garde de source : aucun `NewAuditLogEntry::user(` dans l'onboarding
/// (AC 6) — un jeton d'API atteint ces routes.
#[test]
fn onboarding_sources_never_use_the_user_constructor() {
    for (name, src) in [
        (
            "routes/onboarding.rs",
            include_str!("../src/routes/onboarding.rs"),
        ),
        (
            "repositories/onboarding.rs",
            include_str!("../../kesh-db/src/repositories/onboarding.rs"),
        ),
    ] {
        assert!(
            !src.contains(concat!("NewAuditLogEntry::", "user(")),
            "{name} ne doit pas construire d'entrée par `::user` (dette #431)"
        );
    }
}

// --- Test 13 ----------------------------------------------------------------

/// Test 13 — pool d'UNE connexion : aucune route ne prélève le pool pendant que
/// sa transaction tient la connexion — sinon elle attend et expire (AC 8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn routes_never_draw_from_the_pool_inside_their_transaction(pool: MySqlPool) {
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    kesh_db::repositories::onboarding::init_state(&pool)
        .await
        .unwrap();
    let single = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(2))
        .connect_with(pool.connect_options().as_ref().clone())
        .await
        .unwrap();
    let app = spawn_app(single).await;
    let token = login(&app).await;

    // Revue P1 (A-5, B-2, E-2) : les NEUF routes d'étape, dans l'ordre du
    // parcours, puis `skip-bank` sur un état ramené à l'étape 6.
    for (route, body) in [
        ("language", Some(json!({ "language": "FR" }))),
        ("mode", Some(json!({ "mode": "guided" }))),
        ("start-production", None),
        ("org-type", Some(json!({ "orgType": "Pme" }))),
        ("accounting-language", Some(json!({ "language": "FR" }))),
        (
            "coordinates",
            Some(coordinates_body("Une Seule Connexion SA")),
        ),
        (
            "bank-account",
            Some(json!({ "bankName": "UBS", "iban": IBAN, "qrIban": null })),
        ),
        ("finalize", None),
    ] {
        let (status, body) = post(&app, &token, route, body).await;
        assert_eq!(status, 200, "{route} sur un pool d'une connexion : {body}");
    }
    assert_eq!(state_row(&pool).await.0, 8);
    set_step(&pool, 6).await;
    let (status, body) = post(&app, &token, "skip-bank", None).await;
    assert_eq!(
        status, 200,
        "skip-bank sur un pool d'une connexion : {body}"
    );
}

// --- Test 14 ----------------------------------------------------------------

/// Les six routes que `lock_state_at_step` garde contre l'installation de
/// démonstration (`require_not_demo = true`), à leur étape attendue.
fn demo_guarded_routes() -> Vec<(&'static str, i32, Option<Value>)> {
    vec![
        ("start-production", 2, None),
        ("org-type", 3, Some(json!({ "orgType": "Association" }))),
        ("accounting-language", 4, Some(json!({ "language": "DE" }))),
        ("coordinates", 5, Some(coordinates_body("Démo Modifiée SA"))),
        (
            "bank-account",
            6,
            Some(json!({ "bankName": "UBS", "iban": IBAN, "qrIban": null })),
        ),
        ("skip-bank", 6, None),
    ]
}

async fn set_demo(pool: &MySqlPool, is_demo: bool) {
    sqlx::query("UPDATE onboarding_state SET is_demo = ?")
        .bind(is_demo)
        .execute(pool)
        .await
        .unwrap();
}

async fn bank_accounts_count(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Test 14 — revue P1 (E-1) : une installation de DÉMONSTRATION, à l'étape
/// exacte de la route, est refusée par les six routes gardées — 400, aucune
/// mutation (société, état, comptes, comptes bancaires), aucune trace. Origine
/// prouvée par différence : la même requête, `is_demo` levé, réussit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn demo_installation_is_refused_by_every_production_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    for (route, step, body) in demo_guarded_routes() {
        set_step(&pool, step).await;
        set_demo(&pool, true).await;
        let company = company_snapshot(&pool, company_id).await;
        let state = state_row(&pool).await;
        let audit = audit_count(&pool).await;
        let accounts = accounts_count(&pool, company_id).await;
        let banks = bank_accounts_count(&pool).await;

        let (status, resp) = post(&app, &token, route, body.clone()).await;
        assert_eq!(status, 400, "{route} sur une démonstration : {resp}");
        assert_eq!(
            resp["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED",
            "{route}"
        );
        assert_eq!(
            company_snapshot(&pool, company_id).await,
            company,
            "{route}"
        );
        assert_eq!(state_row(&pool).await, state, "{route}");
        assert_eq!(audit_count(&pool).await, audit, "{route}");
        assert_eq!(accounts_count(&pool, company_id).await, accounts, "{route}");
        assert_eq!(bank_accounts_count(&pool).await, banks, "{route}");

        set_demo(&pool, false).await;
        let (status, resp) = post(&app, &token, route, body).await;
        assert_eq!(status, 200, "{route} hors démonstration : {resp}");
    }
}
