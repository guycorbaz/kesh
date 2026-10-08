//! Routes HTTP du bilan d'ouverture — saisie des soldes de départ (Story 14-4).
//!
//! - `GET  /api/v1/opening-balances/status` — état de l'écran (D6) : premier
//!   exercice + `canEnter` + `reason` (`READY` / `NO_FISCAL_YEAR` /
//!   `FIRST_YEAR_CLOSED` / `ALREADY_HAS_ENTRIES`).
//! - `POST /api/v1/opening-balances` — génère l'écriture d'ouverture (une OD
//!   équilibrée datée au `start_date` du premier exercice).
//! - `POST /api/v1/opening-balances/complete` — **complète** un compte de bilan
//!   oublié, une fois l'ouverture générée (Story 25-7, #445) : écriture
//!   d'ajustement dont la contrepartie est portée au compte de report, calculée
//!   par le serveur (`kesh_db::repositories::opening_complement`).
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
//! - compte inexistant / archivé / cross-tenant → **400**
//!   `INACTIVE_OR_INVALID_ACCOUNTS` (garde `journal_entries` existante) ;
//! - compte de la société, actif, mais non imputable → **400**
//!   `ACCOUNT_NOT_POSTABLE`, avec `details.rejected[{accountId, accountNumber}]`
//!   (même garde, Story 15-5a).

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
use kesh_db::repositories::opening_complement::{self, ComplementLine, ComplementStatus};
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
    /// Mode « compléter » (Story 25-7, AC 3) : vrai seulement si
    /// `complete_reason = READY`.
    pub can_complete: bool,
    /// `READY` / `NO_ENTRIES` / `NO_OPEN_FISCAL_YEAR` / `DATE_LOCKED` /
    /// `NO_RETAINED_EARNINGS` / `RETAINED_EARNINGS_NOT_POSTABLE` /
    /// `NO_COMPLETABLE_ACCOUNT`.
    pub complete_reason: &'static str,
    /// Comptes de bilan actifs, imputables, jamais mouvementés, hors compte de
    /// report.
    pub completable_accounts: Vec<CompletableAccountDto>,
    /// Date prévue du complément — **indicative** : le POST la recalcule sous
    /// verrou.
    pub complement_date: Option<NaiveDate>,
    /// `OPENING_DAY` (premier jour du premier exercice) ou `TODAY`
    /// (régularisation datée du jour).
    pub complement_date_kind: Option<&'static str>,
    /// Exercice qui porterait le complément.
    pub complement_fiscal_year: Option<ComplementFiscalYearDto>,
    /// Compte de report (rôle `RetainedEarnings`), quand il existe.
    pub retained_earnings_account: Option<RetainedEarningsAccountDto>,
}

/// Compte proposé au complément.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletableAccountDto {
    pub id: i64,
    pub number: String,
    pub name: String,
    pub account_type: String,
}

/// Exercice qui porterait le complément.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComplementFiscalYearDto {
    pub id: i64,
    pub name: String,
}

/// Compte de report.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RetainedEarningsAccountDto {
    pub id: i64,
    pub number: String,
    pub name: String,
}

/// Body de `POST /api/v1/opening-balances/complete` — les seuls comptes
/// oubliés ; la contrepartie est calculée par le serveur.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningComplementRequest {
    pub lines: Vec<OpeningComplementLineRequest>,
}

