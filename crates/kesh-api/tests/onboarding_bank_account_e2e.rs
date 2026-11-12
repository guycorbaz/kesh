//! L'étape bancaire de l'assistant pose la description et le compte comptable
//! lié du compte bancaire — Story 34-1 (#966).
//!
//! Le compte de l'assistant naissait toujours sans lien (`journal_account_id =
//! NULL`), et la première réconciliation tombait sur
//! `BANK_ACCOUNT_NOT_CONFIGURED`. La route accepte désormais `description` et
//! `journalAccountId`, facultatifs : **absent = inchangé, `null` = effacé**
//! (C-e341-34), mêmes contrôles qu'à `POST /bank-accounts`, et refus d'un
//! compte de rôle `Receivable` ou `Payable` — les futurs comptes de créance que
//! la finalisation désignera (C-e341-3).
//!
//! ⛔ Le numéro d'un compte ne lui donne aucun rôle (Story 14-3a) : la
//! présélection du 1020 n'existe qu'à l'écran, et les tests de l'AC-4 prouvent
//! que le serveur lit le **rôle** (compte renuméroté refusé, 1100 sans rôle
//! accepté).
//!
//! Montage : base vide, bootstrap (société provisoire), puis le parcours de
//! production **par les routes** jusqu'à l'étape 6 — plan PME chargé, 1020,
//! 1100 (`Receivable`), 2000 (`Payable`), 3000 (produit) et le groupe 10.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use serde_json::{Value, json};
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
/// IBAN de démonstration de la suite (`onboarding_audit_e2e.rs`), synthétique.
const IBAN: &str = "CH93 0076 2011 6238 5295 7";
const IBAN_COMPACT: &str = "CH9300762011623852957";

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

/// Bootstrap sur base vide, puis le parcours de production jusqu'à l'étape 6
/// (plan PME en français). Rend `(app, jeton, société)`.
async fn at_bank_step_on(pool: &MySqlPool, app_pool: MySqlPool) -> (TestApp, String, i64) {
    let app = spawn_app(app_pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    kesh_db::repositories::onboarding::init_state(pool)
        .await
        .unwrap();
    for (route, body) in [
        ("language", Some(json!({ "language": "FR" }))),
        ("mode", Some(json!({ "mode": "guided" }))),
        ("start-production", None),
        ("org-type", Some(json!({ "orgType": "Pme" }))),
        ("accounting-language", Some(json!({ "language": "FR" }))),
        (
            "coordinates",
            Some(json!({
                "name": "Exemple Banque SA",
                "address": {"street": "Rue du Lac", "building": "12", "postalCode": "1003",
                            "city": "Lausanne", "country": "CH"},
                "ideNumber": null,
            })),
        ),
    ] {
        let (status, body) = post(&app, &token, route, body).await;
        assert_eq!(status, 200, "{route} : {body}");
    }
    assert_eq!(step(pool).await, 6);
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    (app, token, company_id)
}

async fn at_bank_step(pool: &MySqlPool) -> (TestApp, String, i64) {
    at_bank_step_on(pool, pool.clone()).await
}

async fn step(pool: &MySqlPool) -> i32 {
    sqlx::query_scalar("SELECT step_completed FROM onboarding_state")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn set_step(pool: &MySqlPool, step: i32) {
    sqlx::query("UPDATE onboarding_state SET step_completed = ?")
        .bind(step)
        .execute(pool)
        .await
        .unwrap();
}

/// L'identifiant du compte de numéro `number` dans la société.
async fn account_id(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
        .bind(company_id)
        .bind(number)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|e| panic!("compte {number} : {e}"))
}

/// `(description, journal_account_id, version)` de chaque compte bancaire.
async fn bank_rows(pool: &MySqlPool) -> Vec<(Option<String>, Option<i64>, i32)> {
    sqlx::query_as("SELECT description, journal_account_id, version FROM bank_accounts ORDER BY id")
        .fetch_all(pool)
        .await
        .unwrap()
}

async fn max_audit_id(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// `(action, détails)` des entrées écrites après le repère `after_id`.
async fn entries_after(pool: &MySqlPool, after_id: i64) -> Vec<(String, Value)> {
    let rows: Vec<(String, Option<Vec<u8>>)> =
        sqlx::query_as("SELECT action, details_json FROM audit_log WHERE id > ? ORDER BY id")
            .bind(after_id)
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

fn actions(entries: &[(String, Value)]) -> Vec<&str> {
    entries.iter().map(|(a, _)| a.as_str()).collect()
}

fn body(extra: Value) -> Value {
    let mut b = json!({ "bankName": "UBS", "iban": IBAN, "qrIban": null });
    for (k, v) in extra.as_object().unwrap() {
        b[k] = v.clone();
    }
    b
}

// --- AC-1, AC-3, AC-5 : création ---------------------------------------------

/// La description (normalisée) et le lien saisis sont écrits, et
/// `bank_account.created` les porte, sans l'IBAN en clair (AC-3, AC-5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_writes_description_and_link(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let bank_ledger = account_id(&pool, company_id, "1020").await;
    let marker = max_audit_id(&pool).await;

    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({
            "description": "  Personnel  ",
            "journalAccountId": bank_ledger,
        }))),
    )
    .await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(resp["stepCompleted"], 7);

    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Personnel".to_string()), Some(bank_ledger), 1)]
    );
    let all = entries_after(&pool, marker).await;
    assert_eq!(
        actions(&all),
        ["bank_account.created", "installation.step_completed"]
    );
    let d = &all[0].1;
    assert_eq!(d["description"], "Personnel");
    assert_eq!(d["journal_account_id"], bank_ledger);
    assert_eq!(d["iban_present"], true);
    let raw = d.to_string();
    assert!(!raw.contains(IBAN_COMPACT) && !raw.contains(IBAN));
}

