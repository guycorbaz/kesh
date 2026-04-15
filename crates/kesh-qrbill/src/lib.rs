//! Kesh QR Bill — Swiss QR Bill SIX 2.2 payload and PDF generation.
//!
//! Standalone crate: zero dependency on `kesh-core` / `kesh-db` (DD-14).
//! Input is plain Rust types plus an injected i18n bundle; output is a PDF byte vector.
//!
//! Entry points:
//! - [`generate_qr_bill_pdf`] — produce the PDF bytes for an invoice + QR bill.
//! - [`validate`] — static validation of `QrBillData` against SIX 2.2 rules.
//! - [`build_payload`] — the raw SIX payload string (for tests / external usage).

pub mod generator;
pub mod pdf;
pub mod types;
pub mod validation;

pub use generator::build_payload;
pub use pdf::{generate_qr_bill_pdf, generate_qr_bill_pdf_with_date};
pub use types::{
    Address, AddressType, Currency, InvoiceLinePdf, InvoicePdfData, QrBillData, QrBillError,
    QrBillI18n, Reference,
};
pub use validation::validate;
