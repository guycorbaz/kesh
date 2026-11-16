//! Routes HTTP pour les écritures comptables en partie double.
//!
//! - `GET /api/v1/journal-entries` — liste des 50 dernières écritures
//!   (authenticated_routes, tout rôle incluant Consultation).
//! - `POST /api/v1/journal-entries` — création atomique d'une écriture
//!   (comptable_routes, Admin + Comptable).
//! - `PUT /api/v1/journal-entries/{id}` — modification avec verrou optimiste
//!   (story 3.3 ; gelée par la 24-4b, rouverte par la Story 15-8a, #532).
//! - `DELETE /api/v1/journal-entries/{id}` — suppression avec audit (story 3.3).
//!
//! Les deux lectures (liste et détail) passent par `services::ecritures`
//! (Story 32-4a) ; les mutations restent ici (C-32-0-4).

use std::str::FromStr;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_core::accounting::{
    self, ForeignAmount, Journal as CoreJournal, JournalEntryDraft, JournalEntryLineDraft,
    RateFromChfError, RateProvenance, fx_chf, rate_from_chf,
};
use kesh_core::errors::CoreError;
use kesh_core::listing::{SortBy, SortDirection};
use kesh_core::types::Money;
use kesh_db::entities::audit_log::Acteur;
use kesh_db::entities::{
    Journal as DbJournal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
    NewJournalEntryLine,
};
use kesh_db::repositories::{fiscal_years, imported_supplier_invoices, journal_entries};

use crate::AppState;
use crate::dates_recues::borner;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::services::acces::ContexteAcces;
use crate::services::ecritures;

/// Défaut de la pagination de la liste des écritures : celui du service
/// (Story 32-4a, C-32-4a-5), qui écrête aussi.
fn default_limit() -> i64 {
    ecritures::LIMITE_DEFAUT
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
    /// Montant au débit (CHF), format string décimal (ex: "100.00"). Parsé
    /// via `Decimal::from_str` — rejet 400 si format invalide.
    ///
    /// Story 37-3a (AC-6, V2) : facultatif. Une ligne CHF exige `debit` **et**
    /// `credit` (400 sinon — et non plus 422) ; sur une ligne en devise, un
    /// seul fourni vaut l'autre à 0, aucun fourni → le serveur pose le CHF du
    /// côté `foreign.side`.
    #[serde(default)]
    pub debit: Option<String>,
    /// Montant au crédit (CHF), format string décimal. Voir `debit`.
    #[serde(default)]
    pub credit: Option<String>,
    /// Projet analytique de la ligne (Epic 19, Story 19-2). Optionnel —
    /// absent ou `null` = ligne non taguée. Validé côté repo (projet de
    /// la company, non archivé) : inconnu/cross-company → 404, archivé → 409.
    #[serde(default)]
    pub project_id: Option<i64>,
    /// Partie en devise (Story 37-3a, AC-6) ; absente pour une ligne CHF.
    #[serde(default)]
    pub foreign: Option<ForeignLineRequest>,
}

