//! Entités `PaymentBatch` et `PaymentBatchItem` (Story 12.3 — #191).
//!
//! Un lot de paiement regroupe N factures fournisseurs réglées par virement
//! (pain.001). Flux deux temps : `generated` (fichier produit, rien posté) →
//! `confirmed` (écritures de règlement postées) ; `cancelled` avant confirmation.
//! L'appartenance à un lot `generated` verrouille la facture (DC1, non-breaking —
//! pas de statut ajouté à `supplier_invoices`).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Lot de paiement persisté (entête).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatch {
    pub id: i64,
    pub company_id: i64,
    pub bank_account_id: i64,
    /// 'generated' | 'confirmed' | 'cancelled'.
    pub status: String,
    pub requested_execution_date: NaiveDate,
    pub total_amount: Decimal,
    pub msg_id: String,
    pub payment_info_id: String,
    pub confirmed_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne d'un lot : une facture fournisseur incluse dans le virement.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct PaymentBatchItem {
    pub id: i64,
    pub payment_batch_id: i64,
    pub supplier_invoice_id: i64,
    pub position: i32,
    pub end_to_end_id: String,
    pub amount: Decimal,
    pub created_at: NaiveDateTime,
}

/// Données de création d'un lot (le repo valide chaque facture, pattern batch).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewPaymentBatch {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub requested_execution_date: NaiveDate,
    pub supplier_invoice_ids: Vec<i64>,
}
