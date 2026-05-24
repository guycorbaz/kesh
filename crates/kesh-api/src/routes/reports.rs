//! Routes API rapports comptables — Story 9-1 (génération JSON) + Story 9-2a (export PDF/CSV).
//!
//! Story 9-1 — 4 endpoints `GET /api/v1/reports/{type}` :
//! - balance-sheet : Bilan
//! - income-statement : Compte de résultat
//! - trial-balance : Balance des comptes
//! - journals : Journaux (5 sections fixes + filter optionnel)
//!
//! Story 9-2a — 4 endpoints `GET /api/v1/reports/{type}/export?format=pdf|csv` :
//! Réponses binaires (`application/pdf` ou `text/csv; charset=utf-8`) avec
//! `Content-Disposition: attachment; filename=...; filename*=UTF-8''...` (RFC 5987).
//!
//! Tous montés dans `authenticated_routes` (Admin + Comptable + Consultation —
//! lecture seule). Multi-tenant strict via `current_user.company_id`.
//!
//! Pattern audit best-effort (Pass 1 ECH-15) : INSERT audit après SELECT métier
//! réussi, dans une transaction dédiée, échec → `warn!` + retour 200.

use axum::{
    Extension, Json,
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::Response,
};
use chrono::NaiveDate;
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::journal_entry::Journal;
use kesh_report::{
    BalanceSheet, IncomeStatement, JournalReport, PdfContext, ReportPeriod, TrialBalance,
    generate_balance_sheet, generate_income_statement, generate_journal_report,
    generate_trial_balance, render_balance_sheet_csv, render_balance_sheet_pdf,
    render_income_statement_csv, render_income_statement_pdf, render_journal_report_csv,
    render_journal_report_pdf, render_trial_balance_csv, render_trial_balance_pdf,
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

/// Query params pour les 4 endpoints d'export (Story 9-2a).
///
/// Le champ `format` est `Option<String>` (pas `enum`) — la validation est faite
/// handler-side via [`validate_format`] qui retourne `AppError::Validation` 400
/// (cohérent JSON, évite le 422 Axum sur deserialization enum failure — Pass 1 BH-H2).
/// Le champ `journal` est ignoré par les 3 premiers rapports (pattern Story 9-1).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportQuery {
    pub fiscal_year_id: i64,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    pub journal: Option<Journal>,
    pub format: Option<String>,
}

/// Discriminant interne PDF/CSV après validation handler-side.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ExportFormat {
    Pdf,
    Csv,
}

impl ExportFormat {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Csv => "csv",
        }
    }
    fn content_type(&self) -> &'static str {
        match self {
            Self::Pdf => "application/pdf",
            Self::Csv => "text/csv; charset=utf-8",
        }
    }
    fn extension(&self) -> &'static str {
        match self {
            Self::Pdf => "pdf",
            Self::Csv => "csv",
        }
    }
}

/// Valide le paramètre `format` query string — strict lowercase (Pass 2 ECH2-H1).
///
/// Rejette : `None`, `Some("")`, `Some("PDF")` (uppercase), `Some("Csv")`
/// (mixed case), tout autre. Cohérent AC #27.
fn validate_format(format: &Option<String>) -> Result<ExportFormat, AppError> {
    match format.as_deref() {
        Some("pdf") => Ok(ExportFormat::Pdf),
        Some("csv") => Ok(ExportFormat::Csv),
        _ => Err(AppError::Validation(
            "format manquant ou invalide, attendu pdf|csv (lowercase strict)".to_string(),
        )),
    }
}

// ===========================================================================
// Story 9-1 — Endpoints JSON
// ===========================================================================

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

// ===========================================================================
// Story 9-2a — Endpoints export PDF/CSV
// ===========================================================================

