//! Annuler un rapprochement bancaire (Story 25-3-b, #418).
//!
//! ⛔ **Pourquoi ce geste existe.** Un rapprochement accepté ne se défaisait
//! pas : la contre-passation directe refuse une écriture rapprochée
//! (`ReversalBlocker::MatchedBankTransaction`), le gel de la 24-4b en interdit
//! la modification, et le règlement client né d'un rapprochement est refusé
//! par son propre geste d'annulation au rang 3. L'écriture était donc
//! **définitivement incorrigible**.
//!
//! ⛔ **Deux familles d'écritures, et seulement deux.** Des cinq chemins qui
//! posent `bank_transactions.matched_entry_id` (`routes/reconciliation.rs`),
//! un seul — l'acceptation d'une proposition « facture » — crée un
//! **règlement client** (ligne `invoice_settlements`) ; les quatre autres
//! (éclatement, règle, rapprochement manuel, éclatement manuel) créent une
//! écriture que **seule la transaction** possède. Le règlement se défait par
//! [`invoice_settlements_write::cancel_settlement_in_tx`], **appelé, jamais
//! réécrit** ; l'écriture propre, par [`journal_entries::reverse_in_tx`].
//!
//! ⛔ **Le lien d'abord, la contre-passation ensuite.** Remettre
//! `matched_entry_id` à `NULL` dans la même transaction, **avant** de
//! contre-passer, fait disparaître le motif `MatchedBankTransaction` du socle —
//! sans aucune autorité nouvelle. Et **jamais par la suppression** : la FK est
//! `ON DELETE SET NULL`, le lien s'effacerait en silence.
//!
//! **Les états qui n'existent pas** (arbitrage Q1 de Guy, 2026-09-25 : Kesh
//! n'est pas en production, aucune donnée n'est à préserver) n'ont **aucun
//! chemin** : un lien antérieur à la Story 24-2, qui pointe sur l'écriture de
//! **vente**, tombe dans le cas « écriture propre » et est refusé par le socle
//! (`OWNED_BY_INVOICE`) — rien n'est écrit ; une transaction `reconciled` sans
//! lien est refusée en [`DbError::Invariant`].

use sqlx::{MySqlConnection, MySqlPool};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::bank_transaction::{BankTransaction, BankTransactionStatus};
use crate::errors::{DbError, SettlementCancelBlocker, map_db_error};
use crate::repositories::settlement_cancellation::{
    SettlementCancelHit, settlement_entry_cancel_blocker,
};
use crate::repositories::{
    audit_log, bank_transactions, invoice_settlements_write, journal_entries,
};

/// Ce que possède l'écriture d'un rapprochement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationKind {
    /// Un **règlement client** (acceptation d'une proposition « facture »).
    InvoiceSettlement { invoice_id: i64, settlement_id: i64 },
    /// Une écriture que seule la transaction possède (éclatement, règle,
    /// rapprochement manuel).
    Entry,
}

impl ReconciliationKind {
    /// Code canonique, exposé par l'API et porté par l'audit.
    pub fn code(self) -> &'static str {
        match self {
            Self::InvoiceSettlement { .. } => "invoice_settlement",
            Self::Entry => "entry",
        }
    }
}

