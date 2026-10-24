//! Story 31-1c (#503) — dépôt du singleton système `_kesh_backup_schedule` :
//! réglage et état de la sauvegarde automatique faite par Kesh.
//!
//! Table **système** (comme `_kesh_version`) : hors de `TABLES_TO_TRUNCATE`,
//! donc jamais exportée dans un `.keshbackup`, jamais restaurée, jamais vidée
//! par la réinitialisation de démonstration. La ligne `id = 1` est posée par
//! la migration `20261024000001_backup_schedule.sql`.
//!
//! Deux familles d'écritures, et une seule règle de version :
//!
//! - [`update_settings`] — le réglage, depuis l'écran : **seul** écrivain qui
//!   incrémente `version` (verrou optimiste), audité `admin.backup_schedule_updated`
//!   (`before` / `after`) dans la même transaction ;
//! - [`record_attempt`], [`record_success`], [`record_failure`] — l'état, par
//!   la tâche de fond : ils ne touchent **jamais** `version` (sans quoi une
//!   sauvegarde terminée entre le chargement du panneau et *Enregistrer*
//!   produirait un 409 sans cause visible), et leurs horodatages sont liés par
//!   l'application (`Utc::now().naive_utc()` de l'appelant), jamais `NOW(3)` —
//!   le fuseau de session MariaDB fausserait la comparaison de `is_due`.

use chrono::NaiveDateTime;
use sqlx::MySqlPool;

use crate::entities::AUDIT_ENTITY_ID_NONE;
use crate::entities::audit_log::NewAuditLogEntry;
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

/// Fréquences permises (heures) — miroir de la contrainte
/// `chk_kesh_backup_schedule_interval`.
pub const ALLOWED_INTERVAL_HOURS: [i32; 4] = [6, 12, 24, 168];
/// Bornes du nombre de sauvegardes conservées — miroir de
/// `chk_kesh_backup_schedule_retention`.
pub const RETENTION_MIN: i32 = 1;
/// Voir [`RETENTION_MIN`].
pub const RETENTION_MAX: i32 = 60;

/// Statut de la dernière tentative (`last_status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackupStatus {
    /// Écrite, toutes les pièces incluses.
    Ok,
    /// Écrite, mais des pièces référencées n'ont pas pu y entrer.
    Partial,
    /// Échec d'écriture ou de relecture.
    Failed,
    /// Refus avant écriture : espace libre sous le seuil.
    InsufficientSpace,
    /// Disque ou quota saturé pendant l'écriture.
    DiskFull,
}

impl BackupStatus {
    /// Code stocké en base (et rendu par l'API).
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Partial => "partial",
            Self::Failed => "failed",
            Self::InsufficientSpace => "insufficient_space",
            Self::DiskFull => "disk_full",
        }
    }

    /// Lit un code stocké ; `None` pour une valeur inconnue (la contrainte
    /// `CHECK` l'interdit, mais la lecture ne panique pas).
    pub fn parse(code: &str) -> Option<Self> {
        Some(match code {
            "ok" => Self::Ok,
            "partial" => Self::Partial,
            "failed" => Self::Failed,
            "insufficient_space" => Self::InsufficientSpace,
            "disk_full" => Self::DiskFull,
            _ => return None,
        })
    }

    /// `true` pour les trois échecs (aucune sauvegarde écrite).
    pub fn is_failure(self) -> bool {
        matches!(
            self,
            Self::Failed | Self::InsufficientSpace | Self::DiskFull
        )
    }
}

/// La ligne `_kesh_backup_schedule`.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct BackupSchedule {
    pub enabled: bool,
    pub interval_hours: i32,
    pub retention_count: i32,
    pub last_attempt_at: Option<NaiveDateTime>,
    pub last_success_at: Option<NaiveDateTime>,
    pub last_success_file: Option<String>,
    pub last_success_bytes: Option<i64>,
    pub last_status: Option<String>,
    pub last_files_not_included: Option<i32>,
    pub last_complete_file: Option<String>,
    pub version: i32,
}

impl BackupSchedule {
    /// Le statut typé de la dernière tentative.
    pub fn status(&self) -> Option<BackupStatus> {
        self.last_status.as_deref().and_then(BackupStatus::parse)
    }
}

/// Le réglage modifiable depuis l'écran.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BackupScheduleSettings {
    pub enabled: bool,
    pub interval_hours: i32,
    pub retention_count: i32,
}

