//! Entité `BankImport` — entête d'un fichier bancaire importé (CAMT.053 v0.1, CSV Story 8-2).
//!
//! Story 8-1b : pendant la persistance, l'entête `bank_imports` est créée
//! atomiquement avec les `bank_transactions` filles via
//! [`crate::repositories::bank_imports::create_with_transactions`]. Le
//! réimport d'un même fichier n'est pas bloqué par la base — l'unicité
//! `(company_id, file_hash)` a été retirée par la Story 8-3 (index non unique
//! `idx_bank_imports_company_hash`) : la route le refuse en
//! `422 BANK_IMPORT_DUPLICATE_FILE` sans `confirmDuplicateFile=true`, et la
//! suppression de l'import (Story 27-1) lève ce refus.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Format source d'un import bancaire — projection finie pour la base.
///
/// Mappé en `VARCHAR(32)` MariaDB. Valeurs **MAJUSCULES** strictes,
/// alignées sur `kesh_core::bank_imports::SourceFormatTag::as_db_str()`.
/// La variante `Csv` (Story 8-2) couvre tous les profils CSV — la
/// distinction par banque vit dans `bank_profiles.id` tracé dans
/// `audit_log.details_json` (cf. spec 8-2 Limitation L4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BankImportSourceFormat {
    Camt053V04,
    Camt053V08,
    /// Story 8-2 T5.0.a — import CSV (multi-encodage + profil banque).
    Csv,
    /// Story v016-2a (#816) — relevé saisi à la main. Une valeur que la 0.16.x
    /// ne décode pas (`from_str` refuse l'inconnu) : d'où la migration
    /// `20261030000001_bank_imports_manual_source`, qui relève
    /// `kesh_version_min_required` à 0.17.0 (C-v016-2-2).
    Manual,
}

impl BankImportSourceFormat {
    /// Code stable utilisé pour la persistance (colonne `source_format`).
    /// Aligné avec `kesh_core::bank_imports::SourceFormatTag::as_db_str()`.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Camt053V04 => "CAMT053_V04",
            Self::Camt053V08 => "CAMT053_V08",
            Self::Csv => "CSV",
            Self::Manual => "MANUAL",
        }
    }
}

impl std::fmt::Display for BankImportSourceFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for BankImportSourceFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CAMT053_V04" => Ok(Self::Camt053V04),
            "CAMT053_V08" => Ok(Self::Camt053V08),
            "CSV" => Ok(Self::Csv),
            "MANUAL" => Ok(Self::Manual),
            other => Err(format!("BankImportSourceFormat inconnu : {other}")),
        }
    }
}

impl Type<MySql> for BankImportSourceFormat {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for BankImportSourceFormat {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for BankImportSourceFormat {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Entête d'import bancaire persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BankImport {
    pub id: i64,
    pub company_id: i64,
    pub bank_account_id: i64,
    pub filename: String,
    /// SHA-256 hex 64 chars.
    pub file_hash: String,
    pub source_format: BankImportSourceFormat,
    pub statement_id: Option<String>,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub opening_balance: Option<Decimal>,
    pub closing_balance: Option<Decimal>,
    pub transaction_count: i32,
    pub imported_at: NaiveDateTime,
    pub imported_by_user_id: i64,
}

/// Données de création d'un entête `bank_imports`.
#[derive(Debug, Clone)]
pub struct NewBankImport {
    pub company_id: i64,
    pub bank_account_id: i64,
    pub filename: String,
    pub file_hash: String,
    pub source_format: BankImportSourceFormat,
    pub statement_id: Option<String>,
    pub period_from: NaiveDate,
    pub period_to: NaiveDate,
    pub opening_balance: Option<Decimal>,
    pub closing_balance: Option<Decimal>,
    pub transaction_count: i32,
    pub imported_by_user_id: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn source_format_as_str_uppercase() {
        assert_eq!(BankImportSourceFormat::Camt053V04.as_str(), "CAMT053_V04");
        assert_eq!(BankImportSourceFormat::Camt053V08.as_str(), "CAMT053_V08");
        assert_eq!(BankImportSourceFormat::Csv.as_str(), "CSV");
        assert_eq!(BankImportSourceFormat::Manual.as_str(), "MANUAL");
    }

    /// Story v016-2a (AC-1) — aller-retour des quatre valeurs, `Display`
    /// compris, et alignement sur `SourceFormatTag::as_db_str()` de kesh-core.
    #[test]
    fn source_format_roundtrip() {
        use kesh_core::bank_imports::SourceFormatTag;
        for (fmt, tag) in [
            (
                BankImportSourceFormat::Camt053V04,
                SourceFormatTag::Camt053V04,
            ),
            (
                BankImportSourceFormat::Camt053V08,
                SourceFormatTag::Camt053V08,
            ),
            (BankImportSourceFormat::Csv, SourceFormatTag::Csv),
            (BankImportSourceFormat::Manual, SourceFormatTag::Manual),
        ] {
            let parsed = BankImportSourceFormat::from_str(fmt.as_str()).unwrap();
            assert_eq!(fmt, parsed);
            assert_eq!(fmt.to_string(), fmt.as_str());
            assert_eq!(fmt.as_str(), tag.as_db_str());
            assert_eq!(tag.to_string(), tag.as_db_str());
        }
    }

    /// Story v016-2a (AC-1) — `"manual"` en minuscules refusé.
    #[test]
    fn source_format_manual_lowercase_rejected() {
        BankImportSourceFormat::from_str("manual").expect_err("doit être MAJUSCULES");
    }

    #[test]
    fn source_format_unknown_rejected() {
        // Story 8-2 T5.0.a : "CSV" est désormais valide (variant Csv ajouté).
        // Test garde-fou : un slug inconnu reste rejeté.
        let err = BankImportSourceFormat::from_str("UNKNOWN_FORMAT_XX").unwrap_err();
        assert!(err.contains("UNKNOWN_FORMAT_XX"));
    }

    #[test]
    fn source_format_lowercase_rejected() {
        // Aligned with as_db_str() UPPERCASE — la spec d'origine commentait
        // des minuscules par erreur ; F8 validate Pass 1 8-1b a corrigé.
        BankImportSourceFormat::from_str("camt053_v04").expect_err("doit être MAJUSCULES");
    }
}
