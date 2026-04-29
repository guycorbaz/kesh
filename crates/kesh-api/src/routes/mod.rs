pub mod accounts;
pub mod auth;
pub mod companies;
pub mod company_invoice_settings;
pub mod contacts;
pub mod fiscal_years;
pub mod health;
pub mod i18n;
pub mod invoice_pdf;
pub mod invoices;
pub mod journal_entries;
pub mod limits;
pub mod onboarding;
pub mod products;
pub mod profile;
pub mod test_endpoints;
pub mod users;
pub mod vat;

use serde::Serialize;

/// Envelope standard pour toutes les réponses de liste paginée.
///
/// Format JSON :
/// ```json
/// { "items": [...], "total": 123, "offset": 50, "limit": 50 }
/// ```
///
/// Story 3.4 — réutilisable par toutes les routes list-like futures
/// (contacts, factures, imports). Extraction dans un module partagé
/// dédié si le pattern se répète — YAGNI pour v0.1.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T: Serialize> {
    pub items: Vec<T>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}
