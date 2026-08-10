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

/// SIX QR Bill address block. Supports both type **K** (Combined, deprecated by
/// SIX 21.11.2025) and type **S** (Structured, now mandatory for generation).
///
/// The 4 free-text data elements map to SIX payload positions per `address_type`:
///
/// | field         | type S (Structured)   | type K (Combined)       |
/// |---------------|-----------------------|-------------------------|
/// | `line1`       | street name (≤70)     | address line 1 (≤70)    |
/// | `line2`       | building number (≤16) | address line 2 (≤70)    |
/// | `postal_code` | postal code / NPA(≤16)| *empty*                 |
/// | `town`        | town / locality (≤35) | *empty*                 |
#[derive(Debug, Clone)]
pub struct Address {
    pub address_type: AddressType,
    /// Name, ≤70 chars.
    pub name: String,
    /// SIX element 3 — street name (type S) or free-form line 1 (type K).
    pub line1: String,
    /// SIX element 4 — building number (type S, ≤16) or free-form line 2 (type K).
    pub line2: String,
    /// SIX element 5 — postal code (type S only; empty for type K).
    pub postal_code: String,
    /// SIX element 6 — town / locality (type S only; empty for type K).
    pub town: String,
    /// SIX element 7 — ISO-3166-1 alpha-2 country code (e.g. "CH", "LI").
    pub country: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressType {
    /// Combined — 2 free-form address lines (deprecated by SIX 21.11.2025).
    Combined,
    /// Structured — separate street / building / postal / town fields.
    /// Mandatory format for QR-bill generation since SIX 21.11.2025.
    Structured,
}

impl AddressType {
    pub fn code(self) -> &'static str {
        match self {
            AddressType::Combined => "K",
            AddressType::Structured => "S",
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
    /// Coordonnées de contact de l'émetteur (Story 16-3a, #151), rendues sous
    /// l'IDE. `None` ⇒ la ligne n'est pas dessinée et le curseur ne descend
    /// pas — même patron conditionnel que `creditor_ide`.
    pub creditor_phone: Option<String>,
    pub creditor_email: Option<String>,
    pub creditor_website: Option<String>,
    pub debtor_name: String,
    pub debtor_address_lines: Vec<String>,
    pub lines: Vec<InvoiceLinePdf>,
    /// Sous-total HT (= Σ des `line_total` HT). Affiché dans le récap TVA (#151).
    pub subtotal_ht: Decimal,
    /// Récapitulatif TVA par taux (#151), calculé par le service via
    /// `kesh_core::accounting::vat::vat_breakdown_by_rate` (arrondi par ligne
    /// DC7). **Vide** = aucune ligne taxée → pas de bloc TVA (0 % / hors champ),
    /// on n'affiche alors que le total (rétro-compatible).
    pub vat_lines: Vec<InvoiceVatLinePdf>,
    /// TTC total, from DB (Decimal(19,4)). Rounded to 2 decimals for display.
    pub total: Decimal,
    pub currency: Currency,
    /// Référence à la facture d'origine, affichée uniquement sur les avoirs
    /// (Story 12.1). `None` pour une facture normale (pas de régression).
    pub origin_reference: Option<String>,
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

/// Une ligne du récapitulatif TVA du PDF (#151) : un taux + le montant de TVA
/// cumulé à ce taux. Purement présentationnel — l'agrégation (arrondi par ligne
/// DC7) est faite en amont par `kesh_core::accounting::vat`.
#[derive(Debug, Clone)]
pub struct InvoiceVatLinePdf {
    /// Taux en pourcent (ex. `8.10` pour 8.1 %).
    pub rate_percent: Decimal,
    /// Montant de TVA cumulé à ce taux.
    pub amount: Decimal,
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
    // Story 12.1 — référence à la facture d'origine (avoirs uniquement).
    "invoice-pdf-origin-reference",
    // Story 16-3a (#151) — coordonnées de contact de l'émetteur.
    //
    // ⚠️ Toute clé ajoutée ici DOIT l'être à la MÊME POSITION dans `DEFAULT_EN` :
    // `get()` résout son repli par `DEFAULT_EN[idx]`. L'appariement positionnel
    // est à la charge du mainteneur — l'assertion de compilation plus bas
    // n'attrape QUE les longueurs divergentes, pas un décalage à longueurs
    // égales (cf. son doc-comment).
    "invoice-pdf-phone",
    "invoice-pdf-email",
    "invoice-pdf-website",
];

/// ⚠️ Invariant tenu **à la compilation** : `I18N_KEYS` et `DEFAULT_EN` ont
/// exactement la même longueur. `get()` résout son repli par `DEFAULT_EN[idx]`
/// **sans borne-check** — une longueur divergente ferait paniquer le rendu, en
/// debug comme en release. Le commentaire l'annonçait sans le tenir ; cette
/// assertion échoue désormais au `cargo build`, pas au premier PDF.
/// *(Revue de code, passe 3.)*
///
/// ⚠️ Ce qu'elle ne tient PAS : l'**appariement** clé ↔ traduction. Insérer une
/// clé au milieu de l'une et ailleurs dans l'autre laisse les longueurs égales,
/// passe `cargo build`, et décale silencieusement le repli de toutes les
/// entrées suivantes. Aucune vérification statique ne peut apparier une clé à
/// sa traduction ; l'ordre reste à la charge du mainteneur, et le seul filet
/// est d'ajouter toute nouvelle clé **en fin** des deux tableaux. Aggravé par
/// les fixtures à `HashMap` vide, qui traversent toutes `DEFAULT_EN`.
/// *(Revue de code, passe 6 — le doc-comment prêtait à l'assertion une garantie
/// de position qu'elle n'a jamais eue.)*
const _: () = assert!(
    I18N_KEYS.len() == DEFAULT_EN.len(),
    "I18N_KEYS et DEFAULT_EN doivent avoir la même longueur"
);

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
    "Original invoice",
    // Story 16-3a (#151) — MÊME ORDRE que les trois dernières de `I18N_KEYS`.
    "Phone",
    "Email",
    "Web",
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

    /// Trop de lignes pour tenir sur un PDF A4 mono-page (avec le récap TVA #151).
    /// Distinct de `PdfGeneration` : le handler HTTP le mappe en **400** « trop
    /// de lignes » (actionnable) plutôt qu'un 500 opaque. Le `usize` = nb de lignes.
    #[error("trop de lignes ({0}) pour un PDF A4 mono-page")]
    TooManyLines(usize),

    /// Le bloc d'en-tête (émetteur + destinataire) déborde sur le tableau des
    /// lignes (Story 16-3a, #151). Le `f32` = ordonnée atteinte, en mm.
    ///
    /// ⚠️ Garde **symétrique** de `TooManyLines`, qui ne surveillait que le bas
    /// de page : le tableau démarre à une ordonnée **constante** que rien ne
    /// repousse, si bien qu'un en-tête trop haut le chevauchait **en silence**.
    #[error("l'en-tête déborde sur le tableau des lignes (y = {0} mm)")]
    HeaderOverflow(f32),
    /// Payload SPC malformé (en-tête absent, type de référence inconnu, structure invalide).
    /// Mappé `INVALID_SPC_PAYLOAD` par la couche d'import (Story 12-5).
    #[error("Payload SPC invalide: {0}")]
    InvalidPayload(String),
}
