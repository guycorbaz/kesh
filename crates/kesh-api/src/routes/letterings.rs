//! Routes HTTP du lettrage manuel (Story 15-1a-i, #518).
//!
//! - `POST /api/v1/letterings` — lettre un groupe de lignes (Comptable, Admin) ;
//! - `GET /api/v1/letterings/{key}` — lit un groupe (tout rôle authentifié) ;
//! - `DELETE /api/v1/letterings/{key}` — délettre un groupe (Comptable, Admin) ;
//! - `GET /api/v1/accounts/{id}/open-items` — les postes ouverts d'un compte à
//!   une date (tout rôle, Story 15-1b) ;
//! - `GET /api/v1/accounts/{id}/lettering-proposals` — les rapprochements que
//!   Kesh propose (tout rôle, Story 15-1b) ; Kesh n'écrit rien.
//!
//! `{key}` accepte la **clé numérique** ou le **code** (`27` ou `AA`) ; une
//! valeur invalide rend 404 ([`kesh_core::lettering::parse_group_reference`]).
//!
//! ⛔ **Toute la logique est dans la primitive unique**
//! (`kesh_db::repositories::letterings`) : ces handlers ne font que les refus
//! de **forme** (plafond de [`kesh_core::lettering::MAX_LINES_PER_GROUP`]
//! lignes, au moins deux identifiants distincts),
//! qui ne lisent pas la base, puis délèguent.
//!
//! ⚠️ **Rejouées sur interblocage** : les deux écrivains appellent l'enveloppe
//! `DbError` [`kesh_db::retry::retry_on_deadlock`] — la séquence de verrous de la
//! primitive forme des cycles résiduels nommés avec la modification, la
//! contre-passation et les insertions de lignes (R7 de la fiche) ; le rejeu en
//! est la défense (Pattern 5). Inscrits `(Traced, Rejouee)` au registre
//! `tests/audit_route_registry.rs`.
//!
//! Anti-IDOR (AC11) : une ligne ou une clé d'une autre société rend le même 404
//! qu'une ligne ou une clé inexistante ; aucun message ne nomme une ligne
//! étrangère.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use kesh_core::lettering as core_lettering;
use kesh_db::errors::DbError;
use kesh_db::repositories::journal_entries::DocumentOwner;
use kesh_db::repositories::letterings::{
    self, Actor, LetteringGroup, LetteringProposals, Mode, OpenItems, Origin, ProposalLine,
};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;

/// Corps du `POST /api/v1/letterings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLetteringRequest {
    /// Identifiants des lignes d'écriture que le lettrage réunit (de 2 à
    /// [`kesh_core::lettering::MAX_LINES_PER_GROUP`]).
    pub line_ids: Vec<i64>,
}

/// Une ligne d'un groupe dans la réponse.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LetteringLineResponse {
    pub id: i64,
    pub entry_id: i64,
    /// ⚠️ Le numéro d'écriture repart à 1 à chaque exercice : il se lit avec
    /// `fiscalYearId` / `fiscalYearName`.
    pub entry_number: i64,
    pub fiscal_year_id: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    /// Décimaux en chaîne (pas d'erreur d'arrondi JSON).
    pub debit: String,
    pub credit: String,
}

/// Réponse d'un groupe (`POST` → 201, `GET` → 200).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LetteringResponse {
    pub key: i64,
    pub code: String,
    pub origin: &'static str,
    pub account_id: i64,
    pub lines: Vec<LetteringLineResponse>,
}

impl From<LetteringGroup> for LetteringResponse {
    fn from(g: LetteringGroup) -> Self {
        Self {
            key: g.key,
            code: g.code,
            origin: g.origin.as_str(),
            account_id: g.account_id,
            lines: g
                .lines
                .into_iter()
                .map(|l| LetteringLineResponse {
                    id: l.id,
                    entry_id: l.entry_id,
                    entry_number: l.entry_number,
                    fiscal_year_id: l.fiscal_year_id,
                    fiscal_year_name: l.fiscal_year_name,
                    date: l.date,
                    debit: l.debit.to_string(),
                    credit: l.credit.to_string(),
                })
                .collect(),
        }
    }
}

