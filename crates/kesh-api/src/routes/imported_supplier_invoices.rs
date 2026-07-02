//! Story 12.5c — endpoints d'import du répertoire de factures (#194).
//!
//! - `POST /api/v1/inbox-import`                                 déclenche un run d'import (Comptable+)
//! - `GET  /api/v1/imported-supplier-invoices?status=`          liste filtrée par statut (Comptable+)
//! - `POST /api/v1/imported-supplier-invoices/{id}/complete`    complétion atomique DC6 (Comptable+)
//! - `POST /api/v1/imported-supplier-invoices/{id}/discard`     écarte une importée (Comptable+)
//! - `GET  /api/v1/imported-supplier-invoices/{id}/source-document`  justificatif avant complétion
//! - `GET  /api/v1/supplier-invoices/{id}/source-document`      justificatif après complétion
//!
//! La complétion (`/complete`) **siège la transaction DC6** : `SELECT … FOR UPDATE`
//! du staging + `create_in_tx` (facture réelle 12-2) + `UPDATE status='completed'`
//! dans **une seule** transaction → impossible d'aboutir à une double facture.

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::Deserialize;

use kesh_core::accounting::vat::line_vat_amount;
use kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice;
use kesh_db::entities::{NewSupplierInvoice, NewSupplierInvoiceLine};
use kesh_db::errors::map_db_error;
use kesh_db::repositories::{imported_supplier_invoices, supplier_invoices};

use crate::AppState;
use crate::document_storage::{self, ReadDocumentError};
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::inbox_import;
use crate::middleware::auth::CurrentUser;
use crate::routes::supplier_invoices::SupplierInvoiceResponse;

/// Domaine valide du query param `status` (AC9 / dette D1).
const VALID_STATUSES: [&str; 3] = ["to_complete", "completed", "discarded"];

// ---------------------------------------------------------------------------
// POST /api/v1/inbox-import
// ---------------------------------------------------------------------------

/// Déclenche un run d'import du répertoire inbox (Comptable+). Retourne le
/// rapport batch `{ accepted, failed, warnings }` (HTTP 200) ; `409` si un import
/// est déjà en cours (verrou de run F6).
pub async fn post_inbox_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
) -> Result<Json<inbox_import::InboxImportReport>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let report = inbox_import::run_inbox_import(&state.pool, &state.config, company.id).await?;
    Ok(Json(report))
}

// ---------------------------------------------------------------------------
// GET /api/v1/imported-supplier-invoices?status=
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct ListImportedQuery {
    /// **Obligatoire** (pas de défaut côté API ; l'affichage par défaut
    /// `to_complete` est une décision frontend 12-5d).
    pub status: Option<String>,
}

/// Liste les factures importées d'une company filtrées par statut (Comptable+).
/// Le `status` est validé à la frontière HTTP (dette D1) : absent ou hors-domaine
/// → `400`, PAS une liste vide silencieuse.
pub async fn list_imported(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Query(params): Query<ListImportedQuery>,
) -> Result<Json<Vec<ImportedSupplierInvoice>>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let status = params.status.as_deref().unwrap_or_default();
    if !VALID_STATUSES.contains(&status) {
        return Err(AppError::Validation(format!(
            "Paramètre 'status' requis et ∈ {{to_complete, completed, discarded}} (reçu : '{status}')"
        )));
    }
    let items = imported_supplier_invoices::list_by_status(&state.pool, company.id, status).await?;
    Ok(Json(items))
}

