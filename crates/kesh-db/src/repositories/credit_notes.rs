//! Repository des avoirs (notes de crédit) — Story 12.1 (FR36, FR37).
//!
//! Modèle v0.2 : avoir TOTAL en single-step (DC3/DC5). `create_credit_note`
//! crée ET émet l'avoir dans une seule transaction : snapshot des lignes de la
//! facture d'origine, attribution du numéro (séquence dédiée), génération de
//! l'écriture de **contre-passation** (swap débit↔crédit, montants positifs —
//! JAMAIS de montants négatifs, interdits par `chk_jel_*_nonneg`), et bascule
//! de la facture d'origine en `cancelled` (DC6).
//!
//! Refus (erreurs métier) : facture inexistante, non `validated` (AC2),
//! déjà encaissée `paid_at IS NOT NULL` (AC2bis), déjà créditée (AC3),
//! exercice fermé sur la date de l'avoir (AC10), comptes non configurés.

use rust_decimal::Decimal;
use sqlx::MySqlPool;

use crate::entities::{
    CreditNote, CreditNoteLine, InvoiceLine, JournalEntryWithLines, NewAuditLogEntry, NewCreditNote,
};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

/// SELECT scopé multi-tenant (anti-IDOR — toujours `AND company_id = ?`).
const FIND_CREDIT_NOTE_SCOPED_SQL: &str = "SELECT id, company_id, contact_id, invoice_id, \
    credit_note_number, status, date, total_amount, journal_entry_id, version, created_at, updated_at \
    FROM credit_notes WHERE id = ? AND company_id = ?";

/// Résultat de l'émission d'un avoir (entête + lignes + écriture de contre-passation).
#[derive(Debug, Clone)]
pub struct IssuedCreditNote {
    pub credit_note: CreditNote,
    pub lines: Vec<CreditNoteLine>,
    pub journal_entry: JournalEntryWithLines,
}

fn credit_note_snapshot_json(cn: &CreditNote, lines: &[CreditNoteLine]) -> serde_json::Value {
    serde_json::json!({
        "id": cn.id,
        "companyId": cn.company_id,
        "contactId": cn.contact_id,
        "invoiceId": cn.invoice_id,
        "creditNoteNumber": cn.credit_note_number,
        "status": cn.status,
        "date": cn.date.to_string(),
        "totalAmount": cn.total_amount.to_string(),
        "journalEntryId": cn.journal_entry_id,
        "lines": lines.iter().map(|l| serde_json::json!({
            "position": l.position,
            "description": l.description,
            "quantity": l.quantity.to_string(),
            "unitPrice": l.unit_price.to_string(),
            "vatRate": l.vat_rate.to_string(),
            "lineTotal": l.line_total.to_string(),
        })).collect::<Vec<_>>(),
    })
}

