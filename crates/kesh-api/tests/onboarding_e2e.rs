//! Tests d'intégration E2E des endpoints onboarding (story 2.2).

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
        .expect("login request");
    let body: serde_json::Value = resp.json().await.expect("login json");
    body["accessToken"]
        .as_str()
        .expect("access token")
        .to_string()
}

fn auth(token: &str) -> String {
    format!("Bearer {token}")
}

// --- Tests ---

/// ⛔ **Un COMPTABLE authentifié ne peut pas réinitialiser — Story 25-1a (#377).**
///
/// La route vivait dans `authenticated_routes`, donc atteignable par tout rôle
/// authentifié ; ses seules gardes étaient des **états** (`step_completed`,
/// `is_demo`, un drapeau d'environnement). *Un état se contourne en amenant le
/// système dans l'état voulu ; un droit, non.*
///
/// ⚠️ **Ce test positif est nécessaire, et la mutation ne le remplace pas.** Le
/// dépôt documente qu'une route chaînée **après** le `route_layer` compile, ne
/// panique pas, et échappe aux deux couches (`lib.rs`, « TOUTE ROUTE S'AJOUTE
/// AU-DESSUS DE CETTE LIGNE »). Neutraliser une garde fait rougir un test ;
/// **une garde jamais posée ne fait rougir personne**. Seul un appel réel avec un
/// rôle insuffisant le voit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_by_comptable_is_forbidden(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // Un comptable actif. (Le preset d'onboarding part d'une installation
    // fraîche : aucune société n'existe encore, d'où la création explicite.)
    create_test_company(&pool).await;
    let company_id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM companies")
        .fetch_one(&pool)
        .await
        .expect("company de test");
    sqlx::query(
        "INSERT INTO users (company_id, username, password_hash, role, active) \
         VALUES (?, 'compt_reset', ?, 'Comptable', TRUE)",
    )
    .bind(company_id)
    .bind(kesh_api::auth::password::hash_password(TEST_ADMIN_PASSWORD).expect("hash"))
    .execute(&pool)
    .await
    .expect("insert comptable");

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "compt_reset", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .expect("login comptable");
    assert_eq!(resp.status(), 200, "le comptable doit pouvoir se connecter");
    let body: serde_json::Value = resp.json().await.expect("login json");
    let token = body["accessToken"].as_str().expect("token").to_string();

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .expect("reset request");

    assert_eq!(
        resp.status(),
        403,
        "un Comptable authentifié doit être refusé par le RBAC du bloc admin"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_state_returns_initial_state(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0);
    assert_eq!(body["isDemo"], false);
    assert!(body["uiMode"].is_null());
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_state_requires_auth(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_language_advances_to_step_1(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 1);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_language_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "XX" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_language_twice_returns_step_already_completed(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // First call succeeds
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // Second call fails
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "DE" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn set_mode_invalid_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // First set language
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();

    // Invalid mode
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "invalid" }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn full_onboarding_flow_demo_path(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Step 0 → 1: language
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Step 1 → 2: mode
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 2);
    assert_eq!(body["uiMode"], "guided");

    // Step 2 → 3: seed demo
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isDemo"], true);
    assert_eq!(body["uiMode"], "guided");

    // Verify state persisted
    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isDemo"], true);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_clears_demo_data(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Complete onboarding
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "expert" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    // Reset — after seed_demo the onboarding is at step 3, which Pass 6 P4 + P6-L8
    // requires KESH_PRODUCTION_RESET=1 to allow (demo deployment opt-in).
    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
    unsafe {
        std::env::set_var("KESH_PRODUCTION_RESET", "1");
    }

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    unsafe {
        match prev {
            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
        }
    }

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0);
    assert_eq!(body["isDemo"], false);
    assert!(body["uiMode"].is_null());
}

