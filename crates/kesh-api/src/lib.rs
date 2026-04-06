//! kesh-api — Serveur HTTP Axum pour Kesh.
//!
//! Cette crate expose à la fois un binaire (`main.rs`) et une
//! bibliothèque (`lib.rs`) pour permettre aux tests d'intégration
//! d'importer `build_router` et les helpers de configuration.

pub mod auth;
pub mod config;
pub mod errors;
pub mod middleware;
pub mod routes;

use std::sync::Arc;

use axum::routing::{get, post, put};
use axum::Router;
use sqlx::MySqlPool;
use tower_http::services::{ServeDir, ServeFile};

use crate::config::Config;
use crate::middleware::rate_limit::RateLimiter;

/// État partagé injecté dans tous les handlers via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub pool: MySqlPool,
    pub config: Arc<Config>,
    pub rate_limiter: Arc<RateLimiter>,
}

/// Construit le routeur principal de l'application (routes publiques
/// uniquement dans cette story).
///
/// - `/health` (public) — healthcheck DB
/// - `/api/v1/auth/login` (public) — authentification
/// - `/api/v1/auth/logout` (public) — invalidation refresh_token
///
/// Les stories futures ajouteront des routes protégées par JWT en
/// construisant un sous-routeur qui applique le middleware
/// `crate::middleware::auth::require_auth` via `route_layer`, puis
/// en le mergeant via `Router::merge()` dans `main.rs`.
///
/// **Note Axum 0.8** : `route_layer` sur un router vide panique
/// (`Adding a route_layer before any routes is a no-op`). On ne
/// construit le sous-routeur protégé qu'au moment où des routes lui
/// sont effectivement ajoutées.
///
/// Le `static_dir` contient le SPA SvelteKit buildé (`frontend/build`),
/// servi en fallback par `ServeDir`/`ServeFile`.
pub fn build_router(state: AppState, static_dir: String) -> Router {
    let fallback = ServeDir::new(&static_dir)
        .fallback(ServeFile::new(format!("{}/index.html", static_dir)));

    // Story 1.8 : sous-routeurs par niveau de rôle (RBAC)
    //
    // Ordre de construction : require_role (inner) → merge → require_auth (outer)
    // Ordre d'exécution (oignon) : require_auth EN PREMIER → require_role EN SECOND → handler

    // Admin-only routes : gestion des utilisateurs
    let admin_routes = Router::new()
        .route(
            "/api/v1/users",
            get(routes::users::list_users).post(routes::users::create_user),
        )
        .route(
            "/api/v1/users/{id}",
            get(routes::users::get_user).put(routes::users::update_user),
        )
        .route(
            "/api/v1/users/{id}/disable",
            put(routes::users::disable_user),
        )
        .route(
            "/api/v1/users/{id}/reset-password",
            put(routes::users::reset_password),
        )
        .route_layer(axum::middleware::from_fn(
            crate::middleware::rbac::require_admin_role,
        ));

    // Routes authentifiées (tout rôle) : changement de mot de passe
    let authenticated_routes = Router::new()
        .route("/api/v1/auth/password", put(routes::auth::change_password));

    // Merge + auth JWT (couche de base pour toutes les routes protégées)
    let protected = Router::new()
        .merge(admin_routes)
        .merge(authenticated_routes)
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            crate::middleware::auth::require_auth,
        ));

    Router::new()
        .route("/health", get(routes::health::health_check))
        .route("/api/v1/auth/login", post(routes::auth::login))
        .route("/api/v1/auth/logout", post(routes::auth::logout))
        .route("/api/v1/auth/refresh", post(routes::auth::refresh))
        .merge(protected)
        .fallback_service(fallback)
        .with_state(state)
}

// NOTE: les stories futures ajouteront leurs routes protégées en
// construisant un sous-routeur ainsi :
//
// ```ignore
// let protected = Router::new()
//     .route("/api/v1/accounts", get(...))
//     .route("/api/v1/journal-entries", post(...))
//     .route_layer(axum::middleware::from_fn_with_state(
//         state.clone(),
//         crate::middleware::auth::require_auth,
//     ));
// let app = build_router(state, static_dir).merge(protected);
// ```
//
// **Important Axum 0.8** : le `route_layer` doit venir APRÈS les
// `route(...)`, sinon Axum panique avec « Adding a route_layer before
// any routes is a no-op ».