/// `foreign` d'une ligne du corps (Story 37-3a, AC-6) : la ligne se passe par
/// l'API seule, sans calcul du client (V1, V2).
///
/// ⚠️ Tous les champs sont des **chaînes** analysées dans le handler : une
/// énumération ou un nombre JSON échouerait à la désérialisation d'axum, qui
/// rend 422 et non le 400 attendu.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignLineRequest {
    /// Devise **annoncée** ; absente, la ligne prend celle de son compte.
    /// Comparée telle quelle à la devise du compte (`FX_CURRENCY_MISMATCH`) ;
    /// jamais stockée.
    #[serde(default)]
    pub currency: Option<String>,
    /// Montant en devise (requis).
    #[serde(default)]
    pub amount: Option<String>,
    /// Cours par unité ; absent, il se déduit du CHF (`rate_from_chf`).
    #[serde(default)]
    pub rate: Option<String>,
    /// Montant CHF de la ligne ; s'il accompagne `debit`/`credit`, il doit
    /// égaler `debit + credit`.
    #[serde(default)]
    pub chf: Option<String>,
    /// `afc` ou `manual` ; absent, le dépôt qualifie la ligne. `revaluation`
    /// est refusé ici (réservé à la réévaluation, 37-5).
    #[serde(default)]
    pub rate_source: Option<String>,
    /// `debit` ou `credit` : le côté où poser le CHF quand `debit`/`credit`
    /// sont absents.
    #[serde(default)]
    pub side: Option<String>,
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
    /// Partie en devise (Story 37-3a, AC-6) — **omise** pour une ligne CHF
    /// (registre C-37-9 : le JSON reste identique en mono-devise ; ⚠️
    /// contrairement aux champs de lettrage, présents à `null`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub foreign: Option<ForeignLineResponse>,
}

/// `foreign` d'une ligne rendue (Story 37-3a, AC-6) : devise **lue sur le
/// compte**, décimaux en chaînes, provenance `afc|manual|revaluation`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ForeignLineResponse {
    pub currency: String,
    pub amount: String,
    pub rate: String,
    pub rate_source: &'static str,
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
    /// Code d'écran du motif, `null` si `modifiable` — l'une de treize valeurs :
    /// `FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED`, `IS_A_REVERSAL`,
    /// `ALREADY_REVERSED`, `OWNED_BY_*`, `MATCHED_BANK_TRANSACTION`,
    /// `DETACHED_SUPPLIER_SETTLEMENT`, `ENTRY_IS_FX_REVALUATION` (Story 37-3a,
    /// juste après la garde d'écriture), `PERIOD_LOCKED`, `ENTRY_LETTERED`
    /// (Story 15-1a-ii, en dernier).
    pub modification_blocked_by: Option<String>,
    /// Numéro de pièce, nom de l'exercice postérieur clos, borne du verrou
    /// (`AAAA-MM-JJ`), ou code de lettrage ; `null` sinon.
    pub modification_blocked_label: Option<String>,
    /// Story 30-2c (AC21, C-30-2-15) — le document de la facture fournisseur
    /// dont cette écriture est l'**achat** ou le **règlement**, `null` sinon
    /// (écriture d'une autre origine, facture sans document). Le détail est le
    /// seul constructeur de cette réponse : `null` veut dire « aucun document »,
    /// jamais « non calculé ».
    pub source_document: Option<EntrySourceDocumentResponse>,
}

/// Le document d'une écriture d'achat ou de règlement (Story 30-2c, AC21) :
/// de quoi afficher le lien vers la facture et ouvrir l'aperçu par
/// `GET /supplier-invoices/{supplierInvoiceId}/source-document`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EntrySourceDocumentResponse {
    pub supplier_invoice_id: i64,
    pub supplier_invoice_number: Option<String>,
    /// Story 33-1b (#926) — le numéro interne, que dit le lien.
    pub internal_number: Option<String>,
    pub original_filename: String,
}

impl From<imported_supplier_invoices::EntrySourceDocument> for EntrySourceDocumentResponse {
    fn from(d: imported_supplier_invoices::EntrySourceDocument) -> Self {
        Self {
            supplier_invoice_id: d.supplier_invoice_id,
            supplier_invoice_number: d.supplier_invoice_number,
            internal_number: d.internal_number,
            original_filename: d.original_filename,
        }
    }
}

