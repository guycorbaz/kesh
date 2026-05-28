//! Initialisation du logging (Story v011-1, Issue #119).
//!
//! Deux sorties coexistent :
//! - **stdout** (`docker logs`) — comportement historique, toujours actif.
//! - **fichier avec rotation** — activé si `KESH_LOG_FILE_PATH` est défini,
//!   via `tracing-appender`. Configurable par [`crate::config::LogConfig`].
//!
//! L'init du subscriber doit avoir lieu **avant** `Config::from_env()` (pour
//! que les erreurs fatales de config soient capturées), mais le layer fichier
//! a besoin de [`LogConfig`]. On lit donc `LogConfig` (parsing infaillible) en
//! premier, puis on installe le subscriber, puis on valide la `Config`.
//!
//! ⚠️ Le [`tracing_appender::non_blocking::WorkerGuard`] retourné par
//! [`init_tracing`] DOIT rester vivant toute la durée du programme : son `Drop`
//! flush le writer non-bloquant. S'il est droppé trop tôt, des logs sont perdus.

use std::path::{Path, PathBuf};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer, fmt};

use crate::config::{LogConfig, LogFormat, LogRotation};

/// Décompose un chemin de fichier de log en `(dir, prefix, suffix)` pour le
/// builder `tracing-appender` (qui ne prend pas un chemin unique mais un
/// répertoire + un préfixe + un suffixe).
///
/// - `/var/log/kesh/kesh.log` → (`/var/log/kesh`, `kesh`, `log`)
/// - `/var/log/kesh/kesh`     → (`/var/log/kesh`, `kesh`, ``)  (pas d'extension)
/// - `kesh.log`               → (`.`, `kesh`, `log`)           (pas de parent)
///
/// Retourne `None` si aucun `file_stem` exploitable (chemin vide, se terminant
/// par `/`, etc.) → le caller bascule en stdout-only (dégradation gracieuse).
fn decompose_path(path: &str) -> Option<(PathBuf, String, String)> {
    let p = Path::new(path);
    let stem = p.file_stem()?.to_str()?.to_string();
    if stem.is_empty() {
        return None;
    }
    let dir = match p.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    };
    let suffix = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_string();
    Some((dir, stem, suffix))
}

fn to_rotation(r: LogRotation) -> Rotation {
    match r {
        LogRotation::Daily => Rotation::DAILY,
        LogRotation::Hourly => Rotation::HOURLY,
        LogRotation::Never => Rotation::NEVER,
    }
}

/// Construit un appender de fichier rotatif depuis la config.
///
/// `max_log_files` n'est appliqué que si la rotation crée de nouveaux
/// fichiers — avec [`Rotation::NEVER`] il est silencieusement ignoré
/// (fichier unique jamais élagué) : on ne l'applique alors pas.
fn build_appender(cfg: &LogConfig, path: &str) -> Result<RollingFileAppender, String> {
    let (dir, prefix, suffix) =
        decompose_path(path).ok_or_else(|| format!("chemin de log invalide: '{path}'"))?;

    let mut builder = RollingFileAppender::builder()
        .rotation(to_rotation(cfg.rotation))
        .filename_prefix(prefix);
    if !suffix.is_empty() {
        builder = builder.filename_suffix(suffix);
    }
    if cfg.rotation != LogRotation::Never {
        builder = builder.max_log_files(cfg.max_files);
    }
    builder.build(&dir).map_err(|e| {
        format!(
            "ouverture du fichier de log impossible ({}): {e}",
            dir.display()
        )
    })
}

