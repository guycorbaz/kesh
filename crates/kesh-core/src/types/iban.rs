//! Type IBAN avec validation ISO 13616 (international).

use crate::errors::CoreError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Longueurs IBAN par pays (source: SWIFT IBAN Registry).
///
/// Cette liste reflète les pays officiellement enregistrés au registre SWIFT.
/// Les entrées doivent être vérifiées contre le document officiel
/// "IBAN Registry" publié sur swift.com avant toute mise en production.
const IBAN_LENGTHS: &[(&str, usize)] = &[
    ("AD", 24), ("AE", 23), ("AL", 28), ("AT", 20), ("AZ", 28),
    ("BA", 20), ("BE", 16), ("BG", 22), ("BH", 22), ("BI", 27),
    ("BR", 29), ("BY", 28), ("CH", 21), ("CR", 22), ("CY", 28),
    ("CZ", 24), ("DE", 22), ("DJ", 27), ("DK", 18), ("DO", 28),
    ("EE", 20), ("EG", 29), ("ES", 24), ("FI", 18),
    ("FO", 18), ("FR", 27), ("GB", 22), ("GE", 22), ("GI", 23),
    ("GL", 18), ("GR", 27), ("GT", 28), ("HR", 21), ("HU", 28),
    ("IE", 22), ("IL", 23), ("IQ", 23), ("IS", 26), ("IT", 27),
    ("JO", 30), ("KW", 30), ("KZ", 20), ("LB", 28), ("LC", 32),
    ("LI", 21), ("LT", 20), ("LU", 20), ("LV", 21), ("LY", 25),
    ("MC", 27), ("MD", 24), ("ME", 22), ("MK", 19), ("MN", 20),
    ("MR", 27), ("MT", 31), ("MU", 30), ("NI", 28), ("NL", 18),
    ("NO", 15), ("OM", 23), ("PK", 24), ("PL", 28), ("PS", 29),
    ("PT", 25), ("QA", 29), ("RO", 24), ("RS", 22), ("RU", 33),
    ("SA", 24), ("SC", 31), ("SD", 18), ("SE", 24), ("SI", 19),
    ("SK", 24), ("SM", 27), ("SN", 28), ("SO", 23), ("ST", 25),
    ("SV", 28), ("TL", 23), ("TN", 24), ("TR", 26), ("UA", 29),
    ("VA", 22), ("VG", 24), ("XK", 20),
];

/// Retourne la longueur attendue pour un code pays IBAN, ou `None` si inconnu.
fn expected_length(country_code: &str) -> Option<usize> {
    IBAN_LENGTHS
        .iter()
        .find(|(cc, _)| *cc == country_code)
        .map(|(_, len)| *len)
}

/// IBAN validé selon ISO 13616 (international).
///
/// Supporte tous les pays du registre SWIFT. La validation inclut
/// la vérification du format, de la longueur par pays et du checksum MOD-97.
///
/// # Exemples
///
/// ```
/// use kesh_core::types::Iban;
///
/// let iban = Iban::new("CH93 0076 2011 6238 5295 7").unwrap();
/// assert_eq!(iban.country_code(), "CH");
/// assert!(iban.is_swiss());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Iban(String);

