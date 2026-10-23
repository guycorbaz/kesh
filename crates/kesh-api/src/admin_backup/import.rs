//! Story 17-3c — Lecture + validation d'un `.keshbackup` à l'import.
//!
//! Toute la validation se fait **avant tout DELETE** (la DB n'est jamais mutée
//! si le backup est rejeté), et **avant** le verrou d'installation et la
//! sauvegarde de sécurité. Story 31-1a (#503) : formats 1 **et** 2, lectures
//! **bornées**, dans cet ordre :
//!
//! 1. `manifest.json` d'abord, lu à travers une borne (`ReadLimits`), puis
//!    désérialisé — le format est alors connu ; `formatVersion ≤ 2` ;
//! 2. inventaire des noms d'entrées **avant toute autre lecture** : format 1,
//!    `manifest.json`, `data/…`, `files/` présent et **vide** ; format 2,
//!    exactement l'ensemble attendu (`manifest.json`, les `data/<table>.ndjson`
//!    du manifeste, `files/`, les `files/<nom>` de `manifest.files`) ;
//! 3. couverture des tables, puis NDJSON à travers une borne **cumulée** pour
//!    l'ensemble des tables, intégrité SHA-256 par table ;
//! 4. format 2 : chaque pièce lue en flux à travers `take(byteSize + 1)` et un
//!    `Sha256` (taille et empreinte), puis **cohérence avec la base
//!    restaurée** : toute pièce emportée est référencée par une ligne de même
//!    empreinte, toute pièce référencée est soit emportée soit déclarée non
//!    incluse.
//!
//! La compatibilité des colonnes avec le schéma destination (AC12c) suit,
//! dans [`check_schema_compat`].
//!
//! ⚠️ Le `.keshbackup` est un **secret** : ne jamais logger son contenu.
//!
//! Angle mort assumé : `ZipArchive::new` décode tout le répertoire central
//! avant que Kesh puisse compter les entrées ; sa mémoire n'est bornée que par
//! la taille reçue (plafond d'import). Les noms en double, que `zip` fusionne
//! à l'ouverture, ne sont pas visibles (seule l'entrée retenue est lue, et
//! elle est vérifiée par son empreinte).

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::io::{Read, Seek};
use std::path::Path;

use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;
use zip::ZipArchive;
use zip::result::ZipError;

use kesh_db::backup::{
    ReferencedFile, TABLES_TO_TRUNCATE, TableRestore, column_constraints, parse_ndjson_rows,
};

use crate::admin_backup::export::nom_de_piece_valide;
use crate::admin_backup::manifest::{BACKUP_FORMAT_VERSION, BackupFileMeta, BackupManifest};
use crate::document_storage::{RestoreOutcome, document_is_intact, restore_document};
use crate::errors::AppError;
use crate::exports::metadata::sha256_hex;

/// Borne de lecture de `manifest.json` (64 Mio).
pub const MANIFEST_MAX_BYTES: u64 = 64 * 1024 * 1024;
/// Borne **cumulée** de lecture des `data/<table>.ndjson` (1 Gio pour toutes).
pub const TABLES_MAX_BYTES: u64 = 1024 * 1024 * 1024;

/// Story 31-1a (AC6b) — bornes des lectures de [`parse_and_verify`]. Des
/// constantes en production ([`ReadLimits::production`]) ; les tests les
/// réduisent (« table trop grande » se teste avec quelques Kio).
#[derive(Debug, Clone, Copy)]
pub struct ReadLimits {
    pub manifest_max_bytes: u64,
    pub tables_max_bytes: u64,
    /// Taille maximale **déclarée** d'une pièce (`byteSize`) : le plafond
    /// d'import, au-delà duquel l'archive ne pourrait de toute façon pas être
    /// reçue.
    pub file_max_bytes: u64,
}

impl ReadLimits {
    /// Bornes de production : [`MANIFEST_MAX_BYTES`], [`TABLES_MAX_BYTES`], et
    /// le plafond d'import `KESH_ADMIN_IMPORT_MAX_MB` pour une pièce.
    pub fn production(import_max_mib: u32) -> Self {
        Self {
            manifest_max_bytes: MANIFEST_MAX_BYTES,
            tables_max_bytes: TABLES_MAX_BYTES,
            file_max_bytes: u64::from(import_max_mib) * 1024 * 1024,
        }
    }
}

/// Backup parsé + vérifié (structure + intégrité), prêt pour le restore.
///
/// Story 31-1a (AC8) : il **porte** l'archive ouverte (`archive`), d'où les
/// pièces sont copiées vers le dossier des documents — le temporaire de
/// réception est anonyme et ne se rouvre pas par son chemin.
pub struct ParsedBackup<R> {
    pub manifest: BackupManifest,
    /// Données par table, dans l'ordre des `column_names` du manifeste.
    pub tables: BTreeMap<String, TableRestore>,
    pub total_rows: usize,
    /// L'archive, `None` une fois prise par l'écriture des pièces.
    pub archive: Option<ZipArchive<R>>,
}

impl<R> std::fmt::Debug for ParsedBackup<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParsedBackup")
            .field("manifest", &self.manifest)
            .field("tables", &self.tables.keys().collect::<Vec<_>>())
            .field("total_rows", &self.total_rows)
            .field("archive", &self.archive.is_some())
            .finish()
    }
}

