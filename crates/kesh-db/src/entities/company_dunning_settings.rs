//! Réglages de rappel par company (singleton) — période de grâce + discriminant seed.
//!
//! Calqué sur `company_invoice_settings` : PK = `company_id` (unicité par company,
//! rend `INSERT IGNORE` idempotent), verrou optimiste `version`. `seeded_at` discrimine
//! « jamais seedé » (NULL → seed lazy des 3 niveaux par défaut) de « vidé volontairement »
//! (NON-NULL avec `dunning_levels` vide → dunning désactivé, PAS de résurrection). D7.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Réglages de rappel persistés (`company_dunning_settings`).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct CompanyDunningSettings {
    pub company_id: i64,
    /// Jours de grâce après l'échéance avant le 1er rappel éligible.
    pub grace_period_days: i32,
    /// Horodatage du seed initial des défauts. NULL = jamais seedé (le seed lazy
    /// posera les 3 niveaux) ; NON-NULL = seedé une fois (pas de résurrection).
    pub seeded_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Payload de remplacement (verrou optimiste `version` géré séparément par le repo ;
/// `seeded_at` jamais piloté par le client — posé uniquement par le seed).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompanyDunningSettingsUpdate {
    pub grace_period_days: i32,
}
