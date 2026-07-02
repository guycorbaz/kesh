//! SIX 2.2 QR Code payload **parser** — inverse of [`crate::generator::build_payload`].
//!
//! Decodes the newline-separated SPC payload string (as extracted from a scanned
//! QR code) into a typed [`ScannedQrBill`]. Robust to third-party emitters:
//! handles both combined (`K`) and structured (`S`) creditor address blocks, and
//! tolerates variable trailing fields (31/32/34 lines — `billing_information` and
//! `AltPmtInf` optional).
//!
//! Reuses [`crate::validation`] for IBAN / QR-IBAN / QRR checks — no duplication.
//! Socle commun avec le scan manuel (Story 12-4) et l'import répertoire (12-5).

use crate::types::QrBillError;
use crate::validation::{validate_iban, validate_qrr};
use rust_decimal::Decimal;
use std::str::FromStr;

/// Parsed Swiss QR Bill payload (creditor + amount + reference).
///
/// The ultimate-debtor block is intentionally not exposed — only the creditor
/// coordinates, amount and reference are needed to import a supplier invoice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedQrBill {
    /// Normalised IBAN or QR-IBAN (no spaces, uppercase).
    pub creditor_iban: String,
    /// `true` if the IBAN is a QR-IBAN (IID in `[30000, 31999]`).
    pub is_qr_iban: bool,
    pub creditor: ScannedAddress,
    /// Amount; `None` when the payload amount field is empty (open amount).
    pub amount: Option<Decimal>,
    /// Currency code, kept verbatim (`CHF`/`EUR`; validated downstream).
    pub currency: String,
    pub reference: ScannedReference,
    pub unstructured_message: Option<String>,
    pub billing_information: Option<String>,
}

/// Creditor address block — combined (`K`) or structured (`S`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedAddress {
    /// `'K'` (combined, two free-form lines) or `'S'` (structured).
    pub address_type: char,
    pub name: String,
    /// Free-form line 1 (`K`) or street (`S`).
    pub street_or_line1: String,
    /// Free-form line 2 (`K`) or building number (`S`).
    pub building_or_line2: String,
    /// Postal code — populated only for structured (`S`) addresses.
    pub postal_code: Option<String>,
    /// Town — populated only for structured (`S`) addresses.
    pub town: Option<String>,
    pub country: String,
}

/// Payment reference. `Scor` is tolerated on parse though Kesh never emits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScannedReference {
    /// QR-Reference (27 digits, validated mod-10 recursive checksum).
    Qrr(String),
    /// Creditor Reference ISO 11649.
    Scor(String),
    /// No reference (`Tp = NON`).
    None,
}

// Line indices in the SIX 2.2 payload (mirror of `generator::build_payload`).
const L_HEADER: usize = 0; // "SPC"
const L_VERSION: usize = 1; // "0200"
const L_IBAN: usize = 3;
const L_CRED_ADRTP: usize = 4;
const L_CRED_NAME: usize = 5;
const L_CRED_LINE1: usize = 6;
const L_CRED_LINE2: usize = 7;
const L_CRED_PSTCD: usize = 8;
const L_CRED_TOWN: usize = 9;
const L_CRED_COUNTRY: usize = 10;
const L_AMOUNT: usize = 18;
const L_CURRENCY: usize = 19;
const L_REF_TYPE: usize = 27;
const L_REF_VALUE: usize = 28;
const L_USTRD: usize = 29;
const L_EPD: usize = 30; // trailer
const L_BILLING: usize = 31; // optional
const MIN_LINES: usize = 31; // indices 0..=30 (billing_information may be absent)

