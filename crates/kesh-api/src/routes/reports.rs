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
//! Story 32-3 — handlers minces : la génération, le contrôle d'accès, la
//! validation de la demande et la trace d'audit (best-effort, écrite après le
//! SELECT métier réussi, échec → `warn!` + retour 200) vivent dans la couche de
//! services (`crate::services::rapports`). Le handler valide le format
//! d'export, charge le contexte PDF, rend, et construit la réponse.

use axum::{
    Extension, Json,
    body::Body,
    extract::{Query, State},
    http::{StatusCode, header},
    response::Response,
};
use chrono::NaiveDate;
use kesh_db::entities::journal_entry::Journal;
use kesh_report::pdf::{ProjectExpensesPdfLabels, ProjectReturnPdfLabels, VatPdfLabels};
use kesh_report::project_report::ProjectPeriodMode;
use kesh_report::{
    AgedReceivables, BalanceSheet, GeneralLedger, IncomeStatement, JournalReport, PdfContext,
    ProjectExpensesReport, ProjectReturnReport, ReportPeriod, TrialBalance, VatReport,
    render_aged_receivables_csv, render_balance_sheet_csv, render_balance_sheet_pdf,
    render_general_ledger_csv, render_general_ledger_pdf, render_income_statement_csv,
    render_income_statement_pdf, render_journal_report_csv, render_journal_report_pdf,
    render_project_expenses_csv, render_project_expenses_pdf, render_project_return_csv,
    render_project_return_pdf, render_trial_balance_csv, render_trial_balance_pdf,
    render_vat_report_csv, render_vat_report_pdf,
};
use serde::Deserialize;

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::services::acces::ContexteAcces;
use crate::services::poignees::Lecture;
use crate::services::rapports::FormatExport as ExportFormat;
use crate::services::rapports::{
    self, Affichage, DemandeGrandLivre, DemandeJournaux, DemandePeriode, DemandeProjet, Export,
    ModeProjet,
};

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

impl ReportQuery {
    fn demande(&self) -> DemandePeriode {
        DemandePeriode {
            fiscal_year_id: self.fiscal_year_id,
            period_start: self.period_start,
            period_end: self.period_end,
        }
    }
}

/// Query params du grand livre.
///
/// ⚠️ **Aucun `fiscalYearId`, et c'est le propre de ce rapport** : le grand livre
/// franchit la borne d'exercice, sans quoi il ne concorderait pas avec le bilan,
/// qui est cumulatif depuis l'origine.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralLedgerQuery {
    pub from: NaiveDate,
    pub to: NaiveDate,
    /// Comptes demandés, séparés par des virgules. Absent ⇒ tous ceux qui ont un
    /// mouvement ou un solde d'ouverture.
    pub account_ids: Option<String>,
    /// Rendre aussi les comptes sans mouvement **et** sans solde.
    pub include_zero: Option<bool>,
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

/// Query params de l'export du grand livre.
///
/// ⚠️ Ni `fiscalYearId`, ni `limit` : le grand livre franchit la borne
/// d'exercice, et un export tronqué n'aurait aucune valeur probante.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralLedgerExportQuery {
    pub format: Option<String>,
    pub from: NaiveDate,
    pub to: NaiveDate,
    pub account_ids: Option<String>,
    pub include_zero: Option<bool>,
}

impl GeneralLedgerQuery {
    fn demande(&self) -> DemandeGrandLivre {
        DemandeGrandLivre {
            from: self.from,
            to: self.to,
            account_ids: self.account_ids.clone(),
            include_zero: self.include_zero.unwrap_or(false),
            offset: self.offset,
            limit: self.limit,
        }
    }
}

