//! Story 8-5b FR47 — moteur de règles d'affectation.
//!
//! Helpers purs (zéro I/O) :
//!
//! - [`rule_matches`] : teste si une [`ReconciliationRule`] match une
//!   [`BankTransaction`] selon son `match_type`. Sign-agnostic — une
//!   rule s'applique aux crédits ET débits (Pass 1 P-H7 ECH-05 : NE
//!   PAS hériter du sign filter 8-4 invoice).
//! - [`first_matching_rule`] : applique une liste ordonnée
//!   `priority ASC, id ASC` (cf. [`kesh_db::repositories::reconciliation_rules::find_active_for_company`])
//!   et retourne la première rule qui match ET dont
//!   `counterparty_account_id` est dans `active_account_ids` (handler
//!   pré-load les comptes actifs en un seul SELECT — cf.
//!   §rule-application R5 + Q5).
//!
//! Normalisation :
//!
//! - [`fn@normalize`] — `trim().to_lowercase()` pour
//!   `CounterpartyContains` / `CounterpartyExact` / `ReferenceContains`.
//! - [`fn@normalize_iban`] inline dans la branche `IbanExact` —
//!   `uppercase + strip whitespace`. NE PAS appliquer
//!   [`fn@normalize`] à un IBAN (Pass 2 Q2 BH-F7) : les IBAN
//!   stockés en DB sont canoniques (uppercase) par construction
//!   handler-side (cf. `kesh_qrbill::validation::normalize_iban`
//!   appliqué au `post_create` rule), donc un `to_lowercase()` ici
//!   casserait l'égalité.

use kesh_db::entities::{BankTransaction, ReconciliationMatchType, ReconciliationRule};
use std::collections::HashSet;

/// Teste si la rule match la transaction. Pure (zéro I/O).
///
/// `match_type` détermine quel champ de `tx` est comparé :
///
/// - `CounterpartyContains` / `CounterpartyExact` →
///   [`BankTransaction::counterparty_name`] (case-insensitive,
///   normalisé via [`fn@normalize`]).
/// - `ReferenceContains` → chaîne de fallback `tx.reference ||
///   tx.end_to_end_id || tx.transaction_id` (Pass 1 P-M BH-F15 : les
///   3 champs sémantiquement « référence » côté banque, premier
///   non-null gagne).
/// - `IbanExact` → [`BankTransaction::counterparty_iban`] avec
///   normalisation canonique (uppercase + strip whitespace) des deux
///   côtés (défense-en-profondeur — l'IBAN DB est censé être canonique
///   par construction au `post_create` rule).
pub fn rule_matches(rule: &ReconciliationRule, tx: &BankTransaction) -> bool {
    use ReconciliationMatchType::*;
    match rule.match_type {
        CounterpartyContains => match_contains(tx.counterparty_name.as_deref(), &rule.match_value),
        CounterpartyExact => match_exact(tx.counterparty_name.as_deref(), &rule.match_value),
        ReferenceContains => {
            // Pass 1 code review LOW EC7 fix : filter empty strings dans
            // le fallback chain. CAMT.053 parser peut produire
            // `reference = Some("")` (XML element present mais empty) —
            // sémantiquement absent, doit fallback sur end_to_end_id /
            // transaction_id.
            let reference = tx
                .reference
                .as_deref()
                .filter(|s| !s.is_empty())
                .or(tx.end_to_end_id.as_deref().filter(|s| !s.is_empty()))
                .or(tx.transaction_id.as_deref().filter(|s| !s.is_empty()));
            match_contains(reference, &rule.match_value)
        }
        IbanExact => match tx.counterparty_iban.as_deref() {
            Some(tx_iban) => normalize_iban(tx_iban) == normalize_iban(&rule.match_value),
            None => false,
        },
    }
}

