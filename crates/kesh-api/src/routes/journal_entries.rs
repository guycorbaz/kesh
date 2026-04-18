//! Routes HTTP pour les écritures comptables en partie double.
//!
//! - `GET /api/v1/journal-entries` — liste des 50 dernières écritures
//!   (authenticated_routes, tout rôle incluant Consultation).
//! - `POST /api/v1/journal-entries` — création atomique d'une écriture
//!   (comptable_routes, Admin + Comptable).
//! - `PUT /api/v1/journal-entries/{id}` — modification avec OL (story 3.3).
//! - `DELETE /api/v1/journal-entries/{id}` — suppression avec audit (story 3.3).

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_core::accounting::{
    self, Journal as CoreJournal, JournalEntryDraft, JournalEntryLineDraft,
};
use kesh_core::errors::CoreError;
use kesh_core::listing::{SortBy, SortDirection};
use kesh_core::types::Money;
use kesh_db::entities::{
    Journal as DbJournal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
    NewJournalEntryLine,
};
use kesh_db::repositories::journal_entries::{JournalEntryListQuery, JournalEntryListResult};
use kesh_db::repositories::{fiscal_years, journal_entries};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

/// Défaut et plafond pour la pagination de la liste des écritures.
const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

fn default_limit() -> i64 {
    DEFAULT_LIMIT
}

fn default_offset() -> i64 {
    0
}

fn default_sort_by() -> SortBy {
    SortBy::default()
}

fn default_sort_dir() -> SortDirection {
    SortDirection::default()
}

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJournalEntryLineRequest {
    pub account_id: i64,
    /// Montant au débit, format string décimal (ex: "100.00"). Parsé
    /// via `Decimal::from_str` — rejet 400 si format invalide.
    pub debit: String,
    /// Montant au crédit, format string décimal.
    pub credit: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub lines: Vec<CreateJournalEntryLineRequest>,
}

