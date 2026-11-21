//! Tests HTTP — Story 48-15 (#1331, #1334) : un texte trop long est **refusé
//! en le disant** par la route des contacts, jamais coupé ni refusé en silence.
//!
//! - **#1334** (AC2) : le prénom et le nom d'une personne sont bornés **chacun**
//!   à 70 caractères (`contacts.first_name` / `last_name` `VARCHAR(70)`), en
//!   `400 VALIDATION_ERROR` qui nomme le champ. Avant la story, seul le nom
//!   composé était borné (255) : un prénom de 71 passait le contrôle et butait
//!   sur MariaDB, `400 DATA_LENGTH_OR_RANGE` générique qui ne nomme rien. C'est
//!   pourquoi chaque test lit `error.code`, non le seul statut.
//! - **AC9** : la borne égale la colonne ; un contact déjà enregistré avec un
//!   prénom et un nom de 70 se réenregistre tel quel.
//! - **Filets** (AC4, AC7 : les refus existent déjà, la story retire les
//!   `maxlength` qui les masquaient à l'écran) : rue de 71, conditions de
//!   paiement de 101, raison sociale de 256, délai de 1000 jours.
//!
//! Harnais calqué sur `contact_client_number_e2e.rs`.

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
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

/// Société factice, utilisateur `Comptable`, puis token.
async fn setup(pool: &MySqlPool) -> (TestApp, String) {
    let company_id = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Démo SA', 'Rue Test 1\n1000 Lausanne', 'Pme', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert")
    .last_insert_id() as i64;
    let hash = hash_password("password-12345").expect("hash should succeed");
    users::create(
        pool,
        NewUser {
            username: "fl_user".to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "fl_user", "password": "password-12345"}))
        .send()
        .await
        .expect("login should succeed");
    let body: serde_json::Value = resp.json().await.expect("json body");
    let token = body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string();
    (app, token)
}

/// Corps d'une personne : prénom, nom, aucune adresse.
fn personne(first: &str, last: &str) -> serde_json::Value {
    json!({
        "contactType": "Personne",
        "name": "",
        "firstName": first,
        "lastName": last,
        "isClient": true,
        "isSupplier": false,
    })
}

/// Corps d'une entreprise, complété par `extra` (clés JSON de l'API).
fn entreprise(name: &str, extra: serde_json::Value) -> serde_json::Value {
    let mut body = json!({
        "contactType": "Entreprise",
        "name": name,
        "isClient": true,
        "isSupplier": false,
    });
    for (k, v) in extra.as_object().expect("objet").iter() {
        body[k] = v.clone();
    }
    body
}

async fn post(app: &TestApp, token: &str, body: &serde_json::Value) -> (u16, serde_json::Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .send()
        .await
        .expect("POST /contacts");
    let status = resp.status().as_u16();
    (status, resp.json().await.expect("json"))
}

async fn put(
    app: &TestApp,
    token: &str,
    id: i64,
    body: &serde_json::Value,
) -> (u16, serde_json::Value) {
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .json(body)
        .send()
        .await
        .expect("PUT /contacts/{id}");
    let status = resp.status().as_u16();
    (status, resp.json().await.expect("json"))
}

async fn get(app: &TestApp, token: &str, id: i64) -> serde_json::Value {
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/contacts/{id}")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("GET /contacts/{id}");
    assert_eq!(resp.status(), 200);
    resp.json().await.expect("json")
}

async fn contact_count(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
        .fetch_one(pool)
        .await
        .expect("count")
}

/// Un refus de validation **nommé** : `400`, `VALIDATION_ERROR` (et non
/// `DATA_LENGTH_OR_RANGE`, le refus générique de MariaDB qui ne nomme rien),
/// message qui commence par `prefixe`.
fn assert_refus_nomme(status: u16, body: &serde_json::Value, prefixe: &str) {
    assert_eq!(status, 400, "{body}");
    assert_eq!(
        body["error"]["code"], "VALIDATION_ERROR",
        "le refus doit venir de la route, non de MariaDB : {body}"
    );
    let message = body["error"]["message"].as_str().unwrap_or_default();
    assert!(
        message.starts_with(prefixe),
        "le message doit nommer le champ (« {prefixe} ») : {message}"
    );
}

