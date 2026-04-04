//! Repository CRUD pour `Company`.
//!
//! MySQL/MariaDB n'a pas de clause `RETURNING` (contrairement à Postgres),
//! d'où le pattern `create` en deux étapes : INSERT puis SELECT via `find_by_id`.
//! Pour garantir l'atomicité INSERT+SELECT (et éviter une race window avec un
//! éventuel DELETE concurrent), les opérations write utilisent une transaction.
//!
//! Utilise les variantes non-macro `sqlx::query_as::<_, T>("...")` pour
//! éviter la dépendance à une DB live au moment du build.

use sqlx::mysql::MySqlPool;

use crate::entities::{Company, CompanyUpdate, NewCompany};
use crate::errors::{map_db_error, DbError};
use crate::repositories::MAX_LIST_LIMIT;

const FIND_BY_ID_SQL: &str =
    "SELECT id, name, address, ide_number, org_type, accounting_language, \
            instance_language, version, created_at, updated_at \
     FROM companies WHERE id = ?";

const LIST_SQL: &str =
    "SELECT id, name, address, ide_number, org_type, accounting_language, \
            instance_language, version, created_at, updated_at \
     FROM companies ORDER BY id LIMIT ? OFFSET ?";

/// Crée une nouvelle company et retourne l'entité persistée.
///
/// INSERT puis SELECT dans une transaction atomique pour éviter une
/// race window avec un DELETE concurrent.
pub async fn create(pool: &MySqlPool, new: NewCompany) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO companies (name, address, ide_number, org_type, accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(&new.name)
    .bind(&new.address)
    .bind(&new.ide_number)
    .bind(new.org_type)
    .bind(new.accounting_language)
    .bind(new.instance_language)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    // Valider que l'AUTO_INCREMENT a bien produit un id exploitable
    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT (AUTO_INCREMENT manquant ?)".into(),
        ));
    }
    let id = match i64::try_from(last_id) {
        Ok(v) => v,
        Err(_) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "last_insert_id {last_id} dépasse i64::MAX"
            )));
        }
    };

    let company_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let company = match company_opt {
        Some(c) => c,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "company {id} introuvable après INSERT"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(company)
}

/// Retrouve une company par son id. Retourne `None` si absente.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<Company>, DbError> {
    sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Liste les companies avec pagination offset/limit.
///
/// `limit` est clampé dans `[0, MAX_LIST_LIMIT]` et `offset` à `>= 0`
/// pour éviter les valeurs invalides et les OOM.
pub async fn list(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<Company>, DbError> {
    let limit = limit.clamp(0, MAX_LIST_LIMIT);
    let offset = offset.max(0);
    sqlx::query_as::<_, Company>(LIST_SQL)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Met à jour une company avec verrouillage optimiste.
///
/// UPDATE puis SELECT dans une transaction atomique. Retourne
/// `DbError::OptimisticLockConflict` si la version en base ne correspond pas
/// à `version`, ou `DbError::NotFound` si l'entité n'existe pas.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: CompanyUpdate,
) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let rows_affected = sqlx::query(
        "UPDATE companies
         SET name = ?, address = ?, ide_number = ?, org_type = ?,
             accounting_language = ?, instance_language = ?,
             version = version + 1
         WHERE id = ? AND version = ?",
    )
    .bind(&changes.name)
    .bind(&changes.address)
    .bind(&changes.ide_number)
    .bind(changes.org_type)
    .bind(changes.accounting_language)
    .bind(changes.instance_language)
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // version += 1 garantit toujours un changement si match → 0 signifie stale ou absent.
        let exists = sqlx::query_as::<_, (i64,)>("SELECT id FROM companies WHERE id = ?")
            .bind(id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
        tx.rollback().await.map_err(map_db_error)?;
        return match exists {
            None => Err(DbError::NotFound),
            Some(_) => Err(DbError::OptimisticLockConflict),
        };
    }

    let company_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    // Défensif : sous REPEATABLE READ InnoDB dans la même transaction, le SELECT
    // après un UPDATE `rows_affected > 0` retourne toujours la ligne mise à jour.
    // Cette branche est techniquement unreachable mais préservée comme garde-fou.
    let company = match company_opt {
        Some(c) => c,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "company {id} introuvable après UPDATE réussi"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(company)
}
