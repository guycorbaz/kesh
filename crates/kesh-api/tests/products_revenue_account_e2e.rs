//! End-to-end HTTP — Story 16-2a (#144) : compte de produit sur la fiche
//! produit du catalogue.
//!
//! **Ces tests vivent au niveau ROUTE, et c'est structurant** : la validation
//! **D3** et sa condition **D4** habitent `routes/products.rs`, pas le
//! repository. Un test de repository ne les exercerait pas — et la variante
//! d'erreur **D10** (`PRODUCT_REVENUE_ACCOUNT_INVALID`) n'existe qu'au rendu
//! HTTP.
//!
//! Couvre :
//! - **D3**, les trois critères de rejet : compte inconnu / d'une autre société
//!   (indiscernables, garde anti-IDOR), compte archivé, compte non-`Revenue` ;
//! - **D10** : le corps porte le **code** `PRODUCT_REVENUE_ACCOUNT_INVALID` et
//!   `details.reason`, et le message désigne **l'article** — jamais le réglage
//!   société ;
//! - **D4, les DEUX sens** : compte inchangé devenu invalide → l'édition passe ;
//!   compte changé vers une valeur invalide → 400. La seconde direction est
//!   indispensable : un prédicat simplement **inversé** passerait le premier
//!   test ;
//! - le retrait d'un compte posé (`Some → null`), qui est un changement au sens
//!   de D4 mais n'a **rien à valider** ;
//! - la non-régression du contrat HTTP : clé absente et `null` explicite valent
//!   tous deux `NULL`, jamais 400.
//!
//! Harnais calqué sur `contact_payment_terms_e2e.rs`.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{users, vat_rates};
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-password".to_string(),
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
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");
    let body: serde_json::Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
}

async fn create_company_user(pool: &MySqlPool, company_id: i64, username: &str, password: &str) {
    let hash = hash_password(password).expect("hash should succeed");
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");
}

/// Comptes seedés d'une société de test, par numéro.
struct SeededCompany {
    id: i64,
    accounts: std::collections::HashMap<&'static str, i64>,
}

/// Société avec exercice, plan minimal et réglages de facturation.
///
/// Le plan porte délibérément **un compte de chaque nature utile** : `3000`
/// (`Revenue`, valide), `4000` (`Expense`, pour le rejet « pas un compte de
/// produit ») et `3999` (`Revenue` mais **archivé**, pour le rejet « archivé »).
async fn create_seeded_company(pool: &MySqlPool, name: &str) -> SeededCompany {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .bind(name)
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("fiscal_year insert");

    let mut accounts = std::collections::HashMap::new();
    for (code, acc_name, account_type, active) in &[
        ("1100", "Créances", "Asset", true),
        ("3000", "Ventes", "Revenue", true),
        ("4000", "Charges", "Expense", true),
        ("3999", "Ventes archivées", "Revenue", false),
    ] {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type, active) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(acc_name)
        .bind(account_type)
        .bind(active)
        .execute(pool)
        .await
        .expect("account insert");
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, default_sales_journal) \
         VALUES (?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .execute(pool)
    .await
    .expect("company_invoice_settings insert");

    vat_rates::seed_default_swiss_rates(pool, company_id)
        .await
        .expect("vat_rates seed");

    SeededCompany {
        id: company_id,
        accounts,
    }
}

/// Corps minimal d'une fiche produit, avec ou sans compte.
fn product_body(name: &str, account: Option<i64>) -> serde_json::Value {
    let mut body = json!({
        "name": name,
        "unitPrice": "100.00",
        "vatRate": "8.10",
    });
    if let Some(id) = account {
        body["defaultRevenueAccountId"] = json!(id);
    }
    body
}

/// Archive un compte directement en base — le geste que l'utilisateur ferait
/// depuis le plan comptable, sans passer par le catalogue.
async fn archive_account(pool: &MySqlPool, account_id: i64) {
    sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("archive account");
}

