//! Tests d'intégration E2E de l'authentification (story 1.5).
//!
//! Chaque test utilise `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`
//! pour obtenir une DB propre avec les migrations appliquées. Le helper
//! `spawn_app` construit un routeur Axum via `build_router` puis merge
//! une route de test protégée `/api/v1/_test/me` pour valider l'injection
//! de `CurrentUser` via le middleware.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use axum::extract::{Extension, State};
use axum::routing::{get, put};
use axum::{Json, Router};
use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::middleware::auth::CurrentUser;
use kesh_api::{build_router, AppState};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
use serde_json::json;
use sqlx::MySqlPool;

/// Secret JWT utilisé par tous les tests ≥ 32 bytes.
const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

/// Application de test : serveur Axum bind sur port éphémère + client HTTP.
struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

/// Mot de passe admin explicite pour les tests E2E.
///
/// Volontairement distinct de `"changeme"` (le défaut de dev) et de
/// `"bootstrap-e2e-password"` (celui utilisé par les tests bootstrap E2E)
/// pour éviter toute confusion entre helpers de test et comportement
/// bootstrap réel. Patch V3.
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

/// Construit un `Config` de test sans toucher aux variables d'env.
///
/// Utilise `from_fields_for_test` qui applique désormais (patch #4) les
/// mêmes invariants que `from_env()` via des `assert!`.
fn test_config() -> Config {
    kesh_api::config::Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100, // High threshold for non-rate-limit tests (timing test does 30+ attempts)
        TimeDelta::minutes(30),
    )
}

/// Config spécifique pour les tests de rate limiting avec seuil bas.
fn test_config_rate_limit(max_attempts: u32, block_secs: i64) -> Config {
    kesh_api::config::Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        TEST_ADMIN_PASSWORD.to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        max_attempts,
        TimeDelta::seconds(block_secs),
    )
}

/// Construit un routeur complet (prod + route de test protégée) et lance
/// un serveur éphémère sur 127.0.0.1:0.
async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
    };

    // Sous-routeur de test protégé : route AVANT layer (Axum 0.8 exige
    // qu'il y ait au moins une route quand on applique route_layer).
    let protected_test_router: Router<AppState> = Router::new()
        .route("/api/v1/_test/me", get(test_me_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            kesh_api::middleware::auth::require_auth,
        ));

    // Routeur prod (public) construit via build_router, puis on merge
    // le sous-routeur de test. build_router fait déjà `.with_state(state)`,
    // donc protected_test_router doit être Router<AppState> avant le merge.
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

    // Patch #12 : attendre activement que le serveur accepte des connexions
    // avant de retourner le TestApp. `yield_now` seul ne garantit pas que
    // `axum::serve` a atteint son `accept()`, causant des flakys sous charge
    // CI (connection refused sur le premier reqwest). On boucle sur un
    // connect TCP court avec timeout total de 2 secondes.
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
        client: reqwest::Client::new(),
    }
}

/// Handler de test qui retourne l'identité extraite du middleware.
async fn test_me_handler(
    Extension(user): Extension<CurrentUser>,
    State(_state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(json!({
        "userId": user.user_id,
        "role": user.role.as_str(),
    })))
}

