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
        "La base a été migrée par Kesh v{db_min}, le binaire courant v{binary} ne peut pas la downgrader sans risque. Restaurer un backup antérieur à Kesh v{db_min}, ou mettre à jour le binaire vers au moins v{db_min}."
    )]
    DowngradeRefused { db_min: Version, binary: Version },

    /// La table `_kesh_version` existe mais la row singleton `id=1` est
    /// absente — état pathologique (restore partiel, TRUNCATE accidentel,
    /// crash entre `CREATE TABLE` et `INSERT` de la migration initiale —
    /// rappel : MariaDB ne supporte pas la DDL transactionnelle).
    /// Le boot DOIT être refusé pour éviter un boot silencieux sans tracking
    /// de version. Diagnostic explicite pour l'opérateur. Voir
    /// `docs/operations.md` (section « Récupération `_kesh_version` ») pour
    /// la procédure complète selon l'origine présumée.
    #[error(
        "La table _kesh_version existe mais la row id=1 est absente. Diagnostic : (a) vérifier `SELECT success FROM _sqlx_migrations WHERE version = 20260522000001` — si `success = 0`, la migration initiale a crashé entre CREATE TABLE et INSERT : DELETE cette ligne dirty puis re-démarrer le binaire. (b) Sinon (TRUNCATE/DELETE manuel post-migration) : INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied) VALUES (1, '0.1.0', '0.1.0'); — remplacer '0.1.0' par la version cible si différente."
    )]
    RowMissing,

    /// Erreur sqlx (DB inaccessible, query échouée, etc.).
    #[error("Erreur base de données pendant la vérification de version : {0}")]
    Sqlx(#[from] sqlx::Error),

    /// La string SemVer fournie (CARGO_PKG_VERSION ou colonne DB) n'est
    /// pas un SemVer valide. Le champ `origin` distingue la provenance
    /// (`"binaire"` pour `binary_version`, `"base de données"` pour la
    /// colonne `_kesh_version`) afin que l'opérateur sache où chercher.
    /// `value` inclut la string fautive (tronquée à 64 chars pour éviter
    /// de polluer les logs si la colonne contient un payload aberrant).
    #[error("SemVer invalide ({origin}) — valeur reçue : {value:?} — détail parser : {error}")]
    InvalidSemver {
        origin: &'static str,
        value: String,
        error: semver::Error,
    },
}

