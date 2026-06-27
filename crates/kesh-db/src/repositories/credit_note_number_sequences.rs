//! Repository pour `credit_note_number_sequences` (Story 12.1 — FR37).
//!
//! Séquence de numérotation des avoirs, **indépendante** de celle des
//! factures, par `(company_id, fiscal_year_id)`. Même garantie « sans trou »
//! que `invoice_number_sequences` (SELECT FOR UPDATE + rollback de la
//! transaction d'émission annulant l'incrément).

use crate::errors::{DbError, map_db_error};

/// Retourne le prochain numéro d'avoir pour `(company_id, fiscal_year_id)`
/// et incrémente atomiquement le compteur.
///
/// **Exige** une transaction ouverte par le caller (voir
/// `credit_notes::create_credit_note`). Prend un `SELECT ... FOR UPDATE` sur
/// la row, lazy-insert idempotent si absente. Garantie « sans trou » : le
/// rollback de la transaction d'émission annule l'incrément.
pub async fn next_number_for(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    fiscal_year_id: i64,
) -> Result<i64, DbError> {
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT next_number FROM credit_note_number_sequences \
         WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let next_number = match current {
        Some(n) => n,
        None => {
            sqlx::query(
                "INSERT IGNORE INTO credit_note_number_sequences \
                 (company_id, fiscal_year_id, next_number) VALUES (?, ?, 1)",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;

            sqlx::query_scalar(
                "SELECT next_number FROM credit_note_number_sequences \
                 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?
        }
    };

    let rows = sqlx::query(
        "UPDATE credit_note_number_sequences \
         SET next_number = next_number + 1, version = version + 1 \
         WHERE company_id = ? AND fiscal_year_id = ?",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows != 1 {
        return Err(DbError::Invariant(format!(
            "UPDATE credit_note_number_sequences : {rows} rows affectées (attendu 1)"
        )));
    }

    Ok(next_number)
}
