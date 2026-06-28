//! Story 12.3 — endpoints lots de paiement pain.001 (mode virement, #191).
//!
//! - `POST   /api/v1/payment-batches`            créer un lot (Comptable+, `{batch, failed}`)
//! - `GET    /api/v1/payment-batches`            liste paginée (tout rôle)
//! - `GET    /api/v1/payment-batches/{id}`       détail + items (tout rôle)
//! - `GET    /api/v1/payment-batches/{id}/pain001` télécharger le fichier XML (tout rôle)
//! - `POST   /api/v1/payment-batches/{id}/confirm` confirmer (Comptable+)
//! - `POST   /api/v1/payment-batches/{id}/cancel`  annuler (Comptable+)

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::{PaymentBatch, PaymentBatchItem};
use kesh_db::errors::DbError;
use kesh_db::repositories::payment_batches;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;

// ---------------------------------------------------------------------------
// Réponses
// ---------------------------------------------------------------------------

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchItemResponse {
    pub supplier_invoice_id: i64,
    pub position: i32,
    pub end_to_end_id: String,
    pub amount: Decimal,
}

impl From<PaymentBatchItem> for PaymentBatchItemResponse {
    fn from(i: PaymentBatchItem) -> Self {
        Self {
            supplier_invoice_id: i.supplier_invoice_id,
            position: i.position,
            end_to_end_id: i.end_to_end_id,
            amount: i.amount,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchResponse {
    pub id: i64,
    pub bank_account_id: i64,
    pub status: String,
    pub requested_execution_date: NaiveDate,
    pub total_amount: Decimal,
    pub msg_id: String,
    pub confirmed_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub items: Vec<PaymentBatchItemResponse>,
}

impl PaymentBatchResponse {
    fn from_parts(b: PaymentBatch, items: Vec<PaymentBatchItem>) -> Self {
        Self {
            id: b.id,
            bank_account_id: b.bank_account_id,
            status: b.status,
            requested_execution_date: b.requested_execution_date,
            total_amount: b.total_amount,
            msg_id: b.msg_id,
            confirmed_at: b.confirmed_at,
            version: b.version,
            created_at: b.created_at,
            items: items.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchListItemResponse {
    pub id: i64,
    pub bank_account_id: i64,
    pub status: String,
    pub requested_execution_date: NaiveDate,
    pub total_amount: Decimal,
    pub created_at: NaiveDateTime,
}

impl From<PaymentBatch> for PaymentBatchListItemResponse {
    fn from(b: PaymentBatch) -> Self {
        Self {
            id: b.id,
            bank_account_id: b.bank_account_id,
            status: b.status,
            requested_execution_date: b.requested_execution_date,
            total_amount: b.total_amount,
            created_at: b.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchFailedItemResponse {
    pub supplier_invoice_id: i64,
    pub error_code: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentBatchResponse {
    /// Le lot créé, ou `null` si aucune facture n'a été acceptée.
    pub batch: Option<PaymentBatchResponse>,
    pub failed: Vec<PaymentBatchFailedItemResponse>,
}

// ---------------------------------------------------------------------------
// Requêtes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePaymentBatchRequest {
    pub bank_account_id: i64,
    pub requested_execution_date: NaiveDate,
    pub supplier_invoice_ids: Vec<i64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmPaymentBatchRequest {
    pub payment_date: NaiveDate,
}

#[derive(Deserialize)]
pub struct ListPaymentBatchesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `POST /api/v1/payment-batches` — créer un lot (Comptable+). `{batch, failed}`.
pub async fn create_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePaymentBatchRequest>,
) -> Result<(StatusCode, Json<CreatePaymentBatchResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let outcome = payment_batches::create_batch(
        &state.pool,
        kesh_db::entities::NewPaymentBatch {
            company_id: company.id,
            bank_account_id: req.bank_account_id,
            requested_execution_date: req.requested_execution_date,
            supplier_invoice_ids: req.supplier_invoice_ids,
        },
        current_user.user_id,
    )
    .await?;

    let batch = outcome
        .batch
        .map(|b| PaymentBatchResponse::from_parts(b.batch, b.items));
    let failed = outcome
        .failed
        .into_iter()
        .map(|f| PaymentBatchFailedItemResponse {
            supplier_invoice_id: f.supplier_invoice_id,
            error_code: f.error_code,
            details: f.details,
        })
        .collect();
    Ok((
        StatusCode::OK,
        Json(CreatePaymentBatchResponse { batch, failed }),
    ))
}

/// `GET /api/v1/payment-batches` — liste paginée.
pub async fn list_payment_batches(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListPaymentBatchesQuery>,
) -> Result<Json<ListResponse<PaymentBatchListItemResponse>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = payment_batches::list(&state.pool, company.id, limit, offset).await?;
    Ok(Json(ListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        offset,
        limit,
    }))
}

/// `GET /api/v1/payment-batches/{id}` — détail + items.
pub async fn get_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<PaymentBatchResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let b = payment_batches::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(PaymentBatchResponse::from_parts(b.batch, b.items)))
}

/// `GET /api/v1/payment-batches/{id}/pain001` — télécharger le fichier XML.
pub async fn get_payment_batch_pain001(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let xml = payment_batches::generate_pain001_xml(&state.pool, company.id, id).await?;
    let filename = format!("pain001-{id}.xml");
    let mut resp = xml.into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/xml; charset=utf-8"),
    );
    if let Ok(cd) = HeaderValue::from_str(&format!("attachment; filename=\"{filename}\"")) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, cd);
    }
    Ok(resp)
}

/// `POST /api/v1/payment-batches/{id}/confirm` — confirmer (Comptable+).
pub async fn confirm_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ConfirmPaymentBatchRequest>,
) -> Result<Json<PaymentBatchResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let b = payment_batches::confirm_batch(
        &state.pool,
        company.id,
        id,
        req.payment_date,
        current_user.user_id,
    )
    .await?;
    Ok(Json(PaymentBatchResponse::from_parts(b.batch, b.items)))
}

/// `POST /api/v1/payment-batches/{id}/cancel` — annuler (Comptable+).
pub async fn cancel_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<PaymentBatchResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let b =
        payment_batches::cancel_batch(&state.pool, company.id, id, current_user.user_id).await?;
    Ok(Json(PaymentBatchResponse::from_parts(b.batch, b.items)))
}
