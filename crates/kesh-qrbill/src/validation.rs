//! SIX 2.2 validation rules for QR Bill data.
//!
//! Implements IBAN mod-97, QR-IBAN IID range (30000-31999), QRR mod-10
//! recursive checksum (SIX Annex B), and field-length constraints.

use crate::types::{Address, Currency, QrBillData, QrBillError, Reference};
use rust_decimal::Decimal;

pub const NAME_MAX: usize = 70;
pub const ADDR_LINE_MAX: usize = 70;
pub const USTRD_MAX: usize = 140;
pub const BILLING_MAX: usize = 140;
pub const COUNTRY_LEN: usize = 2;
pub const IBAN_LEN: usize = 21;
pub const QRR_LEN: usize = 27;

/// Maximum amount allowed by SIX (`Amt` field, 12 chars including decimals).
pub fn amount_max() -> Decimal {
    "999999999.99".parse().unwrap()
}

pub fn amount_min() -> Decimal {
    "0.01".parse().unwrap()
}

/// Normalize an IBAN: trim, uppercase, strip every internal whitespace.
pub fn normalize_iban(raw: &str) -> String {
    raw.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

/// Validate IBAN according to ISO 13616 (mod-97 = 1) and SIX 2.2 (CH or LI only).
pub fn validate_iban(raw: &str) -> Result<String, QrBillError> {
    let iban = normalize_iban(raw);
    // C-Edge (review pass 1 G2 C) : valider ASCII AVANT toute opération de
    // slicing — sinon un input crafté avec du multi-byte (Cyrillique ou
    // accents Unicode laissés par `to_uppercase()`) atteignant 21 bytes
    // panique sur `&iban[..2]`. On utilise `chars().count()` pour la
    // longueur visible, puis on garantit ASCII pour les slices à suivre.
    if !iban.chars().all(|c| c.is_ascii_alphanumeric()) {
        return Err(QrBillError::InvalidIban(
            "caractères non alphanumériques".into(),
        ));
    }
    if iban.len() != IBAN_LEN {
        return Err(QrBillError::InvalidIban(format!(
            "longueur {} (attendu {})",
            iban.len(),
            IBAN_LEN
        )));
    }
    let country = &iban[..2];
    if country != "CH" && country != "LI" {
        return Err(QrBillError::InvalidIban(format!(
            "pays {country} non supporté (CH/LI uniquement)"
        )));
    }
    if !mod97_valid(&iban) {
        return Err(QrBillError::InvalidIban("checksum mod-97".into()));
    }
    Ok(iban)
}

/// Validate a QR-IBAN: same as IBAN + IID (positions 5-8, 0-indexed 4-8) ∈ [30000, 31999].
pub fn validate_qr_iban(raw: &str) -> Result<String, QrBillError> {
    let iban = validate_iban(raw).map_err(|e| match e {
        QrBillError::InvalidIban(msg) => QrBillError::InvalidQrIban(msg),
        other => other,
    })?;
    let iid: u32 = iban[4..9]
        .parse()
        .map_err(|_| QrBillError::InvalidQrIban("IID non numérique".into()))?;
    if !(30000..=31999).contains(&iid) {
        return Err(QrBillError::InvalidQrIban(format!(
            "IID {iid} hors plage QR [30000-31999]"
        )));
    }
    Ok(iban)
}

fn mod97_valid(iban: &str) -> bool {
    // Move first 4 chars to end, convert letters to digits (A=10 ... Z=35), mod 97 == 1.
    let rearranged: String = iban.chars().skip(4).chain(iban.chars().take(4)).collect();
    let mut numeric = String::with_capacity(rearranged.len() * 2);
    for c in rearranged.chars() {
        if c.is_ascii_digit() {
            numeric.push(c);
        } else if c.is_ascii_alphabetic() {
            let n = (c.to_ascii_uppercase() as u8 - b'A' + 10) as u32;
            numeric.push_str(&n.to_string());
        } else {
            return false;
        }
    }
    // Piecewise mod 97.
    let mut rem: u64 = 0;
    for c in numeric.chars() {
        rem = (rem * 10 + c.to_digit(10).unwrap() as u64) % 97;
    }
    rem == 1
}

/// Compute the mod-10 recursive checksum (SIX Annex B) for a 26-digit payload.
///
/// Returns the trailing check digit to append, yielding a 27-digit QRR.
pub fn compute_qrr_checksum(digits26: &str) -> Result<u8, QrBillError> {
    if digits26.len() != 26 || !digits26.chars().all(|c| c.is_ascii_digit()) {
        return Err(QrBillError::InvalidQrr(
            "entrée checksum: attendu 26 chiffres".into(),
        ));
    }
    // Mod-10 recursive lookup table, row-indexed by running carry.
    const TABLE: [[u8; 10]; 10] = [
        [0, 9, 4, 6, 8, 2, 7, 1, 3, 5],
        [9, 4, 6, 8, 2, 7, 1, 3, 5, 0],
        [4, 6, 8, 2, 7, 1, 3, 5, 0, 9],
        [6, 8, 2, 7, 1, 3, 5, 0, 9, 4],
        [8, 2, 7, 1, 3, 5, 0, 9, 4, 6],
        [2, 7, 1, 3, 5, 0, 9, 4, 6, 8],
        [7, 1, 3, 5, 0, 9, 4, 6, 8, 2],
        [1, 3, 5, 0, 9, 4, 6, 8, 2, 7],
        [3, 5, 0, 9, 4, 6, 8, 2, 7, 1],
        [5, 0, 9, 4, 6, 8, 2, 7, 1, 3],
    ];
    let mut carry: usize = 0;
    for c in digits26.chars() {
        let d = c.to_digit(10).unwrap() as usize;
        carry = TABLE[carry][d] as usize;
    }
    Ok(((10 - carry) % 10) as u8)
}

/// Validate a full QRR (27 digits, valid mod-10 recursive checksum).
pub fn validate_qrr(qrr: &str) -> Result<(), QrBillError> {
    if qrr.len() != QRR_LEN || !qrr.chars().all(|c| c.is_ascii_digit()) {
        return Err(QrBillError::InvalidQrr(format!(
            "longueur {} ou caractères non numériques",
            qrr.len()
        )));
    }
    let (body, check) = qrr.split_at(26);
    let expected = compute_qrr_checksum(body)?;
    // C2-Blind (review pass 1 G2 C) : pas de unwrap sur donnée externe.
    // Bien que les checks ASCII en amont rendent le parse infaillible ici,
    // un futur refactor pourrait casser cet invariant. Faille fail-safe.
    let actual = check
        .parse::<u8>()
        .map_err(|_| QrBillError::InvalidQrr("chiffre de contrôle non numérique".into()))?;
    if expected != actual {
        return Err(QrBillError::InvalidQrr(format!(
            "checksum attendu {expected}, obtenu {actual}"
        )));
    }
    Ok(())
}

/// Build a QRR from a customer reference + invoice id.
///
/// Strategy: format = `{7-digit zero-padded company_id}{13-digit zero-padded invoice_id}`
/// → 20-digit significant body + 6 leading-zero padding = 26 digits + 1 check digit = 27.
pub fn build_qrr(company_id: u64, invoice_id: u64) -> Result<String, QrBillError> {
    // C3 (review pass 1 G2 C) : refuser le QRR tout-zéro — SIX exige des
    // chiffres significatifs non-nuls. Les IDs DB démarrent à 1 dans Kesh,
    // mais une seed/test peut passer 0 par erreur.
    if company_id == 0 || invoice_id == 0 {
        return Err(QrBillError::InvalidQrr(
            "company_id et invoice_id doivent être > 0 (QRR tout-zéro interdit par SIX)".into(),
        ));
    }
    if company_id > 9_999_999 {
        return Err(QrBillError::InvalidQrr(format!(
            "company_id {company_id} > 9'999'999 (7 chiffres)"
        )));
    }
    if invoice_id > 9_999_999_999_999 {
        return Err(QrBillError::InvalidQrr(format!(
            "invoice_id {invoice_id} > 13 chiffres"
        )));
    }
    let body = format!("000000{:07}{:013}", company_id, invoice_id); // 26 digits
    debug_assert_eq!(body.len(), 26);
    let check = compute_qrr_checksum(&body)?;
    Ok(format!("{}{}", body, check))
}

/// Vérifie qu'une chaîne ne contient que des caractères autorisés par
/// SIX QR Bill 2.2 §2.5 (Annex C, jeu de caractères restreint).
///
/// C1 (review pass 1 G2 C) : whitelist stricte dérivée de SIX Annex C +
/// 3 ajouts pragmatiques pour couvrir le copy-paste réel macOS/Word/€ :
/// - U+2019 (apostrophe typographique « ' »)
/// - U+2014 (cadratin « — »)
/// - U+20AC (signe euro « € »)
///
/// Caractères explicitement exclus de Annex C : `<`, `>`, `&`, `"` (HTML),
/// caractères de contrôle (`U+007F`, `U+0080-009F`), NBSP (U+00A0), soft
/// hyphen (U+00AD), multiplication × (U+00D7) et division ÷ (U+00F7).
fn check_charset(field: &'static str, value: &str) -> Result<(), QrBillError> {
    for c in value.chars() {
        if !is_six_annex_c_char(c) {
            return Err(QrBillError::InvalidCharset {
                field,
                codepoint: c as u32,
            });
        }
    }
    Ok(())
}

/// Implémente le whitelist SIX Annex C + 3 additions documentées (C1 review
/// pass 1 G2 C). Voir `check_charset` pour la liste des exclusions.
fn is_six_annex_c_char(c: char) -> bool {
    // ASCII printable subset autorisé par Annex C (exclut < > & " et contrôles).
    // CRITICAL pass 2 : RETIRER `\n` du whitelist — c'est le séparateur de
    // payload SIX (cf. generator.rs `lines.join("\n")`). L'admettre dans un
    // champ d'adresse corromprait silencieusement l'ordre des champs après
    // le `join`, produisant un QR Code parseable mais sémantiquement faux.
    if c == ' '
        || matches!(c,
            'A'..='Z' | 'a'..='z' | '0'..='9'
            | '!' | '#' | '$' | '%' | '*' | '+' | ',' | '-' | '.' | '/'
            | ':' | ';' | '=' | '?' | '@' | '_' | '\'' | '(' | ')'
        )
    {
        return true;
    }
    // Latin-1 supplément + Latin Ext-A : restreint aux lettres accentuées.
    // Exclut NBSP (00A0), soft hyphen (00AD), × (00D7), ÷ (00F7).
    let cp = c as u32;
    if (0x00A1..=0x00FF).contains(&cp)
        && cp != 0x00AD
        && cp != 0x00D7
        && cp != 0x00F7
    {
        return true;
    }
    if (0x0100..=0x017F).contains(&cp) {
        return true;
    }
    // Additions pragmatiques C1:a (curly apostrophe, em-dash, euro sign).
    matches!(cp, 0x2019 | 0x2014 | 0x20AC)
}

fn check_len(field: &'static str, value: &str, max: usize) -> Result<(), QrBillError> {
    if value.chars().count() > max {
        return Err(QrBillError::FieldTooLong {
            field,
            max,
            got: value.chars().count(),
        });
    }
    Ok(())
}

fn validate_country(code: &str) -> Result<(), QrBillError> {
    if code.chars().count() != COUNTRY_LEN || !code.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(QrBillError::InvalidCountry(code.into()));
    }
    // C4 (review pass 1 G2 C) : valider contre la liste ISO-3166-1 alpha-2.
    // Une simple regex `^[A-Z]{2}$` accepte 'XX', 'ZZ' qui peuvent être
    // rejetés par les systèmes SEPA en aval. Liste statique des 249 codes
    // valides au 2026-04 (mise à jour au gré des changements ISO).
    if !is_iso_3166_alpha2(code) {
        return Err(QrBillError::InvalidCountry(format!(
            "{code} (non ISO-3166-1 alpha-2)"
        )));
    }
    Ok(())
}

