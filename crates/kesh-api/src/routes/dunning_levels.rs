//! Routes CRUD des niveaux de rappel (`dunning_levels`) — Epic 21, Story 21-3.
//!
//! Calqué sur `routes/vat.rs` : `Response` sans `company_id`, mutations Admin sous
//! sentinel lock + audit dans la même tx, verrou optimiste. GET ouvert à tous les
//! rôles authentifiés. Bornes frais 0..10'000 (scale 2), délai ≥ 0.

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::{DunningLevel, NewDunningLevel, UpdateDunningLevel};
use kesh_db::repositories::{audit_log, bank_accounts, dunning_levels};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::limits::scale_within;

const FEE_MAX: i64 = 10_000;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DunningLevelResponse {
    pub id: i64,
    pub level_number: i16,
    pub delay_days: i32,
    pub fee_amount: Decimal,
    pub version: i32,
}

impl From<DunningLevel> for DunningLevelResponse {
    fn from(l: DunningLevel) -> Self {
        Self {
            id: l.id,
            level_number: l.level_number,
            delay_days: l.delay_days,
            fee_amount: l.fee_amount,
            version: l.version,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDunningLevelBody {
    pub delay_days: i32,
    pub fee_amount: Decimal,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDunningLevelBody {
    pub delay_days: i32,
    pub fee_amount: Decimal,
    pub version: i32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeleteDunningLevelBody {
    pub version: i32,
}

fn validate_delay(delay_days: i32) -> Result<(), AppError> {
    if delay_days < 0 {
        return Err(AppError::Validation(
            "Le délai en jours ne peut être négatif.".into(),
        ));
    }
    Ok(())
}

fn validate_fee(fee: &Decimal) -> Result<(), AppError> {
    if *fee < Decimal::ZERO || *fee > Decimal::from(FEE_MAX) {
        return Err(AppError::Validation(format!(
            "Les frais de rappel doivent être compris entre 0 et {FEE_MAX}."
        )));
    }
    if !scale_within(fee, 2) {
        return Err(AppError::Validation(
            "Les frais de rappel ne peuvent avoir plus de 2 décimales.".into(),
        ));
    }
    Ok(())
}

/// `GET /api/v1/dunning-levels` — tous les niveaux (tout rôle authentifié).
pub async fn list_dunning_levels(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<Vec<DunningLevelResponse>>, AppError> {
    let levels = dunning_levels::list_all_by_company(&state.pool, current_user.company_id).await?;
    Ok(Json(levels.into_iter().map(Into::into).collect()))
}

/// `POST /api/v1/dunning-levels` — ajoute un niveau à la suite (Administrateur).
pub async fn create_dunning_level(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(body): Json<CreateDunningLevelBody>,
) -> Result<(StatusCode, Json<DunningLevelResponse>), AppError> {
    validate_delay(body.delay_days)?;
    validate_fee(&body.fee_amount)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    let new = NewDunningLevel {
        company_id: current_user.company_id,
        delay_days: body.delay_days,
        fee_amount: body.fee_amount,
    };
    let created = dunning_levels::create_for_company(&mut tx, &new).await?;

    let details = serde_json::json!({
        "dunning_level_id": created.id,
        "level_number": created.level_number,
        "delay_days": created.delay_days,
        "fee_amount": created.fee_amount.to_string(),
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "dunning_level.created",
            "dunning_level",
            created.id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok((StatusCode::CREATED, Json(created.into())))
}

/// `PUT /api/v1/dunning-levels/{id}` — modifie délai + frais (Administrateur).
pub async fn update_dunning_level(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateDunningLevelBody>,
) -> Result<Json<DunningLevelResponse>, AppError> {
    validate_delay(body.delay_days)?;
    validate_fee(&body.fee_amount)?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    let before = dunning_levels::find_by_id_for_company(&mut tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    let fields = UpdateDunningLevel {
        delay_days: body.delay_days,
        fee_amount: body.fee_amount,
    };
    let updated = dunning_levels::update_for_company(
        &mut tx,
        current_user.company_id,
        id,
        &fields,
        body.version,
    )
    .await?;

    let details = serde_json::json!({
        "before": { "delay_days": before.delay_days, "fee_amount": before.fee_amount.to_string(), "version": before.version },
        "after": { "delay_days": updated.delay_days, "fee_amount": updated.fee_amount.to_string(), "version": updated.version },
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "dunning_level.updated",
            "dunning_level",
            id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(Json(updated.into()))
}

/// `DELETE /api/v1/dunning-levels/{id}` — supprime + renumérote (Administrateur).
pub async fn delete_dunning_level(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(body): Json<DeleteDunningLevelBody>,
) -> Result<StatusCode, AppError> {
    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Internal(format!("begin tx: {e}")))?;
    bank_accounts::acquire_company_sentinel_lock(&mut tx, current_user.company_id).await?;

    let before = dunning_levels::find_by_id_for_company(&mut tx, current_user.company_id, id)
        .await?
        .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;

    dunning_levels::delete_and_renumber(&mut tx, current_user.company_id, id, body.version).await?;

    let details = serde_json::json!({
        "dunning_level_id": id,
        "level_number": before.level_number,
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::for_actor(
            current_user.user_id,
            current_user.api_key_id,
            "dunning_level.deleted",
            "dunning_level",
            id,
            Some(details),
        ),
    )
    .await?;

    tx.commit()
        .await
        .map_err(|e| AppError::Internal(format!("commit tx: {e}")))?;

    Ok(StatusCode::NO_CONTENT)
}
