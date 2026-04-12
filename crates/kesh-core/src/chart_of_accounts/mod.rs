//! Plan comptable suisse — chargement et validation.
//!
//! Ce module charge les plans comptables standards suisses (PME, Association,
//! Indépendant) depuis des fichiers JSON embarqués dans le binaire via
//! `include_str!()`. Il fournit les types et la validation nécessaires
//! pour alimenter la table `accounts` en DB.

use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::errors::CoreError;

/// Type de compte comptable selon la norme suisse.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Revenue,
    Expense,
}

impl AccountType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Asset => "Asset",
            Self::Liability => "Liability",
            Self::Revenue => "Revenue",
            Self::Expense => "Expense",
        }
    }
}

/// Entrée d'un plan comptable JSON.
///
/// Les noms sont multilingues (clés : `"fr"`, `"de"`, `"it"`, `"en"`).
/// `parent_number` référence le numéro du compte parent dans la hiérarchie.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartEntry {
    pub number: String,
    pub name: HashMap<String, String>,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub parent_number: Option<String>,
}

/// Résout le nom d'un compte dans la langue demandée, avec fallback FR.
pub fn resolve_name(entry: &ChartEntry, lang: &str) -> String {
    let key = lang.to_lowercase();
    entry
        .name
        .get(&key)
        .or_else(|| entry.name.get("fr"))
        .cloned()
        .unwrap_or_else(|| entry.number.clone())
}

// Plans comptables embarqués dans le binaire.
const PME_JSON: &str = include_str!("../../assets/charts/pme.json");
const ASSOCIATION_JSON: &str = include_str!("../../assets/charts/association.json");
const INDEPENDANT_JSON: &str = include_str!("../../assets/charts/independant.json");

/// Charge et valide le plan comptable correspondant au type d'organisation.
///
/// `org_type` doit être `"Pme"`, `"Association"` ou `"Independant"` (insensible à la casse).
///
/// # Validation
/// - Tous les numéros de compte sont uniques.
/// - Chaque `parent_number` référence un numéro existant dans le plan.
pub fn load_chart(org_type: &str) -> Result<Vec<ChartEntry>, CoreError> {
    let json = match org_type.to_lowercase().as_str() {
        "pme" => PME_JSON,
        "association" => ASSOCIATION_JSON,
        "independant" => INDEPENDANT_JSON,
        _ => return Err(CoreError::UnknownChartType(org_type.to_string())),
    };

    let entries: Vec<ChartEntry> = serde_json::from_str(json)
        .map_err(|e| CoreError::InvalidChart(format!("JSON parse error: {e}")))?;

    validate_chart(&entries)?;

    Ok(entries)
}

