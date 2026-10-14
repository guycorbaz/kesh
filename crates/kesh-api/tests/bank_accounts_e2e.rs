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
use kesh_db::entities::address::StructuredAddress;
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
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
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
            email: None,
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
            role: None,
            postable: true,
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
#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

/// Story 29-4b (T6-bis, R-2 de la passe ciblée P2 de la 29-4a, #651) — la
/// garde « principal + autres comptes actifs » de l'archivage décide sur
/// l'état **lu dans sa transaction**, sous le `FOR UPDATE` de la ligne, non
/// sur le `is_primary` du pré-vol (lu sur le pool, hors transaction, et que
/// le rejeu réutilisait à chaque tentative).
///
/// Montage : `B` est principal au pré-vol ; une transaction de test `T` tient
/// la sentinelle `companies` ; l'archivage de `B` est lancé, fait son pré-vol
/// (`is_primary = TRUE`), puis attend la sentinelle ; `T` transfère alors le
/// statut de principal à `A` comme le ferait `flip_primary_off_for_company`
/// (les deux versions augmentent) et commite. L'archivage reprend : `B` n'est
/// plus principal, et sa version a bougé.
///
/// **Au sol** (garde sur la valeur du pré-vol) : `412
/// BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY` — refus d'un compte qui n'est plus
/// principal. **Attendu** : la garde relit `is_primary = FALSE` et passe, puis
/// le verrou optimiste d'`archive_for_company` refuse en `409
/// OPTIMISTIC_LOCK_CONFLICT`, le seul refus juste (le compte a changé depuis
/// que le client l'a lu). Rien n'est archivé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn archive_rereads_the_primary_flag_inside_its_transaction(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let a = ctx.bank_account_id;
    let b = create_bank_account(&pool, ctx.company_id, "CH3908704016075473007").await;
    // `B` principal, `A` actif non principal.
    sqlx::query("UPDATE bank_accounts SET is_primary = (id = ?) WHERE company_id = ?")
        .bind(b)
        .bind(ctx.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let version_b = read_bank_account_version(&pool, b).await;

    // `T` tient la sentinelle de la société.
    let mut t = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(ctx.company_id)
        .fetch_one(&mut *t)
        .await
        .unwrap();

    let client = app.client.clone();
    let url = app.url(&format!("/api/v1/bank-accounts/{b}"));
    let jwt = ctx.jwt.clone();
    let archivage = tokio::spawn(async move {
        client
            .delete(url)
            .bearer_auth(&jwt)
            .json(&json!({ "version": version_b }))
            .send()
            .await
            .unwrap()
    });
    // Le pré-vol a eu lieu (il précède la sentinelle) ; l'archivage attend.
    assert!(
        kesh_db::test_fixtures::attendre_une_requete_en_cours(
            &pool,
            &["FROM companies WHERE id", "FOR UPDATE"],
            || archivage.is_finished(),
        )
        .await,
        "⛔ montage : l'archivage devait attendre la sentinelle tenue par T"
    );

    // Transfert du principal, comme `flip_primary_off_for_company` + `PUT`.
    sqlx::query(
        "UPDATE bank_accounts SET is_primary = (id = ?), version = version + 1 \
         WHERE company_id = ? AND id IN (?, ?)",
    )
    .bind(a)
    .bind(ctx.company_id)
    .bind(a)
    .bind(b)
    .execute(&mut *t)
    .await
    .unwrap();
    t.commit().await.unwrap();

    let resp = archivage.await.unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        (status.as_u16(), body["error"]["code"].as_str()),
        (409, Some("OPTIMISTIC_LOCK_CONFLICT")),
        "⛔ la garde du principal doit relire is_primary dans sa transaction : {body}"
    );
    let archived: bool = sqlx::query_scalar("SELECT archived FROM bank_accounts WHERE id = ?")
        .bind(b)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(!archived, "⛔ rien ne doit être archivé");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

// ===========================================================================
// Story 25-6-a (#389) — non-régression des soldes, écrite et verte CONTRE LE
// CODE D'AVANT la réécriture de `list_by_company_with_balances` (commit
// séparé). Elle fige ce que la route rend aujourd'hui, au JSON près : une
// assertion sur des `Decimal` ne verrait pas `"0"` devenir `"0.0000"`.
// ===========================================================================

async fn insert_fiscal_year_2026(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Écriture équilibrée à deux lignes, en SQL brut. Renvoie son id.
#[allow(clippy::too_many_arguments)]
async fn insert_entry(
    pool: &MySqlPool,
    company_id: i64,
    fiscal_year_id: i64,
    number: i64,
    date: &str,
    debit_account: i64,
    credit_account: i64,
    amount: &str,
) -> i64 {
    let entry_id = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, ?, 'Banque', 'écriture de test')",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .bind(number)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (order, account, debit, credit) in [
        (1, debit_account, amount, "0"),
        (2, credit_account, "0", amount),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(account)
        .bind(order)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .unwrap();
    }
    entry_id
}

