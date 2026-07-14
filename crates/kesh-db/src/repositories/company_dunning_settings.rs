//! Repository du singleton `company_dunning_settings` + seed idempotent des niveaux
//! par défaut.
//!
//! Calqué sur `company_invoice_settings` : get-or-create via `INSERT IGNORE` + `SELECT`
//! (course absorbée par le PK `company_id`), verrou optimiste `version`, no-op KF-004,
//! audit. `seeded_at` n'est jamais piloté par `update` — uniquement par [`ensure_seeded_in_tx`].

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{CompanyDunningSettings, CompanyDunningSettingsUpdate};
use crate::errors::{DbError, map_db_error};
use crate::repositories::{audit_log, bank_accounts};
use sqlx::{MySql, MySqlPool, Transaction};

const COLUMNS: &str = "company_id, grace_period_days, seeded_at, version, created_at, updated_at";

/// Grâce par défaut (jours) posée au seed.
const DEFAULT_GRACE_DAYS: i32 = 5;

fn settings_snapshot_json(s: &CompanyDunningSettings) -> serde_json::Value {
    serde_json::json!({
        "companyId": s.company_id,
        "gracePeriodDays": s.grace_period_days,
        "seededAt": s.seeded_at,
        "version": s.version,
    })
}

fn is_no_op_change(
    before: &CompanyDunningSettings,
    changes: &CompanyDunningSettingsUpdate,
) -> bool {
    before.grace_period_days == changes.grace_period_days
}

/// Get-or-create au niveau pool. MIRROR : garder synchronisé avec la variante `_in_tx`.
pub async fn get_or_create_default(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<CompanyDunningSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let settings = get_or_create_default_in_tx(&mut tx, company_id).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(settings)
}

/// Get-or-create sous transaction (verrouille la row `FOR UPDATE`). MIRROR de
/// [`get_or_create_default`].
pub async fn get_or_create_default_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<CompanyDunningSettings, DbError> {
    sqlx::query("INSERT IGNORE INTO company_dunning_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    sqlx::query_as::<_, CompanyDunningSettings>(&format!(
        "SELECT {COLUMNS} FROM company_dunning_settings WHERE company_id = ? FOR UPDATE"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Met à jour la grâce (verrou optimiste + no-op KF-004 + audit). `seeded_at`
/// n'est jamais modifié ici.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    expected_version: i32,
    user_id: i64,
    changes: CompanyDunningSettingsUpdate,
) -> Result<CompanyDunningSettings, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    sqlx::query("INSERT IGNORE INTO company_dunning_settings (company_id) VALUES (?)")
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = sqlx::query_as::<_, CompanyDunningSettings>(&format!(
        "SELECT {COLUMNS} FROM company_dunning_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    if is_no_op_change(&before, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    let rows = sqlx::query(
        "UPDATE company_dunning_settings SET grace_period_days = ?, version = version + 1 \
         WHERE company_id = ? AND version = ?",
    )
    .bind(changes.grace_period_days)
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

    let after = sqlx::query_as::<_, CompanyDunningSettings>(&format!(
        "SELECT {COLUMNS} FROM company_dunning_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let details = serde_json::json!({
        "before": settings_snapshot_json(&before),
        "after": settings_snapshot_json(&after),
    });
    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "company_dunning_settings.updated",
            "company_dunning_settings",
            company_id,
            Some(details),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Niveaux par défaut seedés au premier accès : (délai depuis étape précédente, frais CHF).
/// 1er rappel +10j/0, 2e +10j/20, mise en demeure +10j/40 (D7).
const DEFAULT_LEVELS: [(i32, &str); 3] = [(10, "0.00"), (10, "20.00"), (10, "40.00")];

/// Seed idempotent des 3 niveaux par défaut + grâce, SOUS sentinel lock (D7).
///
/// - Prend le sentinel lock company (sérialise avec les mutations CRUD dunning).
/// - Crée la row settings si absente (`get_or_create_default_in_tx`).
/// - **Si `seeded_at IS NULL`** : insère les 3 niveaux + pose `grace = 5`, `seeded_at = NOW()`.
/// - **Si `seeded_at IS NOT NULL`** : no-op (déjà seedé une fois → une table `dunning_levels`
///   vide = désactivation VOLONTAIRE, pas de résurrection).
///
/// Idempotent et race-safe : deux appels concurrents se sérialisent sur le sentinel lock ;
/// le second voit `seeded_at` posé et ne re-seede pas.
pub async fn ensure_seeded_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<CompanyDunningSettings, DbError> {
    bank_accounts::acquire_company_sentinel_lock(tx, company_id).await?;
    let settings = get_or_create_default_in_tx(tx, company_id).await?;

    if settings.seeded_at.is_some() {
        return Ok(settings);
    }

    for (level_idx, (delay_days, fee)) in DEFAULT_LEVELS.iter().enumerate() {
        let level_number = (level_idx + 1) as i16;
        let fee: rust_decimal::Decimal = fee.parse().expect("frais de niveau par défaut valide");
        sqlx::query(
            "INSERT INTO dunning_levels (company_id, level_number, delay_days, fee_amount, version) \
             VALUES (?, ?, ?, ?, 0)",
        )
        .bind(company_id)
        .bind(level_number)
        .bind(delay_days)
        .bind(fee)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    }

    sqlx::query(
        "UPDATE company_dunning_settings \
         SET grace_period_days = ?, seeded_at = NOW(3), version = version + 1 \
         WHERE company_id = ?",
    )
    .bind(DEFAULT_GRACE_DAYS)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Relire l'état post-seed.
    sqlx::query_as::<_, CompanyDunningSettings>(&format!(
        "SELECT {COLUMNS} FROM company_dunning_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)
}
