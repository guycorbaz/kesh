//! La piste de contrôle de l'installation — Story 15-7a2 (#434) pour la
//! production, Story 15-7b1 (#434) pour le chargement de la démonstration,
//! Story 15-7b2 (#434, #279, #528) pour la remise à zéro.
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
use common::{audit_count, audit_sequence, create_key_via_http};
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

// Le test 7 de la 15-7a2 (`language` sans société, `company_id` d'audit = l'id
// MORT) est remplacé par le test 6c de la 15-7b2,
// `language_without_company_reattaches_and_revokes` (#528 : la branche
// « aucune société » rattache désormais les principaux orphelins).

// --- Test 8 -----------------------------------------------------------------

/// Test 8 — étapes franchies par jeton d'API : chaque entrée porte
/// `actor_type = 'api_key'` et `actor_api_key_id` (AC 6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn api_key_steps_are_attributed_to_the_key(pool: MySqlPool) {
    let (app, jwt, _) = bootstrap(&pool).await;
    let (key_id, key) =
        create_key_via_http(&app.client, &app.base_url, &jwt, "onboarding", "read-write").await;

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

// ===========================================================================
// Story 15-7b1 (#434) — le chargement de la démonstration laisse sa trace
// ===========================================================================
//
// Montage commun (R-2 de la validation P1) : `bootstrap` sur base vide, SANS
// `create_test_company` — le premier démarrage pose la société PROVISOIRE
// (`is_stub = TRUE`). Chaque test asserte `is_stub = TRUE` avant `seed-demo`,
// faute de quoi ses assertions sur `is_stub` seraient creuses.

async fn is_stub(pool: &MySqlPool, company_id: i64) -> bool {
    sqlx::query_scalar("SELECT is_stub FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn count_for_company(pool: &MySqlPool, table: &str, company_id: i64) -> i64 {
    sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM {table} WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// `language` puis `mode` : l'installation arrive à l'étape 2, celle de
/// `seed-demo`.
async fn to_demo_step(app: &TestApp, token: &str) {
    for (route, body) in [
        ("language", json!({ "language": "FR" })),
        ("mode", json!({ "mode": "guided" })),
    ] {
        let (status, resp) = post(app, token, route, Some(body)).await;
        assert_eq!(status, 200, "{route} : {resp}");
    }
}

/// Les détails attendus de `installation.demo_seeded`, **relus de la base** :
/// une valeur codée en dur dans le test ne prouverait rien sur le code.
async fn expected_demo_details(pool: &MySqlPool, company_id: i64, vat_rates_created: i64) -> Value {
    let org_type: String = sqlx::query_scalar("SELECT org_type FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap();
    let fiscal_year_id: i64 =
        sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .unwrap();
    json!({
        "company_id": company_id,
        "org_type": org_type,
        "accounts_created": accounts_count(pool, company_id).await,
        "fiscal_year_id": fiscal_year_id,
        "vat_rates_created": vat_rates_created,
        "invoice_settings_created": true,
    })
}

// --- Test 1 (15-7b1) --------------------------------------------------------

/// Test 1 (15-7b1, AC 1) — `language` → `mode` → `seed-demo` : la séquence
/// exacte se termine par `installation.demo_seeded` puis l'étape 2 → 3, les
/// détails disent ce que la base contient, et la société n'est plus provisoire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_writes_its_synthesis_then_the_step(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    to_demo_step(&app, &token).await;
    let before = audit_sequence(&pool).await;
    assert_eq!(before.first().map(String::as_str), Some("user.created"));

    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 200, "{body}");

    let mut expected = before.clone();
    expected.extend([
        "installation.demo_seeded".to_string(),
        "installation.step_completed".to_string(),
    ]);
    assert_eq!(audit_sequence(&pool).await, expected);

    let all = entries(&pool).await;
    assert_eq!(
        details_of(&all, "installation.demo_seeded"),
        [&expected_demo_details(&pool, company_id, 4).await]
    );
    assert_eq!(
        all.last().unwrap().1,
        json!({ "from": 2, "to": 3, "step": "seed_demo" })
    );
    let (entity_type, entity_id): (String, i64) = sqlx::query_as(
        "SELECT entity_type, entity_id FROM audit_log WHERE action = 'installation.demo_seeded'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (entity_type.as_str(), entity_id),
        (
            "installation",
            kesh_db::entities::audit_log::AUDIT_ENTITY_ID_NONE
        )
    );
    assert_eq!(state_row(&pool).await.0, 3);
    assert!(!is_stub(&pool, company_id).await, "société plus provisoire");
}

/// Test 1, variante (15-7b1, AC 1) — deux des quatre taux existent déjà
/// (8.10 et 2.60 au 2024-01-01) : `vat_rates_created` dit les DEUX taux
/// réellement insérés, pas les quatre du seed.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_counts_only_the_rates_it_inserted(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    to_demo_step(&app, &token).await;
    sqlx::query(
        "INSERT INTO vat_rates (company_id, category, label, rate, valid_from) \
         VALUES (?, 'normal', 'pré-existant', 8.10, '2024-01-01'), \
                (?, 'reduced', 'pré-existant', 2.60, '2024-01-01')",
    )
    .bind(company_id)
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 200, "{body}");
    let all = entries(&pool).await;
    assert_eq!(
        details_of(&all, "installation.demo_seeded"),
        [&expected_demo_details(&pool, company_id, 2).await]
    );
    assert_eq!(count_for_company(&pool, "vat_rates", company_id).await, 4);
}

// --- Test 2 (15-7b1) --------------------------------------------------------

/// Déclencheur de test SÉLECTIF (F-1 de la validation P1) : il ne fait échouer
/// que l'écriture de l'action donnée, si bien que les autres écritures de la
/// transaction ont lieu — et que la mutation « entrée écrite par le pool, hors
/// de la transaction » survit au 500, où un déclencheur global la masquerait.
fn selective_trigger(action: &str) -> String {
    format!(
        "CREATE TRIGGER t_15_7b1_fail BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = '{action}' THEN \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '15-7b1 atomicity'; END IF; END"
    )
}