// ===========================================================================
// D3 — les trois critères de rejet, au niveau ROUTE, avec le code de D10
// ===========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_rejects_inactive_account(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "Rejet archivé").await;
    create_company_user(&pool, co.id, "u_inactive", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_inactive", "e2e-test-password").await;

    let resp = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body(
            "Article compte archivé",
            Some(co.accounts["3999"]),
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400, "un compte archivé doit être rejeté");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "PRODUCT_REVENUE_ACCOUNT_INVALID",
        "le CODE doit être celui de D10, pas le VALIDATION_ERROR générique — \
         asserter le seul statut laisserait passer la régression"
    );
    assert_eq!(body["error"]["details"]["reason"], "INACTIVE");

    // Le sujet désigne l'ARTICLE : réutiliser le formateur de 16-1a ferait lire
    // « le compte de produit par défaut de la société », qui envoie corriger un
    // réglage que l'utilisateur n'a pas touché.
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains("article"),
        "le message doit désigner l'article, or il dit : {msg}"
    );
    assert!(
        !msg.contains("par défaut de la société"),
        "le message ne doit PAS désigner le réglage société, or il dit : {msg}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_rejects_non_revenue_account(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "Rejet type").await;
    create_company_user(&pool, co.id, "u_type", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_type", "e2e-test-password").await;

    let resp = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body(
            "Article compte de charge",
            Some(co.accounts["4000"]),
        ))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "PRODUCT_REVENUE_ACCOUNT_INVALID");
    assert_eq!(body["error"]["details"]["reason"], "NOT_REVENUE");
}

/// Scoping multi-tenant : le compte d'une AUTRE société est rejeté, et rendu
/// **indiscernable** d'un compte inexistant — c'est la garde anti-IDOR de D3.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_rejects_cross_company_account_like_unknown(pool: MySqlPool) {
    let mine = create_seeded_company(&pool, "Ma société").await;
    let other = create_seeded_company(&pool, "Autre société").await;
    create_company_user(&pool, mine.id, "u_tenant", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_tenant", "e2e-test-password").await;

    // Compte d'une autre société : existe, mais pas pour moi.
    let cross = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body(
            "Article cross-tenant",
            Some(other.accounts["3000"]),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(cross.status(), 400);
    let cross_body: serde_json::Value = cross.json().await.unwrap();

    // Compte franchement inexistant.
    let unknown = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body("Article compte inconnu", Some(999_999_999)))
        .send()
        .await
        .unwrap();
    assert_eq!(unknown.status(), 400);
    let unknown_body: serde_json::Value = unknown.json().await.unwrap();

    assert_eq!(
        cross_body["error"], unknown_body["error"],
        "un compte d'une autre société doit être INDISCERNABLE d'un compte \
         inexistant — toute différence révélerait l'existence d'un id"
    );
    assert_eq!(
        cross_body["error"]["details"]["reason"],
        "UNKNOWN_OR_CROSS_COMPANY"
    );
}

// ===========================================================================
// D4 — les DEUX sens de la condition
// ===========================================================================

/// **Sens 1** — le compte n'a PAS changé et est devenu invalide : l'édition
/// passe.
///
/// Sans D4, renommer un article deviendrait impossible dès qu'un compte a été
/// archivé ailleurs — **et sans issue**, le compte archivé étant absent des
/// propositions du sélecteur : l'utilisateur ne pourrait ni conserver, ni
/// remplacer la valeur.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn renaming_succeeds_when_unchanged_account_became_invalid(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "D4 sens 1").await;
    create_company_user(&pool, co.id, "u_d4a", "e2e-test-password").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "u_d4a", "e2e-test-password").await;

    let created: serde_json::Value = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body(
            "Article à renommer",
            Some(co.accounts["3000"]),
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_i64().unwrap();

    // Le compte est archivé APRÈS coup, depuis le plan comptable.
    archive_account(&pool, co.accounts["3000"]).await;

    // On ne touche qu'au nom, en renvoyant le compte À L'IDENTIQUE.
    let mut body = product_body("Article renommé", Some(co.accounts["3000"]));
    body["version"] = created["version"].clone();
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/products/{id}")))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "le compte n'a pas changé : D4 doit exempter la validation, sinon \
         l'article devient inéditable et sans issue"
    );
    let updated: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(updated["name"], "Article renommé");
    assert_eq!(updated["defaultRevenueAccountId"], co.accounts["3000"]);
}

