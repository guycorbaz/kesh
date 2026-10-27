//! Ce que partagent les outils de données de la 32-9a : les bornes de
//! pagination lues des arguments, la tranche faite par un outil, et les
//! **miroirs** des codes de la base — des énumérations locales, construites
//! par un `match` exhaustif ou une table écrite, que `champ_enum!` admet.
//!
//! Aucune sérialisation ici (garde `sorties_sans_serialisation_parallele`) :
//! les miroirs dérivent `Serialize`, la seule sérialisation est celle de
//! `sortie::donnees`.

use serde::{Deserialize, Serialize};

use crate::mcp::outil::{ErreurOutil, MotifArgument};
use crate::mcp::sortie::champ_enum;

/// La taille de page d'un outil : `defaut` si absente ; refusée hors de
/// `1..=max` (AC-a11 : un `limit` au-delà du maximum de l'outil n'est pas
/// transmis).
pub fn limite(limit: Option<i64>, defaut: i64, max: i64) -> Result<i64, ErreurOutil> {
    match limit {
        None => Ok(defaut),
        Some(l) if (1..=max).contains(&l) => Ok(l),
        Some(_) => Err(ErreurOutil::ArgumentsInvalides {
            argument: "limit",
            motif: MotifArgument::Invalide,
        }),
    }
}

/// Le décalage : 0 si absent ; refusé s'il est négatif.
pub fn decalage(offset: Option<i64>) -> Result<i64, ErreurOutil> {
    match offset {
        None => Ok(0),
        Some(o) if o >= 0 => Ok(o),
        Some(_) => Err(ErreurOutil::ArgumentsInvalides {
            argument: "offset",
            motif: MotifArgument::Invalide,
        }),
    }
}

/// La tranche `[offset, offset + limit)` d'une liste lue en entier.
pub fn tranche<T>(v: Vec<T>, offset: i64, limit: i64) -> Vec<T> {
    let debut = usize::try_from(offset).unwrap_or(usize::MAX);
    let n = usize::try_from(limit).unwrap_or(0);
    v.into_iter().skip(debut).take(n).collect()
}

/// Un nombre d'éléments en `i64`.
pub fn nombre(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

/// Le sens d'un tri, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SensDuTri {
    /// Ascendant.
    Asc,
    /// Descendant.
    Desc,
}

impl SensDuTri {
    /// La valeur du service.
    pub fn du_service(self) -> kesh_core::listing::SortDirection {
        match self {
            Self::Asc => kesh_core::listing::SortDirection::Asc,
            Self::Desc => kesh_core::listing::SortDirection::Desc,
        }
    }

    /// Le texte du résumé.
    pub fn code(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Le sens normal d'un compte au grand livre (`LedgerSection.balance_side`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Sens {
    /// Solde normalement débiteur.
    Debit,
    /// Solde normalement créditeur.
    Credit,
}

/// La table du sens.
pub const SENS: &[(&str, Sens)] = &[("debit", Sens::Debit), ("credit", Sens::Credit)];

/// Le statut d'une facture cliente (contrainte de la base).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatutFacture {
    /// Brouillon.
    Draft,
    /// Validée.
    Validated,
    /// Annulée.
    Cancelled,
}

/// La table du statut d'une facture.
pub const STATUTS_FACTURE: &[(&str, StatutFacture)] = &[
    ("draft", StatutFacture::Draft),
    ("validated", StatutFacture::Validated),
    ("cancelled", StatutFacture::Cancelled),
];

/// Le statut d'une facture fournisseur (contrainte de la base).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatutFactureFournisseur {
    /// Ouverte.
    Open,
    /// Payée.
    Paid,
    /// Annulée.
    Cancelled,
}

/// La table du statut d'une facture fournisseur.
pub const STATUTS_FACTURE_FOURNISSEUR: &[(&str, StatutFactureFournisseur)] = &[
    ("open", StatutFactureFournisseur::Open),
    ("paid", StatutFactureFournisseur::Paid),
    ("cancelled", StatutFactureFournisseur::Cancelled),
];

/// Le mode de règlement d'une facture fournisseur (contrainte de la base).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TypeReglement {
    /// Virement bancaire.
    BankTransfer,
    /// Compte interne.
    InternalAccount,
}

/// La table du mode de règlement.
pub const TYPES_REGLEMENT: &[(&str, TypeReglement)] = &[
    ("bank_transfer", TypeReglement::BankTransfer),
    ("internal_account", TypeReglement::InternalAccount),
];

/// Le statut d'une pièce importée (contrainte `chk_imported_si_status` de la
/// base) — rendu par `kesh_pieces`, et lu en argument (`statut`, requis).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StatutPiece {
    /// À compléter : importée, sans facture fournisseur.
    ToComplete,
    /// Complétée : la facture fournisseur en est créée.
    Completed,
    /// Écartée.
    Discarded,
}

/// La table du statut d'une pièce.
pub const STATUTS_PIECE: &[(&str, StatutPiece)] = &[
    ("to_complete", StatutPiece::ToComplete),
    ("completed", StatutPiece::Completed),
    ("discarded", StatutPiece::Discarded),
];

impl StatutPiece {
    /// La valeur de la base et du service.
    pub fn code(self) -> &'static str {
        match self {
            Self::ToComplete => "to_complete",
            Self::Completed => "completed",
            Self::Discarded => "discarded",
        }
    }
}

champ_enum!(
    Sens,
    StatutFacture,
    StatutFactureFournisseur,
    TypeReglement,
    StatutPiece
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bornes_et_tranche() {
        assert_eq!(limite(None, 12, 22), Ok(12));
        assert_eq!(limite(Some(22), 12, 22), Ok(22));
        assert!(limite(Some(23), 12, 22).is_err());
        assert!(limite(Some(0), 12, 22).is_err());
        assert_eq!(decalage(None), Ok(0));
        assert!(decalage(Some(-1)).is_err());
        assert_eq!(tranche(vec![1, 2, 3, 4], 1, 2), vec![2, 3]);
        assert_eq!(tranche(vec![1, 2], 5, 2), Vec::<i32>::new());
    }
}
