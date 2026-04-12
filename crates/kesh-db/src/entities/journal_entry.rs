//! Entité `JournalEntry` : écriture comptable en partie double.
//!
//! Une écriture comporte un en-tête (`JournalEntry`) et un ensemble de
//! lignes (`JournalEntryLine`), toujours manipulés ensemble dans une
//! transaction unique pour garantir l'atomicité.
//!
//! # Enum `Journal` — deux versions miroirs
//!
//! Cet enum est défini **deux fois** dans le workspace :
//!
//! - `kesh_core::accounting::Journal` — version pure, sans dépendance
//!   SQLx (logique métier et validation).
//! - `kesh_db::entities::journal_entry::Journal` (ici) — version avec
//!   implémentations `sqlx::Type`/`Encode`/`Decode` pour la persistance.
//!
//! Les conversions bidirectionnelles `From<kesh_core::accounting::Journal>`
//! et inverse sont définies dans ce module. Cette duplication volontaire
//! respecte l'orphan rule Rust et la règle ARCH-1 (kesh-core zéro I/O).
//! Pattern identique à `OrgType` et `AccountType`.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Journal comptable — version persistée en base.
///
/// Cet enum est le miroir de [`kesh_core::accounting::Journal`] avec
/// l'ajout des traits SQLx. Toute modification (ajout/suppression de
/// variant) doit être synchronisée avec kesh-core ET la contrainte DB
/// `CHECK BINARY journal IN (...)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Journal {
    /// Achats (facturation fournisseurs).
    Achats,
    /// Ventes (facturation clients).
    Ventes,
    /// Banque (mouvements bancaires).
    Banque,
    /// Caisse (espèces).
    Caisse,
    /// Opérations diverses (écritures de régularisation, clôture, etc.).
    OD,
}

impl Journal {
    /// Retourne la représentation textuelle stockée en base (PascalCase).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Achats => "Achats",
            Self::Ventes => "Ventes",
            Self::Banque => "Banque",
            Self::Caisse => "Caisse",
            Self::OD => "OD",
        }
    }
}

impl std::str::FromStr for Journal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Achats" => Ok(Self::Achats),
            "Ventes" => Ok(Self::Ventes),
            "Banque" => Ok(Self::Banque),
            "Caisse" => Ok(Self::Caisse),
            "OD" => Ok(Self::OD),
            other => Err(format!("Journal inconnu : {other}")),
        }
    }
}

impl Type<MySql> for Journal {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for Journal {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for Journal {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Conversion `kesh_core::Journal` → `kesh_db::Journal`.
impl From<kesh_core::accounting::Journal> for Journal {
    fn from(j: kesh_core::accounting::Journal) -> Self {
        match j {
            kesh_core::accounting::Journal::Achats => Self::Achats,
            kesh_core::accounting::Journal::Ventes => Self::Ventes,
            kesh_core::accounting::Journal::Banque => Self::Banque,
            kesh_core::accounting::Journal::Caisse => Self::Caisse,
            kesh_core::accounting::Journal::OD => Self::OD,
        }
    }
}

/// Conversion `kesh_db::Journal` → `kesh_core::Journal`.
impl From<Journal> for kesh_core::accounting::Journal {
    fn from(j: Journal) -> Self {
        match j {
            Journal::Achats => Self::Achats,
            Journal::Ventes => Self::Ventes,
            Journal::Banque => Self::Banque,
            Journal::Caisse => Self::Caisse,
            Journal::OD => Self::OD,
        }
    }
}

/// En-tête d'écriture comptable persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntry {
    pub id: i64,
    pub company_id: i64,
    pub fiscal_year_id: i64,
    pub entry_number: i64,
    pub entry_date: NaiveDate,
    pub journal: Journal,
    pub description: String,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Ligne d'écriture comptable persistée.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryLine {
    pub id: i64,
    pub entry_id: i64,
    pub account_id: i64,
    pub line_order: i32,
    pub debit: Decimal,
    pub credit: Decimal,
}

/// En-tête + lignes, retourné par le repository pour les lectures.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JournalEntryWithLines {
    pub entry: JournalEntry,
    pub lines: Vec<JournalEntryLine>,
}

/// Données de création d'une écriture (lignes incluses).
///
/// Le `fiscal_year_id` et le `entry_number` sont calculés par le
/// repository — le caller ne les fournit pas.
#[derive(Debug, Clone)]
pub struct NewJournalEntry {
    pub company_id: i64,
    pub entry_date: NaiveDate,
    pub journal: Journal,
    pub description: String,
    pub lines: Vec<NewJournalEntryLine>,
}

/// Données de création d'une ligne d'écriture.
#[derive(Debug, Clone)]
pub struct NewJournalEntryLine {
    pub account_id: i64,
    pub debit: Decimal,
    pub credit: Decimal,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_conversion_roundtrip() {
        for core_variant in [
            kesh_core::accounting::Journal::Achats,
            kesh_core::accounting::Journal::Ventes,
            kesh_core::accounting::Journal::Banque,
            kesh_core::accounting::Journal::Caisse,
            kesh_core::accounting::Journal::OD,
        ] {
            let db_variant: Journal = core_variant.into();
            let back: kesh_core::accounting::Journal = db_variant.into();
            assert_eq!(core_variant, back);
        }
    }

    #[test]
    fn journal_as_str_is_consistent_between_enums() {
        // Garde-fou anti-dérive : si un variant est ajouté d'un côté
        // sans l'autre, les tests de roundtrip casseront, mais cette
        // assertion vérifie aussi que les représentations string sont
        // identiques (critique pour CHECK BINARY DB).
        assert_eq!(
            Journal::Achats.as_str(),
            kesh_core::accounting::Journal::Achats.as_str()
        );
        assert_eq!(
            Journal::Ventes.as_str(),
            kesh_core::accounting::Journal::Ventes.as_str()
        );
        assert_eq!(
            Journal::Banque.as_str(),
            kesh_core::accounting::Journal::Banque.as_str()
        );
        assert_eq!(
            Journal::Caisse.as_str(),
            kesh_core::accounting::Journal::Caisse.as_str()
        );
        assert_eq!(
            Journal::OD.as_str(),
            kesh_core::accounting::Journal::OD.as_str()
        );
    }

    #[test]
    fn journal_from_str_accepts_all_variants() {
        use std::str::FromStr;
        assert_eq!(Journal::from_str("Achats").unwrap(), Journal::Achats);
        assert_eq!(Journal::from_str("Ventes").unwrap(), Journal::Ventes);
        assert_eq!(Journal::from_str("Banque").unwrap(), Journal::Banque);
        assert_eq!(Journal::from_str("Caisse").unwrap(), Journal::Caisse);
        assert_eq!(Journal::from_str("OD").unwrap(), Journal::OD);
    }

    #[test]
    fn journal_from_str_rejects_unknown() {
        use std::str::FromStr;
        assert!(Journal::from_str("Inconnu").is_err());
    }
}
