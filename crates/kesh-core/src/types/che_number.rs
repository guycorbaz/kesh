//! Type numéro IDE suisse (CHE) avec validation modulo 11 (eCH-0097 v2.0).

use crate::errors::CoreError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Poids pour le calcul du checksum modulo 11 (eCH-0097 v2.0).
const CHE_WEIGHTS: [u32; 8] = [5, 4, 3, 2, 7, 6, 5, 4];

/// Numéro d'identification des entreprises suisse (IDE/UID).
///
/// Stocke la forme normalisée sans séparateurs (ex: `"CHE109322551"`).
/// Le format d'affichage (`Display`) utilise les séparateurs : `CHE-109.322.551`.
///
/// Accepte les variantes courantes à la construction :
/// - `CHE-109.322.551`
/// - `CHE109322551`
/// - `che-109.322.551` (minuscules)
/// - `CHE-109.322.551 MWST` / `TVA` / `IVA` (suffixe TVA retiré)
///
/// # Exemples
///
/// ```
/// use kesh_core::types::CheNumber;
///
/// let che = CheNumber::new("CHE-109.322.551").unwrap();
/// assert_eq!(che.formatted(), "CHE-109.322.551");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct CheNumber(String);

impl CheNumber {
    /// Crée un numéro IDE validé.
    ///
    /// Accepte les formats avec ou sans séparateurs, en majuscules ou
    /// minuscules, avec ou sans suffixe TVA (MWST/TVA/IVA).
    pub fn new(input: &str) -> Result<Self, CoreError> {
        let normalized = Self::normalize(input)?;
        Self::validate_checksum(&normalized)?;
        Ok(Self(normalized))
    }

    /// Retourne le numéro IDE formaté avec séparateurs : `CHE-xxx.xxx.xxx`.
    pub fn formatted(&self) -> String {
        let digits = &self.0[3..]; // Skip "CHE"
        format!("CHE-{}.{}.{}", &digits[0..3], &digits[3..6], &digits[6..9])
    }

    /// Retourne la forme normalisée sans séparateurs.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Normalise l'entrée : majuscules, retrait séparateurs et suffixe TVA.
    ///
    /// Tolère les variations de whitespace Unicode (tab, NBSP, etc.) fréquentes
    /// dans les copier-coller depuis des PDF commerciaux.
    fn normalize(input: &str) -> Result<String, CoreError> {
        // Normaliser tous les whitespace Unicode en espace ASCII simple
        // (tab, NBSP U+00A0, etc. → ' ').
        let normalized_ws: String = input
            .chars()
            .map(|c| if c.is_whitespace() { ' ' } else { c })
            .collect();
        let mut s = normalized_ws.trim().to_uppercase();

        // Retirer le suffixe TVA/MWST/IVA (avec un ou plusieurs espaces devant)
        for suffix in &["MWST", "TVA", "IVA"] {
            if s.ends_with(suffix) {
                let stripped = s[..s.len() - suffix.len()].trim_end();
                s = stripped.to_string();
                break;
            }
        }

        // Retirer les séparateurs
        let s = s.replace(['-', '.', ' '], "");

        // Vérifier le préfixe
        if !s.starts_with("CHE") {
            return Err(CoreError::InvalidCheNumber("doit commencer par CHE".into()));
        }

        // Vérifier qu'il y a exactement 9 chiffres après CHE
        let digits = &s[3..];
        if digits.len() != 9 {
            return Err(CoreError::InvalidCheNumber(format!(
                "9 chiffres attendus après CHE, trouvé {}",
                digits.len()
            )));
        }
        if !digits.chars().all(|c| c.is_ascii_digit()) {
            return Err(CoreError::InvalidCheNumber(
                "seuls des chiffres sont attendus après CHE".into(),
            ));
        }

        Ok(s)
    }

    /// Valide le checksum modulo 11 (eCH-0097 v2.0).
    fn validate_checksum(normalized: &str) -> Result<(), CoreError> {
        let digits: Vec<u32> = normalized[3..]
            .chars()
            .map(|c| c.to_digit(10).unwrap())
            .collect();

        let sum: u32 = digits[..8]
            .iter()
            .zip(CHE_WEIGHTS.iter())
            .map(|(d, w)| d * w)
            .sum();

        let remainder = sum % 11;

        if remainder == 1 {
            return Err(CoreError::InvalidCheNumber(
                "numéro invalide (check digit serait 10)".into(),
            ));
        }

        let expected_check = if remainder == 0 { 0 } else { 11 - remainder };

        if digits[8] != expected_check {
            return Err(CoreError::InvalidCheNumber(format!(
                "checksum invalide (attendu {expected_check}, trouvé {})",
                digits[8]
            )));
        }

        Ok(())
    }
}

impl fmt::Display for CheNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}

impl FromStr for CheNumber {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for CheNumber {
    type Error = CoreError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<CheNumber> for String {
    fn from(che: CheNumber) -> Self {
        che.0
    }
}

impl AsRef<str> for CheNumber {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Cas valides ---

