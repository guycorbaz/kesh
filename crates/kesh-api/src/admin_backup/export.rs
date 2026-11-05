//! Story 17-3a — Cœur de l'export complet d'installation (`.keshbackup`).
//!
//! [`build_keshbackup`] produit une sauvegarde **sans émettre d'audit** :
//! l'audit `admin.full_export` est ajouté par le handler `routes::admin`, et
//! la sauvegarde pré-import (17-3c) réutilise ce cœur sans auditer.
//!
//! Depuis la Story 28-3 (#633), la base est lue par **un seul** appel,
//! `kesh_db::backup::lire_installation` (`export_installation` jusqu'à la
//! 0.17.3), qui lit toutes les tables dans un instantané commun
//! (`kesh_db::snapshot::ReadSnapshot`) : le fichier photographie la base à un
//! seul instant, et une saisie concurrente ne peut plus y laisser une ligne
//! sans son parent. Depuis la story v0174-mise-a-jour (#847), l'appelant dit
//! l'inventaire à lire (celui du binaire, ou celui du schéma présent pour la
//! sauvegarde avant mise à jour).
//!
//! Story 31-1a (#503) — **format 2** et **assemblage hors mémoire**, en deux
//! temps et deux seulement (AC4) :
//!
//! 1. [`build_keshbackup`] lit la base (asynchrone, l'instantané ci-dessus,
//!    fichiers référencés compris) ;
//! 2. puis, dans **un seul** `spawn_blocking`, la fermeture `remplir` propre à
//!    chaque appelant écrit l'archive par [`write_keshbackup`] sur son
//!    écrivain (temporaire débordant sur disque pour l'export, fichier
//!    `.partial` pour la sauvegarde pré-import) : tables, puis pièces du
//!    dossier des documents **une par une**, jamais toutes en mémoire.
//!
//! Structure du ZIP (format 2) :
//! ```text
//! data/<table>.ndjson        # une par table de l'inventaire lu
//! files/                     # entrée dossier, toujours écrite, avant les pièces
//! files/<sha256>.<ext>       # une par pièce référencée présente et intacte (Stored)
//! manifest.json              # au ROOT, en dernier (files, filesNotIncluded)
//! ```
//!
//! Angles morts assumés : les fichiers du dossier des documents que la base ne
//! référence pas ne sont pas emportés ; les pièces sont lues **hors** de
//! l'instantané (elles sont nommées par leur contenu et jamais réécrites, si
//! bien qu'une pièce lue est celle que la ligne désigne) ; une archive de plus
//! de 4 Gio n'a pas été mesurée.

use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use regex::Regex;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::admin_backup::manifest::{
    BackupFileMeta, BackupManifest, BackupTableMeta, FileNotIncluded, NotIncludedReason,
    build_backup_manifest_json,
};
use crate::document_storage::{ReadDocumentError, is_safe_storage_path, open_document};
use crate::errors::AppError;
use crate::exports::metadata::sha256_hex;

use kesh_db::backup::{InstallationExport, Inventaire, ReferencedFile, lire_installation};

/// Métadonnées retournées au handler (audit + tracing).
#[derive(Debug, Clone, Copy)]
pub struct KeshBackupMeta {
    /// Taille de l'archive, en octets.
    pub byte_size: u64,
    pub table_count: usize,
    pub total_rows: usize,
    /// Story 31-1a (AC5) — pièces emportées sous `files/`.
    pub file_count: usize,
    /// Story 31-1a (AC5) — somme de leurs tailles.
    pub files_bytes: u64,
    /// Story 31-1a (AC5) — pièces référencées non emportées.
    pub files_not_included: usize,
}

/// Story 31-1a (AC4) — échec d'**écriture** de l'archive.
///
/// Conserve l'`io::Error` d'origine (une `ZipError` y entre par
/// `io::Error::from`, jamais par `format!`) : la sauvegarde automatique de la
/// 31-1c classe `disk_full` sur son `kind()`. Une erreur de **lecture** d'une
/// pièce n'en est pas une : la pièce est alors listée `unreadable` et la
/// sauvegarde continue (AC3).
#[derive(Debug)]
pub struct BackupWriteError {
    /// Étape fautive (journal seulement).
    pub stage: String,
    pub source: io::Error,
}