async fn fetch_credit_note_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    credit_note_id: i64,
) -> Result<Vec<CreditNoteLine>, DbError> {
    sqlx::query_as::<_, CreditNoteLine>(
        "SELECT id, credit_note_id, position, description, quantity, unit_price, vat_rate, \
         line_total, created_at FROM credit_note_lines WHERE credit_note_id = ? ORDER BY position",
    )
    .bind(credit_note_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// Récupère un avoir + ses lignes (scopé company). `None` si introuvable.
pub async fn get(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<(CreditNote, Vec<CreditNoteLine>)>, DbError> {
    let cn = sqlx::query_as::<_, CreditNote>(FIND_CREDIT_NOTE_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;

    let Some(cn) = cn else {
        return Ok(None);
    };

    let lines = sqlx::query_as::<_, CreditNoteLine>(
        "SELECT id, credit_note_id, position, description, quantity, unit_price, vat_rate, \
         line_total, created_at FROM credit_note_lines WHERE credit_note_id = ? ORDER BY position",
    )
    .bind(cn.id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(Some((cn, lines)))
}

/// Liste paginée des avoirs d'une company (les plus récents d'abord).
pub async fn list(
    pool: &MySqlPool,
    company_id: i64,
    limit: i64,
    offset: i64,
) -> Result<(Vec<CreditNote>, i64), DbError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM credit_notes WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    let items = sqlx::query_as::<_, CreditNote>(
        "SELECT id, company_id, contact_id, invoice_id, credit_note_number, status, date, \
         total_amount, journal_entry_id, version, created_at, updated_at \
         FROM credit_notes WHERE company_id = ? ORDER BY date DESC, id DESC LIMIT ? OFFSET ?",
    )
    .bind(company_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok((items, total))
}

/// Génère les lignes de l'écriture de **contre-passation** d'un avoir (DC2).
///
/// Inverse exact de `invoices::generate_invoice_journal_lines` (swap débit↔crédit,
/// montants **positifs**) :
/// - `[0]` Crédit créance (1100) = HT + TVA (annule la créance TTC)
/// - `[1]` Débit produit (3000) = HT (annule le produit)
/// - `[2..]` Débit TVA due (2200) = TVA par taux > 0 (annule la TVA due)
///
/// `lines` = `(line_total, vat_rate)` snapshotés depuis la facture. Le compte
/// TVA due n'est requis que si `total_vat > 0` (AC5).
fn generate_credit_note_journal_lines(
    lines: &[(Decimal, Decimal)],
    receivable_account_id: i64,
    revenue_account_id: i64,
    vat_payable_account_id: Option<i64>,
) -> Result<Vec<crate::entities::NewJournalEntryLine>, DbError> {
    use crate::entities::NewJournalEntryLine;
    use kesh_core::accounting::vat::line_vat_amount;
    use std::collections::BTreeMap;

    let mut total_ht = Decimal::ZERO;
    // Agrégation TVA par taux : BTreeMap → itération ASC (cohérent facture, AC6).
    let mut vat_by_rate: BTreeMap<Decimal, Decimal> = BTreeMap::new();

    for (line_total, vat_rate) in lines {
        total_ht += *line_total;
        let vat = line_vat_amount(*line_total, *vat_rate);
        *vat_by_rate.entry(*vat_rate).or_insert(Decimal::ZERO) += vat;
    }

    // Somme des arrondis par ligne — NE PAS réarrondir.
    let total_vat: Decimal = vat_by_rate.values().copied().sum();

    let mut entry_lines = Vec::with_capacity(2 + vat_by_rate.len());
    // (0) Crédit créance TTC = HT + TVA (annule la créance de la facture).
    entry_lines.push(NewJournalEntryLine {
        account_id: receivable_account_id,
        debit: Decimal::ZERO,
        credit: total_ht + total_vat,
        project_id: None,
    });
    // (1) Débit produit HT (annule le produit).
    entry_lines.push(NewJournalEntryLine {
        account_id: revenue_account_id,
        debit: total_ht,
        credit: Decimal::ZERO,
        project_id: None,
    });

    // (2..N) Débit TVA due par taux > 0 (annule la TVA due). Compte requis
    // seulement si une ligne doit réellement être émise.
    if total_vat > Decimal::ZERO {
        let vat_account = vat_payable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_vat_payable_account_id".into())
        })?;
        for amount in vat_by_rate.values() {
            if *amount > Decimal::ZERO {
                entry_lines.push(NewJournalEntryLine {
                    account_id: vat_account,
                    debit: *amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                });
            }
        }
    }

    Ok(entry_lines)
}

/// Crée et émet un avoir total contre-passant une facture validée (single-step,
/// DC5). Transaction atomique miroir de `invoices::validate_invoice`.
pub async fn create_credit_note(
    pool: &MySqlPool,
    new: NewCreditNote,
    user_id: i64,
) -> Result<IssuedCreditNote, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{
        company_invoice_settings, credit_note_number_sequences, fiscal_years, journal_entries,
    };
    use kesh_core::invoice_format;

    let NewCreditNote {
        company_id,
        invoice_id,
        date,
    } = new;

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Lock facture d'origine + checks métier.
        let invoice = sqlx::query_as::<_, crate::entities::Invoice>(
            "SELECT id, company_id, contact_id, invoice_number, status, date, due_date, \
             payment_terms, total_amount, journal_entry_id, paid_at, project_id, version, \
             created_at, updated_at \
             FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE",
        )
        .bind(invoice_id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

        let invoice = match invoice {
            None => return Err(DbError::NotFound),
            Some(inv) if inv.status != "validated" => {
                return Err(DbError::IllegalStateTransition(format!(
                    "un avoir ne peut viser qu'une facture validée (statut actuel : '{}')",
                    inv.status
                )));
            }
            // AC2bis : pas d'avoir sur une facture déjà encaissée (v0.2, pas de
            // flux de remboursement — éviterait un solde débiteur négatif).
            Some(inv) if inv.paid_at.is_some() => {
                return Err(DbError::IllegalStateTransition(
                    "impossible de créer un avoir sur une facture déjà payée".into(),
                ));
            }
            Some(inv) => inv,
        };

        // AC3 : une seule note de crédit par facture (UNIQUE(invoice_id) en DB ;
        // check explicite pour une erreur métier claire plutôt qu'une FK 1062).
        let existing: Option<i64> =
            sqlx::query_scalar("SELECT id FROM credit_notes WHERE invoice_id = ? FOR UPDATE")
                .bind(invoice_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;
        if existing.is_some() {
            return Err(DbError::IllegalStateTransition(
                "cette facture a déjà un avoir".into(),
            ));
        }

        // (2) Snapshot des lignes de la facture.
        let invoice_lines = sqlx::query_as::<_, InvoiceLine>(
            "SELECT id, invoice_id, position, description, quantity, unit_price, vat_rate, \
             line_total, created_at FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
        )
        .bind(invoice_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(map_db_error)?;

        // (3) Config company (lazy create + lock).
        let settings =
            company_invoice_settings::get_or_create_default_in_tx(&mut tx, company_id).await?;
        let receivable_account_id = settings.default_receivable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_receivable_account_id".into())
        })?;
        let revenue_account_id = settings
            .default_revenue_account_id
            .ok_or_else(|| DbError::ConfigurationRequired("default_revenue_account_id".into()))?;

        // (4) Exercice ouvert couvrant la date de l'avoir (DC10).
        let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, date)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        // (5) Numéro d'avoir (séquence dédiée).
        let seq = credit_note_number_sequences::next_number_for(&mut tx, company_id, fy.id).await?;
        let year = fy
            .start_date
            .format("%Y")
            .to_string()
            .parse::<i32>()
            .ok()
            .ok_or_else(|| {
                DbError::Invariant(format!("fiscal_year start_date inattendu : {}", fy.start_date))
            })?;
        let credit_note_number =
            invoice_format::render(&settings.credit_note_number_format, year, &fy.name, seq)
                .map_err(|e| {
                    DbError::Invariant(format!("rendu numéro avoir échoué (config invalide ?) : {e}"))
                })?;

        // (6) Libellé écriture : « Avoir AV-… (annule F-…) - Contact ».
        let contact_name: String =
            sqlx::query_scalar("SELECT name FROM contacts WHERE id = ? AND company_id = ?")
                .bind(invoice.contact_id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?
                .ok_or(DbError::NotFound)?;
        let invoice_number = invoice.invoice_number.clone().unwrap_or_default();
        let entry_description =
            format!("Avoir {credit_note_number} (annule {invoice_number}) - {contact_name}");

        // (7) Écriture de contre-passation.
        let pairs: Vec<(Decimal, Decimal)> = invoice_lines
            .iter()
            .map(|l| (l.line_total, l.vat_rate))
            .collect();
        let entry_lines = generate_credit_note_journal_lines(
            &pairs,
            receivable_account_id,
            revenue_account_id,
            settings.default_vat_payable_account_id,
        )?;
        let journal: Journal = settings.default_sales_journal;
        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: date,
                journal,
                description: entry_description,
                // Story 19-4 : l'avoir HÉRITE le projet de sa facture d'origine —
                // la contre-passation reprend le même tag, donc net par projet = 0
                // après annulation. Volontairement SANS re-check archivé (annuler
                // une facture d'un projet archivé doit rester possible — miroir
                // pay/cancel-after-archive 19-3, DC3).
                project_id: invoice.project_id,
                lines: entry_lines,
            },
        )
        .await?;

        // (8) total_amount = HT (Σ line_total), miroir invoices.total_amount.
        let total_ht: Decimal = invoice_lines.iter().map(|l| l.line_total).sum();

        // (9) INSERT credit_notes (status='issued', single-step DC5).
        let cn_id: i64 = sqlx::query(
            "INSERT INTO credit_notes \
             (company_id, contact_id, invoice_id, credit_note_number, status, date, \
              total_amount, journal_entry_id) \
             VALUES (?, ?, ?, ?, 'issued', ?, ?, ?)",
        )
        .bind(company_id)
        .bind(invoice.contact_id)
        .bind(invoice_id)
        .bind(&credit_note_number)
        .bind(date)
        .bind(total_ht)
        .bind(je.entry.id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .last_insert_id() as i64;

        // (10) INSERT credit_note_lines (snapshot).
        for line in &invoice_lines {
            sqlx::query(
                "INSERT INTO credit_note_lines \
                 (credit_note_id, position, description, quantity, unit_price, vat_rate, line_total) \
                 VALUES (?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(cn_id)
            .bind(line.position)
            .bind(&line.description)
            .bind(line.quantity)
            .bind(line.unit_price)
            .bind(line.vat_rate)
            .bind(line.line_total)
            .execute(&mut *tx)
            .await
            .map_err(map_db_error)?;
        }

        // (11) Bascule facture d'origine → cancelled (DC6). paid_at conservé
        // (mais ici toujours NULL car AC2bis refuse les factures payées).
        let rows = sqlx::query(
            "UPDATE invoices SET status = 'cancelled', version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'validated'",
        )
        .bind(invoice_id)
        .bind(company_id)
        .bind(invoice.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        // (12) Relire l'avoir + lignes.
        let cn = sqlx::query_as::<_, CreditNote>(FIND_CREDIT_NOTE_SCOPED_SQL)
            .bind(cn_id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;
        let cn_lines = fetch_credit_note_lines(&mut tx, cn_id).await?;

        // (13) Audit : credit_note.created + invoice.cancelled (AC11).
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "credit_note.created".to_string(),
                "credit_note".to_string(),
                cn_id,
                Some(serde_json::json!({
                    "creditNote": credit_note_snapshot_json(&cn, &cn_lines),
                    "journalEntryId": je.entry.id,
                })),
            ),
        )
        .await?;
        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::user(
                user_id,
                "invoice.cancelled".to_string(),
                "invoice".to_string(),
                invoice_id,
                Some(serde_json::json!({
                    "before": { "status": "validated" },
                    "after": { "status": "cancelled" },
                    "creditNoteId": cn_id,
                })),
            ),
        )
        .await?;

        Ok(IssuedCreditNote {
            credit_note: cn,
            lines: cn_lines,
            journal_entry: je,
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

    fn account_of(
        lines: &[crate::entities::NewJournalEntryLine],
        idx: usize,
    ) -> (i64, Decimal, Decimal) {
        let l = &lines[idx];
        (l.account_id, l.debit, l.credit)
    }

    #[test]
    fn contre_passation_single_rate_swaps_debit_credit() {
        // Facture 1000 HT @8.1% → contre-passation : Crédit 1100=1081, Débit 3000=1000, Débit 2200=81.
        let lines = vec![(dec!(1000), dec!(8.1))];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 3);
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(1081.00)));
        assert_eq!(account_of(&je, 1), (3000, dec!(1000), dec!(0)));
        assert_eq!(account_of(&je, 2), (2200, dec!(81.00), dec!(0)));
        // Équilibre.
        let debit: Decimal = je.iter().map(|l| l.debit).sum();
        let credit: Decimal = je.iter().map(|l| l.credit).sum();
        assert_eq!(debit, credit);
    }

    #[test]
    fn contre_passation_multi_rate_one_vat_line_per_rate_asc() {
        // 1000 @8.1% + 500 @2.6% → 2 lignes TVA, triées ASC (2.6 avant 8.1).
        let lines = vec![(dec!(1000), dec!(8.1)), (dec!(500), dec!(2.6))];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        // [0] crédit 1100 TTC, [1] débit 3000 HT, [2] débit 2200 @2.6%, [3] débit 2200 @8.1%.
        assert_eq!(je.len(), 4);
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(1594.00))); // 1500 + 81 + 13
        assert_eq!(account_of(&je, 1), (3000, dec!(1500), dec!(0)));
        assert_eq!(account_of(&je, 2), (2200, dec!(13.00), dec!(0))); // 2.6% de 500
        assert_eq!(account_of(&je, 3), (2200, dec!(81.00), dec!(0))); // 8.1% de 1000
        let debit: Decimal = je.iter().map(|l| l.debit).sum();
        let credit: Decimal = je.iter().map(|l| l.credit).sum();
        assert_eq!(debit, credit);
    }

    #[test]
    fn contre_passation_zero_vat_omits_vat_line() {
        // Tout exonéré (taux 0) → pas de ligne TVA (évite debit=0 & credit=0).
        let lines = vec![(dec!(1000), dec!(0))];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, None).unwrap();
        assert_eq!(je.len(), 2);
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(1000)));
        assert_eq!(account_of(&je, 1), (3000, dec!(1000), dec!(0)));
    }

    #[test]
    fn contre_passation_requires_vat_account_when_vat_positive() {
        let lines = vec![(dec!(1000), dec!(8.1))];
        let err = generate_credit_note_journal_lines(&lines, 1100, 3000, None).unwrap_err();
        assert!(matches!(err, DbError::ConfigurationRequired(_)));
    }
}
