//! Routes API pour les exercices comptables (Story 3.7).
//!
//! Endpoints :
//! - `GET    /api/v1/fiscal-years`        — liste scopée company (DESC).
//! - `GET    /api/v1/fiscal-years/{id}`   — détail scopé company.
//! - `POST   /api/v1/fiscal-years`        — création (Comptable+).
//! - `PUT    /api/v1/fiscal-years/{id}`   — renommage uniquement (Comptable+).
//! - `POST   /api/v1/fiscal-years/{id}/close` — transition Open→Closed (Comptable+).
//! - `POST   /api/v1/fiscal-years/{id}/reopen` — transition Closed→Open (Admin, motif, audit — Story 14-2).
//!
//! Toutes les routes mutatrices forcent `company_id = current_user.company_id`
//! ; tout `companyId` injecté dans le payload est ignoré (multi-tenant
//! defense en profondeur — Story 6.2 + Pass 2 HP2-M9).
//!
//! Pas de `DELETE` : conformément au CO art. 957-964 (10 ans de conservation),
//! aucun handler `delete_fiscal_year` n'est exposé. Une requête
//! `DELETE /api/v1/fiscal-years/{id}` retournera donc `405 Method Not Allowed`
//! (axum) — voir AC #23.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};

use kesh_db::entities::{FiscalYear, FiscalYearStatus, NewFiscalYear};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::{
    self, FY_NAME_DUPLICATE_KEY, FY_NAME_EMPTY_KEY, FY_NAME_TOO_LONG_KEY, FY_OVERLAP_KEY,
    FY_REOPEN_ALREADY_OPEN_KEY, FY_REOPEN_LIFO_BLOCKED_KEY, REOPEN_MOTIF_MAX,
};

use crate::AppState;
use crate::errors::{AppError, t};
use crate::middleware::auth::CurrentUser;
use crate::routes::api_keys::ensure_not_pat;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FiscalYearResponse {
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// `"Open"` ou `"Closed"` (PascalCase, cohérent avec l'enum DB).
    pub status: FiscalYearStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<FiscalYear> for FiscalYearResponse {
    fn from(fy: FiscalYear) -> Self {
        Self {
            id: fy.id,
            company_id: fy.company_id,
            name: fy.name,
            start_date: fy.start_date,
            end_date: fy.end_date,
            status: fy.status,
            created_at: fy.created_at,
            updated_at: fy.updated_at,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateFiscalYearRequest {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFiscalYearRequest {
    pub name: String,
}

/// Story 14-2 — body de `POST /api/v1/fiscal-years/{id}/reopen`.
///
/// Le `motif` est **obligatoire** (validé non vide et ≤ [`REOPEN_MOTIF_MAX`] au
/// handler, décision D4) et tracé dans l'audit `fiscal_year.reopened`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReopenFiscalYearRequest {
    pub motif: String,
}

// ---------------------------------------------------------------------------
// Mapping erreurs DB → AppError
// ---------------------------------------------------------------------------

/// Mapping des erreurs `create()` (Pass 2 HP2-M4 — namespaced Invariant keys).
fn map_create_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_OVERLAP_KEY => AppError::Validation(t(
            "error-fiscal-year-overlap",
            "Cet exercice chevauche un exercice existant",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_DUPLICATE_KEY => AppError::Validation(t(
            "error-fiscal-year-name-duplicate",
            "Un exercice avec ce nom existe déjà",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_EMPTY_KEY => AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )),
        // Story 3.7 Code Review Pass 1 F3 — pré-validation longueur (VARCHAR(50)).
        DbError::Invariant(ref s) if s == FY_NAME_TOO_LONG_KEY => AppError::Validation(t(
            "error-fiscal-year-name-too-long",
            "Le nom de l'exercice est trop long (50 caractères maximum)",
        )),
        DbError::CheckConstraintViolation(_) => AppError::Validation(t(
            "error-fiscal-year-dates-invalid",
            "Les dates de l'exercice sont invalides (la date de fin doit être strictement postérieure à la date de début)",
        )),
        DbError::UniqueConstraintViolation(_) => AppError::Validation(t(
            "error-fiscal-year-conflict",
            "Conflit d'exercice (nom ou date de début déjà utilisé)",
        )),
        DbError::ForeignKeyViolation(m) => {
            // Pass 2 HP2-M6 : log + erreur interne — JWT garantit que company_id existe.
            tracing::error!(
                "FK violation in fiscal_years::create — JWT scope should prevent this: {m}"
            );
            AppError::Internal(format!(
                "FK violation impossible (JWT scope) — investigate: {m}"
            ))
        }
        // Tout le reste (Sqlx, ConnectionUnavailable, NotFound, ...) → mapping standard.
        other => AppError::from(other),
    }
}

fn map_update_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_NAME_EMPTY_KEY => AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )),
        DbError::Invariant(ref s) if s == FY_NAME_DUPLICATE_KEY => AppError::Validation(t(
            "error-fiscal-year-name-duplicate",
            "Un exercice avec ce nom existe déjà",
        )),
        // Story 3.7 Code Review Pass 1 F3 — pré-validation longueur (VARCHAR(50)).
        DbError::Invariant(ref s) if s == FY_NAME_TOO_LONG_KEY => AppError::Validation(t(
            "error-fiscal-year-name-too-long",
            "Le nom de l'exercice est trop long (50 caractères maximum)",
        )),
        other => AppError::from(other),
    }
}

