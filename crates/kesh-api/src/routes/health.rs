//! Healthcheck endpoint (`GET /health`).
//!
//! Story 1.2 : réponse 200 si la DB est joignable, 503 sinon. Reste
//! public (pas de JWT requis).
//!
//! Story 1.5 : refactor vers `State<AppState>`. Le pool est désormais
//! toujours présent au démarrage (l'application refuse de démarrer sans
//! DB), donc plus de gestion `Option<MySqlPool>`. Le comportement dégradé
//! 503 reste déclenché uniquement par l'échec du `SELECT 1`.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;

use crate::AppState;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query("SELECT 1").execute(&state.pool).await {
        Ok(_) => (
            StatusCode::OK,
            Json(json!({
                "status": "ok",
                "database": "connected"
            })),
        ),
        Err(e) => {
            tracing::warn!("Healthcheck DB échoué: {}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "database": "disconnected"
                })),
            )
        }
    }
}
