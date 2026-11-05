//! Story v0174-mise-a-jour (#847) — **la sauvegarde avant mise à jour**.
//!
//! Au démarrage, entre `check_downgrade_protection` et `MIGRATOR.run`
//! (`main.rs`, étape 3c), s'il reste des migrations à appliquer, Kesh écrit une
//! sauvegarde complète de l'état d'avant — base **et** pièces, `.keshbackup` de
//! format 2 — que la version de départ sait restaurer, et refuse de migrer si
//! elle échoue.
//!
//! - **Quand** : [`plan_de_migration`] lit `_sqlx_migrations` contre le
//!   `MIGRATOR` (montantes). Seul un plan *à appliquer* écrit une sauvegarde ;
//!   une base neuve, à jour ou interrompue n'en écrit pas ; un plan illisible ou
//!   incohérent refuse.
//! - **Quoi** : l'inventaire **du schéma présent**
//!   (`kesh_db::backup::Inventaire::SchemaPresent`) — exactement le
//!   `TABLES_TO_TRUNCATE` de la version qui a créé ce schéma, que son import
//!   exige.
//! - **Où** : `<KESH_ADMIN_BACKUP_DIR>/pre-upgrade/` ([`nom_de_fichier`]),
//!   dossier `0700` (`routes::admin::ensure_backup_dir`), fichier `0600`
//!   (`routes::admin::write_backup_file` : `.partial`, `sync_all`, renommage
//!   sans écrasement), puis synchronisation du dossier
//!   ([`BackupEnv::sync_dir`] — son échec n'est qu'un avertissement,
//!   C-v0174m-33).
//! - **Échappatoire** : `KESH_SKIP_PRE_UPGRADE_BACKUP` égale à la version du
//!   binaire, évaluée en premier (C-v0174m-31) ; elle ne franchit pas le plan
//!   incohérent.
//! - **Jamais de suppression** : chaque tentative écrit son fichier
//!   (C-v0174m-25).
//!
//! Le module rend une [`Issue`] ou un [`Refus`] ; il journalise en `info` le
//! début et la fin de l'écriture, et **n'émet aucun avertissement** : `main.rs`
//! les tire de l'[`Issue`] (C-v0174m-34), et le message de refus de
//! [`message_de_refus`].

use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use chrono::NaiveDateTime;
use regex::Regex;
use sqlx::MySqlPool;

use kesh_db::backup::{InstallationExport, Inventaire};
use kesh_db::errors::{DbError, map_db_error};
use kesh_db::version::DowngradeCheckOutcome;

use crate::admin_backup::export::{
    BackupError, BackupWriteError, KeshBackupMeta, build_keshbackup, write_keshbackup,
};
use crate::auto_backup::{
    AUTO_SUBDIR, BackupEnv, NOM_AUTO, fichiers_reguliers, seuil_d_espace, taille_des_documents,
};
use crate::config::Config;
use crate::routes::admin::{ensure_backup_dir, write_backup_file};

/// Sous-dossier de `KESH_ADMIN_BACKUP_DIR` qui reçoit les sauvegardes avant
/// mise à jour.
pub const PRE_UPGRADE_SUBDIR: &str = "pre-upgrade";

/// Le nom de la variable d'échappatoire (messages).
const ECHAPPATOIRE: &str = "KESH_SKIP_PRE_UPGRADE_BACKUP";

/// Longueur maximale d'une version dans le nom du fichier.
const LONGUEUR_VERSION: usize = 40;

/// Nom d'une sauvegarde avant mise à jour ([`nom_de_fichier`]).
static NOM_PRE_UPGRADE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^kesh-pre-upgrade-[0-9]{8}T[0-9]{9}Z-[0-9A-Za-z.-]{1,40}-to-[0-9A-Za-z.-]{1,40}\.keshbackup$",
    )
    .expect("expression valide")
});

