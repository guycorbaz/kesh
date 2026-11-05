//! Repository pour `RefreshToken`.
//!
//! Pattern : requêtes non-macro (`sqlx::query_as::<_, T>`) pour éviter
//! toute dépendance compile-time à une base SQLx. Cohérent avec les
//! autres repositories de `kesh-db`.
//!
//! **Stockage** (Story v0174-session-1, #890) : seule l'**empreinte** du jeton
//! ([`token_digest`]) est en base. Chaque fonction qui reçoit la valeur d'un
//! jeton la hache elle-même avant de la lier (C-v0174s-3) : un appelant passe
//! le jeton clair et ne peut pas oublier de hacher. La colonne s'appelle
//! encore `token` et son commentaire SQL dit encore `'UUID v4 opaque'` ; ils
//! ne se corrigent que par une migration (prochaine migration de la table,
//! issue demandée). Les lignes d'avant la 0.17.4 (jetons clairs) se
//! reconnaissent à [`LEGACY_CLEAR_TOKEN_PREDICATE`] et sont effacées au
//! démarrage et à la restauration complète (C-v0174s-6).

use sha2::{Digest, Sha256};
use sqlx::mysql::MySqlPool;

use crate::entities::{NewRefreshToken, RefreshToken};
use crate::errors::{DbError, map_db_error};

/// Empreinte d'un jeton de renouvellement, seule valeur stockée (#890).
///
/// Les 16 premiers octets de SHA-256(jeton), en hexadécimal minuscule, dont le
/// 13ᵉ chiffre — celui de la version d'un UUID — est forcé à `0`, mis en forme
/// `8-4-4-4-12` : la valeur satisfait `chk_refresh_tokens_token_format` (aucune
/// migration), et ne ressemble jamais à un UUID v4 (`4` à cette place) — ce qui
/// distingue une ligne d'avant la 0.17.4 ([`LEGACY_CLEAR_TOKEN_PREDICATE`]).
/// 124 bits d'empreinte pour 122 bits d'aléa : rien à gagner d'un hachage lent
/// (C-v0174s-5).
pub fn token_digest(clear: &str) -> String {
    let bytes = Sha256::digest(clear.as_bytes());
    let mut hex = String::with_capacity(32);
    for b in &bytes[..16] {
        hex.push_str(&format!("{b:02x}"));
    }
    // 13ᵉ chiffre (index 12) : la version d'un UUID, forcée à `0`.
    hex.replace_range(12..13, "0");
    format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    )
}

/// Une ligne qui ne porte pas une empreinte : son 15ᵉ caractère (le 13ᵉ
/// chiffre, tirets compris) n'est pas `0` — un jeton clair d'avant la 0.17.4
/// (UUID v4, `4` à cette place). Employé par les deux effacements, jamais
/// réécrit ailleurs.
pub const LEGACY_CLEAR_TOKEN_PREDICATE: &str = "SUBSTRING(token, 15, 1) <> '0'";

const FIND_BY_ID_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens WHERE id = ?";

const FIND_ACTIVE_BY_TOKEN_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens \
     WHERE token = ? AND revoked_at IS NULL AND expires_at > NOW(3)";

const FIND_BY_TOKEN_INCLUDE_REVOKED_SQL: &str = "SELECT id, user_id, token, expires_at, created_at, revoked_at, revoked_reason \
     FROM refresh_tokens WHERE token = ?";

/// Crée un refresh token et retourne l'entité persistée.
///
/// `new.token` est le jeton **clair** ; seule son empreinte est stockée.
///
/// Pattern transaction INSERT+SELECT pour compatibilité MySQL
/// (pas de `RETURNING`).
pub async fn create(pool: &MySqlPool, new: NewRefreshToken) -> Result<RefreshToken, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result =
        sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
            .bind(new.user_id)
            .bind(token_digest(&new.token))
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
        .bind(token_digest(token))
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
        .bind(token_digest(token))
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
    .bind(token_digest(token))
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

/// Story v0172-session (#876) — écrit l'échéance d'inactivité d'une session
/// (battement d'activité, `POST /api/v1/auth/activity`).
///
/// Une requête, une écriture : la ligne doit porter ce `token`, appartenir à
/// `user_id`, n'être ni révoquée ni échue à `now` (horloge de l'application,
/// jamais `NOW(3)`). Rend `false` si aucune ligne ne convient (cookie absent
/// ou d'un autre utilisateur, ligne révoquée ou expirée) — l'appelant rend 401.
///
/// **Pas de garde de monotonie, délibérément** (#898, S-B3/S-E4 (a)) : un
/// abaissement du réglage raccourcit l'échéance à la prochaine activité
/// (C-v0172s-38), ce qu'un `GREATEST` interdirait ; deux battements
/// réordonnés n'en déplacent l'échéance que de l'écart (a) du manuel
/// d'administration.
pub async fn set_idle_deadline(
    pool: &MySqlPool,
    token: &str,
    user_id: i64,
    deadline: chrono::NaiveDateTime,
    now: chrono::NaiveDateTime,
) -> Result<bool, DbError> {
    let rows_affected = sqlx::query(
        "UPDATE refresh_tokens SET expires_at = ? \
         WHERE token = ? AND user_id = ? AND revoked_at IS NULL AND expires_at > ?",
    )
    .bind(deadline)
    .bind(token_digest(token))
    .bind(user_id)
    .bind(now)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows_affected > 0)
}

