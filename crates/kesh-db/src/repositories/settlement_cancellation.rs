//! Ce qui empêche d'annuler un règlement — la **queue commune** (Story 25-3-a-1, #414).
//!
//! ⛔ **Pourquoi un module à part.** Annuler un règlement contre-passe son
//! écriture ; ce qui peut l'empêcher tient pour l'essentiel à cette **écriture**
//! — son exercice, son rapprochement bancaire, ses comptes, l'exercice du jour
//! où la contre-passation serait datée —, et non à la pièce qui la possède. Le
//! règlement client (ligne `invoice_settlements`) et le règlement fournisseur
//! (colonnes de `supplier_invoices`, Story 25-3-a-2) ont chacun leur **tête**
//! propre, mais partagent **cette** queue. Une seconde précédence pour le même
//! socle divergerait : la spec a déjà dû corriger une fois l'ordre des deux
//! derniers rangs.

use chrono::Utc;
use sqlx::MySqlConnection;

use crate::errors::{DbError, ReversalBlocker, SettlementCancelBlocker, map_db_error};
use crate::repositories::{fiscal_years, journal_entries};

/// Un motif d'annulation refusée : le motif, l'identifiant de la pièce qui le
/// porte (la transaction bancaire au rang 3), et son étiquette lisible (le
/// numéro du compte archivé au rang 4).
pub type SettlementCancelHit = (SettlementCancelBlocker, Option<i64>, Option<String>);

/// Les rangs 2 à 5 de [`SettlementCancelBlocker`], évalués sur l'**écriture de
/// règlement** `entry_id` — le premier qui s'applique, ou `None`.
///
/// ⚠️ **Rangs 4 et 5 dans l'ordre RÉEL du socle** : `reverse_in_tx` contrôle
/// les comptes archivés (étape 3) avant l'exercice du jour (étape 4). Les
/// annoncer dans l'autre ordre ferait dire un motif à l'écran et en refuser un
/// autre au clic.
///
/// ⚠️ **Limite assumée** : le verrou de période **du jour** (24-4c) n'est pas
/// évalué ici — il se contrôle au clic (`PERIOD_LOCKED`, 400). Une borne ne
/// peut pas être future ; seule une borne égale au jour l'atteindrait.
///
/// **Lecture seule** : aucun verrou n'est posé. L'écriture (le geste) verrouille
/// la facture puis le règlement avant d'appeler cette fonction, et le socle
/// verrouille l'écriture et l'exercice du jour.
///
/// Écriture introuvable (ou d'une autre société) → [`DbError::NotFound`].
pub async fn settlement_entry_cancel_blocker(
    conn: &mut MySqlConnection,
    company_id: i64,
    entry_id: i64,
) -> Result<Option<SettlementCancelHit>, DbError> {
    // Rang 2 — l'exercice de l'écriture de RÈGLEMENT (non celui du jour).
    // Patron : `journal_entries::delete_in_tx`, qui lit `fy.status` joint par
    // `fiscal_year_id`.
    let status: Option<String> = sqlx::query_scalar(
        "SELECT fy.status FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ?",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let status = status.ok_or(DbError::NotFound)?;
    if status == "Closed" {
        return Ok(Some((
            SettlementCancelBlocker::FiscalYearClosed,
            None,
            None,
        )));
    }

    // Rangs 3 et 4 — lus dans le recensement du socle, pour ne pas en écrire
    // un second. ⚠️ Les motifs de PROPRIÉTÉ (règlement, facture fournisseur…)
    // y figurent aussi : ils sont ignorés ici, c'est l'autorité du geste qui
    // les lève.
    let blockers = journal_entries::reversal_blockers(&mut *conn, company_id, entry_id).await?;
    if let Some((_, bank_transaction_id, _)) = blockers
        .iter()
        .find(|(b, _, _)| *b == ReversalBlocker::MatchedBankTransaction)
    {
        return Ok(Some((
            SettlementCancelBlocker::MatchedBankTransaction,
            *bank_transaction_id,
            None,
        )));
    }
    if let Some((_, _, account_number)) = blockers
        .iter()
        .find(|(b, _, _)| *b == ReversalBlocker::AccountArchived)
    {
        return Ok(Some((
            SettlementCancelBlocker::AccountArchived,
            None,
            account_number.clone(),
        )));
    }

    // Rang 5 — l'exercice où la contre-passation serait DATÉE : le jour, comme
    // dans le socle.
    let today = Utc::now().date_naive();
    if !fiscal_years::has_open_covering_date(&mut *conn, company_id, today).await? {
        return Ok(Some((
            SettlementCancelBlocker::NoOpenFiscalYearToday,
            None,
            None,
        )));
    }

    Ok(None)
}
