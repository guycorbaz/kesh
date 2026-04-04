//! Erreurs métier du crate kesh-core.
//!
//! Chaque variante représente une violation de règle métier détectée
//! lors de la validation des types domaine.

use thiserror::Error;

/// Erreurs de validation des types métier kesh-core.
///
/// Les messages `Display` sont destinés au logging serveur uniquement.
/// `kesh-api` mappe chaque variante vers un code structuré et un message
/// traduit via `kesh-i18n`. Ne jamais exposer ces messages au frontend.
#[derive(Debug, Error, Clone, PartialEq)]
pub enum CoreError {
    /// L'IBAN fourni est invalide (format, longueur ou checksum incorrect).
    #[error("IBAN invalide : {0}")]
    InvalidIban(String),

    /// Le QR-IBAN fourni est invalide (IBAN invalide ou QR-IID hors plage 30000-31999).
    #[error("QR-IBAN invalide : {0}")]
    InvalidQrIban(String),

    /// Le numéro IDE (CHE) fourni est invalide (format ou checksum incorrect).
    #[error("Numéro IDE invalide : {0}")]
    InvalidCheNumber(String),
}

impl CoreError {
    /// Retourne le code d'erreur structuré pour le mapping API.
    ///
    /// Utilisé par `kesh-api` pour construire les réponses d'erreur JSON
    /// au format `{ "error": { "code": "INVALID_IBAN", ... } }`.
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::InvalidIban(_) => "INVALID_IBAN",
            Self::InvalidQrIban(_) => "INVALID_QR_IBAN",
            Self::InvalidCheNumber(_) => "INVALID_CHE_NUMBER",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_are_correct() {
        assert_eq!(
            CoreError::InvalidIban("test".into()).error_code(),
            "INVALID_IBAN"
        );
        assert_eq!(
            CoreError::InvalidQrIban("test".into()).error_code(),
            "INVALID_QR_IBAN"
        );
        assert_eq!(
            CoreError::InvalidCheNumber("test".into()).error_code(),
            "INVALID_CHE_NUMBER"
        );
    }

    #[test]
    fn display_messages_contain_detail() {
        let err = CoreError::InvalidIban("checksum incorrect".into());
        assert!(err.to_string().contains("checksum incorrect"));
    }

    #[test]
    fn errors_are_cloneable_and_comparable() {
        let a = CoreError::InvalidIban("test".into());
        let b = a.clone();
        assert_eq!(a, b);
    }
}
