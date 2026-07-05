//! Service d'import du répertoire **inbox** de factures fournisseurs (Story 12-5c, #194).
//!
//! Sur déclenchement manuel (`POST /api/v1/inbox-import`), lit `KESH_INBOX_DIR`,
//! décode le Swiss QR de chaque fichier (12-5b), archive le justificatif
//! (`KESH_DOCUMENTS_DIR`, 12-5b) et crée une facture importée en **staging**
//! (`to_complete`). Le fichier traité avec succès est supprimé de l'inbox ; en
//! échec il est déplacé dans `<inbox>/failed/`.
//!
//! ## Pattern batch (CLAUDE.md, §FailedProposal)
//! Retourne `{ accepted, failed, warnings }` avec **HTTP 200** : aucune erreur
//! per-fichier n'escalade en `AppError` global. Les seules `AppError` globales
//! sont les exceptions amont : `401/403` (RBAC), `409` ([`AppError::InboxImportAlreadyRunning`]),
//! `500` (catastrophe DB/IO).
//!
//! ## Verrou de run (F6, Option A)
//! `GET_LOCK` MariaDB sur une **connexion dédiée** (`pool.acquire()`) tenue
//! pendant tout le run, relâchée par `RELEASE_LOCK` sur la même connexion. Pas
//! d'impact sur `AppState` ni sur les ~33 call-sites de tests (vs Option B
//! `AtomicBool`). La clé de verrou est **namespacée par base de données**
//! (`kesh_inbox_import:{db}`) : en prod (une seule base = un seul inbox physique)
//! la sérialisation est totale ; en test (`#[sqlx::test]` → base isolée) deux
//! tests parallèles n'interfèrent pas.
//!
//! ## Limitation L8 — inbox non partitionné par tenant (Issue #199)
//! `KESH_INBOX_DIR` est un répertoire **global unique** par instance. Chaque
//! fichier est stagé sous le `company_id` du Comptable qui déclenche l'import
//! (puis supprimé de l'inbox au succès). Sur une instance **multi-tenant**, le
//! premier Comptable à importer réclamerait tous les fichiers présents pour sa
//! company. **Sans impact** pour le déploiement cible mono-PME-par-NAS (choix
//! design assumé, umbrella F-NEW-3). Partitionnement par company = follow-up
//! v0.4-milestone (Issue #199) si un déploiement multi-tenant réel apparaît.

use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

use crate::config::Config;
use crate::document_storage::{self, mime_for_ext};
use crate::errors::AppError;
use crate::qr_decode::{self, DecodeConfig, DecodeError};
use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::errors::DbError;
use kesh_db::repositories::imported_supplier_invoices;

// --- Catalogue des `error_code` per-fichier (constantes canoniques) ----------

const ERR_UNSUPPORTED_FILE_TYPE: &str = "UNSUPPORTED_FILE_TYPE";
const ERR_FILE_TOO_LARGE: &str = "FILE_TOO_LARGE";
const ERR_SYMLINK_REJECTED: &str = "SYMLINK_REJECTED";
const ERR_DUPLICATE: &str = "DUPLICATE";
const ERR_NO_QR_CODE_FOUND: &str = "NO_QR_CODE_FOUND";
const ERR_INVALID_SPC_PAYLOAD: &str = "INVALID_SPC_PAYLOAD";
const ERR_INVALID_IBAN: &str = "INVALID_IBAN";
const ERR_PDF_RENDER_ERROR: &str = "PDF_RENDER_ERROR";
const ERR_FILE_READ_ERROR: &str = "FILE_READ_ERROR";
const ERR_FIELD_TOO_LONG: &str = "FIELD_TOO_LONG";

/// Extensions acceptées (liste blanche, AC3 step 4).
const ALLOWED_EXT: [&str; 4] = ["pdf", "png", "jpg", "jpeg"];

