//! Repository pour `FiscalYear`.
//!
//! **Pas de `delete`** : conformément au Code des obligations suisse
//! (art. 957-964), les exercices comptables ne sont jamais supprimés.
//! Les seules transitions autorisées sont `Open` → `Closed` via
//! `close`. La ré-ouverture d'un exercice clos n'est **pas** autorisée
//! au niveau du repository (garde-fou DB).

use chrono::NaiveDate;
use sqlx::mysql::MySqlPool;

use crate::entities::{FiscalYear, NewFiscalYear};
use crate::errors::{map_db_error, DbError};
use crate::repositories::MAX_LIST_LIMIT;

const FIND_BY_ID_SQL: &str =
    "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE id = ?";

const LIST_BY_COMPANY_SQL: &str =
    "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
     FROM fiscal_years WHERE company_id = ? ORDER BY start_date LIMIT ?";

/// Crée un nouvel exercice comptable.
pub async fn create(pool: &MySqlPool, new: NewFiscalYear) -> Result<FiscalYear, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) \
         VALUES (?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(&new.name)
    .bind(new.start_date)
    .bind(new.end_date)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT fiscal_year".into(),
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

    let fy_opt = sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let fy = match fy_opt {
        Some(f) => f,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "fiscal_year {id} introuvable après INSERT"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(fy)
}

/// Retrouve un exercice par son id.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retourne l'exercice (ouvert OU clos) qui couvre une date donnée pour
/// une company, ou `None` si aucun exercice ne correspond.
///
/// **Lock-free** : cette fonction est utilisée comme pré-check côté
/// route handler pour distinguer les erreurs `NO_FISCAL_YEAR` et
/// `FISCAL_YEAR_CLOSED`. Le vrai lock contre les clôtures concurrentes
/// est repris dans `journal_entries::create` via `SELECT ... FOR UPDATE`.
pub async fn find_covering_date(
    pool: &MySqlPool,
    company_id: i64,
    date: NaiveDate,
) -> Result<Option<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(
        "SELECT id, company_id, name, start_date, end_date, status, created_at, updated_at \
         FROM fiscal_years \
         WHERE company_id = ? AND start_date <= ? AND end_date >= ? \
         LIMIT 1",
    )
    .bind(company_id)
    .bind(date)
    .bind(date)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Liste les exercices d'une company, triés par date de début.
///
/// Limité à `MAX_LIST_LIMIT` exercices. Une entreprise a typiquement moins
/// de 100 exercices sur toute sa durée de vie, donc la limite n'est pas
/// atteignable en pratique mais garantit une borne défensive contre les OOM.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<FiscalYear>, DbError> {
    sqlx::query_as::<_, FiscalYear>(LIST_BY_COMPANY_SQL)
        .bind(company_id)
        .bind(MAX_LIST_LIMIT)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Clôture un exercice (`Open` → `Closed`).
///
/// Transaction atomique avec guard SQL `WHERE status = 'Open'`. Conformément
/// au CO suisse (art. 957-964), un exercice clos ne peut JAMAIS être
/// rouvert via cette API.
///
/// Retourne :
/// - `DbError::NotFound` si l'exercice n'existe pas
/// - `DbError::IllegalStateTransition` si l'exercice est déjà clos
///   (le guard `WHERE status = 'Open'` a échoué — transition interdite)
pub async fn close(pool: &MySqlPool, id: i64) -> Result<FiscalYear, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let rows_affected = sqlx::query(
        "UPDATE fiscal_years SET status = 'Closed' WHERE id = ? AND status = 'Open'",
    )
    .bind(id)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        // Soit l'exercice n'existe pas, soit il est déjà clos
        let current: Option<(String,)> =
            sqlx::query_as("SELECT status FROM fiscal_years WHERE id = ?")
                .bind(id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;
        tx.rollback().await.map_err(map_db_error)?;
        return match current {
            None => Err(DbError::NotFound),
            Some((status,)) if status == "Closed" => Err(DbError::IllegalStateTransition(
                format!("fiscal_year {id} déjà clos — réouverture interdite (CO art. 957-964)"),
            )),
            // Défensif : sous REPEATABLE READ InnoDB et dans la même transaction,
            // le SELECT post-UPDATE devrait voir cohérent. Cette branche ne peut
            // survenir que via un trigger inattendu ou une isolation plus faible.
            Some((status,)) if status == "Open" => Err(DbError::Invariant(format!(
                "fiscal_year {id} est Open mais l'UPDATE n'a affecté aucune ligne \
                 (race condition ou trigger inattendu)"
            ))),
            Some((status,)) => Err(DbError::Invariant(format!(
                "fiscal_year {id} a un statut inattendu hors schéma : {status}"
            ))),
        };
    }

    let fy_opt = sqlx::query_as::<_, FiscalYear>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let fy = match fy_opt {
        Some(f) => f,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "fiscal_year {id} introuvable après clôture"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(fy)
}
