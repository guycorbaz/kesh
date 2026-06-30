//! Story 12.2 — endpoints factures fournisseurs & règlement binaire (#191).
//!
//! - `GET    /api/v1/supplier-invoices`          liste paginée (tout rôle)
//! - `GET    /api/v1/supplier-invoices/{id}`     détail + lignes (tout rôle)
//! - `POST   /api/v1/supplier-invoices`          enregistrer (Comptable+, RBAC routing)
//! - `POST   /api/v1/supplier-invoices/{id}/pay` régler binaire (Comptable+)
//! - `POST   /api/v1/supplier-invoices/{id}/cancel` annuler (Comptable+)
//!
//! Erreurs métier (config absente, déjà payée, compte étranger/inactif, etc.)
//! → `AppError` typée 4xx, jamais 500 (AC5/AC9).

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::{SettlementChoice, SupplierInvoice, SupplierInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices;
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
pub struct SupplierInvoiceLineResponse {
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    pub expense_account_id: i64,
}

impl From<SupplierInvoiceLine> for SupplierInvoiceLineResponse {
    fn from(l: SupplierInvoiceLine) -> Self {
        Self {
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
            expense_account_id: l.expense_account_id,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceResponse {
    pub id: i64,
    pub contact_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total_amount: Decimal,
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    pub purchase_journal_entry_id: i64,
    pub settlement_type: Option<String>,
    pub settlement_bank_account_id: Option<i64>,
    pub settlement_account_id: Option<i64>,
    pub settlement_journal_entry_id: Option<i64>,
    pub paid_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub lines: Vec<SupplierInvoiceLineResponse>,
}

impl SupplierInvoiceResponse {
    /// `pub` (Story 12-5c) : appelé par le module `imported_supplier_invoices`
    /// pour construire la réponse de complétion à partir du `SupplierInvoiceWithLines`
    /// retourné par `create_in_tx` (DRY — pas de redéfinition cross-module).
    pub fn from_parts(inv: SupplierInvoice, lines: Vec<SupplierInvoiceLine>) -> Self {
        Self {
            id: inv.id,
            contact_id: inv.contact_id,
            supplier_invoice_number: inv.supplier_invoice_number,
            status: inv.status,
            invoice_date: inv.invoice_date,
            due_date: inv.due_date,
            total_amount: inv.total_amount,
            creditor_iban: inv.creditor_iban,
            creditor_qr_iban: inv.creditor_qr_iban,
            payment_reference: inv.payment_reference,
            expected_payment_amount: inv.expected_payment_amount,
            purchase_journal_entry_id: inv.purchase_journal_entry_id,
            settlement_type: inv.settlement_type,
            settlement_bank_account_id: inv.settlement_bank_account_id,
            settlement_account_id: inv.settlement_account_id,
            settlement_journal_entry_id: inv.settlement_journal_entry_id,
            paid_at: inv.paid_at,
            version: inv.version,
            created_at: inv.created_at,
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceListItemResponse {
    pub id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub supplier_invoice_number: Option<String>,
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub total_amount: Decimal,
}

impl From<kesh_db::repositories::supplier_invoices::SupplierInvoiceListItem>
    for SupplierInvoiceListItemResponse
{
    fn from(it: kesh_db::repositories::supplier_invoices::SupplierInvoiceListItem) -> Self {
        Self {
            id: it.id,
            contact_id: it.contact_id,
            contact_name: it.contact_name,
            supplier_invoice_number: it.supplier_invoice_number,
            status: it.status,
            invoice_date: it.invoice_date,
            due_date: it.due_date,
            total_amount: it.total_amount,
        }
    }
}

// ---------------------------------------------------------------------------
// Requêtes
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierInvoiceLineRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub expense_account_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierInvoiceRequest {
    pub contact_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    pub lines: Vec<CreateSupplierInvoiceLineRequest>,
}

/// Requête de règlement binaire. `settlementType` = `bank_transfer` (→ `bankAccountId`)
/// ou `internal_account` (→ `accountId`).
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaySupplierInvoiceRequest {
    pub settlement_type: String,
    pub bank_account_id: Option<i64>,
    pub account_id: Option<i64>,
    pub payment_date: NaiveDate,
}

impl PaySupplierInvoiceRequest {
    fn into_choice(self) -> Result<(SettlementChoice, NaiveDate), AppError> {
        let choice = match self.settlement_type.as_str() {
            "bank_transfer" => {
                let bank_account_id = self.bank_account_id.ok_or_else(|| {
                    AppError::Validation("bankAccountId requis pour un virement".into())
                })?;
                SettlementChoice::BankTransfer { bank_account_id }
            }
            "internal_account" => {
                let account_id = self.account_id.ok_or_else(|| {
                    AppError::Validation("accountId requis pour un compte interne".into())
                })?;
                SettlementChoice::InternalAccount { account_id }
            }
            other => {
                return Err(AppError::Validation(format!(
                    "settlementType inconnu : '{other}' (attendu bank_transfer | internal_account)"
                )));
            }
        };
        Ok((choice, self.payment_date))
    }
}

#[derive(Deserialize)]
pub struct ListSupplierInvoicesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

/// `GET /api/v1/supplier-invoices` — liste paginée (les plus récentes d'abord).
pub async fn list_supplier_invoices(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListSupplierInvoicesQuery>,
) -> Result<Json<ListResponse<SupplierInvoiceListItemResponse>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = supplier_invoices::list(&state.pool, company.id, limit, offset).await?;
    Ok(Json(ListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        offset,
        limit,
    }))
}

/// `GET /api/v1/supplier-invoices/{id}` — détail + lignes.
pub async fn get_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (inv, lines) = supplier_invoices::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(SupplierInvoiceResponse::from_parts(inv, lines)))
}

/// `POST /api/v1/supplier-invoices` — enregistre une facture fournisseur (Comptable+).
pub async fn create_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateSupplierInvoiceRequest>,
) -> Result<(StatusCode, Json<SupplierInvoiceResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let new = kesh_db::entities::NewSupplierInvoice {
        company_id: company.id,
        contact_id: req.contact_id,
        supplier_invoice_number: req.supplier_invoice_number,
        invoice_date: req.invoice_date,
        due_date: req.due_date,
        creditor_iban: req.creditor_iban,
        creditor_qr_iban: req.creditor_qr_iban,
        payment_reference: req.payment_reference,
        expected_payment_amount: req.expected_payment_amount,
        lines: req
            .lines
            .into_iter()
            .map(|l| kesh_db::entities::NewSupplierInvoiceLine {
                description: l.description,
                quantity: l.quantity,
                unit_price: l.unit_price,
                vat_rate: l.vat_rate,
                expense_account_id: l.expense_account_id,
            })
            .collect(),
    };
    let created = supplier_invoices::create(&state.pool, new, current_user.user_id).await?;
    Ok((
        StatusCode::CREATED,
        Json(SupplierInvoiceResponse::from_parts(
            created.invoice,
            created.lines,
        )),
    ))
}

/// `POST /api/v1/supplier-invoices/{id}/pay` — règlement binaire (Comptable+).
pub async fn pay_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<PaySupplierInvoiceRequest>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (choice, payment_date) = req.into_choice()?;
    let paid = supplier_invoices::pay(
        &state.pool,
        company.id,
        id,
        choice,
        payment_date,
        current_user.user_id,
    )
    .await?;
    Ok(Json(SupplierInvoiceResponse::from_parts(
        paid.invoice,
        paid.lines,
    )))
}

/// `POST /api/v1/supplier-invoices/{id}/cancel` — annule une facture `open` (Comptable+).
pub async fn cancel_supplier_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let cancelled =
        supplier_invoices::cancel(&state.pool, company.id, id, current_user.user_id).await?;
    Ok(Json(SupplierInvoiceResponse::from_parts(
        cancelled.invoice,
        cancelled.lines,
    )))
}
