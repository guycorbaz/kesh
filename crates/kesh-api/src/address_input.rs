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
            // ASCII seulement : `to_uppercase` changerait `ß` en `SS`, un code connu.
            Some(c) if !c.is_empty() => c.to_ascii_uppercase(),
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

    /// Longueurs SIX, forme du pays (deux lettres), puis pays **connu** de la
    /// liste ISO 3166-1 alpha-2 (#817, 0.17.4) — sauf s'il égale, après
    /// normalisation, `stored_country`, le pays déjà enregistré : un contact
    /// enregistré avant la 0.17.4 avec un code inconnu reste modifiable sans
    /// toucher à son pays ; un pays nouveau ou changé doit être connu.
    fn check_lengths(&self, stored_country: Option<&str>) -> Result<(), AppError> {
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
        let unchanged = stored_country.is_some_and(|s| s.trim().to_uppercase() == country);
        if !unchanged && !kesh_qrbill::is_iso_3166_alpha2(&country) {
            return Err(AppError::Validation(format!(
                "adresse : le pays « {country} » n'est pas un code ISO 3166-1 alpha-2 connu (ex. CH, FR, DE)"
            )));
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
    /// Le pays doit être un code ISO connu (#817) : aucune valeur n'est encore
    /// enregistrée à l'assistant.
    ///
    /// « Obligatoire » se juge par la règle de la QR-facture,
    /// [`kesh_qrbill::champ_vide_pour_qr`], et non par `trim().is_empty()` : un champ
    /// fait du seul trait d'union conditionnel (U+00AD), que `trim` laisse, serait
    /// accepté ici puis refusé au PDF (revue de l'intégration 0.17.5, M-1). La règle
    /// vaut pour l'assistant comme pour la correction de l'adresse.
    pub fn validate_required(&self) -> Result<StructuredAddress, AppError> {
        self.check_lengths(None)?;
        for (label, value) in [("NPA", &self.postal_code), ("localité", &self.city)] {
            if kesh_qrbill::champ_vide_pour_qr(value) {
                return Err(AppError::Validation(format!(
                    "adresse : le champ {label} est obligatoire (format structuré exigé par les banques suisses)"
                )));
            }
        }
        Ok(self.build())
    }

    /// [`Self::validate_required`], plus le jeu de caractères de la QR-facture : pour
    /// la correction de l'adresse de la société (Story 33-5, #909), qui ne doit pas
    /// annoncer un succès en laissant la QR-facture en panne. La règle reste celle de
    /// `kesh-qrbill`, et le refus nomme le champ et le caractère.
    pub fn validate_required_for_qr(&self) -> Result<StructuredAddress, AppError> {
        let address = self.validate_required()?;
        for (label, value) in [
            ("rue", &address.street),
            ("numéro", &address.building),
            ("NPA", &address.postal_code),
            ("localité", &address.city),
        ] {
            if let Some(c) = kesh_qrbill::first_qr_refused_char(value) {
                return Err(AppError::Validation(format!(
                    "adresse : le champ {label} contient un caractère que la QR-facture n'admet pas : « {} » (U+{:04X})",
                    c.escape_debug(),
                    c as u32
                )));
            }
        }
        Ok(address)
    }

    /// Validation **optionnelle** (contact / débiteur) : `None` si le bloc est
    /// entièrement vide ; sinon longueurs vérifiées. Une adresse partielle est
    /// acceptée au niveau contact — le bloc débiteur du QR n'est émis que si
    /// l'adresse est complète (cf. génération).
    /// Le pays doit être un code ISO connu (#817) ; c'est la validation d'une
    /// création, où rien n'est encore enregistré.
    pub fn validate_optional(&self) -> Result<Option<StructuredAddress>, AppError> {
        self.validate_optional_keeping(None)
    }

    /// Comme [`Self::validate_optional`], pour une **mise à jour** : un pays égal
    /// (après normalisation) à `stored_country`, celui que la ligne porte déjà,
    /// est accepté même hors de la liste ISO (#817) — une adresse enregistrée
    /// avant la 0.17.4 avec « SU » se modifie sans toucher au pays ; changer le
    /// pays exige un code connu.
    pub fn validate_optional_keeping(
        &self,
        stored_country: Option<&str>,
    ) -> Result<Option<StructuredAddress>, AppError> {
        if self.is_blank() {
            return Ok(None);
        }
        self.check_lengths(stored_country)?;
        Ok(Some(self.build()))
    }
}

/// `true` si le pays d'une adresse **lue** est un code ISO 3166-1 alpha-2 connu
/// (après normalisation : blancs rognés, majuscules), ou s'il n'y a pas de pays
/// (le défaut est `CH`). Sert au champ `addressCountryKnown` des réponses de
/// contact (#817) : l'écran signale un code inconnu sans recopier la liste ISO.
pub fn is_known_country(country: Option<&str>) -> bool {
    match country.map(str::trim) {
        Some(c) if !c.is_empty() => kesh_qrbill::is_iso_3166_alpha2(&c.to_uppercase()),
        _ => true,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn addr(country: Option<&str>) -> StructuredAddressInput {
        StructuredAddressInput {
            street: "Rue du Test".into(),
            building: "1".into(),
            postal_code: "1000".into(),
            city: "Lausanne".into(),
            country: country.map(str::to_string),
        }
    }

    fn message<T: std::fmt::Debug>(r: Result<T, AppError>) -> String {
        match r {
            Err(AppError::Validation(m)) => m,
            other => panic!("refus de validation attendu, obtenu {other:?}"),
        }
    }

    /// AC-2.4 (#817) — codes connus, casse et blancs normalisés, défaut `CH`.
    #[test]
    fn known_countries_are_accepted_817() {
        for (c, attendu) in [
            (Some("CH"), "CH"),
            (Some("ch"), "CH"),
            (Some(" li "), "LI"),
            (Some(""), "CH"),
            (None, "CH"),
        ] {
            let a = addr(c).validate_required().expect("pays connu");
            assert_eq!(a.country, attendu, "{c:?}");
        }
    }

    /// AC-2.4 (#817) — un code de deux lettres hors de la liste ISO est refusé,
    /// et le message le nomme.
    #[test]
    fn unknown_countries_are_refused_and_named_817() {
        for c in ["SU", "XX", "ZZ", "UK"] {
            let m = message(addr(Some(c)).validate_required());
            assert!(m.contains(&format!("« {c} »")), "{c} : {m}");
            let m = message(addr(Some(c)).validate_optional());
            assert!(m.contains(&format!("« {c} »")), "{c} : {m}");
        }
    }

    /// #817 — à la mise à jour, un pays inconnu **inchangé** (casse et blancs
    /// normalisés) est toléré ; changé, il doit être connu.
    #[test]
    fn an_unchanged_stored_country_is_tolerated_817() {
        let a = addr(Some("su"))
            .validate_optional_keeping(Some("SU"))
            .expect("pays inchangé toléré")
            .expect("adresse");
        assert_eq!(a.country, "SU");
        let m = message(addr(Some("XX")).validate_optional_keeping(Some("SU")));
        assert!(m.contains("« XX »"), "{m}");
        assert!(
            addr(Some("CH"))
                .validate_optional_keeping(Some("SU"))
                .is_ok()
        );
    }

    /// #817 — `addressCountryKnown` : connu, inconnu, absent.
    #[test]
    fn known_country_flag_817() {
        assert!(is_known_country(Some("CH")));
        assert!(is_known_country(Some(" ch ")));
        assert!(is_known_country(None));
        assert!(is_known_country(Some("")));
        assert!(!is_known_country(Some("SU")));
    }

    /// AC-2.4 — la forme (deux lettres ASCII) garde son message d'aujourd'hui.
    #[test]
    fn malformed_countries_keep_the_shape_message_817() {
        for c in ["C1", "CHE"] {
            assert_eq!(
                message(addr(Some(c)).validate_required()),
                "adresse : le pays doit être un code ISO à 2 lettres (ex. CH)",
                "{c}"
            );
        }
    }

    /// Revue de l'intégration 0.17.5 (M-1) — l'assistant juge « obligatoire » par la
    /// règle de la QR-facture : un NPA ou une localité sans rien de visible est
    /// refusé, champ nommé ; un U+00AD entre deux lettres reste admis.
    #[test]
    fn required_fields_with_nothing_visible_are_refused() {
        for (postal, city, label) in [
            ("1000", "\u{AD}", "localité"),
            ("\u{AD}\u{A0}\u{AD}", "Lausanne", "NPA"),
            ("", "Lausanne", "NPA"),
            ("1000", "  ", "localité"),
        ] {
            let mut a = addr(None);
            a.postal_code = postal.into();
            a.city = city.into();
            let m = message(a.validate_required());
            assert!(
                m.contains(&format!("le champ {label} est obligatoire")),
                "{postal:?} / {city:?} : {m}"
            );
        }
        let mut a = addr(None);
        a.city = "Lau\u{AD}sanne".into();
        assert!(a.validate_required().is_ok());
    }
}
