//! Tests E2E pour PUT /api/v1/profile/mode (Story 2.5).

mod common;

use std::sync::Arc;

use chrono::TimeDelta;
use common::{audit_actions, audit_details, create_test_company};
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::repositories::onboarding;
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_mode_updates_onboarding_state(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;

    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Init onboarding state
    onboarding::init_state(&pool).await.unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/profile/mode"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "mode": "expert" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 204);

    // Verify persisted
    let state = onboarding::get_state(&pool).await.unwrap().unwrap();
    assert_eq!(state.ui_mode.unwrap().as_str(), "expert");

    // Story 25-1b (AC 5) — la trace porte sur l'INSTALLATION, pas sur
    // l'utilisateur : `ui_mode` vit dans `onboarding_state`, table mono-ligne et
    // GLOBALE, sans `company_id`. Un utilisateur bascule l'installation entière
    // en mode Expert, ce qui ouvre à tous l'écriture directe au journal.
    // ⚠️ Écrire `entity_type = "user"` ici serait mentir sur la portée — et c'est
    // précisément ce que ce test empêche de faire par mégarde.
    assert_eq!(
        audit_actions(&pool, "installation", 0).await,
        vec!["installation.ui_mode_changed".to_string()],
        "entity_type = installation, entity_id = AUDIT_ENTITY_ID_NONE"
    );
    assert!(
        audit_actions(&pool, "user", 0).await.is_empty(),
        "rien ne doit être écrit sous entity_type = user"
    );

    let details = audit_details(&pool, "installation", 0, "installation.ui_mode_changed")
        .await
        .expect("détails présents");
    assert_eq!(details["after"], "expert");
}

/// Story 25-1b (AC 5) — cette route trace MÊME quand rien ne change, et c'est
/// assumé : `update_step` incrémente `version` inconditionnellement, il n'y a
/// aucun court-circuit no-op à garder. *Une décision subie devient un défaut ;
/// écrite — et testée — elle reste une décision.*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_mode_traces_even_when_unchanged(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    onboarding::init_state(&pool).await.unwrap();

    for _ in 0..2 {
        let resp = app
            .client
            .put(app.url("/api/v1/profile/mode"))
            .header("Authorization", format!("Bearer {token}"))
            .json(&json!({ "mode": "expert" }))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }

    // ⚠️ On compte les traces de l'INSTALLATION, pas toutes celles de la base :
    // `ensure_admin_user` écrit désormais la sienne (`user.created` par le chemin
    // d'amorçage, AC 6). Un compteur global aurait ici mesuré autre chose que ce
    // qu'il prétend — et le test aurait rougi pour une raison étrangère à son objet.
    assert_eq!(
        audit_actions(&pool, "installation", 0).await.len(),
        2,
        "deux appels identiques, deux traces — l'égalité se lit dans details_json"
    );
    let details = audit_details(&pool, "installation", 0, "installation.ui_mode_changed")
        .await
        .expect("détails présents");
    assert_eq!(
        details["before"], details["after"],
        "la seconde trace porte before == after, et le dit"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_mode_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;

    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    onboarding::init_state(&pool).await.unwrap();

    let resp = app
        .client
        .put(app.url("/api/v1/profile/mode"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "mode": "invalid" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_mode_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .put(app.url("/api/v1/profile/mode"))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}