    #[test]
    fn valid_official_example() {
        // Exemple officiel eCH-0097 : somme=109, 109%11=10, check=11-10=1
        let che = CheNumber::new("CHE-109.322.551").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
        assert_eq!(che.as_str(), "CHE109322551");
    }

    #[test]
    fn valid_without_separators() {
        let che = CheNumber::new("CHE109322551").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_lowercase() {
        let che = CheNumber::new("che-109.322.551").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_mwst_suffix() {
        let che = CheNumber::new("CHE-109.322.551 MWST").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_tva_suffix() {
        let che = CheNumber::new("CHE-109.322.551 TVA").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_iva_suffix() {
        let che = CheNumber::new("CHE-109.322.551 IVA").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_tab_before_suffix() {
        let che = CheNumber::new("CHE-109.322.551\tMWST").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_nbsp_before_suffix() {
        let che = CheNumber::new("CHE-109.322.551\u{00A0}MWST").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_with_double_space_before_suffix() {
        let che = CheNumber::new("CHE-109.322.551  MWST").unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }

    #[test]
    fn valid_check_digit_zero() {
        // CHE-000.000.000 : sum=0, 0%11=0, check=0
        let che = CheNumber::new("CHE-000.000.000").unwrap();
        assert_eq!(che.formatted(), "CHE-000.000.000");
    }

    #[test]
    fn valid_another_example() {
        // CHE-123.456.788 : 1*5+2*4+3*3+4*2+5*7+6*6+7*5+8*4=168, 168%11=3, check=11-3=8
        let che = CheNumber::new("CHE-123.456.788").unwrap();
        assert_eq!(che.formatted(), "CHE-123.456.788");
    }

    // --- Cas invalides ---

    #[test]
    fn invalid_checksum() {
        let result = CheNumber::new("CHE-109.322.552"); // check digit 2 au lieu de 1
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), "INVALID_CHE_NUMBER");
    }

    #[test]
    fn invalid_check_digit_10() {
        // Besoin d'un cas où remainder=1 → check digit serait 10 → invalide
        // Recherche : digits tels que sum % 11 == 1
        // Ex: 1*5+0*4+0*3+0*2+0*7+0*6+0*5+1*4 = 9 → 9%11=9, check=2 → pas bon
        // Essayons: 2*5+0*4+0*3+0*2+0*7+0*6+0*5+2*4 = 18 → 18%11=7, check=4 → non
        // 1*5+1*4+0*3+0*2+0*7+0*6+0*5+0*4 = 9 → non
        // 3*5+0*4+0*3+0*2+0*7+0*6+0*5+0*4 = 15 → 15%11=4 → non
        // 1*5+0*4+0*3+3*2+0*7+0*6+0*5+0*4 = 11 → 11%11=0 → check=0
        // 1*5+0*4+0*3+0*2+1*7+0*6+0*5+0*4 = 12 → 12%11=1 → INVALIDE!
        let result = CheNumber::new("CHE-100.010.000"); // any last digit is invalid
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("check digit serait 10"));
    }

    #[test]
    fn invalid_prefix() {
        let result = CheNumber::new("ABC-109.322.551");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_too_few_digits() {
        let result = CheNumber::new("CHE-109.322.55");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_too_many_digits() {
        let result = CheNumber::new("CHE-109.322.5512");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_non_digits() {
        let result = CheNumber::new("CHE-10A.322.551");
        assert!(result.is_err());
    }

    // --- Display ---

    #[test]
    fn display_with_separators() {
        let che = CheNumber::new("CHE109322551").unwrap();
        assert_eq!(format!("{che}"), "CHE-109.322.551");
    }

    // --- Serde ---

    #[test]
    fn serde_roundtrip() {
        let che = CheNumber::new("CHE-109.322.551").unwrap();
        let json = serde_json::to_string(&che).unwrap();
        assert_eq!(json, r#""CHE109322551""#); // Normalisé sans séparateurs
        let deserialized: CheNumber = serde_json::from_str(&json).unwrap();
        assert_eq!(che, deserialized);
    }

    #[test]
    fn serde_rejects_invalid() {
        let result = serde_json::from_str::<CheNumber>(r#""INVALID""#);
        assert!(result.is_err());
    }

    #[test]
    fn serde_rejects_bad_checksum() {
        let result = serde_json::from_str::<CheNumber>(r#""CHE109322552""#);
        assert!(result.is_err());
    }

    // --- Conversions ---

    #[test]
    fn into_string() {
        let che = CheNumber::new("CHE-109.322.551").unwrap();
        let s: String = che.into();
        assert_eq!(s, "CHE109322551");
    }

    #[test]
    fn as_ref_str() {
        let che = CheNumber::new("CHE-109.322.551").unwrap();
        let s: &str = che.as_ref();
        assert_eq!(s, "CHE109322551");
    }

    #[test]
    fn try_from_string() {
        let che = CheNumber::try_from("CHE109322551".to_string()).unwrap();
        assert_eq!(che.formatted(), "CHE-109.322.551");
    }
}