/// Délai inter-lecture du check de stabilité (AC3 step 3). Court : un fichier en
/// cours d'écriture (copie inbox) verra sa taille/mtime bouger sur cet intervalle.
const STABILITY_DELAY: std::time::Duration = std::time::Duration::from_millis(25);

// --- Rapport batch -----------------------------------------------------------

/// Rapport d'un run d'import (`{ accepted, failed, warnings }`, HTTP 200).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxImportReport {
    pub accepted: Vec<AcceptedFile>,
    pub failed: Vec<FailedFile>,
    pub warnings: Vec<String>,
}

/// Fichier importé avec succès (staging créé ou `discarded` réactivée).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcceptedFile {
    pub imported_supplier_invoice_id: i64,
    pub file_name: String,
}

/// Échec per-fichier (identifiant business = `file_name`, jamais d'index positionnel).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FailedFile {
    pub file_name: String,
    pub error_code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Issue du traitement d'un fichier. Le disposal inbox (suppression succès /
/// déplacement `failed/`) est fait **dans** `process_one_file` ; ce type ne porte
/// que la ligne de rapport à agréger.
enum FileOutcome {
    Accepted(i64),
    Failed {
        error_code: &'static str,
        details: Option<serde_json::Value>,
    },
    /// Fichier instable (en cours d'écriture) : laissé en place, retenté au
    /// prochain run. `warning` est un message borné (un par fichier instable).
    Skipped(String),
}

// --- Entrée publique : run avec verrou ---------------------------------------

/// Exécute un run d'import complet sous verrou de run (F6). Renvoie le rapport
/// batch (HTTP 200) ou une `AppError` globale (409 si déjà en cours, 500 catastrophe).
pub async fn run_inbox_import(
    pool: &MySqlPool,
    config: &Config,
    company_id: i64,
) -> Result<InboxImportReport, AppError> {
    // (1) Connexion dédiée tenue pendant tout le run (GET_LOCK est lié à la
    //     connexion qui l'acquiert — JAMAIS via pool.execute() qui recyclerait
    //     la connexion et fuiterait le verrou).
    let mut lock_conn = pool.acquire().await.map_err(|e| {
        AppError::Internal(format!("inbox import: acquisition connexion verrou: {e}"))
    })?;

    let db_name: Option<String> = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&mut *lock_conn)
        .await
        .map_err(|e| AppError::Internal(format!("inbox import: SELECT DATABASE(): {e}")))?;
    let lock_key = format!("kesh_inbox_import:{}", db_name.unwrap_or_default());

    // GET_LOCK(key, 0) : 1 = acquis, 0 = déjà tenu (timeout immédiat), NULL = erreur.
    let acquired: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
        .bind(&lock_key)
        .fetch_one(&mut *lock_conn)
        .await
        .map_err(|e| AppError::Internal(format!("inbox import: GET_LOCK: {e}")))?;
    match acquired {
        Some(1) => {}                                               // verrou acquis
        Some(_) => return Err(AppError::InboxImportAlreadyRunning), // 0 = déjà tenu
        None => {
            // NULL = erreur interne MariaDB sur le verrou (jamais « déjà en cours »).
            return Err(AppError::Internal(
                "inbox import: GET_LOCK a retourné NULL (erreur verrou MariaDB)".into(),
            ));
        }
    }

    // (2) Traitement : on capture le résultat puis on relâche TOUJOURS le verrou
    //     (succès comme erreur) sur la même connexion avant de la rendre au pool.
    let result = process_inbox(pool, config, company_id).await;

    let _ = sqlx::query("SELECT RELEASE_LOCK(?)")
        .bind(&lock_key)
        .execute(&mut *lock_conn)
        .await;
    drop(lock_conn);

    result
}

// --- Boucle de traitement ----------------------------------------------------

/// `create_dir_all` puis `canonicalize` (la racine peut être un symlink NAS
/// `/data`→`/volume1` — l'anti-traversal compare des chemins canonicalisés, AC1).
fn ensure_canonical_dir(dir: &str) -> std::io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    std::fs::canonicalize(dir)
}

