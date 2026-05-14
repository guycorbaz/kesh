//! Routes API rapports comptables — Story 9-1.
//!
//! 4 endpoints `GET /api/v1/reports/{type}` :
//! - balance-sheet : Bilan
//! - income-statement : Compte de résultat
//! - trial-balance : Balance des comptes
//! - journals : Journaux (5 sections fixes + filter optionnel)
//!
//! Tous montés dans `authenticated_routes` (Admin + Comptable + Consultation —
//! lecture seule). Multi-tenant strict via `current_user.company_id`.
//!
//! Pattern audit best-effort (Pass 1 ECH-15) : INSERT audit après SELECT métier
//! réussi, dans une transaction dédiée, échec → `warn!` + retour 200.

use axum::{
    Extension, Json,
    extract::{Query, State},
};
use chrono::NaiveDate;
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::journal_entry::Journal;
use kesh_report::{
    BalanceSheet, IncomeStatement, JournalReport, ReportPeriod, TrialBalance,
    generate_balance_sheet, generate_income_statement, generate_journal_report,
    generate_trial_balance,
};
use serde::Deserialize;
use sqlx::MySqlPool;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Query params communs aux 3 premiers rapports (balance-sheet, income-statement,
/// trial-balance).
///
/// Pass 2 AA2-06 : `#[serde(rename_all = "camelCase")]` obligatoire pour exposer
/// `fiscalYearId`, `periodStart`, `periodEnd` côté URL.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportQuery {
    pub fiscal_year_id: i64,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
}

/// Query params spécifiques au rapport journaux (avec `journal` optionnel).
///
/// Pass 3 BH3-13 : `#[serde(flatten)]` ne fonctionne pas avec `serde_urlencoded`
/// (Axum default). Champs dupliqués manuellement.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalReportQuery {
    pub fiscal_year_id: i64,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub journal: Option<Journal>,
}

/// GET /api/v1/reports/balance-sheet
pub async fn get_balance_sheet(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<BalanceSheet>, AppError> {
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_balance_sheet(&state.pool, current_user.company_id, &period).await?;

    emit_report_audit(
        &state.pool,
        current_user.user_id,
        "balance-sheet",
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    Ok(Json(report))
}

/// GET /api/v1/reports/income-statement
pub async fn get_income_statement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<IncomeStatement>, AppError> {
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_income_statement(&state.pool, current_user.company_id, &period).await?;

    emit_report_audit(
        &state.pool,
        current_user.user_id,
        "income-statement",
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    Ok(Json(report))
}

/// GET /api/v1/reports/trial-balance
pub async fn get_trial_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<TrialBalance>, AppError> {
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_trial_balance(&state.pool, current_user.company_id, &period).await?;

    emit_report_audit(
        &state.pool,
        current_user.user_id,
        "trial-balance",
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    Ok(Json(report))
}

/// GET /api/v1/reports/journals?journal=Achats
pub async fn get_journal_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<JournalReportQuery>,
) -> Result<Json<JournalReport>, AppError> {
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report =
        generate_journal_report(&state.pool, current_user.company_id, &period, query.journal)
            .await?;

    let journal_filter_str = query.journal.map(|j| j.as_str().to_string());
    emit_report_audit(
        &state.pool,
        current_user.user_id,
        "journals",
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        journal_filter_str.as_deref(),
    )
    .await;

    Ok(Json(report))
}

/// Validation `fiscalYearId > 0` handler-side (Pass 1 ECH-06 + Pass 3 BH3-01).
///
/// Le parsing serde i64 accepte 0 et négatifs sans erreur. Ce check explicite
/// retourne 400 `VALIDATION_ERROR` en JSON (via `build_response` standard).
fn validate_fiscal_year_id(fiscal_year_id: i64) -> Result<(), AppError> {
    if fiscal_year_id <= 0 {
        return Err(AppError::Validation(format!(
            "fiscalYearId must be > 0 (got {fiscal_year_id})"
        )));
    }
    Ok(())
}

/// Audit log `report.generated` — best-effort (Pass 1 ECH-15).
///
/// Pattern strict : log `warn!` sur erreur, ne JAMAIS faire échouer la réponse rapport.
async fn emit_report_audit(
    pool: &MySqlPool,
    user_id: i64,
    report_type: &str,
    fiscal_year_id: i64,
    period_start: NaiveDate,
    period_end: NaiveDate,
    journal_filter: Option<&str>,
) {
    let result = async {
        let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
        kesh_db::repositories::audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "report.generated".to_string(),
                entity_type: "report".to_string(),
                entity_id: AUDIT_ENTITY_ID_NONE,
                details_json: Some(serde_json::json!({
                    "reportType": report_type,
                    "fiscalYearId": fiscal_year_id,
                    "periodStart": period_start.format("%Y-%m-%d").to_string(),
                    "periodEnd": period_end.format("%Y-%m-%d").to_string(),
                    "journalFilter": journal_filter,
                })),
            },
        )
        .await?;
        tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
        Ok::<(), kesh_db::errors::DbError>(())
    }
    .await;

    if let Err(e) = result {
        tracing::warn!(
            error = ?e,
            user_id,
            report_type,
            fiscal_year_id,
            "audit insert failed (report.generated) — non-blocking"
        );
    }
}
