//! Version tracking + downgrade protection pour le schéma DB Kesh.
//!
//! Story 10-2 — cf.
//! `_bmad-output/implementation-artifacts/10-2-migrations-idempotence-downgrade-protection.md`
//!
//! Ce module expose deux fonctions appelées au boot par
//! `crates/kesh-api/src/main.rs` :
//!
//! - [`check_downgrade_protection`] : appelée **avant** `MIGRATOR.run()`.
//!   Vérifie que le binaire courant (`CARGO_PKG_VERSION`) est suffisamment
//!   récent pour la DB déjà migrée. Retourne `Err(DowngradeRefused)` si
//!   le binaire est plus ancien que `kesh_version_min_required` — le
//!   caller doit exit non-zero avec un message explicite (pas de boot
//!   silencieux sur une DB qu'on ne sait pas lire).
//!
//! - [`record_boot_version`] : appelée **après** `MIGRATOR.run()`. Met
//!   à jour `kesh_version_last_applied` + `last_boot_at` pour audit.
//!
//! Pattern erreur ER_NO_SUCH_TABLE 1146 : `code()` retourne le SQLSTATE
//! `"42S02"` — utiliser `try_downcast_ref::<MySqlDatabaseError>()` puis
//! `.number()` pour obtenir le numéro MariaDB. Cf. errors.rs:150 et
//! retry.rs:73 pour le pattern canonique projet.

use semver::Version;
use sqlx::MySqlPool;

/// Erreurs du module `version`.
#[derive(Debug, thiserror::Error)]
pub enum VersionError {
    /// Le binaire est plus ancien que `kesh_version_min_required` de la DB.
    /// Le boot DOIT être refusé pour éviter une corruption silencieuse.
    #[error(
        "Database was migrated by Kesh v{db_min}, current binary v{binary} cannot downgrade safely. Restore a backup compatible with v{binary} or upgrade the binary."
    )]
    DowngradeRefused { db_min: Version, binary: Version },

    /// Erreur sqlx (DB inaccessible, query échouée, etc.).
    #[error("Database error during version check: {0}")]
    Sqlx(#[from] sqlx::Error),

    /// La string semver fournie (CARGO_PKG_VERSION ou colonne DB) n'est
    /// pas un SemVer valide.
    #[error("Invalid semver string: {0}")]
    InvalidSemver(#[from] semver::Error),
}

/// Résultat non-erreur de [`check_downgrade_protection`].
///
/// Le 4e cas « binary < db_min » n'est pas un variant ici — il est
/// converti en [`VersionError::DowngradeRefused`] car c'est le seul cas
/// qui doit faire échouer le boot.
#[derive(Debug, PartialEq, Eq)]
pub enum DowngradeCheckOutcome {
    /// La table `_kesh_version` n'existe pas encore — DB vierge, la
    /// migration `_kesh_version.sql` la créera ensuite via `MIGRATOR.run()`.
    FreshInstall,
    /// `binary_version == kesh_version_min_required`.
    Aligned,
    /// `binary_version > kesh_version_min_required` — upgrade légitime.
    BinaryAhead { db_min: Version, binary: Version },
}

/// Vérifie que le binaire courant peut lire la DB sans risque de downgrade.
///
/// Appelée AVANT `MIGRATOR.run()` au boot — si la table n'existe pas
/// encore, retourne `Ok(FreshInstall)` et le caller continue (la migration
/// `_kesh_version.sql` créera la table juste après).
///
/// Arguments :
/// - `pool` : pool MariaDB déjà ouvert.
/// - `binary_version` : typiquement `env!("CARGO_PKG_VERSION")`.
pub async fn check_downgrade_protection(
    pool: &MySqlPool,
    binary_version: &str,
) -> Result<DowngradeCheckOutcome, VersionError> {
    let binary = Version::parse(binary_version)?;

    let row: Result<String, sqlx::Error> =
        sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
            .fetch_one(pool)
            .await;

    match row {
        Err(sqlx::Error::Database(ref db_err))
            if db_err
                .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
                .is_some_and(|e| e.number() == 1146) =>
        {
            // ER_NO_SUCH_TABLE 1146 — fresh install, `_kesh_version` n'a
            // pas encore été créée par MIGRATOR.run().
            Ok(DowngradeCheckOutcome::FreshInstall)
        }
        Err(e) => Err(VersionError::Sqlx(e)),
        Ok(db_min_str) => {
            let db_min = Version::parse(&db_min_str)?;
            match binary.cmp(&db_min) {
                std::cmp::Ordering::Less => Err(VersionError::DowngradeRefused { db_min, binary }),
                std::cmp::Ordering::Equal => Ok(DowngradeCheckOutcome::Aligned),
                std::cmp::Ordering::Greater => {
                    Ok(DowngradeCheckOutcome::BinaryAhead { db_min, binary })
                }
            }
        }
    }
}

/// Enregistre la version binaire courante comme dernière à avoir tourné
/// contre cette DB. Met à jour `kesh_version_last_applied` + `last_boot_at`.
///
/// Appelée APRÈS `MIGRATOR.run()` au boot — la table `_kesh_version`
/// existe nécessairement (créée par la migration `_kesh_version.sql`
/// au plus tard à l'instant).
///
/// Cette fonction est **non-fatale** : si l'UPDATE échoue (pool fermé,
/// row absente, etc.), le caller log un warning mais le serveur reste
/// utilisable (le tracking de version est de l'audit, pas un invariant
/// de fonctionnement).
pub async fn record_boot_version(
    pool: &MySqlPool,
    binary_version: &str,
) -> Result<(), VersionError> {
    // Valide la string SemVer côté binaire (mais on stocke la string
    // brute, pas la struct Version, dans le VARCHAR).
    Version::parse(binary_version)?;

    let result = sqlx::query(
        "UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1",
    )
    .bind(binary_version)
    .execute(pool)
    .await?;

    if result.rows_affected() != 1 {
        tracing::warn!(
            rows_affected = result.rows_affected(),
            "record_boot_version: UPDATE affected unexpected number of rows (expected 1) — _kesh_version row id=1 missing?"
        );
    }

    Ok(())
}
