//! Entité `BankTransaction` — transaction bancaire individuelle issue d'un import.
//!
//! Story 8-1b : insertion atomique en bulk via
//! [`crate::repositories::bank_imports::create_with_transactions`]. Le
//! statut `pending` / `reconciled` sera piloté par Story 8-4
//! (réconciliation). `matched_entry_id` est posé maintenant pour éviter
//! une `ALTER TABLE` ultérieure (FK `ON DELETE SET NULL` permet la
//! suppression d'une écriture comptable sans casser la transaction).

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Statut d'une transaction bancaire vis-à-vis de la réconciliation
/// (Story 8-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BankTransactionStatus {
    /// État initial : transaction extraite, pas encore réconciliée.
    Pending,
    /// Transaction réconciliée avec une écriture comptable
    /// (`matched_entry_id` est `Some`).
    Reconciled,
}

impl BankTransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Reconciled => "reconciled",
        }
    }
}

impl std::fmt::Display for BankTransactionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BankTransactionStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "reconciled" => Ok(Self::Reconciled),
            other => Err(format!("BankTransactionStatus inconnu : {other}")),
        }
    }
}

impl Type<MySql> for BankTransactionStatus {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for BankTransactionStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for BankTransactionStatus {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Transaction bancaire persistée en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BankTransaction {
    pub id: i64,
    pub company_id: i64,
    pub import_id: i64,
    pub bank_account_id: i64,
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    /// Montant signé : positif = crédit titulaire, négatif = débit titulaire.
    pub amount: Decimal,
    /// Code devise ISO 4217 (3 chars).
    pub currency: String,
    pub reference: Option<String>,
    /// Détails libres (toujours présent même vide).
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
    pub status: BankTransactionStatus,
    pub matched_entry_id: Option<i64>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'une transaction bancaire (bulk INSERT).
#[derive(Debug, Clone)]
pub struct NewBankTransaction {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Decimal,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn status_as_str_lowercase() {
        assert_eq!(BankTransactionStatus::Pending.as_str(), "pending");
        assert_eq!(BankTransactionStatus::Reconciled.as_str(), "reconciled");
    }

    #[test]
    fn status_roundtrip() {
        for s in [
            BankTransactionStatus::Pending,
            BankTransactionStatus::Reconciled,
        ] {
            let parsed = BankTransactionStatus::from_str(s.as_str()).unwrap();
            assert_eq!(s, parsed);
        }
    }

    #[test]
    fn status_unknown_rejected() {
        let err = BankTransactionStatus::from_str("matched").unwrap_err();
        assert!(err.contains("matched"));
    }
}