/// Liste statique ISO-3166-1 alpha-2 (mise à jour 2026-04).
/// Source : https://www.iso.org/obp/ui/#search/code/
fn is_iso_3166_alpha2(code: &str) -> bool {
    const ISO_3166_ALPHA2: &[&str] = &[
        "AD", "AE", "AF", "AG", "AI", "AL", "AM", "AO", "AQ", "AR", "AS", "AT", "AU", "AW", "AX",
        "AZ", "BA", "BB", "BD", "BE", "BF", "BG", "BH", "BI", "BJ", "BL", "BM", "BN", "BO", "BQ",
        "BR", "BS", "BT", "BV", "BW", "BY", "BZ", "CA", "CC", "CD", "CF", "CG", "CH", "CI", "CK",
        "CL", "CM", "CN", "CO", "CR", "CU", "CV", "CW", "CX", "CY", "CZ", "DE", "DJ", "DK", "DM",
        "DO", "DZ", "EC", "EE", "EG", "EH", "ER", "ES", "ET", "FI", "FJ", "FK", "FM", "FO", "FR",
        "GA", "GB", "GD", "GE", "GF", "GG", "GH", "GI", "GL", "GM", "GN", "GP", "GQ", "GR", "GS",
        "GT", "GU", "GW", "GY", "HK", "HM", "HN", "HR", "HT", "HU", "ID", "IE", "IL", "IM", "IN",
        "IO", "IQ", "IR", "IS", "IT", "JE", "JM", "JO", "JP", "KE", "KG", "KH", "KI", "KM", "KN",
        "KP", "KR", "KW", "KY", "KZ", "LA", "LB", "LC", "LI", "LK", "LR", "LS", "LT", "LU", "LV",
        "LY", "MA", "MC", "MD", "ME", "MF", "MG", "MH", "MK", "ML", "MM", "MN", "MO", "MP", "MQ",
        "MR", "MS", "MT", "MU", "MV", "MW", "MX", "MY", "MZ", "NA", "NC", "NE", "NF", "NG", "NI",
        "NL", "NO", "NP", "NR", "NU", "NZ", "OM", "PA", "PE", "PF", "PG", "PH", "PK", "PL", "PM",
        "PN", "PR", "PS", "PT", "PW", "PY", "QA", "RE", "RO", "RS", "RU", "RW", "SA", "SB", "SC",
        "SD", "SE", "SG", "SH", "SI", "SJ", "SK", "SL", "SM", "SN", "SO", "SR", "SS", "ST", "SV",
        "SX", "SY", "SZ", "TC", "TD", "TF", "TG", "TH", "TJ", "TK", "TL", "TM", "TN", "TO", "TR",
        "TT", "TV", "TW", "TZ", "UA", "UG", "UM", "US", "UY", "UZ", "VA", "VC", "VE", "VG", "VI",
        "VN", "VU", "WF", "WS", "YE", "YT", "ZA", "ZM", "ZW",
    ];
    // Pass 2 : assertion debug — détecte un désordre humain qui casserait
    // silencieusement `binary_search` (codes valides rejetés ou inverse).
    debug_assert!(
        ISO_3166_ALPHA2.windows(2).all(|w| w[0] < w[1]),
        "ISO_3166_ALPHA2 doit rester trié pour binary_search"
    );
    ISO_3166_ALPHA2.binary_search(&code).is_ok()
}

