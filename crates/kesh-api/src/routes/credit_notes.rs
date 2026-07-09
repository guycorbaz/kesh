//! Story 12.1 — endpoints avoirs (notes de crédit).
//!
//! - `GET    /api/v1/credit-notes`        liste paginée (tout rôle)
//! - `POST   /api/v1/credit-notes`        create+issue (Comptable+, RBAC au routing)
//! - `GET    /api/v1/credit-notes/{id}`   détail + lignes (tout rôle)
//! - `GET    /api/v1/credit-notes/{id}/pdf` PDF « Avoir » sans QR Bill (tout rôle)

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::{NaiveDate, NaiveDateTime};
use kesh_db::entities::{CreditNote, CreditNoteLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::{contacts, credit_notes};
use kesh_qrbill::{Currency, InvoiceLinePdf, InvoicePdfData};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::middleware::auth::CurrentUser;
use crate::routes::ListResponse;
use crate::routes::invoice_pdf_service::{
    MAX_LINES_PER_PDF, build_i18n, map_qrbill_error, sanitize_filename, split_lines,
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteLineResponse {
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
}

impl From<CreditNoteLine> for CreditNoteLineResponse {
    fn from(l: CreditNoteLine) -> Self {
        Self {
            position: l.position,
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteResponse {
    pub id: i64,
    pub contact_id: i64,
    pub invoice_id: i64,
    pub credit_note_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub total_amount: Decimal,
    pub journal_entry_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub lines: Vec<CreditNoteLineResponse>,
}

impl CreditNoteResponse {
    fn from_parts(cn: CreditNote, lines: Vec<CreditNoteLine>) -> Self {
        Self {
            id: cn.id,
            contact_id: cn.contact_id,
            invoice_id: cn.invoice_id,
            credit_note_number: cn.credit_note_number,
            status: cn.status,
            date: cn.date,
            total_amount: cn.total_amount,
            journal_entry_id: cn.journal_entry_id,
            version: cn.version,
            created_at: cn.created_at,
            lines: lines.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreditNoteListItemResponse {
    pub id: i64,
    pub contact_id: i64,
    pub invoice_id: i64,
    pub credit_note_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub total_amount: Decimal,
}

impl From<CreditNote> for CreditNoteListItemResponse {
    fn from(cn: CreditNote) -> Self {
        Self {
            id: cn.id,
            contact_id: cn.contact_id,
            invoice_id: cn.invoice_id,
            credit_note_number: cn.credit_note_number,
            status: cn.status,
            date: cn.date,
            total_amount: cn.total_amount,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateCreditNoteRequest {
    pub invoice_id: i64,
    pub date: NaiveDate,
}

#[derive(Deserialize)]
pub struct ListCreditNotesQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// `GET /api/v1/credit-notes` — liste paginée (les plus récents d'abord).
pub async fn list_credit_notes(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListCreditNotesQuery>,
) -> Result<Json<ListResponse<CreditNoteListItemResponse>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let limit = params.limit.unwrap_or(50).clamp(1, 200);
    let offset = params.offset.unwrap_or(0).max(0);
    let (items, total) = credit_notes::list(&state.pool, company.id, limit, offset).await?;
    Ok(Json(ListResponse {
        items: items.into_iter().map(Into::into).collect(),
        total,
        offset,
        limit,
    }))
}

/// `GET /api/v1/credit-notes/{id}` — détail + lignes.
pub async fn get_credit_note(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<CreditNoteResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (cn, lines) = credit_notes::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    Ok(Json(CreditNoteResponse::from_parts(cn, lines)))
}

/// `POST /api/v1/credit-notes` — crée et émet un avoir (Comptable+, RBAC routing).
pub async fn create_credit_note(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateCreditNoteRequest>,
) -> Result<(StatusCode, Json<CreditNoteResponse>), AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let issued = credit_notes::create_credit_note(
        &state.pool,
        kesh_db::entities::NewCreditNote {
            company_id: company.id,
            invoice_id: req.invoice_id,
            date: req.date,
        },
        current_user.user_id,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(CreditNoteResponse::from_parts(
            issued.credit_note,
            issued.lines,
        )),
    ))
}

/// `GET /api/v1/credit-notes/{id}/pdf` — PDF « Avoir » (sans QR Bill).
pub async fn get_credit_note_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let (cn, lines) = credit_notes::get(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;

    if lines.len() > MAX_LINES_PER_PDF {
        return Err(AppError::InvoiceTooManyLinesForPdf(lines.len()));
    }

    let contact = contacts::find_by_id(&state.pool, cn.contact_id)
        .await?
        .ok_or_else(|| {
            AppError::InvoiceNotPdfReady(crate::errors::t(
                "invoice-pdf-error-contact-missing",
                "Le contact lié à l'avoir est introuvable.",
            ))
        })?;

    // Numéro de la facture d'origine (pour la référence affichée).
    let origin_number: Option<String> =
        sqlx::query_scalar("SELECT invoice_number FROM invoices WHERE id = ? AND company_id = ?")
            .bind(cn.invoice_id)
            .bind(company.id)
            .fetch_optional(&state.pool)
            .await
            .map_err(|e| AppError::Database(kesh_db::errors::map_db_error(e)))?
            .flatten();

    let pdf_lines: Vec<InvoiceLinePdf> = lines
        .iter()
        .map(|l| InvoiceLinePdf {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            line_total: l.line_total,
        })
        .collect();

    // TTC = HT + TVA (cohérent avec la contre-passation).
    let ttc: Decimal = lines
        .iter()
        .map(|l| {
            l.line_total + kesh_core::accounting::vat::line_vat_amount(l.line_total, l.vat_rate)
        })
        .sum();

    let pdf_data = InvoicePdfData {
        invoice_number: cn
            .credit_note_number
            .clone()
            .unwrap_or_else(|| format!("#{}", cn.id)),
        invoice_date: cn.date,
        due_date: None,
        payment_terms: None,
        creditor_name: company.name.clone(),
        creditor_address_lines: split_lines(&company.address),
        creditor_ide: company.ide_number.clone(),
        debtor_name: contact.name.clone(),
        debtor_address_lines: split_lines(contact.address.as_deref().unwrap_or("")),
        lines: pdf_lines,
        total: ttc,
        currency: Currency::Chf,
        origin_reference: origin_number,
    };

    // i18n facture + surcharge titre/numéro « Avoir » (clés credit-note-pdf-*).
    let mut i18n = build_i18n(&state.i18n, state.config.locale);
    i18n.entries.insert(
        "invoice-pdf-title",
        state
            .i18n
            .format(&state.config.locale, "credit-note-pdf-title", None),
    );
    i18n.entries.insert(
        "invoice-pdf-number",
        state
            .i18n
            .format(&state.config.locale, "credit-note-pdf-number", None),
    );

    let pdf_bytes =
        kesh_qrbill::generate_credit_note_pdf(&pdf_data, &i18n).map_err(map_qrbill_error)?;

    let filename = sanitize_filename(cn.credit_note_number.as_deref().unwrap_or("avoir"));
    let disposition = format!("inline; filename=\"avoir-{filename}.pdf\"");

    let mut resp = (StatusCode::OK, pdf_bytes).into_response();
    resp.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    resp.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition).unwrap_or_else(|_| HeaderValue::from_static("inline")),
    );
    Ok(resp)
}
