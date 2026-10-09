//! Routes HTTP du lettrage manuel (Story 15-1a-i, #518).
//!
//! - `POST /api/v1/letterings` — lettre un groupe de lignes (Comptable, Admin) ;
//! - `GET /api/v1/letterings/{key}` — lit un groupe (tout rôle authentifié) ;
//! - `DELETE /api/v1/letterings/{key}` — délettre un groupe (Comptable, Admin).
//!
//! `{key}` accepte la **clé numérique** ou le **code** (`27` ou `AA`) ; une
//! valeur invalide rend 404 ([`kesh_core::lettering::parse_group_reference`]).
//!
//! ⛔ **Toute la logique est dans la primitive unique**
//! (`kesh_db::repositories::letterings`) : ces handlers ne font que les refus
//! de **forme** (plafond de 200 lignes, au moins deux identifiants distincts),
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

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use kesh_core::lettering as core_lettering;
use kesh_db::errors::DbError;
use kesh_db::repositories::letterings::{self, Actor, LetteringGroup, Mode, Origin};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;

/// Corps du `POST /api/v1/letterings`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateLetteringRequest {
    /// Identifiants des lignes d'écriture que le lettrage réunit (2 à 200).
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