/// Valide l'intégrité du plan comptable.
fn validate_chart(entries: &[ChartEntry]) -> Result<(), CoreError> {
    let mut numbers = HashSet::new();

    // Vérifier l'unicité des numéros
    for entry in entries {
        if !numbers.insert(&entry.number) {
            return Err(CoreError::InvalidChart(format!(
                "numéro de compte dupliqué : {}",
                entry.number
            )));
        }
    }

    // Vérifier que chaque parent_number référence un numéro existant
    for entry in entries {
        if let Some(ref parent) = entry.parent_number {
            if !numbers.contains(parent) {
                return Err(CoreError::InvalidChart(format!(
                    "compte {} référence un parent inexistant : {}",
                    entry.number, parent
                )));
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_pme_chart() {
        let chart = load_chart("Pme").unwrap();
        assert!(chart.len() >= 80, "PME chart should have ~80+ accounts");

        // Vérifier un compte connu
        let caisse = chart.iter().find(|e| e.number == "1000").unwrap();
        assert_eq!(caisse.name.get("fr").unwrap(), "Caisse");
        assert_eq!(caisse.name.get("de").unwrap(), "Kasse");
        assert_eq!(caisse.account_type, AccountType::Asset);
        assert_eq!(caisse.parent_number.as_deref(), Some("10"));
    }

    #[test]
    fn load_association_chart() {
        let chart = load_chart("Association").unwrap();
        assert!(
            chart.len() >= 50,
            "Association chart should have ~50+ accounts"
        );

        // Comptes spécifiques aux associations
        let cotisations = chart.iter().find(|e| e.number == "3000").unwrap();
        assert_eq!(
            cotisations.name.get("fr").unwrap(),
            "Cotisations des membres"
        );

        let fonds = chart.iter().find(|e| e.number == "2850").unwrap();
        assert_eq!(fonds.name.get("fr").unwrap(), "Fonds affectés");
    }

    #[test]
    fn load_independant_chart() {
        let chart = load_chart("Independant").unwrap();
        assert!(
            chart.len() >= 50,
            "Independant chart should have ~50+ accounts"
        );

        // Comptes spécifiques aux indépendants
        let capital = chart.iter().find(|e| e.number == "2800").unwrap();
        assert_eq!(capital.name.get("fr").unwrap(), "Capital de l'exploitant");

        let prelevements = chart.iter().find(|e| e.number == "2850").unwrap();
        assert_eq!(prelevements.name.get("fr").unwrap(), "Prélèvements privés");
    }

    #[test]
    fn load_chart_case_insensitive() {
        assert!(load_chart("pme").is_ok());
        assert!(load_chart("PME").is_ok());
        assert!(load_chart("Pme").is_ok());
    }

    #[test]
    fn load_chart_unknown_type() {
        let err = load_chart("unknown").unwrap_err();
        assert_eq!(err.error_code(), "UNKNOWN_CHART_TYPE");
    }

    #[test]
    fn all_charts_have_four_languages() {
        let langs = ["fr", "de", "it", "en"];
        for org_type in &["Pme", "Association", "Independant"] {
            let chart = load_chart(org_type).unwrap();
            for entry in &chart {
                for lang in &langs {
                    assert!(
                        entry.name.contains_key(*lang),
                        "Chart {org_type}, account {} missing language {lang}",
                        entry.number
                    );
                }
            }
        }
    }

    #[test]
    fn all_charts_have_unique_numbers() {
        for org_type in &["Pme", "Association", "Independant"] {
            let chart = load_chart(org_type).unwrap();
            let mut seen = HashSet::new();
            for entry in &chart {
                assert!(
                    seen.insert(&entry.number),
                    "Chart {org_type}: duplicate number {}",
                    entry.number
                );
            }
        }
    }

    #[test]
    fn all_charts_have_valid_parent_references() {
        for org_type in &["Pme", "Association", "Independant"] {
            let chart = load_chart(org_type).unwrap();
            let numbers: HashSet<_> = chart.iter().map(|e| &e.number).collect();
            for entry in &chart {
                if let Some(ref parent) = entry.parent_number {
                    assert!(
                        numbers.contains(parent),
                        "Chart {org_type}: account {} references missing parent {parent}",
                        entry.number
                    );
                }
            }
        }
    }

    #[test]
    fn all_charts_root_accounts_have_no_parent() {
        for org_type in &["Pme", "Association", "Independant"] {
            let chart = load_chart(org_type).unwrap();
            let roots: Vec<_> = chart.iter().filter(|e| e.number.len() == 1).collect();
            assert!(
                !roots.is_empty(),
                "Chart {org_type} should have root accounts"
            );
            for root in &roots {
                assert!(
                    root.parent_number.is_none(),
                    "Chart {org_type}: root account {} should have no parent",
                    root.number
                );
            }
        }
    }

    #[test]
    fn resolve_name_returns_requested_language() {
        let entry = ChartEntry {
            number: "1000".to_string(),
            name: HashMap::from([
                ("fr".to_string(), "Caisse".to_string()),
                ("de".to_string(), "Kasse".to_string()),
            ]),
            account_type: AccountType::Asset,
            parent_number: None,
        };
        assert_eq!(resolve_name(&entry, "de"), "Kasse");
        assert_eq!(resolve_name(&entry, "DE"), "Kasse");
    }

    #[test]
    fn resolve_name_falls_back_to_french() {
        let entry = ChartEntry {
            number: "1000".to_string(),
            name: HashMap::from([("fr".to_string(), "Caisse".to_string())]),
            account_type: AccountType::Asset,
            parent_number: None,
        };
        assert_eq!(resolve_name(&entry, "de"), "Caisse");
    }

    #[test]
    fn resolve_name_falls_back_to_number() {
        let entry = ChartEntry {
            number: "1000".to_string(),
            name: HashMap::new(),
            account_type: AccountType::Asset,
            parent_number: None,
        };
        assert_eq!(resolve_name(&entry, "fr"), "1000");
    }

    #[test]
    fn validate_chart_rejects_duplicate_numbers() {
        let entries = vec![
            ChartEntry {
                number: "1000".to_string(),
                name: HashMap::from([("fr".to_string(), "A".to_string())]),
                account_type: AccountType::Asset,
                parent_number: None,
            },
            ChartEntry {
                number: "1000".to_string(),
                name: HashMap::from([("fr".to_string(), "B".to_string())]),
                account_type: AccountType::Asset,
                parent_number: None,
            },
        ];
        let err = validate_chart(&entries).unwrap_err();
        assert!(err.to_string().contains("dupliqué"));
    }

    #[test]
    fn validate_chart_rejects_invalid_parent() {
        let entries = vec![ChartEntry {
            number: "1000".to_string(),
            name: HashMap::from([("fr".to_string(), "Caisse".to_string())]),
            account_type: AccountType::Asset,
            parent_number: Some("999".to_string()),
        }];
        let err = validate_chart(&entries).unwrap_err();
        assert!(err.to_string().contains("parent inexistant"));
    }
}