impl Iban {
    /// Crée un IBAN validé à partir d'une chaîne.
    ///
    /// Accepte les formats avec ou sans espaces.
    /// Retourne `CoreError::InvalidIban` si le format, la longueur
    /// ou le checksum MOD-97 est invalide.
    pub fn new(input: &str) -> Result<Self, CoreError> {
        let normalized = input.replace(' ', "").to_uppercase();

        // Les IBAN sont exclusivement ASCII (2 lettres + 2 chiffres + alphanum).
        // Ce garde-fou empêche les panics de slicing par octet sur des caractères
        // multi-octets UTF-8 (ex: "Aéxxxxx" panique sur normalized[..2]).
        if !normalized.is_ascii() {
            return Err(CoreError::InvalidIban(
                "caractères non-ASCII interdits".into(),
            ));
        }

        if normalized.len() < 5 {
            return Err(CoreError::InvalidIban("trop court".into()));
        }

        // Vérifier format : 2 lettres + 2 chiffres + alphanumériques
        if !normalized[..2].chars().all(|c| c.is_ascii_uppercase()) {
            return Err(CoreError::InvalidIban(
                "code pays doit être 2 lettres".into(),
            ));
        }
        if !normalized[2..4].chars().all(|c| c.is_ascii_digit()) {
            return Err(CoreError::InvalidIban(
                "chiffres de contrôle attendus en positions 3-4".into(),
            ));
        }
        if !normalized[4..].chars().all(|c| c.is_ascii_alphanumeric()) {
            return Err(CoreError::InvalidIban(
                "caractères alphanumériques attendus après le code pays".into(),
            ));
        }

        // Vérifier longueur par pays
        let country_code = &normalized[..2];
        if let Some(expected) = expected_length(country_code) {
            if normalized.len() != expected {
                return Err(CoreError::InvalidIban(format!(
                    "longueur {} incorrecte pour {country_code} (attendu {expected})",
                    normalized.len()
                )));
            }
        } else {
            return Err(CoreError::InvalidIban(format!(
                "code pays inconnu : {country_code}"
            )));
        }

        // Vérifier checksum MOD-97
        if !Self::validate_mod97(&normalized) {
            return Err(CoreError::InvalidIban("checksum MOD-97 invalide".into()));
        }

        Ok(Self(normalized))
    }

    /// Retourne le code pays (2 lettres).
    pub fn country_code(&self) -> &str {
        &self.0[..2]
    }

    /// Retourne le numéro de clearing bancaire suisse (positions 5-9).
    ///
    /// Retourne `Some` uniquement pour les IBAN CH/LI. La structure du BBAN
    /// varie par pays, cette méthode n'est donc valide que pour les IBAN suisses.
    pub fn bank_clearing_number(&self) -> Option<&str> {
        if self.is_swiss() {
            Some(&self.0[4..9])
        } else {
            None
        }
    }

    /// Vérifie si l'IBAN participe au système de paiement suisse (CH ou LI).
    ///
    /// Le Liechtenstein participe à l'infrastructure SIX (Swiss Payment Standards),
    /// d'où l'inclusion de LI dans cette méthode. Les IBAN CH et LI partagent
    /// la structure 21 caractères et peuvent tous deux être utilisés pour
    /// des QR-IBAN (si le QR-IID est dans la plage 30000-31999).
    pub fn is_swiss(&self) -> bool {
        let cc = self.country_code();
        cc == "CH" || cc == "LI"
    }

