//! Repository pour `bank_transactions` — transactions bancaires
//! individuelles d'un import.
//!
//! Story 8-1b ne fait que lire les transactions (la création est faite
//! atomiquement par `bank_imports::create_with_transactions`). Story 8-4
//! ajoutera `update_status`, `match_with_journal_entry` etc.
//!
//! **Multi-tenant double-scope** : `list_by_import` filtre par
//! `(company_id, import_id)` même si `import_id` est unique — le scoping
//! par `company_id` reste appliqué pour Pattern 1 KF-002 (pas de leak
//! même si un attaquant devine un `import_id` valide d'une autre
//! company).

use sqlx::mysql::MySqlPool;

use crate::entities::bank_transaction::BankTransaction;
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, version, \
     created_at, updated_at";

/// Liste les transactions d'un import donné, scopées multi-tenant.
///
/// Tri par `id` (= ordre d'insertion = ordre du fichier source pour
/// CAMT.053 où le parser émet les `<Ntry>` dans l'ordre). Renvoie un
/// `Vec` vide si l'import n'existe pas / appartient à une autre company.
pub async fn list_by_import(
    pool: &MySqlPool,
    company_id: i64,
    import_id: i64,
) -> Result<Vec<BankTransaction>, DbError> {
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND import_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .bind(import_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}
