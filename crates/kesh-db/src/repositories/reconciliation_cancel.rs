//! Annuler un rapprochement bancaire (Story 25-3-b, #418).
//!
//! ⛔ **Pourquoi ce geste existe.** Un rapprochement accepté ne se défaisait
//! pas : la contre-passation directe refuse une écriture rapprochée
//! (`ReversalBlocker::MatchedBankTransaction`), la même garde en interdit la
//! modification (Story 15-8a — une écriture rapprochée reste non modifiable,
//! `MATCHED_BANK_TRANSACTION`), et le règlement client né d'un rapprochement est refusé
//! par son propre geste d'annulation au rang 3. L'écriture était donc
//! **définitivement incorrigible**.
//!
//! ⛔ **Trois familles d'écritures, et seulement trois.** Des sept chemins qui
//! posent `bank_transactions.matched_entry_id`, deux rendent un **règlement
//! client** (ligne `invoice_settlements`) : l'acceptation d'une proposition
//! « facture » (`routes/reconciliation.rs`), qui le crée, et le rattachement
//! d'une transaction au règlement déjà saisi depuis la fiche de la facture
//! (Story 29-8, [`super::reconciliation_link::link_in_tx`]), qui pointe son
//! écriture existante. Un seul rend un **règlement fournisseur** : l'acceptation
//! d'une proposition « facture fournisseur » (Story 30-1b), qui le crée. Les
//! quatre autres (éclatement, règle, rapprochement manuel, éclatement manuel)
//! créent une écriture que **seule la transaction** possède. Le règlement
//! client se défait par [`invoice_settlements_write::cancel_settlement_in_tx`],
//! **appelé, jamais réécrit** ; l'écriture propre, par
//! [`journal_entries::reverse_in_tx`].
//!
//! ⛔ **La troisième : le règlement fournisseur** (Story 30-1a, #686) — une
//! écriture de règlement que pointent les colonnes `settlement_*` d'une facture
//! fournisseur **payée**. Le seul chemin qui la rapproche est l'acceptation
//! d'une facture fournisseur pour un débit (Story 30-1b) ; ce geste a su la
//! défaire **d'abord** (30-1a), pour qu'à aucun commit un tel rapprochement ne
//! soit produisible sans être défaisable. Elle se défait par
//! [`supplier_invoices::cancel_settlement_in_tx`], **appelé, jamais réécrit**.
//! Un règlement **détaché** (facture payée puis annulée, colonnes à `NULL`) n'a
//! plus de propriétaire : il est classé écriture propre et contre-passé.
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
    audit_log, bank_transactions, invoice_settlements_write, journal_entries, supplier_invoices,
};

/// Ce que possède l'écriture d'un rapprochement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReconciliationKind {
    /// Un **règlement client** (acceptation d'une proposition « facture », ou
    /// rattachement d'une transaction à un règlement saisi depuis la fiche —
    /// Story 29-8).
    InvoiceSettlement { invoice_id: i64, settlement_id: i64 },
    /// Un **règlement fournisseur** : l'écriture de règlement d'une facture
    /// fournisseur `paid` (Story 30-1a, #686). Il n'y a pas de seconde ligne :
    /// la facture **est** la ligne de règlement.
    SupplierSettlement { supplier_invoice_id: i64 },
    /// Une écriture que seule la transaction possède (éclatement, règle,
    /// rapprochement manuel) — ou un règlement fournisseur **détaché** de sa
    /// facture annulée.
    Entry,
}

impl ReconciliationKind {
    /// Code canonique, exposé par l'API et porté par l'audit.
    pub fn code(self) -> &'static str {
        match self {
            Self::InvoiceSettlement { .. } => "invoice_settlement",
            Self::SupplierSettlement { .. } => "supplier_settlement",
            Self::Entry => "entry",
        }
    }
}

