//! Helpers transverses kesh-api (Story 9-2b §util refactor).
//!
//! Ce module héberge les helpers partagés entre plusieurs routes :
//!
//! - [`slugify`] — pipeline de slugification ASCII-strict (Story 9-2a, factorisé
//!   ici en Story 9-2b pour réutilisation dans `routes::exports`).
//! - [`build_content_disposition`] — header RFC 6266 + RFC 5987 avec ASCII
//!   fallback + tag langue BCP-47 (Story 9-2a Pass 1 code-review M14 / BH2-M2).
//! - [`map_language_to_bcp47`] — mapping `accounting_language` DB → BCP-47
//!   (Story 9-2b T5.1 / Pass 3 ECH3-H2 — extraction du `match` inline qui
//!   vivait dans `routes::reports::load_pdf_context`).
//! - [`hex_encode`] — encoding hex d'un slice de bytes (Story 8-1b, factorisé
//!   ici en Story 9-2b pour `exports::metadata::sha256_hex`).
//!
//! Toutes les fns sont `pub(crate)` — réservées aux modules internes de la
//! crate `kesh-api`. Si une fn doit devenir publique (consommée hors crate),
//! reconsidérer le scope module-par-module.

use axum::http::HeaderValue;

use crate::errors::AppError;

// ===========================================================================
// Slugify (Story 9-2a, factorisé Story 9-2b)
// ===========================================================================

/// Slugifie une chaîne pour usage dans un filename ASCII strict.
///
/// Pipeline :
/// 1. NFD-strip diacritics (manuelle : remplace les variantes connues).
/// 2. lowercase + ASCII-only (les caractères non-Latin → remplacés par `-`).
/// 3. `[^a-z0-9-]` → `-`.
/// 4. Collapse `-+` → `-`.
/// 5. Truncate à 20 chars.
/// 6. Strip trailing `-`.
/// 7. Si le résultat est vide → `fallback`.
pub(crate) fn slugify(input: &str, fallback: &str) -> String {
    let lowered: String = input
        .to_lowercase()
        .chars()
        .map(strip_diacritics_char)
        .collect();
    let cleaned: String = lowered
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();
    // Collapse repeated dashes
    let mut collapsed = String::with_capacity(cleaned.len());
    let mut prev_dash = false;
    for c in cleaned.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }
    // Truncate à 20 chars (bytes-safe car ASCII)
    let truncated: String = collapsed.chars().take(20).collect();
    // Strip trailing `-`
    let stripped = truncated.trim_end_matches('-');
    if stripped.is_empty() {
        fallback.to_string()
    } else {
        stripped.to_string()
    }
}

/// Strip diacritics manuels (fallback car std n'a pas `unicode-normalization`).
/// Couvre les caractères Latin-1 + Extended-A les plus courants.
fn strip_diacritics_char(c: char) -> char {
    match c {
        'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => 'a',
        'ç' => 'c',
        'è' | 'é' | 'ê' | 'ë' => 'e',
        'ì' | 'í' | 'î' | 'ï' => 'i',
        'ñ' => 'n',
        'ò' | 'ó' | 'ô' | 'õ' | 'ö' => 'o',
        'ù' | 'ú' | 'û' | 'ü' => 'u',
        'ý' | 'ÿ' => 'y',
        other => other,
    }
}

// ===========================================================================
// Content-Disposition (Story 9-2a, factorisé Story 9-2b)
// ===========================================================================

/// Construit le `Content-Disposition` header avec ASCII fallback + RFC 5987
/// UTF-8 (cohérent Pass 1 ECH-M2 + code-review M14).
///
/// Le paramètre `locale_bcp47` est inséré entre les deux apostrophes simples
/// de `filename*=UTF-8'<lang>'…` (RFC 5987 § 3.2.1). Locale vide (`""`)
/// produit `filename*=UTF-8''…` (sans tag langue, syntaxe également valide).
pub(crate) fn build_content_disposition(
    filename: &str,
    locale_bcp47: &str,
) -> Result<HeaderValue, AppError> {
    let ascii_fallback = ascii_fallback_filename(filename);
    let percent_encoded = percent_encode_filename(filename);
    let value = format!(
        "attachment; filename=\"{ascii_fallback}\"; filename*=UTF-8'{locale_bcp47}'{percent_encoded}"
    );
    HeaderValue::from_str(&value)
        .map_err(|e| AppError::Internal(format!("invalid Content-Disposition header: {e}")))
}

