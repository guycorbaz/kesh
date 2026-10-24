//! Story 31-1c (#503) — la sauvegarde automatique faite par Kesh lui-même.
//!
//! Une tâche de fond ([`spawn`], lancée par `main.rs` et par lui seul) contrôle
//! toutes les 5 minutes le réglage `_kesh_backup_schedule` ; quand une
//! sauvegarde est due ([`is_due`]), [`run_once`] écrit une sauvegarde complète
//! (format `.keshbackup` 2, pièces comprises) dans
//! `KESH_ADMIN_BACKUP_DIR/auto/`, la relit, enregistre son état, puis applique
//! la rétention. « Sauvegarder maintenant » (`POST
//! /api/v1/admin/backup-schedule/run`) appelle le même [`run_once`].
//!
//! # Une seule sauvegarde à la fois
//!
//! `AppState::backup_lock` (un `tokio::sync::Mutex<()>` en mémoire) sérialise
//! la sauvegarde automatique, l'export manuel et l'import. [`run_once`] le
//! prend **lui-même**, en `try_lock_owned` : pris ailleurs, il rend
//! [`RunOutcome::Skipped`] sans rien écrire. Toujours un garde **possédé**.
//! Le verrou est **en mémoire** : rien ne protège deux processus Kesh
//! partageant une base l'un de l'autre — Kesh se déploie à une instance par
//! base (angle mort écrit au manuel).
//!
//! # Angles morts assumés
//!
//! - La sauvegarde ne tourne que si Kesh tourne ; une tâche morte se voit au
//!   bandeau « aucune sauvegarde réussie depuis … » ([`is_overdue`]), pas
//!   autrement. La cadence (5 min, `Skip`) et l'isolation d'une panique dans
//!   un tour ne sont pas testées (quelques lignes, relues en revue).
//! - L'espace libre est mesuré **avant** l'écriture : un autre programme peut
//!   remplir le disque pendant (statut `disk_full`).
//! - La relecture prouve que l'annuaire central et le manifeste sont lisibles,
//!   **pas** l'intégrité des entrées `data/` et `files/` (leurs CRC ne sont pas
//!   relus).
//! - Pas d'arrêt gracieux : un arrêt pendant l'écriture laisse un `.partial`,
//!   supprimé au tour suivant.
//! - Horloge déréglée vers le futur : un nom daté du futur passe pour le plus
//!   récent et survit à la rétention (qui trie par nom).

use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};

use chrono::{NaiveDateTime, TimeDelta, Utc};
use regex::Regex;

use kesh_db::backup::InstallationExport;
use kesh_db::errors::DbError;
use kesh_db::repositories::backup_schedule::{self, BackupSchedule, BackupStatus, BackupSuccess};

use crate::AppState;
use crate::admin_backup::export::{KeshBackupMeta, build_keshbackup, write_keshbackup};
use crate::routes::admin::write_backup_file;

/// Sous-dossier de `KESH_ADMIN_BACKUP_DIR` qui reçoit les sauvegardes
/// automatiques.
pub const AUTO_SUBDIR: &str = "auto";

/// Délai avant le premier contrôle, puis entre deux contrôles.
const CADENCE: std::time::Duration = std::time::Duration::from_secs(5 * 60);

/// Marge fixe du seuil d'espace libre (256 Mio).
const MARGE_FIXE: u64 = 256 * 1024 * 1024;

/// Nom d'une sauvegarde automatique (UTC, millisecondes comprises).
/// `[0-9]` et non `\d`, qui est Unicode dans le crate `regex`.
pub(crate) static NOM_AUTO: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^kesh-auto-[0-9]{8}T[0-9]{9}Z\.keshbackup$").expect("expression valide")
});

/// Nom d'une sauvegarde pré-import (forme écrite par
/// `routes::admin::write_pre_import_backup` : horodatage, pid, compteur).
pub(crate) static NOM_PRE_IMPORT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^kesh-pre-import-[0-9]{8}T[0-9]{9}-[0-9]+-[0-9]+\.keshbackup$")
        .expect("expression valide")
});

// ---------------------------------------------------------------------------
// Types publics (les tests d'intégration vivent dans un crate externe, et
// `main.rs` est un crate binaire distinct)
// ---------------------------------------------------------------------------

/// Trait d'union des écrivains d'archive : un objet-trait n'admet qu'un seul
/// trait non automatique (E0225), d'où ce trait et son implémentation
/// générale.
pub trait WriteSeek: Write + Seek + Send {}
impl<T: Write + Seek + Send> WriteSeek for T {}

/// Ce que [`run_once`] demande à son environnement — injecté pour que les
/// tests simulent un disque plein ou presque plein.
pub trait BackupEnv: Send + Sync {
    /// Espace accessible (octets) du système de fichiers de `dir`.
    fn free_bytes(&self, dir: &Path) -> io::Result<u64>;
    /// L'écrivain de l'archive, construit sur un **clone** du descripteur du
    /// `.partial` (possédé : il entre dans le `spawn_blocking`).
    fn wrap_writer(&self, file: File) -> Box<dyn WriteSeek>;
}

/// L'environnement de production : `statvfs`, et le fichier tel quel.
pub struct RealBackupEnv;

impl BackupEnv for RealBackupEnv {
    /// `f_bavail × f_frsize` : l'espace accessible à un utilisateur ordinaire,
    /// non `f_bfree`, qui compterait la réserve de `root`.
    fn free_bytes(&self, dir: &Path) -> io::Result<u64> {
        let stat = rustix::fs::statvfs(dir).map_err(io::Error::from)?;
        Ok(stat.f_bavail.saturating_mul(stat.f_frsize))
    }

    fn wrap_writer(&self, file: File) -> Box<dyn WriteSeek> {
        Box::new(file)
    }
}

/// Ce qui a déclenché une sauvegarde.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    /// La tâche de fond, sur `is_due`.
    Schedule,
    /// « Sauvegarder maintenant » (sans la condition `is_due`).
    Manual,
}

/// Une sauvegarde écrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuccessStatus {
    /// Toutes les pièces référencées incluses.
    Ok,
    /// Des pièces référencées n'ont pas pu y entrer.
    Partial,
}

/// Une sauvegarde non écrite.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FailureStatus {
    /// Échec d'écriture (autre que disque plein) ou de relecture.
    Failed,
    /// Refus avant écriture : espace libre sous le seuil.
    InsufficientSpace,
    /// Disque ou quota saturé pendant l'écriture.
    DiskFull,
}