fn validate_address(field_prefix: &'static str, addr: &Address) -> Result<(), QrBillError> {
    if addr.name.trim().is_empty() {
        return Err(QrBillError::FieldEmpty(field_prefix_name(field_prefix)));
    }
    // Pass 2 : SIX §3.3 — pour le type K (Combined), `line2` doit contenir
    // a minima le code postal + localité. Une chaîne vide produirait un
    // payload techniquement parseable mais non conforme SIX.
    if matches!(addr.address_type, crate::types::AddressType::Combined)
        && addr.line2.trim().is_empty()
    {
        return Err(QrBillError::FieldEmpty(field_prefix_line2(field_prefix)));
    }
    check_len(field_prefix_name(field_prefix), &addr.name, NAME_MAX)?;
    check_len(field_prefix_line1(field_prefix), &addr.line1, ADDR_LINE_MAX)?;
    check_len(field_prefix_line2(field_prefix), &addr.line2, ADDR_LINE_MAX)?;
    check_charset(field_prefix_name(field_prefix), &addr.name)?;
    check_charset(field_prefix_line1(field_prefix), &addr.line1)?;
    check_charset(field_prefix_line2(field_prefix), &addr.line2)?;
    validate_country(&addr.country)?;
    Ok(())
}

fn field_prefix_name(prefix: &'static str) -> &'static str {
    match prefix {
        "creditor" => "creditor.name",
        "debtor" => "debtor.name",
        _ => "address.name",
    }
}
fn field_prefix_line1(prefix: &'static str) -> &'static str {
    match prefix {
        "creditor" => "creditor.line1",
        "debtor" => "debtor.line1",
        _ => "address.line1",
    }
}
fn field_prefix_line2(prefix: &'static str) -> &'static str {
    match prefix {
        "creditor" => "creditor.line2",
        "debtor" => "debtor.line2",
        _ => "address.line2",
    }
}

/// Validate a full `QrBillData` against SIX 2.2 rules.
///
/// Returns `Ok(())` if every field is valid. IBANs are *also* validated for
/// normalization form — callers should use [`normalize_iban`] upstream to avoid
/// surprises.
pub fn validate(data: &QrBillData) -> Result<(), QrBillError> {
    // Auditor pass 1 G2 C : cross-check QR-IBAN ↔ reference type.
    // SIX 2.2 §3.3 : un QR-IBAN (IID 30000-31999) DOIT porter une référence
    // QRR ; un IBAN classique NE PEUT PAS en porter. Sans cette garde, un
    // appelant pourrait passer un QR-IBAN avec Reference::None et produire
    // un payload techniquement valide mais rejeté à la lecture bancaire.
    let normalized = normalize_iban(&data.creditor_iban);
    let is_qr_iban = normalized.len() == IBAN_LEN
        && normalized.is_ascii()
        && normalized[4..9]
            .parse::<u32>()
            .map(|iid| (30000..=31999).contains(&iid))
            .unwrap_or(false);
    match (&data.reference, is_qr_iban) {
        (Reference::Qrr(qrr), true) => {
            validate_qr_iban(&data.creditor_iban)?;
            validate_qrr(qrr)?;
        }
        (Reference::None, false) => {
            validate_iban(&data.creditor_iban)?;
        }
        (Reference::Qrr(_), false) => {
            return Err(QrBillError::InvalidQrIban(
                "référence QRR fournie mais l'IBAN n'est pas un QR-IBAN".into(),
            ));
        }
        (Reference::None, true) => {
            return Err(QrBillError::InvalidIban(
                "QR-IBAN fourni sans référence QRR — combinaison interdite SIX §3.3".into(),
            ));
        }
    }

    validate_address("creditor", &data.creditor)?;
    if let Some(debtor) = &data.ultimate_debtor {
        validate_address("debtor", debtor)?;
    }

    // Amount.
    let amount = data
        .amount
        .ok_or_else(|| QrBillError::InvalidAmount("montant requis en v0.1".into()))?;
    if amount < amount_min() || amount > amount_max() {
        return Err(QrBillError::InvalidAmount(format!(
            "{} hors plage [{}, {}]",
            amount,
            amount_min(),
            amount_max()
        )));
    }

    // Currency: validated at the type level.
    let _: Currency = data.currency;

    if let Some(msg) = &data.unstructured_message {
        check_len("unstructured_message", msg, USTRD_MAX)?;
        check_charset("unstructured_message", msg)?;
    }
    if let Some(bi) = &data.billing_information {
        check_len("billing_information", bi, BILLING_MAX)?;
        check_charset("billing_information", bi)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Known-good Swiss IBAN (SIX sample).
    const IBAN_OK: &str = "CH9300762011623852957";
    // QR-IBAN (IID 30000).
    const QR_IBAN_OK: &str = "CH4431999123000889012";

    #[test]
    fn iban_valid() {
        assert_eq!(validate_iban(IBAN_OK).unwrap(), IBAN_OK);
    }

    #[test]
    fn iban_with_spaces_is_normalized() {
        let with_sp = "CH93 0076 2011 6238 5295 7";
        assert_eq!(validate_iban(with_sp).unwrap(), IBAN_OK);
    }

    #[test]
    fn iban_wrong_country_rejected() {
        let fr = "FR1420041010050500013M02606";
        assert!(matches!(
            validate_iban(fr),
            Err(QrBillError::InvalidIban(_))
        ));
    }

    #[test]
    fn iban_bad_checksum_rejected() {
        // Flip last digit.
        let bad = "CH9300762011623852958";
        assert!(matches!(
            validate_iban(bad),
            Err(QrBillError::InvalidIban(_))
        ));
    }

    #[test]
    fn qr_iban_valid() {
        assert!(validate_qr_iban(QR_IBAN_OK).is_ok());
    }

    #[test]
    fn qr_iban_iid_out_of_range_rejected() {
        // Real IBAN with IID outside [30000-31999] → reject.
        assert!(matches!(
            validate_qr_iban(IBAN_OK),
            Err(QrBillError::InvalidQrIban(_))
        ));
    }

    #[test]
    fn qrr_checksum_roundtrip() {
        let body = "21000000000003139471430009"; // SIX sample body (26 digits)
        let check = compute_qrr_checksum(body).unwrap();
        let full = format!("{body}{check}");
        assert!(validate_qrr(&full).is_ok());
    }

    /// C5 (review pass 1 G2 C) : vecteur SIX officiel — verrouille la
    /// table mod-10 contre toute transposition lignes/colonnes silencieuse.
    /// Source : SIX QR Bill 2.2 IG §3.3.2 « Sample QR Reference ».
    /// QRR documenté : `210000000003139471430009017`
    /// → body = `21000000000313947143000901`, check digit = `7`.
    #[test]
    fn qrr_checksum_matches_six_official_sample() {
        let body = "21000000000313947143000901";
        let check = compute_qrr_checksum(body).unwrap();
        assert_eq!(
            check, 7,
            "SIX 2.2 IG §3.3.2 sample : check digit attendu = 7"
        );
        // Validation complète du QRR officiel.
        assert!(validate_qrr("210000000003139471430009017").is_ok());
    }

    #[test]
    fn build_qrr_rejects_zero_ids() {
        // C3 (review pass 1 G2 C) : IDs nuls produiraient un QRR tout-zéro
        // (refusé par les banques) — le rejet doit intervenir au build.
        assert!(matches!(
            build_qrr(0, 1),
            Err(QrBillError::InvalidQrr(_))
        ));
        assert!(matches!(
            build_qrr(1, 0),
            Err(QrBillError::InvalidQrr(_))
        ));
    }

    #[test]
    fn iban_with_multibyte_input_does_not_panic() {
        // C-Edge (review pass 1 G2 C) : input crafté avec multi-byte ne
        // doit jamais paniquer sur un slice byte-indexé.
        let crafted = "ÀH9300762011623852957"; // 22 bytes (1 multi-byte + 20 ASCII)
        let res = validate_iban(crafted);
        assert!(matches!(res, Err(QrBillError::InvalidIban(_))));
    }

    #[test]
    fn qr_iban_with_none_reference_rejected() {
        // Auditor (review pass 1 G2 C) : QR-IBAN sans QRR = combinaison
        // interdite par SIX §3.3.
        let data = sample_data_with_iban(QR_IBAN_OK, Reference::None);
        assert!(matches!(validate(&data), Err(QrBillError::InvalidIban(_))));
    }

    #[test]
    fn classic_iban_with_qrr_reference_rejected() {
        // Auditor (review pass 1 G2 C) : IBAN classique avec QRR = combinaison
        // interdite par SIX §3.3.
        let body = "21000000000313947143000901";
        let check = compute_qrr_checksum(body).unwrap();
        let qrr = format!("{body}{check}");
        let data = sample_data_with_iban(IBAN_OK, Reference::Qrr(qrr));
        assert!(matches!(validate(&data), Err(QrBillError::InvalidQrIban(_))));
    }

    #[test]
    fn charset_curly_apostrophe_accepted() {
        // C1 (review pass 1 G2 C) : U+2019 (apostrophe macOS/Word) doit
        // passer après l'addition pragmatique au whitelist.
        let addr = Address {
            address_type: crate::types::AddressType::Combined,
            name: "Rue de l\u{2019}H\u{f4}pital".into(),
            line1: "rue".into(),
            line2: "1003 Lausanne".into(),
            country: "CH".into(),
        };
        assert!(validate_address("creditor", &addr).is_ok());
    }

    #[test]
    fn charset_html_unsafe_chars_rejected() {
        // C1 (review pass 1 G2 C) : `<`, `>`, `&`, `"` exclus par Annex C.
        for s in ["A<B", "A>B", "A&B", "A\"B"] {
            let addr = Address {
                address_type: crate::types::AddressType::Combined,
                name: s.into(),
                line1: "rue".into(),
                line2: "1003 Lausanne".into(),
                country: "CH".into(),
            };
            assert!(
                matches!(
                    validate_address("creditor", &addr),
                    Err(QrBillError::InvalidCharset { .. })
                ),
                "expected charset rejection for: {s}"
            );
        }
    }

    fn sample_data_with_iban(iban: &str, reference: Reference) -> QrBillData {
        let mut d = sample_data("100.00".parse().unwrap());
        d.creditor_iban = iban.into();
        d.reference = reference;
        d
    }

    #[test]
    fn qrr_bad_checksum_rejected() {
        let body = "21000000000003139471430009";
        let good = compute_qrr_checksum(body).unwrap();
        let wrong = (good + 1) % 10;
        let bad = format!("{body}{wrong}");
        assert!(matches!(
            validate_qrr(&bad),
            Err(QrBillError::InvalidQrr(_))
        ));
    }

    #[test]
    fn build_qrr_is_deterministic() {
        let a = build_qrr(42, 100).unwrap();
        let b = build_qrr(42, 100).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.len(), 27);
        assert!(validate_qrr(&a).is_ok());
    }

    /// Pass 3 (review pass 3 G2 C) : verrouille l'invariant de tri ISO.
    /// `binary_search` se base sur ce tri ; un désordre humain casserait
    /// silencieusement la validation pays en release (où `debug_assert!`
    /// ne fire pas).
    #[test]
    fn iso_3166_list_is_sorted() {
        // Reconstruit la liste depuis is_iso_3166_alpha2 en sondant chaque
        // bigramme connu — méthode robuste : on appelle simplement
        // `is_iso_3166_alpha2("CH")` et on vérifie que l'invariant tient
        // via le `debug_assert!` interne (qui panique en test).
        assert!(is_iso_3166_alpha2("CH"));
        assert!(is_iso_3166_alpha2("LI"));
        assert!(is_iso_3166_alpha2("ZW")); // dernier alphabétiquement
        assert!(is_iso_3166_alpha2("AD")); // premier alphabétiquement
        assert!(!is_iso_3166_alpha2("XX"));
        assert!(!is_iso_3166_alpha2("ZZ"));
    }

    #[test]
    fn build_qrr_overflow_rejected() {
        assert!(matches!(
            build_qrr(10_000_000, 1),
            Err(QrBillError::InvalidQrr(_))
        ));
    }

    #[test]
    fn address_name_too_long_rejected() {
        let addr = Address {
            address_type: crate::types::AddressType::Combined,
            name: "x".repeat(71),
            line1: "rue".into(),
            line2: "1000 Lausanne".into(),
            country: "CH".into(),
        };
        assert!(matches!(
            validate_address("creditor", &addr),
            Err(QrBillError::FieldTooLong { .. })
        ));
    }

    #[test]
    fn country_invalid_rejected() {
        let addr = Address {
            address_type: crate::types::AddressType::Combined,
            name: "Acme".into(),
            line1: "rue".into(),
            line2: "1000 Lausanne".into(),
            country: "che".into(),
        };
        assert!(matches!(
            validate_address("creditor", &addr),
            Err(QrBillError::InvalidCountry(_))
        ));
    }

    #[test]
    fn amount_zero_rejected() {
        let data = sample_data(Decimal::ZERO);
        assert!(matches!(validate(&data), Err(QrBillError::InvalidAmount(_))));
    }

    #[test]
    fn charset_cjk_rejected() {
        let addr = Address {
            address_type: crate::types::AddressType::Combined,
            name: "张伟".into(),
            line1: "rue".into(),
            line2: "1000 Lausanne".into(),
            country: "CH".into(),
        };
        assert!(matches!(
            validate_address("creditor", &addr),
            Err(QrBillError::InvalidCharset { .. })
        ));
    }

    #[test]
    fn charset_latin_extended_accepted() {
        // Pass 2 : `&` retiré du fixture — exclu d'Annex C strict (cf. C1).
        // Test couvre les diacritiques latin (Ü, ô) qui restent acceptés.
        let addr = Address {
            address_type: crate::types::AddressType::Combined,
            name: "Müller Cie SA".into(),
            line1: "Rue de l'Hôpital 1".into(),
            line2: "1003 Lausanne".into(),
            country: "CH".into(),
        };
        assert!(validate_address("creditor", &addr).is_ok());
    }

    #[test]
    fn amount_over_max_rejected() {
        let data = sample_data("1000000000.00".parse().unwrap());
        assert!(matches!(validate(&data), Err(QrBillError::InvalidAmount(_))));
    }

    fn sample_data(amount: Decimal) -> QrBillData {
        QrBillData {
            creditor_iban: IBAN_OK.into(),
            creditor: Address {
                address_type: crate::types::AddressType::Combined,
                name: "Robert Schneider SA".into(),
                line1: "Rue du Lac 1268".into(),
                line2: "2501 Biel".into(),
                country: "CH".into(),
            },
            ultimate_debtor: Some(Address {
                address_type: crate::types::AddressType::Combined,
                name: "Pia Rutschmann".into(),
                line1: "Marktgasse 28".into(),
                line2: "9400 Rorschach".into(),
                country: "CH".into(),
            }),
            amount: Some(amount),
            currency: Currency::Chf,
            reference: Reference::None,
            unstructured_message: None,
            billing_information: None,
        }
    }
}
