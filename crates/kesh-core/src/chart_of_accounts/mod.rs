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

/// Rôle métier explicite d'un compte (Story 14-3a).
///
/// Le rôle dit **à quoi sert** un compte, indépendamment de son numéro : le plan
/// comptable suisse est un usage, pas une obligation légale, et l'utilisateur
/// peut renuméroter ses comptes. Aucune logique applicative ne doit déduire un
/// rôle d'un numéro.
///
/// # Duplication assumée avec `kesh-db`
///
/// Cet enum existe **en double** : ici (`Deserialize` seul, pour les plans JSON)
/// et dans `kesh_db::entities::account::AccountRole` (avec les impls `sqlx`).
/// Ce n'est **pas** une négligence : `sqlx` n'est une dépendance que de
/// `kesh-db`, et l'orphan rule Rust interdit d'implémenter `Type<MySql>` sur un
/// type de `kesh-core` depuis `kesh-db` (`error[E0117]`) ; l'inverse créerait un
/// cycle Cargo. `AccountType` est dupliqué pour exactement la même raison.
/// Le garde-fou est le test de cohérence, pas la fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum AccountRole {
    /// Créances clients (débiteurs).
    Receivable,
    /// Produit par défaut de facturation.
    DefaultRevenue,
    /// Dettes fournisseurs (créanciers).
    Payable,
    /// Impôt préalable (TVA récupérable).
    VatRecoverable,
    /// TVA due.
    VatPayable,
    /// Décompte TVA.
    VatSettlement,
    /// Capital (social / de l'exploitant / de l'association).
    EquityCapital,
    /// Autres fonds propres : réserves, fonds affectés ou libres, prélèvements
    /// et apports privés. Intitulé volontairement **neutre** — les numéros
    /// 2850/2860 désignent des « fonds » en association mais des mouvements de
    /// capital de l'exploitant chez l'indépendant. La sémantique fine est
    /// portée par le nom du compte, pas par le rôle.
    EquityOther,
    /// Bénéfice / perte reporté.
    RetainedEarnings,
    /// Résultat de l'exercice.
    CurrentYearResult,
}

impl AccountRole {
    /// Les 10 rôles, dans l'ordre de déclaration (source du test de cohérence).
    pub const ALL: [AccountRole; 10] = [
        Self::Receivable,
        Self::DefaultRevenue,
        Self::Payable,
        Self::VatRecoverable,
        Self::VatPayable,
        Self::VatSettlement,
        Self::EquityCapital,
        Self::EquityOther,
        Self::RetainedEarnings,
        Self::CurrentYearResult,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Receivable => "Receivable",
            Self::DefaultRevenue => "DefaultRevenue",
            Self::Payable => "Payable",
            Self::VatRecoverable => "VatRecoverable",
            Self::VatPayable => "VatPayable",
            Self::VatSettlement => "VatSettlement",
            Self::EquityCapital => "EquityCapital",
            Self::EquityOther => "EquityOther",
            Self::RetainedEarnings => "RetainedEarnings",
            Self::CurrentYearResult => "CurrentYearResult",
        }
    }

    /// `true` si au plus **un compte actif** par société peut porter ce rôle.
    ///
    /// # ⚠️ Liste synchronisée à TROIS endroits
    ///
    /// 1. le `CASE WHEN active AND role IN (…)` de la colonne générée
    ///    `accounts.singleton_role` (migration `20260722000001_accounts_role_postable.sql`) ;
    /// 2. `kesh_db::entities::account::AccountRole::is_singleton()` ;
    /// 3. **ici** — nécessaire car [`validate_chart`] est privé à ce crate et
    ///    `kesh-core` ne peut pas atteindre `kesh-db`.
    ///
    /// Toute modification doit toucher les trois. Le test
    /// `singleton_list_matches_sql_generation_expression` (crate `kesh-db`)
    /// compare la liste Rust à l'expression SQL réellement en base.
    pub fn is_singleton(&self) -> bool {
        match self {
            Self::Receivable
            | Self::DefaultRevenue
            | Self::Payable
            | Self::VatRecoverable
            | Self::VatPayable
            | Self::VatSettlement
            | Self::RetainedEarnings
            | Self::CurrentYearResult => true,
            // Multi-valués : une société a couramment plusieurs comptes de
            // capital ou de fonds propres divers (2800 + 2850 + 2860).
            Self::EquityCapital | Self::EquityOther => false,
        }
    }

    /// `true` si `account_type` (PascalCase : `"Asset"`, `"Liability"`,
    /// `"Revenue"`, `"Expense"`) peut porter ce rôle.
    ///
    /// # Contrainte volontairement minimale — code review 14-3a, décision D1
    ///
    /// On ne valide que la frontière **bilan / résultat**, jamais le côté du
    /// bilan :
    ///
    /// - `DefaultRevenue` est le seul rôle d'un compte de **résultat**, et c'est
    ///   forcément un produit ;
    /// - les 9 autres sont des postes de **bilan** — `Asset` ou `Liability`, au
    ///   choix de l'utilisateur.
    ///
    /// Refuser un côté précis serait retomber dans le travers que la Story
    /// 14-3a corrige : encoder *une* lecture du plan suisse. Contre-exemple
    /// concret — l'impôt préalable (`VatRecoverable`, 1171) est un **actif**
    /// alors que la TVA due (`VatPayable`, 2200) est un **passif** ; une table
    /// « rôles TVA → passif » serait fausse dès le premier compte. Ce qui reste
    /// interdit est ce qui est indiscutablement faux : un rôle de bilan posé sur
    /// un compte de charge ou de produit, qui ferait générer à la Story 14-3b
    /// des écritures du mauvais côté.
    ///
    /// Un `account_type` inconnu retourne `false` — mieux vaut refuser une
    /// valeur qu'on ne sait pas juger que l'accepter par défaut.
    pub fn accepts_account_type(&self, account_type: &str) -> bool {
        match self {
            Self::DefaultRevenue => account_type == "Revenue",
            _ => matches!(account_type, "Asset" | "Liability"),
        }
    }
}