/// Test 2 (15-7b1, AC 1) — atomicité de la DERNIÈRE transaction de
/// `seed_demo` : l'échec d'une écriture d'audit annule la levée du drapeau
/// provisoire, les réglages, les taux, l'étape et l'autre entrée. Les quatre
/// premières validations (société renommée, plan, exercice) sont commitées
/// AVANT elle : ce résidu est asserté, et le rejeu échoue — atomicité des
/// quatre premières validations hors périmètre, suivie par #538.
async fn seed_demo_last_transaction_is_atomic(pool: MySqlPool, failing_action: &str) {
    let (app, token, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    // Posé APRÈS la montée à l'étape 2 (R-4) : les étapes `language` et `mode`
    // écrivent elles aussi `installation.step_completed`.
    to_demo_step(&app, &token).await;
    let name_before = company_snapshot(&pool, company_id).await.0;
    let state = state_row(&pool).await;
    let audit = audit_count(&pool).await;
    sqlx::raw_sql(&selective_trigger(failing_action))
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 500, "{body}");

    // La dernière transaction est annulée en entier.
    assert_eq!(audit_count(&pool).await, audit, "aucune entrée neuve");
    let demo_seeded: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'installation.demo_seeded'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(demo_seeded, 0);
    assert_eq!(state_row(&pool).await, state, "étape toujours 2");
    assert!(
        is_stub(&pool, company_id).await,
        "drapeau provisoire intact"
    );
    assert_eq!(count_for_company(&pool, "vat_rates", company_id).await, 0);
    assert_eq!(
        count_for_company(&pool, "company_invoice_settings", company_id).await,
        0
    );

    // Résidu des quatre premières validations, commitées avant elle (#538).
    assert!(accounts_count(&pool, company_id).await > 0, "plan commité");
    assert_eq!(
        count_for_company(&pool, "fiscal_years", company_id).await,
        1
    );
    assert_ne!(
        company_snapshot(&pool, company_id).await.0,
        name_before,
        "société renommée"
    );

    // Rejeu impossible : `bulk_create_from_chart` fait des INSERT secs et
    // `create_for_seed` refuse le chevauchement — 500 jusqu'à une remise à
    // zéro (#538).
    sqlx::raw_sql("DROP TRIGGER t_15_7b1_fail")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 500, "rejeu après échec : {body}");
}

/// Test 2 — le déclencheur fait échouer la SECONDE écriture (l'étape).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_is_atomic_with_its_step_entry(pool: MySqlPool) {
    seed_demo_last_transaction_is_atomic(pool, "installation.step_completed").await;
}

/// Test 2, variante — le déclencheur fait échouer la PREMIÈRE écriture
/// (`installation.demo_seeded`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_is_atomic_with_its_synthesis_entry(pool: MySqlPool) {
    seed_demo_last_transaction_is_atomic(pool, "installation.demo_seeded").await;
}

// --- Test 3 (15-7b1) --------------------------------------------------------

/// Test 3 (15-7b1, AC 7) — `seed-demo` par jeton d'API `read-write` (créé sous
/// le JWT de l'administrateur) : les deux dernières entrées sont attribuées au
/// jeton. Journal lu par le pool — il n'est pas lisible par jeton.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_by_api_key_is_attributed_to_the_key(pool: MySqlPool) {
    let (app, jwt, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    let (key_id, key) =
        create_key_via_http(&app.client, &app.base_url, &jwt, "demo", "read-write").await;
    to_demo_step(&app, &jwt).await;
    let before = audit_sequence(&pool).await;
    assert_eq!(before[..2], ["user.created", "api_key.created"]);
    let marker = max_audit_id(&pool).await;

    let (status, body) = post(&app, &key, "seed-demo", None).await;
    assert_eq!(status, 200, "{body}");

    let mut expected = before.clone();
    expected.extend([
        "installation.demo_seeded".to_string(),
        "installation.step_completed".to_string(),
    ]);
    assert_eq!(audit_sequence(&pool).await, expected);
    let rows: Vec<(String, String, Option<i64>)> = sqlx::query_as(
        "SELECT action, actor_type, actor_api_key_id FROM audit_log WHERE id > ? ORDER BY id",
    )
    .bind(marker)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 2);
    for (action, actor_type, actor_key) in &rows {
        assert_eq!(actor_type, "api_key", "{action}");
        assert_eq!(*actor_key, Some(key_id), "{action}");
    }
}

// --- Test 11 (15-7b1) -------------------------------------------------------

/// Test 11 (15-7b1, AC 1 et 7) — gardes de source. (a) Aucun
/// `NewAuditLogEntry::user(` dans `kesh-seed` : garde de régression (vert dès
/// avant la story ; la preuve de l'AC 7 est le test 3). (b) Plus d'`UPDATE` du
/// drapeau provisoire dans le handler : `clear_stub_in_tx` le lève dans la
/// dernière transaction — aucun test de comportement ne distinguerait un
/// `UPDATE` résiduel redondant (R-1/F-2 de la validation P1).
///
/// Revue P1 (B-4 = A-4) : les deux sources sont **normalisées** avant la
/// recherche — blancs (sauts de ligne compris) réduits à une espace, casse
/// abaissée —, si bien qu'un `UPDATE` remis en forme sur plusieurs lignes ou
/// en minuscules est vu. Angle mort assumé, écrit à la fiche : une requête
/// assemblée à partir de constantes ou de fragments de chaîne.
#[test]
fn seed_demo_sources_keep_the_actor_and_the_stub_in_the_transaction() {
    fn normalized(src: &str) -> String {
        src.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    }
    let seed = normalized(include_str!("../../kesh-seed/src/lib.rs"));
    assert!(
        !seed.contains(concat!("newauditlogentry::", "user(")),
        "kesh-seed ne doit pas construire d'entrée par `::user` (dette #431)"
    );
    let handler = normalized(include_str!("../src/routes/onboarding.rs"));
    assert!(
        !handler.contains(concat!("update companies ", "set is_stub")),
        "le drapeau provisoire se lève dans la transaction de `seed_demo`"
    );
}

// --- Revue P1 de la 15-7b1 (E-1 = A-1, B-3) ---------------------------------