/// Le statut d'une sauvegarde menée à son terme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    Success(SuccessStatus),
    Failure(FailureStatus),
}

impl RunStatus {
    /// Le statut stocké en base.
    pub fn as_backup_status(self) -> BackupStatus {
        match self {
            Self::Success(SuccessStatus::Ok) => BackupStatus::Ok,
            Self::Success(SuccessStatus::Partial) => BackupStatus::Partial,
            Self::Failure(FailureStatus::Failed) => BackupStatus::Failed,
            Self::Failure(FailureStatus::InsufficientSpace) => BackupStatus::InsufficientSpace,
            Self::Failure(FailureStatus::DiskFull) => BackupStatus::DiskFull,
        }
    }
}

/// Ce que rend [`run_once`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunOutcome {
    /// Le verrou était pris par une autre sauvegarde, un export ou un import :
    /// rien n'a été écrit, ni en base ni sur disque.
    Skipped,
    /// La tentative a eu lieu, avec ce statut (enregistré en base).
    Done(RunStatus),
}

// ---------------------------------------------------------------------------
// Le moment d'une sauvegarde (fonctions pures)
// ---------------------------------------------------------------------------

fn intervalle(schedule: &BackupSchedule) -> TimeDelta {
    TimeDelta::hours(i64::from(schedule.interval_hours))
}

/// Un horodatage **postérieur** à `now` (horloge partie dans le futur puis
/// corrigée) vaut nul : sans quoi la différence négative arrêterait la
/// sauvegarde, sans échec ni bandeau, jusqu'à ce que l'horloge le rattrape.
fn passe(valeur: Option<NaiveDateTime>, now: NaiveDateTime) -> Option<NaiveDateTime> {
    valeur.filter(|v| *v <= now)
}

/// Délai avant de retenter après une tentative : une heure. La fiche écrit
/// `min(intervalle, 1 h)` ; aucune fréquence admise
/// (`ALLOWED_INTERVAL_HOURS`, 6 h au moins, gardée par la `CHECK` de la
/// table) n'étant inférieure à une heure, le `min` était mort et le manuel
/// promettait un cas inatteignable (revue de code P1, L-B4, C-31-1-91).
const DELAI_DE_REPRISE: TimeDelta = TimeDelta::hours(1);

/// AC25 — vrai si et seulement si la sauvegarde est activée, que la dernière
/// réussite est nulle ou date d'au moins un intervalle, et que la dernière
/// tentative est nulle ou date d'au moins une heure ([`DELAI_DE_REPRISE`] :
/// un échec se retente au bout d'une heure, pas en boucle). Une valeur
/// postérieure à `now` vaut nulle (horloge revenue en arrière).
pub fn is_due(schedule: &BackupSchedule, now: NaiveDateTime) -> bool {
    schedule.enabled && next_due_at(schedule, now).is_some_and(|t| t <= now)
}

/// AC32 — l'instant à partir duquel [`is_due`] devient vrai, par la même
/// règle (une seule) ; jamais antérieur à `now` ; `None` si la sauvegarde est
/// désactivée.
pub fn next_due_at(schedule: &BackupSchedule, now: NaiveDateTime) -> Option<NaiveDateTime> {
    if !schedule.enabled {
        return None;
    }
    let apres_reussite =
        passe(schedule.last_success_at, now).map_or(now, |t| t + intervalle(schedule));
    let apres_tentative =
        passe(schedule.last_attempt_at, now).map_or(now, |t| t + DELAI_DE_REPRISE);
    Some(apres_reussite.max(apres_tentative).max(now))
}

/// AC25, AC31 (b) — vrai si et seulement si la sauvegarde est activée, qu'une
/// réussite existe, et qu'elle date d'au moins **deux** intervalles — ou
/// qu'elle est postérieure à `now` (horloge revenue en arrière). Calculé par
/// le serveur : le navigateur ne compare aucune date à sa propre horloge.
pub fn is_overdue(schedule: &BackupSchedule, now: NaiveDateTime) -> bool {
    match (schedule.enabled, schedule.last_success_at) {
        (true, Some(reussite)) => reussite > now || now - reussite >= intervalle(schedule) * 2,
        _ => false,
    }
}

// ---------------------------------------------------------------------------
// Classement des erreurs d'écriture
// ---------------------------------------------------------------------------

/// AC30 — `disk_full` si l'erreur dit un disque ou un quota saturé, `failed`
/// sinon. **`kind()` d'abord** : une `ZipError` convertie par
/// `io::Error::from` garde son `kind()` mais perd `raw_os_error()` ; le repli
/// sur le code brut (`ENOSPC` 28, `EDQUOT` 122) ne vaut que pour une erreur
/// née hors du crate `zip`.
fn classify_write_error(err: &io::Error) -> FailureStatus {
    match err.kind() {
        io::ErrorKind::StorageFull | io::ErrorKind::QuotaExceeded => FailureStatus::DiskFull,
        _ if matches!(err.raw_os_error(), Some(28 | 122)) => FailureStatus::DiskFull,
        _ => FailureStatus::Failed,
    }
}

// ---------------------------------------------------------------------------
// Fichiers du dossier de sauvegarde
// ---------------------------------------------------------------------------

/// Le type d'une sauvegarde du serveur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum BackupKind {
    Auto,
    PreImport,
}

impl BackupKind {
    /// Le segment d'URL et la valeur `kind` de l'API.
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::PreImport => "pre-import",
        }
    }

    pub(crate) fn parse(s: &str) -> Option<Self> {
        match s {
            "auto" => Some(Self::Auto),
            "pre-import" => Some(Self::PreImport),
            _ => None,
        }
    }

    pub(crate) fn nom_valide(self, nom: &str) -> bool {
        match self {
            Self::Auto => NOM_AUTO.is_match(nom),
            Self::PreImport => NOM_PRE_IMPORT.is_match(nom),
        }
    }
}

/// Une sauvegarde présente sur le serveur.
#[derive(Debug, Clone)]
pub(crate) struct ServerBackup {
    pub kind: BackupKind,
    pub name: String,
    pub size: u64,
    pub modified: Option<std::time::SystemTime>,
    pub path: PathBuf,
}

