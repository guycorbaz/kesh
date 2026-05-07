//! Repository pour la réconciliation bancaire (Story 8-4).
//!
//! Helpers :
//!
//! - [`find_unpaid_invoices_for_window`] : charge les factures
//!   candidates pour le matching d'une transaction bancaire — filtre
//!   multi-tenant + fenêtre temporelle ± `window_days` + tolérance
//!   amount ± `amount_tolerance`. **PAS de filtre currency** (S4-1
//!   Pass 4 revert : colonne `invoices.currency` n'existe pas v0.1
//!   mono-CHF, cf. L38).
//! - [`find_pending_transactions_for_account`] : charge les
//!   transactions `status='pending'` non-rejetées (`auto_match_rejected_at IS NULL`)
//!   pour un compte donné, multi-tenant scoped, paginées (limit
//!   hardcodé 100 au caller, cf. L24).
//! - [`find_contacts_by_ids`] : batch-load Contacts par liste d'IDs
//!   (M5 Pass 1 + MP4-3 Pass 4 — évite N×M queries dans le handler
//!   GET proposals 4-pass architecture).
//! - [`find_pending_by_ids`] : pré-flight ownership check batch
//!   pour POST /accept et POST /reject (HP3-4 + MP3-3 Pass 3 —
//!   1 SELECT IN avant lock acquisition, valide que tous les IDs
//!   appartiennent au compte fourni).
//!
//! Tous les helpers utilisent l'Executor générique pour accepter
//! `&MySqlPool` (preview) ou `&mut Transaction<MySql>` (handler create
//! dans tx ouverte).

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySql;
use std::collections::HashMap;

use crate::entities::bank_transaction::BankTransaction;
use crate::entities::contact::Contact;
use crate::entities::invoice::Invoice;
use crate::errors::{DbError, map_db_error};

/// Colonnes Invoice pour SELECT (cohérent FIND_INVOICE_SCOPED_SQL).
const INVOICE_COLUMNS: &str = "id, company_id, contact_id, invoice_number, status, date, \
     due_date, payment_terms, total_amount, journal_entry_id, paid_at, version, \
     created_at, updated_at";

/// Colonnes BankTransaction pour SELECT (cohérent bank_transactions::COLUMNS).
const BANK_TX_COLUMNS: &str = "id, company_id, import_id, bank_account_id, booking_date, value_date, \
     amount, currency, reference, details, end_to_end_id, transaction_id, \
     counterparty_iban, counterparty_name, status, matched_entry_id, \
     auto_match_rejected_at, version, created_at, updated_at";

/// Charge les factures candidates pour la réconciliation d'une
/// transaction bancaire (Story 8-4 §candidate-window).
///
/// Filtres :
/// 1. `company_id` (KF-002 Pattern 1, multi-tenant scoping).
/// 2. `status = 'validated' AND paid_at IS NULL AND journal_entry_id IS NOT NULL`
///    (factures éligibles à la réconciliation v0.1).
/// 3. `date BETWEEN tx_date - window_days AND tx_date + window_days`.
/// 4. `total_amount BETWEEN tx_amount - amount_tolerance AND tx_amount + amount_tolerance`.
///
/// **Pas de filtre currency v0.1** — colonne inexistante (cf. L38 + S4-1 Pass 4).
///
/// **Multi-tenant safety (KF-002 Pattern 1)** : filtrage systématique
/// par `company_id`.
///
/// **Index** : `idx_invoices_company_validated_unpaid_date` créé en
/// migration `20260507100001_reconciliation_8_4.sql`.
pub async fn find_unpaid_invoices_for_window<'e, E>(
    executor: E,
    company_id: i64,
    tx_date: NaiveDate,
    tx_amount: Decimal,
    window_days: i64,
    amount_tolerance: Decimal,
) -> Result<Vec<Invoice>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Invoice>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices \
         WHERE company_id = ? \
           AND status = 'validated' \
           AND paid_at IS NULL \
           AND journal_entry_id IS NOT NULL \
           AND date BETWEEN DATE_SUB(?, INTERVAL ? DAY) AND DATE_ADD(?, INTERVAL ? DAY) \
           AND total_amount BETWEEN ? - ? AND ? + ?",
    ))
    .bind(company_id)
    .bind(tx_date)
    .bind(window_days)
    .bind(tx_date)
    .bind(window_days)
    .bind(tx_amount)
    .bind(amount_tolerance)
    .bind(tx_amount)
    .bind(amount_tolerance)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Charge les transactions bancaires `pending` non-rejetées pour un
