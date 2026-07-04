//! Story 8-5a-base FR45 — réconciliation manuelle.
//!
//! Helper public [`build_journal_entry_for_counterparty`] qui construit
//! une `NewJournalEntry` à 2 lignes équilibrées (sign-aware) pour le
//! flow `POST /api/v1/reconciliation/manual`.
//!
//! **Pure** (zéro I/O) — facilite tests unitaires sans mock DB.
//!
//! **Helper public, signature stable contractée pour 8-5a-bis (split)
//! et 8-5b (rules engine)**. Aucune évolution sans CR explicite après
//! merge 8-5a-base.

use chrono::NaiveDate;
use kesh_db::entities::bank_transaction::BankTransaction;
use kesh_db::entities::journal_entry::{Journal, NewJournalEntry, NewJournalEntryLine};
use rust_decimal::Decimal;

/// Construit une `NewJournalEntry` à 2 lignes pour réconciliation manuelle.
/// Pure (zéro I/O). Sign-aware : sign de `tx.amount` détermine le côté
/// débit/crédit.
///
/// **Helper public, signature stable contractée pour 8-5a-bis et 8-5b**.
///
/// Inputs :
/// - `tx` : la `BankTransaction` à matcher (status='pending' assumé,
///   précondition vérifiée handler-side via
///   [`crate::find_strictly_pending_by_id_for_account`] ou équivalent).
/// - `bank_account_journal_id` : compte comptable banque résolu via
///   `bank_account.journal_account_id` (foundation 8-5a-zero).
/// - `counterparty_account_id` : compte de contrepartie choisi par
///   l'utilisateur (typiquement classe 5/6/7 — mais pas d'invariant
///   serveur, le frontend filtre côté UX).
/// - `description` : description de la journal_entry (max 200 chars,
///   validation handler).
/// - `entry_date` : `valueDate ?? tx.booking_date` (résolu côté
///   handler).
///
/// Output : `NewJournalEntry` avec :
/// - `company_id = tx.company_id`
/// - `journal = Journal::Banque` (toute opération bank_transaction
///   tombe dans le journal Banque)
/// - 2 `lines` équilibrées :
///   - Ligne 1 (banque) : `bank_account_journal_id`, débit ou crédit
///     selon `sign(tx.amount)`.
///   - Ligne 2 (contrepartie) : `counterparty_account_id`, opposé sign.
///
/// **Sémantique sign-aware** :
/// - `tx.amount > 0` (crédit titulaire = entrée cash) →
///   débit `bank_ledger`, crédit `counterparty`.
/// - `tx.amount < 0` (débit titulaire = sortie cash) →
///   crédit `bank_ledger`, débit `counterparty`.
///
/// **Précondition `tx.amount != 0`** (F7''' Pass 3 Opus) : si
/// `tx.amount == 0`, les 2 lignes seraient 0/0 (sémantiquement vides).
/// Le handler **doit pré-valider** `tx.amount != Decimal::ZERO` avec
/// 400 `VALIDATION_ERROR { reason: "zero_amount_transaction" }` (cf.
/// step 4bis §validation-handler-side).
///
/// # Panics
///
/// Panics si `tx.amount.is_zero()`. La précondition est garantie par le
/// handler step 4bis ; un panic ici signifie un bug d'invariant.
/// P-M6 Pass 1 code review : `assert!` (vs `debug_assert!`) pour que
/// l'invariant soit aussi enforced en release — fail-fast en cas de
/// breach plutôt que produire silencieusement une écriture 0/0
/// sémantiquement vide qui pollue les comptes.
pub fn build_journal_entry_for_counterparty(
    tx: &BankTransaction,
    bank_account_journal_id: i64,
    counterparty_account_id: i64,
    description: String,
    entry_date: NaiveDate,
    project_id: Option<i64>,
) -> NewJournalEntry {
    assert!(
        !tx.amount.is_zero(),
        "build_journal_entry_for_counterparty assumes tx.amount != 0; \
         handler MUST pré-valider step 4bis (cf. spec §validation-handler-side)"
    );

    // `abs()` pour les montants : la partie double exige des valeurs
    // positives sur les deux côtés débit/crédit ; la sémantique
    // débit/crédit est portée par les colonnes elles-mêmes.
    let abs_amount = tx.amount.abs();

    let (bank_debit, bank_credit, counterparty_debit, counterparty_credit) =
        if tx.amount > Decimal::ZERO {
            // Crédit titulaire (entrée cash) :
            // - banque : débit (compte d'actif augmente)
            // - contrepartie : crédit (compte de produit augmente, ou
            //   diminution d'un passif)
            (abs_amount, Decimal::ZERO, Decimal::ZERO, abs_amount)
        } else {
            // Débit titulaire (sortie cash) :
            // - banque : crédit (compte d'actif diminue)
            // - contrepartie : débit (compte de charge augmente)
            (Decimal::ZERO, abs_amount, abs_amount, Decimal::ZERO)
        };

    NewJournalEntry {
        company_id: tx.company_id,
        entry_date,
        journal: Journal::Banque,
        description,
        // Story 19-5 — tag document-level (mono-usage) : recopié sur les 2
        // lignes (banque + contrepartie) via `line.project_id.or(new.project_id)`
        // dans `journal_entries::create_in_tx`. Validé par le caller AVANT
        // create_in_tx (le repo ne valide pas `new.project_id`, cf. 19-2 DC2).
        project_id,
        lines: vec![
            NewJournalEntryLine {
                account_id: bank_account_journal_id,
                debit: bank_debit,
                credit: bank_credit,
                project_id: None,
            },
            NewJournalEntryLine {
                account_id: counterparty_account_id,
                debit: counterparty_debit,
                credit: counterparty_credit,
                project_id: None,
            },
        ],
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
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 15).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
            amount,
            currency: "CHF".into(),
            reference: Some("TWINT-XYZ".into()),
            details: "Frais TWINT".into(),
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

    /// AC #84 — happy path crédit positif (encaissement).
    /// `tx.amount = +200.00` → ligne 1 banque débit 200 / ligne 2
    /// contrepartie crédit 200.
    #[test]
    fn build_journal_entry_for_counterparty_credit_positive_amount() {
        let tx = make_test_tx(dec!(200.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        let je = build_journal_entry_for_counterparty(
            &tx,
            1020, // bank_account_journal_id
            7510, // counterparty_account_id (Intérêts bancaires)
            "Intérêts mai".to_string(),
            entry_date,
            None, // project_id (Story 19-5)
        );

        assert_eq!(je.company_id, tx.company_id);
        assert_eq!(je.entry_date, entry_date);
        assert!(matches!(je.journal, Journal::Banque));
        assert_eq!(je.description, "Intérêts mai");
        assert_eq!(je.project_id, None);
        assert_eq!(je.lines.len(), 2);

        // Ligne 1 : banque (débit pour entrée cash).
        let line_bank = &je.lines[0];
        assert_eq!(line_bank.account_id, 1020);
        assert_eq!(line_bank.debit, dec!(200.00));
        assert_eq!(line_bank.credit, Decimal::ZERO);

        // Ligne 2 : contrepartie (crédit).
        let line_cp = &je.lines[1];
        assert_eq!(line_cp.account_id, 7510);
        assert_eq!(line_cp.debit, Decimal::ZERO);
        assert_eq!(line_cp.credit, dec!(200.00));

        // Balance double-entry.
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit).sum();
        assert_eq!(total_debit, total_credit, "lignes équilibrées");
    }

    /// AC #83 — happy path débit négatif (paiement).
    /// `tx.amount = -150.00` → ligne 1 banque crédit 150 / ligne 2
    /// contrepartie débit 150.
    #[test]
    fn build_journal_entry_for_counterparty_debit_negative_amount() {
        let tx = make_test_tx(dec!(-150.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        let je = build_journal_entry_for_counterparty(
            &tx,
            1020, // bank_account_journal_id
            6810, // counterparty_account_id (Frais bancaires)
            "Frais TWINT mai".to_string(),
            entry_date,
            None, // project_id (Story 19-5)
        );

        assert_eq!(je.company_id, tx.company_id);
        assert_eq!(je.lines.len(), 2);
        assert!(matches!(je.journal, Journal::Banque));

        // Ligne 1 : banque (crédit pour sortie cash).
        let line_bank = &je.lines[0];
        assert_eq!(line_bank.account_id, 1020);
        assert_eq!(line_bank.debit, Decimal::ZERO);
        assert_eq!(line_bank.credit, dec!(150.00));

        // Ligne 2 : contrepartie (débit).
        let line_cp = &je.lines[1];
        assert_eq!(line_cp.account_id, 6810);
        assert_eq!(line_cp.debit, dec!(150.00));
        assert_eq!(line_cp.credit, Decimal::ZERO);

        // Balance double-entry.
        let total_debit: Decimal = je.lines.iter().map(|l| l.debit).sum();
        let total_credit: Decimal = je.lines.iter().map(|l| l.credit).sum();
        assert_eq!(total_debit, total_credit, "lignes équilibrées");
    }

    /// Story 19-5 — le `project_id` document-level est porté par l'écriture ;
    /// la propagation aux 2 lignes se fait dans `create_in_tx` via
    /// `line.project_id.or(new.project_id)`, donc au niveau builder les lignes
    /// restent `None` et seule l'entête porte le tag.
    #[test]
    fn build_journal_entry_for_counterparty_tags_project_document_level() {
        let tx = make_test_tx(dec!(200.00));
        let entry_date = NaiveDate::from_ymd_opt(2026, 5, 15).unwrap();
        let je = build_journal_entry_for_counterparty(
            &tx,
            1020,
            7510,
            "Loyer projet rénovation".to_string(),
            entry_date,
            Some(42), // project_id
        );

        assert_eq!(
            je.project_id,
            Some(42),
            "tag document-level porté par l'entête"
        );
        // Les lignes restent None au niveau builder — la propagation est faite
        // par le repo (create_in_tx). Documenté par 19-2/19-3.
        assert!(je.lines.iter().all(|l| l.project_id.is_none()));
    }
}