/// Classe l'écriture liée `entry_id` : un règlement client la porte-t-il ?
///
/// `for_update` : le geste la lit en **verrouillant** — cf. [`cancel_in_tx`],
/// étape 2 : une lecture non verrouillante à cet endroit figerait l'instantané
/// de la transaction AVANT le verrou de l'exercice.
async fn classify(
    conn: &mut MySqlConnection,
    company_id: i64,
    entry_id: i64,
    for_update: bool,
) -> Result<ReconciliationKind, DbError> {
    // `uq_invoice_settlements_entry` : une écriture règle UNE facture.
    let sql = format!(
        "SELECT id, invoice_id FROM invoice_settlements \
         WHERE company_id = ? AND journal_entry_id = ?{}",
        if for_update { " FOR UPDATE" } else { "" }
    );
    let row: Option<(i64, i64)> = sqlx::query_as(&sql)
        .bind(company_id)
        .bind(entry_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?;
    Ok(match row {
        Some((settlement_id, invoice_id)) => ReconciliationKind::InvoiceSettlement {
            invoice_id,
            settlement_id,
        },
        None => ReconciliationKind::Entry,
    })
}

/// L'écriture liée d'une transaction `reconciled` — ou l'invariant rompu.
fn matched_entry(bt: &BankTransaction) -> Result<i64, DbError> {
    bt.matched_entry_id.ok_or_else(|| {
        DbError::Invariant(format!(
            "transaction bancaire {} rapprochée sans écriture liée",
            bt.id
        ))
    })
}

/// Lit une transaction bancaire de la société, sans verrou.
async fn find(
    conn: &mut MySqlConnection,
    company_id: i64,
    bank_transaction_id: i64,
    for_update: bool,
) -> Result<BankTransaction, DbError> {
    let sql = format!(
        "SELECT {} FROM bank_transactions WHERE id = ? AND company_id = ?{}",
        bank_transactions::COLUMNS,
        if for_update { " FOR UPDATE" } else { "" }
    );
    sqlx::query_as::<_, BankTransaction>(&sql)
        .bind(bank_transaction_id)
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)
}

/// Le premier motif qui empêche de dé-rapprocher `bank_transaction_id`, ou
/// `None`.
///
/// ⛔ **Une seule précédence, pas de jumeau** : rang 0 ici
/// (`BankTransactionNotReconciled`) ; pour un règlement client, le rang 1 et la
/// queue par [`invoice_settlements_write::settlement_cancel_blocker_unlinking`] ;
/// pour une écriture propre, la queue commune directement. Dans les deux cas,
/// avec l'**exemption étroite** du rang 3 pour **cette** transaction — c'est
/// elle qu'on défait —, une autre transaction pointant la même écriture restant
/// un refus.
///
/// Sert **à la fois** la lecture (l'écran, au clic) et le geste
/// ([`cancel_in_tx`]), qui verrouille avant de l'appeler.
///
/// Transaction introuvable (ou d'une autre société) → [`DbError::NotFound`].
pub async fn cancel_blocker(
    conn: &mut MySqlConnection,
    company_id: i64,
    bank_transaction_id: i64,
) -> Result<Option<SettlementCancelHit>, DbError> {
    let bt = find(conn, company_id, bank_transaction_id, false).await?;
    blocker_for(conn, company_id, &bt).await
}

async fn blocker_for(
    conn: &mut MySqlConnection,
    company_id: i64,
    bt: &BankTransaction,
) -> Result<Option<SettlementCancelHit>, DbError> {
    if bt.status != BankTransactionStatus::Reconciled {
        return Ok(Some((
            SettlementCancelBlocker::BankTransactionNotReconciled,
            None,
            None,
        )));
    }
    let entry_id = matched_entry(bt)?;
    match classify(conn, company_id, entry_id, false).await? {
        ReconciliationKind::InvoiceSettlement { settlement_id, .. } => {
            invoice_settlements_write::settlement_cancel_blocker_unlinking(
                conn,
                company_id,
                settlement_id,
                Some(bt.id),
            )
            .await
        }
        ReconciliationKind::Entry => {
            settlement_entry_cancel_blocker(conn, company_id, entry_id, Some(bt.id)).await
        }
    }
}

/// Une transaction bancaire et ce qui décide de l'annulation de son
/// rapprochement — **lus dans un seul instantané** (la leçon de la revue de la
/// 25-3-a-2 : une réponse qui mêle deux lectures peut se contredire).
#[derive(Debug, Clone)]
pub struct ReconciliationView {
    pub bank_transaction: BankTransaction,
    /// `None` si la transaction n'est pas rapprochée.
    pub kind: Option<ReconciliationKind>,
    /// Le numéro de la facture réglée, pour un règlement client.
    pub invoice_number: Option<String>,
    /// Le premier motif qui refuse l'annulation, ou `None`.
    pub cancel_blocker: Option<SettlementCancelHit>,
}

