//! Entité `Contact` : contact du carnet d'adresses unifié (clients + fournisseurs).
//!
//! Story 4.1 : FR25 (carnet unifié), FR26 (flags client/fournisseur),
//! FR27 (validation IDE CHE côté API), schéma pour FR28 (default_payment_terms).

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type de contact : personne physique ou entreprise (raison sociale).
///
/// Stocké en DB en PascalCase : `"Personne"`, `"Entreprise"`.
/// CHECK BINARY en DB pour éviter les problèmes de collation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContactType {
    Personne,
    Entreprise,
}

impl ContactType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Personne => "Personne",
            Self::Entreprise => "Entreprise",
        }
    }
}

impl std::str::FromStr for ContactType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Personne" => Ok(Self::Personne),
            "Entreprise" => Ok(Self::Entreprise),
            other => Err(format!("ContactType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for ContactType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for ContactType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for ContactType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Contact persisté en base.
///
/// Le champ `ide_number` stocke la forme **normalisée** sans séparateurs
/// (ex: `"CHE109322551"`, 12 chars). La forme d'affichage
/// `"CHE-109.322.551"` est reconstruite côté frontend.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub id: i64,
    pub company_id: i64,
    pub contact_type: ContactType,
    pub name: String,
    pub is_client: bool,
    pub is_supplier: bool,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    pub default_payment_terms: Option<String>,
    pub active: bool,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'un contact. Valeurs déjà trimées et validées par le caller.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewContact {
    pub company_id: i64,
    pub contact_type: ContactType,
    pub name: String,
    pub is_client: bool,
    pub is_supplier: bool,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    pub default_payment_terms: Option<String>,
}

/// Données de modification d'un contact (tous les champs métier).
///
/// **Note importante** : `version` n'est PAS dans cette struct — elle
/// est passée comme paramètre séparé à `contacts::update(...)` (pattern
/// identique à `accounts::update`).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactUpdate {
    pub contact_type: ContactType,
    pub name: String,
    pub is_client: bool,
    pub is_supplier: bool,
    pub address: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    pub default_payment_terms: Option<String>,
}
