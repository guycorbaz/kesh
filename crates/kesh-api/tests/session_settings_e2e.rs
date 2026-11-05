//! Story v0172-session (0.17.3, CR #891) — le réglage de la durée d'inactivité
//! de la session, `GET` et `PUT /api/v1/admin/session-settings` (AC-1).
//!
//! Administrateur seul, clés d'API refusées ; tout rejet du corps rend 400
//! `VALIDATION_ERROR` (jamais le 422 d'Axum) ; une valeur inchangée n'écrit
//! rien ; l'audit `admin.session_settings_updated` porte `before` et `after`.
//! Données synthétiques seulement.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

mod common;

const PASSWORD: &str = "session-settings-pass-12";
const PATH: &str = "/api/v1/admin/session-settings";

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
        "session-settings-admin-password".to_string(),
        "test-secret-32-bytes-minimum-test-secret-padding".to_string(),
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready within 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role) {
    if companies::find_by_id(pool, 1).await.unwrap().is_none() {
        sqlx::query(
            "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
             VALUES (1, 'Test Company', 'Test Address', 'Independant', 'FR', 'FR')",
        )
        .execute(pool)
        .await
        .unwrap();
    }
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash_password(PASSWORD).unwrap(),
            role,
            active: true,
            company_id: 1,
            email: None,
        },
    )
    .await
    .unwrap();
}

async fn login(app: &TestApp, username: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": PASSWORD }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn audits(pool: &MySqlPool) -> Vec<Value> {
    let rows: Vec<Option<Vec<u8>>> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'admin.session_settings_updated' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    rows.into_iter()
        .map(|b| serde_json::from_slice(&b.unwrap_or_default()).unwrap_or(Value::Null))
        .collect()
}

async fn row_count(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM _kesh_installation_settings")
        .fetch_one(pool)
        .await
        .expect("table _kesh_installation_settings")
}

