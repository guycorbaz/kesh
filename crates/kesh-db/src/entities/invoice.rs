//! Entités `Invoice` et `InvoiceLine` (Story 5.1 — FR31, FR32).
//!
//! Les lignes snapshotent `description`, `unit_price`, `vat_rate` au moment
//! de la création : modifier un produit catalogue ne doit PAS altérer une
//! facture existante. Le catalogue n'est qu'un accélérateur de saisie.
//!
//! `total_amount` est recalculé et persisté par le repository à chaque
//! mutation (source de vérité = lignes).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Facture persistée (entête).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Invoice {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne de facture persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceLine {
    pub id: i64,
    pub invoice_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub line_total: Decimal,
    pub created_at: NaiveDateTime,
}

/// Données de création d'une ligne (sans `position` ni `line_total` —
/// calculés par le repository).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoiceLine {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

/// Données de création d'une facture. Le caller a déjà validé/normalisé.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewInvoice {
    pub company_id: i64,
    pub contact_id: i64,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub lines: Vec<NewInvoiceLine>,
}

/// Données de modification d'une facture brouillon.
///
/// `version` est passée séparément au repository (pattern identique à
/// `products::update`). Les lignes remplacent entièrement les anciennes
/// (replace-all — voir Dev Notes).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceUpdate {
    pub contact_id: i64,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub lines: Vec<NewInvoiceLine>,
}