async fn process_inbox(
    pool: &MySqlPool,
    config: &Config,
    company_id: i64,
) -> Result<InboxImportReport, AppError> {
    let inbox_root = ensure_canonical_dir(&config.inbox_dir)
        .map_err(|e| AppError::Internal(format!("inbox import: racine inbox: {e}")))?;
    let documents_root = ensure_canonical_dir(&config.documents_dir)
        .map_err(|e| AppError::Internal(format!("inbox import: racine documents: {e}")))?;

    // `failed/` créé une fois avant la boucle (plus robuste qu'à la volée).
    let failed_dir = inbox_root.join("failed");
    let failed_dir = ensure_canonical_dir(failed_dir.to_string_lossy().as_ref())
        .map_err(|e| AppError::Internal(format!("inbox import: dossier failed/: {e}")))?;

    let entries = std::fs::read_dir(&inbox_root)
        .map_err(|e| AppError::Internal(format!("inbox import: lecture inbox: {e}")))?;

    let mut report = InboxImportReport::default();
    let mut processed = 0usize;
    let mut truncated = false;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue, // entrée disparue en race → ignorée silencieusement
        };
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        // (0) Filtrage type d'entrée : ni fichier régulier ni symlink → ignoré
        //     silencieusement (répertoire dont `failed/`, socket, FIFO, device).
        let sym_meta = match std::fs::symlink_metadata(&path) {
            Ok(m) => m,
            Err(_) => continue, // disparue en race
        };
        let ft = sym_meta.file_type();
        if !ft.is_file() && !ft.is_symlink() {
            continue;
        }

        // Cap MAX_FILES_PER_RUN : ne compte que les candidats (post-filtrage type).
        if processed >= config.inbox_max_files_per_run {
            truncated = true;
            break;
        }
        processed += 1;

        match process_one_file(
            pool,
            config,
            &inbox_root,
            &documents_root,
            &failed_dir,
            company_id,
            &path,
            &file_name,
            &sym_meta,
        )
        .await?
        {
            FileOutcome::Accepted(id) => report.accepted.push(AcceptedFile {
                imported_supplier_invoice_id: id,
                file_name,
            }),
            FileOutcome::Failed {
                error_code,
                details,
            } => report.failed.push(FailedFile {
                file_name,
                error_code,
                details,
            }),
            FileOutcome::Skipped(warning) => report.warnings.push(warning),
        }
    }

    if truncated {
        // Un SEUL message de troncature (borné), pas un par fichier excédentaire.
        report.warnings.push(format!(
            "Limite de {} fichiers par import atteinte — relancez l'import pour traiter le reste.",
            config.inbox_max_files_per_run
        ));
    }

    Ok(report)
}

