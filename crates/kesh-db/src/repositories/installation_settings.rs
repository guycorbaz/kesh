//! Story v0172-session (0.17.3, #876, CR #891) — dépôt du singleton système
//! `_kesh_installation_settings` : les réglages de l'installation faits dans
//! l'interface, aujourd'hui la seule durée d'inactivité de la session.
//!
//! Table **système** (comme `_kesh_version` et `_kesh_backup_schedule`) : hors
//! de `TABLES_TO_TRUNCATE`, donc jamais exportée dans un `.keshbackup`, jamais
//! restaurée, jamais vidée par la réinitialisation de démonstration. **Aucune
//! ligne n'est posée par la migration** : tant que rien n'est enregistré,
//! [`get`] rend les défauts du code (`SESSION_IDLE_DEFAULT`, version 0) —
//! précédent `mcp_server_settings`.
//!
//! Verrou optimiste où `version == 0` désigne « aucune ligne » : `INSERT`
//! (une ligne apparue entre-temps ⇒ conflit) ; `version ≥ 1` : `UPDATE …
//! WHERE version = ?`. Le seul écrivain, [`update_session_idle_minutes`],
//! écrit l'audit `admin.session_settings_updated` (`before` / `after`) dans la
//! même transaction ; une valeur inchangée n'écrit rien (ni ligne, ni audit,
//! ni version), comme `backup_schedule::update_settings`.

use sqlx::MySqlPool;

use crate::entities::AUDIT_ENTITY_ID_NONE;
use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

/// Durée d'inactivité minimale (minutes) — miroir de la contrainte
/// `chk_kesh_installation_settings_idle`.
pub const SESSION_IDLE_MIN: i32 = 1;
/// Durée d'inactivité maximale (minutes) : 24 heures.
pub const SESSION_IDLE_MAX: i32 = 1440;
/// Durée d'inactivité par défaut (minutes), tant que rien n'est enregistré.
pub const SESSION_IDLE_DEFAULT: i32 = 60;

/// Les réglages de l'installation ; `version == 0` : rien n'est enregistré
/// (défauts du code).
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow)]
pub struct InstallationSettings {
    /// Durée d'inactivité de la session, en minutes, dans
    /// `[SESSION_IDLE_MIN, SESSION_IDLE_MAX]`.
    pub session_idle_minutes: i32,
    /// Verrou optimiste.
    pub version: i32,
}

impl InstallationSettings {
    /// Les défauts du code (aucune ligne enregistrée).
    pub const DEFAULTS: Self = Self {
        session_idle_minutes: SESSION_IDLE_DEFAULT,
        version: 0,
    };
}

/// La durée est-elle dans les bornes ?
pub fn is_valid_session_idle_minutes(minutes: i32) -> bool {
    (SESSION_IDLE_MIN..=SESSION_IDLE_MAX).contains(&minutes)
}

const SELECT_SQL: &str =
    "SELECT session_idle_minutes, version FROM _kesh_installation_settings WHERE id = 1";

/// Les réglages, ou les défauts du code s'il n'y a pas de ligne. N'écrit rien.
pub async fn get(pool: &MySqlPool) -> Result<InstallationSettings, DbError> {
    Ok(sqlx::query_as::<_, InstallationSettings>(SELECT_SQL)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?
        .unwrap_or(InstallationSettings::DEFAULTS))
}

