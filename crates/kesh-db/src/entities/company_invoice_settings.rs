//! Entité `CompanyInvoiceSettings` (Story 5.2 — FR35).
//!
//! Relation 1-1 avec `companies` (PK = `company_id`). Row créée à la volée
//! (lazy) via `INSERT IGNORE` au premier accès, avec les DEFAULT définis
//! dans le CREATE TABLE.
//!
//! `default_receivable_account_id` et `default_revenue_account_id` NULL à
//! l'install : forcent l'Admin à les configurer avant la première
//! validation de facture. Le handler `validate` refuse (400
//! `CONFIGURATION_REQUIRED`) si l'un des deux est NULL.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::Journal;

/// Config facturation d'une company (Story 5.2).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInvoiceSettings {
    pub company_id: i64,
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_sales_journal: Journal,
    pub journal_entry_description_template: String,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de mise à jour (PUT /company/invoice-settings). Tous les
/// champs sont requis (remplacement intégral). `version` est géré
/// séparément par le repository (verrou optimiste, pattern contacts/products).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyInvoiceSettingsUpdate {
    pub invoice_number_format: String,
    pub default_receivable_account_id: Option<i64>,
    pub default_revenue_account_id: Option<i64>,
    pub default_sales_journal: Journal,
    pub journal_entry_description_template: String,
}
