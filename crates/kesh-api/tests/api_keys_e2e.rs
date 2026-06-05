//! Tests E2E HTTP pour Story 17-2a — API externe à clé PAT (#100).
//!
//! Couvre :
//! - AC4/AC5 : auth Bearer PAT (happy path, token inconnu, créateur inactif,
//!   clé expirée, clé révoquée).
//! - AC6/DC3 : gate de scope (`read` + méthode mutante → 403 API_KEY_READ_ONLY).
//! - AC7/DC6 : gestion interdite via PAT (403 API_KEY_MANAGEMENT_FORBIDDEN).
//! - AC2/AC7 : création (secret une seule fois), liste (jamais le hash).
//! - AC8/DC5 : audit `actor_type='api_key'` sur une mutation via PAT.
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).
//! Pattern hérité de `bank_accounts_e2e.rs`.

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::{Language, NewAccount, NewBankAccount, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{accounts, bank_accounts, companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

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
            Path::new(env!("CARGO_MANIFEST_DIR"))
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
        users_exist: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
    };
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
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
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
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

async fn create_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            address: "Rue Test 1".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
        },
    )
    .await
    .unwrap()
    .id
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    jwt: String,
}

async fn setup(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let company_id = create_company(pool, label).await;
    let user_id = create_user(pool, &format!("{label}_user"), role, company_id).await;
    let jwt = forge_jwt(user_id, role.as_str(), company_id);
    Ctx {
        company_id,
        user_id,
        jwt,
    }
}