impl BackupScheduleSettings {
    /// `true` si les valeurs satisfont les contraintes de la table.
    pub fn is_valid(&self) -> bool {
        ALLOWED_INTERVAL_HOURS.contains(&self.interval_hours)
            && (RETENTION_MIN..=RETENTION_MAX).contains(&self.retention_count)
    }
}

/// Ce qu'une sauvegarde réussie enregistre.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupSuccess {
    pub at: NaiveDateTime,
    pub file: String,
    pub bytes: i64,
    pub files_not_included: i32,
}

const COLUMNS: &str = "enabled, interval_hours, retention_count, last_attempt_at, \
     last_success_at, last_success_file, last_success_bytes, last_status, \
     last_files_not_included, last_complete_file, version";

/// Exige qu'un `UPDATE` du singleton ait touché exactement une ligne : la
/// ligne `id = 1` est posée par la migration, son absence est une
/// installation incohérente, jamais un succès silencieux.
fn exactly_one(rows: u64, op: &str) -> Result<(), DbError> {
    if rows == 1 {
        Ok(())
    } else {
        Err(DbError::Invariant(format!(
            "_kesh_backup_schedule : {op} a touché {rows} ligne(s) au lieu d'une \
             (ligne id = 1 absente ?)"
        )))
    }
}

/// Lit le singleton.
pub async fn get(pool: &MySqlPool) -> Result<BackupSchedule, DbError> {
    sqlx::query_as::<_, BackupSchedule>(&format!(
        "SELECT {COLUMNS} FROM _kesh_backup_schedule WHERE id = 1"
    ))
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| DbError::Invariant("_kesh_backup_schedule : ligne id = 1 absente".into()))
}

fn settings_json(s: &BackupSchedule) -> serde_json::Value {
    serde_json::json!({
        "enabled": s.enabled,
        "interval_hours": s.interval_hours,
        "retention_count": s.retention_count,
    })
}

/// Met à jour le réglage (verrou optimiste sur `version`, qu'il incrémente),
/// et écrit l'audit `admin.backup_schedule_updated` (`before` / `after`) dans
/// la même transaction. Une version périmée rend
/// [`DbError::OptimisticLockConflict`] ; des valeurs hors domaine,
/// [`DbError::Invariant`] (le handler les refuse avant, en 400). Un réglage
/// inchangé n'écrit rien et ne fait pas avancer la version.
pub async fn update_settings(
    pool: &MySqlPool,
    expected_version: i32,
    user_id: i64,
    settings: BackupScheduleSettings,
) -> Result<BackupSchedule, DbError> {
    if !settings.is_valid() {
        return Err(DbError::Invariant(format!(
            "réglage de sauvegarde hors domaine : {settings:?}"
        )));
    }
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let before = sqlx::query_as::<_, BackupSchedule>(&format!(
        "SELECT {COLUMNS} FROM _kesh_backup_schedule WHERE id = 1 FOR UPDATE"
    ))
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| DbError::Invariant("_kesh_backup_schedule : ligne id = 1 absente".into()))?;
    if before.version != expected_version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }
    if before.enabled == settings.enabled
        && before.interval_hours == settings.interval_hours
        && before.retention_count == settings.retention_count
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(before);
    }
    let rows = sqlx::query(
        "UPDATE _kesh_backup_schedule SET enabled = ?, interval_hours = ?, \
         retention_count = ?, version = version + 1 WHERE id = 1 AND version = ?",
    )
    .bind(settings.enabled)
    .bind(settings.interval_hours)
    .bind(settings.retention_count)
    .bind(expected_version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }
    let after = sqlx::query_as::<_, BackupSchedule>(&format!(
        "SELECT {COLUMNS} FROM _kesh_backup_schedule WHERE id = 1"
    ))
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            "admin.backup_schedule_updated",
            "installation",
            AUDIT_ENTITY_ID_NONE,
            Some(serde_json::json!({
                "before": settings_json(&before),
                "after": settings_json(&after),
            })),
        ),
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(after)
}

