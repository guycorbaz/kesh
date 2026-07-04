//! Story 8-5a-bis FR48 — éclatement de transaction agrégée (split).
//!
//! Helper public [`build_split_journal_entry`] qui construit une
//! `NewJournalEntry` à N+1 lignes équilibrées (1 ligne banque agrégée +
//! N lignes contrepartie) pour le flow `POST /api/v1/reconciliation/split`.
//!
//! **Pure** (zéro I/O) — facilite tests unitaires sans mock DB.
//!
//! **Helper public, signature stable contractée pour 8-5b (rules engine).**
//!
//! Compose le pattern sign-aware de
//! [`crate::manual::build_journal_entry_for_counterparty`] (8-5a-base)
//! sans le composer littéralement (cf. §helper-split-signature de la
//! spec — implémentation directe N+1 lignes plutôt que N appels manual
//! + fusion).

use chrono::NaiveDate;
use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use rust_decimal::Decimal;

use crate::errors::ReconciliationError;

/// Détail d'un split pour [`build_split_journal_entry`].
///
/// `amount` DOIT être strictement positif (validation handler-side §scope
/// point 1 + step 2). Le signe débit/crédit est déterminé par
/// `sign(tx.amount)` au niveau du builder.
#[derive(Debug, Clone)]
pub struct SplitDetail {
    pub account_id: i64,
    pub amount: Decimal,
    pub description: String,
    /// Projet analytique de cette ligne de ventilation (Story 19-5). Un
    /// split éclate une transaction en plusieurs finalités → chaque ligne
    /// peut porter son propre projet. `None` = ligne non taguée. Validé
    /// automatiquement par `journal_entries::create_in_tx` (validation
    /// per-ligne étape 0).
    pub project_id: Option<i64>,
}

/// Résultat de [`validate_split_balance`] en cas de mismatch.
///
/// Mappé vers [`ReconciliationError::SplitImbalance`] via [`From`]
/// — `expected` = `tx.amount.abs()`, `actual` = `sum(splits)`,
/// `difference` = `actual - expected`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitImbalance {
    pub expected: Decimal,
    pub actual: Decimal,
    pub difference: Decimal,
}

