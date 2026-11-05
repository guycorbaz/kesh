//! Story v0172-session (0.17.3, #876, CR #891) — le réglage de la durée
//! d'inactivité de la session, dans Administration → Paramètres (section
//! Session). Administrateur seul ; refusé aux clés d'API par la couche
//! `require_not_pat` du bloc `admin_routes`.
//!
//! - `GET /api/v1/admin/session-settings` ⇒ `{ sessionIdleMinutes, version,
//!   min, max }` (défaut 60, version 0, tant que rien n'est enregistré) ;
//! - `PUT /api/v1/admin/session-settings` `{ sessionIdleMinutes, version }` —
//!   corps lu par un extracteur qui rend **400 `VALIDATION_ERROR`** sur tout
//!   rejet (jamais le 422 d'Axum) ; hors bornes ⇒ 400, rien d'écrit ; version
//!   périmée ⇒ 409 `OPTIMISTIC_LOCK_CONFLICT` ; sinon 200 et le nouvel état.
//!   Rejouée sur interblocage (1213) : deux premiers enregistrements
//!   simultanés rendent 200 et 409, jamais 500.
//!   L'audit `admin.session_settings_updated` est écrit par le dépôt dans la
//!   même transaction ; une valeur inchangée n'écrit rien.
//!
//! Une session applique la nouvelle valeur à sa prochaine activité (le
//! battement relit N) ; une session déjà inactive garde son échéance.

use axum::Extension;
use axum::Json;
use axum::extract::State;
use serde::{Deserialize, Serialize};

use kesh_db::repositories::installation_settings::{
    self, InstallationSettings, SESSION_IDLE_MAX, SESSION_IDLE_MIN,
};

use crate::AppState;
use crate::errors::{AppError, t};
use crate::extractors::impl_validation_extractor;
use crate::middleware::auth::CurrentUser;

/// Réglage de la durée d'inactivité, avec ses bornes.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettingsResponse {
    pub session_idle_minutes: i32,
    pub version: i32,
    pub min: i32,
    pub max: i32,
}

impl From<InstallationSettings> for SessionSettingsResponse {
    fn from(s: InstallationSettings) -> Self {
        Self {
            session_idle_minutes: s.session_idle_minutes,
            version: s.version,
            min: SESSION_IDLE_MIN,
            max: SESSION_IDLE_MAX,
        }
    }
}

/// `GET /api/v1/admin/session-settings`.
pub async fn get_session_settings(
    State(state): State<AppState>,
) -> Result<Json<SessionSettingsResponse>, AppError> {
    Ok(Json(installation_settings::get(&state.pool).await?.into()))
}

/// Corps de `PUT /api/v1/admin/session-settings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSessionSettingsRequest {
    pub session_idle_minutes: i32,
    pub version: i32,
}

impl_validation_extractor!(
    UpdateSessionSettingsBody,
    UpdateSessionSettingsRequest,
    "sessionIdleMinutes : entier, version : entier"
);

/// `PUT /api/v1/admin/session-settings`.
pub async fn update_session_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    UpdateSessionSettingsBody(req): UpdateSessionSettingsBody,
) -> Result<Json<SessionSettingsResponse>, AppError> {
    if !installation_settings::is_valid_session_idle_minutes(req.session_idle_minutes) {
        // Le repli égale la valeur fr-CH du catalogue (G9, `textes_coherents.rs`).
        return Err(AppError::Validation(t(
            "admin-session-settings-error-out-of-domain",
            "Durée d'inactivité hors bornes : de 1 minute à 24 heures.",
        )));
    }
    // Revue de code P1 (S-B1/S-E1, choix C-v0172s-59) : deux `PUT` simultanés
    // à version 0 sur table vide prennent chacun le verrou d'intervalle du
    // `SELECT … FOR UPDATE`, puis leurs `INSERT` s'attendent l'un l'autre :
    // InnoDB annule la victime en 1213. Rejouée, elle relit la ligne validée
    // par l'autre et rend le conflit optimiste (409), non un 500.
    let s = kesh_db::retry::retry_on_deadlock(
        "installation_settings::update_session_idle_minutes",
        || {
            installation_settings::update_session_idle_minutes(
                &state.pool,
                req.version,
                current_user.user_id,
                req.session_idle_minutes,
            )
        },
    )
    .await?;
    Ok(Json(s.into()))
}
