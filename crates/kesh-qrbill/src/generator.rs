//! SIX 2.2 QR Code payload builder and QR matrix renderer.
//!
//! The payload is a newline-separated string whose fields follow the strict
//! order defined by SIX 2.2 §3. ECC level M, UTF-8.

use crate::types::{AddressType, QrBillData, QrBillError};
use crate::validation::{normalize_iban, validate};
use qrcodegen::{QrCode, QrCodeEcc};
use rust_decimal::{Decimal, RoundingStrategy};

/// Build the full QR Code payload string for a validated `QrBillData`.
///
/// Calls [`validate`] internally — the caller does not need to pre-validate.
pub fn build_payload(data: &QrBillData) -> Result<String, QrBillError> {
    validate(data)?;

    let iban = normalize_iban(&data.creditor_iban);
    let mut lines: Vec<String> = Vec::with_capacity(34);

    // Header.
    lines.push("SPC".into());
    lines.push("0200".into());
    lines.push("1".into());

    // Creditor Information — IBAN.
    lines.push(iban);

    // Creditor block (type K).
    lines.push(AddressType::Combined.code().into());
    lines.push(data.creditor.name.clone());
    lines.push(data.creditor.line1.clone());
    lines.push(data.creditor.line2.clone());
    lines.push(String::new()); // PstCd (empty in type K)
    lines.push(String::new()); // TmNm (empty in type K)
    lines.push(data.creditor.country.clone());

    // Ultimate Creditor (7 empty lines in v0.1).
    for _ in 0..7 {
        lines.push(String::new());
    }

    // CcyAmt.
    let amount = data.amount.expect("validated: amount required");
    let rounded = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    lines.push(format_amount_payload(rounded));
    lines.push(data.currency.code().into());

    // Ultimate Debtor block (type K if present; otherwise 7 empty lines).
    match &data.ultimate_debtor {
        Some(debtor) => {
            lines.push(AddressType::Combined.code().into());
            lines.push(debtor.name.clone());
            lines.push(debtor.line1.clone());
            lines.push(debtor.line2.clone());
            lines.push(String::new());
            lines.push(String::new());
            lines.push(debtor.country.clone());
        }
        None => {
            for _ in 0..7 {
                lines.push(String::new());
            }
        }
    }

    // RmtInf.
    lines.push(data.reference.tp_code().into());
    lines.push(data.reference.ref_value().into());

    // AddInf.
    lines.push(data.unstructured_message.clone().unwrap_or_default());
    lines.push("EPD".into());
    lines.push(data.billing_information.clone().unwrap_or_default());

    // AltPmtInf omitted (both lines absent — valid per SIX).

    let payload = lines.join("\n");
    // M-Edge (review pass 1 G2 C) : SIX 2.2 §3 limite stricte 997 caractères.
    // Pass 2 : compter en BYTES (capacité QR Code en mode UTF-8) et non en
    // codepoints — un caractère Latin Extended-A = 2 octets, donc 500 chars
    // peuvent dépasser 997 octets et excéder la capacité QR.
    const SIX_PAYLOAD_MAX_BYTES: usize = 997;
    if payload.len() > SIX_PAYLOAD_MAX_BYTES {
        return Err(QrBillError::PdfGeneration(format!(
            "payload {} octets > {SIX_PAYLOAD_MAX_BYTES} (limite SIX 2.2 §3)",
            payload.len()
        )));
    }
    Ok(payload)
}

/// Render the payload into a QR matrix at ECC level M.
pub fn render_qr_image(payload: &str) -> Result<QrCode, QrBillError> {
    QrCode::encode_text(payload, QrCodeEcc::Medium)
        .map_err(|e| QrBillError::PdfGeneration(format!("qrcode: {e}")))
}