/// Crée un utilisateur dans la DB pour les besoins du test.
async fn create_user(pool: &MySqlPool, username: &str, password: &str, active: bool) -> i64 {
    let phc = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: phc,
            role: Role::Comptable,
            active,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

// =========================================================================
// === Login tests ===
// =========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_success_returns_tokens(pool: MySqlPool) {
    let user_id = create_user(&pool, "alice", "password123", true).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "alice", "password": "password123"}))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());
    assert_eq!(body["expiresIn"], 900); // 15 min * 60

    // Décoder le JWT pour vérifier le sub
    let token = body["accessToken"].as_str().unwrap();
    let mut validation = jsonwebtoken::Validation::new(Algorithm::HS256);
    validation.leeway = 60;
    validation.required_spec_claims = ["exp", "sub", "iat"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(TEST_JWT_SECRET),
        &validation,
    )
    .expect("JWT should decode");
    assert_eq!(decoded.claims.sub, user_id.to_string());
    assert_eq!(decoded.claims.role, "Comptable");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_unknown_username_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "nobody", "password": "whatever"}))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_wrong_password_returns_401(pool: MySqlPool) {
    create_user(&pool, "bob", "correct-password", true).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "bob", "password": "wrong-password"}))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_inactive_user_returns_401(pool: MySqlPool) {
    create_user(&pool, "carol", "password123", false).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "carol", "password": "password123"}))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_empty_fields_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "", "password": ""}))
        .send()
        .await
        .expect("request should succeed");

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Patch #15 : cas per-field — seul le username vide.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_empty_username_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "   ", "password": "non-empty"}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Patch #15 : cas per-field — seul le password vide.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_empty_password_returns_400(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "alice", "password": ""}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Patch V2 anti-régression : un password composé UNIQUEMENT de whitespace
/// doit être rejeté avec 400 (symétrie avec le username trim et fermeture
/// du side-channel d'énumération "password vide" vs "password whitespace").
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_whitespace_only_password_returns_400(pool: MySqlPool) {
    create_user(&pool, "alice-v2", "real-password", true).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "alice-v2", "password": "    "}))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        400,
        "whitespace-only password should be rejected at validation"
    );
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

/// Patch V2 anti-régression : un password qui COMMENCE ou FINIT par un
/// espace mais contient du contenu non-whitespace doit être accepté
/// (byte-exact semantics préservées).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_password_with_leading_trailing_spaces_is_accepted(pool: MySqlPool) {
    // Créer un user avec un password qui a des espaces autour
    let user_pwd = "  secret  ";
    create_user(&pool, "alice-v2b", user_pwd, true).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "alice-v2b", "password": user_pwd}))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "password with leading/trailing spaces is valid (byte-exact semantics)"
    );
}

/// Patch #8 anti-régression : un username avec whitespace autour (`"alice "`)
/// doit matcher le user `"alice"` en base et fonctionner comme un login normal.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_trims_username_whitespace(pool: MySqlPool) {
    create_user(&pool, "alice", "password123", true).await;
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "  alice  ", "password": "password123"}))
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        200,
        "whitespace-wrapped username should be trimmed and match the user"
    );
}

/// Patch #9 anti-régression : l'en-tête Authorization avec scheme
/// lowercase (`bearer`) doit être accepté (RFC 7235 §2.1).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_accepts_lowercase_bearer_scheme(pool: MySqlPool) {
    let user_id = create_user(&pool, "case-insensitive-user", "password123", true).await;
    let app = spawn_app(pool).await;

    let login_resp: serde_json::Value = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "case-insensitive-user", "password": "password123"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login_resp["accessToken"].as_str().unwrap();

    // Lowercase "bearer"
    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Mixed case "BeArEr"
    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("BeArEr {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["userId"], user_id);
}

/// Patch #16 : logout avec refreshToken manquant ou null doit être
/// rejeté par serde avant d'atteindre le handler (422 Unprocessable Entity).
/// Document le comportement attendu et prévient toute future régression
/// vers un 500 ou un 204 silencieux.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn logout_with_missing_refresh_token_field_returns_422(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({})) // champ refreshToken absent
        .send()
        .await
        .unwrap();

    assert_eq!(
        resp.status(),
        422,
        "missing refreshToken field should be rejected by serde with 422"
    );
}

