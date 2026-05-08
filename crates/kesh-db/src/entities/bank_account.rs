//! Entité `BankAccount` : compte bancaire associé à une company.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Compte bancaire persisté en base.
///
/// Story 8-5a-zero — ajout du champ `journal_account_id` (Option<i64>)
/// qui lie le `bank_account` à un compte du plan comptable
/// (typiquement classe 1 — 1020 Caisse / 1030 Banque). Initialement
/// NULL pour les rows pré-migration : le user **doit** configurer
/// avant d'utiliser FR45 (8-5a-base) ou FR48 (8-5a-bis), via la route
/// `PATCH /api/v1/bank-accounts/{id}`.
///
/// Sérialisation `journalAccountId` (camelCase) cohérente avec la
/// convention `kesh-api`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BankAccount {
    pub id: i64,
    pub company_id: i64,
    pub bank_name: String,
    /// IBAN normalisé sans espaces.
    pub iban: String,
    /// QR-IBAN optionnel (plage QR-IID 30000-31999).
    pub qr_iban: Option<String>,
    pub is_primary: bool,
    /// Compte du plan comptable lié à ce bank_account (classe 1 typique).
    /// `None` = non configuré → FR45/FR48 retourneront 412 BANK_ACCOUNT_NOT_CONFIGURED
    /// (livré par 8-5a-base, pas par 8-5a-zero).
    pub journal_account_id: Option<i64>,
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