/// GET /api/v1/reports/balance-sheet/export?format=pdf|csv
pub async fn export_balance_sheet(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "balance-sheet",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_balance_sheet(&state.pool, current_user.company_id, &period).await?;

    let (ctx, company_name) = load_pdf_context(&state.pool, current_user.company_id).await?;

    let body: Vec<u8> = match format {
        ExportFormat::Pdf => render_balance_sheet_pdf(&report, &ctx)?,
        ExportFormat::Csv => render_csv_to_vec(|w| render_balance_sheet_csv(&report, w))?,
    };

    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);

    emit_report_export_audit(
        &state.pool,
        current_user.user_id,
        "balance-sheet",
        format.as_str(),
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    // Pass 1 code-review H10 : type_slug localisé via i18n (fr-CH → `bilan`).
    let type_slug = resolve_type_slug(&state, &ctx.locale, "balance-sheet");
    // Pass 1 code-review M14 : Content-Disposition `filename*=UTF-8'<lang>'…`
    build_export_response_with_locale(
        format,
        body,
        &type_slug,
        &company_name,
        &period,
        &ctx.locale,
    )
}

/// GET /api/v1/reports/income-statement/export?format=pdf|csv
pub async fn export_income_statement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "income-statement",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_income_statement(&state.pool, current_user.company_id, &period).await?;
    let (ctx, company_name) = load_pdf_context(&state.pool, current_user.company_id).await?;

    let body: Vec<u8> = match format {
        ExportFormat::Pdf => render_income_statement_pdf(&report, &ctx)?,
        ExportFormat::Csv => render_csv_to_vec(|w| render_income_statement_csv(&report, w))?,
    };

    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);

    emit_report_export_audit(
        &state.pool,
        current_user.user_id,
        "income-statement",
        format.as_str(),
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    // Pass 1 code-review H10 : type_slug localisé via i18n.
    let type_slug = resolve_type_slug(&state, &ctx.locale, "income-statement");
    // Pass 1 code-review M14 : Content-Disposition `filename*=UTF-8'<lang>'…`
    build_export_response_with_locale(
        format,
        body,
        &type_slug,
        &company_name,
        &period,
        &ctx.locale,
    )
}

/// GET /api/v1/reports/trial-balance/export?format=pdf|csv
pub async fn export_trial_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "trial-balance",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let period = ReportPeriod::resolve(
        &state.pool,
        current_user.company_id,
        query.fiscal_year_id,
        query.period_start,
        query.period_end,
    )
    .await?;

    let report = generate_trial_balance(&state.pool, current_user.company_id, &period).await?;
    let (ctx, company_name) = load_pdf_context(&state.pool, current_user.company_id).await?;

    let body: Vec<u8> = match format {
        ExportFormat::Pdf => render_trial_balance_pdf(&report, &ctx)?,
        ExportFormat::Csv => render_csv_to_vec(|w| render_trial_balance_csv(&report, w))?,
    };

    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);

    emit_report_export_audit(
        &state.pool,
        current_user.user_id,
        "trial-balance",
        format.as_str(),
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        None,
    )
    .await;

    // Pass 1 code-review H10 : type_slug localisé via i18n.
    let type_slug = resolve_type_slug(&state, &ctx.locale, "trial-balance");
    // Pass 1 code-review M14 : Content-Disposition `filename*=UTF-8'<lang>'…`
    build_export_response_with_locale(
        format,
        body,
        &type_slug,
        &company_name,
        &period,
        &ctx.locale,
    )
}