/// Un corps de la 0.17.5 (champs absents) crée le compte sans description ni
/// lien ; l'audit le dit (`null`) (AC-1, AC-5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_without_new_fields_creates_an_unlinked_account(pool: MySqlPool) {
    let (app, token, _) = at_bank_step(&pool).await;
    let marker = max_audit_id(&pool).await;

    let (status, resp) = post(&app, &token, "bank-account", Some(body(json!({})))).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(bank_rows(&pool).await, [(None, None, 1)]);
    let all = entries_after(&pool, marker).await;
    let d = &all[0].1;
    assert_eq!(d["description"], Value::Null);
    assert_eq!(d["journal_account_id"], Value::Null);
    assert!(d.as_object().unwrap().contains_key("description"));
}

/// Une description faite d'espaces vaut absente ; `null` aussi (AC-2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_blank_description_is_absent(pool: MySqlPool) {
    let (app, token, _) = at_bank_step(&pool).await;
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(
            json!({ "description": "   ", "journalAccountId": null }),
        )),
    )
    .await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(bank_rows(&pool).await, [(None, None, 1)]);
}

// --- AC-3 : rejeux -----------------------------------------------------------

/// Rejeu identique → `Unchanged` (aucune trace de compte) ; corps 0.17.5 →
/// `Unchanged`, description et lien intacts ; `journalAccountId: null` →
/// `Updated`, lien effacé, description intacte ; description seule changée →
/// `Updated` ; `description: null` → effacée (AC-1, AC-3, AC-5 ; M1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_replays(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let bank_ledger = account_id(&pool, company_id, "1020").await;
    let full = body(json!({ "description": "Personnel", "journalAccountId": bank_ledger }));
    assert_eq!(
        post(&app, &token, "bank-account", Some(full.clone()))
            .await
            .0,
        200
    );

    // (a) rejeu identique : no-op.
    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    assert_eq!(
        post(&app, &token, "bank-account", Some(full.clone()))
            .await
            .0,
        200
    );
    assert_eq!(
        actions(&entries_after(&pool, marker).await),
        ["installation.step_completed"]
    );
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Personnel".to_string()), Some(bank_ledger), 1)]
    );

    // (b) corps 0.17.5, même banque et IBAN : inchangé, rien d'effacé (le
    // no-op ; l'UPDATE sans les champs : `bank_step_update_without_new_fields_keeps_them`).
    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    assert_eq!(
        post(&app, &token, "bank-account", Some(body(json!({}))))
            .await
            .0,
        200
    );
    assert_eq!(
        actions(&entries_after(&pool, marker).await),
        ["installation.step_completed"]
    );
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Personnel".to_string()), Some(bank_ledger), 1)]
    );

    // (c) même corps, lien à null : modifié, lien effacé (M1).
    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    assert_eq!(
        post(
            &app,
            &token,
            "bank-account",
            Some(body(json!({ "journalAccountId": null })))
        )
        .await
        .0,
        200
    );
    let all = entries_after(&pool, marker).await;
    assert_eq!(
        actions(&all),
        ["bank_account.updated", "installation.step_completed"]
    );
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Personnel".to_string()), None, 2)]
    );
    let d = &all[0].1;
    assert_eq!(d["before"]["description"], "Personnel");
    assert_eq!(d["after"]["description"], "Personnel");
    assert_eq!(d["before"]["journal_account_id"], bank_ledger);
    assert_eq!(d["after"]["journal_account_id"], Value::Null);
    assert_eq!(d["iban_changed"], false);

    // (d) seule la description change : modifié.
    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    assert_eq!(
        post(
            &app,
            &token,
            "bank-account",
            Some(body(json!({ "description": "Épargne" })))
        )
        .await
        .0,
        200
    );
    let all = entries_after(&pool, marker).await;
    assert_eq!(
        actions(&all),
        ["bank_account.updated", "installation.step_completed"]
    );
    assert_eq!(all[0].1["after"]["description"], "Épargne");
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Épargne".to_string()), None, 3)]
    );

    // (e) description à null : effacée ; le lien, redonné, est reposé.
    set_step(&pool, 6).await;
    assert_eq!(
        post(
            &app,
            &token,
            "bank-account",
            Some(body(
                json!({ "description": null, "journalAccountId": bank_ledger })
            ))
        )
        .await
        .0,
        200
    );
    assert_eq!(bank_rows(&pool).await, [(None, Some(bank_ledger), 4)]);

    // (f) revue P1 (B-1, E-3, A-2) : une description PRÉSENTE mais faite
    // d'espaces, ou vide, vaut `null` — elle EFFACE la description posée
    // (C-e341-34 : seul un champ absent laisse inchangé). Mutation M12 :
    // « blanc traité comme absent ».
    for (blanc, version) in [("   ", 6), ("", 8)] {
        set_step(&pool, 6).await;
        assert_eq!(
            post(
                &app,
                &token,
                "bank-account",
                Some(body(json!({ "description": "Personnel" })))
            )
            .await
            .0,
            200
        );
        assert_eq!(
            bank_rows(&pool).await,
            [(
                Some("Personnel".to_string()),
                Some(bank_ledger),
                version - 1
            )],
            "avant le blanc {blanc:?}"
        );
        set_step(&pool, 6).await;
        assert_eq!(
            post(
                &app,
                &token,
                "bank-account",
                Some(body(json!({ "description": blanc })))
            )
            .await
            .0,
            200
        );
        assert_eq!(
            bank_rows(&pool).await,
            [(None, Some(bank_ledger), version)],
            "description {blanc:?} : effacée"
        );
    }
}

