//! Story 10-5 — Tests d'intégration auth via cookies HttpOnly (T9).
//!
//! 4 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` couvrant :
//!
//! - `login_sets_two_httponly_cookies` (AC #1) — login émet 2 headers
//!   Set-Cookie avec flags HttpOnly + SameSite=Strict + paths corrects
//!   (`/` pour access, `/api/v1/auth` pour refresh).
//! - `authenticated_request_with_cookie_only` (AC #6) — request authentifiée
//!   avec cookie seul (sans header Authorization) → 200 OK.
//! - `authenticated_request_with_authorization_only` (AC #6 fallback) — request
//!   authentifiée avec header Bearer seul (sans cookie) → 200 OK.
//! - `logout_invalidates_cookie` (AC #4) — logout émet Set-Cookie avec
//!   Max-Age=0 + révoque refresh_token en DB.
//!
//! Pass 1 F-T9-P1-10 — utilise un helper LOCAL `spawn_app_with_cookie_jar`
//! (pas dans `common/`) qui retourne un `TestApp` avec
//! `reqwest::Client::builder().cookie_store(true).build()`. NE PAS modifier
//! `spawn_app()` global dans `auth_e2e.rs` (préserve l'isolation de session
//! des 19+ tests existants qui utilisent Authorization: Bearer explicite).

use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Extension, State};
use axum::routing::get;
use axum::{Json, Router};
use chrono::TimeDelta;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::middleware::auth::CurrentUser;
use kesh_api::{AppState, build_router};
use kesh_db::repositories::companies;
use serde_json::json;
use sqlx::MySqlPool;

// Pattern aligné sur tests/auth_e2e.rs:175 — create test company id=1 via raw SQL.

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-cookies-admin-password";

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
    // CR Pass 3 ECH3-H2 : `.with_test_mode(true)` est REQUIS pour que les
    // cookies émis aient `Secure=false` — sans ça, reqwest peut refuser de
    // renvoyer les cookies `Secure` sur l'URL HTTP du test (RFC 6265 §5.4).
    // Le commentaire historique ligne ~219 (« Note Pass 4 F-TEST-SECURE-
    // ASSERTION-P4-5 : flag Secure ABSENT en test_mode → false ») partait du
    // postulat — corrigé ici — que test_mode était bien activé.
    kesh_api::config::Config::from_fields_for_test(
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
    .with_test_mode(true)
}

/// Helper LOCAL Story 10-5 — TestApp avec cookie jar reqwest activé.
///
/// Différent du `spawn_app()` global d'`auth_e2e.rs` (qui utilise
/// `reqwest::Client::new()` sans cookie_store) — voir Pass 1 F-T9-P1-10
/// rationale dans la spec.
async fn spawn_app_with_cookie_jar(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = std::sync::Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
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
        i18n: i18n.clone(),
    };

    let protected_test_router: Router<AppState> = Router::new()
        .route("/api/v1/_test/me", get(test_me_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            kesh_api::middleware::auth::require_auth,
        ));

    let prod_router = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let app = prod_router.merge(protected_test_router.with_state(state));

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

    // Attendre que le serveur accepte les connexions.
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
        base_url: format!("http://{}", addr),
        client: reqwest::Client::builder()
            .cookie_store(true)
            .build()
            .expect("build reqwest client with cookie_store"),
    }
}

async fn test_me_handler(
    Extension(user): Extension<CurrentUser>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "userId": user.user_id,
        "role": user.role.as_str(),
        "companyId": user.company_id,
    })))
}

/// Crée la company de test id=1 via raw SQL (pattern aligné sur auth_e2e.rs:175).
async fn ensure_test_company(pool: &MySqlPool) {
    let existing = companies::find_by_id(pool, 1)
        .await
        .expect("find should succeed");
    if existing.is_none() {
        sqlx::query(
            "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
             VALUES (1, 'Test Company', 'Test Address', 'Independant', 'FR', 'FR')",
        )
        .execute(pool)
        .await
        .expect("company insert should succeed");
    }
}

