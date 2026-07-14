//! Routes `GET`/`PUT`/`DELETE /api/v1/admin/email-templates` (Epic 20 #224,
//! Story 20-1) — socle CRUD Admin-only du sous-système de templates d'e-mail.
//!
//! Tout Admin-only (pas de lecture Comptable+ en v1) : le seul consommateur
//! est la page Admin « Modèles d'e-mail » (Story 20-2), elle-même
//! Admin-only. Le rendu réel à l'envoi (Story 20-3b) se fait côté serveur
//! via le repository directement, pas via cette route REST.
//!
//! `{template_type}`/`{language}` sont extraits en `Path<(String, String)>`
//! puis parsés manuellement (`FromStr`), pattern `parse_journal`
//! (`company_invoice_settings.rs`) — pas d'extraction `Path<(Enum, Enum)>`
//! (chemin non éprouvé dans ce codebase).

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_db::entities::{EffectiveEmailTemplate, EmailTemplate, EmailTemplateType, Language};
use kesh_db::repositories::{dunning_levels, email_templates};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailTemplateResponse {
    pub template_type: EmailTemplateType,
    pub language: Language,
    /// Niveau de rappel (0 = générique / `invoice_send`). Epic 21.
    pub level_number: i16,
    pub subject: String,
    pub body: String,
    /// `None` quand `is_default = true` (rien à verrouiller).
    pub version: Option<i32>,
    pub is_default: bool,
    pub allowed_variables: Vec<String>,
}

impl From<EffectiveEmailTemplate> for EmailTemplateResponse {
    fn from(t: EffectiveEmailTemplate) -> Self {
        Self {
            template_type: t.template_type,
            language: t.language,
            level_number: t.level_number,
            subject: t.subject,
            body: t.body,
            version: t.version,
            is_default: t.is_default,
            allowed_variables: t.allowed_variables,
        }
    }
}

impl From<EmailTemplate> for EmailTemplateResponse {
    fn from(t: EmailTemplate) -> Self {
        Self {
            allowed_variables: t.template_type.allowed_variables_owned(),
            template_type: t.template_type,
            language: t.language,
            level_number: t.level_number,
            subject: t.subject,
            body: t.body,
            version: Some(t.version),
            is_default: false,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailTemplateRequest {
    pub subject: String,
    pub body: String,
    /// `None` = le client croit qu'aucun override n'existe encore (création).
    pub expected_version: Option<i32>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_template_type(raw: &str) -> Result<EmailTemplateType, AppError> {
    raw.parse::<EmailTemplateType>()
        .map_err(AppError::Validation)
}

fn parse_language(raw: &str) -> Result<Language, AppError> {
    raw.parse::<Language>().map_err(AppError::Validation)
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/admin/email-templates` — les 4 combinaisons type×langue
/// résolues (override ou défaut). Jamais de tableau vide (AC #16).
pub async fn list_email_templates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<EmailTemplateResponse>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    // Borne dynamique des niveaux de rappel exposés = MAX(niveaux configurés, 3).
    // Calculée par l'appelant pour garder le repo email_templates découplé de dunning_levels.
    let max_reminder_level =
        dunning_levels::count_for_company(&state.pool, company.id).await? as i16;
    let list =
        email_templates::list_effective_for_company(&state.pool, company.id, max_reminder_level)
            .await?;
    Ok(Json(list.into_iter().map(Into::into).collect()))
}

/// `GET /api/v1/admin/email-templates/{template_type}/{language}` — un
/// template effectif unique. Jamais 404 pour une combinaison valide.
pub async fn get_email_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((template_type_raw, language_raw)): Path<(String, String)>,
) -> Result<Json<EmailTemplateResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let template_type = parse_template_type(&template_type_raw)?;
    let language = parse_language(&language_raw)?;

    // Niveau 0 (générique) : le segment de niveau des routes arrive avec l'UI 21-4.
    let effective =
        email_templates::get_effective(&state.pool, company.id, template_type, language, 0).await?;
    Ok(Json(effective.into()))
}

/// `PUT /api/v1/admin/email-templates/{template_type}/{language}` — crée ou
/// modifie l'override. Valide non-vide + tokens `{var}` connus avant toute
/// persistance (AC #13).
pub async fn update_email_template(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((template_type_raw, language_raw)): Path<(String, String)>,
    Json(req): Json<UpdateEmailTemplateRequest>,
) -> Result<Json<EmailTemplateResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let template_type = parse_template_type(&template_type_raw)?;
    let language = parse_language(&language_raw)?;

    let subject = req.subject.trim();
    let body = req.body.trim();
    if subject.is_empty() || body.is_empty() {
        return Err(AppError::Validation(
            "Le sujet et le corps du template ne peuvent pas être vides".to_string(),
        ));
    }

    kesh_core::email_template_engine::validate_tokens(
        subject,
        body,
        template_type.allowed_variables(),
    )
    .map_err(|unknown_vars| AppError::EmailTemplateUnknownVariables { unknown_vars })?;

    let updated = email_templates::upsert_override(
        &state.pool,
        company.id,
        template_type,
        language,
        0,
        req.expected_version,
        current_user.user_id,
        current_user.api_key_id,
        subject.to_string(),
        body.to_string(),
    )
    .await?;

    Ok(Json(updated.into()))
}

/// `DELETE /api/v1/admin/email-templates/{template_type}/{language}` —
/// restaure le défaut (supprime l'override). Idempotent → toujours `204`.
pub async fn restore_email_template_default(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path((template_type_raw, language_raw)): Path<(String, String)>,
) -> Result<StatusCode, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let template_type = parse_template_type(&template_type_raw)?;
    let language = parse_language(&language_raw)?;

    email_templates::restore_default(
        &state.pool,
        company.id,
        template_type,
        language,
        0,
        current_user.user_id,
        current_user.api_key_id,
    )
    .await?;

    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Tests unitaires (validation)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_template_type_ok() {
        assert_eq!(
            parse_template_type("invoice_send").unwrap(),
            EmailTemplateType::InvoiceSend
        );
    }

    #[test]
    fn parse_template_type_unknown_rejected() {
        assert!(parse_template_type("unknown_type").is_err());
        assert!(parse_template_type("InvoiceSend").is_err()); // casse matters
    }

    #[test]
    fn parse_language_ok() {
        assert_eq!(parse_language("FR").unwrap(), Language::Fr);
    }

    #[test]
    fn parse_language_unknown_rejected() {
        assert!(parse_language("fr").is_err()); // casse matters (BINARY)
        assert!(parse_language("XX").is_err());
    }
}
