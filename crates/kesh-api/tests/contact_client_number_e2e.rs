//! End-to-end HTTP tests — Story 16-3b (#151) : numéro de client du contact.
//!
//! Couvre AC3 et AC4 :
//! - aller-retour `POST` → `GET /contacts/{id}` (la couture `From<Contact>`
//!   qu'**aucun compilateur ne vérifie** : omettre la ligne compile, stocke, et
//!   rend `null` pour toujours) ;
//! - `clientNumber: ""` stocké `NULL`, et **deux** contacts créés ainsi tous
//!   deux acceptés — sans quoi la garde de D2 tiendrait sur une convention de
//!   politesse du client ;
//! - doublon en création **et** en modification → **409** avec le code
//!   `CLIENT_NUMBER_ALREADY_EXISTS` asserté (un test qui n'assert que « ce
//!   n'est pas 200 » laisserait passer un 500) ;
//! - numéro d'un contact archivé réattribuable **à travers l'API**.
//!
//! Harnais calqué sur `contact_payment_terms_e2e.rs`, amputé du seed de
//! facturation : cette story ne crée aucune facture.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
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

/// Crée une société et son utilisateur. Extrait de `setup` pour que les tests
/// d'isolation inter-locataires puissent en créer une **seconde** : sans deux
/// sociétés, rien ne distingue `UNIQUE (company_id, client_number_uniq)` d'un
/// `UNIQUE (client_number_uniq)` global.
async fn create_company_with_user(pool: &MySqlPool, company_name: &str, username: &str) {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .bind(company_name)
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    let hash = hash_password("password-12345").expect("hash should succeed");
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

/// Société minimale + utilisateur, puis token. Aucun seed de facturation.
async fn setup(pool: &MySqlPool) -> (TestApp, String) {
    create_company_with_user(pool, "CN Test Co", "cn_user").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "cn_user", "password-12345").await;
    (app, token)
}

fn payload(name: &str, client_number: Option<&str>) -> serde_json::Value {
    let mut body = json!({
        "contactType": "Entreprise",
        "name": name,
        "isClient": true,
        "isSupplier": false,
    });
    if let Some(cn) = client_number {
        body["clientNumber"] = json!(cn);
    }
    body
}

async fn post_contact(
    app: &TestApp,
    token: &str,
    name: &str,
    client_number: Option<&str>,
) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&payload(name, client_number))
        .send()
        .await
        .expect("contact create request")
}

/// Extrait `error.code` d'un corps d'erreur, en échouant de façon lisible si la
/// réponse n'a pas la forme attendue.
fn error_code(body: &serde_json::Value) -> &str {
    body["error"]["code"]
        .as_str()
        .unwrap_or_else(|| panic!("corps d'erreur inattendu : {body}"))
}

/// AC3 — la valeur traverse réellement `POST` puis `GET /contacts/{id}`.
///
/// Mutation que ce test doit tuer : retirer `client_number: c.client_number`
/// d'`impl From<Contact> for ContactResponse`. Elle compile, stocke la valeur,
/// et rend `null` pour toujours.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn client_number_roundtrips_from_post_to_get(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    let resp = post_contact(&app, &token, "CN Roundtrip SA", Some("CLI-2026-00042")).await;
    assert_eq!(resp.status(), 201);
    let created: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(created["clientNumber"], "CLI-2026-00042");

    let id = created["id"].as_i64().expect("id");
    let fetched: serde_json::Value = app
        .client
        .get(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("get request")
        .json()
        .await
        .expect("json");
    assert_eq!(
        fetched["clientNumber"], "CLI-2026-00042",
        "le GET doit rendre le numéro — aucun compilateur ne vérifie cette couture"
    );
}