/// GET /api/v1/reports/journals/export?format=pdf|csv&journal=Ventes
pub async fn export_journal_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    validate_fiscal_year_id(query.fiscal_year_id)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "journals",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

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
    let (mut ctx, company_name) = load_pdf_context(&state.pool, current_user.company_id).await?;
    if let Some(j) = query.journal {
        ctx.journal_filter_label = Some(j.as_str().to_string());
    }

    let body: Vec<u8> = match format {
        ExportFormat::Pdf => render_journal_report_pdf(&report, &ctx)?,
        ExportFormat::Csv => render_csv_to_vec(|w| render_journal_report_csv(&report, w))?,
    };

    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);

    let journal_filter_str = query.journal.map(|j| j.as_str().to_string());
    emit_report_export_audit(
        &state.pool,
        current_user.user_id,
        "journals",
        format.as_str(),
        query.fiscal_year_id,
        period.start_date,
        period.end_date,
        journal_filter_str.as_deref(),
    )
    .await;

    // Pass 1 code-review H10 : type_slug localisé via i18n.
    let type_slug = resolve_type_slug(&state, &ctx.locale, "journals");
    // Pass 1 code-review M14 : Content-Disposition `filename*=UTF-8'<lang>'…`
    build_export_response_with_locale(
        format,
        body,
        &type_slug,
        &company_name,
        &period,
        &ctx.locale,
    )
}

// ===========================================================================
// Helpers privés Story 9-2a
// ===========================================================================

/// Charge le `PdfContext` côté handler (1 query DB `SELECT name, accounting_language
/// FROM companies WHERE id = ?`). Libellés résolus avec les defaults FR-CH v0.1
/// (Pass 3 ECH3-C1 — accept lib `kesh-i18n` non utilisée pour les sérialiseurs
/// PDF/CSV, DD-14 pattern).
///
/// Retourne `(ctx, company_name)` — `company_name` est aussi utilisé séparément
/// pour construire le filename.
///
/// **Pass 1 code-review H5 (BH2-H1) — scoping multi-tenant** : `company_id`
/// provient de `Extension<CurrentUser>` injecté par `middleware/auth.rs` à
/// partir du claim JWT signé HS256 (cf. `middleware/auth.rs:96`). Le claim
/// `company_id` est garanti = la company dont l'utilisateur est membre, le JWT
/// étant signé côté serveur lors du login. Il n'y a donc pas d'IDOR ici tant
/// que le JWT guard est sain (éprouvé Story 1-5/1-6 + 7-1). Ajouter
/// `AND company_id = ?` ne fournirait aucune protection supplémentaire car la
/// valeur testée serait identique à la PK. Documenté comme dette de défense
/// en profondeur — voir Limitations L13.
async fn load_pdf_context(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<(PdfContext, String), AppError> {
    let row: (String, String) =
        sqlx::query_as("SELECT name, accounting_language FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(kesh_db::errors::map_db_error)?;
    let (company_name, locale_code) = row;

    // v0.1 : tous libellés FR-CH par défaut. L'extension i18n complète des
    // libellés PDF est reportée à v0.2 (L4 + L11). Le code locale est exposé
    // via PdfContext pour traceability future.
    //
    // Story 9-2b T5.1 (Pass 3 ECH3-H2) : mapping extrait vers `util::map_language_to_bcp47`
    // pour réutilisation par `routes::exports`. Politique fallback inchangée.
    let bcp47 = crate::util::map_language_to_bcp47(&locale_code);

    let mut ctx = PdfContext::fr_ch_default(company_name.clone());
    ctx.locale = bcp47.to_string();
    Ok((ctx, company_name))
}

/// Résout le slug localisé d'un type de rapport via `kesh-i18n` (clé
/// `reports-filename-{report_type}`).
///
/// Pass 1 code-review H10 (AA2-H1 + AA4-F1) : AC #22 mandate que `{typeSlug}`
/// du filename soit la **valeur** de la clé i18n `reports-filename-{reportType}`
/// résolue dans la locale active (fr-CH → `bilan`, de-CH → `bilanz`, etc.).
/// Avant ce patch, le slug brut anglais (`balance-sheet`, etc.) était passé à
/// `build_filename` qui le slugifiait sans traduction — divergence AC #22.
fn resolve_type_slug(state: &AppState, locale_bcp47: &str, report_type: &str) -> String {
    let locale = kesh_i18n::Locale::from(locale_bcp47);
    let key = format!("reports-filename-{report_type}");
    let resolved = state.i18n.format(&locale, &key, None);
    // Sécurité : si la clé est absente partout, `format` retourne la clé brute.
    // Dans ce cas on retombe sur le report_type anglais (fallback).
    if resolved == key {
        tracing::warn!(
            i18n_key = %key,
            locale = %locale_bcp47,
            "reports-filename-* manquant en i18n, fallback report_type brut"
        );
        report_type.to_string()
    } else {
        resolved
    }
}

/// Helper : exécute un closure CSV writer dans un `Vec<u8>` (cf. L5 — pas de
/// streaming Axum body v0.1).
fn render_csv_to_vec<F>(f: F) -> Result<Vec<u8>, AppError>
where
    F: FnOnce(&mut Vec<u8>) -> Result<(), kesh_report::ReportError>,
{
    let mut buf = Vec::new();
    f(&mut buf)?;
    Ok(buf)
}

/// Construit la `Response` HTTP binaire avec headers Content-Type + Content-Disposition
/// (T5.3 + RFC 5987 + AA-M3 + Pass 1 code-review M14 — BH2-M2).
///
/// Le paramètre `locale_bcp47` est inséré entre les deux apostrophes simples
/// de `filename*=UTF-8'<lang>'…` (RFC 5987 § 3.2.1). Locale vide (`""`)
/// produit `filename*=UTF-8''…` (sans tag langue, syntaxe également valide).
fn build_export_response_with_locale(
    format: ExportFormat,
    body: Vec<u8>,
    type_slug: &str,
    company_name: &str,
    period: &ReportPeriod,
    locale_bcp47: &str,
) -> Result<Response, AppError> {
    let filename = build_filename(type_slug, company_name, period, format.extension());

    let content_disposition = crate::util::build_content_disposition(&filename, locale_bcp47)?;

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, format.content_type())
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(format!("response build: {e}")))?;

    Ok(response)
}