/// Ligne de complément : une seule colonne non nulle ; l'autre vaut `"0"`, est
/// vide ou absente.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpeningComplementLineRequest {
    pub account_id: i64,
    #[serde(default)]
    pub debit: Option<String>,
    #[serde(default)]
    pub credit: Option<String>,
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
/// `ACCOUNT_NOT_POSTABLE`, `NotFound` → 404, …).
fn map_opening_balances_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_OPENING_ALREADY_HAS_ENTRIES_KEY => {
            AppError::IllegalState(t(
                "error-opening-balances-already-has-entries",
                "La société contient déjà des écritures : le bilan d'ouverture ne peut plus être généré. Un compte oublié se complète depuis l'écran Soldes de départ.",
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
/// Priorité d'évaluation : `NO_FISCAL_YEAR` > `ALREADY_HAS_ENTRIES` >
/// `FIRST_YEAR_CLOSED` > `READY`.
///
/// **Amendement D6 (Pass 3 code review, ECH3-2)** : `ALREADY_HAS_ENTRIES`
/// prime désormais sur `FIRST_YEAR_CLOSED`. Avec l'ordre initial de la spec,
/// une société au premier exercice clos ET contenant des écritures (le cas
/// réel typique) lisait « un administrateur doit rouvrir l'exercice » —
/// prescription d'une opération réglementaire (Admin + motif + audit + garde
/// LIFO) qui ne pouvait PAS débloquer l'écran (le statut retombait ensuite sur
/// `ALREADY_HAS_ENTRIES`). Avec cet ordre, `FIRST_YEAR_CLOSED` ne s'affiche
/// que quand la réouverture mène réellement à `READY` (company vierge) — le
/// message reste vrai dans toutes les configurations. Testé par
/// `status_closed_with_entries_reports_already_has_entries`.
///
/// Lock-free (pré-check UX) : l'autorité reste `create_opening_entry` qui
/// re-vérifie statut + count **sous** le `FOR UPDATE` au POST.
pub async fn opening_balances_status(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<OpeningBalancesStatusResponse>, AppError> {
    let first = fiscal_years::find_first_by_company(&state.pool, current_user.company_id).await?;
    let complement = opening_complement::complement_status(
        &state.pool,
        current_user.company_id,
        chrono::Utc::now().date_naive(),
    )
    .await?;
    let respond = |fiscal_year, can_enter, reason| {
        Json(status_response(fiscal_year, can_enter, reason, &complement))
    };

    let Some(fy) = first else {
        return Ok(respond(None, false, "NO_FISCAL_YEAR"));
    };

    let summary = OpeningBalancesFiscalYear {
        id: fy.id,
        name: fy.name,
        start_date: fy.start_date,
        status: fy.status,
    };

    // Garde « company vierge » (P3-BH3-1) ÉVALUÉE AVANT le statut clos
    // (amendement D6, ECH3-2) : ≥1 écriture dans N'IMPORTE QUEL exercice
    // verrouille l'écran — rouvrir l'exercice n'y changerait rien, le message
    // `first-year-closed` serait un mauvais conseil.
    let count = journal_entries::count_by_company(&state.pool, current_user.company_id).await?;
    if count > 0 {
        return Ok(respond(Some(summary), false, "ALREADY_HAS_ENTRIES"));
    }

    if summary.status == FiscalYearStatus::Closed {
        return Ok(respond(Some(summary), false, "FIRST_YEAR_CLOSED"));
    }

    Ok(respond(Some(summary), true, "READY"))
}

/// Assemble la réponse du status : champs historiques + mode « compléter ».
fn status_response(
    fiscal_year: Option<OpeningBalancesFiscalYear>,
    can_enter: bool,
    reason: &'static str,
    complement: &ComplementStatus,
) -> OpeningBalancesStatusResponse {
    OpeningBalancesStatusResponse {
        fiscal_year,
        can_enter,
        reason,
        can_complete: complement.can_complete(),
        complete_reason: complement.complete_reason(),
        completable_accounts: complement
            .completable_accounts
            .iter()
            .map(|a| CompletableAccountDto {
                id: a.id,
                number: a.number.clone(),
                name: a.name.clone(),
                account_type: a.account_type.clone(),
            })
            .collect(),
        complement_date: complement.date.as_ref().map(|d| d.date),
        complement_date_kind: complement.date.as_ref().map(|d| d.branch.as_str()),
        complement_fiscal_year: complement.date.as_ref().map(|d| ComplementFiscalYearDto {
            id: d.fiscal_year_id,
            name: d.fiscal_year_name.clone(),
        }),
        retained_earnings_account: complement.retained_earnings.as_ref().map(|r| {
            RetainedEarningsAccountDto {
                id: r.id,
                number: r.number.clone(),
                name: r.name.clone(),
            }
        }),
    }
}

/// `POST /api/v1/opening-balances` — génère l'écriture d'ouverture (Comptable+).
///
/// Le handler force `journal = OD` et `entry_date = start_date` du premier
/// exercice (jamais fournis par le client, D5). La description est rendue dans
/// la **langue comptable de la company** (`Locale::from(company.accounting_language)`,
/// P1-H1 — champ persistant immuable, PAS la locale serveur globale de
/// `errors::t()`).
///
/// Gardes (dans l'ordre — priorité alignée sur `GET /status`, amendement
/// ECH3-2 propagé au POST en Pass 4) :
/// 1. Pré-checks handler lock-free : premier exercice existe (`400
///    no-fiscal-year`), PUIS company vierge (`409 already-has-entries` — prime
///    sur le statut clos, même verdict que `GET /status`), PUIS statut `Open`
///    (`400 first-year-closed`, même outcome que le re-check sous lock,
///    P3-AA-2) ;
/// 2. Garde « comptes de bilan » (D4, défense en profondeur) : toute ligne dont
///    le type **retourné** est `Revenue`/`Expense` → `400
///    non-balance-account` ; un id absent (inexistant / autre company) retombe
///    dans `create_in_tx` → `INACTIVE_OR_INVALID_ACCOUNTS` (P3-AA-1/BH3-6) ;
/// 3. `accounting::validate` (équilibre / ≥2 lignes / montants ≥ 0 /
///    débit⊕crédit) via `map_core_error` (DRY) ;
/// 4. `create_opening_entry` : sentinel `companies` + garde « company vierge »
///    puis statut, **sous** les `FOR UPDATE` (autorité anti-course,
///    P1-C1/P3-BH3-1/P4).
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`] ; le `NewJournalEntry` est cloné à
/// chaque tentative, et `map_opening_balances_error` s'applique après le rejeu.
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

    // Pré-check 2 (lock-free, priorité amendée ECH3-2 propagée au POST en
    // Pass 4 — BH4/ECH4 convergés) : company non-vierge PRIME sur exercice
    // clos, même ordre que `GET /status` et que le re-check sous lock du repo.
    // Sans ce pré-check, un POST direct sur « clos + écritures » rendait le
    // conseil trompeur « rouvrez l'exercice » (qui ne débloquerait rien).
    // MÊME outcome que le chemin sous-lock : 409 `IllegalState`, même clé.
    let count = journal_entries::count_by_company(&state.pool, company.id).await?;
    if count > 0 {
        return Err(AppError::IllegalState(t(
            "error-opening-balances-already-has-entries",
            "La société contient déjà des écritures : le bilan d'ouverture ne peut plus être généré. Un compte oublié se complète depuis l'écran Soldes de départ.",
        )));
    }

    // Pré-check 3 : statut Open — MÊME outcome que le chemin sous-lock
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
    let result = kesh_db::retry::retry_on_deadlock("opening_balances::generate", || {
        journal_entries::create_opening_entry(
            &state.pool,
            company.id,
            fiscal_year.id,
            current_user.user_id,
            new.clone(),
        )
    })
    .await
    .map_err(map_opening_balances_error)?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(result)),
    ))
}

/// Contrôle de forme refusé (AC 4, « dans le handler »).
fn complement_invalid(error_code: &'static str, key: &str, fallback: &str) -> AppError {
    AppError::OpeningComplementInvalid {
        error_code,
        message: t(key, fallback),
    }
}

/// Montant d'une colonne : absente, vide ou `"0"` → zéro ; sinon un décimal
/// strictement positif d'au plus quatre décimales (`DECIMAL(19,4)` : au-delà,
/// l'arrondi à l'écriture déséquilibrerait la contrepartie calculée avant lui).
fn invalid_amount() -> AppError {
    complement_invalid(
        "OPENING_COMPLEMENT_INVALID_AMOUNT",
        "error-opening-complement-invalid-amount",
        "Chaque ligne porte un montant strictement positif, au plus quatre décimales, au débit OU au crédit ; la contrepartie qui en résulte (débits moins crédits) ne peut dépasser 999’999’999’999’999.9999.",
    )
}

fn parse_complement_amount(raw: Option<&str>) -> Result<Decimal, AppError> {
    let invalid = invalid_amount;
    let raw = raw.map(str::trim).unwrap_or("");
    if raw.is_empty() {
        return Ok(Decimal::ZERO);
    }
    let value = Decimal::from_str(raw).map_err(|_| invalid())?;
    // Borne de `DECIMAL(19,4)` : quinze chiffres entiers. Au-delà, l'insertion
    // échouerait — et la somme des lignes pourrait dépasser la capacité de
    // `Decimal` avant même d'y arriver (revue de code P1, B-F3).
    let max = Decimal::from_str("999999999999999.9999").expect("borne constante");
    if value.is_sign_negative() || value.normalize().scale() > 4 || value > max {
        return Err(invalid());
    }
    Ok(value)
}

/// Contrôles de forme du complément, **sans lecture de la base** (AC 4) :
/// au moins une ligne, plafond `MAX_LINES_PER_ENTRY − 1` (une ligne réservée à
/// la contrepartie), montants valides sur une seule colonne, comptes distincts.
fn parse_complement_lines(req: &OpeningComplementRequest) -> Result<Vec<ComplementLine>, AppError> {
    if req.lines.is_empty() {
        return Err(complement_invalid(
            "OPENING_COMPLEMENT_NO_LINES",
            "error-opening-complement-no-lines",
            "Saisissez au moins un compte à compléter.",
        ));
    }
    if req.lines.len() > MAX_LINES_PER_ENTRY - 1 {
        return Err(complement_invalid(
            "OPENING_COMPLEMENT_TOO_MANY_LINES",
            "error-opening-complement-too-many-lines",
            "Trop de lignes pour un seul complément.",
        ));
    }
    let mut lines = Vec::with_capacity(req.lines.len());
    let mut seen = std::collections::HashSet::new();
    for line in &req.lines {
        let debit = parse_complement_amount(line.debit.as_deref())?;
        let credit = parse_complement_amount(line.credit.as_deref())?;
        // Exactement une colonne non nulle.
        if (debit > Decimal::ZERO) == (credit > Decimal::ZERO) {
            return Err(invalid_amount());
        }
        if !seen.insert(line.account_id) {
            return Err(complement_invalid(
                "OPENING_COMPLEMENT_DUPLICATE_ACCOUNT",
                "error-opening-complement-duplicate-account",
                "Un même compte figure sur deux lignes : regroupez-les.",
            ));
        }
        lines.push(ComplementLine {
            account_id: line.account_id,
            debit,
            credit,
        });
    }
    // La contrepartie (Σ débit − Σ crédit) doit tenir elle aussi dans
    // `DECIMAL(19,4)` : chaque ligne bornée ne suffit pas, leur somme pourrait
    // dépasser (revue de code P2, F-6 / R-L3).
    let gap: Decimal = lines.iter().map(|l| l.debit - l.credit).sum();
    if gap.abs() > Decimal::from_str("999999999999999.9999").expect("borne constante") {
        return Err(invalid_amount());
    }
    Ok(lines)
}

/// `POST /api/v1/opening-balances/complete` — complète un ou plusieurs comptes
/// de bilan oubliés à l'ouverture (Story 25-7, AC 4). Comptable+.
///
/// Les contrôles de forme se font ici ; les gardes métier, la date, la
/// contrepartie et l'écriture se font sous verrou dans
/// [`opening_complement::create_opening_complement`]. L'appel est rejoué sur
/// interblocage par l'enveloppe `DbError` [`kesh_db::retry::retry_on_deadlock`],
/// comme toute route qui écrit au journal : l'ordre des verrous ne fait que
/// réduire la fréquence des interblocages, des cycles résiduels existent (fiche
/// de la story 25-7, section « Cycles ») ; chaque tentative refait toutes les
/// gardes.
pub async fn complete_opening_balances(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<OpeningComplementRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let lines = parse_complement_lines(&req)?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let locale = Locale::from(company.accounting_language.as_str());
    let description = state
        .i18n
        .format(&locale, "opening-balances-complement-description", None);
    let today = chrono::Utc::now().date_naive();

    let result = kesh_db::retry::retry_on_deadlock("opening_balances::complete", || {
        let pool = &state.pool;
        let lines = lines.clone();
        let description = description.clone();
        async move {
            opening_complement::create_opening_complement(
                pool,
                company.id,
                current_user.user_id,
                &lines,
                description,
                today,
            )
            .await
        }
    })
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(result)),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line(
        account_id: i64,
        debit: Option<&str>,
        credit: Option<&str>,
    ) -> OpeningComplementLineRequest {
        OpeningComplementLineRequest {
            account_id,
            debit: debit.map(Into::into),
            credit: credit.map(Into::into),
        }
    }

    fn code_of(r: Result<Vec<ComplementLine>, AppError>) -> &'static str {
        match r {
            Err(AppError::OpeningComplementInvalid { error_code, .. }) => error_code,
            other => panic!("attendu un refus de forme, obtenu {other:?}"),
        }
    }

    #[test]
    fn complement_forme_refus_par_cause() {
        let req = |lines| OpeningComplementRequest { lines };
        assert_eq!(
            code_of(parse_complement_lines(&req(vec![]))),
            "OPENING_COMPLEMENT_NO_LINES"
        );
        let many = (0..MAX_LINES_PER_ENTRY as i64)
            .map(|i| line(i, Some("1"), None))
            .collect();
        assert_eq!(
            code_of(parse_complement_lines(&req(many))),
            "OPENING_COMPLEMENT_TOO_MANY_LINES"
        );
        for bad in [
            line(1, Some("0"), Some("0")),
            line(1, Some("5"), Some("5")),
            line(1, Some("-5"), None),
            line(1, Some("abc"), None),
            line(1, Some("1.00001"), None),
            line(1, Some("1000000000000000"), None),
            line(1, None, None),
        ] {
            assert_eq!(
                code_of(parse_complement_lines(&req(vec![bad]))),
                "OPENING_COMPLEMENT_INVALID_AMOUNT"
            );
        }
        assert_eq!(
            code_of(parse_complement_lines(&req(vec![
                line(1, Some("5"), None),
                line(1, None, Some("5")),
            ]))),
            "OPENING_COMPLEMENT_DUPLICATE_ACCOUNT"
        );
    }

    #[test]
    fn complement_forme_borne_la_contrepartie() {
        let max = "999999999999999";
        let req = OpeningComplementRequest {
            lines: vec![line(1, Some(max), None), line(2, Some(max), None)],
        };
        assert_eq!(
            code_of(parse_complement_lines(&req)),
            "OPENING_COMPLEMENT_INVALID_AMOUNT"
        );
        // Deux lignes maximales qui se compensent : la contrepartie est nulle.
        let req = OpeningComplementRequest {
            lines: vec![line(1, Some(max), None), line(2, None, Some(max))],
        };
        assert!(parse_complement_lines(&req).is_ok());
    }

    #[test]
    fn complement_forme_accepte_zero_vide_ou_absent_dans_l_autre_colonne() {
        let ok = parse_complement_lines(&OpeningComplementRequest {
            lines: vec![
                line(1, Some("12.3400"), Some("0")),
                line(2, Some(""), Some("5")),
                line(3, None, Some("0.0001")),
            ],
        })
        .unwrap();
        assert_eq!(ok.len(), 3);
        assert_eq!(ok[0].debit, Decimal::from_str("12.34").unwrap());
    }

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
