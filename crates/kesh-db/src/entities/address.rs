//! Adresse postale **structurée** (conformité SIX QR-bill type S, #213).
//!
//! Depuis l'échéance SIX du 21.11.2025, seule l'adresse structurée (rue,
//! n° de bâtiment, NPA, localité, pays en champs séparés) est acceptée par
//! les QR-factures et pain.001. Cette struct est la source de vérité ;
//! [`StructuredAddress::combined`] en dérive une chaîne d'affichage libre
//! (stockée dans la colonne `address` conservée pour l'UI et l'historique).
//!
//! Longueurs alignées SIX 2.2 §3.3 : rue ≤70, n° ≤16, NPA ≤16, localité ≤35,
//! pays = ISO-3166-1 alpha-2.

use serde::{Deserialize, Serialize};

/// Adresse structurée (5 composants). Les limites de longueur SIX sont
/// vérifiées côté API avant persistance (messages i18n).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StructuredAddress {
    /// Rue (sans le numéro). SIX element 3, ≤70.
    pub street: String,
    /// Numéro de bâtiment. SIX element 4, ≤16. Peut être vide.
    pub building: String,
    /// NPA / code postal. SIX element 5, ≤16.
    pub postal_code: String,
    /// Localité. SIX element 6, ≤35.
    pub city: String,
    /// Pays ISO-3166-1 alpha-2 (défaut `CH`).
    pub country: String,
}

impl StructuredAddress {
    /// Recompose une chaîne d'affichage libre sur 2 lignes (`rue n°\nNPA localité`),
    /// dérivée pour la colonne `address` conservée. Champs vides omis proprement.
    pub fn combined(&self) -> String {
        let line1 = if self.building.trim().is_empty() {
            self.street.trim().to_string()
        } else {
            format!("{} {}", self.street.trim(), self.building.trim())
        };
        let line2 = format!("{} {}", self.postal_code.trim(), self.city.trim());
        [line1, line2.trim().to_string()]
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// `true` si aucun composant significatif n'est renseigné (adresse absente).
    pub fn is_empty(&self) -> bool {
        self.street.trim().is_empty()
            && self.building.trim().is_empty()
            && self.postal_code.trim().is_empty()
            && self.city.trim().is_empty()
    }
}
