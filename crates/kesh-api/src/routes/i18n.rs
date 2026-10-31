//! Handler pour `/api/v1/i18n/messages` (story 2.1).
//!
//! Retourne toutes les traductions dans la langue de l'installation
//! (`KESH_LANG`, `state.config.locale`) : la langue de l'interface.

use axum::Json;
use axum::extract::State;
use serde::Serialize;
use std::collections::HashMap;

use crate::AppState;

/// Réponse de l'endpoint i18n.
#[derive(Debug, Serialize)]
pub struct I18nResponse {
    pub locale: String,
    pub messages: HashMap<String, String>,
}

/// `GET /api/v1/i18n/messages` — Retourne les traductions dans la langue de
/// l'installation (`KESH_LANG`).
pub async fn get_messages(State(state): State<AppState>) -> Json<I18nResponse> {
    let locale = &state.config.locale;
    let messages = state.i18n.all_messages(locale);

    Json(I18nResponse {
        locale: locale.to_string(),
        messages,
    })
}
