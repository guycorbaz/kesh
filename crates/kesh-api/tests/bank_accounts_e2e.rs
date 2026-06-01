//! Tests E2E HTTP pour Story 8-5a-zero — foundation
//! `bank_account.journal_account_id` link.
//!
//! Couvre les ACs #75-#82 :
//! - GET  /api/v1/bank-accounts (list, all roles)
//! - PATCH /api/v1/bank-accounts/{id} (link/unlink, Comptable+ uniquement)
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test
//! avec migrations auto-appliquées).
//!
//! Pattern hérité de `reconciliation_e2e.rs` (8-4) — `spawn_app` + JWT
//! forgé directement avec le secret de test.
#![allow(clippy::too_many_arguments)]

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

// ============================================================
// Spawn helpers (pattern reconciliation_e2e.rs)
// ============================================================

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

// ============================================================
// Domain seed helpers
// ============================================================

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

async fn create_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "UBS".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: true,
        },
    )
    .await
    .unwrap()
    .id
}

async fn create_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
        },
    )
    .await
    .unwrap()
    .id
}

async fn archive_account(pool: &MySqlPool, account_id: i64, user_id: i64) {
    accounts::archive(pool, account_id, 1, user_id)
        .await
        .expect("archive account");
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

struct Ctx {
    company_id: i64,
    user_id: i64,
    bank_account_id: i64,
    asset_account_id: i64,
    jwt: String,
}

async fn setup_full(pool: &MySqlPool, label: &str, iban: &str, role: Role) -> Ctx {
    let company_id = create_company(pool, label).await;
    let username = format!("{label}_user");
    let user_id = create_user(pool, &username, role, company_id).await;
    let bank_account_id = create_bank_account(pool, company_id, iban).await;
    let asset_account_id = create_account(
        pool,
        company_id,
        user_id,
        "1020",
        "Caisse banque",
        AccountType::Asset,
    )
    .await;
    let jwt = forge_jwt(user_id, role_str(role), company_id);
    Ctx {
        company_id,
        user_id,
        bank_account_id,
        asset_account_id,
        jwt,
    }
}

async fn read_bank_account_version(pool: &MySqlPool, id: i64) -> i32 {
    sqlx::query_scalar::<_, i32>("SELECT version FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ============================================================
// AC #78 — Happy path PATCH link
// ============================================================

/// AC #78 — Comptable+ peut lier un bank_account à un compte 1020 Asset
/// actif. Le PATCH retourne 200 + entité mise à jour avec
/// `journalAccountId` posé et version bumpée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_links_journal_account_returns_200_with_updated_entity(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;

    assert_eq!(resp.status(), 200, "PATCH should succeed");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["id"].as_i64().unwrap(), ctx.bank_account_id);
    assert_eq!(
        body["journalAccountId"].as_i64().unwrap(),
        ctx.asset_account_id
    );
    assert_eq!(body["version"].as_i64().unwrap(), (version + 1) as i64);

    // Vérifier audit_log inscrit.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'bank_account' AND entity_id = ? AND action = 'bank_account.updated'",
    )
    .bind(ctx.bank_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit_count, 1,
        "audit_log bank_account.updated should be inserted"
    );
}

// ============================================================
// AC #79 — Archived account → 404 ACCOUNT_NOT_FOUND (anti-énumération)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_rejects_archived_account_with_404(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;

    // Archive l'asset account.
    archive_account(&pool, ctx.asset_account_id, ctx.user_id).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;

    assert_eq!(
        resp.status(),
        404,
        "archived account → 404 anti-énumération"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
}

// ============================================================
// AC #80 — Wrong account type (Revenue) → 400 INVALID_ACCOUNT_TYPE
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_rejects_revenue_account_with_400_invalid_type(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let revenue_id = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": revenue_id,
            "version": version,
        }),
    )
    .await;

    assert_eq!(
        resp.status(),
        400,
        "Revenue account → 400 INVALID_ACCOUNT_TYPE"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_ACCOUNT_TYPE");
    assert_eq!(body["error"]["details"]["accountType"], "Revenue");
    assert_eq!(
        body["error"]["details"]["allowedTypes"],
        json!(["Asset", "Liability"])
    );
}

// ============================================================
// AC #81 — Multi-tenant safety : compte d'une autre company → 404
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_does_not_leak_cross_tenant_account(pool: MySqlPool) {
    let ctx_a = setup_full(&pool, "AcmeA", "CH4431999123000889012", Role::Comptable).await;
    // Company B avec son propre asset account 1020.
    let company_b = create_company(&pool, "AcmeB").await;
    let user_b = create_user(&pool, "user_b", Role::Comptable, company_b).await;
    let asset_b = create_account(
        &pool,
        company_b,
        user_b,
        "1020",
        "Caisse banque B",
        AccountType::Asset,
    )
    .await;

    let version = read_bank_account_version(&pool, ctx_a.bank_account_id).await;
    // user_a tente de lier son bank_account à un asset account de company B → 404.
    let resp = ctx_patch(
        &pool,
        &ctx_a,
        json!({
            "journalAccountId": asset_b,
            "version": version,
        }),
    )
    .await;

    assert_eq!(resp.status(), 404, "cross-tenant account → 404 (KF-002)");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_FOUND");
}