/// Test anti-régression timing-attack (M5/M-B).
///
/// Mesure N=10 médianes pour 3 scénarios (user absent, inactif, actif +
/// bad password). Tolérance large 5× pour absorber le jitter CI, mais
/// suffisante pour détecter une suppression accidentelle de `dummy_verify`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_timing_normalized(pool: MySqlPool) {
    create_user(&pool, "active_user", "correct-password", true).await;
    create_user(&pool, "inactive_user", "correct-password", false).await;
    let app = spawn_app(pool).await;

    // N=10 comme spécifié (patch #6) — robuste statistiquement.
    // Coût : Argon2 ~50ms × 30 appels ≈ 1.5s. Acceptable pour CI.
    const N: usize = 10;
    let mut durations_absent = Vec::with_capacity(N);
    let mut durations_inactive = Vec::with_capacity(N);
    let mut durations_bad_pwd = Vec::with_capacity(N);

    // Chauffe LazyLock DUMMY_HASH via un premier appel absent (ignoré dans les mesures)
    let _ = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "warmup", "password": "warmup"}))
        .send()
        .await;

    for _ in 0..N {
        let start = Instant::now();
        let _ = app
            .client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "ghost", "password": "any"}))
            .send()
            .await
            .unwrap();
        durations_absent.push(start.elapsed().as_millis() as u64);

        let start = Instant::now();
        let _ = app
            .client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "inactive_user", "password": "correct-password"}))
            .send()
            .await
            .unwrap();
        durations_inactive.push(start.elapsed().as_millis() as u64);

        let start = Instant::now();
        let _ = app
            .client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "active_user", "password": "wrong"}))
            .send()
            .await
            .unwrap();
        durations_bad_pwd.push(start.elapsed().as_millis() as u64);
    }

    fn median(mut v: Vec<u64>) -> u64 {
        v.sort();
        let n = v.len();
        assert!(n > 0, "median of empty vec");
        if n % 2 == 1 {
            v[n / 2]
        } else {
            // Patch #17 : moyenne des deux valeurs centrales sur N pair.
            (v[n / 2 - 1] + v[n / 2]) / 2
        }
    }

    let median_absent = median(durations_absent);
    let median_inactive = median(durations_inactive);
    let median_bad_pwd = median(durations_bad_pwd);

    eprintln!(
        "Timing medians (ms): absent={}, inactive={}, bad_pwd={}",
        median_absent, median_inactive, median_bad_pwd
    );

    let max = median_absent.max(median_inactive).max(median_bad_pwd);
    let min = median_absent.min(median_inactive).min(median_bad_pwd);

    assert!(
        min >= 10,
        "all medians should be > 10ms (sanity check — Argon2 should take ~50ms), got min={}ms",
        min
    );
    assert!(
        (max as f64) / (min as f64) < 5.0,
        "timing ratio max/min should be < 5x, got max={}ms min={}ms (ratio={})",
        max,
        min,
        (max as f64) / (min as f64)
    );
}

