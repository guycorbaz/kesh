//! Entité `Product` : élément du catalogue produits/services (Story 4.2).
//!
//! FR29 : stockage du catalogue utilisé pour pré-remplir les lignes de
//! factures (FR30, câblé Story 5.1).
//!
//! - `unit_price` en `DECIMAL(19,4)` (cohérent journal_entry_lines.debit/credit)
//!   → `rust_decimal::Decimal` côté Rust, sérialisé en string via feature `serde-str`.
//! - `vat_rate` en `DECIMAL(5,2)` : pourcentage direct (ex: `8.10` pour 8.1 %).

use chrono::NaiveDateTime;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Produit persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: i64,
    pub company_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un produit. Valeurs déjà trimées et validées par le caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewProduct {
    pub company_id: i64,
    pub name: String,
    pub description: Option<String>,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}

/// Données de modification d'un produit.
///
/// **Note** : `version` n'est pas dans cette struct — elle est passée
/// comme paramètre séparé à `products::update(...)` (pattern identique à
/// `contacts::update`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProductUpdate {
    pub name: String,
    pub description: Option<String>,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
}