// ============================================================
// AC #82 — RBAC : Consultation → 403 Forbidden
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_requires_comptable_role(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;

    assert_eq!(resp.status(), 403, "Consultation → 403 Forbidden");
}

// ============================================================
// GET list — 200 + journalAccountId remonté
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_bank_accounts_returns_journal_account_id_when_set(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    // Lie le bank_account au compte.
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;
    assert_eq!(resp.status(), 200);

    // GET list.
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let arr = body.as_array().expect("list array");
    assert_eq!(arr.len(), 1);
    assert_eq!(
        arr[0]["journalAccountId"].as_i64().unwrap(),
        ctx.asset_account_id
    );
    assert_eq!(arr[0]["bankName"], "UBS");
}

/// Helper : exécute le PATCH avec le JWT du ctx.
async fn ctx_patch(pool: &MySqlPool, ctx: &Ctx, body: Value) -> reqwest::Response {
    let app = spawn_app(pool.clone()).await;
    app.client
        .patch(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap()
}

// ============================================================
// AC #78 (P-M2 Pass 1 code review Sonnet 4.6) — PATCH sur un
// bank_account_id inexistant retourne 404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND.
// Code partagé v0.1 (dette naming L64 — renommer en BANK_ACCOUNT_NOT_FOUND
// ou créer un variant dédié en v0.2 si le frontend distingue les contextes).
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_returns_404_bank_account_not_found_for_unknown_id(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Body valide structurellement (passe l'extracteur), mais bank_account_id
    // n'existe pas → 404.
    let resp = app
        .client
        .patch(app.url("/api/v1/bank-accounts/999999"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "journalAccountId": ctx.asset_account_id,
            "version": 1,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 404, "unknown bank_account_id → 404");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND",
        "code partagé v0.1 (dette L64)"
    );
}

// ============================================================
// P-H1 Pass 1 code review Sonnet 4.6 — body PATCH malformé doit
// retourner 400 VALIDATION_ERROR (standard Kesh) et non le 422 Axum natif.
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_malformed_body_returns_400_validation_error(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Champ `version` manquant (struct exige journalAccountId + version).
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "journalAccountId": ctx.asset_account_id,
            // pas de version
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "body malformé doit retourner 400, pas le 422 Axum natif"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "VALIDATION_ERROR",
        "shape standard Kesh {{error:{{code,message}}}}"
    );
}

// ============================================================
// Pass 3 Opus 4.7 (BH2-M-3 confirmation) — double PATCH idempotent
// avec MÊME journalAccountId ET version mise à jour entre les deux
// appels DOIT laisser audit_count = 1.
//
// Ce test valide end-to-end le pattern KF-004 court-circuit no-op +
// la correction P-C1 (tuple `(updated, before)` atomique) :
// - 1er PATCH : link → version 1 → 2, audit_log inséré.
// - 2nd PATCH : même journalAccountId, version 2 (à jour) → no-op
//   short-circuit (version 2 → 2), audit_log NON inséré.
//
// Garde-fou contre régression future si le handler oublie le
// `if updated.version != before.version` ou si le repo perd le
// court-circuit no-op.
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn patch_bank_account_idempotent_no_op_does_not_duplicate_audit_log(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let pre_version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    // 1er PATCH : link.
    let resp1 = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": pre_version,
        }),
    )
    .await;
    assert_eq!(resp1.status(), 200);
    let body1: Value = resp1.json().await.unwrap();
    let v1 = body1["version"].as_i64().unwrap();
    assert_eq!(v1, (pre_version + 1) as i64);

    // 2nd PATCH : même journalAccountId, version mise à jour → no-op
    // short-circuit (handler skip audit_log car updated.version == before.version).
    let resp2 = ctx_patch(
        &pool,
        &ctx,
        json!({
            "journalAccountId": ctx.asset_account_id,
            "version": v1,
        }),
    )
    .await;
    assert_eq!(resp2.status(), 200, "no-op idempotent doit retourner 200");
    let body2: Value = resp2.json().await.unwrap();
    assert_eq!(
        body2["version"].as_i64().unwrap(),
        v1,
        "no-op : version inchangée"
    );

    // audit_count doit rester à 1 (un seul audit_log écrit pour le 1er PATCH).
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'bank_account' AND entity_id = ? AND action = 'bank_account.updated'",
    )
    .bind(ctx.bank_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit_count, 1,
        "double PATCH idempotent : audit_count doit rester 1 (KF-004 court-circuit)"
    );
}