/// **Sens 2** — le compte CHANGE vers une valeur invalide : 400.
///
/// Indispensable en plus du sens 1 : la mutation qui retire la condition D4
/// rend la validation *inconditionnelle* et ne dit rien d'un prédicat
/// simplement **inversé**, qui passerait le premier test.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn changing_account_to_invalid_is_rejected(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "D4 sens 2").await;
    create_company_user(&pool, co.id, "u_d4b", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_d4b", "e2e-test-password").await;

    let created: serde_json::Value = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body(
            "Article compte valide",
            Some(co.accounts["3000"]),
        ))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_i64().unwrap();

    // Le compte CHANGE, vers un compte de charge.
    let mut body = product_body("Article compte valide", Some(co.accounts["4000"]));
    body["version"] = created["version"].clone();
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/products/{id}")))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "changer vers un compte invalide doit être rejeté"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "PRODUCT_REVENUE_ACCOUNT_INVALID");
    assert_eq!(body["error"]["details"]["reason"], "NOT_REVENUE");
}

/// Retirer un compte déjà posé (`Some → null`) est un **changement** au sens de
/// D4, mais il n'y a **rien à valider**.
///
/// Sans la garde `None`, une implémentation qui valide dès que « le compte
/// change » rendrait un compte **impossible à retirer une fois posé**.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn clearing_an_existing_account_succeeds(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "Retrait compte").await;
    create_company_user(&pool, co.id, "u_clear", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_clear", "e2e-test-password").await;

    let created: serde_json::Value = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body("Article à vider", Some(co.accounts["3000"])))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let id = created["id"].as_i64().unwrap();
    assert_eq!(created["defaultRevenueAccountId"], co.accounts["3000"]);

    let mut body = product_body("Article à vider", None);
    body["defaultRevenueAccountId"] = serde_json::Value::Null;
    body["version"] = created["version"].clone();
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/products/{id}")))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "retirer un compte ne valide rien — sinon il devient impossible à retirer"
    );
    let updated: serde_json::Value = resp.json().await.unwrap();
    assert!(updated["defaultRevenueAccountId"].is_null());
}

// ===========================================================================
// Non-régression du contrat HTTP
// ===========================================================================

/// Clé **absente** et `null` **explicite** valent tous deux `NULL`, jamais 400.
///
/// C'est ce que garantit `#[serde(default)]` : sans lui, un `Option<T>` reste
/// obligatoire en serde et l'omission casserait toute intégration existante.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn absent_key_and_explicit_null_both_mean_no_account(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "Contrat HTTP").await;
    create_company_user(&pool, co.id, "u_http", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_http", "e2e-test-password").await;

    // 1. Clé absente — le corps d'un client qui ignore le nouveau champ.
    let absent = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body("Article sans la clé", None))
        .send()
        .await
        .unwrap();
    assert_eq!(
        absent.status(),
        201,
        "la clé absente ne doit jamais rendre 400"
    );
    let absent_body: serde_json::Value = absent.json().await.unwrap();
    assert!(
        absent_body["defaultRevenueAccountId"].is_null(),
        "le champ est TOUJOURS restitué, à null — jamais omis de la réponse"
    );

    // 2. `null` explicite.
    let mut body = product_body("Article null explicite", None);
    body["defaultRevenueAccountId"] = serde_json::Value::Null;
    let explicit = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(
        explicit.status(),
        201,
        "`null` explicite ne doit jamais rendre 400"
    );
    let explicit_body: serde_json::Value = explicit.json().await.unwrap();
    assert!(explicit_body["defaultRevenueAccountId"].is_null());
}

/// Le chemin nominal : créer avec un compte valide, le relire.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn valid_account_is_accepted_and_returned(pool: MySqlPool) {
    let co = create_seeded_company(&pool, "Nominal").await;
    create_company_user(&pool, co.id, "u_ok", "e2e-test-password").await;
    let app = spawn_app(pool).await;
    let token = login(&app, "u_ok", "e2e-test-password").await;

    let resp = app
        .client
        .post(app.url("/api/v1/products"))
        .bearer_auth(&token)
        .json(&product_body("Article nominal", Some(co.accounts["3000"])))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let created: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(created["defaultRevenueAccountId"], co.accounts["3000"]);

    let id = created["id"].as_i64().unwrap();
    let read: serde_json::Value = app
        .client
        .get(app.url(&format!("/api/v1/products/{id}")))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(read["defaultRevenueAccountId"], co.accounts["3000"]);
}