/// Lit `{key}` (clé ou code) ; invalide → 404, comme une clé inexistante.
fn parse_reference(reference: &str) -> Result<i64, AppError> {
    core_lettering::parse_group_reference(reference).ok_or(AppError::Database(DbError::NotFound))
}

/// POST /api/v1/letterings — lettre un groupe de lignes (origine `manual`).
///
/// Refus de forme d'abord, sans lire la base (400 `LETTERING_TOO_MANY_LINES`,
/// 400 `LETTERING_TOO_FEW_LINES`) ; puis la primitive, dont la lecture
/// verrouillante scopée rend le 404 avant toute autre cause (AC6, AC11).
pub async fn create_lettering(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateLetteringRequest>,
) -> Result<(StatusCode, Json<LetteringResponse>), AppError> {
    letterings::check_manual_line_count(&req.line_ids)?;
    core_lettering::check_line_ids(&req.line_ids)
        .map_err(|_| AppError::Database(DbError::LetteringTooFewLines))?;

    let company = get_company_for(&current_user, &state.pool).await?;
    let actor = Actor {
        user_id: current_user.user_id,
        api_key_id: current_user.api_key_id,
    };
    let group = kesh_db::retry::retry_on_deadlock("letterings::create", || {
        let pool = &state.pool;
        let line_ids = req.line_ids.clone();
        async move {
            let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
            let group = letterings::create_group_in_tx(
                &mut tx,
                company.id,
                &line_ids,
                Origin::Manual,
                Mode::Manual,
                actor,
            )
            .await?;
            tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
            Ok(group)
        }
    })
    .await?;

    Ok((StatusCode::CREATED, Json(LetteringResponse::from(group))))
}

/// GET /api/v1/letterings/{key} — lit un groupe, sans verrou.
pub async fn get_lettering(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(reference): Path<String>,
) -> Result<Json<LetteringResponse>, AppError> {
    let key = parse_reference(&reference)?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    let group = letterings::find_group(&mut conn, company.id, key)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(LetteringResponse::from(group)))
}

/// DELETE /api/v1/letterings/{key} — délettre un groupe (mode `Manual` : un
/// groupe `document` ou une paire `reversal` dont une ligne est celle d'une
/// pièce ne se délettre pas à la main ; un groupe tout en période close non
/// plus — AC5).
pub async fn delete_lettering(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(reference): Path<String>,
) -> Result<StatusCode, AppError> {
    let key = parse_reference(&reference)?;
    let company = get_company_for(&current_user, &state.pool).await?;
    let actor = Actor {
        user_id: current_user.user_id,
        api_key_id: current_user.api_key_id,
    };
    kesh_db::retry::retry_on_deadlock("letterings::delete", || {
        let pool = &state.pool;
        async move {
            let mut tx = pool.begin().await.map_err(kesh_db::errors::map_db_error)?;
            letterings::dissolve_group_in_tx(&mut tx, company.id, key, Mode::Manual, actor).await?;
            tx.commit().await.map_err(kesh_db::errors::map_db_error)?;
            Ok(())
        }
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------------------------------------------------------------------------
// Story 15-1b — les postes ouverts et les propositions (lecture seule)
// ---------------------------------------------------------------------------

/// Défaut et plafond de la page des postes ouverts (patron de la liste des
/// écritures).
const OPEN_ITEMS_DEFAULT_LIMIT: i64 = 50;
/// Défaut des propositions.
const PROPOSALS_DEFAULT_LIMIT: i64 = 100;
/// Plafond commun des deux routes.
const MAX_LIMIT: i64 = 500;

fn default_open_items_limit() -> i64 {
    OPEN_ITEMS_DEFAULT_LIMIT
}

fn default_proposals_limit() -> i64 {
    PROPOSALS_DEFAULT_LIMIT
}

/// Paramètres de `GET /accounts/{id}/open-items`. `limit`/`offset` non
/// numériques : rejetés par l'extracteur `Query` d'Axum (sa réponse propre).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenItemsQuery {
    /// `AAAA-MM-JJ` ; absent (ou vide) → aujourd'hui.
    pub as_of: Option<String>,
    #[serde(default = "default_open_items_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

/// Paramètres de `GET /accounts/{id}/lettering-proposals` — **pas d'`offset`**.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalsQuery {
    #[serde(default = "default_proposals_limit")]
    pub limit: i64,
}

/// La pièce qui possède l'écriture d'une ligne. `type` par
/// `DocumentKind::as_str()` — la table écrite une fois, dans `kesh-db`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentResponse {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub id: i64,
    pub number: Option<String>,
    /// Pour `settlement` : la facture réglée ; `null` sinon.
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
}

impl From<DocumentOwner> for DocumentResponse {
    fn from(o: DocumentOwner) -> Self {
        Self {
            kind: o.kind.as_str(),
            id: o.id,
            number: o.number,
            invoice_id: o.invoice_id,
            invoice_number: o.invoice_number,
        }
    }
}

/// Une ligne ouverte à `asOf` (AC1).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenItemResponse {
    pub line_id: i64,
    pub entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub journal: String,
    pub description: String,
    pub debit: String,
    pub credit: String,
    pub document: Option<DocumentResponse>,
    pub lettering_code: Option<String>,
    pub lettering_origin: Option<&'static str>,
    pub lettered_on: Option<NaiveDate>,
    pub reason: &'static str,
    pub document_state: Option<&'static str>,
    pub amount_due: Option<String>,
    pub manually_letterable: bool,
    pub in_open_period: bool,
}