/// Payload de modification d'une écriture existante — inclut `version`
/// pour le verrouillage optimiste.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub version: i32,
    pub lines: Vec<CreateJournalEntryLineRequest>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryLineResponse {
    pub id: i64,
    pub account_id: i64,
    pub line_order: i32,
    /// Stringifié pour éviter les erreurs d'arrondi JSON (JavaScript
    /// ne supporte que les f64).
    pub debit: String,
    pub credit: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryResponse {
    pub id: i64,
    pub company_id: i64,
    pub fiscal_year_id: i64,
    pub entry_number: i64,
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub version: i32,
    pub lines: Vec<JournalEntryLineResponse>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<JournalEntryLine> for JournalEntryLineResponse {
    fn from(l: JournalEntryLine) -> Self {
        Self {
            id: l.id,
            account_id: l.account_id,
            line_order: l.line_order,
            debit: l.debit.to_string(),
            credit: l.credit.to_string(),
        }
    }
}

fn convert_entry(entry: JournalEntry, lines: Vec<JournalEntryLine>) -> JournalEntryResponse {
    JournalEntryResponse {
        id: entry.id,
        company_id: entry.company_id,
        fiscal_year_id: entry.fiscal_year_id,
        entry_number: entry.entry_number,
        entry_date: entry.entry_date,
        journal: entry.journal.into(),
        description: entry.description,
        version: entry.version,
        lines: lines
            .into_iter()
            .map(JournalEntryLineResponse::from)
            .collect(),
        created_at: entry.created_at,
        updated_at: entry.updated_at,
    }
}

impl From<JournalEntryWithLines> for JournalEntryResponse {
    fn from(w: JournalEntryWithLines) -> Self {
        convert_entry(w.entry, w.lines)
    }
}

// ---------------------------------------------------------------------------
// CoreError → AppError mapping
// ---------------------------------------------------------------------------

fn map_core_error(err: CoreError) -> AppError {
    match err {
        CoreError::EntryUnbalanced { debit, credit } => AppError::EntryUnbalanced {
            debit: debit.to_string(),
            credit: credit.to_string(),
        },
        CoreError::EntryNeedsTwoLines => {
            AppError::Validation("Écriture invalide : au moins deux lignes requises".into())
        }
        CoreError::EntryDescriptionEmpty => {
            AppError::Validation("Écriture invalide : le libellé est obligatoire".into())
        }
        CoreError::EntryNegativeAmount => AppError::Validation(
            "Écriture invalide : montant négatif non permis en saisie directe".into(),
        ),
        CoreError::EntryLineDebitCreditExclusive => AppError::Validation(
            "Écriture invalide : chaque ligne doit avoir soit un débit soit un crédit (exclusif)"
                .into(),
        ),
        CoreError::EntryZeroTotal => {
            AppError::Validation("Écriture invalide : le total ne peut pas être nul".into())
        }
        // Variantes non-écriture — fallback sur Validation générique.
        other => AppError::Validation(format!("Erreur métier : {other}")),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// DTO de query params pour `GET /api/v1/journal-entries`.
///
/// Tous les champs sont optionnels. Les dates sont reçues en string
/// (format ISO `YYYY-MM-DD`) et parsées au niveau du handler pour
/// permettre une erreur de validation explicite si le format est invalide.
/// Les montants sont reçus en string décimale et parsés via
/// `Decimal::from_str`.
///
/// **Note sérialisation** : les noms de champs sont en `camelCase` via
/// `rename_all`, mais les variants des enums imbriqués (`SortBy`,
/// `SortDirection`, `Journal`) restent en PascalCase par défaut Rust/serde
/// — cohérent avec `Journal` story 3.2 et documenté dans la spec story 3.4.
#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListJournalEntriesQuery {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub amount_min: Option<String>,
    #[serde(default)]
    pub amount_max: Option<String>,
    #[serde(default)]
    pub date_from: Option<String>,
    #[serde(default)]
    pub date_to: Option<String>,
    #[serde(default)]
    pub journal: Option<CoreJournal>,
    #[serde(default = "default_sort_by")]
    pub sort_by: SortBy,
    #[serde(default = "default_sort_dir")]
    pub sort_dir: SortDirection,
    #[serde(default = "default_offset")]
    pub offset: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

/// GET /api/v1/journal-entries — liste paginée avec filtres et tri.
///
/// Query params : `description`, `amountMin`, `amountMax`, `dateFrom`,
/// `dateTo`, `journal`, `sortBy`, `sortDir`, `offset`, `limit`. Tous
/// optionnels. Voir `ListJournalEntriesQuery`.
///
/// Retour : envelope `ListResponse<JournalEntryResponse>` avec
/// `{ items, total, offset, limit }`.
///
/// Comportement par défaut d'Axum sur `Query<T>` : en cas d'erreur de
/// désérialisation (ex: `sortBy=invalid`), Axum retourne un 400 Bad
/// Request avec un corps texte non-JSON. **Intentionnel pour v0.1**
/// (Option A story 3.4) — refactor en extractor custom possible
/// post-MVP si besoin d'un format d'erreur cohérent.
pub async fn list_journal_entries(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListJournalEntriesQuery>,
) -> Result<Json<ListResponse<JournalEntryResponse>>, AppError> {
    // Clamp canonique — source de vérité (le repository a un garde-fou
    // défensif mais ne remonte pas d'erreur).
    let clamped_limit = params.limit.clamp(1, MAX_LIMIT);
    let clamped_offset = params.offset.max(0);

    // Parse des dates optionnelles.
    let date_from = match params.date_from.as_deref() {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::from_str(s)
                .map_err(|e| AppError::Validation(format!("dateFrom invalide ({e})")))?,
        ),
        _ => None,
    };
    let date_to = match params.date_to.as_deref() {
        Some(s) if !s.is_empty() => Some(
            NaiveDate::from_str(s)
                .map_err(|e| AppError::Validation(format!("dateTo invalide ({e})")))?,
        ),
        _ => None,
    };

    // Parse des montants optionnels.
    let amount_min = match params.amount_min.as_deref() {
        Some(s) if !s.is_empty() => Some(
            Decimal::from_str(s)
                .map_err(|e| AppError::Validation(format!("amountMin invalide ({e})")))?,
        ),
        _ => None,
    };
    let amount_max = match params.amount_max.as_deref() {
        Some(s) if !s.is_empty() => Some(
            Decimal::from_str(s)
                .map_err(|e| AppError::Validation(format!("amountMax invalide ({e})")))?,
        ),
        _ => None,
    };

    // P6 : rejeter les montants négatifs (évite BETWEEN -100 AND max
    // qui ne filtre rien, ou BETWEEN 0 AND -100 qui renvoie toujours vide).
    if let Some(min) = amount_min {
        if min < Decimal::ZERO {
            return Err(AppError::Validation(
                "amountMin ne peut pas être négatif".into(),
            ));
        }
    }
    if let Some(max) = amount_max {
        if max < Decimal::ZERO {
            return Err(AppError::Validation(
                "amountMax ne peut pas être négatif".into(),
            ));
        }
    }

    // P2 : cross-validation des bornes. Un filtre min > max retournerait
    // 0 résultats silencieusement — l'utilisateur croirait à une absence
    // de données au lieu d'un filtre incohérent. Mieux vaut rejeter.
    if let (Some(min), Some(max)) = (amount_min, amount_max) {
        if min > max {
            return Err(AppError::Validation(
                "amountMin doit être inférieur ou égal à amountMax".into(),
            ));
        }
    }
    if let (Some(from), Some(to)) = (date_from, date_to) {
        if from > to {
            return Err(AppError::Validation(
                "dateFrom doit être inférieur ou égal à dateTo".into(),
            ));
        }
    }

    // Trim description (garde-fou cohérent avec create/update).
    let description = params
        .description
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let query = JournalEntryListQuery {
        description,
        amount_min,
        amount_max,
        date_from,
        date_to,
        journal: params.journal.map(DbJournal::from),
        sort_by: params.sort_by,
        sort_dir: params.sort_dir,
        limit: clamped_limit,
        offset: clamped_offset,
    };

    let result: JournalEntryListResult =
        journal_entries::list_by_company_paginated(&state.pool, current_user.company_id, query).await?;

    Ok(Json(ListResponse {
        items: result
            .items
            .into_iter()
            .map(JournalEntryResponse::from)
            .collect(),
        total: result.total,
        offset: result.offset,
        limit: result.limit,
    }))
}

/// Limite haute sur le nombre de lignes par écriture (garde-fou DoS).
/// Largement au-dessus des usages réels (une écriture standard a 2-10
/// lignes) mais empêche un client abusif de soumettre 100k lignes dans
/// une seule transaction.
const MAX_LINES_PER_ENTRY: usize = 500;

/// Longueur maximale du libellé (alignée sur la colonne `description VARCHAR(500)`).
const MAX_DESCRIPTION_LEN: usize = 500;

/// POST /api/v1/journal-entries — crée une écriture en partie double.
pub async fn create_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateJournalEntryRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // P5 : trim du libellé dès l'entrée — unique source de vérité.
    let trimmed_description = req.description.trim().to_string();

    // P6 : validation longueur libellé avant tout appel DB.
    if trimmed_description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "libellé trop long (max {MAX_DESCRIPTION_LEN} caractères)"
        )));
    }

    // P7 : borne haute sur le nombre de lignes (vecteur DoS).
    if req.lines.len() > MAX_LINES_PER_ENTRY {
        return Err(AppError::Validation(format!(
            "trop de lignes dans l'écriture (max {MAX_LINES_PER_ENTRY})"
        )));
    }

    // Parse des montants (string → Decimal). Rejet 400 si format invalide.
    let mut line_drafts: Vec<JournalEntryLineDraft> = Vec::with_capacity(req.lines.len());
    for (idx, line) in req.lines.iter().enumerate() {
        let debit = Decimal::from_str(&line.debit).map_err(|e| {
            AppError::Validation(format!("ligne {}: débit invalide ({e})", idx + 1))
        })?;
        let credit = Decimal::from_str(&line.credit).map_err(|e| {
            AppError::Validation(format!("ligne {}: crédit invalide ({e})", idx + 1))
        })?;
        line_drafts.push(JournalEntryLineDraft {
            account_id: line.account_id,
            debit: Money::new(debit),
            credit: Money::new(credit),
        });
    }

    // Pré-check exercice couvrant la date (distingue NO_FISCAL_YEAR
    // et FISCAL_YEAR_CLOSED pour l'UX).
    let covering =
        fiscal_years::find_covering_date(&state.pool, company.id, req.entry_date).await?;
    let fiscal_year = match covering {
        None => {
            return Err(AppError::NoFiscalYear {
                date: req.entry_date.to_string(),
            });
        }
        Some(fy) if fy.status == kesh_db::entities::FiscalYearStatus::Closed => {
            return Err(AppError::FiscalYearClosed {
                date: req.entry_date.to_string(),
            });
        }
        Some(fy) => fy,
    };

    // Garde-fou #1 : validation métier pure (kesh-core).
    let draft = JournalEntryDraft {
        date: req.entry_date,
        journal: req.journal,
        description: trimmed_description.clone(),
        lines: line_drafts,
    };
    // P4 : on récupère le BalancedEntry validé et on l'utilise pour
    // construire le NewJournalEntry, éliminant la duplication fragile
    // entre line_drafts et line_decimals (ex-security theater).
    let balanced = accounting::validate(draft).map_err(map_core_error)?;
    let validated = balanced.into_draft();

    // Construction du NewJournalEntry pour kesh-db depuis les données
    // garanties équilibrées par `validate()`.
    let new = NewJournalEntry {
        company_id: company.id,
        entry_date: validated.date,
        journal: DbJournal::from(validated.journal),
        description: validated.description,
        lines: validated
            .lines
            .into_iter()
            .map(|l| NewJournalEntryLine {
                account_id: l.account_id,
                debit: l.debit.amount(),
                credit: l.credit.amount(),
            })
            .collect(),
    };

    // Création atomique (re-lock FY + numérotation + INSERT + balance check).
    // P2 : mapping stable via variants DbError dédiés (plus de matching sur
    // le contenu du message).
    let result = journal_entries::create(&state.pool, fiscal_year.id, current_user.user_id, new)
        .await
        .map_err(|e| match e {
            // Race condition : clôture concurrente après le pré-check.
            kesh_db::errors::DbError::FiscalYearClosed => AppError::FiscalYearClosed {
                date: req.entry_date.to_string(),
            },
            other => AppError::from(other),
        })?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(result)),
    ))
}