impl VersionError {
    /// Helper interne : wrap `semver::Error` en `VersionError::InvalidSemver`
    /// avec origin + valeur (tronquée 64 chars logiques). Utilisé par les
    /// deux `Version::parse()` du module (binary_version et db_min_str).
    ///
    /// Troncature par `chars().take(MAX_LEN)` (jamais par bytes) — un slice
    /// `&value[..N]` sur une string UTF-8 panic si N tombe au milieu d'un
    /// caractère multi-byte (accent `é` = 2 bytes, idéogramme CJK = 3 bytes,
    /// emoji = 4 bytes). Le payload DB peut contenir ce genre de string si
    /// la colonne `_kesh_version` a été UPDATE manuellement avec une valeur
    /// non-ASCII corrompue — il serait ironique que le helper conçu pour
    /// éviter de polluer les logs panique avant que le log soit écrit.
    fn invalid_semver(origin: &'static str, value: &str, error: semver::Error) -> Self {
        const MAX_LEN: usize = 64;
        let truncated = if value.chars().count() > MAX_LEN {
            let mut s: String = value.chars().take(MAX_LEN).collect();
            s.push('…');
            s
        } else {
            value.to_string()
        };
        Self::InvalidSemver {
            origin,
            value: truncated,
            error,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_semver_passes_short_ascii_through() {
        let err = VersionError::invalid_semver(
            "test",
            "not-a-semver",
            semver::Version::parse("nope").unwrap_err(),
        );
        match err {
            VersionError::InvalidSemver { origin, value, .. } => {
                assert_eq!(origin, "test");
                assert_eq!(value, "not-a-semver");
            }
            _ => panic!("expected InvalidSemver variant"),
        }
    }

    #[test]
    fn invalid_semver_truncates_long_ascii_with_ellipsis() {
        let long = "x".repeat(100);
        let err = VersionError::invalid_semver(
            "test",
            &long,
            semver::Version::parse("nope").unwrap_err(),
        );
        match err {
            VersionError::InvalidSemver { value, .. } => {
                assert!(value.ends_with('…'), "truncated value should end with …");
                assert_eq!(value.chars().count(), 65, "64 chars + ellipsis");
            }
            _ => panic!("expected InvalidSemver variant"),
        }
    }

    // --- Story 17-3c : check_import_version_compat (pure, pas de DB) ---

    #[test]
    fn import_compat_ok_when_min_required_equals_or_below_binary() {
        // min_required == binary → OK.
        assert!(check_import_version_compat("0.1.8", "0.1.8").is_ok());
        // min_required < binary (binaire plus récent) → OK.
        assert!(check_import_version_compat("0.1.0", "0.2.0").is_ok());
        assert!(check_import_version_compat("0.1.8", "0.2.0").is_ok());
    }

    #[test]
    fn import_compat_refused_when_min_required_above_binary() {
        // Le backup exige >= 0.2.0, le binaire est 0.1.8 → DowngradeRefused (409).
        match check_import_version_compat("0.2.0", "0.1.8") {
            Err(VersionError::DowngradeRefused { db_min, binary }) => {
                assert_eq!(db_min.to_string(), "0.2.0");
                assert_eq!(binary.to_string(), "0.1.8");
            }
            other => panic!("expected DowngradeRefused, got {other:?}"),
        }
    }

    #[test]
    fn import_compat_handles_prereleases_per_semver() {
        // 0.2.0-rc1 < 0.2.0 (pré-release) : un binaire 0.2.0-rc1 ne peut lire
        // un backup exigeant 0.2.0 stable.
        assert!(check_import_version_compat("0.2.0", "0.2.0-rc1").is_err());
        // L'inverse est OK (binaire stable >= min pré-release).
        assert!(check_import_version_compat("0.2.0-rc1", "0.2.0").is_ok());
    }

    #[test]
    fn import_compat_rejects_invalid_semver_as_invalid_not_refused() {
        match check_import_version_compat("pas-semver", "0.1.8") {
            Err(VersionError::InvalidSemver { origin, .. }) => assert_eq!(origin, "backup"),
            other => panic!("expected InvalidSemver, got {other:?}"),
        }
    }

    /// Régression Pass 3 BH3-1/ECH3-1 : le slice `&value[..MAX_LEN]` byte-based
    /// panic si MAX_LEN tombe au milieu d'un caractère UTF-8 multi-byte.
    /// La troncature `chars().take(MAX_LEN)` est char-safe par construction.
    #[test]
    fn invalid_semver_handles_multibyte_utf8_at_boundary() {
        // 65 caractères grecs (α = 2 bytes UTF-8) → 130 bytes total.
        // La 65e position en bytes tomberait au milieu d'un caractère.
        let multibyte = "α".repeat(65);
        let err = VersionError::invalid_semver(
            "test",
            &multibyte,
            semver::Version::parse("nope").unwrap_err(),
        );
        match err {
            VersionError::InvalidSemver { value, .. } => {
                assert!(value.ends_with('…'));
                assert_eq!(value.chars().count(), 65, "64 chars + ellipsis");
            }
            _ => panic!("expected InvalidSemver variant"),
        }
    }
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
    let binary = Version::parse(binary_version)
        .map_err(|e| VersionError::invalid_semver("binaire", binary_version, e))?;

    let row: Result<String, sqlx::Error> =
        sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
            .fetch_one(pool)
            .await;

    match row {
        // ER_NO_SUCH_TABLE 1146 — fresh install : la table `_kesh_version`
        // n'existe pas encore (sera créée par MIGRATOR.run() juste après).
        // À distinguer du cas `RowNotFound` (table présente, row id=1
        // absente — état pathologique géré par le bras suivant).
        Err(sqlx::Error::Database(ref db_err))
            if db_err
                .try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
                .is_some_and(|e| e.number() == 1146) =>
        {
            Ok(DowngradeCheckOutcome::FreshInstall)
        }
        // Table présente mais row id=1 absente (TRUNCATE accidentel ou
        // restore partiel) — état pathologique. Refus de boot avec
        // message diagnostique explicite plutôt que message sqlx générique.
        Err(sqlx::Error::RowNotFound) => Err(VersionError::RowMissing),
        Err(e) => Err(VersionError::Sqlx(e)),
        Ok(db_min_str) => {
            let db_min = Version::parse(&db_min_str)
                .map_err(|e| VersionError::invalid_semver("base de données", &db_min_str, e))?;
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

/// Story 17-3c — Vérifie la compatibilité de version à l'**import** d'un
/// `.keshbackup`. Contrairement à [`check_downgrade_protection`] (qui lit la
/// DB au boot), cette fonction est **pure** : elle compare deux strings SemVer
/// (aucune I/O), donc testable sans base.
///
/// Sémantique (DC4) : refus **ssi `manifest_min_required > binary`** — le
/// binaire destination ne sait pas lire un schéma exigeant une version plus
/// récente que lui (downgrade-protection, identique à 10-2). On **NE refuse
/// PAS** sur `kesh_version > binary` seul (sur-restrictif : une source d'une
/// version mineure plus récente mais non-breaking reste importable ; la compat
/// schéma est vérifiée par ailleurs via le check colonnes bidirectionnel).
///
/// En cas de refus, retourne [`VersionError::DowngradeRefused`] (le caller
/// 17-3c le mappe en `409`). Les strings non-SemVer retournent
/// [`VersionError::InvalidSemver`] (le caller le mappe en `400`, manifeste
/// corrompu). Gère les pré-releases per semver.org (`0.2.0-rc1 < 0.2.0`).
pub fn check_import_version_compat(
    manifest_min_required: &str,
    binary: &str,
) -> Result<(), VersionError> {
    let min = Version::parse(manifest_min_required)
        .map_err(|e| VersionError::invalid_semver("backup", manifest_min_required, e))?;
    let bin =
        Version::parse(binary).map_err(|e| VersionError::invalid_semver("binaire", binary, e))?;
    if min > bin {
        Err(VersionError::DowngradeRefused {
            db_min: min,
            binary: bin,
        })
    } else {
        Ok(())
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
    // Valide la string SemVer côté binaire ET canonicalize la représentation
    // stockée. `binary_version` passé par l'appelant peut être `env!("CARGO_PKG_VERSION")`
    // (résolu compile-time, toujours SemVer-valide) ou une string ad-hoc (tests).
    // On stocke `parsed.to_string()` plutôt que la string brute pour s'assurer
    // que la colonne contient toujours une forme canonique SemVer (cohérent
    // avec la lecture côté `check_downgrade_protection` qui parse à nouveau).
    let parsed = Version::parse(binary_version)
        .map_err(|e| VersionError::invalid_semver("binaire", binary_version, e))?;
    let canonical = parsed.to_string();

    let result = sqlx::query(
        "UPDATE _kesh_version SET kesh_version_last_applied = ?, last_boot_at = NOW() WHERE id = 1",
    )
    .bind(&canonical)
    .execute(pool)
    .await?;

    // Note couverture : la branche `rows_affected != 1` n'est exercée par
    // aucun test (l'invariant CHECK id=1 + INSERT migration garantit la
    // row en pratique). Elle existe comme défense contre TRUNCATE/DELETE
    // manuel : `warn!` non-fatal pour ne pas casser le serveur sur cet
    // état pathologique (l'audit de version est informationnel), tout en
    // alertant clairement l'opérateur via les logs.
    if result.rows_affected() != 1 {
        let n = result.rows_affected();
        let cause = if n == 0 {
            "row _kesh_version id=1 absente (DELETE/TRUNCATE manuel ?)"
        } else {
            "plusieurs rows id=1 (invariant CHECK + PK violé manuellement ?)"
        };
        tracing::warn!(
            rows_affected = n,
            "record_boot_version: UPDATE a affecté {} rows (attendu 1) — {}",
            n,
            cause
        );
    }

    Ok(())
}