/// Lit une transaction et sa vue d'annulation dans une transaction de lecture
/// (instantané `REPEATABLE READ`, aucune lecture verrouillante), ou `None` si
/// elle n'existe pas pour cette société.
pub async fn get_view(
    pool: &MySqlPool,
    company_id: i64,
    bank_transaction_id: i64,
) -> Result<Option<ReconciliationView>, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let bt = match find(&mut tx, company_id, bank_transaction_id, false).await {
        Ok(bt) => bt,
        Err(DbError::NotFound) => return Ok(None),
        Err(e) => return Err(e),
    };
    let (kind, invoice_number) = if bt.status == BankTransactionStatus::Reconciled {
        let kind = classify(&mut tx, company_id, matched_entry(&bt)?, false).await?;
        let number = match kind {
            ReconciliationKind::InvoiceSettlement { invoice_id, .. } => {
                sqlx::query_scalar::<_, Option<String>>(
                    "SELECT invoice_number FROM invoices WHERE id = ? AND company_id = ?",
                )
                .bind(invoice_id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?
                .flatten()
            }
            ReconciliationKind::Entry => None,
        };
        (Some(kind), number)
    } else {
        (None, None)
    };
    let cancel_blocker = blocker_for(&mut tx, company_id, &bt).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(Some(ReconciliationView {
        bank_transaction: bt,
        kind,
        invoice_number,
        cancel_blocker,
    }))
}

/// Ce que rend l'annulation d'un rapprochement.
#[derive(Debug, Clone)]
pub struct ReconciliationCancellation {
    /// La transaction relue, revenue à `pending`.
    pub bank_transaction: BankTransaction,
    /// L'écriture inverse, datée du jour.
    pub reversal_journal_entry_id: i64,
    /// La facture dont le règlement a été retiré, pour un règlement client.
    pub invoice_id: Option<i64>,
}

