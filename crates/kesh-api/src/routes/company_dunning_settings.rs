//! Routes du singleton `company_dunning_settings` — Epic 21, Story 21-3.
//!
//! GET (tout rôle authentifié) déclenche le seed lazy des 3 niveaux par défaut
//! (`ensure_seeded_in_tx`, sous sentinel lock). PUT (Administrateur) modifie la
//! période de grâce avec verrou optimiste.

use axum::extract::State;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_db::entities::{CompanyDunningSettings, CompanyDunningSettingsUpdate};
use kesh_db::repositories::company_dunning_settings;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningSettingsResponse {
    pub grace_period_days: i32,
    pub seeded_at: Option<chrono::NaiveDateTime>,
    pub version: i32,
}

impl From<CompanyDunningSettings> for DunningSettingsResponse {
    fn from(s: CompanyDunningSettings) -> Self {
        Self {
            grace_period_days: s.grace_period_days,
            seeded_at: s.seeded_at,
            version: s.version,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDunningSettingsRequest {
    pub grace_period_days: i32,
    pub version: i32,
}

/// `GET /api/v1/company/dunning-settings` — réglages + seed lazy des défauts.
pub async fn get_dunning_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<DunningSettingsResponse>, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    // Seed lazy (idempotent, sous sentinel lock) : la 1re visite pose les 3 niveaux
    // par défaut + la grâce, et renvoie l'état à jour.
    let settings =
        company_dunning_settings::ensure_seeded_in_tx(&mut tx, current_user.company_id).await?;
    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;
    Ok(Json(settings.into()))
}

/// `PUT /api/v1/company/dunning-settings` — modifie la grâce (Administrateur).
pub async fn update_dunning_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateDunningSettingsRequest>,
) -> Result<Json<DunningSettingsResponse>, AppError> {
    if req.grace_period_days < 0 {
        return Err(AppError::Validation(
            "La période de grâce ne peut être négative.".into(),
        ));
    }

    let updated = company_dunning_settings::update(
        &state.pool,
        current_user.company_id,
        req.version,
        current_user.user_id,
        CompanyDunningSettingsUpdate {
            grace_period_days: req.grace_period_days,
        },
    )
    .await?;

    Ok(Json(updated.into()))
}
