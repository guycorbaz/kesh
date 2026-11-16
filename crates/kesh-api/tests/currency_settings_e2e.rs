//! Story 37-1 (AC-3, AC-3 bis) — `PUT /api/v1/companies/current/currency-settings`
//! par la route : rôles (Comptable 403, clé d'API `API_KEY_ADMIN_FORBIDDEN`,
//! Administrateur en session Bearer sans cookie), forme du corps
//! (`missing`, `unexpected`), chaque `reason` de `FX_ACCOUNT_INVALID`, compte de
//! conversion, irréversibilité (409 `MULTI_CURRENCY_IRREVERSIBLE`), version
//! périmée, no-op, exercices à l'activation, mode d'un exercice créé ensuite ou
//! rouvert, garde D-CR7-2 (`LATER_FISCAL_YEAR_MONO_CURRENCY`) et motif C4.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";
const ROUTE: &str = "/api/v1/companies/current/currency-settings";

// ============================================================
// Harnais (patron rejeu_interblocage_e2e.rs)
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
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
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
    TestApp {
        base_url: format!("http://{addr}"),
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

struct Ctx {
    app: TestApp,
    jwt: String,
    s: SeededCompany,
    /// Charge de change (6950), compte de conversion (1090), valides.
    charge: i64,
    transit: i64,
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    let charge = compte(
        pool,
        s.company_id,
        "6950",
        "Expense",
        true,
        true,
        None,
        None,
    )
    .await;
    let transit = compte(pool, s.company_id, "1090", "Asset", true, true, None, None).await;
    Ctx {
        app,
        jwt,
        s,
        charge,
        transit,
    }
}

#[allow(clippy::too_many_arguments)]
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    actif: bool,
    imputable: bool,
    role: Option<&str>,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, role, currency) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(actif)
    .bind(imputable)
    .bind(role)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn envoyer(ctx: &Ctx, methode: reqwest::Method, chemin: &str, corps: Value) -> (u16, Value) {
    let resp = ctx
        .app
        .client
        .request(methode, ctx.app.url(chemin))
        .bearer_auth(&ctx.jwt)
        .json(&corps)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn version(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT version FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn corps(
    enabled: bool,
    gain: Option<i64>,
    perte: Option<i64>,
    transit: Option<i64>,
    v: i64,
) -> Value {
    json!({
        "multiCurrencyEnabled": enabled,
        "fxGainAccountId": gain,
        "fxLossAccountId": perte,
        "fxTransitAccountId": transit,
        "version": v,
    })
}

/// `PUT` du réglage à la version courante.
async fn regler(
    pool: &MySqlPool,
    ctx: &Ctx,
    enabled: bool,
    gain: Option<i64>,
    perte: Option<i64>,
    transit: Option<i64>,
) -> (u16, Value) {
    let v = version(pool, ctx.s.company_id).await;
    envoyer(
        ctx,
        reqwest::Method::PUT,
        ROUTE,
        corps(enabled, gain, perte, transit, v),
    )
    .await
}

async fn activer(pool: &MySqlPool, ctx: &Ctx) -> Value {
    let (status, body) = regler(
        pool,
        ctx,
        true,
        Some(ctx.charge),
        Some(ctx.charge),
        Some(ctx.transit),
    )
    .await;
    assert_eq!(status, 200, "activation : {body}");
    body
}

async fn audits(pool: &MySqlPool, action: &str) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = ?")
        .bind(action)
        .fetch_one(pool)
        .await
        .unwrap()
}

fn assert_fx(status: u16, body: &Value, field: &str, reason: &str) {
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "FX_ACCOUNT_INVALID", "{body}");
    assert_eq!(body["error"]["details"]["field"], field, "{body}");
    assert_eq!(body["error"]["details"]["reason"], reason, "{body}");
    assert!(
        body["error"]["details"].get("holderAccountId").is_none(),
        "compte de la société : pas de porteur — {body}"
    );
    assert!(
        !body["error"]["message"].as_str().unwrap_or("").is_empty(),
        "le message du serveur est affiché tel quel par l'écran — {body}"
    );
}

