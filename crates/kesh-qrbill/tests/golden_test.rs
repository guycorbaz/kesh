//! Golden / déterminisme — Story 5.3 AC15.
//!
//! Plan C (validé avec Guy) : on ne compare pas l'octet exact du PDF (printpdf
//! introduit un random dans le second élément du trailer `/ID`). À la place :
//! 1. Le **payload QR Bill** est intégralement déterministe et reproductible.
//! 2. La **taille** du PDF ne varie pas entre générations successives avec la
//!    même date de création figée.

use chrono::NaiveDate;
use kesh_qrbill::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, QrBillData,
    QrBillI18n, Reference, generate_qr_bill_pdf_with_date, generator::build_payload,
};
use rust_decimal_macros::dec;
use std::collections::HashMap;
use time::{Date, Month, OffsetDateTime, Time, UtcOffset};

fn fixed_creation_date() -> OffsetDateTime {
    let d = Date::from_calendar_date(2026, Month::January, 1).unwrap();
    OffsetDateTime::new_in_offset(d, Time::MIDNIGHT, UtcOffset::UTC)
}

fn sample_qr_data() -> QrBillData {
    QrBillData {
        creditor_iban: "CH4431999123000889012".into(),
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
        amount: Some(dec!(1234.56)),
        currency: Currency::Chf,
        reference: Reference::Qrr("210000000003139471430009017".into()),
        unstructured_message: Some("Facture F-2026-0042".into()),
        billing_information: None,
    }
}

fn sample_invoice() -> InvoicePdfData {
    InvoicePdfData {
        invoice_number: "F-2026-0042".into(),
        invoice_date: NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
        due_date: Some(NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
        payment_terms: Some("30 jours net".into()),
        creditor_name: "Robert Schneider SA".into(),
        creditor_address_lines: vec!["Rue du Lac 1268".into(), "2501 Biel".into()],
        creditor_ide: Some("CHE-123.456.789".into()),
        // Story 16-3a (#151) — la fixture de base ne porte AUCUNE coordonnée :
        // c'est le cas nominal d'une société qui n'a rien renseigné, et il doit
        // rendre le PDF d'avant la story. Les tests qui les exercent les posent.
        creditor_phone: None,
        creditor_email: None,
        creditor_website: None,
        debtor_name: "Pia Rutschmann".into(),
        debtor_address_lines: vec!["Marktgasse 28".into(), "9400 Rorschach".into()],
        debtor_client_number: None,
        lines: vec![InvoiceLinePdf {
            description: "Conseil".into(),
            quantity: dec!(1),
            unit_price: dec!(1234.56),
            vat_rate: dec!(7.70),
            line_total: dec!(1234.56),
        }],
        subtotal_ht: dec!(1234.56),
        vat_lines: vec![InvoiceVatLinePdf {
            rate_percent: dec!(7.70),
            amount: dec!(95.06), // 1234.56 × 7.70 % arrondi
        }],
        total: dec!(1329.62), // 1234.56 + 95.06
        currency: Currency::Chf,
        origin_reference: None,
    }
}

#[test]
fn payload_is_deterministic_across_10_runs() {
    let data = sample_qr_data();
    let first = build_payload(&data).expect("build payload");
    for _ in 0..9 {
        let next = build_payload(&data).unwrap();
        assert_eq!(
            first, next,
            "payload QR Bill doit être strictement déterministe"
        );
    }
}

#[test]
fn pdf_size_stable_with_fixed_date() {
    let data = sample_qr_data();
    let invoice = sample_invoice();
    let i18n = QrBillI18n::new(HashMap::new());
    let date = fixed_creation_date();
    let a = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, date).unwrap();
    let b = generate_qr_bill_pdf_with_date(&data, &invoice, &i18n, date).unwrap();
    // Les octets peuvent différer sur le random trailer /ID (printpdf), mais
    // la taille et le préfixe PDF doivent rester stables.
    assert_eq!(a.len(), b.len(), "taille PDF instable");
    assert!(a.starts_with(b"%PDF-1."));
}
