//! Entité `BankAccount` : compte bancaire associé à une company.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Compte bancaire persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BankAccount {
    pub id: i64,
    pub company_id: i64,
    pub bank_name: String,
    /// IBAN normalisé sans espaces.
    pub iban: String,
    /// QR-IBAN optionnel (plage QR-IID 30000-31999).
    pub qr_iban: Option<String>,
    pub is_primary: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un compte bancaire.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBankAccount {
    pub company_id: i64,
    pub bank_name: String,
    pub iban: String,
    pub qr_iban: Option<String>,
    pub is_primary: bool,
}
