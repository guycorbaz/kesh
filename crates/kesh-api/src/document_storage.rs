//! Stockage des justificatifs importés (Story 12-5b — #194).
//!
//! Les copies des fichiers de factures importées sont écrites sur le filesystem
//! (`KESH_DOCUMENTS_DIR`, hors DB) sous le nom **`{sha256hex}.{ext}`** — le nom
//! d'origine n'est **jamais** utilisé pour construire un chemin (anti-traversal),
//! il n'est conservé qu'en colonne DB (affichage).
//!
//! L'orchestration (lecture inbox, idempotence, déplacement `failed/`) et le
//! mapping HTTP des erreurs (404/410 sur fichier absent) vivent en 12-5c ; ce
//! module n'expose que les primitives `store_document` / `read_document`.

use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

use kesh_db::entities::DocumentMeta;
use sha2::{Digest, Sha256};

/// Compteur process-unique pour nommer les fichiers temporaires d'écriture
/// atomique (pas de `Math.random`/horloge disponibles côté config — cf. helper
/// de test). Combiné au PID, garantit l'unicité même sous appels concurrents.
static TMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Erreur de lecture d'un justificatif, distinguant le fichier **absent**
/// (`NotFound` — mappé 404/410 en 12-5c, jamais 500) d'une autre erreur I/O.
#[derive(Debug)]
pub enum ReadDocumentError {
    /// Le fichier n'existe pas sur disque (ex. après un restore métadonnée-seule, L1).
    NotFound,
    /// `storage_path` invalide (traversal / composant non attendu).
    InvalidPath,
    /// Autre erreur d'I/O.
    Io(std::io::Error),
}

impl std::fmt::Display for ReadDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotFound => write!(f, "justificatif non disponible (fichier absent)"),
            Self::InvalidPath => write!(f, "chemin de stockage invalide"),
            Self::Io(e) => write!(f, "erreur I/O: {e}"),
        }
    }
}

impl std::error::Error for ReadDocumentError {}

