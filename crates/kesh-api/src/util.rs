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
//! - [`refus_qr_iban`] — le refus d'un QR-IBAN saisi, sans mot répété et
//!   désignant le bon champ (Story v017-6, #832).
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
    slugify_max(input, fallback, 20)
}

/// Longueur maximale du **type** d'un rapport dans le nom du fichier exporté
/// (`reports-filename-*`). Story 36-15 (CR #1161) : « dépenses par compte
/// analytique » fait 30 caractères (`rendimento-per-conto-analitico`, 30 ;
/// `ausgaben-pro-analytisches-konto`, 31) ; coupé à 20, le nom disait
/// « depenses-par-compte » — « compte » seul, qui désigne le compte du plan. Le
/// nom de la société reste coupé à 20 ([`slugify`]).
pub(crate) const LONGUEUR_MAX_SLUG_DE_TYPE: usize = 40;

/// [`slugify`] avec une longueur maximale donnée (en caractères ASCII, après
/// nettoyage) au lieu de 20.
pub(crate) fn slugify_max(input: &str, fallback: &str, max: usize) -> String {
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
    // Truncate à `max` chars (bytes-safe car ASCII)
    let truncated: String = collapsed.chars().take(max).collect();
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

/// L'échappement des cellules texte des CSV — **une seule règle**, celle de
/// [`kesh_core::csv::csv_sanitize`] (déplacée de ce module par la Story 37-5
/// pour servir aussi `kesh-report`, comportement identique ; ses tests l'ont
/// suivie). Réexportée ici pour les sites d'appel de `kesh-api`.
pub(crate) use kesh_core::csv::csv_sanitize;

// ===========================================================================
// Origine web (Story 32-5c, factorisée Story 32-6a)
// ===========================================================================

/// Une origine `http(s)://hôte[:port]` sous sa sérialisation ASCII (schéma et
/// hôte en minuscules, port par défaut **du schéma** retiré — `:80` pour
/// `http`, `:443` pour `https`, et seulement eux —, nom international en
/// `xn--`) ; `None` si le texte n'en est pas une (sans schéma, schéma autre,
/// chemin, requête, fragment, identifiants, blancs, `null`).
///
/// Une seule règle, deux consommateurs (AC-a10 de la 32-6a) : la validation des
/// origines admises à l'écriture (`routes::mcp_server_settings::canonical_origin`)
/// et la comparaison de l'en-tête `Origin` reçu par `/mcp` (`mcp::reseau`), si
/// bien que l'en-tête et l'entrée stockée se normalisent de la même façon.
pub(crate) fn origine_canonique(brut: &str) -> Option<String> {
    let entree = brut.trim();
    // « Sans chemin » se lit sur le texte saisi : `url` rend « / » pour
    // `http://h` comme pour `http://h/`.
    let (schema, autorite) = entree.split_once("://")?;
    if !matches!(schema.to_ascii_lowercase().as_str(), "http" | "https")
        || autorite.is_empty()
        || autorite.contains(['/', '?', '#', '@', '\\'])
        || autorite.chars().any(char::is_whitespace)
    {
        return None;
    }
    let url = url::Url::parse(entree).ok()?;
    url.host()?;
    Some(url.origin().ascii_serialization())
}

// ===========================================================================
// Refus d'un QR-IBAN (Story v017-6, #832)
// ===========================================================================

/// Le refus d'un QR-IBAN saisi, servi en `VALIDATION_ERROR` : un seul « QR-IBAN
/// invalide : », et le bon champ désigné.
///
/// `QrIban::new` propage **telle quelle** l'erreur d'un IBAN mal formé
/// (`CoreError::InvalidIban`, dont le `Display` dit « IBAN invalide : … ») : la
/// servir nue désignerait le champ IBAN. Ce cas est donc étiqueté du champ
/// QR-IBAN ; toute autre variante (`InvalidQrIban`) porte déjà « QR-IBAN
/// invalide : » et passe telle quelle, sans second préfixe.
///
/// Un helper, deux sites : l'étape banque de l'assistant
/// (`routes::onboarding`) et la création d'un compte bancaire
/// (`routes::bank_accounts`).
pub(crate) fn refus_qr_iban(e: kesh_core::errors::CoreError) -> AppError {
    match e {
        kesh_core::errors::CoreError::InvalidIban(motif) => {
            AppError::Validation(format!("QR-IBAN invalide : {motif}"))
        }
        autre => AppError::Validation(autre.to_string()),
    }
}

// ===========================================================================
// Gardes lexicales (tests)
// ===========================================================================

/// Lignes de code d'un fichier, commentaires `//` écartés, tronqué au
/// premier `#[cfg(test)]` — l'aide des gardes lexicales « lancé par
/// `main.rs` seulement » (Story 31-1c, `auto_backup` ; Story 37-2,
/// `exchange_rates`), extraite ici pour être partagée (refs #727).
///
/// ⚠️ **Limite assumée** (Story 32-1a, #711 E-9 (3), C-32-1-38) : la lecture
/// est ligne à ligne, sans analyse lexicale. Seules les lignes **entières**
/// qui commencent par `//` sont écartées ; un commentaire de fin de ligne, un
/// commentaire `/* … */` et le **contenu des chaînes** restent lus comme du
/// code. La troncature s'arrête à la première ligne dont le contenu est
/// exactement `#[cfg(test)]`. Conséquence : un jeton **exigé** par une garde,
/// écrit seulement dans un commentaire de bloc ou une chaîne, la satisferait à
/// tort — muet ; un jeton **interdit** écrit au même endroit la fait rougir —
/// bruyant. Correction suivie par #727 (extraction d'un balayeur commun).
#[cfg(test)]
pub(crate) fn code(source: &str) -> String {
    source
        .lines()
        .take_while(|l| l.trim() != "#[cfg(test)]")
        .filter(|l| !l.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // `csv_sanitize` : tests dans `kesh_core::csv` (Story 37-5).

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

    /// Story 36-15 (CR #1161) — le type d'un rapport garde « compte analytique » en
    /// entier dans les quatre langues ; coupé à 20, il disait « compte » seul.
    #[test]
    fn slugify_max_keeps_the_analytical_report_types_whole() {
        for type_slug in [
            "depenses-par-compte-analytique",
            "rendement-par-compte-analytique",
            "ausgaben-pro-analytisches-konto",
            "rendite-pro-analytisches-konto",
            "spese-per-conto-analitico",
            "rendimento-per-conto-analitico",
            "expenses-by-analytical-account",
            "return-by-analytical-account",
        ] {
            assert_eq!(
                slugify_max(type_slug, "report", LONGUEUR_MAX_SLUG_DE_TYPE),
                type_slug
            );
        }
        assert_eq!(
            slugify("depenses-par-compte-analytique", "report"),
            "depenses-par-compte",
            "témoin : la coupe à 20 de `slugify` perdait « analytique »"
        );
        assert_eq!(slugify_max("abc-def", "x", 4), "abc");
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