async fn admin_user_id(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT id FROM users ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn is_demo(pool: &MySqlPool) -> bool {
    sqlx::query_scalar("SELECT is_demo FROM onboarding_state")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Ce que la dernière transaction de `seed_demo` n'a PAS écrit : ni synthèse,
/// ni entrée neuve, ni étape (ni version), ni levée du drapeau provisoire, ni
/// réglages, ni taux, ni `is_demo`.
async fn assert_last_transaction_wrote_nothing(
    pool: &MySqlPool,
    company_id: i64,
    audit_before: &[String],
    state_before: (i32, i32),
) {
    assert_eq!(
        audit_sequence(pool).await,
        audit_before,
        "aucune entrée neuve"
    );
    assert_eq!(
        state_row(pool).await,
        state_before,
        "étape et version inchangées"
    );
    assert!(!is_demo(pool).await, "is_demo non levé");
    assert!(is_stub(pool, company_id).await, "drapeau provisoire intact");
    assert_eq!(count_for_company(pool, "vat_rates", company_id).await, 0);
    assert_eq!(
        count_for_company(pool, "company_invoice_settings", company_id).await,
        0
    );
}

/// Revue P1 (E-1 = A-1) — la revérification de l'étape SOUS VERROU, prouvée
/// sans le handler (dont la pré-vérification non verrouillée la masquerait) :
/// `kesh_seed::seed_demo` appelé directement sur une installation déjà passée
/// à l'étape `step` rend `StepAlreadyCompleted`, et sa dernière transaction
/// n'écrit rien. Les quatre premières validations, elles, ont commité (#538).
async fn seed_demo_refuses_a_passed_step_under_lock(pool: MySqlPool, step: i32) {
    let (app, token, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    to_demo_step(&app, &token).await;
    set_step(&pool, step).await;
    let audit = audit_sequence(&pool).await;
    let state = state_row(&pool).await;
    assert_eq!(state.0, step, "montage : étape posée");

    let result = kesh_seed::seed_demo(
        &pool,
        &kesh_i18n::Locale::FrCh,
        (admin_user_id(&pool).await, None),
    )
    .await;
    assert!(
        matches!(result, Err(kesh_seed::SeedError::StepAlreadyCompleted)),
        "étape {step} : {result:?}"
    );
    assert_last_transaction_wrote_nothing(&pool, company_id, &audit, state).await;
    // Résidu des quatre premières validations (#538) : la garde est bien
    // celle de la DERNIÈRE transaction, non une garde amont.
    assert!(accounts_count(&pool, company_id).await > 0, "plan commité");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_refuses_step_3_under_lock(pool: MySqlPool) {
    seed_demo_refuses_a_passed_step_under_lock(pool, 3).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_refuses_step_4_under_lock(pool: MySqlPool) {
    seed_demo_refuses_a_passed_step_under_lock(pool, 4).await;
}

/// Revue P1 (E-1 = A-1) — le 400 du handler sur la course `start-production`
/// contre `seed-demo`, rendue déterministe : un déclencheur franchit l'étape
/// au moment où `create_for_seed` insère l'exercice — après la
/// pré-vérification non verrouillée, avant la dernière transaction. La route
/// rend `400 ONBOARDING_STEP_ALREADY_COMPLETED` (et non 500), et la dernière
/// transaction n'écrit rien. Le résidu (société renommée, plan, exercice) est
/// celui de #538.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_race_with_start_production_is_a_400(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    assert!(
        is_stub(&pool, company_id).await,
        "montage : société provisoire"
    );
    to_demo_step(&app, &token).await;
    let audit = audit_sequence(&pool).await;
    sqlx::raw_sql(
        "CREATE TRIGGER t_15_7b1_race AFTER INSERT ON fiscal_years FOR EACH ROW \
         UPDATE onboarding_state SET step_completed = 3",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");

    let state = state_row(&pool).await;
    assert_eq!(state.0, 3, "l'étape franchie par le déclencheur");
    assert_last_transaction_wrote_nothing(&pool, company_id, &audit, state).await;
    assert_eq!(
        count_for_company(&pool, "fiscal_years", company_id).await,
        1,
        "le déclencheur a bien joué"
    );
}

/// Revue P1 (B-3) — le prédicat de rejeu, sur une VRAIE erreur 1213 rendue
/// par MariaDB (`SIGNAL … MYSQL_ERRNO = 1213`) et passée par `map_db_error`,
/// comme `final_attempt` la voit : reconnue ; une autre erreur de base (1205),
/// une erreur de dépôt et le refus d'étape ne le sont pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn is_seed_retryable_accepts_1213_and_only_it(pool: MySqlPool) {
    use kesh_seed::{SeedAttemptError, is_seed_retryable};
    async fn signal(pool: &MySqlPool, errno: u32) -> SeedAttemptError {
        let e = sqlx::query(&format!(
            "SIGNAL SQLSTATE '40001' SET MYSQL_ERRNO = {errno}, MESSAGE_TEXT = '15-7b1'"
        ))
        .execute(pool)
        .await
        .unwrap_err();
        SeedAttemptError::Db(kesh_db::errors::map_db_error(e))
    }
    assert!(is_seed_retryable(&signal(&pool, 1213).await));
    assert!(!is_seed_retryable(&signal(&pool, 1205).await));
    assert!(!is_seed_retryable(&SeedAttemptError::Db(
        kesh_db::errors::DbError::OptimisticLockConflict
    )));
    assert!(!is_seed_retryable(&SeedAttemptError::StepAlreadyCompleted));
}

/// Revue P1 (B-3) — le rejeu de la dernière transaction, de bout en bout : un
/// déclencheur lève une 1213 à la PREMIÈRE écriture de la synthèse, et à elle
/// seule (compteur dans une table MyISAM, que l'annulation n'efface pas).
/// Le second essai aboutit : 200, une seule synthèse, l'étape franchie une
/// fois. Sans rejeu (prédicat faux ou `retry_with` retiré), la route rendrait
/// 500.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_last_transaction_is_replayed_on_deadlock(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    to_demo_step(&app, &token).await;
    let before = audit_sequence(&pool).await;
    sqlx::raw_sql(
        "CREATE TABLE t_15_7b1_once (n INT NOT NULL) ENGINE = MyISAM; \
         INSERT INTO t_15_7b1_once VALUES (0); \
         CREATE TRIGGER t_15_7b1_deadlock BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = 'installation.demo_seeded' \
                   AND (SELECT n FROM t_15_7b1_once) = 0 THEN \
           UPDATE t_15_7b1_once SET n = n + 1; \
           SIGNAL SQLSTATE '40001' SET MYSQL_ERRNO = 1213, MESSAGE_TEXT = '15-7b1 deadlock'; \
         END IF; END",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 200, "{body}");

    let fired: i32 = sqlx::query_scalar("SELECT n FROM t_15_7b1_once")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fired, 1, "le déclencheur a levé une 1213, une fois");
    let mut expected = before;
    expected.extend([
        "installation.demo_seeded".to_string(),
        "installation.step_completed".to_string(),
    ]);
    assert_eq!(audit_sequence(&pool).await, expected);
    assert_eq!(state_row(&pool).await.0, 3);
    assert!(!is_stub(&pool, company_id).await);
    assert_eq!(count_for_company(&pool, "vat_rates", company_id).await, 4);
}

// ===========================================================================
// Story 15-7b2 (#434, #279, #528) — la remise à zéro laisse sa trace, vide
// tout, et garde la société
// ===========================================================================

use kesh_db::backup::{TABLES_TO_TRUNCATE, reset_cleared_tables};

/// `id` d'une société qu'aucune ligne ne porte : les principaux orphelins du
/// montage la désignent (#528).
const DEAD_COMPANY_ID: i64 = 987_654;

/// `POST /api/v1/onboarding/reset` avec `KESH_PRODUCTION_RESET` posé le temps
/// de la requête (nextest : un processus par test ; `cargo test` de la CI :
/// un fil). Restauré avant toute assertion.
async fn reset_with_flag(app: &TestApp, token: &str) -> (u16, Value) {
    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
    // SAFETY (Rust 2024) : mutation d'environnement du processus de test,
    // restaurée aussitôt — patron de `onboarding_e2e.rs`.
    unsafe { std::env::set_var("KESH_PRODUCTION_RESET", "true") };
    let out = post(app, token, "reset", None).await;
    unsafe {
        match prev {
            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
        }
    }
    out
}

/// `POST /api/v1/onboarding/reset` **sans** le drapeau.
async fn reset_without_flag(app: &TestApp, token: &str) -> (u16, Value) {
    // SAFETY : cf. `reset_with_flag`.
    unsafe { std::env::remove_var("KESH_PRODUCTION_RESET") };
    post(app, token, "reset", None).await
}

/// `COUNT(*)` de chaque table de la liste canonique.
async fn table_counts(pool: &MySqlPool) -> std::collections::BTreeMap<&'static str, i64> {
    let mut out = std::collections::BTreeMap::new();
    for t in TABLES_TO_TRUNCATE {
        let n: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{t}`"))
            .fetch_one(pool)
            .await
            .unwrap();
        out.insert(*t, n);
    }
    out
}

/// Les lignes d'une table, chaque ligne réduite au texte de ses colonnes, triées
/// par `id` — instantané comparable sans dépendre des types SQL.
async fn snapshot(pool: &MySqlPool, table: &str) -> Vec<String> {
    let cols: Vec<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(!cols.is_empty(), "table {table} sans colonnes");
    let concat = cols
        .iter()
        .map(|c| format!("COALESCE(CAST(`{c}` AS CHAR), '∅')"))
        .collect::<Vec<_>>()
        .join(", '|', ");
    sqlx::query_scalar(&format!(
        "SELECT CONCAT({concat}) FROM `{table}` ORDER BY 1"
    ))
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Une ligne d'audit lue : `(action, entity_type, entity_id, company_id, id,
/// détails)` — détails bruts.
type AuditRow = (String, String, i64, Option<i64>, i64, Vec<u8>);

/// [`AuditRow`], détails décodés.
type AuditEntry = (String, String, i64, Option<i64>, i64, Value);

/// L'unique entrée d'audit après une remise à zéro.
async fn sole_entry(pool: &MySqlPool) -> AuditEntry {
    let rows: Vec<AuditRow> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, company_id, id, details_json FROM audit_log",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    assert_eq!(
        rows.len(),
        1,
        "exactement une entrée après la remise à zéro"
    );
    let (a, t, e, c, id, raw) = rows.into_iter().next().unwrap();
    (a, t, e, c, id, serde_json::from_slice(&raw).unwrap())
}

/// `(MIN(id), MAX(id), COUNT(*))` d'`audit_log`.
async fn audit_range(pool: &MySqlPool) -> (Option<i64>, Option<i64>, i64) {
    sqlx::query_as("SELECT MIN(id), MAX(id), COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `seed-demo` sur le montage commun : la démonstration est à l'étape 3.
async fn seeded_demo(pool: &MySqlPool) -> (TestApp, String, i64) {
    let (app, token, company_id) = bootstrap(pool).await;
    to_demo_step(&app, &token).await;
    let (status, body) = post(&app, &token, "seed-demo", None).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(state_row(pool).await.0, 3);
    (app, token, company_id)
}

/// Requête authentifiée quelconque (hors `/onboarding`).
async fn call(
    app: &TestApp,
    token: &str,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
) -> (u16, Value) {
    let mut req = app
        .client
        .request(method, app.url(path))
        .header("Authorization", format!("Bearer {token}"));
    if let Some(b) = body {
        req = req.json(&b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

/// Peuple la société d'une démonstration au-delà du seed : un contact, un
/// article, une facture **validée** (écriture), un avoir, une clé d'API saine
/// et un jeton de réinitialisation de mot de passe — L4-3 de la P4 : sans eux,
/// « inchangés » serait vrai à vide.
async fn populate(pool: &MySqlPool, app: &TestApp, token: &str, company_id: i64) -> i64 {
    let (contact_id, _product_id) =
        kesh_db::test_fixtures::seed_contact_and_product(pool, company_id)
            .await
            .unwrap();
    let today = chrono::Local::now().date_naive().to_string();
    let (status, body) = call(
        app,
        token,
        reqwest::Method::POST,
        "/api/v1/invoices",
        Some(json!({
            "contactId": contact_id,
            "date": today,
            // Sans TVA : le plan de démonstration ne désigne pas de compte de
            // TVA due, que la validation exigerait d'une ligne taxée.
            "lines": [{"description": "Conseil", "quantity": "1",
                       "unitPrice": "100.00", "vatRate": "0.00"}]
        })),
    )
    .await;
    assert_eq!(status, 201, "facture : {body}");
    let invoice_id = body["id"].as_i64().unwrap();
    let (status, body) = call(
        app,
        token,
        reqwest::Method::POST,
        &format!("/api/v1/invoices/{invoice_id}/validate"),
        None,
    )
    .await;
    assert_eq!(status, 200, "validation : {body}");
    let (status, body) = call(
        app,
        token,
        reqwest::Method::POST,
        "/api/v1/credit-notes",
        Some(json!({ "invoiceId": invoice_id, "date": today })),
    )
    .await;
    assert_eq!(status, 201, "avoir : {body}");
    let (key_id, _key) =
        create_key_via_http(&app.client, &app.base_url, token, "saine", "read-write").await;
    sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) \
         VALUES (?, SHA2('jeton', 256), NOW(3) + INTERVAL 1 HOUR)",
    )
    .bind(admin_user_id(pool).await)
    .execute(pool)
    .await
    .unwrap();
    key_id
}

// --- Test 4 (15-7b2) --------------------------------------------------------

/// Les tables vides **avant** la remise à zéro du test 4, malgré un montage
/// peuplé : ce que ni le seed ni le montage n'amorcent, dit et non ignoré
/// (R3 de la P1). Liste fermée, assertée égale à l'ensemble relevé.
const EMPTY_BEFORE_RESET: &[&str] = &[
    "bank_accounts",
    "bank_imports",
    "bank_profiles",
    "bank_transactions",
    "company_dunning_settings",
    "contact_persons",
    "dunning_levels",
    "email_templates",
    "imported_supplier_invoices",
    "invoice_reminders",
    "invoice_settlements",
    "payment_batch_items",
    "payment_batches",
    "projects",
    "reconciliation_rules",
    "supplier_invoice_lines",
    "supplier_invoices",
];

/// Test 4 (15-7b2, AC 2, 3, 5) — remise à zéro réussie d'une démonstration
/// peuplée : **chaque** table vidée l'est (#279, la classe entière), la piste
/// porte exactement `installation.reset` et ce qu'elle dit est exact, les
/// tables conservées sont inchangées.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_clears_every_company_table_and_traces_itself(pool: MySqlPool) {
    let (app, token, company_id) = seeded_demo(&pool).await;
    let sane_key = populate(&pool, &app, &token, company_id).await;

    let before = table_counts(&pool).await;
    for t in [
        "accounts",
        "fiscal_years",
        "vat_rates",
        "company_invoice_settings",
        "contacts",
        "products",
        "invoices",
        "invoice_lines",
        "credit_notes",
        "credit_note_lines",
        "journal_entries",
        "journal_entry_lines",
        "audit_log",
    ] {
        assert!(before[t] > 0, "montage : {t} doit être peuplée");
    }
    let cleared = reset_cleared_tables();
    let empty: std::collections::BTreeSet<&str> =
        cleared.iter().copied().filter(|t| before[t] == 0).collect();
    assert_eq!(
        empty,
        EMPTY_BEFORE_RESET.iter().copied().collect(),
        "liste fermée des tables vides avant la remise à zéro"
    );
    let (min_id, max_id, count) = audit_range(&pool).await;
    let preserved_before: Vec<Vec<String>> = {
        let mut v = Vec::new();
        for t in [
            "users",
            "api_keys",
            "refresh_tokens",
            "password_reset_tokens",
        ] {
            v.push(snapshot(&pool, t).await);
        }
        v
    };
    assert!(!preserved_before[3].is_empty(), "montage : jeton posé");
    let key_before: (Option<chrono::NaiveDateTime>, i32) =
        sqlx::query_as("SELECT revoked_at, version FROM api_keys WHERE id = ?")
            .bind(sane_key)
            .fetch_one(&pool)
            .await
            .unwrap();

    let (status, body) = reset_with_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["stepCompleted"], 0);
    assert_eq!(body["isDemo"], false);

    let (action, entity_type, entity_id, audit_company, id, details) = sole_entry(&pool).await;
    assert_eq!(action, "installation.reset");
    assert_eq!(entity_type, "installation");
    assert_eq!(
        entity_id,
        kesh_db::entities::audit_log::AUDIT_ENTITY_ID_NONE
    );
    assert_eq!(audit_company, Some(company_id));
    assert!(id > max_id.unwrap(), "l'id suit la plage effacée");
    assert_eq!(
        details,
        json!({
            "step_before": 3,
            "is_demo_before": true,
            "company_id": company_id,
            "company_recreated": false,
            "audit_entries_erased": count,
            "erased_id_min": min_id,
            "erased_id_max": max_id,
            "tables_cleared": cleared,
            "users_repointed": 0,
            "api_keys_revoked": [],
            "api_keys_repointed": 0,
        })
    );
    let after = table_counts(&pool).await;
    for t in &cleared {
        if *t != "audit_log" {
            assert_eq!(after[t], 0, "#279 : {t} doit être vidée");
        }
    }
    let mut preserved_after = Vec::new();
    for t in [
        "users",
        "api_keys",
        "refresh_tokens",
        "password_reset_tokens",
    ] {
        preserved_after.push(snapshot(&pool, t).await);
    }
    assert_eq!(
        preserved_after, preserved_before,
        "tables conservées inchangées"
    );
    let key_after: (Option<chrono::NaiveDateTime>, i32) =
        sqlx::query_as("SELECT revoked_at, version FROM api_keys WHERE id = ?")
            .bind(sane_key)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(key_after, key_before, "la clé saine n'est pas révoquée");
    assert!(key_after.0.is_none());
    assert_eq!(after["companies"], 1);
    assert_eq!(after["onboarding_state"], 1);
}

// --- Test 5 (15-7b2) --------------------------------------------------------

/// Les 19 colonnes de `companies` que le montage du test 5 perturbe — liste
/// FERMÉE : l'inventaire du schéma doit l'égaler, aux quatre exclusions près.
const PERTURBED: &[&str] = &[
    "name",
    "address",
    "ide_number",
    "org_type",
    "accounting_language",
    "instance_language",
    "country",
    "is_stub",
    "address_street",
    "address_building",
    "address_postal_code",
    "address_city",
    "address_country",
    "first_name",
    "last_name",
    "email",
    "phone",
    "website",
    "books_locked_through",
];

/// Colonnes de `companies` exclues de la comparaison au stub de référence.
const NOT_COMPARED: &[&str] = &["id", "version", "created_at", "updated_at"];

/// Test 5 (15-7b2, AC 4) — #528 et les colonnes du stub : la société survit à
/// la remise à zéro (même id, même jeton, `GET /companies/current` → 200) et
/// **chaque** colonne revient à la valeur d'un stub fraîchement inséré.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_restores_the_stub_columns_in_place(pool: MySqlPool) {
    let (app, token, company_id) = seeded_demo(&pool).await;
    sqlx::query(
        "UPDATE companies SET name = 'Démo perturbée', address = 'x', \
            ide_number = 'CHE109322551', org_type = 'Pme', accounting_language = 'DE', \
            instance_language = 'DE', country = 'FR', is_stub = FALSE, \
            address_street = 'x', address_building = 'x', address_postal_code = 'x', \
            address_city = 'x', address_country = 'FR', first_name = 'x', last_name = 'x', \
            email = 'x@example.ch', phone = 'x', website = 'x', \
            books_locked_through = '2024-12-31' \
         WHERE id = ?",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap();

    // Inventaire : une colonne neuve rougit tant qu'elle n'est pas classée.
    let columns: std::collections::BTreeSet<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'companies'",
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .collect();
    let expected: std::collections::BTreeSet<String> = PERTURBED
        .iter()
        .chain(NOT_COMPARED)
        .map(|c| c.to_string())
        .collect();
    assert_eq!(columns, expected, "colonnes de companies à classer");

    let (status, current) = call(
        &app,
        &token,
        reqwest::Method::GET,
        "/api/v1/companies/current",
        None,
    )
    .await;
    assert_eq!(status, 200);
    assert_eq!(current["company"]["id"], company_id);

    let (status, body) = reset_with_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["isStub"], true, "la réponse dit la société provisoire");

    let (status, current) = call(
        &app,
        &token,
        reqwest::Method::GET,
        "/api/v1/companies/current",
        None,
    )
    .await;
    assert_eq!(status, 200, "même jeton, même société : {current}");
    assert_eq!(current["company"]["id"], company_id);
    let user_company: i64 = sqlx::query_scalar("SELECT company_id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_company, company_id);

    // Référence : un stub inséré dans une transaction ANNULÉE — la base ne
    // compte jamais deux sociétés au moment d'une remise à zéro.
    let compared: Vec<&str> = PERTURBED.to_vec();
    let mismatch = compared
        .iter()
        .map(|c| format!("IF(a.`{c}` <=> b.`{c}`, NULL, '{c}')"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut tx = pool.begin().await.unwrap();
    let reference =
        kesh_db::repositories::companies::insert_stub(&mut *tx, kesh_db::entities::Language::Fr)
            .await
            .unwrap();
    let differing: Option<String> = sqlx::query_scalar(&format!(
        "SELECT CONCAT_WS(',', {mismatch}) FROM companies a, companies b \
         WHERE a.id = ? AND b.id = ?"
    ))
    .bind(company_id)
    .bind(reference)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.rollback().await.unwrap();
    assert_eq!(
        differing.as_deref().unwrap_or(""),
        "",
        "colonnes qui diffèrent d'un stub neuf"
    );

    // `language` n'emprunte plus le chemin « aucune société ».
    let marker = max_audit_id(&pool).await;
    let (status, body) = post(&app, &token, "language", Some(json!({ "language": "DE" }))).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        actions_after(&pool, marker).await,
        ["company.updated", "installation.step_completed"]
    );
}

// --- Tests 6a à 6d (15-7b2) -------------------------------------------------

/// Rend la clé et l'utilisateur ORPHELINS — l'état laissé par une remise à
/// zéro v0.12.x (#528) — par une connexion **détachée du pool**
/// (`FOREIGN_KEY_CHECKS=0` n'y retourne jamais). `drop_companies` : la société
/// est aussi effacée (installation sans société).
async fn make_orphans(pool: &MySqlPool, drop_companies: bool) {
    use sqlx::Connection;
    let mut conn = pool.acquire().await.unwrap().detach();
    sqlx::raw_sql("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut conn)
        .await
        .unwrap();
    if drop_companies {
        sqlx::raw_sql("DELETE FROM companies")
            .execute(&mut conn)
            .await
            .unwrap();
    }
    sqlx::query("UPDATE users SET company_id = ?")
        .bind(DEAD_COMPANY_ID)
        .execute(&mut conn)
        .await
        .unwrap();
    sqlx::query("UPDATE api_keys SET company_id = ?")
        .bind(DEAD_COMPANY_ID)
        .execute(&mut conn)
        .await
        .unwrap();
    conn.close().await.unwrap();
}

/// Ce que l'entrée d'audit doit dire de la clé révoquée, relu de la base.
async fn revoked_key_json(pool: &MySqlPool, key_id: i64) -> Value {
    let (name, created_by, created_at, last_used_at): (
        String,
        i64,
        chrono::NaiveDateTime,
        Option<chrono::NaiveDateTime>,
    ) = sqlx::query_as(
        "SELECT name, created_by_user_id, created_at, last_used_at FROM api_keys WHERE id = ?",
    )
    .bind(key_id)
    .fetch_one(pool)
    .await
    .unwrap();
    json!([{
        "id": key_id,
        "name": name,
        "created_by_user_id": created_by,
        "created_at": created_at,
        "last_used_at": last_used_at,
    }])
}

/// La clé est révoquée, désigne `company_id`, et son jeton n'authentifie plus.
async fn assert_key_revoked_on(pool: &MySqlPool, key_id: i64, key: &str, company_id: i64) {
    let (company, revoked): (i64, bool) =
        sqlx::query_as("SELECT company_id, revoked_at IS NOT NULL FROM api_keys WHERE id = ?")
            .bind(key_id)
            .fetch_one(pool)
            .await
            .unwrap();
    assert_eq!(company, company_id, "clé rattachée à la société");
    assert!(revoked, "clé orpheline révoquée");
    let auth = kesh_db::repositories::api_keys::find_active_auth_by_key_hash(
        pool,
        &kesh_api::auth::api_key::sha256_hex(key),
    )
    .await
    .unwrap();
    assert!(auth.is_none(), "le jeton n'authentifie plus");
}

/// Après reconnexion, `GET /companies/current` → 200 avec `company_id`.
async fn assert_current_after_login(app: &TestApp, company_id: i64) {
    let fresh = login(app).await;
    let (status, body) = call(
        app,
        &fresh,
        reqwest::Method::GET,
        "/api/v1/companies/current",
        None,
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["company"]["id"], company_id);
}

/// Test 6a (15-7b2, AC 4, 5) — installation atteinte par #528 **avec**
/// société : l'utilisateur est rattaché à la société conservée, la clé active
/// orpheline est révoquée puis rattachée, et l'entrée le dit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_reattaches_the_orphans_of_a_528_installation(pool: MySqlPool) {
    let (app, token, company_id) = bootstrap(&pool).await;
    let (key_id, key) = create_key_via_http(
        &app.client,
        &app.base_url,
        &token,
        "née en démo",
        "read-write",
    )
    .await;
    make_orphans(&pool, false).await;

    let (status, body) = reset_without_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");

    let user_company: i64 = sqlx::query_scalar("SELECT company_id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_company, company_id);
    assert_key_revoked_on(&pool, key_id, &key, company_id).await;
    let (_, _, _, audit_company, _, details) = sole_entry(&pool).await;
    assert_eq!(
        audit_company,
        Some(company_id),
        "écrite après le rattachement"
    );
    assert_eq!(details["users_repointed"], 1);
    assert_eq!(details["company_recreated"], false);
    assert_eq!(
        details["api_keys_revoked"],
        revoked_key_json(&pool, key_id).await
    );
    assert_eq!(details["api_keys_repointed"], 1);
}

/// Test 6b (15-7b2, AC 2, 4, 5) — installation atteinte par #528 **sans**
/// société : la remise à zéro répond 200 (et non 500), recrée la société
/// provisoire et y rattache utilisateur et clé (révoquée). Après reconnexion,
/// la société courante est la recréée — l'ancien jeton n'est pas asserté
/// (limite de l'AC 4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_recreates_the_company_of_a_528_installation(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;
    let (key_id, key) = create_key_via_http(
        &app.client,
        &app.base_url,
        &token,
        "née en démo",
        "read-write",
    )
    .await;
    make_orphans(&pool, true).await;

    let (status, body) = reset_without_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");

    let ids: Vec<(i64, bool)> = sqlx::query_as("SELECT id, is_stub FROM companies")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids.len(), 1, "une société stub recréée");
    let (new_id, is_stub) = ids[0];
    assert!(is_stub);
    let user_company: i64 = sqlx::query_scalar("SELECT company_id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_company, new_id);
    assert_key_revoked_on(&pool, key_id, &key, new_id).await;
    let (_, _, _, audit_company, _, details) = sole_entry(&pool).await;
    assert_eq!(audit_company, Some(new_id));
    assert_eq!(details["company_recreated"], true);
    assert_eq!(details["company_id"], new_id);
    assert_eq!(details["users_repointed"], 1);
    assert_eq!(details["api_keys_repointed"], 1);
    assert_current_after_login(&app, new_id).await;
}

/// Test 6c (15-7b2, AC 4 ; remplace le test 7 de la 15-7a2) — `language` sur
/// une installation sans société : la société créée **rattache** l'utilisateur
/// et ne **réveille** aucune clé (F4-1/R4-1 de la P4) ; `company.created` le
/// dit, et son `company_id` d'audit est la société créée, non plus l'id mort.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn language_without_company_reattaches_and_revokes(pool: MySqlPool) {
    let (app, token, dead_company_id) = bootstrap(&pool).await;
    let (key_id, key) = create_key_via_http(
        &app.client,
        &app.base_url,
        &token,
        "née en démo",
        "read-write",
    )
    .await;
    make_orphans(&pool, true).await;
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
    assert_ne!(new_id, dead_company_id);
    let user_company: i64 = sqlx::query_scalar("SELECT company_id FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(user_company, new_id, "utilisateur rattaché");
    assert_key_revoked_on(&pool, key_id, &key, new_id).await;
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
    assert_eq!(
        audit_company_id,
        Some(new_id),
        "#528 fermé : plus d'id mort"
    );
    let details: Value = serde_json::from_slice(&raw).unwrap();
    assert_eq!(
        details,
        json!({
            "instance_language": "IT",
            "is_stub": true,
            "users_repointed": 1,
            "api_keys_revoked": revoked_key_json(&pool, key_id).await,
            "api_keys_repointed": 1,
        })
    );
    assert_current_after_login(&app, new_id).await;
}

/// Test 6d (15-7b2, AC 2) — deux sociétés : la remise à zéro rend 500
/// (`Invariant`), et tables, sociétés et piste sont intactes.
async fn reset_refuses_two_companies(pool: MySqlPool, second_is_stub: bool) {
    let (app, token, _) = bootstrap(&pool).await;
    if second_is_stub {
        // #542 : une seconde société provisoire, sans utilisateur ni clé.
        kesh_db::repositories::companies::insert_stub(&pool, kesh_db::entities::Language::Fr)
            .await
            .unwrap();
    } else {
        common::create_test_company(&pool).await;
    }
    let before = table_counts(&pool).await;
    let companies_before = snapshot(&pool, "companies").await;
    let audit_before = audit_sequence(&pool).await;

    let (status, body) = reset_without_flag(&app, &token).await;
    assert_eq!(status, 500, "{body}");
    assert_eq!(table_counts(&pool).await, before);
    assert_eq!(snapshot(&pool, "companies").await, companies_before);
    assert_eq!(audit_sequence(&pool).await, audit_before);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_refuses_a_second_real_company(pool: MySqlPool) {
    reset_refuses_two_companies(pool, false).await;
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_refuses_a_second_stub_company(pool: MySqlPool) {
    reset_refuses_two_companies(pool, true).await;
}

// --- Tests 7, 7b, 7c (15-7b2) -----------------------------------------------

/// Test 7 (15-7b2, AC 2) — les trois gardes, par HTTP : étape 7 ⇒ 400 ;
/// production à l'étape 3 ⇒ 403 ; démonstration à l'étape 3 sans drapeau ⇒
/// 403. Chaque fois, tables et piste intactes. Séquentiel : il prouve les
/// gardes, pas qu'elles sont évaluées sous le verrou (test 7b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_guards_refuse_and_erase_nothing(pool: MySqlPool) {
    let (app, token, _) = seeded_demo(&pool).await;
    for (step, is_demo, flag, expected, code) in [
        (7, true, true, 400, "ONBOARDING_STEP_ALREADY_COMPLETED"),
        (3, false, true, 403, "ONBOARDING_RESET_FORBIDDEN"),
        (3, true, false, 403, "ONBOARDING_RESET_FORBIDDEN"),
    ] {
        set_step(&pool, step).await;
        set_demo(&pool, is_demo).await;
        let before = table_counts(&pool).await;
        let audit = audit_sequence(&pool).await;
        let (status, body) = if flag {
            reset_with_flag(&app, &token).await
        } else {
            reset_without_flag(&app, &token).await
        };
        assert_eq!(status, expected, "étape {step}, démo {is_demo} : {body}");
        assert_eq!(body["error"]["code"], code);
        assert_eq!(table_counts(&pool).await, before, "étape {step}");
        assert_eq!(audit_sequence(&pool).await, audit, "étape {step}");
    }
}

/// Test 7b (15-7b2, AC 2 ; F-2 de la P1) — **les gardes sont évaluées sous le
/// verrou de la transaction qui efface**, de façon déterministe : une connexion
/// A tient `onboarding_state` ; `reset_demo` est vu bloqué sur son `FOR
/// UPDATE` ; A passe l'étape 7 et commite ⇒ `StepAlreadyCompleted`, rien
/// d'effacé. Montage peuplé (F3-5 de la P3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_guards_are_evaluated_under_the_erasing_lock(pool: MySqlPool) {
    use sqlx::Connection;
    let (app, token, company_id) = seeded_demo(&pool).await;
    let _ = populate(&pool, &app, &token, company_id).await;
    let before = table_counts(&pool).await;
    for t in [
        "accounts",
        "fiscal_years",
        "journal_entries",
        "journal_entry_lines",
        "audit_log",
    ] {
        assert!(before[t] > 0, "montage : {t}");
    }
    let actor = (admin_user_id(&pool).await, None);

    let mut a = sqlx::MySqlConnection::connect_with(&pool.connect_options())
        .await
        .unwrap();
    let mut tx_a = a.begin().await.unwrap();
    sqlx::query(kesh_db::repositories::onboarding::LOCK_SQL)
        .fetch_one(&mut *tx_a)
        .await
        .unwrap();

    let p = pool.clone();
    let handle = tokio::spawn(async move { kesh_seed::reset_demo(&p, actor, true).await });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["onboarding_state", "FOR UPDATE"],
            || handle.is_finished(),
        )
        .await,
        "reset_demo doit attendre le verrou d'état"
    );
    sqlx::query("UPDATE onboarding_state SET step_completed = 7")
        .execute(&mut *tx_a)
        .await
        .unwrap();
    tx_a.commit().await.unwrap();

    let result = handle.await.unwrap();
    assert!(
        matches!(result, Err(kesh_seed::SeedError::StepAlreadyCompleted)),
        "{result:?}"
    );
    assert_eq!(table_counts(&pool).await, before, "rien d'effacé");
}

/// Test 7c (15-7b2, AC 2 ; R6 de la P1) — ligne d'état absente, `reset_demo`
/// appelé directement (le handler la recréerait) ⇒ `Invariant`, rien d'effacé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_without_state_row_is_an_invariant(pool: MySqlPool) {
    let (_app, _token, _) = seeded_demo(&pool).await;
    kesh_db::repositories::onboarding::delete_state(&pool)
        .await
        .unwrap();
    let before = table_counts(&pool).await;
    let result = kesh_seed::reset_demo(&pool, (admin_user_id(&pool).await, None), true).await;
    assert!(
        matches!(
            result,
            Err(kesh_seed::SeedError::Db(
                kesh_db::errors::DbError::Invariant(_)
            ))
        ),
        "{result:?}"
    );
    assert_eq!(table_counts(&pool).await, before);
}

// --- Test 8 (15-7b2) --------------------------------------------------------

/// Test 8 (15-7b2, AC 2, 6 ; F-2 de la P4) — un échec injecté à la dernière
/// écriture ne laisse **rien** effacé, et la connexion de l'essai — qui portait
/// `FOREIGN_KEY_CHECKS=0` — n'est **jamais rendue** au pool : sur un pool à une
/// connexion, l'identifiant de connexion change (c'est la preuve). Puis, le
/// déclencheur retiré, la même remise à zéro réussit et rend un
/// `ResetOutcome` exact.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_failure_erases_nothing_and_never_returns_its_connection(pool: MySqlPool) {
    let (_app, _token, company_id) = seeded_demo(&pool).await;
    let log_bin: i64 = sqlx::query_scalar("SELECT CAST(@@log_bin AS SIGNED)")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        log_bin, 0,
        "pré-requis du montage : journal binaire inactif (sinon CREATE TRIGGER exige \
         SUPER ou log_bin_trust_function_creators)"
    );
    let pool1 = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with((*pool.connect_options()).clone())
        .await
        .unwrap();
    let conn_id: i64 = sqlx::query_scalar("SELECT CAST(CONNECTION_ID() AS SIGNED)")
        .fetch_one(&pool1)
        .await
        .unwrap();
    sqlx::raw_sql(
        "CREATE TRIGGER t_15_7b2_fail BEFORE INSERT ON audit_log FOR EACH ROW \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '15-7b2 atomicity'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let before = table_counts(&pool).await;
    let company_before = snapshot(&pool, "companies").await;
    let state_before = snapshot(&pool, "onboarding_state").await;
    let actor = (admin_user_id(&pool).await, None);

    let result = kesh_seed::reset_demo(&pool1, actor, true).await;
    assert!(result.is_err(), "{result:?}");
    assert_eq!(table_counts(&pool).await, before, "rien d'effacé");
    assert_eq!(snapshot(&pool, "companies").await, company_before);
    assert_eq!(snapshot(&pool, "onboarding_state").await, state_before);

    let (new_id, fk): (i64, i64) = sqlx::query_as(
        "SELECT CAST(CONNECTION_ID() AS SIGNED), CAST(@@SESSION.foreign_key_checks AS SIGNED)",
    )
    .fetch_one(&pool1)
    .await
    .unwrap();
    assert_ne!(
        new_id, conn_id,
        "la connexion de l'essai a été fermée, non rendue"
    );
    assert_eq!(fk, 1);

    sqlx::raw_sql("DROP TRIGGER t_15_7b2_fail")
        .execute(&pool)
        .await
        .unwrap();
    let erased = audit_range(&pool).await.2;
    let outcome = kesh_seed::reset_demo(&pool1, actor, true).await.unwrap();
    assert_eq!(outcome.company_id, company_id);
    assert!(!outcome.company_recreated);
    assert_eq!(outcome.audit_entries_erased, erased as u64);
    assert!(outcome.principals.is_empty());
}