async fn add_bank_account(pool: &MySqlPool, company_id: i64, iban: &str) -> i64 {
    bank_accounts::create(
        pool,
        NewBankAccount {
            company_id,
            bank_name: "PostFinance".into(),
            iban: iban.into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id
}

async fn link(pool: &MySqlPool, bank_account_id: i64, account_id: i64) {
    sqlx::query("UPDATE bank_accounts SET journal_account_id = ? WHERE id = ?")
        .bind(account_id)
        .bind(bank_account_id)
        .execute(pool)
        .await
        .unwrap();
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn list_bank_accounts_balances_non_regression(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let fy = insert_fiscal_year_2026(&pool, ctx.company_id).await;
    let revenue = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "3200",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let other_asset = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "1021",
        "Banque 2",
        AccountType::Asset,
    )
    .await;

    // A : lié, deux écritures (+500, −120).
    link(&pool, ctx.bank_account_id, ctx.asset_account_id).await;
    let e1 = insert_entry(
        &pool,
        ctx.company_id,
        fy,
        1,
        "2026-03-10",
        ctx.asset_account_id,
        revenue,
        "500.00",
    )
    .await;
    insert_entry(
        &pool,
        ctx.company_id,
        fy,
        2,
        "2026-04-02",
        revenue,
        ctx.asset_account_id,
        "120.00",
    )
    .await;
    // B : lié, aucune écriture. C : non lié.
    let b = add_bank_account(&pool, ctx.company_id, "CH9300762011623852957").await;
    link(&pool, b, other_asset).await;
    let c = add_bank_account(&pool, ctx.company_id, "CH5604835012345678009").await;

    // Une transaction RAPPROCHÉE présente sur A : la réécriture ne doit pas
    // multiplier les lignes du grand livre par les transactions.
    let import_id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', 'b1b2c3d4e5f6789012345678901234567890123456789012345678901234abcd', \
                 'CAMT053', '2026-03-01', '2026-03-31', 500.00, 1, ?)",
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
         (company_id, import_id, bank_account_id, booking_date, amount, currency, details, status, matched_entry_id) \
         VALUES (?, ?, ?, '2026-03-10', 500.00, 'CHF', 'tx', 'reconciled', ?)",
    )
    .bind(ctx.company_id)
    .bind(import_id)
    .bind(ctx.bank_account_id)
    .bind(e1)
    .execute(&pool)
    .await
    .unwrap();

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
    let got: Vec<(i64, Value, Value)> = body
        .as_array()
        .expect("liste")
        .iter()
        .map(|r| {
            (
                r["id"].as_i64().unwrap(),
                r["currentBalance"].clone(),
                r["lastTransactionDate"].clone(),
            )
        })
        .collect();
    assert_eq!(
        got,
        vec![
            (ctx.bank_account_id, json!("380.0000"), json!("2026-04-02")),
            (b, json!("0"), Value::Null),
            (c, Value::Null, Value::Null),
        ],
        "soldes, dates et ordre de la liste, au JSON près"
    );
}

/// Story 25-6-a (#389) — les trois champs neufs traversent la frontière HTTP, en
/// chaînes décimales (jamais des nombres JSON).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn list_bank_accounts_exposes_the_last_statement(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let fy = insert_fiscal_year_2026(&pool, ctx.company_id).await;
    let revenue = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "3200",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    link(&pool, ctx.bank_account_id, ctx.asset_account_id).await;
    insert_entry(
        &pool,
        ctx.company_id,
        fy,
        1,
        "2026-03-10",
        ctx.asset_account_id,
        revenue,
        "500.00",
    )
    .await;
    sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', REPEAT('c', 64), 'CAMT053', '2026-03-01', '2026-03-31', 480.00, 0, ?)",
    )
    .bind(ctx.company_id)
    .bind(ctx.bank_account_id)
    .bind(ctx.user_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool.clone()).await;
    let body: Value = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let a = &body.as_array().expect("liste")[0];
    assert_eq!(a["statementClosingBalance"], json!("480.00"));
    assert_eq!(a["statementDate"], json!("2026-03-31"));
    assert_eq!(a["ledgerBalanceAtStatement"], json!("500.0000"));
}

/// T-a16 (Story 29-1a, AC3, C-29-1-25) — l'écart servi au tableau de bord :
/// `statementGap` en chaîne pour un écart réel, `null` sans relevé. Mutation
/// tuée : M-a21 (`statement_gap: None` en dur dans le handler de liste).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn list_bank_accounts_serves_the_statement_gap(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Consultation).await;
    let fy = insert_fiscal_year_2026(&pool, ctx.company_id).await;
    let revenue = create_account(
        &pool,
        ctx.company_id,
        ctx.user_id,
        "3200",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    link(&pool, ctx.bank_account_id, ctx.asset_account_id).await;
    // Un second compte bancaire, sans relevé.
    let without = create_bank_account(&pool, ctx.company_id, "CH5604835012345678009").await;
    insert_entry(
        &pool,
        ctx.company_id,
        fy,
        1,
        "2026-03-10",
        ctx.asset_account_id,
        revenue,
        "800.00",
    )
    .await;
    sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', REPEAT('d', 64), 'CAMT053', '2026-03-01', '2026-03-31', 1000.00, 0, ?)",
    )
    .bind(ctx.company_id)
    .bind(ctx.bank_account_id)
    .bind(ctx.user_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool.clone()).await;
    let body: Value = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let arr = body.as_array().expect("liste");
    let with = arr
        .iter()
        .find(|a| a["id"] == json!(ctx.bank_account_id))
        .expect("compte relevé");
    let gap = with["statementGap"].as_str().expect("écart en chaîne");
    assert_eq!(
        gap.parse::<rust_decimal::Decimal>().unwrap(),
        rust_decimal::Decimal::from(-200)
    );
    let none = arr
        .iter()
        .find(|a| a["id"] == json!(without))
        .expect("compte sans relevé");
    assert_eq!(none["statementGap"], Value::Null);
}