/// L'état des migrations de la base, lu avant `MIGRATOR.run`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Plan {
    /// `_sqlx_migrations` absente (erreur 1146) ou vide.
    Neuve,
    /// Une ligne `success = 0` : `sqlx` refusera de toute façon.
    Interrompue,
    /// Toutes les migrations montantes du binaire sont appliquées.
    AJour,
    /// `n` migrations montantes du binaire ne sont pas appliquées.
    AAppliquer { n: usize },
}

/// Ce que l'étape a fait, sans refus. `main.rs` en tire les avertissements
/// (C-v0174m-34).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issue {
    /// `check_downgrade_protection` a rendu `FreshInstall` : rien à sauvegarder.
    InstallationNeuve,
    /// Aucune migration à appliquer.
    AJour,
    /// Une migration interrompue : `sqlx` refusera de démarrer ensuite.
    Interrompue,
    /// L'échappatoire, égale à la version du binaire, a sauté la sauvegarde
    /// alors qu'une migration est à appliquer (ou que le plan est illisible) :
    /// `main.rs` avertit « migration sans sauvegarde ».
    Contournee,
    /// La sauvegarde est écrite.
    Ecrite {
        /// Le fichier écrit.
        chemin: PathBuf,
        /// La valeur de l'échappatoire, posée mais ignorée (autre version).
        echappatoire_ignoree: Option<String>,
        /// L'erreur de la synchronisation du dossier, si elle a échoué : le
        /// fichier est écrit et synchronisé, la migration se fait.
        dossier_non_synchronise: Option<String>,
    },
}

/// Pourquoi la sauvegarde n'a pas pu être préparée : `main.rs` journalise
/// [`message_de_refus`] et s'arrête **avant** `MIGRATOR.run`.
#[derive(Debug)]
pub enum Refus {
    /// Plan illisible (`Some`) ou **incohérent** (`None` : *neuve* alors que
    /// `check_downgrade_protection` n'a pas rendu `FreshInstall`).
    Plan { erreur: Option<sqlx::Error> },
    /// Le dossier `pre-upgrade/` n'a pas pu être créé.
    Dossier { dir: PathBuf, erreur: String },
    /// Espace insuffisant (`Ok(libre)`, sous `seuil`) ou illisible (`Err`),
    /// en octets.
    Espace {
        dir: PathBuf,
        libre: Result<u64, io::Error>,
        seuil: u64,
    },
    /// Lecture de la base (version de départ, ou installation).
    Lecture { erreur: DbError },
    /// Écriture de l'archive.
    Ecriture {
        chemin: PathBuf,
        erreur: BackupWriteError,
    },
    /// Tâche d'écriture interrompue (panique).
    Tache { erreur: String },
}

