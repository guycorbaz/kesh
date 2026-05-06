//! Détection d'encoding pour fichiers CSV bancaires (Story 8-2 §encoding-detection).
//!
//! Algorithme déterministe (R4 epic-8.md résolu) :
//! 0. **Empty file guard** — `bytes.is_empty()` → `CsvError::EmptyFile`.
//! 1. **BOM check** — `encoding_rs::Encoding::for_bom(bytes)` (lib gère
//!    elle-même les slices courts, **pas** `&bytes[..3]` qui panique sur
//!    fichiers 1-2 bytes).
//! 2. **Sans BOM, passe 1** — `std::str::from_utf8` strict (couvre ASCII pur
//!    + UTF-8 sans BOM).
//! 3. **Sans BOM, passe 2** — `chardetng::EncodingDetector` sur **1024 bytes
//!    max** (perf prévisible sur fichiers > 50 MB).
//!
//! Garde-fou petit fichier (< 64 bytes) : si passe 1 échoue, refus
//! `UnsupportedEncoding` avec `detected = None` (chardetng peu fiable
//! sur petits échantillons).

use crate::error::CsvError;

/// Encoding supporté par le parser CSV v0.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedEncoding {
    /// UTF-8 (avec ou sans BOM, après consommation BOM si présent).
    Utf8,
    /// ISO-8859-1 / windows-1252 (alias acceptés).
    Iso8859_1,
}

impl DetectedEncoding {
    /// Nom canonique pour comparaison vs `profile.encoding` et logging.
    pub fn as_str(&self) -> &'static str {
        match self {
            DetectedEncoding::Utf8 => "UTF-8",
            DetectedEncoding::Iso8859_1 => "ISO-8859-1",
        }
    }
}

/// Seuil minimum d'octets pour que `chardetng` produise un verdict fiable
/// (en-dessous, on refuse plutôt que de risquer un faux positif).
const CHARDETNG_MIN_SAMPLE: usize = 64;

/// Taille max de l'échantillon passé à chardetng (perf prévisible sur
/// fichiers gros, cohérent avec doc §encoding-detection priorité 2).
const CHARDETNG_MAX_SAMPLE: usize = 1024;

/// Détecte l'encoding du fichier CSV. Retourne aussi le nombre de bytes
/// consommés par le BOM (0 si pas de BOM).
pub fn detect_encoding(bytes: &[u8]) -> Result<(DetectedEncoding, usize), CsvError> {
    // Priorité 0 : empty file guard.
    if bytes.is_empty() {
        return Err(CsvError::EmptyFile {
            reason: "0 bytes".to_string(),
        });
    }

    // Priorité 1 : BOM check. `for_bom` accepte des slices courts, ne
    // panique pas sur fichiers 1-2 bytes (Pass 1 H3).
    if let Some((encoding, bom_len)) = encoding_rs::Encoding::for_bom(bytes) {
        if encoding == encoding_rs::UTF_8 {
            return Ok((DetectedEncoding::Utf8, bom_len));
        }
        // UTF-16 LE/BE ou autres BOMs → reject v0.1.
        return Err(CsvError::UnsupportedEncoding {
            detected: Some(encoding.name().to_string()),
        });
    }

    // Priorité 2 : passe UTF-8 strict (couvre ASCII pur AC #4 + UTF-8
    // sans BOM).
    if std::str::from_utf8(bytes).is_ok() {
        return Ok((DetectedEncoding::Utf8, 0));
    }

    // Garde-fou petit fichier : chardetng peu fiable < 64 bytes.
    if bytes.len() < CHARDETNG_MIN_SAMPLE {
        return Err(CsvError::UnsupportedEncoding { detected: None });
    }

    // Priorité 3 : chardetng sur 1024 bytes max (Pass 1 M12 cohérence
    // — `feed(&bytes[..min(1024, len)], true)`, pas le fichier entier).
    let sample = &bytes[..bytes.len().min(CHARDETNG_MAX_SAMPLE)];
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(sample, true);
    let encoding = detector.guess(None, true);

    if encoding == encoding_rs::WINDOWS_1252 {
        // chardetng retourne windows-1252 pour ISO-8859-1 (extension
        // surensemble). On accepte les deux comme ISO-8859-1 v0.1.
        return Ok((DetectedEncoding::Iso8859_1, 0));
    }

    Err(CsvError::UnsupportedEncoding {
        detected: Some(encoding.name().to_string()),
    })
}

