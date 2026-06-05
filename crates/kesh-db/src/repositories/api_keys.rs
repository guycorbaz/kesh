//! Repository CRUD pour `ApiKey` (clés d'accès API externes / PAT).
//!
//! Story 17-2a (#100). Multi-tenant scoping strict (KF-002) : toutes les
//! lectures/mutations sont scopées `company_id`, et `find_by_id_for_company`
//! retourne `None` (jamais un leak d'existence) pour une clé d'une autre
//! company.
//!
//! **Sécurité (DC1)** : on ne stocke jamais le secret en clair. Le lookup
//! d'authentification [`find_active_auth_by_key_hash`] se fait par
//! `key_hash = SHA-256(token)` via l'index UNIQUE (lookup O(1)).

use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::api_key::{ApiKey, ApiKeyScope, NewApiKey};
use crate::entities::user::Role;
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, company_id, created_by_user_id, name, key_hash, scope, expires_at, last_used_at, revoked_at, version, created_at, updated_at";

/// Ligne d'authentification PAT — jointure `api_keys` × `users` permettant de
/// construire le `CurrentUser` en **une seule requête** (DC2). Relit l'état
/// COURANT du créateur (`creator_role` / `creator_active`) → une désactivation
/// ou un changement de rôle invalide immédiatement le PAT.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ApiKeyAuthRow {
    pub api_key_id: i64,
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub scope: ApiKeyScope,
    pub expires_at: Option<NaiveDateTime>,
    pub creator_role: Role,
    pub creator_active: bool,
}

/// Insère une nouvelle clé API dans une transaction en cours et retourne
/// l'entité persistée. Le `key_hash` (SHA-256 hex) est fourni par l'appelant
/// (`kesh-api::auth::api_key::generate_pat`) — ce crate ne hache jamais.
pub async fn create_in_tx(
    tx: &mut Transaction<'_, MySql>,
    new: NewApiKey,
) -> Result<ApiKey, DbError> {
    let result = sqlx::query(
        "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope, expires_at) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.created_by_user_id)
    .bind(&new.name)
    .bind(&new.key_hash)
    .bind(new.scope)
    .bind(new.expires_at)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT api_keys".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    sqlx::query_as::<_, ApiKey>(&format!("SELECT {COLUMNS} FROM api_keys WHERE id = ?"))
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)
}

/// Liste les clés d'une company, triées `created_at DESC`.
///
/// - `include_revoked = false` (défaut) : filtre `revoked_at IS NULL`.
/// - `include_revoked = true` : retourne aussi les révoquées (page de gestion
///   AC7 — l'historique est signifiant, `revoked_at` affiché).
pub async fn list_by_company(
    pool: &MySqlPool,
    company_id: i64,
    include_revoked: bool,
) -> Result<Vec<ApiKey>, DbError> {
    let sql = if include_revoked {
        format!(
            "SELECT {COLUMNS} FROM api_keys WHERE company_id = ? ORDER BY created_at DESC, id DESC"
        )
    } else {
        format!(
            "SELECT {COLUMNS} FROM api_keys WHERE company_id = ? AND revoked_at IS NULL ORDER BY created_at DESC, id DESC"
        )
    };
    sqlx::query_as::<_, ApiKey>(&sql)
        .bind(company_id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Cherche une clé par id **scopée multi-tenant** (anti-énumération KF-002 :
/// `None` si la clé n'existe pas OU appartient à une autre company).
pub async fn find_by_id_for_company(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<ApiKey>, DbError> {
    sqlx::query_as::<_, ApiKey>(&format!(
        "SELECT {COLUMNS} FROM api_keys WHERE company_id = ? AND id = ? LIMIT 1"
    ))
    .bind(company_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Lookup d'authentification PAT (DC2) : retrouve une clé **active**
/// (non révoquée, non expirée) par son `key_hash`, jointe au créateur pour
/// relire son rôle/état courant. Retourne `None` si aucune clé active.
///
/// Filtre : `key_hash = ? AND revoked_at IS NULL AND (expires_at IS NULL OR
/// expires_at > NOW(3))`.
pub async fn find_active_auth_by_key_hash(
    pool: &MySqlPool,
    key_hash: &str,
) -> Result<Option<ApiKeyAuthRow>, DbError> {
    sqlx::query_as::<_, ApiKeyAuthRow>(
        "SELECT k.id AS api_key_id, k.company_id AS company_id, \
                k.created_by_user_id AS created_by_user_id, k.scope AS scope, \
                k.expires_at AS expires_at, u.role AS creator_role, u.active AS creator_active \
         FROM api_keys k \
         JOIN users u ON u.id = k.created_by_user_id \
         WHERE k.key_hash = ? AND k.revoked_at IS NULL \
           AND (k.expires_at IS NULL OR k.expires_at > NOW(3)) \
         LIMIT 1",
    )
    .bind(key_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Révoque (soft-delete) une clé dans une transaction en cours, avec
/// optimistic lock sur `version`. Scopé company (multi-tenant).
///
/// `rows_affected == 0` → `OptimisticLockConflict` (version stale, déjà
/// révoquée, ou clé d'une autre company). Le caller doit avoir vérifié
/// l'existence via [`find_by_id_for_company`] au préalable (404 distinct).
pub async fn revoke_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    id: i64,
    version: i32,
) -> Result<(), DbError> {
    let rows = sqlx::query(
        "UPDATE api_keys \
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

/// Met à jour `last_used_at = NOW(3)` (best-effort, AC5). Non-transactionnel,
/// eventual consistency : un échec est ignoré par l'appelant (n'échoue pas la
/// requête authentifiée). Identifié par `key_hash` (indexé UNIQUE).
pub async fn touch_last_used(pool: &MySqlPool, key_hash: &str) -> Result<(), DbError> {
    sqlx::query("UPDATE api_keys SET last_used_at = NOW(3) WHERE key_hash = ?")
        .bind(key_hash)
        .execute(pool)
        .await
        .map_err(map_db_error)?;
    Ok(())
}