impl<R> ParsedBackup<R> {
    /// Story 31-1a (AC7, AC9) — les pièces que référence la base **du backup**
    /// (celle que l'import restaure) : couples (chemin, empreinte)
    /// dédoublonnés, triés. Une colonne de référence absente des
    /// `columnNames` d'une table vaut « aucune référence » pour cette table
    /// (`invoices.pdf_storage_path` n'existe pas avant la migration
    /// `20261003000001`).
    pub fn referenced_files(&self) -> Vec<ReferencedFile> {
        let mut couples = BTreeSet::new();
        for (table, chemin, empreinte) in [
            ("imported_supplier_invoices", "storage_path", "file_hash"),
            ("invoices", "pdf_storage_path", "pdf_sha256"),
        ] {
            let Some(t) = self.tables.get(table) else {
                continue;
            };
            let col = |nom: &str| t.column_names.iter().position(|c| c == nom);
            let Some(i_chemin) = col(chemin) else {
                continue;
            };
            let i_empreinte = col(empreinte);
            for row in &t.rows {
                let Some(Value::String(storage_path)) = row.get(i_chemin) else {
                    continue;
                };
                let sha256 = match i_empreinte.and_then(|i| row.get(i)) {
                    Some(Value::String(s)) => s.clone(),
                    _ => String::new(),
                };
                couples.insert(ReferencedFile {
                    storage_path: storage_path.clone(),
                    sha256,
                });
            }
        }
        couples.into_iter().collect()
    }
}

fn invalide(msg: impl Into<String>) -> AppError {
    AppError::InvalidBackupStructure(msg.into())
}

/// Lit une entrée à travers `take(max + 1)` : au-delà de `max` octets,
/// « entrée … trop grande ». La borne ne dépend pas de la taille que déclare
/// l'en-tête ZIP, qui est déclarative.
fn lire_borne(entry: impl Read, nom: &str, max: u64) -> Result<Vec<u8>, AppError> {
    let mut buf = Vec::new();
    entry
        .take(max.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(|e| invalide(format!("lecture {nom} : {e}")))?;
    if buf.len() as u64 > max {
        return Err(invalide(format!("entrée {nom} trop grande")));
    }
    Ok(buf)
}

/// Parse le conteneur ZIP et vérifie structure, intégrité et cohérence
/// (cf. en-tête du module), sans toucher la DB. Générique sur la source : un
/// `File` (le temporaire de réception) en production, un `Cursor` dans les
/// tests. Retourne [`AppError::InvalidBackupStructure`] (→400) sur toute
/// anomalie. Synchrone, et lent sur une grosse archive (lecture disque,
/// SHA-256) : l'appelant l'exécute en `spawn_blocking`.
pub fn parse_and_verify<R: Read + Seek>(
    reader: R,
    limits: &ReadLimits,
) -> Result<ParsedBackup<R>, AppError> {
    let mut zip =
        ZipArchive::new(reader).map_err(|e| invalide(format!("conteneur ZIP illisible : {e}")))?;

    // (1) Le manifeste d'abord, borné.
    let manifest_bytes = match zip.by_name("manifest.json") {
        Ok(entry) => lire_borne(entry, "manifest.json", limits.manifest_max_bytes)?,
        Err(ZipError::FileNotFound) => return Err(invalide("manifest.json absent")),
        Err(e) => return Err(invalide(format!("lecture manifest : {e}"))),
    };
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| invalide(format!("manifest.json illisible : {e}")))?;

    if manifest.format_version > BACKUP_FORMAT_VERSION {
        return Err(invalide(format!(
            "formatVersion {} non supporté (maximum {})",
            manifest.format_version, BACKUP_FORMAT_VERSION
        )));
    }
    let format_2 = manifest.format_version >= 2;

    // (2) Inventaire des noms, avant toute autre lecture.
    let noms: Vec<String> = zip.file_names().map(str::to_string).collect();
    if format_2 {
        verifier_listes_de_pieces(&manifest, limits)?;
        let mut attendus: HashSet<String> = HashSet::new();
        attendus.insert("manifest.json".into());
        attendus.insert("files/".into());
        for t in manifest.tables.keys() {
            attendus.insert(format!("data/{t}.ndjson"));
        }
        for f in &manifest.files {
            attendus.insert(format!("files/{}", f.path));
        }
        for nom in &noms {
            if !attendus.contains(nom) {
                return Err(invalide(format!(
                    "entrée inattendue dans le backup : {nom}"
                )));
            }
        }
        let presents: HashSet<&str> = noms.iter().map(String::as_str).collect();
        if !presents.contains("files/") {
            return Err(invalide("dossier files/ absent du backup"));
        }
        for f in &manifest.files {
            if !presents.contains(format!("files/{}", f.path).as_str()) {
                return Err(invalide(format!(
                    "la pièce {:?} du manifeste est absente de l'archive",
                    f.path
                )));
            }
        }
    } else {
        verifier_format_1(&manifest, &noms)?;
    }

    // Couverture : chaque table applicative attendue doit figurer au manifeste
    // (sinon on ne saurait pas la restaurer correctement).
    for &t in TABLES_TO_TRUNCATE {
        if !manifest.tables.contains_key(t) {
            return Err(invalide(format!("table '{t}' absente du manifeste")));
        }
    }
    // Réciproque (review Pass 1) : aucune table hors inventaire applicatif dans
    // le manifeste — sinon ses lignes seraient comptées dans `total_rows` mais
    // jamais restaurées (table absente de TABLES_TO_TRUNCATE) ⇒ la garde de
    // cohérence du handler lèverait un faux `500` au lieu d'un `400` propre.
    let known: HashSet<&str> = TABLES_TO_TRUNCATE.iter().copied().collect();
    for table in manifest.tables.keys() {
        if !known.contains(table.as_str()) {
            return Err(invalide(format!(
                "table '{table}' présente dans le manifeste mais hors inventaire applicatif"
            )));
        }
    }

    // (3) Intégrité SHA-256 + parsing NDJSON par table (avant tout DELETE),
    // sous une borne cumulée.
    let mut tables = BTreeMap::new();
    let mut total_rows = 0usize;
    let mut reste = limits.tables_max_bytes;
    for (table, meta) in &manifest.tables {
        // Défense en profondeur (review Pass 1, symétrie avec l'export 17-3a) :
        // un nom de colonne contenant un backtick casserait le back-quoting de
        // l'INSERT du restore. `check_schema_compat` le rejetterait aussi (la
        // colonne n'existe pas côté dest), mais on refuse au plus tôt.
        if let Some(bad) = meta.column_names.iter().find(|c| c.contains('`')) {
            return Err(invalide(format!(
                "table '{table}' : nom de colonne invalide (backtick) : {bad}"
            )));
        }
        let nom = format!("data/{table}.ndjson");
        let data = match zip.by_name(&nom) {
            Ok(entry) => lire_borne(entry, &nom, reste)?,
            Err(ZipError::FileNotFound) => return Err(invalide(format!("{nom} absent"))),
            Err(e) => return Err(invalide(format!("lecture {nom} : {e}"))),
        };
        reste -= data.len() as u64;
        let sha = sha256_hex(&data);
        if sha != meta.sha256 {
            return Err(invalide(format!(
                "intégrité SHA-256 invalide pour la table '{table}' (fichier altéré ?)"
            )));
        }
        let rows = parse_ndjson_rows(&data, &meta.column_names)
            .map_err(|e| invalide(format!("NDJSON '{table}' invalide : {e}")))?;
        if rows.len() != meta.row_count {
            return Err(invalide(format!(
                "table '{table}' : {} lignes lues, {} déclarées au manifeste",
                rows.len(),
                meta.row_count
            )));
        }
        total_rows += rows.len();
        tables.insert(
            table.clone(),
            TableRestore {
                column_names: meta.column_names.clone(),
                rows,
            },
        );
    }

    // (4) Format 2 : pièces (taille, empreinte), puis cohérence avec la base.
    if format_2 {
        for f in &manifest.files {
            verifier_piece(&mut zip, f)?;
        }
    }
    let mut parsed = ParsedBackup {
        manifest,
        tables,
        total_rows,
        archive: None,
    };
    if format_2 {
        verifier_coherence(&parsed)?;
    }
    parsed.archive = Some(zip);
    Ok(parsed)
}

