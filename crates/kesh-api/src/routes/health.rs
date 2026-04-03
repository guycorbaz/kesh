use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use sqlx::MySqlPool;

pub async fn health_check(
    State(pool): State<Option<MySqlPool>>,
) -> impl IntoResponse {
    match &pool {
        Some(pool) => match sqlx::query("SELECT 1").execute(pool).await {
            Ok(_) => (
                StatusCode::OK,
                Json(json!({
                    "status": "ok",
                    "database": "connected"
                })),
            ),
            Err(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "status": "degraded",
                    "database": "disconnected",
                    "error": e.to_string()
                })),
            ),
        },
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "status": "degraded",
                "database": "unavailable",
                "error": "Pool de connexion non initialisé"
            })),
        ),
    }
}