async fn exercice_sql(pool: &MySqlPool, company_id: i64, annee: i32, statut: &str) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(format!("Exercice {annee}"))
    .bind(format!("{annee}-01-01"))
    .bind(format!("{annee}-12-31"))
    .bind(statut)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn mode_exercice(pool: &MySqlPool, id: i64) -> bool {
    sqlx::query_scalar("SELECT multi_currency FROM fiscal_years WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ============================================================
// Rôles et session (V6)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_comptable_recoit_403(pool: MySqlPool) {
    let mut ctx = monter(&pool).await;
    let comptable: i64 = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES ('compta', ?, 'Comptable', TRUE, ?)",
    )
    .bind(kesh_db::test_fixtures::ADMIN_PASSWORD_HASH)
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    ctx.jwt = forge_jwt(comptable, "Comptable", ctx.s.company_id);
    let (status, body) = regler(
        &pool,
        &ctx,
        true,
        Some(ctx.charge),
        Some(ctx.charge),
        Some(ctx.transit),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(audits(&pool, "company.currency_settings_updated").await, 0);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_d_api_de_l_administrateur_est_refusee(pool: MySqlPool) {
    let mut ctx = monter(&pool).await;
    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/settings/api-keys",
        json!({ "name": "clé admin", "scope": "read-write" }),
    )
    .await;
    assert_eq!(status, 201, "création de la clé : {body}");
    ctx.jwt = body["key"].as_str().unwrap().to_string();
    let (status, body) = regler(
        &pool,
        &ctx,
        true,
        Some(ctx.charge),
        Some(ctx.charge),
        Some(ctx.transit),
    )
    .await;
    assert_eq!(status, 403, "{body}");
    assert_eq!(body["error"]["code"], "API_KEY_ADMIN_FORBIDDEN", "{body}");
    assert_eq!(version(&pool, ctx.s.company_id).await, 1);
}

/// V6 : l'Administrateur se connecte par `POST /auth/login` et envoie le `PUT`
/// en `Authorization: Bearer <accessToken>`, **sans** cookie (le client n'a pas
/// de magasin de cookies) → 200.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_administrateur_en_session_bearer_sans_cookie_active_le_mode(pool: MySqlPool) {
    let mut ctx = monter(&pool).await;
    let resp = ctx
        .app
        .client
        .post(ctx.app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": "admin123" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    ctx.jwt = body["accessToken"].as_str().unwrap().to_string();

    let body = activer(&pool, &ctx).await;
    assert_eq!(body["multiCurrencyEnabled"], true);
    assert_eq!(body["fxGainAccountId"], ctx.charge);
    assert_eq!(body["fxTransitAccountId"], ctx.transit);
    assert_eq!(body["baseCurrency"], "CHF");
    assert_eq!(body["version"], 2);
}

// ============================================================
// Forme du corps, raisons, compte de conversion
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_forme_du_corps_passe_avant_tout(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    for (gain, perte, transit, champ) in [
        (None, Some(ctx.charge), Some(ctx.transit), "fxGainAccountId"),
        (Some(ctx.charge), None, Some(ctx.transit), "fxLossAccountId"),
        (
            Some(ctx.charge),
            Some(ctx.charge),
            None,
            "fxTransitAccountId",
        ),
    ] {
        let (status, body) = regler(&pool, &ctx, true, gain, perte, transit).await;
        assert_fx(status, &body, champ, "missing");
    }
    let (status, body) = regler(&pool, &ctx, false, None, None, Some(ctx.transit)).await;
    assert_fx(status, &body, "fxTransitAccountId", "unexpected");

    // Après activation : vider un compte → missing ; `false` avec un identifiant
    // → unexpected (la forme passe avant l'irréversibilité).
    activer(&pool, &ctx).await;
    let (status, body) = regler(&pool, &ctx, true, Some(ctx.charge), None, Some(ctx.transit)).await;
    assert_fx(status, &body, "fxLossAccountId", "missing");
    let (status, body) = regler(&pool, &ctx, false, Some(ctx.charge), None, None).await;
    assert_fx(status, &body, "fxGainAccountId", "unexpected");
    assert_eq!(audits(&pool, "company.currency_settings_updated").await, 1);
}

/// Chaque `reason` de la liste close, par la route, sur le compte des gains ;
/// puis le type du compte de conversion (`Expense` et `Revenue` refusés).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn chaque_raison_est_rendue_par_la_route(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    let archive = compte(&pool, c, "6952", "Expense", false, true, None, None).await;
    let titre = compte(&pool, c, "6953", "Expense", true, false, None, None).await;
    let actif = compte(&pool, c, "1027", "Asset", true, true, None, None).await;
    let en_devise = compte(&pool, c, "6954", "Expense", true, true, None, Some("USD")).await;
    let role_ca = compte(
        &pool,
        c,
        "3200",
        "Revenue",
        true,
        true,
        Some("DefaultRevenue"),
        None,
    )
    .await;
    let produit_reglages = ctx.s.accounts["3000"];

    for (gain, reason) in [
        (i64::MAX, "unknown"),
        (archive, "archived"),
        (titre, "notPostable"),
        (actif, "accountType"),
        (en_devise, "foreignCurrency"),
        (role_ca, "revenueAccount"),
        (produit_reglages, "revenueAccount"),
    ] {
        let (status, body) = regler(
            &pool,
            &ctx,
            true,
            Some(gain),
            Some(ctx.charge),
            Some(ctx.transit),
        )
        .await;
        assert_fx(status, &body, "fxGainAccountId", reason);
    }
    let produit = compte(&pool, c, "6960", "Revenue", true, true, None, None).await;
    for transit in [ctx.charge, produit] {
        let (status, body) = regler(
            &pool,
            &ctx,
            true,
            Some(ctx.charge),
            Some(ctx.charge),
            Some(transit),
        )
        .await;
        assert_fx(status, &body, "fxTransitAccountId", "accountType");
    }
    assert_eq!(version(&pool, c).await, 1, "aucun refus n'écrit");
    assert_eq!(audits(&pool, "company.currency_settings_updated").await, 0);
}