/// Format amount for the QR payload: point decimal, no thousand separator, 2 decimals.
///
/// C1-Blind (review pass 1 G2 C) : utilise le ré-arrondi explicite + format
/// `{:.2}` plutôt qu'un slicing byte qui (1) tronquerait silencieusement
/// au-delà de 2 décimales si le `round_dp` upstream sautait, (2) panique
/// théorique sur multi-byte (impossible avec Decimal aujourd'hui mais piège).
fn format_amount_payload(amount: Decimal) -> String {
    // CRITICAL pass 2 : `format!("{:.2}", ...)` garantit point décimal +
    // exactement 2 décimales quelle que soit la représentation interne
    // (notation scientifique éventuelle de `Decimal::to_string()`).
    let rounded = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    format!("{:.2}", rounded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Address, AddressType, Currency, QrBillData, Reference};
    use rust_decimal_macros::dec;

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
    fn payload_starts_with_spc_header() {
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        assert!(payload.starts_with("SPC\n0200\n1\n"));
        assert!(payload.contains("\nEPD\n") || payload.ends_with("\nEPD\n") || payload.contains("EPD"));
    }

    #[test]
    fn payload_has_exactly_32_lines() {
        // Pass 2 : strict `==32` (sans AltPmtInf, billing_information=None
        // mais ligne toujours émise → toujours 32). 33 ne peut être atteint
        // que si AltPmtInf est ajouté (hors v0.1).
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        let n = payload.split('\n').count();
        assert_eq!(n, 32, "expected exactly 32 lines (no AltPmtInf), got {n}");
    }

    #[test]
    fn amount_uses_point_no_thousand_sep() {
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        // Amount line is the 22nd (index 21): 3 header + 1 iban + 7 creditor + 7 uc + amt
        let lines: Vec<&str> = payload.split('\n').collect();
        assert_eq!(lines[3 + 1 + 7 + 7], "1234.50");
    }

    #[test]
    fn reference_qrr_populates_tp_and_ref() {
        let qrr = crate::validation::build_qrr(42, 100).unwrap();
        let data = sample(Reference::Qrr(qrr.clone()), "CH4431999123000889012");
        let payload = build_payload(&data).unwrap();
        let lines: Vec<&str> = payload.split('\n').collect();
        // Find "QRR" line.
        let idx = lines.iter().position(|l| *l == "QRR").expect("QRR line");
        assert_eq!(lines[idx + 1], qrr);
    }

    #[test]
    fn reference_none_empty_ref() {
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        let lines: Vec<&str> = payload.split('\n').collect();
        let idx = lines.iter().position(|l| *l == "NON").unwrap();
        assert_eq!(lines[idx + 1], "");
    }

    #[test]
    fn qr_matrix_renders() {
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        let code = render_qr_image(&payload).unwrap();
        // Size for ECC M and this payload length: at least version 10 (57x57).
        assert!(code.size() >= 41);
    }

    #[test]
    fn qr_roundtrip_via_rxing() {
        // Round-trip: encode payload, render QR matrix, rasterize into a bitmap,
        // then decode with `rxing` and assert the decoded string matches.
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        let code = render_qr_image(&payload).unwrap();

        // Build an 8-bit luminance bitmap: 1 module = 10×10 px, 4-module quiet zone.
        let scale = 10_usize;
        let quiet = 4_usize;
        let size = code.size() as usize;
        let dim = (size + 2 * quiet) * scale;
        let mut bitmap = vec![255u8; dim * dim];
        for y in 0..size {
            for x in 0..size {
                if code.get_module(x as i32, y as i32) {
                    for dy in 0..scale {
                        for dx in 0..scale {
                            let py = (quiet + y) * scale + dy;
                            let px = (quiet + x) * scale + dx;
                            bitmap[py * dim + px] = 0;
                        }
                    }
                }
            }
        }

        use rxing::BarcodeFormat;
        let result = rxing::helpers::detect_in_luma(
            bitmap,
            dim as u32,
            dim as u32,
            Some(BarcodeFormat::QR_CODE),
        )
        .expect("rxing decode");
        assert_eq!(result.getText(), payload);
    }
}
