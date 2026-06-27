//! kesh-report — Générateurs de rapports comptables.
//!
//! Cette crate fournit les 4 rapports comptables réglementaires :
//! - [`balance_sheet`] — Bilan (actifs / passifs / capitaux propres)
//! - [`income_statement`] — Compte de résultat (produits / charges)
//! - [`trial_balance`] — Balance des comptes (soldes débit/crédit)
//! - [`journal_report`] — Journaux (Achats, Ventes, Banque, Caisse, OD)
//!
//! Tous les rapports retournent des structures `Serialize` en camelCase pour
//! consommation directe par les routes HTTP de `kesh-api`.
//!
//! Story 9-2a — Sérialiseurs export PDF & CSV :
//! - [`pdf`] — 4 fonctions `render_*_pdf` (printpdf 0.7, Helvetica builtin).
//! - [`csv`] — 4 fonctions `render_*_csv` (csv 1.3, BOM UTF-8, `;` delimiter, CRLF).

pub mod balance_sheet;
pub mod csv;
pub mod errors;
pub mod income_statement;
pub mod journal_report;
pub mod pdf;
pub mod period;
pub mod trial_balance;
pub mod vat_report;

pub use balance_sheet::{AccountBalance, BalanceSheet, generate as generate_balance_sheet};
pub use errors::ReportError;
pub use income_statement::{IncomeStatement, generate as generate_income_statement};
pub use journal_report::{
    JournalEntryLineRow, JournalEntryRow, JournalReport, JournalSection,
    generate as generate_journal_report,
};
pub use period::ReportPeriod;
pub use trial_balance::{TrialBalance, TrialBalanceRow, generate as generate_trial_balance};
pub use vat_report::{VatReport, VatReportRow, generate as generate_vat_report};

// Story 9-2a — re-exports PDF & CSV
pub use csv::{
    render_balance_sheet_csv, render_income_statement_csv, render_journal_report_csv,
    render_trial_balance_csv,
};
pub use pdf::{
    PdfContext, SectionLabels, render_balance_sheet_pdf, render_income_statement_pdf,
    render_journal_report_pdf, render_trial_balance_pdf,
};

// Story 11-2 — re-exports rapport TVA
pub use csv::render_vat_report_csv;
pub use pdf::{VatPdfLabels, render_vat_report_pdf};