// ============================================================
// Irréversibilité, version, no-op
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn desactiver_est_refuse_et_ne_change_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    activer(&pool, &ctx).await;
    let v = version(&pool, ctx.s.company_id).await;
    let (status, body) = regler(&pool, &ctx, false, None, None, None).await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        body["error"]["code"], "MULTI_CURRENCY_IRREVERSIBLE",
        "{body}"
    );
    assert_eq!(body["error"]["details"], json!({}), "{body}");
    let enabled: bool =
        sqlx::query_scalar("SELECT multi_currency_enabled FROM companies WHERE id = ?")
            .bind(ctx.s.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(enabled);
    assert_eq!(version(&pool, ctx.s.company_id).await, v);
    assert!(
        mode_exercice(&pool, ctx.s.fiscal_year_id).await,
        "exercice inchangé"
    );
    assert_eq!(audits(&pool, "company.currency_settings_updated").await, 1);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_version_perimee_est_refusee_et_le_no_op_n_ecrit_rien(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::PUT,
        ROUTE,
        corps(
            true,
            Some(ctx.charge),
            Some(ctx.charge),
            Some(ctx.transit),
            9,
        ),
    )
    .await;
    assert_eq!(status, 409, "{body}");
    assert_eq!(body["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT", "{body}");

    // No-op en mono-devise.
    let (status, body) = regler(&pool, &ctx, false, None, None, None).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["version"], 1);

    // No-op après activation.
    let active = activer(&pool, &ctx).await;
    let (status, body) = regler(
        &pool,
        &ctx,
        true,
        Some(ctx.charge),
        Some(ctx.charge),
        Some(ctx.transit),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["version"], active["version"]);
    assert_eq!(audits(&pool, "company.currency_settings_updated").await, 1);
}