impl BackupWriteError {
    pub fn new(stage: impl Into<String>, source: impl Into<io::Error>) -> Self {
        Self {
            stage: stage.into(),
            source: source.into(),
        }
    }
}

impl std::fmt::Display for BackupWriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} : {}", self.stage, self.source)
    }
}

impl std::error::Error for BackupWriteError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Story 31-1a (AC4, C-31-1-60) — échec de [`build_keshbackup`] : lecture de
/// la base, écriture de l'archive (l'`io::Error` conservée), ou tâche
/// d'assemblage interrompue (panique).
#[derive(Debug)]
pub enum BackupError {
    Db(kesh_db::errors::DbError),
    Write(BackupWriteError),
    Task(String),
}

impl From<BackupError> for AppError {
    /// Tout échec de sauvegarde est un [`AppError::AdminFullExportFailed`]
    /// (détail au journal, jamais au client). L'import le convertit à l'appel
    /// par `routes::admin::avant_sauvegarde`.
    fn from(e: BackupError) -> Self {
        match e {
            BackupError::Db(e) => AppError::AdminFullExportFailed(format!("db: {e}")),
            BackupError::Write(e) => AppError::AdminFullExportFailed(format!("écriture : {e}")),
            BackupError::Task(e) => AppError::AdminFullExportFailed(format!("tâche : {e}")),
        }
    }
}

/// Produit une sauvegarde, **sans audit** : lit la base par l'unique
/// `lire_installation(pool, inventaire)` — un seul instantané, clos avant
/// l'écriture (Story 28-3, #633) —, puis exécute `remplir(export,
/// documents_dir)` dans **un seul** `tokio::task::spawn_blocking` et rend ce
/// qu'il rend.
///
/// `remplir` appelle [`write_keshbackup`] sur l'écrivain de l'appelant ; il
/// **possède** tout ce qu'il touche (aucun emprunt n'entre dans la tâche).
/// C'est le **seul** lecteur de la base de la sauvegarde dans `kesh-api` :
/// les gardes `tests::build_keshbackup_ne_lit_la_base_qu_une_fois` et
/// `tests::chaque_lecture_n_a_qu_un_appelant` y tiennent.
///
/// Story v0174-mise-a-jour (#847) — `inventaire` dit quelles tables lire :
/// [`Inventaire::Binaire`] pour l'export, la sauvegarde automatique et la
/// sauvegarde pré-import ; [`Inventaire::SchemaPresent`] pour la sauvegarde
/// avant mise à jour (`crate::pre_upgrade_backup`), prise sur une base que les
/// migrations du binaire n'ont pas encore touchée.
///
/// Erreurs : [`BackupError`]. ⚠️ L'import l'appelle pour **produire** la
/// sauvegarde pré-import : il convertit l'échec par
/// `routes::admin::avant_sauvegarde` en `AdminPreImportBackupFailed`
/// (Story 15-13b, #576) ; l'export le rend en `AdminFullExportFailed`.
pub async fn build_keshbackup<T, F>(
    pool: &MySqlPool,
    inventaire: Inventaire,
    documents_dir: PathBuf,
    remplir: F,
) -> Result<T, BackupError>
where
    F: FnOnce(InstallationExport, PathBuf) -> Result<T, BackupWriteError> + Send + 'static,
    T: Send + 'static,
{
    let export = lire_installation(pool, inventaire)
        .await
        .map_err(BackupError::Db)?;
    tokio::task::spawn_blocking(move || remplir(export, documents_dir))
        .await
        .map_err(|e| BackupError::Task(e.to_string()))?
        .map_err(BackupError::Write)
}

