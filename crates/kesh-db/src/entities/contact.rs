//! Entité `Contact` : contact du carnet d'adresses unifié (clients + fournisseurs).
//!
//! Story 4.1 : FR25 (carnet unifié), FR26 (flags client/fournisseur),
//! FR27 (validation IDE CHE côté API), schéma pour FR28 (default_payment_terms).

use crate::entities::Language;
use crate::entities::address::StructuredAddress;
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

impl Contact {
    /// Reconstruit l'adresse structurée (#213) depuis les colonnes plates.
    /// `None` si aucun composant significatif n'est renseigné.
    pub fn structured_address(&self) -> Option<StructuredAddress> {
        let sa = StructuredAddress {
            street: self.address_street.clone().unwrap_or_default(),
            building: self.address_building.clone().unwrap_or_default(),
            postal_code: self.address_postal_code.clone().unwrap_or_default(),
            city: self.address_city.clone().unwrap_or_default(),
            country: self.address_country.clone().unwrap_or_else(|| "CH".into()),
        };
        if sa.is_empty() { None } else { Some(sa) }
    }
}

/// Recompose la chaîne d'affichage `address` (colonne dérivée #213) depuis les
/// composants structurés optionnels d'un contact. `None` si tous vides.
pub fn derive_contact_address_display(
    street: Option<&str>,
    building: Option<&str>,
    postal: Option<&str>,
    city: Option<&str>,
) -> Option<String> {
    let sa = StructuredAddress {
        street: street.unwrap_or_default().to_string(),
        building: building.unwrap_or_default().to_string(),
        postal_code: postal.unwrap_or_default().to_string(),
        city: city.unwrap_or_default().to_string(),
        country: String::new(),
    };
    if sa.is_empty() {
        None
    } else {
        Some(sa.combined())
    }
}

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

/// Civilité du contact (Story 20-3b1, décision #12 epic-20).
///
/// Sert à résoudre la variable `{salutation}` des templates d'e-mail
/// (genre × langue × type de contact). Stockée en DB en PascalCase :
/// `"Monsieur"`, `"Madame"`, `"Neutre"` (CHECK en DB). `Neutre` est le
/// défaut — formule neutre (« Madame, Monsieur ») à l'envoi.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Salutation {
    Monsieur,
    Madame,
    #[default]
    Neutre,
}

impl Salutation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Monsieur => "Monsieur",
            Self::Madame => "Madame",
            Self::Neutre => "Neutre",
        }
    }
}

impl std::str::FromStr for Salutation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Monsieur" => Ok(Self::Monsieur),
            "Madame" => Ok(Self::Madame),
            "Neutre" => Ok(Self::Neutre),
            other => Err(format!("Salutation inconnue : {other}")),
        }
    }
}

impl Type<MySql> for Salutation {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for Salutation {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for Salutation {
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
    /// Prénom / nom (#213) — renseignés uniquement pour un contact `Personne`.
    /// `name` reste l'affichage canonique recomposé (« Prénom Nom »).
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_client: bool,
    pub is_supplier: bool,
    /// Chaîne d'affichage libre **dérivée** des champs structurés (#213).
    pub address: Option<String>,
    /// Adresse structurée (débiteur QR-bill type S / pain.001, #213). Optionnelle.
    pub address_street: Option<String>,
    pub address_building: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    /// Numéro de client attribué par l'émetteur (Story 16-3b, #151). Saisi,
    /// jamais auto-généré (D1). Unique par société **entre contacts actifs**
    /// (un contact archivé libère son numéro, cf. migration 20260810000001).
    pub client_number: Option<String>,
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). `None` = non renseigné (le texte
    /// libre `default_payment_terms` reste la seule source, lecture seule).
    /// Renseigné → prime sur le texte libre (libellé auto-généré).
    pub default_payment_terms_days: Option<i32>,
    /// Langue de correspondance (Story 20-3b1). `None` = hérite de
    /// `companies.instance_language` (résolution à l'envoi).
    pub language: Option<Language>,
    /// Civilité pour `{salutation}` (Story 20-3b1). Défaut `Neutre`.
    pub salutation: Salutation,
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
    /// Prénom / nom (#213) — renseignés uniquement pour un contact `Personne`.
    /// `name` reste l'affichage canonique recomposé (« Prénom Nom »).
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_client: bool,
    pub is_supplier: bool,
    /// Chaîne d'affichage libre **dérivée** des champs structurés (#213).
    pub address: Option<String>,
    /// Adresse structurée (débiteur QR-bill type S / pain.001, #213). Optionnelle.
    pub address_street: Option<String>,
    pub address_building: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    /// Numéro de client attribué par l'émetteur (Story 16-3b, #151). Déjà
    /// normalisé par le caller (`normalize_optional` : trim, `""` → `None`) —
    /// `""` n'est PAS `NULL` pour un index UNIQUE et percuterait le cas
    /// majoritaire des contacts sans numéro.
    pub client_number: Option<String>,
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). Prime sur le texte libre.
    pub default_payment_terms_days: Option<i32>,
    /// Langue de correspondance (Story 20-3b1). `None` = hérite instance.
    pub language: Option<Language>,
    /// Civilité pour `{salutation}` (Story 20-3b1).
    pub salutation: Salutation,
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
    /// Prénom / nom (#213) — renseignés uniquement pour un contact `Personne`.
    /// `name` reste l'affichage canonique recomposé (« Prénom Nom »).
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub is_client: bool,
    pub is_supplier: bool,
    /// Chaîne d'affichage libre **dérivée** des champs structurés (#213).
    pub address: Option<String>,
    /// Adresse structurée (débiteur QR-bill type S / pain.001, #213). Optionnelle.
    pub address_street: Option<String>,
    pub address_building: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_city: Option<String>,
    pub address_country: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub ide_number: Option<String>,
    /// Numéro de client attribué par l'émetteur (Story 16-3b, #151). Déjà
    /// normalisé par le caller (`normalize_optional` : trim, `""` → `None`) —
    /// `""` n'est PAS `NULL` pour un index UNIQUE et percuterait le cas
    /// majoritaire des contacts sans numéro.
    pub client_number: Option<String>,
    pub default_payment_terms: Option<String>,
    /// Délai de paiement en jours (#245). Prime sur le texte libre.
    pub default_payment_terms_days: Option<i32>,
    /// Langue de correspondance (Story 20-3b1). `None` = hérite instance.
    pub language: Option<Language>,
    /// Civilité pour `{salutation}` (Story 20-3b1).
    pub salutation: Salutation,
}