/// Crée une clé via l'endpoint HTTP (JWT) et retourne le secret clair `kesh_pat_…`.
async fn create_key_via_http(app: &TestApp, jwt: &str, name: &str, scope: &str) -> (i64, String) {
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&json!({ "name": name, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé doit réussir");
    let body: Value = resp.json().await.unwrap();
    let id = body["id"].as_i64().unwrap();
    let key = body["key"].as_str().unwrap().to_string();
    assert!(key.starts_with("kesh_pat_"), "secret au format attendu");
    (id, key)
}

// ============================================================
// AC2/AC7 — création (secret une fois) + liste (jamais le hash)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_returns_secret_once_and_list_hides_hash(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;

    let (id, key) = create_key_via_http(&app, &ctx.jwt, "CI integration", "read-write").await;

    // La liste ne doit jamais exposer le hash ni le secret.
    let resp = app
        .client
        .get(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let raw = resp.text().await.unwrap();
    assert!(raw.contains("\"id\":"), "liste non vide");
    assert!(!raw.contains("keyHash"), "le hash ne doit pas être sérialisé");
    assert!(!raw.contains(&key), "le secret ne doit jamais réapparaître");

    let body: Value = serde_json::from_str(&raw).unwrap();
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["id"].as_i64().unwrap(), id);
    assert_eq!(body[0]["scope"].as_str().unwrap(), "read-write");

    // Audit api_key.created (actor_type='user').
    let actor: String = sqlx::query_scalar(
        "SELECT actor_type FROM audit_log WHERE action = 'api_key.created' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor, "user", "création de clé auditée comme user");
}

// ============================================================
// AC4/AC5 — auth Bearer PAT happy path
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn read_write_pat_can_call_protected_get(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (_id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "PAT valide → accès GET protégé");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unknown_pat_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let _ctx = setup(&pool, "Acme", Role::Comptable).await;

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", "Bearer kesh_pat_thisdoesnotexist000000")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "PAT inconnu → 401");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn inactive_creator_pat_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (_id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    // Désactive le créateur → le PAT doit être invalidé immédiatement (DC2/AC5).
    sqlx::query("UPDATE users SET active = FALSE WHERE id = ?")
        .bind(ctx.user_id)
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "créateur inactif → PAT 401");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expired_pat_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    // Force une expiration dans le passé.
    sqlx::query("UPDATE api_keys SET expires_at = NOW(3) - INTERVAL 1 DAY WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "clé expirée → 401");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn revoked_pat_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    // Révoque via l'endpoint HTTP (JWT).
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/settings/api-keys/{id}")))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&json!({ "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204, "révocation → 204");

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "clé révoquée → 401 dès la requête suivante");

    // Audit api_key.revoked.
    let actor: String = sqlx::query_scalar(
        "SELECT actor_type FROM audit_log WHERE action = 'api_key.revoked' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actor, "user");
}

// ============================================================
// AC6/DC3 — gate de scope
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn read_scope_pat_post_returns_403_read_only(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (_id, key) = create_key_via_http(&app, &ctx.jwt, "ro", "read").await;

    // Une méthode mutante (POST) avec une clé read → 403 API_KEY_READ_ONLY,
    // AVANT toute logique métier.
    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .json(&json!({ "bankName": "X", "iban": "CH4431999123000889012" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "read + POST → 403");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"].as_str().unwrap(), "API_KEY_READ_ONLY");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn read_scope_pat_get_allowed(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (_id, key) = create_key_via_http(&app, &ctx.jwt, "ro", "read").await;

    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "read + GET → autorisé");
}

// ============================================================
// AC7/DC6 — gestion interdite via PAT
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn read_write_pat_cannot_manage_keys(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;
    let (_id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    // Même une clé read-write ne peut pas lister les clés (anti auto-propagation).
    let resp = app
        .client
        .get(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "PAT sur gestion des clés → 403");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"].as_str().unwrap(),
        "API_KEY_MANAGEMENT_FORBIDDEN"
    );

    // Et ne peut pas créer une clé non plus.
    let resp = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {key}"))
        .json(&json!({ "name": "clone", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "PAT ne peut pas créer de clé");
}

// ============================================================
// AC8/DC5 — audit actor_type='api_key' sur mutation via PAT
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn pat_mutation_is_audited_as_api_key(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = setup(&pool, "Acme", Role::Comptable).await;

    // Bank account + compte d'actif pour le PATCH (pas de gate onboarding sur PATCH).
    let bank_account_id = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: ctx.company_id,
            bank_name: "UBS".into(),
            iban: "CH4431999123000889012".into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id;
    let asset_account_id = accounts::create(
        &pool,
        ctx.user_id,
        NewAccount {
            company_id: ctx.company_id,
            number: "1020".into(),
            name: "Caisse".into(),
            account_type: AccountType::Asset,
            parent_id: None,
        },
    )
    .await
    .unwrap()
    .id;

    let (key_id, key) = create_key_via_http(&app, &ctx.jwt, "rw", "read-write").await;

    // Mutation via PAT : PATCH /bank-accounts/{id} (handler utilise from_current_user).
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/bank-accounts/{bank_account_id}")))
        .header("Authorization", format!("Bearer {key}"))
        .json(&json!({ "journalAccountId": asset_account_id, "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "PATCH via PAT read-write → 200");

    // L'audit de la mutation doit porter actor_type='api_key' + actor_api_key_id.
    let row: (String, Option<i64>, i64) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, user_id FROM audit_log \
         WHERE entity_type = 'bank_account' AND entity_id = ? AND action = 'bank_account.updated' \
         ORDER BY id DESC LIMIT 1",
    )
    .bind(bank_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0, "api_key", "mutation PAT auditée comme api_key");
    assert_eq!(row.1, Some(key_id), "actor_api_key_id = id de la clé");
    assert_eq!(
        row.2, ctx.user_id,
        "user_id = créateur de la clé (imputabilité)"
    );
}

// ============================================================
// AC3 — isolation multi-tenant (cross-company → invisible / 404)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn keys_are_company_scoped(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let a = setup(&pool, "CompanyA", Role::Comptable).await;
    let b = setup(&pool, "CompanyB", Role::Comptable).await;

    let (id_a, _key_a) = create_key_via_http(&app, &a.jwt, "a-key", "read-write").await;

    // B ne voit pas la clé de A.
    let resp = app
        .client
        .get(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {}", b.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(
        body.as_array().unwrap().is_empty(),
        "B ne doit voir aucune clé de A (scoping per-company)"
    );

    // B ne peut pas révoquer la clé de A → 404 (anti-énumération KF-002).
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/settings/api-keys/{id_a}")))
        .header("Authorization", format!("Bearer {}", b.jwt))
        .json(&json!({ "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404, "révocation cross-company → 404");
}
