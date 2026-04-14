//! Routes CRUD pour les factures brouillon (Story 5.1 — FR31, FR32).
//!
//! - GETs (`list`, `get`) → `authenticated_routes` (tout rôle).
//! - Mutations (`create`, `update`, `delete`) → `comptable_routes`.
//!
//! `total_amount` est recalculé par le repository à partir des lignes —
//! le frontend peut l'afficher en temps réel, mais la valeur persistée
//! est celle du backend.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::{Extension, Json};
use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

use kesh_core::listing::SortDirection;
use kesh_db::entities::invoice::{Invoice, InvoiceLine, InvoiceUpdate, NewInvoice, NewInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{
    companies, contacts,
    invoices::{self, InvoiceListItem, InvoiceListQuery, InvoiceSortBy},
};

use crate::AppState;
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::limits::{
    MAX_DECIMAL_SCALE, MAX_LINE_TOTAL, MAX_QUANTITY, MAX_UNIT_PRICE, scale_within,
};
use crate::routes::vat::validate_vat_rate;

// ---------------------------------------------------------------------------
// Limites
// ---------------------------------------------------------------------------

const MAX_DESCRIPTION_LEN: usize = 1000;
const MAX_PAYMENT_TERMS_LEN: usize = 255;
const MAX_LIST_LIMIT: i64 = 100;
const DEFAULT_LIST_LIMIT: i64 = 20;
const MAX_SEARCH_LEN: usize = 100;
/// Cap lignes par facture : anti-DoS (N+1 INSERT dans transaction) + anti-overflow
/// `total_amount` (avec MAX_LINE_TOTAL = 10¹², 200 lignes = 2·10¹⁴ < DECIMAL(19,4) max).
const MAX_LINES: usize = 200;

const VAT_ERROR_MSG: &str = "Taux TVA non autorisé. Valeurs acceptées : 0.00%, 2.60%, 3.80%, 8.10%";

// ---------------------------------------------------------------------------
// DTOs
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceLineRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvoiceRequest {
    pub contact_id: i64,
    pub date: NaiveDate,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub payment_terms: Option<String>,
    pub lines: Vec<CreateInvoiceLineRequest>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvoiceRequest {
    pub contact_id: i64,
    pub date: NaiveDate,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub payment_terms: Option<String>,
    pub lines: Vec<CreateInvoiceLineRequest>,
    pub version: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListInvoicesQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub contact_id: Option<i64>,
    #[serde(default)]
    pub date_from: Option<NaiveDate>,
    #[serde(default)]
    pub date_to: Option<NaiveDate>,
    #[serde(default)]
    pub sort_by: Option<InvoiceSortBy>,
    #[serde(default)]
    pub sort_direction: Option<SortDirection>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub offset: Option<i64>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLineResponse {
    pub id: i64,
    pub invoice_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    pub created_at: NaiveDateTime,
}

impl From<InvoiceLine> for InvoiceLineResponse {
    fn from(l: InvoiceLine) -> Self {
        Self {
            id: l.id,
            invoice_id: l.invoice_id,
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
            created_at: l.created_at,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceResponse {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub journal_entry_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub lines: Vec<InvoiceLineResponse>,
}

impl InvoiceResponse {
    fn from_parts(invoice: Invoice, lines: Vec<InvoiceLine>) -> Self {
        Self {
            id: invoice.id,
            company_id: invoice.company_id,
            contact_id: invoice.contact_id,
            invoice_number: invoice.invoice_number,
            status: invoice.status,
            date: invoice.date,
            due_date: invoice.due_date,
            payment_terms: invoice.payment_terms,
            total_amount: invoice.total_amount,
            journal_entry_id: invoice.journal_entry_id,
            version: invoice.version,
            created_at: invoice.created_at,
            updated_at: invoice.updated_at,
            lines: lines.into_iter().map(InvoiceLineResponse::from).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListItemResponse {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl From<InvoiceListItem> for InvoiceListItemResponse {
    fn from(i: InvoiceListItem) -> Self {
        Self {
            id: i.id,
            company_id: i.company_id,
            contact_id: i.contact_id,
            contact_name: i.contact_name,
            invoice_number: i.invoice_number,
            status: i.status,
            date: i.date,
            due_date: i.due_date,
            payment_terms: i.payment_terms,
            total_amount: i.total_amount,
            version: i.version,
            created_at: i.created_at,
            updated_at: i.updated_at,
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async fn get_company(state: &AppState) -> Result<kesh_db::entities::Company, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;
    list.into_iter()
        .next()
        .ok_or_else(|| AppError::Internal("Aucune company en base".into()))
}

fn normalize_optional(s: Option<String>) -> Option<String> {
    s.and_then(|v| {
        let t: String = v.trim().nfc().collect();
        if t.is_empty() { None } else { Some(t) }
    })
}

fn validate_line(req: CreateInvoiceLineRequest, index: usize) -> Result<NewInvoiceLine, AppError> {
    let description: String = req.description.trim().nfc().collect();
    if description.is_empty() {
        return Err(AppError::Validation(format!(
            "Ligne {} : la description est obligatoire",
            index + 1
        )));
    }
    if description.chars().count() > MAX_DESCRIPTION_LEN {
        return Err(AppError::Validation(format!(
            "Ligne {} : description trop longue (max {MAX_DESCRIPTION_LEN})",
            index + 1
        )));
    }

    if req.quantity <= Decimal::ZERO {
        return Err(AppError::Validation(format!(
            "Ligne {} : la quantité doit être strictement positive",
            index + 1
        )));
    }
    if !scale_within(&req.quantity, MAX_DECIMAL_SCALE) {
        return Err(AppError::Validation(format!(
            "Ligne {} : la quantité doit avoir au plus {MAX_DECIMAL_SCALE} décimales",
            index + 1
        )));
    }
    if req.quantity > *MAX_QUANTITY {
        return Err(AppError::Validation(format!(
            "Ligne {} : quantité trop élevée",
            index + 1
        )));
    }

    if req.unit_price < Decimal::ZERO {
        return Err(AppError::Validation(format!(
            "Ligne {} : le prix unitaire doit être positif ou nul",
            index + 1
        )));
    }
    if !scale_within(&req.unit_price, MAX_DECIMAL_SCALE) {
        return Err(AppError::Validation(format!(
            "Ligne {} : le prix unitaire doit avoir au plus {MAX_DECIMAL_SCALE} décimales",
            index + 1
        )));
    }
    if req.unit_price > *MAX_UNIT_PRICE {
        return Err(AppError::Validation(format!(
            "Ligne {} : prix unitaire trop élevé",
            index + 1
        )));
    }

    if !validate_vat_rate(&req.vat_rate) {
        return Err(AppError::Validation(VAT_ERROR_MSG.into()));
    }

    // Anti-overflow : `qty × unit_price` doit rester sous `MAX_LINE_TOTAL`
    // pour garantir que `Σ line_total` sur MAX_LINES tient dans DECIMAL(19,4).
    let line_total = req.quantity * req.unit_price;
    if line_total > *MAX_LINE_TOTAL {
        return Err(AppError::Validation(format!(
            "Ligne {} : total de ligne trop élevé",
            index + 1
        )));
    }

    Ok(NewInvoiceLine {
        description,
        quantity: req.quantity,
        unit_price: req.unit_price,
        vat_rate: req.vat_rate,
    })
}

fn validate_payment_terms(pt: Option<String>) -> Result<Option<String>, AppError> {
    let norm = normalize_optional(pt);
    if let Some(ref s) = norm {
        if s.chars().count() > MAX_PAYMENT_TERMS_LEN {
            return Err(AppError::Validation(format!(
                "Les conditions de paiement doivent faire au plus {MAX_PAYMENT_TERMS_LEN} caractères"
            )));
        }
    }
    Ok(norm)
}

async fn ensure_contact_belongs_to_company(
    state: &AppState,
    contact_id: i64,
    company_id: i64,
) -> Result<(), AppError> {
    let contact = contacts::find_by_id(&state.pool, contact_id).await?;
    match contact {
        None => Err(AppError::Validation("Contact introuvable".into())),
        Some(c) if c.company_id != company_id => {
            Err(AppError::Validation("Contact introuvable".into()))
        }
        Some(c) if !c.active => Err(AppError::Validation("Contact archivé".into())),
        Some(_) => Ok(()),
    }
}

fn validate_lines(reqs: Vec<CreateInvoiceLineRequest>) -> Result<Vec<NewInvoiceLine>, AppError> {
    if reqs.is_empty() {
        return Err(AppError::Validation(
            "Une facture doit contenir au moins une ligne".into(),
        ));
    }
    if reqs.len() > MAX_LINES {
        return Err(AppError::Validation(format!(
            "Une facture doit contenir au plus {MAX_LINES} lignes"
        )));
    }
    reqs.into_iter()
        .enumerate()
        .map(|(i, l)| validate_line(l, i))
        .collect()
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

pub async fn list_invoices(
    State(state): State<AppState>,
    Query(params): Query<ListInvoicesQuery>,
) -> Result<Json<ListResponse<InvoiceListItemResponse>>, AppError> {
    let company = get_company(&state).await?;

    let limit = params.limit.unwrap_or(DEFAULT_LIST_LIMIT);
    if !(1..=MAX_LIST_LIMIT).contains(&limit) {
        return Err(AppError::Validation(format!(
            "limit doit être compris entre 1 et {MAX_LIST_LIMIT}"
        )));
    }
    let offset = params.offset.unwrap_or(0);
    if offset < 0 {
        return Err(AppError::Validation(
            "offset doit être positif ou nul".into(),
        ));
    }

    // Normalisation search : trim côté handler, collapse empty → None (cohérent payment_terms).
    let search = match params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        Some(s) if s.chars().count() > MAX_SEARCH_LEN => {
            return Err(AppError::Validation(format!(
                "search doit faire au plus {MAX_SEARCH_LEN} caractères"
            )));
        }
        Some(s) => Some(s.to_string()),
        None => None,
    };

    if let Some(ref st) = params.status {
        if !matches!(st.as_str(), "draft" | "validated" | "cancelled") {
            return Err(AppError::Validation("status invalide".into()));
        }
    }

    if let (Some(df), Some(dt)) = (params.date_from, params.date_to) {
        if df > dt {
            return Err(AppError::Validation(
                "dateFrom doit être antérieur ou égal à dateTo".into(),
            ));
        }
    }

    let query = InvoiceListQuery {
        search,
        status: params.status,
        contact_id: params.contact_id,
        date_from: params.date_from,
        date_to: params.date_to,
        sort_by: params.sort_by.unwrap_or_default(),
        sort_direction: params.sort_direction.unwrap_or(SortDirection::Desc),
        limit,
        offset,
    };

    let result = invoices::list_by_company_paginated(&state.pool, company.id, query).await?;
    Ok(Json(ListResponse {
        items: result
            .items
            .into_iter()
            .map(InvoiceListItemResponse::from)
            .collect(),
        total: result.total,
        limit: result.limit,
        offset: result.offset,
    }))
}

pub async fn get_invoice(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company(&state).await?;
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(InvoiceResponse::from_parts(invoice, lines)))
}

pub async fn create_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<InvoiceResponse>), AppError> {
    let company = get_company(&state).await?;

    ensure_contact_belongs_to_company(&state, req.contact_id, company.id).await?;
    let lines = validate_lines(req.lines)?;
    let payment_terms = validate_payment_terms(req.payment_terms)?;

    let new = NewInvoice {
        company_id: company.id,
        contact_id: req.contact_id,
        date: req.date,
        due_date: req.due_date,
        payment_terms,
        lines,
    };

    let (invoice, persisted_lines) =
        invoices::create(&state.pool, current_user.user_id, new).await?;
    Ok((
        StatusCode::CREATED,
        Json(InvoiceResponse::from_parts(invoice, persisted_lines)),
    ))
}

pub async fn update_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<UpdateInvoiceRequest>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company(&state).await?;

    ensure_contact_belongs_to_company(&state, req.contact_id, company.id).await?;
    let lines = validate_lines(req.lines)?;
    let payment_terms = validate_payment_terms(req.payment_terms)?;

    let changes = InvoiceUpdate {
        contact_id: req.contact_id,
        date: req.date,
        due_date: req.due_date,
        payment_terms,
        lines,
    };

    let (invoice, persisted_lines) = invoices::update(
        &state.pool,
        company.id,
        id,
        req.version,
        current_user.user_id,
        changes,
    )
    .await?;
    Ok(Json(InvoiceResponse::from_parts(invoice, persisted_lines)))
}

pub async fn delete_invoice(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let company = get_company(&state).await?;
    invoices::delete(&state.pool, company.id, id, current_user.user_id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/v1/invoices/:id/validate` (Story 5.2 — comptable_routes).
///
/// Transition atomique `draft → validated` : attribue un numéro,
/// génère l'écriture comptable, persiste. Renvoie la facture
/// validée (incluant `invoiceNumber`, `journalEntryId`, statut).
pub async fn validate_invoice_handler(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company(&state).await?;
    let _validated =
        invoices::validate_invoice(&state.pool, company.id, id, current_user.user_id).await?;

    // Recharger avec lignes pour la réponse (cohérent avec get_invoice).
    let (invoice, lines) = invoices::find_by_id_with_lines(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(InvoiceResponse::from_parts(invoice, lines)))
}

// ---------------------------------------------------------------------------
// Tests unitaires
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn line(desc: &str, qty: Decimal, price: Decimal, vat: Decimal) -> CreateInvoiceLineRequest {
        CreateInvoiceLineRequest {
            description: desc.into(),
            quantity: qty,
            unit_price: price,
            vat_rate: vat,
        }
    }

    #[test]
    fn validate_lines_rejects_empty() {
        let err = validate_lines(vec![]).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_zero_quantity() {
        let err = validate_line(line("X", dec!(0), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_negative_quantity() {
        let err = validate_line(line("X", dec!(-1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_empty_description() {
        let err = validate_line(line("   ", dec!(1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_description_too_long() {
        let long = "a".repeat(1001);
        let err = validate_line(line(&long, dec!(1), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_bad_vat() {
        let err = validate_line(line("X", dec!(1), dec!(10), dec!(99.99)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_accepts_valid_vats() {
        for v in ["0.00", "2.60", "3.80", "8.10"] {
            let d: Decimal = v.parse().unwrap();
            assert!(validate_line(line("X", dec!(1), dec!(10), d), 0).is_ok());
        }
    }

    #[test]
    fn validate_line_rejects_scale_above_four() {
        let err = validate_line(line("X", dec!(1.12345), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        let err = validate_line(line("X", dec!(1), dec!(10.12345), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_rejects_over_caps() {
        let err = validate_line(line("X", dec!(1000001), dec!(10), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
        let err =
            validate_line(line("X", dec!(1), dec!(1000000000.0001), dec!(8.10)), 0).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }

    #[test]
    fn validate_line_normalizes_description_to_nfc() {
        // "é" NFD → doit devenir NFC.
        let nfd = "Caf\u{0065}\u{0301}";
        let v = validate_line(line(nfd, dec!(1), dec!(10), dec!(8.10)), 0).unwrap();
        assert_eq!(v.description, "Caf\u{00E9}");
    }

    #[test]
    fn validate_payment_terms_trims_and_collapses_empty() {
        assert_eq!(validate_payment_terms(None).unwrap(), None);
        assert_eq!(validate_payment_terms(Some("  ".into())).unwrap(), None);
        assert_eq!(
            validate_payment_terms(Some("  30 j  ".into())).unwrap(),
            Some("30 j".into())
        );
    }

    #[test]
    fn validate_payment_terms_rejects_too_long() {
        let long = "a".repeat(256);
        let err = validate_payment_terms(Some(long)).unwrap_err();
        assert!(matches!(err, AppError::Validation(_)));
    }
}
