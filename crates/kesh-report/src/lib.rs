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

pub use balance_sheet::{generate as generate_balance_sheet, AccountBalance, BalanceSheet};
pub use errors::ReportError;
pub use income_statement::{generate as generate_income_statement, IncomeStatement};
pub use journal_report::{
    generate as generate_journal_report, JournalEntryLineRow, JournalEntryRow, JournalReport,
    JournalSection,
};
pub use period::ReportPeriod;
pub use trial_balance::{generate as generate_trial_balance, TrialBalance, TrialBalanceRow};