/// Type MIME canonique pour une extension de la liste blanche.
pub fn mime_for_ext(ext: &str) -> &'static str {
    match ext.to_ascii_lowercase().as_str() {
        "pdf" => "application/pdf",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

/// `true` si `ext` est un composant d'extension sûr (alphanumérique court, pas de
/// séparateur de chemin) — garde-fou anti-traversal sur le nommage archivé.
fn is_safe_ext(ext: &str) -> bool {
    !ext.is_empty() && ext.len() <= 8 && ext.chars().all(|c| c.is_ascii_alphanumeric())
}

/// `true` si `storage_path` est un nom de fichier simple `{hex}.{ext}` sans
/// composant de traversal (pas de `/`, `\`, `..`, ni chemin absolu).
fn is_safe_storage_path(storage_path: &str) -> bool {
    !storage_path.is_empty()
        && !storage_path.contains('/')
        && !storage_path.contains('\\')
        && !storage_path.contains("..")
        && Path::new(storage_path).components().count() == 1
}

/// Écrit `bytes` dans `documents_dir` sous `{sha256hex}.{ext}` et retourne les
/// métadonnées du justificatif. Crée `documents_dir` s'il n'existe pas.
///
/// Le nommage est dérivé du **hash du contenu** (jamais de `original_filename`)
/// → pas de collision, pas de traversal. Réécrire un contenu identique est
/// idempotent (même chemin).
///
/// # Errors
/// - `InvalidInput` si `ext` n'est pas une extension sûre.
/// - Toute erreur I/O de création de répertoire ou d'écriture.
pub fn store_document(
    documents_dir: &Path,
    bytes: &[u8],
    ext: &str,
    original_filename: &str,
    mime_type: &str,
) -> std::io::Result<DocumentMeta> {
    if !is_safe_ext(ext) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("extension non sûre: {ext:?}"),
        ));
    }
    // Un contenu vide ne porte aucun QR (le décodage échoue en amont) et
    // violerait le CHECK `byte_size > 0` côté DB → refus explicite à la primitive
    // pour un invariant cohérent (code-review Pass 3).
    if bytes.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "contenu vide (0 octet)",
        ));
    }

    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let sha256 = format!("{:x}", hasher.finalize());

    let storage_path = format!("{sha256}.{}", ext.to_ascii_lowercase());

    std::fs::create_dir_all(documents_dir)?;
    let full_path = documents_dir.join(&storage_path);

    // Écriture atomique : fichier temporaire unique → `fsync` → `rename(2)`
    // (atomique sur le même filesystem POSIX). Un crash entre la troncature et la
    // fin d'écriture ne peut donc PAS laisser un justificatif partiel sous le nom
    // SHA-256 « valide » (que `read_document` servirait ensuite sans le détecter).
    let tmp_path = documents_dir.join(format!(
        ".{sha256}.{}.{}.tmp",
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let write_result = (|| -> std::io::Result<()> {
        let mut f = std::fs::File::create(&tmp_path)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        std::fs::rename(&tmp_path, &full_path)
    })();
    if write_result.is_err() {
        let _ = std::fs::remove_file(&tmp_path); // nettoyage best-effort du tmp
    }
    write_result?;

    Ok(DocumentMeta {
        storage_path,
        original_filename: original_filename.to_string(),
        sha256,
        mime_type: mime_type.to_string(),
        byte_size: bytes.len() as i64,
    })
}

/// Lit un justificatif archivé. `storage_path` est le chemin **relatif** stocké
/// en DB (`{hex}.{ext}`). Distingue le fichier absent (`NotFound`) d'une autre
/// erreur I/O pour que 12-5c renvoie 404/410 plutôt que 500 (F7).
///
/// # Errors
/// - [`ReadDocumentError::InvalidPath`] si `storage_path` contient un composant
///   de traversal.
/// - [`ReadDocumentError::NotFound`] si le fichier n'existe pas.
/// - [`ReadDocumentError::Io`] pour toute autre erreur.
pub fn read_document(
    documents_dir: &Path,
    storage_path: &str,
) -> Result<Vec<u8>, ReadDocumentError> {
    if !is_safe_storage_path(storage_path) {
        return Err(ReadDocumentError::InvalidPath);
    }
    let full_path = documents_dir.join(storage_path);
    match std::fs::read(&full_path) {
        Ok(bytes) => Ok(bytes),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(ReadDocumentError::NotFound),
        Err(e) => Err(ReadDocumentError::Io(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        // Nom déterministe par test (pas de Math.random / Date dispo) ; nettoyé en début de test.
        p.push(format!("kesh-doc-store-test-{tag}"));
        let _ = std::fs::remove_dir_all(&p);
        p
    }

    #[test]
    fn store_then_read_roundtrip() {
        let dir = temp_dir("roundtrip");
        let bytes = b"%PDF-1.4 fake content";
        let meta = store_document(&dir, bytes, "pdf", "facture.pdf", "application/pdf").unwrap();

        // Nommage = {sha256}.pdf, pas le nom d'origine.
        assert!(meta.storage_path.ends_with(".pdf"));
        assert_eq!(meta.storage_path.len(), 64 + 4); // 64 hex + ".pdf"
        assert_eq!(meta.original_filename, "facture.pdf");
        assert_eq!(meta.byte_size, bytes.len() as i64);
        assert_eq!(meta.sha256.len(), 64);

        let read = read_document(&dir, &meta.storage_path).unwrap();
        assert_eq!(read, bytes);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn same_content_yields_same_path() {
        let dir = temp_dir("idempotent");
        let a = store_document(&dir, b"abc", "png", "a.png", "image/png").unwrap();
        let b = store_document(&dir, b"abc", "png", "b.png", "image/png").unwrap();
        assert_eq!(a.storage_path, b.storage_path);
        assert_eq!(a.sha256, b.sha256);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_missing_file_yields_not_found() {
        let dir = temp_dir("missing");
        std::fs::create_dir_all(&dir).unwrap();
        let err = read_document(&dir, "deadbeef.pdf").unwrap_err();
        assert!(matches!(err, ReadDocumentError::NotFound));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_rejects_traversal() {
        let dir = temp_dir("traversal");
        assert!(matches!(
            read_document(&dir, "../etc/passwd"),
            Err(ReadDocumentError::InvalidPath)
        ));
        assert!(matches!(
            read_document(&dir, "sub/file.pdf"),
            Err(ReadDocumentError::InvalidPath)
        ));
    }

    #[test]
    fn store_rejects_unsafe_ext() {
        let dir = temp_dir("badext");
        assert!(store_document(&dir, b"x", "../p", "x", "x").is_err());
        assert!(store_document(&dir, b"x", "", "x", "x").is_err());
    }

    #[test]
    fn store_rejects_empty_content() {
        let dir = temp_dir("empty");
        let err = store_document(&dir, b"", "pdf", "vide.pdf", "application/pdf").unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn mime_for_ext_maps_known() {
        assert_eq!(mime_for_ext("pdf"), "application/pdf");
        assert_eq!(mime_for_ext("PNG"), "image/png");
        assert_eq!(mime_for_ext("jpeg"), "image/jpeg");
        assert_eq!(mime_for_ext("xyz"), "application/octet-stream");
    }
}
