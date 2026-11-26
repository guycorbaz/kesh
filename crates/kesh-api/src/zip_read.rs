//! Lecture bornée d'une archive ZIP reçue en mémoire — Story 39-11e.
//!
//! Deux usages :
//!
//! - [`read_bounded`], la lecture d'**une** entrée à travers `take(max + 1)`,
//!   partagée avec l'import de sauvegarde (`admin_backup::import`) : la borne
//!   porte sur les octets **réellement produits** par la décompression, jamais
//!   sur la taille que déclare l'en-tête ZIP (déclarative) ;
//! - [`extract_csv_entries`], le développement d'un zip de relevés (le zip que
//!   Wise fait télécharger) en ses fichiers CSV, avec les bornes et les refus
//!   de la Story 39-11e (D3, D4).
//!
//! Rien n'est écrit sur disque : tout se décompresse en mémoire. Les fonctions
//! sont **synchrones** ; un appelant asynchrone les exécute en
//! `tokio::task::spawn_blocking`.
//!
//! Angles morts assumés, les mêmes que ceux de la sauvegarde :
//! `ZipArchive::new` décode tout le répertoire central avant qu'on puisse
//! compter les entrées (sa mémoire n'est bornée que par la taille reçue, donc
//! par la limite du corps de la requête) ; deux entrées de même nom, le crate
//! n'en garde qu'une, sans le dire.

use std::io::{self, Cursor, Read};

use zip::{CompressionMethod, ZipArchive};

/// Lit `entry` à travers `take(max + 1)` : `Ok(None)` si elle produit plus de
/// `max` octets (rien n'est alloué au-delà de `max + 1`), `Ok(Some(octets))`
/// sinon. Une erreur de lecture (CRC faux, flux compressé corrompu) est
/// rendue telle quelle ; lue jusqu'au bout, une entrée a vu son CRC contrôlé.
pub fn read_bounded(entry: impl Read, max: u64) -> io::Result<Option<Vec<u8>>> {
    let mut buf = Vec::new();
    entry.take(max.saturating_add(1)).read_to_end(&mut buf)?;
    Ok((buf.len() as u64 <= max).then_some(buf))
}

/// Les octets commencent par une signature ZIP : en-tête local
/// (`PK\x03\x04`) ou fin de répertoire central d'une archive vide
/// (`PK\x05\x06`). Un relevé CSV de Wise commence par `TransferWise ID` (ou un
/// BOM) : jamais par ces quatre octets.
pub fn looks_like_zip(bytes: &[u8]) -> bool {
    bytes.starts_with(b"PK\x03\x04") || bytes.starts_with(b"PK\x05\x06")
}

/// Raison fermée d'un refus d'archive (`details.reason` de `422
/// WISE_ZIP_INVALID`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZipRefusalReason {
    /// Archive illisible : répertoire central faux, entrée corrompue, CRC
    /// faux à la lecture.
    Unreadable,
    /// Plus d'entrées que la borne (`max`).
    TooManyEntries,
    /// Taille décompressée cumulée au-delà de la borne (`maxBytes`).
    TooLarge,
    /// Nom d'entrée absolu, sortant de la racine, avec `\` ou `:`.
    UnsafePath,
    /// Entrée retenue chiffrée.
    Encrypted,
    /// Entrée retenue d'une méthode autre que *stored* ou *deflate*.
    UnsupportedCompression,
    /// Aucune entrée retenue.
    NoCsv,
}

impl ZipRefusalReason {
    /// Code stable, en `SCREAMING_SNAKE_CASE`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Unreadable => "UNREADABLE",
            Self::TooManyEntries => "TOO_MANY_ENTRIES",
            Self::TooLarge => "TOO_LARGE",
            Self::UnsafePath => "UNSAFE_PATH",
            Self::Encrypted => "ENCRYPTED",
            Self::UnsupportedCompression => "UNSUPPORTED_COMPRESSION",
            Self::NoCsv => "NO_CSV",
        }
    }
}

/// Refus d'une archive : la raison et, quand il y en a une, l'entrée en cause
/// (son nom brut, tel qu'écrit dans l'archive ; à borner avant de l'afficher).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZipRefusal {
    pub reason: ZipRefusalReason,
    pub entry: Option<String>,
}