// ============================================================
// Story 15-5b (AC12, AC16, #427) — compte comptable lié non imputable
// ============================================================

/// Rend un compte **non imputable** sans le désactiver (patron
/// `products_revenue_account_e2e.rs::set_account_not_postable`) : le compte
/// ne diffère d'un compte accepté que par `postable`.
async fn set_account_not_postable(pool: &MySqlPool, account_id: i64) {
    sqlx::query("UPDATE accounts SET postable = FALSE, version = version + 1 WHERE id = ?")
        .bind(account_id)
        .execute(pool)
        .await
        .expect("set account not postable");
}

async fn link_directly(pool: &MySqlPool, bank_account_id: i64, account_id: i64) {
    sqlx::query(
        "UPDATE bank_accounts SET journal_account_id = ?, version = version + 1 WHERE id = ?",
    )
    .bind(account_id)
    .bind(bank_account_id)
    .execute(pool)
    .await
    .unwrap();
}

async fn assert_not_postable_15_5b(resp: reqwest::Response, account_id: i64, number: &str) {
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_POSTABLE", "{body}");
    assert_eq!(
        body["error"]["details"]["rejected"],
        json!([{ "accountId": account_id, "accountNumber": number }])
    );
}

async fn put_bank_account(
    pool: &MySqlPool,
    ctx: &Ctx,
    bank_account_id: i64,
    body: Value,
) -> reqwest::Response {
    let app = spawn_app(pool.clone()).await;
    app.client
        .put(app.url(&format!("/api/v1/bank-accounts/{bank_account_id}")))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap()
}

/// AC12 — création : un compte lié non imputable est refusé en 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_bank_account_rejects_non_postable_journal_account(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    set_account_not_postable(&pool, ctx.asset_account_id).await;
    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
            "isPrimary": false,
            "journalAccountId": ctx.asset_account_id,
        }))
        .send()
        .await
        .unwrap();
    assert_not_postable_15_5b(resp, ctx.asset_account_id, "1020").await;
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
        .bind(ctx.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1, "aucun compte bancaire créé");
}

/// AC12 — remplacement (PUT) et lien (PATCH) vers un **nouveau** compte non
/// imputable : 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_and_patch_reject_new_non_postable_journal_account(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    set_account_not_postable(&pool, ctx.asset_account_id).await;

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = put_bank_account(
        &pool,
        &ctx,
        ctx.bank_account_id,
        json!({
            "bankName": "UBS",
            "iban": "CH4431999123000889012",
            "isPrimary": true,
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;
    assert_not_postable_15_5b(resp, ctx.asset_account_id, "1020").await;

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({ "journalAccountId": ctx.asset_account_id, "version": version }),
    )
    .await;
    assert_not_postable_15_5b(resp, ctx.asset_account_id, "1020").await;
    let linked: Option<i64> =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(linked, None, "rien n'est lié");
}

/// AC12 (exemption) — PUT et PATCH qui renvoient le compte **déjà lié**,
/// devenu non imputable après coup : 200 (D-A0).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_and_patch_accept_unchanged_non_postable_journal_account(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    link_directly(&pool, ctx.bank_account_id, ctx.asset_account_id).await;
    set_account_not_postable(&pool, ctx.asset_account_id).await;

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = put_bank_account(
        &pool,
        &ctx,
        ctx.bank_account_id,
        json!({
            "bankName": "UBS renommée",
            "iban": "CH4431999123000889012",
            "isPrimary": true,
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());

    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({ "journalAccountId": ctx.asset_account_id, "version": version }),
    )
    .await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// AC12 (ordre) — compte bancaire inconnu + compte non imputable : 404 ;
/// version périmée + compte non imputable : 409. Sur le PUT et le PATCH.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn bank_account_errors_precede_non_postable_refusal(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    set_account_not_postable(&pool, ctx.asset_account_id).await;
    let app = spawn_app(pool.clone()).await;

    let resp = put_bank_account(
        &pool,
        &ctx,
        999_999,
        json!({
            "bankName": "UBS",
            "iban": "CH4431999123000889012",
            "journalAccountId": ctx.asset_account_id,
            "version": 1,
        }),
    )
    .await;
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");

    let resp = app
        .client
        .patch(app.url("/api/v1/bank-accounts/999999"))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "journalAccountId": ctx.asset_account_id, "version": 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 404);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND");

    let resp = put_bank_account(
        &pool,
        &ctx,
        ctx.bank_account_id,
        json!({
            "bankName": "UBS",
            "iban": "CH4431999123000889012",
            "journalAccountId": ctx.asset_account_id,
            "version": 99,
        }),
    )
    .await;
    assert_eq!(resp.status(), 409);

    let resp = ctx_patch(
        &pool,
        &ctx,
        json!({ "journalAccountId": ctx.asset_account_id, "version": 99 }),
    )
    .await;
    assert_eq!(resp.status(), 409);
}

