//! **ORACLE GELÉ** — Story 15-1b-0, D4 (1). Ne PAS faire suivre le code de
//! production.
//!
//! Copie **à l'identique** (SQL, décodage, ordre des rangs) de
//! `kesh_db::repositories::journal_entries::reversal_blockers` telle qu'au
//! commit `056997b0` (identique à `f8888750`, base de la story) — avant sa
//! réécriture sur `document_owners`. Seul le nom de la fonction change
//! (`reversal_blockers_frozen`) et les chemins sont qualifiés.
//!
//! **Pourquoi elle ne doit pas suivre** : elle est la preuve que la refonte
//! rend ce que rendait l'ancien code. Réécrite sur le nouveau, la comparaison
//! serait verte par construction (l'AC7 de l'Epic 23, § « Inventorier les sites
//! NON RÉSOLUS » du `CLAUDE.md`). Un changement VOULU de comportement se
//! déclare dans le test qui la compare, jamais ici.
//!
//! ⚠️ Ses `LIMIT 1` sans `ORDER BY` ne sont déterministes qu'avec une pièce
//! de chaque type par écriture : la fixture n'en pose jamais deux du même type.
//!
//! Inclus par `#[path]` dans `tests/document_owners.rs` et dans lui seul (un
//! fichier d'un sous-dossier de `tests/` sans `main.rs` n'est pas une cible).

use kesh_db::errors::{DbError, ReversalBlocker, map_db_error};
use kesh_db::repositories::journal_entries::ReversalBlockerHit;

pub async fn reversal_blockers_frozen<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Vec<ReversalBlockerHit>, DbError>
where
    E: sqlx::Executor<'e, Database = sqlx::MySql>,
{
    /// Les sept causes de blocage, telles que la requête les rend.
    ///
    /// ⚠️ Une struct nommée plutôt qu'un 7-uplet : `clippy::type_complexity`
    /// refuse le second, et il avait surtout l'inconvénient de rendre l'ordre
    /// des colonnes muet — sept `Option<i64>` d'affilée ne se relisent pas.
    #[derive(sqlx::FromRow)]
    struct BlockerRow {
        reverses_entry_id: Option<i64>,
        reversed_by: Option<i64>,
        invoice_id: Option<i64>,
        invoice_number: Option<String>,
        credit_note_id: Option<i64>,
        credit_note_number: Option<String>,
        supplier_invoice_id: Option<i64>,
        supplier_invoice_number: Option<String>,
        settlement_id: Option<i64>,
        bank_transaction_id: Option<i64>,
        archived_account_number: Option<String>,
    }

    let row: Option<BlockerRow> = sqlx::query_as(
        "SELECT \
           je.reverses_entry_id, \
           (SELECT r.id FROM journal_entries r \
             WHERE r.reverses_entry_id = je.id AND r.company_id = je.company_id LIMIT 1) AS reversed_by, \
           (SELECT i.id FROM invoices i WHERE i.journal_entry_id = je.id LIMIT 1) AS invoice_id, \
           (SELECT i.invoice_number FROM invoices i WHERE i.journal_entry_id = je.id LIMIT 1) AS invoice_number, \
           (SELECT c.id FROM credit_notes c WHERE c.journal_entry_id = je.id LIMIT 1) AS credit_note_id, \
           (SELECT c.credit_note_number FROM credit_notes c WHERE c.journal_entry_id = je.id LIMIT 1) AS credit_note_number, \
           (SELECT s.id FROM supplier_invoices s \
             WHERE s.purchase_journal_entry_id = je.id OR s.settlement_journal_entry_id = je.id \
             LIMIT 1) AS supplier_invoice_id, \
           (SELECT s.supplier_invoice_number FROM supplier_invoices s \
             WHERE s.purchase_journal_entry_id = je.id OR s.settlement_journal_entry_id = je.id \
             LIMIT 1) AS supplier_invoice_number, \
           (SELECT st.id FROM invoice_settlements st WHERE st.journal_entry_id = je.id LIMIT 1) AS settlement_id, \
           (SELECT bt.id FROM bank_transactions bt WHERE bt.matched_entry_id = je.id LIMIT 1) AS bank_transaction_id, \
           (SELECT a.number FROM journal_entry_lines jel \
             JOIN accounts a ON a.id = jel.account_id \
             WHERE jel.entry_id = je.id AND a.active = FALSE \
             ORDER BY a.number LIMIT 1) AS archived_account_number \
         FROM journal_entries je \
         WHERE je.id = ? AND je.company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(executor)
    .await
    .map_err(map_db_error)?;

    let row = row.ok_or(DbError::NotFound)?;

    // ⚠️ Ordre = précédence. Ne pas réordonner sans réordonner la spec (D6).
    // ⚠️ Le NUMÉRO de la pièce accompagne son identifiant quand il existe : un
    // message qui dit « la facture F-2026-014 » se comprend, un `documentId: 47`
    // ne se comprend pas. Toutes les pièces n'en ont pas — un règlement et une
    // transaction bancaire n'ont que leur identifiant.
    let mut blockers: Vec<ReversalBlockerHit> = Vec::new();
    if row.reverses_entry_id.is_some() {
        blockers.push((ReversalBlocker::IsAReversal, row.reverses_entry_id, None));
    }
    if row.reversed_by.is_some() {
        blockers.push((ReversalBlocker::AlreadyReversed, row.reversed_by, None));
    }
    if row.invoice_id.is_some() {
        blockers.push((
            ReversalBlocker::OwnedByInvoice,
            row.invoice_id,
            row.invoice_number,
        ));
    }
    if row.credit_note_id.is_some() {
        blockers.push((
            ReversalBlocker::OwnedByCreditNote,
            row.credit_note_id,
            row.credit_note_number,
        ));
    }
    if row.supplier_invoice_id.is_some() {
        blockers.push((
            ReversalBlocker::OwnedBySupplierInvoice,
            row.supplier_invoice_id,
            row.supplier_invoice_number,
        ));
    }
    if row.settlement_id.is_some() {
        blockers.push((ReversalBlocker::OwnedBySettlement, row.settlement_id, None));
    }
    if row.bank_transaction_id.is_some() {
        blockers.push((
            ReversalBlocker::MatchedBankTransaction,
            row.bank_transaction_id,
            None,
        ));
    }
    if row.archived_account_number.is_some() {
        // ⛔ En dernier : c'est le seul motif que l'utilisateur peut lever
        // lui-même (réactiver le compte), et l'annoncer avant un motif de
        // propriété ferait croire qu'une facture deviendrait contre-passable.
        //
        // ⚠️ L'étiquette porte le **numéro du compte**, et `document_id` reste
        // `None` : un compte n'est pas une pièce. Sans ce numéro, l'écran dirait
        // « réactivez-LE » sans dire lequel, sur une écriture qui peut porter dix
        // lignes — le refus qui NOMME (le 400 de l'écriture) étant devenu
        // inatteignable depuis que le bouton est masqué. *(Passe 2 de revue.)*
        blockers.push((
            ReversalBlocker::AccountArchived,
            None,
            row.archived_account_number,
        ));
    }
    Ok(blockers)
}
