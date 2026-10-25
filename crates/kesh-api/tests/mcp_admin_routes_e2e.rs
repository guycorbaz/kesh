//! Story 32-5c (refs #723) — les routes de gestion des clés et des réglages du
//! serveur MCP.
//!
//! - AC-c1 : les six routes, par HTTP, avec une session d'Administrateur ;
//! - AC-c2 : les gardes — Comptable → `403 FORBIDDEN`, clé MCP → 401, sans
//!   authentification → 401, sur chaque route (les refus d'un PAT, aux deux
//!   portées, sont tenus par `admin_pat_denied_e2e`) ;
//! - AC-c3 : la validation des clés — 400 traduit, rien d'écrit, aucun audit ;
//! - AC-c4 : la validation des réglages et le verrou optimiste (`version: 0`
//!   sur une ligne existante → 409, jamais d'écrasement) ;
//! - AC-c5 : les quatre entrées d'audit, sans secret ni condensat ;
//! - AC-c6 : une clé créée par la route s'authentifie avec ses portées et sa
//!   politique exactes ; la politique changée est rendue ; révoquée, elle
//!   n'authentifie plus.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::api_key::{MCP_PREFIX, sha256_hex};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::services::acces::{Canal, Portee};
use kesh_api::services::cles_mcp::authentifier;
use kesh_api::services::poignees::{Lecture, Trace};
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role, SendValidation};
use kesh_db::repositories::{companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    /// `method path`, jeton facultatif en `Authorization: Bearer`, corps JSON.
    async fn send(
        &self,
        method: &str,
        path: &str,
        bearer: Option<&str>,
        body: Value,
    ) -> reqwest::Response {
        let m = reqwest::Method::from_bytes(method.as_bytes()).expect("méthode HTTP valide");
        let mut req = self.client.request(m, self.url(path)).json(&body);
        if let Some(b) = bearer {
            req = req.header("Authorization", format!("Bearer {b}"));
        }
        req.send().await.expect("requête HTTP")
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "admin-password-for-tests-12".to_string(),
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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: Role, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.as_str().to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

async fn societe(pool: &MySqlPool, nom: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: nom.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

/// Un utilisateur du rôle voulu et sa session (JWT forgé).
async fn session(pool: &MySqlPool, company_id: i64, nom: &str, role: Role) -> (i64, String) {
    let id = users::create(
        pool,
        NewUser {
            username: nom.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id;
    (id, forge_jwt(id, role, company_id))
}

/// Une société (précédée d'une société leurre : identifiants distincts) et sa
/// session d'Administrateur.
async fn livres(pool: &MySqlPool) -> (i64, i64, String) {
    societe(pool, "Leurre").await;
    let c = societe(pool, "Livres MCP").await;
    let (u, jwt) = session(pool, c, "admin-mcp", Role::Admin).await;
    (c, u, jwt)
}

async fn code(resp: reqwest::Response) -> (u16, String) {
    let status = resp.status().as_u16();
    let body: Value = resp
        .json()
        .await
        .unwrap_or_else(|e| panic!("corps JSON attendu (statut {status}) : {e}"));
    (
        status,
        body["error"]["code"].as_str().unwrap_or("").to_string(),
    )
}

/// Crée une clé par la route ; rend le corps de la réponse 201.
async fn creer(app: &TestApp, jwt: &str, body: Value) -> Value {
    let resp = app
        .send("POST", "/api/v1/admin/mcp/keys", Some(jwt), body)
        .await;
    assert_eq!(resp.status(), 201, "création");
    resp.json().await.unwrap()
}

async fn compte(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

fn poignees(pool: &MySqlPool) -> (Lecture, Trace) {
    (Lecture::new(pool.clone()), Trace::new(pool.clone()))
}

const BONNE_CLE: &str = r#"{"name":"agent","scopes":["books:read"]}"#;

// ---------------------------------------------------------------------------
// AC-c1, AC-c6 — créer, lister, authentifier, changer la politique, révoquer
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c1_c6_a_created_key_authenticates_with_its_scopes_and_policy(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (c, u, jwt) = livres(&pool).await;
    let (lecture, trace) = poignees(&pool);

    let expires = (Utc::now() + TimeDelta::days(30)).to_rfc3339();
    let cree = creer(
        &app,
        &jwt,
        json!({
            // Blancs Unicode et largeur nulle aux bords (B1 = E4) : rognés.
            "name": "\u{00A0}\u{3000}Agent comptable\u{200B}",
            // Doublon et désordre : normalisés.
            "scopes": ["invoices:read", "books:read", "invoices:read"],
            "expiresAt": expires,
        }),
    )
    .await;
    let jeton = cree["key"].as_str().expect("secret rendu").to_string();
    assert!(jeton.starts_with(MCP_PREFIX), "préfixe : {jeton}");
    assert_eq!(cree["name"], "Agent comptable");
    assert_eq!(cree["scopes"], json!(["books:read", "invoices:read"]));
    assert_eq!(cree["sendValidation"], "required", "politique par défaut");
    assert_eq!(cree["createdByUserId"], u);
    assert_eq!(cree["version"], 1);
    assert!(cree["expiresAt"].is_string());
    let id = cree["id"].as_i64().unwrap();

    // AC-c6 — l'authentification rend exactement ce que la route a reçu.
    let cle = authentifier(&lecture, &trace, &jeton)
        .await
        .expect("la clé créée par la route authentifie");
    assert_eq!(cle.cle_id, id);
    assert_eq!(cle.acces.company_id(), c);
    assert_eq!(cle.acces.user_id(), u);
    let attendues: BTreeSet<Portee> = [Portee::BooksRead, Portee::InvoicesRead].into();
    assert_eq!(cle.acces.canal(), &Canal::Mcp { portees: attendues });
    assert_eq!(cle.validation, SendValidation::Required);

    // La politique changée est rendue par la route et par l'authentification.
    let resp = app
        .send(
            "PUT",
            &format!("/api/v1/admin/mcp/keys/{id}/send-validation"),
            Some(&jwt),
            json!({ "sendValidation": "exempt", "version": 1 }),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let modifiee: Value = resp.json().await.unwrap();
    assert_eq!(modifiee["sendValidation"], "exempt");
    assert_eq!(modifiee["version"], 2);
    assert!(
        modifiee.get("key").is_none(),
        "le secret ne se relit jamais"
    );
    let cle = authentifier(&lecture, &trace, &jeton).await.unwrap();
    assert_eq!(cle.validation, SendValidation::Exempt);

    // Une version périmée : 409, rien de changé.
    let (s, k) = code(
        app.send(
            "PUT",
            &format!("/api/v1/admin/mcp/keys/{id}/send-validation"),
            Some(&jwt),
            json!({ "sendValidation": "required", "version": 1 }),
        )
        .await,
    )
    .await;
    assert_eq!((s, k.as_str()), (409, "OPTIMISTIC_LOCK_CONFLICT"));
    let (s, k) = code(
        app.send(
            "DELETE",
            &format!("/api/v1/admin/mcp/keys/{id}"),
            Some(&jwt),
            json!({ "version": 1 }),
        )
        .await,
    )
    .await;
    assert_eq!((s, k.as_str()), (409, "OPTIMISTIC_LOCK_CONFLICT"));
    assert!(authentifier(&lecture, &trace, &jeton).await.is_ok());

    // Révoquée : 204, et elle n'authentifie plus.
    let resp = app
        .send(
            "DELETE",
            &format!("/api/v1/admin/mcp/keys/{id}"),
            Some(&jwt),
            json!({ "version": 2 }),
        )
        .await;
    assert_eq!(resp.status(), 204);
    assert!(matches!(
        authentifier(&lecture, &trace, &jeton).await,
        Err(AppError::Unauthenticated(_))
    ));

    // Une clé révoquée ne change plus de politique (C-32-5-30) et ne se
    // révoque pas deux fois.
    for (method, path, body) in [
        (
            "PUT",
            format!("/api/v1/admin/mcp/keys/{id}/send-validation"),
            json!({ "sendValidation": "required", "version": 3 }),
        ),
        (
            "DELETE",
            format!("/api/v1/admin/mcp/keys/{id}"),
            json!({ "version": 3 }),
        ),
    ] {
        let (s, k) = code(app.send(method, &path, Some(&jwt), body).await).await;
        assert_eq!(
            (s, k.as_str()),
            (409, "OPTIMISTIC_LOCK_CONFLICT"),
            "{method} {path}"
        );
    }

    // La liste la garde, révoquée.
    let liste: Value = app
        .send("GET", "/api/v1/admin/mcp/keys", Some(&jwt), json!({}))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(liste.as_array().unwrap().len(), 1);
    assert!(liste[0]["revokedAt"].is_string());
    assert_eq!(liste[0]["sendValidation"], "exempt");
}

/// AC-c1 — la liste ne porte jamais le secret ni le condensat (Mc2), et ne
/// rend que les clés de la société.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c1_the_list_never_carries_the_secret_nor_its_hash(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _u, jwt) = livres(&pool).await;
    let autre = societe(&pool, "Autre société").await;
    let (_u2, jwt_autre) = session(&pool, autre, "admin-autre", Role::Admin).await;

    let cree = creer(&app, &jwt, serde_json::from_str(BONNE_CLE).unwrap()).await;
    let jeton = cree["key"].as_str().unwrap().to_string();
    let condensat = sha256_hex(&jeton);
    creer(
        &app,
        &jwt_autre,
        json!({"name": "autre", "scopes": ["contacts:read"]}),
    )
    .await;

    let resp = app
        .send("GET", "/api/v1/admin/mcp/keys", Some(&jwt), json!({}))
        .await;
    assert_eq!(resp.status(), 200);
    let texte = resp.text().await.unwrap();
    assert!(!texte.contains(&jeton), "secret dans la liste : {texte}");
    assert!(
        !texte.contains(&condensat),
        "condensat dans la liste : {texte}"
    );
    let liste: Value = serde_json::from_str(&texte).unwrap();
    let liste = liste.as_array().unwrap();
    assert_eq!(liste.len(), 1, "seules les clés de la société : {texte}");
    let ligne = liste[0].as_object().unwrap();
    for interdit in ["key", "keyHash", "key_hash"] {
        assert!(!ligne.contains_key(interdit), "champ {interdit} : {texte}");
    }
    assert_eq!(ligne["name"], "agent");
    assert_eq!(ligne["scopes"], json!(["books:read"]));
    assert_eq!(ligne["unknownScopes"], json!([]));
    assert_eq!(ligne["lastUsedAt"], Value::Null);
}

/// AC-c1 — une clé d'une autre société est indiscernable d'une clé absente :
/// 404 à la révocation comme au changement de politique, et rien ne change.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c1_a_key_of_another_company_is_not_found(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _u, jwt) = livres(&pool).await;
    let autre = societe(&pool, "Autre société").await;
    let (_u2, jwt_autre) = session(&pool, autre, "admin-autre", Role::Admin).await;
    let cree = creer(&app, &jwt_autre, serde_json::from_str(BONNE_CLE).unwrap()).await;
    let id = cree["id"].as_i64().unwrap();

    for (method, path, body) in [
        (
            "DELETE",
            format!("/api/v1/admin/mcp/keys/{id}"),
            json!({ "version": 1 }),
        ),
        (
            "PUT",
            format!("/api/v1/admin/mcp/keys/{id}/send-validation"),
            json!({ "sendValidation": "exempt", "version": 1 }),
        ),
        (
            "DELETE",
            "/api/v1/admin/mcp/keys/999999".to_string(),
            json!({ "version": 1 }),
        ),
    ] {
        let (s, k) = code(app.send(method, &path, Some(&jwt), body).await).await;
        assert_eq!((s, k.as_str()), (404, "NOT_FOUND"), "{method} {path}");
    }
    let (revoquees, exemptes) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT CAST(SUM(revoked_at IS NOT NULL) AS SIGNED), \
                CAST(SUM(send_validation = 'exempt') AS SIGNED) FROM mcp_keys",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((revoquees, exemptes), (0, 0));
}

// ---------------------------------------------------------------------------
// AC-c3 — la validation des clés
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c3_an_invalid_key_request_writes_nothing_and_traces_nothing(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _u, jwt) = livres(&pool).await;
    let passe = (Utc::now() - TimeDelta::minutes(1)).to_rfc3339();
    let long = "n".repeat(256);

    for body in [
        json!({ "name": "", "scopes": ["books:read"] }),
        json!({ "name": "   ", "scopes": ["books:read"] }),
        json!({ "name": "\u{00A0}\u{3000}\u{200B}", "scopes": ["books:read"] }),
        json!({ "name": long, "scopes": ["books:read"] }),
        json!({ "name": "agent", "scopes": [] }),
        json!({ "name": "agent", "scopes": ["books:write"] }),
        json!({ "name": "agent", "scopes": ["admin"] }),
        json!({ "name": "agent", "scopes": ["read"] }),
        json!({ "name": "agent", "scopes": ["books:read", "books:write"] }),
        json!({ "name": "agent", "scopes": ["books:read"], "expiresAt": passe }),
        json!({ "name": "agent", "scopes": ["books:read"], "sendValidation": "none" }),
        json!({ "name": "agent", "scopes": ["books:read"], "sendValidation": "Exempt" }),
    ] {
        let (s, k) = code(
            app.send("POST", "/api/v1/admin/mcp/keys", Some(&jwt), body.clone())
                .await,
        )
        .await;
        assert_eq!((s, k.as_str()), (400, "VALIDATION_ERROR"), "corps {body}");
    }
    // Une version de verrou invalide, avant tout accès.
    let (s, k) = code(
        app.send(
            "DELETE",
            "/api/v1/admin/mcp/keys/1",
            Some(&jwt),
            json!({ "version": 0 }),
        )
        .await,
    )
    .await;
    assert_eq!((s, k.as_str()), (400, "VALIDATION_ERROR"));

    assert_eq!(compte(&pool, "SELECT COUNT(*) FROM mcp_keys").await, 0);
    assert_eq!(
        compte(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'mcp\\_%'"
        )
        .await,
        0,
        "aucun audit pour un refus"
    );

    // Témoin : le même corps, valide, écrit.
    creer(&app, &jwt, serde_json::from_str(BONNE_CLE).unwrap()).await;
    assert_eq!(compte(&pool, "SELECT COUNT(*) FROM mcp_keys").await, 1);
}

/// AC-c3 — le message d'une portée inconnue la nomme et passe par le
/// catalogue (`fr-CH`, langue de l'instance des tests).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c3_an_unknown_scope_is_named_in_the_message(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _u, jwt) = livres(&pool).await;
    let resp = app
        .send(
            "POST",
            "/api/v1/admin/mcp/keys",
            Some(&jwt),
            json!({ "name": "agent", "scopes": ["books:write"] }),
        )
        .await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    let message = body["error"]["message"].as_str().unwrap();
    assert!(message.contains("books:write"), "{message}");
    assert!(message.contains("prompts:write"), "{message}");
    assert!(
        !message.contains('\u{2068}'),
        "marques d'isolation : {message:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-c4 — les réglages
// ---------------------------------------------------------------------------

fn reglages(version: i64, networks: Value) -> Value {
    json!({
        "enabled": true,
        "addressFilterEnabled": true,
        "allowedNetworks": networks,
        "trustedProxies": ["192.168.1.2"],
        "allowedOrigins": ["HTTPS://NAS.Local:8443"],
        "version": version,
    })
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c4_settings_defaults_save_and_optimistic_lock(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (c, _u, jwt) = livres(&pool).await;

    // Sans ligne : les défauts, version 0, et rien d'écrit par la lecture.
    let defauts: Value = app
        .send("GET", "/api/v1/admin/mcp/settings", Some(&jwt), json!({}))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(defauts["enabled"], false);
    assert_eq!(defauts["addressFilterEnabled"], true);
    assert_eq!(defauts["version"], 0);
    assert_eq!(defauts["updatedAt"], Value::Null);
    assert_eq!(defauts["allowedNetworks"].as_array().unwrap().len(), 7);
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_server_settings").await,
        0
    );

    // Premier enregistrement (version 0) : canonique, sans doublon.
    let resp = app
        .send(
            "PUT",
            "/api/v1/admin/mcp/settings",
            Some(&jwt),
            reglages(
                0,
                json!(["10.0.0.0/8", "192.168.1.10", "192.168.1.10/32", "FC00::/7"]),
            ),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let v1: Value = resp.json().await.unwrap();
    assert_eq!(v1["version"], 1);
    assert_eq!(v1["enabled"], true);
    assert_eq!(
        v1["allowedNetworks"],
        json!(["10.0.0.0/8", "192.168.1.10/32", "fc00::/7"])
    );
    assert_eq!(v1["trustedProxies"], json!(["192.168.1.2/32"]));
    assert_eq!(v1["allowedOrigins"], json!(["https://nas.local:8443"]));
    assert!(v1["updatedAt"].is_string());

    // `version: 0` alors qu'une ligne existe : 409, la ligne reste (Mc4).
    let (s, k) = code(
        app.send(
            "PUT",
            "/api/v1/admin/mcp/settings",
            Some(&jwt),
            json!({
                "enabled": false, "addressFilterEnabled": false,
                "allowedNetworks": [], "trustedProxies": [], "allowedOrigins": [],
                "version": 0,
            }),
        )
        .await,
    )
    .await;
    assert_eq!((s, k.as_str()), (409, "OPTIMISTIC_LOCK_CONFLICT"));
    // Version périmée (et version d'avenir) : 409.
    for version in [2, -1] {
        let (s, k) = code(
            app.send(
                "PUT",
                "/api/v1/admin/mcp/settings",
                Some(&jwt),
                reglages(version, json!([])),
            )
            .await,
        )
        .await;
        assert_eq!(
            (s, k.as_str()),
            (409, "OPTIMISTIC_LOCK_CONFLICT"),
            "version {version}"
        );
    }
    let relu: Value = app
        .send("GET", "/api/v1/admin/mcp/settings", Some(&jwt), json!({}))
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(relu["version"], 1);
    assert_eq!(relu["enabled"], true, "la ligne n'a pas été écrasée");
    assert_eq!(relu["allowedNetworks"], v1["allowedNetworks"]);

    // Version courante : écrit.
    let resp = app
        .send(
            "PUT",
            "/api/v1/admin/mcp/settings",
            Some(&jwt),
            reglages(1, json!(["100.64.0.0/10"])),
        )
        .await;
    assert_eq!(resp.status(), 200);
    let v2: Value = resp.json().await.unwrap();
    assert_eq!(v2["version"], 2);
    assert_eq!(v2["allowedNetworks"], json!(["100.64.0.0/10"]));

    // AC-c5 — deux entrées, avant et après.
    let details: Vec<(i64, String)> = sqlx::query_as(
        "SELECT entity_id, CAST(details_json AS CHAR) FROM audit_log \
         WHERE action = 'mcp_server_settings.updated' AND entity_type = 'mcp_server_settings' \
         ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(details.len(), 2, "une entrée par enregistrement réussi");
    let premier: Value = serde_json::from_str(&details[0].1).unwrap();
    assert_eq!(details[0].0, c);
    assert_eq!(premier["before"]["enabled"], false, "avant = défauts");
    assert_eq!(premier["after"]["allowedNetworks"], v1["allowedNetworks"]);
    let second: Value = serde_json::from_str(&details[1].1).unwrap();
    assert_eq!(second["before"]["allowedNetworks"], v1["allowedNetworks"]);
    assert_eq!(second["after"]["allowedNetworks"], json!(["100.64.0.0/10"]));
}

/// AC-c4 — une entrée invalide : 400, rien d'écrit, aucun audit. Le réseau à
/// bits d'hôte est refusé avec sa forme attendue, jamais tronqué (Mc5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c4_an_invalid_entry_writes_nothing(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, _u, jwt) = livres(&pool).await;
    let trop: Vec<String> = (0..65).map(|i| format!("10.0.0.{i}")).collect();
    let longue = format!("https://{}.ch", "a".repeat(250));

    let resp = app
        .send(
            "PUT",
            "/api/v1/admin/mcp/settings",
            Some(&jwt),
            reglages(0, json!(["10.1.2.3/8"])),
        )
        .await;
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let message = body["error"]["message"].as_str().unwrap();
    assert!(message.contains("10.0.0.0/8"), "forme attendue : {message}");

    let mut corps = vec![
        reglages(0, json!(["nas.local"])),
        reglages(0, json!(["10.0.0.0/33"])),
        reglages(0, json!(trop)),
    ];
    let mut origine = reglages(0, json!([]));
    origine["allowedOrigins"] = json!(["https://nas.local/mcp"]);
    corps.push(origine);
    let mut origine = reglages(0, json!([]));
    origine["allowedOrigins"] = json!(["ftp://nas.local"]);
    corps.push(origine);
    let mut origine = reglages(0, json!([]));
    origine["allowedOrigins"] = json!([longue]);
    corps.push(origine);
    let mut mandataire = reglages(0, json!([]));
    mandataire["trustedProxies"] = json!(["192.168.1.0/16"]);
    corps.push(mandataire);
    for body in corps {
        let (s, k) = code(
            app.send(
                "PUT",
                "/api/v1/admin/mcp/settings",
                Some(&jwt),
                body.clone(),
            )
            .await,
        )
        .await;
        assert_eq!((s, k.as_str()), (400, "VALIDATION_ERROR"), "corps {body}");
    }
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_server_settings").await,
        0
    );
    assert_eq!(
        compte(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'mcp\\_%'"
        )
        .await,
        0
    );
}

// ---------------------------------------------------------------------------
// AC-c5 — l'audit
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c5_the_key_audit_entries_never_carry_the_secret_nor_its_hash(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (_c, u, jwt) = livres(&pool).await;
    let cree = creer(
        &app,
        &jwt,
        json!({ "name": "agent", "scopes": ["documents:read", "books:read"], "sendValidation": "exempt" }),
    )
    .await;
    let jeton = cree["key"].as_str().unwrap().to_string();
    let condensat = sha256_hex(&jeton);
    let id = cree["id"].as_i64().unwrap();
    assert_eq!(
        app.send(
            "PUT",
            &format!("/api/v1/admin/mcp/keys/{id}/send-validation"),
            Some(&jwt),
            json!({ "sendValidation": "required", "version": 1 }),
        )
        .await
        .status(),
        200
    );
    assert_eq!(
        app.send(
            "DELETE",
            &format!("/api/v1/admin/mcp/keys/{id}"),
            Some(&jwt),
            json!({ "version": 2 }),
        )
        .await
        .status(),
        204
    );

    let lignes: Vec<(String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, user_id, CAST(details_json AS CHAR) FROM audit_log \
         WHERE entity_type = 'mcp_key' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let actions: Vec<&str> = lignes.iter().map(|l| l.0.as_str()).collect();
    assert_eq!(
        actions,
        vec![
            "mcp_key.created",
            "mcp_key.send_validation_changed",
            "mcp_key.revoked"
        ]
    );
    for (action, _, entity_id, user_id, details) in &lignes {
        assert_eq!((*entity_id, *user_id), (id, u), "{action}");
        assert!(!details.contains(&jeton), "{action} : secret au détail");
        assert!(
            !details.contains(&condensat),
            "{action} : condensat au détail"
        );
        assert!(
            !details.contains(MCP_PREFIX),
            "{action} : préfixe au détail"
        );
    }
    let cree: Value = serde_json::from_str(&lignes[0].4).unwrap();
    assert_eq!(
        cree,
        json!({ "name": "agent", "scopes": ["books:read", "documents:read"], "sendValidation": "exempt" })
    );
    let change: Value = serde_json::from_str(&lignes[1].4).unwrap();
    assert_eq!(
        change,
        json!({ "name": "agent", "before": "exempt", "after": "required" })
    );
    let revoquee: Value = serde_json::from_str(&lignes[2].4).unwrap();
    assert_eq!(
        revoquee,
        json!({ "name": "agent", "scopes": ["books:read", "documents:read"], "sendValidation": "required" })
    );
    // Nulle part au journal, quelle que soit l'entrée.
    let partout = compte(
        &pool,
        &format!(
            "SELECT COUNT(*) FROM audit_log WHERE details_json LIKE '%{condensat}%' \
             OR details_json LIKE '%{jeton}%'"
        ),
    )
    .await;
    assert_eq!(partout, 0);
}

// ---------------------------------------------------------------------------
// AC-c2 — les gardes (le PAT : `admin_pat_denied_e2e`)
// ---------------------------------------------------------------------------

const COUPLES: [(&str, &str); 6] = [
    ("GET", "/api/v1/admin/mcp/keys"),
    ("POST", "/api/v1/admin/mcp/keys"),
    ("DELETE", "/api/v1/admin/mcp/keys/1"),
    ("PUT", "/api/v1/admin/mcp/keys/1/send-validation"),
    ("GET", "/api/v1/admin/mcp/settings"),
    ("PUT", "/api/v1/admin/mcp/settings"),
];

/// Un corps valide pour chaque couple : un refus ne peut venir que d'une garde.
fn corps_valide(method: &str, path: &str) -> Value {
    match (method, path) {
        ("POST", _) => serde_json::from_str(BONNE_CLE).unwrap(),
        ("DELETE", _) => json!({ "version": 1 }),
        ("PUT", p) if p.ends_with("send-validation") => {
            json!({ "sendValidation": "exempt", "version": 1 })
        }
        ("PUT", _) => reglages(0, json!([])),
        _ => json!({}),
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn c2_a_comptable_an_mcp_key_and_no_authentication_are_refused_everywhere(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let (c, _u, jwt) = livres(&pool).await;
    let (_uc, jwt_comptable) = session(&pool, c, "comptable", Role::Comptable).await;
    let cree = creer(&app, &jwt, serde_json::from_str(BONNE_CLE).unwrap()).await;
    let jeton_mcp = cree["key"].as_str().unwrap().to_string();
    assert_eq!(
        cree["id"], 1,
        "montage : la clé 1 existe, les couples la visent"
    );

    for (method, path) in COUPLES {
        let corps = corps_valide(method, path);
        let (s, k) = code(
            app.send(method, path, Some(&jwt_comptable), corps.clone())
                .await,
        )
        .await;
        assert_eq!(
            (s, k.as_str()),
            (403, "FORBIDDEN"),
            "Comptable : {method} {path}"
        );
        let resp = app
            .send(method, path, Some(&jeton_mcp), corps.clone())
            .await;
        assert_eq!(resp.status(), 401, "clé MCP : {method} {path}");
        let resp = app.send(method, path, None, corps).await;
        assert_eq!(
            resp.status(),
            401,
            "sans authentification : {method} {path}"
        );
    }
    // Rien n'a bougé : la clé est intacte, aucun réglage, une seule entrée.
    assert_eq!(compte(&pool, "SELECT COUNT(*) FROM mcp_keys").await, 1);
    assert_eq!(
        compte(
            &pool,
            "SELECT COUNT(*) FROM mcp_keys WHERE revoked_at IS NULL AND version = 1 \
             AND send_validation = 'required'"
        )
        .await,
        1
    );
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_server_settings").await,
        0
    );
    assert_eq!(
        compte(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE action LIKE 'mcp\\_%'"
        )
        .await,
        1
    );

    // Témoin : l'Administrateur passe sur chacun des six couples, avec les mêmes
    // corps — la politique avant la révocation, qui consomme la version 2.
    for (method, path) in [COUPLES[0], COUPLES[1], COUPLES[3], COUPLES[4], COUPLES[5]] {
        let resp = app
            .send(method, path, Some(&jwt), corps_valide(method, path))
            .await;
        assert!(
            resp.status().is_success(),
            "Administrateur : {method} {path} → {}",
            resp.status()
        );
    }
    {
        let (method, path) = COUPLES[2];
        let resp = app
            .send(method, path, Some(&jwt), json!({ "version": 2 }))
            .await;
        assert!(
            resp.status().is_success(),
            "Administrateur : {method} {path} → {}",
            resp.status()
        );
    }
}