// ---------------------------------------------------------------------------
// POST /api/v1/imported-supplier-invoices/{id}/complete
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteImportLineRequest {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub expense_account_id: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteImportRequest {
    pub contact_id: i64,
    pub invoice_date: NaiveDate,
    pub supplier_invoice_number: Option<String>,
    pub due_date: Option<NaiveDate>,
    pub lines: Vec<CompleteImportLineRequest>,
}

/// Complétion atomique d'une facture importée (DC6, Comptable+). Toute la
/// séquence (verrou staging → validations métier → `create_in_tx` → `UPDATE
/// completed`) vit dans **une transaction** ; sur échec, rollback total (pas
/// d'écriture comptable partielle, staging reste `to_complete`).
pub async fn complete_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<CompleteImportRequest>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let user_id = current_user.user_id;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;

    let result = async {
        // (1) Verrou staging + garde anti-double / anti-IDOR (scopé company).
        let staging =
            imported_supplier_invoices::find_by_id_scoped_for_update(&mut tx, company.id, id)
                .await?
                .ok_or(AppError::ImportedInvoiceNotFound)?;

        // (2) Statut : seule une importée `to_complete` peut être complétée.
        if staging.status != "to_complete" {
            return Err(AppError::ImportNotPendingCompletion {
                current_status: staging.status.clone(),
            });
        }

        // (3) Devise — CHF UNIQUEMENT en v0.4 (F-OPUS-1, décision Guy 2026-06-30).
        if staging.currency != "CHF" {
            return Err(AppError::ImportCompletionRejected {
                error_code: "CURRENCY_NOT_SUPPORTED",
                message: format!(
                    "Devise non supportée en v0.4 (CHF uniquement) : '{}'. Écartez la facture et saisissez-la manuellement.",
                    staging.currency
                ),
                details: Some(serde_json::json!({ "currency": staging.currency })),
            });
        }

        // (4) Routage IBAN (F9) + cohérence QRR ↔ QR-IBAN (SIX SPC 2.2).
        let (creditor_iban, creditor_qr_iban) = if staging.is_qr_iban {
            if staging.reference_type != "QRR" {
                return Err(iban_reference_mismatch(
                    "Un QR-IBAN exige une référence QRR.",
                ));
            }
            (None, Some(staging.creditor_iban.clone()))
        } else {
            if staging.reference_type == "QRR" {
                return Err(iban_reference_mismatch(
                    "Une référence QRR exige un QR-IBAN.",
                ));
            }
            (Some(staging.creditor_iban.clone()), None)
        };

        // (5) Mapping payment_reference (C5-1) : QRR/SCOR → valeur, sinon None.
        let payment_reference = match staging.reference_type.as_str() {
            "QRR" | "SCOR" => staging.reference_value.clone(),
            _ => None,
        };

        // Construction des lignes de la facture réelle.
        let lines: Vec<NewSupplierInvoiceLine> = req
            .lines
            .iter()
            .map(|l| NewSupplierInvoiceLine {
                description: l.description.clone(),
                quantity: l.quantity,
                unit_price: l.unit_price,
                vat_rate: l.vat_rate,
                expense_account_id: l.expense_account_id,
            })
            .collect();

        // (6) Réconciliation montant (F2) : si le QR porte un montant, exiger
        //     l'égalité EXACTE Σ lignes TTC == staging.amount (pleine précision,
        //     PAS round2 — garantit total_amount == staging.amount ==
        //     expected_payment_amount par construction, F-OPUS-2). Σ TTC calculé
        //     EXACTEMENT comme create_in_tx (TVA par ligne via line_vat_amount).
        if let Some(qr_amount) = staging.amount {
            let mut sum_ht = Decimal::ZERO;
            let mut sum_vat = Decimal::ZERO;
            for l in &lines {
                let line_total = l.quantity * l.unit_price;
                sum_ht += line_total;
                sum_vat += line_vat_amount(line_total, l.vat_rate);
            }
            let sum_ttc = sum_ht + sum_vat;
            if sum_ttc != qr_amount {
                return Err(AppError::ImportCompletionRejected {
                    error_code: "AMOUNT_MISMATCH",
                    message: format!(
                        "Le total des lignes ({sum_ttc}) ne correspond pas au montant du QR ({qr_amount})."
                    ),
                    details: Some(serde_json::json!({
                        "expected": qr_amount.to_string(),
                        "actual": sum_ttc.to_string(),
                    })),
                });
            }
        }

        // (7) Création de la facture réelle dans la MÊME transaction (DC6).
        let new = NewSupplierInvoice {
            company_id: company.id,
            contact_id: req.contact_id,
            supplier_invoice_number: req.supplier_invoice_number.clone(),
            invoice_date: req.invoice_date,
            due_date: req.due_date,
            creditor_iban,
            creditor_qr_iban,
            payment_reference,
            expected_payment_amount: staging.amount,
            // Tag projet à la complétion d'un import = extension future → None ici.
            project_id: None,
            lines,
        };
        let created = supplier_invoices::create_in_tx(&mut tx, new, user_id).await?;

        // Lien staging → facture réelle + passage `completed`.
        imported_supplier_invoices::mark_completed(
            &mut tx,
            company.id,
            id,
            created.invoice.id,
        )
        .await?;

        Ok(created)
    }
    .await;

    match result {
        Ok(created) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Database(map_db_error(e)))?;
            Ok(Json(SupplierInvoiceResponse::from_parts(
                created.invoice,
                created.lines,
            )))
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Helper : rejet `IBAN_REFERENCE_MISMATCH` (HTTP 400) avec message contextuel.
fn iban_reference_mismatch(message: &str) -> AppError {
    AppError::ImportCompletionRejected {
        error_code: "IBAN_REFERENCE_MISMATCH",
        message: message.to_string(),
        details: None,
    }
}

// ---------------------------------------------------------------------------
// POST /api/v1/imported-supplier-invoices/{id}/discard
// ---------------------------------------------------------------------------

/// Écarte une facture importée `to_complete` → `discarded` (Comptable+). Le
/// justificatif archivé est **conservé** (v0.4). FOR UPDATE + garde statut.
pub async fn discard_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<StatusCode, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;

    let mut tx = state
        .pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;

    let result = async {
        let staging =
            imported_supplier_invoices::find_by_id_scoped_for_update(&mut tx, company.id, id)
                .await?
                .ok_or(AppError::ImportedInvoiceNotFound)?;
        if staging.status != "to_complete" {
            return Err(AppError::ImportNotPendingCompletion {
                current_status: staging.status.clone(),
            });
        }
        imported_supplier_invoices::mark_discarded(&mut tx, company.id, id).await?;
        Ok(())
    }
    .await;

    match result {
        Ok(()) => {
            tx.commit()
                .await
                .map_err(|e| AppError::Database(map_db_error(e)))?;
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// Download du justificatif
// ---------------------------------------------------------------------------

/// `GET /imported-supplier-invoices/{id}/source-document` — justificatif **avant**
/// complétion (résout via le staging, anti-IDOR scopé company).
pub async fn get_imported_source_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let row = imported_supplier_invoices::find_by_id_scoped(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::SourceDocumentNotFound)?;
    serve_document(&state, &row)
}

/// `GET /supplier-invoices/{id}/source-document` — justificatif **après**
/// complétion (résout l'`imported_supplier_invoices` lié, scopé company). Une
/// facture créée directement 12-2 (sans import) → 404 (L5).
pub async fn get_supplier_invoice_source_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(supplier_invoice_id): Path<i64>,
) -> Result<Response, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let row = imported_supplier_invoices::find_by_supplier_invoice_id_scoped(
        &state.pool,
        company.id,
        supplier_invoice_id,
    )
    .await?
    .ok_or(AppError::SourceDocumentNotFound)?;
    serve_document(&state, &row)
}

