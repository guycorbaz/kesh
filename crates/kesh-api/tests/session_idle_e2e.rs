//! Story v0172-session (0.17.3, #876, CR #891) — l'échéance d'inactivité de la
//! session, côté serveur (AC-3).
//!
//! La durée N se règle en base (`_kesh_installation_settings`, posée ici par du
//! SQL brut) ; `refresh_tokens.expires_at` porte l'échéance de la session
//! (dernière activité + N) et le jeton d'accès en porte une copie, la
//! revendication `idle_exp` (secondes Unix), contrôlée par le middleware sans
//! tolérance. Le renouvellement **recopie** l'échéance (il n'est pas une
//! activité) ; seul le battement `POST /api/v1/auth/activity` la prolonge.
//!
//! Les jetons sont décodés ici par `jsonwebtoken` en `serde_json::Value`, et
//! forgés de même (avec ou sans `idle_exp`) : les tests n'emploient aucune API
//! neuve de la crate, si bien qu'ils compilent sur le code d'avant le correctif
//! — où ils rougissent (table absente, route absente, revendication ignorée).
//! Données synthétiques seulement.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use chrono::{NaiveDateTime, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{companies, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

mod common;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const PASSWORD: &str = "session-idle-password-12";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    state: AppState,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

/// Configuration de test : jeton d'accès de 15 minutes, comme la production.
fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "session-idle-admin-password".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
    .with_test_mode(true)
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready within 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
        state,
    }
}

async fn ensure_company(pool: &MySqlPool) {
    if companies::find_by_id(pool, 1).await.unwrap().is_none() {
        sqlx::query(
            "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
             VALUES (1, 'Test Company', 'Test Address', 'Independant', 'FR', 'FR')",
        )
        .execute(pool)
        .await
        .expect("company insert");
    }
}

async fn create_user(pool: &MySqlPool, username: &str, role: Role) -> i64 {
    ensure_company(pool).await;
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash_password(PASSWORD).unwrap(),
            role,
            active: true,
            company_id: 1,
            email: None,
        },
    )
    .await
    .expect("user create")
    .id
}

/// Pose la durée d'inactivité N (minutes) par du SQL brut.
async fn set_idle_minutes(pool: &MySqlPool, minutes: i32) {
    sqlx::query(
        "INSERT INTO _kesh_installation_settings (id, session_idle_minutes, version) \
         VALUES (1, ?, 1) ON DUPLICATE KEY UPDATE session_idle_minutes = VALUES(session_idle_minutes)",
    )
    .bind(minutes)
    .execute(pool)
    .await
    .expect("réglage de la durée d'inactivité (table _kesh_installation_settings)");
}

/// Connexion : rend (jeton d'accès, jeton de renouvellement, corps).
async fn login(app: &TestApp, username: &str) -> (String, String, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": username, "password": PASSWORD }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login");
    let body: Value = resp.json().await.unwrap();
    (
        body["accessToken"].as_str().unwrap().to_string(),
        body["refreshToken"].as_str().unwrap().to_string(),
        body,
    )
}

/// Les revendications d'un jeton signé par le secret de test, sans contrôle
/// d'échéance.
fn claims(token: &str) -> Value {
    let mut v = Validation::new(Algorithm::HS256);
    v.validate_exp = false;
    v.required_spec_claims = Default::default();
    jsonwebtoken::decode::<Value>(token, &DecodingKey::from_secret(TEST_JWT_SECRET), &v)
        .expect("décodage du jeton")
        .claims
}

/// Forge un jeton d'accès : `exp = now + exp_offset`, et `idle_exp` si fourni.
fn forge(user_id: i64, role: &str, exp_offset: i64, idle_exp: Option<i64>) -> String {
    let now = Utc::now().timestamp();
    let mut c = json!({
        "sub": user_id.to_string(),
        "role": role,
        "company_id": 1,
        "iat": now,
        "exp": now + exp_offset,
    });
    if let Some(d) = idle_exp {
        c["idle_exp"] = json!(d);
    }
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &c,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

/// L'échéance de la ligne du jeton `token` (jeton clair ; la ligne porte son
/// empreinte depuis la 0.17.4, #890).
async fn row_expires_at(pool: &MySqlPool, token: &str) -> NaiveDateTime {
    sqlx::query_scalar("SELECT expires_at FROM refresh_tokens WHERE token = ?")
        .bind(common::independent_refresh_digest(token))
        .fetch_one(pool)
        .await
        .expect("ligne refresh_tokens")
}

fn assert_close(actual: NaiveDateTime, expected: NaiveDateTime, what: &str) {
    let d = (actual - expected).num_milliseconds().abs();
    assert!(
        d <= 2_000,
        "{what} : {actual} au lieu de {expected} (± 2 s), écart {d} ms"
    );
}

/// Le jeton d'accès posé par un `Set-Cookie`, s'il y en a un non vide.
fn access_cookie(resp: &reqwest::Response) -> Option<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|h| h.to_str().ok())
        .find_map(|c| c.strip_prefix("kesh_access_token="))
        .map(|c| c.split(';').next().unwrap_or("").to_string())
        .filter(|v| !v.is_empty())
}