/// Le point d'entrée, appelé par `main.rs` entre `check_downgrade_protection`
/// (dont il reçoit l'issue) et `MIGRATOR.run`.
///
/// 1. `FreshInstall` → [`Issue::InstallationNeuve`] ;
/// 2. le plan est lu ([`plan_de_migration`]) ;
/// 3. l'échappatoire égale à `binaire` est évaluée **avant** l'issue du plan :
///    plan *à appliquer* ou illisible → [`Issue::Contournee`] ; *à jour*,
///    *interrompue* → l'issue du plan ; *neuve* (incohérent) → refus ;
/// 4. sinon : plan illisible ou incohérent → refus ; *à jour*, *interrompue*
///    → l'issue du plan ; *à appliquer* → la version de départ est lue, puis
///    la sauvegarde écrite ([`ecrire_la_sauvegarde`]) ; une échappatoire posée
///    à une autre valeur est rendue dans `echappatoire_ignoree`.
pub async fn sauvegarder_avant_migration(
    pool: &MySqlPool,
    config: &Config,
    outcome: &DowngradeCheckOutcome,
    binaire: &str,
    environnement: Arc<dyn BackupEnv>,
    now: NaiveDateTime,
) -> Result<Issue, Refus> {
    if matches!(outcome, DowngradeCheckOutcome::FreshInstall) {
        return Ok(Issue::InstallationNeuve);
    }
    let echappatoire = config.skip_pre_upgrade_backup.as_deref();
    let plan = plan_de_migration(pool).await;
    if echappatoire == Some(binaire) {
        return match plan {
            Err(_) | Ok(Plan::AAppliquer { .. }) => Ok(Issue::Contournee),
            Ok(Plan::AJour) => Ok(Issue::AJour),
            Ok(Plan::Interrompue) => Ok(Issue::Interrompue),
            Ok(Plan::Neuve) => Err(Refus::Plan { erreur: None }),
        };
    }
    let n = match plan.map_err(|e| Refus::Plan { erreur: Some(e) })? {
        Plan::Neuve => return Err(Refus::Plan { erreur: None }),
        Plan::Interrompue => return Ok(Issue::Interrompue),
        Plan::AJour => return Ok(Issue::AJour),
        Plan::AAppliquer { n } => n,
    };
    let de: String =
        sqlx::query_scalar("SELECT kesh_version_last_applied FROM _kesh_version WHERE id = 1")
            .fetch_one(pool)
            .await
            .map_err(|e| Refus::Lecture {
                erreur: map_db_error(e),
            })?;
    tracing::info!(
        "Mise à jour de la base : {n} migration(s) à appliquer (de la {de} à la {binaire}). \
         Sauvegarde complète avant mise à jour en cours ; le serveur répondra une fois la mise \
         à jour terminée."
    );
    let mut issue = ecrire_la_sauvegarde(pool, config, environnement, &de, binaire, now).await?;
    if let Issue::Ecrite {
        echappatoire_ignoree,
        ..
    } = &mut issue
    {
        *echappatoire_ignoree = echappatoire.map(str::to_owned);
    }
    Ok(issue)
}

/// Le plan des migrations : `_sqlx_migrations` contre les migrations
/// **montantes** du `MIGRATOR`. Erreur 1146 (table absente) ou table vide →
/// [`Plan::Neuve`] ; une ligne `success = 0` → [`Plan::Interrompue`] ; aucune
/// montante absente → [`Plan::AJour`] ; sinon [`Plan::AAppliquer`]. Toute
/// autre erreur est rendue (refus, C-v0174m-16). Une migration appliquée
/// inconnue du binaire ne change pas le plan : `sqlx` refusera ensuite.
pub(crate) async fn plan_de_migration(pool: &MySqlPool) -> Result<Plan, sqlx::Error> {
    let lignes: Vec<(i64, bool)> =
        match sqlx::query_as("SELECT version, success FROM _sqlx_migrations")
            .fetch_all(pool)
            .await
        {
            Ok(l) => l,
            Err(sqlx::Error::Database(ref e))
                if e.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>()
                    .is_some_and(|e| e.number() == 1146) =>
            {
                return Ok(Plan::Neuve);
            }
            Err(e) => return Err(e),
        };
    if lignes.is_empty() {
        return Ok(Plan::Neuve);
    }
    if lignes.iter().any(|(_, succes)| !succes) {
        return Ok(Plan::Interrompue);
    }
    let appliquees: HashSet<i64> = lignes.into_iter().map(|(v, _)| v).collect();
    let n = kesh_db::MIGRATOR
        .iter()
        .filter(|m| !m.migration_type.is_down_migration())
        .filter(|m| !appliquees.contains(&m.version))
        .count();
    Ok(if n == 0 {
        Plan::AJour
    } else {
        Plan::AAppliquer { n }
    })
}

