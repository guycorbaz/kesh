//! Repository pour `invoice_number_sequences` (Story 5.2 — FR33).
//!
//! Compteur séquentiel par `(company_id, fiscal_year_id)` incrémenté
//! atomiquement à la validation d'une facture. Garantit « sans trou »
//! par `SELECT ... FOR UPDATE` + rollback de la transaction de
//! validation annulant aussi l'UPDATE du compteur.

use crate::errors::{DbError, map_db_error};

/// Retourne le prochain numéro de facture pour `(company_id, fiscal_year_id)`
/// et incrémente atomiquement le compteur.
///
/// **Exige** une transaction ouverte par le caller avec déjà les locks
/// appropriés (voir `invoices::validate_invoice`, section Concurrence).
/// Cette fonction prend elle-même un `SELECT ... FOR UPDATE` sur la
/// row `(company_id, fiscal_year_id)`.
///
/// Si la row n'existe pas encore, elle est insérée (lazy) via
/// `INSERT IGNORE ... VALUES(company_id, fiscal_year_id, 1)` suivi d'un
/// `SELECT ... FOR UPDATE` retry — idempotent en cas de course.
///
/// Pattern d'incrémentation :
/// 1. `SELECT next_number FOR UPDATE`
/// 2. `UPDATE SET next_number = next_number + 1, version = version + 1`
/// 3. Retourne la valeur lue en (1).
///
/// **Garantie « sans trou »** : si la transaction de validation rollback
/// après cet appel, l'UPDATE du compteur est annulé aussi — le numéro
/// lu n'a pas été consommé.
pub async fn next_number_for(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    fiscal_year_id: i64,
) -> Result<i64, DbError> {
    // Étape 1 : tenter un SELECT FOR UPDATE sur la row existante.
    let current: Option<i64> = sqlx::query_scalar(
        "SELECT next_number FROM invoice_number_sequences \
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
            // Lazy insert : la row n'existe pas pour ce (company, fy).
            // INSERT IGNORE pour idempotence en cas de course (un 2e
            // caller parallèle peut avoir créé la row entre notre SELECT
            // et cet INSERT — IGNORE évite l'erreur 1062).
            sqlx::query(
                "INSERT IGNORE INTO invoice_number_sequences \
                 (company_id, fiscal_year_id, next_number) VALUES (?, ?, 1)",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;

            // Re-SELECT FOR UPDATE pour verrouiller la row (qu'elle
            // vienne d'être créée par nous ou par un concurrent).
            sqlx::query_scalar(
                "SELECT next_number FROM invoice_number_sequences \
                 WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
            )
            .bind(company_id)
            .bind(fiscal_year_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?
        }
    };

    // Étape 2 : incrémenter.
    let rows = sqlx::query(
        "UPDATE invoice_number_sequences \
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
            "UPDATE invoice_number_sequences : {rows} rows affectées (attendu 1)"
        )));
    }

    Ok(next_number)
}