impl From<JournalEntryLine> for JournalEntryLineResponse {
    fn from(l: JournalEntryLine) -> Self {
        let foreign = l.foreign().map(|f| ForeignLineResponse {
            currency: f.currency,
            amount: f.amount.normalize().to_string(),
            rate: f.rate.normalize().to_string(),
            rate_source: f.source.as_str(),
        });
        Self {
            foreign,
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
        CoreError::EntryTotalOutOfRange => AppError::Validation(
            "Écriture invalide : le total des débits ou des crédits dépasse la valeur admise"
                .into(),
        ),
        // Story 37-3a (AC-3) — un bras par variante : sans lui, le code et les
        // `details` n'atteindraient jamais le client (repli ci-dessous).
        e @ (CoreError::FxValueOutOfRange { .. }
        | CoreError::FxLineRoundsToZero { .. }
        | CoreError::FxAmountMismatch { .. }) => AppError::FxLine(e),
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
    // Story 32-4a — bornes, validation (neuf refus, même ordre, mêmes
    // messages) et lecture : `services::ecritures::lister`.
    let demande = ecritures::DemandeListeEcritures {
        description: params.description,
        account_id: params.account_id,
        amount_min: params.amount_min,
        amount_max: params.amount_max,
        date_from: params.date_from,
        date_to: params.date_to,
        journal: params.journal,
        sort_by: params.sort_by,
        sort_dir: params.sort_dir,
        limit: Some(params.limit),
        offset: Some(params.offset),
    };
    let result = ecritures::lister(
        &ContexteAcces::depuis_api(&current_user),
        &state.lecture(),
        demande,
    )
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
    // Story 32-4a — société, écriture, puis les quatre motifs dans UNE
    // transaction de lecture (Story 15-1b-0, D2) : `services::ecritures::detail`,
    // où vit la garde `get_journal_entry_reads_in_one_transaction`.
    let detail = ecritures::detail(
        &ContexteAcces::depuis_api(&current_user),
        &state.lecture(),
        id,
    )
    .await?;
    let blocker = detail.blocage_contre_passation;
    let modification = detail.blocage_modification;

    Ok(Json(JournalEntryDetailResponse {
        entry: JournalEntryResponse::from(detail.ecriture),
        reversed_by_entry_id: detail.contre_passee_par,
        reversable: blocker.is_none(),
        reversal_blocked_by: blocker.as_ref().map(|(b, _, _)| b.code().to_string()),
        reversal_blocked_label: blocker.and_then(|(_, _, label)| label),
        modifiable: modification.is_none(),
        modification_blocked_by: modification.as_ref().map(|m| m.code().to_string()),
        modification_blocked_label: modification.and_then(|m| m.label()),
        source_document: detail.piece_source.map(EntrySourceDocumentResponse::from),
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
    // Story 35-5a0 — la date du jour, lue une fois sur l'horloge de l'état.
    let aujourd_hui = state.horloge.aujourd_hui();

    let created = kesh_db::retry::retry_on_deadlock("journal_entries::reverse", || {
        journal_entries::reverse(
            &state.pool,
            company.id,
            id,
            Acteur::from(&current_user),
            aujourd_hui,
        )
    })
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(JournalEntryResponse::from(created)),
    ))
}

/// POST /api/v1/journal-entries/{id}/reverse-at-next-fiscal-year-start —
/// contre-passe une écriture au **premier jour de l'exercice qui suit le
/// sien**, dans cet exercice (Story 29-1b, #232) : les transitoires de fin
/// d'exercice.
///
/// **Sans corps** — `/reverse` n'en accepte pas davantage (C-29-1-10). Même
/// contre-passation que [`reverse_journal_entry`], à la datation près
/// (`journal_entries::reverse_at_next_fiscal_year_start`) ; refus propres
/// `409 NEXT_FISCAL_YEAR_MISSING` et `409 NEXT_FISCAL_YEAR_CLOSED`, les autres
/// sont ceux de la contre-passation.
///
/// ⚠️ **Rejouée sur interblocage**, comme `/reverse` : mêmes verrous, dans le
/// même ordre, donc mêmes cycles connus.
pub async fn reverse_journal_entry_at_next_fiscal_year_start(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<JournalEntryResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let created = kesh_db::retry::retry_on_deadlock(
        "journal_entries::reverse_at_next_fiscal_year_start",
        || {
            journal_entries::reverse_at_next_fiscal_year_start(
                &state.pool,
                company.id,
                id,
                Acteur::from(&current_user),
            )
        },
    )
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

    // Parse des montants (string → Decimal) et analyse de `foreign` (Story
    // 37-3a, AC-6), dans l'ordre du corps. Rejet 400 si format invalide.
    let mut line_drafts: Vec<JournalEntryLineDraft> = Vec::with_capacity(lines.len());
    for (idx, line) in lines.iter().enumerate() {
        let (debit, credit, foreign) = parse_line_amounts(
            idx + 1,
            line.debit.as_deref(),
            line.credit.as_deref(),
            line.foreign.as_ref(),
        )?;
        line_drafts.push(JournalEntryLineDraft {
            account_id: line.account_id,
            debit: Money::new(debit),
            credit: Money::new(credit),
            project_id: line.project_id,
            foreign,
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
                // Story 37-3a : la partie en devise analysée, provenance
                // éventuellement `None` (qualifiée par le dépôt).
                foreign: l.foreign,
            })
            .collect(),
    })
}

/// Analyse un décimal du corps d'une ligne (refus 400 `Validation`).
fn parse_decimal(line: usize, champ: &str, s: &str) -> Result<Decimal, AppError> {
    Decimal::from_str(s)
        .map_err(|e| AppError::Validation(format!("ligne {line}: {champ} invalide ({e})")))
}

/// Analyse les montants d'**une** ligne du corps — `debit`, `credit` et
/// `foreign` (Story 37-3a, AC-6) — et rend `(débit, crédit, partie en devise)`,
/// la ligne complétée **sans lecture de la base** (V1, V2, CR-6) :
///
/// - ligne CHF (sans `foreign`) : `debit` **et** `credit` requis (400) ;
/// - ligne en devise : `amount` requis ; un `debit`, `credit` ou `chf` négatif
///   → `EntryNegativeAmount` avant tout calcul ; un seul de `debit`/`credit`
///   fourni vaut l'autre à 0 ; `chf` et `debit`/`credit` fournis doivent
///   s'égaler (`FX_AMOUNT_MISMATCH`) ; sans `rate`, le cours se déduit du CHF
///   de référence (`rate_from_chf`, marqueur `rate_derived`) — ni cours ni CHF
///   → 400 ; sans `debit`/`credit`, `side` est requis et reçoit le CHF (fourni,
///   ou `round(montant × cours, 2)`) ; avec, `side` doit désigner la seule
///   colonne non nulle s'il y en a exactement une ;
/// - `rateSource` parmi `afc`, `manual` (absent → `None`, qualifié par le
///   dépôt) ; `revaluation` et toute autre valeur → 400.
///
/// `pub(crate)` : les routes d'ouverture et de complément (37-3b) emploient la
/// même analyse.
pub(crate) fn parse_line_amounts(
    line: usize,
    debit: Option<&str>,
    credit: Option<&str>,
    foreign: Option<&ForeignLineRequest>,
) -> Result<(Decimal, Decimal, Option<ForeignAmount>), AppError> {
    let debit = debit.map(|s| parse_decimal(line, "débit", s)).transpose()?;
    let credit = credit
        .map(|s| parse_decimal(line, "crédit", s))
        .transpose()?;
    let Some(f) = foreign else {
        return match (debit, credit) {
            (Some(d), Some(c)) => Ok((d, c, None)),
            _ => Err(AppError::Validation(format!(
                "ligne {line}: débit et crédit requis"
            ))),
        };
    };
    let amount = f
        .amount
        .as_deref()
        .ok_or_else(|| AppError::Validation(format!("ligne {line}: montant en devise requis")))
        .and_then(|s| parse_decimal(line, "montant en devise", s))?;
    let rate = f
        .rate
        .as_deref()
        .map(|s| parse_decimal(line, "cours", s))
        .transpose()?;
    let chf = f
        .chf
        .as_deref()
        .map(|s| parse_decimal(line, "montant CHF", s))
        .transpose()?;
    let source = match f.rate_source.as_deref() {
        None => None,
        Some("afc") => Some(RateProvenance::Afc),
        Some("manual") => Some(RateProvenance::Manual),
        Some(other) => {
            return Err(AppError::Validation(format!(
                "ligne {line}: provenance du cours inconnue ({other}) : afc ou manual"
            )));
        }
    };
    let side = match f.side.as_deref() {
        None => None,
        Some("debit") => Some(true),
        Some("credit") => Some(false),
        Some(other) => {
            return Err(AppError::Validation(format!(
                "ligne {line}: côté inconnu ({other}) : debit ou credit"
            )));
        }
    };
    // Signe d'abord (l'ordre des refus de l'AC-3 ne dépend pas de `rate`).
    if [debit, credit, chf]
        .iter()
        .flatten()
        .any(|d| d.is_sign_negative() && !d.is_zero())
    {
        return Err(map_core_error(CoreError::EntryNegativeAmount));
    }
    let fournis = debit.is_some() || credit.is_some();
    // Somme vérifiée : deux montants proches de la borne de `Decimal` ne
    // doivent pas faire paniquer le handler (revue de code P2).
    let somme = debit
        .unwrap_or_default()
        .checked_add(credit.unwrap_or_default())
        .ok_or(AppError::FxLine(CoreError::FxValueOutOfRange { line }))?;
    let chf_ref = chf.or(fournis.then_some(somme));
    if let (Some(c), true) = (chf, fournis)
        && c != somme
    {
        return Err(AppError::FxLine(CoreError::FxAmountMismatch {
            line,
            expected: c,
            provided: somme,
        }));
    }
    let (rate, rate_derived) = match rate {
        Some(r) => (r, false),
        None => {
            let c = chf_ref.ok_or_else(|| {
                AppError::Validation(format!("ligne {line}: cours ou montant CHF requis"))
            })?;
            let r = rate_from_chf(&amount, &c).map_err(|e| {
                AppError::FxLine(match e {
                    RateFromChfError::OutOfRange => CoreError::FxValueOutOfRange { line },
                    RateFromChfError::Mismatch { expected } => CoreError::FxAmountMismatch {
                        line,
                        expected,
                        provided: c,
                    },
                })
            })?;
            (r, true)
        }
    };
    let (debit, credit) = if fournis {
        let (d, c) = (debit.unwrap_or_default(), credit.unwrap_or_default());
        if let Some(cote_debit) = side
            && (d.is_zero() != c.is_zero())
            && cote_debit == d.is_zero()
        {
            return Err(AppError::Validation(format!(
                "ligne {line}: le côté ne désigne pas la colonne renseignée"
            )));
        }
        (d, c)
    } else {
        let cote_debit = side.ok_or_else(|| {
            AppError::Validation(format!(
                "ligne {line}: côté requis (debit ou credit) quand débit et crédit sont absents"
            ))
        })?;
        let montant = match chf {
            Some(c) => c,
            None => fx_chf(&amount, &rate)
                .map_err(|_| AppError::FxLine(CoreError::FxValueOutOfRange { line }))?,
        };
        if cote_debit {
            (montant, Decimal::ZERO)
        } else {
            (Decimal::ZERO, montant)
        }
    };
    Ok((
        debit,
        credit,
        Some(ForeignAmount {
            declared_currency: f.currency.clone(),
            amount,
            rate,
            source,
            rate_derived,
        }),
    ))
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
    borner(req.entry_date, "entryDate")?;
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
            Acteur::from(&current_user),
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

    let acteur = Acteur::from(&current_user);
    let updated = kesh_db::retry::retry_on_deadlock("journal_entries::update", || {
        let pool = &state.pool;
        let new = new.clone();
        async move { journal_entries::update(pool, company.id, id, req.version, acteur, new).await }
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

    let acteur = Acteur::from(&current_user);
    kesh_db::retry::retry_on_deadlock("journal_entries::delete", || {
        let pool = &state.pool;
        async move { journal_entries::delete_by_id(pool, company.id, id, acteur).await }
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
            amount_currency: None,
            exchange_rate: None,
            rate_source: None,
            account_currency: None,
        });
        let json = serde_json::to_value(&resp).unwrap();
        assert_eq!(json["projectId"], 7);
        // Story 37-3a (AC-6) — une ligne CHF n'a pas de clé `foreign`.
        assert!(json.get("foreign").is_none(), "{json}");
    }

    /// Story 37-3a (AC-6) — une ligne en devise expose `foreign` : devise lue
    /// sur le compte, décimaux en chaînes sans zéros terminaux ; un montant posé
    /// sur un compte sans devise se lit comme une ligne CHF (pas de `foreign`).
    #[test]
    fn line_response_exposes_foreign_from_the_account_currency() {
        let ligne = |account_currency: Option<&str>| JournalEntryLine {
            id: 1,
            entry_id: 2,
            account_id: 3,
            line_order: 1,
            debit: rust_decimal_macros::dec!(1125.83),
            credit: rust_decimal_macros::dec!(0),
            project_id: None,
            lettering_key: None,
            lettering_origin: None,
            amount_currency: Some(rust_decimal_macros::dec!(1234.5600)),
            exchange_rate: Some(rust_decimal_macros::dec!(0.9119300000)),
            rate_source: Some("afc".into()),
            account_currency: account_currency.map(str::to_string),
        };
        let json =
            serde_json::to_value(JournalEntryLineResponse::from(ligne(Some("USD")))).unwrap();
        assert_eq!(
            json["foreign"],
            serde_json::json!({
                "currency": "USD",
                "amount": "1234.56",
                "rate": "0.91193",
                "rateSource": "afc",
            })
        );
        let chf = serde_json::to_value(JournalEntryLineResponse::from(ligne(None))).unwrap();
        assert!(chf.get("foreign").is_none(), "{chf}");
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
            amount_currency: None,
            exchange_rate: None,
            rate_source: None,
            account_currency: None,
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

    /// Revue de code P4 de la 37-3a — le bras `EntryTotalOutOfRange` de
    /// `map_core_error` rend 400 `VALIDATION_ERROR`, non le repli générique.
    #[tokio::test]
    async fn map_core_error_total_hors_intervalle_rend_validation_error() {
        let resp = map_core_error(CoreError::EntryTotalOutOfRange).into_response();
        let (status, body) = body_json(resp).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
        let msg = body["error"]["message"].as_str().unwrap_or_default();
        assert!(msg.contains("dépasse la valeur admise"), "message : {msg}");
    }

    /// Revue de code P3 de la 37-3a — la somme `debit + credit` d'une ligne en
    /// devise, au débordement de `Decimal`, rend 400 `FX_VALUE_OUT_OF_RANGE`
    /// et ne panique pas (la route, non le noyau, fait cette somme).
    #[test]
    fn parse_line_amounts_somme_qui_deborde_rend_fx_value_out_of_range() {
        let max = "79228162514264337593543950335";
        let f = ForeignLineRequest {
            currency: Some("USD".into()),
            amount: Some("100".into()),
            rate: Some("0.9".into()),
            chf: None,
            rate_source: None,
            side: None,
        };
        let err = parse_line_amounts(3, Some(max), Some(max), Some(&f)).unwrap_err();
        assert!(
            matches!(
                err,
                AppError::FxLine(CoreError::FxValueOutOfRange { line: 3 })
            ),
            "attendu FxValueOutOfRange ligne 3, reçu {err:?}"
        );
    }
}