/// Motif C4 : un compte inchangé devenu invalide n'empêche pas d'enregistrer
/// l'autre.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_inchange_devenu_invalide_n_empeche_pas_d_enregistrer_l_autre(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    activer(&pool, &ctx).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.transit)
        .execute(&pool)
        .await
        .unwrap();
    let autre = compte(
        &pool,
        ctx.s.company_id,
        "6955",
        "Expense",
        true,
        true,
        None,
        None,
    )
    .await;
    let (status, body) = regler(
        &pool,
        &ctx,
        true,
        Some(autre),
        Some(ctx.charge),
        Some(ctx.transit),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["fxGainAccountId"], autre);
}

// ============================================================
// Exercices : activation, création, réouverture (AC-3, AC-3 bis)
// ============================================================

/// Activation : exercices ouverts multi-devises, clos inchangés, identifiants au
/// détail d'audit ; l'exercice clos puis rouvert reste mono-devise ; un exercice
/// créé ensuite est multi-devises.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_activation_et_le_mode_des_exercices(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let clos = exercice_sql(&pool, ctx.s.company_id, 2019, "Closed").await;
    activer(&pool, &ctx).await;
    assert!(mode_exercice(&pool, ctx.s.fiscal_year_id).await);
    assert!(!mode_exercice(&pool, clos).await);
    let details = common::audit_details(
        &pool,
        "company",
        ctx.s.company_id,
        "company.currency_settings_updated",
    )
    .await
    .unwrap();
    assert_eq!(details["fiscalYearIds"], json!([ctx.s.fiscal_year_id]));

    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/fiscal-years/{clos}/reopen"),
        json!({ "motif": "Correction" }),
    )
    .await;
    assert_eq!(status, 200, "réouverture : {body}");
    assert_eq!(
        body["multiCurrency"], false,
        "rouvert, il reste mono-devise"
    );

    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/fiscal-years",
        json!({ "name": "Exercice 2031", "startDate": "2031-01-01", "endDate": "2031-12-31" }),
    )
    .await;
    assert_eq!(status, 201, "création : {body}");
    assert_eq!(
        body["multiCurrency"], true,
        "créé après l'activation : multi-devises"
    );
}

/// D-CR7-2 : créer un exercice avant un exercice clos à l'activation →
/// `LATER_FISCAL_YEAR_CLOSED` ; une fois celui-ci rouvert →
/// `LATER_FISCAL_YEAR_MONO_CURRENCY` nommant l'exercice rouvert, rien de créé ni
/// d'audité.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn creer_avant_un_exercice_mono_devise_est_refuse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let clos = exercice_sql(&pool, ctx.s.company_id, 2019, "Closed").await;
    activer(&pool, &ctx).await;
    let nouveau =
        json!({ "name": "Exercice 2018", "startDate": "2018-01-01", "endDate": "2018-12-31" });

    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/fiscal-years",
        nouveau.clone(),
    )
    .await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "LATER_FISCAL_YEAR_CLOSED", "{body}");

    let (status, body) = envoyer(
        &ctx,
        reqwest::Method::POST,
        &format!("/api/v1/fiscal-years/{clos}/reopen"),
        json!({ "motif": "Correction" }),
    )
    .await;
    assert_eq!(status, 200, "réouverture : {body}");

    let (status, body) =
        envoyer(&ctx, reqwest::Method::POST, "/api/v1/fiscal-years", nouveau).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(
        body["error"]["code"], "LATER_FISCAL_YEAR_MONO_CURRENCY",
        "{body}"
    );
    assert_eq!(body["error"]["details"]["fiscalYearId"], clos);
    assert_eq!(body["error"]["details"]["fiscalYearName"], "Exercice 2019");
    let crees: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM fiscal_years WHERE name = 'Exercice 2018'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(crees, 0);
    assert_eq!(audits(&pool, "fiscal_year.created").await, 0);
}