// --- Test 9 (15-7b2) --------------------------------------------------------

/// Test 9 (15-7b2, AC 5) — piste vide avant la remise à zéro : rien d'effacé,
/// plage à `null`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_of_an_empty_trail(pool: MySqlPool) {
    let (app, token, _) = bootstrap(&pool).await;
    sqlx::raw_sql("DELETE FROM audit_log")
        .execute(&pool)
        .await
        .unwrap();
    let (status, body) = reset_without_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");
    let (_, _, _, _, _, details) = sole_entry(&pool).await;
    assert_eq!(details["audit_entries_erased"], 0);
    assert!(details["erased_id_min"].is_null());
    assert!(details["erased_id_max"].is_null());
    assert_eq!(details["step_before"], 0);
}

// --- Tests 13 et 13b (15-7b2) -----------------------------------------------

/// Test 13 (15-7b2, AC 2 ; R3-1 de la P3) — le prédicat de rejeu sur une
/// **vraie** 1213, née d'un cycle de verrous InnoDB entre deux connexions hors
/// du pool, et passée par `map_db_error` ⇒ vrai ; une attente expirée (1205)
/// ⇒ faux ; `StepAlreadyCompleted`, `ResetForbidden`, `Db(NotFound)` ⇒ faux.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn is_seed_retryable_on_a_real_deadlock(pool: MySqlPool) {
    use kesh_seed::{SeedAttemptError, is_seed_retryable};
    use sqlx::Connection;
    sqlx::raw_sql(
        "CREATE TABLE reset_retry_probe (id INT PRIMARY KEY) ENGINE=InnoDB; \
         INSERT INTO reset_retry_probe VALUES (1), (2);",
    )
    .execute(&pool)
    .await
    .unwrap();
    let opts = pool.connect_options();
    let mut a = sqlx::MySqlConnection::connect_with(&opts).await.unwrap();
    let mut b = sqlx::MySqlConnection::connect_with(&opts).await.unwrap();
    let lock = "SELECT id FROM reset_retry_probe WHERE id = ? FOR UPDATE";
    let mut ta = a.begin().await.unwrap();
    let mut tb = b.begin().await.unwrap();
    sqlx::query(lock).bind(1).fetch_one(&mut *ta).await.unwrap();
    sqlx::query(lock).bind(2).fetch_one(&mut *tb).await.unwrap();
    let (ra, rb) = tokio::join!(
        sqlx::query(lock).bind(2).fetch_one(&mut *ta),
        sqlx::query(lock).bind(1).fetch_one(&mut *tb),
    );
    let victim = match (ra, rb) {
        (Err(e), Ok(_)) | (Ok(_), Err(e)) => e,
        other => panic!("un cycle doit désigner exactement une victime : {other:?}"),
    };
    let deadlock = SeedAttemptError::Db(kesh_db::errors::map_db_error(victim));
    assert!(is_seed_retryable(&deadlock), "1213 réelle : {deadlock:?}");
    // Annulations explicites : une `Transaction` lâchée ne fait que mettre son
    // ROLLBACK en file, et le verrou survivrait jusqu'au prochain usage de la
    // connexion. Celle de la victime est déjà annulée par InnoDB.
    let _ = ta.rollback().await;
    let _ = tb.rollback().await;

    let mut c = sqlx::MySqlConnection::connect_with(&opts).await.unwrap();
    let mut tc = c.begin().await.unwrap();
    sqlx::query(lock).bind(1).fetch_one(&mut *tc).await.unwrap();
    let mut d = sqlx::MySqlConnection::connect_with(&opts).await.unwrap();
    sqlx::query("SET SESSION innodb_lock_wait_timeout = 1")
        .execute(&mut d)
        .await
        .unwrap();
    let mut td = d.begin().await.unwrap();
    let timeout = sqlx::query(lock)
        .bind(1)
        .fetch_one(&mut *td)
        .await
        .unwrap_err();
    assert!(!is_seed_retryable(&SeedAttemptError::Db(
        kesh_db::errors::map_db_error(timeout)
    )));
    assert!(!is_seed_retryable(&SeedAttemptError::StepAlreadyCompleted));
    assert!(!is_seed_retryable(&SeedAttemptError::ResetForbidden));
    assert!(!is_seed_retryable(&SeedAttemptError::Db(
        kesh_db::errors::DbError::NotFound
    )));
}

