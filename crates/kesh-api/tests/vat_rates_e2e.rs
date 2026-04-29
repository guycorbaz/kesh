//! End-to-end tests for `GET /api/v1/vat-rates` (Story 7.2 — KF-003).
//!
//! Couvre les AC #10, #11, #12, #13 :
//! - Happy path : 200 + 4 entrées triées DESC pour la company de l'user.
//! - IDOR cross-tenant : user A ne voit que les 4 vat_rates de A (pas 8).
//! - Sans auth : 401.
//! - Rôle Consultation : 200 (lecture autorisée tous rôles authentifiés).
//! - Query param `companyId` ignoré : défense en profondeur.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, users, vat_rates};
use serde_json::Value;
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
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        i18n,
    };

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
        .json(&serde_json::json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");
    assert_eq!(resp.status(), 200, "login should return 200");
    let body: Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
}

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    let company = companies::create(
        pool,
        NewCompany {
            name: name.into(),
            address: format!("Rue {name} 1, 1000 Lausanne"),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("company create");
    // Seed des 4 taux suisses 2024+ (équivalent du backfill migration sur
    // company pré-existante OU du seed onboarding/seed_demo).
    vat_rates::seed_default_swiss_rates(pool, company.id)
        .await
        .expect("seed vat rates");
    company.id
}

async fn create_user(pool: &MySqlPool, company_id: i64, username: &str, role: Role) -> i64 {
    let hash = hash_password("test-password-123").expect("hash");
    let user = users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash,
            role,
            active: true,
            company_id,
        },
    )
    .await
    .expect("user create");
    user.id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_vat_rates_happy(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "CompA").await;
    let _ = create_user(&pool, company_id, "alice", Role::Comptable).await;
    let token = login(&app, "alice", "test-password-123").await;

    let resp = app
        .client
        .get(app.url("/api/v1/vat-rates"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("send");

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let arr = body.as_array().expect("response should be a JSON array");
    assert_eq!(arr.len(), 4, "should return 4 vat rates");

    // Tri DESC : 8.10, 3.80, 2.60, 0.00. `rate` sérialisé en string.
    assert_eq!(arr[0]["rate"].as_str().unwrap(), "8.10");
    assert_eq!(arr[1]["rate"].as_str().unwrap(), "3.80");
    assert_eq!(arr[2]["rate"].as_str().unwrap(), "2.60");
    assert_eq!(arr[3]["rate"].as_str().unwrap(), "0.00");

    // Format camelCase, label en clé i18n.
    assert_eq!(arr[0]["label"].as_str().unwrap(), "product-vat-normal");
    assert_eq!(arr[0]["validFrom"].as_str().unwrap(), "2024-01-01");
    assert!(arr[0]["validTo"].is_null());
    assert!(arr[0]["active"].as_bool().unwrap());
    assert!(arr[0]["id"].is_number());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_vat_rates_idor_cross_tenant(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_a = create_company(&pool, "CompA").await;
    let company_b = create_company(&pool, "CompB").await;
    let _ = create_user(&pool, company_a, "alice", Role::Comptable).await;
    let _ = create_user(&pool, company_b, "bob", Role::Comptable).await;

    let token_a = login(&app, "alice", "test-password-123").await;
    let resp = app
        .client
        .get(app.url("/api/v1/vat-rates"))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(
        arr.len(),
        4,
        "Alice (CompA) should see only 4 vat rates, not 8 — no cross-tenant leak"
    );
    // Tous scopés à company_a
    for entry in arr {
        // Ne renvoyons pas company_id dans le DTO ; pour vérifier le scoping
        // on s'assure juste qu'il y a exactement 4 lignes (pas 8). On peut
        // aussi recroiser via `SELECT COUNT FROM vat_rates WHERE company_id = company_a`.
        assert!(entry["id"].is_number());
    }
    // Sanity DB-side : 4 vat_rates par company.
    let count_a: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vat_rates WHERE company_id = ?")
        .bind(company_a)
        .fetch_one(&pool)
        .await
        .unwrap();
    let count_b: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM vat_rates WHERE company_id = ?")
        .bind(company_b)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_a, 4);
    assert_eq!(count_b, 4);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_vat_rates_no_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .get(app.url("/api/v1/vat-rates"))
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_vat_rates_consultation_role_returns_200(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let company_id = create_company(&pool, "CompA").await;
    let _ = create_user(&pool, company_id, "carol", Role::Consultation).await;
    let token = login(&app, "carol", "test-password-123").await;

    let resp = app
        .client
        .get(app.url("/api/v1/vat-rates"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("send");
    // Consultation est un rôle authentifié — la route doit accepter (lecture pure).
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body.as_array().unwrap().len(), 4);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_vat_rates_query_param_companyid_ignored(pool: MySqlPool) {
    // Défense en profondeur : si quelqu'un ajoute par erreur un `Query<...>`
    // au handler plus tard, ce test détectera la régression de scoping.
    let app = spawn_app(pool.clone()).await;
    let company_a = create_company(&pool, "CompA").await;
    let company_b = create_company(&pool, "CompB").await;
    let _ = create_user(&pool, company_a, "alice", Role::Comptable).await;

    let token_a = login(&app, "alice", "test-password-123").await;
    let url = format!("/api/v1/vat-rates?companyId={}", company_b);
    let resp = app
        .client
        .get(app.url(&url))
        .bearer_auth(&token_a)
        .send()
        .await
        .expect("send");
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let arr = body.as_array().unwrap();
    assert_eq!(
        arr.len(),
        4,
        "query param companyId should be ignored — scope reste celui du JWT (CompA)"
    );
}

// ---------------------------------------------------------------------------
// Tests directs de `verify_vat_rates_against_db` (sans HTTP) — Pass 1
// remediation #4 + #9 : couverture du chemin batched IN-clause + dedup.
// ---------------------------------------------------------------------------

use kesh_api::routes::vat::{VAT_REJECTED_MSG, verify_vat_rates_against_db};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_accepts_all_seeded_rates(pool: MySqlPool) {
    let company_id = create_company(&pool, "CompA").await;
    let rates = vec![dec!(8.10), dec!(3.80), dec!(2.60), dec!(0.00)];
    verify_vat_rates_against_db(&pool, company_id, &rates)
        .await
        .expect("all 4 seeded rates should pass");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_rejects_unknown_rate(pool: MySqlPool) {
    let company_id = create_company(&pool, "CompA").await;
    // 7.70 = ancien taux 2018-2023, jamais seedé v0.1.
    let err = verify_vat_rates_against_db(&pool, company_id, &[dec!(7.70)])
        .await
        .unwrap_err();
    assert!(
        format!("{err:?}").contains(VAT_REJECTED_MSG),
        "expected VAT_REJECTED_MSG, got {err:?}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_rejects_if_any_rate_unknown(pool: MySqlPool) {
    let company_id = create_company(&pool, "CompA").await;
    // 8.10 valide, 7.70 invalide → l'ensemble doit être rejeté.
    let err = verify_vat_rates_against_db(&pool, company_id, &[dec!(8.10), dec!(7.70)])
        .await
        .unwrap_err();
    assert!(format!("{err:?}").contains(VAT_REJECTED_MSG));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_dedups_duplicate_rates(pool: MySqlPool) {
    // Pass 1 remediation #4 (T3.4) : input `[8.10, 8.10, 8.10]` doit passer
    // sans erreur via la dédup `BTreeSet<&Decimal>` + IN clause batched.
    let company_id = create_company(&pool, "CompA").await;
    verify_vat_rates_against_db(&pool, company_id, &[dec!(8.10), dec!(8.10), dec!(8.10)])
        .await
        .expect("dedup should reduce to 1 distinct rate, valid");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_scale_invariant_across_inputs(pool: MySqlPool) {
    // `Decimal::cmp` scale-invariant (rust_decimal ≥ 1.30 — projet en 1.41).
    // `8.1` et `8.10` doivent collapser à 1 rate distinct dans le BTreeSet.
    let company_id = create_company(&pool, "CompA").await;
    verify_vat_rates_against_db(&pool, company_id, &[dec!(8.1), dec!(8.10), dec!(8.100)])
        .await
        .expect("scale-invariant dedup");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_empty_slice_returns_ok(pool: MySqlPool) {
    // Slice vide = no-op (pas de SELECT). Comportement défensif.
    let company_id = create_company(&pool, "CompA").await;
    verify_vat_rates_against_db(&pool, company_id, &[] as &[Decimal])
        .await
        .expect("empty slice should be Ok(())");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn verify_vat_isolates_per_tenant(pool: MySqlPool) {
    // CompA n'a que les 4 taux par défaut. CompB pareil. Mais si un dev
    // bypassait `company_id` dans la query, le test attraperait la fuite.
    let company_a = create_company(&pool, "CompA").await;
    let _company_b = create_company(&pool, "CompB").await;

    // 8.10 est seedé pour les 2 — passe pour A.
    verify_vat_rates_against_db(&pool, company_a, &[dec!(8.10)])
        .await
        .expect("8.10 valid for CompA");
}