/// Un corps de la 0.17.5 qui **change la banque** passe par l'`UPDATE` : la
/// description et le lien, absents du corps, n'y sont pas nommés et restent
/// tels quels (AC-1, C-e341-34 ; M7 — le cas « même banque » s'arrête au no-op
/// et ne l'exerce pas).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_update_without_new_fields_keeps_them(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let bank_ledger = account_id(&pool, company_id, "1020").await;
    let full = body(json!({ "description": "Personnel", "journalAccountId": bank_ledger }));
    assert_eq!(post(&app, &token, "bank-account", Some(full)).await.0, 200);

    set_step(&pool, 6).await;
    let marker = max_audit_id(&pool).await;
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(json!({ "bankName": "UBS Suisse", "iban": IBAN, "qrIban": null })),
    )
    .await;
    assert_eq!(status, 200, "{resp}");
    let all = entries_after(&pool, marker).await;
    assert_eq!(
        actions(&all),
        ["bank_account.updated", "installation.step_completed"]
    );
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Personnel".to_string()), Some(bank_ledger), 2)]
    );
    assert_eq!(all[0].1["after"]["description"], "Personnel");
    assert_eq!(all[0].1["after"]["journal_account_id"], bank_ledger);
}

// --- AC-2 : mêmes contrôles qu'à la création ---------------------------------