/// Traite un fichier inbox de bout en bout et **dispose** du fichier (suppression
/// au succès / déplacement `failed/` à l'échec). Ne renvoie `Err(AppError)` que
/// pour une **catastrophe** (DB non-classifiable, IO de disposal) — tout le reste
/// est encapsulé en `FileOutcome::Failed` (HTTP 200).
#[allow(clippy::too_many_arguments)]
async fn process_one_file(
    pool: &MySqlPool,
    config: &Config,
    inbox_root: &Path,
    documents_root: &Path,
    failed_dir: &Path,
    company_id: i64,
    path: &Path,
    file_name: &str,
    sym_meta: &std::fs::Metadata,
) -> Result<FileOutcome, AppError> {
    // (1) Symlink rejeté (jamais suivi/ouvert).
    if sym_meta.file_type().is_symlink() {
        return dispose_failed(
            path,
            failed_dir,
            file_name,
            &random_suffix(),
            ERR_SYMLINK_REJECTED,
            None,
        );
    }

    // (2) Taille via stat (PAS de lecture du contenu) — anti-DoS mémoire.
    if sym_meta.len() > config.inbox_max_file_bytes {
        return dispose_failed(
            path,
            failed_dir,
            file_name,
            &random_suffix(),
            ERR_FILE_TOO_LARGE,
            Some(
                serde_json::json!({ "byteSize": sym_meta.len(), "maxBytes": config.inbox_max_file_bytes }),
            ),
        );
    }

    // (3) Stabilité : taille + mtime identiques sur 2 lectures espacées.
    let len1 = sym_meta.len();
    let mtime1 = sym_meta.modified().ok();
    tokio::time::sleep(STABILITY_DELAY).await;
    let meta2 = match std::fs::metadata(path) {
        Ok(m) => m,
        // `dispose_failed` distingue ENOENT (disparu → skip) d'une autre erreur IO
        // (présent mais illisible → failed FILE_READ_ERROR).
        Err(_) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_FILE_READ_ERROR,
                None,
            );
        }
    };
    if meta2.len() != len1 || meta2.modified().ok() != mtime1 {
        // Instable : laissé en place, retenté au prochain run (ni accepted ni failed).
        return Ok(FileOutcome::Skipped(format!(
            "Fichier « {file_name} » en cours d'écriture — ignoré ce tour, relancez l'import."
        )));
    }

    // (4-bis) Anti-TOCTOU symlink : re-vérifier juste avant lecture (le check (1)
    //         et l'ouverture sont séparés). Atténuation sans dépendance `libc`
    //         (O_NOFOLLOW) — risque modéré documenté (inbox semi-contrôlée admin, L6).
    match std::fs::symlink_metadata(path) {
        Ok(m) if m.file_type().is_symlink() => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_SYMLINK_REJECTED,
                None,
            );
        }
        Ok(_) => {}
        Err(_) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_FILE_READ_ERROR,
                None,
            );
        }
    }

    // Anti-traversal : le chemin résolu doit rester sous la racine inbox
    // canonicalisée (AC1/AC4). Un fichier dans `failed/` (sous-dossier) est exclu
    // du traitement — il a déjà échoué. Refus défensif si hors racine.
    match std::fs::canonicalize(path) {
        Ok(canon) if !canon.starts_with(inbox_root) || canon.starts_with(failed_dir) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_FILE_READ_ERROR,
                None,
            );
        }
        Ok(_) => {}
        Err(_) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_FILE_READ_ERROR,
                None,
            );
        }
    }

    // Lecture du contenu (taille déjà bornée).
    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(_) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &random_suffix(),
                ERR_FILE_READ_ERROR,
                None,
            );
        }
    };

    // (4) Type : extension ∈ liste blanche ET magic bytes cohérents.
    let ext = ext_of(file_name);
    let ext_ok = ext
        .as_deref()
        .map(|e| ALLOWED_EXT.contains(&e))
        .unwrap_or(false);
    let ext = ext.unwrap_or_default();
    if !ext_ok || !magic_matches(&ext, &bytes) {
        return dispose_failed(
            path,
            failed_dir,
            file_name,
            &random_suffix(),
            ERR_UNSUPPORTED_FILE_TYPE,
            None,
        );
    }

    // (5) Hash SHA-256 (avant décodage) → court-circuit doublon (AC6).
    let file_hash = sha256_hex(&bytes);
    let disambig = file_hash[..8].to_string();

    match imported_supplier_invoices::find_by_company_hash(pool, company_id, &file_hash).await? {
        Some(existing) if existing.status == "discarded" => {
            // Réactivation `discarded` → `to_complete` (F5). Le justificatif est
            // déjà archivé (storage_path de la row) → pas de ré-archivage, on
            // supprime juste la copie inbox.
            let reactivated =
                imported_supplier_invoices::reactivate_to_complete(pool, company_id, existing.id)
                    .await?;
            if reactivated {
                remove_inbox_file(path);
                return Ok(FileOutcome::Accepted(existing.id));
            }
            // Race : la row n'est plus `discarded` → traité comme doublon.
            return dispose_failed(path, failed_dir, file_name, &disambig, ERR_DUPLICATE, None);
        }
        Some(_) => {
            // Row `to_complete` / `completed` existante → doublon.
            return dispose_failed(path, failed_dir, file_name, &disambig, ERR_DUPLICATE, None);
        }
        None => {}
    }

    // (6) Décodage (12-5b) : PDF vs image.
    let decode_result = if ext == "pdf" {
        qr_decode::decode_spc_from_pdf_bytes(
            &bytes,
            DecodeConfig {
                max_pages: config.inbox_max_pdf_pages,
                ..DecodeConfig::default()
            },
        )
    } else {
        qr_decode::decode_spc_from_image_bytes(&bytes)
    };
    let scanned = match decode_result {
        Ok(Some(s)) => s,
        Ok(None) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &disambig,
                ERR_NO_QR_CODE_FOUND,
                None,
            );
        }
        Err(e) => {
            let code = map_decode_error(&e);
            return dispose_failed(path, failed_dir, file_name, &disambig, code, None);
        }
    };

    // (7) Archivage du justificatif (12-5b) — nom = `{sha256hex}.{ext}`.
    let mime = mime_for_ext(&ext);
    let doc = match document_storage::store_document(documents_root, &bytes, &ext, file_name, mime)
    {
        Ok(d) => d,
        Err(_) => {
            return dispose_failed(
                path,
                failed_dir,
                file_name,
                &disambig,
                ERR_FILE_READ_ERROR,
                None,
            );
        }
    };
    let storage_path = doc.storage_path.clone();

    // (8) Staging : INSERT imported_supplier_invoices (status='to_complete').
    let new = NewImportedSupplierInvoice::from_scanned(company_id, &scanned, doc);
    match imported_supplier_invoices::create(pool, &new).await {
        Ok(row) => {
            remove_inbox_file(path);
            Ok(FileOutcome::Accepted(row.id))
        }
        Err(DbError::UniqueConstraintViolation(_)) => {
            // Race sur UNIQUE (company_id, file_hash) : le justificatif archivé est
            // partagé (content-addressed) avec la row gagnante → NE PAS le supprimer.
            dispose_failed(path, failed_dir, file_name, &disambig, ERR_DUPLICATE, None)
        }
        Err(DbError::DataLengthOrRange(_)) => {
            // Champ QR tiers sur-long (hors SIX 2.2) → échec par-fichier propre (D2).
            // Nettoyage best-effort de l'orphelin archivé (content-addressed →
            // idempotent : un ré-import réécrirait le même chemin).
            let _ = std::fs::remove_file(documents_root.join(&storage_path));
            dispose_failed(
                path,
                failed_dir,
                file_name,
                &disambig,
                ERR_FIELD_TOO_LONG,
                None,
            )
        }
        // Toute autre DbError = catastrophe (pool mort, etc.) → 500 global.
        Err(e) => Err(AppError::Database(e)),
    }
}

