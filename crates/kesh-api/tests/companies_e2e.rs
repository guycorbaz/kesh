//! Tests E2E pour GET /api/v1/companies/current (Story 2.4).

mod common;

use std::sync::Arc;

use chrono::TimeDelta;
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use serde_json::json;
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

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn companies_current_returns_company(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["company"]["name"], "Test Company");
    assert!(body["bankAccounts"].is_array());
    assert_eq!(body["bankAccounts"].as_array().unwrap().len(), 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn companies_current_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

/// **AC8 — l'ALLER-RETOUR complet** : écrire par la route, relire par `GET`.
///
/// ⚠️ **C'est le seul test qui voit un DTO oublié.** `CompanyJson` est un
/// miroir **écrit à la main** — struct Rust, son `impl From<Company>`, et
/// l'interface TypeScript — qu'aucun compilateur ne vérifie contre l'entité.
/// Sans ce test, oublier le `From` laisse la valeur **stockée en base**,
/// **rendue sur le PDF**, et **invisible dans l'écran de réglages** : tous les
/// gates passent au vert, et le défaut ne se voit qu'à l'usage.
///
/// L'assertion porte donc sur le **corps HTTP relu**, pas sur la réponse du
/// `PUT` ni sur la base — c'est la seule chose que le frontend consomme.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn contact_details_survive_the_round_trip_to_the_settings_screen(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let current: serde_json::Value = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let version = current["company"]["version"].as_i64().unwrap();
    assert!(
        current["company"]["phone"].is_null(),
        "montage : la société ne doit porter aucune coordonnée au départ, \
         sinon le test ne mesure pas l'écriture"
    );

    let resp = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "phone": "+41 21 123 45 67",
            "website": "https://demo.ch",
            "version": version,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "l'écriture doit réussir");

    // Le GET est ce que consomme l'écran de réglages — c'est LUI qui fait foi.
    let relu: serde_json::Value = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(
        relu["company"]["phone"], "+41 21 123 45 67",
        "le téléphone doit revenir dans le GET — s'il est absent, le DTO \
         `CompanyJson` ou son `impl From<Company>` a été oublié, et l'écran de \
         réglages affichera « — » pour toujours"
    );
    assert_eq!(
        relu["company"]["website"], "https://demo.ch",
        "le site web doit revenir dans le GET — même piège que ci-dessus"
    );
}

/// Le champ vidé **efface** la valeur : la ligne disparaît du PDF (D2).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn empty_contact_details_clear_the_stored_values(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let v0 = app
        .client
        .get(app.url("/api/v1/companies/current"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap()["company"]["version"]
        .as_i64()
        .unwrap();

    let posed: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "phone": "+41 21 123 45 67", "website": null, "version": v0 }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(posed["phone"], "+41 21 123 45 67", "montage : valeur posée");

    let cleared: serde_json::Value = app
        .client
        .put(app.url("/api/v1/companies/current/contact-details"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "phone": "   ",
            "website": null,
            "version": posed["version"].as_i64().unwrap(),
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(
        cleared["phone"].is_null(),
        "un champ vide (ou blanc) doit EFFACER la valeur, pas la conserver \
         ni stocker une chaîne vide : la ligne doit disparaître du PDF"
    );
}
