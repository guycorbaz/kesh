//! Routes profil utilisateur — préférences.

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::Deserialize;

use kesh_db::entities::onboarding::UiMode;
use kesh_db::repositories::onboarding;

use crate::errors::AppError;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ModeRequest {
    pub mode: String,
}

/// PUT /api/v1/profile/mode — met à jour le mode Guidé/Expert.
///
/// Persiste dans `onboarding_state.ui_mode` via `update_step()`,
/// en gardant step_completed et is_demo inchangés.
pub async fn set_mode(
    State(state): State<AppState>,
    Json(body): Json<ModeRequest>,
) -> Result<StatusCode, AppError> {
    let ui_mode: UiMode = body
        .mode
        .parse()
        .map_err(|_| AppError::Validation(format!("Mode invalide : {}", body.mode)))?;

    let current = onboarding::get_state(&state.pool)
        .await?
        .ok_or_else(|| AppError::Internal("Aucun état d'onboarding".into()))?;

    onboarding::update_step(
        &state.pool,
        current.step_completed,
        current.is_demo,
        Some(ui_mode),
        current.version,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}