/// P8 / H4 — even if `is_demo = true` (potentially corrupted flag), `reset()` must
/// refuse to wipe a finalized onboarding (step >= 7). The secondary step gate is
/// the security floor; the is_demo flag alone is not sufficient.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_blocks_step_7_even_when_is_demo_flag_is_true(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Initialize onboarding_state, then forge a corrupted state: is_demo=true + step=7
    // simulates either DB tampering or a bug that flipped is_demo on a finalized tenant.
    sqlx::query("INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) VALUES (TRUE, 7, TRUE, 'guided', 1)")
        .execute(&pool)
        .await
        .expect("seed onboarding_state row");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "reset() must refuse step >= 7 regardless of is_demo"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");

    // Verify the state was NOT wiped
    let row: (i32, bool) = sqlx::query_as(
        "SELECT step_completed, is_demo FROM onboarding_state WHERE singleton = TRUE",
    )
    .fetch_one(&pool)
    .await
    .expect("read onboarding_state");
    assert_eq!(row.0, 7, "step must remain 7 after refused reset");
    assert!(row.1, "is_demo must remain true after refused reset");
}

/// P6-L5 — Positive path: when KESH_PRODUCTION_RESET=1 and is_demo=true at step > 2,
/// reset() must succeed. Pairs with `reset_blocks_production_past_step_2` and
/// `reset_blocks_step_7_even_when_is_demo_flag_is_true` to cover the gate matrix.
///
/// Note: this test mutates a process-level env var. It uses `serial_test::serial`
/// or a similar guard if available; here we set + unset around the request and
/// rely on the test harness running tests sequentially (cargo test --test-threads=1
/// per `.cargo/config.toml`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_allows_demo_at_step_5_when_env_var_set(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Demo user mid-flow (step 5 = post-org-type, post-coordinates).
    sqlx::query(
        "INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) \
         VALUES (TRUE, 5, TRUE, 'guided', 1)",
    )
    .execute(&pool)
    .await
    .expect("seed onboarding_state row");

    // SAFETY: process-wide env mutation. Tests run with --test-threads=1 (see
    // .cargo/config.toml RUST_TEST_THREADS=2 — but reset env var read happens
    // inside the request handler in this same process). We restore the previous
    // value after the request to avoid leaking into subsequent tests.
    let prev = std::env::var("KESH_PRODUCTION_RESET").ok();
    // SAFETY (Rust 2024): set_var/remove_var are unsafe due to potential races
    // with other threads reading env. Acceptable inside a serialized test.
    unsafe {
        std::env::set_var("KESH_PRODUCTION_RESET", "true");
    }

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    // Restore env var BEFORE assertions so a panic doesn't leak state.
    unsafe {
        match prev {
            Some(v) => std::env::set_var("KESH_PRODUCTION_RESET", v),
            None => std::env::remove_var("KESH_PRODUCTION_RESET"),
        }
    }

    assert_eq!(
        resp.status(),
        200,
        "reset() with KESH_PRODUCTION_RESET=true must succeed for is_demo=true at step 5"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 0, "state should be reset to step 0");
    assert_eq!(body["isDemo"], false, "is_demo cleared after reset");
}

/// P8 / P4 — production path (is_demo=false) past step 2 must always be blocked,
/// regardless of KESH_PRODUCTION_RESET. P6-L8: distinct ONBOARDING_RESET_FORBIDDEN
/// error code (403) so the client can distinguish policy-refusal from a finalized
/// onboarding (which uses ONBOARDING_STEP_ALREADY_COMPLETED at 400).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reset_blocks_production_past_step_2(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    sqlx::query("INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode, version) VALUES (TRUE, 5, FALSE, 'expert', 1)")
        .execute(&pool)
        .await
        .expect("seed onboarding_state row");

    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/reset"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_RESET_FORBIDDEN");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn seed_demo_at_wrong_step_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Try seed-demo at step 0 (should require step 2)
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_STEP_ALREADY_COMPLETED");
}

/// Story v011-2 (Issue #120, AC #10bis) — fresh install : pas de `create_test_company`,
/// le bootstrap crée la company stub (`is_stub=TRUE`). Le path demo considère la
/// company comme configurée : `seed_demo` repasse `is_stub=FALSE`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fresh_install_stub_cleared_by_seed_demo(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    // PAS de create_test_company : le bootstrap crée la company stub.
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;

    // Sanity : isStub true au départ.
    let resp = app
        .client
        .get(app.url("/api/v1/onboarding/state"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["isStub"], true);

    // Avance à step 2 (langue + mode).
    app.client
        .post(app.url("/api/v1/onboarding/language"))
        .header("Authorization", auth(&token))
        .json(&json!({ "language": "FR" }))
        .send()
        .await
        .unwrap();
    app.client
        .post(app.url("/api/v1/onboarding/mode"))
        .header("Authorization", auth(&token))
        .json(&json!({ "mode": "guided" }))
        .send()
        .await
        .unwrap();

    // seed-demo → step 3, isStub doit retomber à false.
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["stepCompleted"], 3);
    assert_eq!(body["isStub"], false);
}