/// Parse a SIX 2.2 SPC payload string into a [`ScannedQrBill`].
///
/// # Errors
/// - [`QrBillError::InvalidPayload`] — missing/invalid header, unknown reference
///   type, or too few lines.
/// - [`QrBillError::InvalidIban`] / [`QrBillError::InvalidQrIban`] — bad IBAN.
/// - [`QrBillError::InvalidQrr`] — bad QR-Reference checksum.
/// - [`QrBillError::InvalidAmount`] — non-parsable amount.
pub fn parse_spc_payload(text: &str) -> Result<ScannedQrBill, QrBillError> {
    // Tolerate CRLF line endings from arbitrary QR decoders.
    let lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();

    if lines.len() < MIN_LINES {
        return Err(QrBillError::InvalidPayload(format!(
            "{} lignes (minimum {MIN_LINES})",
            lines.len()
        )));
    }
    if lines[L_HEADER] != "SPC" || lines[L_VERSION] != "0200" {
        return Err(QrBillError::InvalidPayload(
            "en-tête SPC/0200 absent".into(),
        ));
    }
    if lines[L_EPD] != "EPD" {
        return Err(QrBillError::InvalidPayload("trailer EPD absent".into()));
    }

    // IBAN — validate and detect QR-IBAN via the IID (positions 4..9).
    let creditor_iban = validate_iban(lines[L_IBAN])?;
    let is_qr_iban = creditor_iban[4..9]
        .parse::<u32>()
        .map(|iid| (30000..=31999).contains(&iid))
        .unwrap_or(false);

    // Creditor address — type K (combined) or S (structured).
    let address_type = lines[L_CRED_ADRTP].chars().next().unwrap_or('K');
    let structured = address_type == 'S';
    let creditor = ScannedAddress {
        address_type,
        name: lines[L_CRED_NAME].to_string(),
        street_or_line1: lines[L_CRED_LINE1].to_string(),
        building_or_line2: lines[L_CRED_LINE2].to_string(),
        postal_code: if structured {
            non_empty(lines[L_CRED_PSTCD])
        } else {
            None
        },
        town: if structured {
            non_empty(lines[L_CRED_TOWN])
        } else {
            None
        },
        country: lines[L_CRED_COUNTRY].to_string(),
    };

    // Amount — empty field means open amount.
    let amount = if lines[L_AMOUNT].is_empty() {
        None
    } else {
        Some(
            Decimal::from_str(lines[L_AMOUNT])
                .map_err(|_| QrBillError::InvalidAmount(lines[L_AMOUNT].to_string()))?,
        )
    };

    let currency = lines[L_CURRENCY].to_string();

    // Reference.
    let ref_value = lines[L_REF_VALUE];
    let reference = match lines[L_REF_TYPE] {
        "QRR" => {
            validate_qrr(ref_value)?;
            ScannedReference::Qrr(ref_value.to_string())
        }
        "SCOR" => ScannedReference::Scor(ref_value.to_string()),
        "NON" => ScannedReference::None,
        other => {
            return Err(QrBillError::InvalidPayload(format!(
                "type de référence inconnu: {other}"
            )));
        }
    };

    Ok(ScannedQrBill {
        creditor_iban,
        is_qr_iban,
        creditor,
        amount,
        currency,
        reference,
        unstructured_message: non_empty(lines[L_USTRD]),
        // billing_information optional (line 31 absent for 31-line payloads).
        // AltPmtInf (lines 32-33) read implicitly by `get` but ignored.
        billing_information: lines.get(L_BILLING).copied().and_then(non_empty),
    })
}