async fn heartbeat(app: &TestApp, access: &str, refresh: &str, body: Value) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/auth/activity"))
        .header(
            "Cookie",
            format!("kesh_access_token={access}; kesh_refresh_token={refresh}"),
        )
        .json(&body)
        .send()
        .await
        .unwrap()
}

// --- (a) et (j) connexion ---------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn login_sets_exact_idle_deadline(pool: MySqlPool) {
    create_user(&pool, "alice", Role::Comptable).await;
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    let before = Utc::now().naive_utc();
    let (at, rt, body) = login(&app, "alice").await;
    let exp_at = row_expires_at(&pool, &rt).await;
    assert_close(
        exp_at,
        before + TimeDelta::minutes(7),
        "échéance de connexion",
    );
    assert_eq!(
        claims(&at)["idle_exp"].as_i64(),
        Some(exp_at.and_utc().timestamp()),
        "idle_exp = expires_at de la ligne, à la seconde"
    );
    assert_eq!(body["idleTimeoutMinutes"], 7);
    let rest = body["idleExpiresIn"].as_i64().expect("idleExpiresIn");
    assert!((418..=420).contains(&rest), "idleExpiresIn = {rest}");
}

// --- (b) middleware ----------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn middleware_refuses_past_idle_exp_whatever_exp(pool: MySqlPool) {
    let uid = create_user(&pool, "bob", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let now = Utc::now().timestamp();
    let me = |tok: String| {
        app.client
            .get(app.url("/api/v1/auth/me"))
            .header("Authorization", format!("Bearer {tok}"))
            .send()
    };
    let passe = forge(uid, "Comptable", 900, Some(now - 1));
    assert_eq!(
        me(passe).await.unwrap().status(),
        401,
        "idle_exp passée ⇒ 401"
    );
    let futur = forge(uid, "Comptable", 900, Some(now + 120));
    assert_eq!(
        me(futur).await.unwrap().status(),
        200,
        "idle_exp future ⇒ 200"
    );
    let sans = forge(uid, "Comptable", 900, None);
    assert_eq!(
        me(sans).await.unwrap().status(),
        200,
        "jeton sans idle_exp (émis avant la mise à jour) ⇒ 200"
    );
}

// --- (c) et (j) renouvellement -----------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refresh_copies_the_deadline(pool: MySqlPool) {
    create_user(&pool, "carol", Role::Comptable).await;
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt, _) = login(&app, "carol").await;
    let target = Utc::now().naive_utc() + TimeDelta::minutes(5);
    let r = sqlx::query("UPDATE refresh_tokens SET expires_at = ? WHERE token = ?")
        .bind(target)
        .bind(common::independent_refresh_digest(&rt))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(r.rows_affected(), 1);
    let posee = row_expires_at(&pool, &rt).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({ "refreshToken": rt }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let new_rt = body["refreshToken"].as_str().unwrap();
    assert_eq!(
        row_expires_at(&pool, new_rt).await,
        posee,
        "le renouvellement recopie l'échéance (à la milliseconde)"
    );
    assert_eq!(
        claims(body["accessToken"].as_str().unwrap())["idle_exp"].as_i64(),
        Some(posee.and_utc().timestamp())
    );
    assert_eq!(body["idleTimeoutMinutes"], 7);
}