/// Format 1 (17-3a à 0.14.x) : la règle d'origine — `manifest.json`,
/// `data/<table>.ndjson`, et `files/` **présent et vide** ; aucune liste de
/// pièces au manifeste.
fn verifier_format_1(manifest: &BackupManifest, noms: &[String]) -> Result<(), AppError> {
    if !manifest.files.is_empty() || !manifest.files_not_included.is_empty() {
        return Err(invalide(
            "un manifeste de format 1 ne déclare aucune pièce (files, filesNotIncluded)",
        ));
    }
    let mut has_files_dir = false;
    for nom in noms {
        if nom == "manifest.json" {
            continue;
        }
        if let Some(table) = nom
            .strip_prefix("data/")
            .and_then(|n| n.strip_suffix(".ndjson"))
        {
            // Nom de table = base du fichier ; rejeter les sous-chemins.
            if table.is_empty() || table.contains('/') {
                return Err(invalide(format!("entrée data/ invalide : {nom}")));
            }
        } else if nom == "files/" {
            has_files_dir = true;
        } else if nom.starts_with("files/") {
            // Toute entrée sous files/ (fichier OU sous-dossier) ⇒ non-vide
            // (review Pass 1 : un sous-dossier `files/x/` finissant par '/'
            // n'était pas détecté).
            return Err(invalide(
                "le dossier files/ doit être vide dans une sauvegarde de format 1",
            ));
        } else {
            return Err(invalide(format!(
                "entrée inattendue dans le backup : {nom}"
            )));
        }
    }
    // AC12(a) : `files/` doit être **présent**. Un backup authentique 17-3a
    // l'émet toujours (review Pass 1 : sa présence n'était pas vérifiée).
    if !has_files_dir {
        return Err(invalide("dossier files/ absent du backup"));
    }
    Ok(())
}

/// Format 2 (AC6) : les noms de `files` ont la forme `<sha>.<ext>` et leur
/// préfixe est l'empreinte déclarée ; `byteSize` ne dépasse pas le plafond
/// d'import ; aucun chemin deux fois dans une liste, ni dans les deux. Les
/// chemins de `filesNotIncluded` ne sont soumis à **aucune** expression : ils
/// ne servent jamais de chemin, et l'export y écrit tels quels ceux d'une ligne
/// en anomalie.
fn verifier_listes_de_pieces(
    manifest: &BackupManifest,
    limits: &ReadLimits,
) -> Result<(), AppError> {
    let mut vus: HashSet<&str> = HashSet::new();
    for f in &manifest.files {
        if !nom_de_piece_valide(&f.path) || f.path.split('.').next() != Some(f.sha256.as_str()) {
            return Err(invalide(format!(
                "nom de pièce invalide dans le manifeste : {:?}",
                f.path
            )));
        }
        if f.byte_size > limits.file_max_bytes {
            return Err(invalide(format!(
                "entrée files/{} trop grande ({} octets déclarés)",
                f.path, f.byte_size
            )));
        }
        if !vus.insert(f.path.as_str()) {
            return Err(invalide(format!("pièce déclarée deux fois : {:?}", f.path)));
        }
    }
    let mut non_inclus: HashSet<&str> = HashSet::new();
    for f in &manifest.files_not_included {
        if vus.contains(f.path.as_str()) || !non_inclus.insert(f.path.as_str()) {
            return Err(invalide(format!("pièce déclarée deux fois : {:?}", f.path)));
        }
    }
    Ok(())
}

/// Lit une pièce de l'archive en flux, à travers `take(byteSize + 1)` et un
/// `Sha256` : au-delà de la taille déclarée → « trop grande » (défense contre
/// l'archive piégée) ; plus courte → taille différente ; autre empreinte →
/// « intégrité SHA-256 invalide ».
fn verifier_piece<R: Read + Seek>(
    zip: &mut ZipArchive<R>,
    piece: &BackupFileMeta,
) -> Result<(), AppError> {
    let nom = format!("files/{}", piece.path);
    let entry = zip
        .by_name(&nom)
        .map_err(|e| invalide(format!("lecture {nom} : {e}")))?;
    let mut hasher = Sha256::new();
    let mut borne = entry.take(piece.byte_size.saturating_add(1));
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = borne
            .read(&mut buf)
            .map_err(|e| invalide(format!("lecture {nom} : {e}")))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total += n as u64;
    }
    if total > piece.byte_size {
        return Err(invalide(format!("entrée {nom} trop grande")));
    }
    if total != piece.byte_size {
        return Err(invalide(format!(
            "taille de la pièce {nom} différente de celle déclarée ({total} octets lus)"
        )));
    }
    if format!("{:x}", hasher.finalize()) != piece.sha256 {
        return Err(invalide(format!(
            "intégrité SHA-256 invalide pour le fichier {nom} (fichier altéré ?)"
        )));
    }
    Ok(())
}