async fn put(app: &TestApp, jwt: &str, body: Value) -> reqwest::Response {
    app.client
        .put(app.url(PATH))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&body)
        .send()
        .await
        .unwrap()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn admin_reads_the_default(pool: MySqlPool) {
    create_user(&pool, "admin1", Role::Admin).await;
    let app = spawn_app(pool).await;
    let jwt = login(&app, "admin1").await;
    let resp = app
        .client
        .get(app.url(PATH))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body,
        json!({ "sessionIdleMinutes": 60, "version": 0, "min": 1, "max": 1440 })
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn non_admins_and_api_keys_are_refused(pool: MySqlPool) {
    create_user(&pool, "admin1", Role::Admin).await;
    create_user(&pool, "compta", Role::Comptable).await;
    create_user(&pool, "lecteur", Role::Consultation).await;
    let app = spawn_app(pool).await;
    for who in ["compta", "lecteur"] {
        let jwt = login(&app, who).await;
        let resp = app
            .client
            .get(app.url(PATH))
            .header("Authorization", format!("Bearer {jwt}"))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403, "GET par {who}");
        let resp = put(
            &app,
            &jwt,
            json!({ "sessionIdleMinutes": 30, "version": 0 }),
        )
        .await;
        assert_eq!(resp.status(), 403, "PUT par {who}");
    }
    let jwt = login(&app, "admin1").await;
    let (_id, key) =
        common::create_key_via_http(&app.client, &app.base_url, &jwt, "réglage", "read-write")
            .await;
    let resp = app
        .client
        .get(app.url(PATH))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "GET par une clé d'API");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn malformed_or_out_of_range_bodies_are_400(pool: MySqlPool) {
    create_user(&pool, "admin1", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = login(&app, "admin1").await;
    for body in [
        json!({ "sessionIdleMinutes": 0, "version": 0 }),
        json!({ "sessionIdleMinutes": 1441, "version": 0 }),
        json!({ "sessionIdleMinutes": 1.5, "version": 0 }),
        json!({ "sessionIdleMinutes": "abc", "version": 0 }),
        json!({ "version": 0 }),
    ] {
        let resp = put(&app, &jwt, body.clone()).await;
        assert_eq!(resp.status(), 400, "corps {body}");
        let b: Value = resp.json().await.unwrap();
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR", "corps {body}");
    }
    assert_eq!(row_count(&pool).await, 0, "rien d'écrit");
    assert!(audits(&pool).await.is_empty(), "aucun audit");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_writes_audits_and_locks(pool: MySqlPool) {
    create_user(&pool, "admin1", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = login(&app, "admin1").await;

    // Valeur inchangée à version 0 : aucune ligne, aucun audit.
    let resp = put(
        &app,
        &jwt,
        json!({ "sessionIdleMinutes": 60, "version": 0 }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let b: Value = resp.json().await.unwrap();
    assert_eq!(b["sessionIdleMinutes"], 60);
    assert_eq!(b["version"], 0);
    assert_eq!(row_count(&pool).await, 0);
    assert!(audits(&pool).await.is_empty());

    let resp = put(
        &app,
        &jwt,
        json!({ "sessionIdleMinutes": 30, "version": 0 }),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let b: Value = resp.json().await.unwrap();
    assert_eq!(b["sessionIdleMinutes"], 30);
    assert_eq!(b["version"], 1);
    let a = audits(&pool).await;
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["before"]["sessionIdleMinutes"], 60);
    assert_eq!(a[0]["after"]["sessionIdleMinutes"], 30);

    // Version périmée ⇒ 409.
    let resp = put(
        &app,
        &jwt,
        json!({ "sessionIdleMinutes": 45, "version": 0 }),
    )
    .await;
    assert_eq!(resp.status(), 409);
    let b: Value = resp.json().await.unwrap();
    assert_eq!(b["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    assert_eq!(audits(&pool).await.len(), 1);
}

/// Revue de code P1 (S-B1/S-E1, choix C-v0172s-59) — deux premiers
/// enregistrements simultanés (version 0, table vide) : l'un gagne, l'autre
/// rend **409**, jamais 500.
///
/// Montage déterministe, sur le patron de `rejeu_interblocage_e2e.rs` : une
/// transaction de test **alourdie** (500 lignes de lest : InnoDB choisit pour
/// victime la plus légère) prend le verrou d'intervalle de `id = 1` par le même
/// `SELECT … FOR UPDATE` que le dépôt ; la route, lancée en tâche, le prend
/// aussi (les verrous d'intervalle sont compatibles) puis attend sur son
/// `INSERT` ; la transaction de test insère à son tour : cycle, InnoDB annule
/// la route (1213). Sans rejeu, la route rend 500 ; rejouée, elle relit la
/// ligne validée par la transaction de test et rend le conflit optimiste.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn concurrent_first_puts_yield_a_conflict_not_a_500(pool: MySqlPool) {
    create_user(&pool, "admin1", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = login(&app, "admin1").await;
    let capture = common::capture_rejeu::CaptureRejeu::installer();

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS lest_rejeu (id INT AUTO_INCREMENT PRIMARY KEY, x INT) \
         ENGINE=InnoDB",
    )
    .execute(&pool)
    .await
    .unwrap();
    let mut tx = pool.begin().await.unwrap();
    for _ in 0..10 {
        sqlx::query("INSERT INTO lest_rejeu (x) SELECT seq FROM seq_1_to_50")
            .execute(&mut *tx)
            .await
            .unwrap();
    }
    let vu: Option<(i32, i32)> = sqlx::query_as(
        "SELECT session_idle_minutes, version FROM _kesh_installation_settings \
         WHERE id = 1 FOR UPDATE",
    )
    .fetch_optional(&mut *tx)
    .await
    .unwrap();
    assert!(vu.is_none(), "montage : la table doit être vide");

    let client = app.client.clone();
    let url = app.url(PATH);
    let jwt_tache = jwt.clone();
    let tache = tokio::spawn(async move {
        let resp = client
            .put(url)
            .header("Authorization", format!("Bearer {jwt_tache}"))
            .json(&json!({ "sessionIdleMinutes": 30, "version": 0 }))
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json::<Value>().await.unwrap_or(Value::Null))
    });
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["INSERT INTO _kesh_installation_settings"],
            || tache.is_finished(),
        )
        .await,
        "la route n'a pas attendu sur son INSERT : le cycle n'est pas formé"
    );
    sqlx::query(
        "INSERT INTO _kesh_installation_settings (id, session_idle_minutes, version) \
         VALUES (1, 45, 1)",
    )
    .execute(&mut *tx)
    .await
    .expect("la transaction de test, alourdie, ne doit pas être la victime");
    tx.commit().await.unwrap();

    let (status, corps) = tache.await.unwrap();
    assert_eq!(status, 409, "corps : {corps}");
    assert_eq!(corps["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    capture.exiger_un_rejeu("session_settings::update_session_settings");
    let ligne: (i32, i32) = sqlx::query_as(
        "SELECT session_idle_minutes, version FROM _kesh_installation_settings WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ligne, (45, 1), "la valeur gagnante reste");
    assert!(audits(&pool).await.is_empty(), "le perdant n'audite rien");
}
