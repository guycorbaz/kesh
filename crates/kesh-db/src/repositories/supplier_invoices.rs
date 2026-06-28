//! Repository des factures fournisseurs & règlement binaire — Story 12.2 (#191).
//!
//! - [`create`] : enregistre une facture fournisseur (chemin A). Single-step
//!   transactionnel : poste l'écriture d'achat (D charge / D impôt préalable
//!   1171 / C créanciers 2000, réutilise la logique TVA Epic 18) et passe la
//!   facture en `open`.
//! - [`pay`] : règlement binaire. Poste `D 2000 / C contrepartie` (virement →
//!   `bank_account.journal_account_id` ; compte interne → compte libre) et
//!   passe en `paid`. Le compte débité ET le montant sont relus depuis la
//!   ligne de crédit de l'écriture d'achat → solde 2000 garanti à 0 [O-M2].
//! - [`cancel`] : contre-passe l'écriture d'achat (relecture lignes + swap
//!   débit↔crédit, montants positifs) et passe en `cancelled` [DC7].
//!
//! Helper de génération d'écriture en kesh-db (PAS kesh-core : `NewJournalEntryLine`
//! et `DbError` y sont définis — dépendance circulaire sinon [O-C1]).

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::MySqlPool;

use crate::entities::{
    NewAuditLogEntry, NewJournalEntryLine, NewSupplierInvoice, SettlementChoice, SupplierInvoice,
    SupplierInvoiceLine,
};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

/// SELECT scopé multi-tenant (anti-IDOR — toujours `AND company_id = ?`).
const FIND_SCOPED_SQL: &str = "SELECT id, company_id, contact_id, supplier_invoice_number, status, \
    invoice_date, due_date, total_amount, creditor_iban, creditor_qr_iban, payment_reference, \
    expected_payment_amount, purchase_journal_entry_id, settlement_type, settlement_bank_account_id, \
    settlement_account_id, settlement_journal_entry_id, paid_at, version, created_at, updated_at \
    FROM supplier_invoices WHERE id = ? AND company_id = ?";

const LINE_COLUMNS: &str = "id, supplier_invoice_id, position, description, quantity, unit_price, \
    vat_rate, line_total, expense_account_id, created_at";

/// Résultat de l'enregistrement / paiement (entête + lignes).
#[derive(Debug, Clone)]
pub struct SupplierInvoiceWithLines {
    pub invoice: SupplierInvoice,
    pub lines: Vec<SupplierInvoiceLine>,
}

/// Ligne de liste (entête + nom du fournisseur, AC11).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SupplierInvoiceListItem {
    pub id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub supplier_invoice_number: Option<String>,
    pub status: String,
    pub invoice_date: chrono::NaiveDate,
    pub due_date: Option<chrono::NaiveDate>,
    pub total_amount: Decimal,
}

fn snapshot_json(inv: &SupplierInvoice, lines: &[SupplierInvoiceLine]) -> serde_json::Value {
    serde_json::json!({
        "id": inv.id,
        "companyId": inv.company_id,
        "contactId": inv.contact_id,
        "supplierInvoiceNumber": inv.supplier_invoice_number,
        "status": inv.status,
        "invoiceDate": inv.invoice_date.to_string(),
        "totalAmount": inv.total_amount.to_string(),
        "purchaseJournalEntryId": inv.purchase_journal_entry_id,
        "settlementType": inv.settlement_type,
        "settlementJournalEntryId": inv.settlement_journal_entry_id,
        "lines": lines.iter().map(|l| serde_json::json!({
            "position": l.position,
            "description": l.description,
            "quantity": l.quantity.to_string(),
            "unitPrice": l.unit_price.to_string(),
            "vatRate": l.vat_rate.to_string(),
            "lineTotal": l.line_total.to_string(),
            "expenseAccountId": l.expense_account_id,
        })).collect::<Vec<_>>(),
    })
}