// =========================================================================
// === Logout tests ===
// =========================================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn logout_revokes_refresh_token(pool: MySqlPool) {
    create_user(&pool, "dave", "password123", true).await;
    let app = spawn_app(pool.clone()).await;

    // 1. Login pour obtenir un refresh_token
    let login_resp: serde_json::Value = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "dave", "password": "password123"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let refresh_token = login_resp["refreshToken"].as_str().unwrap().to_string();

    // 2. Logout
    let resp = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({"refreshToken": refresh_token}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    // 3. Vérifier en base que revoked_at IS NOT NULL
    let revoked_at: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT revoked_at FROM refresh_tokens WHERE token = ?")
            .bind(&refresh_token)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(revoked_at.is_some(), "refresh_token should be revoked in DB");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn logout_idempotent(pool: MySqlPool) {
    create_user(&pool, "eve", "password123", true).await;
    let app = spawn_app(pool).await;

    let login_resp: serde_json::Value = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "eve", "password": "password123"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let refresh_token = login_resp["refreshToken"].as_str().unwrap().to_string();

    // Deux logouts consécutifs → 204 dans les deux cas
    for _ in 0..2 {
        let resp = app
            .client
            .post(app.url("/api/v1/auth/logout"))
            .json(&json!({"refreshToken": refresh_token}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 204);
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn logout_unknown_token_returns_204(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let random_uuid = uuid::Uuid::new_v4().to_string();

    let resp = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({"refreshToken": random_uuid}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

// =========================================================================
// === Protected route tests ===
// =========================================================================

/// Forge un JWT arbitraire pour les tests (permet de tester les cas
/// d'échec : expiré, mauvaise signature, sub non-parseable).
fn forge_jwt(
    sub: &str,
    role: &str,
    exp_offset_secs: i64,
    secret: &[u8],
) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        iat: now,
        exp: now + exp_offset_secs,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .expect("forge encode should succeed")
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_without_jwt_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_with_valid_jwt_returns_200(pool: MySqlPool) {
    let user_id = create_user(&pool, "frank", "password123", true).await;
    let app = spawn_app(pool).await;

    let login_resp: serde_json::Value = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "frank", "password": "password123"}))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let token = login_resp["accessToken"].as_str().unwrap();

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["userId"], user_id);
    assert_eq!(body["role"], "Comptable");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_with_expired_jwt_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    // exp dans le passé au-delà du leeway 60s
    let token = forge_jwt("42", "Comptable", -120, TEST_JWT_SECRET);

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_with_expired_jwt_within_leeway_returns_200(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    // exp dans le passé MAIS dans le leeway 60s
    let token = forge_jwt("42", "Comptable", -30, TEST_JWT_SECRET);

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "token within leeway should be accepted");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_with_wrong_signature_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let wrong_secret = b"wrong-secret-32-bytes-minimum-padding-long-enough";
    let token = forge_jwt("42", "Comptable", 900, wrong_secret);

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn protected_route_with_malformed_sub_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let token = forge_jwt("not-a-number", "Comptable", 900, TEST_JWT_SECRET);

    let resp = app
        .client
        .get(app.url("/api/v1/_test/me"))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// =========================================================================
// === Bootstrap E2E scenarios (AC9, AC10 — Task 10.3) ===
// =========================================================================
//
// Ces scénarios couvrent l'intégralité du flux bootstrap admin au niveau
// E2E : construction d'un AppState + appel à ensure_admin_user + login via
// HTTP avec les credentials bootstrap.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn bootstrap_creates_admin_and_login_works_end_to_end(pool: MySqlPool) {
    // Construire un Config de test avec un admin_password connu
    let config = Config::from_fields_for_test(
        "mysql://stub:stub@127.0.0.1:3306/stub".to_string(),
        "admin".to_string(),
        "bootstrap-e2e-password".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        5,
        TimeDelta::minutes(30),
    );

    // Bootstrap sur DB vide
    ensure_admin_user(&pool, &config)
        .await
        .expect("bootstrap should succeed on empty DB");

    // Vérifier en base que l'admin existe
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = 'admin' AND role = 'Admin' AND active = TRUE")
        .fetch_one(&pool)
        .await
        .expect("count should succeed");
    assert_eq!(count, 1, "admin should be created by bootstrap");

    // Lancer le serveur E2E avec l'AppState post-bootstrap et tenter un login HTTP
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "admin", "password": "bootstrap-e2e-password"}))
        .send()
        .await
        .expect("login request should succeed");

    assert_eq!(resp.status(), 200, "bootstrapped admin should be able to login");
    let body: serde_json::Value = resp.json().await.expect("json body");
    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());

    // Vérifier que le JWT contient role=Admin
    let token = body["accessToken"].as_str().unwrap();
    let mut validation = jsonwebtoken::Validation::new(Algorithm::HS256);
    validation.leeway = 60;
    validation.required_spec_claims = ["exp", "sub", "iat"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(TEST_JWT_SECRET),
        &validation,
    )
    .expect("JWT should decode");
    assert_eq!(decoded.claims.role, "Admin");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn bootstrap_is_idempotent_at_e2e_level(pool: MySqlPool) {
    let config = Config::from_fields_for_test(
        "mysql://stub:stub@127.0.0.1:3306/stub".to_string(),
        "admin".to_string(),
        "idempotent-pwd".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        5,
        TimeDelta::minutes(30),
    );

    // Trois appels consécutifs : le premier crée, les suivants sont no-op
    for i in 0..3 {
        ensure_admin_user(&pool, &config)
            .await
            .unwrap_or_else(|e| panic!("bootstrap call #{i} should succeed: {e:?}"));
    }

    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .expect("count should succeed");
    assert_eq!(
        count, 1,
        "three bootstrap calls should produce exactly one admin"
    );
}

