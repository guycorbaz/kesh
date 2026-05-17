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

use chrono::NaiveDate;
use sqlx::MySql;
use sqlx::mysql::MySqlPool;

use crate::entities::bank_transaction::BankTransaction;
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Story 9-2b T3.2.7 — Liste **toutes** les transactions bancaires d'une
/// company sans filtre `import_id`, pour l'export global ZIP. Tri stable
/// `id ASC` (ordre d'insertion).
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankTransaction>, DbError> {
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

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

/// Charge les transactions existantes dans la fenêtre
/// `[period_from, period_to]` (bornes incluses) pour le compte
/// donné, scopées multi-tenant (Story 8-3 T3.1).
///
/// Utilisé par le handler `bank_imports::create` pour la détection de
/// doublons ligne-par-ligne (cf. `kesh_core::bank_imports::detect_duplicate_lines`).
///
/// **Multi-tenant safety (KF-002 Pattern 1)** : filtrage systématique
/// par `(company_id, bank_account_id)` — les transactions cross-tenant
/// ne sont jamais retournées.
///
/// **Index** : utilise
/// `idx_bank_transactions_company_account_date (company_id, bank_account_id, booking_date)`
/// (créé en migration `20260504000001_bank_imports.sql`).
///
/// **Executor générique** : accepte `&MySqlPool` (preview) ou
/// `&mut Transaction<MySql>` (handler create dans une tx ouverte).
pub async fn find_in_dedup_window<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    period_from: NaiveDate,
    period_to: NaiveDate,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    // L-4 (Pass 2 review) — bornes inversées (`from > to`) résulteraient
    // en BETWEEN MariaDB silencieusement vide ; surfacing en debug.
    debug_assert!(
        period_from <= period_to,
        "find_in_dedup_window: period_from {} > period_to {}",
        period_from,
        period_to
    );
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? \
           AND booking_date BETWEEN ? AND ?"
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(period_from)
    .bind(period_to)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}
