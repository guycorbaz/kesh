//! Entité `BankProfile` — profil de format CSV par banque (Story 8-2 FR53).
//!
//! Le `column_mapping` est stocké en JSON natif MariaDB (`JSON NOT NULL`
//! avec `CHECK JSON_VALID`). Côté Rust, il est sérialisé en `String` à
//! l'INSERT/UPDATE et désérialisé via [`BankProfile::parse_column_mapping`]
//! quand le caller a besoin de manipuler la struct typée
//! [`kesh_import::ColumnMapping`].
//!
//! Pas de FK `bank_imports.bank_profile_id` v0.1 (cf. spec 8-2 Limitation
//! L4). La trace du profil utilisé pour un import vit uniquement dans
//! `audit_log.details_json` avec snapshot `bank_profile_name` (Pass 2 M'3).

use chrono::NaiveDateTime;
use kesh_import::ColumnMapping;
use serde::{Deserialize, Serialize};

use crate::errors::DbError;

/// Profil CSV bancaire persisté en base.
///
/// `column_mapping_json` est la string JSON brute (telle que stockée
/// en colonne `JSON`). Utiliser [`BankProfile::parse_column_mapping`]
/// pour obtenir la struct typée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct BankProfile {
    pub id: i64,
    pub company_id: i64,
    pub bank_name: String,
    pub filename_pattern: Option<String>,
    /// JSON brut de la struct `ColumnMapping`. Désérialiser via
    /// [`Self::parse_column_mapping`].
    pub column_mapping_json: String,
    pub date_format: String,
    pub decimal_separator: String, // CHAR(1) MariaDB → String côté Rust
    pub field_separator: String,   // CHAR(1) MariaDB → String
    pub encoding: Option<String>,
    pub header_row_count: u8,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl BankProfile {
    /// Désérialise `column_mapping_json` en [`ColumnMapping`] typé.
    pub fn parse_column_mapping(&self) -> Result<ColumnMapping, DbError> {
        serde_json::from_str(&self.column_mapping_json).map_err(|e| {
            DbError::Invariant(format!(
                "bank_profile.column_mapping_json invalide (id={}) : {}",
                self.id, e
            ))
        })
    }

    /// Récupère `decimal_separator` comme `char` (la colonne MariaDB
    /// est `CHAR(1)` mais sqlx mysql la lit en `String`).
    ///
    /// Pass 1 review G2-BH-10 + G2-EH-6 : log défensif si la string
    /// stockée en DB est vide (ne devrait pas arriver via l'API qui
    /// rejette les strings vides, mais protection contre corruption
    /// ou migration manuelle).
    pub fn decimal_separator_char(&self) -> char {
        self.decimal_separator.chars().next().unwrap_or_else(|| {
            tracing::error!(
                "bank_profile id={} : decimal_separator vide en DB, fallback '.'",
                self.id
            );
            '.'
        })
    }

    /// Récupère `field_separator` comme `char` (idem decimal_separator).
    pub fn field_separator_char(&self) -> char {
        self.field_separator.chars().next().unwrap_or_else(|| {
            tracing::error!(
                "bank_profile id={} : field_separator vide en DB, fallback ','",
                self.id
            );
            ','
        })
    }
}

/// Données de création / remplacement (PUT full replacement) d'un profil.
///
/// Pas de `id`/`company_id`/`created_at`/`updated_at` (gérés par DB +
/// handler API). `column_mapping_json` est déjà sérialisé par le
/// handler (qui reçoit la struct typée du JSON request body).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewBankProfile {
    pub bank_name: String,
    pub filename_pattern: Option<String>,
    pub column_mapping_json: String,
    pub date_format: String,
    pub decimal_separator: String,
    pub field_separator: String,
    pub encoding: Option<String>,
    pub header_row_count: u8,
}