/// Story 31-1a (AC1, AC6) — forme d'un nom de pièce : `<sha256>.<ext>`, en
/// minuscules. `[0-9a-f]` et non `\d` : `\d` est Unicode dans le crate `regex`.
static NOM_DE_PIECE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[0-9a-f]{64}\.[a-z0-9]{1,8}$").expect("expression valide"));

/// Story 31-1a (AC1, AC6) — `true` si `path` satisfait **les deux** règles
/// d'un nom de pièce : l'expression `^[0-9a-f]{64}\.[a-z0-9]{1,8}$` et
/// `is_safe_storage_path`. Partagée par l'export et l'import.
pub(crate) fn nom_de_piece_valide(path: &str) -> bool {
    NOM_DE_PIECE.is_match(path) && is_safe_storage_path(path)
}

/// Story 31-1a (AC1) — partage des fichiers référencés : ceux à lire, et les
/// chemins en **anomalie**, listés `mismatch` sans lecture.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct Classement {
    pub a_lire: Vec<ReferencedFile>,
    pub incoherents: Vec<String>,
}

/// Story 31-1a (AC1) — classe les couples (chemin, empreinte) lus en base,
/// **par égalité d'octets** :
///
/// - (ii) un même chemin sous deux empreintes : anomalie, aucune n'est
///   choisie ;
/// - (iii) un chemin qui ne satisfait pas l'une des deux règles de
///   [`nom_de_piece_valide`] : anomalie ;
/// - (i) un préfixe qui n'est pas l'empreinte déclarée par la ligne :
///   anomalie ;
/// - sinon, la pièce est à lire. Deux chemins de même empreinte sont deux
///   pièces.
///
/// Chaque chemin en anomalie apparaît une fois, tel quel.
pub(crate) fn classer_les_pieces(refs: &[ReferencedFile]) -> Classement {
    let mut par_chemin: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for r in refs {
        par_chemin
            .entry(r.storage_path.as_str())
            .or_default()
            .insert(r.sha256.as_str());
    }
    let mut classement = Classement::default();
    for (chemin, empreintes) in par_chemin {
        let unique = (empreintes.len() == 1).then(|| *empreintes.iter().next().unwrap_or(&""));
        match unique {
            Some(sha) if nom_de_piece_valide(chemin) && chemin.split('.').next() == Some(sha) => {
                classement.a_lire.push(ReferencedFile {
                    storage_path: chemin.to_string(),
                    sha256: sha.to_string(),
                });
            }
            _ => classement.incoherents.push(chemin.to_string()),
        }
    }
    classement
}