impl ZipRefusal {
    fn new(reason: ZipRefusalReason, entry: Option<&str>) -> Self {
        Self {
            reason,
            entry: entry.map(str::to_string),
        }
    }
}

/// Un fichier CSV retenu d'une archive : son nom (brut) et ses octets.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveCsv {
    pub name: String,
    pub bytes: Vec<u8>,
}

/// Ce que rend le développement d'une archive : les CSV retenus, **par nom**,
/// et les noms des entrées ignorées, dans l'ordre du répertoire.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExtractedCsv {
    pub csvs: Vec<ArchiveCsv>,
    pub ignored: Vec<String>,
}

/// Le nom finit par `.csv`, sans casse (comparaison d'octets ASCII : jamais de
/// découpe au milieu d'un caractère).
fn has_csv_extension(name: &str) -> bool {
    let b = name.as_bytes();
    b.len() >= 4 && b[b.len() - 4..].eq_ignore_ascii_case(b".csv")
}

/// Développe une archive ZIP en ses fichiers CSV (Story 39-11e, D3 et D4).
///
/// **Retenues** : les entrées fichier, à la racine (aucun `/` dans le nom),
/// dont le nom finit par `.csv` sans casse ; toute autre entrée (dossier,
/// entrée dans un dossier, autre extension, zip imbriqué) est **ignorée**,
/// jamais ouverte, et nommée dans [`ExtractedCsv::ignored`].
///
/// **Ordre des contrôles**, la première faute l'emportant :
///
/// 1. ouverture de l'archive → [`ZipRefusalReason::Unreadable`] ;
/// 2. plus de `max_entries` entrées (répertoire central, avant toute
///    décompression) → [`ZipRefusalReason::TooManyEntries`] ;
/// 3. entrée par entrée, dans l'ordre du répertoire, **par ses métadonnées
///    seules** (`by_index_raw`, sans décompresser — `by_index` refuserait
///    d'ouvrir une entrée chiffrée ou de méthode non gérée et ferait dire
///    « illisible ») : le chemin sur **toutes** les entrées
///    ([`ZipRefusalReason::UnsafePath`] : nom absolu ou sortant de la racine,
///    `enclosed_name()` à `None` ; barre oblique inverse ; deux-points d'une
///    lettre de lecteur, que `enclosed_name()` ne refuse que sous Windows),
///    puis, sur les seules entrées retenues, le chiffrement
///    ([`ZipRefusalReason::Encrypted`]) et la méthode
///    ([`ZipRefusalReason::UnsupportedCompression`]) ;
/// 4. lecture des retenues, par nom, chacune lue jusqu'au bout (CRC contrôlé)
///    à travers [`read_bounded`] avec le **reste** de `budget` :
///    [`ZipRefusalReason::Unreadable`] sur une erreur de lecture,
///    [`ZipRefusalReason::TooLarge`] au-delà du reste ; le budget est diminué
///    des octets lus (il se partage entre les archives d'une même requête) ;
/// 5. aucune entrée retenue → [`ZipRefusalReason::NoCsv`].
pub fn extract_csv_entries(
    bytes: &[u8],
    max_entries: usize,
    budget: &mut u64,
) -> Result<ExtractedCsv, ZipRefusal> {
    use ZipRefusalReason::*;
    let mut archive =
        ZipArchive::new(Cursor::new(bytes)).map_err(|_| ZipRefusal::new(Unreadable, None))?;
    if archive.len() > max_entries {
        return Err(ZipRefusal::new(TooManyEntries, None));
    }

    let mut kept: Vec<(String, usize)> = Vec::new();
    let mut ignored: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let entry = archive
            .by_index_raw(i)
            .map_err(|_| ZipRefusal::new(Unreadable, None))?;
        let name = entry.name().to_string();
        if entry.enclosed_name().is_none() || name.contains('\\') || name.contains(':') {
            return Err(ZipRefusal::new(UnsafePath, Some(&name)));
        }
        let retained = !entry.is_dir() && !name.contains('/') && has_csv_extension(&name);
        if !retained {
            ignored.push(name);
            continue;
        }
        if entry.encrypted() {
            return Err(ZipRefusal::new(Encrypted, Some(&name)));
        }
        if !matches!(
            entry.compression(),
            CompressionMethod::Stored | CompressionMethod::Deflated
        ) {
            return Err(ZipRefusal::new(UnsupportedCompression, Some(&name)));
        }
        kept.push((name, i));
    }
    kept.sort();

    let mut csvs = Vec::with_capacity(kept.len());
    for (name, i) in kept {
        let entry = archive
            .by_index(i)
            .map_err(|_| ZipRefusal::new(Unreadable, Some(&name)))?;
        match read_bounded(entry, *budget) {
            Ok(Some(buf)) => {
                *budget -= buf.len() as u64;
                csvs.push(ArchiveCsv { name, bytes: buf });
            }
            Ok(None) => return Err(ZipRefusal::new(TooLarge, Some(&name))),
            Err(_) => return Err(ZipRefusal::new(Unreadable, Some(&name))),
        }
    }
    if csvs.is_empty() {
        return Err(ZipRefusal::new(NoCsv, None));
    }
    Ok(ExtractedCsv { csvs, ignored })
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use zip::write::SimpleFileOptions;

    use super::*;

    const MAX: usize = 64;
    const BUDGET: u64 = 1024 * 1024;

    /// Une archive fabriquée : (nom, octets, méthode), ou un dossier quand le
    /// nom finit par `/`.
    fn archive(entries: &[(&str, &[u8], CompressionMethod)]) -> Vec<u8> {
        let mut w = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for (name, bytes, method) in entries {
            let options = SimpleFileOptions::default().compression_method(*method);
            if name.ends_with('/') {
                w.add_directory(*name, options).unwrap();
            } else {
                w.start_file(*name, options).unwrap();
                w.write_all(bytes).unwrap();
            }
        }
        w.finish().unwrap().into_inner()
    }

    fn deflated(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let v: Vec<_> = entries
            .iter()
            .map(|(n, b)| (*n, *b, CompressionMethod::Deflated))
            .collect();
        archive(&v)
    }

    fn extract(bytes: &[u8]) -> Result<ExtractedCsv, ZipRefusal> {
        let mut budget = BUDGET;
        extract_csv_entries(bytes, MAX, &mut budget)
    }

    fn reason(bytes: &[u8]) -> (ZipRefusalReason, Option<String>) {
        let e = extract(bytes).unwrap_err();
        (e.reason, e.entry)
    }

    /// Réécrit un champ de deux octets de chaque en-tête (local à `local`,
    /// central à `central`, décalages depuis la signature).
    fn patch_u16(bytes: &mut [u8], local: usize, central: usize, value: u16) {
        patch(bytes, b"PK\x03\x04", local, &value.to_le_bytes());
        patch(bytes, b"PK\x01\x02", central, &value.to_le_bytes());
    }

    fn patch(bytes: &mut [u8], sig: &[u8], offset: usize, value: &[u8]) {
        let at: Vec<usize> = bytes
            .windows(4)
            .enumerate()
            .filter(|(_, w)| *w == sig)
            .map(|(i, _)| i)
            .collect();
        assert!(!at.is_empty());
        for i in at {
            bytes[i + offset..i + offset + value.len()].copy_from_slice(value);
        }
    }

    #[test]
    fn la_signature_d_un_zip() {
        assert!(looks_like_zip(&deflated(&[("a.csv", b"x")])));
        assert!(looks_like_zip(b"PK\x05\x06\0\0"));
        assert!(!looks_like_zip(b"TransferWise ID,Date"));
        assert!(!looks_like_zip(b"\xEF\xBB\xBFTransferWise ID"));
        assert!(!looks_like_zip(b"PK"));
    }

    #[test]
    fn read_bounded_s_arrete_a_la_borne() {
        assert_eq!(read_bounded(&b"abc"[..], 3).unwrap(), Some(b"abc".to_vec()));
        assert_eq!(read_bounded(&b"abcd"[..], 3).unwrap(), None);
        assert_eq!(read_bounded(&b""[..], 0).unwrap(), Some(vec![]));
    }

    #[test]
    fn les_csv_a_la_racine_sont_retenus_par_nom_le_reste_ignore() {
        let zip = archive(&[
            (
                "statement_9_USD_2020-01-01_2020-12-31.csv",
                b"u",
                CompressionMethod::Deflated,
            ),
            ("__MACOSX/", b"", CompressionMethod::Stored),
            (
                "__MACOSX/._statement.csv",
                b"m",
                CompressionMethod::Deflated,
            ),
            (
                "statement_9_CHF_2020-01-01_2020-12-31.CSV",
                b"c",
                CompressionMethod::Stored,
            ),
            ("releve.pdf", b"p", CompressionMethod::Deflated),
            ("imbrique.zip", b"PK\x03\x04", CompressionMethod::Stored),
            (
                "a/../dans-un-dossier.csv",
                b"d",
                CompressionMethod::Deflated,
            ),
            ("./point.csv", b"d", CompressionMethod::Deflated),
        ]);
        let got = extract(&zip).unwrap();
        let noms: Vec<&str> = got.csvs.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            noms,
            [
                "statement_9_CHF_2020-01-01_2020-12-31.CSV",
                "statement_9_USD_2020-01-01_2020-12-31.csv"
            ]
        );
        assert_eq!(got.csvs[0].bytes, b"c");
        assert_eq!(
            got.ignored,
            [
                "__MACOSX/",
                "__MACOSX/._statement.csv",
                "releve.pdf",
                "imbrique.zip",
                "a/../dans-un-dossier.csv",
                "./point.csv"
            ]
        );
    }

    #[test]
    fn une_archive_sans_csv_ou_vide() {
        assert_eq!(
            reason(&deflated(&[("a.pdf", b"x"), ("b.xml", b"y")])),
            (ZipRefusalReason::NoCsv, None)
        );
        // L'archive vide : la seule fin de répertoire central.
        let vide = zip::ZipWriter::new(Cursor::new(Vec::new()))
            .finish()
            .unwrap()
            .into_inner();
        assert!(looks_like_zip(&vide));
        assert_eq!(reason(&vide).0, ZipRefusalReason::NoCsv);
    }

    #[test]
    fn une_archive_illisible() {
        assert_eq!(
            reason(b"PK\x03\x04pas une archive").0,
            ZipRefusalReason::Unreadable
        );
        // CRC faux : contrôlé en fin de lecture.
        let mut zip = archive(&[("a.csv", b"contenu", CompressionMethod::Stored)]);
        patch(&mut zip, b"PK\x03\x04", 14, &[0, 0, 0, 0]);
        patch(&mut zip, b"PK\x01\x02", 16, &[0, 0, 0, 0]);
        assert_eq!(
            reason(&zip),
            (ZipRefusalReason::Unreadable, Some("a.csv".into()))
        );
    }

    #[test]
    fn soixante_quatre_entrees_admises_soixante_cinq_refusees() {
        let noms: Vec<String> = (0..65).map(|i| format!("r{i:02}.csv")).collect();
        let entrees: Vec<(&str, &[u8])> = noms.iter().map(|n| (n.as_str(), &b"x"[..])).collect();
        let got = extract(&deflated(&entrees[..64])).unwrap();
        assert_eq!(got.csvs.len(), 64);
        assert!(got.csvs.iter().all(|c| c.bytes == b"x"));
        assert_eq!(
            reason(&deflated(&entrees)),
            (ZipRefusalReason::TooManyEntries, None)
        );
    }

    #[test]
    fn les_chemins_dangereux_refusent_l_archive() {
        for nom in [
            "../x.csv", "/x.csv", "a\\x.csv", "C:x.csv", "C:/x.csv", "../a.pdf",
        ] {
            let zip = deflated(&[("bon.csv", b"x"), (nom, b"y")]);
            assert_eq!(
                reason(&zip),
                (ZipRefusalReason::UnsafePath, Some(nom.to_string())),
                "{nom}"
            );
        }
    }

    #[test]
    fn une_entree_retenue_chiffree_ou_d_une_autre_methode() {
        // Indicateur de chiffrement (bit 0 des drapeaux) posé sur l'entrée.
        let mut zip = archive(&[("a.csv", b"contenu", CompressionMethod::Stored)]);
        patch_u16(&mut zip, 6, 8, 1);
        assert_eq!(
            reason(&zip),
            (ZipRefusalReason::Encrypted, Some("a.csv".into()))
        );
        // Méthode 12 (bzip2), non gérée par le crate tel qu'il est configuré.
        let mut zip = archive(&[("a.csv", b"contenu", CompressionMethod::Stored)]);
        patch_u16(&mut zip, 8, 10, 12);
        assert_eq!(
            reason(&zip),
            (
                ZipRefusalReason::UnsupportedCompression,
                Some("a.csv".into())
            )
        );
        // La même méthode sur une entrée ignorée : jamais ouverte, sans refus.
        let mut zip = archive(&[("a.pdf", b"contenu", CompressionMethod::Stored)]);
        patch_u16(&mut zip, 8, 10, 12);
        assert_eq!(reason(&zip).0, ZipRefusalReason::NoCsv);
    }

    #[test]
    fn une_bombe_est_refusee_sur_les_octets_produits() {
        // 4 Mio de zéros compressés en quelques Ko, borne à 1 Mio.
        let zeros = vec![0u8; 4 * 1024 * 1024];
        let zip = deflated(&[("bombe.csv", &zeros)]);
        assert!(zip.len() < 64 * 1024, "{}", zip.len());
        assert_eq!(
            reason(&zip),
            (ZipRefusalReason::TooLarge, Some("bombe.csv".into()))
        );
        // L'en-tête ment : taille décompressée déclarée de 10 octets (central
        // et local). La borne porte sur les octets produits : jamais admise.
        let mut menteuse = zip.clone();
        patch(&mut menteuse, b"PK\x03\x04", 22, &10u32.to_le_bytes());
        patch(&mut menteuse, b"PK\x01\x02", 24, &10u32.to_le_bytes());
        let e = extract(&menteuse).unwrap_err();
        assert!(
            matches!(
                e.reason,
                ZipRefusalReason::TooLarge | ZipRefusalReason::Unreadable
            ),
            "{e:?}"
        );
    }

    #[test]
    fn la_borne_se_partage_entre_les_archives_d_une_requete() {
        let six = vec![b'x'; 600];
        let a = deflated(&[("a.csv", &six)]);
        let b = deflated(&[("b.csv", &six)]);
        let mut budget = 1000;
        assert!(extract_csv_entries(&a, MAX, &mut budget).is_ok());
        assert_eq!(budget, 400);
        assert_eq!(
            extract_csv_entries(&b, MAX, &mut budget).unwrap_err(),
            ZipRefusal::new(ZipRefusalReason::TooLarge, Some("b.csv"))
        );
        // Chacune seule passe.
        let mut budget = 1000;
        assert!(extract_csv_entries(&b, MAX, &mut budget).is_ok());
    }

    #[test]
    fn l_ordre_des_controles() {
        // Chemin faux sur une entrée ignorée, avant le chiffrement d'une retenue
        // placée après elle ; et avant `NO_CSV`.
        let zip = deflated(&[("../a.pdf", b"x")]);
        assert_eq!(reason(&zip).0, ZipRefusalReason::UnsafePath);
        // Trop d'entrées l'emporte sur un chemin faux.
        let noms: Vec<String> = (0..65).map(|i| format!("../r{i}.csv")).collect();
        let entrees: Vec<(&str, &[u8])> = noms.iter().map(|n| (n.as_str(), &b"x"[..])).collect();
        assert_eq!(
            reason(&deflated(&entrees)).0,
            ZipRefusalReason::TooManyEntries
        );
    }

    #[test]
    fn les_codes_sont_stables() {
        use ZipRefusalReason::*;
        let codes: Vec<&str> = [
            Unreadable,
            TooManyEntries,
            TooLarge,
            UnsafePath,
            Encrypted,
            UnsupportedCompression,
            NoCsv,
        ]
        .map(ZipRefusalReason::as_str)
        .to_vec();
        assert_eq!(
            codes,
            [
                "UNREADABLE",
                "TOO_MANY_ENTRIES",
                "TOO_LARGE",
                "UNSAFE_PATH",
                "ENCRYPTED",
                "UNSUPPORTED_COMPRESSION",
                "NO_CSV"
            ]
        );
    }
}
