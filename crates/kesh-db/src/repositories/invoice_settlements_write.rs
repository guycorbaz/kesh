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
use crate::errors::{DbError, map_db_error};
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
            // mais il doit être ACTIF : régler sur un compte archivé produirait
            // une écriture qu'aucun écran ne montre plus.
            let active: Option<bool> = sqlx::query_scalar(
                "SELECT active FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
            )
            .bind(account_id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
            match active {
                None | Some(false) => return Err(DbError::InactiveOrInvalidAccounts),
                Some(true) => account_id,
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