// =========================================================================
// Story 1.6 E2E tests
// =========================================================================

/// Spawn app avec une config custom (pour tests rate limiting).
async fn spawn_app_with_config(pool: MySqlPool, config: Config) -> TestApp {
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
    };

    // build_router already includes login, logout, refresh (public) and
    // change_password (protected). We just add the test-only route.
    let protected_test = Router::new()
        .route("/api/v1/_test/me", get(test_me_handler))
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            kesh_api::middleware::auth::require_auth,
        ))
        .with_state(state.clone());

    let app = kesh_api::build_router(state, "nonexistent-static".to_string())
        .merge(protected_test);

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
        client: reqwest::Client::new(),
    }
}

/// Helper : login and return (access_token, refresh_token)
async fn login_and_get_tokens(app: &TestApp, username: &str, password: &str) -> (String, String) {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "login should succeed");
    let body: serde_json::Value = resp.json().await.unwrap();
    (
        body["accessToken"].as_str().unwrap().to_string(),
        body["refreshToken"].as_str().unwrap().to_string(),
    )
}

// --- 9.1 refresh_success_returns_new_tokens ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_success_returns_new_tokens(pool: MySqlPool) {
    create_user(&pool, "user1", "password123!", true).await;
    let app = spawn_app(pool).await;
    let (_at, rt) = login_and_get_tokens(&app, "user1", "password123!").await;

    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());
    assert!(body["expiresIn"].as_i64().unwrap() > 0);
    // New refresh token must be different from old
    assert_ne!(body["refreshToken"].as_str().unwrap(), rt);
}

// --- 9.2 refresh_rotates_token ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_rotates_token(pool: MySqlPool) {
    create_user(&pool, "user2", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt) = login_and_get_tokens(&app, "user2", "password123!").await;

    // Refresh : old token should be revoked with reason=rotation
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Verify old token is revoked with reason=rotation in DB
    let row: (Option<String>,) = sqlx::query_as(
        "SELECT revoked_reason FROM refresh_tokens WHERE token = ?",
    )
    .bind(&rt)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(row.0.as_deref(), Some("rotation"));
}

// --- 9.3 refresh_replay_after_rotation_revokes_all ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_replay_after_rotation_revokes_all(pool: MySqlPool) {
    let user_id = create_user(&pool, "user3", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt_old) = login_and_get_tokens(&app, "user3", "password123!").await;

    // Refresh : rotates old token
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt_old}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    let rt_new = body["refreshToken"].as_str().unwrap();

    // Replay old token → theft detection → mass revoke
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt_old}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // New token should also be revoked (mass revoke)
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt_new}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Verify all tokens are revoked in DB
    let active_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(active_count, 0, "all tokens should be revoked after theft detection");
}

// --- 9.4 refresh_after_logout_does_not_mass_revoke ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_after_logout_does_not_mass_revoke(pool: MySqlPool) {
    let user_id = create_user(&pool, "user4", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt1) = login_and_get_tokens(&app, "user4", "password123!").await;
    // Create a second session
    let (_at2, rt2) = login_and_get_tokens(&app, "user4", "password123!").await;

    // Logout session 1
    app.client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({"refreshToken": rt1}))
        .send()
        .await
        .unwrap();

    // Re-present logged-out token → should NOT mass revoke
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt1}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);

    // Session 2 should still work
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt2}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "second session should survive after logout of first");
}