/// compte donné, scopées multi-tenant, paginées via `limit`.
///
/// Filtres :
/// - `company_id` (KF-002 Pattern 1).
/// - `bank_account_id` (scope local).
/// - `status = 'pending'`.
/// - `auto_match_rejected_at IS NULL` (exclut les transactions
///   rejetées manuellement, réservées pour 8-5 manual).
///
/// Tri par `booking_date DESC, id DESC` pour présenter les
/// transactions récentes d'abord (UX nominal).
pub async fn find_pending_transactions_for_account<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    limit: i64,
) -> Result<Vec<BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, BankTransaction>(&format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? \
           AND bank_account_id = ? \
           AND status = 'pending' \
           AND auto_match_rejected_at IS NULL \
         ORDER BY booking_date DESC, id DESC \
         LIMIT ?",
    ))
    .bind(company_id)
    .bind(bank_account_id)
    .bind(limit)
    .fetch_all(executor)
    .await
    .map_err(map_db_error)
}

/// Batch-load Contacts par liste d'IDs (MP4-3 Pass 4 — éviter N×M
/// queries dans le handler GET proposals 4-pass architecture).
/// Retourne une `HashMap<i64, Contact>` keyed par `contact.id`.
///
/// **Multi-tenant safety** : filtre `company_id`.
/// **Soft-delete** (cf. L37) : ne filtre PAS `active = false` v0.1
/// — si un contact est désactivé entre import et reconciliation, le
/// matching produit quand même un contact_score (poids 0.10 donc
/// impact mineur).
pub async fn find_contacts_by_ids<'e, E>(
    executor: E,
    company_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, Contact>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    // Build IN clause manuellement avec placeholders.
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT id, company_id, contact_type, name, is_client, is_supplier, \
                address, email, phone, ide_number, default_payment_terms, \
                active, version, created_at, updated_at \
         FROM contacts \
         WHERE company_id = ? AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, Contact>(&sql).bind(company_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|c| (c.id, c)).collect())
}

/// Helper inline pour charger un `Contact` par id scopée multi-tenant
/// — variant Executor générique du `contacts::find_by_id_in_company`
/// qui ne prenait que `&MySqlPool` (utilisable dans transaction).
pub async fn find_contact_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<Contact>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Contact>(
        "SELECT id, company_id, contact_type, name, is_client, is_supplier, \
         address, email, phone, ide_number, default_payment_terms, active, version, \
         created_at, updated_at FROM contacts WHERE id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Helper inline pour charger une `Invoice` par id scopée multi-tenant.
/// Utilisé par le handler accept (step 5) — équivalent à
/// `FIND_INVOICE_SCOPED_SQL` d'`invoices.rs:41` mais accessible
/// publiquement.
pub async fn find_invoice_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<Invoice>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    sqlx::query_as::<_, Invoice>(&format!(
        "SELECT {INVOICE_COLUMNS} FROM invoices WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)
}

/// Pré-flight ownership check batch (HP3-4 + MP3-3 Pass 3) : vérifie
/// que tous les `bank_transaction_id` fournis appartiennent au
/// `(company_id, bank_account_id)` donné. 1 SELECT IN avant lock
/// acquisition.
///
/// Retourne `HashMap<id, BankTransaction>` keyed par `tx.id`. Les IDs
/// non-trouvés sont absents de la HashMap — le caller compare
/// `result.len() == ids.len()` pour détecter le mismatch.
pub async fn find_pending_by_ids<'e, E>(
    executor: E,
    company_id: i64,
    bank_account_id: i64,
    ids: &[i64],
) -> Result<HashMap<i64, BankTransaction>, DbError>
where
    E: sqlx::Executor<'e, Database = MySql>,
{
    if ids.is_empty() {
        return Ok(HashMap::new());
    }
    let placeholders = std::iter::repeat_n("?", ids.len())
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "SELECT {BANK_TX_COLUMNS} FROM bank_transactions \
         WHERE company_id = ? AND bank_account_id = ? AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_as::<_, BankTransaction>(&sql)
        .bind(company_id)
        .bind(bank_account_id);
    for id in ids {
        q = q.bind(id);
    }
    let rows = q.fetch_all(executor).await.map_err(map_db_error)?;
    Ok(rows.into_iter().map(|t| (t.id, t)).collect())
}