/// PUT /api/v1/journal-entries/{id} — modifie une écriture existante
/// (verrouillage optimiste, FR24 immutabilité post-clôture).
pub async fn update_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateJournalEntryRequest>,
) -> Result<Json<JournalEntryResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let trimmed_description = req.description.trim().to_string();

    if trimmed_description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "libellé trop long (max {MAX_DESCRIPTION_LEN} caractères)"
        )));
    }

    if req.lines.len() > MAX_LINES_PER_ENTRY {
        return Err(AppError::Validation(format!(
            "trop de lignes dans l'écriture (max {MAX_LINES_PER_ENTRY})"
        )));
    }

    // Parse des montants.
    let mut line_drafts: Vec<JournalEntryLineDraft> = Vec::with_capacity(req.lines.len());
    for (idx, line) in req.lines.iter().enumerate() {
        let debit = Decimal::from_str(&line.debit).map_err(|e| {
            AppError::Validation(format!("ligne {}: débit invalide ({e})", idx + 1))
        })?;
        let credit = Decimal::from_str(&line.credit).map_err(|e| {
            AppError::Validation(format!("ligne {}: crédit invalide ({e})", idx + 1))
        })?;
        line_drafts.push(JournalEntryLineDraft {
            account_id: line.account_id,
            debit: Money::new(debit),
            credit: Money::new(credit),
        });
    }

    // Pré-check FY lock-free (distingue NO_FISCAL_YEAR et FISCAL_YEAR_CLOSED
    // pour l'UX). La vérification fine « date dans l'exercice courant de
    // l'entry » est faite DANS la tx du repository (M4 anti-TOCTOU).
    let covering =
        fiscal_years::find_covering_date(&state.pool, company.id, req.entry_date).await?;
    match covering {
        None => {
            return Err(AppError::NoFiscalYear {
                date: req.entry_date.to_string(),
            });
        }
        Some(fy) if fy.status == kesh_db::entities::FiscalYearStatus::Closed => {
            return Err(AppError::FiscalYearClosed {
                date: req.entry_date.to_string(),
            });
        }
        Some(_) => {}
    }

    // Validation métier (garde-fou #1 partie double).
    let draft = JournalEntryDraft {
        date: req.entry_date,
        journal: req.journal,
        description: trimmed_description,
        lines: line_drafts,
    };
    let balanced = accounting::validate(draft).map_err(map_core_error)?;
    let validated = balanced.into_draft();

    let new = NewJournalEntry {
        company_id: company.id,
        entry_date: validated.date,
        journal: DbJournal::from(validated.journal),
        description: validated.description,
        lines: validated
            .lines
            .into_iter()
            .map(|l| NewJournalEntryLine {
                account_id: l.account_id,
                debit: l.debit.amount(),
                credit: l.credit.amount(),
            })
            .collect(),
    };

    let result = journal_entries::update(
        &state.pool,
        company.id,
        id,
        req.version,
        current_user.user_id,
        new,
    )
    .await
    .map_err(|e| match e {
        // Race : clôture concurrente après le pré-check → message contextuel.
        kesh_db::errors::DbError::FiscalYearClosed => AppError::FiscalYearClosed {
            date: req.entry_date.to_string(),
        },
        // TOCTOU cross-exercice dans la tx → message contextuel.
        kesh_db::errors::DbError::DateOutsideFiscalYear => AppError::DateOutsideFiscalYear {
            date: req.entry_date.to_string(),
        },
        other => AppError::from(other),
    })?;

    Ok(Json(JournalEntryResponse::from(result)))
}