/// Format 2 (AC7) — cohérence entre les pièces et la base du backup : toute
/// pièce de `files` est référencée par une ligne de même empreinte ; toute
/// pièce référencée est dans `files` ou `filesNotIncluded` (comparaison par
/// égalité de chaînes, si bien qu'un chemin hors forme listé par l'export
/// satisfait la règle).
fn verifier_coherence<R>(parsed: &ParsedBackup<R>) -> Result<(), AppError> {
    let references = parsed.referenced_files();
    let couples: HashSet<(&str, &str)> = references
        .iter()
        .map(|r| (r.storage_path.as_str(), r.sha256.as_str()))
        .collect();
    for f in &parsed.manifest.files {
        if !couples.contains(&(f.path.as_str(), f.sha256.as_str())) {
            return Err(invalide(format!(
                "la pièce {:?} n'est référencée par aucune ligne de même empreinte",
                f.path
            )));
        }
    }
    let declares: HashSet<&str> = parsed
        .manifest
        .files
        .iter()
        .map(|f| f.path.as_str())
        .chain(
            parsed
                .manifest
                .files_not_included
                .iter()
                .map(|f| f.path.as_str()),
        )
        .collect();
    for r in &references {
        if !declares.contains(r.storage_path.as_str()) {
            return Err(invalide(format!(
                "pièce référencée ni sauvegardée ni déclarée manquante : {:?} \
                 (sauvegarde incomplète ou falsifiée)",
                r.storage_path
            )));
        }
    }
    Ok(())
}

/// Story 31-1a (AC8) — écrit chaque pièce de l'archive sous `documents_dir`,
/// **en flux** et atomiquement (`restore_document`), puis synchronise le
/// dossier lui-même. Rend le nombre de pièces restaurées (écrites, ou déjà
/// présentes et intactes). Synchrone : l'appelant l'exécute en
/// `spawn_blocking`.
///
/// La synchronisation du dossier met sur disque les entrées de dossier avant
/// le `COMMIT` de la restauration — non testable (durabilité après coupure).
/// Son échec (montages réseau ou FUSE qui refusent `fsync` sur un dossier) est
/// journalisé et ne refuse pas l'import : les contenus sont déjà synchronisés
/// un à un. Aucun fichier du dossier n'est jamais supprimé.
///
/// # Errors
/// Le premier échec de lecture de l'archive ou d'écriture d'une pièce (texte
/// pour le journal) : l'appelant refuse alors la restauration, base intacte.
/// Les pièces déjà écrites restent (nommées par leur contenu, inoffensives).
pub(crate) fn ecrire_les_pieces<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    pieces: &[BackupFileMeta],
    documents_dir: &Path,
) -> Result<usize, String> {
    let mut restaurees = 0usize;
    for piece in pieces {
        let nom = format!("files/{}", piece.path);
        let entry = archive
            .by_name(&nom)
            .map_err(|e| format!("lecture de {nom} : {e}"))?;
        match restore_document(
            documents_dir,
            &piece.path,
            entry,
            piece.byte_size,
            &piece.sha256,
        ) {
            Ok(RestoreOutcome::Written) => {
                tracing::debug!(path = ?piece.path, "pièce restaurée");
            }
            Ok(RestoreOutcome::Replaced) => {
                tracing::warn!(path = ?piece.path, "pièce présente mais altérée : remplacée par celle de la sauvegarde");
            }
            Ok(RestoreOutcome::AlreadyIntact) => {}
            Err(e) => {
                return Err(format!("écriture de la pièce {:?} : {e}", piece.path));
            }
        }
        restaurees += 1;
    }
    if !pieces.is_empty()
        && let Err(e) = std::fs::File::open(documents_dir).and_then(|d| d.sync_all())
    {
        tracing::warn!(
            dir = ?documents_dir,
            kind = ?e.kind(),
            "synchronisation du dossier des documents refusée (montage sans fsync de dossier ?) \
             — import poursuivi, les pièces sont synchronisées une à une"
        );
    }
    Ok(restaurees)
}

/// Story 31-1a (AC9) — nombre de pièces référencées par la base **restaurée**
/// qui ne sont pas lisibles et intactes dans `documents_dir` après l'import
/// (chaque pièce présente est hachée : synchrone, en `spawn_blocking` chez
/// l'appelant). Le journal nomme chacune (`?path`) ; la réponse, non.
pub(crate) fn compter_les_pieces_manquantes(
    references: &[ReferencedFile],
    documents_dir: &Path,
) -> usize {
    references
        .iter()
        .filter(|r| {
            let manque = !document_is_intact(documents_dir, &r.storage_path, &r.sha256);
            if manque {
                tracing::warn!(path = ?r.storage_path, "pièce référencée manquante après l'import");
            }
            manque
        })
        .count()
}

