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

/// Genre d'un cours de change d'exercice (Story 37-1) : cours **journalier**
/// ou cours **mensuel**. Colonne `ENUM('day','month')`, écrite en minuscules
/// au HTTP comme en base.
///
/// Story 37-2 : le type vit dans `kesh-core` (les fonctions pures des cours en
/// ont besoin, et `kesh-core` ne dépend pas de `kesh-db`) ; il est réexporté
/// ici. L'orphan rule interdit d'y implémenter les traits `sqlx` : les colonnes
/// se lisent en texte, converties par `TryFrom<String>`
/// (`#[sqlx(try_from = "String")]`), et s'écrivent par `RateKind::as_str`.
pub use kesh_core::exchange_rates::RateKind;

/// Exercice comptable persisté en base.
///
/// **Mode et genres de cours (Story 37-1).** `multi_currency` dit si
/// l'exercice admet des lignes sur des comptes en devise : il prend le réglage
/// de la société à sa création, passe à `true` à l'activation du multi-devises
/// s'il est ouvert, et ne change plus ensuite (un exercice clos à
/// l'activation reste mono-devise, même rouvert).
///
/// Sens des genres — le cours d'ouverture est le cours de clôture de
/// l'exercice précédent, pour que les valeurs CHF concordent d'un exercice à
/// l'autre :
/// - ouverture [`RateKind::Day`] : cours journalier de la **veille** de
///   `start_date` (jour de clôture de l'exercice précédent) ;
/// - ouverture [`RateKind::Month`] : cours mensuel du **mois de la veille** de
///   `start_date` (dernier mois de l'exercice précédent) ;
/// - clôture [`RateKind::Day`] : cours journalier de `end_date` ;
/// - clôture [`RateKind::Month`] : cours mensuel du mois de `end_date`.
///
/// À la création, l'ouverture reprend le genre de clôture de l'exercice de la
/// société qui finit la veille de `start_date` (`day` sinon), et la clôture
/// prend `day`.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FiscalYear {
    pub id: i64,
    pub company_id: i64,
    /// Nom lisible, unique par company (ex: "Exercice 2026").
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub status: FiscalYearStatus,
    /// Exercice multi-devises (voir la doc du type).
    pub multi_currency: bool,
    /// Genre du cours d'ouverture (voir la doc du type).
    #[sqlx(try_from = "String")]
    pub opening_rate_kind: RateKind,
    /// Genre du cours de clôture (voir la doc du type).
    #[sqlx(try_from = "String")]
    pub closing_rate_kind: RateKind,
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