/// Écrit le `.keshbackup` complet sur `w` (synchrone, appelé dans le
/// `spawn_blocking` de [`build_keshbackup`]) et rend l'écrivain et les
/// métadonnées.
///
/// Les pièces sont lues **une seule fois** (C-31-1-53) : ouverture par
/// [`open_document`] (`NotFound` → `missing`, autre erreur → `unreadable`,
/// aucune entrée commencée), taille lue sur le descripteur ouvert, entrée
/// `files/<nom>` commencée (`Stored`, `0644`), puis copie à la main à travers
/// `take(taille + 1)` et un `Sha256`. Empreinte ou taille différentes →
/// l'entrée est **retirée** (`abort_file`) et la pièce listée `altered` ; erreur
/// de lecture pendant la copie → retirée, `unreadable`. Seule une erreur
/// d'**écriture** de l'archive (ou d'`abort_file`) abandonne la sauvegarde.
/// Aucune entrée fausse n'entre donc dans l'archive.
pub fn write_keshbackup<W: Write + Seek>(
    export: &InstallationExport,
    documents_dir: &Path,
    w: W,
) -> Result<(W, KeshBackupMeta), BackupWriteError> {
    let mut tables_meta: BTreeMap<String, BackupTableMeta> = BTreeMap::new();
    let mut total_rows = 0usize;

    let mut zip = ZipWriter::new(w);
    let opts: SimpleFileOptions = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .unix_permissions(0o644);

    // Une entrée NDJSON par table (ordre de l'inventaire lu).
    for (table, table_export) in &export.tables {
        total_rows += table_export.row_count;
        tables_meta.insert(
            table.to_string(),
            BackupTableMeta {
                row_count: table_export.row_count,
                sha256: sha256_hex(&table_export.ndjson),
                column_names: table_export.column_names.clone(),
            },
        );
        zip.start_file(format!("data/{table}.ndjson"), opts)
            .map_err(|e| BackupWriteError::new(format!("zip start data/{table}"), e))?;
        zip.write_all(&table_export.ndjson)
            .map_err(|e| BackupWriteError::new(format!("zip write data/{table}"), e))?;
    }

    // L'entrée dossier `files/`, toujours, et avant les pièces : l'import exige
    // sa présence (format 1 et 2), et `abort_file` sait rembobiner derrière
    // elle (elle a un `data_start`).
    zip.add_directory("files/", opts)
        .map_err(|e| BackupWriteError::new("zip add_directory files/", e))?;

    let Classement {
        a_lire,
        incoherents,
    } = classer_les_pieces(&export.referenced_files);
    let mut not_included: Vec<FileNotIncluded> = incoherents
        .into_iter()
        .map(|path| {
            tracing::warn!(
                ?path,
                raison = "mismatch",
                "pièce référencée non sauvegardée"
            );
            FileNotIncluded {
                path,
                reason: NotIncludedReason::Mismatch,
            }
        })
        .collect();
    let mut files: Vec<BackupFileMeta> = Vec::new();
    for piece in &a_lire {
        match copier_piece(&mut zip, documents_dir, piece)? {
            Ok(meta) => files.push(meta),
            Err(reason) => not_included.push(FileNotIncluded {
                path: piece.storage_path.clone(),
                reason,
            }),
        }
    }
    not_included.sort_by(|a, b| a.path.cmp(&b.path));

    let file_count = files.len();
    let files_bytes = files.iter().map(|f| f.byte_size).sum();
    let files_not_included = not_included.len();

    // Manifest en dernier (cohérent lecture humaine + pattern 9-2b).
    let manifest = BackupManifest::new(
        export.min_required.clone(),
        export.instance_id,
        tables_meta,
        files,
        not_included,
    );
    let manifest_bytes = build_backup_manifest_json(&manifest)
        .map_err(|e| BackupWriteError::new("manifest", io::Error::other(e.to_string())))?;
    zip.start_file("manifest.json", opts)
        .map_err(|e| BackupWriteError::new("zip start manifest", e))?;
    zip.write_all(&manifest_bytes)
        .map_err(|e| BackupWriteError::new("zip write manifest", e))?;

    let mut w = zip
        .finish()
        .map_err(|e| BackupWriteError::new("zip finish", e))?;
    let byte_size = w
        .seek(SeekFrom::End(0))
        .map_err(|e| BackupWriteError::new("taille de l'archive", e))?;

    Ok((
        w,
        KeshBackupMeta {
            byte_size,
            // Story v0174-mise-a-jour : les tables écrites, non la liste du
            // binaire — l'inventaire du schéma présent peut être plus court.
            table_count: export.tables.len(),
            total_rows,
            file_count,
            files_bytes,
            files_not_included,
        },
    ))
}