/// Fichiers **réguliers** de `dir` (`symlink_metadata` : un lien n'est
/// jamais suivi) dont le nom satisfait `garder`. Un dossier absent rend une
/// liste vide ; un dossier illisible (`read_dir` en échec : droits, `dir`
/// qui n'est pas un dossier) rend `Err`. Une **entrée** illisible (entrée du
/// dossier ou métadonnées en échec) est sautée, journalisée si elle ne s'est
/// pas simplement évanouie : elle ne fait pas échouer les autres (revue de
/// code P1, E-4, C-31-1-88).
fn fichiers_reguliers(
    dir: &Path,
    garder: impl Fn(&str) -> bool,
) -> io::Result<Vec<(String, std::fs::Metadata, PathBuf)>> {
    let entrees = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e),
    };
    let mut fichiers = Vec::new();
    for entree in entrees {
        let entree = match entree {
            Ok(e) => e,
            Err(e) => {
                tracing::warn!(dir = %dir.display(), "entrée du dossier de sauvegarde illisible : {e}");
                continue;
            }
        };
        let Some(nom) = entree.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if !garder(&nom) {
            continue;
        }
        let chemin = entree.path();
        // Un fichier disparu entre la lecture du dossier et celle de ses
        // métadonnées (renommé, supprimé) n'est plus là : il est sauté ; des
        // métadonnées illisibles (droits) aussi, journalisées.
        let meta = match std::fs::symlink_metadata(&chemin) {
            Ok(m) => m,
            Err(e) if e.kind() == io::ErrorKind::NotFound => continue,
            Err(e) => {
                tracing::warn!(path = %chemin.display(), "fichier de sauvegarde illisible : {e}");
                continue;
            }
        };
        if meta.file_type().is_file() {
            fichiers.push((nom, meta, chemin));
        }
    }
    Ok(fichiers)
}

/// AC32 — les sauvegardes du serveur : `auto/kesh-auto-*.keshbackup` et
/// `kesh-pre-import-*.keshbackup` du dossier de sauvegarde, fichiers réguliers
/// dont le nom satisfait l'expression de leur type (aucun `.partial`, aucun
/// autre fichier), les plus récentes en tête.
///
/// Un dossier `auto` **illisible** (remplacé par un fichier, droits cassés)
/// n'emporte pas la liste : ses lignes manquent, l'erreur est journalisée, et
/// les sauvegardes pré-import restent servies — c'est la voie normale pour en
/// récupérer une (manuel). Le problème se lit à l'état de la sauvegarde
/// automatique, que le tour suivant met en échec (`failed`) faute de pouvoir
/// y écrire (revue de code P1, L-B2 / E-4, C-31-1-88). Seul un dossier de
/// sauvegarde lui-même illisible rend `Err` (rien ne peut alors être listé).
pub(crate) fn lister_les_sauvegardes(backup_dir: &Path) -> io::Result<Vec<ServerBackup>> {
    let mut liste = Vec::new();
    for (kind, dir) in [
        (BackupKind::Auto, backup_dir.join(AUTO_SUBDIR)),
        (BackupKind::PreImport, backup_dir.to_path_buf()),
    ] {
        let fichiers = match fichiers_reguliers(&dir, |n| kind.nom_valide(n)) {
            Ok(f) => f,
            Err(e) if kind == BackupKind::Auto => {
                tracing::warn!(dir = %dir.display(), "dossier des sauvegardes automatiques illisible : {e}");
                Vec::new()
            }
            Err(e) => return Err(e),
        };
        for (name, meta, path) in fichiers {
            liste.push(ServerBackup {
                kind,
                name,
                size: meta.len(),
                modified: meta.modified().ok(),
                path,
            });
        }
    }
    liste.sort_by(|a, b| {
        b.modified
            .cmp(&a.modified)
            .then_with(|| b.name.cmp(&a.name))
    });
    Ok(liste)
}

/// Ouvre en lecture une sauvegarde trouvée par [`lister_les_sauvegardes`],
/// **sans suivre** un lien symbolique (`O_NOFOLLOW`, comme la liste qui les
/// écarte par `symlink_metadata`). `Ok(None)` — le téléchargement rend alors
/// 404, comme pour un nom absent — si le fichier a disparu depuis la liste
/// (rétention, suppression à la main), s'il est devenu un lien, ou s'il n'est
/// plus un fichier régulier ; `Err` pour toute autre erreur (revue de code P1,
/// L-B1 / E-5, C-31-1-89).
pub(crate) fn ouvrir_la_sauvegarde(path: &Path) -> io::Result<Option<File>> {
    use rustix::fs::{Mode, OFlags};
    use rustix::io::Errno;
    let fd = match rustix::fs::open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    ) {
        Ok(fd) => fd,
        Err(Errno::NOENT | Errno::LOOP) => return Ok(None),
        Err(e) => return Err(e.into()),
    };
    let fichier = File::from(fd);
    if !fichier.metadata()?.file_type().is_file() {
        return Ok(None);
    }
    Ok(Some(fichier))
}

/// Un `.partial` que personne n'a modifié depuis ce délai est abandonné.
pub(crate) const PARTIEL_ABANDONNE_APRES: std::time::Duration =
    std::time::Duration::from_secs(10 * 60);

/// Supprime les fichiers réguliers `<prefixe>*.keshbackup.partial` de `dir`
/// laissés par un arrêt brutal pendant une écriture — ceux-là **seuls** : un
/// `.partial` modifié depuis moins de [`PARTIEL_ABANDONNE_APRES`] est en cours
/// d'écriture (chaque écriture avance sa date de modification) et reste. Le
/// verrou `backup_lock` garantit qu'aucune écriture **de ce processus** n'est
/// en cours ; le délai protège celle d'un autre (deux processus sur un même
/// dossier, ou des tests parallèles qui le partagent — C-31-1-82). L'écart
/// se mesure **dans les deux sens** : une date postérieure à `maintenant`
/// d'au moins ce délai (horloge revenue en arrière depuis l'arrêt brutal)
/// vaut abandon ; à moins de ce délai dans le futur (horloge d'un partage
/// réseau légèrement en avance), le fichier reste (revue de code P1, E-3,
/// C-31-1-90). Date de modification illisible : le fichier reste. Un échec
/// est journalisé ; rend le nombre de fichiers supprimés.
pub(crate) fn supprimer_les_partiels(dir: &Path, prefixe: &str) -> usize {
    let partiels = match fichiers_reguliers(dir, |n| {
        n.starts_with(prefixe) && n.ends_with(".keshbackup.partial")
    }) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(dir = %dir.display(), "lecture du dossier de sauvegarde : {e}");
            return 0;
        }
    };
    let maintenant = std::time::SystemTime::now();
    let abandonne = |meta: &std::fs::Metadata| {
        meta.modified()
            .ok()
            .map(|m| match maintenant.duration_since(m) {
                Ok(age) => age,
                Err(futur) => futur.duration(),
            })
            .is_some_and(|ecart| ecart >= PARTIEL_ABANDONNE_APRES)
    };
    let mut supprimes = 0;
    for (_, meta, chemin) in partiels {
        if !abandonne(&meta) {
            tracing::info!(
                path = %chemin.display(),
                "sauvegarde partielle récente laissée (écriture peut-être en cours)"
            );
            continue;
        }
        match std::fs::remove_file(&chemin) {
            Ok(()) => {
                supprimes += 1;
                tracing::info!(path = %chemin.display(), "sauvegarde partielle résiduelle supprimée");
            }
            Err(e) => tracing::warn!(
                path = %chemin.display(),
                "sauvegarde partielle résiduelle non supprimée : {e}"
            ),
        }
    }
    supprimes
}

