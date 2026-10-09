//! Routes HTTP pour les écritures comptables en partie double.
//!
//! - `GET /api/v1/journal-entries` — liste des 50 dernières écritures
//!   (authenticated_routes, tout rôle incluant Consultation).
//! - `POST /api/v1/journal-entries` — création atomique d'une écriture
//!   (comptable_routes, Admin + Comptable).
//! - `PUT /api/v1/journal-entries/{id}` — modification avec verrou optimiste
//!   (story 3.3 ; gelée par la 24-4b, rouverte par la Story 15-8a, #532).
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
    /// Projet analytique de la ligne (Epic 19, Story 19-2). Optionnel —
    /// absent ou `null` = ligne non taguée. Validé côté repo (projet de
    /// la company, non archivé) : inconnu/cross-company → 404, archivé → 409.
    #[serde(default)]
    pub project_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub lines: Vec<CreateJournalEntryLineRequest>,
}

/// Corps du `PUT /api/v1/journal-entries/{id}` (Story 15-8a, #532) : celui du
/// `POST`, plus la `version` lue (verrou optimiste).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub lines: Vec<CreateJournalEntryLineRequest>,
    pub version: i32,
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
    /// Projet analytique de la ligne (Epic 19). `null` = non taguée.
    pub project_id: Option<i64>,
    /// Clé du groupe de lettrage (Story 15-1a-i, #518) ; `null` si la ligne
    /// est ouverte. ⚠️ Toujours présents, nuls ou non : un champ du contrat.
    pub lettering_key: Option<i64>,
    /// Code affiché du groupe (`code_from_key` de la clé), `null` si ouverte.
    pub lettering_code: Option<String>,
    /// Origine du groupe (`document`, `reversal`, `manual`), `null` si ouverte.
    pub lettering_origin: Option<String>,
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
    /// Écriture que celle-ci contre-passe (Story 24-4a, #380).
    pub reverses_entry_id: Option<i64>,
    pub lines: Vec<JournalEntryLineResponse>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Détail d'une écriture, enrichi de quoi DÉCIDER côté écran (Story 24-4a).
///
/// ⛔ Ces trois champs vivent sur le **détail seul**, jamais sur la liste
/// paginée : aucun écran ne les y consomme, et les calculer par ligne
/// obligerait à joindre cinq tables plus `journal_entry_lines` → `accounts`.
///
/// Sans eux, l'écran devinerait — il ne pourrait ni masquer le bouton ni dire
/// pourquoi, et se rabattrait sur un 409 découvert APRÈS le clic.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryDetailResponse {
    #[serde(flatten)]
    pub entry: JournalEntryResponse,
    /// Écriture qui contre-passe celle-ci. **Dérivé** de l'`UNIQUE`, pas une
    /// colonne — une seconde colonne serait un second état à tenir cohérent.
    pub reversed_by_entry_id: Option<i64>,
    pub reversable: bool,
    /// Code canonique du motif, `null` si `reversable`. ⚠️ Un code, jamais une
    /// phrase : la traduction se fait à l'écran, dans les quatre locales.
    pub reversal_blocked_by: Option<String>,
    /// **Étiquette** de ce qui bloque — numéro de pièce, ou numéro du compte
    /// archivé.
    ///
    /// ⛔ Sans elle, l'écran dirait « réactivez-**le** » sans dire lequel : le
    /// refus qui NOMME les comptes est un 400 de l'ÉCRITURE, devenu
    /// inatteignable depuis que le bouton est masqué avant le clic.
    /// *(Relevé en passe 2 de revue de code.)*
    pub reversal_blocked_label: Option<String>,
    /// L'écriture se modifie-t-elle (Story 15-8a, D8) ? Motif d'écran, sur
    /// l'état **présent** : le `PUT` reste seul juge (corps, version, date).
    pub modifiable: bool,
    /// Code d'écran du motif, `null` si `modifiable` — l'une de douze valeurs :
    /// `FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED`, `IS_A_REVERSAL`,
    /// `ALREADY_REVERSED`, `OWNED_BY_*`, `MATCHED_BANK_TRANSACTION`,
    /// `DETACHED_SUPPLIER_SETTLEMENT`, `PERIOD_LOCKED`, `ENTRY_LETTERED`
    /// (Story 15-1a-ii, en dernier).
    pub modification_blocked_by: Option<String>,
    /// Numéro de pièce, nom de l'exercice postérieur clos, borne du verrou
    /// (`AAAA-MM-JJ`), ou code de lettrage ; `null` sinon.
    pub modification_blocked_label: Option<String>,
}

