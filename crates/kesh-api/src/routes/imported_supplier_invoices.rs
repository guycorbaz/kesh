//! Story 12.5c — endpoints d'import du répertoire de factures (#194).
//!
//! - `POST /api/v1/inbox-import`                                 déclenche un run d'import (Comptable+)
//! - `GET  /api/v1/imported-supplier-invoices?status=`          liste filtrée par statut (Comptable+)
//! - `POST /api/v1/imported-supplier-invoices/{id}/complete`    complétion atomique DC6 (Comptable+)
//! - `POST /api/v1/imported-supplier-invoices/{id}/discard`     écarte une importée (Comptable+)
//! - `GET  /api/v1/imported-supplier-invoices/{id}/supplier-candidates`  fournisseurs
//!   proposés par l'IBAN du document (Comptable+, Story 30-2b)
//! - `GET  /api/v1/imported-supplier-invoices/{id}/source-document`  justificatif avant complétion
//! - `GET  /api/v1/supplier-invoices/{id}/source-document`      document d'une facture
//!   (tout rôle authentifié depuis la Story 30-2c)
//! - `POST   /api/v1/supplier-invoices/{id}/source-document`    joindre un document à
//!   traiter à une facture existante (Comptable+, Story 30-2c)
//! - `DELETE /api/v1/supplier-invoices/{id}/source-document`    détacher le document
//!   (Comptable+, Story 30-2c)
//!
//! La complétion (`/complete`) **siège la transaction DC6** : `SELECT … FOR UPDATE`
//! du staging + `create_in_tx` (facture réelle 12-2) + `UPDATE status='completed'`
//! dans **une seule** transaction → impossible d'aboutir à une double facture.

use axum::Extension;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use kesh_core::accounting::vat::line_vat_amount;
use kesh_db::entities::audit_log::NewAuditLogEntry;
use kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice;
use kesh_db::errors::map_db_error;
use kesh_db::repositories::{audit_log, contacts, imported_supplier_invoices, supplier_invoices};

use crate::AppState;
use crate::audit::AuditActor;
use crate::document_storage::{self, ReadDocumentError};
use crate::errors::AppError;
use crate::helpers::get_company_for;
use crate::inbox_import;
use crate::middleware::auth::CurrentUser;
use crate::routes::contacts::ContactResponse;
use crate::routes::supplier_invoices::{
    CreateSupplierInvoiceRequest, PaymentDetails, PaymentSource, SupplierInvoiceResponse,
    normalize_payment_field,
};

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
    let report = inbox_import::run_inbox_import(
        &state.pool,
        &state.config,
        company.id,
        current_user.user_id,
        current_user.api_key_id,
    )
    .await?;
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

/// Complétion atomique d'une facture importée (DC6, Comptable+). Toute la
/// séquence (verrou staging → validations métier → `create_in_tx` → `UPDATE
/// completed`) vit dans **une transaction** ; sur échec, rollback total (pas
/// d'écriture comptable partielle, staging reste `to_complete`).
///
/// Story 30-2b (AC13, C-30-2-10) : le corps est **celui de** `POST
/// /supplier-invoices` ([`CreateSupplierInvoiceRequest`]) — projet compris ; un
/// client qui n'envoie pas les champs ajoutés reste accepté (tous facultatifs).
/// La réponse est relue, une fois la transaction validée, par la fonction du GET
/// ([`SupplierInvoiceResponse::load_with_settlement_cancellation`]).
///
/// ⚠️ Angle mort écrit (revue P1 de la 30-2b, C-30-2-42) : une erreur de base
/// **entre** le `COMMIT` et cette relecture rend une erreur alors que la facture
/// est créée et le document `completed` ; un nouvel essai rend `409
/// IMPORT_NOT_PENDING_COMPLETION`. Rien n'est perdu ni faussé — patron de `pay`.
///
/// ⚠️ **Rejouée sur interblocage** (Story 15-5e2) par l'enveloppe `AppError`
/// [`crate::retry::retry_app_on_deadlock`] : la transaction vit dans
/// [`complete_import_once`], une tentative.
pub async fn complete_import(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
    Json(req): Json<CreateSupplierInvoiceRequest>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let created =
        crate::retry::retry_app_on_deadlock("imported_supplier_invoices::complete", || {
            complete_import_once(&state.pool, &current_user, company.id, id, &req)
        })
        .await?;
    Ok(Json(
        SupplierInvoiceResponse::load_with_settlement_cancellation(
            &state.pool,
            company.id,
            created.invoice.id,
        )
        .await?,
    ))
}

