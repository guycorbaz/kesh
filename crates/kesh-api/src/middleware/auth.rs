//! Middleware d'authentification JWT.
//!
//! Pattern Axum 0.8 : middleware fonctionnel (`from_fn_with_state`)
//! qui extrait le JWT de l'en-tête `Authorization`, le décode, et
//! injecte un `CurrentUser` dans les `Extensions` de la requête.
//! Les handlers protégés récupèrent l'identité via `Extension<CurrentUser>`.
//!
//! **Pourquoi pas `from_extractor::<CurrentUser>()`** : en Axum 0.8,
//! `from_extractor` appelle l'extractor avec `State = ()`, ce qui empêche
//! l'accès à `jwt_secret` stocké dans `AppState`. Le pattern
//! `from_fn_with_state` est la solution idiomatique pour les guards
//! stateful.

use axum::extract::{Request, State};
use axum::http::header::AUTHORIZATION;
use axum::middleware::Next;
use axum::response::Response;
use kesh_db::entities::Role;
use std::str::FromStr;

use crate::AppState;
use crate::auth::jwt;
use crate::errors::AppError;

/// Identité extraite du JWT valide, injectée dans la requête.
///
/// Story 6.2: `company_id` ajouté pour multi-tenant scoping.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: i64,
    pub role: Role,
    pub company_id: i64,
}