/// AC30 — somme des tailles des fichiers réguliers du premier niveau du
/// dossier des documents (`store_document` n'écrit qu'à ce niveau) : majorant
/// des pièces que l'archive emportera. Dossier absent : 0 (aucune pièce
/// encore) ; dossier illisible : 0, journalisé ; une entrée illisible est
/// sautée, journalisée, et la somme ne la compte pas (C-31-1-88).
fn taille_des_documents(documents_dir: &Path) -> u64 {
    match fichiers_reguliers(documents_dir, |_| true) {
        Ok(fichiers) => fichiers.iter().map(|(_, m, _)| m.len()).sum(),
        Err(e) => {
            tracing::warn!(
                dir = %documents_dir.display(),
                "taille du dossier des documents illisible, comptée nulle : {e}"
            );
            0
        }
    }
}

/// AC30 — `max(dernière sauvegarde, documents) × 1,2 + 256 Mio`.
fn seuil_d_espace(derniere: u64, documents: u64) -> u64 {
    let base = derniere.max(documents);
    base.saturating_add(base / 5).saturating_add(MARGE_FIXE)
}

/// AC29 — rétention : parmi les fichiers réguliers de `auto/` au nom exact
/// d'une sauvegarde automatique, triés par nom (= par date), supprime les plus
/// anciens au-delà de `garder`, **jamais** un nom de `proteges` (la sauvegarde
/// qui vient d'être écrite, la dernière sauvegarde complète). Rend le nombre
/// de fichiers supprimés ; un échec est journalisé. Un fichier dont les
/// métadonnées sont illisibles n'est ni compté ni supprimé : la rétention en
/// garde alors plus, jamais moins (C-31-1-88).
fn appliquer_la_retention(auto_dir: &Path, garder: usize, proteges: &[&str]) -> usize {
    let mut noms: Vec<(String, PathBuf)> =
        match fichiers_reguliers(auto_dir, |n| NOM_AUTO.is_match(n)) {
            Ok(f) => f.into_iter().map(|(n, _, p)| (n, p)).collect(),
            Err(e) => {
                tracing::warn!(dir = %auto_dir.display(), "rétention : lecture du dossier : {e}");
                return 0;
            }
        };
    noms.sort();
    let surplus = noms.len().saturating_sub(garder);
    let mut supprimes = 0;
    for (nom, chemin) in noms.into_iter().take(surplus) {
        if proteges.contains(&nom.as_str()) {
            continue;
        }
        match std::fs::remove_file(&chemin) {
            Ok(()) => supprimes += 1,
            Err(e) => tracing::warn!(path = %chemin.display(), "rétention : suppression : {e}"),
        }
    }
    supprimes
}

