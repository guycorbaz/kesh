//! Repository pour le journal d'audit.
//!
//! **Pas de méthode `delete`** : CO art. 957-964 impose la conservation
//! 10 ans. Les entrées sont inamovibles.
//!
//! La méthode principale [`insert_in_tx`] prend une transaction en cours
//! pour garantir l'atomicité avec l'opération auditée (UPDATE/DELETE
//! d'une écriture, etc.). Si la transaction ROLLBACK, l'entrée d'audit
//! disparaît avec le reste — garantie de cohérence.

use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::audit_log::{AuditLogEntry, NewAuditLogEntry};
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, user_id, action, entity_type, entity_id, details_json, created_at";

/// Insère une entrée d'audit dans une transaction en cours.
///
/// **Atomicité critique** : cette fonction prend `&mut Transaction<MySql>`
/// et **ne commit jamais**. Le caller (update/delete d'une écriture)
/// gère le commit global. Si l'opération auditée échoue après l'INSERT
/// audit, le ROLLBACK caller supprime aussi cette entrée d'audit —
/// garantie d'intégrité.
pub async fn insert_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: NewAuditLogEntry,
) -> Result<AuditLogEntry, DbError> {
    let result = sqlx::query(
        "INSERT INTO audit_log (user_id, action, entity_type, entity_id, details_json) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(new.user_id)
    .bind(&new.action)
    .bind(&new.entity_type)
    .bind(new.entity_id)
    .bind(&new.details_json)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // P6 : double check — `last_insert_id == 0` attrape les cas
    // `INSERT IGNORE` (non utilisés ici) ; `rows_affected == 0` attrape
    // un INSERT silencieusement ignoré par un trigger inattendu ou un
    // mode SQL permissif. Les deux gardes sont peu coûteuses.
    if result.rows_affected() == 0 {
        return Err(DbError::Invariant(
            "rows_affected == 0 après INSERT audit_log".into(),
        ));
    }
    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT audit_log".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let entry = sqlx::query_as::<_, AuditLogEntry>(&format!(
        "SELECT {COLUMNS} FROM audit_log WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    Ok(entry)
}

/// Liste les entrées d'audit pour une entité donnée, triées du plus
/// récent au plus ancien.
///
/// Utilisé par les tests d'intégration et par la future UI de
/// consultation (story 3.5 ou post-MVP).
pub async fn find_by_entity(
    pool: &MySqlPool,
    entity_type: &str,
    entity_id: i64,
    limit: i64,
) -> Result<Vec<AuditLogEntry>, DbError> {
    sqlx::query_as::<_, AuditLogEntry>(&format!(
        "SELECT {COLUMNS} FROM audit_log \
         WHERE entity_type = ? AND entity_id = ? \
         ORDER BY created_at DESC, id DESC \
         LIMIT ?"
    ))
    .bind(entity_type)
    .bind(entity_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one admin user")
    }

    #[tokio::test]
    async fn test_insert_and_find() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;

        let entity_id = 999_999_i64; // ID fictif, pas de FK à respecter
        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "test.inserted".to_string(),
                entity_type: "test_entity".to_string(),
                entity_id,
                details_json: Some(json!({"foo": "bar"})),
            },
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted.user_id, user_id);
        assert_eq!(inserted.action, "test.inserted");

        let found = find_by_entity(&pool, "test_entity", entity_id, 10)
            .await
            .unwrap();
        assert!(found.iter().any(|e| e.id == inserted.id));

        // Cleanup
        sqlx::query("DELETE FROM audit_log WHERE id = ?")
            .bind(inserted.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_insert_preserves_json_details() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;
        let entity_id = 888_888_i64;

        let details = json!({
            "before": {"description": "Old", "lines": 2},
            "after": {"description": "New", "lines": 3},
        });

        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "test.json".to_string(),
                entity_type: "test_entity".to_string(),
                entity_id,
                details_json: Some(details.clone()),
            },
        )
        .await
        .unwrap();
        tx.commit().await.unwrap();

        assert_eq!(inserted.details_json.as_ref().unwrap(), &details);

        sqlx::query("DELETE FROM audit_log WHERE id = ?")
            .bind(inserted.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_rollback_preserves_no_audit() {
        let pool = test_pool().await;
        let user_id = get_admin_user_id(&pool).await;
        let entity_id = 777_777_i64;

        let mut tx = pool.begin().await.unwrap();
        let inserted = insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "test.rollback".to_string(),
                entity_type: "test_entity".to_string(),
                entity_id,
                details_json: None,
            },
        )
        .await
        .unwrap();
        let audit_id = inserted.id;
        tx.rollback().await.unwrap();

        // Après rollback, l'entrée ne doit plus exister.
        let exists: Option<i64> = sqlx::query_scalar("SELECT id FROM audit_log WHERE id = ?")
            .bind(audit_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(exists.is_none(), "audit_log entry should be rolled back");
    }
}