/// Middleware qui exige un JWT valide.
///
/// Appliqué via `Router::route_layer(from_fn_with_state(state, require_auth))`
/// sur le sous-routeur `protected` dans `lib.rs::build_router`.
///
/// En cas d'échec (header manquant/malformé, JWT invalide/expiré), retourne
/// un `AppError::Unauthenticated` qui mappe vers 401. En cas de succès,
/// insère `CurrentUser` dans les extensions de la requête.
// SEC: active check at login only — on ne refait pas une requête DB à
// chaque requête protégée pour vérifier users.active. Un user désactivé
// sera déconnecté au prochain refresh (story 1.6).
//
// SEC: role staleness — la fenêtre est la même pour le rôle. Si un admin
// demote un user de Admin → Consultation, le JWT existant continue de
// porter role: Admin jusqu'à l'expiration (15 min + 60 s de leeway).
// Pour une appli comptable avec exigences d'audit, les opérations à fort
// privilège (changement de plan comptable, clôture d'exercice) peuvent
// ré-vérifier la DB avec `refresh_from_db(user_id)` si nécessaire, mais
// ce n'est pas automatique. Documenté dans la spec story 1.5 Dev Notes.
//
// SEC: company_id staleness (Story 6.2) — idem role. Si un user est déplacé
// vers une autre company au cours de sa session, le JWT existant continue de
// porter l'ancien company_id jusqu'à l'expiration. La fenêtre de staleness
// est proportionnelle au TTL JWT configurable via `KESH_JWT_EXPIRY_MINUTES`
// (défaut 15 min, max 24h dans config.rs). Si TTL=480 min (8h), la staleness
// company_id est 8h. Risque accepté pour l'architecture multi-tenant mono-user.
pub async fn require_auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    let header = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthenticated("missing authorization header".into()))?;

    // RFC 7235 §2.1 : le scheme HTTP auth est case-insensitive.
    // On accepte `Bearer`, `bearer`, `BEARER`, etc. via un test sur les
    // 7 premiers caractères (6 pour le scheme + 1 espace obligatoire).
    const BEARER_PREFIX_LEN: usize = 7; // "Bearer "
    if header.len() < BEARER_PREFIX_LEN
        || !header.as_bytes()[..6].eq_ignore_ascii_case(b"bearer")
        || header.as_bytes()[6] != b' '
    {
        return Err(AppError::Unauthenticated(
            "malformed authorization header".into(),
        ));
    }
    let token = header[BEARER_PREFIX_LEN..].trim();

    let claims = jwt::decode(token, state.config.jwt_secret_bytes())?;

    let user_id: i64 = claims
        .sub
        .parse()
        .map_err(|_| AppError::Unauthenticated("invalid sub claim".into()))?;

    let role: Role = Role::from_str(&claims.role)
        .map_err(|_| AppError::Unauthenticated("invalid role claim".into()))?;

    let company_id = claims.company_id;

    req.extensions_mut().insert(CurrentUser {
        user_id,
        role,
        company_id,
    });
    Ok(next.run(req).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::jwt;
    use crate::config::Config;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::routing::get;
    use axum::{Extension, Router};
    use chrono::TimeDelta;
    use sqlx::MySqlPool;
    use sqlx::mysql::MySqlPoolOptions;
    use std::sync::Arc;
    use tower::ServiceExt;

    const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

    /// Handler factice protégé qui renvoie 200 + l'id extrait (Story 6.2: include company_id).
    async fn echo_handler(Extension(user): Extension<CurrentUser>) -> String {
        format!(
            "{}:{}:{}",
            user.user_id,
            user.role.as_str(),
            user.company_id
        )
    }

    /// Construit un pool « bidon » qui n'est jamais vraiment utilisé par
    /// les tests middleware (le middleware ne touche pas la DB). On utilise
    /// `connect_lazy` qui ne tente aucune connexion tant qu'aucune requête
    /// SQL n'est émise.
    fn stub_pool() -> MySqlPool {
        MySqlPoolOptions::new()
            .max_connections(1)
            .connect_lazy("mysql://stub:stub@127.0.0.1:3306/stub")
            .expect("lazy pool should build")
    }

    /// Construit un `AppState` de test (le pool n'est jamais touché par
    /// le middleware, seul `config.jwt_secret_bytes()` est lu).
    fn test_state() -> AppState {
        let config = Config::from_fields_for_test(
            "mysql://stub:stub@127.0.0.1:3306/stub".to_string(),
            "admin".to_string(),
            "stub-admin-password".to_string(),
            String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
            TimeDelta::minutes(15),
            TimeDelta::days(30),
            TimeDelta::minutes(15),
            TimeDelta::minutes(15),
            5,
            TimeDelta::minutes(30),
            12,
        );
        let rate_limiter = crate::middleware::rate_limit::RateLimiter::new(&config);
        let i18n = kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n");
        AppState {
            pool: stub_pool(),
            config: Arc::new(config),
            rate_limiter: std::sync::Arc::new(rate_limiter),
            i18n: std::sync::Arc::new(i18n),
        }
    }

    fn protected_router(state: AppState) -> Router {
        Router::new()
            .route("/protected", get(echo_handler))
            .route_layer(axum::middleware::from_fn_with_state(
                state.clone(),
                require_auth,
            ))
            .with_state(state)
    }

    async fn response_status(app: Router, req: Request<Body>) -> StatusCode {
        app.oneshot(req).await.unwrap().status()
    }

    #[tokio::test]
    async fn missing_authorization_header_returns_401() {
        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn malformed_authorization_header_returns_401() {
        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "NotBearer whatever")
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn garbage_jwt_returns_401() {
        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer not-a-real-jwt")
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn expired_jwt_beyond_leeway_returns_401() {
        let token = jwt::encode(
            42,
            Role::Comptable,
            5,
            TEST_JWT_SECRET,
            TimeDelta::seconds(-120), // expired 120s ago, beyond leeway=60
        )
        .expect("encode");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn expired_jwt_within_leeway_returns_200() {
        let token = jwt::encode(
            42,
            Role::Comptable,
            5,
            TEST_JWT_SECRET,
            TimeDelta::seconds(-30), // expired 30s ago, within leeway=60
        )
        .expect("encode");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            response_status(app, req).await,
            StatusCode::OK,
            "token within leeway should be accepted"
        );
    }

    #[tokio::test]
    async fn valid_jwt_returns_200_and_injects_current_user() {
        let token = jwt::encode(
            1234,
            Role::Admin,
            5,
            TEST_JWT_SECRET,
            TimeDelta::minutes(15),
        )
        .expect("encode");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Vérifier que le handler a reçu CurrentUser avec les bonnes valeurs
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(body, "1234:Admin:5");
    }

    /// Patch V6 : header `Authorization: Bearer ` exactement 7 chars —
    /// le scheme est valide (case-insensitive + single space), mais
    /// le token après trim est vide. Doit retourner 401 via jwt::decode
    /// qui rejette une chaîne vide.
    #[tokio::test]
    async fn bearer_scheme_with_empty_token_returns_401() {
        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", "Bearer ")
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn wrong_signature_returns_401() {
        let other_secret = b"other-secret-32-bytes-minimum-padding-long-enough";
        let token =
            jwt::encode(1, Role::Admin, 5, other_secret, TimeDelta::minutes(15)).expect("encode");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();
        assert_eq!(response_status(app, req).await, StatusCode::UNAUTHORIZED);
    }
}
