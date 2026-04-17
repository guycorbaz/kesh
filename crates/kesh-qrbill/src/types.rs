//! Public types for the `kesh-qrbill` crate.
//!
//! All types are self-contained — no dependency on other `kesh-*` crates (DD-14).
//! Callers convert their domain entities into these types at the boundary.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashMap;
use thiserror::Error;

/// QR Bill payload + PDF input — the full data required to emit a Swiss QR Bill.
#[derive(Debug, Clone)]
pub struct QrBillData {
    /// IBAN or QR-IBAN, 21 characters, without spaces. Country code must be CH or LI.
    pub creditor_iban: String,
    /// Creditor address block (SIX type K).
    pub creditor: Address,
    /// Ultimate debtor address block (always `Some` in v0.1 — no "au porteur" support).
    pub ultimate_debtor: Option<Address>,
    /// Amount to be paid. `None` = open amount (not supported in v0.1 — always `Some`).
    pub amount: Option<Decimal>,
    /// Currency — `CHF` or `EUR`.
    pub currency: Currency,
    /// Reference type + value. QRR (27 digits) for QR-IBAN, None otherwise in v0.1.
    pub reference: Reference,
    /// Free-form message to the debtor, ≤140 chars.
    pub unstructured_message: Option<String>,
    /// Structured billing information, ≤140 chars — left `None` in v0.1.
    pub billing_information: Option<String>,
}

/// SIX QR Bill address block. Only type K (Combined) supported in v0.1.
#[derive(Debug, Clone)]
pub struct Address {
    pub address_type: AddressType,
    /// Name, ≤70 chars.
    pub name: String,
    /// Line 1 (street or free-form line), ≤70 chars.
    pub line1: String,
    /// Line 2 (postal code + city or free-form line), ≤70 chars.
    pub line2: String,
    /// ISO-3166-1 alpha-2 country code (e.g. "CH", "LI").
    pub country: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    /// Combined — 2 free-form address lines (v0.1 default).
    Combined,
}

impl AddressType {
    pub fn code(self) -> &'static str {
        match self {
            AddressType::Combined => "K",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Chf,
    Eur,
}

impl Currency {
    pub fn code(self) -> &'static str {
        match self {
            Currency::Chf => "CHF",
            Currency::Eur => "EUR",
        }
    }
}

/// Reference type — SIX 2.2 §3. SCOR not supported in v0.1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reference {
    /// QR-Reference (27 digits, mod-10 recursive checksum).
    Qrr(String),
    /// No reference — `Tp = NON`, `Ref` empty.
    None,
}

impl Reference {
    pub fn tp_code(&self) -> &'static str {
        match self {
            Reference::Qrr(_) => "QRR",
            Reference::None => "NON",
        }
    }

    pub fn ref_value(&self) -> &str {
        match self {
            Reference::Qrr(s) => s.as_str(),
            Reference::None => "",
        }
    }
}

/// Invoice part of the PDF (top section, above the QR Bill).
#[derive(Debug, Clone)]
pub struct InvoicePdfData {
    pub invoice_number: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub creditor_name: String,
    pub creditor_address_lines: Vec<String>,
    /// Formatted IDE number, e.g. "CHE-123.456.789".
    pub creditor_ide: Option<String>,
    pub debtor_name: String,
    pub debtor_address_lines: Vec<String>,
    pub lines: Vec<InvoiceLinePdf>,
    /// TTC total, from DB (Decimal(19,4)). Rounded to 2 decimals for display.
    pub total: Decimal,
    pub currency: Currency,
}

#[derive(Debug, Clone)]
pub struct InvoiceLinePdf {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    /// VAT rate in percent (e.g. 7.70 for 7.7%).
    pub vat_rate: Decimal,
    pub line_total: Decimal,
}

