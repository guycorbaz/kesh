//! Routes `GET`/`PUT /api/v1/company/invoice-settings` (Story 5.2 — FR35).
//!
//! - `GET` : tout rôle authentifié (lecture config). Crée la row avec les
//!   DEFAULT si absente (pattern upsert read, cf. repository).
//! - `PUT` : Admin uniquement (paramétrage société).
//!
//! La validation métier (format, comptes Asset/Revenue actifs,
//! journal whitelist) vit ici. Le repository ne fait que persister +
//! auditer.

use axum::extract::State;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_core::invoice_format;
use kesh_db::entities::{
    CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal, account::AccountType,
};
use kesh_db::repositories::{accounts, companies, company_invoice_settings};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceSettingsResponse {
    pub company_id: i64,
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_sales_journal: String,
    pub journal_entry_description_template: String,
    pub version: i32,
}

impl From<CompanyInvoiceSettings> for InvoiceSettingsResponse {
    fn from(s: CompanyInvoiceSettings) -> Self {
        Self {
            company_id: s.company_id,
            invoice_number_format: s.invoice_number_format,
            default_receivable_account_id: s.default_receivable_account_id,
            default_revenue_account_id: s.default_revenue_account_id,
            default_sales_journal: s.default_sales_journal.as_str().to_string(),
            journal_entry_description_template: s.journal_entry_description_template,
            version: s.version,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvoiceSettingsRequest {
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_sales_journal: String,
    pub journal_entry_description_template: String,
    pub version: i32,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn get_company(state: &AppState) -> Result<kesh_db::entities::Company, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;
    list.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Aucune company en base".into()))
}

fn parse_journal(raw: &str) -> Result<Journal, AppError> {
    raw.parse::<Journal>()
        .map_err(|_| AppError::Validation(format!("Journal inconnu : '{raw}'")))
}

async fn validate_account(
    state: &AppState,
    company_id: i64,
    account_id: Option<i64>,
    expected: AccountType,
    field_label: &str,
) -> Result<(), AppError> {
    let Some(id) = account_id else {
        return Ok(());
    };
    let account = accounts::find_by_id(&state.pool, id)
        .await?
        .ok_or_else(|| AppError::Validation(format!("{field_label} : compte introuvable")))?;
    if account.company_id != company_id {
        return Err(AppError::Validation(format!(
            "{field_label} : compte introuvable"
        )));
    }
    if !account.active {
        return Err(AppError::Validation(format!(
            "{field_label} : compte archivé"
        )));
    }
    if account.account_type != expected {
        return Err(AppError::Validation(format!(
            "{field_label} : type de compte incompatible (attendu {})",
            expected.as_str()
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/company/invoice-settings` — lecture config (tout rôle auth).
pub async fn get_invoice_settings(
    State(state): State<AppState>,
) -> Result<Json<InvoiceSettingsResponse>, AppError> {
    let company = get_company(&state).await?;
    let settings = company_invoice_settings::get_or_create_default(&state.pool, company.id).await?;
    Ok(Json(settings.into()))
}

/// `PUT /api/v1/company/invoice-settings` — mise à jour config (Admin).
pub async fn update_invoice_settings(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<UpdateInvoiceSettingsRequest>,
) -> Result<Json<InvoiceSettingsResponse>, AppError> {
    let company = get_company(&state).await?;

    // 1. Valider le format.
    invoice_format::validate_template(&req.invoice_number_format)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 2. Valider le template de description.
    invoice_format::validate_description_template(&req.journal_entry_description_template)
        .map_err(|e| AppError::Validation(e.to_string()))?;

    // 3. Valider le journal (whitelist via FromStr).
    let journal = parse_journal(&req.default_sales_journal)?;

    // 4. Valider les comptes (existence, scope company, type, actif).
    validate_account(
        &state,
        company.id,
        req.default_receivable_account_id,
        AccountType::Asset,
        "Compte créance",
    )
    .await?;
    validate_account(
        &state,
        company.id,
        req.default_revenue_account_id,
        AccountType::Revenue,
        "Compte produit",
    )
    .await?;

    // 5. Persister.
    let update = CompanyInvoiceSettingsUpdate {
        invoice_number_format: req.invoice_number_format,
        default_receivable_account_id: req.default_receivable_account_id,
        default_revenue_account_id: req.default_revenue_account_id,
        default_sales_journal: journal,
        journal_entry_description_template: req.journal_entry_description_template,
    };
    let settings = company_invoice_settings::update(
        &state.pool,
        company.id,
        req.version,
        current_user.user_id,
        update,
    )
    .await?;

    Ok(Json(settings.into()))
}

// ---------------------------------------------------------------------------
// Tests unitaires (validation)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_journal_ok() {
        assert_eq!(parse_journal("Ventes").unwrap(), Journal::Ventes);
        assert_eq!(parse_journal("OD").unwrap(), Journal::OD);
    }

    #[test]
    fn parse_journal_unknown_rejected() {
        assert!(parse_journal("Sales").is_err());
        assert!(parse_journal("ventes").is_err()); // casse matters (BINARY)
    }
}
