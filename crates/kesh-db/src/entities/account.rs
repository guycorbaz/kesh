//! Entité `Account` : compte du plan comptable d'une company.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{
    encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo, Decode, Encode, MySql, Type,
};

/// Type de compte comptable.
///
/// Stocké en DB en PascalCase : `"Asset"`, `"Liability"`, `"Revenue"`, `"Expense"`.
/// CHECK BINARY en DB pour éviter les problèmes de collation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "Asset",
            Self::Liability => "Liability",
            Self::Revenue => "Revenue",
            Self::Expense => "Expense",
        }
    }
}

impl std::str::FromStr for AccountType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Asset" => Ok(Self::Asset),
            "Liability" => Ok(Self::Liability),
            "Revenue" => Ok(Self::Revenue),
            "Expense" => Ok(Self::Expense),
            other => Err(format!("AccountType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for AccountType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for AccountType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for AccountType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Compte comptable persisté en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: i64,
    pub company_id: i64,
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un compte.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewAccount {
    pub company_id: i64,
    pub number: String,
    pub name: String,
    pub account_type: AccountType,
    pub parent_id: Option<i64>,
}

/// Données de modification d'un compte.
/// Le numéro n'est PAS modifiable après création.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountUpdate {
    pub name: String,
    pub account_type: AccountType,
}