/// Setup admin via ensure_admin_user + ensure_test_company. Retourne (admin_username, company_id).
async fn setup_admin(pool: &MySqlPool) -> (String, i64) {
    ensure_test_company(pool).await;
    let config = test_config();
    ensure_admin_user(pool, &config)
        .await
        .expect("ensure admin");
    (config.admin_username.clone(), 1)
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_sets_two_httponly_cookies(pool: MySqlPool) {
    let (admin_username, _company_id) = setup_admin(&pool).await;
    let app = spawn_app_with_cookie_jar(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "username": admin_username,
            "password": TEST_ADMIN_PASSWORD,
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200, "login should succeed");

    // Extraire tous les headers Set-Cookie (peut y avoir plusieurs).
    let set_cookies: Vec<String> = resp
        .headers()
        .get_all("set-cookie")
        .into_iter()
        .map(|h| h.to_str().unwrap().to_string())
        .collect();

    assert_eq!(
        set_cookies.len(),
        2,
        "should emit 2 Set-Cookie headers (access + refresh), got {set_cookies:?}"
    );

    // Trouver les 2 cookies par nom.
    let access_cookie = set_cookies
        .iter()
        .find(|c| c.starts_with("kesh_access_token="))
        .expect("kesh_access_token cookie missing");
    let refresh_cookie = set_cookies
        .iter()
        .find(|c| c.starts_with("kesh_refresh_token="))
        .expect("kesh_refresh_token cookie missing");

    // AC #1 + AC #2 — flags HttpOnly et SameSite=Strict obligatoires.
    // CR Pass 3 ECH3-H2 : `test_config()` active maintenant `test_mode=true`,
    // donc `.secure(!test_mode)` → `false`. Le flag `Secure` est ABSENT des
    // headers ici — testé en présence par les tests E2E Playwright en browser
    // context (xss-token-protection.spec.ts Scénario c).
    assert!(
        access_cookie.contains("HttpOnly"),
        "access cookie missing HttpOnly: {access_cookie}"
    );
    assert!(
        access_cookie.contains("SameSite=Strict"),
        "access cookie missing SameSite=Strict: {access_cookie}"
    );
    assert!(
        access_cookie.contains("Path=/;") || access_cookie.contains("Path=/ "),
        "access cookie path should be /, got: {access_cookie}"
    );
    // CR Pass 3 AA3-M3 : asserter la valeur exacte `Max-Age=900` (= jwt_expiry
    // 15 minutes de `test_config()` paramètre 5 `TimeDelta::minutes(15)`).
    // Protège contre une régression silencieuse F-COOKIE-LIFETIME-P3-1 qui
    // utiliserait `refresh_inactivity` (15 min sliding) au lieu de la durée
    // canonique du token sur l'access cookie.
    assert!(
        access_cookie.contains("Max-Age=900"),
        "access cookie Max-Age should be 900 (jwt_expiry 15min), got: {access_cookie}"
    );

    assert!(
        refresh_cookie.contains("HttpOnly"),
        "refresh cookie missing HttpOnly: {refresh_cookie}"
    );
    assert!(
        refresh_cookie.contains("SameSite=Strict"),
        "refresh cookie missing SameSite=Strict: {refresh_cookie}"
    );
    assert!(
        refresh_cookie.contains("Path=/api/v1/auth"),
        "refresh cookie path should be /api/v1/auth (scope restriction), got: {refresh_cookie}"
    );
    // CR Pass 3 AA3-M3 : asserter la valeur exacte `Max-Age=2592000` (= refresh
    // hard ceiling 30 jours de `test_config()` paramètre 6 `TimeDelta::days(30)`).
    // F-COOKIE-LIFETIME-P3-1 : ce ceiling est `refresh_token_max_lifetime` (30j
    // hard côté browser), PAS `refresh_inactivity` (15 min sliding côté serveur).
    assert!(
        refresh_cookie.contains("Max-Age=2592000"),
        "refresh cookie Max-Age should be 2592000 (refresh_token_max_lifetime 30d hard ceiling, NOT refresh_inactivity 15min sliding), got: {refresh_cookie}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn authenticated_request_with_cookie_only(pool: MySqlPool) {
    // AC #6 — cookie seul (sans header Authorization) → 200 OK.
    let (admin_username, _company_id) = setup_admin(&pool).await;
    let app = spawn_app_with_cookie_jar(pool).await;

    // Login → reqwest cookie_store stocke automatiquement les cookies.
    let login_resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "username": admin_username,
            "password": TEST_ADMIN_PASSWORD,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_resp.status(), 200);

    // Request authentifiée — pas de header Authorization (juste cookie).
    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "authenticated request with cookie only should succeed (AC #6)"
    );

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["role"].as_str().unwrap(), "Admin");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn authenticated_request_with_authorization_only(pool: MySqlPool) {
    // AC #6 fallback — header Bearer seul (sans cookie) → 200 OK.
    let (admin_username, _company_id) = setup_admin(&pool).await;
    let app = spawn_app_with_cookie_jar(pool).await;

    // Login pour obtenir l'access_token du body (D1 Option A — tokens
    // conservés en body pour rétro-compat).
    let login_resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "username": admin_username,
            "password": TEST_ADMIN_PASSWORD,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_resp.status(), 200);
    let login_body: serde_json::Value = login_resp.json().await.unwrap();
    let access_token = login_body["accessToken"].as_str().unwrap().to_string();

    // Nouveau client SANS cookie_store pour s'assurer qu'aucun cookie n'est envoyé.
    let bare_client = reqwest::Client::new();
    let resp = bare_client
        .get(app.url("/api/v1/_test/me"))
        .bearer_auth(&access_token)
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "authenticated request with Authorization Bearer fallback should succeed (AC #6)"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn logout_invalidates_cookie(pool: MySqlPool) {
    // AC #4 / AC #15(d) — logout émet Set-Cookie Max-Age=0 + révoque refresh_token DB.
    // CR Pass 1 H1 : ajout DB revocation check + 401 post-logout (cohérent
    // pattern auth_e2e.rs:638-645 `logout_revokes_refresh_token`).
    let (admin_username, _company_id) = setup_admin(&pool).await;
    let pool_for_query = pool.clone(); // Clone pour DB query post-logout (pool est moved par spawn_app)
    let app = spawn_app_with_cookie_jar(pool).await;

    // Login.
    let login_resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "username": admin_username,
            "password": TEST_ADMIN_PASSWORD,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_resp.status(), 200);

    // Capture le refresh_token depuis le body (D1 Option A — tokens conservés en body).
    let login_body: serde_json::Value = login_resp.json().await.unwrap();
    let refresh_token = login_body["refreshToken"]
        .as_str()
        .expect("login response should contain refreshToken (D1)")
        .to_string();

    // Vérifier que le refresh_token est NON-révoqué avant logout (sanity check).
    let revoked_before: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM refresh_tokens WHERE token = ?")
            .bind(&refresh_token)
            .fetch_one(&pool_for_query)
            .await
            .expect("refresh_token should exist post-login");
    assert!(
        revoked_before.is_none(),
        "refresh_token should NOT be revoked before logout, got: {revoked_before:?}"
    );

    // Logout (sans body — le browser envoie le cookie automatiquement).
    let logout_resp = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({})) // body vide accepté (refresh_token Option<String>)
        .send()
        .await
        .unwrap();

    assert_eq!(
        logout_resp.status(),
        204,
        "logout should return 204 No Content"
    );

    // Vérifier les 2 Set-Cookie avec Max-Age=0.
    let set_cookies: Vec<String> = logout_resp
        .headers()
        .get_all("set-cookie")
        .into_iter()
        .map(|h| h.to_str().unwrap().to_string())
        .collect();

    assert_eq!(
        set_cookies.len(),
        2,
        "logout should emit 2 expired Set-Cookie headers, got {set_cookies:?}"
    );

    // CR Pass 3 BH3-M1 ∪ AA3-M4 : discriminer par nom et asserter `Path` par
    // cookie expiré. HTML5 cookie deletion (RFC 6265 §3.1) exige que le triple
    // `(name, path, domain)` du Set-Cookie expiré matche celui du cookie
    // stocké pour le supprimer. Sans assertion `Path=`, un futur refactor qui
    // mettrait `Path=/api/v1/auth` sur l'access cookie expiré ne purgerait
    // PAS le cookie access stocké `Path=/` → user resterait apparemment
    // authentifié sur les autres routes API. Le test passait silencieusement
    // (Max-Age=0 + HttpOnly OK), masquant la régression.
    let expired_access = set_cookies
        .iter()
        .find(|c| c.starts_with("kesh_access_token="))
        .expect("expired kesh_access_token cookie missing post-logout");
    let expired_refresh = set_cookies
        .iter()
        .find(|c| c.starts_with("kesh_refresh_token="))
        .expect("expired kesh_refresh_token cookie missing post-logout");

    assert!(
        expired_access.contains("Max-Age=0"),
        "expired access cookie should have Max-Age=0 for invalidation: {expired_access}"
    );
    assert!(
        expired_access.contains("HttpOnly"),
        "expired access cookie should still have HttpOnly: {expired_access}"
    );
    assert!(
        expired_access.contains("Path=/;") || expired_access.contains("Path=/ "),
        "expired access cookie must keep Path=/ to match the original (HTML5 RFC 6265 §3.1), got: {expired_access}"
    );

    assert!(
        expired_refresh.contains("Max-Age=0"),
        "expired refresh cookie should have Max-Age=0 for invalidation: {expired_refresh}"
    );
    assert!(
        expired_refresh.contains("HttpOnly"),
        "expired refresh cookie should still have HttpOnly: {expired_refresh}"
    );
    assert!(
        expired_refresh.contains("Path=/api/v1/auth"),
        "expired refresh cookie must keep Path=/api/v1/auth to match the original (HTML5 RFC 6265 §3.1), got: {expired_refresh}"
    );

    // CR Pass 1 H1 — AC #15(d) DB revocation check : le refresh_token doit
    // être marqué `revoked_at IS NOT NULL` en base après logout (révocation
    // applicative préservée Story 1.6 + Story 10-5 cookie-only flow).
    let revoked_after: Option<chrono::DateTime<chrono::Utc>> =
        sqlx::query_scalar("SELECT revoked_at FROM refresh_tokens WHERE token = ?")
            .bind(&refresh_token)
            .fetch_one(&pool_for_query)
            .await
            .expect("refresh_token row should still exist post-logout");
    assert!(
        revoked_after.is_some(),
        "refresh_token should be revoked_at IS NOT NULL after logout, got: {revoked_after:?}"
    );

    // CR Pass 4 AA4-L1 — AC #15(d) scénario (iii) : « requête authentifiée
    // subséquente avec l'ancien cookie → 401 ». L'invariant était documenté
    // en commentaire Pass 1 H1 mais sans assertion exécutable. La requête
    // GET sur la route protégée `_test/me` utilise le même `app.client`
    // qui a déjà reçu les `Set-Cookie: Max-Age=0` du logout — reqwest
    // `cookie_store` les a purgés automatiquement → cookie absent →
    // middleware require_auth retourne 401. Si une régression future
    // (e.g. cookie pas purgé ou révocation DB cassée) laissait le cookie
    // valide post-logout, ce test échouerait explicitement.
    let post_logout_resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(
        post_logout_resp.status(),
        401,
        "authenticated request after logout should return 401 (cookie purged client-side + DB revoked, AC #15(d) scenario iii)"
    );
}