/// AC2 (#1334) — `POST` : un prénom de 71 caractères est refusé, le message
/// nomme « Prénom », rien n'est écrit. Rouge sans la borne : M2.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_71_character_first_name_is_refused_and_named(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, body) = post(&app, &token, &personne(&"P".repeat(71), "Dupont")).await;
    assert_refus_nomme(status, &body, "Prénom : 70 caractères maximum");
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");
}

/// AC2 (#1334) — `POST` : un nom de 71 caractères est refusé, le message nomme
/// « Nom ». Rouge sans la borne : M3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_71_character_last_name_is_refused_and_named(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, body) = post(&app, &token, &personne("Jean", &"N".repeat(71))).await;
    assert_refus_nomme(status, &body, "Nom : 70 caractères maximum");
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");
}

/// AC2 (#1334) — 100 + 100 : le nom composé (201) tient sous la borne de 255
/// qui seule existait ; c'est le cas que le contrôle du composé laissait
/// passer jusqu'à MariaDB.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_100_plus_100_person_is_refused_by_the_route(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, body) = post(&app, &token, &personne(&"P".repeat(100), &"N".repeat(100))).await;
    assert_refus_nomme(status, &body, "Prénom : 70 caractères maximum");
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");
}

/// AC2 (#1334) — `PUT` : même refus, nommé, et le contact reste tel qu'il
/// était (valeurs et `version`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_refuses_a_71_character_first_or_last_name(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, created) = post(&app, &token, &personne("Jean", "Dupont")).await;
    assert_eq!(status, 201, "{created}");
    let id = created["id"].as_i64().expect("id");
    let version = created["version"].as_i64().expect("version");

    for (first, last, prefixe) in [
        ("P".repeat(71), "Dupont".to_string(), "Prénom :"),
        ("Jean".to_string(), "N".repeat(71), "Nom :"),
    ] {
        let mut body = personne(&first, &last);
        body["version"] = json!(version);
        let (status, refus) = put(&app, &token, id, &body).await;
        assert_refus_nomme(status, &refus, prefixe);
        let relu = get(&app, &token, id).await;
        assert_eq!(relu["firstName"], "Jean");
        assert_eq!(relu["lastName"], "Dupont");
        assert_eq!(relu["version"], json!(version), "rien d'écrit");
    }
}

/// Revue P1 (B4, E3), AC9 — la borne compte des **caractères** sur la valeur
/// **rognée**, comme la colonne `VARCHAR(70)` en `utf8mb4` : 70 « é » (140
/// octets) entourés de blancs sont acceptés et stockés rognés, 70 « ß » aussi ;
/// 71 « é » sont refusés. Rouge si la borne comptait en octets (mutation M21,
/// `exceeds_len` en `len()`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn multibyte_and_padded_names_are_counted_in_characters_after_trimming(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (first, last) = ("é".repeat(70), "ß".repeat(70));
    let (status, created) = post(&app, &token, &personne(&format!("  {first}  "), &last)).await;
    assert_eq!(
        status, 201,
        "70 caractères multi-octets, blancs autour : {created}"
    );
    assert_eq!(created["firstName"], first.as_str(), "stocké rogné, entier");
    assert_eq!(created["lastName"], last.as_str());

    let (status, body) = post(&app, &token, &personne(&"é".repeat(71), "Dupont")).await;
    assert_refus_nomme(status, &body, "Prénom : 70 caractères maximum");
}

/// AC2 (filet) et AC9 — 70 et 70 : acceptés, stockés entiers ; le même contact
/// se réenregistre tel quel (`200`). Une valeur déjà en base tient dans son
/// `VARCHAR(70)` : la borne égale la colonne, aucune donnée existante n'est
/// refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_70_plus_70_person_is_accepted_and_saves_again_unchanged(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (first, last) = ("P".repeat(70), "N".repeat(70));
    let (status, created) = post(&app, &token, &personne(&first, &last)).await;
    assert_eq!(status, 201, "70 et 70 doivent être acceptés : {created}");
    assert_eq!(created["firstName"], first.as_str());
    assert_eq!(created["lastName"], last.as_str());

    let id = created["id"].as_i64().expect("id");
    let mut body = personne(&first, &last);
    body["version"] = created["version"].clone();
    let (status, saved) = put(&app, &token, id, &body).await;
    assert_eq!(status, 200, "un contact existant se réenregistre : {saved}");
    assert_eq!(saved["firstName"], first.as_str());
    assert_eq!(saved["lastName"], last.as_str());
}