/// Enregistre l'instant d'une tentative, sans condition et sans toucher
/// `version`. Appelée par la tâche **après** la prise du verrou en mémoire.
pub async fn record_attempt(pool: &MySqlPool, at: NaiveDateTime) -> Result<(), DbError> {
    let rows = sqlx::query("UPDATE _kesh_backup_schedule SET last_attempt_at = ? WHERE id = 1")
        .bind(at)
        .execute(pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
    exactly_one(rows, "record_attempt")
}

/// Enregistre une sauvegarde écrite : `ok` si aucune pièce ne manque (le
/// fichier devient alors `last_complete_file`), `partial` sinon
/// (`last_complete_file` inchangé). Ne touche pas `version`.
pub async fn record_success(pool: &MySqlPool, success: &BackupSuccess) -> Result<(), DbError> {
    let complete = success.files_not_included == 0;
    let status = if complete {
        BackupStatus::Ok
    } else {
        BackupStatus::Partial
    };
    let rows = sqlx::query(
        "UPDATE _kesh_backup_schedule SET last_success_at = ?, last_success_file = ?, \
         last_success_bytes = ?, last_files_not_included = ?, last_status = ?, \
         last_complete_file = IF(?, ?, last_complete_file) WHERE id = 1",
    )
    .bind(success.at)
    .bind(&success.file)
    .bind(success.bytes)
    .bind(success.files_not_included)
    .bind(status.as_str())
    .bind(complete)
    .bind(&success.file)
    .execute(pool)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    exactly_one(rows, "record_success")
}

/// Enregistre un échec (`failed`, `insufficient_space`, `disk_full`) : seul
/// `last_status` change — la dernière réussite reste ce qu'elle était. Ne
/// touche pas `version`.
pub async fn record_failure(pool: &MySqlPool, status: BackupStatus) -> Result<(), DbError> {
    if !status.is_failure() {
        return Err(DbError::Invariant(format!(
            "record_failure appelé avec un statut de réussite : {status:?}"
        )));
    }
    let rows = sqlx::query("UPDATE _kesh_backup_schedule SET last_status = ? WHERE id = 1")
        .bind(status.as_str())
        .execute(pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
    exactly_one(rows, "record_failure")
}

#[cfg(test)]
mod tests {
    //! Story 31-1c (AC23, AC24) — la ligne singleton, ses contraintes, et la
    //! règle de version.

    use super::*;
    use chrono::NaiveDate;

    fn instant(h: u32, m: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 24)
            .unwrap()
            .and_hms_milli_opt(h, m, 0, 123)
            .unwrap()
    }

    /// Un administrateur pour signer l'audit.
    async fn un_admin(pool: &MySqlPool) -> i64 {
        crate::test_fixtures::seed_accounting_company(pool)
            .await
            .expect("seed")
            .admin_user_id
    }

    /// AC24 — la ligne existe après migration, avec les défauts.
    #[sqlx::test(migrations = "./test-schema")]
    async fn singleton_present_avec_les_defauts(pool: MySqlPool) {
        let s = get(&pool).await.expect("ligne singleton");
        assert!(s.enabled);
        assert_eq!((s.interval_hours, s.retention_count, s.version), (24, 7, 1));
        assert_eq!(s.last_status, None);
        assert_eq!(s.last_attempt_at, None);
    }

    // Le vrai chemin des migrations pose la même ligne : c'est l'assertion de
    // montage de `tests/test_schema_guard.rs::squash_seeds_the_backup_schedule_row`
    // (seul fichier de la crate autorisé à monter le vrai `MIGRATOR` ici).

    /// AC24 — une seconde ligne, et chaque valeur hors domaine, sont refusées
    /// par la base elle-même.
    #[sqlx::test(migrations = "./test-schema")]
    async fn contraintes_de_la_table(pool: MySqlPool) {
        for (sql, quoi) in [
            (
                "INSERT INTO _kesh_backup_schedule (id) VALUES (2)",
                "seconde ligne",
            ),
            (
                "UPDATE _kesh_backup_schedule SET interval_hours = 5 WHERE id = 1",
                "interval_hours = 5",
            ),
            (
                "UPDATE _kesh_backup_schedule SET retention_count = 0 WHERE id = 1",
                "retention_count = 0",
            ),
            (
                "UPDATE _kesh_backup_schedule SET retention_count = 61 WHERE id = 1",
                "retention_count = 61",
            ),
            (
                "UPDATE _kesh_backup_schedule SET last_status = 'x' WHERE id = 1",
                "last_status = 'x'",
            ),
        ] {
            assert!(
                sqlx::query(sql).execute(&pool).await.is_err(),
                "{quoi} doit être refusé par la base"
            );
        }
        // Témoin : une valeur du domaine passe.
        sqlx::query("UPDATE _kesh_backup_schedule SET interval_hours = 168, last_status = 'disk_full' WHERE id = 1")
            .execute(&pool)
            .await
            .expect("valeurs du domaine acceptées");
    }

    /// AC32 — seul `update_settings` incrémente `version` ; `record_*` non
    /// (tue M-c15 au niveau du dépôt).
    #[sqlx::test(migrations = "./test-schema")]
    async fn seul_le_reglage_fait_avancer_la_version(pool: MySqlPool) {
        record_attempt(&pool, instant(1, 0)).await.unwrap();
        record_success(
            &pool,
            &BackupSuccess {
                at: instant(1, 1),
                file: "kesh-auto-20261024T010100123Z.keshbackup".into(),
                bytes: 4096,
                files_not_included: 0,
            },
        )
        .await
        .unwrap();
        record_failure(&pool, BackupStatus::DiskFull).await.unwrap();
        let s = get(&pool).await.unwrap();
        assert_eq!(s.version, 1, "l'état ne touche pas la version");
        assert_eq!(s.status(), Some(BackupStatus::DiskFull));
        assert_eq!(
            s.last_success_at,
            Some(instant(1, 1)),
            "milliseconde gardée"
        );

        let admin = un_admin(&pool).await;
        let apres = update_settings(
            &pool,
            1,
            admin,
            BackupScheduleSettings {
                enabled: false,
                interval_hours: 6,
                retention_count: 3,
            },
        )
        .await
        .unwrap();
        assert_eq!(apres.version, 2);
        assert!(!apres.enabled);
        assert!(matches!(
            update_settings(
                &pool,
                1,
                admin,
                BackupScheduleSettings {
                    enabled: true,
                    interval_hours: 6,
                    retention_count: 3,
                },
            )
            .await,
            Err(DbError::OptimisticLockConflict)
        ));
    }

    /// AC28/AC29 — `partial` garde la dernière sauvegarde complète ; `ok` la
    /// remplace.
    #[sqlx::test(migrations = "./test-schema")]
    async fn partial_garde_la_derniere_complete(pool: MySqlPool) {
        let succes = |m: u32, manquantes: i32| BackupSuccess {
            at: instant(2, m),
            file: format!("kesh-auto-202610240{m}00123Z.keshbackup"),
            bytes: 10,
            files_not_included: manquantes,
        };
        record_success(&pool, &succes(1, 0)).await.unwrap();
        record_success(&pool, &succes(2, 3)).await.unwrap();
        let s = get(&pool).await.unwrap();
        assert_eq!(s.status(), Some(BackupStatus::Partial));
        assert_eq!(s.last_files_not_included, Some(3));
        assert_eq!(
            s.last_complete_file.as_deref(),
            Some(succes(1, 0).file.as_str())
        );
        assert_eq!(
            s.last_success_file.as_deref(),
            Some(succes(2, 3).file.as_str())
        );
        record_success(&pool, &succes(3, 0)).await.unwrap();
        let s = get(&pool).await.unwrap();
        assert_eq!(s.status(), Some(BackupStatus::Ok));
        assert_eq!(
            s.last_complete_file.as_deref(),
            Some(succes(3, 0).file.as_str())
        );
    }

    /// `record_failure` refuse un statut de réussite ; le réglage refuse une
    /// valeur hors domaine sans écrire.
    #[sqlx::test(migrations = "./test-schema")]
    async fn refus_des_valeurs_hors_domaine(pool: MySqlPool) {
        assert!(record_failure(&pool, BackupStatus::Ok).await.is_err());
        let admin = un_admin(&pool).await;
        assert!(
            update_settings(
                &pool,
                1,
                admin,
                BackupScheduleSettings {
                    enabled: true,
                    interval_hours: 5,
                    retention_count: 7,
                },
            )
            .await
            .is_err()
        );
        assert_eq!(get(&pool).await.unwrap().version, 1);
    }

    /// AC24 — l'export d'installation ne lit pas la table (elle n'est pas dans
    /// `TABLES_TO_TRUNCATE`, et donc ni exportée, ni vidée par un import, ni
    /// par la réinitialisation de démonstration, qui dérivent de cette liste).
    #[test]
    fn table_systeme_hors_inventaire() {
        assert!(
            !crate::backup::TABLES_TO_TRUNCATE.contains(&"_kesh_backup_schedule"),
            "_kesh_backup_schedule est une table système : l'ajouter à l'inventaire \
             rendrait toute sauvegarde antérieure non importable"
        );
    }

    /// AC24 — `export_installation` ne rend pas la table.
    #[sqlx::test(migrations = "./test-schema")]
    async fn export_sans_la_table(pool: MySqlPool) {
        let export = crate::backup::export_installation(&pool)
            .await
            .expect("export");
        assert!(
            !export
                .tables
                .iter()
                .any(|(t, _)| *t == "_kesh_backup_schedule"),
            "l'export ne doit pas contenir la table système"
        );
    }
}
