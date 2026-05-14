//! kesh-report — Générateurs de rapports comptables.
//!
//! Cette crate fournit les 4 rapports comptables réglementaires :
//! - [`balance_sheet`] — Bilan (actifs / passifs / capitaux propres)
//! - [`income_statement`] — Compte de résultat (produits / charges)
//! - [`trial_balance`] — Balance des comptes (soldes débit/crédit)
//! - [`journal_report`] — Journaux (Achats, Ventes, Banque, Caisse, OD)
//!
//! Tous les rapports retournent des structures `Serialize` en camelCase pour
//! consommation directe par les routes HTTP de `kesh-api`. Le rendu PDF/CSV
//! est délégué à Story 9-2.

pub mod balance_sheet;
pub mod errors;
pub mod income_statement;
pub mod journal_report;
pub mod period;
pub mod trial_balance;

pub use balance_sheet::{AccountBalance, BalanceSheet, generate as generate_balance_sheet};
pub use errors::ReportError;
pub use income_statement::{IncomeStatement, generate as generate_income_statement};
pub use journal_report::{
    JournalEntryLineRow, JournalEntryRow, JournalReport, JournalSection,
    generate as generate_journal_report,
};
pub use period::ReportPeriod;
pub use trial_balance::{TrialBalance, TrialBalanceRow, generate as generate_trial_balance};