/// AC28 — relit une sauvegarde écrite : annuaire central et `manifest.json`
/// (JSON portant `formatVersion`). Ne relit pas les CRC des entrées.
fn relire(path: &Path) -> Result<(), String> {
    let fichier = File::open(path).map_err(|e| format!("ouverture : {e}"))?;
    let mut archive = zip::ZipArchive::new(fichier).map_err(|e| format!("annuaire : {e}"))?;
    let mut manifeste = archive
        .by_name("manifest.json")
        .map_err(|e| format!("manifeste : {e}"))?;
    let mut octets = Vec::new();
    (&mut manifeste)
        .take(64 * 1024 * 1024)
        .read_to_end(&mut octets)
        .map_err(|e| format!("lecture du manifeste : {e}"))?;
    let valeur: serde_json::Value =
        serde_json::from_slice(&octets).map_err(|e| format!("manifeste illisible : {e}"))?;
    if valeur.get("formatVersion").is_none() {
        return Err("manifeste sans formatVersion".into());
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Une sauvegarde
// ---------------------------------------------------------------------------

/// L'issue de l'écriture, rendue par la fermeture du `spawn_blocking`.
enum Ecriture {
    Ecrite { nom: String, meta: KeshBackupMeta },
    Refusee(FailureStatus),
}

/// Le nom d'une sauvegarde automatique prise à `now` (UTC, millisecondes).
pub fn nom_de_sauvegarde(now: NaiveDateTime) -> String {
    format!("kesh-auto-{}Z.keshbackup", now.format("%Y%m%dT%H%M%S%3f"))
}

/// Écrit la sauvegarde dans `auto/` — tout ce qui touche le disque, dans le
/// seul `spawn_blocking` de `build_keshbackup` : dossier `0700`, `.partial`
/// résiduels, seuil d'espace, écriture par `write_backup_file` (`0600`,
/// renommage, jamais d'écrasement), relecture.
fn ecrire(
    export: InstallationExport,
    documents_dir: PathBuf,
    auto_dir: PathBuf,
    nom: String,
    derniere_taille: u64,
    environnement: Arc<dyn BackupEnv>,
) -> Ecriture {
    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
    if let Err(e) = builder.create(&auto_dir) {
        tracing::error!(dir = %auto_dir.display(), "sauvegarde automatique : dossier : {e}");
        return Ecriture::Refusee(FailureStatus::Failed);
    }
    supprimer_les_partiels(&auto_dir, "kesh-auto-");

    let documents = taille_des_documents(&documents_dir);
    let seuil = seuil_d_espace(derniere_taille, documents);
    match environnement.free_bytes(&auto_dir) {
        Ok(libre) if libre < seuil => {
            tracing::warn!(
                libre,
                seuil,
                "sauvegarde automatique refusée : espace insuffisant"
            );
            return Ecriture::Refusee(FailureStatus::InsufficientSpace);
        }
        Ok(_) => {}
        Err(e) => {
            tracing::error!(dir = %auto_dir.display(), "sauvegarde automatique : espace libre : {e}");
            return Ecriture::Refusee(FailureStatus::Failed);
        }
    }

    let chemin = auto_dir.join(&nom);
    let ecrit = write_backup_file(&chemin, |fichier| {
        let ecrivain = environnement.wrap_writer(fichier);
        write_keshbackup(&export, &documents_dir, ecrivain).map(|(_, meta)| meta)
    });
    let meta = match ecrit {
        Ok(meta) => meta,
        Err(e) => {
            let statut = classify_write_error(&e.source);
            tracing::error!(?statut, "sauvegarde automatique : écriture : {e}");
            return Ecriture::Refusee(statut);
        }
    };
    if let Err(e) = relire(&chemin) {
        tracing::error!(path = %chemin.display(), "sauvegarde automatique illisible, supprimée : {e}");
        if let Err(e) = std::fs::remove_file(&chemin) {
            tracing::warn!(path = %chemin.display(), "sauvegarde illisible non supprimée : {e}");
        }
        return Ecriture::Refusee(FailureStatus::Failed);
    }
    Ecriture::Ecrite { nom, meta }
}

/// AC26 à AC30 — une sauvegarde automatique, **sans** la condition [`is_due`]
/// (la boucle de fond la vérifie avant d'appeler ; « Sauvegarder maintenant »
/// ne la vérifie pas).
///
/// 1. Prend `backup_lock` en `try_lock_owned` : pris → [`RunOutcome::Skipped`],
///    rien d'écrit ;
/// 2. lit le réglage, enregistre la tentative (`last_attempt_at = now`) ;
/// 3. lit la base et écrit l'archive ([`build_keshbackup`], un seul
///    `spawn_blocking`) : seuil d'espace, écriture, relecture ;
/// 4. enregistre le statut ; après un succès (`ok` ou `partial`) seulement,
///    applique la rétention.
///
/// Le garde du verrou est tenu jusqu'à la fin, rétention comprise. Erreur :
/// seule une lecture ou une écriture de l'état en base échoue en `Err` ; toute
/// autre défaillance est un statut enregistré.
pub async fn run_once(
    state: &AppState,
    environnement: Arc<dyn BackupEnv>,
    now: NaiveDateTime,
    trigger: Trigger,
) -> Result<RunOutcome, DbError> {
    // À la milliseconde : la précision de `DATETIME(3)` et du nom du fichier,
    // pour que le nom, `last_attempt_at` et `last_success_at` disent le même
    // instant.
    let now = chrono::SubsecRound::trunc_subsecs(now, 3);
    let Ok(_garde) = state.backup_lock.clone().try_lock_owned() else {
        tracing::info!(?trigger, "tour sauté : une autre sauvegarde est en cours");
        return Ok(RunOutcome::Skipped);
    };

    let reglage = backup_schedule::get(&state.pool).await?;
    backup_schedule::record_attempt(&state.pool, now).await?;

    let backup_dir = PathBuf::from(&state.config.admin_backup_dir);
    let auto_dir = backup_dir.join(AUTO_SUBDIR);
    let nom = nom_de_sauvegarde(now);
    let derniere_taille = reglage
        .last_success_bytes
        .and_then(|b| u64::try_from(b).ok())
        .unwrap_or(0);
    let documents_dir = PathBuf::from(&state.config.documents_dir);
    let (auto, nom_ecrit) = (auto_dir.clone(), nom.clone());
    let remplir = move |export: InstallationExport, documents: PathBuf| {
        Ok(ecrire(
            export,
            documents,
            auto,
            nom_ecrit,
            derniere_taille,
            environnement,
        ))
    };
    let ecriture = match build_keshbackup(&state.pool, documents_dir, remplir).await {
        Ok(e) => e,
        Err(e) => {
            tracing::error!(?trigger, "sauvegarde automatique : {e:?}");
            Ecriture::Refusee(FailureStatus::Failed)
        }
    };

    let statut = match ecriture {
        Ecriture::Refusee(echec) => {
            let statut = RunStatus::Failure(echec);
            backup_schedule::record_failure(&state.pool, statut.as_backup_status()).await?;
            tracing::error!(?trigger, ?echec, "sauvegarde automatique en échec");
            statut
        }
        Ecriture::Ecrite { nom, meta } => {
            let manquantes = i32::try_from(meta.files_not_included).unwrap_or(i32::MAX);
            backup_schedule::record_success(
                &state.pool,
                &BackupSuccess {
                    at: now,
                    file: nom.clone(),
                    bytes: i64::try_from(meta.byte_size).unwrap_or(i64::MAX),
                    files_not_included: manquantes,
                },
            )
            .await?;
            let statut = if manquantes == 0 {
                tracing::info!(?trigger, file = %nom, bytes = meta.byte_size, "sauvegarde automatique écrite");
                SuccessStatus::Ok
            } else {
                tracing::warn!(
                    ?trigger,
                    file = %nom,
                    pieces_non_incluses = manquantes,
                    "sauvegarde automatique écrite, incomplète"
                );
                SuccessStatus::Partial
            };
            let complete = match statut {
                SuccessStatus::Ok => Some(nom.clone()),
                SuccessStatus::Partial => reglage.last_complete_file.clone(),
            };
            let garder = usize::try_from(reglage.retention_count).unwrap_or(1).max(1);
            let supprimees = tokio::task::spawn_blocking(move || {
                let mut proteges = vec![nom.as_str()];
                if let Some(c) = complete.as_deref() {
                    proteges.push(c);
                }
                appliquer_la_retention(&auto_dir, garder, &proteges)
            })
            .await
            .unwrap_or_else(|e| {
                tracing::warn!("rétention interrompue : {e}");
                0
            });
            if supprimees > 0 {
                tracing::info!(supprimees, "rétention des sauvegardes automatiques");
            }
            RunStatus::Success(statut)
        }
    };
    Ok(RunOutcome::Done(statut))
}

/// AC26 — lance la tâche de fond : premier contrôle 5 minutes après le
/// démarrage, puis toutes les 5 minutes (`MissedTickBehavior::Skip`). Chaque
/// tour s'exécute dans son propre `tokio::spawn` : une panique y est
/// journalisée et n'arrête pas la boucle. Appelée par `main.rs` **seulement**
/// (ni `build_router`, ni les tests : elle écrirait dans chaque test).
pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut cadence = tokio::time::interval_at(tokio::time::Instant::now() + CADENCE, CADENCE);
        cadence.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            cadence.tick().await;
            let etat = state.clone();
            if let Err(e) = tokio::spawn(async move { un_tour(&etat).await }).await {
                tracing::error!("tour de sauvegarde automatique interrompu : {e}");
            }
        }
    });
}

