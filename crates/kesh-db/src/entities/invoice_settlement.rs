//! Règlement d'une facture client — la liaison entre une facture et l'écriture
//! comptable qui en éteint tout ou partie.
//!
//! ⚠️ **Une facture se règle en PLUSIEURS fois**, contrairement à une facture
//! fournisseur (`supplier_invoices.settlement_journal_entry_id`, au singulier).
//! Un débiteur paie en deux virements, un avoir éteint une part de la créance :
//! c'est pourquoi la liaison est une table et non une colonne.
//!
//! ⚠️ **Rien ici ne dit si la facture est soldée.** Le résiduel se calcule —
//! `TTC − avoir émis − Σ amount` — il ne se stocke pas. Un montant dû rangé en
//! colonne dérive du grand livre à la première divergence, et on aurait recréé
//! un chiffre qui ment.
//!
//! `FromRow` sans `Serialize` : patron `invoice_reminder`, anti-fuite du
//! `company_id` vers le client.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;

/// Une ligne de règlement, telle que persistée.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InvoiceSettlement {
    pub id: i64,
    pub company_id: i64,
    pub invoice_id: i64,
    /// L'écriture qui porte le mouvement — `D banque / C créance`.
    pub journal_entry_id: i64,
    /// Montant réglé, **toujours strictement positif** (`chk_…_amount_positive`).
    /// Un remboursement se passe en contre-passation, pas en montant signé.
    pub amount: Decimal,
    /// Date de valeur du règlement — celle de l'écriture, pas celle de la saisie.
    pub settled_on: NaiveDate,
    /// `bank_transfer` ou `internal_account` — vocabulaire repris **mot pour
    /// mot** de `chk_supplier_invoices_settlement_type` (Story 24-3).
    pub settlement_type: String,
    /// Renseigné ssi `settlement_type = 'bank_transfer'` (contrainte DB).
    pub settlement_bank_account_id: Option<i64>,
    /// Renseigné ssi `settlement_type = 'internal_account'` (contrainte DB).
    pub settlement_account_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

/// Ce qu'il faut fournir pour enregistrer un règlement.
#[derive(Debug, Clone)]
pub struct NewInvoiceSettlement {
    pub company_id: i64,
    pub invoice_id: i64,
    pub journal_entry_id: i64,
    pub amount: Decimal,
    pub settled_on: NaiveDate,
    /// Le MODE, et sa contrepartie. Story 24-3 (#372).
    ///
    /// ⚠️ **Il vit sur le RÈGLEMENT, pas sur la facture** — contrairement au
    /// symétrique fournisseur, où le règlement est unique par construction. Une
    /// facture client peut être réglée moitié en espèces, moitié par virement.
    pub choice: SettlementChoice,
}

/// Le mode de règlement, et rien d'autre : ce n'est **pas** une table de modes.
///
/// ⛔ Réexporté depuis `supplier_invoice` plutôt que redéfini. Deux vocabulaires
/// pour la même notion coûteraient à chaque lecture, et le contrat DB est
/// littéralement le même (`('bank_transfer', 'internal_account')`).
pub use super::supplier_invoice::SettlementChoice;

impl SettlementChoice {
    /// La valeur persistée en `settlement_type`.
    pub fn type_str(&self) -> &'static str {
        match self {
            SettlementChoice::BankTransfer { .. } => "bank_transfer",
            SettlementChoice::InternalAccount { .. } => "internal_account",
        }
    }

    /// Les deux références de contrepartie, dans l'ordre
    /// `(settlement_bank_account_id, settlement_account_id)` — **exactement une
    /// est renseignée**, ce que `chk_invoice_settlements_counterparty` impose.
    pub fn counterparty_refs(&self) -> (Option<i64>, Option<i64>) {
        match self {
            SettlementChoice::BankTransfer { bank_account_id } => (Some(*bank_account_id), None),
            SettlementChoice::InternalAccount { account_id } => (None, Some(*account_id)),
        }
    }
}