/// Refus de forme et de compte, avant toute écriture, l'étape restant à 6 ;
/// message et code identiques à `POST /bank-accounts` (AC-2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_validates_like_the_creation(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let revenue = account_id(&pool, company_id, "3000").await;
    let group = account_id(&pool, company_id, "10").await;

    let long = "x".repeat(61);
    let cas: [(Value, &str, Option<&str>); 6] = [
        (
            json!({ "description": long }),
            "VALIDATION_ERROR",
            Some("La description ne peut pas dépasser 60 caractères"),
        ),
        (
            json!({ "description": "Ligne\nsuivante au milieu" }),
            "VALIDATION_ERROR",
            Some(
                "La description ne peut pas contenir de saut de ligne ni de caractère de contrôle ou de mise en page",
            ),
        ),
        (
            json!({ "journalAccountId": revenue }),
            "INVALID_ACCOUNT_TYPE",
            None,
        ),
        (
            json!({ "journalAccountId": group }),
            "ACCOUNT_NOT_POSTABLE",
            None,
        ),
        (
            json!({ "journalAccountId": 99_999_999 }),
            "ACCOUNT_NOT_FOUND",
            None,
        ),
        (
            json!({ "journalAccountId": 0 }),
            "VALIDATION_ERROR",
            Some("journalAccountId doit être strictement positif"),
        ),
    ];
    for (extra, code, message) in cas {
        let (status, resp) = post(&app, &token, "bank-account", Some(body(extra.clone()))).await;
        let attendu = if code == "ACCOUNT_NOT_FOUND" {
            404
        } else {
            400
        };
        assert_eq!(status, attendu, "{extra} : {resp}");
        assert_eq!(resp["error"]["code"], code, "{extra} : {resp}");
        if let Some(m) = message {
            assert_eq!(resp["error"]["message"], m, "{extra}");
        }
        assert_eq!(step(&pool).await, 6, "{extra}");
        assert!(bank_rows(&pool).await.is_empty(), "{extra}");
    }

    // Un compte inactif vaut inexistant (404), comme à la création.
    let other = account_id(&pool, company_id, "1010").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(other)
        .execute(&pool)
        .await
        .unwrap();
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": other }))),
    )
    .await;
    assert_eq!(status, 404, "{resp}");
    assert_eq!(resp["error"]["code"], "ACCOUNT_NOT_FOUND");
}