/// Applique la liste ordonnée de rules à une tx et retourne la première
/// qui match **ET** dont `counterparty_account_id` apparaît dans
/// `active_account_ids` (i.e. le compte de contrepartie n'est pas
/// archivé — AC #116).
///
/// L'ordre `priority ASC, id ASC` est imposé par le SQL de
/// [`kesh_db::repositories::reconciliation_rules::find_active_for_company`]
/// — ce helper ne re-sort pas (le caller a la responsabilité de fournir
/// une slice déjà ordonnée).
///
/// Filtre `r.active` en plus du tri DB pour défense-en-profondeur si
/// jamais un caller injectait des rules listées via
/// `list_by_company_paginated(active_filter=None)`.
pub fn first_matching_rule<'a>(
    rules: &'a [ReconciliationRule],
    tx: &BankTransaction,
    active_account_ids: &HashSet<i64>,
) -> Option<&'a ReconciliationRule> {
    rules
        .iter()
        .filter(|r| r.active)
        .filter(|r| active_account_ids.contains(&r.counterparty_account_id))
        .find(|r| rule_matches(r, tx))
}

/// Normalisation `trim + to_lowercase` pour text matching
/// (`CounterpartyContains` / `CounterpartyExact` / `ReferenceContains`).
///
/// **Pass 2 Q2 BH-F7** : NE PAS utiliser pour `IbanExact` — qui exige
/// `uppercase + strip whitespace` (cf. [`normalize_iban`]).
fn normalize(s: &str) -> String {
    s.trim().to_lowercase()
}

/// Normalisation IBAN canonique (Pass 1 P-H4 ECH-03 + BH-F29) :
/// `uppercase + strip whitespace`. Identique à
/// `kesh_qrbill::validation::normalize_iban` — réimplémenté inline
/// plutôt que de dépendre de `kesh-qrbill` pour éviter le coupling
/// transverse (le matching IBAN n'a aucune relation sémantique avec
/// QR-Bill génération).
fn normalize_iban(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_uppercase()
}

fn match_contains(haystack: Option<&str>, needle: &str) -> bool {
    let Some(h) = haystack else {
        return false;
    };
    normalize(h).contains(&normalize(needle))
}

fn match_exact(haystack: Option<&str>, needle: &str) -> bool {
    let Some(h) = haystack else {
        return false;
    };
    normalize(h) == normalize(needle)
}

