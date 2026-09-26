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
/// **Lecture seule** : aucun verrou n'est posé ici. ⛔ **Un geste qui s'en sert
/// pour REFUSER doit donc verrouiller avant** — la pièce, puis l'écriture de
/// règlement **et son exercice** (`… JOIN fiscal_years … FOR UPDATE`) : sans ce
/// dernier verrou, une clôture validée entre la lecture du rang 2 et la
/// contre-passation passerait inaperçue, le socle ne verrouillant l'exercice de
/// l'origine qu'APRÈS (passe 1 de revue de code, 25-3-a-1). Patron :
/// `invoice_settlements_write::cancel_settlement_in_tx`, étape 2-bis.
///
/// ⛔ **L'exemption étroite du rang 3** (`unlinking`, Story 25-3-b) : le
/// dé-rapprochement de la transaction `unlinking` défait **ce** lien-là —
/// il ne peut donc pas en être empêché. Le rang 3 ne tient alors que si une
/// **autre** transaction pointe la même écriture, lue par une requête dédiée :
/// l'identifiant que rend `reversal_blockers` sort d'un `LIMIT 1` **sans
/// `ORDER BY`**, et « c'est celui qu'on défait » ne prouverait pas qu'il n'y en
/// a pas d'autre. Les annulations de règlement passent `None` : leur
/// comportement est inchangé.
///
/// Écriture introuvable (ou d'une autre société) → [`DbError::NotFound`].
pub async fn settlement_entry_cancel_blocker(
    conn: &mut MySqlConnection,
    company_id: i64,
    entry_id: i64,
    unlinking: Option<i64>,
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
    let matched = match unlinking {
        None => blockers
            .iter()
            .find(|(b, _, _)| *b == ReversalBlocker::MatchedBankTransaction)
            .map(|(_, bank_transaction_id, _)| *bank_transaction_id),
        Some(unlinked) => sqlx::query_scalar::<_, i64>(
            "SELECT id FROM bank_transactions \
             WHERE company_id = ? AND matched_entry_id = ? AND id <> ? ORDER BY id LIMIT 1",
        )
        .bind(company_id)
        .bind(entry_id)
        .bind(unlinked)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .map(Some),
    };
    if let Some(bank_transaction_id) = matched {
        return Ok(Some((
            SettlementCancelBlocker::MatchedBankTransaction,
            bank_transaction_id,
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
