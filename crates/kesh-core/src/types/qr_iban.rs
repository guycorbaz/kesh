//! Type QR-IBAN avec validation SIX (plage QR-IID 30000-31999).

use crate::errors::CoreError;
use crate::types::Iban;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// QR-IBAN validé selon les spécifications SIX.
///
/// Un QR-IBAN est un IBAN suisse ou liechtensteinois dont le numéro de
/// clearing bancaire (QR-IID, positions 5-9) est dans la plage 30000-31999.
/// Il est utilisé exclusivement pour les paiements entrants via QR Bill.
///
/// # Exemples
///
/// ```
/// use kesh_core::types::QrIban;
///
/// let qr = QrIban::new("CH44 3199 9123 0008 8901 2").unwrap();
/// assert_eq!(qr.qr_iid(), 31999);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct QrIban {
    iban: Iban,
    iid: u32,
}

impl QrIban {
    /// Crée un QR-IBAN validé.
    ///
    /// L'entrée doit être un IBAN suisse/LI valide dont le QR-IID
    /// (positions 5-9) est dans la plage 30000-31999.
    ///
    /// Fix #6 : Les erreurs IBAN sous-jacentes sont propagées telles quelles
    /// (sans re-wrapping en `InvalidQrIban`) pour préserver le diagnostic exact.
    pub fn new(input: &str) -> Result<Self, CoreError> {
        let iban = Iban::new(input)?;

        if !iban.is_swiss() {
            return Err(CoreError::InvalidQrIban(
                "QR-IBAN doit être suisse ou liechtensteinois".into(),
            ));
        }

        let iid: u32 = iban.as_str()[4..9]
            .parse()
            .map_err(|_| CoreError::InvalidQrIban("QR-IID non numérique".into()))?;

        if !(30000..=31999).contains(&iid) {
            return Err(CoreError::InvalidQrIban(format!(
                "QR-IID {iid} hors plage 30000-31999"
            )));
        }

        Ok(Self { iban, iid })
    }

    /// Retourne le QR-IID (numéro de clearing QR, 30000-31999).
    pub fn qr_iid(&self) -> u32 {
        self.iid
    }

    /// Retourne l'IBAN sous-jacent.
    pub fn as_iban(&self) -> &Iban {
        &self.iban
    }

    /// Retourne la chaîne IBAN normalisée.
    pub fn as_str(&self) -> &str {
        self.iban.as_str()
    }
}

impl fmt::Display for QrIban {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.iban)
    }
}

impl FromStr for QrIban {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for QrIban {
    type Error = CoreError;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.parse()
    }
}

impl From<QrIban> for String {
    fn from(qr: QrIban) -> Self {
        String::from(qr.iban)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_qr_iban_upper_bound() {
        let qr = QrIban::new("CH44 3199 9123 0008 8901 2").unwrap();
        assert_eq!(qr.qr_iid(), 31999);
        assert_eq!(qr.as_iban().country_code(), "CH");
    }

    #[test]
    fn valid_qr_iban_lower_bound() {
        // QR-IID 30000 (check digits calculés via MOD-97)
        let qr = QrIban::new("CH57 3000 0123 4567 8901 2").unwrap();
        assert_eq!(qr.qr_iid(), 30000);
    }

    #[test]
    fn regular_iban_rejected() {
        // CH93 0076... has IID 00762 — not in QR range
        let result = QrIban::new("CH93 0076 2011 6238 5295 7");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().error_code(), "INVALID_QR_IBAN");
    }

    #[test]
    fn german_iban_rejected() {
        let result = QrIban::new("DE89 3704 0044 0532 0130 00");
        assert!(result.is_err());
    }

    #[test]
    fn iid_below_range_rejected() {
        // IID 29999 is below the QR range
        let result = QrIban::new("CH56 2999 9123 4567 8901 2");
        assert!(result.is_err());
    }

    #[test]
    fn iid_above_range_rejected() {
        // IID 32000 is above the QR range
        let result = QrIban::new("CH36 3200 0123 4567 8901 2");
        assert!(result.is_err());
    }

    #[test]
    fn display() {
        let qr = QrIban::new("CH44 3199 9123 0008 8901 2").unwrap();
        assert_eq!(format!("{qr}"), "CH4431999123000889012");
    }

    #[test]
    fn serde_roundtrip() {
        let qr = QrIban::new("CH4431999123000889012").unwrap();
        let json = serde_json::to_string(&qr).unwrap();
        assert_eq!(json, r#""CH4431999123000889012""#);
        let deserialized: QrIban = serde_json::from_str(&json).unwrap();
        assert_eq!(qr, deserialized);
    }

    #[test]
    fn serde_rejects_regular_iban() {
        let result = serde_json::from_str::<QrIban>(r#""CH9300762011623852957""#);
        assert!(result.is_err());
    }

    #[test]
    fn into_string() {
        let qr = QrIban::new("CH4431999123000889012").unwrap();
        let s: String = qr.into();
        assert_eq!(s, "CH4431999123000889012");
    }

    #[test]
    fn from_str_trait() {
        let qr: QrIban = "CH4431999123000889012".parse().unwrap();
        assert_eq!(qr.qr_iid(), 31999);
    }
}