/// Construit le filename `kesh-{type_slug}-{company_slug}-{periodStart}_{periodEnd}.{ext}`
/// (T5.4 + AC #22 + Pass 1 ECH-C1 + Pass 3 BH3-M3 + Pass 4 ECH4-L3).
///
/// Story 9-2b §util : `slugify` factorisé dans `crate::util` (réutilisé par
/// `routes::exports::build_global_filename`).
fn build_filename(type_slug: &str, company_name: &str, period: &ReportPeriod, ext: &str) -> String {
    let slug_type = crate::util::slugify(type_slug, "report");
    let slug_company = crate::util::slugify(company_name, "company");
    let period_start = period.start_date.format("%Y-%m-%d").to_string();
    let period_end = period.end_date.format("%Y-%m-%d").to_string();
    format!("kesh-{slug_type}-{slug_company}-{period_start}_{period_end}.{ext}")
}

// ===========================================================================
// Helpers communs (audit, validation)
// ===========================================================================

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
///
/// ## Convention projet — `audit_log.details_json` JSON keys
///
/// **Snake_case obligatoire** pour les clés JSON de `details_json` (cohérent SQL
/// JSON path `details_json->>'$.field_name'` future-proof, AC #23 Story 9-2b
/// explicite). Les autres surfaces API restent **camelCase** par convention
/// REST/JS :
///
/// | Surface | Convention | Exemple |
/// |---|---|---|
/// | Query params HTTP URL | camelCase | `?fiscalYearId=...&periodStart=...` |
/// | Request/Response body HTTP | camelCase (via `#[serde(rename_all = "camelCase")]`) | `{"reportType": "..."}` |
/// | `AppError.details` HTTP error body | camelCase | `{"details": {"fiscalYearId": ...}}` (cf. `errors.rs`) |
/// | Frontend `metadata.json` | camelCase | `{"keshVersion", "exportDate"}` |
/// | **`audit_log.details_json` (serveur)** | **snake_case** | `{"report_type", "fiscal_year_id"}` |
///
/// **Référence canonique** : `emit_global_export_audit` (Story 9-2b
/// `crates/kesh-api/src/routes/exports.rs`) — premier `emit_*_audit` snake_case.
/// Migration camelCase → snake_case appliquée 2026-05-18 Story 9-5-2 pour
/// `emit_report_audit` (Story 9-1) + `emit_report_export_audit` (Story 9-2a).
/// Référence pour futures fonctions audit Epic 10+ (`vat.calculated`,
/// `payment.created`, etc.).
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
                // Story 9-5-2 (Epic 9.5) — clés snake_case pour cohérence SQL JSON
                // path `details_json->>'$.field_name'` future-proof (cf. convention
                // §audit_log JSON keys documentée au-dessus de la fonction).
                details_json: Some(serde_json::json!({
                    "report_type": report_type,
                    "fiscal_year_id": fiscal_year_id,
                    "period_start": period_start.format("%Y-%m-%d").to_string(),
                    "period_end": period_end.format("%Y-%m-%d").to_string(),
                    "journal_filter": journal_filter,
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

/// Story 9-2a — audit log `report.exported` (best-effort, distinct de
/// `report.generated` pour faciliter les requêtes audit — Pass 1 BH-M2).
///
/// Fonction **séparée** de `emit_report_audit` — la modifier briserait les
/// 4 callers Story 9-1.
#[allow(clippy::too_many_arguments)]
async fn emit_report_export_audit(
    pool: &MySqlPool,
    user_id: i64,
    report_type: &str,
    format: &str,
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
                action: "report.exported".to_string(),
                entity_type: "report".to_string(),
                entity_id: AUDIT_ENTITY_ID_NONE,
                // Story 9-5-2 (Epic 9.5) — clés snake_case (cf. convention
                // §audit_log JSON keys au-dessus de `emit_report_audit`).
                details_json: Some(serde_json::json!({
                    "report_type": report_type,
                    "format": format,
                    "fiscal_year_id": fiscal_year_id,
                    "period_start": period_start.format("%Y-%m-%d").to_string(),
                    "period_end": period_end.format("%Y-%m-%d").to_string(),
                    "journal_filter": journal_filter,
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
            format,
            fiscal_year_id,
            "audit insert failed (report.exported) — non-blocking"
        );
    }
}

// ===========================================================================
// Tests unit helpers Story 9-2a (slugify, filename, content-disposition)
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn period_for_test() -> ReportPeriod {
        ReportPeriod {
            fiscal_year_id: 1,
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        }
    }

    // Tests `slugify*` migrés vers `crate::util::tests` (Story 9-2b §util).

    #[test]
    fn validate_format_accepts_lowercase_pdf_csv() {
        assert_eq!(
            validate_format(&Some("pdf".to_string())).unwrap(),
            ExportFormat::Pdf
        );
        assert_eq!(
            validate_format(&Some("csv".to_string())).unwrap(),
            ExportFormat::Csv
        );
    }

    #[test]
    fn validate_format_rejects_uppercase() {
        assert!(validate_format(&Some("PDF".to_string())).is_err());
        assert!(validate_format(&Some("Csv".to_string())).is_err());
    }

    #[test]
    fn validate_format_rejects_none_empty_invalid() {
        assert!(validate_format(&None).is_err());
        assert!(validate_format(&Some(String::new())).is_err());
        assert!(validate_format(&Some("xml".to_string())).is_err());
    }

    #[test]
    fn build_filename_kesh_pattern() {
        let period = period_for_test();
        let name = build_filename("balance-sheet", "CI Test Company", &period, "pdf");
        assert_eq!(
            name,
            "kesh-balance-sheet-ci-test-company-2026-01-01_2026-12-31.pdf"
        );
    }

    #[test]
    fn build_filename_handles_non_ascii_company() {
        let period = period_for_test();
        let name = build_filename("balance-sheet", "Müller AG", &period, "csv");
        assert_eq!(
            name,
            "kesh-balance-sheet-muller-ag-2026-01-01_2026-12-31.csv"
        );
    }

    // Tests `content_disposition_*` + `percent_encode_*` migrés vers
    // `crate::util::tests` (Story 9-2b §util).
}