impl From<JournalEntryLine> for JournalEntryLineResponse {
    fn from(l: JournalEntryLine) -> Self {
        Self {
            id: l.id,
            account_id: l.account_id,
            line_order: l.line_order,
            debit: l.debit.to_string(),
            credit: l.credit.to_string(),
            project_id: l.project_id,
            lettering_key: l.lettering_key,
            lettering_code: l
                .lettering_key
                .map(kesh_db::repositories::letterings::code_of),
            lettering_origin: l.lettering_origin,
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
        reverses_entry_id: entry.reverses_entry_id,
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

/// Promu `pub(crate)` (Story 14-4, P1-L1-ECH) : réutilisé par
/// `routes::opening_balances` pour mapper les erreurs `accounting::validate`
/// sans dupliquer les arms (`ENTRY_UNBALANCED`, `EntryNeedsTwoLines`, …).
pub(crate) fn map_core_error(err: CoreError) -> AppError {
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
    /// Ne garder que les écritures touchant ce compte (issue #374).
    #[serde(default)]
    pub account_id: Option<i64>,
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
/// Query params : `description`, `accountId`, `amountMin`, `amountMax`,
/// `dateFrom`, `dateTo`, `journal`, `sortBy`, `sortDir`, `offset`, `limit`.
/// Tous optionnels. Voir `ListJournalEntriesQuery`.
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
    if let Some(min) = amount_min
        && min < Decimal::ZERO
    {
        return Err(AppError::Validation(
            "amountMin ne peut pas être négatif".into(),
        ));
    }
    if let Some(max) = amount_max
        && max < Decimal::ZERO
    {
        return Err(AppError::Validation(
            "amountMax ne peut pas être négatif".into(),
        ));
    }

    // P2 : cross-validation des bornes. Un filtre min > max retournerait
    // 0 résultats silencieusement — l'utilisateur croirait à une absence
    // de données au lieu d'un filtre incohérent. Mieux vaut rejeter.
    if let (Some(min), Some(max)) = (amount_min, amount_max)
        && min > max
    {
        return Err(AppError::Validation(
            "amountMin doit être inférieur ou égal à amountMax".into(),
        ));
    }
    if let (Some(from), Some(to)) = (date_from, date_to)
        && from > to
    {
        return Err(AppError::Validation(
            "dateFrom doit être inférieur ou égal à dateTo".into(),
        ));
    }

    // `accountId <= 0` est un 400 plutôt qu'un filtre qui ne rendrait rien :
    // un filtre silencieusement vide se lit comme « pas d'écritures », ce qui
    // est le pire des deux messages.
    if let Some(id) = params.account_id
        && id <= 0
    {
        return Err(AppError::Validation(
            "accountId doit être un identifiant positif".into(),
        ));
    }
    let account_id = params.account_id;

    // Trim description (garde-fou cohérent avec create/update).
    let description = params
        .description
        .as_ref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    let query = JournalEntryListQuery {
        description,
        account_id,
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
        journal_entries::list_by_company_paginated(&state.pool, current_user.company_id, query)
            .await?;

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
/// `pub(crate)` : borne réutilisée par `routes::opening_balances`
/// (défense en profondeur miroir, Story 14-4).
pub(crate) const MAX_LINES_PER_ENTRY: usize = 500;

/// Longueur maximale du libellé (alignée sur la colonne `description VARCHAR(500)`).
const MAX_DESCRIPTION_LEN: usize = 500;

/// GET /api/v1/journal-entries/{id} — détail d'une écriture (lignes incluses),
/// scopée à la company courante.
///
/// **P10 defense in depth** : le scoping `company_id` est délégué à
/// [`journal_entries::find_by_id`] (anti-énumération cross-tenant KF-002).
/// Une écriture inexistante OU appartenant à une autre company retourne
/// `404 NOT_FOUND`, jamais `403` (pas de fuite d'existence).
pub async fn get_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<JournalEntryDetailResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let entry = journal_entries::find_by_id(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    let blocker = journal_entries::reversal_blocker(&state.pool, company.id, id).await?;
    let reversed_by_entry_id = journal_entries::reversed_by(&state.pool, company.id, id).await?;
    let modification = journal_entries::modification_blocker(&state.pool, company.id, id).await?;

    Ok(Json(JournalEntryDetailResponse {
        entry: JournalEntryResponse::from(entry),
        reversed_by_entry_id,
        reversable: blocker.is_none(),
        reversal_blocked_by: blocker.as_ref().map(|(b, _, _)| b.code().to_string()),
        reversal_blocked_label: blocker.and_then(|(_, _, label)| label),
        modifiable: modification.is_none(),
        modification_blocked_by: modification.as_ref().map(|m| m.code().to_string()),
        modification_blocked_label: modification.and_then(|m| m.label()),
    }))
}

/// POST /api/v1/journal-entries/{id}/reverse — contre-passe une écriture.
///
/// ⛔ **Crée une écriture ; de l'origine, ne touche qu'à la marque de lettrage.**
/// L'origine reste intacte dans ses montants, comptes, dates et libellés : c'est
/// l'exigence de l'art. 958f CO — la correction doit être apparente, non
/// substituée à ce qu'elle corrige. Seules ses lignes lettrables encore
/// ouvertes reçoivent la marque `reversal` qui les apparie à leur miroir (R6,
/// Story 15-1a-ii) ; la réponse `201` porte ces lignes lettrées.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`].
pub async fn reverse_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let created = kesh_db::retry::retry_on_deadlock("journal_entries::reverse", || {
        journal_entries::reverse(&state.pool, company.id, id, current_user.user_id)
    })
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(created)),
    ))
}

/// Prépare le `NewJournalEntry` d'un `POST` ou d'un `PUT` (Story 15-8a, D4 —
/// extraite du `POST`, jamais recopiée) : trim et longueur du libellé, borne
/// de lignes, parse des montants, `accounting::validate`.
///
/// ⛔ **Refus de FORME uniquement : ne lit pas la base.** C'est ce qui permet
/// de la placer avant le 404 sans révéler l'existence d'une ressource d'une
/// autre société, et hors de la fermeture rejouée sur interblocage.
///
/// `company_id` est celui de la société de l'appelant, jamais une donnée du
/// corps.
fn prepare_new_journal_entry(
    company_id: i64,
    entry_date: NaiveDate,
    journal: CoreJournal,
    description: &str,
    lines: &[CreateJournalEntryLineRequest],
) -> Result<NewJournalEntry, AppError> {
    // P5 : trim du libellé dès l'entrée — unique source de vérité.
    let trimmed_description = description.trim().to_string();

    // P6 : validation longueur libellé avant tout appel DB.
    if trimmed_description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "libellé trop long (max {MAX_DESCRIPTION_LEN} caractères)"
        )));
    }

    // P7 : borne haute sur le nombre de lignes (vecteur DoS).
    if lines.len() > MAX_LINES_PER_ENTRY {
        return Err(AppError::Validation(format!(
            "trop de lignes dans l'écriture (max {MAX_LINES_PER_ENTRY})"
        )));
    }

    // Parse des montants (string → Decimal). Rejet 400 si format invalide.
    let mut line_drafts: Vec<JournalEntryLineDraft> = Vec::with_capacity(lines.len());
    for (idx, line) in lines.iter().enumerate() {
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
            project_id: line.project_id,
        });
    }

    // Garde-fou #1 : validation métier pure (kesh-core).
    let draft = JournalEntryDraft {
        date: entry_date,
        journal,
        description: trimmed_description,
        lines: line_drafts,
    };
    // P4 : on récupère le BalancedEntry validé et on l'utilise pour
    // construire le NewJournalEntry, éliminant la duplication fragile
    // entre line_drafts et line_decimals (ex-security theater).
    let balanced = accounting::validate(draft).map_err(map_core_error)?;
    let validated = balanced.into_draft();

    // Construction du NewJournalEntry pour kesh-db depuis les données
    // garanties équilibrées par `validate()`.
    Ok(NewJournalEntry {
        company_id,
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
                // Tag analytique par-ligne (19-2) — a traversé validate()
                // verbatim depuis la request.
                project_id: l.project_id,
            })
            .collect(),
    })
}