/// AC3 — `""` est normalisé en `NULL`, et **deux** contacts créés ainsi sont
/// tous deux acceptés. C'est le cas MAJORITAIRE que D2 prétend protéger :
/// pour un index UNIQUE, `""` est une valeur comme une autre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn empty_client_number_is_stored_as_null_and_never_collides(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    // ⚠️ Le statut se vérifie AVANT de lire le corps. Sans cette assertion, un
    // `400` de validation rend un corps d'erreur où `clientNumber` est absent —
    // donc `Value::Null` — et `is_null()` passe : le test resterait vert en
    // ayant cessé de mesurer ce qu'il prétend mesurer. C'est le mode d'échec du
    // test muet, déjà payé deux fois dans ce dépôt.
    let resp = post_contact(&app, &token, "CN Empty A", Some("   ")).await;
    assert_eq!(
        resp.status(),
        201,
        "une valeur blanche doit être acceptée à la création"
    );
    let first: serde_json::Value = resp.json().await.expect("json");
    assert!(
        first["clientNumber"].is_null(),
        "une valeur blanche doit être stockée NULL, obtenu {}",
        first["clientNumber"]
    );

    let resp = post_contact(&app, &token, "CN Empty B", Some("")).await;
    assert_eq!(
        resp.status(),
        201,
        "un second contact sans numéro doit être accepté"
    );
    let second: serde_json::Value = resp.json().await.expect("json");
    assert!(second["clientNumber"].is_null());
}

/// AC4 — doublon à la création : **409** et code d'erreur dédié.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn duplicate_client_number_on_create_returns_409_with_dedicated_code(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    assert_eq!(
        post_contact(&app, &token, "CN Dup A", Some("CLI-1"))
            .await
            .status(),
        201
    );

    let resp = post_contact(&app, &token, "CN Dup B", Some("CLI-1")).await;
    assert_eq!(
        resp.status(),
        409,
        "aligné sur le précédent IDE de la même table"
    );
    let body: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(error_code(&body), "CLIENT_NUMBER_ALREADY_EXISTS");
}

/// AC4 — doublon à la **modification** : le chemin `PUT` doit rendre le même
/// 409, et non un 500 opaque sur l'erreur SQLx `1062`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn duplicate_client_number_on_update_returns_409_with_dedicated_code(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    post_contact(&app, &token, "CN Held", Some("CLI-HELD"))
        .await
        .json::<serde_json::Value>()
        .await
        .expect("json");
    let other: serde_json::Value = post_contact(&app, &token, "CN Other", None)
        .await
        .json()
        .await
        .expect("json");
    let id = other["id"].as_i64().expect("id");
    let version = other["version"].as_i64().expect("version");

    let mut body = payload("CN Other", Some("CLI-HELD"));
    body["version"] = json!(version);
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&body)
        .send()
        .await
        .expect("update request");
    assert_eq!(resp.status(), 409);
    let err: serde_json::Value = resp.json().await.expect("json");
    assert_eq!(error_code(&err), "CLIENT_NUMBER_ALREADY_EXISTS");
}

/// AC1 cas 4, vu depuis l'API : archiver libère le numéro. Le test repository
/// prouve la contrainte SQL ; celui-ci prouve que le chemin HTTP en bénéficie.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archiving_a_contact_frees_its_client_number_through_the_api(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    let first: serde_json::Value = post_contact(&app, &token, "CN Archived", Some("CLI-ARCH"))
        .await
        .json()
        .await
        .expect("json");
    let id = first["id"].as_i64().expect("id");
    let version = first["version"].as_i64().expect("version");

    let archive = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}/archive")))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({ "version": version }))
        .send()
        .await
        .expect("archive request");
    assert_eq!(archive.status(), 200, "archivage attendu OK");

    let resp = post_contact(&app, &token, "CN Successor", Some("CLI-ARCH")).await;
    assert_eq!(
        resp.status(),
        201,
        "le numéro d'un contact archivé doit être réattribuable"
    );
}

