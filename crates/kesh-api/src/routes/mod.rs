pub mod accounts;
pub mod auth;
pub mod companies;
pub mod contacts;
pub mod health;
pub mod i18n;
pub mod journal_entries;
pub mod onboarding;
pub mod profile;
pub mod users;

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
