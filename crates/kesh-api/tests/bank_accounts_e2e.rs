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