/// Mapping des erreurs `reopen()` (Story 14-2, décision D7 — miroir
/// `map_create_error`).
///
/// Les deux conflits de réouverture partagent le **code machine** 409
/// `ILLEGAL_STATE_TRANSITION` mais portent des **messages utilisateur distincts
/// et localisés** (via `AppError::IllegalState(t(...))`). Ils sont émis par le
/// repo en `DbError::Invariant(<KEY>)` namespacés — jamais en
/// `DbError::IllegalStateTransition` (dont le `Display` log-only écraserait le
/// message). Toute autre `Invariant` (dont `FY_REOPEN_UNEXPECTED_KEY`) retombe
/// vers le mapping global (→ 500, défensif).
fn map_reopen_error(err: DbError) -> AppError {
    match err {
        DbError::Invariant(ref s) if s == FY_REOPEN_ALREADY_OPEN_KEY => AppError::IllegalState(t(
            "error-fiscal-year-already-open",
            "Cet exercice est déjà ouvert.",
        )),
        DbError::Invariant(ref s) if s == FY_REOPEN_LIFO_BLOCKED_KEY => AppError::IllegalState(t(
            "error-fiscal-year-reopen-blocked",
            "Réouverture impossible : un exercice postérieur est clôturé ; rouvrez-le d'abord.",
        )),
        other => AppError::from(other),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/fiscal-years` — liste les exercices de la company courante,
/// triés `start_date DESC`.
pub async fn list_fiscal_years(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<FiscalYearResponse>>, AppError> {
    let list = fiscal_years::list_by_company(&state.pool, current_user.company_id).await?;
    Ok(Json(
        list.into_iter().map(FiscalYearResponse::from).collect(),
    ))
}

/// `GET /api/v1/fiscal-years/{id}` — détail d'un exercice scopé company.
///
/// Retourne 404 si l'exercice appartient à une autre company (anti-énumération).
pub async fn get_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    let fy = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(FiscalYearResponse::from(fy)))
}

/// `POST /api/v1/fiscal-years` — crée un exercice (Comptable+).
///
/// Le `company_id` est forcé depuis le JWT — tout champ `companyId` du payload
/// est ignoré (multi-tenant defense en profondeur).
pub async fn create_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateFiscalYearRequest>,
) -> Result<(StatusCode, Json<FiscalYearResponse>), AppError> {
    // Pré-validation client-side back-up : nom non vide.
    let name = req.name.trim().to_string();
    if name.is_empty() {
        return Err(AppError::Validation(t(
            "error-fiscal-year-name-empty",
            "Le nom de l'exercice est obligatoire",
        )));
    }

    let new = NewFiscalYear {
        company_id: current_user.company_id,
        name,
        start_date: req.start_date,
        end_date: req.end_date,
    };

    let fy = fiscal_years::create(&state.pool, current_user.user_id, new)
        .await
        .map_err(map_create_error)?;

    Ok((StatusCode::CREATED, Json(FiscalYearResponse::from(fy))))
}

/// `PUT /api/v1/fiscal-years/{id}` — renomme un exercice (Comptable+).
///
/// Seul le `name` est mutable. Les dates et le statut ne peuvent pas être
/// modifiés via cette route. Renommage autorisé même si `status='Closed'`
/// (le CO protège les montants/dates, pas un libellé descriptif).
pub async fn update_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateFiscalYearRequest>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    // Multi-tenant scoping : 404 si l'exercice n'existe pas dans cette company.
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let fy = fiscal_years::update_name(
        &state.pool,
        current_user.user_id,
        current_user.company_id,
        id,
        req.name,
    )
    .await
    .map_err(map_update_error)?;

    Ok(Json(FiscalYearResponse::from(fy)))
}

/// `POST /api/v1/fiscal-years/{id}/close` — transition Open → Closed (Comptable+).
///
/// Renvoie 409 `ILLEGAL_STATE_TRANSITION` si l'exercice est déjà clos
/// (mappé automatiquement via `AppError::Database(DbError::IllegalStateTransition)`).
pub async fn close_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    let fy = fiscal_years::close(
        &state.pool,
        current_user.user_id,
        current_user.company_id,
        id,
    )
    .await?;
    Ok(Json(FiscalYearResponse::from(fy)))
}

/// `POST /api/v1/fiscal-years/{id}/reopen` — transition Closed → Open (Story 14-2).
///
/// **Admin uniquement** (route montée dans `admin_routes`, scellé par
/// `require_admin_role` — aucun check de rôle dans ce handler) et **interdite
/// aux clés PAT** (`ensure_not_pat`, décision D5 : opération privilégiée
/// réglementaire, jamais une automatisation). Le motif est **obligatoire** et
/// **borné** ; il est tracé dans l'audit `fiscal_year.reopened`.
///
/// Réponses : 200 `FiscalYearResponse` (Open) ; 400 si motif vide / trop long ;
/// 404 si l'exercice n'existe pas dans cette company (anti-énumération) ; 409
/// `ILLEGAL_STATE_TRANSITION` (message distinct, D7) si déjà ouvert ou garde
/// LIFO violée ; 403 pour Comptable/Consultation/PAT ; 401 sans auth.
pub async fn reopen_fiscal_year(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ReopenFiscalYearRequest>,
) -> Result<Json<FiscalYearResponse>, AppError> {
    // D5 — anti-PAT : la réouverture n'est pas une opération d'automatisation.
    ensure_not_pat(&current_user)?;

    // Multi-tenant scoping D'ABORD : 404 si l'exercice n'existe pas dans cette
    // company (anti-énumération, miroir `close_fiscal_year` — check d'existence
    // avant toute validation de body, cohérent avec les handlers-sœurs).
    let _existing = fiscal_years::find_by_id_in_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    // Validation du motif au handler (D4). `trim()` puis `chars().count()`
    // (scalaires Unicode) — le frontend miroite cette borne sur `[...trimmed]`.
    let motif = req.motif.trim().to_string();
    if motif.is_empty() {
        return Err(AppError::Validation(t(
            "error-fiscal-year-reopen-motif-empty",
            "Le motif de réouverture est obligatoire.",
        )));
    }
    if motif.chars().count() > REOPEN_MOTIF_MAX {
        return Err(AppError::Validation(t(
            "error-fiscal-year-reopen-motif-too-long",
            "Le motif de réouverture est trop long (500 caractères maximum).",
        )));
    }

    let fy = fiscal_years::reopen(
        &state.pool,
        current_user.user_id,
        current_user.company_id,
        id,
        motif,
    )
    .await
    .map_err(map_reopen_error)?;

    Ok(Json(FiscalYearResponse::from(fy)))
}