/// CR Pass 3 AA3-M2 — T4.3 — test intégration sur le **vrai** endpoint
/// `GET /api/v1/auth/me` (AC #5). Les 4 tests précédents montaient un handler
/// custom `/api/v1/_test/me` qui ne couvrait PAS la route réelle consommée par
/// le frontend (`hydrate()` dans `auth.svelte.ts`). Une régression sur le
/// shape de `MeResponse` (e.g. `user_id` au lieu de `userId`, `format!("{:?}",
/// role)` qui diverge de `role.as_str()`, `expires_in` négatif sans clamp,
/// `username` manquant) passerait silencieusement les autres tests.
///
/// Vérifie : login → cookie auto-stocké → GET /api/v1/auth/me → 200 + body
/// `{ userId, username, role, expiresIn }` avec valeurs cohérentes.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn me_endpoint_returns_user_identity_from_cookie(pool: MySqlPool) {
    let (admin_username, _company_id) = setup_admin(&pool).await;
    let app = spawn_app_with_cookie_jar(pool).await;

    // Login → cookie store reqwest stocke automatiquement les cookies HttpOnly.
    let login_resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({
            "username": admin_username,
            "password": TEST_ADMIN_PASSWORD,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(login_resp.status(), 200, "login should succeed");
    let login_body: serde_json::Value = login_resp.json().await.unwrap();
    let expected_user_id = login_body["userId"]
        .as_i64()
        .expect("login response should contain userId i64 (D6)");

    // GET /api/v1/auth/me — vraie route prod, pas le handler custom /_test/me.
    let me_resp = app
        .client
        .get(app.url("/api/v1/auth/me"))
        .send()
        .await
        .unwrap();

    assert_eq!(
        me_resp.status(),
        200,
        "GET /api/v1/auth/me should return 200 OK with cookie auth (AC #5)"
    );

    let body: serde_json::Value = me_resp.json().await.unwrap();

    // Shape strict — chaque champ exigé par AC #5 doit être présent et bien typé.
    let user_id = body["userId"]
        .as_i64()
        .expect("MeResponse.userId should be present as i64 (AC #5 + camelCase serde)");
    let username = body["username"]
        .as_str()
        .expect("MeResponse.username should be present as string (AC #5)");
    let role = body["role"]
        .as_str()
        .expect("MeResponse.role should be present as string (AC #5)");
    let expires_in = body["expiresIn"]
        .as_i64()
        .expect("MeResponse.expiresIn should be present as i64 (AC #5 + camelCase serde)");

    assert_eq!(
        user_id, expected_user_id,
        "MeResponse.userId should match login response userId"
    );
    assert_eq!(
        username, admin_username,
        "MeResponse.username should match the logged-in admin username"
    );
    assert_eq!(
        role, "Admin",
        "MeResponse.role should be the canonical string 'Admin' for the admin user"
    );
    assert!(
        expires_in > 0,
        "MeResponse.expiresIn should be positive (token fresh post-login), got: {expires_in}"
    );
    assert!(
        expires_in <= 900,
        "MeResponse.expiresIn should be ≤ 900s (jwt_expiry 15min ceiling), got: {expires_in}"
    );
}
