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
//!
//! Story 31-1a (#503) — la sauvegarde complète emporte ces fichiers : elle les
//! lit par [`open_document`] (jamais par une jointure de chemin hors de ce
//! module) et les restaure par [`restore_document`], qui partage avec
//! [`store_document`] le même cœur d'écriture atomique.

use std::fs::File;
use std::io::{Read, Write};
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
pub(crate) fn is_safe_storage_path(storage_path: &str) -> bool {
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
    write_atomically(documents_dir, &storage_path, &sha256, |f| {
        f.write_all(bytes)
    })?;

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

/// Cœur d'écriture atomique d'un fichier du dossier des documents, commun à
/// [`store_document`] et à [`restore_document`] (Story 31-1a, C-31-1-6).
///
/// Écrit par `fill` dans un temporaire unique du même dossier
/// (`.{tag}.{pid}.{n}.tmp`), le synchronise (`sync_all`), puis le renomme en
/// `final_name` (`rename(2)`, atomique sur un même système de fichiers, et qui
/// **remplace** un fichier existant de ce nom). Un échec à n'importe quelle
/// étape supprime le temporaire au mieux : un crash ne laisse jamais un
/// contenu partiel sous un nom « valide ». Le dossier doit exister.
fn write_atomically<T, E: From<std::io::Error>>(
    documents_dir: &Path,
    final_name: &str,
    tag: &str,
    fill: impl FnOnce(&mut File) -> Result<T, E>,
) -> Result<T, E> {
    let tmp_path = documents_dir.join(format!(
        ".{tag}.{}.{}.tmp",
        std::process::id(),
        TMP_COUNTER.fetch_add(1, Ordering::Relaxed)
    ));
    let result = (|| -> Result<T, E> {
        let mut f = File::create(&tmp_path)?;
        let value = fill(&mut f)?;
        f.sync_all()?;
        std::fs::rename(&tmp_path, documents_dir.join(final_name))?;
        Ok(value)
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp_path); // nettoyage best-effort du tmp
    }
    result
}

/// Story 31-1a (#503, AC3) — ouvre un fichier du dossier des documents, après
/// avoir contrôlé son nom par [`is_safe_storage_path`] : **aucun** chemin venu
/// de la base n'est joint au dossier ailleurs que dans ce module.
///
/// # Errors
/// - [`ReadDocumentError::InvalidPath`] si `storage_path` n'est pas un nom
///   simple (le disque n'est pas touché) ;
/// - [`ReadDocumentError::NotFound`] si le fichier n'existe pas ;
/// - [`ReadDocumentError::Io`] pour toute autre erreur d'ouverture.
pub(crate) fn open_document(
    documents_dir: &Path,
    storage_path: &str,
) -> Result<File, ReadDocumentError> {
    if !is_safe_storage_path(storage_path) {
        return Err(ReadDocumentError::InvalidPath);
    }
    match File::open(documents_dir.join(storage_path)) {
        Ok(f) => Ok(f),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Err(ReadDocumentError::NotFound),
        Err(e) => Err(ReadDocumentError::Io(e)),
    }
}

/// Empreinte SHA-256 (hex) et nombre d'octets de ce que rend `source`, lu par
/// blocs de 64 Kio — jamais le fichier entier en mémoire.
pub(crate) fn sha256_of(mut source: impl Read) -> std::io::Result<(String, u64)> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut total = 0u64;
    loop {
        let n = source.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        total += n as u64;
    }
    Ok((format!("{:x}", hasher.finalize()), total))
}

/// Story 31-1a (AC8, AC9) — `true` si le fichier `storage_path` existe dans le
/// dossier des documents et que son contenu a l'empreinte `sha256` déclarée.
/// Un nom hors forme, un fichier absent, illisible ou altéré rendent `false`.
pub(crate) fn document_is_intact(documents_dir: &Path, storage_path: &str, sha256: &str) -> bool {
    match open_document(documents_dir, storage_path) {
        Ok(f) => matches!(sha256_of(f), Ok((sha, _)) if sha == sha256),
        Err(_) => false,
    }
}