/// AC12 — un PUT qui promeut un compte en principal et échoue sur ce refus
/// **n'a rien démoté** : la transaction est abandonnée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_primary_refused_for_non_postable_does_not_demote_old_primary(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let second = bank_accounts::create(
        &pool,
        NewBankAccount {
            company_id: ctx.company_id,
            bank_name: "PostFinance".into(),
            iban: "CH3908704016075473007".into(),
            qr_iban: None,
            is_primary: false,
        },
    )
    .await
    .unwrap()
    .id;
    set_account_not_postable(&pool, ctx.asset_account_id).await;
    let version = read_bank_account_version(&pool, second).await;
    let resp = put_bank_account(
        &pool,
        &ctx,
        second,
        json!({
            "bankName": "PostFinance",
            "iban": "CH3908704016075473007",
            "isPrimary": true,
            "journalAccountId": ctx.asset_account_id,
            "version": version,
        }),
    )
    .await;
    assert_not_postable_15_5b(resp, ctx.asset_account_id, "1020").await;
    let old_is_primary: bool =
        sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(old_is_primary, "l'ancien principal l'est toujours");
}

// ===========================================================================
// Story 15-6c (#474) — un compte bancaire ne se lie pas au compte débiteurs ni
// au compte créanciers désignés : 400 `BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`.
// ===========================================================================

/// Montage 15-6c : `setup_full` (compte bancaire principal, `1020`) + les
/// comptes `1100` (actif) et `2000` (passif), désignés en SQL comme compte
/// débiteurs et compte créanciers. Rend `(ctx, receivable, payable)`.
async fn setup_claims(pool: &MySqlPool) -> (Ctx, i64, i64) {
    complete_onboarding(pool).await;
    let ctx = setup_full(pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let receivable = create_account(
        pool,
        ctx.company_id,
        ctx.user_id,
        "1100",
        "Débiteurs",
        AccountType::Asset,
    )
    .await;
    let payable = create_account(
        pool,
        ctx.company_id,
        ctx.user_id,
        "2000",
        "Créanciers",
        AccountType::Liability,
    )
    .await;
    designate_claims(pool, ctx.company_id, Some(receivable), Some(payable)).await;
    (ctx, receivable, payable)
}

async fn designate_claims(
    pool: &MySqlPool,
    company_id: i64,
    receivable: Option<i64>,
    payable: Option<i64>,
) {
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_receivable_account_id, default_payable_account_id) \
         VALUES (?, ?, ?) \
         ON DUPLICATE KEY UPDATE default_receivable_account_id = VALUES(default_receivable_account_id), \
             default_payable_account_id = VALUES(default_payable_account_id)",
    )
    .bind(company_id)
    .bind(receivable)
    .bind(payable)
    .execute(pool)
    .await
    .unwrap();
}

async fn assert_ledger_refusal(
    resp: reqwest::Response,
    account_id: i64,
    number: &str,
    claim: &str,
) {
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_eq!(
        body["error"]["code"], "BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT",
        "{body}"
    );
    assert_eq!(
        body["error"]["details"],
        json!({ "accountId": account_id, "accountNumber": number, "claim": claim }),
    );
}