/// POST /api/v1/journal-entries — crée une écriture en partie double.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`] ; le `NewJournalEntry` est cloné à
/// chaque tentative, et la correspondance d'erreur vient après le rejeu.
pub async fn create_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateJournalEntryRequest>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    // Refus de FORME d'abord (Story 15-8a, C-15-8-16) : ils ne lisent pas la
    // base. ⚠️ Effet de bord assumé : un corps déséquilibré ET sans exercice
    // rend désormais `ENTRY_UNBALANCED` avant `NO_FISCAL_YEAR`.
    let new = prepare_new_journal_entry(
        company.id,
        req.entry_date,
        req.journal,
        &req.description,
        &req.lines,
    )?;

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

    // Création atomique (re-lock FY + numérotation + INSERT + balance check).
    // P2 : mapping stable via variants DbError dédiés (plus de matching sur
    // le contenu du message).
    let result = kesh_db::retry::retry_on_deadlock("journal_entries::create", || {
        journal_entries::create(
            &state.pool,
            fiscal_year.id,
            current_user.user_id,
            new.clone(),
        )
    })
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

/// PUT /api/v1/journal-entries/{id} — modifie une écriture tant que son
/// exercice est ouvert (Story 15-8a, #532 — révise le gel de la 24-4b).
///
/// Corps : `{ entryDate, journal, description, lines, version }` ; réponse
/// `200` + l'écriture (forme du `POST`). Le cadre, l'ordre des refus, la
/// sérialisation et les cycles de verrous sont au doc-comment de
/// [`journal_entries::update`].
///
/// ⚠️ Les refus de **forme** (400 du corps, `accounting::validate`) précèdent
/// le 404, comme au `POST` : ils ne dépendent d'aucune donnée de la base. Pas de
/// pré-contrôle `find_covering_date` : l'exercice est celui **de l'écriture**,
/// connu seulement sous le verrou.
///
/// ⛔ **Rejoué sur interblocage** par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`] (C-15-8-19, Story 15-5e2) : l'ordre de
/// verrous du `PUT` referme quatre cycles — trois hérités, et depuis la
/// Story 15-1a-ii le cycle lignes ↔ écriture avec l'acte 1 du lettrage (ligne
/// du `PUT` dans « Where This Applies » du Pattern 5). La transaction est rejouée entière — le
/// repository ouvre et ferme la sienne, l'interblocage l'a annulée sans rien
/// écrire, et le contrôle de `version` refuserait un second passage. La
/// préparation reste hors de la fermeture.
pub async fn update_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateJournalEntryRequest>,
) -> Result<Json<JournalEntryResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let new = prepare_new_journal_entry(
        company.id,
        req.entry_date,
        req.journal,
        &req.description,
        &req.lines,
    )?;

    let updated = kesh_db::retry::retry_on_deadlock("journal_entries::update", || {
        let pool = &state.pool;
        let new = new.clone();
        async move {
            journal_entries::update(
                pool,
                company.id,
                id,
                req.version,
                current_user.user_id,
                current_user.api_key_id,
                new,
            )
            .await
        }
    })
    .await?;

    Ok(Json(JournalEntryResponse::from(updated)))
}

