//! Enregistrement manuel d'un règlement client — Story 24-3 (#372).
//!
//! ⛔ **Le gabarit est `supplier_invoices::pay_in_tx`**, comme pour la 24-2, et
//! son étape la moins évidente reste la même : le compte de créance se lit **sur
//! l'écriture de vente**, jamais sur les réglages. C'est ce qui garantit que le
//! compte se solde exactement, quoi qu'il soit arrivé à la configuration entre
//! l'émission de la facture et son règlement.
//!
//! ⚠️ **Le mode de règlement est indifférent au traitement comptable.** Espèces,
//! poste, compensation, virement : seule change la contrepartie. La distinction
//! « virement → écriture / espèces → simple marquage » n'est pas comptablement
//! fondée — ce que l'import CAMT automatise, c'est la *détection* du paiement,
//! pas son *enregistrement*.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::MySqlPool;

use crate::entities::NewAuditLogEntry;
use crate::entities::{
    Journal, NewInvoiceSettlement, NewJournalEntry, NewJournalEntryLine, SettlementChoice,
};
use crate::errors::{DbError, SettlementCancelBlocker, map_db_error};
use crate::repositories::journal_entries::ReversalAuthority;
use crate::repositories::settlement_cancellation::{
    SettlementCancelHit, settlement_entry_cancel_blocker,
};
use crate::repositories::{audit_log, fiscal_years, invoice_settlements, journal_entries};

/// Ce que rend un règlement enregistré : l'écriture créée, et le résiduel après.
#[derive(Debug, Clone)]
pub struct SettlementOutcome {
    pub journal_entry_id: i64,
    pub amount_due_after: Decimal,
    /// `true` ssi le résiduel est tombé à zéro — c'est ce qui pose `paid_at`.
    pub fully_settled: bool,
}

