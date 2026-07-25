//! Routes HTTP du bilan d'ouverture — saisie des soldes de départ (Story 14-4).
//!
//! - `GET  /api/v1/opening-balances/status` — état de l'écran (D6) : premier
//!   exercice + `canEnter` + `reason` (`READY` / `NO_FISCAL_YEAR` /
//!   `FIRST_YEAR_CLOSED` / `ALREADY_HAS_ENTRIES`).
//! - `POST /api/v1/opening-balances` — génère l'écriture d'ouverture (une OD
//!   équilibrée datée au `start_date` du premier exercice).
//!
//! Les deux routes sont montées dans `comptable_routes`
//! (`require_comptable_role`) — Consultation → 403, non-auth → 401. **PAS**
//! d'`ensure_not_pat` (P3-BH3-4) : l'endpoint est au niveau Comptable, comme la
//! création d'écriture normale — une clé PAT `read-write` peut l'appeler (une
//! clé `read` est rejetée en amont par le middleware `ApiKeyReadOnly`).
//!
//! Contrat d'erreur du POST (AC-B, pattern D7 de 14-2) :
//! - company non-vierge → **409** `ILLEGAL_STATE_TRANSITION` (code machine
//!   partagé) + message distinct `error-opening-balances-already-has-entries` ;
//! - aucun exercice / premier exercice clos / compte de résultat / < 2 lignes /
//!   montant négatif → **400** `VALIDATION_ERROR` (messages distincts) ;
//! - déséquilibre → **400** `ENTRY_UNBALANCED` ;
//! - compte inexistant / archivé / non-postable / cross-tenant → **400**
//!   `INACTIVE_OR_INVALID_ACCOUNTS` (garde `journal_entries` existante).

use std::str::FromStr;

use axum::extract::State;
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_core::accounting::{
    self, Journal as CoreJournal, JournalEntryDraft, JournalEntryLineDraft,
};
use kesh_core::types::Money;
use kesh_db::entities::{
    FiscalYearStatus, Journal as DbJournal, NewJournalEntry, NewJournalEntryLine,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::{
    FY_OPENING_ALREADY_HAS_ENTRIES_KEY, FY_OPENING_FIRST_YEAR_CLOSED_KEY,
};
use kesh_db::repositories::{accounts, fiscal_years, journal_entries};
use kesh_i18n::Locale;

use crate::AppState;
use crate::errors::{AppError, t};
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::journal_entries::{JournalEntryResponse, MAX_LINES_PER_ENTRY, map_core_error};

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

/// Ligne de soldes de départ. Montants en **string décimale** (jamais des
/// nombres JSON — CO 957-964, parse `Decimal::from_str` miroir
/// `create_journal_entry`). Aucun libellé de ligne ni TVA (P3-BH3-3 : l'entité
/// ligne d'écriture n'a ni l'un ni l'autre) ; aucun `project_id` (une écriture
/// d'ouverture n'est pas analytique — et son absence garantit que
/// `create_in_tx` ne verrouille QUE `fiscal_years`, cf. note atomicité D5).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningBalanceLineRequest {
    pub account_id: i64,
    pub debit: String,
    pub credit: String,
}

/// Body de `POST /api/v1/opening-balances`. `journal` et `entry_date` sont
/// **absents à dessein** : forcés serveur (`OD` + `start_date` du premier
/// exercice) — anti-injection (D5).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningBalancesRequest {
    pub lines: Vec<OpeningBalanceLineRequest>,
}

/// Résumé du premier exercice pour l'écran (D6).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningBalancesFiscalYear {
    pub id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    /// `"Open"` ou `"Closed"` (PascalCase, cohérent avec l'enum DB).
    pub status: FiscalYearStatus,
}

/// Réponse de `GET /api/v1/opening-balances/status` (D6).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningBalancesStatusResponse {
    /// Premier exercice de la company (`start_date` ASC), `null` si aucun.
    pub fiscal_year: Option<OpeningBalancesFiscalYear>,
    /// `true` ssi premier exercice existe + `Open` + company vierge.
    pub can_enter: bool,
    /// `READY` / `NO_FISCAL_YEAR` / `FIRST_YEAR_CLOSED` / `ALREADY_HAS_ENTRIES`.
    pub reason: &'static str,
}

// ---------------------------------------------------------------------------
// Mapping erreurs DB → AppError
// ---------------------------------------------------------------------------

