//! Healthcheck endpoint (`GET /health`).
//!
//! Story 1.2 : réponse 200 si la DB est joignable, 503 sinon. Reste
//! public (pas de JWT requis).
//!
//! Story 1.5 : refactor vers `State<AppState>`. Le pool est désormais
//! toujours présent au démarrage (l'application refuse de démarrer sans
//! DB), donc plus de gestion `Option<MySqlPool>`. Le comportement dégradé
//! 503 reste déclenché uniquement par l'échec du `SELECT 1`.
//!
//! Story 10.3 : shape body alignée sur `{ status, db, version }` consommée
//! par le frontend `apiHealth.pollHealth()` (banner dégradé + auto-recovery)
//! et par le smoke test post-build `release.yml` (epic-10.md). `version` est
//! résolu via `env!("CARGO_PKG_VERSION")`, donc figé à la compilation.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    let version = env!("CARGO_PKG_VERSION");
    // Story 17-4c (DC9) — feature-flag recovery exposé dans les DEUX branches
    // (200/503), indépendant de l'état DB : le frontend (17-4d) conditionne
    // l'affichage du lien « mot de passe oublié ? » sur ce flag.
    let forgot_password_enabled = state.config.forgot_password_enabled;
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "db": true,
                "version": version,
                "forgotPasswordEnabled": forgot_password_enabled,
            })),
        ),
        Err(e) => {
            tracing::warn!("Healthcheck DB échoué: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "db": false,
                    "version": version,
                    "forgotPasswordEnabled": forgot_password_enabled,
                })),
            )
        }
    }
}