/// Ordre des refus (AC-4) : un refus de forme précède le refus d'étape — les
/// contrôles lisent sur le pool, avant la transaction, comme à la création.
///
/// ⚠️ Le refus d'étape est un **400** `ONBOARDING_STEP_ALREADY_COMPLETED`
/// (`errors.rs`), non un 409 comme l'écrit la fiche : le test distingue donc les
/// refus par leur **code**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_form_refusal_precedes_the_step_refusal(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    set_step(&pool, 7).await;
    let revenue = account_id(&pool, company_id, "3000").await;

    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "description": "x".repeat(61) }))),
    )
    .await;
    assert_eq!(
        (status, &resp["error"]["code"]),
        (400, &json!("VALIDATION_ERROR")),
        "{resp}"
    );
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": revenue }))),
    )
    .await;
    assert_eq!(
        (status, &resp["error"]["code"]),
        (400, &json!("INVALID_ACCOUNT_TYPE"))
    );
    // Témoin : un corps valide à l'étape 7 rend bien le refus d'étape.
    let (status, resp) = post(&app, &token, "bank-account", Some(body(json!({})))).await;
    assert_eq!(
        (status, &resp["error"]["code"]),
        (400, &json!("ONBOARDING_STEP_ALREADY_COMPLETED")),
        "{resp}"
    );
}

// --- AC-4 : pas de lien vers un futur compte de créance ----------------------

/// Les comptes de rôle `Receivable` et `Payable` sont refusés, avec le code et
/// le rendu de `POST /bank-accounts`, l'étape restant à 6, aucun compte
/// bancaire écrit (AC-4 ; M3).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_refuses_the_claim_role_accounts(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    for (number, mot) in [("1100", "débiteurs"), ("2000", "créanciers")] {
        let id = account_id(&pool, company_id, number).await;
        let marker = max_audit_id(&pool).await;
        let (status, resp) = post(
            &app,
            &token,
            "bank-account",
            Some(body(json!({ "journalAccountId": id }))),
        )
        .await;
        assert_eq!(status, 400, "{number} : {resp}");
        assert_eq!(
            resp["error"]["code"], "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
            "{number}"
        );
        let message = resp["error"]["message"].as_str().unwrap();
        assert!(
            message.contains(number) && message.contains(mot),
            "{number} : {message}"
        );
        assert_eq!(step(&pool).await, 6, "{number}");
        assert!(bank_rows(&pool).await.is_empty(), "{number}");
        assert!(entries_after(&pool, marker).await.is_empty(), "{number}");
    }
}

/// Le serveur lit le **rôle**, non le numéro (Story 14-3a ; AC-4, M3b) : le
/// compte de rôle `Receivable` renuméroté reste refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_refuses_a_renumbered_receivable(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let receivable = account_id(&pool, company_id, "1100").await;
    sqlx::query("UPDATE accounts SET number = '1105' WHERE id = ?")
        .bind(receivable)
        .execute(&pool)
        .await
        .unwrap();
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": receivable }))),
    )
    .await;
    assert_eq!(status, 400, "{resp}");
    assert_eq!(
        resp["error"]["code"],
        "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT"
    );
    assert!(resp["error"]["message"].as_str().unwrap().contains("1105"));
    assert!(bank_rows(&pool).await.is_empty());
}

/// … et un compte numéroté 1100 **sans** rôle se lie (AC-4, M3b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_accepts_an_1100_without_role(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let id_1100 = account_id(&pool, company_id, "1100").await;
    sqlx::query("UPDATE accounts SET role = NULL WHERE id = ?")
        .bind(id_1100)
        .execute(&pool)
        .await
        .unwrap();
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": id_1100 }))),
    )
    .await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(bank_rows(&pool).await, [(None, Some(id_1100), 1)]);
}

/// Revue P1 (A-3) : le compte de rôle `Payable` renuméroté reste refusé, comme
/// le `Receivable` (`bank_step_refuses_a_renumbered_receivable`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_refuses_a_renumbered_payable(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let payable = account_id(&pool, company_id, "2000").await;
    sqlx::query("UPDATE accounts SET number = '2005' WHERE id = ?")
        .bind(payable)
        .execute(&pool)
        .await
        .unwrap();
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": payable }))),
    )
    .await;
    assert_eq!(status, 400, "{resp}");
    assert_eq!(
        resp["error"]["code"],
        "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT"
    );
    let message = resp["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("2005") && message.contains("créanciers"),
        "{message}"
    );
    assert!(bank_rows(&pool).await.is_empty());
}

