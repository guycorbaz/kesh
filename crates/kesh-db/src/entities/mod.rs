//! Entités de données persistées.
//!
//! Chaque entité correspond à une table MariaDB. Les structs de création
//! (`New*`) et de mise à jour (`*Update`) ne contiennent que les champs
//! modifiables par le client — l'id, la version et les timestamps sont
//! gérés par la base.

pub mod account;
pub mod address;
pub mod api_key;
pub mod audit_log;
pub mod bank_account;
pub mod bank_import;
pub mod bank_profile;
pub mod bank_transaction;
pub mod company;
pub mod company_invoice_settings;
pub mod contact;
pub mod contact_person;
pub mod credit_note;
pub mod fiscal_year;
pub mod imported_supplier_invoice;
pub mod invoice;
pub mod invoice_number_sequence;
pub mod journal_entry;
pub mod onboarding;
pub mod password_reset_token;
pub mod payment_batch;
pub mod product;
pub mod project;
pub mod reconciliation_rule;
pub mod refresh_token;
pub mod supplier_invoice;
pub mod user;
pub mod vat_rate;

pub use account::{Account, AccountType, AccountUpdate, NewAccount};
pub use api_key::{ApiKey, ApiKeyScope, NewApiKey};
pub use audit_log::{AUDIT_ENTITY_ID_NONE, ActorType, AuditLogEntry, NewAuditLogEntry};
pub use bank_account::{BankAccount, NewBankAccount};
pub use bank_import::{BankImport, BankImportSourceFormat, NewBankImport};
pub use bank_profile::{BankProfile, NewBankProfile};
pub use bank_transaction::{BankTransaction, BankTransactionStatus, NewBankTransaction};
pub use company::{Company, CompanyUpdate, Language, NewCompany, OrgType};
pub use company_invoice_settings::{CompanyInvoiceSettings, CompanyInvoiceSettingsUpdate};
pub use contact::{Contact, ContactType, ContactUpdate, NewContact};
pub use credit_note::{CreditNote, CreditNoteLine, NewCreditNote};
pub use fiscal_year::{FiscalYear, FiscalYearStatus, NewFiscalYear};
pub use imported_supplier_invoice::{
    DocumentMeta, ImportedSupplierInvoice, NewImportedSupplierInvoice,
};
pub use invoice::{Invoice, InvoiceLine, InvoiceUpdate, NewInvoice, NewInvoiceLine};
pub use invoice_number_sequence::InvoiceNumberSequence;
pub use journal_entry::{
    Journal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
    NewJournalEntryLine,
};
pub use onboarding::{OnboardingState, UiMode};
pub use password_reset_token::PasswordResetToken;
pub use payment_batch::{NewPaymentBatch, PaymentBatch, PaymentBatchItem};
pub use product::{NewProduct, Product, ProductUpdate};
pub use project::{NewProject, Project, UpdateProject};
pub use reconciliation_rule::{
    NewReconciliationRule, ReconciliationMatchType, ReconciliationRule, UpdateReconciliationRule,
};
pub use refresh_token::{NewRefreshToken, RefreshToken};
pub use supplier_invoice::{
    NewSupplierInvoice, NewSupplierInvoiceLine, SettlementChoice, SupplierInvoice,
    SupplierInvoiceLine,
};
pub use user::{NewUser, Role, User, UserUpdate};
pub use vat_rate::{NewVatRate, UpdateVatRate, VatRate, vat_category};
