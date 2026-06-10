//! Repository pour `PasswordResetToken` (magic-link recovery, Story 17-4a).
//!
//! **Sécurité (DC3)** : on ne stocke jamais le token en clair. Le lookup de
//! validation [`find_valid_by_hash`] se fait par `token_hash = SHA-256(token)`
//! via l'index UNIQUE (lookup O(1)), avec filtre `used_at IS NULL AND
//! expires_at > NOW(3)` (usage unique + TTL, DC8).
//!
//! Le hash et les timestamps sont fournis par `kesh-api` (génération du token
//! brut + SHA-256 + calcul `expires_at = now + 30min`) — ce crate ne génère ni
//! ne hache jamais.

use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;

use crate::entities::password_reset_token::PasswordResetToken;
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "id, user_id, token_hash, expires_at, used_at, created_at";

/// Insère un nouveau token de réinitialisation et retourne l'entité persistée.
///
/// `token_hash` = `SHA-256(token)` hex (fourni par l'appelant). `expires_at`
/// est calculé par l'appelant (`now + 30min`, DC8).
pub async fn create(
    pool: &MySqlPool,
    user_id: i64,
    token_hash: &str,
    expires_at: NaiveDateTime,
) -> Result<PasswordResetToken, DbError> {
    let result = sqlx::query(
        "INSERT INTO password_reset_tokens (user_id, token_hash, expires_at) VALUES (?, ?, ?)",
    )
    .bind(user_id)
    .bind(token_hash)
    .bind(expires_at)
    .execute(pool)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT password_reset_tokens".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    sqlx::query_as::<_, PasswordResetToken>(&format!(
        "SELECT {COLUMNS} FROM password_reset_tokens WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

/// Retrouve un token **valide** par son hash : non utilisé et non expiré.
///
/// Filtre `used_at IS NULL AND expires_at > NOW(3)` (DC8). Retourne `None` si
/// le hash est inconnu, déjà consommé, ou expiré — l'appelant traite ces trois
/// cas de manière indistincte (`400 INVALID_OR_EXPIRED_TOKEN`, anti-fuite).
pub async fn find_valid_by_hash(
    pool: &MySqlPool,
    token_hash: &str,
) -> Result<Option<PasswordResetToken>, DbError> {
    sqlx::query_as::<_, PasswordResetToken>(&format!(
        "SELECT {COLUMNS} FROM password_reset_tokens \
         WHERE token_hash = ? AND used_at IS NULL AND expires_at > NOW(3)"
    ))
    .bind(token_hash)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)
}

/// Marque un token comme consommé (`used_at = NOW(3)`), usage unique (DC8).
///
/// Garde `AND used_at IS NULL` dans l'UPDATE : défense en profondeur au niveau
/// DB contre un double-consume. `mark_used` prend `&MySqlPool` (pas la
/// transaction de l'appelant), donc l'unicité ne peut PAS être garantie par le
/// seul contrat applicatif 17-4c — une fenêtre TOCTOU existe entre
/// [`find_valid_by_hash`] et `mark_used` sous requêtes concurrentes. La garde
/// SQL la ferme : seule la 1re consommation affecte une ligne et fige
/// l'horodatage réel. Un 2e appel sur un token déjà consommé (ou un `id`
/// inexistant) affecte 0 ligne → `DbError::NotFound` (17-4c le mappe en
/// `400 INVALID_OR_EXPIRED_TOKEN`).
pub async fn mark_used(pool: &MySqlPool, id: i64) -> Result<(), DbError> {
    let rows_affected = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = NOW(3) WHERE id = ? AND used_at IS NULL",
    )
    .bind(id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows_affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Invalide tous les tokens pendants d'un utilisateur (marque `used_at`).
///
/// Appelé à la création d'un nouveau token forgot-password pour éviter
/// l'accumulation de tokens valides simultanés. Idempotent (0 ligne si aucun
/// token pendant). Retourne le nombre de tokens invalidés.
pub async fn invalidate_all_for_user(pool: &MySqlPool, user_id: i64) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(
        "UPDATE password_reset_tokens SET used_at = NOW(3) WHERE user_id = ? AND used_at IS NULL",
    )
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows_affected)
}