// --- 9.5 refresh_with_expired_token_returns_401 ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_with_expired_token_returns_401(pool: MySqlPool) {
    let user_id = create_user(&pool, "user5", "password123!", true).await;
    // Insert an already-expired token directly in DB
    let expired_token = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)",
    )
    .bind(user_id)
    .bind(&expired_token)
    .bind(chrono::Utc::now().naive_utc() - chrono::TimeDelta::hours(1))
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": expired_token}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_REFRESH_TOKEN");
}

// --- 9.6 refresh_with_unknown_token_returns_401 ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_with_unknown_token_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": uuid::Uuid::new_v4().to_string()}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_REFRESH_TOKEN");
}

// --- 9.7 refresh_with_inactive_user_returns_401 ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_with_inactive_user_returns_401(pool: MySqlPool) {
    create_user(&pool, "user7", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt) = login_and_get_tokens(&app, "user7", "password123!").await;

    // Deactivate user directly in DB
    sqlx::query("UPDATE users SET active = false WHERE username = 'user7'")
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// --- 9.8 refresh_returns_updated_role ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn refresh_returns_updated_role(pool: MySqlPool) {
    create_user(&pool, "user8", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt) = login_and_get_tokens(&app, "user8", "password123!").await;

    // Change role directly in DB
    sqlx::query("UPDATE users SET role = 'Admin' WHERE username = 'user8'")
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    // Decode the new JWT and verify role is updated
    let new_at = body["accessToken"].as_str().unwrap();
    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.required_spec_claims = ["exp", "sub", "iat"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let decoded = jsonwebtoken::decode::<Claims>(
        new_at,
        &jsonwebtoken::DecodingKey::from_secret(TEST_JWT_SECRET),
        &validation,
    )
    .expect("JWT should decode");
    assert_eq!(decoded.claims.role, "Admin");
}

// --- 9.9 rate_limit_blocks_after_threshold ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rate_limit_blocks_after_threshold(pool: MySqlPool) {
    create_user(&pool, "user9", "password123!", true).await;
    let config = test_config_rate_limit(5, 1800);
    let app = spawn_app_with_config(pool, config).await;

    // 5 failed attempts
    for i in 0..5 {
        let resp = app
            .client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "user9", "password": "wrong"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401, "attempt {} should return 401", i + 1);
    }

    // 6th attempt → 429
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "user9", "password": "wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
    let has_retry_after = resp.headers().get("retry-after").is_some();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "RATE_LIMITED");
    assert!(has_retry_after, "429 response should include Retry-After header");
}

// --- 9.10 rate_limit_resets_after_block_duration ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rate_limit_resets_after_block_duration(pool: MySqlPool) {
    create_user(&pool, "user10", "password123!", true).await;
    let config = test_config_rate_limit(2, 1); // block for 1 second
    let app = spawn_app_with_config(pool, config).await;

    // 2 failed → blocked
    for _ in 0..2 {
        app.client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "user10", "password": "wrong"}))
            .send()
            .await
            .unwrap();
    }
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "user10", "password": "password123!"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429, "should be blocked");

    // Wait for block to expire
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;

    // Should be able to login now
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "user10", "password": "password123!"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "should be unblocked after block_duration");
}

// --- 9.11 rate_limit_resets_on_success ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rate_limit_resets_on_success(pool: MySqlPool) {
    create_user(&pool, "user11", "password123!", true).await;
    let config = test_config_rate_limit(5, 1800);
    let app = spawn_app_with_config(pool, config).await;

    // 3 failed attempts
    for _ in 0..3 {
        app.client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "user11", "password": "wrong"}))
            .send()
            .await
            .unwrap();
    }

    // 1 success → resets counter
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "user11", "password": "password123!"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // 5 more failed attempts before block (counter reset to 0)
    for i in 0..5 {
        let resp = app
            .client
            .post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "user11", "password": "wrong"}))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 401, "attempt {} after reset should be 401", i + 1);
    }

    // 6th → now blocked
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "user11", "password": "wrong"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 429);
}

