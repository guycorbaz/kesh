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
    /// Story 33-1b (#926, C-e331-26) — les numéros de la facture, que dit l'écran.
    pub internal_number: Option<String>,
    pub supplier_invoice_number: Option<String>,
    pub position: i32,
    pub end_to_end_id: String,
    pub amount: Decimal,
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
    /// La réponse d'un lot : les numéros de ses factures viennent d'une seule lecture,
    /// scopée société (Story 33-1b, C-e331-26) ; `PaymentBatchItem` ne les porte pas.
    ///
    /// ⚠️ La lecture suit le commit du lot : son échec ne doit pas annoncer en échec une
    /// opération enregistrée, règlements compris. L'écran retombe alors sur `#id`.
    async fn from_parts(
        pool: &sqlx::MySqlPool,
        company_id: i64,
        b: PaymentBatch,
        items: Vec<PaymentBatchItem>,
    ) -> Self {
        let ids: Vec<i64> = items.iter().map(|i| i.supplier_invoice_id).collect();
        let mut numeros = payment_batches::invoice_numbers(pool, company_id, &ids)
            .await
            .unwrap_or_else(|e| {
                tracing::warn!(batch_id = b.id, error = %e, "numéros des factures d'un lot illisibles");
                Default::default()
            });
        let items = items
            .into_iter()
            .map(|i| {
                let (internal_number, supplier_invoice_number) = numeros
                    .remove(&i.supplier_invoice_id)
                    .unwrap_or((None, None));
                PaymentBatchItemResponse {
                    supplier_invoice_id: i.supplier_invoice_id,
                    internal_number,
                    supplier_invoice_number,
                    position: i.position,
                    end_to_end_id: i.end_to_end_id,
                    amount: i.amount,
                }
            })
            .collect();
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
            items,
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
///
/// ⚠️ **Rejouée sur interblocage** (Story 30-1a, #686, AC-a8) par l'enveloppe
/// `DbError` [`kesh_db::retry::retry_on_deadlock`], comme la confirmation : la
/// Story 30-1b ouvre un cycle entre une acceptation de plusieurs propositions
/// et la création d'un lot (verrous facture → compte → articles), dont la
/// victime finirait sinon en `500`. Rejeu **sûr** : tout se fait dans une
/// transaction, une tentative ne rend `Err` qu'annulée, aucun fichier n'est
/// écrit (le pain.001 est produit au `GET`) — aucun double lot.
///
/// **Borne** : plus de [`MAX_IN_CLAUSE_IDS`] identifiants, comptés sur la
/// longueur **brute** de `supplierInvoiceIds` (avant dédoublonnage), est un
/// `400 VALIDATION_ERROR`, avant toute lecture — le verrouillage de tête du
/// dépôt lie un paramètre par identifiant.
///
/// [`MAX_IN_CLAUSE_IDS`]: kesh_db::repositories::reconciliation::MAX_IN_CLAUSE_IDS
pub async fn create_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreatePaymentBatchRequest>,
) -> Result<(StatusCode, Json<CreatePaymentBatchResponse>), AppError> {
    let max = kesh_db::repositories::reconciliation::MAX_IN_CLAUSE_IDS;
    if req.supplier_invoice_ids.len() > max {
        return Err(AppError::Validation(format!(
            "un lot de paiement ne peut pas dépasser {max} factures"
        )));
    }
    let company = get_company_for(&current_user, &state.pool).await?;
    let new = kesh_db::entities::NewPaymentBatch {
        company_id: company.id,
        bank_account_id: req.bank_account_id,
        requested_execution_date: req.requested_execution_date,
        supplier_invoice_ids: req.supplier_invoice_ids,
    };
    let outcome = kesh_db::retry::retry_on_deadlock("payment_batches::create", || {
        payment_batches::create_batch(&state.pool, new.clone(), current_user.user_id)
    })
    .await?;

    let batch = match outcome.batch {
        Some(b) => {
            Some(PaymentBatchResponse::from_parts(&state.pool, company.id, b.batch, b.items).await)
        }
        None => None,
    };
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
    Ok(Json(
        PaymentBatchResponse::from_parts(&state.pool, company.id, b.batch, b.items).await,
    ))
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
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `DbError`
/// [`kesh_db::retry::retry_on_deadlock`].
pub async fn confirm_payment_batch(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<ConfirmPaymentBatchRequest>,
) -> Result<Json<PaymentBatchResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let b = kesh_db::retry::retry_on_deadlock("payment_batches::confirm", || {
        payment_batches::confirm_batch(
            &state.pool,
            company.id,
            id,
            req.payment_date,
            current_user.user_id,
        )
    })
    .await?;
    Ok(Json(
        PaymentBatchResponse::from_parts(&state.pool, company.id, b.batch, b.items).await,
    ))
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
    Ok(Json(
        PaymentBatchResponse::from_parts(&state.pool, company.id, b.batch, b.items).await,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    /// Revue P1 de la 33-1b (E-L1, A-F5) : la lecture des numéros suit le commit du lot. Un
    /// pool qui ne répond plus laisse la réponse entière, sans numéro : le lot est écrit.
    #[tokio::test]
    async fn an_unreadable_number_leaves_the_answer_whole() {
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .acquire_timeout(Duration::from_millis(50))
            .connect_lazy("mysql://kesh:x@127.0.0.1:1/kesh")
            .unwrap();
        let moment = NaiveDate::from_ymd_opt(2026, 6, 20)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let lot = PaymentBatch {
            id: 7,
            company_id: 1,
            bank_account_id: 2,
            status: "confirmed".into(),
            requested_execution_date: moment.date(),
            total_amount: Decimal::ONE,
            msg_id: "M".into(),
            payment_info_id: "P".into(),
            confirmed_at: Some(moment),
            version: 2,
            created_at: moment,
            updated_at: moment,
        };
        let ligne = PaymentBatchItem {
            id: 1,
            payment_batch_id: 7,
            supplier_invoice_id: 42,
            position: 1,
            end_to_end_id: "E".into(),
            amount: Decimal::ONE,
            created_at: moment,
        };
        let reponse = PaymentBatchResponse::from_parts(&pool, 1, lot, vec![ligne]).await;
        assert_eq!(reponse.status, "confirmed");
        assert_eq!(reponse.items[0].supplier_invoice_id, 42);
        assert_eq!(reponse.items[0].internal_number, None);
        assert_eq!(reponse.items[0].supplier_invoice_number, None);
    }
}
