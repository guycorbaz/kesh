//! Repository `imported_supplier_invoices` (Story 12.5b — #194).
//!
//! Socle de persistance des factures fournisseurs **importées** (staging). La
//! logique d'orchestration de l'import (verrou de run, réactivation `discarded`,
//! déplacement `failed/`) et la complétion atomique (`create_in_tx`, DC6) vivent
//! en **12-5c** ; ce module n'expose que le CRUD de base, scopé multi-tenant.

use crate::entities::imported_supplier_invoice::{
    ImportedSupplierInvoice, NewImportedSupplierInvoice,
};
use crate::errors::{DbError, map_db_error};
use sqlx::MySqlPool;

/// SELECT scopé multi-tenant (anti-IDOR — toujours `AND company_id = ?`).
const FIND_BY_ID_SQL: &str =
    "SELECT * FROM imported_supplier_invoices WHERE id = ? AND company_id = ?";

/// Insère une facture importée en staging (`status = 'to_complete'`).
///
/// # Errors
/// - [`DbError::UniqueConstraintViolation`] si `(company_id, file_hash)` existe
///   déjà (doublon — mappé `DUPLICATE` par le service 12-5c).
/// - Autre [`DbError`] si un champ QR non conforme dépasse la largeur de sa
///   colonne (un QR tiers hors SIX 2.2 : nom > 70, message > 140, etc.). Le
///   parseur 12-5a ne borne pas les longueurs ; la validation/normalisation des
///   champs sur-longs et le mapping en échec **par-fichier** (vs 500 global)
///   relèvent de la couche d'ingestion 12-5c. Voir dette déférée code-review M2.
pub async fn create(
    pool: &MySqlPool,
    new: &NewImportedSupplierInvoice,
) -> Result<ImportedSupplierInvoice, DbError> {
    let id = sqlx::query(
        "INSERT INTO imported_supplier_invoices \
         (company_id, file_hash, storage_path, original_filename, mime_type, byte_size, \
          creditor_iban, is_qr_iban, creditor_address_type, creditor_name, creditor_line1, \
          creditor_line2, creditor_postal_code, creditor_town, creditor_country, \
          reference_type, reference_value, amount, currency, unstructured_message, billing_information) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.file_hash)
    .bind(&new.storage_path)
    .bind(&new.original_filename)
    .bind(&new.mime_type)
    .bind(new.byte_size)
    .bind(&new.creditor_iban)
    .bind(new.is_qr_iban)
    .bind(&new.creditor_address_type)
    .bind(&new.creditor_name)
    .bind(&new.creditor_line1)
    .bind(&new.creditor_line2)
    .bind(&new.creditor_postal_code)
    .bind(&new.creditor_town)
    .bind(&new.creditor_country)
    .bind(&new.reference_type)
    .bind(&new.reference_value)
    .bind(new.amount)
    .bind(&new.currency)
    .bind(&new.unstructured_message)
    .bind(&new.billing_information)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .last_insert_id() as i64;

    find_by_id_scoped(pool, new.company_id, id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Récupère une facture importée par id, scopée à la company (anti-IDOR).
pub async fn find_by_id_scoped(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(FIND_BY_ID_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Recherche par hash de fichier dans une company (idempotence / réactivation 12-5c).
pub async fn find_by_company_hash(
    pool: &MySqlPool,
    company_id: i64,
    file_hash: &str,
) -> Result<Option<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices WHERE company_id = ? AND file_hash = ?",
    )
    .bind(company_id)
    .bind(file_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les factures importées d'une company filtrées par statut
/// (use-case principal `to_complete`).
///
/// **Contrat appelant** : `status` DOIT être l'une des valeurs du domaine
/// (`to_complete` | `completed` | `discarded`). Un statut hors-domaine n'est pas
/// rejeté ici (paramètre bindé, pas d'injection) mais retourne une liste vide —
/// la validation du query param HTTP est à la charge de l'endpoint 12-5c
/// (frontière d'entrée), cf. dette déférée code-review.
pub async fn list_by_status(
    pool: &MySqlPool,
    company_id: i64,
    status: &str,
) -> Result<Vec<ImportedSupplierInvoice>, DbError> {
    sqlx::query_as::<_, ImportedSupplierInvoice>(
        "SELECT * FROM imported_supplier_invoices \
         WHERE company_id = ? AND status = ? ORDER BY created_at DESC, id DESC",
    )
    .bind(company_id)
    .bind(status)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}