/// AC4 (#1331, filet) — une rue de 71 caractères est refusée par la route,
/// avec un message qui dit le champ et le nombre saisi. Le refus existe déjà
/// (`address_input.rs`, `check_lengths`) : la story retire le `maxlength` qui
/// le masquait à l'écran.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_71_character_street_is_refused_with_its_count(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let body = entreprise(
        "Démo SA",
        json!({ "addressStructured": {
            "street": "R".repeat(71), "building": "1", "postalCode": "1000",
            "city": "Lausanne", "country": "CH"
        }}),
    );
    let (status, refus) = post(&app, &token, &body).await;
    assert_refus_nomme(
        status,
        &refus,
        "adresse : le champ rue dépasse 70 caractères (71)",
    );
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");
}

/// AC7 (filets) — les champs du PDF dont l'écran perd son `maxlength` : les
/// refus existent, chacun est cité ici, à la borne et un cran au-delà.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn pdf_fields_are_refused_past_their_bound_and_accepted_at_it(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;

    let refus = [
        (
            entreprise("Démo SA", json!({ "defaultPaymentTerms": "c".repeat(101) })),
            "Les conditions de paiement doivent faire au plus 100 caractères",
        ),
        (
            entreprise(&"R".repeat(256), json!({})),
            "Le nom doit faire au plus 255 caractères",
        ),
        (
            entreprise("Démo SA", json!({ "defaultPaymentTermsDays": 1000 })),
            "Le délai de paiement doit être compris entre 0 et 365 jours",
        ),
    ];
    for (body, prefixe) in &refus {
        let (status, rendu) = post(&app, &token, body).await;
        assert_refus_nomme(status, &rendu, prefixe);
    }
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");

    for body in [
        entreprise(
            "Démo SA 1",
            json!({ "defaultPaymentTerms": "c".repeat(100) }),
        ),
        entreprise(&"R".repeat(255), json!({})),
        entreprise("Démo SA 3", json!({ "defaultPaymentTermsDays": 365 })),
    ] {
        let (status, created) = post(&app, &token, &body).await;
        assert_eq!(status, 201, "à la borne, accepté : {created}");
    }
}

/// Story v0200-corrections-mineures (#1336, AC4, filet) — l'écran ne coupe plus
/// l'e-mail ni le téléphone : le serveur refuse ce qui dépasse, `error.code` lu,
/// et accepte la borne. Refus préexistants (`contacts.rs`, `MAX_EMAIL_LEN`,
/// `MAX_PHONE_LEN`) : ce test ne rougit sous aucune mutation de la story.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn email_and_phone_are_refused_past_their_bound_and_accepted_at_it_1336(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let courriel = |n: usize| format!("{}@example.com", "a".repeat(n - "@example.com".len()));

    let (status, rendu) = post(
        &app,
        &token,
        &entreprise("Démo SA", json!({ "email": courriel(321) })),
    )
    .await;
    assert_refus_nomme(status, &rendu, "L'email doit faire au plus 320 caractères");
    let (status, rendu) = post(
        &app,
        &token,
        &entreprise("Démo SA", json!({ "phone": "0".repeat(51) })),
    )
    .await;
    assert_refus_nomme(
        status,
        &rendu,
        "Le téléphone doit faire au plus 50 caractères",
    );
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");

    for body in [
        entreprise("Démo SA 1", json!({ "email": courriel(320) })),
        entreprise("Démo SA 2", json!({ "phone": "0".repeat(50) })),
    ] {
        let (status, created) = post(&app, &token, &body).await;
        assert_eq!(status, 201, "à la borne, accepté : {created}");
    }
}

/// Le geste que dit tout refus d'un champ invisible (#1343, D5).
const GESTE_1343: &str = "effacez le champ et saisissez-le de nouveau";