impl std::str::FromStr for AccountRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .iter()
            .find(|r| r.as_str() == s)
            .copied()
            .ok_or_else(|| format!("AccountRole inconnu : {s}"))
    }
}

/// Entrée d'un plan comptable JSON.
///
/// Les noms sont multilingues (clés : `"fr"`, `"de"`, `"it"`, `"en"`).
/// `parent_number` référence le numéro du compte parent dans la hiérarchie.
/// `role` est optionnel : un plan sans annotation reste valide (non-breaking).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChartEntry {
    pub number: String,
    pub name: HashMap<String, String>,
    #[serde(rename = "type")]
    pub account_type: AccountType,
    pub parent_number: Option<String>,
    #[serde(default)]
    pub role: Option<AccountRole>,
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
        if let Some(ref parent) = entry.parent_number
            && !numbers.contains(parent)
        {
            return Err(CoreError::InvalidChart(format!(
                "compte {} référence un parent inexistant : {}",
                entry.number, parent
            )));
        }
    }

    // Story 14-3a : un rôle singleton ne peut être porté que par un compte par
    // société — le détecter dès le JSON évite qu'une faute de frappe dans un
    // plan livré ne se transforme en violation de `uq_accounts_company_singleton_role`
    // au moment du seed, où le diagnostic serait bien plus obscur.
    let mut singletons = HashSet::new();
    for entry in entries {
        if let Some(role) = entry.role
            && role.is_singleton()
            && !singletons.insert(role)
        {
            return Err(CoreError::InvalidChart(format!(
                "rôle singleton dupliqué : {} (compte {})",
                role.as_str(),
                entry.number
            )));
        }
    }

    // Code review 14-3a (D1) : un rôle posé sur un type de compte incompatible
    // — p. ex. `Payable` sur une charge — passerait le seed sans bruit et ferait
    // générer à la Story 14-3b des écritures du mauvais côté du bilan.
    for entry in entries {
        if let Some(role) = entry.role
            && !role.accepts_account_type(entry.account_type.as_str())
        {
            return Err(CoreError::InvalidChart(format!(
                "compte {} : le rôle {} est incompatible avec le type {}",
                entry.number,
                role.as_str(),
                entry.account_type.as_str()
            )));
        }
    }

    Ok(())
}