/// Lit le justificatif archivé et construit la réponse de téléchargement.
///
/// Sémantique d'absence, **JAMAIS 500** : row présente mais fichier disque absent
/// (restore métadonnée-seule, L1/F7) → `410 Gone` ; `InvalidPath` (corruption
/// interne, ne devrait jamais arriver) → 500.
fn serve_document(state: &AppState, row: &ImportedSupplierInvoice) -> Result<Response, AppError> {
    let documents_dir = std::path::Path::new(state.config.documents_dir.as_str());
    let bytes = match document_storage::read_document(documents_dir, &row.storage_path) {
        Ok(b) => b,
        Err(ReadDocumentError::NotFound) => return Err(AppError::SourceDocumentGone),
        Err(ReadDocumentError::InvalidPath) => {
            return Err(AppError::Internal(format!(
                "justificatif storage_path invalide: {}",
                row.storage_path
            )));
        }
        Err(ReadDocumentError::Io(e)) => {
            return Err(AppError::Internal(format!("lecture justificatif: {e}")));
        }
    };

    let mut resp = bytes.into_response();
    if let Ok(ct) = HeaderValue::from_str(&row.mime_type) {
        resp.headers_mut().insert(header::CONTENT_TYPE, ct);
    }
    if let Ok(cd) = HeaderValue::from_str(&content_disposition(&row.original_filename)) {
        resp.headers_mut().insert(header::CONTENT_DISPOSITION, cd);
    }
    Ok(resp)
}

/// Construit un en-tête `Content-Disposition` robuste aux noms **accentués**
/// (courant en Suisse romande : `Reçu_Müller_été.pdf`), code-review 12-5c EC2/BH4.
///
/// - `filename=` : repli ASCII assaini (strip non-ASCII + guillemets/backslash/
///   contrôles) — les vieux clients le lisent.
/// - `filename*=UTF-8''<pct>` (RFC 5987) : nom complet UTF-8 percent-encodé — les
///   clients modernes le préfèrent. Sans lui, un nom accentué faisait échouer
///   `HeaderValue::from_str` (valeurs d'en-tête HTTP = ASCII only) → header omis,
///   le navigateur retombait sur le nom dérivé de l'URL.
fn content_disposition(original_filename: &str) -> String {
    let ascii_fallback: String = original_filename
        .chars()
        .filter(|c| c.is_ascii() && *c != '"' && *c != '\\' && !c.is_control())
        .collect();
    let ascii_fallback = if ascii_fallback.is_empty() {
        "source-document".to_string()
    } else {
        ascii_fallback
    };
    format!(
        "attachment; filename=\"{ascii_fallback}\"; filename*=UTF-8''{}",
        rfc5987_encode(original_filename)
    )
}

/// Percent-encode RFC 5987 : seuls les `attr-char` (alphanum + `!#$&+-.^_`|~`)
/// restent littéraux ; tout le reste (espace, accents UTF-8, ...) est `%HH` sur
/// chaque octet UTF-8.
fn rfc5987_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        let keep = b.is_ascii_alphanumeric()
            || matches!(
                b,
                b'!' | b'#' | b'$' | b'&' | b'+' | b'-' | b'.' | b'^' | b'_' | b'`' | b'|' | b'~'
            );
        if keep {
            out.push(b as char);
        } else {
            out.push('%');
            out.push_str(&format!("{b:02X}"));
        }
    }
    out
}