/// Injected translations — the crate has no direct dependency on `kesh-i18n`.
///
/// The API caller fills this HashMap with already-translated strings keyed by
/// stable identifiers (see [`I18N_KEYS`] below). Missing keys fall back to their
/// English default (best-effort, tests should never hit fallback).
#[derive(Debug, Clone, Default)]
pub struct QrBillI18n {
    pub entries: HashMap<&'static str, String>,
}

impl QrBillI18n {
    pub fn new(entries: HashMap<&'static str, String>) -> Self {
        Self { entries }
    }

    pub fn get(&self, key: &'static str) -> &str {
        if let Some(v) = self.entries.get(key) {
            return v.as_str();
        }
        match I18N_KEYS.iter().position(|k| *k == key) {
            Some(idx) => DEFAULT_EN[idx],
            None => {
                // M1-Blind (review pass 1 G2 C) : pas d'IO stderr depuis une lib
                // publiable. `debug_assert!` panique en debug ; en release on
                // retourne silencieusement la clé (le caller reverra le raw key
                // dans le PDF, ce qui rend le bug visible sans polluer stderr).
                debug_assert!(false, "QrBillI18n::get called with unknown key: {key}");
                key
            }
        }
    }
}

/// Stable i18n keys used by the generator. Keep in sync with `DEFAULT_EN`.
pub const I18N_KEYS: &[&str] = &[
    "invoice-pdf-title",
    "invoice-pdf-date",
    "invoice-pdf-due-date",
    "invoice-pdf-number",
    "invoice-pdf-ide",
    "invoice-pdf-recipient",
    "invoice-pdf-description",
    "invoice-pdf-quantity",
    "invoice-pdf-unit-price",
    "invoice-pdf-vat",
    "invoice-pdf-line-total",
    "invoice-pdf-subtotal",
    "invoice-pdf-total",
    "invoice-pdf-total-ttc",
    "invoice-pdf-payment-terms",
    "invoice-pdf-qr-section-payment",
    "invoice-pdf-qr-section-receipt",
    "invoice-pdf-qr-account",
    "invoice-pdf-qr-reference",
    "invoice-pdf-qr-additional-info",
    "invoice-pdf-qr-payable-by",
    "invoice-pdf-qr-currency",
    "invoice-pdf-qr-amount",
    "invoice-pdf-qr-acceptance-point",
    "invoice-pdf-qr-separate-before-paying",
];

/// English fallback for each key (same ordering as `I18N_KEYS`).
const DEFAULT_EN: &[&str] = &[
    "Invoice",
    "Date",
    "Due date",
    "Invoice number",
    "VAT ID",
    "Recipient",
    "Description",
    "Qty",
    "Unit price",
    "VAT",
    "Total",
    "Subtotal",
    "Total",
    "Total (incl. VAT)",
    "Payment terms",
    "Payment part",
    "Receipt",
    "Account / Payable to",
    "Reference",
    "Additional information",
    "Payable by",
    "Currency",
    "Amount",
    "Acceptance point",
    "Separate before paying in",
];

#[derive(Debug, Error)]
pub enum QrBillError {
    #[error("IBAN invalide: {0}")]
    InvalidIban(String),
    #[error("QR-IBAN invalide: {0}")]
    InvalidQrIban(String),
    #[error("Référence QRR invalide: {0}")]
    InvalidQrr(String),
    #[error("Champ {field} trop long (max {max}, got {got})")]
    FieldTooLong {
        field: &'static str,
        max: usize,
        got: usize,
    },
    #[error("Champ {0} vide (requis)")]
    FieldEmpty(&'static str),
    #[error("Montant invalide: {0}")]
    InvalidAmount(String),
    #[error("Devise invalide: {0}")]
    InvalidCurrency(String),
    #[error("Code pays invalide: {0} (attendu ISO-3166-1 alpha-2)")]
    InvalidCountry(String),
    #[error("Champ {field} contient un caractère non autorisé par SIX 2.2: U+{codepoint:04X}")]
    InvalidCharset { field: &'static str, codepoint: u32 },
    #[error("Erreur génération PDF: {0}")]
    PdfGeneration(String),
}
