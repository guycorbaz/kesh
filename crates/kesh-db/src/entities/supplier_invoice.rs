//! Entités `SupplierInvoice` et `SupplierInvoiceLine` (Story 12.2 — #191).
//!
//! Une facture fournisseur (créancier) est ENREGISTRÉE (écriture d'achat
//! D charge / D impôt préalable 1171 / C créanciers 2000, single-step) puis
//! RÉGLÉE via un choix binaire (virement→compte bancaire source / compte
//! interne→compte libre du plan comptable). Entité DÉDIÉE, distincte des
//! factures de vente (`invoices`).
//!
//! `total_amount` est le TTC (Σ HT + Σ TVA) = la ligne C 2000 de l'écriture
//! d'achat → le règlement débite exactement ce montant et solde 2000 à 0.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Facture fournisseur persistée (entête).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoice {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    /// Numéro de la facture chez le fournisseur (texte libre, optionnel).
    pub supplier_invoice_number: Option<String>,
    /// 'open' (enregistrée, impayée) | 'paid' (réglée) | 'cancelled' (annulée).
    pub status: String,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    /// TTC (Σ HT + Σ TVA), montant dû au créancier.
    pub total_amount: Decimal,
    // Coordonnées de paiement (données de la facture, orthogonales au mode).
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    /// Projet analytique (Epic 19, Story 19-3) — document-level.
    pub project_id: Option<i64>,
    /// Écriture d'achat (toujours présente, posée à l'enregistrement).
    pub purchase_journal_entry_id: i64,
    // Règlement (renseigné au paiement uniquement).
    pub settlement_type: Option<String>,
    pub settlement_bank_account_id: Option<i64>,
    pub settlement_account_id: Option<i64>,
    pub settlement_journal_entry_id: Option<i64>,
    pub paid_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne de facture fournisseur persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct SupplierInvoiceLine {
    pub id: i64,
    pub supplier_invoice_id: i64,
    pub position: i32,
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    /// HT de la ligne (= quantity × unit_price).
    pub line_total: Decimal,
    /// Compte de charge (6xxx) de la ligne.
    pub expense_account_id: i64,
    pub created_at: NaiveDateTime,
}

/// Données de création d'une facture fournisseur (chemin A : saisie manuelle).
/// Le caller fournit les lignes (≠ avoir qui snapshot la facture d'origine).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSupplierInvoice {
    pub company_id: i64,
    pub contact_id: i64,
    pub supplier_invoice_number: Option<String>,
    pub invoice_date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub creditor_iban: Option<String>,
    pub creditor_qr_iban: Option<String>,
    pub payment_reference: Option<String>,
    pub expected_payment_amount: Option<Decimal>,
    /// Projet analytique (Epic 19, Story 19-3) — document-level, propagé sur les lignes
    /// de l'écriture d'achat. `None` = pas de projet.
    pub project_id: Option<i64>,
    pub lines: Vec<NewSupplierInvoiceLine>,
}

/// Ligne fournie à la création. `line_total` = `quantity × unit_price` (HT),
/// calculé côté repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewSupplierInvoiceLine {
    pub description: String,
    pub quantity: Decimal,
    pub unit_price: Decimal,
    pub vat_rate: Decimal,
    pub expense_account_id: i64,
}

/// Choix binaire de règlement (action « Payer »). PAS de table de modes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SettlementChoice {
    /// Virement : contrepartie = `bank_account.journal_account_id` du compte source.
    BankTransfer { bank_account_id: i64 },
    /// Compte interne : contrepartie = compte libre du plan comptable.
    InternalAccount { account_id: i64 },
}
