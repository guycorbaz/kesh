//! Repository CRUD pour `BankAccount`.

use sqlx::mysql::MySqlPool;

use crate::entities::bank_account::{BankAccount, NewBankAccount};
use crate::errors::{map_db_error, DbError};

const FIND_BY_ID_SQL: &str =
    "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at \
     FROM bank_accounts WHERE id = ?";

/// Crée un nouveau compte bancaire et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewBankAccount) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.bank_name)
    .bind(&new.iban)
    .bind(&new.qr_iban)
    .bind(new.is_primary)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT bank_accounts".into(),
        ));
    }
    let id = i64::try_from(last_id).map_err(|_| {
        DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX"))
    })?;

    let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            DbError::Invariant(format!("bank_account {id} introuvable après INSERT"))
        })?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(account)
}

/// Retourne le compte bancaire principal d'une company (ou None).
pub async fn find_primary(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Option<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE LIMIT 1",
    )
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les comptes bancaires d'une company.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<BankAccount>, DbError> {
    sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? ORDER BY is_primary DESC, id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Upsert du compte bancaire principal (idempotent pour retries).
///
/// Utilise SELECT FOR UPDATE dans une transaction unique pour éviter le
/// TOCTOU entre la lecture et l'écriture.
pub async fn upsert_primary(
    pool: &MySqlPool,
    new: NewBankAccount,
) -> Result<BankAccount, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let existing = sqlx::query_as::<_, BankAccount>(
        "SELECT id, company_id, bank_name, iban, qr_iban, is_primary, version, created_at, updated_at \
         FROM bank_accounts WHERE company_id = ? AND is_primary = TRUE LIMIT 1 FOR UPDATE",
    )
    .bind(new.company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    match existing {
        Some(account) => {
            let rows = sqlx::query(
                "UPDATE bank_accounts SET bank_name = ?, iban = ?, qr_iban = ?, version = version + 1 \
                 WHERE id = ? AND version = ?",
            )
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(account.id)
            .bind(account.version)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?
            .rows_affected();

            if rows == 0 {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            }

            let updated = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(account.id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            Ok(updated)
        }
        None => {
            let result = sqlx::query(
                "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(new.company_id)
            .bind(&new.bank_name)
            .bind(&new.iban)
            .bind(&new.qr_iban)
            .bind(new.is_primary)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

            let id = i64::try_from(result.last_insert_id()).map_err(|_| {
                DbError::Invariant("last_insert_id overflow".into())
            })?;

            let account = sqlx::query_as::<_, BankAccount>(FIND_BY_ID_SQL)
                .bind(id)
                .fetch_one(&mut *tx)
                .await
                .map_err(map_db_error)?;

            tx.commit().await.map_err(map_db_error)?;
            Ok(account)
        }
    }
}