/// `true` si l'entrée doit être créée non-postable.
///
/// Deux causes, toutes deux **chart-agnostiques** (aucun numéro codé en dur) :
/// 1. l'entrée est le parent d'une autre entrée du plan — c'est un compte titre
///    ou de regroupement, on ne poste pas dessus ;
/// 2. l'entrée porte le rôle [`AccountRole::CurrentYearResult`] — en modèle
///    « temps réel virtuel » (Story 14-1), l'application **calcule** le résultat
///    de l'exercice à chaque rendu ; y poster serait un double-comptage garanti.
///
/// [`AccountRole::RetainedEarnings`] reste postable : un utilisateur qui migre
/// depuis un autre logiciel doit pouvoir poser son report à nouveau d'ouverture.
///
/// Cette fonction est la **contrepartie exacte** des deux `UPDATE` de backfill
/// de la migration `20260722000001_accounts_role_postable.sql` ; l'invariant
/// « seed ≡ backfill » est vérifié par un test dédié dans `kesh-db`.
pub fn is_postable(entry: &ChartEntry, parent_numbers: &HashSet<&str>) -> bool {
    if parent_numbers.contains(entry.number.as_str()) {
        return false;
    }
    entry.role != Some(AccountRole::CurrentYearResult)
}

/// Collecte les numéros qui sont parents d'au moins une entrée du plan.
pub fn parent_numbers(entries: &[ChartEntry]) -> HashSet<&str> {
    entries
        .iter()
        .filter_map(|e| e.parent_number.as_deref())
        .collect()
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
            role: None,
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
            role: None,
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
            role: None,
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
                role: None,
            },
            ChartEntry {
                number: "1000".to_string(),
                name: HashMap::from([("fr".to_string(), "B".to_string())]),
                account_type: AccountType::Asset,
                parent_number: None,
                role: None,
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
            role: None,
        }];
        let err = validate_chart(&entries).unwrap_err();
        assert!(err.to_string().contains("parent inexistant"));
    }

    // =======================================================================
    // Story 14-3a — rôles dans les plans comptables
    // =======================================================================

    #[test]
    fn each_chart_carries_expected_roles() {
        for org in ["Pme", "Association", "Independant"] {
            let chart = load_chart(org).unwrap();
            let with_role: Vec<_> = chart.iter().filter(|e| e.role.is_some()).collect();
            assert!(
                with_role.len() >= 10,
                "{org} : au moins 10 comptes doivent porter un rôle, trouvé {}",
                with_role.len()
            );

            // Les 8 rôles singleton apparaissent EXACTEMENT une fois par plan.
            for role in AccountRole::ALL.iter().filter(|r| r.is_singleton()) {
                let n = chart.iter().filter(|e| e.role == Some(*role)).count();
                assert_eq!(
                    n,
                    1,
                    "{org} : le rôle singleton {} doit apparaître exactement une fois (trouvé {n})",
                    role.as_str()
                );
            }
        }
    }

    #[test]
    fn equity_other_differs_across_charts_but_stays_multi_valued() {
        // PME : 2900 seul. Association/Indépendant : 2850 + 2860. Ensembles
        // disjoints — d'où un backfill unique WHERE number IN ('2900','2850','2860').
        let pme: Vec<&str> = load_chart("Pme")
            .unwrap()
            .iter()
            .filter(|e| e.role == Some(AccountRole::EquityOther))
            .map(|e| e.number.clone().leak() as &str)
            .collect();
        assert_eq!(pme, vec!["2900"]);

        for org in ["Association", "Independant"] {
            let chart = load_chart(org).unwrap();
            let mut nums: Vec<&str> = chart
                .iter()
                .filter(|e| e.role == Some(AccountRole::EquityOther))
                .map(|e| e.number.as_str())
                .collect();
            nums.sort_unstable();
            assert_eq!(nums, vec!["2850", "2860"], "{org}");
        }
        assert!(!AccountRole::EquityOther.is_singleton());
    }

    #[test]
    fn validate_chart_rejects_duplicate_singleton_role() {
        let mk = |number: &str, role: Option<AccountRole>| ChartEntry {
            number: number.into(),
            name: HashMap::from([("fr".to_string(), format!("Compte {number}"))]),
            account_type: AccountType::Asset,
            parent_number: None,
            role,
        };

        // Deux comptes portant Receivable (singleton) → rejet.
        let dup = vec![
            mk("1100", Some(AccountRole::Receivable)),
            mk("1101", Some(AccountRole::Receivable)),
        ];
        let err = validate_chart(&dup).expect_err("doublon de rôle singleton attendu");
        assert!(
            format!("{err:?}").contains("Receivable"),
            "le message doit nommer le rôle : {err:?}"
        );

        // Deux comptes EquityOther (multi-valué) → accepté.
        let ok = vec![
            mk("2850", Some(AccountRole::EquityOther)),
            mk("2860", Some(AccountRole::EquityOther)),
        ];
        validate_chart(&ok).expect("EquityOther est multi-valué");
    }

    #[test]
    fn role_accepts_only_the_right_side_of_the_chart() {
        // `DefaultRevenue` est le seul rôle d'un compte de résultat.
        assert!(AccountRole::DefaultRevenue.accepts_account_type("Revenue"));
        assert!(!AccountRole::DefaultRevenue.accepts_account_type("Asset"));
        assert!(!AccountRole::DefaultRevenue.accepts_account_type("Liability"));

        // Les 9 autres sont des postes de bilan, des DEUX côtés : l'impôt
        // préalable est un actif, la TVA due un passif — d'où la contrainte
        // volontairement lâche (D1).
        assert!(AccountRole::VatRecoverable.accepts_account_type("Asset"));
        assert!(AccountRole::VatPayable.accepts_account_type("Liability"));
        assert!(AccountRole::Receivable.accepts_account_type("Asset"));
        assert!(AccountRole::EquityCapital.accepts_account_type("Liability"));

        // Mais jamais sur un compte de résultat.
        for role in AccountRole::ALL {
            if role == AccountRole::DefaultRevenue {
                continue;
            }
            assert!(
                !role.accepts_account_type("Expense"),
                "{} ne doit pas être posable sur une charge",
                role.as_str()
            );
            assert!(
                !role.accepts_account_type("Revenue"),
                "{} ne doit pas être posable sur un produit",
                role.as_str()
            );
        }

        // Type inconnu → refus (on ne juge pas ce qu'on ne connaît pas).
        assert!(!AccountRole::Receivable.accepts_account_type("Bogus"));
    }

    #[test]
    fn validate_chart_rejects_role_on_incompatible_type() {
        let entries = vec![ChartEntry {
            number: "6500".into(),
            name: HashMap::from([("fr".to_string(), "Frais admin".to_string())]),
            account_type: AccountType::Expense,
            parent_number: None,
            role: Some(AccountRole::Payable),
        }];
        let err = validate_chart(&entries).expect_err("Payable sur une charge doit être rejeté");
        let msg = format!("{err:?}");
        assert!(
            msg.contains("Payable") && msg.contains("Expense"),
            "le message doit nommer le rôle et le type : {msg}"
        );
    }

    #[test]
    fn shipped_charts_have_coherent_role_types() {
        // Les 3 plans livrés doivent passer la contrainte D1 — `load_chart`
        // appelle déjà `validate_chart`, ce test rend l'intention explicite.
        for org in ["Pme", "Association", "Independant"] {
            let chart = load_chart(org).unwrap_or_else(|e| panic!("plan {org} invalide : {e:?}"));
            for entry in &chart {
                if let Some(role) = entry.role {
                    assert!(
                        role.accepts_account_type(entry.account_type.as_str()),
                        "plan {org}, compte {} : rôle {} sur type {}",
                        entry.number,
                        role.as_str(),
                        entry.account_type.as_str()
                    );
                }
            }
        }
    }

    #[test]
    fn chart_without_role_field_still_parses() {
        // Non-breaking : un plan JSON sans `role` reste valide (#[serde(default)]).
        let json = r#"[{"number":"1","name":{"fr":"Actifs"},"type":"Asset","parentNumber":null}]"#;
        let entries: Vec<ChartEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(entries[0].role, None);
    }

    #[test]
    fn postability_is_chart_agnostic() {
        let chart = load_chart("Pme").unwrap();
        let parents = parent_numbers(&chart);

        let by_number = |n: &str| chart.iter().find(|e| e.number == n).unwrap();

        // Comptes titres (ont des enfants) → non-postables, sans référence au numéro.
        assert!(
            !is_postable(by_number("1"), &parents),
            "1 est un compte titre"
        );
        assert!(
            !is_postable(by_number("10"), &parents),
            "10 est un compte titre"
        );
        assert!(
            !is_postable(by_number("28"), &parents),
            "28 est un compte titre"
        );

        // Feuille ordinaire → postable.
        assert!(is_postable(by_number("1000"), &parents));

        // Le compte de résultat est non-postable (l'app le calcule, Story 14-1).
        assert!(!is_postable(by_number("2979"), &parents));

        // Le report à nouveau reste postable (soldes d'ouverture d'un migrant).
        assert!(is_postable(by_number("2970"), &parents));
    }

    #[test]
    fn account_role_from_str_round_trip() {
        for r in AccountRole::ALL {
            assert_eq!(r.as_str().parse::<AccountRole>().unwrap(), r);
        }
        assert!("Bogus".parse::<AccountRole>().is_err());
    }
}