/// Un tour de la boucle de fond.
async fn un_tour(state: &AppState) {
    let reglage = match backup_schedule::get(&state.pool).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("sauvegarde automatique : lecture du réglage : {e}");
            return;
        }
    };
    let now = Utc::now().naive_utc();
    if reglage.last_success_at.is_some_and(|t| t > now)
        || reglage.last_attempt_at.is_some_and(|t| t > now)
    {
        tracing::warn!(
            "sauvegarde automatique : horloge en arrière (dernière date enregistrée dans le futur)"
        );
    }
    if !is_due(&reglage, now) {
        return;
    }
    match run_once(state, Arc::new(RealBackupEnv), now, Trigger::Schedule).await {
        Ok(RunOutcome::Skipped) => {}
        Ok(RunOutcome::Done(statut)) => tracing::debug!(?statut, "tour de sauvegarde terminé"),
        Err(e) => tracing::error!("sauvegarde automatique : état en base : {e}"),
    }
}

#[cfg(test)]
mod tests {
    //! Story 31-1c — fonctions pures (AC25, AC29, AC30) et gardes lexicales
    //! (AC26, AC27).

    use super::*;
    use chrono::NaiveDate;

    fn t(h: u32, m: u32, s: u32, ms: u32) -> NaiveDateTime {
        NaiveDate::from_ymd_opt(2026, 10, 24)
            .unwrap()
            .and_hms_milli_opt(h, m, s, ms)
            .unwrap()
    }

    fn reglage(
        enabled: bool,
        interval_hours: i32,
        last_success_at: Option<NaiveDateTime>,
        last_attempt_at: Option<NaiveDateTime>,
    ) -> BackupSchedule {
        BackupSchedule {
            enabled,
            interval_hours,
            retention_count: 7,
            last_attempt_at,
            last_success_at,
            last_success_file: None,
            last_success_bytes: None,
            last_status: None,
            last_files_not_included: None,
            last_complete_file: None,
            version: 1,
        }
    }

    // --- is_due (AC25) ----------------------------------------------------

    #[test]
    fn is_due_jamais_tente() {
        assert!(is_due(&reglage(true, 24, None, None), t(10, 0, 0, 0)));
    }

    #[test]
    fn is_due_desactive() {
        assert!(!is_due(&reglage(false, 24, None, None), t(10, 0, 0, 0)));
    }

    /// Égalité exacte de l'intervalle → dû (tue M-c1).
    #[test]
    fn is_due_a_l_egalite() {
        let s = reglage(true, 6, Some(t(4, 0, 0, 0)), Some(t(4, 0, 0, 0)));
        assert!(is_due(&s, t(10, 0, 0, 0)));
    }

    /// Une milliseconde avant l'intervalle → pas dû.
    #[test]
    fn is_due_une_milliseconde_avant() {
        let s = reglage(true, 6, Some(t(4, 0, 0, 1)), Some(t(4, 0, 0, 1)));
        assert!(!is_due(&s, t(10, 0, 0, 0)));
    }

    /// Un échec il y a 10 minutes (réussite ancienne) → pas dû : on retente au
    /// bout d'une heure (tue M-c2).
    #[test]
    fn is_due_echec_il_y_a_dix_minutes() {
        let s = reglage(
            true,
            24,
            Some(t(0, 0, 0, 0) - TimeDelta::days(2)),
            Some(t(9, 50, 0, 0)),
        );
        assert!(!is_due(&s, t(10, 0, 0, 0)));
    }

    /// … et dû une heure après l'échec.
    #[test]
    fn is_due_echec_il_y_a_une_heure() {
        let s = reglage(
            true,
            24,
            Some(t(0, 0, 0, 0) - TimeDelta::days(2)),
            Some(t(9, 0, 0, 0)),
        );
        assert!(is_due(&s, t(10, 0, 0, 0)));
    }

    /// Dernière réussite dans le futur (horloge revenue en arrière) → dû
    /// (tue M-c22).
    #[test]
    fn is_due_derniere_reussite_dans_le_futur() {
        let s = reglage(true, 24, Some(t(23, 0, 0, 0)), Some(t(8, 0, 0, 0)));
        assert!(is_due(&s, t(10, 0, 0, 0)));
    }

    /// Dernière tentative dans le futur → dû (tue M-c22).
    #[test]
    fn is_due_derniere_tentative_dans_le_futur() {
        let s = reglage(true, 24, None, Some(t(23, 0, 0, 0)));
        assert!(is_due(&s, t(10, 0, 0, 0)));
    }

    /// `next_due_at` suit la même règle (R6 de la P5) : réussite future et
    /// tentative récente → l'heure de reprise, pas « maintenant ».
    #[test]
    fn next_due_at_meme_regle_qu_is_due() {
        let now = t(10, 0, 0, 0);
        let s = reglage(true, 24, Some(t(23, 0, 0, 0)), Some(t(9, 50, 0, 0)));
        assert_eq!(next_due_at(&s, now), Some(t(10, 50, 0, 0)));
        assert!(!is_due(&s, now));
        let apres = reglage(true, 24, Some(t(9, 0, 0, 0)), Some(t(9, 0, 0, 0)));
        assert_eq!(
            next_due_at(&apres, now),
            Some(t(9, 0, 0, 0) + TimeDelta::hours(24))
        );
        assert_eq!(next_due_at(&reglage(false, 24, None, None), now), None);
        assert_eq!(next_due_at(&reglage(true, 24, None, None), now), Some(now));
    }

    // --- is_overdue (AC25, AC31) -------------------------------------------