async fn fetch_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    invoice_id: i64,
) -> Result<Vec<SupplierInvoiceLine>, DbError> {
    sqlx::query_as::<_, SupplierInvoiceLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM supplier_invoice_lines WHERE supplier_invoice_id = ? ORDER BY position"
    ))
    .bind(invoice_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Génère les lignes de l'écriture d'**achat** d'une facture fournisseur.
///
/// Réutilise la logique TVA Epic 18 (`kesh_core::accounting::vat::line_vat_amount`) :
/// - une ligne **Débit charge (HT)** par ligne de facture (pas d'agrégation
///   inter-lignes, même compte → lignes distinctes) ;
/// - une ligne **Débit impôt préalable 1171** unique agrégée (Σ TVA), **OMISE
///   si Σ TVA = 0** (sinon viole `chk_jel_debit_credit_exclusive`) ;
/// - une ligne **Crédit créanciers 2000 (TTC)** = Σ HT + Σ TVA.
///
/// `lines` = `(line_total HT, vat_rate, expense_account_id)`. Le compte de TVA
/// récupérable n'est requis que si Σ TVA > 0.
fn generate_purchase_journal_lines(
    lines: &[(Decimal, Decimal, i64)],
    payable_account_id: i64,
    recoverable_account_id: Option<i64>,
) -> Result<(Vec<NewJournalEntryLine>, Decimal), DbError> {
    use kesh_core::accounting::vat::line_vat_amount;

    let mut total_ht = Decimal::ZERO;
    let mut total_vat = Decimal::ZERO;
    let mut entry_lines: Vec<NewJournalEntryLine> = Vec::with_capacity(lines.len() + 2);

    // (0..N) Débit charge par ligne (HT).
    for (line_total, vat_rate, expense_account_id) in lines {
        total_ht += *line_total;
        total_vat += line_vat_amount(*line_total, *vat_rate);
        entry_lines.push(NewJournalEntryLine {
            account_id: *expense_account_id,
            debit: *line_total,
            credit: Decimal::ZERO,
        });
    }

    // (N) Débit impôt préalable 1171 agrégé — OMIS si Σ TVA = 0.
    if total_vat > Decimal::ZERO {
        let vat_account = recoverable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_vat_recoverable_account_id".into())
        })?;
        entry_lines.push(NewJournalEntryLine {
            account_id: vat_account,
            debit: total_vat,
            credit: Decimal::ZERO,
        });
    }

    // (N+1) Crédit créanciers 2000 (TTC).
    let total_ttc = total_ht + total_vat;
    entry_lines.push(NewJournalEntryLine {
        account_id: payable_account_id,
        debit: Decimal::ZERO,
        credit: total_ttc,
    });

    Ok((entry_lines, total_ttc))
}

/// Récupère une facture fournisseur + ses lignes (scopé company). `None` si introuvable.
pub async fn get(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<(SupplierInvoice, Vec<SupplierInvoiceLine>)>, DbError> {
    let inv = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;

    let Some(inv) = inv else {
        return Ok(None);
    };

    let lines = sqlx::query_as::<_, SupplierInvoiceLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM supplier_invoice_lines WHERE supplier_invoice_id = ? ORDER BY position"
    ))
    .bind(inv.id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(Some((inv, lines)))
}

