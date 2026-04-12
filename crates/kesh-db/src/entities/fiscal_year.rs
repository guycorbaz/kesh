//! Entité `FiscalYear` : exercice comptable d'une entreprise.
//!
//! Conformément au Code des obligations suisse (art. 957-964), les exercices
//! ne sont **jamais** supprimés. La table est protégée par `ON DELETE RESTRICT`
//! sur la FK vers `companies`, et aucune méthode `delete` n'est exposée par
//! le repository.

use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Statut d'un exercice comptable.
///
/// Transition unique autorisée : `Open` → `Closed`. Une fois clos, les
/// écritures de l'exercice deviennent immutables (enforcement dans
/// `kesh-core/accounting` story 3.x, puis `kesh-api` story 12.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum FiscalYearStatus {
    /// Exercice ouvert : écritures modifiables
    Open,
    /// Exercice clos : écritures immutables
    Closed,
}

impl FiscalYearStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::Closed => "Closed",
        }
    }
}

impl std::str::FromStr for FiscalYearStatus {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Open" => Ok(Self::Open),
            "Closed" => Ok(Self::Closed),
            other => Err(format!("FiscalYearStatus inconnu : {other}")),
        }
    }
}

impl Type<MySql> for FiscalYearStatus {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for FiscalYearStatus {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for FiscalYearStatus {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Exercice comptable persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FiscalYear {
    pub id: i64,
    pub company_id: i64,
    /// Nom lisible, unique par company (ex: "Exercice 2026").
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: FiscalYearStatus,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un exercice comptable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFiscalYear {
    pub company_id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}