/// Une tentative de la complétion d'une facture importée : ouvre la
/// transaction, verrouille le `staging`, contrôle (statut, devise, IBAN,
/// montant — **après** le verrou, dans cet ordre, car ils lisent la ligne
/// verrouillée), crée la facture réelle, marque le `staging` et conclut.
///
/// Le corps est reçu **par référence** : chaque tentative reconstruit ce
/// qu'elle consomme. Les erreurs de base arrivent en
/// `AppError::Database(DbError::Sqlx(_))`, ce que le prédicat de
/// [`crate::retry::retry_app_on_deadlock`] reconnaît.
async fn complete_import_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    company_id: i64,
    id: i64,
    req: &CreateSupplierInvoiceRequest,
) -> Result<supplier_invoices::SupplierInvoiceWithLines, AppError> {
    let user_id = current_user.user_id;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;

    let result = async {
        // (1) Verrou staging + garde anti-double / anti-IDOR (scopé company).
        let staging =
            imported_supplier_invoices::find_by_id_scoped_for_update(&mut tx, company_id, id)
                .await?
                .ok_or(AppError::ImportedInvoiceNotFound)?;

        // (2) Statut : seule une importée `to_complete` peut être complétée.
        if staging.status != "to_complete" {
            return Err(AppError::ImportNotPendingCompletion {
                current_status: staging.status.clone(),
            });
        }

        // (3) Devise — CHF UNIQUEMENT en v0.4 (F-OPUS-1, décision Guy 2026-06-30).
        //     Story 30-2a (C-30-2-19) : le contrôle ne vaut que pour un document
        //     AVEC QR ; sans QR, aucune devise n'est lue, la facture est en CHF
        //     comme toute facture fournisseur saisie.
        if let Some(currency) = staging.currency.as_deref()
            && currency != "CHF"
        {
            return Err(AppError::ImportCompletionRejected {
                error_code: "CURRENCY_NOT_SUPPORTED",
                message: format!(
                    "Devise non supportée (CHF uniquement) : '{currency}'. Retirez le document de la liste, ou créez la facture sans document puis joignez-le."
                ),
                details: Some(serde_json::json!({ "currency": currency })),
            });
        }

        // (4) Source des coordonnées de paiement (Story 30-2b, AC11/AC12).
        //     - Document AVEC QR : routage IBAN (F9) + cohérence QRR ↔ QR-IBAN
        //       (SIX SPC 2.2), valeurs lues sur la ligne VERROUILLÉE ; le corps
        //       ne doit en porter aucune (`QR_FIELDS_FROM_DOCUMENT`, refusé par
        //       la conversion commune).
        //     - Document SANS QR : elles viennent du corps, validées par la
        //       conversion commune comme pour `POST /supplier-invoices`
        //       (AC12-bis, `INVALID_PAYMENT_DETAILS`).
        let source = match staging.creditor_iban.clone() {
            None => PaymentSource::Body,
            Some(iban) => {
                let reference_is_qrr = staging.reference_type.as_deref() == Some("QRR");
                let (creditor_iban, creditor_qr_iban) = if staging.is_qr_iban == Some(true) {
                    if !reference_is_qrr {
                        return Err(iban_reference_mismatch(
                            "Un QR-IBAN exige une référence QRR.",
                        ));
                    }
                    (None, Some(iban))
                } else {
                    if reference_is_qrr {
                        return Err(iban_reference_mismatch(
                            "Une référence QRR exige un QR-IBAN.",
                        ));
                    }
                    (Some(iban), None)
                };
                // (5) Mapping payment_reference (C5-1) : QRR/SCOR → valeur,
                //     sinon None.
                let payment_reference = match staging.reference_type.as_deref() {
                    Some("QRR" | "SCOR") => staging.reference_value.clone(),
                    _ => None,
                };
                PaymentSource::Document {
                    payment: PaymentDetails {
                        creditor_iban,
                        creditor_qr_iban,
                        payment_reference,
                    },
                    expected_payment_amount: staging.amount,
                }
            }
        };

        // Conversion COMMUNE aux deux routes (AC13) : projet compris (fin de
        // l'« extension future » de la 12-5c), validation de l'AC12-bis.
        let new = req.to_new_supplier_invoice(company_id, source)?;

        // (6) Réconciliation montant (F2) : si le QR porte un montant (jamais un
        //     document sans QR, dont `amount` est `None` — Story 30-2a), exiger
        //     l'égalité EXACTE Σ lignes TTC == staging.amount (pleine précision,
        //     PAS round2 — garantit total_amount == staging.amount ==
        //     expected_payment_amount par construction, F-OPUS-2). Σ TTC calculé
        //     EXACTEMENT comme create_in_tx (TVA par ligne via line_vat_amount).
        if let Some(qr_amount) = staging.amount {
            let mut sum_ht = Decimal::ZERO;
            let mut sum_vat = Decimal::ZERO;
            for l in &new.lines {
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
        let created = supplier_invoices::create_in_tx(&mut tx, new, user_id).await?;

        // Lien staging → facture réelle + passage `completed`.
        imported_supplier_invoices::mark_completed(
            &mut tx,
            company_id,
            id,
            created.invoice.id,
        )
        .await?;

        // Story 25-1b (AC 3) — la transition de la PIÈCE laisse sa propre trace.
        // La facture fournisseur créée juste au-dessus est déjà auditée
        // (`supplier_invoice.created`, dans `create_in_tx`) ; sans celle-ci, le
        // cycle de vie d'une pièce disait `created` → `discarded` quand elle est
        // rejetée, et `created` → RIEN quand elle aboutit. C'était l'asymétrie.
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "imported_supplier_invoice.completed",
                "imported_supplier_invoice",
                id,
                Some(serde_json::json!({
                    "supplier_invoice_id": created.invoice.id,
                })),
            ),
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
            Ok(created)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

// ---------------------------------------------------------------------------
// GET /api/v1/imported-supplier-invoices/{id}/supplier-candidates
// ---------------------------------------------------------------------------

/// Nombre maximal de fournisseurs proposés (AC14, C-30-2-7) — le serveur n'en
/// rend jamais plus que l'écran n'en affiche.
const SUPPLIER_CANDIDATES_LIMIT: i64 = 3;

/// Réponse de `supplier-candidates` : des contacts, **jamais** l'IBAN.
#[derive(Serialize)]
pub struct SupplierCandidatesResponse {
    pub candidates: Vec<ContactResponse>,
}

/// Story 30-2b (AC14) — les fournisseurs qui ont déjà une facture portant
/// l'IBAN (ou le QR-IBAN) du créancier de ce document (Comptable+).
///
/// L'IBAN est lu **sur le document**, borné à la société, jamais passé dans
/// l'URL ; il ne sort ni dans la réponse ni dans les journaux. Un document sans
/// QR rend une liste vide ; un document inconnu ou d'une autre société, `404
/// IMPORTED_INVOICE_NOT_FOUND` (patron de la complétion). C'est la première des
/// deux sources de proposition — la seconde, par nom, est calculée à l'écran.
///
/// ⚠️ **Proposer, jamais créer ni présélectionner** (`CLAUDE.md`, « un
/// appariement automatique propose ») : l'écran affiche ces contacts avec un
/// bouton « Retenir », rien de plus.
pub async fn get_supplier_candidates(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result<Json<SupplierCandidatesResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    let staging = imported_supplier_invoices::find_by_id_scoped(&state.pool, company.id, id)
        .await?
        .ok_or(AppError::ImportedInvoiceNotFound)?;
    let candidates = match staging.creditor_iban.as_deref() {
        None => Vec::new(),
        Some(iban) => {
            contacts::find_suppliers_by_invoice_iban(
                &state.pool,
                company.id,
                &normalize_payment_field(iban).to_uppercase(),
                SUPPLIER_CANDIDATES_LIMIT,
            )
            .await?
        }
    };
    Ok(Json(SupplierCandidatesResponse {
        candidates: candidates.into_iter().map(ContactResponse::from).collect(),
    }))
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
/// Story 30-2a : c'est le geste « Retirer de la liste » de l'écran « Documents
/// à traiter » ; un nouveau dépôt du même fichier le réactive.
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

        // Story 25-1b (AC 3) — le rejet d'une pièce laisse une trace, dans la
        // transaction qui le produit. `creditor_iban` n'est JAMAIS journalisé :
        // seule sa présence l'est (convention du dépôt, cf. `bank_accounts.rs`).
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                &current_user,
                "imported_supplier_invoice.discarded",
                "imported_supplier_invoice",
                id,
                Some(serde_json::json!({
                    "original_filename": staging.original_filename,
                    "creditor_name": staging.creditor_name,
                    "amount": staging.amount,
                    "currency": staging.currency,
                    // Story 30-2a (AC7) : lu sur l'`Option`, non plus sur une
                    // chaîne — un document sans QR trace `false`.
                    "iban_present": staging.creditor_iban.is_some(),
                    "is_qr_iban": staging.is_qr_iban,
                })),
            ),
        )
        .await?;
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

/// `GET /supplier-invoices/{id}/source-document` — le document d'une facture
/// fournisseur (résout l'`imported_supplier_invoices` lié et `completed`, scopé
/// company). Une facture sans document → `404 SOURCE_DOCUMENT_NOT_FOUND`.
///
/// Story 30-2c (AC22, C-30-2-14) : **ouverte à tout rôle authentifié** — elle est
/// montée parmi les lectures, comme la fiche de la facture qu'elle illustre ; la
/// lecture d'un document **à traiter** (route ci-dessus) reste Comptable+.
/// L'isolation par société est celle de la lecture partagée.
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

// ---------------------------------------------------------------------------
// Story 30-2c (#685) — joindre un document après coup, le détacher
// ---------------------------------------------------------------------------

/// Corps de `POST /supplier-invoices/{id}/source-document`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachSourceDocumentRequest {
    pub imported_supplier_invoice_id: i64,
}

/// `POST /api/v1/supplier-invoices/{id}/source-document` — joint un document à
/// traiter à une facture fournisseur existante (Comptable+, Story 30-2c, AC19).
///
/// Tout statut de facture est accepté ; aucun champ de la facture n'est modifié
/// (C-30-2-12). Refus : `404 IMPORTED_INVOICE_NOT_FOUND` (document inconnu ou
/// d'une autre société), `404 NOT_FOUND` (facture), `409
/// IMPORT_NOT_PENDING_COMPLETION` (document déjà traité ou retiré), `409
/// SOURCE_DOCUMENT_ALREADY_ATTACHED` (la facture a déjà un document). Réponse :
/// la facture relue une fois la transaction validée, `sourceDocument` calculé.
///
/// ⚠️ **Rejouée sur interblocage** ([`crate::retry::retry_app_on_deadlock`]) :
/// la transaction vit dans [`attach_source_document_once`]. Cycle connu, rare :
/// avec l'acceptation d'un lot de rapprochement et la confirmation d'un lot de
/// paiement (C-30-2-46).
///
/// ⚠️ Angles morts écrits (revue P1 de la 30-2c, C-30-2-52) : une erreur de
/// base **entre** le `COMMIT` et la relecture rend une erreur alors que le
/// document est joint (patron de `pay` et de `complete_import`, C-30-2-42) ; et
/// la demande n'est pas idempotente — renvoyée après un succès, elle rend `409
/// IMPORT_NOT_PENDING_COMPLETION` (C-30-2-21 : aucune lecture préalable du
/// lien). Rien n'est perdu ni faussé ; `api-external.md` dit de relire la facture.
pub async fn attach_source_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(supplier_invoice_id): Path<i64>,
    Json(req): Json<AttachSourceDocumentRequest>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    crate::retry::retry_app_on_deadlock("imported_supplier_invoices::attach", || {
        attach_source_document_once(
            &state.pool,
            &current_user,
            company.id,
            supplier_invoice_id,
            req.imported_supplier_invoice_id,
        )
    })
    .await?;
    Ok(Json(
        SupplierInvoiceResponse::load_with_settlement_cancellation(
            &state.pool,
            company.id,
            supplier_invoice_id,
        )
        .await?,
    ))
}

