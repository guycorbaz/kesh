//! Entité `Company` : données de l'entreprise/organisation utilisant Kesh.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type d'organisation.
///
/// ASCII-only par design : `"Independant"` sans accent pour éviter les
/// problèmes de collation MariaDB. Ne PAS "corriger" en `"Indépendant"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum OrgType {
    /// Indépendant (travailleur indépendant, freelance)
    Independant,
    /// Association à but non lucratif
    Association,
    /// Petite et moyenne entreprise (SA, Sàrl, etc.)
    Pme,
}

impl OrgType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Independant => "Independant",
            Self::Association => "Association",
            Self::Pme => "Pme",
        }
    }
}

impl std::str::FromStr for OrgType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Independant" => Ok(Self::Independant),
            "Association" => Ok(Self::Association),
            "Pme" => Ok(Self::Pme),
            other => Err(format!("OrgType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for OrgType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for OrgType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for OrgType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Langue supportée par Kesh (interface et libellés comptables).
///
/// Stocké en DB en majuscules : `"FR"/"DE"/"IT"/"EN"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Language {
    /// Français
    Fr,
    /// Allemand (Deutsch)
    De,
    /// Italien
    It,
    /// Anglais
    En,
}

impl Language {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Fr => "FR",
            Self::De => "DE",
            Self::It => "IT",
            Self::En => "EN",
        }
    }
}

impl std::str::FromStr for Language {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "FR" => Ok(Self::Fr),
            "DE" => Ok(Self::De),
            "IT" => Ok(Self::It),
            "EN" => Ok(Self::En),
            other => Err(format!("Language inconnue : {other}")),
        }
    }
}

impl Type<MySql> for Language {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for Language {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for Language {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Entreprise/organisation persistée en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Company {
    pub id: i64,
    pub name: String,
    pub address: String,
    /// Numéro IDE suisse (CHExxxxxxxxx, normalisé sans séparateurs).
    /// Optionnel — certaines organisations n'ont pas d'IDE.
    ///
    /// TODO(story future) : migrer vers `Option<kesh_core::types::CheNumber>`
    /// pour rendre l'état invalide impossible à représenter. Pour le MVP,
    /// la validation est assurée par (1) la regex CHECK en DB et (2) la
    /// validation `CheNumber` côté `kesh-api` avant l'appel au repository.
    pub ide_number: Option<String>,
    pub org_type: OrgType,
    /// Langue des libellés comptables (noms de comptes, etc.).
    pub accounting_language: Language,
    /// Langue de l'interface utilisateur.
    pub instance_language: Language,
    /// Version pour optimistic locking (incrémentée à chaque update).
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// Données de création d'une company. Pas d'id, version ni timestamps :
/// gérés par la base.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewCompany {
    pub name: String,
    pub address: String,
    pub ide_number: Option<String>,
    pub org_type: OrgType,
    pub accounting_language: Language,
    pub instance_language: Language,
}

/// Données de mise à jour d'une company.
///
/// Sémantique de **remplacement complet** : tous les champs modifiables sont
/// requis. Pas de patch partiel dans cette story — si un besoin émerge, une
/// story future introduira un struct `CompanyPatch` séparé.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyUpdate {
    pub name: String,
    pub address: String,
    pub ide_number: Option<String>,
    pub org_type: OrgType,
    pub accounting_language: Language,
    pub instance_language: Language,
}