// --- 9.12 change_password_revokes_all_tokens ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn change_password_revokes_all_tokens(pool: MySqlPool) {
    create_user(&pool, "user12", "old-password12", true).await;
    let app = spawn_app(pool.clone()).await;
    let (at, rt_old) = login_and_get_tokens(&app, "user12", "old-password12").await;

    // Change password
    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({"currentPassword": "old-password12", "newPassword": "new-password-12chars"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["accessToken"].is_string());
    assert!(body["refreshToken"].is_string());

    // Old refresh token should be revoked
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt_old}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "old refresh token should be revoked after password change");
}

// --- 9.13 change_password_wrong_current_returns_401 ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn change_password_wrong_current_returns_401(pool: MySqlPool) {
    create_user(&pool, "user13", "correct-pwd12", true).await;
    let app = spawn_app(pool).await;
    let (at, _rt) = login_and_get_tokens(&app, "user13", "correct-pwd12").await;

    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({"currentPassword": "wrong-password", "newPassword": "new-password-12chars"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_CREDENTIALS");
}

// --- 9.14 change_password_returns_new_tokens ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn change_password_returns_new_tokens(pool: MySqlPool) {
    create_user(&pool, "user14", "old-password14", true).await;
    let app = spawn_app(pool).await;
    let (at, _rt) = login_and_get_tokens(&app, "user14", "old-password14").await;

    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({"currentPassword": "old-password14", "newPassword": "new-password-14chars"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    // New tokens should work
    let new_rt = body["refreshToken"].as_str().unwrap();
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": new_rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "new refresh token from password change should work");
}

// --- 9.15 change_password_too_short_returns_400 ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn change_password_too_short_returns_400(pool: MySqlPool) {
    create_user(&pool, "user15", "password-15ch!", true).await;
    let app = spawn_app(pool).await;
    let (at, _rt) = login_and_get_tokens(&app, "user15", "password-15ch!").await;

    let resp = app
        .client
        .put(app.url("/api/v1/auth/password"))
        .header("Authorization", format!("Bearer {at}"))
        .json(&json!({"currentPassword": "password-15ch!", "newPassword": "short"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
}

// --- 9.16 cleanup_removes_old_tokens ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cleanup_removes_old_tokens(pool: MySqlPool) {
    let user_id = create_user(&pool, "user16", "password123!", true).await;

    // Insert old expired token (8 days ago)
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(user_id)
    .bind(uuid::Uuid::new_v4().to_string())
    .bind(chrono::Utc::now().naive_utc() - chrono::TimeDelta::days(8))
    .bind(chrono::Utc::now().naive_utc() - chrono::TimeDelta::days(9))
    .execute(&pool)
    .await
    .unwrap();

    // Insert old revoked token (8 days ago)
    let old_revoked = uuid::Uuid::new_v4().to_string();
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at, revoked_at, revoked_reason) VALUES (?, ?, ?, ?, 'logout')",
    )
    .bind(user_id)
    .bind(&old_revoked)
    .bind(chrono::Utc::now().naive_utc() + chrono::TimeDelta::days(1))
    .bind(chrono::Utc::now().naive_utc() - chrono::TimeDelta::days(8))
    .execute(&pool)
    .await
    .unwrap();

    let before_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(before_count, 2);

    // Cleanup : remove tokens older than 7 days
    let cutoff = (chrono::Utc::now() - chrono::TimeDelta::days(7)).naive_utc();
    let removed = kesh_db::repositories::refresh_tokens::delete_expired_and_revoked(&pool, cutoff)
        .await
        .unwrap();
    assert_eq!(removed, 2, "both old tokens should be removed");

    let after_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(after_count, 0);
}

// --- 9.17 login_timing_still_normalized (anti-regression) ---
// Explicit anti-regression test: verifies that rate limiter integration
// did not break the timing normalization from story 1.5.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn login_timing_still_normalized(pool: MySqlPool) {
    create_user(&pool, "timing_active", "correct-password", true).await;
    create_user(&pool, "timing_inactive", "correct-password", false).await;
    let app = spawn_app(pool).await;

    const N: usize = 10;
    let mut d_absent = Vec::with_capacity(N);
    let mut d_inactive = Vec::with_capacity(N);
    let mut d_bad_pwd = Vec::with_capacity(N);

    // Warm up
    let _ = app.client.post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": "warmup", "password": "warmup"}))
        .send().await;

    for _ in 0..N {
        let s = Instant::now();
        let _ = app.client.post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "ghost_timing", "password": "any"}))
            .send().await.unwrap();
        d_absent.push(s.elapsed().as_millis() as u64);

        let s = Instant::now();
        let _ = app.client.post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "timing_inactive", "password": "correct-password"}))
            .send().await.unwrap();
        d_inactive.push(s.elapsed().as_millis() as u64);

        let s = Instant::now();
        let _ = app.client.post(app.url("/api/v1/auth/login"))
            .json(&json!({"username": "timing_active", "password": "wrong"}))
            .send().await.unwrap();
        d_bad_pwd.push(s.elapsed().as_millis() as u64);
    }

    fn median(mut v: Vec<u64>) -> u64 {
        v.sort();
        let n = v.len();
        if n % 2 == 1 { v[n / 2] } else { (v[n / 2 - 1] + v[n / 2]) / 2 }
    }

    let m_a = median(d_absent);
    let m_i = median(d_inactive);
    let m_b = median(d_bad_pwd);
    eprintln!("Story 1.6 timing medians (ms): absent={m_a}, inactive={m_i}, bad_pwd={m_b}");

    let max = m_a.max(m_i).max(m_b);
    let min = m_a.min(m_i).min(m_b);
    assert!(min >= 10, "all medians should be > 10ms (Argon2 ~50ms), got min={min}ms");
    assert!((max as f64) / (min as f64) < 5.0,
        "timing ratio max/min should be < 5x, got max={max}ms min={min}ms");
}

