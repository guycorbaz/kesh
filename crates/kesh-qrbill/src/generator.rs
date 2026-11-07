//! SIX 2.2 QR Code payload builder and QR matrix renderer.
//!
//! The payload is a newline-separated string whose fields follow the strict
//! order defined by SIX 2.2 §3. ECC level M, UTF-8.

use crate::texte::texte_code;
use crate::types::{Address, QrBillData, QrBillError};
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

    // Creditor block (type S structured or K combined, per address_type).
    push_address_block(&mut lines, &data.creditor);

    // Ultimate Creditor (7 empty lines in v0.1).
    for _ in 0..7 {
        lines.push(String::new());
    }

    // CcyAmt.
    let amount = data.amount.expect("validated: amount required");
    let rounded = amount.round_dp_with_strategy(2, RoundingStrategy::MidpointAwayFromZero);
    lines.push(format_amount_payload(rounded));
    lines.push(data.currency.code().into());

    // Ultimate Debtor block (structured/combined if present; else 7 empty lines).
    match &data.ultimate_debtor {
        Some(debtor) => push_address_block(&mut lines, debtor),
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
    lines.push(texte_code(
        data.unstructured_message.as_deref().unwrap_or_default(),
    ));
    lines.push("EPD".into());
    lines.push(texte_code(
        data.billing_information.as_deref().unwrap_or_default(),
    ));

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
/// Push the 7 SIX payload data elements of an address block (creditor or debtor):
/// AdrTp, Name, StrtNmOrAddrLine1, BldgNbOrAddrLine2, PstCd, TwnNm, Ctry.
///
/// For type **S** (Structured) `postal_code`/`town` carry the NPA/locality; for
/// type **K** (Combined) they are empty strings (SIX §3.3).
fn push_address_block(lines: &mut Vec<String>, addr: &Address) {
    lines.push(addr.address_type.code().into());
    lines.push(texte_code(&addr.name));
    lines.push(texte_code(&addr.line1));
    lines.push(texte_code(&addr.line2));
    lines.push(texte_code(&addr.postal_code));
    lines.push(texte_code(&addr.town));
    lines.push(addr.country.clone());
}

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
                postal_code: String::new(),
                town: String::new(),
                country: "CH".into(),
            },
            ultimate_debtor: Some(Address {
                address_type: AddressType::Combined,
                name: "Pia Rutschmann".into(),
                line1: "Marktgasse 28".into(),
                line2: "9400 Rorschach".into(),
                postal_code: String::new(),
                town: String::new(),
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
    fn structured_creditor_emits_type_s_block() {
        // Conformité SIX 21.11.2025 : la génération doit produire une adresse
        // STRUCTURÉE (type S) — AdrTp=S, rue/n°/NPA/localité en éléments séparés.
        let mut data = sample(Reference::None, "CH9300762011623852957");
        data.creditor = Address {
            address_type: AddressType::Structured,
            name: "Robert Schneider SA".into(),
            line1: "Rue du Lac".into(),
            line2: "1268".into(),
            postal_code: "2501".into(),
            town: "Biel".into(),
            country: "CH".into(),
        };
        let payload = build_payload(&data).unwrap();
        let lines: Vec<&str> = payload.split('\n').collect();
        // Creditor block = indices 4..=10 (3 header + 1 iban).
        assert_eq!(lines[4], "S", "AdrTp créancier doit être S (structuré)");
        assert_eq!(lines[5], "Robert Schneider SA");
        assert_eq!(lines[6], "Rue du Lac", "element 3 = rue");
        assert_eq!(lines[7], "1268", "element 4 = n° bâtiment");
        assert_eq!(lines[8], "2501", "element 5 = NPA (non vide en type S)");
        assert_eq!(
            lines[9], "Biel",
            "element 6 = localité (non vide en type S)"
        );
        assert_eq!(lines[10], "CH");
    }

    #[test]
    fn payload_starts_with_spc_header() {
        let data = sample(Reference::None, "CH9300762011623852957");
        let payload = build_payload(&data).unwrap();
        assert!(payload.starts_with("SPC\n0200\n1\n"));
        assert!(
            payload.contains("\nEPD\n") || payload.ends_with("\nEPD\n") || payload.contains("EPD")
        );
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

    /// **AC5** (Story 33-12a, #941) — chacun des douze champs texte porte
    /// `texte_code` de sa valeur, à sa position exacte. Les valeurs sont toutes
    /// distinctes : un champ échangé ou oublié rougit à sa position.
    #[test]
    fn payload_porte_texte_code_a_chaque_position() {
        let adresse = |prefixe: &str| Address {
            address_type: AddressType::Structured,
            name: format!("{prefixe}\u{2019}nom"),
            line1: format!("\u{A0}{prefixe}\u{2014}rue "),
            line2: format!("{prefixe}\u{AD}12"),
            postal_code: format!(" {prefixe}10"),
            town: format!("{prefixe}\u{A0}Ville\u{AD}"),
            country: "CH".into(),
        };
        let mut data = sample(Reference::None, "CH9300762011623852957");
        // Une lettre hors WinAnsi dans chaque champ : le code la garde (C-e3312-1).
        data.creditor = adresse("Č");
        data.ultimate_debtor = Some(adresse("Ł"));
        data.unstructured_message = Some("\u{A0}Message\u{2019}ć ".into());
        data.billing_information = Some("//Ș1\u{AD}/10 ".into());

        let payload = build_payload(&data).unwrap();
        let lignes: Vec<&str> = payload.split('\n').collect();
        for (position, attendu) in [
            (5, "Č'nom"),
            (6, "Č-rue"),
            (7, "Č12"),
            (8, "Č10"),
            (9, "Č Ville"),
            (21, "Ł'nom"),
            (22, "Ł-rue"),
            (23, "Ł12"),
            (24, "Ł10"),
            (25, "Ł Ville"),
            (29, "Message'ć"),
            (31, "//Ș1/10"),
        ] {
            assert_eq!(lignes[position], attendu, "position {position}");
        }
    }

    /// **AC8** (Story 33-12a) — une rue faite d'invisibles reste admise, et sort
    /// vide du code au lieu d'y porter des blancs.
    #[test]
    fn rue_d_invisibles_sort_vide_du_code() {
        let mut data = sample(Reference::None, "CH9300762011623852957");
        data.creditor = Address {
            address_type: AddressType::Structured,
            name: "Max".into(),
            line1: "\u{A0}\u{AD} ".into(),
            line2: String::new(),
            postal_code: "1003".into(),
            town: "Lausanne".into(),
            country: "CH".into(),
        };
        let payload = build_payload(&data).unwrap();
        assert_eq!(payload.split('\n').nth(6), Some(""));
    }

    /// **Revue P1 (E-L3)** — la borne de 997 octets porte sur le code normalisé : un
    /// brut au-delà, dont la normalisation raccourcit les octets, passe ; un code
    /// réellement au-delà reste refusé.
    #[test]
    fn borne_de_997_octets_sur_le_code_normalise() {
        let mut data = sample(Reference::None, "CH9300762011623852957");
        data.creditor.name = "\u{2019}".repeat(70);
        data.unstructured_message = Some("\u{2019}".repeat(140));
        data.billing_information = Some("\u{2019}".repeat(140));
        let brut: usize = 350 * '\u{2019}'.len_utf8();
        assert!(brut > 997, "montage : {brut} octets bruts");
        assert!(build_payload(&data).unwrap().len() <= 997);

        let mut trop = sample(Reference::None, "CH9300762011623852957");
        trop.creditor.name = "é".repeat(70);
        trop.creditor.line1 = "é".repeat(70);
        trop.ultimate_debtor = Some(trop.creditor.clone());
        trop.unstructured_message = Some("é".repeat(140));
        trop.billing_information = Some("é".repeat(140));
        assert!(matches!(
            build_payload(&trop),
            Err(QrBillError::PdfGeneration(_))
        ));
    }

    /// **Revue P2 (P2-L5)** — la borne elle-même : 997 octets passent, 998 non.
    #[test]
    fn borne_de_997_octets_exacte() {
        let mut data = sample(Reference::None, "CH9300762011623852957");
        data.creditor.name = "é".repeat(70);
        data.creditor.line1 = "é".repeat(70);
        data.ultimate_debtor = Some(data.creditor.clone());
        data.unstructured_message = Some("é".repeat(140));
        let sans = build_payload(&data).unwrap().len();
        let reste = 997 - sans;
        assert!(reste <= 140, "montage : {reste} octets à combler");

        data.billing_information = Some("x".repeat(reste));
        assert_eq!(build_payload(&data).unwrap().len(), 997);

        data.billing_information = Some("x".repeat(reste + 1));
        assert!(matches!(
            build_payload(&data),
            Err(QrBillError::PdfGeneration(_))
        ));
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
