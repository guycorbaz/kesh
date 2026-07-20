//! Repository des niveaux de rappel (`dunning_levels`) — collection company-scoped.
//!
//! Calqué sur `vat_rates` : mutations prenant une `&mut Transaction` (le handler ouvre
//! la tx, prend le sentinel lock via [`crate::repositories::bank_accounts::acquire_company_sentinel_lock`],
//! mute, audit, commit), verrou optimiste `version` (double garde). **Différence clé**
//! vs `vat_rates` : hard-delete + renumérotation (pas de soft-delete `active`) — les niveaux
//! sont numérotés contigus (D5) et l'historique est protégé par les snapshots
//! `invoice_reminders` (21-5a).

use crate::entities::{DunningLevel, NewDunningLevel, UpdateDunningLevel};
use crate::errors::{DbError, map_db_error};
use sqlx::{MySql, MySqlPool, Transaction};

const COLUMNS: &str =
    "id, company_id, level_number, delay_days, fee_amount, version, created_at, updated_at";

/// Tous les niveaux d'une company, ordonnés par numéro de niveau.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<DunningLevel>, DbError> {
    sqlx::query_as::<_, DunningLevel>(&format!(
        "SELECT {COLUMNS} FROM dunning_levels WHERE company_id = ? ORDER BY level_number"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Nombre de niveaux configurés pour une company (borne de cascade des templates,
/// éligibilité 21-5a). Prend un pool (lecture hors tx).
pub async fn count_for_company(pool: &MySqlPool, company_id: i64) -> Result<i64, DbError> {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM dunning_levels WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error)
}

/// Un niveau par son `level_number`, scopé company (lookup du frais snapshot d'un
/// rappel manuel, 21-5a). `None` si le niveau n'existe pas.
pub async fn find_by_level_number(
    pool: &MySqlPool,
    company_id: i64,
    level_number: i16,
) -> Result<Option<DunningLevel>, DbError> {
    sqlx::query_as::<_, DunningLevel>(&format!(
        "SELECT {COLUMNS} FROM dunning_levels WHERE company_id = ? AND level_number = ?"
    ))
    .bind(company_id)
    .bind(level_number)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Le plus haut `level_number` d'une company (0 si aucun) — sous tx.
async fn max_level_in_tx(tx: &mut Transaction<'_, MySql>, company_id: i64) -> Result<i16, DbError> {
    let max: Option<i16> =
        sqlx::query_scalar("SELECT MAX(level_number) FROM dunning_levels WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
    Ok(max.unwrap_or(0))
}

/// Helper interne : un niveau par id, scopé company, sous tx.
pub async fn find_by_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
) -> Result<Option<DunningLevel>, DbError> {
    sqlx::query_as::<_, DunningLevel>(&format!(
        "SELECT {COLUMNS} FROM dunning_levels WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Crée un niveau en l'ajoutant à la suite (`level_number = MAX+1`). À appeler
/// SOUS sentinel lock (le handler l'a pris) pour sérialiser le calcul de MAX+1.
pub async fn create_for_company(
    tx: &mut Transaction<'_, MySql>,
    new: &NewDunningLevel,
) -> Result<DunningLevel, DbError> {
    let next_level = max_level_in_tx(tx, new.company_id).await? + 1;
    let result = sqlx::query(
        "INSERT INTO dunning_levels (company_id, level_number, delay_days, fee_amount, version) \
         VALUES (?, ?, ?, ?, 0)",
    )
    .bind(new.company_id)
    .bind(next_level)
    .bind(new.delay_days)
    .bind(new.fee_amount)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    find_by_id_for_company(tx, new.company_id, result.last_insert_id() as i64)
        .await?
        .ok_or_else(|| DbError::Invariant("niveau de rappel introuvable après insertion".into()))
}

/// Modifie `delay_days` + `fee_amount` (le `level_number` est immutable). Verrou
/// optimiste double garde (pré-check + `rows_affected == 0`).
pub async fn update_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    fields: &UpdateDunningLevel,
    expected_version: i32,
) -> Result<DunningLevel, DbError> {
    let existing = find_by_id_for_company(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let rows = sqlx::query(
        "UPDATE dunning_levels SET delay_days = ?, fee_amount = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ?",
    )
    .bind(fields.delay_days)
    .bind(fields.fee_amount)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    find_by_id_for_company(tx, company_id, id)
        .await?
        .ok_or_else(|| DbError::Invariant("niveau de rappel introuvable après update".into()))
}

/// Supprime un niveau (hard-delete) PUIS renumérote les niveaux suivants pour
/// préserver la contiguïté (D5). À appeler SOUS sentinel lock.
///
/// ⚠️ La renumérotation `level_number - 1` bumpe AUSSI `version` sur les lignes
/// déplacées — sinon le verrou optimiste serait contourné : un client détenant
/// l'ancien `version` d'un niveau renuméroté éditerait silencieusement une ligne
/// qui a changé d'identité fonctionnelle.
///
/// L'historique des rappels déjà envoyés est protégé par les snapshots
/// `invoice_reminders` (21-5a), d'où le hard-delete (vs soft-delete `vat_rates`).
pub async fn delete_and_renumber(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    expected_version: i32,
) -> Result<(), DbError> {
    let existing = find_by_id_for_company(tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;
    if existing.version != expected_version {
        return Err(DbError::OptimisticLockConflict);
    }

    let deleted =
        sqlx::query("DELETE FROM dunning_levels WHERE id = ? AND company_id = ? AND version = ?")
            .bind(id)
            .bind(company_id)
            .bind(expected_version)
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();
    if deleted == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    sqlx::query(
        "UPDATE dunning_levels SET level_number = level_number - 1, version = version + 1 \
         WHERE company_id = ? AND level_number > ?",
    )
    .bind(company_id)
    .bind(existing.level_number)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(())
}
