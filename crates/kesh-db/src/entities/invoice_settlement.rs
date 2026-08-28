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
}
