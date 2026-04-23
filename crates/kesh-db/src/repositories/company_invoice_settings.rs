//! Repository pour `company_invoice_settings` (Story 5.2 — FR35).
//!
//! Relation 1-1 avec `companies`. Row créée à la volée (lazy) via
//! `INSERT IGNORE` au premier accès. Pattern « upsert read ».
//!
//! **Deux signatures de get-or-create** pour éviter les transactions
//! imbriquées :
//! - [`get_or_create_default`] : version pool-level pour le handler
//!   `GET /company/invoice-settings`.
//! - [`get_or_create_default_in_tx`] : version tx-level utilisée par
//!   [`invoices::validate_invoice`](super::invoices::validate_invoice)
//!   pour charger la config dans la transaction atomique de validation.
//!
//! **Duplication contrôlée** : le corps métier est dupliqué (5 lignes)
//! entre les deux fonctions avec un commentaire `MIRROR` de part et
//! d'autre. Le fallback HRTB générique sur `Executor` est notoirement
//! fragile avec SQLx 0.8 — duplication préférée (cf. spec pass 2 P13).

use sqlx::mysql::MySqlPool;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate, Journal};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "company_id, invoice_number_format, default_receivable_account_id, \
    default_revenue_account_id, default_sales_journal, journal_entry_description_template, \
    version, created_at, updated_at";

fn settings_snapshot_json(s: &CompanyInvoiceSettings) -> serde_json::Value {
    serde_json::json!({
        "companyId": s.company_id,
        "invoiceNumberFormat": s.invoice_number_format,
        "defaultReceivableAccountId": s.default_receivable_account_id,
        "defaultRevenueAccountId": s.default_revenue_account_id,
        "defaultSalesJournal": s.default_sales_journal.as_str(),
        "journalEntryDescriptionTemplate": s.journal_entry_description_template,
        "version": s.version,
    })
}

/// Retourne la config de la company (ou la crée avec les DEFAULT si
/// absente). Version **pool-level** — ouvre sa propre transaction pour
/// l'INSERT IGNORE + SELECT afin de garantir l'atomicité lazy-create.
pub async fn get_or_create_default(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // MIRROR: garder synchronisé avec get_or_create_default_in_tx.
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(settings)
}

/// Retourne la config, tx-aware. Utilisé par `validate_invoice` pour
/// éviter d'ouvrir une transaction imbriquée. Le caller fournit la
/// transaction déjà ouverte.
///
/// **SELECT FOR UPDATE** obligatoire (review pass 1 P1) : verrouille la
/// row pour empêcher un `PUT /company/invoice-settings` concurrent de
/// modifier la config entre la lecture et le commit de `validate_invoice`.
/// Sans ce verrou, l'écriture comptable pourrait utiliser les anciens
/// comptes pendant que l'audit snapshotait les nouveaux (TOCTOU).
pub async fn get_or_create_default_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    // MIRROR: garder synchronisé avec get_or_create_default.
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(settings)
}

/// Met à jour la config (tous les champs) avec verrou optimiste et audit.
///
/// Le caller doit avoir validé les données en amont (format, comptes,
/// journal). Audit log wrapper `{before, after}`.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    expected_version: i32,
    user_id: i64,
    changes: CompanyInvoiceSettingsUpdate,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // S'assurer que la row existe (création lazy si absente).
    sqlx::query("INSERT IGNORE INTO company_invoice_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE company_invoice_settings \
         SET invoice_number_format = ?, default_receivable_account_id = ?, \
             default_revenue_account_id = ?, default_sales_journal = ?, \
             journal_entry_description_template = ?, version = version + 1 \
         WHERE company_id = ? AND version = ?",
    )
    .bind(&changes.invoice_number_format)
    .bind(changes.default_receivable_account_id)
    .bind(changes.default_revenue_account_id)
    .bind(changes.default_sales_journal)
    .bind(&changes.journal_entry_description_template)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    let after = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let audit_details = serde_json::json!({
        "before": settings_snapshot_json(&before),
        "after": settings_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "company_invoice_settings.updated".to_string(),
            entity_type: "company_invoice_settings".to_string(),
            entity_id: company_id,
            details_json: Some(audit_details),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Creates company_invoice_settings with auto-prefill of default accounts (1100, 3000).
/// Called during onboarding finalization (after chart of accounts is loaded).
///
/// Standard Swiss account numbers:
/// - 1100: Receivables (clients/créances)
/// - 3000: Revenue (ventes/produits)
pub async fn insert_with_defaults(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<CompanyInvoiceSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Lookup accounts 1100 (receivable) and 3000 (revenue) for this company.
    // Returns None if accounts don't exist (valid in production if chart doesn't include them).
    let receivable = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND number = '1100' AND active = true LIMIT 1"
    )
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    let revenue = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT id FROM accounts WHERE company_id = ? AND number = '3000' AND active = true LIMIT 1"
    )
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .flatten();

    // If either account doesn't exist in the chart, they'll be NULL in the settings.
    // This is acceptable for non-standard charts or during chart setup.

    // INSERT IGNORE for idempotency (finalize can be retried on browser crash/refresh).
    // If already exists, silently succeeds and we fetch the existing row below.
    sqlx::query(
        "INSERT IGNORE INTO company_invoice_settings \
         (company_id, invoice_number_format, default_receivable_account_id, \
          default_revenue_account_id, default_sales_journal, journal_entry_description_template) \
         VALUES (?, 'F-{YEAR}-{SEQ:04}', ?, ?, 'Ventes', '{YEAR}-{INVOICE_NUMBER}')"
    )
    .bind(company_id)
    .bind(receivable)
    .bind(revenue)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    // Fetch the (possibly already-existing) row to return in response
    let settings = sqlx::query_as::<_, CompanyInvoiceSettings>(&format!(
        "SELECT {COLUMNS} FROM company_invoice_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(settings)
}

// Reference to avoid unused import warning if Journal is not referenced
// elsewhere in this file (needed for the SQL bind).
#[allow(dead_code)]
const _JOURNAL_TYPE_MARKER: Option<Journal> = None;
