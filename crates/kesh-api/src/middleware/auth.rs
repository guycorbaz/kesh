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
use std::sync::atomic::Ordering;

use crate::AppState;
use crate::auth::jwt;
use crate::errors::AppError;

/// Identité extraite du JWT valide, injectée dans la requête.
///
/// Story 6.2: `company_id` ajouté pour multi-tenant scoping.
/// Story 10-5 (D2 acté Pass 1 F-L-P1-14): `exp` ajouté pour permettre au
/// handler `/api/v1/auth/me` de calculer `expires_in` (secondes restantes
/// avant expiration JWT) sans re-décoder le token.
/// Story 17-2a (DC2) : `api_key_id` ajouté pour distinguer le chemin
/// d'authentification. `None` = session JWT (UI web). `Some(id)` = PAT
/// (`Authorization: Bearer kesh_pat_…`). Ce champ permet à l'audit des routes
/// métier de marquer `actor_type='api_key'` (cf. `crate::audit::AuditActor`),
/// et au gate de gestion des clés (DC6) de rejeter les requêtes PAT.
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: i64,
    pub role: Role,
    pub company_id: i64,
    pub exp: i64,
    /// Story 17-2a (DC2) — `Some(<id clé>)` si authentifié par PAT, `None` si JWT.
    pub api_key_id: Option<i64>,
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
    jar: axum_extra::extract::CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Story 10-5 (T3.1) : lecture du JWT en priorité depuis le cookie
    // `kesh_access_token` (HttpOnly, set par /login + /refresh), fallback sur
    // le header `Authorization: Bearer` (préserve la compat avec les 19+
    // tests `*_e2e.rs` historiques et les éventuels clients API non-browser).
    let token_opt: Option<String> = if let Some(cookie) = jar.get("kesh_access_token") {
        Some(cookie.value().to_string())
    } else {
        req.headers().get(AUTHORIZATION).and_then(|h| {
            h.to_str().ok().and_then(|header| {
                // RFC 7235 §2.1 : le scheme HTTP auth est case-insensitive.
                const BEARER_PREFIX_LEN: usize = 7; // "Bearer "
                if header.len() < BEARER_PREFIX_LEN
                    || !header.as_bytes()[..6].eq_ignore_ascii_case(b"bearer")
                    || header.as_bytes()[6] != b' '
                {
                    None
                } else {
                    Some(header[BEARER_PREFIX_LEN..].trim().to_string())
                }
            })
        })
    };

    // Story v011-5 (AC #13) — gate 423 Locked APRÈS extraction du token,
    // AVANT JWT decode. Si aucun token + users vide → 423 (setup-required,
    // plus précis que 401). Si token + users vide → 423 (theoretical edge :
    // ancien JWT en cookie mais DB truncate, force redirect /setup).
    // Si aucun token + users existent → 401 nominal (handled ci-dessous).
    // Lecture lock-free `Acquire` cohérente avec le `Release` au store
    // (setup::create_admin + main.rs init).
    if !state.users_exist.load(Ordering::Acquire) {
        return Err(AppError::SetupRequired);
    }

    let token = match token_opt {
        Some(t) => t,
        None => {
            // Pas de cookie, pas de header Authorization valide → distinguer
            // le « missing » du « malformed » pour le diagnostic. Vu que
            // l'absence de header retourne None ci-dessus mais aussi un
            // header présent mais malformé, on rapporte un message générique.
            // (Préserve la compat des tests existants sur 401 + message.)
            return Err(AppError::Unauthenticated(
                "missing or malformed authorization (no cookie, no Bearer header)".into(),
            ));
        }
    };

    // Story 17-2a (AC4/AC6, DC3) — discrimination JWT vs PAT APRÈS extraction
    // du token ET le gate `users_exist`. Le préfixe `kesh_pat_` est testé
    // case-sensitive exact (octets) : `KESH_PAT_` / `kesh_pat ` (espace)
    // tombent en `jwt::decode` (échec 401). Un cookie ne contient jamais
    // `kesh_pat_` (en pratique seul le chemin bearer déclenche le PAT), et un
    // vrai JWS base64url ne commence jamais par `kesh_pat_`. Ne JAMAIS passer
    // un PAT à `jwt::decode` (fausse erreur loggée + fuite timing).
    let current_user = if token.starts_with(crate::auth::api_key::PAT_PREFIX) {
        let (current_user, scope) = crate::auth::api_key::validate_pat(&token, &state.pool).await?;

        // Gate de scope (DC3/AC6) — UNIQUEMENT sur le chemin PAT. `scope=read`
        // → seules GET/HEAD/OPTIONS autorisées ; toute méthode mutante →
        // 403 API_KEY_READ_ONLY (rejet global en amont, jamais FailedProposal,
        // F-OPUS-7). Le chemin JWT (UI web) n'est jamais soumis à ce gate.
        if scope == kesh_db::entities::ApiKeyScope::Read && !is_safe_method(req.method()) {
            return Err(AppError::ApiKeyReadOnly);
        }
        current_user
    } else {
        let claims = jwt::decode(&token, state.config.jwt_secret_bytes())?;

        let user_id: i64 = claims
            .sub
            .parse()
            .map_err(|_| AppError::Unauthenticated("invalid sub claim".into()))?;

        let role: Role = Role::from_str(&claims.role)
            .map_err(|_| AppError::Unauthenticated("invalid role claim".into()))?;

        CurrentUser {
            user_id,
            role,
            company_id: claims.company_id,
            exp: claims.exp,
            api_key_id: None,
        }
    };

    req.extensions_mut().insert(current_user);
    Ok(next.run(req).await)
}

