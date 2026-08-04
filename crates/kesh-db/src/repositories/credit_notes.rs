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
            // Story 16-1a (D5) : compte de produit recopié depuis la ligne de
            // facture — c'est lui que la contre-passation débite.
            "revenueAccountId": l.revenue_account_id,
        })).collect::<Vec<_>>(),
    })
}

async fn fetch_credit_note_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    credit_note_id: i64,
) -> Result<Vec<CreditNoteLine>, DbError> {
    sqlx::query_as::<_, CreditNoteLine>(
        "SELECT id, credit_note_id, position, description, quantity, unit_price, vat_rate, \
         line_total, revenue_account_id, created_at \
         FROM credit_note_lines WHERE credit_note_id = ? ORDER BY position",
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
         line_total, revenue_account_id, created_at \
         FROM credit_note_lines WHERE credit_note_id = ? ORDER BY position",
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
/// - `[1..M]` Débit produit, **une ligne par compte de produit effectif** dont le
///   montant agrégé est `> 0`, triées par `account_id` croissant (annule chaque
///   crédit produit de la facture) — Story 16-1a, décision D5
/// - `[M+1..]` Débit TVA due (2200) = TVA par taux > 0 (annule la TVA due)
///
/// `lines` = `(line_total, vat_rate, revenue_account_id)` snapshotés depuis la
/// facture. Le compte TVA due n'est requis que si `total_vat > 0` (AC5).
///
/// # Pourquoi la ventilation doit être ici aussi (D5)
///
/// C'est le point le plus grave de la Story 16-1a. Ventiler côté facture sans
/// toucher l'avoir produirait ceci : une facture créditée sur `3200` serait
/// extournée sur le compte de produit par défaut, mettons `3000`. Les deux
/// écritures ne s'annulent plus → **résidu permanent** au crédit de `3200` et
/// au débit de `3000`. Le bilan reste équilibré, donc rien ne signale l'erreur,
/// mais le compte de résultat est faux. La correction doit vivre dans la même
/// story que sa cause.
///
/// Le troisième membre du triplet est lu sur les **lignes de la facture**
/// (`invoice_lines`), dans la même transaction que la copie vers
/// `credit_note_lines` — les deux valeurs sont donc identiques par construction,
/// mais la source est bien la facture.
///
/// **Nature exacte de la protection** : la transaction verrouille la ligne
/// `invoices` (`SELECT … FOR UPDATE`, cf. étape (1) de [`create_credit_note`]),
/// **pas** les `invoice_lines` elles-mêmes — leur `SELECT` est nu. La cohérence
/// tient donc *indirectement* : le seul écrivain de
/// `invoice_lines.revenue_account_id` après validation est la matérialisation D2
/// de `validate_invoice`, qui détient ce même verrou sur `invoices`. Quiconque
/// ajouterait une **seconde** voie de mutation de cette colonne devrait donc
/// prendre le verrou sur `invoices`, ou poser un verrou dédié : il n'y a rien
/// ici qui l'en empêche.
///
/// *Une exception à cette énumération, relevée en passe 5 et jugée sans effet* :
/// la restauration d'une sauvegarde ([`crate::backup::restore_tables_in_tx`])
/// réécrit `invoice_lines` — colonnes introspectées, `revenue_account_id`
/// comprise — sans prendre ce verrou. Elle ne rompt pas l'invariant pour autant,
/// puisqu'elle remplace `invoice_lines` **et** `credit_note_lines` depuis un même
/// instantané : le miroir facture/avoir reste cohérent, et son `DELETE FROM
/// invoices` se heurterait de toute façon au verrou d'un avoir en vol.
///
/// *(Revue 16-1a : la doc affirmait en passe 1 que le triplet venait du snapshot
/// `credit_note_lines` — faux ; la correction de passe 2 a écrit « verrouillées »
/// — imprécis, aucun `FOR UPDATE` ne porte sur `invoice_lines`. Passe 3.)* `None`
/// (depuis le backfill de 16-1a-bis, une facture dont l'écriture a été
/// retouchée à la main — cf. D-B2) se replie sur
/// `default_revenue_account_id`, exactement comme le faisait tout l'avoir avant
/// la story.
fn generate_credit_note_journal_lines(
    lines: &[(Decimal, Decimal, Option<i64>)],
    receivable_account_id: i64,
    default_revenue_account_id: i64,
    vat_payable_account_id: Option<i64>,
) -> Result<Vec<crate::entities::NewJournalEntryLine>, DbError> {
    use crate::entities::NewJournalEntryLine;
    use kesh_core::accounting::vat::line_vat_amount;
    use std::collections::BTreeMap;

    let mut total_ht = Decimal::ZERO;
    // Agrégation HT par compte effectif : miroir strict de la facture (D4/D5).
    let mut ht_by_account: BTreeMap<i64, Decimal> = BTreeMap::new();
    // Agrégation TVA par taux : BTreeMap → itération ASC (cohérent facture, AC6).
    let mut vat_by_rate: BTreeMap<Decimal, Decimal> = BTreeMap::new();

    for (line_total, vat_rate, revenue_account_id) in lines {
        total_ht += *line_total;
        let effective_account = revenue_account_id.unwrap_or(default_revenue_account_id);
        *ht_by_account
            .entry(effective_account)
            .or_insert(Decimal::ZERO) += *line_total;
        let vat = line_vat_amount(*line_total, *vat_rate);
        *vat_by_rate.entry(*vat_rate).or_insert(Decimal::ZERO) += vat;
    }

    // Somme des arrondis par ligne — NE PAS réarrondir.
    let total_vat: Decimal = vat_by_rate.values().copied().sum();

    let mut entry_lines = Vec::with_capacity(1 + ht_by_account.len() + vat_by_rate.len());
    // (0) Crédit créance TTC = HT + TVA (annule la créance de la facture).
    entry_lines.push(NewJournalEntryLine {
        account_id: receivable_account_id,
        debit: Decimal::ZERO,
        credit: total_ht + total_vat,
        project_id: None,
    });
    // (1..M) Débit produit HT par compte effectif (annule chaque crédit produit).
    // Mêmes clés, mêmes montants, même ordre que la ventilation de la facture :
    // c'est ce qui fait que les deux écritures s'annulent compte par compte.
    for (account_id, amount) in &ht_by_account {
        if *amount > Decimal::ZERO {
            entry_lines.push(NewJournalEntryLine {
                account_id: *account_id,
                debit: *amount,
                credit: Decimal::ZERO,
                project_id: None,
            });
        }
    }

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
        let invoice = sqlx::query_as::<_, crate::entities::Invoice>(&format!(
            "{} FOR UPDATE",
            super::invoices::FIND_INVOICE_SCOPED_SQL
        ))
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

        // (2) Snapshot des lignes de la facture. Liste de colonnes partagée
        // avec `invoices` (Story 16-1a) : dupliquée, elle avait déjà manqué
        // l'ajout d'une colonne sans que la compilation le signale.
        let invoice_lines = sqlx::query_as::<_, InvoiceLine>(&format!(
            "SELECT {} FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
            super::invoices::LINE_COLUMNS
        ))
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
                DbError::Invariant(format!(
                    "fiscal_year start_date inattendu : {}",
                    fy.start_date
                ))
            })?;
        let credit_note_number =
            invoice_format::render(&settings.credit_note_number_format, year, &fy.name, seq)
                .map_err(|e| {
                    DbError::Invariant(format!(
                        "rendu numéro avoir échoué (config invalide ?) : {e}"
                    ))
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

        // (6 bis) Story 16-1a (D4-bis / AC13-bis) — avoir sur une pièce
        // entièrement à zéro : même bug latent que côté facture (ligne
        // d'écriture `debit = 0, credit = 0` → violation de
        // `chk_jel_debit_credit_exclusive` → 500 SQL). Erreur métier avant
        // toute construction d'écriture. `line_total >= 0` (CHECK DB), donc une
        // somme nulle implique des lignes toutes nulles, donc une TVA nulle.
        let total_ht: Decimal = invoice_lines.iter().map(|l| l.line_total).sum();
        if total_ht == Decimal::ZERO {
            return Err(DbError::InvalidInput("creditNoteTotalZero".into()));
        }

        // (6 ter) Story 16-1a (D5-bis / AC11-bis) — un compte ventilé sur la
        // facture a pu être archivé depuis sa validation : `accounts::archive`
        // ne consulte pas `invoice_lines`. La contre-passation ne peut alors
        // pas être postée (la garde `active` de `create_in_tx` est
        // inconditionnelle), et se replier sur le défaut société recréerait
        // exactement le résidu que D5 combat. L'avoir échoue donc — mais en
        // nommant la ligne et le compte à réactiver, au lieu du
        // `400 INACTIVE_OR_INVALID_ACCOUNTS` générique.
        //
        // `active` UNIQUEMENT : ni `postable` ni `account_type` ne sont
        // re-vérifiés. La contre-passation doit viser les **mêmes** comptes que
        // l'écriture d'origine, quelle qu'ait été leur évolution de
        // configuration entre-temps. Seule l'inactivité est bloquante, parce
        // qu'elle l'est en base.
        {
            let mut sites: Vec<(i32, i64)> = invoice_lines
                .iter()
                .filter_map(|l| l.revenue_account_id.map(|a| (l.position + 1, a)))
                .collect();
            if invoice_lines.iter().any(|l| l.revenue_account_id.is_none()) {
                // Ligne 0 = le compte par défaut de la société, qu'aucune ligne
                // ne porte (miroir du `None` côté facture).
                sites.push((0, revenue_account_id));
            }
            let mut ids: Vec<i64> = sites.iter().map(|(_, a)| *a).collect();
            ids.sort_unstable();
            ids.dedup();

            let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "SELECT id, number FROM accounts \
                 WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})"
            );
            let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
            for id in &ids {
                q = q.bind(id);
            }
            let active_rows = q.fetch_all(&mut *tx).await.map_err(map_db_error)?;
            let active_ids: std::collections::HashSet<i64> =
                active_rows.iter().map(|(id, _)| *id).collect();

            if active_ids.len() != ids.len() {
                // Récupérer les numéros des comptes inactifs pour un message
                // actionnable (le SELECT ci-dessus les a exclus).
                let placeholders = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                let sql = format!(
                    "SELECT id, number FROM accounts \
                     WHERE company_id = ? AND id IN ({placeholders})"
                );
                let mut q = sqlx::query_as::<_, (i64, String)>(&sql).bind(company_id);
                for id in &ids {
                    q = q.bind(id);
                }
                let numbers: std::collections::HashMap<i64, String> = q
                    .fetch_all(&mut *tx)
                    .await
                    .map_err(map_db_error)?
                    .into_iter()
                    .collect();

                let mut rejected: Vec<crate::errors::RejectedRevenueAccount> = sites
                    .iter()
                    .filter(|(_, a)| !active_ids.contains(a))
                    .map(
                        |(position, account_id)| crate::errors::RejectedRevenueAccount {
                            line_number: if *position == 0 {
                                None
                            } else {
                                Some(*position)
                            },
                            account_id: *account_id,
                            account_number: numbers.get(account_id).cloned(),
                            reason: crate::errors::RevenueAccountRejection::Inactive,
                        },
                    )
                    .collect();
                rejected.sort_by_key(|r| (r.line_number.is_some(), r.line_number, r.account_id));
                rejected.dedup_by_key(|r| (r.line_number, r.account_id));
                return Err(DbError::CreditNoteRevenueAccountsArchived(rejected));
            }
        }

        // (7) Écriture de contre-passation.
        let triplets: Vec<(Decimal, Decimal, Option<i64>)> = invoice_lines
            .iter()
            .map(|l| (l.line_total, l.vat_rate, l.revenue_account_id))
            .collect();
        let entry_lines = generate_credit_note_journal_lines(
            &triplets,
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
            // Flux automatique (avoir) : garde de postabilité désactivée
            // (Story 14-3b, D-A0).
            false,
        )
        .await?;

        // (8) total_amount = HT (Σ line_total), miroir invoices.total_amount.
        // Déjà calculé en (6 bis) pour la garde « pièce à zéro ».

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
                 (credit_note_id, position, description, quantity, unit_price, vat_rate, \
                  line_total, revenue_account_id) \
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(cn_id)
            .bind(line.position)
            .bind(&line.description)
            .bind(line.quantity)
            .bind(line.unit_price)
            .bind(line.vat_rate)
            .bind(line.line_total)
            // Story 16-1a (D5) : le compte de la ligne de facture est recopié
            // dans le snapshot — c'est ce qui rend la contre-passation
            // l'« inverse exact » même si le défaut société change ensuite.
            .bind(line.revenue_account_id)
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
        let lines = vec![(dec!(1000), dec!(8.1), None)];
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
        let lines = vec![(dec!(1000), dec!(8.1), None), (dec!(500), dec!(2.6), None)];
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
        let lines = vec![(dec!(1000), dec!(0), None)];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, None).unwrap();
        assert_eq!(je.len(), 2);
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(1000)));
        assert_eq!(account_of(&je, 1), (3000, dec!(1000), dec!(0)));
    }

    #[test]
    fn contre_passation_requires_vat_account_when_vat_positive() {
        let lines = vec![(dec!(1000), dec!(8.1), None)];
        let err = generate_credit_note_journal_lines(&lines, 1100, 3000, None).unwrap_err();
        assert!(matches!(err, DbError::ConfigurationRequired(_)));
    }

    // -----------------------------------------------------------------------
    // Story 16-1a (AC16) — miroir strict des tests de ventilation de la
    // facture (`invoices::tests`, AC15). Les quatre tests ci-dessus sont la
    // non-régression mono-compte : leurs triplets ont tous `None`, donc un
    // seul compte effectif, donc les mêmes écritures qu'avant la story.
    // -----------------------------------------------------------------------

    /// Somme des débits portés par un compte donné.
    fn debit_on(lines: &[crate::entities::NewJournalEntryLine], account_id: i64) -> Decimal {
        lines
            .iter()
            .filter(|l| l.account_id == account_id)
            .map(|l| l.debit)
            .sum()
    }

    /// (g') Deux comptes explicites → deux **débits** produit distincts.
    #[test]
    fn contre_passation_ventilates_two_explicit_accounts() {
        let lines = vec![
            (dec!(1000), dec!(0), Some(3200)),
            (dec!(400), dec!(0), Some(3400)),
        ];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 3, "créance + 2 débits produit, pas de TVA");
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(1400)));
        assert_eq!(debit_on(&je, 3200), dec!(1000));
        assert_eq!(debit_on(&je, 3400), dec!(400));
        assert_eq!(
            debit_on(&je, 3000),
            dec!(0),
            "aucun repli sur le défaut société"
        );
        let debit: Decimal = je.iter().map(|l| l.debit).sum();
        let credit: Decimal = je.iter().map(|l| l.credit).sum();
        assert_eq!(debit, credit);
    }

    /// (h') Multi-comptes × multi-taux : ventilations indépendantes.
    #[test]
    fn contre_passation_ventilates_accounts_and_rates_independently() {
        let lines = vec![
            (dec!(1000), dec!(8.1), Some(3200)),
            (dec!(500), dec!(2.6), Some(3200)),
            (dec!(400), dec!(8.1), Some(3400)),
        ];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 5, "créance + 2 comptes produit + 2 taux");
        assert_eq!(debit_on(&je, 3200), dec!(1500));
        assert_eq!(debit_on(&je, 3400), dec!(400));
        assert_eq!(debit_on(&je, 2200), dec!(126.40));
        assert_eq!(account_of(&je, 0), (1100, dec!(0), dec!(2026.40)));
        let debit: Decimal = je.iter().map(|l| l.debit).sum();
        let credit: Decimal = je.iter().map(|l| l.credit).sum();
        assert_eq!(debit, credit);
    }

    /// (i') Compte à montant agrégé nul → filtré (évite debit=0 & credit=0).
    #[test]
    fn contre_passation_filters_zero_amount_account() {
        let lines = vec![
            (dec!(1000), dec!(0), Some(3200)),
            (dec!(0), dec!(0), Some(3400)),
        ];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 2);
        assert!(je.iter().all(|l| l.account_id != 3400));
    }

    /// (j') Invariant D3-bis en miroir : `None` et le défaut explicite
    /// fusionnent en un seul débit produit.
    #[test]
    fn contre_passation_merges_null_and_explicit_default_account() {
        let lines = vec![(dec!(600), dec!(0), None), (dec!(400), dec!(0), Some(3000))];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 2, "un SEUL débit produit");
        assert_eq!(debit_on(&je, 3000), dec!(1000));
    }

    /// (k') Ordre déterministe par `account_id` croissant, en miroir de la
    /// facture — c'est ce qui permet de comparer les deux écritures ligne à
    /// ligne dans le test pivot d'AC17.
    #[test]
    fn contre_passation_revenue_accounts_sorted_ascending() {
        let lines = vec![
            (dec!(100), dec!(0), Some(3400)),
            (dec!(200), dec!(0), Some(3200)),
            (dec!(300), dec!(0), Some(3000)),
        ];
        let je = generate_credit_note_journal_lines(&lines, 1100, 3000, Some(2200)).unwrap();
        assert_eq!(je.len(), 4);
        let accounts: Vec<i64> = je[1..].iter().map(|l| l.account_id).collect();
        assert_eq!(accounts, vec![3000, 3200, 3400]);
    }

    /// (l') **Le cœur de D5** : facture ventilée puis avoir total → les deux
    /// écritures s'annulent **compte par compte**. C'est le garde-fou du mode
    /// de défaillance le plus grave de la story — un résidu laisserait le bilan
    /// équilibré et le compte de résultat faux, sans aucun signal.
    ///
    /// Le pendant en base (avec changement du défaut société entre les deux
    /// pièces) est le test d'intégration d'AC17.
    #[test]
    fn contre_passation_cancels_invoice_entry_account_by_account() {
        use std::collections::BTreeMap;

        fn line(line_total: Decimal, vat_rate: Decimal, account: Option<i64>) -> InvoiceLine {
            InvoiceLine {
                id: 1,
                invoice_id: 1,
                position: 1,
                description: "ligne test".into(),
                quantity: dec!(1),
                unit_price: line_total,
                vat_rate,
                line_total,
                revenue_account_id: account,
                created_at: chrono::NaiveDate::from_ymd_opt(2026, 1, 1)
                    .unwrap()
                    .and_hms_opt(0, 0, 0)
                    .unwrap(),
            }
        }

        let invoice_lines = [
            line(dec!(1000), dec!(8.1), Some(3200)),
            line(dec!(400), dec!(2.6), Some(3400)),
            // Une ligne au repli, pour couvrir aussi le compte par défaut.
            line(dec!(250), dec!(8.1), None),
        ];
        let invoice_je = crate::repositories::invoices::generate_invoice_journal_lines(
            &invoice_lines,
            1100,
            3000,
            Some(2200),
        )
        .unwrap();

        let triplets: Vec<(Decimal, Decimal, Option<i64>)> = invoice_lines
            .iter()
            .map(|l| (l.line_total, l.vat_rate, l.revenue_account_id))
            .collect();
        let credit_je =
            generate_credit_note_journal_lines(&triplets, 1100, 3000, Some(2200)).unwrap();

        // Net par compte = 0 sur CHAQUE compte, pas seulement en total.
        let mut net: BTreeMap<i64, Decimal> = BTreeMap::new();
        for l in invoice_je.iter().chain(credit_je.iter()) {
            *net.entry(l.account_id).or_insert(Decimal::ZERO) += l.debit - l.credit;
        }
        for (account_id, amount) in &net {
            assert_eq!(
                *amount,
                Decimal::ZERO,
                "résidu de {amount} sur le compte {account_id} — les deux écritures ne s'annulent pas"
            );
        }
        assert!(
            net.contains_key(&3200) && net.contains_key(&3400) && net.contains_key(&3000),
            "les 3 comptes de produit doivent être présents (2 explicites + le repli)"
        );
    }
}