async fn bank_count(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM bank_accounts WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn is_primary(pool: &MySqlPool, id: i64) -> bool {
    sqlx::query_scalar("SELECT is_primary FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Test 1 (AC3) — POST vers le compte débiteurs, puis vers le compte
/// créanciers : 400, `details.claim` juste, rien n'est créé et l'ancien
/// principal n'est pas démoté (la création demandait `isPrimary: true`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_bank_account_refuses_a_claim_account_ledger(pool: MySqlPool) {
    let (ctx, receivable, payable) = setup_claims(&pool).await;
    let app = spawn_app(pool.clone()).await;
    for (target, number, claim) in [
        (receivable, "1100", "receivable"),
        (payable, "2000", "payable"),
    ] {
        let resp = app
            .client
            .post(app.url("/api/v1/bank-accounts"))
            .bearer_auth(&ctx.jwt)
            .json(&json!({
                "bankName": "PostFinance",
                "iban": "CH3908704016075473007",
                "isPrimary": true,
                "journalAccountId": target,
            }))
            .send()
            .await
            .unwrap();
        assert_ledger_refusal(resp, target, number, claim).await;
        assert_eq!(
            bank_count(&pool, ctx.company_id).await,
            1,
            "rien n'est créé"
        );
        assert!(
            is_primary(&pool, ctx.bank_account_id).await,
            "rien n'est démoté"
        );
    }
}

/// Test 2 (AC4) — PUT et PATCH vers chacun des deux comptes de créance : 400,
/// `version` inchangée, lien inchangé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_and_patch_refuse_a_claim_account_ledger(pool: MySqlPool) {
    let (ctx, receivable, payable) = setup_claims(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;
    for (target, number, claim) in [
        (receivable, "1100", "receivable"),
        (payable, "2000", "payable"),
    ] {
        let resp = app
            .client
            .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
            .bearer_auth(&ctx.jwt)
            .json(&json!({
                "bankName": "UBS",
                "iban": "CH4431999123000889012",
                "isPrimary": true,
                "journalAccountId": target,
                "version": version,
            }))
            .send()
            .await
            .unwrap();
        assert_ledger_refusal(resp, target, number, claim).await;

        let resp = app
            .client
            .patch(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
            .bearer_auth(&ctx.jwt)
            .json(&json!({ "journalAccountId": target, "version": version }))
            .send()
            .await
            .unwrap();
        assert_ledger_refusal(resp, target, number, claim).await;

        assert_eq!(
            read_bank_account_version(&pool, ctx.bank_account_id).await,
            version
        );
    }
    let linked: Option<i64> =
        sqlx::query_scalar("SELECT journal_account_id FROM bank_accounts WHERE id = ?")
            .bind(ctx.bank_account_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(linked, None);
}

/// Test 3 (AC4, exemption) — un compte bancaire **déjà** lié au compte
/// débiteurs (donnée antérieure, posée en SQL) accepte un PUT qui ne change
/// que son nom, et refuse un PUT qui le relie au compte créanciers.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn already_linked_claim_account_stays_editable(pool: MySqlPool) {
    let (ctx, receivable, payable) = setup_claims(&pool).await;
    link(&pool, ctx.bank_account_id, receivable).await;
    let app = spawn_app(pool.clone()).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "UBS renommée",
            "iban": "CH4431999123000889012",
            "isPrimary": true,
            "journalAccountId": receivable,
            "version": version,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "UBS renommée",
            "iban": "CH4431999123000889012",
            "isPrimary": true,
            "journalAccountId": payable,
            "version": version + 1,
        }))
        .send()
        .await
        .unwrap();
    assert_ledger_refusal(resp, payable, "2000", "payable").await;
}