/// Story v0172-session (#876, C-v0172s-17) — ce jeton a-t-il été révoqué par
/// **rotation** depuis moins de `secs` secondes ?
///
/// Mesuré à l'horloge de la base, la même qui a écrit `revoked_at = NOW(3)`
/// (`revoke_by_token`) : la grâce de rotation laisse deux onglets présenter
/// le même jeton à quelques secondes d'intervalle sans déclencher la
/// détection de vol.
pub async fn was_rotated_within(pool: &MySqlPool, token: &str, secs: i64) -> Result<bool, DbError> {
    let n: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens \
         WHERE token = ? AND revoked_reason = 'rotation' \
           AND revoked_at > NOW(3) - INTERVAL ? SECOND",
    )
    .bind(token_digest(token))
    .bind(secs)
    .fetch_one(pool)
    .await
    .map_err(map_db_error)?;
    Ok(n > 0)
}

/// Story v0172-session (#876, C-v0172s-23) — révoque **toutes** les lignes
/// actives, dans la transaction de la restauration complète.
///
/// Motif imposé par la contrainte `chk_refresh_tokens_revoked_reason` (cinq
/// valeurs) : l'appelant passe `"password_change"`, la distinction étant
/// portée par l'audit `admin.full_import`.
pub async fn revoke_all_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    reason: &str,
) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW(3), revoked_reason = ? \
         WHERE revoked_at IS NULL",
    )
    .bind(reason)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows_affected)
}

/// Story v0174-session-1 (#890, C-v0174s-6) — efface, dans la transaction de
/// la restauration complète, toute ligne qui ne porte pas une empreinte
/// ([`LEGACY_CLEAR_TOKEN_PREDICATE`]) : une sauvegarde d'avant la 0.17.4 n'y
/// replante aucun jeton clair. Un refus ultérieur de l'import annule aussi
/// l'effacement. Rend le nombre de lignes effacées.
pub async fn delete_clear_tokens_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(&format!(
        "DELETE FROM refresh_tokens WHERE {LEGACY_CLEAR_TOKEN_PREDICATE}"
    ))
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    Ok(rows_affected)
}

/// Supprime physiquement les tokens expirés ou révoqués depuis plus de
/// `older_than`, et — depuis la 0.17.4 (#890, C-v0174s-6) — toute ligne d'avant
/// la 0.17.4 qui porte un jeton clair ([`LEGACY_CLEAR_TOKEN_PREDICATE`]).
/// Utilisé au démarrage pour le nettoyage (AC8).
pub async fn delete_expired_and_revoked(
    pool: &MySqlPool,
    older_than: chrono::NaiveDateTime,
) -> Result<u64, DbError> {
    let rows_affected = sqlx::query(&format!(
        "DELETE FROM refresh_tokens \
         WHERE (revoked_at IS NOT NULL AND revoked_at < ?) \
            OR (expires_at < ?) \
            OR ({LEGACY_CLEAR_TOKEN_PREDICATE})"
    ))
    .bind(older_than)
    .bind(older_than)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    Ok(rows_affected)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-1 (#890) — le vecteur recalculé hors du code :
    /// `printf '%s' '00000000-0000-4000-8000-000000000000' | sha256sum`
    /// ⇒ `db8055e0e0307d5a016bec4dc338d698…` ; le 13ᵉ chiffre (`7`) est forcé
    /// à `0`.
    #[test]
    fn token_digest_matches_the_vector() {
        assert_eq!(
            token_digest("00000000-0000-4000-8000-000000000000"),
            "db8055e0-e030-0d5a-016b-ec4dc338d698"
        );
    }

    /// AC-1 — sur 1 000 UUID v4 : l'empreinte satisfait la contrainte de la
    /// colonne (`chk_refresh_tokens_token_format`), porte `0` à la position
    /// 15 (le prédicat des lignes d'avant ne la vise jamais) et diffère de
    /// l'entrée.
    #[test]
    fn token_digest_fits_the_column_and_is_never_a_v4_uuid() {
        let re =
            regex::Regex::new("^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
                .unwrap();
        for _ in 0..1_000 {
            let clear = uuid::Uuid::new_v4().to_string();
            let d = token_digest(&clear);
            assert!(re.is_match(&d), "forme de la colonne : {d}");
            assert_eq!(&d[14..15], "0", "position 15 : {d}");
            assert_ne!(d, clear);
            // Le jeton clair, lui, est toujours visé par le prédicat.
            assert_ne!(&clear[14..15], "0");
        }
    }
}
