//! Story v0174-correctifs-2 (#817, AC-2.5 et AC-2.6) — le pays d'une adresse de
//! contact est contrôlé sur la liste ISO 3166-1 alpha-2, à l'écriture
//! seulement : un code inconnu (« XX ») est refusé à la création et à la
//! modification ; un contact enregistré avant la 0.17.4 avec un tel code
//! (« SU ») se lit toujours, se modifie sans toucher au pays, et le signale
//! (`addressCountryKnown: false`). Données inventées.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
use serde_json::{Value, json};
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
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < Duration::from_secs(2) => {
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

/// Société minimale, un comptable, son jeton ; rend aussi l'id de la société.
async fn setup(pool: &MySqlPool) -> (TestApp, String, i64) {
    let company_id = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Pays 817 SA', 'Rue du Test 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert")
    .last_insert_id() as i64;
    users::create(
        pool,
        NewUser {
            username: "pays817".into(),
            password_hash: hash_password("password-12345").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "pays817", "password": "password-12345"}))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    let token = body["accessToken"].as_str().unwrap().to_string();
    (app, token, company_id)
}

fn address(country: &str) -> Value {
    json!({
        "street": "Rue des Essais",
        "building": "7",
        "postalCode": "1200",
        "city": "Genève",
        "country": country,
    })
}

async fn send(app: &TestApp, token: &str, method: &str, path: &str, body: Value) -> (u16, Value) {
    let req = match method {
        "POST" => app.client.post(app.url(path)),
        "PUT" => app.client.put(app.url(path)),
        _ => app.client.get(app.url(path)),
    };
    let req = req.bearer_auth(token);
    let req = if method == "GET" {
        req
    } else {
        req.json(&body)
    };
    let resp = req.send().await.unwrap();
    let st = resp.status().as_u16();
    (st, resp.json().await.unwrap_or(Value::Null))
}

async fn contact_count(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// AC-2.5 — la création refuse « XX » (rien d'écrit) et accepte « FR ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_refuses_an_unknown_iso_country_817(pool: MySqlPool) {
    let (app, token, _) = setup(&pool).await;
    let (st, body) = send(
        &app,
        &token,
        "POST",
        "/api/v1/contacts",
        json!({ "contactType": "Entreprise", "name": "Essai XX", "isClient": true,
                "addressStructured": address("XX") }),
    )
    .await;
    assert_eq!(st, 400, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    assert!(
        body["error"]["message"]
            .as_str()
            .unwrap()
            .contains("« XX »"),
        "{body}"
    );
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");

    let (st, body) = send(
        &app,
        &token,
        "POST",
        "/api/v1/contacts",
        json!({ "contactType": "Entreprise", "name": "Essai FR", "isClient": true,
                "addressStructured": address("FR") }),
    )
    .await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["addressStructured"]["country"], "FR");
    assert_eq!(body["addressCountryKnown"], true, "{body}");
}

/// AC-2.6 — un contact au pays « SU », posé par SQL comme une installation
/// 0.17.3 : il se lit, se liste, se modifie sans toucher au pays ; un pays
/// changé doit être valide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_stored_unknown_country_reads_and_edits_817(pool: MySqlPool) {
    let (app, token, company_id) = setup(&pool).await;
    let id = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, address_street, \
         address_building, address_postal_code, address_city, address_country, address) \
         VALUES (?, 'Entreprise', 'Ancien SU', TRUE, 'Rue des Essais', '7', '1200', 'Genève', \
         'SU', 'Rue des Essais 7\n1200 Genève')",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let path = format!("/api/v1/contacts/{id}");

    let (st, one) = send(&app, &token, "GET", &path, Value::Null).await;
    assert_eq!(st, 200, "{one}");
    assert_eq!(one["addressStructured"]["country"], "SU");
    assert_eq!(one["addressCountryKnown"], false, "{one}");
    let (st, list) = send(&app, &token, "GET", "/api/v1/contacts", Value::Null).await;
    assert_eq!(st, 200, "{list}");
    let listed = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == id)
        .cloned()
        .expect("le contact est listé");
    assert_eq!(listed["addressCountryKnown"], false, "{listed}");

    // Le téléphone seul change ; l'adresse revient telle quelle, en minuscules.
    let mut body = json!({ "contactType": "Entreprise", "name": "Ancien SU", "isClient": true,
        "addressStructured": address("su"), "phone": "+41 21 000 00 00",
        "version": one["version"] });
    let (st, after) = send(&app, &token, "PUT", &path, body.clone()).await;
    assert_eq!(st, 200, "{after}");
    assert_eq!(after["addressStructured"]["country"], "SU");
    assert_eq!(after["phone"], "+41 21 000 00 00");
    assert_eq!(after["addressCountryKnown"], false);

    // Un pays changé hors de la liste : refusé, contact inchangé en base.
    body["addressStructured"] = address("XX");
    body["version"] = after["version"].clone();
    let (st, refused) = send(&app, &token, "PUT", &path, body.clone()).await;
    assert_eq!(st, 400, "{refused}");
    assert_eq!(refused["error"]["code"], "VALIDATION_ERROR");
    assert!(
        refused["error"]["message"]
            .as_str()
            .unwrap()
            .contains("« XX »"),
        "{refused}"
    );
    let stored: String = sqlx::query_scalar("SELECT address_country FROM contacts WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, "SU", "contact inchangé");

    // Corrigé en « CH » : accepté, le pays est connu.
    body["addressStructured"] = address("CH");
    let (st, fixed) = send(&app, &token, "PUT", &path, body).await;
    assert_eq!(st, 200, "{fixed}");
    assert_eq!(fixed["addressStructured"]["country"], "CH");
    assert_eq!(fixed["addressCountryKnown"], true);
}
