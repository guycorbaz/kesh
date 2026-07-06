//! DTO d'entrée + validation d'une **adresse structurée** (conformité SIX
//! QR-bill type S, #213). Partagé entre l'onboarding/paramètres société
//! (créancier, adresse requise) et les contacts (débiteur, adresse optionnelle).
//!
//! NPA **libre/international** (décision produit 2026-07-05) : aucune contrainte
//! « 4 chiffres » bloquante, seules les longueurs SIX §3.3 sont imposées.

use kesh_db::entities::address::StructuredAddress;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// Limites SIX 2.2 §3.3 type S.
const STREET_MAX: usize = 70;
const BUILDING_MAX: usize = 16;
const POSTAL_MAX: usize = 16;
const CITY_MAX: usize = 35;

/// Payload adresse structurée reçu du frontend (camelCase).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredAddressInput {
    #[serde(default)]
    pub street: String,
    #[serde(default)]
    pub building: String,
    #[serde(default)]
    pub postal_code: String,
    #[serde(default)]
    pub city: String,
    /// ISO-3166-1 alpha-2 ; défaut `CH` si absent/vide.
    #[serde(default)]
    pub country: Option<String>,
}

impl StructuredAddressInput {
    fn normalized_country(&self) -> String {
        match self.country.as_deref().map(str::trim) {
            Some(c) if !c.is_empty() => c.to_uppercase(),
            _ => "CH".to_string(),
        }
    }

    /// `true` si aucun composant significatif n'est renseigné (adresse absente).
    pub fn is_blank(&self) -> bool {
        self.street.trim().is_empty()
            && self.building.trim().is_empty()
            && self.postal_code.trim().is_empty()
            && self.city.trim().is_empty()
    }

    fn check_lengths(&self) -> Result<(), AppError> {
        let checks = [
            ("rue", self.street.trim().chars().count(), STREET_MAX),
            ("numéro", self.building.trim().chars().count(), BUILDING_MAX),
            ("NPA", self.postal_code.trim().chars().count(), POSTAL_MAX),
            ("localité", self.city.trim().chars().count(), CITY_MAX),
        ];
        for (label, got, max) in checks {
            if got > max {
                return Err(AppError::Validation(format!(
                    "adresse : le champ {label} dépasse {max} caractères ({got})"
                )));
            }
        }
        let country = self.normalized_country();
        if country.len() != 2 || !country.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(AppError::Validation(
                "adresse : le pays doit être un code ISO à 2 lettres (ex. CH)".into(),
            ));
        }
        Ok(())
    }

    fn build(&self) -> StructuredAddress {
        StructuredAddress {
            street: self.street.trim().to_string(),
            building: self.building.trim().to_string(),
            postal_code: self.postal_code.trim().to_string(),
            city: self.city.trim().to_string(),
            country: self.normalized_country(),
        }
    }

    /// Validation **requise** (créancier / société) : NPA et localité obligatoires
    /// (SIX type S), longueurs respectées.
    pub fn validate_required(&self) -> Result<StructuredAddress, AppError> {
        self.check_lengths()?;
        if self.postal_code.trim().is_empty() || self.city.trim().is_empty() {
            return Err(AppError::Validation(
                "adresse : le NPA et la localité sont obligatoires (format structuré exigé par les banques suisses)".into(),
            ));
        }
        Ok(self.build())
    }

    /// Validation **optionnelle** (contact / débiteur) : `None` si le bloc est
    /// entièrement vide ; sinon longueurs vérifiées. Une adresse partielle est
    /// acceptée au niveau contact — le bloc débiteur du QR n'est émis que si
    /// l'adresse est complète (cf. génération).
    pub fn validate_optional(&self) -> Result<Option<StructuredAddress>, AppError> {
        if self.is_blank() {
            return Ok(None);
        }
        self.check_lengths()?;
        Ok(Some(self.build()))
    }
}

impl From<&StructuredAddress> for StructuredAddressInput {
    fn from(a: &StructuredAddress) -> Self {
        Self {
            street: a.street.clone(),
            building: a.building.clone(),
            postal_code: a.postal_code.clone(),
            city: a.city.clone(),
            country: Some(a.country.clone()),
        }
    }
}