/// La plus grande sauvegarde présente dans `auto/` et `pre-upgrade/` (octets) :
/// la « dernière » du seuil d'espace (C-v0174m-7). Un dossier illisible compte
/// pour rien.
fn plus_grande_sauvegarde(backup_dir: &Path, dir: &Path) -> u64 {
    let auto = fichiers_reguliers(&backup_dir.join(AUTO_SUBDIR), |n| NOM_AUTO.is_match(n))
        .unwrap_or_default();
    let avant = fichiers_reguliers(dir, |n| NOM_PRE_UPGRADE.is_match(n)).unwrap_or_default();
    auto.iter()
        .chain(avant.iter())
        .map(|(_, m, _)| m.len())
        .max()
        .unwrap_or(0)
}

/// L'écriture de la sauvegarde, `de` (version de départ) lu par l'appelant :
/// dossier `pre-upgrade/` créé (`0700`), seuil d'espace, lecture de la base à
/// l'inventaire du schéma présent et écriture de l'archive
/// (`build_keshbackup`, `write_backup_file`), synchronisation du dossier.
/// Journalise en `info` le chemin et la taille du fichier écrit.
pub(crate) async fn ecrire_la_sauvegarde(
    pool: &MySqlPool,
    config: &Config,
    environnement: Arc<dyn BackupEnv>,
    de: &str,
    vers: &str,
    now: NaiveDateTime,
) -> Result<Issue, Refus> {
    let backup_dir = PathBuf::from(&config.admin_backup_dir);
    let dir = backup_dir.join(PRE_UPGRADE_SUBDIR);
    ensure_backup_dir(&dir.to_string_lossy())
        .await
        .map_err(|erreur| Refus::Dossier {
            dir: dir.clone(),
            erreur,
        })?;

    let documents_dir = PathBuf::from(&config.documents_dir);
    let seuil = seuil_d_espace(
        plus_grande_sauvegarde(&backup_dir, &dir),
        taille_des_documents(&documents_dir),
    );
    match environnement.free_bytes(&dir) {
        Ok(libre) if libre >= seuil => {}
        libre => return Err(Refus::Espace { dir, libre, seuil }),
    }

    let chemin = dir.join(nom_de_fichier(now, de, vers));
    let (fichier, ecrivains) = (chemin.clone(), Arc::clone(&environnement));
    let remplir = move |export: InstallationExport, documents: PathBuf| {
        write_backup_file(&fichier, |f| {
            write_keshbackup(&export, &documents, ecrivains.wrap_writer(f)).map(|(_, meta)| meta)
        })
    };
    let meta: KeshBackupMeta =
        build_keshbackup(pool, Inventaire::SchemaPresent, documents_dir, remplir)
            .await
            .map_err(|e| match e {
                BackupError::Db(erreur) => Refus::Lecture { erreur },
                BackupError::Write(erreur) => Refus::Ecriture {
                    chemin: chemin.clone(),
                    erreur,
                },
                BackupError::Task(erreur) => Refus::Tache { erreur },
            })?;

    let dossier_non_synchronise = environnement.sync_dir(&dir).err().map(|e| e.to_string());
    tracing::info!(
        "Sauvegarde avant mise à jour écrite : {} ({} octets).",
        chemin.display(),
        meta.byte_size
    );
    Ok(Issue::Ecrite {
        chemin,
        echappatoire_ignoree: None,
        dossier_non_synchronise,
    })
}

/// Une version réduite aux caractères `[0-9A-Za-z.-]`, 40 au plus ; vide →
/// `inconnue`.
fn version_du_nom(version: &str) -> String {
    let propre: String = version
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '.' || *c == '-')
        .take(LONGUEUR_VERSION)
        .collect();
    if propre.is_empty() {
        "inconnue".to_string()
    } else {
        propre
    }
}

/// Le nom d'une sauvegarde avant mise à jour prise à `now` (UTC, à la
/// milliseconde), de la version `de` à la version `vers` :
/// `kesh-pre-upgrade-<AAAAMMJJTHHMMSSmmm>Z-<de>-to-<vers>.keshbackup`. Le tri
/// par nom est chronologique.
pub(crate) fn nom_de_fichier(now: NaiveDateTime, de: &str, vers: &str) -> String {
    format!(
        "kesh-pre-upgrade-{}Z-{}-to-{}.keshbackup",
        now.format("%Y%m%dT%H%M%S%3f"),
        version_du_nom(de),
        version_du_nom(vers)
    )
}