/// Test 4 (AC6, ordre) — PUT avec une `version` périmée vers le compte
/// débiteurs : **409** (la version précède le refus). Réglage débiteurs
/// `NULL` : le lien vers l'ancien compte débiteurs est accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn stale_version_precedes_claim_refusal_and_null_setting_allows_link(pool: MySqlPool) {
    let (ctx, receivable, payable) = setup_claims(&pool).await;
    let app = spawn_app(pool.clone()).await;
    let version = read_bank_account_version(&pool, ctx.bank_account_id).await;

    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({
            "bankName": "UBS",
            "iban": "CH4431999123000889012",
            "isPrimary": true,
            "journalAccountId": receivable,
            "version": version + 5,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409, "{}", resp.text().await.unwrap());

    designate_claims(&pool, ctx.company_id, None, Some(payable)).await;
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "journalAccountId": receivable, "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

// ===========================================================================
// Story 26-2a (#622) — description optionnelle du compte bancaire.
// ===========================================================================

async fn db_description(pool: &MySqlPool, id: i64) -> Option<String> {
    sqlx::query_scalar("SELECT description FROM bank_accounts WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn post_bank(app: &TestApp, ctx: &Ctx, body: Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

async fn put_bank(app: &TestApp, ctx: &Ctx, id: i64, body: Value) -> (u16, Value) {
    let resp = app
        .client
        .put(app.url(&format!("/api/v1/bank-accounts/{id}")))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

async fn list_banks(app: &TestApp, ctx: &Ctx) -> Vec<Value> {
    let resp = app
        .client
        .get(app.url("/api/v1/bank-accounts"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    resp.json().await.unwrap()
}

/// AC4 — `POST` : la description est normalisée (`trim`), stockée, rendue en
/// `201` et portée par l'audit `bank_account.created` ; sans elle → `null`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_bank_account_avec_et_sans_description(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let (status, body) = post_bank(
        &app,
        &ctx,
        json!({ "bankName": "UBS", "description": "  Épargne  ", "iban": "CH3908704016075473007" }),
    )
    .await;
    assert_eq!(status, 201, "body: {body}");
    assert_eq!(body["description"], "Épargne");
    let id = body["id"].as_i64().unwrap();
    assert_eq!(db_description(&pool, id).await.as_deref(), Some("Épargne"));
    let audit: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.created' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit["description"], "Épargne");

    let (status, body) = post_bank(
        &app,
        &ctx,
        json!({ "bankName": "UBS", "iban": "CH9300762011623852957" }),
    )
    .await;
    assert_eq!(status, 201, "body: {body}");
    assert_eq!(body["description"], Value::Null);
    let id = body["id"].as_i64().unwrap();
    assert_eq!(db_description(&pool, id).await, None);
    let audit: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.created' AND entity_id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit["description"], Value::Null);
}

/// AC3 — bornes (60 et 61 en **littéraux**, jamais la constante : M4, M15),
/// caractères et non octets, normalisation, caractères refusés, ordre
/// trim / contrôle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn post_bank_account_validation_de_la_description(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let acceptes: Vec<(String, Option<&str>)> = vec![
        ("a".repeat(60), None),
        ("\u{e9}".repeat(60), None),
        ("  Épargne  ".into(), Some("Épargne")),
        ("Épargne\n".into(), Some("Épargne")),
        ("   ".into(), Some("")),
    ];
    for (saisie, attendu) in acceptes {
        let (status, body) = post_bank(
            &app,
            &ctx,
            json!({ "bankName": "UBS", "description": saisie, "iban": "CH3908704016075473007" }),
        )
        .await;
        assert_eq!(status, 201, "{saisie:?} doit être accepté — body: {body}");
        let id = body["id"].as_i64().unwrap();
        let stocke = db_description(&pool, id).await;
        match attendu {
            None => assert_eq!(stocke.as_deref(), Some(saisie.as_str())),
            Some("") => assert_eq!(stocke, None, "{saisie:?} doit être stocké NULL"),
            Some(a) => assert_eq!(stocke.as_deref(), Some(a)),
        }
    }

    let trop_longs = ["a".repeat(61), "\u{e9}".repeat(61)];
    for saisie in trop_longs {
        let (status, body) = post_bank(
            &app,
            &ctx,
            json!({ "bankName": "UBS", "description": saisie, "iban": "CH3908704016075473007" }),
        )
        .await;
        assert_eq!(status, 400, "61 caractères doivent être refusés");
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        assert_eq!(
            body["error"]["message"],
            "La description ne peut pas dépasser 60 caractères"
        );
    }

    for saisie in [
        "a\nb",
        "Épar\ngne",
        "a\u{2028}b",
        "a\u{202E}b",
        "a\tb",
        "a\u{200B}b",
    ] {
        let (status, body) = post_bank(
            &app,
            &ctx,
            json!({ "bankName": "UBS", "description": saisie, "iban": "CH3908704016075473007" }),
        )
        .await;
        assert_eq!(status, 400, "{saisie:?} doit être refusé");
        assert_eq!(
            body["error"]["message"],
            "La description ne peut pas contenir de saut de ligne ni de caractère de contrôle ou de mise en page"
        );
    }
}

/// AC3 — ordre des refus de forme épinglé : banque → description → IBAN →
/// QR-IBAN, sur un corps qui les viole tous puis les corrige un à un.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ordre_des_refus_de_forme(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let trop_long = "a".repeat(61);

    let cas = [
        (
            json!({ "bankName": "  ", "description": trop_long, "iban": "BAD", "qrIban": "BAD" }),
            "Le nom de la banque ne peut pas être vide",
        ),
        (
            json!({ "bankName": "UBS", "description": trop_long, "iban": "BAD", "qrIban": "BAD" }),
            "La description ne peut pas dépasser 60 caractères",
        ),
        (
            json!({ "bankName": "UBS", "description": "Épargne", "iban": "BAD", "qrIban": "BAD" }),
            "IBAN invalide",
        ),
        (
            json!({ "bankName": "UBS", "description": "Épargne", "iban": "CH3908704016075473007", "qrIban": "BAD" }),
            "QR-IBAN invalide",
        ),
    ];
    for (corps, debut) in cas {
        let (status, body) = post_bank(&app, &ctx, corps.clone()).await;
        assert_eq!(status, 400, "corps {corps}");
        let message = body["error"]["message"].as_str().unwrap();
        assert!(
            message.starts_with(debut),
            "corps {corps} : attendu « {debut}… », obtenu « {message} »"
        );
    }
}

/// AC3 (C-26-2-4) — pas d'unicité : deux comptes de la même société, même
/// banque, même description → `201` et `201`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn description_non_unique(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    for iban in ["CH3908704016075473007", "CH9300762011623852957"] {
        let (status, body) = post_bank(
            &app,
            &ctx,
            json!({ "bankName": "UBS", "description": "Épargne", "iban": iban }),
        )
        .await;
        assert_eq!(status, 201, "body: {body}");
    }
}

/// AC4 — `PUT` écrit la description (audit `before`/`after`), puis un `PUT`
/// dont le corps **omet** `description` l'**efface** (remplacement complet,
/// C-26-2-19 — cible de M13) : `200`, `null` en réponse et en base.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_ecrit_puis_efface_la_description_absente(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let id = ctx.bank_account_id;

    let version = read_bank_account_version(&pool, id).await;
    let (status, body) = put_bank(
        &app,
        &ctx,
        id,
        json!({ "bankName": "UBS", "description": "Personnel", "iban": "CH4431999123000889012", "isPrimary": true, "version": version }),
    )
    .await;
    assert_eq!(status, 200, "body: {body}");
    assert_eq!(body["description"], "Personnel");
    assert_eq!(
        db_description(&pool, id).await.as_deref(),
        Some("Personnel")
    );
    let audit: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.updated' AND entity_id = ? \
         AND JSON_EXTRACT(details_json, '$.trigger') = 'full_update' ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit["before"]["description"], Value::Null);
    assert_eq!(audit["after"]["description"], "Personnel");

    // Corps écrit pour la 0.13.0 : sans le champ.
    let (status, body) = put_bank(
        &app,
        &ctx,
        id,
        json!({ "bankName": "UBS", "iban": "CH4431999123000889012", "isPrimary": true, "version": version + 1 }),
    )
    .await;
    assert_eq!(status, 200, "body: {body}");
    assert_eq!(body["description"], Value::Null);
    assert_eq!(db_description(&pool, id).await, None);
    let audit: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'bank_account.updated' AND entity_id = ? \
         AND JSON_EXTRACT(details_json, '$.trigger') = 'full_update' ORDER BY id DESC LIMIT 1",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audit["before"]["description"], "Personnel");
    assert_eq!(audit["after"]["description"], Value::Null);
}