// --- Story 31-5 (#707) : la démonstration valide une facture avec TVA ---

/// Mène le chemin démonstration jusqu'à `seed-demo` (étapes 0 → 3) et rend l'id
/// de la société démo.
async fn reach_demo(app: &TestApp, pool: &MySqlPool, token: &str) -> i64 {
    for (path, body) in [
        ("/api/v1/onboarding/language", json!({ "language": "FR" })),
        ("/api/v1/onboarding/mode", json!({ "mode": "guided" })),
    ] {
        let resp = app
            .client
            .post(app.url(path))
            .header("Authorization", auth(token))
            .json(&body)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 200, "{path}");
    }
    let resp = app
        .client
        .post(app.url("/api/v1/onboarding/seed-demo"))
        .header("Authorization", auth(token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "seed-demo");
    sqlx::query_scalar::<_, i64>("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le compte qui porte un rôle singleton dans la société, relu en base (jamais
/// un numéro codé en dur : l'utilisateur peut renuméroter).
async fn role_account(pool: &MySqlPool, company_id: i64, role: &str) -> i64 {
    sqlx::query_scalar::<_, i64>(
        "SELECT id FROM accounts WHERE company_id = ? AND singleton_role = ?",
    )
    .bind(company_id)
    .bind(role)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|e| panic!("compte du rôle {role} : {e}"))
}

/// AC2 (#707) — après `seed-demo`, une facture d'une ligne à 8,1 % se **valide**
/// (`200`), et l'écriture produite crédite du montant de TVA le compte du rôle
/// `VatPayable`. Rouge avant le correctif : `400 CONFIGURATION_REQUIRED`
/// (aucun compte de TVA désigné par la démonstration).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn demo_validates_an_invoice_with_vat(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    let company_id = reach_demo(&app, &pool, &token).await;

    let resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "contactType": "Entreprise",
            "name": "Client Démo Synthétique SA",
            "isClient": true,
            "isSupplier": false,
            "email": "client@example.invalid",
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "contact");
    let contact_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // Date dans l'exercice que crée la démonstration (année courante).
    let today = chrono::Utc::now().date_naive().to_string();
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", auth(&token))
        .json(&json!({
            "contactId": contact_id,
            "date": today,
            "lines": [
                {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
            ]
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "facture");
    let invoice_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{invoice_id}/validate")))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body = resp.text().await.unwrap();
    assert_eq!(status, 200, "validation de la facture démo : {body}");

    let vat_payable = role_account(&pool, company_id, "VatPayable").await;
    let credit: Option<rust_decimal::Decimal> = sqlx::query_scalar(
        "SELECT jel.credit FROM journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.id = ? AND jel.account_id = ?",
    )
    .bind(invoice_id)
    .bind(vat_payable)
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(
        credit,
        Some(rust_decimal::Decimal::new(81000, 4)),
        "l'écriture crédite le compte du rôle VatPayable de 8.10"
    );
}

/// AC2 (#707) — après `seed-demo`, `GET /company/invoice-settings` porte les
/// **trois** comptes de TVA : chacun égal au compte qui porte son rôle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn demo_designates_the_three_vat_accounts(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    create_test_company(&pool).await;
    ensure_admin_user(&pool, &test_config()).await.unwrap();
    let token = login(&app).await;
    let company_id = reach_demo(&app, &pool, &token).await;

    let resp = app
        .client
        .get(app.url("/api/v1/company/invoice-settings"))
        .header("Authorization", auth(&token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    let payable = role_account(&pool, company_id, "VatPayable").await;
    let recoverable = role_account(&pool, company_id, "VatRecoverable").await;
    let settlement = role_account(&pool, company_id, "VatSettlement").await;
    assert_eq!(
        body["defaultVatPayableAccountId"].as_i64(),
        Some(payable),
        "TVA due"
    );
    assert_eq!(
        body["defaultVatRecoverableAccountId"].as_i64(),
        Some(recoverable),
        "impôt préalable"
    );
    assert_eq!(
        body["defaultVatDecompteAccountId"].as_i64(),
        Some(settlement),
        "décompte TVA"
    );
}
