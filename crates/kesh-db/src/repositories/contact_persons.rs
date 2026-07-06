//! Repository CRUD pour `ContactPerson` (#213) — personnes de contact d'une
//! entreprise (CRM, purement informatif). Scopé `company_id` + `contact_id`.
//!
//! Archivage soft (`active = FALSE`) cohérent avec `contacts`. Verrouillage
//! optimiste (`version`) sur update. Pas d'audit log (donnée informative).

use sqlx::mysql::MySqlPool;

use crate::entities::contact_person::{ContactPerson, ContactPersonUpdate, NewContactPerson};
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, contact_id, first_name, last_name, role, email, phone, \
    active, version, created_at, updated_at";

/// Liste les personnes actives d'un contact (scopée company, tri par nom).
pub async fn list_by_contact(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
) -> Result<Vec<ContactPerson>, DbError> {
    sqlx::query_as::<_, ContactPerson>(&format!(
        "SELECT {COLUMNS} FROM contact_persons \
         WHERE company_id = ? AND contact_id = ? AND active = TRUE \
         ORDER BY last_name, first_name"
    ))
    .bind(company_id)
    .bind(contact_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Récupère une personne par id, scopée company (anti-IDOR).
pub async fn find_by_id_in_company(
    pool: &MySqlPool,
    id: i64,
    company_id: i64,
) -> Result<Option<ContactPerson>, DbError> {
    sqlx::query_as::<_, ContactPerson>(&format!(
        "SELECT {COLUMNS} FROM contact_persons WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Crée une personne de contact et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewContactPerson) -> Result<ContactPerson, DbError> {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO contact_persons (company_id, contact_id, first_name, last_name, role, email, phone) \
         VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(new.company_id)
    .bind(new.contact_id)
    .bind(&new.first_name)
    .bind(&new.last_name)
    .bind(&new.role)
    .bind(&new.email)
    .bind(&new.phone)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)?;

    find_by_id_in_company(pool, id, new.company_id)
        .await?
        .ok_or_else(|| DbError::Invariant(format!("contact_person {id} introuvable après INSERT")))
}

/// Met à jour une personne active (verrouillage optimiste).
pub async fn update(
    pool: &MySqlPool,
    id: i64,
    company_id: i64,
    version: i32,
    changes: ContactPersonUpdate,
) -> Result<ContactPerson, DbError> {
    let rows = sqlx::query(
        "UPDATE contact_persons SET first_name = ?, last_name = ?, role = ?, email = ?, phone = ?, \
         version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND active = TRUE",
    )
    .bind(&changes.first_name)
    .bind(&changes.last_name)
    .bind(&changes.role)
    .bind(&changes.email)
    .bind(&changes.phone)
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Distinguer NotFound d'un conflit de version.
        return match find_by_id_in_company(pool, id, company_id).await? {
            None => Err(DbError::NotFound),
            Some(_) => Err(DbError::OptimisticLockConflict),
        };
    }
    find_by_id_in_company(pool, id, company_id)
        .await?
        .ok_or(DbError::NotFound)
}

/// Archive (soft-delete) une personne de contact.
pub async fn archive(pool: &MySqlPool, id: i64, company_id: i64) -> Result<(), DbError> {
    let rows = sqlx::query(
        "UPDATE contact_persons SET active = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND active = TRUE",
    )
    .bind(id)
    .bind(company_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}