/// Copie une pièce dans l'archive (une passe). `Ok(Err(raison))` : la pièce
/// n'est pas emportée, la sauvegarde continue ; `Err` : l'archive ne s'écrit
/// plus, la sauvegarde est abandonnée.
fn copier_piece<W: Write + Seek>(
    zip: &mut ZipWriter<W>,
    documents_dir: &Path,
    piece: &ReferencedFile,
) -> Result<Result<BackupFileMeta, NotIncludedReason>, BackupWriteError> {
    let path = &piece.storage_path;
    let mut fichier = match open_document(documents_dir, path) {
        Ok(f) => f,
        Err(ReadDocumentError::NotFound) => {
            tracing::warn!(
                ?path,
                raison = "missing",
                "pièce référencée non sauvegardée"
            );
            return Ok(Err(NotIncludedReason::Missing));
        }
        Err(ReadDocumentError::InvalidPath) => {
            // Inatteignable : `classer_les_pieces` a déjà écarté les noms hors forme.
            tracing::warn!(
                ?path,
                raison = "mismatch",
                "pièce référencée non sauvegardée"
            );
            return Ok(Err(NotIncludedReason::Mismatch));
        }
        Err(ReadDocumentError::Io(e)) => {
            tracing::warn!(?path, kind = ?e.kind(), raison = "unreadable", "pièce référencée non sauvegardée");
            return Ok(Err(NotIncludedReason::Unreadable));
        }
    };
    let byte_size = match fichier.metadata() {
        Ok(m) => m.len(),
        Err(e) => {
            tracing::warn!(?path, kind = ?e.kind(), raison = "unreadable", "pièce référencée non sauvegardée");
            return Ok(Err(NotIncludedReason::Unreadable));
        }
    };

    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Stored)
        .unix_permissions(0o644)
        .large_file(byte_size >= u64::from(u32::MAX));
    zip.start_file(format!("files/{path}"), options)
        .map_err(|e| BackupWriteError::new("zip start files/", e))?;

    // Boucle écrite à la main, et non `io::copy` : une erreur de LECTURE de la
    // pièce (la pièce est écartée) ne doit pas se confondre avec une erreur
    // d'ÉCRITURE de l'archive (la sauvegarde est abandonnée).
    let mut hasher = Sha256::new();
    let mut borne = (&mut fichier).take(byte_size.saturating_add(1));
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = match borne.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => {
                tracing::warn!(?path, kind = ?e.kind(), raison = "unreadable", "pièce référencée non sauvegardée");
                zip.abort_file()
                    .map_err(|e| BackupWriteError::new("zip abort_file", e))?;
                return Ok(Err(NotIncludedReason::Unreadable));
            }
        };
        hasher.update(&buf[..n]);
        zip.write_all(&buf[..n])
            .map_err(|e| BackupWriteError::new("zip write files/", e))?;
        total += n as u64;
    }
    let sha256 = format!("{:x}", hasher.finalize());
    if total != byte_size || sha256 != piece.sha256 {
        tracing::warn!(
            ?path,
            raison = "altered",
            "pièce référencée non sauvegardée"
        );
        zip.abort_file()
            .map_err(|e| BackupWriteError::new("zip abort_file", e))?;
        return Ok(Err(NotIncludedReason::Altered));
    }
    Ok(Ok(BackupFileMeta {
        path: path.clone(),
        sha256,
        byte_size,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Corps de production de `build_keshbackup` : code avant la première ligne
    /// `#[cfg(test)]`, lignes `//` (donc `///`) écartées, de la ligne
    /// `pub async fn build_keshbackup` (générique depuis la Story 31-1a : le
    /// motif ne porte plus la parenthèse — R3 de la validation P5) à la
    /// première ligne réduite à `}`. Même découpe que
    /// `routes::admin::tests::code_de_production`.
    fn corps_de_build_keshbackup() -> String {
        let prod: Vec<&str> = include_str!("export.rs")
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect();
        let debut = prod
            .iter()
            .position(|l| l.starts_with("pub async fn build_keshbackup"))
            .expect("assertion de montage : `pub async fn build_keshbackup` introuvable");
        let fin = debut
            + prod[debut..]
                .iter()
                .position(|l| *l == "}")
                .expect("assertion de montage : fin de fonction introuvable");
        assert!(fin > debut + 1, "assertion de montage : corps vide");
        prod[debut..=fin].join("\n")
    }

    /// Story 28-3 (#633), AC5 (g)(ii) — garde lexicale : `build_keshbackup` ne
    /// nomme le pool que deux fois — son paramètre et l'unique
    /// `lire_installation(pool, inventaire)` (`export_installation(pool)`
    /// jusqu'à la 0.17.3, story v0174-mise-a-jour) —, si bien qu'aucune lecture
    /// n'y échappe à l'instantané commun (M10 : une relecture dans un second
    /// instantané compile, et seule cette garde la voit).
    #[test]
    fn build_keshbackup_ne_lit_la_base_qu_une_fois() {
        let corps = corps_de_build_keshbackup();
        let n = regex::Regex::new(r"\bpool\b")
            .expect("regex")
            .find_iter(&corps)
            .count();
        assert_eq!(
            n, 2,
            "`pool` doit apparaître exactement deux fois (paramètre, lire_installation) :\n{corps}"
        );
        // Revue de code P1 (L3) : le second `pool` est bien l'appel
        // `lire_installation(pool, inventaire)` — une autre fonction qui
        // prendrait le pool laisserait le compte à deux.
        assert!(
            corps.contains("lire_installation(pool, inventaire)"),
            "`build_keshbackup` doit lire par `lire_installation(pool, inventaire)` :\n{corps}"
        );
    }

    /// Les `.rs` d'un dossier, récursivement.
    fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap() {
            let p = e.unwrap().path();
            if p.is_dir() {
                sources(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }

    /// Les lignes de code de **production** d'un fichier : vide s'il commence
    /// par `#![cfg(test)]` (fichier de tests, tel
    /// `pre_upgrade_backup/tests.rs` — V5-5 de la story v0174-mise-a-jour),
    /// sinon tout ce qui précède sa première ligne `#[cfg(test)]`, lignes `//`
    /// écartées.
    fn lignes_de_production(texte: &str) -> Vec<&str> {
        if texte
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && !l.starts_with("//"))
            == Some("#![cfg(test)]")
        {
            return Vec::new();
        }
        texte
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect()
    }

    /// Story 31-1a (AC4, C-31-1-56), récrite à la story v0174-mise-a-jour
    /// (#847) — **chacune des deux fonctions de lecture n'a qu'un appelant,
    /// `lire_installation`**, et celle-ci n'en a qu'un, `build_keshbackup` :
    ///
    /// - dans le code de production de `kesh-api` (tous les `.rs` de `src/`,
    ///   [`lignes_de_production`]), `lire_installation(` n'apparaît qu'une fois,
    ///   dans `admin_backup/export.rs`, et ni `export_installation(` ni
    ///   `export_installation_schema_present(` n'apparaissent ;
    /// - dans le code de production de `kesh-db/src/backup.rs`, chacune des
    ///   deux n'est appelée qu'une fois, dans le corps de `lire_installation`.
    ///
    /// Un appelant qui lirait la base lui-même, hors de l'unique point
    /// d'entrée, fait rougir ce test.
    #[test]
    fn chaque_lecture_n_a_qu_un_appelant() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let mut fichiers = Vec::new();
        sources(&src, &mut fichiers);
        let mut sites = Vec::new();
        let mut directs = Vec::new();
        for f in &fichiers {
            let texte = std::fs::read_to_string(f).unwrap();
            let relatif = f.strip_prefix(&src).unwrap().to_string_lossy().to_string();
            for l in lignes_de_production(&texte) {
                for _ in l.matches("lire_installation(") {
                    sites.push(relatif.clone());
                }
                for motif in [
                    "export_installation(",
                    "export_installation_schema_present(",
                ] {
                    if l.contains(motif) {
                        directs.push(format!("{relatif} : {}", l.trim()));
                    }
                }
            }
        }
        assert_eq!(
            sites,
            vec!["admin_backup/export.rs".to_string()],
            "`lire_installation(` ne doit être appelée que par `build_keshbackup`"
        );
        assert!(
            corps_de_build_keshbackup().contains("lire_installation(pool, inventaire)"),
            "l'unique appel doit être celui de `build_keshbackup`"
        );
        assert!(
            directs.is_empty(),
            "lecture de l'installation hors de `lire_installation` :\n{}",
            directs.join("\n")
        );

        // Côté `kesh-db` : les deux lectures ne sont appelées que par
        // `lire_installation`.
        let backup = include_str!("../../../kesh-db/src/backup.rs");
        let prod = lignes_de_production(backup);
        let debut = prod
            .iter()
            .position(|l| l.starts_with("pub async fn lire_installation("))
            .expect("assertion de montage : `lire_installation` introuvable");
        let fin = debut
            + prod[debut..]
                .iter()
                .position(|l| *l == "}")
                .expect("assertion de montage : fin de `lire_installation`");
        for motif in [
            "export_installation(pool)",
            "export_installation_schema_present(pool)",
        ] {
            let appels: Vec<usize> = prod
                .iter()
                .enumerate()
                .filter(|(_, l)| l.contains(motif) && !l.trim_start().starts_with("pub async fn"))
                .map(|(i, _)| i)
                .collect();
            assert_eq!(
                appels.len(),
                1,
                "`{motif}` doit être appelée exactement une fois dans kesh-db/src/backup.rs"
            );
            assert!(
                (debut..=fin).contains(&appels[0]),
                "`{motif}` doit être appelée par `lire_installation` seulement"
            );
        }
    }

    fn piece(chemin: &str, sha: &str) -> ReferencedFile {
        ReferencedFile {
            storage_path: chemin.to_string(),
            sha256: sha.to_string(),
        }
    }

    fn h(c: char) -> String {
        std::iter::repeat_n(c, 64).collect()
    }

    /// AC1 — une pièce cohérente est à lire ; (i) préfixe ≠ empreinte, (ii) un
    /// chemin sous deux empreintes, (iii) des noms hors forme sont des
    /// anomalies, chacune une fois, telles quelles.
    #[test]
    fn classer_les_pieces_isole_les_anomalies() {
        let a = h('a');
        let b = h('b');
        let refs = vec![
            piece(&format!("{a}.pdf"), &a),
            piece(&format!("{a}.png"), &b),
            piece(&format!("{b}.pdf"), &b),
            piece(&format!("{b}.pdf"), &a),
            piece("../x.pdf", &a),
            piece("/etc/hostname", &a),
            piece(&format!("{}.pdf", a.to_uppercase()), &a),
            piece(&format!("{a}.PDF"), &a),
            piece(&format!("{a}.pdf\n"), &a),
        ];
        let c = classer_les_pieces(&refs);
        assert_eq!(c.a_lire, vec![piece(&format!("{a}.pdf"), &a)]);
        let mut attendu = vec![
            format!("{a}.png"),
            format!("{b}.pdf"),
            "../x.pdf".to_string(),
            "/etc/hostname".to_string(),
            format!("{}.pdf", a.to_uppercase()),
            format!("{a}.PDF"),
            format!("{a}.pdf\n"),
        ];
        attendu.sort();
        assert_eq!(c.incoherents, attendu);
    }

    /// AC1 (M-a13) — deux chemins de même empreinte sont deux pièces à lire.
    #[test]
    fn classer_les_pieces_garde_deux_chemins_de_meme_empreinte() {
        let a = h('c');
        let refs = vec![
            piece(&format!("{a}.jpeg"), &a),
            piece(&format!("{a}.jpg"), &a),
        ];
        let c = classer_les_pieces(&refs);
        assert_eq!(c.a_lire, refs);
        assert!(c.incoherents.is_empty());
    }

    /// AC4 (F10) — une erreur de l'archive garde son `kind()` dans
    /// `BackupWriteError` (la 31-1c classe `disk_full` dessus).
    #[test]
    fn backup_write_error_garde_le_kind() {
        let err = BackupWriteError::new(
            "zip",
            zip::result::ZipError::Io(io::Error::from(io::ErrorKind::StorageFull)),
        );
        assert_eq!(err.source.kind(), io::ErrorKind::StorageFull);
    }
}