/// AC10 — le numéro est cherchable. Sans cela, la moitié du « so that » de la
/// story (« retrouver un contact depuis une facture papier ») n'a **aucun**
/// chemin : l'utilisateur tenant une facture portant « N° client :
/// CLI-2026-00042 » ne peut littéralement pas remonter au contact.
///
/// Les trois cas couvrent les **deux** branches de la clause de recherche :
/// - terme complet et fragment → branche FULLTEXT + `LIKE` ;
/// - terme fait **uniquement** d'opérateurs FULLTEXT (`escape_boolean_ft` les
///   retire tous, l'échappé est vide) → branche `LIKE` seule. C'est celle
///   qu'un traitement partiel laisse muette : elle compile, et cesse
///   simplement de chercher le numéro.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn contacts_are_searchable_by_client_number(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    post_contact(&app, &token, "CN Searchable SA", Some("CLI-2026-00042")).await;
    post_contact(&app, &token, "CN Unrelated SA", None).await;

    async fn search(app: &TestApp, token: &str, term: &str) -> serde_json::Value {
        app.client
            .get(app.url("/api/v1/contacts"))
            .query(&[("search", term)])
            .header("Authorization", format!("Bearer {token}"))
            .send()
            .await
            .expect("list request")
            .json()
            .await
            .expect("json")
    }

    for (term, why) in [
        ("CLI-2026-00042", "numéro exact"),
        ("00042", "fragment du numéro"),
        ("-", "terme fait uniquement d'opérateurs FULLTEXT"),
    ] {
        let body = search(&app, &token, term).await;
        let items = body["items"].as_array().expect("items");
        assert_eq!(
            items.len(),
            1,
            "recherche « {term} » ({why}) : 1 contact attendu, obtenu {}",
            items.len()
        );
        assert_eq!(items[0]["name"], "CN Searchable SA");
    }
}

/// AC1 — l'unicité est **par société**, pas globale.
///
/// ⚠️ Sans deux locataires, aucun des autres tests ne distingue
/// `UNIQUE (company_id, client_number_uniq)` d'un `UNIQUE (client_number_uniq)`
/// global : ils passent tous dans les deux cas. La régression serait pourtant
/// grave et **invisible au locataire** — la société B se verrait refuser en 409
/// un numéro qu'elle ne peut ni voir ni libérer, parce qu'il appartient à A.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn same_client_number_is_allowed_across_two_companies(pool: MySqlPool) {
    let (app, token_a) = setup(&pool).await;
    create_company_with_user(&pool, "CN Test Co B", "cn_user_b").await;
    let token_b = login(&app, "cn_user_b", "password-12345").await;

    assert_eq!(
        post_contact(&app, &token_a, "Client de A", Some("CLI-1"))
            .await
            .status(),
        201,
        "la société A doit pouvoir attribuer CLI-1"
    );
    assert_eq!(
        post_contact(&app, &token_b, "Client de B", Some("CLI-1"))
            .await
            .status(),
        201,
        "la société B doit pouvoir attribuer LE MÊME CLI-1 : l'unicité est \
         portée par (company_id, client_number_uniq), pas par le numéro seul"
    );
}

/// AC3 — la borne de longueur est refusée **par la route**, en 400 métier.
///
/// ⚠️ Sans ce test, supprimer le garde de `validate_common` ne casse rien
/// visiblement : la valeur atteint un `VARCHAR(50)`, et selon le mode SQL
/// MariaDB rend soit une erreur 1406 remontée en 500 opaque, soit — hors mode
/// strict — une **troncature silencieuse** qui fabrique en prime un faux
/// doublon avec tout numéro partageant le même préfixe de 50 caractères.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn client_number_over_max_length_is_rejected_by_the_route(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    let too_long = "C".repeat(51);
    let resp = post_contact(&app, &token, "CN Too Long", Some(&too_long)).await;
    assert_eq!(
        resp.status(),
        400,
        "51 caractères doivent être refusés par la route, pas par MariaDB"
    );

    let at_limit = "D".repeat(50);
    assert_eq!(
        post_contact(&app, &token, "CN At Limit", Some(&at_limit))
            .await
            .status(),
        201,
        "50 caractères — la borne exacte — doivent rester acceptés"
    );
}
