//! End-to-end HTTP tests — personnes de contact (#213), et leur **traçage**
//! (Story 25-1b, AC 2).
//!
//! ⛔ **Ce fichier n'existait pas, et c'est la story qui l'a découvert.** Les
//! trois routes `contact_persons` — création, modification, archivage —
//! n'avaient **aucun test**, ni d'intégration ni de bout en bout. La story ne
//! pouvait pas s'en accommoder : *on n'ajoute pas une trace à du code que rien
//! n'exerce.*
//!
//! Ce que ces tests établissent, au-delà de l'audit : les trois routes
//! fonctionnent, le verrouillage optimiste tient, et l'archivage est bien un
//! soft-delete.
//!
//! Harnais calqué sur `contact_payment_terms_e2e.rs`.

mod common;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use common::{audit_actions, audit_actor, audit_count, audit_details};
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

    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("serveur de test non prêt : {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

/// Société + utilisateur Comptable + contact de type Entreprise.
///
/// Le type `Entreprise` n'est pas décoratif : `ensure_entreprise_contact` refuse
/// d'attacher une personne à un contact Particulier.
struct Ctx {
    jwt: String,
    contact_id: i64,
    user_id: i64,
}

async fn setup(pool: &MySqlPool, app: &TestApp) -> Ctx {
    let company_id = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('CP Test Co', 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("insert société")
    .last_insert_id() as i64;

    let hash = hash_password("comptable-password-12").expect("hash");
    let user = users::create(
        pool,
        NewUser {
            username: "comptable".into(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("insert utilisateur");

    let contact_id = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Dubarde SàRL', 1)",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("insert contact")
    .last_insert_id() as i64;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "comptable", "password": "comptable-password-12"}))
        .send()
        .await
        .expect("login");
    let body: Value = resp.json().await.expect("json");
    let jwt = body["accessToken"]
        .as_str()
        .expect("accessToken")
        .to_string();

    Ctx {
        jwt,
        contact_id,
        user_id: user.id,
    }
}

async fn create_person(app: &TestApp, ctx: &Ctx, first: &str, last: &str) -> Value {
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/contacts/{}/persons", ctx.contact_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({"firstName": first, "lastName": last, "role": "Directeur"}))
        .send()
        .await
        .expect("POST persons");
    assert_eq!(resp.status(), 201, "création d'une personne de contact");
    resp.json().await.expect("json")
}

/// AC 2 — la création laisse une trace attribuée, avec son instantané.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_person_writes_audit(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;

    let person = create_person(&app, &ctx, "Anne", "Dubarde").await;
    let id = person["id"].as_i64().expect("id");

    assert_eq!(
        audit_actions(&pool, "contact_person", id).await,
        vec!["contact_person.created".to_string()]
    );

    let details = audit_details(&pool, "contact_person", id, "contact_person.created")
        .await
        .expect("détails présents");
    assert_eq!(details["last_name"], "Dubarde");
    assert_eq!(details["contact_id"], ctx.contact_id);

    // ⚠️ `from_current_user` et non `::user` : cette route vit dans
    // `comptable_routes`, donc HORS du seul `require_not_pat` du dépôt — un
    // jeton d'API l'atteint, et `::user` y écrirait un acteur faux.
    let (actor_type, api_key_id, user_id, actor_label) =
        audit_actor(&pool, "contact_person", id, "contact_person.created").await;
    assert_eq!(actor_type, "user", "connexion par JWT dans ce test");
    assert_eq!(api_key_id, None);
    assert_eq!(user_id, ctx.user_id);
    assert_eq!(actor_label, "comptable");
}

/// AC 2 — la modification porte l'avant ET l'après.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn update_person_writes_before_and_after(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;

    let person = create_person(&app, &ctx, "Anne", "Dubarde").await;
    let id = person["id"].as_i64().unwrap();
    let version = person["version"].as_i64().unwrap();

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/contact-persons/{id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "firstName": "Anne-Sophie",
            "lastName": "Dubarde",
            "role": "Directrice",
            "version": version,
        }))
        .send()
        .await
        .expect("PUT person");
    assert_eq!(resp.status(), 200);

    let details = audit_details(&pool, "contact_person", id, "contact_person.updated")
        .await
        .expect("détails présents");
    assert_eq!(
        details["before"]["first_name"], "Anne",
        "l'avant est lu DANS la transaction qui modifie"
    );
    assert_eq!(details["after"]["first_name"], "Anne-Sophie");
    assert_eq!(details["before"]["role"], "Directeur");
    assert_eq!(details["after"]["role"], "Directrice");
}

/// AC 2 — l'archivage nomme ce qu'il archive.
///
/// ⛔ C'est le cas le plus fragile des trois : le handler ne connaît QUE
/// l'identifiant. Sans pré-chargement, la trace ne nommerait personne — et une
/// trace anonyme ne répond à personne.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archive_person_names_what_it_archives(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;

    let person = create_person(&app, &ctx, "Anne", "Dubarde").await;
    let id = person["id"].as_i64().unwrap();

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/contact-persons/{id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .expect("DELETE person");
    assert_eq!(resp.status(), 204);

    assert_eq!(
        audit_actions(&pool, "contact_person", id).await,
        vec![
            "contact_person.created".to_string(),
            "contact_person.archived".to_string()
        ],
        "le cycle de vie complet se lit dans la piste"
    );

    let details = audit_details(&pool, "contact_person", id, "contact_person.archived")
        .await
        .expect("l'archivage porte un instantané, pas un identifiant nu");
    assert_eq!(details["last_name"], "Dubarde");
    assert_eq!(details["contact_id"], ctx.contact_id);

    // L'archivage est un soft-delete : la ligne survit, désactivée.
    let active: bool = sqlx::query_scalar("SELECT active FROM contact_persons WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .expect("la ligne existe encore");
    assert!(!active, "soft-delete : la ligne reste, inactive");
}

/// AC 9 — un refus n'écrit RIEN, et c'est la transaction qui le garantit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn version_conflict_writes_no_audit(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, &app).await;

    let person = create_person(&app, &ctx, "Anne", "Dubarde").await;
    let id = person["id"].as_i64().unwrap();
    let before = audit_count(&pool).await;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/contact-persons/{id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "firstName": "Anne",
            "lastName": "Dubarde",
            "role": "Directeur",
            "version": 999,
        }))
        .send()
        .await
        .expect("PUT person");
    assert_eq!(resp.status(), 409, "version périmée → conflit");

    assert_eq!(
        audit_count(&pool).await,
        before,
        "le refus n'écrit aucune trace — le rollback s'en charge, pas un `if`"
    );
}