/// Story 31-1a (AC8) — ce qu'a fait [`restore_document`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RestoreOutcome {
    /// Le fichier, absent, a été écrit.
    Written,
    /// Un fichier de ce nom était présent mais **altéré** (ou illisible) : il
    /// a été remplacé atomiquement.
    Replaced,
    /// Un fichier de ce nom était déjà présent **et intact** : laissé tel quel.
    AlreadyIntact,
}

/// Story 31-1a (AC8) — pourquoi [`restore_document`] n'a rien écrit.
#[derive(Debug)]
pub(crate) enum RestoreDocumentError {
    /// Nom hors forme (jamais joint au dossier).
    InvalidPath,
    /// La source dépasse la taille déclarée, est plus courte, ou n'a pas
    /// l'empreinte déclarée : rien n'est renommé.
    ContentMismatch(String),
    /// Erreur d'entrée-sortie (dossier non inscriptible, disque plein…).
    Io(std::io::Error),
}

impl From<std::io::Error> for RestoreDocumentError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl std::fmt::Display for RestoreDocumentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPath => write!(f, "chemin de stockage invalide"),
            Self::ContentMismatch(detail) => write!(f, "contenu inattendu : {detail}"),
            Self::Io(e) => write!(f, "erreur I/O: {e}"),
        }
    }
}