/// Annule le rapprochement de `bank_transaction_id` **dans une transaction
/// fournie**, sans `BEGIN` ni `COMMIT` (Story 25-3-b, #418).
///
/// Dans cet ordre, qui est porteur :
/// 1. verrou de la transaction bancaire ; non rapprochée → refus (rang 0) ;
/// 2. classement du lien par une lecture **verrouillante** de la ligne de
///    règlement, puis verrou de la facture. ⛔ **Aucune lecture non
///    verrouillante avant l'étape 3** : sous `REPEATABLE READ`, l'instantané se
///    fige à la PREMIÈRE d'entre elles, et une lecture faite ici figerait l'état
///    d'avant une clôture concurrente — le rang 2, relu après avoir attendu le
///    verrou de l'exercice, verrait encore l'exercice ouvert (défaut trouvé par
///    le test d'entrelacement, au développement). ⚠️ **Prix assumé** : la ligne
///    de règlement est verrouillée avant la facture, l'inverse de
///    `cancel_settlement_in_tx` ; les deux ne se croisent que sur le **même**
///    règlement annulé au même instant depuis la fiche facture et depuis
///    l'import — l'interblocage (1213) est alors rejoué par la route du
///    dé-rapprochement ; côté fiche facture, cf. #463 ;
/// 3. verrou de l'écriture **et de son exercice** avant de juger — sans lui,
///    une clôture validée entre la lecture du rang 2 et la contre-passation
///    passerait inaperçue (leçon de la revue de la 25-3-a-1) ;
/// 4. les motifs, par [`cancel_blocker`] **exempté** pour cette transaction :
///    le geste refuse lui-même les rangs 0 et 2
///    ([`DbError::ReconciliationNotCancellable`]) ; le rang 1 est refusé par
///    le geste du règlement, les rangs 3 à 5 par la contre-passation — une
///    seule garde par motif ;
/// 5. le lien défait, **avant** toute contre-passation, et le marqueur de
///    rejet remis à `NULL` — sans quoi la transaction ne reviendrait jamais
///    dans les propositions ;
/// 6. le règlement annulé, ou l'écriture contre-passée ;
/// 7. l'audit `reconciliation.cancelled`, par `for_actor` (clé API portée).
///
/// ⚠️ Au rang 1, `cancel_settlement_in_tx` refuse **après** que le lien a été
/// défait : l'erreur remonte, et le rollback de l'appelant rétablit le lien.
///
/// N'exécute **pas** de rollback : l'appelant, propriétaire de la
/// transaction, en est responsable.
pub async fn cancel_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_transaction_id: i64,
    user_id: i64,
    actor_api_key_id: Option<i64>,
) -> Result<ReconciliationCancellation, DbError> {
    // (1) Verrou de la transaction bancaire.
    let bt = find(tx, company_id, bank_transaction_id, true).await?;
    if bt.status != BankTransactionStatus::Reconciled {
        return Err(DbError::ReconciliationNotCancellable {
            blocker: SettlementCancelBlocker::BankTransactionNotReconciled,
        });
    }
    let entry_id = matched_entry(&bt)?;

    // (2) Classement par lecture VERROUILLANTE (la ligne de règlement), puis
    //     verrou de la facture. ⛔ Aucune lecture non verrouillante avant (3).
    let kind = classify(tx, company_id, entry_id, true).await?;
    if let ReconciliationKind::InvoiceSettlement { invoice_id, .. } = kind {
        sqlx::query("SELECT id FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE")
            .bind(invoice_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
    }

    // (3) ⛔ Verrou de l'écriture ET de son exercice, avant de juger.
    sqlx::query(
        "SELECT fy.id FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
    )
    .bind(entry_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)?;

    // (4) Les motifs — forme EXEMPTÉE : le lien existe encore, et la forme
    //     non exemptée refuserait chaque dé-rapprochement sur son propre lien.
    if let Some((SettlementCancelBlocker::FiscalYearClosed, _, _)) =
        blocker_for(tx, company_id, &bt).await?
    {
        return Err(DbError::ReconciliationNotCancellable {
            blocker: SettlementCancelBlocker::FiscalYearClosed,
        });
    }

    // (5) Le lien défait, AVANT la contre-passation.
    let updated = sqlx::query(
        "UPDATE bank_transactions \
         SET matched_entry_id = NULL, status = 'pending', auto_match_rejected_at = NULL, \
             version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ? AND status = 'reconciled' AND version = ?",
    )
    .bind(bank_transaction_id)
    .bind(company_id)
    .bind(bt.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if updated.rows_affected() != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (6) Le règlement annulé — geste de la 25-3-a-1, appelé —, ou l'écriture
    //     contre-passée.
    let (reversal_journal_entry_id, invoice_id, settlement_id) = match kind {
        ReconciliationKind::InvoiceSettlement {
            invoice_id,
            settlement_id,
        } => {
            let done = invoice_settlements_write::cancel_settlement_in_tx(
                tx,
                company_id,
                invoice_id,
                settlement_id,
                user_id,
            )
            .await?;
            (
                done.reversal_journal_entry_id,
                Some(invoice_id),
                Some(settlement_id),
            )
        }
        ReconciliationKind::Entry => {
            let reversal =
                journal_entries::reverse_in_tx(tx, company_id, entry_id, user_id).await?;
            (reversal.entry.id, None, None)
        }
    };

    // (7) L'audit du geste. ⚠️ Pour un règlement client, trois lignes au
    //     total (celle-ci, `invoice.settlement_cancelled`,
    //     `journal_entry.reversed`) : chacune nomme son objet.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor_api_key_id,
            "reconciliation.cancelled",
            "bank_transaction",
            bank_transaction_id,
            Some(serde_json::json!({
                "kind": kind.code(),
                "matchedEntryId": entry_id,
                "reversalJournalEntryId": reversal_journal_entry_id,
                "invoiceId": invoice_id,
                "settlementId": settlement_id,
                "amount": bt.amount,
                "wasPreviouslyRejected": bt.auto_match_rejected_at.is_some(),
            })),
        ),
    )
    .await?;

    let bank_transaction = find(tx, company_id, bank_transaction_id, false).await?;
    Ok(ReconciliationCancellation {
        bank_transaction,
        reversal_journal_entry_id,
        invoice_id,
    })
}

/// Annule un rapprochement, transaction comprise — mince enveloppement de
/// [`cancel_in_tx`], sans verrou de compte ni rejeu (réservés à la route).
pub async fn cancel(
    pool: &MySqlPool,
    company_id: i64,
    bank_transaction_id: i64,
    user_id: i64,
) -> Result<ReconciliationCancellation, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match cancel_in_tx(&mut tx, company_id, bank_transaction_id, user_id, None).await {
        Ok(done) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(done)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}
