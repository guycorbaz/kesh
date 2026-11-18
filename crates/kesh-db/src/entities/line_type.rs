//! Type d'une ligne de facture client et d'avoir (Story 49-1a, #1242).
//!
//! Une ligne est **chiffrée** (`priced`, la seule forme d'avant la 0.20.0 : un
//! article du catalogue comme une saisie libre, qui ne se distinguent qu'à
//! l'écran) ou **non chiffrée** : un commentaire (`comment`) ou un titre de
//! section (`heading`). Une ligne non chiffrée porte des valeurs **neutres** —
//! quantité, prix unitaire, taux de TVA et total à zéro, aucun compte de
//! produit — que la contrainte `chk_*_unpriced_neutral` impose en base : elle ne
//! contribue à aucune somme, et seuls les lecteurs qui regroupent, comptent,
//! écrivent ou rendent ligne par ligne ont à la filtrer.
//!
//! Stocké en base dans `invoice_lines.line_type` et `credit_note_lines.line_type`
//! (`VARCHAR(16) NOT NULL DEFAULT 'priced'`, CHECK `chk_*_line_type`), sérialisé
//! en minuscules en JSON.

use serde::{Deserialize, Serialize};
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type d'une ligne de facture ou d'avoir.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LineType {
    /// Ligne chiffrée : quantité > 0, prix, taux, compte de produit.
    #[default]
    Priced,
    /// Commentaire : texte seul, valeurs neutres.
    Comment,
    /// Titre de section : texte seul, valeurs neutres.
    Heading,
}

impl LineType {
    /// Représentation persistée et JSON (minuscule).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Priced => "priced",
            Self::Comment => "comment",
            Self::Heading => "heading",
        }
    }

    /// `true` pour une ligne chiffrée, la seule qui porte un montant.
    pub fn is_priced(&self) -> bool {
        matches!(self, Self::Priced)
    }

    /// **La garde de l'API** (D9 de la Story 49-1a) : les types qu'un client
    /// HTTP peut créer ou poser par modification.
    ///
    /// Le dépôt accepte les trois types ; c'est cette fonction, appelée par la
    /// validation des lignes de `kesh-api`, qui retient `comment` et `heading`
    /// tant que leur rendu PDF (Story 49-1b) et leur écran (Story 49-1c) ne
    /// sont pas livrés — sans elle, un commentaire s'imprimerait
    /// `0 × 0.00 = 0.00`. La Story 49-1c ouvre `comment`, la Story 49-3
    /// `heading`.
    pub fn open_to_api(&self) -> bool {
        match self {
            Self::Priced => true,
            Self::Comment | Self::Heading => false,
        }
    }
}

impl std::fmt::Display for LineType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for LineType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "priced" => Ok(Self::Priced),
            "comment" => Ok(Self::Comment),
            "heading" => Ok(Self::Heading),
            other => Err(format!("type de ligne inconnu : {other}")),
        }
    }
}

impl Type<MySql> for LineType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for LineType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for LineType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn as_str_and_from_str_round_trip() {
        for t in [LineType::Priced, LineType::Comment, LineType::Heading] {
            assert_eq!(LineType::from_str(t.as_str()).unwrap(), t);
            assert_eq!(serde_json::to_value(t).unwrap(), t.as_str());
        }
        assert!(LineType::from_str("subtotal").is_err());
        assert!(LineType::from_str("Priced").is_err());
    }

    #[test]
    fn default_is_priced() {
        assert_eq!(LineType::default(), LineType::Priced);
    }

    /// La garde de la 49-1a : seul `priced` est ouvert à l'API.
    #[test]
    fn only_priced_is_open_to_api() {
        assert!(LineType::Priced.open_to_api());
        assert!(!LineType::Comment.open_to_api());
        assert!(!LineType::Heading.open_to_api());
    }
}