/// Enregistre la durée d'inactivité sous verrou optimiste et écrit l'audit
/// `admin.session_settings_updated` dans la même transaction.
///
/// - valeur hors bornes ⇒ [`DbError::Invariant`] (le handler la refuse avant,
///   en 400) ;
/// - version périmée, ou ligne apparue alors que `expected_version == 0` ⇒
///   [`DbError::OptimisticLockConflict`] ;
/// - valeur inchangée ⇒ rien d'écrit, l'état courant est rendu.
pub async fn update_session_idle_minutes(
    pool: &MySqlPool,
    expected_version: i32,
    user_id: i64,
    minutes: i32,
) -> Result<InstallationSettings, DbError> {
    if !is_valid_session_idle_minutes(minutes) {
        return Err(DbError::Invariant(format!(
            "durée d'inactivité hors bornes : {minutes}"
        )));
    }
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let before = sqlx::query_as::<_, InstallationSettings>(&format!("{SELECT_SQL} FOR UPDATE"))
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .unwrap_or(InstallationSettings::DEFAULTS);
    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }
    if before.session_idle_minutes == minutes {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }
    if expected_version == 0 {
        let inserted = sqlx::query(
            "INSERT INTO _kesh_installation_settings (id, session_idle_minutes, version) \
             VALUES (1, ?, 1)",
        )
        .bind(minutes)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error);
        match inserted {
            Ok(_) => {}
            Err(DbError::UniqueConstraintViolation(_)) => {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::OptimisticLockConflict);
            }
            Err(e) => return Err(e),
        }
    } else {
        let rows = sqlx::query(
            "UPDATE _kesh_installation_settings SET session_idle_minutes = ?, \
             version = version + 1 WHERE id = 1 AND version = ?",
        )
        .bind(minutes)
        .bind(expected_version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
    }
    let after = sqlx::query_as::<_, InstallationSettings>(SELECT_SQL)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "admin.session_settings_updated",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "before": { "sessionIdleMinutes": before.session_idle_minutes },
                "after": { "sessionIdleMinutes": after.session_idle_minutes },
            })),
        ),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{NewUser, Role};
    use crate::repositories::users;

    async fn admin(pool: &MySqlPool) -> i64 {
        crate::test_fixtures::seed_stub_company_only(pool)
            .await
            .unwrap();
        let company_id: i64 = sqlx::query_scalar("SELECT MIN(id) FROM companies")
            .fetch_one(pool)
            .await
            .unwrap();
        users::create(
            pool,
            NewUser {
                username: "admin-reglages".into(),
                password_hash: "$argon2id$v=19$m=19456,t=2,p=1$mock$mock_hash_for_tests".into(),
                role: Role::Admin,
                active: true,
                company_id,
                email: None,
            },
        )
        .await
        .unwrap()
        .id
    }

    async fn audits(pool: &MySqlPool) -> Vec<serde_json::Value> {
        let rows: Vec<Option<Vec<u8>>> = sqlx::query_scalar(
            "SELECT details_json FROM audit_log \
             WHERE action = 'admin.session_settings_updated' ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap();
        rows.into_iter()
            .map(|b| serde_json::from_slice(&b.unwrap_or_default()).unwrap())
            .collect()
    }

    async fn rows(pool: &MySqlPool) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM _kesh_installation_settings")
            .fetch_one(pool)
            .await
            .unwrap()
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn empty_table_reads_the_defaults(pool: MySqlPool) {
        assert_eq!(
            get(&pool).await.unwrap(),
            InstallationSettings {
                session_idle_minutes: 60,
                version: 0
            }
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn update_inserts_then_updates_with_audit_and_lock(pool: MySqlPool) {
        let uid = admin(&pool).await;
        let s = update_session_idle_minutes(&pool, 0, uid, 90)
            .await
            .unwrap();
        assert_eq!(
            s,
            InstallationSettings {
                session_idle_minutes: 90,
                version: 1
            }
        );
        let a = audits(&pool).await;
        assert_eq!(a.len(), 1);
        assert_eq!(a[0]["before"]["sessionIdleMinutes"], 60);
        assert_eq!(a[0]["after"]["sessionIdleMinutes"], 90);

        assert!(matches!(
            update_session_idle_minutes(&pool, 0, uid, 30).await,
            Err(DbError::OptimisticLockConflict)
        ));
        let s = update_session_idle_minutes(&pool, 1, uid, 30)
            .await
            .unwrap();
        assert_eq!(s.version, 2);
        assert_eq!(get(&pool).await.unwrap().session_idle_minutes, 30);
        assert_eq!(audits(&pool).await.len(), 2);
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn unchanged_value_writes_nothing(pool: MySqlPool) {
        let uid = admin(&pool).await;
        let s = update_session_idle_minutes(&pool, 0, uid, 60)
            .await
            .unwrap();
        assert_eq!(s, InstallationSettings::DEFAULTS);
        assert_eq!(rows(&pool).await, 0, "aucune ligne créée");
        assert!(audits(&pool).await.is_empty(), "aucun audit");

        update_session_idle_minutes(&pool, 0, uid, 90)
            .await
            .unwrap();
        let s = update_session_idle_minutes(&pool, 1, uid, 90)
            .await
            .unwrap();
        assert_eq!(s.version, 1, "version inchangée");
        assert_eq!(audits(&pool).await.len(), 1, "aucun audit de plus");
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn out_of_range_values_are_refused(pool: MySqlPool) {
        let uid = admin(&pool).await;
        for m in [0, 1441, -5] {
            assert!(matches!(
                update_session_idle_minutes(&pool, 0, uid, m).await,
                Err(DbError::Invariant(_))
            ));
        }
        assert_eq!(rows(&pool).await, 0);
    }

    #[sqlx::test(migrations = "./test-schema")]
    async fn constraints_refuse_bad_rows(pool: MySqlPool) {
        sqlx::query("INSERT INTO _kesh_installation_settings (id) VALUES (1)")
            .execute(&pool)
            .await
            .unwrap();
        for sql in [
            "INSERT INTO _kesh_installation_settings (id) VALUES (2)",
            "UPDATE _kesh_installation_settings SET session_idle_minutes = 0 WHERE id = 1",
            "UPDATE _kesh_installation_settings SET session_idle_minutes = 1441 WHERE id = 1",
        ] {
            assert!(
                sqlx::query(sql).execute(&pool).await.is_err(),
                "la contrainte doit refuser : {sql}"
            );
        }
    }

    #[test]
    fn system_table_is_not_in_the_backup_inventory() {
        assert!(
            !crate::backup::TABLES_TO_TRUNCATE.contains(&"_kesh_installation_settings"),
            "_kesh_installation_settings est une table système : l'ajouter à l'inventaire \
             rendrait toute sauvegarde antérieure inimportable"
        );
    }
}