/// Story 31-1a (#503, AC8) — restaure un fichier du dossier des documents
/// depuis `source` (l'entrée `files/<storage_path>` d'une sauvegarde), **en
/// flux** : la source est copiée dans le temporaire à travers
/// `take(byte_size + 1)` et un `Sha256`, puis taille et empreinte sont
/// comparées **avant** le renommage — jamais le fichier entier en mémoire.
///
/// Un fichier de même nom déjà présent et intact est laissé
/// ([`RestoreOutcome::AlreadyIntact`]) ; présent mais altéré, il est remplacé
/// atomiquement. Crée le dossier s'il manque. Le dossier lui-même n'est **pas**
/// synchronisé ici : l'appelant le fait une fois, après le dernier fichier.
pub(crate) fn restore_document(
    documents_dir: &Path,
    storage_path: &str,
    source: impl Read,
    byte_size: u64,
    expected_sha256: &str,
) -> Result<RestoreOutcome, RestoreDocumentError> {
    if !is_safe_storage_path(storage_path) {
        return Err(RestoreDocumentError::InvalidPath);
    }
    let present = match open_document(documents_dir, storage_path) {
        Ok(f) => {
            if matches!(sha256_of(f), Ok((sha, _)) if sha == expected_sha256) {
                return Ok(RestoreOutcome::AlreadyIntact);
            }
            true
        }
        Err(_) => false,
    };
    let ecrit = if present {
        RestoreOutcome::Replaced
    } else {
        RestoreOutcome::Written
    };
    std::fs::create_dir_all(documents_dir)?;
    write_atomically(documents_dir, storage_path, expected_sha256, |f| {
        let mut hasher = Sha256::new();
        let mut borne = source.take(byte_size.saturating_add(1));
        let mut buf = vec![0u8; 64 * 1024];
        let mut total = 0u64;
        loop {
            let n = borne.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            f.write_all(&buf[..n])?;
            total += n as u64;
        }
        if total != byte_size {
            return Err(RestoreDocumentError::ContentMismatch(format!(
                "{total} octets lus, {byte_size} déclarés"
            )));
        }
        if format!("{:x}", hasher.finalize()) != expected_sha256 {
            return Err(RestoreDocumentError::ContentMismatch(
                "empreinte SHA-256 différente".into(),
            ));
        }
        Ok(ecrit)
    })
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

    // =======================================================================
    // Story 31-1a (#503) — lecture et restauration pour la sauvegarde
    // =======================================================================

    fn sha(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    /// Les fichiers du dossier (temporaires compris).
    fn contenu_du_dossier(dir: &Path) -> Vec<String> {
        let mut v: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .collect();
        v.sort();
        v
    }

    /// AC3 (M-a12) — `open_document` refuse tout nom qui n'est pas simple,
    /// **sans toucher au disque** : le dossier passé n'existe même pas, et
    /// `/etc/hostname` existe sur le poste.
    #[test]
    fn open_document_refuse_les_chemins_hors_forme() {
        let absent = Path::new("/nonexistent-kesh-31-1a");
        for chemin in ["../x", "/etc/hostname", "a/b", ""] {
            assert!(
                matches!(
                    open_document(absent, chemin),
                    Err(ReadDocumentError::InvalidPath)
                ),
                "{chemin:?} doit être refusé"
            );
        }
        assert!(matches!(
            open_document(absent, "abc.pdf"),
            Err(ReadDocumentError::NotFound)
        ));
    }

    /// AC8 — restauration en flux : écrit, puis laisse un fichier intact.
    #[test]
    fn restore_document_ecrit_puis_laisse_l_intact() {
        let dir = tempfile::tempdir().unwrap();
        let docs = dir.path().join("docs");
        let contenu = b"piece restauree";
        let nom = format!("{}.pdf", sha(contenu));
        let issue = restore_document(
            &docs,
            &nom,
            &contenu[..],
            contenu.len() as u64,
            &sha(contenu),
        )
        .unwrap();
        assert_eq!(issue, RestoreOutcome::Written);
        assert_eq!(std::fs::read(docs.join(&nom)).unwrap(), contenu);
        let issue = restore_document(
            &docs,
            &nom,
            &contenu[..],
            contenu.len() as u64,
            &sha(contenu),
        )
        .unwrap();
        assert_eq!(issue, RestoreOutcome::AlreadyIntact);
        assert_eq!(contenu_du_dossier(&docs), vec![nom]);
    }

    /// AC8 — un fichier présent mais altéré est remplacé.
    #[test]
    fn restore_document_remplace_un_fichier_altere() {
        let dir = tempfile::tempdir().unwrap();
        let contenu = b"bonne version";
        let nom = format!("{}.pdf", sha(contenu));
        std::fs::write(dir.path().join(&nom), b"version alteree").unwrap();
        let issue = restore_document(
            dir.path(),
            &nom,
            &contenu[..],
            contenu.len() as u64,
            &sha(contenu),
        )
        .unwrap();
        assert_eq!(issue, RestoreOutcome::Replaced);
        assert_eq!(std::fs::read(dir.path().join(&nom)).unwrap(), contenu);
    }

    /// AC8 — une source plus longue que sa taille déclarée (lecture bornée à
    /// `byte_size + 1`), plus courte, ou d'une autre empreinte : rien n'est
    /// renommé, aucun temporaire ne reste.
    #[test]
    fn restore_document_refuse_un_contenu_inattendu() {
        let dir = tempfile::tempdir().unwrap();
        let contenu = b"contenu declare";
        let nom = format!("{}.pdf", sha(contenu));
        for (source, taille) in [
            (&b"contenu declare et davantage"[..], contenu.len() as u64),
            (&b"contenu"[..], contenu.len() as u64),
            (&b"autre contenu!!"[..], contenu.len() as u64),
        ] {
            let err = restore_document(dir.path(), &nom, source, taille, &sha(contenu))
                .expect_err("contenu inattendu");
            assert!(
                matches!(err, RestoreDocumentError::ContentMismatch(_)),
                "{err}"
            );
        }
        assert!(contenu_du_dossier(dir.path()).is_empty());
        assert!(matches!(
            restore_document(dir.path(), "../x.pdf", &b"x"[..], 1, &sha(b"x")),
            Err(RestoreDocumentError::InvalidPath)
        ));
    }

    /// AC8 — un dossier des documents non inscriptible (ici, un fichier) rend
    /// une erreur d'entrée-sortie.
    #[test]
    fn restore_document_dossier_inutilisable() {
        let fichier = tempfile::NamedTempFile::new().unwrap();
        let contenu = b"x";
        let nom = format!("{}.pdf", sha(contenu));
        assert!(matches!(
            restore_document(fichier.path(), &nom, &contenu[..], 1, &sha(contenu)),
            Err(RestoreDocumentError::Io(_))
        ));
    }

    /// AC9 — `document_is_intact` : présent et intact seulement.
    #[test]
    fn document_is_intact_distingue_les_cas() {
        let dir = tempfile::tempdir().unwrap();
        let contenu = b"intact";
        let nom = format!("{}.pdf", sha(contenu));
        assert!(!document_is_intact(dir.path(), &nom, &sha(contenu)));
        std::fs::write(dir.path().join(&nom), contenu).unwrap();
        assert!(document_is_intact(dir.path(), &nom, &sha(contenu)));
        assert!(!document_is_intact(dir.path(), &nom, &sha(b"autre")));
        assert!(!document_is_intact(dir.path(), "../x.pdf", &sha(contenu)));
    }

    /// Fichiers `.rs` d'un dossier, récursivement.
    fn sources_rust(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
        for entree in std::fs::read_dir(dir).unwrap() {
            let p = entree.unwrap().path();
            if p.is_dir() {
                sources_rust(&p, out);
            } else if p.extension().is_some_and(|e| e == "rs") {
                out.push(p);
            }
        }
    }

    /// AC2 — inventaire clos des **écrivains** du dossier des documents : dans
    /// le code de production de tous les crates (`src/`, avant la première
    /// ligne `#[cfg(test)]`, lignes `//` écartées), les appelants de
    /// `store_document(` sont exactement `inbox_import.rs` et
    /// `routes/issued_invoice_pdf.rs`. Un appelant neuf écrit une pièce que la
    /// sauvegarde doit connaître : mettre à jour l'AC1 de la Story 31-1 (la
    /// lecture des fichiers référencés, `kesh_db::backup`), puis cette liste.
    #[test]
    fn appelants_de_store_document_inventaire_clos() {
        let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        let mut fichiers = Vec::new();
        for c in std::fs::read_dir(crates).unwrap() {
            let src = c.unwrap().path().join("src");
            if src.is_dir() {
                sources_rust(&src, &mut fichiers);
            }
        }
        assert!(
            fichiers.len() > 50,
            "assertion de montage : les sources des crates doivent être trouvées"
        );
        // `\b` : `restore_document(` n'est pas un appel de `store_document(`.
        let appel = regex::Regex::new(r"\bstore_document\(").unwrap();
        let mut appelants: Vec<String> = Vec::new();
        for f in fichiers {
            let texte = std::fs::read_to_string(&f).unwrap();
            let appelle = texte
                .lines()
                .take_while(|l| l.trim() != "#[cfg(test)]")
                .filter(|l| !l.trim_start().starts_with("//"))
                .any(|l| appel.is_match(l) && !l.contains("fn store_document("));
            if appelle {
                let rel = f
                    .strip_prefix(crates)
                    .unwrap()
                    .to_string_lossy()
                    .to_string();
                appelants.push(rel);
            }
        }
        appelants.sort();
        assert_eq!(
            appelants,
            vec![
                "kesh-api/src/inbox_import.rs".to_string(),
                "kesh-api/src/routes/issued_invoice_pdf.rs".to_string(),
            ],
            "écrivain neuf du dossier des documents : mettre à jour la lecture des \
             fichiers référencés (Story 31-1a, AC1 — kesh_db::backup), puis cette liste"
        );
    }
}