/// Une tentative du rattachement : [`imported_supplier_invoices::attach_in_tx`]
/// (document, facture, sentinelle de la société, lien), puis l'audit
/// `imported_supplier_invoice.attached`, dans la même transaction.
async fn attach_source_document_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    company_id: i64,
    supplier_invoice_id: i64,
    imported_id: i64,
) -> Result<(), AppError> {
    use kesh_db::repositories::imported_supplier_invoices::AttachRefusal;

    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let result = async {
        let staging = match imported_supplier_invoices::attach_in_tx(
            &mut tx,
            company_id,
            imported_id,
            supplier_invoice_id,
        )
        .await?
        {
            Ok(staging) => staging,
            Err(AttachRefusal::DocumentNotFound) => return Err(AppError::ImportedInvoiceNotFound),
            Err(AttachRefusal::DocumentNotPending { current_status }) => {
                return Err(AppError::ImportNotPendingCompletion { current_status });
            }
            Err(AttachRefusal::InvoiceNotFound) => {
                return Err(AppError::Database(kesh_db::errors::DbError::NotFound));
            }
            Err(AttachRefusal::AlreadyAttached) => {
                return Err(AppError::SourceDocumentAlreadyAttached);
            }
        };
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "imported_supplier_invoice.attached",
                "imported_supplier_invoice",
                imported_id,
                Some(serde_json::json!({
                    "supplier_invoice_id": supplier_invoice_id,
                    "original_filename": staging.original_filename,
                })),
            ),
        )
        .await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => tx
            .commit()
            .await
            .map_err(|e| AppError::Database(map_db_error(e))),
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// `DELETE /api/v1/supplier-invoices/{id}/source-document` — détache le document
/// d'une facture fournisseur (Comptable+, Story 30-2c, AC24, C-30-2-13).
///
/// Le document repasse « à traiter » avec ses données QR ; la facture, ses
/// lignes et ses écritures ne sont pas touchées ; aucun fichier n'est supprimé.
/// Tout statut de facture est accepté. Facture inconnue, d'une autre société ou
/// sans document → `404 SOURCE_DOCUMENT_NOT_FOUND`. Réponse : la facture relue,
/// `sourceDocument: null`.
///
/// ⚠️ Angle mort assumé : un document **avec QR** détaché peut servir à créer
/// une seconde facture (geste humain explicite ; le manuel dit d'annuler la
/// facture fausse d'abord).
///
/// ⚠️ Angle mort écrit (revue P1 de la 30-2c, C-30-2-52) : une erreur de base
/// **entre** le `COMMIT` et la relecture rend une erreur alors que le document
/// est détaché ; un nouvel essai rend `404 SOURCE_DOCUMENT_NOT_FOUND`. Rien
/// n'est perdu ni faussé.
///
/// ⚠️ **Rejouée sur interblocage**, comme le rattachement.
pub async fn detach_source_document(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Path(supplier_invoice_id): Path<i64>,
) -> Result<Json<SupplierInvoiceResponse>, AppError> {
    let company = get_company_for(&current_user, &state.pool).await?;
    crate::retry::retry_app_on_deadlock("imported_supplier_invoices::detach", || {
        detach_source_document_once(&state.pool, &current_user, company.id, supplier_invoice_id)
    })
    .await?;
    Ok(Json(
        SupplierInvoiceResponse::load_with_settlement_cancellation(
            &state.pool,
            company.id,
            supplier_invoice_id,
        )
        .await?,
    ))
}

/// Une tentative du détachement : [`imported_supplier_invoices::detach_in_tx`]
/// (la ligne du document seule), puis l'audit
/// `imported_supplier_invoice.detached`, dans la même transaction.
async fn detach_source_document_once(
    pool: &sqlx::MySqlPool,
    current_user: &CurrentUser,
    company_id: i64,
    supplier_invoice_id: i64,
) -> Result<(), AppError> {
    let mut tx = pool
        .begin()
        .await
        .map_err(|e| AppError::Database(map_db_error(e)))?;
    let result = async {
        let staging =
            imported_supplier_invoices::detach_in_tx(&mut tx, company_id, supplier_invoice_id)
                .await?
                .ok_or(AppError::SourceDocumentNotFound)?;
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::from_current_user(
                current_user,
                "imported_supplier_invoice.detached",
                "imported_supplier_invoice",
                staging.id,
                Some(serde_json::json!({
                    "supplier_invoice_id": supplier_invoice_id,
                    "original_filename": staging.original_filename,
                })),
            ),
        )
        .await?;
        Ok(())
    }
    .await;
    match result {
        Ok(()) => tx
            .commit()
            .await
            .map_err(|e| AppError::Database(map_db_error(e))),
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
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