/// AC4 — `GET` porte `description` sur chaque élément, `null` si absente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_liste_porte_la_description(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (_, body) = post_bank(
        &app,
        &ctx,
        json!({ "bankName": "UBS", "description": "Épargne", "iban": "CH3908704016075473007" }),
    )
    .await;
    let decrit = body["id"].as_i64().unwrap();

    let list = list_banks(&app, &ctx).await;
    assert_eq!(list.len(), 2);
    for item in &list {
        assert!(
            item.as_object().unwrap().contains_key("description"),
            "clé description absente : {item}"
        );
        if item["id"].as_i64() == Some(decrit) {
            assert_eq!(item["description"], "Épargne");
        } else {
            assert_eq!(item["description"], Value::Null);
        }
    }
}

/// AC4 — `PATCH` (lien au grand livre) et `DELETE` (archivage) rendent la
/// description inchangée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn patch_et_delete_rendent_la_description_inchangee(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let id = ctx.bank_account_id;
    sqlx::query("UPDATE bank_accounts SET description = 'Personnel' WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();

    let version = read_bank_account_version(&pool, id).await;
    let resp = app
        .client
        .patch(app.url(&format!("/api/v1/bank-accounts/{id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "journalAccountId": ctx.asset_account_id, "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["description"], "Personnel");

    let resp = app
        .client
        .delete(app.url(&format!("/api/v1/bank-accounts/{id}")))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version + 1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["archived"], true);
    assert_eq!(body["description"], "Personnel");
    assert_eq!(
        db_description(&pool, id).await.as_deref(),
        Some("Personnel")
    );
}

/// AC2 — démotion de bout en bout : un `PUT isPrimary=true` sur un second
/// compte démote l'ancien principal, qui garde sa description au `GET`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn demotion_du_principal_conserve_sa_description(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let ancien = ctx.bank_account_id;
    sqlx::query("UPDATE bank_accounts SET description = 'Personnel' WHERE id = ?")
        .bind(ancien)
        .execute(&pool)
        .await
        .unwrap();
    let (_, body) = post_bank(
        &app,
        &ctx,
        json!({ "bankName": "PostFinance", "iban": "CH3908704016075473007", "isPrimary": false }),
    )
    .await;
    let second = body["id"].as_i64().unwrap();

    let (status, body) = put_bank(
        &app,
        &ctx,
        second,
        json!({ "bankName": "PostFinance", "iban": "CH3908704016075473007", "isPrimary": true, "version": 1 }),
    )
    .await;
    assert_eq!(status, 200, "body: {body}");

    let list = list_banks(&app, &ctx).await;
    let ancien_json = list
        .iter()
        .find(|b| b["id"].as_i64() == Some(ancien))
        .expect("ancien principal listé");
    assert_eq!(ancien_json["isPrimary"], false);
    assert_eq!(ancien_json["description"], "Personnel");
}

/// AC4 — les messages de forme des deux extracteurs nomment `description?`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn messages_des_extracteurs_nomment_la_description(pool: MySqlPool) {
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;

    let (status, body) = post_bank(
        &app,
        &ctx,
        json!({ "bankName": "UBS", "description": 42, "iban": "CH3908704016075473007" }),
    )
    .await;
    assert_eq!(status, 400);
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("`description?` str"),
        "message : {message}"
    );

    let (status, body) = put_bank(
        &app,
        &ctx,
        ctx.bank_account_id,
        json!({ "bankName": "UBS", "description": 42, "iban": "CH3908704016075473007", "version": 1 }),
    )
    .await;
    assert_eq!(status, 400);
    let message = body["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("`description?` str"),
        "message : {message}"
    );
}

// ============================================================
// Story 27-1 (#632) — la suppression d'un import non rapproché et ses effets
// sur le compte bancaire : archivage (H6), tuile du relevé (H9).
// ============================================================

/// Insère un import (`closing_balance` facultatif) portant `n` transactions
/// `pending` ; rend son identifiant.
async fn insert_import(
    pool: &MySqlPool,
    ctx: &Ctx,
    hash_char: char,
    period: (&str, &str),
    closing: Option<&str>,
    n: usize,
) -> i64 {
    let id = sqlx::query(
        "INSERT INTO bank_imports \
         (company_id, bank_account_id, filename, file_hash, source_format, period_from, period_to, \
          closing_balance, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'r.xml', REPEAT(?, 64), 'CAMT053_V04', ?, ?, ?, ?, ?)",
    )
    .bind(ctx.company_id)
    .bind(ctx.bank_account_id)
    .bind(hash_char.to_string())
    .bind(period.0)
    .bind(period.1)
    .bind(closing)
    .bind(n as i32)
    .bind(ctx.user_id)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for _ in 0..n {
        sqlx::query(
            "INSERT INTO bank_transactions \
             (company_id, import_id, bank_account_id, booking_date, amount, currency, details) \
             VALUES (?, ?, ?, ?, 10.00, 'CHF', 'test')",
        )
        .bind(ctx.company_id)
        .bind(id)
        .bind(ctx.bank_account_id)
        .bind(period.0)
        .execute(pool)
        .await
        .unwrap();
    }
    id
}

