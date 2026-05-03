//! Conversion des relevés bancaires importés (`kesh-import`) vers les types
//! domaine `kesh-core`.
//!
//! Ce module matérialise la décision architecture #7 : les crates publiables
//! (`kesh-import`, `kesh-payment`, `kesh-qrbill`) ont des types autonomes,
//! et les conversions vers les types domaine vivent côté `kesh-core` via
//! `From`/`Into`. La direction de dépendance Cargo est donc
//! `kesh-core → kesh-import` (jamais l'inverse).
//!
//! # Statut spike (2026-05-03)
//!
//! Cette première itération est volontairement minimale : un type
//! [`BankTransactionDraft`] avec uniquement les champs portés par
//! `ImportedTransaction`, sans les clés étrangères (`bank_account_id`,
//! `import_id`, `company_id`) qui sont assignées au moment de la persistance
//! (Story 8-1).
//!
//! La spec complète (validation IBAN via [`crate::types::Iban`], gestion des
//! références ESR/QR-Référence, mapping de la devise, contrôle de cohérence
//! du solde de clôture conformément à CR-010 #62) appartient à Story 8-1 —
//! ce module sera étendu par cette story.

use chrono::NaiveDate;
use kesh_import::ImportedTransaction;

use crate::types::Money;

/// Brouillon de transaction bancaire — image projetée d'un
/// [`ImportedTransaction`] dans le vocabulaire domaine `kesh-core`.
///
/// Les clés étrangères (`bank_account_id`, `import_id`, `company_id`) ne
/// figurent pas dans ce type : elles sont injectées par
/// `kesh-api::routes::bank_imports` au moment de la persistance, à partir du
/// contexte de la requête (compte sélectionné par l'utilisateur, tenant
/// courant, hash du fichier importé).
///
/// L'IBAN de la contrepartie est conservé sous forme de `String` brute à ce
/// stade : la validation MOD-97 via [`crate::types::Iban`] est laissée à
/// Story 8-1 pour permettre un import « tolérant » (transaction conservée
/// même si l'IBAN est mal formé, avec un statut d'avertissement) plutôt
/// qu'un rejet strict du fichier entier.
#[derive(Clone, Debug, PartialEq)]
pub struct BankTransactionDraft {
    pub booking_date: NaiveDate,
    pub value_date: Option<NaiveDate>,
    pub amount: Money,
    pub currency: String,
    pub reference: Option<String>,
    pub details: String,
    pub end_to_end_id: Option<String>,
    pub transaction_id: Option<String>,
    pub counterparty_iban: Option<String>,
    pub counterparty_name: Option<String>,
}

impl From<ImportedTransaction> for BankTransactionDraft {
    fn from(tx: ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency,
            reference: tx.reference,
            details: tx.details,
            end_to_end_id: tx.end_to_end_id,
            transaction_id: tx.transaction_id,
            counterparty_iban: tx.counterparty_iban,
            counterparty_name: tx.counterparty_name,
        }
    }
}

impl From<&ImportedTransaction> for BankTransactionDraft {
    fn from(tx: &ImportedTransaction) -> Self {
        Self {
            booking_date: tx.booking_date,
            value_date: tx.value_date,
            amount: Money::new(tx.amount),
            currency: tx.currency.clone(),
            reference: tx.reference.clone(),
            details: tx.details.clone(),
            end_to_end_id: tx.end_to_end_id.clone(),
            transaction_id: tx.transaction_id.clone(),
            counterparty_iban: tx.counterparty_iban.clone(),
            counterparty_name: tx.counterparty_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn fixture() -> ImportedTransaction {
        ImportedTransaction {
            booking_date: NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
            value_date: Some(NaiveDate::from_ymd_opt(2026, 5, 3).unwrap()),
            amount: dec!(1234.56),
            currency: "CHF".to_string(),
            reference: Some("RF18539007547034".to_string()),
            details: "Loyer mai 2026".to_string(),
            end_to_end_id: None,
            transaction_id: Some("BANK-TX-42".to_string()),
            counterparty_iban: Some("CH9300762011623852957".to_string()),
            counterparty_name: Some("Acme SA".to_string()),
        }
    }

    #[test]
    fn from_owned_imported_transaction() {
        let tx = fixture();
        let draft: BankTransactionDraft = tx.into();
        assert_eq!(draft.booking_date, NaiveDate::from_ymd_opt(2026, 5, 2).unwrap());
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
        assert_eq!(draft.currency, "CHF");
        assert_eq!(draft.reference.as_deref(), Some("RF18539007547034"));
        assert_eq!(draft.transaction_id.as_deref(), Some("BANK-TX-42"));
    }

    #[test]
    fn from_borrowed_imported_transaction_does_not_consume() {
        let tx = fixture();
        let draft: BankTransactionDraft = (&tx).into();
        // tx still usable after the conversion.
        assert_eq!(tx.amount, dec!(1234.56));
        assert_eq!(draft.amount, Money::new(dec!(1234.56)));
    }

    #[test]
    fn negative_amount_preserved_as_signed_money() {
        let tx = ImportedTransaction {
            amount: dec!(-50.00),
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.amount.is_negative());
        assert_eq!(draft.amount, Money::new(dec!(-50.00)));
    }

    #[test]
    fn missing_optional_fields_propagate_as_none() {
        let tx = ImportedTransaction {
            value_date: None,
            reference: None,
            end_to_end_id: None,
            transaction_id: None,
            counterparty_iban: None,
            counterparty_name: None,
            ..fixture()
        };
        let draft: BankTransactionDraft = tx.into();
        assert!(draft.value_date.is_none());
        assert!(draft.reference.is_none());
        assert!(draft.end_to_end_id.is_none());
        assert!(draft.transaction_id.is_none());
        assert!(draft.counterparty_iban.is_none());
        assert!(draft.counterparty_name.is_none());
    }
}
