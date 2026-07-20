//! Repositories : fonctions libres par entité pour les opérations CRUD.
//!
//! Un fichier par entité. Pattern : `create`, `find_by_id`, `update`, `list`
//! comme API standard. Les méthodes spécifiques (`find_by_username`,
//! `list_by_company`, `close`) s'ajoutent là où c'est nécessaire.

pub mod accounts;
pub mod api_keys;
pub mod audit_log;
pub mod bank_accounts;
pub mod bank_imports;
pub mod bank_profiles;
pub mod bank_transactions;
pub mod companies;
pub mod company_dunning_settings;
pub mod company_invoice_settings;
pub mod contact_persons;
pub mod contacts;
pub mod credit_note_number_sequences;
pub mod credit_notes;
pub mod dunning_eligibility;
pub mod dunning_levels;
pub mod email_templates;
pub mod fiscal_years;
pub mod imported_supplier_invoices;
pub mod invoice_number_sequences;
pub mod invoice_reminders;
pub mod invoices;
pub mod journal_entries;
pub mod onboarding;
pub mod password_reset_tokens;
pub mod payment_batches;
pub mod products;
pub mod projects;
pub mod reconciliation;
pub mod reconciliation_rules;
pub mod refresh_tokens;
pub mod supplier_invoices;
pub mod users;
pub mod vat_rates;

/// Limite haute pour les appels `list()` : évite les OOM par `fetch_all`
/// sur de gros résultats. S'applique à toutes les entités.
pub const MAX_LIST_LIMIT: i64 = 1000;