    #[test]
    fn is_overdue_deux_intervalles_moins_une_milliseconde() {
        let s = reglage(true, 6, Some(t(10, 0, 0, 1) - TimeDelta::hours(12)), None);
        assert!(!is_overdue(&s, t(10, 0, 0, 0)));
    }

    /// Égalité de deux intervalles → en retard (tue M-c9).
    #[test]
    fn is_overdue_a_l_egalite() {
        let s = reglage(true, 6, Some(t(10, 0, 0, 0) - TimeDelta::hours(12)), None);
        assert!(is_overdue(&s, t(10, 0, 0, 0)));
    }

    /// Un seul intervalle dépassé → pas en retard (tue M-c9).
    #[test]
    fn is_overdue_un_intervalle() {
        let s = reglage(true, 6, Some(t(10, 0, 0, 0) - TimeDelta::hours(7)), None);
        assert!(!is_overdue(&s, t(10, 0, 0, 0)));
    }

    /// Réussite dans le futur → en retard (tue M-c22).
    #[test]
    fn is_overdue_dans_le_futur() {
        let s = reglage(true, 24, Some(t(23, 0, 0, 0)), None);
        assert!(is_overdue(&s, t(10, 0, 0, 0)));
    }

    #[test]
    fn is_overdue_sans_reussite() {
        assert!(!is_overdue(
            &reglage(true, 6, None, Some(t(1, 0, 0, 0))),
            t(10, 0, 0, 0)
        ));
    }

    #[test]
    fn is_overdue_desactive() {
        let s = reglage(false, 6, Some(t(0, 0, 0, 0) - TimeDelta::days(9)), None);
        assert!(!is_overdue(&s, t(10, 0, 0, 0)));
    }

    // --- classify_write_error (AC30) ----------------------------------------

    #[test]
    fn classify_storage_full() {
        let e = io::Error::from(io::ErrorKind::StorageFull);
        assert_eq!(classify_write_error(&e), FailureStatus::DiskFull);
    }

    #[test]
    fn classify_quota_exceeded() {
        let e = io::Error::from(io::ErrorKind::QuotaExceeded);
        assert_eq!(classify_write_error(&e), FailureStatus::DiskFull);
    }

    #[test]
    fn classify_code_brut_28() {
        let e = io::Error::from_raw_os_error(28);
        assert_eq!(classify_write_error(&e), FailureStatus::DiskFull);
    }

    #[test]
    fn classify_permission_refusee() {
        let e = io::Error::from(io::ErrorKind::PermissionDenied);
        assert_eq!(classify_write_error(&e), FailureStatus::Failed);
    }

    /// Une `ZipError` convertie garde son `kind()` (le cas que la règle
    /// « `kind()` d'abord » vise).
    #[test]
    fn classify_zip_error_convertie() {
        let zip_err = zip::result::ZipError::Io(io::Error::from(io::ErrorKind::StorageFull));
        let e = io::Error::from(zip_err);
        assert_eq!(classify_write_error(&e), FailureStatus::DiskFull);
    }

    // --- seuil, noms ---------------------------------------------------------

    #[test]
    fn seuil_prend_le_plus_grand_des_deux() {
        assert_eq!(seuil_d_espace(0, 0), MARGE_FIXE);
        assert_eq!(seuil_d_espace(1000, 0), 1200 + MARGE_FIXE);
        assert_eq!(seuil_d_espace(10, 5000), 6000 + MARGE_FIXE);
    }

    #[test]
    fn nom_a_la_milliseconde_et_conforme() {
        let nom = nom_de_sauvegarde(t(1, 2, 3, 45));
        assert_eq!(nom, "kesh-auto-20261024T010203045Z.keshbackup");
        assert!(NOM_AUTO.is_match(&nom));
        assert!(!NOM_AUTO.is_match("kesh-auto-20261024T010203045Z.keshbackup.partial"));
        assert!(!NOM_AUTO.is_match("kesh-auto-２０261024T010203045Z.keshbackup"));
        assert!(NOM_PRE_IMPORT.is_match("kesh-pre-import-20261024T010203045-42-0.keshbackup"));
    }

    // --- fichiers (revue de code P1) -----------------------------------------

    /// Date la modification de `chemin` à `maintenant ± secondes`.
    fn dater(chemin: &Path, futur: bool, secondes: u64) {
        let ecart = std::time::Duration::from_secs(secondes);
        let maintenant = std::time::SystemTime::now();
        File::options()
            .write(true)
            .open(chemin)
            .unwrap()
            .set_modified(if futur {
                maintenant + ecart
            } else {
                maintenant - ecart
            })
            .unwrap();
    }

    /// E-3 (C-31-1-90) — un `.partial` dont la date de modification s'écarte
    /// de `maintenant` d'au moins dix minutes, **dans un sens ou dans l'autre**,
    /// est abandonné : une heure dans le futur (horloge revenue en arrière)
    /// est supprimé ; une minute dans le futur ou dans le passé reste.
    #[test]
    fn partiel_date_du_futur() {
        let dir = tempfile::tempdir().unwrap();
        let noms = [
            (
                "kesh-auto-20260101T000000000Z.keshbackup.partial",
                true,
                3600,
            ),
            ("kesh-auto-20260102T000000000Z.keshbackup.partial", true, 60),
            (
                "kesh-auto-20260103T000000000Z.keshbackup.partial",
                false,
                60,
            ),
        ];
        for (nom, futur, secondes) in noms {
            let chemin = dir.path().join(nom);
            std::fs::write(&chemin, b"x").unwrap();
            dater(&chemin, futur, secondes);
        }
        assert_eq!(supprimer_les_partiels(dir.path(), "kesh-auto-"), 1);
        let mut restants: Vec<String> = std::fs::read_dir(dir.path())
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        restants.sort();
        assert_eq!(restants, vec![noms[1].0.to_string(), noms[2].0.to_string()]);
    }