/// Décode `bytes` (post BOM skip) avec l'encoding détecté.
///
/// Pour UTF-8 retourne directement la slice convertie en `String`.
/// **Note Pass 1 review G1 H4** : revalidation `from_utf8` ici est volontaire
/// (defense-in-depth). Coût mineur (~25ms sur 50 MB) accepté pour préserver
/// l'invariant que `decode_bytes` retourne toujours du UTF-8 valide même
/// si `detect_encoding` a un bug — `from_utf8_unchecked` éviterait cette
/// passe mais introduit de l'unsafe avec confiance dans la lib externe.
/// Pour ISO-8859-1, mapping byte 0xXX → char(0xXX) en utilisant
/// `encoding_rs::WINDOWS_1252` (qui couvre ISO-8859-1 strict + extensions).
pub fn decode_bytes(bytes: &[u8], encoding: DetectedEncoding) -> Result<String, CsvError> {
    match encoding {
        DetectedEncoding::Utf8 => {
            std::str::from_utf8(bytes)
                .map(String::from)
                .map_err(|e| CsvError::DecodingFailed {
                    encoding: "UTF-8".to_string(),
                    byte_offset: e.valid_up_to(),
                })
        }
        DetectedEncoding::Iso8859_1 => {
            let (cow, _, had_errors) = encoding_rs::WINDOWS_1252.decode(bytes);
            if had_errors {
                return Err(CsvError::DecodingFailed {
                    encoding: "ISO-8859-1".to_string(),
                    byte_offset: 0,
                });
            }
            Ok(cow.into_owned())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_file() {
        let err = detect_encoding(b"").unwrap_err();
        assert!(matches!(err, CsvError::EmptyFile { .. }));
    }

    #[test]
    fn detects_and_strips_utf8_bom() {
        let bytes = b"\xEF\xBB\xBFdate,amount\n";
        let (enc, bom_len) = detect_encoding(bytes).unwrap();
        assert_eq!(enc, DetectedEncoding::Utf8);
        assert_eq!(bom_len, 3);
    }

    #[test]
    fn parses_ascii_as_utf8() {
        let bytes = b"date,amount\n2026-01-15,100.00\n";
        let (enc, bom_len) = detect_encoding(bytes).unwrap();
        assert_eq!(enc, DetectedEncoding::Utf8);
        assert_eq!(bom_len, 0);
    }

    #[test]
    fn detects_iso_8859_1_via_heuristic() {
        // 100+ bytes ISO-8859-1 avec accents suisses-français
        // (é = 0xE9, ç = 0xE7, à = 0xE0, ü = 0xFC répétés).
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"date;montant;libelle\n");
        for _ in 0..5 {
            bytes.extend_from_slice(
                b"2026-01-15;100,00;Gen\xE8ve - Soci\xE9t\xE9 Z\xFCrich \xE7a va\n",
            );
        }
        let (enc, bom_len) = detect_encoding(&bytes).unwrap();
        assert_eq!(enc, DetectedEncoding::Iso8859_1);
        assert_eq!(bom_len, 0);
    }

    #[test]
    fn rejects_utf16_le_bom() {
        let bytes = b"\xFF\xFEd\x00a\x00t\x00e\x00";
        let err = detect_encoding(bytes).unwrap_err();
        match err {
            CsvError::UnsupportedEncoding { detected } => {
                assert!(detected.is_some());
                assert!(detected.unwrap().contains("UTF-16"));
            }
            other => panic!("expected UnsupportedEncoding, got {:?}", other),
        }
    }

    #[test]
    fn for_bom_handles_short_slices_safely() {
        // 1 byte : pas de panique.
        let (enc, bom_len) = detect_encoding(b"a").unwrap();
        assert_eq!(enc, DetectedEncoding::Utf8);
        assert_eq!(bom_len, 0);

        // 2 bytes : pas de panique.
        let (enc, bom_len) = detect_encoding(b"ab").unwrap();
        assert_eq!(enc, DetectedEncoding::Utf8);
        assert_eq!(bom_len, 0);

        // BOM tronqué (1 byte) : non-ASCII → fichier petit < 64 bytes
        // → reject UnsupportedEncoding (passe 1 fail, garde-fou).
        let err = detect_encoding(b"\xEF").unwrap_err();
        assert!(matches!(
            err,
            CsvError::UnsupportedEncoding { detected: None }
        ));

        // BOM tronqué (2 bytes) : idem.
        let err = detect_encoding(b"\xEF\xBB").unwrap_err();
        assert!(matches!(
            err,
            CsvError::UnsupportedEncoding { detected: None }
        ));
    }

    #[test]
    fn small_non_utf8_file_rejected() {
        // 50 bytes (< 64) avec contenu non-UTF-8 → reject sans
        // chardetng (garde-fou petit fichier).
        let bytes = vec![0xE9; 50];
        let err = detect_encoding(&bytes).unwrap_err();
        assert!(matches!(
            err,
            CsvError::UnsupportedEncoding { detected: None }
        ));
    }

    #[test]
    fn decode_utf8_roundtrip() {
        let bytes = "Genève".as_bytes();
        let s = decode_bytes(bytes, DetectedEncoding::Utf8).unwrap();
        assert_eq!(s, "Genève");
    }

    #[test]
    fn decode_iso_8859_1_to_utf8() {
        // 0xE9 = é en ISO-8859-1.
        let bytes = b"Gen\xE8ve";
        let s = decode_bytes(bytes, DetectedEncoding::Iso8859_1).unwrap();
        assert_eq!(s, "Genève");
    }
}