impl GeneralLedgerExportQuery {
    fn demande(&self) -> DemandeGrandLivre {
        DemandeGrandLivre {
            from: self.from,
            to: self.to,
            account_ids: self.account_ids.clone(),
            include_zero: self.include_zero.unwrap_or(false),
            // Sans effet : l'export n'a jamais de pagination (service).
            offset: None,
            limit: None,
        }
    }
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

impl JournalReportQuery {
    fn demande(&self) -> DemandeJournaux {
        DemandeJournaux {
            periode: DemandePeriode {
                fiscal_year_id: self.fiscal_year_id,
                period_start: self.period_start,
                period_end: self.period_end,
            },
            journal: self.journal,
        }
    }
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

impl ExportQuery {
    /// La demande d'un rapport à période — `journal` ignoré.
    fn demande(&self) -> DemandePeriode {
        DemandePeriode {
            fiscal_year_id: self.fiscal_year_id,
            period_start: self.period_start,
            period_end: self.period_end,
        }
    }

    /// La demande du rapport des journaux.
    fn demande_journaux(&self) -> DemandeJournaux {
        DemandeJournaux {
            periode: self.demande(),
            journal: self.journal,
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

/// Une valeur qu'un `Rapport` expose toujours pour son service — son absence
/// serait un défaut de code, non une erreur de la demande.
fn attendu<V>(valeur: Option<V>) -> Result<V, AppError> {
    valeur.ok_or_else(|| {
        AppError::Internal("rapport sans les bornes de son nom de fichier".to_string())
    })
}

/// Enregistre au `span` d'export la taille du corps et la durée depuis `start` —
/// après le rendu, avant la trace (Story 32-3, C-32-3-28).
fn mesurer(span: &tracing::Span, start: std::time::Instant, body: &[u8]) {
    span.record("byte_size", body.len());
    span.record("duration_ms", start.elapsed().as_millis() as u64);
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
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport = rapports::bilan(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/income-statement
pub async fn get_income_statement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<IncomeStatement>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport =
        rapports::compte_de_resultat(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/trial-balance
/// `GET /api/v1/reports/general-ledger` — l'extrait d'un ou plusieurs comptes.
///
/// Répond à la question que ni la balance ni le bilan ne savent traiter : **de
/// quoi ce solde est-il fait ?**
///
/// ⚠️ Scoping : `company_id` est appliqué **dans les requêtes du générateur**, à
/// la fois sur `accounts` et sur `journal_entries` — `journal_entry_lines` ne
/// porte pas de `company_id`.
pub async fn get_general_ledger(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<GeneralLedgerQuery>,
) -> Result<Json<GeneralLedger>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport =
        rapports::grand_livre(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// `GET /api/v1/reports/general-ledger/export?format=pdf|csv`
///
/// ⚠️ **Sans pagination, et c'est le point.** L'écran s'arrête à
/// [`kesh_report::MAX_LEDGER_LIMIT`] lignes par compte ; l'export, jamais — c'est
/// lui qui porte l'obligation de produire le livre en entier (Olico art. 3), et le
/// bandeau « seules N lignes sont affichées » de l'écran renvoie à lui.
pub async fn export_general_ledger(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<GeneralLedgerExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "general-ledger",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport = rapports::grand_livre(&acces, &lecture, query.demande(), Export(format)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let period = *attendu(rapport.bornes_grand_livre())?;

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                ExportFormat::Pdf => render_general_ledger_pdf(report, &ctx)?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_general_ledger_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

    let type_slug = resolve_type_slug(&state, &ctx.locale, "general-ledger");
    build_project_export_response(
        format,
        body,
        &type_slug,
        &company_name,
        period.from,
        period.to,
        &ctx.locale,
    )
}

pub async fn get_trial_balance(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<TrialBalance>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport = rapports::balance(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/journals?journal=Achats
pub async fn get_journal_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<JournalReportQuery>,
) -> Result<Json<JournalReport>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport = rapports::journaux(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
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

    let span = tracing::info_span!(
        "report_export",
        report_type = "balance-sheet",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport = rapports::bilan(&acces, &lecture, query.demande(), Export(format)).await?;

    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let period = attendu(rapport.periode())?.clone();

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                ExportFormat::Pdf => render_balance_sheet_pdf(report, &ctx)?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_balance_sheet_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

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

// ===========================================================================
// Story 21-7 — Balance âgée débiteurs (aged receivables)
// ===========================================================================
//
// Divergences délibérées vs les rapports comptables ci-dessus :
// - **Pas de `ReportPeriod`/`fiscalYearId`** : un rapport « as-of » à date unique
//   (aujourd'hui, v1). D'où ses fonctions d'audit propres (services::rapports) et
//   un filename bespoke (`build_filename` exige un `&ReportPeriod`, cf. validate
//   21-7 H1).
// - **RBAC divergent (D24)** : la vue JSON est tous-rôles (`authenticated_routes`),
//   l'export CSV est Comptable+ (`comptable_routes`, cf. lib.rs).

/// Query de l'export balance âgée : seul le format (CSV) est pertinent.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgedExportQuery {
    pub format: Option<String>,
}

/// Valide le format de l'export balance âgée — **défaut `csv`, seul `csv`
/// accepté** (21-7 : pas de PDF). Distincte de `validate_format` (qui rejette
/// `None`) : ici `None` retombe sur csv.
fn validate_aged_export_format(format: &Option<String>) -> Result<(), AppError> {
    match format.as_deref() {
        None | Some("csv") => Ok(()),
        _ => Err(AppError::Validation(
            "format invalide, seul csv est supporté pour la balance âgée".to_string(),
        )),
    }
}

/// GET /api/v1/reports/aged-receivables — balance âgée arrêtée à aujourd'hui.
/// Tous rôles (lecture seule, D-7b). Scoping multi-tenant via `company_id`.
pub async fn get_aged_receivables(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<AgedReceivables>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport = rapports::balance_agee(&acces, &state.lecture(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/aged-receivables/export?format=csv — export CSV.
/// **Comptable+** (D24, monté dans `comptable_routes`).
pub async fn export_aged_receivables(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<AgedExportQuery>,
) -> Result<Response, AppError> {
    validate_aged_export_format(&query.format)?;

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport = rapports::balance_agee(&acces, &lecture, Export(ExportFormat::Csv)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    // La date du contenu et de la trace, calculée une fois par le service.
    let as_of = attendu(rapport.as_of())?;
    let body: Vec<u8> = rapport
        .exporter(&state.trace(), |report| {
            render_csv_to_vec(|w| render_aged_receivables_csv(report, w))
        })
        .await?;

    // Filename « date unique » (patron `exports.rs::build_global_filename`, PAS
    // `build_filename` qui exige un `&ReportPeriod` — validate 21-7 H1).
    let slug_company = crate::util::slugify(&company_name, "company");
    let filename = format!(
        "kesh-balance-agee-{}-{}.csv",
        slug_company,
        as_of.format("%Y-%m-%d")
    );
    let content_disposition = crate::util::build_content_disposition(&filename, &ctx.locale)?;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/csv; charset=utf-8")
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
}

/// GET /api/v1/reports/income-statement/export?format=pdf|csv
pub async fn export_income_statement(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "income-statement",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport =
        rapports::compte_de_resultat(&acces, &lecture, query.demande(), Export(format)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let period = attendu(rapport.periode())?.clone();

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                ExportFormat::Pdf => render_income_statement_pdf(report, &ctx)?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_income_statement_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

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

// ===========================================================================
// Story 19-6a — Rapports analytiques par projet (Dépenses par projet)
// ===========================================================================

/// Query params des rapports par projet (JSON + export).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectReportQuery {
    pub project_id: i64,
    /// `fiscal_year` (exercice unique) ou `cumulative` (depuis l'origine).
    pub mode: ProjectModeParam,
    /// Requis si `mode == fiscal_year`.
    pub fiscal_year_id: Option<i64>,
    pub period_start: Option<NaiveDate>,
    pub period_end: Option<NaiveDate>,
    /// Export uniquement — ignoré par les endpoints JSON.
    pub format: Option<String>,
}

impl ProjectReportQuery {
    fn demande(&self) -> DemandeProjet {
        DemandeProjet {
            project_id: self.project_id,
            mode: match self.mode {
                ProjectModeParam::FiscalYear => ModeProjet::Exercice,
                ProjectModeParam::Cumulative => ModeProjet::Cumule,
            },
            fiscal_year_id: self.fiscal_year_id,
            period_start: self.period_start,
            period_end: self.period_end,
        }
    }
}

/// Mode de période (paramètre URL).
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectModeParam {
    FiscalYear,
    Cumulative,
}

/// GET /api/v1/reports/project-expenses
pub async fn get_project_expenses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProjectReportQuery>,
) -> Result<Json<ProjectExpensesReport>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport =
        rapports::depenses_projet(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/project-expenses/export?format=pdf|csv
pub async fn export_project_expenses(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProjectReportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport =
        rapports::depenses_projet(&acces, &lecture, query.demande(), Export(format)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let (start, end) = project_mode_dates(attendu(rapport.mode_projet())?);

    let body = rapport
        .exporter(&state.trace(), |report| {
            Ok(match format {
                ExportFormat::Pdf => render_project_expenses_pdf(
                    report,
                    &ctx,
                    &ProjectExpensesPdfLabels::fr_ch_defaults(),
                )?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_project_expenses_csv(report, w))?,
            })
        })
        .await?;

    let type_slug = resolve_type_slug(&state, &ctx.locale, "project-expenses");
    build_project_export_response(
        format,
        body,
        &type_slug,
        &company_name,
        start,
        end,
        &ctx.locale,
    )
}

/// Bornes de date effectives d'un mode (pour le filename).
fn project_mode_dates(mode: &ProjectPeriodMode) -> (NaiveDate, NaiveDate) {
    match mode {
        ProjectPeriodMode::FiscalYear { period } => (period.start_date, period.end_date),
        ProjectPeriodMode::Cumulative { start, end } => {
            // start None → borne basse = epoch projet inconnue ; on met la date de fin
            // en début aussi n'a pas de sens ; utilise une date minimale lisible.
            (start.unwrap_or(*end), *end)
        }
    }
}

/// Variante de [`build_export_response_with_locale`] avec dates explicites (les
/// rapports projet n'ont pas de `ReportPeriod` en mode cumulé).
fn build_project_export_response(
    format: ExportFormat,
    body: Vec<u8>,
    type_slug: &str,
    company_name: &str,
    start: NaiveDate,
    end: NaiveDate,
    locale_bcp47: &str,
) -> Result<Response, AppError> {
    let slug_type = crate::util::slugify(type_slug, "report");
    let slug_company = crate::util::slugify(company_name, "company");
    let filename = format!(
        "kesh-{slug_type}-{slug_company}-{}_{}.{}",
        start.format("%Y-%m-%d"),
        end.format("%Y-%m-%d"),
        format.extension()
    );
    let content_disposition = crate::util::build_content_disposition(&filename, locale_bcp47)?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, format.content_type())
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(Body::from(body))
        .map_err(|e| AppError::Internal(format!("response build: {e}")))
}

// ---------------------------------------------------------------------------
// Story 19-6b — Rapport « Rendement par projet »
// ---------------------------------------------------------------------------

/// GET /api/v1/reports/project-return
pub async fn get_project_return(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProjectReportQuery>,
) -> Result<Json<ProjectReturnReport>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport =
        rapports::rendement_projet(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/project-return/export?format=pdf|csv
pub async fn export_project_return(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ProjectReportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;
    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport =
        rapports::rendement_projet(&acces, &lecture, query.demande(), Export(format)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let (start, end) = project_mode_dates(attendu(rapport.mode_projet())?);

    let body = rapport
        .exporter(&state.trace(), |report| {
            Ok(match format {
                ExportFormat::Pdf => render_project_return_pdf(
                    report,
                    &ctx,
                    &ProjectReturnPdfLabels::fr_ch_defaults(),
                )?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_project_return_csv(report, w))?,
            })
        })
        .await?;

    let type_slug = resolve_type_slug(&state, &ctx.locale, "project-return");
    build_project_export_response(
        format,
        body,
        &type_slug,
        &company_name,
        start,
        end,
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

    let span = tracing::info_span!(
        "report_export",
        report_type = "trial-balance",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport = rapports::balance(&acces, &lecture, query.demande(), Export(format)).await?;
    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let period = attendu(rapport.periode())?.clone();

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                ExportFormat::Pdf => render_trial_balance_pdf(report, &ctx)?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_trial_balance_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

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

    let span = tracing::info_span!(
        "report_export",
        report_type = "journals",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport =
        rapports::journaux(&acces, &lecture, query.demande_journaux(), Export(format)).await?;
    let (mut ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    if let Some(j) = query.journal {
        ctx.journal_filter_label = Some(j.as_str().to_string());
    }
    let period = attendu(rapport.periode())?.clone();

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                ExportFormat::Pdf => render_journal_report_pdf(report, &ctx)?,
                ExportFormat::Csv => render_csv_to_vec(|w| render_journal_report_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

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
// Story 11-2 — Rapport TVA (JSON + export PDF/CSV)
// ===========================================================================

/// GET /api/v1/reports/vat
///
/// Rapport TVA due (vente) par taux pour la période. Lecture seule, tous rôles
/// authentifiés (FR65). Anti-IDOR via `current_user.company_id`.
pub async fn get_vat_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ReportQuery>,
) -> Result<Json<VatReport>, AppError> {
    let acces = ContexteAcces::depuis_api(&current_user);
    let rapport = rapports::tva(&acces, &state.lecture(), query.demande(), Affichage).await?;
    Ok(Json(rapport.afficher(&state.trace()).await))
}

/// GET /api/v1/reports/vat/export?format=pdf|csv
pub async fn export_vat_report(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ExportQuery>,
) -> Result<Response, AppError> {
    let format = validate_format(&query.format)?;

    let span = tracing::info_span!(
        "report_export",
        report_type = "vat",
        format = format.as_str(),
        byte_size = tracing::field::Empty,
        duration_ms = tracing::field::Empty
    );
    let _enter = span.enter();
    let start = std::time::Instant::now();

    let acces = ContexteAcces::depuis_api(&current_user);
    let lecture = state.lecture();
    let rapport = rapports::tva(&acces, &lecture, query.demande(), Export(format)).await?;

    let (ctx, company_name) = load_pdf_context(&acces, &lecture).await?;
    let period = attendu(rapport.periode())?.clone();

    let body = rapport
        .exporter(&state.trace(), |report| {
            let body: Vec<u8> = match format {
                // Libellés PDF = defaults FR-CH v0.1 (cohérent `load_pdf_context` /
                // `PdfContext::fr_ch_default` ; i18n PDF complète déférée v0.2+).
                ExportFormat::Pdf => {
                    render_vat_report_pdf(report, &ctx, &VatPdfLabels::fr_ch_defaults())?
                }
                ExportFormat::Csv => render_csv_to_vec(|w| render_vat_report_csv(report, w))?,
            };
            mesurer(&span, start, &body);
            Ok(body)
        })
        .await?;

    let type_slug = resolve_type_slug(&state, &ctx.locale, "vat");
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

/// Met en forme le `PdfContext` d'un export à partir de l'identité de la société,
/// lue par le service `rapports::identite_societe` (Story 32-3 : le handler ne
/// lit plus rien lui-même). Libellés résolus avec les defaults FR-CH v0.1
/// (Pass 3 ECH3-C1 — accept lib `kesh-i18n` non utilisée pour les sérialiseurs
/// PDF/CSV, DD-14 pattern).
///
/// Retourne `(ctx, company_name)` — `company_name` est aussi utilisé séparément
/// pour construire le filename.
///
/// Appelée **après** la génération, comme avant la 32-3 : un échec ici laisse le
/// `Rapport` abandonné sans `exporter`, donc sans trace.
async fn load_pdf_context(
    acces: &ContexteAcces,
    lecture: &Lecture,
) -> Result<(PdfContext, String), AppError> {
    let (company_name, locale_code) = rapports::identite_societe(acces, lecture).await?;

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