/// DELETE /api/v1/journal-entries/{id} — supprime une écriture avec
/// enregistrement audit atomique. Refusé si l'exercice est clos.
pub async fn delete_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Propagation directe via `?` : `DbError::FiscalYearClosed` est mappé
    // par le match exhaustif dans `errors.rs` vers 400 avec le message
    // générique i18n (asymétrie volontaire avec UPDATE qui a la date de
    // la requête, cf. story 3.3 §Décisions H3).
    journal_entries::delete_by_id(&state.pool, company.id, id, current_user.user_id).await?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::response::IntoResponse;
    use http_body_util::BodyExt;

    async fn body_json(resp: axum::response::Response) -> (StatusCode, serde_json::Value) {
        let (parts, body) = resp.into_parts();
        let bytes = body.collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        (parts.status, json)
    }

    #[tokio::test]
    async fn entry_unbalanced_maps_to_400() {
        let resp = AppError::EntryUnbalanced {
            debit: "100.00".into(),
            credit: "80.00".into(),
        }
        .into_response();
        let (status, body) = body_json(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "ENTRY_UNBALANCED");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("100.00"));
        assert!(msg.contains("80.00"));
        assert!(msg.contains("déséquilibrée"));
    }

    #[tokio::test]
    async fn no_fiscal_year_maps_to_400() {
        let resp = AppError::NoFiscalYear {
            date: "2030-01-15".into(),
        }
        .into_response();
        let (status, body) = body_json(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "NO_FISCAL_YEAR");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("2030-01-15"));
    }

    #[tokio::test]
    async fn fiscal_year_closed_maps_to_400() {
        let resp = AppError::FiscalYearClosed {
            date: "2025-06-30".into(),
        }
        .into_response();
        let (status, body) = body_json(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "FISCAL_YEAR_CLOSED");
        let msg = body["error"]["message"].as_str().unwrap();
        assert!(msg.contains("2025-06-30"));
        assert!(msg.contains("CO art. 957-964"));
    }
}