/// Enregistre un règlement manuel et passe son écriture.
///
/// ⛔ **Le trop-perçu est refusé, jamais écrit** : sinon le compte de créance
/// passerait créditeur, une anomalie que le grand livre signalerait — mais après
/// coup.
pub async fn settle_invoice(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    invoice_id: i64,
    choice: SettlementChoice,
    amount: Decimal,
    settled_on: NaiveDate,
) -> Result<SettlementOutcome, DbError> {
    if amount <= Decimal::ZERO {
        return Err(DbError::InvalidInput("amountMustBePositive".into()));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // (1) Verrou facture + garde de statut.
    let (status, sale_entry_id, project_id, invoice_number, invoice_date): (
        String,
        Option<i64>,
        Option<i64>,
        Option<String>,
        NaiveDate,
    ) = sqlx::query_as(
        "SELECT status, journal_entry_id, project_id, invoice_number, date FROM invoices \
         WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(invoice_id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .ok_or(DbError::NotFound)?;

    if status != "validated" {
        return Err(DbError::IllegalStateTransition(format!(
            "seule une facture validée peut être réglée (statut actuel : '{status}')"
        )));
    }
    let sale_entry_id = sale_entry_id
        .ok_or_else(|| DbError::Invariant("facture validée sans écriture de vente".into()))?;

    // ⛔ Un règlement ne PRÉCÈDE pas sa facture. Règle héritée de `mark_as_paid`,
    // que cette fonction remplace — et la seule de ses gardes qui reste vraie.
    //
    // ⚠️ La tolérance d'UN JOUR n'est pas de la complaisance : `settled_on` est
    // une date de valeur bancaire tandis que `invoice.date` est une date métier
    // locale, et l'écart de fuseau suffit à faire apparaître un règlement
    // « la veille » d'une facture émise le même jour.
    if settled_on < invoice_date - chrono::Duration::days(1) {
        return Err(DbError::InvalidInput("settledOnBeforeInvoiceDate".into()));
    }

    // (2) ⛔ Le compte de créance vient de l'écriture de vente. Miroir strict de
    //     l'étape (2) de `pay_in_tx`, qui lit la ligne de CRÉDIT de l'achat.
    let receivable_account_id: i64 = sqlx::query_scalar(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? AND jel.debit > 0 \
         ORDER BY jel.id LIMIT 1",
    )
    .bind(sale_entry_id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| DbError::Invariant("écriture de vente sans ligne de débit".into()))?;

    // (3) Contrepartie selon le mode.
    let counterparty_account_id = match choice {
        SettlementChoice::BankTransfer { bank_account_id } => {
            let journal_account_id: Option<Option<i64>> = sqlx::query_scalar(
                "SELECT journal_account_id FROM bank_accounts \
                 WHERE id = ? AND company_id = ? FOR UPDATE",
            )
            .bind(bank_account_id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
            journal_account_id
                .ok_or(DbError::NotFound)?
                .ok_or_else(|| {
                    DbError::ConfigurationRequired("bank_account.journal_account_id".into())
                })?
        }
        SettlementChoice::InternalAccount { account_id } => {
            // ⚠️ N'importe quel compte du plan — caisse, poste, compensation —
            // mais il doit être ACTIF **et IMPUTABLE**.
            //
            // `active` : régler sur un compte archivé produirait une écriture
            // qu'aucun écran ne montre plus.
            //
            // `postable` : ce flux passe `enforce_postable = false` plus bas, si
            // bien que la garde de la Story 14-3b ne s'applique PAS ici — c'est
            // donc ce SELECT, et lui seul, qui tient la promesse. Sans lui, un
            // compte de regroupement, le 2979 *Résultat de l'exercice* ou un
            // compte de clôture 9000/9100/9200 (Story 24-5) peut recevoir une
            // écriture : le choix vient du client, et l'écran le proposait.
            //
            // ⛔ Relevé en passe 3 de revue de code de la Story 24-5 (#375),
            // finding P3-1 — CRITICAL. Le défaut que la 24-5 ferme était
            // rouvert ICI, et par un geste ORDINAIRE : « Enregistrer un
            // règlement » → « Compte interne » → 9000. Ce n'était donc pas le
            // trou « API seulement » que le manuel décrivait. Les trois flux de
            // réconciliation, eux, restent ouverts et sont suivis par #427 —
            // mais aucun de leurs écrans n'offre ces comptes.
            let account: Option<(bool, bool)> = sqlx::query_as(
                "SELECT active, postable FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
            )
            .bind(account_id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
            match account {
                Some((true, true)) => account_id,
                _ => return Err(DbError::InactiveOrInvalidAccounts),
            }
        }
    };

    // (4) ⛔ Le trop-perçu est refusé AVANT toute écriture.
    let due_before = invoice_settlements::amount_due(&mut *tx, invoice_id).await?;
    if amount > due_before {
        return Err(DbError::InvalidInput(format!(
            "overpayment: amountDue={due_before}, amount={amount}"
        )));
    }

    // (5) Exercice OUVERT couvrant la date de règlement.
    let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, settled_on)
        .await?
        .ok_or(DbError::FiscalYearInvalid)?;

    // (6) `D contrepartie / C créance`.
    let label = invoice_number.unwrap_or_else(|| invoice_id.to_string());
    let journal = match choice {
        SettlementChoice::BankTransfer { .. } => Journal::Banque,
        // ⚠️ Le journal Caisse ne vaut que pour la caisse ; un compte interne
        // quelconque (compensation) relève des opérations diverses. On ne peut
        // pas le deviner du seul `account_id`, donc OD — neutre et exact.
        SettlementChoice::InternalAccount { .. } => Journal::OD,
    };
    let je = journal_entries::create_in_tx(
        &mut tx,
        fy.id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: settled_on,
            journal,
            description: format!("Règlement facture {label}"),
            project_id,
            lines: vec![
                NewJournalEntryLine {
                    account_id: counterparty_account_id,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: receivable_account_id,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
        // Flux automatique : garde de postabilité désactivée (14-3b, D-A0).
        false,
    )
    .await?;

    // (7) La liaison.
    invoice_settlements::create_in_tx(
        &mut tx,
        NewInvoiceSettlement {
            company_id,
            invoice_id,
            journal_entry_id: je.entry.id,
            amount,
            settled_on,
            choice,
        },
    )
    .await?;

    // (8) ⛔ `paid_at` est la PROJECTION du résiduel à zéro, pas un drapeau.
    let due_after = invoice_settlements::amount_due(&mut *tx, invoice_id).await?;
    let fully_settled = due_after <= Decimal::ZERO;
    if fully_settled {
        sqlx::query(
            "UPDATE invoices SET paid_at = ?, version = version + 1, updated_at = NOW(3) \
             WHERE id = ? AND company_id = ? AND status = 'validated'",
        )
        .bind(settled_on.and_hms_opt(0, 0, 0).expect("minuit est valide"))
        .bind(invoice_id)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;
    }

    // (9) ⛔ L'audit, dans la MÊME transaction. Un règlement est un fait
    //     comptable : s'il s'enregistre sans laisser de trace, la piste d'audit
    //     ment par omission — et l'action est nommée d'après son EFFET RÉEL, un
    //     règlement partiel n'étant pas `invoice.paid`.
    let action = if fully_settled {
        "invoice.paid"
    } else {
        "invoice.partially_settled"
    };
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry::user(
            user_id,
            action,
            "invoice",
            invoice_id,
            Some(serde_json::json!({
                "paid_via": "manual_settlement",
                "settlement_type": choice.type_str(),
                "settlement_journal_entry_id": je.entry.id,
                "settled_amount": amount,
                "settled_on": settled_on,
                "amount_due_after": due_after,
                "fully_settled": fully_settled,
            })),
        ),
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(SettlementOutcome {
        journal_entry_id: je.entry.id,
        amount_due_after: due_after,
        fully_settled,
    })
}

// ---------------------------------------------------------------------------
// Story 25-3-a-1 (#414) — annuler un règlement client
// ---------------------------------------------------------------------------

/// Ce qui empêche d'annuler le règlement `settlement_id` — la **tête** client
/// (rang 1), puis la queue commune sur son écriture (rangs 2 à 5).
///
/// ⛔ **Une seule fonction pour lire et pour écrire** : `GET …/settlements`
/// l'appelle pour masquer le bouton avant le clic, [`cancel_settlement_in_tx`]
/// pour refuser. Une lecture qui divergerait de l'écriture afficherait un
/// bouton qui échoue.
///
/// Règlement introuvable (ou d'une autre société) → [`DbError::NotFound`].
pub async fn settlement_cancel_blocker(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    settlement_id: i64,
) -> Result<Option<SettlementCancelHit>, DbError> {
    let row: Option<(i64, String)> = sqlx::query_as(
        "SELECT s.journal_entry_id, i.status FROM invoice_settlements s \
         JOIN invoices i ON i.id = s.invoice_id AND i.company_id = s.company_id \
         WHERE s.id = ? AND s.company_id = ?",
    )
    .bind(settlement_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let (entry_id, status) = row.ok_or(DbError::NotFound)?;

    // Rang 1. ⚠️ Une facture qui porte un règlement ne peut être que
    // `validated` ou `cancelled` : l'encaissement exige `validated`, la
    // dévalidation refuse une facture réglée, et `cancelled` ne naît en
    // production que de l'avoir. Hors `validated`, c'est donc un avoir.
    if status != "validated" {
        return Ok(Some((SettlementCancelBlocker::InvoiceCredited, None, None)));
    }
    settlement_entry_cancel_blocker(conn, company_id, entry_id).await
}

/// Ce que rend une annulation de règlement.
#[derive(Debug, Clone)]
pub struct SettlementCancellation {
    /// L'écriture inverse, datée du jour.
    pub reversal_journal_entry_id: i64,
    /// Le reste dû après l'annulation.
    pub amount_due_after: Decimal,
}

/// Annule un règlement client **dans une transaction fournie**, sans `BEGIN`
/// ni `COMMIT` (Story 25-3-a-1, #414).
///
/// Contre-passe l'écriture de règlement (datée du jour), **retire** la ligne
/// `invoice_settlements` (arbitrage : retirée, non marquée annulée — le
/// résiduel se calcule depuis cette table), projette `paid_at` et journalise.
///
/// ⛔ **Forme `_in_tx` publique, et rien qui présuppose l'appelant HTTP** : le
/// dé-rapprochement (25-3-b) l'appellera après avoir défait le lien bancaire
/// dans la même transaction — ce qui lève le rang 3 sans aucune exemption.
///
/// ⛔ **Qui refuse** : ce geste ne refuse lui-même que les rangs 1 et 2
/// ([`DbError::SettlementNotCancellable`]) ; les rangs 3 à 5 sont refusés par
/// la contre-passation, avec son erreur canonique (409 `EntryNotReversable`,
/// 400 qui nomme les comptes, 400 `FiscalYearInvalid`). Une seule garde par
/// motif : deux gardes du même motif masqueraient chacune la mutation de
/// l'autre.
///
/// N'exécute **pas** de rollback : l'appelant, propriétaire de la transaction,
/// en est responsable.
pub async fn cancel_settlement_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    invoice_id: i64,
    settlement_id: i64,
    user_id: i64,
) -> Result<SettlementCancellation, DbError> {
    // (1) Verrou facture — même ordre que `settle_invoice`.
    let locked: Option<i64> =
        sqlx::query_scalar("SELECT id FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE")
            .bind(invoice_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    locked.ok_or(DbError::NotFound)?;

    // (2) Verrou sur la ligne de règlement, scopée par société ET facture : un
    //     règlement d'une autre facture n'existe pas pour cette route.
    let settlement: Option<(i64, Decimal, NaiveDate)> = sqlx::query_as(
        "SELECT journal_entry_id, amount, settled_on FROM invoice_settlements \
         WHERE id = ? AND company_id = ? AND invoice_id = ? FOR UPDATE",
    )
    .bind(settlement_id)
    .bind(company_id)
    .bind(invoice_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    let (entry_id, amount, settled_on) = settlement.ok_or(DbError::NotFound)?;

    // (2-bis) ⛔ Verrou sur l'EXERCICE de l'écriture de règlement — et sur
    //     l'écriture elle-même, jointe. Sans lui, le rang 2 serait une lecture
    //     sans verrou : une clôture (`fiscal_years::close`, un simple `UPDATE`)
    //     validée entre cette lecture et la contre-passation passerait
    //     inaperçue, et l'on annulerait un règlement d'exercice clos. Le socle
    //     ne le rattraperait pas : il verrouille bien l'exercice de l'origine,
    //     mais APRÈS cette lecture, et n'en relit jamais le statut.
    //     Patron : `journal_entries::delete_in_tx`, qui verrouille écriture et
    //     exercice dans la même requête. *(Passe 1 de revue de code.)*
    //
    //     ⚠️ La lecture du rang 2, plus bas, reste une lecture sans verrou — et
    //     elle voit l'état juste : la clôture concurrente attend désormais ce
    //     verrou, et l'instantané de lecture de la transaction ne se fige qu'à
    //     sa première lecture non verrouillante, qui vient APRÈS.
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

    // (3) Les motifs du geste. Rangs 1-2 : refusés ici. Rangs 3-5 : laissés au
    //     socle, qui les refuse avec son erreur canonique.
    if let Some((
        blocker @ (SettlementCancelBlocker::InvoiceCredited
        | SettlementCancelBlocker::FiscalYearClosed),
        _,
        _,
    )) = settlement_cancel_blocker(tx, company_id, settlement_id).await?
    {
        return Err(DbError::SettlementNotCancellable { blocker });
    }

    // (4) La contre-passation, au titre de ce règlement et de lui seul.
    let reversal = journal_entries::reverse_owned_in_tx(
        tx,
        company_id,
        entry_id,
        user_id,
        ReversalAuthority::ClientSettlement { settlement_id },
    )
    .await?;

    // (5) Le retrait.
    sqlx::query("DELETE FROM invoice_settlements WHERE id = ? AND company_id = ?")
        .bind(settlement_id)
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // (6) ⛔ `paid_at` est la PROJECTION du résiduel : il retombe à `NULL` si
    //     le résiduel redevient positif, et seulement alors. Un résiduel resté
    //     ≤ 0 le laisse intact — branche défensive, que l'application ne
    //     produit pas (trop-perçu refusé ; facture créditée arrêtée au rang 1).
    //     ⚠️ `version` et `updated_at` bougent **toujours** : l'encaissement ne
    //     les bumpe qu'au solde, mais une annulation change toujours l'état.
    let due_after = invoice_settlements::amount_due(&mut **tx, invoice_id).await?;
    let sql = if due_after > Decimal::ZERO {
        "UPDATE invoices SET paid_at = NULL, version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ?"
    } else {
        "UPDATE invoices SET version = version + 1, updated_at = NOW(3) \
         WHERE id = ? AND company_id = ?"
    };
    sqlx::query(sql)
        .bind(invoice_id)
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    // (7) L'audit du GESTE. ⚠️ La contre-passation a déjà écrit
    //     `journal_entry.reversed` sur l'écriture : deux lignes, c'est voulu —
    //     l'une dit le fait comptable, l'autre le geste métier.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "invoice.settlement_cancelled",
            "invoice",
            invoice_id,
            Some(serde_json::json!({
                "settlementId": settlement_id,
                "settledAmount": amount,
                "settledOn": settled_on,
                "settlementJournalEntryId": entry_id,
                "reversalJournalEntryId": reversal.entry.id,
                "amountDueAfter": due_after,
            })),
        ),
    )
    .await?;

    Ok(SettlementCancellation {
        reversal_journal_entry_id: reversal.entry.id,
        amount_due_after: due_after,
    })
}

/// Annule un règlement client, transaction comprise — mince enveloppement de
/// [`cancel_settlement_in_tx`].
pub async fn cancel_settlement(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    invoice_id: i64,
    settlement_id: i64,
) -> Result<SettlementCancellation, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match cancel_settlement_in_tx(&mut tx, company_id, invoice_id, settlement_id, user_id).await {
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