async fn delete_import_as(app: &TestApp, ctx: &Ctx, id: i64) -> u16 {
    app.client
        .delete(app.url(&format!("/api/v1/bank-imports/{id}")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .status()
        .as_u16()
}

async fn archive(app: &TestApp, pool: &MySqlPool, ctx: &Ctx) -> reqwest::Response {
    let version = read_bank_account_version(pool, ctx.bank_account_id).await;
    app.client
        .delete(app.url(&format!("/api/v1/bank-accounts/{}", ctx.bank_account_id)))
        .bearer_auth(&ctx.jwt)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap()
}

/// H6 (C-27-1-12) — la garde d'archivage se lève une fois les imports
/// supprimés, et son message nomme ce remède. Le test fige le comportement
/// accepté, dans les deux sens.
///
/// ⚠️ Le montage de ce fichier n'initialise pas la langue des erreurs : sans
/// `init_error_i18n`, le message lu serait le **repli** Rust, et la phrase du
/// catalogue pourrait disparaître sans que rien ne rougisse (mutation M20,
/// survivante au premier passage de la campagne). D'où le catalogue `fr-CH`
/// posé ici. La langue est globale au processus : sous `cargo test` (un
/// processus par binaire), les tests suivants liront le catalogue `fr-CH`.
/// La garde G9 (`textes_coherents.rs`) tient égal au catalogue le repli de
/// ce message et ceux des deux autres messages de la story — depuis la revue
/// de code P1 (E4) seulement : elle ne couvre pas tous les replis de ce
/// binaire, et un test qui lirait un repli différent de son catalogue y
/// changerait de résultat selon l'ordre d'exécution.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deleting_the_imports_lifts_the_archive_guard(pool: MySqlPool) {
    let bundle = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(bundle, kesh_i18n::Locale::FrCh);
    complete_onboarding(&pool).await;
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let import_id = insert_import(&pool, &ctx, 'e', ("2026-01-01", "2026-01-31"), None, 2).await;

    let resp = archive(&app, &pool, &ctx).await;
    assert_eq!(resp.status(), 412);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "BANK_ACCOUNT_HAS_TRANSACTIONS");
    let msg = body["error"]["message"].as_str().unwrap();
    assert!(
        msg.contains(
            "Les imports dont aucune transaction n'est rapprochée peuvent être supprimés \
             (Quotidien → Importer des relevés)."
        ),
        "le message du 412 nomme le remède : {msg}"
    );
    // Story 28-4a (#642, AC9) — et propose la clôture.
    assert!(
        msg.contains("clore le compte : il passe en lecture seule"),
        "le message du 412 propose la clôture : {msg}"
    );

    assert_eq!(delete_import_as(&app, &ctx, import_id).await, 204);
    let resp = archive(&app, &pool, &ctx).await;
    assert_eq!(resp.status(), 200, "plus aucune transaction : archivable");
}

/// H9 (revue P3, R3-7) — la tuile « Relevé du … » se recalcule sur le relevé
/// restant portant un solde de clôture dont la période se termine le plus tard,
/// puis disparaît.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deleting_a_statement_recomputes_the_last_statement(pool: MySqlPool) {
    let ctx = setup_full(&pool, "Acme", "CH4431999123000889012", Role::Comptable).await;
    let march = insert_import(
        &pool,
        &ctx,
        'f',
        ("2026-03-01", "2026-03-31"),
        Some("480.00"),
        1,
    )
    .await;
    let april = insert_import(
        &pool,
        &ctx,
        'g',
        ("2026-04-01", "2026-04-30"),
        Some("520.00"),
        1,
    )
    .await;
    let app = spawn_app(pool.clone()).await;
    let tile = || async {
        let body: Value = app
            .client
            .get(app.url("/api/v1/bank-accounts"))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let a = body.as_array().expect("liste")[0].clone();
        (
            a["statementClosingBalance"].clone(),
            a["statementDate"].clone(),
        )
    };

    assert_eq!(
        tile().await,
        (json!("520.00"), json!("2026-04-30")),
        "montage"
    );
    assert_eq!(delete_import_as(&app, &ctx, april).await, 204);
    assert_eq!(
        tile().await,
        (json!("480.00"), json!("2026-03-31")),
        "le relevé restant"
    );
    assert_eq!(delete_import_as(&app, &ctx, march).await, 204);
    let (balance, date) = tile().await;
    assert!(
        balance.is_null() && date.is_null(),
        "plus de relevé exposé : {balance} {date}"
    );
}