// ---------------------------------------------------------------------------
// Tests unit (Story 8-5b T3.4 — ≥ 8 tests, Pass 2 Q10 ajoute le #8).
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use kesh_db::entities::bank_transaction::BankTransactionStatus;
    use rust_decimal::Decimal;
    use std::collections::HashSet;
    use std::str::FromStr;

    fn make_tx(
        counterparty_name: Option<&str>,
        counterparty_iban: Option<&str>,
        reference: Option<&str>,
        end_to_end_id: Option<&str>,
        transaction_id: Option<&str>,
    ) -> BankTransaction {
        BankTransaction {
            id: 1,
            company_id: 1,
            import_id: 1,
            bank_account_id: 1,
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 13).unwrap(),
            value_date: None,
            amount: Decimal::from_str("100.00").unwrap(),
            currency: "CHF".to_string(),
            reference: reference.map(String::from),
            details: String::new(),
            end_to_end_id: end_to_end_id.map(String::from),
            transaction_id: transaction_id.map(String::from),
            counterparty_iban: counterparty_iban.map(String::from),
            counterparty_name: counterparty_name.map(String::from),
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: NaiveDate::from_ymd_opt(2026, 5, 13)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2026, 5, 13)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    fn make_rule(
        id: i64,
        match_type: ReconciliationMatchType,
        match_value: &str,
        counterparty_account_id: i64,
        priority: i32,
        active: bool,
    ) -> ReconciliationRule {
        ReconciliationRule {
            id,
            company_id: 1,
            label: format!("Rule {id}"),
            match_type,
            match_value: match_value.to_string(),
            counterparty_account_id,
            priority,
            active,
            applied_count: 0,
            last_applied_at: None,
            version: 1,
            created_at: NaiveDate::from_ymd_opt(2026, 5, 13)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
            updated_at: NaiveDate::from_ymd_opt(2026, 5, 13)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    // Test 1 — AC #113 CounterpartyContains insensible à la casse.
    #[test]
    fn rule_matches_counterparty_contains() {
        let tx = make_tx(Some("Swisscom (Schweiz) AG"), None, None, None, None);
        let rule = make_rule(
            1,
            ReconciliationMatchType::CounterpartyContains,
            "swisscom",
            1,
            100,
            true,
        );
        assert!(rule_matches(&rule, &tx));

        // Pas de match : aucune occurrence.
        let rule2 = make_rule(
            2,
            ReconciliationMatchType::CounterpartyContains,
            "Sunrise",
            1,
            100,
            true,
        );
        assert!(!rule_matches(&rule2, &tx));

        // tx.counterparty_name = None → pas de match.
        let tx_none = make_tx(None, None, None, None, None);
        assert!(!rule_matches(&rule, &tx_none));
    }

    // Test 2 — AC #113 CounterpartyExact + normalize (case + trim).
    #[test]
    fn rule_matches_counterparty_exact_normalize_case() {
        let tx = make_tx(Some("  SWISSCOM SA  "), None, None, None, None);
        let rule = make_rule(
            1,
            ReconciliationMatchType::CounterpartyExact,
            "swisscom sa",
            1,
            100,
            true,
        );
        // trim + lowercase des 2 côtés → égal.
        assert!(rule_matches(&rule, &tx));

        // Différent : "Swisscom" exact ne match pas "Swisscom SA".
        let rule2 = make_rule(
            2,
            ReconciliationMatchType::CounterpartyExact,
            "Swisscom",
            1,
            100,
            true,
        );
        assert!(!rule_matches(&rule2, &tx));
    }

    // Test 3 — AC #113 IbanExact normalisé (uppercase + strip whitespace).
    #[test]
    fn rule_matches_iban_exact() {
        let tx = make_tx(None, Some("CH9300762011623852957"), None, None, None);

        // Match exact.
        let rule_canon = make_rule(
            1,
            ReconciliationMatchType::IbanExact,
            "CH9300762011623852957",
            1,
            100,
            true,
        );
        assert!(rule_matches(&rule_canon, &tx));

        // Match avec whitespace dans le rule.match_value → normalisé.
        let rule_spaced = make_rule(
            2,
            ReconciliationMatchType::IbanExact,
            "CH93 0076 2011 6238 5295 7",
            1,
            100,
            true,
        );
        assert!(rule_matches(&rule_spaced, &tx));

        // Match avec lowercase dans le rule → normalisé en uppercase.
        let rule_lower = make_rule(
            3,
            ReconciliationMatchType::IbanExact,
            "ch9300762011623852957",
            1,
            100,
            true,
        );
        assert!(rule_matches(&rule_lower, &tx));

        // Pas de match : IBAN différent.
        let rule_other = make_rule(
            4,
            ReconciliationMatchType::IbanExact,
            "CH4912300000087654321",
            1,
            100,
            true,
        );
        assert!(!rule_matches(&rule_other, &tx));

        // tx.counterparty_iban = None → pas de match.
        let tx_none = make_tx(None, None, None, None, None);
        assert!(!rule_matches(&rule_canon, &tx_none));
    }

    // Test 4 — AC #113 ReferenceContains avec fallback chain.
    #[test]
    fn rule_matches_reference_fallback_chain() {
        let rule = make_rule(
            1,
            ReconciliationMatchType::ReferenceContains,
            "INV-",
            1,
            100,
            true,
        );

        // tx.reference présent → utilisé en priorité.
        let tx_ref = make_tx(None, None, Some("INV-2026-001"), Some("E2E"), Some("TXN"));
        assert!(rule_matches(&rule, &tx_ref));

        // tx.reference absent, end_to_end_id présent → fallback (2).
        let tx_e2e = make_tx(None, None, None, Some("INV-2026-002"), Some("TXN"));
        assert!(rule_matches(&rule, &tx_e2e));

        // Les 2 premiers absents, transaction_id présent → fallback (3).
        let tx_txn = make_tx(None, None, None, None, Some("INV-2026-003"));
        assert!(rule_matches(&rule, &tx_txn));

        // Tous absents → pas de match.
        let tx_none = make_tx(None, None, None, None, None);
        assert!(!rule_matches(&rule, &tx_none));

        // Aucun ne contient le pattern → pas de match.
        let tx_no_match = make_tx(None, None, Some("PAYMENT-XYZ"), None, None);
        assert!(!rule_matches(&rule, &tx_no_match));
    }

    // Test 5 — AC #115 first_matching_rule respecte priority ASC.
    #[test]
    fn first_matching_rule_respects_priority_order() {
        let tx = make_tx(Some("Swisscom AG"), None, None, None, None);
        let rules = vec![
            // priority 100 (premier en ordre tri) → mais ne match pas.
            make_rule(
                1,
                ReconciliationMatchType::CounterpartyContains,
                "Sunrise",
                100,
                100,
                true,
            ),
            // priority 200 → ne match pas non plus.
            make_rule(
                2,
                ReconciliationMatchType::CounterpartyExact,
                "Salt",
                100,
                200,
                true,
            ),
            // priority 300 → match !
            make_rule(
                3,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                300,
                true,
            ),
            // priority 400 → match aussi mais ordre tardif.
            make_rule(
                4,
                ReconciliationMatchType::CounterpartyContains,
                "AG",
                100,
                400,
                true,
            ),
        ];
        let active: HashSet<i64> = [100].into_iter().collect();
        let matched = first_matching_rule(&rules, &tx, &active);
        assert_eq!(
            matched.map(|r| r.id),
            Some(3),
            "must pick first matching by priority order"
        );
    }

    // Test 6 — AC #116 first_matching_rule skip rules sur compte archivé.
    #[test]
    fn first_matching_rule_skips_inactive_account() {
        let tx = make_tx(Some("Swisscom"), None, None, None, None);
        let rules = vec![
            // priority 100 → matche, mais counterparty_account_id=999 N'EST PAS dans active_account_ids.
            make_rule(
                1,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                999,
                100,
                true,
            ),
            // priority 200 → matche aussi, compte 100 actif.
            make_rule(
                2,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                200,
                true,
            ),
        ];
        let active: HashSet<i64> = [100].into_iter().collect(); // 999 archivé
        let matched = first_matching_rule(&rules, &tx, &active);
        assert_eq!(
            matched.map(|r| r.id),
            Some(2),
            "must skip rule with archived account (999) and pick rule 2"
        );
    }

    // Test 7 — AC #117 first_matching_rule skip rules archivées.
    #[test]
    fn first_matching_rule_skips_inactive_rule() {
        let tx = make_tx(Some("Swisscom"), None, None, None, None);
        let rules = vec![
            // priority 100 → matche mais active=false → skip.
            make_rule(
                1,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                100,
                false,
            ),
            // priority 200 → matche, active=true.
            make_rule(
                2,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                200,
                true,
            ),
        ];
        let active: HashSet<i64> = [100].into_iter().collect();
        let matched = first_matching_rule(&rules, &tx, &active);
        assert_eq!(matched.map(|r| r.id), Some(2));
    }

    // Test 8 — NEW Pass 2 Q10 — AC #115b id ASC tiebreaker à priority égale.
    #[test]
    fn first_matching_rule_respects_id_tiebreaker_on_equal_priority() {
        let tx = make_tx(Some("Swisscom"), None, None, None, None);
        // Deux rules avec même priority=200, même match → caller injecte
        // l'ordre `priority ASC, id ASC` via `find_active_for_company`.
        // Le helper ne re-sort pas, il prend la première qui match.
        let rules = vec![
            // id=5 priority=200.
            make_rule(
                5,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                200,
                true,
            ),
            // id=42 priority=200 (vient après dans la slice ordonnée DB).
            make_rule(
                42,
                ReconciliationMatchType::CounterpartyContains,
                "Swisscom",
                100,
                200,
                true,
            ),
        ];
        let active: HashSet<i64> = [100].into_iter().collect();
        let matched = first_matching_rule(&rules, &tx, &active);
        assert_eq!(
            matched.map(|r| r.id),
            Some(5),
            "must take first by id at equal priority (caller pré-trie via SQL)"
        );
    }
}