    /// Retourne l'IBAN normalisé sans espaces.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Retourne l'IBAN formaté avec des espaces par groupes de 4.
    pub fn formatted(&self) -> String {
        self.0
            .as_bytes()
            .chunks(4)
            .map(|chunk| {
                std::str::from_utf8(chunk)
                    .expect("invariant: IBAN est ASCII après validation par Iban::new")
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Validation MOD-97 (ISO 13616).
    fn validate_mod97(iban: &str) -> bool {
        // Déplacer les 4 premiers caractères à la fin
        let rearranged = format!("{}{}", &iban[4..], &iban[..4]);

        // Convertir les lettres en nombres et calculer modulo 97
        let mut remainder: u64 = 0;
        for ch in rearranged.chars() {
            let val = if ch.is_ascii_digit() {
                (ch as u64) - ('0' as u64)
            } else {
                (ch as u64) - ('A' as u64) + 10
            };

            if val >= 10 {
                // Lettre convertie en 2 chiffres
                remainder = (remainder * 100 + val) % 97;
            } else {
                remainder = (remainder * 10 + val) % 97;
            }
        }

        remainder == 1
    }
}

impl fmt::Display for Iban {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Iban {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Iban {
    type Error = CoreError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<Iban> for String {
    fn from(iban: Iban) -> Self {
        iban.0
    }
}

impl AsRef<str> for Iban {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- IBAN suisses valides ---

    #[test]
    fn valid_swiss_iban() {
        let iban = Iban::new("CH93 0076 2011 6238 5295 7").unwrap();
        assert_eq!(iban.country_code(), "CH");
        assert!(iban.is_swiss());
        assert_eq!(iban.bank_clearing_number(), Some("00762"));
        assert_eq!(iban.as_str(), "CH9300762011623852957");
    }

    #[test]
    fn valid_swiss_iban_no_spaces() {
        let iban = Iban::new("CH9300762011623852957").unwrap();
        assert_eq!(iban.as_str(), "CH9300762011623852957");
    }

    #[test]
    fn formatted_groups_of_4() {
        let iban = Iban::new("CH9300762011623852957").unwrap();
        assert_eq!(iban.formatted(), "CH93 0076 2011 6238 5295 7");
    }

    // --- IBAN internationaux valides ---

    #[test]
    fn valid_german_iban() {
        let iban = Iban::new("DE89 3704 0044 0532 0130 00").unwrap();
        assert_eq!(iban.country_code(), "DE");
        assert!(!iban.is_swiss());
        assert_eq!(iban.bank_clearing_number(), None);
    }

    #[test]
    fn valid_french_iban() {
        let iban = Iban::new("FR76 3000 6000 0112 3456 7890 189").unwrap();
        assert_eq!(iban.country_code(), "FR");
    }

    #[test]
    fn valid_austrian_iban() {
        let iban = Iban::new("AT61 1904 3002 3457 3201").unwrap();
        assert_eq!(iban.country_code(), "AT");
    }

    #[test]
    fn valid_liechtenstein_iban() {
        let iban = Iban::new("LI21 0881 0000 2324 013A A").unwrap();
        assert!(iban.is_swiss()); // LI is treated as Swiss for clearing number
    }

    // --- IBAN invalides ---

    #[test]
    fn invalid_checksum() {
        let result = Iban::new("CH93 0076 2011 6238 5295 0"); // checksum altéré
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), "INVALID_IBAN");
    }

    #[test]
    fn invalid_length_for_country() {
        let result = Iban::new("CH93 0076 2011 6238 5295"); // trop court pour CH (20 vs 21)
        assert!(result.is_err());
    }

    #[test]
    fn too_short() {
        let result = Iban::new("CH93");
        assert!(result.is_err());
    }

    #[test]
    fn unknown_country() {
        let result = Iban::new("XX12 3456 7890 1234 5678 90");
        assert!(result.is_err());
    }

    #[test]
    fn non_ascii_rejected_without_panic() {
        // Cas qui panicait avant le fix : caractère UTF-8 multi-octet
        // à une position créant un slice non-aligné sur byte boundary.
        let result = Iban::new("Aéxxxxxxxxxxxxxxxx");
        assert!(result.is_err());
    }

    #[test]
    fn non_ascii_country_code_rejected() {
        let result = Iban::new("éé12345678901234567");
        assert!(result.is_err());
    }

    #[test]
    fn lowercase_accepted() {
        let iban = Iban::new("ch93 0076 2011 6238 5295 7").unwrap();
        assert_eq!(iban.country_code(), "CH");
    }

    // --- Serde round-trip ---

    #[test]
    fn serde_roundtrip() {
        let iban = Iban::new("CH9300762011623852957").unwrap();
        let json = serde_json::to_string(&iban).unwrap();
        assert_eq!(json, r#""CH9300762011623852957""#);
        let deserialized: Iban = serde_json::from_str(&json).unwrap();
        assert_eq!(iban, deserialized);
    }

    #[test]
    fn serde_rejects_invalid() {
        let result = serde_json::from_str::<Iban>(r#""INVALID""#);
        assert!(result.is_err());
    }

    // --- Traits ---

    #[test]
    fn display_without_spaces() {
        let iban = Iban::new("CH93 0076 2011 6238 5295 7").unwrap();
        assert_eq!(format!("{iban}"), "CH9300762011623852957");
    }

    #[test]
    fn from_str_trait() {
        let iban: Iban = "CH9300762011623852957".parse().unwrap();
        assert_eq!(iban.as_str(), "CH9300762011623852957");
    }

    #[test]
    fn try_from_string() {
        let iban = Iban::try_from("CH9300762011623852957".to_string()).unwrap();
        assert_eq!(iban.as_str(), "CH9300762011623852957");
    }

    #[test]
    fn into_string() {
        let iban = Iban::new("CH9300762011623852957").unwrap();
        let s: String = iban.into();
        assert_eq!(s, "CH9300762011623852957");
    }

    #[test]
    fn as_ref_str() {
        let iban = Iban::new("CH9300762011623852957").unwrap();
        let s: &str = iban.as_ref();
        assert_eq!(s, "CH9300762011623852957");
    }
}
