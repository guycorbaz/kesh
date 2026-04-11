//! Validation de l'équilibre des écritures en partie double.
//!
//! Garde-fou métier indépendant de la base : toutes les règles d'intégrité
//! sont appliquées ici avant toute tentative de persistance. La couche
//! `kesh-db` applique une seconde vérification via des contraintes DB et
//! un re-calcul post-INSERT, conformément à la stratégie "defense in
//! depth" décrite dans l'architecture (ARCH-28).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::errors::CoreError;
use crate::types::Money;

/// Journal comptable dans lequel l'écriture est saisie.
///
/// **Note architecture** : cet enum est défini ici (kesh-core) comme
/// version pure sans dépendance SQLx. Un enum miroir existe dans
/// `kesh-db/entities/journal_entry.rs` avec les implémentations
/// `sqlx::Type`/`Encode`/`Decode`, et les conversions `From`/`Into`
/// vivent côté kesh-db pour respecter l'orphan rule Rust et la règle
/// ARCH-1 (kesh-core sans I/O). Si un variant est ajouté ici, il DOIT
/// être ajouté dans kesh-db ET dans la contrainte DB `CHECK BINARY`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Journal {
    /// Achats (facturation fournisseurs).
    Achats,
    /// Ventes (facturation clients).
    Ventes,
    /// Banque (mouvements bancaires).
    Banque,
    /// Caisse (espèces).
    Caisse,
    /// Opérations diverses (écritures de régularisation, clôture, etc.).
    OD,
}

impl Journal {
    /// Retourne la représentation textuelle stockée en base (PascalCase).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Achats => "Achats",
            Self::Ventes => "Ventes",
            Self::Banque => "Banque",
            Self::Caisse => "Caisse",
            Self::OD => "OD",
        }
    }
}

impl std::str::FromStr for Journal {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Achats" => Ok(Self::Achats),
            "Ventes" => Ok(Self::Ventes),
            "Banque" => Ok(Self::Banque),
            "Caisse" => Ok(Self::Caisse),
            "OD" => Ok(Self::OD),
            other => Err(format!("Journal inconnu : {other}")),
        }
    }
}

/// Ligne d'écriture en cours de saisie (brouillon, avant validation).
///
/// Exactement un des deux montants (`debit` ou `credit`) doit être
/// strictement positif, l'autre strictement zéro — vérifié par
/// [`validate`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntryLineDraft {
    /// Identifiant du compte du plan comptable (clé étrangère DB).
    pub account_id: i64,
    /// Montant au débit. `Money::zero()` si cette ligne est une ligne
    /// au crédit.
    pub debit: Money,
    /// Montant au crédit. `Money::zero()` si cette ligne est une ligne
    /// au débit.
    pub credit: Money,
}

/// Brouillon d'écriture avant validation.
///
/// Un brouillon peut être invalide — utiliser [`validate`] pour obtenir
/// une [`BalancedEntry`] garantie équilibrée par construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntryDraft {
    /// Date de l'écriture (date calendaire suisse, pas d'heure).
    pub date: NaiveDate,
    /// Journal cible.
    pub journal: Journal,
    /// Libellé de l'écriture (obligatoire, non vide après trim).
    pub description: String,
    /// Lignes de l'écriture (au moins 2).
    pub lines: Vec<JournalEntryLineDraft>,
}

/// Écriture validée et équilibrée.
///
/// Ce type ne peut être construit que via [`validate`], garantissant
/// par construction que :
///
/// - Le libellé est non vide.
/// - Il y a au moins 2 lignes.
/// - Aucune ligne n'a de montant négatif.
/// - Chaque ligne a exactement un des deux montants strictement positif.
/// - Le total des débits égale le total des crédits.
/// - Le total est strictement positif.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BalancedEntry {
    draft: JournalEntryDraft,
    total: Money,
}

impl BalancedEntry {
    /// Retourne le brouillon sous-jacent (utilisé par la couche persistance).
    pub fn draft(&self) -> &JournalEntryDraft {
        &self.draft
    }

    /// Consomme la `BalancedEntry` et retourne le brouillon.
    pub fn into_draft(self) -> JournalEntryDraft {
        self.draft
    }