/// Story v0200-corrections-mineures (#1343, AC11) — `POST` : un prénom, un nom ou
/// une raison sociale faits du seul trait d'union conditionnel sont refusés, le
/// message nomme le champ et dit le geste ; rien n'est écrit. Rouges : M13
/// (prénom), M14 (raison sociale), M16.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn invisible_names_are_refused_and_named_1343(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    for (body, prefixe) in [
        (
            personne("\u{AD}", "Dupont"),
            "Prénom : aucun caractère visible",
        ),
        (personne("Jean", "\u{AD}"), "Nom : aucun caractère visible"),
        (
            entreprise("\u{AD}", json!({})),
            "Raison sociale : aucun caractère visible",
        ),
    ] {
        let (status, rendu) = post(&app, &token, &body).await;
        assert_refus_nomme(status, &rendu, prefixe);
        let message = rendu["error"]["message"].as_str().unwrap_or_default();
        assert!(message.contains(GESTE_1343), "geste absent : {message}");
    }
    assert_eq!(contact_count(&pool).await, 0, "rien d'écrit");
}

/// AC11 — `PUT` : même refus, et le contact reste tel qu'il était.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_refuses_invisible_names_1343(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, created) = post(&app, &token, &personne("Jean", "Dupont")).await;
    assert_eq!(status, 201, "{created}");
    let id = created["id"].as_i64().expect("id");
    let version = created["version"].as_i64().expect("version");
    for (first, last, prefixe) in [
        ("\u{AD}", "Dupont", "Prénom : aucun caractère visible"),
        ("Jean", "\u{200B}\u{FEFF}", "Nom : aucun caractère visible"),
    ] {
        let mut body = personne(first, last);
        body["version"] = json!(version);
        let (status, refus) = put(&app, &token, id, &body).await;
        assert_refus_nomme(status, &refus, prefixe);
        let message = refus["error"]["message"].as_str().unwrap_or_default();
        assert!(message.contains(GESTE_1343), "geste absent : {message}");
        let relu = get(&app, &token, id).await;
        assert_eq!(relu["firstName"], "Jean");
        assert_eq!(relu["version"], json!(version), "rien d'écrit");
    }

    // Revue P1 (A2) — la raison sociale invisible d'une « Entreprise », en `PUT`. Rouge : M14.
    let (status, societe) = post(&app, &token, &entreprise("Démo SA", json!({}))).await;
    assert_eq!(status, 201, "{societe}");
    let id = societe["id"].as_i64().expect("id");
    let mut body = entreprise("\u{AD}", json!({}));
    body["version"] = societe["version"].clone();
    let (status, refus) = put(&app, &token, id, &body).await;
    assert_refus_nomme(status, &refus, "Raison sociale : aucun caractère visible");
    let message = refus["error"]["message"].as_str().unwrap_or_default();
    assert!(message.contains(GESTE_1343), "geste absent : {message}");
    let relu = get(&app, &token, id).await;
    assert_eq!(relu["name"], "Démo SA");
    assert_eq!(relu["version"], societe["version"], "rien d'écrit");
}

/// AC11 (filets) — un prénom vide garde le message d'aujourd'hui ; un prénom qui
/// MÊLE invisible et visible passe tel quel ; le même contact se réenregistre
/// (AC12 b).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn empty_and_mixed_names_keep_their_behaviour_1343(pool: MySqlPool) {
    let (app, token) = setup(&pool).await;
    let (status, rendu) = post(&app, &token, &personne("", "Dupont")).await;
    assert_refus_nomme(
        status,
        &rendu,
        "Le prénom et le nom sont obligatoires pour une personne",
    );
    let (status, rendu) = post(&app, &token, &entreprise("  ", json!({}))).await;
    assert_refus_nomme(
        status,
        &rendu,
        "La raison sociale est obligatoire pour une entreprise",
    );

    let (status, created) = post(&app, &token, &personne("Jean\u{AD}Pierre", "Dupont")).await;
    assert_eq!(status, 201, "{created}");
    assert_eq!(created["firstName"], "Jean\u{AD}Pierre");
    let id = created["id"].as_i64().expect("id");
    let mut body = personne("Jean\u{AD}Pierre", "Dupont");
    body["version"] = created["version"].clone();
    let (status, rendu) = put(&app, &token, id, &body).await;
    assert_eq!(status, 200, "réenregistré : {rendu}");
}