/// Réponse de `GET /accounts/{id}/open-items` (AC1). `balance`, `openTotal` et
/// `total` portent sur tout l'ensemble, non sur la page.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenItemsResponse {
    pub account_id: i64,
    pub account_number: String,
    pub as_of: NaiveDate,
    pub balance: String,
    pub open_total: String,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
    pub items: Vec<OpenItemResponse>,
}

impl From<OpenItems> for OpenItemsResponse {
    fn from(v: OpenItems) -> Self {
        Self {
            account_id: v.account_id,
            account_number: v.account_number,
            as_of: v.as_of,
            balance: v.balance.to_string(),
            open_total: v.open_total.to_string(),
            total: v.total,
            offset: v.offset,
            limit: v.limit,
            items: v
                .items
                .into_iter()
                .map(|i| OpenItemResponse {
                    line_id: i.line_id,
                    entry_id: i.entry_id,
                    entry_number: i.entry_number,
                    fiscal_year_name: i.fiscal_year_name,
                    date: i.date,
                    journal: i.journal,
                    description: i.description,
                    debit: i.debit.to_string(),
                    credit: i.credit.to_string(),
                    document: i.document.map(DocumentResponse::from),
                    lettering_code: i.lettering_code,
                    lettering_origin: i.lettering_origin.map(Origin::as_str),
                    lettered_on: i.lettered_on,
                    reason: i.reason.as_str(),
                    document_state: i.document_state.map(|s| s.as_str()),
                    amount_due: i.amount_due.map(|d| d.to_string()),
                    manually_letterable: i.manually_letterable,
                    in_open_period: i.in_open_period,
                })
                .collect(),
        }
    }
}

/// Une ligne d'une paire proposée — sous-ensemble d'[`OpenItemResponse`], même
/// sérialisation.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalLineResponse {
    pub line_id: i64,
    pub entry_id: i64,
    pub entry_number: i64,
    pub fiscal_year_name: String,
    pub date: NaiveDate,
    pub journal: String,
    pub description: String,
    pub document: Option<DocumentResponse>,
    pub in_open_period: bool,
}