/// Méthode HTTP « sûre » (lecture seule) au sens du gate de scope PAT (DC3).
/// `GET`/`HEAD`/`OPTIONS` sont autorisées pour une clé `read` ; toute autre
/// méthode (POST/PUT/PATCH/DELETE) est une mutation refusée.
fn is_safe_method(method: &axum::http::Method) -> bool {
    matches!(
        *method,
        axum::http::Method::GET | axum::http::Method::HEAD | axum::http::Method::OPTIONS
    )
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

    /// Gate de scope PAT (DC3) : seules GET/HEAD/OPTIONS sont « safe » ; toute
    /// autre méthode est une mutation refusée pour une clé `scope='read'`.
    /// Couvre l'unité demandée par AC13 (le comportement bout-en-bout est aussi
    /// testé dans `api_keys_e2e.rs`).
    #[test]
    fn is_safe_method_allows_only_read_methods() {
        use axum::http::Method;
        // Méthodes sûres (lecture) → autorisées même en scope read.
        assert!(is_safe_method(&Method::GET));
        assert!(is_safe_method(&Method::HEAD));
        assert!(is_safe_method(&Method::OPTIONS));
        // Méthodes mutantes → refusées en scope read (403 API_KEY_READ_ONLY).
        assert!(!is_safe_method(&Method::POST));
        assert!(!is_safe_method(&Method::PUT));
        assert!(!is_safe_method(&Method::PATCH));
        assert!(!is_safe_method(&Method::DELETE));
        // Méthode hors-liste (deny par défaut, conservateur).
        assert!(!is_safe_method(&Method::TRACE));
        assert!(!is_safe_method(&Method::CONNECT));
    }

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
        // Story v011-5 — test_state défaute `users_exist=true` (régime
        // nominal middleware tests qui veulent tester 401/200, pas 423).
        // Les tests qui veulent valider le gate 423 utilisent un store
        // dédié (cf. tests `users_exist_false_returns_423`).
        AppState {
            pool: stub_pool(),
            config: Arc::new(config),
            rate_limiter: std::sync::Arc::new(rate_limiter),
            // Story 17-4c — littéral-exception (test_state) : limiter recovery.
            rate_limiter_recovery: std::sync::Arc::new(crate::build_recovery_rate_limiter()),
            i18n: std::sync::Arc::new(i18n),
            users_exist: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true)),
            // Story 17-4b — littéral-exception (test_state) : mailer no-op.
            mailer: std::sync::Arc::new(crate::mail::NoopMailer),
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

    /// Story v011-5 AC #13 — `users_exist=false` + pas de token → 423 Locked.
    #[tokio::test]
    async fn users_exist_false_returns_423_no_token() {
        let mut state = test_state();
        state.users_exist = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let app = protected_router(state);
        let req = Request::builder()
            .uri("/protected")
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            response_status(app, req).await,
            StatusCode::LOCKED,
            "users_exist=false should trigger 423 SETUP_REQUIRED"
        );
    }

    /// Story v011-5 AC #13 — `users_exist=false` + JWT cookie présent → 423 quand même.
    /// Edge theoretical : ancien JWT en cookie mais DB truncate. Force redirect /setup.
    #[tokio::test]
    async fn users_exist_false_returns_423_even_with_valid_jwt() {
        let mut state = test_state();
        state.users_exist = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

        let token = jwt::encode(42, Role::Admin, 5, TEST_JWT_SECRET, TimeDelta::minutes(15))
            .expect("encode");

        let app = protected_router(state);
        let req = Request::builder()
            .uri("/protected")
            .header("Cookie", format!("kesh_access_token={}", token))
            .body(Body::empty())
            .unwrap();
        assert_eq!(
            response_status(app, req).await,
            StatusCode::LOCKED,
            "users_exist=false must short-circuit even valid JWT"
        );
    }

    /// Story 10-5 (T3.2 — CR Pass 1 AA-C1) : middleware lit le cookie
    /// `kesh_access_token` en priorité, sans header Authorization présent.
    #[tokio::test]
    async fn require_auth_accepts_cookie_no_authorization() {
        let token = jwt::encode(
            42,
            Role::Comptable,
            7,
            TEST_JWT_SECRET,
            TimeDelta::minutes(15),
        )
        .expect("encode");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Cookie", format!("kesh_access_token={}", token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(
            response.status(),
            StatusCode::OK,
            "cookie-only auth should succeed (Story 10-5 T3.1)"
        );

        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(body, "42:Comptable:7");
    }

    /// Story 10-5 (T3.2 — CR Pass 1 AA-C1) : si cookie ET header sont
    /// présents, le cookie est utilisé en priorité (T3.1 strict cookie-first).
    #[tokio::test]
    async fn require_auth_prefers_cookie_over_header_when_both_present() {
        // Cookie token : user 42 / Admin
        let cookie_token = jwt::encode(42, Role::Admin, 5, TEST_JWT_SECRET, TimeDelta::minutes(15))
            .expect("encode cookie token");

        // Header token : user 99 / Comptable (rôle différent pour discriminer)
        let header_token = jwt::encode(
            99,
            Role::Comptable,
            5,
            TEST_JWT_SECRET,
            TimeDelta::minutes(15),
        )
        .expect("encode header token");

        let app = protected_router(test_state());
        let req = Request::builder()
            .uri("/protected")
            .header("Cookie", format!("kesh_access_token={}", cookie_token))
            .header("Authorization", format!("Bearer {}", header_token))
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(req).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        // Le handler doit retourner les claims du COOKIE (42:Admin), pas du header (99:Comptable).
        use http_body_util::BodyExt;
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = std::str::from_utf8(&bytes).unwrap();
        assert_eq!(
            body, "42:Admin:5",
            "cookie should take precedence over Authorization header"
        );
    }
}
