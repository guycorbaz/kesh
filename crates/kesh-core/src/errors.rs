//! Erreurs métier du crate kesh-core.
//!
//! Chaque variante représente une violation de règle métier détectée
//! lors de la validation des types domaine.

use crate::types::Money;
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

    /// Le type d'organisation ne correspond à aucun plan comptable connu.
    #[error("Type d'organisation inconnu pour le plan comptable : {0}")]
    UnknownChartType(String),

    /// Le fichier JSON du plan comptable contient des données invalides.
    #[error("Plan comptable invalide : {0}")]
    InvalidChart(String),

    /// Une écriture doit contenir au moins deux lignes (partie double).
    #[error("Écriture invalide : au moins deux lignes requises")]
    EntryNeedsTwoLines,

    /// Le libellé de l'écriture est vide ou ne contient que des espaces.
    #[error("Écriture invalide : le libellé est obligatoire")]
    EntryDescriptionEmpty,

    /// Une ligne d'écriture contient un montant négatif (non permis en
    /// saisie directe ; les avoirs sont gérés par des écritures de
    /// contre-passation dans la story 3.3 et l'epic 10).
    #[error("Écriture invalide : montant négatif non permis en saisie directe")]
    EntryNegativeAmount,

    /// Une ligne doit avoir EXACTEMENT un des deux montants (débit ou
    /// crédit) strictement positif. Les cas rejetés : les deux à zéro,
    /// les deux positifs simultanément.
    #[error(
        "Écriture invalide : chaque ligne doit avoir soit un débit soit un crédit (exclusif)"
    )]
    EntryLineDebitCreditExclusive,

    /// Le total des débits ne correspond pas au total des crédits.
    ///
    /// FR21 : message utilisateur construit à partir des montants par
    /// `kesh-api` via `AppError::EntryUnbalanced { debit, credit }`.
    #[error("Écriture déséquilibrée : débits={debit}, crédits={credit}")]
    EntryUnbalanced {
        /// Total des débits de l'écriture.
        debit: Money,
        /// Total des crédits de l'écriture.
        credit: Money,
    },

    /// Une écriture dont le total est zéro n'est pas persistée.
    #[error("Écriture invalide : le total ne peut pas être nul")]
    EntryZeroTotal,
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
            Self::UnknownChartType(_) => "UNKNOWN_CHART_TYPE",
            Self::InvalidChart(_) => "INVALID_CHART",
            Self::EntryNeedsTwoLines => "ENTRY_NEEDS_TWO_LINES",
            Self::EntryDescriptionEmpty => "ENTRY_DESCRIPTION_EMPTY",
            Self::EntryNegativeAmount => "ENTRY_NEGATIVE_AMOUNT",
            Self::EntryLineDebitCreditExclusive => "ENTRY_LINE_DEBIT_CREDIT_EXCLUSIVE",
            Self::EntryUnbalanced { .. } => "ENTRY_UNBALANCED",
            Self::EntryZeroTotal => "ENTRY_ZERO_TOTAL",
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
