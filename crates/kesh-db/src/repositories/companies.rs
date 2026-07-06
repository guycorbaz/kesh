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
use crate::errors::{DbError, map_db_error};
use crate::repositories::MAX_LIST_LIMIT;

const FIND_BY_ID_SQL: &str = "SELECT id, name, first_name, last_name, address, address_street, address_building, \
            address_postal_code, address_city, address_country, ide_number, org_type, \
            accounting_language, instance_language, is_stub, version, created_at, updated_at \
     FROM companies WHERE id = ?";

const LIST_SQL: &str = "SELECT id, name, first_name, last_name, address, address_street, address_building, \
            address_postal_code, address_city, address_country, ide_number, org_type, \
            accounting_language, instance_language, is_stub, version, created_at, updated_at \
     FROM companies ORDER BY id LIMIT ? OFFSET ?";

/// Crée une nouvelle company et retourne l'entité persistée.
///
/// INSERT puis SELECT dans une transaction atomique pour éviter une
/// race window avec un DELETE concurrent.
pub async fn create(pool: &MySqlPool, new: NewCompany) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Colonne `address` dérivée (#213) : recomposée depuis les champs structurés.
    let addr = &new.address_structured;
    let result = sqlx::query(
        "INSERT INTO companies (name, first_name, last_name, address, address_street, address_building, \
             address_postal_code, address_city, address_country, ide_number, org_type, \
             accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(&new.name)
    .bind(&new.first_name)
    .bind(&new.last_name)
    .bind(addr.combined())
    .bind(&addr.street)
    .bind(&addr.building)
    .bind(&addr.postal_code)
    .bind(&addr.city)
    .bind(&addr.country)
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

/// Compare l'état persisté au payload — `true` si aucun champ métier ne diffère
/// (KF-004 : court-circuit no-op pour ne pas bumper version inutilement).
fn is_no_op_change(before: &Company, changes: &CompanyUpdate) -> bool {
    before.name == changes.name
        && before.first_name == changes.first_name
        && before.last_name == changes.last_name
        && before.structured_address() == changes.address_structured
        && before.ide_number == changes.ide_number
        && before.org_type == changes.org_type
        && before.accounting_language == changes.accounting_language
        && before.instance_language == changes.instance_language
}

/// Met à jour une company avec verrouillage optimiste.
///
/// SELECT before → version check applicatif → court-circuit no-op (KF-004) →
/// UPDATE puis SELECT after, le tout dans une transaction atomique. Retourne
/// `DbError::OptimisticLockConflict` si la version en base ne correspond pas
/// à `version`, ou `DbError::NotFound` si l'entité n'existe pas.
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: CompanyUpdate,
) -> Result<Company, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Snapshot "before" pour permettre la détection no-op (KF-004).
    let before_opt = sqlx::query_as::<_, Company>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before = match before_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(c) if c.version != version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(c) => c,
    };

    // KF-004 : court-circuit no-op AVANT toute mutation.
    // NOTE concurrence (KF-004): sous REPEATABLE READ + plain SELECT, si une tx
    // parallèle commit entre notre BEGIN et ce check, on retourne notre snapshot
    // stale au lieu d'un 409. Race acceptée v0.1 (cf. spec 7-3 §race-condition).
    // Mitigation future: SELECT FOR UPDATE partout (non v0.1).
    if is_no_op_change(&before, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }

    let addr = &changes.address_structured;
    let rows_affected = sqlx::query(
        "UPDATE companies
         SET name = ?, first_name = ?, last_name = ?, address = ?, address_street = ?, address_building = ?,
             address_postal_code = ?, address_city = ?, address_country = ?,
             ide_number = ?, org_type = ?,
             accounting_language = ?, instance_language = ?,
             version = version + 1
         WHERE id = ? AND version = ?",
    )
    .bind(&changes.name)
    .bind(&changes.first_name)
    .bind(&changes.last_name)
    .bind(addr.combined())
    .bind(&addr.street)
    .bind(&addr.building)
    .bind(&addr.postal_code)
    .bind(&addr.city)
    .bind(&addr.country)
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
        // Défensif : ne devrait pas arriver puisque la version-check applicative
        // a déjà validé la version. Race théorique entre le SELECT et l'UPDATE.
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
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
