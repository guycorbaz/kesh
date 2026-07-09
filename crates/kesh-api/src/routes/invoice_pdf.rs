//! Story 5.3 — endpoint `GET /api/v1/invoices/:id/pdf`.
//!
//! Thin wrapper HTTP depuis la Story 20.3a : la génération complète
//! (chargement DB, validations, mapping, PDF) vit dans
//! `invoice_pdf_service::render`. Le handler ne garde que l'authentification
//! (`get_company_for`), le log, et la construction de la `Response`
//! (`Content-Type` + `Content-Disposition`). La langue est résolue via
//! `state.config.locale` (instance-level, pattern Story 2.1) — pas de champ
//! langue sur `CurrentUser`.

use crate::middleware::auth::CurrentUser;
use axum::Extension;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};

use crate::AppState;
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::routes::invoice_pdf_service;

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

    let rendered = invoice_pdf_service::render(
        &state.pool,
        &state.i18n,
        state.config.locale,
        company.id,
        id,
    )
    .await?;

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