/// Construit une `NewJournalEntry` à N+1 lignes pour split (FR48).
/// Pure (zéro I/O). Sign-aware : sign de `tx.amount` détermine le côté
/// débit/crédit de la ligne banque vs des lignes contreparties.
///
/// **Helper public, signature stable contractée pour 8-5b.**
///
/// Inputs :
/// - `tx` : la `BankTransaction` à éclater (status='pending' assumé,
///   précondition vérifiée handler-side via
///   [`kesh_db::repositories::reconciliation::find_strictly_pending_by_id_for_account`]).
/// - `bank_account_journal_id` : compte comptable banque résolu via
///   `bank_account.journal_account_id` (foundation 8-5a-zero).
/// - `splits` : tableau `SplitDetail`. Précondition caller : `splits.len()`
///   ∈ [2, 50] ET `splits[i].amount > 0` ET `sum(splits) == tx.amount.abs()`
///   (validations handler-side §validation-handler-side-split steps 1-7).
/// - `description` : description top-level de la journal_entry. Le caller
///   construit typiquement `format!("Éclatement transaction agrégée ({} lignes)", splits.len())`
///   (M3''' Pass 3 Opus).
/// - `entry_date` : `body.value_date.or(tx.value_date).unwrap_or(tx.booking_date)`
///   (3 couches cohérent post_manual reconciliation.rs:1281, F4''' Pass 3).
///
/// Output : `NewJournalEntry` avec :
/// - `company_id = tx.company_id`
/// - `journal = Journal::Banque`
/// - N+1 `lines` équilibrées :
///   - Ligne 1 (banque) : `bank_account_journal_id`, montant `tx.amount.abs()`,
///     débit ou crédit selon `sign(tx.amount)`.
///   - Lignes 2..N+1 (contreparties) : pour chaque `splits[i]`, sign
///     opposé à la banque.
///
/// **Sémantique sign-aware** :
/// - `tx.amount > 0` (crédit titulaire = entrée cash) →
///   débit `bank_ledger` (total), crédit N comptes contrepartie.
/// - `tx.amount < 0` (débit titulaire = sortie cash) →
///   crédit `bank_ledger` (total), débit N comptes contrepartie.
///
/// # Panics
///
/// Panics si `tx.amount.is_zero()`. La précondition est garantie par le
/// handler step 6bis (M2''' Pass 3 Opus) ; un panic ici signifie un bug
/// d'invariant — fail-fast cohérent avec
/// [`crate::manual::build_journal_entry_for_counterparty`].
pub fn build_split_journal_entry(
    tx: &BankTransaction,
    bank_account_journal_id: i64,
    splits: &[SplitDetail],
    description: String,
    entry_date: NaiveDate,
) -> NewJournalEntry {
    assert!(
        !tx.amount.is_zero(),
        "build_split_journal_entry assumes tx.amount != 0; \
         handler MUST pré-valider step 6bis (cf. spec §validation-handler-side-split)"
    );

    let abs_amount = tx.amount.abs();

    // Sign-aware : sortie cash → banque crédit + splits débit ;
    // entrée cash → banque débit + splits crédit.
    let entry_is_inflow = tx.amount > Decimal::ZERO;

    let mut lines: Vec<NewJournalEntryLine> = Vec::with_capacity(splits.len() + 1);

    // Ligne 1 — banque agrégée sur tx.amount.abs().
    let (bank_debit, bank_credit) = if entry_is_inflow {
        (abs_amount, Decimal::ZERO)
    } else {
        (Decimal::ZERO, abs_amount)
    };
    lines.push(NewJournalEntryLine {
        account_id: bank_account_journal_id,
        debit: bank_debit,
        credit: bank_credit,
        project_id: None,
    });

    // Lignes 2..N+1 — N contreparties, sign opposé à la banque.
    for split in splits {
        let (cp_debit, cp_credit) = if entry_is_inflow {
            (Decimal::ZERO, split.amount)
        } else {
            (split.amount, Decimal::ZERO)
        };
        lines.push(NewJournalEntryLine {
            account_id: split.account_id,
            debit: cp_debit,
            credit: cp_credit,
            // Story 19-5 — projet par ligne de ventilation (multi-usage).
            project_id: split.project_id,
        });
    }

    NewJournalEntry {
        company_id: tx.company_id,
        entry_date,
        journal: Journal::Banque,
        description,
        // Story 19-5 — le tag est porté par ligne (SplitDetail.project_id),
        // pas au niveau document : la ligne banque reste non taguée.
        project_id: None,
        lines,
    }
}

/// Vérifie `sum(splits[*]) == tx_amount.abs()` Decimal exact (pas de tolérance).
///
/// **Précondition** : caller DOIT vérifier `splits.len() >= 2` ET
/// `splits.len() <= 50` ET `splits[i] > 0` AVANT (cohérent §helper-split-signature).
/// `tx_amount` est passé brut (signed) ; `.abs()` est appliqué en interne.
///
/// Retourne `Err(SplitImbalance)` si mismatch, mappé en
/// [`ReconciliationError::SplitImbalance`] via [`From`] (cf. helper).
pub fn validate_split_balance(
    tx_amount: Decimal,
    splits: &[Decimal],
) -> Result<(), SplitImbalance> {
    let sum: Decimal = splits.iter().sum();
    let expected = tx_amount.abs();
    if sum != expected {
        return Err(SplitImbalance {
            expected,
            actual: sum,
            difference: sum - expected,
        });
    }
    Ok(())
}