// --- (d), (e), (j) battement --------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heartbeat_writes_deadline_from_idle_ms(pool: MySqlPool) {
    let uid = create_user(&pool, "dave", Role::Comptable).await;
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt, _) = login(&app, "dave").await;
    // (e) le jeton présenté dure 5 minutes (≠ 15 de la configuration).
    let presented = forge(uid, "Comptable", 300, Some(Utc::now().timestamp() + 400));
    let exp_presented = claims(&presented)["exp"].as_i64().unwrap();

    let before = Utc::now().naive_utc();
    let resp = heartbeat(&app, &presented, &rt, json!({ "idleMs": 10_000 })).await;
    assert_eq!(resp.status(), 200);
    let reissued = access_cookie(&resp).expect("cookie d'accès réémis");
    let body: Value = resp.json().await.unwrap();
    let exp_at = row_expires_at(&pool, &rt).await;
    assert_close(
        exp_at,
        before - TimeDelta::seconds(10) + TimeDelta::minutes(7),
        "échéance du battement",
    );
    assert_eq!(body["idleTimeoutMinutes"], 7);
    let c = claims(&reissued);
    assert_eq!(c["idle_exp"].as_i64(), Some(exp_at.and_utc().timestamp()));
    assert_eq!(
        c["exp"].as_i64(),
        Some(exp_presented),
        "le battement ne prolonge pas le jeton d'accès"
    );
    assert_eq!(c["sub"], uid.to_string());

    // Borne haute : l'âge ne peut pas dépasser N.
    let before = Utc::now().naive_utc();
    let resp = heartbeat(&app, &reissued, &rt, json!({ "idleMs": 999_999_999u64 })).await;
    assert_eq!(resp.status(), 200);
    assert_close(row_expires_at(&pool, &rt).await, before, "échéance bornée");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heartbeat_rejects_malformed_body_with_400(pool: MySqlPool) {
    create_user(&pool, "erin", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (at, rt, _) = login(&app, "erin").await;
    let avant = row_expires_at(&pool, &rt).await;
    for body in [json!({ "idleMs": -1 }), json!({ "idleMs": 1.5 }), json!({})] {
        let resp = heartbeat(&app, &at, &rt, body.clone()).await;
        assert_eq!(resp.status(), 400, "corps {body}");
        let b: Value = resp.json().await.unwrap();
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR", "corps {body}");
    }
    assert_eq!(row_expires_at(&pool, &rt).await, avant, "rien d'écrit");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heartbeat_within_leeway_writes_but_sets_no_access_cookie(pool: MySqlPool) {
    let uid = create_user(&pool, "fred", Role::Comptable).await;
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt, _) = login(&app, "fred").await;
    let presented = forge(uid, "Comptable", -30, Some(Utc::now().timestamp() + 400));
    let before = Utc::now().naive_utc();
    let resp = heartbeat(&app, &presented, &rt, json!({ "idleMs": 0 })).await;
    assert_eq!(resp.status(), 200);
    assert!(
        access_cookie(&resp).is_none(),
        "aucun Set-Cookie d'accès quand exp est passé"
    );
    assert_close(
        row_expires_at(&pool, &rt).await,
        before + TimeDelta::minutes(7),
        "échéance écrite",
    );
}

// --- (f) cookie d'un autre, ligne révoquée ou expirée, clé d'API --------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heartbeat_refuses_foreign_revoked_or_expired_rows(pool: MySqlPool) {
    create_user(&pool, "gina", Role::Comptable).await;
    create_user(&pool, "hugo", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (at_g, _rt_g, _) = login(&app, "gina").await;
    let (_at_h, rt_h, _) = login(&app, "hugo").await;
    let avant = row_expires_at(&pool, &rt_h).await;
    let resp = heartbeat(&app, &at_g, &rt_h, json!({ "idleMs": 0 })).await;
    assert_eq!(resp.status(), 401, "cookie d'un autre utilisateur");
    assert_eq!(
        row_expires_at(&pool, &rt_h).await,
        avant,
        "sa ligne inchangée"
    );

    let (at_g2, rt_g2, _) = login(&app, "gina").await;
    let r = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW(3), revoked_reason = 'logout' WHERE token = ?",
    )
    .bind(common::independent_refresh_digest(&rt_g2))
    .execute(&pool)
    .await
    .unwrap();
    assert_eq!(r.rows_affected(), 1);
    let resp = heartbeat(&app, &at_g2, &rt_g2, json!({ "idleMs": 0 })).await;
    assert_eq!(resp.status(), 401, "ligne révoquée");

    let (at_g3, rt_g3, _) = login(&app, "gina").await;
    let r = sqlx::query("UPDATE refresh_tokens SET expires_at = ? WHERE token = ?")
        .bind(Utc::now().naive_utc() - TimeDelta::seconds(5))
        .bind(common::independent_refresh_digest(&rt_g3))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(r.rows_affected(), 1);
    let resp = heartbeat(&app, &at_g3, &rt_g3, json!({ "idleMs": 0 })).await;
    assert_eq!(resp.status(), 401, "ligne expirée");

    let resp = app
        .client
        .post(app.url("/api/v1/auth/activity"))
        .header("Authorization", format!("Bearer {at_g3}"))
        .json(&json!({ "idleMs": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "sans cookie de renouvellement");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn heartbeat_refuses_api_keys(pool: MySqlPool) {
    create_user(&pool, "ivan", Role::Admin).await;
    let app = spawn_app(pool.clone()).await;
    let (at, _rt, _) = login(&app, "ivan").await;
    let (_id, key) =
        common::create_key_via_http(&app.client, &app.base_url, &at, "battement", "read-write")
            .await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/activity"))
        .header("Authorization", format!("Bearer {key}"))
        .json(&json!({ "idleMs": 0 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "clé d'API refusée");

    // Revue de code P1 (S-B4/S-E8) : le refus de la clé d'API précède la
    // lecture du corps — un corps illisible rend 403, non 400.
    for corps in [
        json!({ "idleMs": -1 }),
        json!({}),
        json!({ "idleMs": "abc" }),
    ] {
        let resp = app
            .client
            .post(app.url("/api/v1/auth/activity"))
            .header("Authorization", format!("Bearer {key}"))
            .json(&corps)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403, "clé d'API, corps {corps}");
    }
    // Témoin : le même corps illisible, par une session, rend bien 400.
    let resp = app
        .client
        .post(app.url("/api/v1/auth/activity"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({ "idleMs": -1 }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400, "session, corps illisible");
}

// --- (g) au-delà de l'échéance ---------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn nothing_is_accepted_past_the_deadline(pool: MySqlPool) {
    let uid = create_user(&pool, "jane", Role::Comptable).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt, _) = login(&app, "jane").await;
    let passe = Utc::now().naive_utc() - TimeDelta::minutes(1);
    let r = sqlx::query("UPDATE refresh_tokens SET expires_at = ? WHERE token = ?")
        .bind(passe)
        .bind(common::independent_refresh_digest(&rt))
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(r.rows_affected(), 1);
    let posee = row_expires_at(&pool, &rt).await;
    let at = forge(uid, "Comptable", 900, Some(passe.and_utc().timestamp()));
    let resp = heartbeat(&app, &at, &rt, json!({ "idleMs": 0 })).await;
    assert_eq!(resp.status(), 401, "battement après l'échéance");
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({ "refreshToken": rt }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "renouvellement après l'échéance");
    assert_eq!(
        row_expires_at(&pool, &rt).await,
        posee,
        "aucune ligne prolongée"
    );
    let actives: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND expires_at > ?",
    )
    .bind(uid)
    .bind(Utc::now().naive_utc())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actives, 0, "aucune ligne vivante");
}

// --- (h) assistant -----------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn setup_admin_sets_exact_idle_deadline(pool: MySqlPool) {
    kesh_db::test_fixtures::truncate_all(&pool).await.unwrap();
    kesh_db::test_fixtures::seed_stub_company_only(&pool)
        .await
        .unwrap();
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    app.state.users_exist.store(false, Ordering::Release);
    let before = Utc::now().naive_utc();
    let resp = app
        .client
        .post(app.url("/api/v1/setup/admin"))
        .json(&json!({ "username": "premier", "password": "first-admin-password-secure-12" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rt = body["refreshToken"].as_str().unwrap();
    let exp_at = row_expires_at(&pool, rt).await;
    assert_close(
        exp_at,
        before + TimeDelta::minutes(7),
        "échéance de l'assistant",
    );
    assert_eq!(
        claims(body["accessToken"].as_str().unwrap())["idle_exp"].as_i64(),
        Some(exp_at.and_utc().timestamp())
    );
    assert_eq!(body["idleTimeoutMinutes"], 7);
}

// --- (i) changement de mot de passe -----------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn change_password_sets_exact_idle_deadline(pool: MySqlPool) {
    create_user(&pool, "kim", Role::Comptable).await;
    set_idle_minutes(&pool, 7).await;
    let app = spawn_app(pool.clone()).await;
    let (at, _rt, _) = login(&app, "kim").await;
    let before = Utc::now().naive_utc();
    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({ "currentPassword": PASSWORD, "newPassword": "un-autre-mot-de-passe-12" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rt = body["refreshToken"].as_str().unwrap();
    let exp_at = row_expires_at(&pool, rt).await;
    assert_close(
        exp_at,
        before + TimeDelta::minutes(7),
        "échéance du mot de passe",
    );
    assert_eq!(
        claims(body["accessToken"].as_str().unwrap())["idle_exp"].as_i64(),
        Some(exp_at.and_utc().timestamp())
    );
}

// --- /me rend la durée et le reste --------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn me_returns_idle_fields(pool: MySqlPool) {
    let uid = create_user(&pool, "lea", Role::Comptable).await;
    set_idle_minutes(&pool, 9).await;
    let app = spawn_app(pool).await;
    let now = Utc::now().timestamp();
    let tok = forge(uid, "Comptable", 900, Some(now + 300));
    let body: Value = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .header("Authorization", format!("Bearer {tok}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(body["idleTimeoutMinutes"], 9);
    let rest = body["idleExpiresIn"].as_i64().expect("idleExpiresIn");
    assert!((298..=300).contains(&rest), "idleExpiresIn = {rest}");
    let sans = forge(uid, "Comptable", 900, None);
    let body: Value = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .header("Authorization", format!("Bearer {sans}"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["idleExpiresIn"].is_null(), "sans idle_exp ⇒ null");
}
