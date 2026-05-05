//! Routes Story 8-2 — Profils CSV bancaires (FR53).
//!
//! - `POST /api/v1/bank-profiles` (Comptable+) — création.
//! - `GET /api/v1/bank-profiles?page=&per_page=` (tous rôles) — liste.
//! - `GET /api/v1/bank-profiles/{id}` (tous rôles) — détail.
//! - `PUT /api/v1/bank-profiles/{id}` (Comptable+) — full replacement
//!   (Pass 1 M18 — sémantique REST stricte).
//! - `DELETE /api/v1/bank-profiles/{id}` (Comptable+).
//!
//! **Multi-tenant** : tous handlers scopent par `current_user.company_id`
//! (KF-002, 404 systématique cross-tenant).
//!
//! **Validation** : avant repo, le payload est validé via
//! `kesh_import::CsvProfile::validate()` (Pass 2 + Pass 3 patches —
//! XOR amount/debit_credit, séparateurs distincts, regex compilable,
//! chrono format, longueurs, encoding ∈ {UTF-8, ISO-8859-1}).

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};

use kesh_db::entities::bank_profile::{BankProfile, NewBankProfile};
use kesh_db::errors::DbError;
use kesh_db::repositories::bank_profiles;
use kesh_import::{ColumnMapping, CsvError, CsvProfile};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BankProfileResponse {
    pub id: i64,
    pub bank_name: String,
    pub filename_pattern: Option<String>,
    pub column_mapping: ColumnMapping,
    pub date_format: String,
    pub decimal_separator: String,
    pub field_separator: String,
    pub encoding: Option<String>,
    pub header_row_count: u8,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
}

impl BankProfileResponse {
    fn try_from_db(profile: BankProfile) -> Result<Self, AppError> {
        let column_mapping = profile.parse_column_mapping().map_err(AppError::Database)?;
        Ok(Self {
            id: profile.id,
            bank_name: profile.bank_name,
            filename_pattern: profile.filename_pattern,
            column_mapping,
            date_format: profile.date_format,
            decimal_separator: profile.decimal_separator,
            field_separator: profile.field_separator,
            encoding: profile.encoding,
            header_row_count: profile.header_row_count,
            created_at: profile.created_at,
            updated_at: profile.updated_at,
        })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateOrUpdateBankProfileRequest {
    pub bank_name: String,
    pub filename_pattern: Option<String>,
    pub column_mapping: ColumnMapping,
    pub date_format: String,
    pub decimal_separator: String,
    pub field_separator: String,
    pub encoding: Option<String>,
    pub header_row_count: u8,
}

impl CreateOrUpdateBankProfileRequest {
    /// Convertit la request vers `CsvProfile` (kesh-import) pour
    /// appliquer la validation métier complète.
    fn to_csv_profile(&self) -> Result<CsvProfile, AppError> {
        let field_separator = self.field_separator.chars().next().ok_or_else(|| {
            AppError::BankCsvProfileValidation("field_separator vide".to_string())
        })?;
        let decimal_separator = self.decimal_separator.chars().next().ok_or_else(|| {
            AppError::BankCsvProfileValidation("decimal_separator vide".to_string())
        })?;
        Ok(CsvProfile {
            bank_name: self.bank_name.clone(),
            filename_pattern: self.filename_pattern.clone(),
            column_mapping: self.column_mapping.clone(),
            date_format: self.date_format.clone(),
            decimal_separator,
            field_separator,
            encoding: self.encoding.clone(),
            header_row_count: self.header_row_count,
        })
    }

    fn to_new_bank_profile(&self) -> Result<NewBankProfile, AppError> {
        let column_mapping_json = serde_json::to_string(&self.column_mapping).map_err(|e| {
            AppError::BankCsvProfileValidation(format!("column_mapping serialization : {}", e))
        })?;
        Ok(NewBankProfile {
            bank_name: self.bank_name.clone(),
            filename_pattern: self.filename_pattern.clone(),
            column_mapping_json,
            date_format: self.date_format.clone(),
            decimal_separator: self.decimal_separator.clone(),
            field_separator: self.field_separator.clone(),
            encoding: self.encoding.clone(),
            header_row_count: self.header_row_count,
        })
    }
}

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Valide le payload via `CsvProfile::validate()` puis mappe les
/// `CsvError::ProfileMisconfigured(reason)` vers
/// `AppError::BankCsvProfileValidation(reason)`.
fn validate_request(req: &CreateOrUpdateBankProfileRequest) -> Result<NewBankProfile, AppError> {
    let csv_profile = req.to_csv_profile()?;
    csv_profile.validate().map_err(|e| match e {
        CsvError::ProfileMisconfigured(reason) => AppError::BankCsvProfileValidation(reason),
        other => AppError::BankCsvProfileValidation(other.to_string()),
    })?;
    req.to_new_bank_profile()
}

/// Mappe `DbError::UniqueConstraintViolation` → `BankCsvProfileDuplicate`.
fn map_db_error_to_app_error(err: DbError) -> AppError {
    match err {
        DbError::UniqueConstraintViolation(_) => AppError::BankCsvProfileDuplicate,
        DbError::NotFound => AppError::Database(DbError::NotFound),
        other => AppError::Database(other),
    }
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// POST /api/v1/bank-profiles — crée un profil (Comptable+).
pub async fn create(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateOrUpdateBankProfileRequest>,
) -> Result<(StatusCode, Json<BankProfileResponse>), AppError> {
    let new_profile = validate_request(&req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let profile = bank_profiles::create(
        &mut tx,
        current_user.company_id,
        &new_profile,
        current_user.user_id,
    )
    .await
    .map_err(map_db_error_to_app_error)?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok((
        StatusCode::CREATED,
        Json(BankProfileResponse::try_from_db(profile)?),
    ))
}

/// GET /api/v1/bank-profiles — liste paginée (tous rôles authentifiés).
pub async fn list(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ListResponse<BankProfileResponse>>, AppError> {
    let page = query.page.max(1);
    let per_page = query.per_page.clamp(1, 200);
    let offset = (page - 1) * per_page;

    let (profiles, total) =
        bank_profiles::list_by_company(&state.pool, current_user.company_id, per_page, offset)
            .await?;

    let items = profiles
        .into_iter()
        .map(BankProfileResponse::try_from_db)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ListResponse {
        items,
        total,
        limit: per_page,
        offset,
    }))
}

/// GET /api/v1/bank-profiles/{id} — détail (tous rôles).
pub async fn detail(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<BankProfileResponse>, AppError> {
    let profile = bank_profiles::find_by_id_for_company(&state.pool, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(BankProfileResponse::try_from_db(profile)?))
}

/// PUT /api/v1/bank-profiles/{id} — full replacement (Comptable+).
pub async fn update(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<CreateOrUpdateBankProfileRequest>,
) -> Result<Json<BankProfileResponse>, AppError> {
    let new_profile = validate_request(&req)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    let profile = bank_profiles::update(
        &mut tx,
        current_user.company_id,
        id,
        &new_profile,
        current_user.user_id,
    )
    .await
    .map_err(map_db_error_to_app_error)?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok(Json(BankProfileResponse::try_from_db(profile)?))
}

/// DELETE /api/v1/bank-profiles/{id} — supprime (Comptable+).
pub async fn delete(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;
    bank_profiles::delete(&mut tx, current_user.company_id, id, current_user.user_id)
        .await
        .map_err(map_db_error_to_app_error)?;
    tx.commit()
        .await
        .map_err(|e| AppError::Database(DbError::Sqlx(e)))?;

    Ok(StatusCode::NO_CONTENT)
}