/// Map an empty string to `None`, a non-empty one to `Some(owned)`.
fn non_empty(s: &str) -> Option<String> {
    if s.is_empty() {
        None
    } else {
        Some(s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generator::build_payload;
    use crate::types::{Address, AddressType, Currency, QrBillData, Reference};
    use crate::validation::build_qrr;
    use rust_decimal_macros::dec;

    const IBAN_OK: &str = "CH9300762011623852957";
    const QR_IBAN_OK: &str = "CH4431999123000889012";

    fn sample(reference: Reference, iban: &str) -> QrBillData {
        QrBillData {
            creditor_iban: iban.into(),
            creditor: Address {
                address_type: AddressType::Combined,
                name: "Robert Schneider SA".into(),
                line1: "Rue du Lac 1268".into(),
                line2: "2501 Biel".into(),
                country: "CH".into(),
            },
            ultimate_debtor: Some(Address {
                address_type: AddressType::Combined,
                name: "Pia Rutschmann".into(),
                line1: "Marktgasse 28".into(),
                line2: "9400 Rorschach".into(),
                country: "CH".into(),
            }),
            amount: Some(dec!(1234.50)),
            currency: Currency::Chf,
            reference,
            unstructured_message: Some("Facture F-2026-0042".into()),
            billing_information: None,
        }
    }

    #[test]
    fn round_trip_type_k_no_reference() {
        let data = sample(Reference::None, IBAN_OK);
        let payload = build_payload(&data).unwrap();
        let scanned = parse_spc_payload(&payload).unwrap();

        assert_eq!(scanned.creditor_iban, IBAN_OK);
        assert!(!scanned.is_qr_iban);
        assert_eq!(scanned.creditor.address_type, 'K');
        assert_eq!(scanned.creditor.name, "Robert Schneider SA");
        assert_eq!(scanned.creditor.street_or_line1, "Rue du Lac 1268");
        assert_eq!(scanned.creditor.building_or_line2, "2501 Biel");
        assert_eq!(scanned.creditor.postal_code, None);
        assert_eq!(scanned.creditor.town, None);
        assert_eq!(scanned.creditor.country, "CH");
        assert_eq!(scanned.amount, Some(dec!(1234.50)));
        assert_eq!(scanned.currency, "CHF");
        assert_eq!(scanned.reference, ScannedReference::None);
        assert_eq!(
            scanned.unstructured_message.as_deref(),
            Some("Facture F-2026-0042")
        );
        assert_eq!(scanned.billing_information, None);
    }

    #[test]
    fn round_trip_type_k_with_qrr() {
        let qrr = build_qrr(42, 100).unwrap();
        let data = sample(Reference::Qrr(qrr.clone()), QR_IBAN_OK);
        let payload = build_payload(&data).unwrap();
        let scanned = parse_spc_payload(&payload).unwrap();

        assert_eq!(scanned.creditor_iban, QR_IBAN_OK);
        assert!(scanned.is_qr_iban);
        assert_eq!(scanned.reference, ScannedReference::Qrr(qrr));
    }

    /// Build a structured-address (type S) payload by hand — Kesh's generator
    /// only emits type K, so a round-trip is impossible (cf. spec AC4 / F-NEW-4).
    fn structured_payload() -> String {
        let mut lines = vec![
            "SPC",
            "0200",
            "1",
            IBAN_OK, // header + iban
            "S",
            "Acme Trading GmbH",
            "Bahnhofstrasse",
            "42",
            "8001",
            "Zürich",
            "CH", // creditor S
        ];
        // Ultimate creditor — 7 empty lines.
        lines.extend([""; 7]);
        lines.push("100.00"); // amount
        lines.push("CHF"); // currency
        // Ultimate debtor — 7 empty lines.
        lines.extend([""; 7]);
        lines.push("NON"); // ref type
        lines.push(""); // ref value
        lines.push("Bestellung 7788"); // unstructured message
        lines.push("EPD"); // trailer
        lines.join("\n")
    }

    #[test]
    fn parses_structured_address() {
        let scanned = parse_spc_payload(&structured_payload()).unwrap();
        assert_eq!(scanned.creditor.address_type, 'S');
        assert_eq!(scanned.creditor.street_or_line1, "Bahnhofstrasse");
        assert_eq!(scanned.creditor.building_or_line2, "42");
        assert_eq!(scanned.creditor.postal_code.as_deref(), Some("8001"));
        assert_eq!(scanned.creditor.town.as_deref(), Some("Zürich"));
        assert_eq!(scanned.creditor.country, "CH");
        assert_eq!(scanned.amount, Some(dec!(100.00)));
        assert_eq!(scanned.reference, ScannedReference::None);
    }

    #[test]
    fn tolerates_31_32_and_34_lines() {
        // 31 lines: billing_information absent entirely.
        let base = structured_payload();
        assert_eq!(base.split('\n').count(), 31);
        let s31 = parse_spc_payload(&base).unwrap();
        assert_eq!(s31.billing_information, None);

        // 32 lines: billing_information present.
        let p32 = format!("{base}\n//S1/10/INV-1");
        let s32 = parse_spc_payload(&p32).unwrap();
        assert_eq!(s32.billing_information.as_deref(), Some("//S1/10/INV-1"));

        // 34 lines: AltPmtInf present (lines 32-33) — read but ignored.
        let p34 = format!("{base}\n//S1/10/INV-1\nName AV1: UV;Ultapayment\nName AV2: XY");
        let s34 = parse_spc_payload(&p34).unwrap();
        assert_eq!(s34.billing_information.as_deref(), Some("//S1/10/INV-1"));
    }

    #[test]
    fn empty_amount_yields_none() {
        let base = structured_payload();
        // Replace the amount line (index 18) with empty.
        let mut lines: Vec<String> = base.split('\n').map(String::from).collect();
        lines[L_AMOUNT].clear();
        let scanned = parse_spc_payload(&lines.join("\n")).unwrap();
        assert_eq!(scanned.amount, None);
    }

    #[test]
    fn rejects_non_spc_header() {
        let bad = "XXX\n0200\n1\n".to_string() + &"\n".repeat(40);
        assert!(matches!(
            parse_spc_payload(&bad),
            Err(QrBillError::InvalidPayload(_))
        ));
    }

    #[test]
    fn rejects_invalid_iban() {
        let mut lines: Vec<String> = structured_payload().split('\n').map(String::from).collect();
        lines[L_IBAN] = "CH00INVALIDIBAN000000".to_string();
        assert!(matches!(
            parse_spc_payload(&lines.join("\n")),
            Err(QrBillError::InvalidIban(_))
        ));
    }

    #[test]
    fn rejects_invalid_qrr_checksum() {
        // Valid QR-IBAN + QRR type but corrupt the check digit.
        let qrr = build_qrr(42, 100).unwrap();
        let mut bad_qrr = qrr.clone();
        let last: u32 = bad_qrr.pop().unwrap().to_digit(10).unwrap();
        bad_qrr.push(char::from_digit((last + 1) % 10, 10).unwrap());
        let data = sample(Reference::Qrr(qrr), QR_IBAN_OK);
        let payload = build_payload(&data).unwrap();
        let mut lines: Vec<String> = payload.split('\n').map(String::from).collect();
        lines[L_REF_VALUE] = bad_qrr;
        assert!(matches!(
            parse_spc_payload(&lines.join("\n")),
            Err(QrBillError::InvalidQrr(_))
        ));
    }

    #[test]
    fn rejects_unknown_reference_type() {
        let mut lines: Vec<String> = structured_payload().split('\n').map(String::from).collect();
        lines[L_REF_TYPE] = "ZZZ".to_string();
        assert!(matches!(
            parse_spc_payload(&lines.join("\n")),
            Err(QrBillError::InvalidPayload(_))
        ));
    }

    #[test]
    fn rejects_too_few_lines() {
        assert!(matches!(
            parse_spc_payload("SPC\n0200\n1"),
            Err(QrBillError::InvalidPayload(_))
        ));
    }
}