/// Liste paginée des factures fournisseurs d'une company (les plus récentes d'abord).
pub async fn list(
    pool: &MySqlPool,
    company_id: i64,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SupplierInvoiceListItem>, i64), DbError> {
    let total: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM supplier_invoices WHERE company_id = ?")
            .bind(company_id)
            .fetch_one(pool)
            .await
            .map_err(map_db_error)?;

    // JOIN contacts pour le nom du fournisseur (AC11). INNER JOIN sûr : contact_id
    // est une FK RESTRICT non-nullable.
    let items = sqlx::query_as::<_, SupplierInvoiceListItem>(
        "SELECT si.id, si.contact_id, c.name AS contact_name, si.supplier_invoice_number, \
         si.status, si.invoice_date, si.due_date, si.total_amount \
         FROM supplier_invoices si JOIN contacts c ON c.id = si.contact_id \
         WHERE si.company_id = ? ORDER BY si.invoice_date DESC, si.id DESC LIMIT ? OFFSET ?",
    )
    .bind(company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok((items, total))
}

/// Enregistre une facture fournisseur et poste son écriture d'achat (single-step).
pub async fn create(
    pool: &MySqlPool,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{company_invoice_settings, fiscal_years, journal_entries};

    let NewSupplierInvoice {
        company_id,
        contact_id,
        supplier_invoice_number,
        invoice_date,
        due_date,
        creditor_iban,
        creditor_qr_iban,
        payment_reference,
        expected_payment_amount,
        lines,
    } = new;

    if lines.is_empty() {
        return Err(DbError::IllegalStateTransition(
            "une facture fournisseur doit avoir au moins une ligne".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Fournisseur valide (existe, company-scoped, is_supplier).
        let supplier: Option<(String, bool)> = sqlx::query_as(
            "SELECT name, is_supplier FROM contacts WHERE id = ? AND company_id = ?",
        )
        .bind(contact_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;
        let (contact_name, is_supplier) = supplier.ok_or(DbError::NotFound)?;
        if !is_supplier {
            return Err(DbError::IllegalStateTransition(
                "le contact n'est pas un fournisseur".into(),
            ));
        }

        // (2) Valider lignes : montant HT > 0, compte de charge company-scoped + actif.
        let mut pairs: Vec<(Decimal, Decimal, i64)> = Vec::with_capacity(lines.len());
        let mut computed_lines: Vec<(Decimal, &crate::entities::NewSupplierInvoiceLine)> =
            Vec::with_capacity(lines.len());
        for line in &lines {
            let line_total = line.quantity * line.unit_price;
            if line.quantity <= Decimal::ZERO
                || line.unit_price <= Decimal::ZERO
                || line_total <= Decimal::ZERO
            {
                return Err(DbError::IllegalStateTransition(
                    "chaque ligne doit avoir une quantité et un prix strictement positifs".into(),
                ));
            }
            if line.vat_rate < Decimal::ZERO || line.vat_rate > Decimal::from(100) {
                return Err(DbError::IllegalStateTransition(
                    "taux de TVA hors bornes (0-100)".into(),
                ));
            }
            // Compte de charge : doit exister, être actif, company-scoped, et de
            // type Expense (AC6 — sinon l'écriture débiterait un Passif/Actif).
            let acct: Option<(bool, String)> = sqlx::query_as(
                "SELECT active, account_type FROM accounts WHERE id = ? AND company_id = ? FOR UPDATE",
            )
            .bind(line.expense_account_id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
            match acct {
                Some((true, ref t)) if t == "Expense" => {}
                _ => return Err(DbError::InactiveOrInvalidAccounts),
            }
            pairs.push((line_total, line.vat_rate, line.expense_account_id));
            computed_lines.push((line_total, line));
        }

        // (3) Exercice ouvert couvrant la date de facture.
        let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, invoice_date)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        // (4) Config company : comptes créanciers 2000 (requis) + 1171 récupérable (optionnel).
        let settings =
            company_invoice_settings::get_or_create_default_in_tx(&mut tx, company_id).await?;
        let payable_account_id = settings
            .default_payable_account_id
            .ok_or_else(|| DbError::ConfigurationRequired("default_payable_account_id".into()))?;

        // (5) Écriture d'achat.
        let (entry_lines, total_ttc) = generate_purchase_journal_lines(
            &pairs,
            payable_account_id,
            settings.default_vat_recoverable_account_id,
        )?;
        let number_label = supplier_invoice_number.clone().unwrap_or_default();
        let description = format!("Facture fournisseur {number_label} - {contact_name}");
        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: invoice_date,
                journal: Journal::Achats,
                description,
                lines: entry_lines,
            },
        )
        .await?;

        // (6) INSERT supplier_invoices (status='open').
        let inv_id: i64 = sqlx::query(
            "INSERT INTO supplier_invoices \
             (company_id, contact_id, supplier_invoice_number, status, invoice_date, due_date, \
              total_amount, creditor_iban, creditor_qr_iban, payment_reference, \
              expected_payment_amount, purchase_journal_entry_id) \
             VALUES (?, ?, ?, 'open', ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(contact_id)
        .bind(&supplier_invoice_number)
        .bind(invoice_date)
        .bind(due_date)
        .bind(total_ttc)
        .bind(&creditor_iban)
        .bind(&creditor_qr_iban)
        .bind(&payment_reference)
        .bind(expected_payment_amount)
        .bind(je.entry.id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .last_insert_id() as i64;

        // (7) INSERT supplier_invoice_lines.
        for (position, (line_total, line)) in computed_lines.iter().enumerate() {
            sqlx::query(
                "INSERT INTO supplier_invoice_lines \
                 (supplier_invoice_id, position, description, quantity, unit_price, vat_rate, \
                  line_total, expense_account_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(inv_id)
            .bind(position as i32)
            .bind(&line.description)
            .bind(line.quantity)
            .bind(line.unit_price)
            .bind(line.vat_rate)
            .bind(*line_total)
            .bind(line.expense_account_id)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        // (8) Relire + audit.
        let inv = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
            .bind(inv_id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
        let inv_lines = fetch_lines(&mut tx, inv_id).await?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "supplier_invoice.created".to_string(),
                "supplier_invoice".to_string(),
                inv_id,
                Some(serde_json::json!({
                    "supplierInvoice": snapshot_json(&inv, &inv_lines),
                    "journalEntryId": je.entry.id,
                })),
            ),
        )
        .await?;

        Ok(SupplierInvoiceWithLines {
            invoice: inv,
            lines: inv_lines,
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

/// Règle une facture fournisseur `open` (choix binaire). Poste l'écriture de
/// règlement et passe en `paid`.
pub async fn pay(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    choice: SettlementChoice,
    payment_date: chrono::NaiveDate,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{fiscal_years, journal_entries};

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Verrou facture + garde statut.
        let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
            .bind(id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
        if inv.status != "open" {
            return Err(DbError::IllegalStateTransition(format!(
                "seule une facture ouverte peut être réglée (statut actuel : '{}')",
                inv.status
            )));
        }

        // (2) Compte débité (créanciers) ET montant = ligne de crédit de l'écriture
        //     d'achat → solde 2000 garanti à 0 quelle que soit l'évolution des settings.
        let (payable_account_id, ttc): (i64, Decimal) = sqlx::query_as(
            "SELECT jel.account_id, jel.credit FROM journal_entry_lines jel \
             JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE jel.entry_id = ? AND je.company_id = ? AND jel.credit > 0 \
             ORDER BY jel.id LIMIT 1",
        )
        .bind(inv.purchase_journal_entry_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| {
            DbError::Invariant("écriture d'achat sans ligne de crédit créanciers".into())
        })?;

        // (3) Contrepartie selon le choix binaire (company-scoped).
        let (counterparty_account_id, settlement_type, bank_account_id, internal_account_id, journal) =
            match choice {
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
                    let counterparty = journal_account_id
                        .ok_or(DbError::NotFound)?
                        .ok_or_else(|| {
                            DbError::ConfigurationRequired(
                                "bank_account.journal_account_id".into(),
                            )
                        })?;
                    (
                        counterparty,
                        "bank_transfer",
                        Some(bank_account_id),
                        None,
                        Journal::Banque,
                    )
                }
                SettlementChoice::InternalAccount { account_id } => {
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
                        Some(true) => {}
                    }
                    (account_id, "internal_account", None, Some(account_id), Journal::OD)
                }
            };

        // (4) Exercice ouvert couvrant la date de règlement.
        let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, payment_date)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        // (5) Écriture de règlement : D 2000 / C contrepartie (TTC).
        let number_label = inv
            .supplier_invoice_number
            .clone()
            .unwrap_or_else(|| inv.id.to_string());
        let contact_name: String =
            sqlx::query_scalar("SELECT name FROM contacts WHERE id = ? AND company_id = ?")
                .bind(inv.contact_id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?
                .unwrap_or_default();
        let entry_lines = vec![
            NewJournalEntryLine {
                account_id: payable_account_id,
                debit: ttc,
                credit: Decimal::ZERO,
            },
            NewJournalEntryLine {
                account_id: counterparty_account_id,
                debit: Decimal::ZERO,
                credit: ttc,
            },
        ];
        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: payment_date,
                journal,
                description: format!("Règlement fournisseur {number_label} - {contact_name}"),
                lines: entry_lines,
            },
        )
        .await?;

        // (6) UPDATE facture → paid.
        let rows = sqlx::query(
            "UPDATE supplier_invoices SET status = 'paid', settlement_type = ?, \
             settlement_bank_account_id = ?, settlement_account_id = ?, \
             settlement_journal_entry_id = ?, paid_at = CURRENT_TIMESTAMP(3), version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'open'",
        )
        .bind(settlement_type)
        .bind(bank_account_id)
        .bind(internal_account_id)
        .bind(je.entry.id)
        .bind(id)
        .bind(company_id)
        .bind(inv.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        // (7) Relire + audit.
        let updated = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
        let inv_lines = fetch_lines(&mut tx, id).await?;
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "supplier_invoice.paid".to_string(),
                "supplier_invoice".to_string(),
                id,
                Some(serde_json::json!({
                    "settlementType": settlement_type,
                    "settlementJournalEntryId": je.entry.id,
                    "amount": ttc.to_string(),
                })),
            ),
        )
        .await?;

        Ok(SupplierInvoiceWithLines {
            invoice: updated,
            lines: inv_lines,
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

/// Annule une facture fournisseur `open` : contre-passe l'écriture d'achat
/// (relecture des lignes + swap débit↔crédit) et passe en `cancelled` [DC7].
pub async fn cancel(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{fiscal_years, journal_entries};

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Verrou facture + garde statut.
        let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
            .bind(id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?
            .ok_or(DbError::NotFound)?;
        if inv.status != "open" {
            return Err(DbError::IllegalStateTransition(format!(
                "seule une facture ouverte peut être annulée (statut actuel : '{}')",
                inv.status
            )));
        }

        // (2) Relire les lignes de l'écriture d'achat et les inverser (swap D↔C).
        let purchase_lines: Vec<(i64, Decimal, Decimal)> = sqlx::query_as(
            "SELECT jel.account_id, jel.debit, jel.credit FROM journal_entry_lines jel \
             JOIN journal_entries je ON je.id = jel.entry_id \
             WHERE jel.entry_id = ? AND je.company_id = ? ORDER BY jel.id",
        )
        .bind(inv.purchase_journal_entry_id)
        .bind(company_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;
        if purchase_lines.is_empty() {
            return Err(DbError::Invariant(
                "écriture d'achat introuvable pour contre-passation".into(),
            ));
        }
        let reversal_lines: Vec<NewJournalEntryLine> = purchase_lines
            .iter()
            .map(|(account_id, debit, credit)| NewJournalEntryLine {
                account_id: *account_id,
                debit: *credit,
                credit: *debit,
            })
            .collect();

        // (3) Exercice ouvert couvrant la date du jour.
        let today = Utc::now().date_naive();
        let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, today)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        let number_label = inv
            .supplier_invoice_number
            .clone()
            .unwrap_or_else(|| inv.id.to_string());
        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: today,
                journal: Journal::OD,
                description: format!("Annulation facture fournisseur {number_label}"),
                lines: reversal_lines,
            },
        )
        .await?;

        // (4) UPDATE facture → cancelled.
        let rows = sqlx::query(
            "UPDATE supplier_invoices SET status = 'cancelled', version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'open'",
        )
        .bind(id)
        .bind(company_id)
        .bind(inv.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        let updated = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
        let inv_lines = fetch_lines(&mut tx, id).await?;
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "supplier_invoice.cancelled".to_string(),
                "supplier_invoice".to_string(),
                id,
                Some(serde_json::json!({
                    "reversalJournalEntryId": je.entry.id,
                })),
            ),
        )
        .await?;

        Ok(SupplierInvoiceWithLines {
            invoice: updated,
            lines: inv_lines,
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn purchase_lines_single_rate() {
        // 1000 HT @8.1% → D 6000=1000, D 1171=81, C 2000=1081.
        let lines = vec![(dec!(1000), dec!(8.1), 6000)];
        let (je, ttc) = generate_purchase_journal_lines(&lines, 2000, Some(1171)).unwrap();
        assert_eq!(je.len(), 3);
        assert_eq!(
            (je[0].account_id, je[0].debit, je[0].credit),
            (6000, dec!(1000), dec!(0))
        );
        assert_eq!(
            (je[1].account_id, je[1].debit, je[1].credit),
            (1171, dec!(81.00), dec!(0))
        );
        assert_eq!(
            (je[2].account_id, je[2].debit, je[2].credit),
            (2000, dec!(0), dec!(1081.00))
        );
        assert_eq!(ttc, dec!(1081.00));
        let debit: Decimal = je.iter().map(|l| l.debit).sum();
        let credit: Decimal = je.iter().map(|l| l.credit).sum();
        assert_eq!(debit, credit);
    }

    #[test]
    fn purchase_lines_multi_same_account_distinct_lines() {
        // 2 lignes, même compte de charge 6000 → 2 lignes débit distinctes (pas d'agrégation).
        let lines = vec![(dec!(100), dec!(8.1), 6000), (dec!(200), dec!(8.1), 6000)];
        let (je, _) = generate_purchase_journal_lines(&lines, 2000, Some(1171)).unwrap();
        // 2 lignes charge + 1 ligne TVA + 1 ligne 2000 = 4.
        assert_eq!(je.len(), 4);
        assert_eq!(je[0].account_id, 6000);
        assert_eq!(je[1].account_id, 6000);
        assert_eq!(je[0].debit, dec!(100));
        assert_eq!(je[1].debit, dec!(200));
    }

    #[test]
    fn purchase_lines_zero_vat_omits_1171() {
        // Tout exonéré → pas de ligne 1171 (évite debit=0 & credit=0).
        let lines = vec![(dec!(1000), dec!(0), 6000)];
        let (je, ttc) = generate_purchase_journal_lines(&lines, 2000, None).unwrap();
        assert_eq!(je.len(), 2);
        assert_eq!((je[0].account_id, je[0].debit), (6000, dec!(1000)));
        assert_eq!((je[1].account_id, je[1].credit), (2000, dec!(1000)));
        assert_eq!(ttc, dec!(1000));
    }

    #[test]
    fn purchase_lines_requires_recoverable_account_when_vat_positive() {
        let lines = vec![(dec!(1000), dec!(8.1), 6000)];
        let err = generate_purchase_journal_lines(&lines, 2000, None).unwrap_err();
        assert!(matches!(err, DbError::ConfigurationRequired(_)));
    }
}
