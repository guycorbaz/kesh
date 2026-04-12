//! Repository pour `RefreshToken`.
//!
//! Pattern : requêtes non-macro (`sqlx::query_as::<_, T>`) pour éviter
//! toute dépendance compile-time à une base SQLx. Cohérent avec les
//! autres repositories de `kesh-db`.

use sqlx::mysql::MySqlPool;

use crate::entities::{NewRefreshToken, RefreshToken};
use crate::errors::{DbError, map_db_error};

const FIND_BY_ID_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens WHERE id = ?";

const FIND_ACTIVE_BY_TOKEN_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens \
     WHERE token = ? AND revoked_at IS NULL AND expires_at > NOW(3)";

const FIND_BY_TOKEN_INCLUDE_REVOKED_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens WHERE token = ?";

/// Crée un refresh token et retourne l'entité persistée.
///
/// Pattern transaction INSERT+SELECT pour compatibilité MySQL
/// (pas de `RETURNING`).
pub async fn create(pool: &MySqlPool, new: NewRefreshToken) -> Result<RefreshToken, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result =
        sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
            .bind(new.user_id)
            .bind(&new.token)
            .bind(new.expires_at)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT refresh_token".into(),
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

    let token_opt = sqlx::query_as::<_, RefreshToken>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let token = match token_opt {
        Some(t) => t,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "refresh_token {id} introuvable après INSERT"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(token)
}

/// Retrouve un refresh token actif par sa valeur.
///
/// « Actif » = non révoqué (`revoked_at IS NULL`) **et** non expiré
/// (`expires_at > NOW()`). Les tokens expirés ou révoqués retournent `None`.
pub async fn find_active_by_token(
    pool: &MySqlPool,
    token: &str,
) -> Result<Option<RefreshToken>, DbError> {
    sqlx::query_as::<_, RefreshToken>(FIND_ACTIVE_BY_TOKEN_SQL)
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retrouve un refresh token par sa valeur, y compris les tokens révoqués.
///
/// Utilisé par le flux refresh (story 1.6) pour détecter les replays
/// (token révoqué par rotation → détection de vol).
pub async fn find_by_token_include_revoked(
    pool: &MySqlPool,
    token: &str,
) -> Result<Option<RefreshToken>, DbError> {
    sqlx::query_as::<_, RefreshToken>(FIND_BY_TOKEN_INCLUDE_REVOKED_SQL)
        .bind(token)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Révoque un refresh token par sa valeur avec une raison.
///
/// Retourne `true` si une ligne a été effectivement révoquée (token
/// existait, était actif), `false` sinon (token inexistant, déjà révoqué,
/// ou expiré). Cette sémantique rend le logout **idempotent** : un
/// double appel ne produit pas d'erreur.
pub async fn revoke_by_token(pool: &MySqlPool, token: &str, reason: &str) -> Result<bool, DbError> {
    let rows_affected = sqlx::query(
        "UPDATE refresh_tokens \
         SET revoked_at = NOW(3), revoked_reason = ? \
         WHERE token = ? AND revoked_at IS NULL",
    )
    .bind(reason)
    .bind(token)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    Ok(rows_affected > 0)
}

/// Révoque tous les refresh tokens actifs d'un utilisateur avec une raison.
///
/// Utilisé au changement de mot de passe (reason="password_change"),
/// à la détection de vol (reason="theft_detected"), et à la
/// désactivation de compte (reason="admin_disable", story 1.7).
///
/// Retourne le nombre de tokens effectivement révoqués.
pub async fn revoke_all_for_user(
    pool: &MySqlPool,
    user_id: i64,
    reason: &str,
) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(
        "UPDATE refresh_tokens \
         SET revoked_at = NOW(3), revoked_reason = ? \
         WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(reason)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    Ok(rows_affected)
}

/// Supprime physiquement les tokens expirés ou révoqués depuis plus de
/// `older_than`. Utilisé au démarrage pour le nettoyage (AC8).
pub async fn delete_expired_and_revoked(
    pool: &MySqlPool,
    older_than: chrono::NaiveDateTime,
) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(
        "DELETE FROM refresh_tokens \
         WHERE (revoked_at IS NOT NULL AND revoked_at < ?) \
            OR (expires_at < ?)",
    )
    .bind(older_than)
    .bind(older_than)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    Ok(rows_affected)
}
