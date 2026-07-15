//! Repository de l'historique des rappels débiteurs (`invoice_reminders`).
//!
//! Table **append-only** : jamais de `DELETE`/`UPDATE` destructif — l'annulation d'un
//! envoi accidentel est un `cancelled_at` SOFT (préserve la preuve pré-contentieuse).
//! Le **niveau courant** d'une facture = `MAX(level_number)` des rappels **non-annulés**
//! (PAS un COUNT — permet le ré-envoi et le rappel manuel à niveau choisi sans fausser
//! la cadence, D18). Toutes les lectures/écritures sont scopées `company_id` (anti-IDOR :
//! cross-tenant → vide / not-found, jamais 403).

use crate::entities::{InvoiceReminder, NewInvoiceReminder};
use crate::errors::{DbError, map_db_error};
use sqlx::{MySql, MySqlPool, Transaction};

const COLUMNS: &str = "id, company_id, invoice_id, level_number, fee_amount, sent_at, channel, \
     sent_to, subject, body, note, actor_user_id, cancelled_at, created_at";

/// Un rappel par id, scopé company, sous tx (pour l'annulation Admin).
pub async fn find_by_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<InvoiceReminder>, DbError> {
    sqlx::query_as::<_, InvoiceReminder>(&format!(
        "SELECT {COLUMNS} FROM invoice_reminders WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Historique complet d'une facture (annulés inclus, distinguables par `cancelled_at`),
/// le plus récent d'abord. Scopé company.
pub async fn list_for_invoice(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
) -> Result<Vec<InvoiceReminder>, DbError> {
    sqlx::query_as::<_, InvoiceReminder>(&format!(
        "SELECT {COLUMNS} FROM invoice_reminders \
         WHERE company_id = ? AND invoice_id = ? ORDER BY sent_at DESC, id DESC"
    ))
    .bind(company_id)
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Niveau courant d'une facture = `MAX(level_number)` des rappels **non-annulés**
/// (0 si aucun), sous tx (calcul à faire sous le même verrou que l'insertion).
pub async fn current_level_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    invoice_id: i64,
) -> Result<i16, DbError> {
    let max: Option<i16> = sqlx::query_scalar(
        "SELECT MAX(level_number) FROM invoice_reminders \
         WHERE company_id = ? AND invoice_id = ? AND cancelled_at IS NULL",
    )
    .bind(company_id)
    .bind(invoice_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(max.unwrap_or(0))
}

/// Variante pool (lecture hors tx) — utile aux tests et lectures simples.
pub async fn current_level(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
) -> Result<i16, DbError> {
    let max: Option<i16> = sqlx::query_scalar(
        "SELECT MAX(level_number) FROM invoice_reminders \
         WHERE company_id = ? AND invoice_id = ? AND cancelled_at IS NULL",
    )
    .bind(company_id)
    .bind(invoice_id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)?;
    Ok(max.unwrap_or(0))
}

/// Append d'un rappel (manuel en 21-5a ; e-mail en 21-5b réutilisera ce chemin), sous tx.
/// Le caller écrit l'audit `invoice.reminder_sent` dans la MÊME tx.
pub async fn insert_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: &NewInvoiceReminder,
) -> Result<InvoiceReminder, DbError> {
    let result = sqlx::query(
        "INSERT INTO invoice_reminders \
         (company_id, invoice_id, level_number, fee_amount, sent_at, channel, sent_to, \
          subject, body, note, actor_user_id) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.invoice_id)
    .bind(new.level_number)
    .bind(new.fee_amount)
    .bind(new.sent_at)
    .bind(new.channel.as_str())
    .bind(new.sent_to.as_deref())
    .bind(&new.subject)
    .bind(&new.body)
    .bind(new.note.as_deref())
    .bind(new.actor_user_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    find_by_id_for_company(tx, new.company_id, result.last_insert_id() as i64)
        .await?
        .ok_or_else(|| DbError::Invariant("rappel introuvable après insertion".into()))
}

/// Annulation SOFT d'un rappel (Admin) : pose `cancelled_at = UTC_TIMESTAMP(6)`. Scopé
/// company. Idempotent : ne réécrit pas un `cancelled_at` déjà posé (garde `IS NULL`).
/// Retourne `true` si une ligne active a été annulée, `false` sinon (déjà annulé /
/// introuvable / cross-tenant). Le caller écrit l'audit `invoice.reminder_cancelled`.
pub async fn cancel_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<bool, DbError> {
    let rows = sqlx::query(
        "UPDATE invoice_reminders SET cancelled_at = UTC_TIMESTAMP(6) \
         WHERE id = ? AND company_id = ? AND cancelled_at IS NULL",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows > 0)
}