    /// L-B1 / E-5 (C-31-1-89) — l'ouverture d'une sauvegarde listée ne suit
    /// pas un lien symbolique et tient pour absent un fichier disparu ou qui
    /// n'est plus un fichier régulier (`Ok(None)` → 404) ; un fichier régulier
    /// s'ouvre et se lit.
    #[test]
    fn ouverture_sans_suivre_les_liens() {
        let dir = tempfile::tempdir().unwrap();
        let regulier = dir.path().join("a.keshbackup");
        std::fs::write(&regulier, b"archive").unwrap();
        let lien = dir.path().join("b.keshbackup");
        std::os::unix::fs::symlink(&regulier, &lien).unwrap();
        let sous_dossier = dir.path().join("c.keshbackup");
        std::fs::create_dir(&sous_dossier).unwrap();

        let mut lu = String::new();
        ouvrir_la_sauvegarde(&regulier)
            .unwrap()
            .expect("fichier régulier ouvert")
            .read_to_string(&mut lu)
            .unwrap();
        assert_eq!(lu, "archive");
        assert!(ouvrir_la_sauvegarde(&lien).unwrap().is_none(), "lien suivi");
        assert!(
            ouvrir_la_sauvegarde(&dir.path().join("absent.keshbackup"))
                .unwrap()
                .is_none(),
            "fichier disparu"
        );
        assert!(
            ouvrir_la_sauvegarde(&sous_dossier).unwrap().is_none(),
            "dossier ouvert"
        );
    }

    // --- gardes lexicales ----------------------------------------------------

    /// Lignes de code d'un fichier, commentaires `//` écartés, tronqué au
    /// premier `#[cfg(test)]`.
    fn code(source: &str) -> String {
        source
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// AC26 (M-c21) — `auto_backup::spawn(` apparaît exactement une fois dans
    /// `main.rs` (lignes `//` écartées : un appel mis en commentaire ne compte
    /// pas) et aucune dans `lib.rs`, où il se déclencherait dans chaque test
    /// monté par `build_router`.
    #[test]
    fn lancement_par_main_seulement() {
        let main = code(include_str!("main.rs"));
        let lib = code(include_str!("lib.rs"));
        assert_eq!(
            main.matches("auto_backup::spawn(").count(),
            1,
            "main.rs doit lancer la sauvegarde automatique, une fois"
        );
        assert_eq!(
            lib.matches("auto_backup::spawn(").count(),
            0,
            "lib.rs ne doit pas lancer la tâche (elle tournerait dans les tests)"
        );
        // Assertion de montage : la troncature écarte bien les commentaires.
        assert_eq!(
            code("// auto_backup::spawn(x)\nfn f() {}")
                .matches("auto_backup::spawn(")
                .count(),
            0
        );
    }

    /// Extrait l'argument parenthésé qui suit la première occurrence de
    /// `appel` dans `texte` (parenthèses appariées).
    fn argument_de(texte: &str, appel: &str) -> String {
        let debut = texte
            .find(appel)
            .unwrap_or_else(|| panic!("{appel} introuvable"))
            + appel.len()
            - 1;
        let mut profondeur = 0usize;
        for (i, c) in texte[debut..].char_indices() {
            match c {
                '(' => profondeur += 1,
                ')' => {
                    profondeur -= 1;
                    if profondeur == 0 {
                        return texte[debut..debut + i + 1].to_string();
                    }
                }
                _ => {}
            }
        }
        panic!("parenthèse fermante de {appel} introuvable")
    }

    /// Corps de la fonction `signature` (jusqu'à la première ligne `}`).
    fn corps(texte: &str, signature: &str) -> String {
        let debut = texte
            .find(signature)
            .unwrap_or_else(|| panic!("{signature} introuvable"));
        let fin = debut + texte[debut..].find("\n}").expect("fin de fonction");
        texte[debut..fin].to_string()
    }

    /// AC27 (M-c18, M-c24) — garde LEXICALE des gardes possédés, contre
    /// l'inattention (une variable intermédiaire la contourne) :
    /// - aucune forme empruntée (`backup_lock.lock(`, `backup_lock.try_lock(`)
    ///   dans `routes/admin.rs`, `routes/backup_schedule.rs` ni ce fichier ;
    /// - `try_lock_owned` dans ce fichier seulement ; la route de « Sauvegarder
    ///   maintenant » ne touche pas le verrou ;
    /// - dans `full_import`, `lock_owned` se trouve dans la fermeture du
    ///   `tokio::spawn` ;
    /// - dans `full_export`, le garde est pris avant la fermeture `remplir`,
    ///   nommée avant l'appel de `build_keshbackup`, et il est nommé **dans**
    ///   cette fermeture (il y est déplacé, donc vit dans le `spawn_blocking`).
    #[test]
    fn gardes_possedes() {
        let admin = code(include_str!("routes/admin.rs"));
        let planif = code(include_str!("routes/backup_schedule.rs"));
        let ici = code(include_str!("auto_backup.rs"));
        for (nom, texte) in [
            ("routes/admin.rs", &admin),
            ("routes/backup_schedule.rs", &planif),
            ("auto_backup.rs", &ici),
        ] {
            for interdit in ["backup_lock.lock(", "backup_lock.try_lock("] {
                assert!(
                    !texte.contains(interdit),
                    "{nom} : garde emprunté `{interdit}`"
                );
            }
        }
        assert!(
            ici.contains("try_lock_owned()"),
            "run_once prend le verrou en try_lock_owned"
        );
        assert!(
            !admin.contains("try_lock_owned"),
            "routes/admin.rs n'emploie pas try_lock_owned"
        );
        assert!(
            !planif.contains("backup_lock"),
            "la route de planification ne touche pas le verrou"
        );

        let import = corps(&admin, "pub async fn full_import(");
        let tache = argument_de(&import, "tokio::spawn(");
        assert_eq!(
            import.matches("lock_owned()").count(),
            1,
            "un seul garde dans full_import"
        );
        assert!(
            tache.contains("backup_lock.clone().lock_owned()"),
            "full_import : garde pris dans la tâche"
        );

        let export = corps(&admin, "pub async fn full_export(");
        let prise = export
            .find("backup_lock.clone().lock_owned()")
            .expect("full_export prend le verrou");
        let ouverture = export
            .find("let remplir =")
            .expect("fermeture remplir nommée");
        let appel = export
            .find("build_keshbackup(")
            .expect("appel de build_keshbackup");
        assert!(
            prise < ouverture && ouverture < appel,
            "ordre : garde, fermeture, appel"
        );
        let fermeture = &export[ouverture..appel];
        let nom_du_garde = export[..prise]
            .rsplit("let ")
            .next()
            .and_then(|s| s.split([' ', '=']).next())
            .expect("nom du garde")
            .to_string();
        assert!(
            fermeture.contains(&nom_du_garde),
            "full_export : le garde `{nom_du_garde}` doit être déplacé dans la fermeture remplir"
        );
    }
}