impl From<SplitImbalance> for ReconciliationError {
    fn from(e: SplitImbalance) -> Self {
        ReconciliationError::SplitImbalance {
            expected: e.expected,
            actual: e.actual,
            difference: e.difference,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDateTime;
    use kesh_db::entities::bank_transaction::{BankTransaction, BankTransactionStatus};
    use rust_decimal_macros::dec;

    fn make_test_tx(amount: Decimal) -> BankTransaction {
        BankTransaction {
            id: 42,
            company_id: 7,
            import_id: 1,
            bank_account_id: 17,
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 31).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 31).unwrap()),
            amount,
            currency: "CHF".into(),
            reference: Some("SALARIES-MAY".into()),
            details: "Paiement salaires multi".into(),
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            status: BankTransactionStatus::Pending,
            matched_entry_id: None,
            auto_match_rejected_at: None,
            version: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        }
    }

    fn split(account_id: i64, amount: Decimal, description: &str) -> SplitDetail {
        SplitDetail {
            account_id,
            amount,
            description: description.to_string(),
            project_id: None,
        }
    }

    /// Comme [`split`] mais avec un projet analytique (Story 19-5).
    fn split_with_project(
        account_id: i64,
        amount: Decimal,
        description: &str,
        project_id: Option<i64>,
    ) -> SplitDetail {
        SplitDetail {
            account_id,
            amount,
            description: description.to_string(),
            project_id,
        }
    }

    /// AC #93 — split d'une sortie cash (`tx.amount = -10700`) en 3 lignes
    /// contreparties (5000+4500+1200). Vérifie 1 ligne banque crédit
    /// `10700` + 3 lignes débit (équilibre partie double).
    #[test]
    fn split_build_je_creates_n_plus_1_lines_for_debit_tx() {
        let tx = make_test_tx(dec!(-10700.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let splits = vec![
            split(5000, dec!(5000.00), "Salaire Alice"),
            split(5000, dec!(4500.00), "Salaire Bob"),
            split(5700, dec!(1200.00), "Charges sociales"),
        ];

        let je = build_split_journal_entry(
            &tx,
            1020,
            &splits,
            "Éclatement transaction agrégée (3 lignes)".to_string(),
            entry_date,
        );

        assert_eq!(je.company_id, tx.company_id);
        assert_eq!(je.entry_date, entry_date);
        assert!(matches!(je.journal, Journal::Banque));
        assert_eq!(je.lines.len(), 4, "1 ligne banque + 3 splits = 4");

        // Ligne 1 banque : crédit 10700 (sortie cash).
        let line_bank = &je.lines[0];
        assert_eq!(line_bank.account_id, 1020);
        assert_eq!(line_bank.debit, Decimal::ZERO);
        assert_eq!(line_bank.credit, dec!(10700.00));

        // Lignes 2..4 splits : tous débit, total = 10700.
        let line_alice = &je.lines[1];
        assert_eq!(line_alice.account_id, 5000);
        assert_eq!(line_alice.debit, dec!(5000.00));
        assert_eq!(line_alice.credit, Decimal::ZERO);

        let line_bob = &je.lines[2];
        assert_eq!(line_bob.account_id, 5000);
        assert_eq!(line_bob.debit, dec!(4500.00));
        assert_eq!(line_bob.credit, Decimal::ZERO);

        let line_charges = &je.lines[3];
        assert_eq!(line_charges.account_id, 5700);
        assert_eq!(line_charges.debit, dec!(1200.00));
        assert_eq!(line_charges.credit, Decimal::ZERO);

        // Balance partie double globale.
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit).sum();
        assert_eq!(total_debit, total_credit, "lignes équilibrées");
        assert_eq!(total_debit, dec!(10700.00));
    }

    /// AC #94 — split d'une entrée cash (`tx.amount = +5000`) en 2 lignes
    /// contreparties (3000+2000). Vérifie 1 ligne banque débit `5000`
    /// + 2 lignes crédit.
    #[test]
    fn split_build_je_creates_n_plus_1_lines_for_credit_tx() {
        let tx = make_test_tx(dec!(5000.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let splits = vec![
            split(7510, dec!(3000.00), "Intérêts mai"),
            split(6900, dec!(2000.00), "Remboursement frais"),
        ];

        let je = build_split_journal_entry(
            &tx,
            1020,
            &splits,
            "Éclatement transaction agrégée (2 lignes)".to_string(),
            entry_date,
        );

        assert_eq!(je.lines.len(), 3, "1 banque + 2 splits = 3");

        // Ligne 1 banque : débit 5000 (entrée cash).
        let line_bank = &je.lines[0];
        assert_eq!(line_bank.account_id, 1020);
        assert_eq!(line_bank.debit, dec!(5000.00));
        assert_eq!(line_bank.credit, Decimal::ZERO);

        // Lignes 2..3 splits : tous crédit.
        let line_interets = &je.lines[1];
        assert_eq!(line_interets.account_id, 7510);
        assert_eq!(line_interets.debit, Decimal::ZERO);
        assert_eq!(line_interets.credit, dec!(3000.00));

        let line_rembours = &je.lines[2];
        assert_eq!(line_rembours.account_id, 6900);
        assert_eq!(line_rembours.debit, Decimal::ZERO);
        assert_eq!(line_rembours.credit, dec!(2000.00));

        let total_debit: Decimal = je.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit).sum();
        assert_eq!(total_debit, total_credit);
        assert_eq!(total_debit, dec!(5000.00));
    }

    /// Story 19-5 — chaque ligne de ventilation porte son propre projet
    /// (multi-usage), la ligne banque reste non taguée et le tag
    /// document-level de l'écriture reste `None`.
    #[test]
    fn split_build_je_tags_project_per_line() {
        let tx = make_test_tx(dec!(-10700.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 31).unwrap();
        let splits = vec![
            split_with_project(5000, dec!(5000.00), "Rénovation chalet", Some(11)),
            split_with_project(5000, dec!(4500.00), "Rénovation appart", Some(22)),
            split_with_project(5700, dec!(1200.00), "Divers non affecté", None),
        ];

        let je = build_split_journal_entry(
            &tx,
            1020,
            &splits,
            "Éclatement transaction agrégée (3 lignes)".to_string(),
            entry_date,
        );

        // Tag document-level None : le split porte le tag par-ligne.
        assert_eq!(je.project_id, None);
        // Ligne banque (index 0) non taguée.
        assert_eq!(je.lines[0].project_id, None);
        // Lignes de ventilation : projet par ligne.
        assert_eq!(je.lines[1].project_id, Some(11));
        assert_eq!(je.lines[2].project_id, Some(22));
        assert_eq!(je.lines[3].project_id, None);
    }

    /// AC #93 — `validate_split_balance` retourne `Ok(())` quand
    /// `sum(splits) == tx_amount.abs()` exact.
    #[test]
    fn split_validate_balance_exact_match_ok() {
        // Sortie cash -10700 ; splits 5000+4500+1200 = 10700 exact.
        let result = validate_split_balance(
            dec!(-10700.00),
            &[dec!(5000.00), dec!(4500.00), dec!(1200.00)],
        );
        assert!(result.is_ok());

        // Entrée cash +5000 ; splits 3000+2000 = 5000 exact.
        let result = validate_split_balance(dec!(5000.00), &[dec!(3000.00), dec!(2000.00)]);
        assert!(result.is_ok());

        // Précision Decimal : .50 + .50 = 1.00 exact (pas IEEE 754).
        let result = validate_split_balance(dec!(-1.00), &[dec!(0.50), dec!(0.50)]);
        assert!(result.is_ok());
    }

    /// AC #95 — `validate_split_balance` retourne
    /// `Err(SplitImbalance { expected, actual, difference })` quand
    /// `sum != tx_amount.abs()`. La conversion via `From<SplitImbalance>
    /// for ReconciliationError` est testée implicitement.
    #[test]
    fn split_validate_balance_imbalance_returns_error() {
        // Sortie cash -10700 ; splits 5000+4500+1000 = 10500 (200 missing).
        let result = validate_split_balance(
            dec!(-10700.00),
            &[dec!(5000.00), dec!(4500.00), dec!(1000.00)],
        );
        let err = result.expect_err("imbalance expected");
        assert_eq!(err.expected, dec!(10700.00));
        assert_eq!(err.actual, dec!(10500.00));
        assert_eq!(err.difference, dec!(-200.00));

        // Mapping vers ReconciliationError via From.
        let rec_err: ReconciliationError = err.into();
        match rec_err {
            ReconciliationError::SplitImbalance {
                expected,
                actual,
                difference,
            } => {
                assert_eq!(expected, dec!(10700.00));
                assert_eq!(actual, dec!(10500.00));
                assert_eq!(difference, dec!(-200.00));
            }
            _ => panic!("expected SplitImbalance variant"),
        }
    }

    /// Regression — `splits` excess (200 de trop) : difference > 0.
    #[test]
    fn split_validate_balance_excess_returns_positive_difference() {
        // splits sum=10900 > abs(-10700)=10700.
        let result = validate_split_balance(
            dec!(-10700.00),
            &[dec!(5000.00), dec!(4500.00), dec!(1400.00)],
        );
        let err = result.expect_err("imbalance expected");
        assert_eq!(err.expected, dec!(10700.00));
        assert_eq!(err.actual, dec!(10900.00));
        assert_eq!(err.difference, dec!(200.00));
    }
}