/// DELETE /api/v1/journal-entries/{id} — supprime une écriture **dans le cadre
/// de la modification** (Story 15-8b, #532) : exercice ouvert, aucun exercice
/// postérieur clos, ni contre-passée ni contre-passation, aucune pièce, pas un
/// paiement détaché, date postérieure à la borne du verrou de période, aucune
/// ligne lettrée (Story 15-1a-ii, `409 ENTRY_LETTERED`, en dernier). Réponse
/// `204`. Les refus et leur ordre sont au doc-comment de
/// [`journal_entries::delete_in_tx`] ; le mappage HTTP est le même qu'au `PUT`
/// (même garde, mêmes codes).
///
/// La trace `journal_entry.deleted` porte l'instantané complet et l'acteur —
/// la **clé d'API** quand c'est elle qui appelle (C-15-8-8). Le numéro de
/// l'écriture n'est jamais réattribué.
///
/// ⛔ **Rejoué sur interblocage** par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`] (C-15-8-19, Story 15-5e2) — par uniformité
/// avec le `PUT` à l'origine, et contre un cycle connu depuis la Story 15-1a-ii :
/// l'acte 1 du lettrage prend une ligne puis son écriture, le `DELETE`
/// l'écriture puis ses lignes. L'ordre de verrous est écriture et exercice
/// (jointure) → exercices postérieurs → lignes de l'écriture (`FOR UPDATE`,
/// étape 3-quinquies) → contrôles des clés étrangères au `DELETE`, sans autre
/// `INSERT` que l'audit. La transaction est rejouée entière ; l'interblocage
/// l'a annulée sans rien écrire.
pub async fn delete_journal_entry(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    kesh_db::retry::retry_on_deadlock("journal_entries::delete", || {
        let pool = &state.pool;
        async move {
            journal_entries::delete_by_id(
                pool,
                company.id,
                id,
                current_user.user_id,
                current_user.api_key_id,
            )
            .await
        }
    })
    .await?;

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

    // -- Story 19-2 : tag analytique par-ligne ------------------------------

    /// Le `projectId` d'une ligne est optionnel : absent → `None`,
    /// `null` → `None`, entier → `Some` (compat clients pré-19-2).
    #[test]
    fn line_request_project_id_is_optional() {
        let absent: CreateJournalEntryLineRequest =
            serde_json::from_str(r#"{"accountId":1,"debit":"10","credit":"0"}"#).unwrap();
        assert_eq!(absent.project_id, None);

        let null: CreateJournalEntryLineRequest =
            serde_json::from_str(r#"{"accountId":1,"debit":"10","credit":"0","projectId":null}"#)
                .unwrap();
        assert_eq!(null.project_id, None);

        let tagged: CreateJournalEntryLineRequest =
            serde_json::from_str(r#"{"accountId":1,"debit":"10","credit":"0","projectId":7}"#)
                .unwrap();
        assert_eq!(tagged.project_id, Some(7));
    }

    /// La réponse ligne expose `projectId` en camelCase (null si non taguée).
    #[test]
    fn line_response_serializes_project_id() {
        let resp = JournalEntryLineResponse::from(JournalEntryLine {
            id: 1,
            entry_id: 2,
            account_id: 3,
            line_order: 1,
            debit: rust_decimal_macros::dec!(10),
            credit: rust_decimal_macros::dec!(0),
            project_id: Some(7),
            lettering_key: None,
            lettering_origin: None,
        });
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["projectId"], 7);
    }

    /// Story 15-1a-i (AC14) — la ligne expose les trois champs du lettrage :
    /// présents et nuls sur une ligne ouverte, la clé, son code
    /// (`code_from_key`) et l'origine sur une ligne lettrée.
    #[test]
    fn journal_entry_line_response_exposes_lettering() {
        let ligne = |lettering_key: Option<i64>, lettering_origin: Option<&str>| JournalEntryLine {
            id: 27,
            entry_id: 2,
            account_id: 3,
            line_order: 1,
            debit: rust_decimal_macros::dec!(10),
            credit: rust_decimal_macros::dec!(0),
            project_id: None,
            lettering_key,
            lettering_origin: lettering_origin.map(str::to_string),
        };
        let ouverte =
            serde_json::to_value(JournalEntryLineResponse::from(ligne(None, None))).unwrap();
        for champ in ["letteringKey", "letteringCode", "letteringOrigin"] {
            assert!(
                ouverte.get(champ).is_some_and(serde_json::Value::is_null),
                "{champ} doit être présent et nul sur une ligne ouverte : {ouverte}"
            );
        }
        let lettree = serde_json::to_value(JournalEntryLineResponse::from(ligne(
            Some(27),
            Some("manual"),
        )))
        .unwrap();
        assert_eq!(lettree["letteringKey"], 27);
        assert_eq!(
            lettree["letteringCode"],
            kesh_core::lettering::code_from_key(27)
        );
        assert_eq!(lettree["letteringCode"], "AA");
        assert_eq!(lettree["letteringOrigin"], "manual");
    }

    /// Projet inconnu/cross-company → 404 ; projet archivé → 409 (mapping
    /// DbError générique réutilisé par la validation par-ligne 19-2).
    #[tokio::test]
    async fn project_validation_errors_map_to_4xx() {
        let resp = AppError::from(kesh_db::errors::DbError::NotFound).into_response();
        let (status, _) = body_json(resp).await;
        assert_eq!(status, StatusCode::NOT_FOUND);

        let resp = AppError::from(kesh_db::errors::DbError::IllegalStateTransition(
            "le projet analytique est archivé".into(),
        ))
        .into_response();
        let (status, body) = body_json(resp).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(body["error"]["code"], "ILLEGAL_STATE_TRANSITION");
    }
}