/// Remplace les caractères non-ASCII par `_` pour le fallback `filename="..."`.
fn ascii_fallback_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| {
            if c.is_ascii() && c != '"' && c != '\\' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

/// Percent-encode RFC 5987 (réservé + non-ASCII).
///
/// Pass 1 code-review H7 (BH2-H3) : le cast `b as char` n'est correct que si
/// `b < 128` (ASCII). La branche `is_safe` est aujourd'hui garantie ASCII via
/// `is_ascii_alphanumeric()` + le set `-._~`, mais un refactor élargissant le
/// filtre (e.g. à `is_alphanumeric()` Unicode) introduirait silencieusement un
/// bug — le `debug_assert!` ci-dessous protège contre cette régression.
pub(crate) fn percent_encode_filename(filename: &str) -> String {
    let mut out = String::with_capacity(filename.len());
    for b in filename.bytes() {
        let is_safe = b.is_ascii_alphanumeric() || matches!(b, b'-' | b'.' | b'_' | b'~');
        if is_safe {
            debug_assert!(
                b < 128,
                "is_safe doit garantir un byte ASCII (b < 128) avant `b as char`"
            );
            out.push(b as char);
        } else {
            out.push_str(&format!("%{b:02X}"));
        }
    }
    out
}

// ===========================================================================
// Language → BCP-47 (Story 9-2b T5.1 / Pass 3 ECH3-H2)
// ===========================================================================

/// Mappe le code langue stocké en DB (`accounting_language`, valeurs SCREAMING
/// `"FR" / "DE" / "IT" / "EN"` cf. `entities::company::Language::as_str`) vers
/// l'identifiant BCP-47 régionalisé Suisse correspondant.
///
/// Une valeur inconnue (corruption DB, future variant non-mappée) déclenche un
/// `warn!` log et retombe sur `fr-CH` pour conserver une réponse cohérente côté
/// client — politique Story 9-2a Pass 1 code-review M3.
///
/// Signature `&str` (PAS l'enum `Language`) car les sites d'appel actuels
/// (`load_pdf_context`, `routes::exports::export_global`) lisent
/// `accounting_language` comme `String` via une `sqlx::query_as`.
pub(crate) fn map_language_to_bcp47(locale_code: &str) -> &'static str {
    match locale_code {
        "FR" => "fr-CH",
        "DE" => "de-CH",
        "IT" => "it-CH",
        "EN" => "en-CH",
        other => {
            tracing::warn!(
                unknown_locale = %other,
                "accounting_language inconnu, fallback fr-CH"
            );
            "fr-CH"
        }
    }
}

// ===========================================================================
// Hex encoding (Story 8-1b, factorisé Story 9-2b)
// ===========================================================================

/// Encode un slice de bytes en chaîne hexadécimale minuscule (`{:02x}`).
///
/// Réutilisé par :
/// - `routes::bank_imports` — hash SHA-256 du fichier importé.
/// - `exports::metadata::sha256_hex` — hash SHA-256 de chaque CSV du ZIP
///   global (Story 9-2b).
///
/// Decision §hex-encoding (Pass 1 BH-HIGH-04) : on n'introduit pas la crate
/// `hex = "0.4"` pour ~10 lignes triviales — pattern local conservé.
pub(crate) fn hex_encode(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- slugify -----

    #[test]
    fn slugify_strips_diacritics_lowercase() {
        assert_eq!(slugify("Müller AG", "fallback"), "muller-ag");
        assert_eq!(slugify("Café Crème", "fallback"), "cafe-creme");
    }

    #[test]
    fn slugify_collapses_repeated_dashes() {
        assert_eq!(slugify("Kesh ---   SA", "fallback"), "kesh-sa");
    }

    #[test]
    fn slugify_truncates_to_20_chars_and_strips_trailing_dash() {
        let result = slugify("Acme SA Fribourg Extension Long", "fallback");
        assert!(result.len() <= 20);
        assert!(
            !result.ends_with('-'),
            "result must not end with `-`, got: {result}"
        );
    }

    #[test]
    fn slugify_empty_falls_back() {
        assert_eq!(slugify("", "company"), "company");
        assert_eq!(slugify("北京公司", "company"), "company");
    }

    // ----- Content-Disposition -----

    #[test]
    fn content_disposition_handles_unicode_via_rfc5987() {
        let header = build_content_disposition("kesh-bilan-müller-ag-2026.pdf", "").unwrap();
        let header_str = header.to_str().unwrap();
        assert!(header_str.contains("filename="));
        assert!(header_str.contains("filename*=UTF-8''"));
        assert!(header_str.contains("kesh-bilan-m_ller-ag-2026.pdf"));
        assert!(header_str.contains("%C3%BC"));
    }

    #[test]
    fn content_disposition_with_locale_tag_includes_language() {
        let header = build_content_disposition("kesh-bilan-test-2026.pdf", "fr-CH").unwrap();
        let header_str = header.to_str().unwrap();
        assert!(
            header_str.contains("filename*=UTF-8'fr-CH'"),
            "RFC 5987 tag langue manquant, got: {header_str}"
        );
    }

    #[test]
    fn percent_encode_keeps_safe_chars() {
        assert_eq!(
            percent_encode_filename("file-name_2026.pdf"),
            "file-name_2026.pdf"
        );
    }

    #[test]
    fn percent_encode_encodes_unicode() {
        let result = percent_encode_filename("ü");
        assert_eq!(result, "%C3%BC");
    }

    // ----- map_language_to_bcp47 -----

    #[test]
    fn map_language_to_bcp47_known_locales() {
        assert_eq!(map_language_to_bcp47("FR"), "fr-CH");
        assert_eq!(map_language_to_bcp47("DE"), "de-CH");
        assert_eq!(map_language_to_bcp47("IT"), "it-CH");
        assert_eq!(map_language_to_bcp47("EN"), "en-CH");
    }

    #[test]
    fn map_language_to_bcp47_unknown_falls_back_to_fr_ch() {
        assert_eq!(map_language_to_bcp47("XX"), "fr-CH");
        assert_eq!(map_language_to_bcp47(""), "fr-CH");
    }

    // ----- hex_encode -----

    #[test]
    fn hex_encode_empty() {
        assert_eq!(hex_encode(&[]), "");
    }

    #[test]
    fn hex_encode_basic_bytes() {
        assert_eq!(hex_encode(&[0x00, 0x01, 0x0F, 0xFF]), "00010fff");
    }

    #[test]
    fn hex_encode_full_byte_range_sample() {
        let bytes: Vec<u8> = (0u8..=255).collect();
        let encoded = hex_encode(&bytes);
        assert_eq!(encoded.len(), 512);
        assert!(encoded.starts_with("000102030405"));
        assert!(encoded.ends_with("fcfdfeff"));
    }
}
