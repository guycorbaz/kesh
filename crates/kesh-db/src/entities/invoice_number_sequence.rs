//! Entité `InvoiceNumberSequence` (Story 5.2 — FR33).
//!
//! Compteur séquentiel par `(company_id, fiscal_year_id)`, incrémenté
//! atomiquement à la validation d'une facture via `SELECT FOR UPDATE`
//! dans la transaction de validation. Rollback = compteur intact (pas
//! de trou — exigence comptable suisse).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Compteur persisté. Une row par (company_id, fiscal_year_id).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceNumberSequence {
    pub id: i64,
    pub company_id: i64,
    pub fiscal_year_id: i64,
    pub next_number: i64,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