/// Mapping des erreurs de `create_opening_entry` (miroir `map_reopen_error`
/// 14-2, décision D7).
///
/// - `ALREADY_HAS_ENTRIES` → `AppError::IllegalState` (**409**, code machine
///   partagé `ILLEGAL_STATE_TRANSITION`, message distinct localisé).
/// - `FIRST_YEAR_CLOSED` → `AppError::Validation` (**400**) — **même** outcome
///   que le pré-check handler hors-lock (P3-AA-2 : code + message identiques
///   quel que soit le timing ; PAS le variant `AppError::FiscalYearClosed` qui
///   divergerait en `FISCAL_YEAR_CLOSED`).
///
/// Toute autre erreur retombe vers le mapping global (`INACTIVE_OR_INVALID_ACCOUNTS`,
/// `NotFound` → 404, …).
fn map_opening_balances_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_OPENING_ALREADY_HAS_ENTRIES_KEY => {
            AppError::IllegalState(t(
                "error-opening-balances-already-has-entries",
                "La société contient déjà des écritures : le bilan d'ouverture ne peut plus être généré. Corrigez l'écriture d'ouverture via le journal.",
            ))
        }
        DbError::Invariant(ref s) if s == FY_OPENING_FIRST_YEAR_CLOSED_KEY => {
            AppError::Validation(t(
                "error-opening-balances-first-year-closed",
                "Le premier exercice est clôturé : rouvrez-le avant de saisir les soldes de départ.",
            ))
        }
        other => AppError::from(other),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/opening-balances/status` — état de l'écran « Soldes de
/// départ » (D6). Comptable+ (monté `comptable_routes` — PAS
/// `authenticated_routes`, qui laisserait passer Consultation, P1-M2-ECH).
///
/// Priorité d'évaluation : `NO_FISCAL_YEAR` > `FIRST_YEAR_CLOSED` >
/// `ALREADY_HAS_ENTRIES` > `READY`.
///
/// Lock-free (pré-check UX) : l'autorité reste `create_opening_entry` qui
/// re-vérifie statut + count **sous** le `FOR UPDATE` au POST.
pub async fn opening_balances_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OpeningBalancesStatusResponse>, AppError> {
    let first = fiscal_years::find_first_by_company(&state.pool, current_user.company_id).await?;

    let Some(fy) = first else {
        return Ok(Json(OpeningBalancesStatusResponse {
            fiscal_year: None,
            can_enter: false,
            reason: "NO_FISCAL_YEAR",
        }));
    };

    let closed = fy.status == FiscalYearStatus::Closed;
    let summary = OpeningBalancesFiscalYear {
        id: fy.id,
        name: fy.name,
        start_date: fy.start_date,
        status: fy.status,
    };

    if closed {
        return Ok(Json(OpeningBalancesStatusResponse {
            fiscal_year: Some(summary),
            can_enter: false,
            reason: "FIRST_YEAR_CLOSED",
        }));
    }

    // Garde « company vierge » (P3-BH3-1) : ≥1 écriture dans N'IMPORTE QUEL
    // exercice verrouille l'écran.
    let count = journal_entries::count_by_company(&state.pool, current_user.company_id).await?;
    if count > 0 {
        return Ok(Json(OpeningBalancesStatusResponse {
            fiscal_year: Some(summary),
            can_enter: false,
            reason: "ALREADY_HAS_ENTRIES",
        }));
    }

    Ok(Json(OpeningBalancesStatusResponse {
        fiscal_year: Some(summary),
        can_enter: true,
        reason: "READY",
    }))
}

/// `POST /api/v1/opening-balances` — génère l'écriture d'ouverture (Comptable+).
///
/// Le handler force `journal = OD` et `entry_date = start_date` du premier
/// exercice (jamais fournis par le client, D5). La description est rendue dans
/// la **langue comptable de la company** (`Locale::from(company.accounting_language)`,
/// P1-H1 — champ persistant immuable, PAS la locale serveur globale de
/// `errors::t()`).
///
/// Gardes (dans l'ordre) :
/// 1. Pré-checks handler lock-free : premier exercice existe (`400
///    no-fiscal-year`) + statut `Open` (`400 first-year-closed`, même outcome
///    que le re-check sous lock, P3-AA-2) ;
/// 2. Garde « comptes de bilan » (D4, défense en profondeur) : toute ligne dont
///    le type **retourné** est `Revenue`/`Expense` → `400
///    non-balance-account` ; un id absent (inexistant / autre company) retombe
///    dans `create_in_tx` → `INACTIVE_OR_INVALID_ACCOUNTS` (P3-AA-1/BH3-6) ;
/// 3. `accounting::validate` (équilibre / ≥2 lignes / montants ≥ 0 /
///    débit⊕crédit) via `map_core_error` (DRY) ;
/// 4. `create_opening_entry` : statut + garde « company vierge » **sous** le
///    `fiscal_years FOR UPDATE` (autorité anti-course, P1-C1/P3-BH3-1).
pub async fn generate_opening_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<OpeningBalancesRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Borne haute défensive sur le nombre de lignes (miroir
    // `create_journal_entry` — vecteur DoS).
    if req.lines.len() > MAX_LINES_PER_ENTRY {
        return Err(AppError::Validation(format!(
            "trop de lignes dans l'écriture (max {MAX_LINES_PER_ENTRY})"
        )));
    }

    // Parse des montants (string → Decimal), miroir `create_journal_entry`.
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
            project_id: None,
        });
    }

    // Pré-check 1 : premier exercice (lock-free, UX). L'autorité anti-course
    // est le re-check sous lock dans `create_opening_entry`.
    let Some(fiscal_year) = fiscal_years::find_first_by_company(&state.pool, company.id).await?
    else {
        return Err(AppError::Validation(t(
            "error-opening-balances-no-fiscal-year",
            "Aucun exercice comptable : créez d'abord un exercice avant de saisir les soldes de départ.",
        )));
    };

    // Pré-check 2 : statut Open — MÊME outcome que le chemin sous-lock
    // (`AppError::Validation`, même clé — P3-AA-2, PAS `AppError::FiscalYearClosed`).
    if fiscal_year.status == FiscalYearStatus::Closed {
        return Err(AppError::Validation(t(
            "error-opening-balances-first-year-closed",
            "Le premier exercice est clôturé : rouvrez-le avant de saisir les soldes de départ.",
        )));
    }

    // Garde « comptes de bilan » (D4) : rejet UNIQUEMENT sur un type retourné
    // Revenue/Expense (fausserait le P&L de l'exercice courant). Les ids
    // absents retombent dans `create_in_tx` → INACTIVE_OR_INVALID_ACCOUNTS.
    let account_ids: Vec<i64> = req.lines.iter().map(|l| l.account_id).collect();
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    let types = accounts::find_types_by_ids_in_tx(&mut tx, company.id, &account_ids).await?;
    // Lecture seule : rollback best-effort (le drop-guard SQLx couvre l'échec).
    let _ = tx.rollback().await;
    if types
        .iter()
        .any(|(_, ty)| ty == "Revenue" || ty == "Expense")
    {
        return Err(AppError::Validation(t(
            "error-opening-balances-non-balance-account",
            "Le bilan d'ouverture ne peut toucher que des comptes de bilan (actifs et passifs) — retirez les comptes de produits et de charges.",
        )));
    }

    // Description rendue dans la langue COMPTABLE de la company (P1-H1) —
    // champ persistant immuable, miroir `routes/invoices.rs` (descriptions
    // d'écritures de facturation).
    let locale = Locale::from(company.accounting_language.as_str());
    let description = state
        .i18n
        .format(&locale, "opening-balances-entry-description", None);

    // Garde-fou #1 : validation métier pure (équilibre / ≥2 lignes / montants
    // ≥ 0 / débit⊕crédit) — miroir `create_journal_entry` + `map_core_error`.
    let draft = JournalEntryDraft {
        date: fiscal_year.start_date,
        journal: CoreJournal::OD,
        description,
        lines: line_drafts,
    };
    let balanced = accounting::validate(draft).map_err(map_core_error)?;
    let validated = balanced.into_draft();

    let new = NewJournalEntry {
        company_id: company.id,
        // Forcé serveur : 1er jour du premier exercice (D5).
        entry_date: validated.date,
        journal: DbJournal::from(validated.journal),
        description: validated.description,
        project_id: None,
        lines: validated
            .lines
            .into_iter()
            .map(|l| NewJournalEntryLine {
                account_id: l.account_id,
                debit: l.debit.amount(),
                credit: l.credit.amount(),
                project_id: None,
            })
            .collect(),
    };

    // Création atomique dédiée : statut + « company vierge » sous le
    // `fiscal_years FOR UPDATE` (P1-C1) — PAS le wrapper `create()`.
    let result = journal_entries::create_opening_entry(
        &state.pool,
        company.id,
        fiscal_year.id,
        current_user.user_id,
        new,
    )
    .await
    .map_err(map_opening_balances_error)?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(result)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `ALREADY_HAS_ENTRIES` → 409 code partagé + message distinct (D7).
    #[test]
    fn map_already_has_entries_is_illegal_state() {
        let err = map_opening_balances_error(DbError::Invariant(
            FY_OPENING_ALREADY_HAS_ENTRIES_KEY.to_string(),
        ));
        match err {
            AppError::IllegalState(msg) => {
                assert!(!msg.is_empty());
            }
            other => panic!("attendu IllegalState, obtenu {other:?}"),
        }
    }

    /// `FIRST_YEAR_CLOSED` → 400 Validation (même outcome que le pré-check
    /// handler, P3-AA-2 — PAS `AppError::FiscalYearClosed`).
    #[test]
    fn map_first_year_closed_is_validation() {
        let err = map_opening_balances_error(DbError::Invariant(
            FY_OPENING_FIRST_YEAR_CLOSED_KEY.to_string(),
        ));
        match err {
            AppError::Validation(msg) => {
                assert!(!msg.is_empty());
            }
            other => panic!("attendu Validation, obtenu {other:?}"),
        }
    }

    /// Toute autre `Invariant` retombe vers le mapping global (500 défensif).
    #[test]
    fn map_other_invariant_falls_through() {
        let err = map_opening_balances_error(DbError::Invariant("autre:clef".to_string()));
        assert!(matches!(err, AppError::Database(DbError::Invariant(_))));
    }
}
