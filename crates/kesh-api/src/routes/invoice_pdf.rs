//! Story 5.3 — endpoint `GET /api/v1/invoices/:id/pdf`.
//!
//! Thin wrapper HTTP depuis la Story 20.3a : la génération complète
//! (chargement DB, validations, mapping, PDF) vit dans
//! `invoice_pdf_service::render_document`. Le handler ne garde que l'authentification
//! (`get_company_for`), le log, et la construction de la `Response`
//! (`Content-Type` + `Content-Disposition`).
//!
//! Story 25-6-b (#387) : le PDF servi est le document **émis** — figé au
//! premier rendu après validation, dans la langue du **client**, puis relu à
//! l'identique (`issued_invoice_pdf`). Le premier téléchargement écrit donc
//! (fichier, colonnes, audit `invoice.pdf_frozen`), quel que soit le rôle de
//! l'acteur (arbitrage 8). ⚠️ `HEAD` aussi, axum le servant par ce handler.

use crate::middleware::auth::CurrentUser;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use axum::{Extension, Json};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::routes::invoices::InvoiceResponse;
use crate::routes::issued_invoice_pdf::{self, PdfContext};

/// `GET /api/v1/invoices/:id/pdf` — téléchargement PDF d'une facture validée.
pub async fn get_invoice_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    tracing::info!(
        user_id = current_user.user_id,
        role = ?current_user.role,
        invoice_id = id,
        "PDF download requested"
    );

    let ctx = PdfContext {
        pool: &state.pool,
        i18n: &state.i18n,
        documents_dir: std::path::Path::new(state.config.documents_dir.as_str()),
        company: &company,
        user_id: current_user.user_id,
        actor_api_key_id: current_user.api_key_id,
    };
    let rendered = issued_invoice_pdf::get_or_freeze(&ctx, id).await?;

    // Content-Disposition : filename sanitizé (par le service).
    let disposition = format!(
        "inline; filename=\"facture-{}.pdf\"",
        rendered.filename_base
    );

    let mut resp = (StatusCode::OK, rendered.bytes).into_response();
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

/// `POST /api/v1/invoices/:id/pdf/refreeze` — **Admin** (arbitrage 7, Story
/// 25-6-b). Refige le PDF d'une facture figée dont le fichier a disparu : un
/// **nouveau** document, pas l'original, tracé au journal d'audit avec
/// l'ancienne et la nouvelle empreinte. Sans corps. Rend la facture à jour.
pub async fn refreeze_invoice_pdf(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<InvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let ctx = PdfContext {
        pool: &state.pool,
        i18n: &state.i18n,
        documents_dir: std::path::Path::new(state.config.documents_dir.as_str()),
        company: &company,
        user_id: current_user.user_id,
        actor_api_key_id: current_user.api_key_id,
    };
    issued_invoice_pdf::refreeze(&ctx, id).await?;
    let (invoice, lines) =
        kesh_db::repositories::invoices::find_by_id_with_lines(&state.pool, company.id, id)
            .await?
            .ok_or(AppError::Database(kesh_db::errors::DbError::NotFound))?;
    Ok(Json(InvoiceResponse::from_parts(invoice, lines)))
}
