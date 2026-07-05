//! Repository des lots de paiement pain.001 (mode virement) — Story 12.3 (#191).
//!
//! Flux deux temps :
//! - [`create_batch`] : valide chaque facture (pattern batch `FailedProposal`),
//!   verrouille (`SELECT … FOR UPDATE` sur `supplier_invoices`), persiste le lot
//!   `generated` + ses items. AUCUNE écriture comptable. Génère le fichier
//!   pain.001 à la demande via [`generate_pain001_xml`].
//! - [`confirm_batch`] : poste les écritures de règlement de TOUTES les factures
//!   du lot dans UNE transaction atomique (réutilise `supplier_invoices::pay_in_tx`,
//!   guard-free), passe le lot `confirmed`. Pré-check du compte source archivé.
//! - [`cancel_batch`] : passe le lot `cancelled` (items conservés), déverrouille.

use rust_decimal::Decimal;
use sqlx::MySqlPool;

use crate::entities::{
    NewAuditLogEntry, NewPaymentBatch, PaymentBatch, PaymentBatchItem, SettlementChoice,
};
use crate::errors::{DbError, map_db_error};
use crate::repositories::{audit_log, supplier_invoices};

/// Échec per-facture lors de la génération d'un lot (pattern `FailedProposal`).
#[derive(Debug, Clone)]
pub struct PaymentBatchFailedItem {
    pub supplier_invoice_id: i64,
    pub error_code: String,
    pub details: Option<serde_json::Value>,
}

/// Lot + ses items (résultat de lecture / création).
#[derive(Debug, Clone)]
pub struct PaymentBatchWithItems {
    pub batch: PaymentBatch,
    pub items: Vec<PaymentBatchItem>,
}

/// Résultat de `create_batch` : le lot créé (si au moins une facture acceptée)
/// + la liste des factures refusées per-facture.
#[derive(Debug, Clone)]
pub struct CreateBatchOutcome {
    pub batch: Option<PaymentBatchWithItems>,
    pub failed: Vec<PaymentBatchFailedItem>,
}

const BATCH_COLS: &str = "id, company_id, bank_account_id, status, requested_execution_date, \
    total_amount, msg_id, payment_info_id, confirmed_at, version, created_at, updated_at";
const ITEM_COLS: &str = "id, payment_batch_id, supplier_invoice_id, position, end_to_end_id, \
    amount, created_at";

/// Coordonnées d'une facture acceptée, retenues pour l'INSERT des items.
struct Accepted {
    supplier_invoice_id: i64,
    amount: Decimal,
}

