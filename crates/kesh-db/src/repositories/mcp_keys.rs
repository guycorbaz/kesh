//! Repository des clés du serveur MCP (`mcp_keys`, Story 32-5a, refs #723).
//!
//! Motif du dépôt des clés d'API (`api_keys.rs`, Story 17-2a) : tout est
//! scopé par société (KF-002) — `find_by_id_for_company` rend `None` pour une
//! clé d'une autre société, jamais une fuite d'existence ; le secret n'est
//! jamais stocké, la recherche d'authentification passe par le condensat
//! (`key_hash`, index UNIQUE).
//!
//! ⚠️ La colonne `scopes` est `JSON` : MariaDB la rend en `BLOB`, d'où le
//! `CAST(scopes AS CHAR)` de chaque lecture (motif `bank_profiles`).
//!
//! **Différence avec les clés d'API** : la recherche d'authentification
//! ([`find_active_auth_by_key_hash`]) joint aussi `companies` — une clé
//! orpheline (société absente) n'est jamais trouvée, avant même toute
//! réparation (C-32-5-12).

use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::mcp_key::{McpKey, McpScopes, NewMcpKey, SendValidation};
use crate::entities::user::Role;
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, created_by_user_id, name, key_hash, \
     CAST(scopes AS CHAR) AS scopes, send_validation, expires_at, last_used_at, revoked_at, \
     version, created_at, updated_at";

/// Ligne d'authentification d'une clé MCP : la clé, jointe à son créateur
/// (rôle et état **courants**) et à sa société.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct McpKeyAuthRow {
    pub mcp_key_id: i64,
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub scopes: McpScopes,
    pub send_validation: SendValidation,
    pub expires_at: Option<NaiveDateTime>,
    pub creator_role: Role,
    pub creator_active: bool,
}

/// Insère une clé dans une transaction en cours et rend l'entité persistée.
/// Le condensat arrive déjà calculé (`kesh-api`).
pub async fn create_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: NewMcpKey,
) -> Result<McpKey, DbError> {
    let result = sqlx::query(
        "INSERT INTO mcp_keys \
         (company_id, created_by_user_id, name, key_hash, scopes, send_validation, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.created_by_user_id)
    .bind(&new.name)
    .bind(&new.key_hash)
    .bind(&new.scopes)
    .bind(new.send_validation)
    .bind(new.expires_at)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT mcp_keys".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    sqlx::query_as::<_, McpKey>(&format!("SELECT {COLUMNS} FROM mcp_keys WHERE id = ?"))
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Liste les clés d'une société, les plus récentes d'abord ; les révoquées
/// seulement si `include_revoked`.
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_revoked: bool,
) -> Result<Vec<McpKey>, DbError> {
    let filtre = if include_revoked {
        ""
    } else {
        " AND revoked_at IS NULL"
    };
    sqlx::query_as::<_, McpKey>(&format!(
        "SELECT {COLUMNS} FROM mcp_keys WHERE company_id = ?{filtre} \
         ORDER BY created_at DESC, id DESC"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Cherche une clé par id, **scopée par société** : `None` si elle n'existe
/// pas ou appartient à une autre société.
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<McpKey>, DbError> {
    sqlx::query_as::<_, McpKey>(&format!(
        "SELECT {COLUMNS} FROM mcp_keys WHERE company_id = ? AND id = ? LIMIT 1"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Recherche d'authentification : une clé **active** (non révoquée, non
/// expirée) par son condensat, jointe à son créateur **et à sa société**.
/// `None` si aucune clé active ou si la société a disparu (clé orpheline).
///
/// Le rôle et l'état du créateur sont rendus, non filtrés : c'est
/// l'authentification (32-5b) qui décide de leur sens.
pub async fn find_active_auth_by_key_hash(
    pool: &MySqlPool,
    key_hash: &str,
) -> Result<Option<McpKeyAuthRow>, DbError> {
    sqlx::query_as::<_, McpKeyAuthRow>(
        "SELECT k.id AS mcp_key_id, k.company_id AS company_id, \
                k.created_by_user_id AS created_by_user_id, \
                CAST(k.scopes AS CHAR) AS scopes, k.send_validation AS send_validation, \
                k.expires_at AS expires_at, u.role AS creator_role, u.active AS creator_active \
         FROM mcp_keys k \
         JOIN users u ON u.id = k.created_by_user_id \
         JOIN companies c ON c.id = k.company_id \
         WHERE k.key_hash = ? AND k.revoked_at IS NULL \
           AND (k.expires_at IS NULL OR k.expires_at > NOW(3)) \
         LIMIT 1",
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Révoque une clé (verrou optimiste sur `version`, scopé par société).
///
/// `rows_affected == 0` → `OptimisticLockConflict` (version périmée, clé déjà
/// révoquée ou d'une autre société). L'appelant vérifie l'existence par
/// [`find_by_id_for_company`] au préalable (404 distinct).
pub async fn revoke_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    version: i32,
) -> Result<(), DbError> {
    let rows = sqlx::query(
        "UPDATE mcp_keys \
         SET revoked_at = NOW(3), version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND revoked_at IS NULL",
    )
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }
    Ok(())
}

/// Change la politique de validation d'une clé **non révoquée** (verrou
/// optimiste, scopé par société). `rows_affected == 0` →
/// `OptimisticLockConflict` (version périmée, clé révoquée ou d'une autre
/// société). Rend la version nouvelle.
pub async fn set_send_validation_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    send_validation: SendValidation,
) -> Result<i32, DbError> {
    let rows = sqlx::query(
        "UPDATE mcp_keys \
         SET send_validation = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND revoked_at IS NULL",
    )
    .bind(send_validation)
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }
    Ok(version + 1)
}

/// Pose `last_used_at = NOW(3)` (*best-effort* : l'appelant journalise un
/// échec sans faire échouer la requête). Ne réveille jamais une clé révoquée
/// entre la recherche et cette écriture.
pub async fn touch_last_used(pool: &MySqlPool, key_hash: &str) -> Result<(), DbError> {
    sqlx::query(
        "UPDATE mcp_keys SET last_used_at = NOW(3) WHERE key_hash = ? AND revoked_at IS NULL",
    )
    .bind(key_hash)
    .execute(pool)
    .await
    .map_err(map_db_error)?;
    Ok(())
}