// --- Helpers -----------------------------------------------------------------

fn ext_of(name: &str) -> Option<String> {
    Path::new(name)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
}

/// Magic bytes cohérents avec l'extension déclarée (AC3 step 4).
fn magic_matches(ext: &str, bytes: &[u8]) -> bool {
    match ext {
        "pdf" => bytes.starts_with(b"%PDF"),
        "png" => bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]),
        "jpg" | "jpeg" => bytes.starts_with(&[0xFF, 0xD8, 0xFF]),
        _ => false,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

/// Mappe une [`DecodeError`] (12-5b) vers un `error_code` du catalogue (AC2).
/// `ImageDecode` → `UNSUPPORTED_FILE_TYPE` : le magic-bytes a déjà filtré les
/// non-images ; à ce stade, format reconnu mais corrompu/non décodable (PAS
/// `FILE_READ_ERROR`, réservé aux I/O — le contenu a déjà été lu).
fn map_decode_error(e: &DecodeError) -> &'static str {
    match e {
        DecodeError::ImageDecode(_) => ERR_UNSUPPORTED_FILE_TYPE,
        DecodeError::PdfRender(_) => ERR_PDF_RENDER_ERROR,
        DecodeError::InvalidSpcPayload(_) => ERR_INVALID_SPC_PAYLOAD,
        DecodeError::InvalidIban(_) => ERR_INVALID_IBAN,
    }
}