/// Revue P1 (A-3, mutation M11) : les rôles se lisent sur `singleton_role`, qui
/// ne vaut que pour un compte **actif**. Un ancien compte débiteurs archivé
/// garde sa colonne `role`, mais n'est plus le futur compte de créance : c'est
/// le compte actif qui porte le rôle à sa suite qui doit être refusé. Une
/// lecture sur `role` (`ORDER BY id`) trouverait l'archivé et laisserait passer
/// le compte actif.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_refuses_the_active_receivable_not_an_archived_one(pool: MySqlPool) {
    let (app, token, company_id) = at_bank_step(&pool).await;
    let old_receivable = account_id(&pool, company_id, "1100").await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(old_receivable)
        .execute(&pool)
        .await
        .unwrap();
    // 1109 vient après 1100 dans le plan : son identifiant est plus grand.
    let new_receivable = account_id(&pool, company_id, "1109").await;
    sqlx::query("UPDATE accounts SET role = 'Receivable' WHERE id = ?")
        .bind(new_receivable)
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        old_receivable < new_receivable,
        "montage : l'archivé d'abord"
    );
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "journalAccountId": new_receivable }))),
    )
    .await;
    assert_eq!(status, 400, "{resp}");
    assert_eq!(
        resp["error"]["code"],
        "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT"
    );
    assert!(bank_rows(&pool).await.is_empty());
}

/// Pool d'UNE connexion : la lecture des rôles se fait dans la transaction de
/// l'étape, jamais sur le pool (sinon elle attendrait et expirerait), et le
/// parcours finalise avec un compte lié (patron du test 13 de
/// `onboarding_audit_e2e.rs`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_with_link_on_a_single_connection_pool(pool: MySqlPool) {
    let single = MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(2))
        .connect_with(pool.connect_options().as_ref().clone())
        .await
        .unwrap();
    let (app, token, company_id) = at_bank_step_on(&pool, single).await;
    let bank_ledger = account_id(&pool, company_id, "1020").await;
    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(
            json!({ "description": "Courant", "journalAccountId": bank_ledger }),
        )),
    )
    .await;
    assert_eq!(status, 200, "{resp}");
    let (status, resp) = post(&app, &token, "finalize", None).await;
    assert_eq!(status, 200, "{resp}");
    assert_eq!(step(&pool).await, 8);
    assert_eq!(
        bank_rows(&pool).await,
        [(Some("Courant".to_string()), Some(bank_ledger), 1)]
    );
}

/// Story 34-2 (revue P2, B2-3 = E2-5 = A LOW-6) — l'étape bancaire borne le nom
/// de la banque comme la création : 256 caractères de deux octets → `400
/// VALIDATION_ERROR`, rien d'écrit ; 255 passent (la borne est en caractères).
/// Tue M12 (appel de `refuse_too_long_bank_name` retiré de l'assistant).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_step_bounds_the_bank_name_in_characters(pool: MySqlPool) {
    let (app, token, _) = at_bank_step(&pool).await;

    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "bankName": "é".repeat(256) }))),
    )
    .await;
    assert_eq!(
        (status, resp["error"]["code"].clone()),
        (400, json!("VALIDATION_ERROR")),
        "{resp}"
    );
    assert!(bank_rows(&pool).await.is_empty(), "rien d'écrit");

    let (status, resp) = post(
        &app,
        &token,
        "bank-account",
        Some(body(json!({ "bankName": "é".repeat(255) }))),
    )
    .await;
    assert_eq!(status, 200, "255 caractères, à la borne : {resp}");
    assert_eq!(bank_rows(&pool).await.len(), 1);
}