/// Vérifie la compatibilité de schéma **bidirectionnelle** (AC12c) entre le
/// backup (source) et la base destination, via `INFORMATION_SCHEMA.COLUMNS`.
///
/// - (c1) chaque colonne source ⊆ colonnes destination (sinon `unknownColumns`) ;
/// - (c2) chaque colonne destination `NOT NULL` sans défaut, non-générée,
///   non-auto-increment ⊆ colonnes source (sinon `missingRequiredColumns`).
///
/// Retourne [`AppError::ImportSchemaMismatch`] (→400) au premier écart, et
/// [`AppError::AdminFullImportFailed`] si la lecture du schéma échoue.
/// ⚠️ Appelée **avant** la sauvegarde pré-import : l'appelant de l'import la
/// convertit par `routes::admin::avant_sauvegarde` (Story 15-13b, #576), sans
/// quoi le message annoncerait une sauvegarde qui n'existe pas. Tout nouvel
/// appelant d'avant sauvegarde doit reprendre cette conversion.
pub async fn check_schema_compat<R>(
    pool: &MySqlPool,
    parsed: &ParsedBackup<R>,
) -> Result<(), AppError> {
    for (table, data) in &parsed.tables {
        // `onboarding_state` n'est jamais restaurée (DC11) → inutile de valider
        // son schéma, et évite un faux `IMPORT_SCHEMA_MISMATCH` si son schéma
        // destination diverge de la source (review Pass 4).
        if table == "onboarding_state" {
            continue;
        }
        let dest = column_constraints(pool, table).await.map_err(|e| {
            AppError::AdminFullImportFailed(format!("lecture schéma '{table}' : {e}"))
        })?;

        let dest_names: HashSet<&str> = dest.iter().map(|c| c.name.as_str()).collect();
        let src_names: HashSet<&str> = data.column_names.iter().map(|s| s.as_str()).collect();

        // (c1) colonnes source inconnues côté destination.
        let mut unknown: Vec<String> = data
            .column_names
            .iter()
            .filter(|c| !dest_names.contains(c.as_str()))
            .cloned()
            .collect();
        unknown.sort();

        // (c2) colonnes destination obligatoires absentes de la source.
        let mut missing: Vec<String> = dest
            .iter()
            .filter(|c| c.is_required() && !src_names.contains(c.name.as_str()))
            .map(|c| c.name.clone())
            .collect();
        missing.sort();

        if !unknown.is_empty() || !missing.is_empty() {
            return Err(AppError::ImportSchemaMismatch {
                table: table.clone(),
                unknown_columns: unknown,
                missing_required_columns: missing,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    /// Bornes larges : celles de la production, plafond d'import de 2 Gio.
    fn larges() -> ReadLimits {
        ReadLimits::production(2048)
    }

    fn parse(bytes: Vec<u8>) -> Result<ParsedBackup<Cursor<Vec<u8>>>, AppError> {
        parse_and_verify(Cursor::new(bytes), &larges())
    }

    /// Construit un `.keshbackup` minimal en mémoire pour les tests de parsing.
    /// `tables` : (nom, column_names, ndjson_bytes).
    fn build_test_backup(
        format_version_override: Option<u32>,
        tables: &[(&str, Vec<&str>, &[u8])],
        include_all_required: bool,
        files_extra: Option<&str>,
    ) -> Vec<u8> {
        Fabrique {
            format: format_version_override.unwrap_or(BACKUP_FORMAT_VERSION),
            tables: tables
                .iter()
                .map(|(n, c, d)| {
                    (
                        n.to_string(),
                        c.iter().map(|s| s.to_string()).collect(),
                        d.to_vec(),
                    )
                })
                .collect(),
            completer: include_all_required,
            entrees: files_extra
                .map(|x| vec![(format!("files/{x}"), b"binaire interdit".to_vec())])
                .unwrap_or_default(),
            ..Fabrique::default()
        }
        .fabriquer()
    }

    /// Fabrique d'archives de test, format 1 ou 2.
    struct Fabrique {
        format: u32,
        /// Tables explicites : (nom, colonnes, NDJSON).
        tables: Vec<(String, Vec<String>, Vec<u8>)>,
        /// Compléter les tables applicatives manquantes (vides).
        completer: bool,
        /// `files` du manifeste.
        files: Vec<Value>,
        /// `filesNotIncluded` du manifeste.
        non_inclus: Vec<Value>,
        /// Entrées supplémentaires de l'archive (pièces, intrus), Stored.
        entrees: Vec<(String, Vec<u8>)>,
        /// Écrire l'entrée dossier `files/`.
        dossier_files: bool,
    }

    impl Default for Fabrique {
        fn default() -> Self {
            Self {
                format: BACKUP_FORMAT_VERSION,
                tables: Vec::new(),
                completer: true,
                files: Vec::new(),
                non_inclus: Vec::new(),
                entrees: Vec::new(),
                dossier_files: true,
            }
        }
    }

    impl Fabrique {
        fn fabriquer(&self) -> Vec<u8> {
            let mut cursor = Cursor::new(Vec::<u8>::new());
            {
                let mut zip = ZipWriter::new(&mut cursor);
                let opts: SimpleFileOptions =
                    SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
                let mut meta = serde_json::Map::new();
                for (name, cols, ndjson) in &self.tables {
                    zip.start_file(format!("data/{name}.ndjson"), opts).unwrap();
                    zip.write_all(ndjson).unwrap();
                    let row_count = std::str::from_utf8(ndjson)
                        .unwrap()
                        .lines()
                        .filter(|l| !l.is_empty())
                        .count();
                    meta.insert(
                        name.to_string(),
                        serde_json::json!({
                            "rowCount": row_count,
                            "sha256": sha256_hex(ndjson),
                            "columnNames": cols,
                        }),
                    );
                }
                if self.completer {
                    for &t in TABLES_TO_TRUNCATE {
                        if !meta.contains_key(t) {
                            zip.start_file(format!("data/{t}.ndjson"), opts).unwrap();
                            meta.insert(
                                t.to_string(),
                                serde_json::json!({
                                    "rowCount": 0,
                                    "sha256": sha256_hex(b""),
                                    "columnNames": ["id"],
                                }),
                            );
                        }
                    }
                }
                if self.dossier_files {
                    zip.add_directory("files/", opts).unwrap();
                }
                let stored = opts.compression_method(CompressionMethod::Stored);
                for (nom, contenu) in &self.entrees {
                    zip.start_file(nom.as_str(), stored).unwrap();
                    zip.write_all(contenu).unwrap();
                }
                let mut manifest = serde_json::json!({
                    "formatVersion": self.format,
                    "keshVersion": "0.15.0",
                    "keshVersionMinRequired": "0.1.0",
                    "instanceId": 1,
                    "exportDate": "2026-10-24T00:00:00Z",
                    "tables": meta,
                });
                if !self.files.is_empty() || !self.non_inclus.is_empty() {
                    manifest["files"] = Value::Array(self.files.clone());
                    manifest["filesNotIncluded"] = Value::Array(self.non_inclus.clone());
                }
                zip.start_file("manifest.json", opts).unwrap();
                zip.write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
                    .unwrap();
                zip.finish().unwrap();
            }
            cursor.into_inner()
        }
    }

    fn sha(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    /// Une pièce cohérente : nom `<sha>.pdf`, déclarée au manifeste, présente
    /// dans l'archive et référencée par une ligne de
    /// `imported_supplier_invoices`.
    fn avec_une_piece(contenu: &[u8]) -> (Fabrique, String) {
        let nom = format!("{}.pdf", sha(contenu));
        let ligne = format!(
            "{{\"id\":1,\"storage_path\":\"{nom}\",\"file_hash\":\"{}\"}}\n",
            sha(contenu)
        );
        let f = Fabrique {
            tables: vec![(
                "imported_supplier_invoices".into(),
                vec!["id".into(), "storage_path".into(), "file_hash".into()],
                ligne.into_bytes(),
            )],
            files: vec![serde_json::json!({
                "path": nom, "sha256": sha(contenu), "byteSize": contenu.len(),
            })],
            entrees: vec![(format!("files/{nom}"), contenu.to_vec())],
            ..Fabrique::default()
        };
        (f, nom)
    }

    fn message(err: AppError) -> String {
        match err {
            AppError::InvalidBackupStructure(m) => m,
            autre => panic!("InvalidBackupStructure attendue, trouvé {autre:?}"),
        }
    }

    #[test]
    fn parse_valid_backup_succeeds() {
        let ndjson = b"{\"id\":1,\"name\":\"Acme\"}\n";
        let backup = build_test_backup(
            None,
            &[("companies", vec!["id", "name"], ndjson)],
            true,
            None,
        );
        let parsed = parse(backup).expect("valid backup");
        assert_eq!(parsed.manifest.format_version, BACKUP_FORMAT_VERSION);
        assert_eq!(parsed.tables["companies"].rows.len(), 1);
        assert_eq!(parsed.total_rows, 1);
    }

    /// Story 31-1a (AC6, M-a10) : le format 2 est valide, le 3 est refusé.
    #[test]
    fn parse_rejects_future_format_version() {
        let backup = build_test_backup(Some(3), &[("companies", vec!["id"], b"")], true, None);
        let m = message(parse(backup).unwrap_err());
        assert!(
            m.contains("formatVersion 3 non supporté (maximum 2)"),
            "{m}"
        );
    }

    #[test]
    fn parse_rejects_sha_tamper() {
        // Construit un backup valide, puis on falsifie un NDJSON sans MAJ du
        // SHA en re-zippant à la main : plus simple — on altère via un backup
        // où le sha déclaré ne matche pas. On force un mismatch en réécrivant.
        let good = b"{\"id\":1}\n";
        let backup = build_test_backup(None, &[("companies", vec!["id"], good)], true, None);
        // Re-parse OK d'abord.
        assert!(parse(backup).is_ok());

        // Falsification : on reconstruit avec un ndjson différent mais un sha
        // figé de l'ancien contenu via un manifeste forgé.
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zip.start_file("data/companies.ndjson", opts).unwrap();
            zip.write_all(b"{\"id\":999}\n").unwrap(); // contenu altéré
            let mut meta = serde_json::Map::new();
            meta.insert(
                "companies".into(),
                serde_json::json!({
                    "rowCount": 1,
                    "sha256": sha256_hex(good), // SHA de l'ancien contenu → mismatch
                    "columnNames": ["id"],
                }),
            );
            for &t in TABLES_TO_TRUNCATE {
                if t != "companies" {
                    zip.start_file(format!("data/{t}.ndjson"), opts).unwrap();
                    meta.insert(
                        t.into(),
                        serde_json::json!({ "rowCount": 0, "sha256": sha256_hex(b""), "columnNames": ["id"] }),
                    );
                }
            }
            zip.add_directory("files/", opts).unwrap();
            let manifest = serde_json::json!({
                "formatVersion": 1, "keshVersion": "0.1.8", "keshVersionMinRequired": "0.1.0",
                "instanceId": 1, "exportDate": "2026-06-09T00:00:00Z", "tables": meta,
            });
            zip.start_file("manifest.json", opts).unwrap();
            zip.write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
                .unwrap();
            zip.finish().unwrap();
        }
        let tampered = cursor.into_inner();
        let err = parse(tampered).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("SHA-256")),
            "expected SHA mismatch, got {err:?}"
        );
    }

    /// Format 1 : la règle d'origine — `files/` doit être vide. (Story 31-1a :
    /// le test force le format 1, la valeur par défaut étant désormais 2.)
    #[test]
    fn parse_rejects_non_empty_files_dir() {
        let backup = build_test_backup(
            Some(1),
            &[("companies", vec!["id"], b"")],
            true,
            Some("rogue.bin"),
        );
        let err = parse(backup).unwrap_err();
        assert!(matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("files/")));
    }

    #[test]
    fn parse_rejects_missing_table_coverage() {
        // Un seul fichier table, sans compléter les autres → couverture KO.
        let backup = build_test_backup(
            None,
            &[("companies", vec!["id"], b"{\"id\":1}\n")],
            false,
            None,
        );
        let err = parse(backup).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("absente du manifeste"))
        );
    }

    #[test]
    fn parse_rejects_missing_manifest() {
        // ZIP sans manifest.json.
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts = SimpleFileOptions::default();
            zip.add_directory("files/", opts).unwrap();
            zip.finish().unwrap();
        }
        let err = parse(cursor.into_inner()).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("manifest.json absent"))
        );
    }

    // =======================================================================
    // Story 31-1a (#503) — format 2 et lectures bornées (T-a1)
    // =======================================================================

    /// AC6, AC7 — une sauvegarde de format 2 cohérente se lit, archive comprise ;
    /// sa pièce est référencée par la base du backup.
    #[test]
    fn format_2_valide() {
        let (f, nom) = avec_une_piece(b"piece valide");
        let parsed = parse(f.fabriquer()).expect("format 2 valide");
        assert_eq!(parsed.manifest.files.len(), 1);
        assert!(parsed.archive.is_some());
        assert_eq!(
            parsed.referenced_files(),
            vec![ReferencedFile {
                storage_path: nom,
                sha256: sha(b"piece valide"),
            }]
        );
    }

    /// AC6 (M-a4) — une ligne de `manifest.files` sans entrée dans l'archive.
    #[test]
    fn format_2_ligne_du_manifeste_sans_entree() {
        let (mut f, _) = avec_une_piece(b"sans entree");
        f.entrees.clear();
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("absente de l'archive"), "{m}");
    }

    /// AC6b (2) — une entrée `files/*` sans ligne au manifeste est refusée.
    #[test]
    fn format_2_entree_sans_ligne() {
        let (mut f, _) = avec_une_piece(b"avec intrus");
        f.entrees.push(("files/intrus.pdf".into(), b"x".to_vec()));
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(
            m.contains("entrée inattendue dans le backup : files/intrus.pdf"),
            "{m}"
        );
    }

    /// AC6 — un sous-dossier sous `files/` est refusé.
    #[test]
    fn format_2_sous_dossier_refuse() {
        let (mut f, _) = avec_une_piece(b"sous dossier");
        f.entrees.push(("files/x/".into(), Vec::new()));
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("entrée inattendue"), "{m}");
    }

    /// AC6 — un nom qui n'a pas la forme `<sha>.<ext>`.
    #[test]
    fn format_2_nom_invalide() {
        let contenu = b"nom invalide";
        let f = Fabrique {
            files: vec![serde_json::json!({
                "path": "facture.pdf", "sha256": sha(contenu), "byteSize": contenu.len(),
            })],
            entrees: vec![("files/facture.pdf".into(), contenu.to_vec())],
            ..Fabrique::default()
        };
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("nom de pièce invalide"), "{m}");
    }

    /// AC6 (M-a3) — un nom dont le préfixe n'est pas l'empreinte déclarée.
    #[test]
    fn format_2_prefixe_different_de_l_empreinte() {
        let contenu = b"prefixe";
        let nom = format!("{}.pdf", sha(b"autre contenu"));
        let f = Fabrique {
            files: vec![serde_json::json!({
                "path": nom, "sha256": sha(contenu), "byteSize": contenu.len(),
            })],
            entrees: vec![(format!("files/{nom}"), contenu.to_vec())],
            ..Fabrique::default()
        };
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("nom de pièce invalide"), "{m}");
    }

    /// AC7 (M-a5) — le contenu n'a pas l'empreinte déclarée (nom et
    /// déclaration cohérents, contenu substitué de même taille).
    #[test]
    fn format_2_empreinte_fausse() {
        let (mut f, nom) = avec_une_piece(b"contenu juste");
        f.entrees = vec![(format!("files/{nom}"), b"contenu faux!".to_vec())];
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(
            m.contains("intégrité SHA-256 invalide pour le fichier"),
            "{m}"
        );
    }

    /// AC7 (M-a6) — une entrée plus grande que son `byteSize`, **dont
    /// l'empreinte déclarée est celle du contenu entier** : seule la borne la
    /// refuse (sans elle, l'empreinte concorderait).
    #[test]
    fn format_2_entree_plus_grande_que_declaree() {
        let contenu = b"contenu entier, plus long que declare";
        let nom = format!("{}.pdf", sha(contenu));
        let ligne = format!(
            "{{\"id\":1,\"storage_path\":\"{nom}\",\"file_hash\":\"{}\"}}\n",
            sha(contenu)
        );
        let f = Fabrique {
            tables: vec![(
                "imported_supplier_invoices".into(),
                vec!["id".into(), "storage_path".into(), "file_hash".into()],
                ligne.into_bytes(),
            )],
            files: vec![serde_json::json!({
                "path": nom, "sha256": sha(contenu), "byteSize": 10,
            })],
            entrees: vec![(format!("files/{nom}"), contenu.to_vec())],
            ..Fabrique::default()
        };
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("trop grande"), "{m}");
    }

    /// AC6b — un `byteSize` supérieur au plafond d'import (ici `u64::MAX`) est
    /// refusé dès la lecture du manifeste, sans panique.
    #[test]
    fn format_2_byte_size_au_dela_du_plafond() {
        let (mut f, nom) = avec_une_piece(b"plafond");
        f.files = vec![serde_json::json!({
            "path": nom, "sha256": sha(b"plafond"), "byteSize": u64::MAX,
        })];
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("trop grande"), "{m}");
    }

    /// AC6 — un chemin dans les deux listes, ou deux fois dans une liste.
    #[test]
    fn format_2_chemin_declare_deux_fois() {
        let (mut f, nom) = avec_une_piece(b"deux fois");
        f.non_inclus = vec![serde_json::json!({ "path": nom, "reason": "missing" })];
        assert!(message(parse(f.fabriquer()).unwrap_err()).contains("deux fois"));

        let (mut f, _) = avec_une_piece(b"doublon");
        f.non_inclus = vec![
            serde_json::json!({ "path": "a.pdf", "reason": "missing" }),
            serde_json::json!({ "path": "a.pdf", "reason": "altered" }),
        ];
        assert!(message(parse(f.fabriquer()).unwrap_err()).contains("deux fois"));
    }

    /// AC6 (M-a18) — un chemin hors forme dans `filesNotIncluded`, que la base
    /// référence tel quel, est **accepté** : une sauvegarde authentique
    /// s'importe.
    #[test]
    fn format_2_non_inclus_hors_forme_accepte() {
        let f = Fabrique {
            tables: vec![(
                "imported_supplier_invoices".into(),
                vec!["id".into(), "storage_path".into(), "file_hash".into()],
                format!(
                    "{{\"id\":1,\"storage_path\":\"../x.pdf\",\"file_hash\":\"{}\"}}\n",
                    sha(b"x")
                )
                .into_bytes(),
            )],
            non_inclus: vec![serde_json::json!({ "path": "../x.pdf", "reason": "mismatch" })],
            ..Fabrique::default()
        };
        parse(f.fabriquer()).expect("hors forme accepté dans filesNotIncluded");
    }

    /// AC7 — format 2 : une pièce référencée ni emportée ni déclarée → 400 ;
    /// la même absence en format 1 n'est pas contrôlée (M-a15).
    #[test]
    fn coherence_appliquee_au_seul_format_2() {
        let ligne = format!(
            "{{\"id\":1,\"storage_path\":\"{}.pdf\",\"file_hash\":\"{}\"}}\n",
            sha(b"y"),
            sha(b"y")
        );
        let tables = vec![(
            "imported_supplier_invoices".to_string(),
            vec!["id".to_string(), "storage_path".into(), "file_hash".into()],
            ligne.into_bytes(),
        )];
        let f2 = Fabrique {
            tables: tables.clone(),
            ..Fabrique::default()
        };
        assert!(message(parse(f2.fabriquer()).unwrap_err()).contains("ni sauvegardée"));
        let f1 = Fabrique {
            format: 1,
            tables,
            ..Fabrique::default()
        };
        parse(f1.fabriquer()).expect("format 1 : pas de contrôle de cohérence");
    }

    /// AC6b (1) — un manifeste plus grand que sa borne.
    #[test]
    fn manifeste_trop_grand() {
        let bytes = Fabrique::default().fabriquer();
        let limites = ReadLimits {
            manifest_max_bytes: 64,
            ..larges()
        };
        let m = message(parse_and_verify(Cursor::new(bytes), &limites).unwrap_err());
        assert!(m.contains("entrée manifest.json trop grande"), "{m}");
    }

    /// AC6b (3) (M-a14) — la borne des tables est **cumulée** : deux tables
    /// chacune sous la borne, leur somme au-dessus → 400 ; une seule passe.
    #[test]
    fn tables_trop_grandes_en_cumul() {
        let ligne =
            |n: usize| format!("{{\"id\":{n},\"name\":\"{}\"}}\n", "x".repeat(600)).into_bytes();
        let limites = ReadLimits {
            tables_max_bytes: 1000,
            ..larges()
        };
        let une = Fabrique {
            tables: vec![(
                "companies".into(),
                vec!["id".into(), "name".into()],
                ligne(1),
            )],
            ..Fabrique::default()
        };
        parse_and_verify(Cursor::new(une.fabriquer()), &limites).expect("une table sous la borne");
        let deux = Fabrique {
            tables: vec![
                (
                    "companies".into(),
                    vec!["id".into(), "name".into()],
                    ligne(1),
                ),
                (
                    "contacts".into(),
                    vec!["id".into(), "name".into()],
                    ligne(2),
                ),
            ],
            ..Fabrique::default()
        };
        let m = message(parse_and_verify(Cursor::new(deux.fabriquer()), &limites).unwrap_err());
        assert!(m.contains("trop grande"), "{m}");
    }

    /// AC6b (2) — l'entrée inattendue est refusée **avant** la lecture des
    /// tables : la même archive porte une table au-delà de la borne, et c'est
    /// l'entrée inattendue que le message nomme.
    #[test]
    fn entree_inattendue_refusee_avant_les_tables() {
        let mut f = Fabrique {
            tables: vec![(
                "companies".into(),
                vec!["id".into(), "name".into()],
                format!("{{\"id\":1,\"name\":\"{}\"}}\n", "x".repeat(600)).into_bytes(),
            )],
            ..Fabrique::default()
        };
        f.entrees.push(("ailleurs.txt".into(), b"x".to_vec()));
        let limites = ReadLimits {
            tables_max_bytes: 100,
            ..larges()
        };
        let m = message(parse_and_verify(Cursor::new(f.fabriquer()), &limites).unwrap_err());
        assert!(
            m.contains("entrée inattendue dans le backup : ailleurs.txt"),
            "{m}"
        );
    }

    /// Format 1 — un manifeste qui déclarerait des pièces est incohérent.
    #[test]
    fn format_1_ne_declare_aucune_piece() {
        let (mut f, _) = avec_une_piece(b"format 1");
        f.format = 1;
        f.entrees.clear();
        let m = message(parse(f.fabriquer()).unwrap_err());
        assert!(m.contains("format 1 ne déclare aucune pièce"), "{m}");
    }

    /// AC8 — `ecrire_les_pieces` restaure la pièce de l'archive en flux.
    #[test]
    fn ecrire_les_pieces_restaure_l_archive() {
        let (f, nom) = avec_une_piece(b"a restaurer");
        let mut parsed = parse(f.fabriquer()).unwrap();
        let mut archive = parsed.archive.take().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let n = ecrire_les_pieces(&mut archive, &parsed.manifest.files, dir.path()).unwrap();
        assert_eq!(n, 1);
        assert_eq!(
            std::fs::read(dir.path().join(&nom)).unwrap(),
            b"a restaurer"
        );
        assert_eq!(
            compter_les_pieces_manquantes(&parsed.referenced_files(), dir.path()),
            0
        );
        let vide = tempfile::tempdir().unwrap();
        assert_eq!(
            compter_les_pieces_manquantes(&parsed.referenced_files(), vide.path()),
            1
        );
    }
}