/// Initialise le subscriber `tracing` global (stdout + fichier conditionnel)
/// et retourne le [`WorkerGuard`] du writer fichier (à conserver vivant).
///
/// Retourne `None` si aucun layer fichier n'est actif (path absent ou échec
/// d'ouverture → dégradation stdout-only).
#[must_use = "le WorkerGuard doit rester vivant pour flush les logs fichier"]
pub fn init_tracing(cfg: &LogConfig) -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

    // Layer stdout — reproduit le comportement historique (`docker logs`).
    let stdout_layer = fmt::layer();

    // Layer fichier conditionnel : `Option<BoxedLayer>` est lui-même un Layer
    // (no-op si `None`).
    let (file_layer, guard) = match cfg.file_path.as_deref() {
        Some(path) => match build_appender(cfg, path) {
            Ok(appender) => {
                let (non_blocking, guard) = tracing_appender::non_blocking(appender);
                let layer = fmt::layer().with_writer(non_blocking).with_ansi(false); // jamais de codes couleur dans un fichier
                let boxed = match cfg.format {
                    LogFormat::Json => layer.json().boxed(),
                    LogFormat::Pretty => layer.boxed(),
                };
                (Some(boxed), Some(guard))
            }
            Err(e) => {
                // Avant `.init()` : `tracing::*` serait perdu → stderr direct.
                eprintln!("[kesh-api] {e} — logs fichier désactivés, stdout conservé");
                (None, None)
            }
        },
        None => (None, None),
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(stdout_layer)
        .with(file_layer)
        .init();

    guard
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decompose_full_path() {
        let (dir, prefix, suffix) = decompose_path("/var/log/kesh/kesh.log").unwrap();
        assert_eq!(dir, PathBuf::from("/var/log/kesh"));
        assert_eq!(prefix, "kesh");
        assert_eq!(suffix, "log");
    }

    #[test]
    fn decompose_no_extension() {
        let (dir, prefix, suffix) = decompose_path("/var/log/kesh/kesh").unwrap();
        assert_eq!(dir, PathBuf::from("/var/log/kesh"));
        assert_eq!(prefix, "kesh");
        assert_eq!(suffix, "");
    }

    #[test]
    fn decompose_no_parent_falls_back_to_cwd() {
        let (dir, prefix, suffix) = decompose_path("kesh.log").unwrap();
        assert_eq!(dir, PathBuf::from("."));
        assert_eq!(prefix, "kesh");
        assert_eq!(suffix, "log");
    }

    #[test]
    fn decompose_empty_path_is_invalid() {
        assert!(decompose_path("").is_none());
    }

    #[test]
    fn build_appender_never_keeps_single_file() {
        // Avec NEVER, `max_files` est ignoré et le fichier est `{prefix}.{suffix}`.
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("kesh.log");
        let cfg = LogConfig {
            file_path: Some(log_path.to_str().unwrap().to_string()),
            rotation: LogRotation::Never,
            max_files: 7,
            format: LogFormat::Pretty,
        };
        // Ne doit pas paniquer ni erreur (répertoire temp inscriptible).
        assert!(build_appender(&cfg, log_path.to_str().unwrap()).is_ok());
    }

    /// AC #11 — un appender construit vers un répertoire temporaire produit
    /// bien un fichier contenant les entrées émises.
    #[test]
    fn file_appender_writes_entries() {
        let dir = tempfile::tempdir().unwrap();
        let log_path = dir.path().join("kesh.log");
        let cfg = LogConfig {
            file_path: Some(log_path.to_str().unwrap().to_string()),
            rotation: LogRotation::Never, // fichier unique = dir/kesh.log
            max_files: 7,
            format: LogFormat::Pretty,
        };

        let appender = build_appender(&cfg, log_path.to_str().unwrap()).unwrap();
        // `NonBlocking` est `Clone` + `MakeWriter` → utilisable directement.
        let (non_blocking, guard) = tracing_appender::non_blocking(appender);

        // Subscriber local scoped au test (jamais le global `.init()`).
        let subscriber = fmt().with_writer(non_blocking).with_ansi(false).finish();
        tracing::subscriber::with_default(subscriber, || {
            tracing::info!("v011-1-test-log-entry-marker");
        });

        drop(guard); // flush + join du worker non-bloquant

        let content = std::fs::read_to_string(&log_path).unwrap();
        assert!(
            content.contains("v011-1-test-log-entry-marker"),
            "le fichier de log doit contenir l'entrée émise, got: {content:?}"
        );
    }
}