// --- 9.18 all_refresh_error_codes_are_identical ---
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn all_refresh_error_codes_are_identical(pool: MySqlPool) {
    let user_id = create_user(&pool, "user18", "password123!", true).await;
    let app = spawn_app(pool.clone()).await;
    let (_at, rt) = login_and_get_tokens(&app, "user18", "password123!").await;

    // Case 1: unknown token
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": uuid::Uuid::new_v4().to_string()}))
        .send()
        .await
        .unwrap();
    let body1: serde_json::Value = resp.json().await.unwrap();

    // Case 2: expired token
    let expired = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
        .bind(user_id)
        .bind(&expired)
        .bind(chrono::Utc::now().naive_utc() - chrono::TimeDelta::hours(1))
        .execute(&pool)
        .await
        .unwrap();
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": expired}))
        .send()
        .await
        .unwrap();
    let body2: serde_json::Value = resp.json().await.unwrap();

    // Case 3: revoked token (rotate, then re-present)
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200); // first refresh succeeds
    let resp = app
        .client
        .post(app.url("/api/v1/auth/refresh"))
        .json(&json!({"refreshToken": rt})) // re-present rotated token
        .send()
        .await
        .unwrap();
    let body3: serde_json::Value = resp.json().await.unwrap();

    // All should return the same error code (anti-enumeration)
    assert_eq!(body1["error"]["code"], "INVALID_REFRESH_TOKEN");
    assert_eq!(body2["error"]["code"], "INVALID_REFRESH_TOKEN");
    assert_eq!(body3["error"]["code"], "INVALID_REFRESH_TOKEN");
}