async fn fetch_items(pool: &MySqlPool, batch_id: i64) -> Result<Vec<PaymentBatchItem>, DbError> {
    sqlx::query_as::<_, PaymentBatchItem>(&format!(
        "SELECT {ITEM_COLS} FROM payment_batch_items WHERE payment_batch_id = ? ORDER BY position"
    ))
    .bind(batch_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Crée un lot de paiement (génération). Pattern batch : chaque facture est
/// validée indépendamment ; les valides forment le lot `generated`.
pub async fn create_batch(
    pool: &MySqlPool,
    new: NewPaymentBatch,
    user_id: i64,
) -> Result<CreateBatchOutcome, DbError> {
    let NewPaymentBatch {
        company_id,
        bank_account_id,
        requested_execution_date,
        supplier_invoice_ids,
    } = new;

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (0) Compte bancaire source : existe, company-scoped, non archivé, journal lié.
        let bank: Option<(Option<i64>, bool)> = sqlx::query_as(
            "SELECT journal_account_id, archived FROM bank_accounts \
             WHERE id = ? AND company_id = ?",
        )
        .bind(bank_account_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;
        let (journal_account_id, archived) = bank.ok_or(DbError::NotFound)?;
        if archived {
            return Err(DbError::IllegalStateTransition(
                "le compte bancaire source est archivé".into(),
            ));
        }
        if journal_account_id.is_none() {
            return Err(DbError::ConfigurationRequired(
                "bank_account.journal_account_id".into(),
            ));
        }

        // (1) Valider chaque facture (FailedProposal per-facture).
        let mut accepted: Vec<Accepted> = Vec::new();
        let mut failed: Vec<PaymentBatchFailedItem> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for sid in supplier_invoice_ids {
            if !seen.insert(sid) {
                continue; // doublon dans la requête : ignoré silencieusement.
            }
            match validate_invoice_for_batch(&mut tx, company_id, sid).await? {
                Ok(acc) => accepted.push(acc),
                Err(f) => failed.push(f),
            }
        }

        if accepted.is_empty() {
            // Rien à payer → pas de lot. Rollback (les verrous tombent).
            return Ok(CreateBatchOutcome { batch: None, failed });
        }

        // (2) INSERT lot (msg_id/payment_info_id calculés depuis l'id, ≤35).
        let total: Decimal = accepted.iter().map(|a| a.amount).sum();
        let batch_id: i64 = sqlx::query(
            "INSERT INTO payment_batches \
             (company_id, bank_account_id, status, requested_execution_date, total_amount, \
              msg_id, payment_info_id) VALUES (?, ?, 'generated', ?, ?, '', '')",
        )
        .bind(company_id)
        .bind(bank_account_id)
        .bind(requested_execution_date)
        .bind(total)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .last_insert_id() as i64;

        let msg_id = format!("KESH-{batch_id:012}");
        let payment_info_id = format!("PMT-{batch_id:012}");
        sqlx::query("UPDATE payment_batches SET msg_id = ?, payment_info_id = ? WHERE id = ?")
            .bind(&msg_id)
            .bind(&payment_info_id)
            .bind(batch_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;

        // (3) INSERT items (end_to_end_id ≤35, persisté pour reproduction fidèle).
        for (position, acc) in accepted.iter().enumerate() {
            let e2e = format!("PAY-{batch_id:08}-{:08}", acc.supplier_invoice_id);
            sqlx::query(
                "INSERT INTO payment_batch_items \
                 (payment_batch_id, supplier_invoice_id, position, end_to_end_id, amount) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(batch_id)
            .bind(acc.supplier_invoice_id)
            .bind(position as i32)
            .bind(&e2e)
            .bind(acc.amount)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        // (4) Relire + audit.
        let batch = sqlx::query_as::<_, PaymentBatch>(&format!(
            "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ?"
        ))
        .bind(batch_id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
        let items = sqlx::query_as::<_, PaymentBatchItem>(&format!(
            "SELECT {ITEM_COLS} FROM payment_batch_items WHERE payment_batch_id = ? ORDER BY position"
        ))
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "payment_batch.generated".to_string(),
                "payment_batch".to_string(),
                batch_id,
                Some(serde_json::json!({
                    "bankAccountId": bank_account_id,
                    "nbOfTxs": items.len(),
                    "totalAmount": total.to_string(),
                })),
            ),
        )
        .await?;

        Ok(CreateBatchOutcome {
            batch: Some(PaymentBatchWithItems { batch, items }),
            failed,
        })
    }
    .await;

    match result {
        Ok(v) => {
            // Si un lot a été créé, commit ; sinon rollback (rien à persister).
            if v.batch.is_some() {
                tx.commit().await.map_err(map_db_error)?;
            } else {
                let _ = tx.rollback().await;
            }
            Ok(v)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Valide une facture pour inclusion dans un lot. Verrouille la facture
/// (`FOR UPDATE`, sérialise vs pay/cancel/autre lot). Retourne `Ok(Accepted)`
/// ou `Err(PaymentBatchFailedItem)` (échec métier per-facture), ou un `DbError`
/// (échec infra → propage).
async fn validate_invoice_for_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    sid: i64,
) -> Result<Result<Accepted, PaymentBatchFailedItem>, DbError> {
    fn fail(sid: i64, code: &str) -> Result<Accepted, PaymentBatchFailedItem> {
        Err(PaymentBatchFailedItem {
            supplier_invoice_id: sid,
            error_code: code.to_string(),
            details: None,
        })
    }

    // (status, creditor_iban, creditor_qr_iban, payment_reference, total_amount)
    type InvoiceCoordsRow = (
        String,
        Option<String>,
        Option<String>,
        Option<String>,
        Decimal,
    );
    let row: Option<InvoiceCoordsRow> = sqlx::query_as(
        "SELECT status, creditor_iban, creditor_qr_iban, payment_reference, total_amount \
             FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(sid)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let Some((status, iban, qr_iban, _reference, total_amount)) = row else {
        return Ok(fail(sid, "SUPPLIER_INVOICE_NOT_FOUND"));
    };
    if status != "open" {
        return Ok(fail(sid, "SUPPLIER_INVOICE_NOT_OPEN"));
    }
    if iban.is_none() && qr_iban.is_none() {
        return Ok(fail(sid, "NO_PAYMENT_COORDINATES"));
    }

    // Déjà dans un lot `generated` ?
    let in_batch: Option<i64> = sqlx::query_scalar(
        "SELECT pbi.id FROM payment_batch_items pbi \
         JOIN payment_batches pb ON pb.id = pbi.payment_batch_id \
         WHERE pbi.supplier_invoice_id = ? AND pb.status = 'generated' LIMIT 1",
    )
    .bind(sid)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if in_batch.is_some() {
        return Ok(fail(sid, "ALREADY_IN_GENERATED_BATCH"));
    }

    // Validation des coordonnées (kesh-qrbill). Préférence QR-IBAN si présent.
    if let Some(qr) = &qr_iban {
        if kesh_qrbill::validation::validate_qr_iban(qr).is_err() {
            return Ok(fail(sid, "INVALID_QR_IBAN"));
        }
    } else if let Some(ib) = &iban
        && kesh_qrbill::validation::validate_iban(ib).is_err()
    {
        return Ok(fail(sid, "INVALID_IBAN"));
    }

    Ok(Ok(Accepted {
        supplier_invoice_id: sid,
        amount: total_amount,
    }))
}

/// Confirme un lot `generated` : poste les règlements de toutes les factures
/// dans UNE transaction atomique. Pré-check du compte source archivé.
pub async fn confirm_batch(
    pool: &MySqlPool,
    company_id: i64,
    batch_id: i64,
    payment_date: chrono::NaiveDate,
    user_id: i64,
) -> Result<PaymentBatchWithItems, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Verrou lot + garde statut.
        let batch = sqlx::query_as::<_, PaymentBatch>(&format!(
            "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ? FOR UPDATE"
        ))
        .bind(batch_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
        if batch.status != "generated" {
            return Err(DbError::IllegalStateTransition(format!(
                "seul un lot généré peut être confirmé (statut actuel : '{}')",
                batch.status
            )));
        }

        // (2) Pré-check compte source archivé (pay_in_tx ne le porte pas) [M-FRESH-1].
        let archived: Option<bool> = sqlx::query_scalar(
            "SELECT archived FROM bank_accounts WHERE id = ? AND company_id = ?",
        )
        .bind(batch.bank_account_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;
        match archived {
            None => return Err(DbError::NotFound),
            Some(true) => {
                return Err(DbError::IllegalStateTransition(
                    "le compte bancaire source est archivé".into(),
                ));
            }
            Some(false) => {}
        }

        // (3) Régler chaque facture du lot (atomique, pay_in_tx GUARD-FREE).
        let items = sqlx::query_as::<_, PaymentBatchItem>(&format!(
            "SELECT {ITEM_COLS} FROM payment_batch_items WHERE payment_batch_id = ? ORDER BY position"
        ))
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;

        for item in &items {
            supplier_invoices::pay_in_tx(
                &mut tx,
                company_id,
                item.supplier_invoice_id,
                SettlementChoice::BankTransfer {
                    bank_account_id: batch.bank_account_id,
                },
                payment_date,
                user_id,
            )
            .await?;
        }

        // (4) Lot → confirmed.
        let rows = sqlx::query(
            "UPDATE payment_batches SET status = 'confirmed', confirmed_at = CURRENT_TIMESTAMP(3), \
             version = version + 1 WHERE id = ? AND company_id = ? AND version = ? AND status = 'generated'",
        )
        .bind(batch_id)
        .bind(company_id)
        .bind(batch.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        let updated = sqlx::query_as::<_, PaymentBatch>(&format!(
            "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ?"
        ))
        .bind(batch_id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "payment_batch.confirmed".to_string(),
                "payment_batch".to_string(),
                batch_id,
                Some(serde_json::json!({ "nbOfTxs": items.len() })),
            ),
        )
        .await?;

        Ok(PaymentBatchWithItems {
            batch: updated,
            items,
        })
    }
    .await;

    match result {
        Ok(v) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(v)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Annule un lot `generated` (items conservés pour l'historique ; le guard ne
/// matche que `status='generated'`, donc les factures sont déverrouillées).
pub async fn cancel_batch(
    pool: &MySqlPool,
    company_id: i64,
    batch_id: i64,
    user_id: i64,
) -> Result<PaymentBatchWithItems, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        let batch = sqlx::query_as::<_, PaymentBatch>(&format!(
            "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ? FOR UPDATE"
        ))
        .bind(batch_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
        if batch.status != "generated" {
            return Err(DbError::IllegalStateTransition(format!(
                "seul un lot généré peut être annulé (statut actuel : '{}')",
                batch.status
            )));
        }

        let rows = sqlx::query(
            "UPDATE payment_batches SET status = 'cancelled', version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'generated'",
        )
        .bind(batch_id)
        .bind(company_id)
        .bind(batch.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        let updated = sqlx::query_as::<_, PaymentBatch>(&format!(
            "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ?"
        ))
        .bind(batch_id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;
        let items = sqlx::query_as::<_, PaymentBatchItem>(&format!(
            "SELECT {ITEM_COLS} FROM payment_batch_items WHERE payment_batch_id = ? ORDER BY position"
        ))
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "payment_batch.cancelled".to_string(),
                "payment_batch".to_string(),
                batch_id,
                None,
            ),
        )
        .await?;

        Ok(PaymentBatchWithItems {
            batch: updated,
            items,
        })
    }
    .await;

    match result {
        Ok(v) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(v)
        }
        Err(e) => {
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Récupère un lot + ses items (scopé company). `None` si introuvable.
pub async fn get(
    pool: &MySqlPool,
    company_id: i64,
    batch_id: i64,
) -> Result<Option<PaymentBatchWithItems>, DbError> {
    let batch = sqlx::query_as::<_, PaymentBatch>(&format!(
        "SELECT {BATCH_COLS} FROM payment_batches WHERE id = ? AND company_id = ?"
    ))
    .bind(batch_id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;
    let Some(batch) = batch else {
        return Ok(None);
    };
    let items = fetch_items(pool, batch_id).await?;
    Ok(Some(PaymentBatchWithItems { batch, items }))
}

/// Liste paginée des lots d'une company (les plus récents d'abord).
pub async fn list(
    pool: &MySqlPool,
    company_id: i64,
    limit: i64,
    offset: i64,
) -> Result<(Vec<PaymentBatch>, i64), DbError> {
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM payment_batches WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(map_db_error)?;
    let items = sqlx::query_as::<_, PaymentBatch>(&format!(
        "SELECT {BATCH_COLS} FROM payment_batches WHERE company_id = ? \
         ORDER BY created_at DESC, id DESC LIMIT ? OFFSET ?"
    ))
    .bind(company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;
    Ok((items, total))
}

/// Assemble et génère le fichier pain.001 d'un lot (scopé company).
pub async fn generate_pain001_xml(
    pool: &MySqlPool,
    company_id: i64,
    batch_id: i64,
) -> Result<String, DbError> {
    use kesh_payment::pain001::{
        CreditorAccount, Pain001Batch, Pain001Tx, PaymentReference, generate_pain001,
    };

    let Some(PaymentBatchWithItems { batch, items }) = get(pool, company_id, batch_id).await?
    else {
        return Err(DbError::NotFound);
    };

    // Débiteur : compte bancaire source.
    let (debtor_iban,): (String,) =
        sqlx::query_as("SELECT iban FROM bank_accounts WHERE id = ? AND company_id = ?")
            .bind(batch.bank_account_id)
            .bind(company_id)
            .fetch_optional(pool)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
    // Société : nom (partie initiatrice + débiteur).
    let (company_name,): (String,) = sqlx::query_as("SELECT name FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;

    let mut transactions = Vec::with_capacity(items.len());
    for item in &items {
        let inv: (String, Option<String>, Option<String>, Option<String>) = sqlx::query_as(
            "SELECT COALESCE((SELECT name FROM contacts WHERE id = si.contact_id), ''), \
                    si.creditor_iban, si.creditor_qr_iban, si.payment_reference \
             FROM supplier_invoices si WHERE si.id = ? AND si.company_id = ?",
        )
        .bind(item.supplier_invoice_id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
        let (creditor_name, iban, qr_iban, reference) = inv;

        // Préférence QR-IBAN (M2-1).
        let (creditor_account, payment_reference) = if let Some(qr) = qr_iban {
            (
                CreditorAccount::QrIban(qr),
                PaymentReference::Qrr(reference.unwrap_or_default()),
            )
        } else {
            (
                CreditorAccount::Iban(iban.unwrap_or_default()),
                PaymentReference::Unstructured(reference.unwrap_or_default()),
            )
        };

        transactions.push(Pain001Tx {
            end_to_end_id: item.end_to_end_id.clone(),
            amount: item.amount,
            creditor_name,
            creditor_account,
            reference: payment_reference,
        });
    }

    let pain = Pain001Batch {
        msg_id: batch.msg_id.clone(),
        payment_info_id: batch.payment_info_id.clone(),
        creation_dt: batch.created_at,
        initiating_party: company_name.clone(),
        debtor_name: company_name,
        debtor_iban,
        requested_execution_date: batch.requested_execution_date,
        transactions,
    };

    generate_pain001(&pain).map_err(|e| DbError::Invariant(format!("génération pain.001 : {e}")))
}