/// Suffixe d'anti-collision pour les fichiers `failed/` sans hash de contenu
/// disponible (symlink, oversized, read-error). 8 hex d'un UUID v4.
fn random_suffix() -> String {
    uuid::Uuid::new_v4().simple().to_string()[..8].to_string()
}

/// Dispose d'un fichier en échec : tente le déplacement vers `failed/` puis
/// renvoie l'issue de rapport correspondante.
///
/// **Race fichier disparu (BH1)** : si le fichier s'est volatilisé entre sa
/// détection (`read_dir`) et ici (un outil d'upload qui supprime la source après
/// copie), `move_to_failed` renvoie `Ok(false)` (ENOENT) → l'issue est
/// `Skipped` (comme la race `read_dir`), PAS un `failed[]` ni un 500 qui
/// abattrait tout le run. Une autre erreur IO du `rename` reste une catastrophe → 500.
fn dispose_failed(
    path: &Path,
    failed_dir: &Path,
    file_name: &str,
    disambig: &str,
    error_code: &'static str,
    details: Option<serde_json::Value>,
) -> Result<FileOutcome, AppError> {
    if move_to_failed(path, failed_dir, file_name, disambig)? {
        Ok(FileOutcome::Failed {
            error_code,
            details,
        })
    } else {
        Ok(FileOutcome::Skipped(format!(
            "Fichier « {file_name} » disparu pendant le traitement (race) — ignoré ce tour."
        )))
    }
}

/// Déplace un fichier vers `failed/` sous `{stem}_{disambig}.{ext}`.
///
/// `{stem}` est extrait via `Path::file_stem()` (filename seul, jamais de
/// composant de chemin — un nom malveillant `../../x.pdf` ne s'échappe pas).
/// Le suffixe `{disambig}` (hash8 du contenu ou UUID8) évite qu'un 2ᵉ fichier
/// homonyme écrase le 1ᵉʳ (`rename` POSIX écrase atomiquement une cible homonyme).
///
/// Retour : `Ok(true)` = déplacé ; `Ok(false)` = source disparue (ENOENT, race) ;
/// `Err` = autre erreur IO (catastrophe → 500).
fn move_to_failed(
    path: &Path,
    failed_dir: &Path,
    file_name: &str,
    disambig: &str,
) -> Result<bool, AppError> {
    let p = Path::new(file_name);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty() && !s.contains('/') && !s.contains(".."))
        .unwrap_or("file");
    let target_name = match p.extension().and_then(|e| e.to_str()) {
        Some(ext) => format!("{stem}_{disambig}.{}", ext.to_ascii_lowercase()),
        None => format!("{stem}_{disambig}"),
    };
    let target = failed_dir.join(target_name);
    match std::fs::rename(path, &target) {
        Ok(()) => Ok(true),
        // Source disparue entre détection et déplacement → race bénigne (skip).
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        // Autre échec de disposal = IO catastrophe (le fichier resterait dans
        // l'inbox et serait re-traité en boucle) → 500.
        Err(e) => Err(AppError::Internal(format!(
            "inbox import: déplacement vers failed/: {e}"
        ))),
    }
}

/// Supprime un fichier inbox traité avec succès (tolère `ENOENT`, idempotent).
fn remove_inbox_file(path: &Path) {
    if let Err(e) = std::fs::remove_file(path)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!("inbox import: suppression du fichier traité échouée: {e}");
    }
}
