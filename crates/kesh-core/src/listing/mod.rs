//! Types de listing : tri et direction pour les listes paginées.
//!
//! Enums réutilisables par toutes les listes (écritures, contacts,
//! factures). Les variants sont volontairement en **PascalCase** pour
//! cohérence avec le pattern existant (`Journal` story 3.2 : `Achats`,
//! `Ventes`, etc.). Le mélange avec les noms de champs DTO en
//! `camelCase` (via `#[serde(rename_all = "camelCase")]` sur les DTOs
//! routes) est intentionnel et documenté dans la spec story 3.4.
//!
//! **Anti-SQL-injection** : les méthodes `as_sql_*` retournent des
//! littéraux `&'static str` — whitelist stricte. Les valeurs ne sont
//! JAMAIS concaténées depuis l'input utilisateur.

use serde::{Deserialize, Serialize};

/// Direction de tri pour les listes paginées.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortDirection {
    /// Ascendant (plus petit → plus grand).
    Asc,
    /// Descendant (plus grand → plus petit).
    Desc,
}

impl SortDirection {
    /// Retourne le mot-clé SQL littéral (whitelist anti-injection).
    pub fn as_sql_keyword(&self) -> &'static str {
        match self {
            Self::Asc => "ASC",
            Self::Desc => "DESC",
        }
    }
}

impl Default for SortDirection {
    /// Défaut : `Desc` (convention comptable — plus récent en haut).
    fn default() -> Self {
        Self::Desc
    }
}

/// Colonne de tri pour les listes d'écritures comptables.
///
/// Les variants correspondent aux colonnes de `journal_entries`
/// autorisées pour le tri. Le tri par « Total » n'est pas supporté
/// en v0.1 (nécessiterait un SUM par ligne trop coûteux).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SortBy {
    /// Date de l'écriture (`journal_entries.entry_date`).
    EntryDate,
    /// Numéro séquentiel (`journal_entries.entry_number`).
    EntryNumber,
    /// Journal comptable (`journal_entries.journal`).
    Journal,
    /// Libellé (`journal_entries.description`).
    Description,
}

impl SortBy {
    /// Retourne la colonne SQL littérale (whitelist anti-injection).
    ///
    /// **CRITIQUE** : les valeurs retournées doivent toujours être des
    /// `&'static str` littéraux. Ne jamais construire dynamiquement
    /// depuis l'input utilisateur.
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            Self::EntryDate => "entry_date",
            Self::EntryNumber => "entry_number",
            Self::Journal => "journal",
            Self::Description => "description",
        }
    }
}

impl Default for SortBy {
    /// Défaut : `EntryDate` (tri chronologique standard).
    fn default() -> Self {
        Self::EntryDate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort_direction_sql_keyword() {
        assert_eq!(SortDirection::Asc.as_sql_keyword(), "ASC");
        assert_eq!(SortDirection::Desc.as_sql_keyword(), "DESC");
    }

    #[test]
    fn sort_direction_default_is_desc() {
        assert_eq!(SortDirection::default(), SortDirection::Desc);
    }

    #[test]
    fn sort_by_all_variants_have_sql_column() {
        // Exhaustivité des variants — si un variant est ajouté, ce test
        // doit être mis à jour (garde-fou anti-dérive).
        assert_eq!(SortBy::EntryDate.as_sql_column(), "entry_date");
        assert_eq!(SortBy::EntryNumber.as_sql_column(), "entry_number");
        assert_eq!(SortBy::Journal.as_sql_column(), "journal");
        assert_eq!(SortBy::Description.as_sql_column(), "description");
    }

    #[test]
    fn sort_by_default_is_entry_date() {
        assert_eq!(SortBy::default(), SortBy::EntryDate);
    }

    #[test]
    fn sort_direction_serde_roundtrip() {
        for dir in [SortDirection::Asc, SortDirection::Desc] {
            let json = serde_json::to_string(&dir).unwrap();
            let back: SortDirection = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dir);
        }
    }

    #[test]
    fn sort_direction_serde_pascal_case() {
        assert_eq!(
            serde_json::to_string(&SortDirection::Asc).unwrap(),
            "\"Asc\""
        );
        assert_eq!(
            serde_json::to_string(&SortDirection::Desc).unwrap(),
            "\"Desc\""
        );
    }

    #[test]
    fn sort_by_serde_roundtrip() {
        for sort in [
            SortBy::EntryDate,
            SortBy::EntryNumber,
            SortBy::Journal,
            SortBy::Description,
        ] {
            let json = serde_json::to_string(&sort).unwrap();
            let back: SortBy = serde_json::from_str(&json).unwrap();
            assert_eq!(back, sort);
        }
    }

    #[test]
    fn sort_by_serde_pascal_case() {
        assert_eq!(
            serde_json::to_string(&SortBy::EntryDate).unwrap(),
            "\"EntryDate\""
        );
        assert_eq!(
            serde_json::to_string(&SortBy::EntryNumber).unwrap(),
            "\"EntryNumber\""
        );
    }

    #[test]
    fn sort_by_rejects_snake_case_deserialization() {
        // Défense contre les tentatives d'injection via query params.
        // `entry_date` (snake_case) n'est PAS un variant valide.
        let result: Result<SortBy, _> = serde_json::from_str("\"entry_date\"");
        assert!(result.is_err());
    }

    #[test]
    fn sort_by_rejects_sql_injection_attempt() {
        let malicious = "\"entry_date; DROP TABLE journal_entries--\"";
        let result: Result<SortBy, _> = serde_json::from_str(malicious);
        assert!(
            result.is_err(),
            "l'enum doit rejeter toute valeur hors whitelist"
        );
    }
}
