//! Routes profil utilisateur — préférences.

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;

use kesh_db::entities::audit_log::{AUDIT_ENTITY_ID_NONE, NewAuditLogEntry};
use kesh_db::entities::onboarding::UiMode;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::{audit_log, onboarding};

use crate::AppState;
use crate::audit::AuditActor;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

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
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<ModeRequest>,
) -> Result<StatusCode, AppError> {
    let ui_mode: UiMode = body
        .mode
        .parse()
        .map_err(|_| AppError::Validation(format!("Mode invalide : {}", body.mode)))?;

    let current = onboarding::get_state(&state.pool)
        .await?
        .ok_or_else(|| AppError::Internal("Aucun état d'onboarding".into()))?;

    // Story 25-1b (AC 5, 9) — la trace porte sur l'INSTALLATION, pas sur
    // l'utilisateur : `ui_mode` vit dans `onboarding_state`, table mono-ligne et
    // globale, sans `company_id`. Un utilisateur bascule donc l'installation
    // entière en mode Expert, ce qui ouvre à tous l'écriture directe au journal.
    // Écrire `entity_type = "user"` ici serait mentir sur la portée.
    //
    // ⚠️ L'audit se pose DANS LE HANDLER, jamais dans `update_step` : dix
    // appelants la partagent (le seed et huit routes d'onboarding), et la trace
    // y serait écrite à chaque étape de l'installation.
    //
    // ⚠️ Une trace part même quand rien ne change : `update_step` incrémente
    // `version` inconditionnellement, donc un PUT répété avec le même mode
    // écrira `before == after`. Assumé — l'égalité se lit dans `details_json`.
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    onboarding::update_step_in_tx(
        &mut tx,
        current.step_completed,
        current.is_demo,
        Some(ui_mode),
        current.version,
    )
    .await?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::from_current_user(
            &current_user,
            "installation.ui_mode_changed",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "before": current.ui_mode,
                "after": ui_mode,
            })),
        ),
    )
    .await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;

    Ok(StatusCode::NO_CONTENT)
}
