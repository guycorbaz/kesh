//! Kesh QR Bill — Swiss QR Bill SIX 2.2 payload and PDF generation.
//!
//! Depends on `kesh-core` only (Story 22-1, D3 — no longer standalone), never on `kesh-db`
//! (DD-14). `kesh-report` uses its Helvetica width table ([`largeur_mm`]) to right-align
//! report amounts — a shared font metric, not a shared PDF layer.
//! Input is plain Rust types plus an injected i18n bundle; output is a PDF byte vector.
//!
//! Entry points:
//! - [`generate_qr_bill_pdf`] — produce the PDF bytes for an invoice + QR bill.
//! - [`validate`] — static validation of `QrBillData` against SIX 2.2 rules.
//! - [`build_payload`] — the raw SIX payload string (for tests / external usage).
//! - [`parse_spc_payload`] — decode a scanned SPC payload back into typed data.

#[cfg(feature = "test-support")]
pub mod empreinte;
pub mod generator;
pub(crate) mod largeurs;
pub mod parser;
pub mod pdf;
pub(crate) mod texte;
pub mod types;
pub mod validation;

pub use generator::build_payload;
pub use largeurs::largeur_mm;
pub use parser::{ScannedAddress, ScannedQrBill, ScannedReference, parse_spc_payload};
pub use pdf::{
    REMINDER_LABEL_MAX_CHARS, REMINDER_NOTE_MAX_CHARS, generate_credit_note_pdf,
    generate_credit_note_pdf_with_date, generate_qr_bill_pdf, generate_qr_bill_pdf_with_date,
};
pub use types::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, InvoiceVatLinePdf, LinePdfKind,
    QrBillData, QrBillError, QrBillI18n, Reference, ReminderPdf,
};
pub use validation::{champ_vide_pour_qr, first_qr_refused_char, is_iso_3166_alpha2, validate};