/// Octets → Mio, par défaut.
fn mio_bas(octets: u64) -> u64 {
    octets / (1024 * 1024)
}

/// Octets → Mio, par excès.
fn mio_haut(octets: u64) -> u64 {
    octets.div_ceil(1024 * 1024)
}

/// Le message de refus (`error`), `vers` étant la version du binaire. Il ne
/// cite jamais `DATABASE_URL`. Le conseil de l'échappatoire vaut pour toutes
/// les causes, plan illisible compris (elle est évaluée en premier), **sauf**
/// le plan incohérent, qu'elle ne franchit pas.
pub fn message_de_refus(refus: &Refus, vers: &str) -> String {
    let cause = match refus {
        Refus::Plan { erreur: Some(e) } => format!("état des migrations illisible : {e}"),
        Refus::Plan { erreur: None } => "état des migrations incohérent (_kesh_version présente, \
                                         _sqlx_migrations absente ou vide)"
            .to_string(),
        Refus::Dossier { dir, erreur } => format!("dossier {} : {erreur}", dir.display()),
        Refus::Espace {
            dir,
            libre: Ok(libre),
            seuil,
        } => format!(
            "espace insuffisant dans {} ({} Mio libres, {} Mio requis)",
            dir.display(),
            mio_bas(*libre),
            mio_haut(*seuil)
        ),
        Refus::Espace {
            dir, libre: Err(e), ..
        } => format!("espace libre de {} illisible : {e}", dir.display()),
        Refus::Lecture { erreur } => format!("lecture de la base : {erreur}"),
        Refus::Ecriture { chemin, erreur } => {
            format!("écriture de {} : {erreur}", chemin.display())
        }
        Refus::Tache { erreur } => format!("tâche interrompue : {erreur}"),
    };
    let debut = format!(
        "FATAL : mise à jour refusée — la sauvegarde avant mise à jour n'a pas pu être préparée \
         ({cause}). Aucune migration n'a été appliquée par cette tentative. Corrigez la cause"
    );
    if matches!(refus, Refus::Plan { erreur: None }) {
        format!("{debut}.")
    } else {
        format!(
            "{debut}, ou, en connaissance de cause, posez {ECHAPPATOIRE}={vers} dans le .env pour \
             migrer sans sauvegarde (le compose doit la transmettre ; manuel d'administration, « La \
             sauvegarde avant mise à jour »)."
        )
    }
}

/// L'avertissement de l'échappatoire utilisée (`warn`, [`Issue::Contournee`]).
pub fn message_echappatoire_utilisee(vers: &str) -> String {
    format!(
        "{ECHAPPATOIRE}={vers} : migration sans sauvegarde préalable, à la demande de \
         l'administrateur. Retirez cette variable après la mise à jour."
    )
}

/// L'avertissement de l'échappatoire ignorée (`warn`, `echappatoire_ignoree`).
pub fn message_echappatoire_ignoree(valeur: &str, vers: &str) -> String {
    format!("{ECHAPPATOIRE}={valeur} ignorée : elle ne vaut que pour la version {vers}.")
}

/// L'avertissement du dossier non synchronisé (`warn`,
/// `dossier_non_synchronise`).
pub fn message_dossier_non_synchronise(chemin: &Path, erreur: &str) -> String {
    let dir = chemin.parent().unwrap_or(chemin);
    format!(
        "Synchronisation du dossier {} refusée ({erreur}) — montage sans fsync de dossier ? La \
         sauvegarde {} est écrite et synchronisée ; la mise à jour continue.",
        dir.display(),
        chemin.display()
    )
}

#[cfg(test)]
mod tests;