impl From<ProposalLine> for ProposalLineResponse {
    fn from(l: ProposalLine) -> Self {
        Self {
            line_id: l.line_id,
            entry_id: l.entry_id,
            entry_number: l.entry_number,
            fiscal_year_name: l.fiscal_year_name,
            date: l.date,
            journal: l.journal,
            description: l.description,
            document: l.document.map(DocumentResponse::from),
            in_open_period: l.in_open_period,
        }
    }
}

/// Une paire proposée (AC5).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalResponse {
    pub amount: String,
    pub days_apart: i64,
    pub reversal_pair: bool,
    pub debit: ProposalLineResponse,
    pub credit: ProposalLineResponse,
}

/// Réponse de `GET /accounts/{id}/lettering-proposals` (AC5).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProposalsResponse {
    pub account_id: i64,
    pub candidate_count: usize,
    pub total: usize,
    pub limit: usize,
    pub items: Vec<ProposalResponse>,
}

impl From<LetteringProposals> for ProposalsResponse {
    fn from(p: LetteringProposals) -> Self {
        Self {
            account_id: p.account_id,
            candidate_count: p.candidate_count,
            total: p.total,
            limit: p.limit,
            items: p
                .items
                .into_iter()
                .map(|i| ProposalResponse {
                    amount: i.amount.to_string(),
                    days_apart: i.days_apart,
                    reversal_pair: i.reversal_pair,
                    debit: ProposalLineResponse::from(i.debit),
                    credit: ProposalLineResponse::from(i.credit),
                })
                .collect(),
        }
    }
}

/// `asOf` : lu en chaîne, parsé comme `dateFrom` de la liste des écritures ;
/// mal formé → 400 `VALIDATION_ERROR` ; absent ou vide → aujourd'hui, selon la
/// convention de la balance âgée (`Utc::now().naive_utc().date()` — l'écart
/// UTC/heure suisse la nuit est hérité). Aucune borne.
fn parse_as_of(as_of: Option<&str>) -> Result<NaiveDate, AppError> {
    match as_of {
        Some(s) if !s.is_empty() => s
            .parse::<NaiveDate>()
            .map_err(|e| AppError::Validation(format!("asOf invalide ({e})"))),
        _ => Ok(chrono::Utc::now().naive_utc().date()),
    }
}

/// GET /api/v1/accounts/{id}/open-items — les postes ouverts du compte à
/// `asOf` (AC1). Ordre des refus (AC7) : paramètres (`Query` d'Axum, puis
/// `asOf` → 400 avant toute lecture), compte introuvable ou d'une autre société
/// (404), compte non lettrable (409 `LETTERING_ACCOUNT_NOT_LETTERABLE`).
pub async fn get_open_items(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(account_id): Path<i64>,
    Query(params): Query<OpenItemsQuery>,
) -> Result<Json<OpenItemsResponse>, AppError> {
    let as_of = parse_as_of(params.as_of.as_deref())?;
    let limit = params.limit.clamp(1, MAX_LIMIT);
    let offset = params.offset.max(0);
    let company = get_company_for(&current_user, &state.pool).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    let vue =
        letterings::open_items(&mut conn, company.id, account_id, as_of, limit, offset).await?;
    Ok(Json(OpenItemsResponse::from(vue)))
}

/// GET /api/v1/accounts/{id}/lettering-proposals — les paires que Kesh propose
/// (AC5) ; 422 `LETTERING_PROPOSALS_TOO_MANY_LINES` au-delà du plafond de
/// candidates. Mêmes refus du compte que les postes ouverts.
pub async fn get_lettering_proposals(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(account_id): Path<i64>,
    Query(params): Query<ProposalsQuery>,
) -> Result<Json<ProposalsResponse>, AppError> {
    // `clamp(1, 500)` : la valeur est positive, la conversion ne perd rien.
    let limit = params.limit.clamp(1, MAX_LIMIT) as usize;
    let company = get_company_for(&current_user, &state.pool).await?;
    let mut conn = state
        .pool
        .acquire()
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    let propositions =
        letterings::lettering_proposals(&mut conn, company.id, account_id, limit).await?;
    Ok(Json(ProposalsResponse::from(propositions)))
}