/// La lecture qui reconnaît un **règlement fournisseur** (Story 30-1a, AC-a1),
/// sans son éventuel ` FOR UPDATE` — publique pour le test de son plan
/// (`EXPLAIN`, T-a12).
///
/// ⚠️ **Le plan doit passer par l'index de la clé étrangère**
/// `fk_supplier_invoices_settlement_je` : en forme verrouillante, un parcours
/// par `idx_supplier_invoices_company_status` verrouillerait **toutes** les
/// factures payées de la société. Le test du plan l'asserte.
///
/// `status = 'paid'` : une facture qui quitte `paid` remet la colonne à `NULL`
/// (annulation du règlement, annulation de la facture) — le filtre ne fait que
/// le redire, et un règlement détaché reste ainsi une écriture propre.
pub const SUPPLIER_SETTLEMENT_CLASSIFY_SQL: &str = "SELECT id FROM supplier_invoices \
     WHERE company_id = ? AND settlement_journal_entry_id = ? AND status = 'paid'";

/// Classe l'écriture liée `entry_id` : un règlement client la porte-t-il, puis
/// un règlement fournisseur ? Sinon, c'est une écriture propre.
///
/// `for_update` : le geste la lit en **verrouillant** — cf. [`cancel_in_tx`],
/// étape 2 : une lecture non verrouillante à cet endroit figerait l'instantané
/// de la transaction AVANT le verrou de l'exercice. Les deux bras ont la
/// **même** forme (verrouillante ou non).
///
/// ⚠️ **Ordre des bras, et son coût** (Story 30-1a, AC-a1, AC-a4) : règlement
/// client, puis règlement fournisseur, puis écriture propre. Une écriture ne
/// peut pas être à la fois l'une et l'autre ; l'ordre ne change que
/// l'empreinte des verrous. Le bras fournisseur n'est lu qu'après un échec du
/// bras client, donc pour un rapprochement fournisseur ou une écriture propre ;
/// sur une écriture propre, la lecture verrouillante qui ne trouve rien pose un
/// **verrou d'intervalle** sur `fk_supplier_invoices_settlement_je` — même
/// coût que la lecture de `invoice_settlements` sur une écriture propre. Une
/// écriture propre récente (identifiant supérieur à tous les
/// `settlement_journal_entry_id`) tient l'intervalle de queue : un `pay_in_tx`
/// concurrent (étape 6, `UPDATE … SET settlement_journal_entry_id = <écriture
/// neuve>`) **attend** la fin du dé-rapprochement. Cycle possible avec le
/// verrou de l'exercice (`pay_in_tx` le tient par
/// `fiscal_years::find_open_covering_date … FOR UPDATE` avant son `UPDATE` ; le
/// dé-rapprochement tient l'intervalle avant l'exercice, étape 3) : **rejoué
/// des deux côtés** — `retry_app_on_deadlock` pour le dé-rapprochement,
/// `retry_on_deadlock` pour le paiement depuis la fiche et pour la
/// confirmation d'un lot.
///
/// ⚠️ **Second cycle, constaté au développement** (Story 30-1a, T-a6,
/// C-30-1-42) : la lecture verrouillante passe par l'index secondaire et tient
/// son enregistrement **avant** la ligne de la facture. Un geste qui tient la
/// facture par sa **clé primaire** puis réécrit `settlement_journal_entry_id`
/// — l'annulation de la facture (`supplier_invoices::cancel_in_tx`, qui
/// détache le règlement) — demande ensuite cet enregistrement : `1213`. C'est
/// le prix déjà assumé côté client (étape 2 de [`cancel_in_tx`] : la ligne de
/// règlement avant la facture), et le même remède : les deux routes
/// **rejouent**, et l'issue rejouée est cohérente — dé-rapprochement puis
/// annulation d'une facture `open`, ou annulation puis dé-rapprochement d'un
/// règlement détaché (écriture propre).
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
    if let Some((settlement_id, invoice_id)) = row {
        return Ok(ReconciliationKind::InvoiceSettlement {
            invoice_id,
            settlement_id,
        });
    }
    // Story 30-1a (#686, AC-a1) — le règlement fournisseur, dans la MÊME forme.
    let sql = format!(
        "{SUPPLIER_SETTLEMENT_CLASSIFY_SQL}{}",
        if for_update { " FOR UPDATE" } else { "" }
    );
    let supplier: Option<i64> = sqlx::query_scalar(&sql)
        .bind(company_id)
        .bind(entry_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?;
    Ok(match supplier {
        Some(supplier_invoice_id) => ReconciliationKind::SupplierSettlement {
            supplier_invoice_id,
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

/// Le premier motif qui empêche de dé-rapprocher la transaction `bt`, ou
/// `None`.
///
/// ⛔ **Une seule précédence, pas de jumeau** : rang 0 ici
/// (`BankTransactionNotReconciled`) ; pour un règlement client, le rang 1 et la
/// queue par [`invoice_settlements_write::settlement_cancel_blocker_unlinking`] ;
/// pour un règlement fournisseur (Story 30-1a), le rang 1 et la queue par
/// [`supplier_invoices::supplier_settlement_cancel_blocker_unlinking`] ;
/// pour une écriture propre, la queue commune directement. Dans les trois cas,
/// avec l'**exemption étroite** du rang 3 pour **cette** transaction — c'est
/// elle qu'on défait —, une autre transaction pointant la même écriture restant
/// un refus.
///
/// ⛔ **La seule fonction qui calcule les motifs** : [`get_view`] l'appelle pour
/// la lecture (l'écran, au clic), [`cancel_in_tx`] pour refuser, sur la
/// transaction qu'il vient de verrouiller. Elle prend la transaction déjà lue,
/// pour que chaque appelant choisisse sa lecture (verrouillante ou non).
async fn cancel_blocker(
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
        ReconciliationKind::SupplierSettlement {
            supplier_invoice_id,
        } => {
            supplier_invoices::supplier_settlement_cancel_blocker_unlinking(
                conn,
                company_id,
                supplier_invoice_id,
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
    /// Le numéro (celui du fournisseur, libre) de la facture fournisseur
    /// réglée, pour un règlement fournisseur (Story 30-1a).
    pub supplier_invoice_number: Option<String>,
    /// Le premier motif qui refuse l'annulation, ou `None`.
    pub cancel_blocker: Option<SettlementCancelHit>,
}

/// Le numéro (nullable) d'une pièce, par `sql` (`… WHERE id = ? AND
/// company_id = ?`), dans l'instantané de la vue.
async fn number_of(
    conn: &mut MySqlConnection,
    sql: &str,
    id: i64,
    company_id: i64,
) -> Result<Option<String>, DbError> {
    Ok(sqlx::query_scalar::<_, Option<String>>(sql)
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut *conn)
        .await
        .map_err(map_db_error)?
        .flatten())
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
    let (kind, invoice_number, supplier_invoice_number) =
        if bt.status == BankTransactionStatus::Reconciled {
            let kind = classify(&mut tx, company_id, matched_entry(&bt)?, false).await?;
            match kind {
                ReconciliationKind::InvoiceSettlement { invoice_id, .. } => {
                    let number = number_of(
                        &mut tx,
                        "SELECT invoice_number FROM invoices WHERE id = ? AND company_id = ?",
                        invoice_id,
                        company_id,
                    )
                    .await?;
                    (Some(kind), number, None)
                }
                ReconciliationKind::SupplierSettlement {
                    supplier_invoice_id,
                } => {
                    let number = number_of(
                        &mut tx,
                        "SELECT supplier_invoice_number FROM supplier_invoices \
                         WHERE id = ? AND company_id = ?",
                        supplier_invoice_id,
                        company_id,
                    )
                    .await?;
                    (Some(kind), None, number)
                }
                ReconciliationKind::Entry => (Some(kind), None, None),
            }
        } else {
            (None, None, None)
        };
    let blocker = cancel_blocker(&mut tx, company_id, &bt).await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(Some(ReconciliationView {
        bank_transaction: bt,
        kind,
        invoice_number,
        supplier_invoice_number,
        cancel_blocker: blocker,
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
    /// La facture fournisseur dont le paiement a été retiré, pour un règlement
    /// fournisseur (Story 30-1a).
    pub supplier_invoice_id: Option<i64>,
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
///    dé-rapprochement, et côté fiche facture par celle de l'annulation du
///    règlement (Story 15-5e1, #463). **Règlement fournisseur** (Story 30-1a,
///    AC-a4) : la lecture verrouillante du classement verrouille la ligne
///    `supplier_invoices` — c'est **la** ligne de règlement du fournisseur, il
///    n'y en a pas de seconde à verrouiller ensuite —, par l'index de
///    `settlement_journal_entry_id` ; la facture est ainsi tenue **avant**
///    l'écriture, comme dans `supplier_invoices::cancel_settlement_in_tx`
///    (facture → écriture) : même ordre que le geste appelé ;
/// 3. verrou de l'écriture **et de son exercice** avant de juger — sans lui,
///    une clôture validée entre la lecture du rang 2 et la contre-passation
///    passerait inaperçue (leçon de la revue de la 25-3-a-1) ;
/// 4. les motifs, par [`cancel_blocker`] **exempté** pour cette transaction :
///    le geste refuse lui-même les rangs 0, 2 et 2 bis — le lettrage de pièce
///    figé par la période, Story 15-1a2-0
///    ([`DbError::ReconciliationNotCancellable`]) ; le rang 1 est refusé par
///    le geste du règlement, les rangs 3 à 5 par la contre-passation — une
///    seule garde par motif ;
/// 5. le lien défait, **avant** toute contre-passation, et le marqueur de
///    rejet remis à `NULL` — sans quoi la transaction ne reviendrait jamais
///    dans les propositions ;
/// 6. le règlement annulé — client par
///    [`invoice_settlements_write::cancel_settlement_in_tx`], fournisseur par
///    [`supplier_invoices::cancel_settlement_in_tx`] (Story 30-1a), **appelés,
///    jamais réécrits** —, ou l'écriture contre-passée ;
///
///    6 bis. la revérification **verrouillante** du compte clos
///    ([`super::bank_accounts::recheck_open_locked_in_tx`], Story 29-4b, #651),
///    **dernière acquisition de verrou** : un partagé sur la ligne du compte,
///    qui attend une clôture en cours. La route l'exécute sous le verrou nommé
///    du compte (`with_account_lock`, `GET_LOCK`) : cette attente se fait
///    **en gardant** `GET_LOCK`, si bien que les autres gestes de
///    réconciliation du même compte échouent pendant ce temps en
///    `ReconciliationAccountLocked` au bout du délai du verrou nommé ;
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
    //     verrou de la facture client. ⛔ Aucune lecture non verrouillante
    //     avant (3). Pour un règlement fournisseur (Story 30-1a), la ligne
    //     verrouillée par le classement EST la facture : rien de plus à tenir.
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

    // Story 28-4a (#642, W9) — compte clos : lecture non verrouillante APRÈS
    // (3), avant les motifs de (4) — « aucune lecture non verrouillante avant
    // l'étape 3 » (doc ci-dessus) ; règle de placement de
    // `bank_accounts::ensure_open_in_tx`. Ne lit pas `archived`. Revérifiée sous
    // verrou après l'étape (6) (Story 29-4b, #651).
    super::bank_accounts::ensure_open_in_tx(tx, company_id, bt.bank_account_id).await?;

    // (4) Les motifs — forme EXEMPTÉE : le lien existe encore, et la forme
    //     non exemptée refuserait chaque dé-rapprochement sur son propre lien.
    //     Motif LIÉ (`blocker @`), jamais écrit en dur dans l'erreur : le rang
    //     2 bis (Story 15-1a2-0) est refusé ici, dans SA famille, AVANT que le
    //     lien soit défait — sans lui, il remonterait de
    //     `cancel_settlement_in_tx` sous le texte « ce règlement ».
    if let Some((
        blocker @ (SettlementCancelBlocker::FiscalYearClosed
        | SettlementCancelBlocker::DocumentLetteringInClosedPeriods),
        _,
        _,
    )) = cancel_blocker(tx, company_id, &bt).await?
    {
        return Err(DbError::ReconciliationNotCancellable { blocker });
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

    // (6) Le règlement annulé — geste de la 25-3-a-1 (client) ou de la
    //     25-3-a-2 (fournisseur, Story 30-1a), appelé —, ou l'écriture
    //     contre-passée.
    let supplier_invoice_id = match kind {
        ReconciliationKind::SupplierSettlement {
            supplier_invoice_id,
        } => Some(supplier_invoice_id),
        _ => None,
    };
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
        ReconciliationKind::SupplierSettlement {
            supplier_invoice_id: id,
        } => {
            let done =
                supplier_invoices::cancel_settlement_in_tx(tx, company_id, id, user_id).await?;
            (done.reversal_journal_entry_id, None, None)
        }
        ReconciliationKind::Entry => {
            let reversal =
                journal_entries::reverse_in_tx(tx, company_id, entry_id, user_id).await?;
            (reversal.entry.id, None, None)
        }
    };

    // (6 bis) Story 29-4b (#651, W9, AC6) — revérification VERROUILLANTE du
    //     compte clos (`closed_on` seul, jamais `archived`), en DERNIÈRE
    //     acquisition de verrou : après toutes les écritures (le règlement
    //     annulé, dont la revérification de W25 sur le compte du règlement —
    //     le même pour un règlement né d'un rapprochement : redemande d'un
    //     partagé tenu —, ou la contre-passation), avant l'audit. ⚠️ Limite :
    //     « le même compte » tient à la construction de l'acceptation
    //     (`routes/reconciliation.rs`, `SettlementChoice::BankTransfer` avec le
    //     compte de l'import), non à une contrainte du schéma. Un règlement
    //     dont le compte différerait de celui de la transaction (donnée
    //     héritée ou restaurée — le code ne le produit pas) ferait prendre
    //     deux partagés neufs sur deux comptes : la revérification de W25 ne
    //     serait plus la dernière acquisition, et un cycle à quatre (deux
    //     annulations croisées, deux clôtures en file) deviendrait possible,
    //     dont les clôtures, les plus légères, seraient victimes et rejouées
    //     (raisonné, non joué). Règlement fournisseur (Story 30-1a) : même
    //     limite — la revérification de W29 porte sur
    //     `settlement_bank_account_id`, le compte de l'import pour
    //     l'acceptation de la 30-1b (même construction) ; avant elle, seul un
    //     état posé par un test rapproche un règlement fournisseur.
    //     Une clôture commitée depuis la naissance de la vue (garde de la 28-4)
    //     est vue ici ; une clôture qui attend encore voit, à son tour, la
    //     transaction redevenue `pending`. ⚠️ La route exécute ce geste sous le
    //     verrou nommé du compte (`with_account_lock`) : pendant qu'une clôture
    //     en file attend derrière un écrivain long, cette revérification attend
    //     **en gardant** `GET_LOCK`, et les autres gestes de réconciliation du
    //     compte échouent en `ReconciliationAccountLocked` au bout du délai.
    //     Aucun cycle mixte (raisonné) : aucun détenteur d'un partagé du compte
    //     ne demande ensuite `GET_LOCK`, les routes de réconciliation le prenant
    //     en tête. Voir `bank_accounts::recheck_open_locked_in_tx` (dont son
    //     angle mort `innodb_snapshot_isolation`).
    super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bt.bank_account_id).await?;

    // (7) L'audit du geste. ⚠️ Pour un règlement client, trois lignes au
    //     total (celle-ci, `invoice.settlement_cancelled`,
    //     `journal_entry.reversed`) : chacune nomme son objet. Pour un
    //     règlement fournisseur (Story 30-1a, AC-a5), `supplierInvoiceId` est
    //     posé, `invoiceId` et `settlementId` restent nuls — contrat additif ;
    //     le geste appelé écrit `supplier_invoice.settlement_cancelled`.
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
                "supplierInvoiceId": supplier_invoice_id,
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
        supplier_invoice_id,
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