/// Test 13b (15-7b2, C-15-7b2-2) — le rejeu de `reset_demo` **lui-même**, de
/// bout en bout : un déclencheur lève une 1213 à la première écriture
/// d'`installation.reset`, et à elle seule (compteur en table MyISAM, que
/// l'annulation n'efface pas). Le second essai aboutit : 200, une entrée, les
/// tables vidées. Sans rejeu, la route rendrait 500.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_replays_a_deadlocked_attempt(pool: MySqlPool) {
    let (app, token, _) = seeded_demo(&pool).await;
    sqlx::raw_sql(
        "CREATE TABLE t_15_7b2_once (n INT NOT NULL) ENGINE = MyISAM; \
         INSERT INTO t_15_7b2_once VALUES (0); \
         CREATE TRIGGER t_15_7b2_deadlock BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = 'installation.reset' \
                   AND (SELECT n FROM t_15_7b2_once) = 0 THEN \
           UPDATE t_15_7b2_once SET n = n + 1; \
           SIGNAL SQLSTATE '40001' SET MYSQL_ERRNO = 1213, MESSAGE_TEXT = '15-7b2 deadlock'; \
         END IF; END",
    )
    .execute(&pool)
    .await
    .unwrap();

    let (status, body) = reset_with_flag(&app, &token).await;
    assert_eq!(status, 200, "{body}");
    let fired: i32 = sqlx::query_scalar("SELECT n FROM t_15_7b2_once")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(fired, 1, "le déclencheur a levé une 1213, une fois");
    let (action, ..) = sole_entry(&pool).await;
    assert_eq!(action, "installation.reset");
    assert_eq!(table_counts(&pool).await["accounts"], 0);
    assert_eq!(state_row(&pool).await.0, 0);
}
