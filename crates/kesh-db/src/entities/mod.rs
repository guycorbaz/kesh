//! Entités de données persistées.
//!
//! Chaque entité correspond à une table MariaDB. Les structs de création
//! (`New*`) et de mise à jour (`*Update`) ne contiennent que les champs
//! modifiables par le client — l'id, la version et les timestamps sont
//! gérés par la base.

pub mod account;
pub mod audit_log;
pub mod bank_account;
pub mod company;
pub mod contact;
pub mod fiscal_year;
pub mod invoice;
pub mod journal_entry;
pub mod onboarding;
pub mod product;
pub mod refresh_token;
pub mod user;

pub use account::{Account, AccountType, AccountUpdate, NewAccount};
pub use audit_log::{AuditLogEntry, NewAuditLogEntry};
pub use bank_account::{BankAccount, NewBankAccount};
pub use company::{Company, CompanyUpdate, Language, NewCompany, OrgType};
pub use contact::{Contact, ContactType, ContactUpdate, NewContact};
pub use fiscal_year::{FiscalYear, FiscalYearStatus, NewFiscalYear};
pub use invoice::{Invoice, InvoiceLine, InvoiceUpdate, NewInvoice, NewInvoiceLine};
pub use journal_entry::{
    Journal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
    NewJournalEntryLine,
};
pub use onboarding::{OnboardingState, UiMode};
pub use product::{NewProduct, Product, ProductUpdate};
pub use refresh_token::{NewRefreshToken, RefreshToken};
pub use user::{NewUser, Role, User, UserUpdate};
