//! Repository CRUD pour `User`.
//!
//! Le `password_hash` est un texte opaque stocké tel quel (le hachage
//! Argon2id est fait dans `kesh-api`, story 1.5). Ce repository ne fait
//! aucune validation ni transformation du hash.

use sqlx::mysql::MySqlPool;

use crate::entities::{NewUser, User, UserUpdate};
use crate::errors::{map_db_error, DbError};
use crate::repositories::MAX_LIST_LIMIT;

const FIND_BY_ID_SQL: &str =
    "SELECT id, username, password_hash, role, active, version, created_at, updated_at \
     FROM users WHERE id = ?";

const FIND_BY_USERNAME_SQL: &str =
    "SELECT id, username, password_hash, role, active, version, created_at, updated_at \
     FROM users WHERE username = ?";

const LIST_SQL: &str =
    "SELECT id, username, password_hash, role, active, version, created_at, updated_at \
     FROM users ORDER BY id LIMIT ? OFFSET ?";

/// Crée un nouvel utilisateur et retourne l'entité persistée.
pub async fn create(pool: &MySqlPool, new: NewUser) -> Result<User, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active) VALUES (?, ?, ?, ?)",
    )
    .bind(&new.username)
    .bind(&new.password_hash)
    .bind(new.role)
    .bind(new.active)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT user".into(),
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

    let user_opt = sqlx::query_as::<_, User>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "user {id} introuvable après INSERT"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(user)
}

/// Retrouve un utilisateur par son id.
pub async fn find_by_id(pool: &MySqlPool, id: i64) -> Result<Option<User>, DbError> {
    sqlx::query_as::<_, User>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Retrouve un utilisateur par son nom d'utilisateur.
///
/// Utilisé par la story 1.5 (auth) pour le login.
///
/// # Sémantique de comparaison
///
/// La collation `utf8mb4_unicode_ci` est **case-insensitive**, donc
/// `find_by_username("Alice")` matchera `"alice"`. Documenté ici pour éviter
/// toute surprise côté auth.
///
/// # Sécurité — Enumeration via timing attack
///
/// **Ce repository n'offre AUCUNE protection contre l'énumération d'utilisateurs
/// par timing attack.** Un appel retournant `None` est significativement plus
/// rapide qu'un appel retournant `Some(user)` suivi d'une vérification Argon2id.
/// Un attaquant peut ainsi distinguer les usernames existants des inexistants.
///
/// **Responsabilité story 1.5** : côté `kesh-api`, le handler de login DOIT
/// exécuter un Argon2id `verify` factice (dummy hash) quand `find_by_username`
/// retourne `None`, afin de normaliser les durées de réponse.
pub async fn find_by_username(
    pool: &MySqlPool,
    username: &str,
) -> Result<Option<User>, DbError> {
    sqlx::query_as::<_, User>(FIND_BY_USERNAME_SQL)
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)
}

/// Compte le nombre total d'utilisateurs (pour la pagination).
pub async fn count(pool: &MySqlPool) -> Result<i64, DbError> {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;
    Ok(row.0)
}

/// Compte le nombre d'utilisateurs actifs avec un rôle donné.
pub async fn count_active_by_role(
    pool: &MySqlPool,
    role: crate::entities::Role,
) -> Result<i64, DbError> {
    let row: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM users WHERE role = ? AND active = TRUE")
            .bind(role)
            .fetch_one(pool)
            .await
            .map_err(map_db_error)?;
    Ok(row.0)
}

/// Liste les utilisateurs avec pagination offset/limit.
///
/// `limit` est clampé dans `[0, MAX_LIST_LIMIT]`, `offset` à `>= 0`.
pub async fn list(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<User>, DbError> {
    let limit = limit.clamp(0, MAX_LIST_LIMIT);
    let offset = offset.max(0);
    sqlx::query_as::<_, User>(LIST_SQL)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

/// Met à jour le hash du mot de passe d'un utilisateur (story 1.6).
///
/// Incrémente aussi la `version` (optimistic lock). Retourne une erreur
/// `NotFound` si l'utilisateur n'existe pas.
pub async fn update_password(
    pool: &MySqlPool,
    user_id: i64,
    new_password_hash: &str,
) -> Result<(), DbError> {
    let rows_affected = sqlx::query(
        "UPDATE users SET password_hash = ?, version = version + 1 WHERE id = ?",
    )
    .bind(new_password_hash)
    .bind(user_id)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Met à jour le rôle et/ou l'activation d'un utilisateur avec verrouillage optimiste.
///
/// Le `password_hash` et le `username` ne sont PAS modifiables ici — story 1.7
/// introduira des flux dédiés pour ces cas (`change_password`, `rename_user`).
pub async fn update_role_and_active(
    pool: &MySqlPool,
    id: i64,
    version: i32,
    changes: UserUpdate,
) -> Result<User, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let rows_affected = sqlx::query(
        "UPDATE users
         SET role = ?, active = ?, version = version + 1
         WHERE id = ? AND version = ?",
    )
    .bind(changes.role)
    .bind(changes.active)
    .bind(id)
    .bind(version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows_affected == 0 {
        let exists = sqlx::query_as::<_, (i64,)>("SELECT id FROM users WHERE id = ?")
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

    let user_opt = sqlx::query_as::<_, User>(FIND_BY_ID_SQL)
        .bind(id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let user = match user_opt {
        Some(u) => u,
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::Invariant(format!(
                "user {id} introuvable après UPDATE réussi"
            )));
        }
    };

    tx.commit().await.map_err(map_db_error)?;
    Ok(user)
}