    /// Total des débits (= total des crédits, par garantie).
    pub fn total(&self) -> Money {
        self.total
    }
}

/// Valide un brouillon d'écriture et retourne une [`BalancedEntry`] si
/// toutes les règles métier passent.
///
/// # Ordre de validation (important pour les messages d'erreur)
///
/// 1. Libellé non vide (après trim)
/// 2. Au moins 2 lignes
/// 3. Pas de montant négatif
/// 4. Exclusivité débit/crédit par ligne
/// 5. Total des débits = total des crédits
/// 6. Total strictement positif
pub fn validate(draft: JournalEntryDraft) -> Result<BalancedEntry, CoreError> {
    if draft.description.trim().is_empty() {
        return Err(CoreError::EntryDescriptionEmpty);
    }

    if draft.lines.len() < 2 {
        return Err(CoreError::EntryNeedsTwoLines);
    }

    let zero = Money::zero();

    for line in &draft.lines {
        if line.debit.is_negative() || line.credit.is_negative() {
            return Err(CoreError::EntryNegativeAmount);
        }

        let debit_positive = line.debit > zero;
        let credit_positive = line.credit > zero;

        // Exclusivité stricte : (debit > 0) XOR (credit > 0).
        if debit_positive == credit_positive {
            return Err(CoreError::EntryLineDebitCreditExclusive);
        }
    }

    let total_debit: Money = draft.lines.iter().map(|l| l.debit).sum();
    let total_credit: Money = draft.lines.iter().map(|l| l.credit).sum();

    if total_debit != total_credit {
        return Err(CoreError::EntryUnbalanced {
            debit: total_debit,
            credit: total_credit,
        });
    }

    // Garde-fou défensif : `EntryLineDebitCreditExclusive` garantit déjà
    // que chaque ligne a un montant strictement positif (débit XOR crédit),
    // donc `total_debit > 0` est implicite. Cette branche n'est donc
    // **atteignable** depuis `validate()` que si la règle d'exclusivité
    // est un jour relâchée — elle reste présente comme protection
    // défensive. `debug_assert!` vérifie l'invariant en tests.
    debug_assert!(
        total_debit > zero,
        "invariant violé : total_debit devrait être > 0 après la règle d'exclusivité"
    );
    if total_debit == zero {
        return Err(CoreError::EntryZeroTotal);
    }

    Ok(BalancedEntry {
        draft,
        total: total_debit,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 4, 10).unwrap()
    }

    fn line(account_id: i64, debit: Money, credit: Money) -> JournalEntryLineDraft {
        JournalEntryLineDraft {
            account_id,
            debit,
            credit,
        }
    }

    fn draft(lines: Vec<JournalEntryLineDraft>) -> JournalEntryDraft {
        JournalEntryDraft {
            date: date(),
            journal: Journal::Banque,
            description: "Test écriture".to_string(),
            lines,
        }
    }

    #[test]
    fn valid_two_line_entry() {
        let d = draft(vec![
            line(1, Money::new(dec!(100)), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(100))),
        ]);
        let balanced = validate(d).unwrap();
        assert_eq!(balanced.total(), Money::new(dec!(100)));
    }

    #[test]
    fn valid_multi_line_entry() {
        // 50 + 50 = 100 débit ; 100 crédit.
        let d = draft(vec![
            line(1, Money::new(dec!(50)), Money::zero()),
            line(2, Money::new(dec!(50)), Money::zero()),
            line(3, Money::zero(), Money::new(dec!(100))),
        ]);
        let balanced = validate(d).unwrap();
        assert_eq!(balanced.total(), Money::new(dec!(100)));
    }

    #[test]
    fn valid_exact_decimals() {
        // 19.95 + 0.05 = 20.00 — garantie exactitude rust_decimal.
        let d = draft(vec![
            line(1, Money::new(dec!(19.95)), Money::zero()),
            line(2, Money::new(dec!(0.05)), Money::zero()),
            line(3, Money::zero(), Money::new(dec!(20.00))),
        ]);
        let balanced = validate(d).unwrap();
        assert_eq!(balanced.total(), Money::new(dec!(20.00)));
    }

    #[test]
    fn valid_four_decimal_places() {
        // AC#3b / dette i18n : rust_decimal accepte jusqu'à 4 décimales
        // sans arrondi applicatif en saisie manuelle.
        let d = draft(vec![
            line(1, Money::new(dec!(10.1234)), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(10.1234))),
        ]);
        validate(d).unwrap();
    }

    #[test]
    fn rejects_zero_lines() {
        let d = draft(vec![]);
        assert!(matches!(validate(d), Err(CoreError::EntryNeedsTwoLines)));
    }

    #[test]
    fn rejects_single_line() {
        let d = draft(vec![line(1, Money::new(dec!(100)), Money::zero())]);
        assert!(matches!(validate(d), Err(CoreError::EntryNeedsTwoLines)));
    }

    #[test]
    fn rejects_unbalanced() {
        let d = draft(vec![
            line(1, Money::new(dec!(100)), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(80))),
        ]);
        match validate(d) {
            Err(CoreError::EntryUnbalanced { debit, credit }) => {
                assert_eq!(debit, Money::new(dec!(100)));
                assert_eq!(credit, Money::new(dec!(80)));
            }
            other => panic!("expected EntryUnbalanced, got {:?}", other),
        }
    }

    #[test]
    fn rejects_debit_and_credit_on_same_line() {
        let d = draft(vec![
            line(1, Money::new(dec!(100)), Money::new(dec!(50))),
            line(2, Money::zero(), Money::new(dec!(50))),
        ]);
        assert!(matches!(
            validate(d),
            Err(CoreError::EntryLineDebitCreditExclusive)
        ));
    }

    #[test]
    fn rejects_line_with_both_zero() {
        let d = draft(vec![
            line(1, Money::zero(), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(100))),
        ]);
        assert!(matches!(
            validate(d),
            Err(CoreError::EntryLineDebitCreditExclusive)
        ));
    }

    #[test]
    fn rejects_negative_amount() {
        let d = draft(vec![
            line(1, Money::new(dec!(-100)), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(-100))),
        ]);
        assert!(matches!(validate(d), Err(CoreError::EntryNegativeAmount)));
    }

    #[test]
    fn rejects_empty_description() {
        let mut d = draft(vec![
            line(1, Money::new(dec!(100)), Money::zero()),
            line(2, Money::zero(), Money::new(dec!(100))),
        ]);
        d.description = "   ".to_string();
        assert!(matches!(validate(d), Err(CoreError::EntryDescriptionEmpty)));
    }

    #[test]
    fn rejects_zero_total() {
        // Impossible par construction de lignes valides (chaque ligne
        // doit avoir un montant > 0) — ce test vérifie le garde-fou.
        // On ne peut pas l'atteindre via `validate()` avec des lignes
        // conformes à `EntryLineDebitCreditExclusive`. Ce test est
        // défensif : si la règle de ligne exclusive est un jour
        // relâchée, le garde-fou total > 0 reste actif.
        //
        // Note : le test direct du variant `EntryZeroTotal` n'est donc
        // pas atteignable depuis `validate()` dans l'état actuel des
        // règles — on teste à la place que le variant existe et est
        // distinct.
        let err = CoreError::EntryZeroTotal;
        assert_eq!(err.error_code(), "ENTRY_ZERO_TOTAL");
    }

    #[test]
    fn journal_as_str_roundtrip() {
        use std::str::FromStr;
        for j in [
            Journal::Achats,
            Journal::Ventes,
            Journal::Banque,
            Journal::Caisse,
            Journal::OD,
        ] {
            assert_eq!(Journal::from_str(j.as_str()).unwrap(), j);
        }
    }

    #[test]
    fn journal_from_str_unknown() {
        use std::str::FromStr;
        assert!(Journal::from_str("Inconnu").is_err());
    }

    #[test]
    fn journal_serde_roundtrip() {
        let j = Journal::Achats;
        let json = serde_json::to_string(&j).unwrap();
        assert_eq!(json, "\"Achats\"");
        let back: Journal = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Journal::Achats);
    }
}