// ===========================================================================
// Story v014-1 — Tests intégration POST/PUT/DELETE (F6 Pass 1 code review)
// ===========================================================================

/// Marque onboarding step_completed=7 pour permettre les CRUD post-onboarding.
/// La table `onboarding_state` est singleton (UNIQUE constraint sur `singleton`).
async fn complete_onboarding(pool: &MySqlPool) {
    sqlx::query(
        "INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode) \
         VALUES (TRUE, 7, FALSE, 'guided') \
         ON DUPLICATE KEY UPDATE step_completed = 7, version = version + 1",
    )
    .execute(pool)
    .await
    .unwrap();
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_bank_account_happy_path_returns_201_with_entity(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
            "isPrimary": false,
        }))
        .send()
        .await
        .unwrap();

    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 201, "body: {body}");
    assert_eq!(body["bankName"], "PostFinance");
    assert_eq!(body["iban"], "CH3908704016075473007");
    assert_eq!(body["isPrimary"], false);
    assert_eq!(body["archived"], false);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_bank_account_invalid_iban_returns_400(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "Bad Bank",
            "iban": "INVALID_IBAN",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_bank_account_without_onboarding_complete_returns_412(pool: MySqlPool) {
    // Ne PAS appeler complete_onboarding — step_completed < 7.
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 412);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ONBOARDING_NOT_COMPLETE");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_bank_account_consultation_role_returns_403(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let app = spawn_app(pool.clone()).await;

    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 403);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn post_bank_account_primary_collision_silently_demotes_old(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Premier compte est primary (setup_full crée bank_account avec is_primary=true).
    // Vérifier.
    let initial_primary_is_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(initial_primary_is_primary);

    // POST un nouveau compte avec isPrimary=true → transition silencieuse atomique.
    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
            "isPrimary": true,
        }))
        .send()
        .await
        .unwrap();

    // F3 Pass 3 Opus : pas de 409 — transition silencieuse atomique.
    assert_eq!(resp.status(), 201);

    // L'ancien primary a été démoté.
    let old_is_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(
        !old_is_primary,
        "ancien primary doit être démoté silencieusement"
    );

    // Audit log primary_transition présent.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE action = 'bank_account.updated' \
         AND entity_id = ? \
         AND JSON_EXTRACT(details_json, '$.trigger') = 'primary_transition'",
    )
    .bind(ctx.bank_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit_count, 1,
        "audit log primary_transition doit être présent"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_bank_account_happy_path_full_update(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "Updated Bank",
            "iban": "CH3908704016075473007",
            "isPrimary": true,
            "version": version,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["bankName"], "Updated Bank");
    assert_eq!(body["iban"], "CH3908704016075473007");
    assert_eq!(body["version"], version + 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn put_bank_account_stale_version_returns_409(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // version=0 → stale (la version réelle est 1+).
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "Stale",
            "iban": "CH3908704016075473007",
            "version": 0,
        }))
        .send()
        .await
        .unwrap();

    // version < 1 → 400 Validation (handler check), pas 409.
    assert_eq!(resp.status(), 400);

    // Avec version=99 stale (mais valide).
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "Stale",
            "iban": "CH3908704016075473007",
            "version": 99,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 409);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_bank_account_happy_path_archive(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["archived"], true);

    // Audit log bank_account.archived présent.
    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE action = 'bank_account.archived' AND entity_id = ?",
    )
    .bind(ctx.bank_account_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit_count, 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_bank_account_with_transactions_returns_412(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Insère un bank_import puis une bank_transaction directement (bypass parser).
    let import_id: i64 = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'test.csv', 'a1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd', 'CSV', '2026-01-01', '2026-01-31', 1, ?)",
    )
    .bind(ctx.company_id)
    .bind(ctx.bank_account_id)
    .bind(ctx.user_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO bank_transactions \
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, status) \
         VALUES (?, ?, ?, '2026-01-15', 100.00, 'CHF', 'test tx', 'pending')",
    )
    .bind(ctx.company_id)
    .bind(import_id)
    .bind(ctx.bank_account_id)
    .execute(&pool)
    .await
    .unwrap();

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 412);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_ACCOUNT_HAS_TRANSACTIONS");
    assert_eq!(body["error"]["details"]["transactionCount"], 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_primary_with_other_active_returns_412(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Crée un 2e compte actif (le 1er ctx.bank_account_id reste primary).
    let _second_id = create_bank_account(&pool, ctx.company_id, "CH3908704016075473007").await;

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 412);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_primary_unique_is_allowed_ac10(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    // Aucun autre compte actif → AC#10 autorise l'archivage du primary unique.
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
}
