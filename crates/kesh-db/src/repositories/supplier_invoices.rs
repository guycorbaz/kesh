//! Repository des factures fournisseurs & règlement binaire — Story 12.2 (#191).
//!
//! - Le cycle **brouillon → validée → payée** (Story v017-4a, #849) :
//!   [`create_draft`] enregistre un brouillon **sans écriture** ;
//!   [`update_draft`] le remplace ; [`delete_draft`] le supprime ;
//!   [`validate`] poste l'écriture d'achat (D charge / D impôt préalable /
//!   C créanciers, logique TVA Epic 18) et passe la facture en `open`
//!   (« validée ») ; [`unvalidate`] — « Repasser en brouillon » à l'écran —
//!   supprime l'écriture et ramène le brouillon (règle de la facture client) ;
//!   depuis la Story v0172-annulees (#868), une facture **annulée** y repasse
//!   aussi, son achat et sa contre-passation supprimés.
//!   [`create_validated`] compose création et validation, pour les seeds et
//!   les tests.
//! - [`pay`] : règlement binaire. Poste `D 2000 / C contrepartie` (virement →
//!   `bank_account.journal_account_id` ; compte interne → compte libre) et
//!   passe en `paid`. Le compte débité ET le montant sont relus depuis la
//!   ligne de crédit de l'écriture d'achat → solde 2000 garanti à 0 [O-M2].
//! - [`cancel`] : contre-passe l'écriture d'achat (relecture lignes + swap
//!   débit↔crédit, montants positifs) et passe en `cancelled` [DC7]. L'annulée
//!   n'est plus définitive depuis la 0.17.2 : saisie par erreur, elle repasse
//!   en brouillon par [`unvalidate`].
//!
//! Helper de génération d'écriture en kesh-db (PAS kesh-core : `NewJournalEntryLine`
//! et `DbError` y sont définis — dépendance circulaire sinon [O-C1]).

use std::collections::BTreeSet;

use rust_decimal::Decimal;
use sqlx::MySqlPool;

use crate::entities::{
    NewAuditLogEntry, NewJournalEntryLine, NewSupplierInvoice, SettlementChoice, SupplierInvoice,
    SupplierInvoiceLine,
};
use crate::errors::{ClaimSide, DbError, NonPostableAccount, map_db_error};
use crate::repositories::company_invoice_settings::{DesignatedRole, GeneratedLines};
use crate::repositories::invoice_settlements::{self, ClaimSubject};
use crate::repositories::{audit_log, letterings};

/// SELECT scopé multi-tenant (anti-IDOR — toujours `AND company_id = ?`).
const FIND_SCOPED_SQL: &str = "SELECT id, company_id, contact_id, supplier_invoice_number, \
    internal_number, status, invoice_date, due_date, total_amount, creditor_iban, creditor_qr_iban, payment_reference, \
    expected_payment_amount, project_id, purchase_journal_entry_id, settlement_type, settlement_bank_account_id, \
    settlement_account_id, settlement_journal_entry_id, paid_at, version, created_at, updated_at \
    FROM supplier_invoices WHERE id = ? AND company_id = ?";

/// Les colonnes de [`SupplierInvoice`], préfixées de l'alias `si`, pour les
/// requêtes qui joignent d'autres tables (la réconciliation, Story 30-1b) —
/// même liste et même ordre que [`FIND_SCOPED_SQL`]. Qui ajoute un champ à
/// l'entité l'ajoute aux deux.
pub const SUPPLIER_INVOICE_COLUMNS_SI: &str = "si.id, si.company_id, si.contact_id, \
    si.supplier_invoice_number, si.internal_number, si.status, si.invoice_date, si.due_date, \
    si.total_amount, \
    si.creditor_iban, si.creditor_qr_iban, si.payment_reference, si.expected_payment_amount, \
    si.project_id, si.purchase_journal_entry_id, si.settlement_type, \
    si.settlement_bank_account_id, si.settlement_account_id, si.settlement_journal_entry_id, \
    si.paid_at, si.version, si.created_at, si.updated_at";

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
    pub internal_number: Option<String>,
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
        "internalNumber": inv.internal_number,
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
) -> Result<(GeneratedLines, Decimal), DbError> {
    // Story 15-5d (C39) — les rôles de réglage effectivement écrits : les
    // créanciers toujours, la TVA récupérable dans la seule branche qui l'écrit.
    let mut roles: BTreeSet<DesignatedRole> = BTreeSet::new();
    roles.insert(DesignatedRole::Payable);

    // Story v017-4a (#849, C-v017-4-27) — les sommes viennent de la fonction pure
    // unique de `kesh-core`, celle qui donne aussi son total au brouillon : la
    // validation écrit le montant que le brouillon affichait (DRY).
    let pairs: Vec<(Decimal, Decimal)> = lines.iter().map(|(ht, rate, _)| (*ht, *rate)).collect();
    let (_, total_vat, total_ttc) = kesh_core::accounting::purchase_invoice_totals(&pairs);
    let mut entry_lines: Vec<NewJournalEntryLine> = Vec::with_capacity(lines.len() + 2);

    // (0..N) Débit charge par ligne (HT).
    for (line_total, _, expense_account_id) in lines {
        entry_lines.push(NewJournalEntryLine {
            account_id: *expense_account_id,
            debit: *line_total,
            credit: Decimal::ZERO,
            project_id: None,
        });
    }

    // (N) Débit impôt préalable 1171 agrégé — OMIS si Σ TVA = 0.
    if total_vat > Decimal::ZERO {
        let vat_account = recoverable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_vat_recoverable_account_id".into())
        })?;
        roles.insert(DesignatedRole::VatRecoverable);
        entry_lines.push(NewJournalEntryLine {
            account_id: vat_account,
            debit: total_vat,
            credit: Decimal::ZERO,
            project_id: None,
        });
    }

    // (N+1) Crédit créanciers 2000 (TTC).
    entry_lines.push(NewJournalEntryLine {
        account_id: payable_account_id,
        debit: Decimal::ZERO,
        credit: total_ttc,
        project_id: None,
    });

    Ok((
        GeneratedLines {
            lines: entry_lines,
            roles,
        },
        total_ttc,
    ))
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

/// Les statuts d'une facture fournisseur — l'ensemble fermé de
/// `chk_supplier_invoices_status_v2` (Story v017-4a).
pub const SUPPLIER_INVOICE_STATUSES: [&str; 4] = ["draft", "open", "paid", "cancelled"];

/// L'ordre de la liste (Story 33-1b, #926) : une liste fermée, jamais de texte du
/// client dans la requête.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SupplierInvoiceSort {
    /// `invoice_date DESC, id DESC` : l'ordre d'avant la 0.17.5.
    #[default]
    DateDesc,
    /// Le numéro interne, tri **naturel** (`FF-2026-999` avant `FF-2026-1000`,
    /// `NATURAL_SORT_KEY`, MariaDB 10.7 au moins), sans numéro à la fin.
    InternalNumberAsc,
    /// De même, décroissant ; sans numéro à la fin aussi.
    InternalNumberDesc,
}

impl SupplierInvoiceSort {
    fn order_by(self) -> &'static str {
        match self {
            Self::DateDesc => "si.invoice_date DESC, si.id DESC",
            Self::InternalNumberAsc => {
                "si.internal_number IS NULL, NATURAL_SORT_KEY(si.internal_number) ASC, si.id ASC"
            }
            Self::InternalNumberDesc => {
                "si.internal_number IS NULL, NATURAL_SORT_KEY(si.internal_number) DESC, si.id DESC"
            }
        }
    }
}

/// Le filtre de [`list_filtered`], neutre par défaut.
#[derive(Debug, Clone, Copy, Default)]
pub struct SupplierInvoiceListFilter<'a> {
    /// Les statuts retenus ; **vide = tous** (Story v017-4a, AC-15) — la route refuse
    /// un filtre vide ou inconnu avant d'appeler.
    pub statuses: &'a [&'a str],
    /// La recherche (Story 33-1b) : numéro interne, numéro et nom du fournisseur.
    /// `None`, vide ou blanche : aucun filtre.
    pub q: Option<&'a str>,
    /// L'ordre ; défaut : la date.
    pub sort: SupplierInvoiceSort,
}

/// Liste paginée des factures fournisseurs d'une company (les plus récentes
/// d'abord), filtrée par statut. Cf. [`list_filtered`].
pub async fn list(
    pool: &MySqlPool,
    company_id: i64,
    statuses: &[&str],
    limit: i64,
    offset: i64,
) -> Result<(Vec<SupplierInvoiceListItem>, i64), DbError> {
    list_filtered(
        pool,
        company_id,
        SupplierInvoiceListFilter {
            statuses,
            ..Default::default()
        },
        limit,
        offset,
    )
    .await
}

/// Liste paginée des factures fournisseurs d'une company, selon `filtre`. `total`
/// compte le filtre, recherche comprise ; tout passe par des marqueurs, `%` et `_`
/// de la recherche sont échappés (Story 33-1b, #926).
pub async fn list_filtered(
    pool: &MySqlPool,
    company_id: i64,
    filtre: SupplierInvoiceListFilter<'_>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SupplierInvoiceListItem>, i64), DbError> {
    let statuses = filtre.statuses;
    let mut filter = if statuses.is_empty() {
        String::new()
    } else {
        format!(
            " AND si.status IN ({})",
            statuses.iter().map(|_| "?").collect::<Vec<_>>().join(", ")
        )
    };
    let motif = filtre
        .q
        .map(str::trim)
        .filter(|q| !q.is_empty())
        .map(|q| format!("%{}%", crate::util::search::escape_like(q)));
    if motif.is_some() {
        filter.push_str(
            " AND (si.internal_number LIKE ? ESCAPE '\\\\' \
             OR si.supplier_invoice_number LIKE ? ESCAPE '\\\\' \
             OR c.name LIKE ? ESCAPE '\\\\')",
        );
    }
    // JOIN contacts pour le nom du fournisseur (AC11), au décompte aussi : la recherche
    // le lit. INNER JOIN sûr : contact_id est une FK RESTRICT non-nullable.
    let count_sql = format!(
        "SELECT COUNT(*) FROM supplier_invoices si JOIN contacts c ON c.id = si.contact_id \
         WHERE si.company_id = ?{filter}"
    );
    let mut count = sqlx::query_scalar::<_, i64>(&count_sql).bind(company_id);
    for s in statuses {
        count = count.bind(*s);
    }
    if let Some(m) = &motif {
        count = count.bind(m).bind(m).bind(m);
    }
    let total: i64 = count.fetch_one(pool).await.map_err(map_db_error)?;

    let items_sql = format!(
        "SELECT si.id, si.contact_id, c.name AS contact_name, si.supplier_invoice_number, \
         si.internal_number, si.status, si.invoice_date, si.due_date, si.total_amount \
         FROM supplier_invoices si JOIN contacts c ON c.id = si.contact_id \
         WHERE si.company_id = ?{filter} ORDER BY {} LIMIT ? OFFSET ?",
        filtre.sort.order_by()
    );
    let mut q = sqlx::query_as::<_, SupplierInvoiceListItem>(&items_sql).bind(company_id);
    for s in statuses {
        q = q.bind(*s);
    }
    if let Some(m) = &motif {
        q = q.bind(m).bind(m).bind(m);
    }
    let items = q
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok((items, total))
}

/// La passe de **forme** des lignes d'une facture fournisseur — Story v017-3b
/// (#687, D9) : pure, sans I/O ni verrou.
///
/// Pour chaque ligne, **dans cet ordre** : positivité de la quantité, du prix et
/// du produit (`IllegalStateTransition`, « strictement positifs ») ; puis le
/// montant au centime ([`kesh_core::accounting::purchase_line_total`],
/// [`DbError::SupplierLineNotInCentimes`] avec le rang 1-based) ; puis les bornes
/// du taux de TVA. Le premier refus l'emporte, ligne par ligne. Rend les
/// `line_total` dans l'ordre des lignes : l'appelant ne refait **aucun** second
/// produit.
///
/// **Un seul ordre, deux chemins** : la saisie l'appelle (le brouillon,
/// [`create_draft_in_tx`] et [`update_draft_in_tx`], puis [`validate_in_tx`]),
/// et la complétion d'une pièce importée
/// (`POST /imported-supplier-invoices/{id}/complete`) l'appelle avant son
/// contrôle `AMOUNT_MISMATCH`, de sorte qu'une même ligne y reçoit le même refus.
/// `pub` : la route de `kesh-api` l'appelle, et les tests d'intégration de
/// `crates/kesh-db/tests/` l'atteignent.
pub fn purchase_lines_form(
    lines: &[crate::entities::NewSupplierInvoiceLine],
) -> Result<Vec<Decimal>, DbError> {
    let mut totals = Vec::with_capacity(lines.len());
    for (index, line) in lines.iter().enumerate() {
        // (a) Positivité, AVANT le centime : un produit nul ou négatif ne sort
        //     jamais en « pas au centime » (validation P2, R1 ; mutation M10).
        if line.quantity <= Decimal::ZERO
            || line.unit_price <= Decimal::ZERO
            || line.quantity * line.unit_price <= Decimal::ZERO
        {
            return Err(DbError::IllegalStateTransition(
                "chaque ligne doit avoir une quantité et un prix strictement positifs".into(),
            ));
        }
        // (b) Le centime — la seule définition de la règle (kesh-core).
        let line_total = kesh_core::accounting::purchase_line_total(line.quantity, line.unit_price)
            .map_err(|e| DbError::SupplierLineNotInCentimes {
                line: index + 1,
                line_total: e.line_total,
            })?;
        // (c) Les bornes du taux de TVA.
        if line.vat_rate < Decimal::ZERO || line.vat_rate > Decimal::from(100) {
            return Err(DbError::IllegalStateTransition(
                "taux de TVA hors bornes (0-100)".into(),
            ));
        }
        totals.push(line_total);
    }
    Ok(totals)
}

// ---------------------------------------------------------------------------
// Story v017-4a (#849) — le cycle brouillon → validée → payée
// ---------------------------------------------------------------------------

/// Exécute `$body` dans une transaction possédée `$tx` : `COMMIT` sur succès,
/// `ROLLBACK` sinon — l'enveloppe commune des fonctions `pool` de ce cycle.
macro_rules! in_own_tx {
    ($pool:expr, |$tx:ident| $body:expr) => {{
        let mut $tx = $pool.begin().await.map_err(map_db_error)?;
        match $body {
            Ok(v) => {
                $tx.commit().await.map_err(map_db_error)?;
                Ok(v)
            }
            Err(e) => {
                let _ = $tx.rollback().await;
                Err(e)
            }
        }
    }};
}

/// Enregistre un **brouillon** (Story v017-4a, AC-6), transaction comprise —
/// cf. [`create_draft_in_tx`].
pub async fn create_draft(
    pool: &MySqlPool,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    in_own_tx!(pool, |tx| create_draft_in_tx(&mut tx, new, user_id).await)
}

/// Enregistre une facture **et la valide** dans une même transaction
/// (Story v017-4a, AC-12) : [`create_draft_in_tx`] puis [`validate_in_tx`] —
/// aucun corps dupliqué. **Pour les seeds et les tests** : aucune route ne
/// l'appelle (une facture se saisit en brouillon, puis se valide).
pub async fn create_validated(
    pool: &MySqlPool,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    in_own_tx!(pool, |tx| create_validated_in_tx(&mut tx, new, user_id)
        .await)
}

/// [`create_validated`] dans une transaction fournie.
pub async fn create_validated_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    let company_id = new.company_id;
    let draft = create_draft_in_tx(tx, new, user_id).await?;
    validate_in_tx(
        tx,
        company_id,
        draft.invoice.id,
        draft.invoice.version,
        user_id,
    )
    .await
}

/// Les contrôles **de saisie** d'une facture fournisseur, communs à la
/// création d'un brouillon et à son `PUT` (Story v017-4a, AC-6, AC-8 ;
/// C-v017-4-4), dans l'ordre de l'ancienne création : lignes non vides, projet
/// (existe, de la société, non archivé), fournisseur, **forme** des lignes
/// ([`purchase_lines_form`]), comptes de charge (de la société, actifs, de type
/// charge, imputables — les non imputables nommés ensemble).
///
/// ⛔ **Aucun verrou** (C-v017-4-28) : le projet est lu par une lecture
/// ordinaire, non par `projects::validate_taggable_in_tx` (sentinelle
/// `companies` et projet `FOR UPDATE`) ; les comptes de même. Les jugements qui
/// comptent pour le journal (projet archivé, compte archivé ou non imputable
/// entre-temps) sont **refaits sous verrou** à la validation ([`validate_in_tx`]).
/// Les contrôles liés à l'écriture (exercice, réglages, comptes désignés) ne
/// sont pas faits ici : un brouillon hors de tout exercice s'enregistre.
///
/// Rend les `line_total` de la passe de forme, dans l'ordre des lignes.
async fn check_draft_input_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    new: &NewSupplierInvoice,
) -> Result<Vec<Decimal>, DbError> {
    if new.lines.is_empty() {
        return Err(DbError::IllegalStateTransition(
            "une facture fournisseur doit avoir au moins une ligne".into(),
        ));
    }
    if let Some(pid) = new.project_id {
        let archived: Option<bool> =
            sqlx::query_scalar("SELECT archived FROM projects WHERE id = ? AND company_id = ?")
                .bind(pid)
                .bind(new.company_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_db_error)?;
        match archived {
            None => return Err(DbError::NotFound),
            Some(true) => {
                return Err(DbError::IllegalStateTransition(
                    "le projet analytique est archivé".into(),
                ));
            }
            Some(false) => {}
        }
    }
    let supplier: Option<bool> =
        sqlx::query_scalar("SELECT is_supplier FROM contacts WHERE id = ? AND company_id = ?")
            .bind(new.contact_id)
            .bind(new.company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    if !supplier.ok_or(DbError::NotFound)? {
        return Err(DbError::IllegalStateTransition(
            "le contact n'est pas un fournisseur".into(),
        ));
    }
    let line_totals = purchase_lines_form(&new.lines)?;
    let mut non_postable: Vec<NonPostableAccount> = Vec::new();
    for line in &new.lines {
        let acct: Option<(bool, bool, String, String)> = sqlx::query_as(
            "SELECT active, postable, account_type, number FROM accounts \
             WHERE id = ? AND company_id = ?",
        )
        .bind(line.expense_account_id)
        .bind(new.company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
        match acct {
            Some((true, true, ref t, _)) if t == "Expense" => {}
            Some((true, false, ref t, number)) if t == "Expense" => {
                non_postable.push(NonPostableAccount {
                    account_id: line.expense_account_id,
                    account_number: number,
                });
            }
            _ => return Err(DbError::InactiveOrInvalidAccounts),
        }
    }
    if !non_postable.is_empty() {
        return Err(DbError::accounts_not_postable(non_postable));
    }
    Ok(line_totals)
}

/// Le TTC d'un brouillon, par la fonction pure unique de `kesh-core`
/// ([`kesh_core::accounting::purchase_invoice_totals`], C-v017-4-27).
fn draft_total(lines: &[crate::entities::NewSupplierInvoiceLine], totals: &[Decimal]) -> Decimal {
    let pairs: Vec<(Decimal, Decimal)> = lines
        .iter()
        .zip(totals)
        .map(|(l, t)| (*t, l.vat_rate))
        .collect();
    kesh_core::accounting::purchase_invoice_totals(&pairs).2
}

/// Insère les lignes d'une facture, positions renumérotées depuis 0.
async fn insert_lines(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    invoice_id: i64,
    lines: &[crate::entities::NewSupplierInvoiceLine],
    totals: &[Decimal],
) -> Result<(), DbError> {
    for (position, (line, line_total)) in lines.iter().zip(totals).enumerate() {
        sqlx::query(
            "INSERT INTO supplier_invoice_lines \
             (supplier_invoice_id, position, description, quantity, unit_price, vat_rate, \
              line_total, expense_account_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(invoice_id)
        .bind(position as i32)
        .bind(&line.description)
        .bind(line.quantity)
        .bind(line.unit_price)
        .bind(line.vat_rate)
        .bind(*line_total)
        .bind(line.expense_account_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    }
    Ok(())
}

/// Relit une facture et ses lignes dans la transaction.
async fn reread(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    let invoice = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let lines = fetch_lines(tx, id).await?;
    Ok(SupplierInvoiceWithLines { invoice, lines })
}

/// Enregistre un **brouillon** de facture fournisseur, **sans écriture**
/// (Story v017-4a, #849, AC-6 ; C-v017-4-4), dans la transaction fournie.
///
/// Contrôles de saisie de [`check_draft_input_in_tx`], sans verrou ; total
/// (`total_amount`, TTC) par la fonction pure de `kesh-core` ; `INSERT` en
/// `'draft'`, `purchase_journal_entry_id` nul ; lignes ; audit
/// `supplier_invoice.created` (instantané, **sans** `journalEntryId`).
///
/// Réutilisé par la route `POST /supplier-invoices` et par la complétion d'un
/// document (`POST /imported-supplier-invoices/{id}/complete`), qui l'appelle
/// dans **la même transaction** que le verrou du document et `mark_completed`
/// (DC6 12-5c : jamais de double facture).
///
/// # Verrous
///
/// **Aucun verrou exclusif** : les lectures sont ordinaires ; seule
/// l'insertion prend les verrous **partagés** de ses clés étrangères (société,
/// fournisseur, projet, comptes des lignes). Ils ne sont pas compatibles avec
/// tout — une validation aux comptes croisés tient l'exclusif —, d'où le rejeu
/// gardé de la route (`retry_on_deadlock("supplier_invoices::create", …)`,
/// C-v017-4-37).
pub async fn create_draft_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    let line_totals = check_draft_input_in_tx(tx, &new).await?;
    let total = draft_total(&new.lines, &line_totals);
    let inv_id: i64 = sqlx::query(
        "INSERT INTO supplier_invoices \
         (company_id, contact_id, supplier_invoice_number, status, invoice_date, due_date, \
          total_amount, creditor_iban, creditor_qr_iban, payment_reference, \
          expected_payment_amount, project_id, purchase_journal_entry_id) \
         VALUES (?, ?, ?, 'draft', ?, ?, ?, ?, ?, ?, ?, ?, NULL)",
    )
    .bind(new.company_id)
    .bind(new.contact_id)
    .bind(&new.supplier_invoice_number)
    .bind(new.invoice_date)
    .bind(new.due_date)
    .bind(total)
    .bind(&new.creditor_iban)
    .bind(&new.creditor_qr_iban)
    .bind(&new.payment_reference)
    .bind(new.expected_payment_amount)
    .bind(new.project_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .last_insert_id() as i64;
    insert_lines(tx, inv_id, &new.lines, &line_totals).await?;
    let created = reread(tx, new.company_id, inv_id).await?;
    audit_log::insert_in_tx(
        &mut *tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.created".to_string(),
            "supplier_invoice".to_string(),
            inv_id,
            Some(serde_json::json!({
                "supplierInvoice": snapshot_json(&created.invoice, &created.lines),
            })),
        ),
    )
    .await?;
    Ok(created)
}

/// Verrouille la facture `id` (`FOR UPDATE`) et exige qu'elle soit un
/// **brouillon** à la version `version` (Story v017-4a, AC-8) : facture
/// absente ou d'une autre société → [`DbError::NotFound`] ; pas un brouillon →
/// [`DbError::IllegalStateTransition`] (avant la version) ; version périmée →
/// [`DbError::OptimisticLockConflict`].
///
/// Appelée par la route `PUT` **avant** les règles « document » (qui lisent le
/// document joint), puis de nouveau par [`update_draft_in_tx`] — le second
/// `FOR UPDATE` ne prend aucun verrou neuf.
pub async fn lock_draft_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    version: i32,
) -> Result<SupplierInvoice, DbError> {
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    if inv.status != "draft" {
        return Err(DbError::IllegalStateTransition(format!(
            "seul un brouillon se modifie (statut actuel : '{}')",
            inv.status
        )));
    }
    if inv.version != version {
        return Err(DbError::OptimisticLockConflict);
    }
    Ok(inv)
}

/// Modifie un brouillon, transaction comprise — cf. [`update_draft_in_tx`].
/// Pour les tests et le dépôt : la route tient sa propre transaction, où les
/// règles « document » passent entre le verrou et la modification.
pub async fn update_draft(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    in_own_tx!(pool, |tx| update_draft_in_tx(
        &mut tx, company_id, id, version, new, user_id
    )
    .await)
}

/// **Remplace** un brouillon (Story v017-4a, #849, AC-8 ; C-v017-4-5) : en-tête
/// et **toutes** les lignes (supprimées puis réinsérées, positions depuis 0),
/// mêmes contrôles qu'à la création, `version + 1`, aucune écriture au journal,
/// audit `supplier_invoice.updated` (instantanés avant et après).
///
/// `new.company_id` est ignoré au profit de `company_id` (la société de la
/// requête). Ordre des verrous : la facture `FOR UPDATE`
/// ([`lock_draft_in_tx`]), puis les clés étrangères des lignes insérées.
pub async fn update_draft_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    mut new: NewSupplierInvoice,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    lock_draft_in_tx(tx, company_id, id, version).await?;
    new.company_id = company_id;
    let before_lines = fetch_lines(tx, id).await?;
    let before = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let line_totals = check_draft_input_in_tx(tx, &new).await?;
    let total = draft_total(&new.lines, &line_totals);
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET contact_id = ?, supplier_invoice_number = ?, \
         invoice_date = ?, due_date = ?, total_amount = ?, creditor_iban = ?, \
         creditor_qr_iban = ?, payment_reference = ?, expected_payment_amount = ?, \
         project_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'draft'",
    )
    .bind(new.contact_id)
    .bind(&new.supplier_invoice_number)
    .bind(new.invoice_date)
    .bind(new.due_date)
    .bind(total)
    .bind(&new.creditor_iban)
    .bind(&new.creditor_qr_iban)
    .bind(&new.payment_reference)
    .bind(new.expected_payment_amount)
    .bind(new.project_id)
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows != 1 {
        return Err(DbError::OptimisticLockConflict);
    }
    sqlx::query("DELETE FROM supplier_invoice_lines WHERE supplier_invoice_id = ?")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    insert_lines(tx, id, &new.lines, &line_totals).await?;
    let after = reread(tx, company_id, id).await?;
    audit_log::insert_in_tx(
        &mut *tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.updated".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(serde_json::json!({
                "before": snapshot_json(&before, &before_lines),
                "after": snapshot_json(&after.invoice, &after.lines),
            })),
        ),
    )
    .await?;
    Ok(after)
}

/// Ce que rend la suppression d'un brouillon.
#[derive(Debug, Clone)]
pub struct DeletedDraft {
    /// Le document qui était joint, rendu « à traiter », s'il y en avait un.
    pub detached_document_id: Option<i64>,
}

/// Supprime un brouillon, transaction comprise — cf. [`delete_draft_in_tx`].
/// **Sans enveloppe de rejeu** : la route l'enveloppe ; le test piloté de
/// l'ordre des verrous (M20-quater) l'appelle nu, pour qu'un interblocage se
/// voie.
pub async fn delete_draft(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<DeletedDraft, DbError> {
    in_own_tx!(pool, |tx| delete_draft_in_tx(
        &mut tx, company_id, id, user_id
    )
    .await)
}

/// Supprime un **brouillon** de facture fournisseur (Story v017-4a, #849,
/// AC-9 ; C-v017-4-6, C-v017-4-34) ; son document joint, s'il en a un, est
/// rendu « à traiter ».
///
/// Refus : validée → [`DbError::SupplierInvoiceMustBeUnvalidatedFirst`] ;
/// payée, annulée → [`DbError::IllegalStateTransition`] ; présente dans un lot
/// de paiement, quel que soit le sort du lot →
/// [`DbError::SupplierInvoiceHasPaymentBatchHistory`] (limite L8 : jamais une
/// erreur de clé étrangère). Un refus annule tout, détachement compris.
///
/// # Ordre des verrous — la facture d'abord, puis le document
///
/// La facture `FOR UPDATE`, son statut, les articles de lot en lecture
/// verrouillante, **puis** [`super::imported_supplier_invoices::detach_in_tx`]
/// (l'entrée de `uq_imported_supplier_invoice`, ou son intervalle), puis le
/// `DELETE`. C'est l'ordre « facture → entrée d'unicité » de
/// [`super::imported_supplier_invoices::attach_in_tx`] : un rattachement
/// concurrent attend la facture. L'ordre inverse formerait un cycle avec lui
/// (doc-comment de `detach_in_tx`) — test piloté
/// `deleting_a_draft_during_an_attachment_does_not_deadlock` (M20-quater).
///
/// Audits `imported_supplier_invoice.detached` (si un document est rendu) puis
/// `supplier_invoice.deleted` (instantané), dans la transaction.
pub async fn delete_draft_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<DeletedDraft, DbError> {
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    match inv.status.as_str() {
        "draft" => {}
        "open" => return Err(DbError::SupplierInvoiceMustBeUnvalidatedFirst),
        other => {
            return Err(DbError::IllegalStateTransition(format!(
                "seul un brouillon se supprime (statut actuel : '{other}')"
            )));
        }
    }
    let batch: Option<i64> = sqlx::query_scalar(
        "SELECT payment_batch_id FROM payment_batch_items WHERE supplier_invoice_id = ? \
         ORDER BY payment_batch_id DESC LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if let Some(payment_batch_id) = batch {
        return Err(DbError::SupplierInvoiceHasPaymentBatchHistory { payment_batch_id });
    }
    let detached = super::imported_supplier_invoices::detach_in_tx(tx, company_id, id).await?;
    if let Some(doc) = &detached {
        audit_log::insert_in_tx(
            &mut *tx,
            NewAuditLogEntry::user(
                user_id,
                "imported_supplier_invoice.detached".to_string(),
                "imported_supplier_invoice".to_string(),
                doc.id,
                Some(serde_json::json!({
                    "supplier_invoice_id": id,
                    "original_filename": doc.original_filename,
                })),
            ),
        )
        .await?;
    }
    let lines = fetch_lines(tx, id).await?;
    audit_log::insert_in_tx(
        &mut *tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.deleted".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(serde_json::json!({ "supplierInvoice": snapshot_json(&inv, &lines) })),
        ),
    )
    .await?;
    sqlx::query("DELETE FROM supplier_invoice_lines WHERE supplier_invoice_id = ?")
        .bind(id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let rows = sqlx::query(
        "DELETE FROM supplier_invoices WHERE id = ? AND company_id = ? AND status = 'draft'",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows != 1 {
        return Err(DbError::Invariant(
            "delete_draft_in_tx : brouillon verrouillé, aucune ligne supprimée".into(),
        ));
    }
    Ok(DeletedDraft {
        detached_document_id: detached.map(|d| d.id),
    })
}

/// Le libellé d'une pièce au journal : la forme longue de
/// [`kesh_core::invoice_format::supplier_invoice_long_label`], à défaut `#<id>`, ce qui ne
/// se produit que pour une facture posée hors de l'application. Écrit une seule fois pour
/// l'écriture d'achat et le règlement (C-e331-10).
fn document_long_label(id: i64, interne: Option<&str>, fournisseur: Option<&str>) -> String {
    kesh_core::invoice_format::supplier_invoice_long_label(interne, fournisseur)
        .unwrap_or_else(|| format!("#{id}"))
}

/// Borne du saut d'un numéro déjà pris (C-e331-8) : le saut n'est qu'un filet pour une
/// donnée posée hors de l'application, et au-delà, c'est cette donnée qu'il faut regarder.
const MAX_INTERNAL_NUMBER_SKIPS: i64 = 100_000;

/// Attribue le numéro interne d'une facture fournisseur à sa validation (Story 33-1a,
/// #926), étape (3 bis) de [`validate_in_tx`] : l'appelant tient déjà les réglages et
/// l'exercice `fy` en `FOR UPDATE`.
///
/// **La séquence appartient à l'année de début de l'exercice** (C-e331-8), si bien que
/// deux exercices commençant la même année civile la partagent :
///
/// 1. les **autres** exercices de l'année, tous statuts (un exercice clos a émis des
///    numéros) : leurs `id` par une lecture ordinaire, puis leur compteur `FOR UPDATE`
///    **un par un**, triés par date (patron de [`super::fiscal_years::close`] : une lecture
///    verrouillante par intervalle prendrait ses verrous dans l'ordre du plan). Une seule
///    ligne par année, le cas courant : aucun verrou neuf ;
/// 2. le départ est le plus grand de ces compteurs et de celui de `fy`, jamais un numéro
///    libre plus bas : c'est ce qui fait qu'un numéro supprimé ne revient pas ;
/// 3. le contrôle de collision ne sert plus que de filet, **vers le haut**, pour une donnée
///    posée hors de l'application ; sans lui, elle bloquerait toute validation ;
/// 4. seul le compteur de `fy` avance. Les compteurs ne redescendent jamais.
///
/// ⚠️ **Les lectures de compteur et le contrôle de collision sont verrouillants** :
/// l'instantané de la transaction est pris avant l'attente sur les réglages (`fetch_lines`
/// précède l'étape (2 bis)), et une lecture ordinaire y rendrait un compteur périmé
/// (C-e331-2). La liste des `id` du point 1, elle, ne l'est pas : un exercice créé pendant
/// cette attente, et dont un numéro serait émis puis supprimé dans le même temps, y
/// échapperait. Course résiduelle assumée, sans chemin réaliste (C-e331-23).
///
/// Le nom d'exercice reste vide : le format n'a pas de `{FY}`
/// ([`kesh_core::invoice_format::validate_numbering_template`]), et `render` refuserait sinon
/// un nom à accolade. Une erreur de rendu ne vient donc que d'un format posé hors de
/// l'application : `DbError::Invariant`, comme aux sites client et avoir.
async fn attribute_internal_number_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    fy: &crate::entities::FiscalYear,
    format: &str,
) -> Result<String, DbError> {
    use chrono::Datelike;
    let year = fy.start_date.year();
    let borne = |annee: i32| {
        chrono::NaiveDate::from_ymd_opt(annee, 1, 1).ok_or_else(|| {
            DbError::Invariant(format!("année de début d'exercice inattendue : {annee}"))
        })
    };
    let autres: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM fiscal_years WHERE company_id = ? AND start_date >= ? AND start_date < ? \
         AND id <> ? ORDER BY start_date",
    )
    .bind(company_id)
    .bind(borne(year)?)
    .bind(borne(year + 1)?)
    .bind(fy.id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    const COMPTEUR_SQL: &str = "SELECT supplier_invoice_next_number FROM fiscal_years \
         WHERE id = ? AND company_id = ? FOR UPDATE";
    let mut seq: i64 = 1;
    for id in autres.into_iter().chain(std::iter::once(fy.id)) {
        let compteur: i64 = sqlx::query_scalar(COMPTEUR_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_one(&mut **tx)
            .await
            .map_err(map_db_error)?;
        seq = seq.max(compteur);
    }

    for _ in 0..MAX_INTERNAL_NUMBER_SKIPS {
        let numero = kesh_core::invoice_format::render(format, year, "", seq).map_err(|e| {
            DbError::Invariant(format!(
                "rendu du numéro interne échoué (format invalide ?) : {e}"
            ))
        })?;
        let pris: Option<i64> = sqlx::query_scalar(
            "SELECT id FROM supplier_invoices WHERE company_id = ? AND internal_number = ? \
             FOR UPDATE",
        )
        .bind(company_id)
        .bind(&numero)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
        if pris.is_some() {
            seq += 1;
            continue;
        }
        sqlx::query(
            "UPDATE fiscal_years SET supplier_invoice_next_number = ?, updated_at = updated_at \
             WHERE id = ? AND company_id = ?",
        )
        .bind(seq + 1)
        .bind(fy.id)
        .bind(company_id)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
        return Ok(numero);
    }
    Err(DbError::Invariant(format!(
        "numéro interne introuvable après {MAX_INTERNAL_NUMBER_SKIPS} essais, à partir de {seq}"
    )))
}

/// Valide un brouillon, transaction comprise — cf. [`validate_in_tx`].
pub async fn validate(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    in_own_tx!(pool, |tx| validate_in_tx(
        &mut tx, company_id, id, version, user_id
    )
    .await)
}

/// Dévalide une facture, transaction comprise — cf. [`unvalidate_in_tx`].
pub async fn unvalidate(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    in_own_tx!(pool, |tx| unvalidate_in_tx(
        &mut tx, company_id, id, version, user_id
    )
    .await)
}

/// **Dévalide** une facture fournisseur validée (Story v017-4a, #849, AC-11 ;
/// C-v017-4-8) : elle redevient brouillon, et son écriture d'achat est
/// **supprimée** — la règle de la facture client (`invoices::unvalidate`), non
/// une contre-passation. Le trou de numérotation est assumé (limite L2) ; le
/// journal d'audit garde l'écriture (`journal_entry.deleted`).
///
/// # Une facture annulée repasse en brouillon (Story v0172-annulees, #868)
///
/// Depuis la 0.17.2, le geste — « Repasser en brouillon » à l'écran — vaut
/// aussi pour une facture **annulée** : son écriture d'achat **et** sa
/// contre-passation sont supprimées, contre-passation d'abord (C-v0172a-2),
/// par le socle `delete_in_tx` ; les paires de lettrage `reversal` que
/// l'annulation avait posées entre elles sont dissoutes avant (C-v0172a-3) ;
/// le paiement détaché d'une annulée qui avait été payée doit être
/// contre-passé d'abord (C-v0172a-5). La facture garde ses lignes, son
/// numéro, son fournisseur ; son document reste joint (C-v0172a-12).
///
/// # Précédence (testée paire par paire) — composée par écriture (C-v0172a-19)
///
/// `NotFound` → statut (payée : [`SupplierUnvalidationBlocker::Paid`], ou
/// `PaidReconciled` si le règlement est rapproché ; brouillon :
/// `IllegalStateTransition`) → version → lot `generated` → (annulée) paiement
/// détaché non contre-passé → écriture d'achat rapprochée, puis (annulée)
/// contre-passation rapprochée → (annulée) pré-contrôle de l'exercice clos de
/// l'achat ou de la contre-passation (C-v0172a-4 : le mode `System` de la
/// dissolution exige un exercice ouvert) → les refus de
/// [`super::journal_entries::delete_in_tx`] pour la **contre-passation**, puis
/// pour l'**achat** : exercice clos, exercice postérieur clos, écriture
/// contre-passée (inatteignable : la contre-passation part d'abord), période
/// verrouillée, lettrage.
///
/// # Ordre des verrous — tous avant toute lecture ordinaire
///
/// La facture `FOR UPDATE` ; (payée) la transaction rapprochée du règlement
/// en lecture verrouillante ; les articles de lot `generated` et la
/// transaction rapprochée en **lecture verrouillante** (`LOCK IN SHARE MODE`,
/// patron de [`lock_for_bank_settlement_in_tx`] (c)) ; (annulée) l'écriture
/// d'achat et son exercice, puis la contre-passation et son exercice,
/// `FOR UPDATE` — la contre-passation trouvée par une lecture **verrouillante**
/// sur `reverses_entry_id`, jamais par `journal_entries::reversed_by`, lecture
/// ordinaire qui ouvrirait l'instantané avant les verrous (V1-1, M14) ; les
/// transactions rapprochées des deux écritures ; **puis seulement** les
/// lectures ordinaires (audit du paiement détaché, `fetch_lines`) ; les lignes
/// (dissolution) ; l'`UPDATE` ; enfin `delete_in_tx` ×2. Rejouée par la route
/// (`retry_on_deadlock("supplier_invoices::unvalidate", …)`).
///
/// ⚠️ **Second appelant de `delete_in_tx` avec `enforce_ownership = false`**,
/// après `invoices::unvalidate` : la facture supprime **ses** propres
/// écritures, sous ses propres gardes.
///
/// ⚠️ *Lecture avant le premier acte de `delete_in_tx`* (revue P1, B4 = E-4) :
/// `fetch_lines`, lecture ordinaire pour l'instantané d'audit, précède le verrou
/// d'écriture et d'exercice de `delete_in_tx`, dont le doc-comment exige l'inverse
/// (« une lecture ordinaire placée avant le verrou figerait une vue antérieure »).
/// L'écart est assumé, comme dans `invoices::unvalidate` : la contre-passation de
/// l'écriture d'achat ne passe que par `cancel_in_tx`, qui attend le verrou de la
/// facture tenu ici, si bien que `reversed_by` ne peut changer pendant la
/// transaction ; et `books_locked_through` est lu sans verrou par conception.
///
/// [`SupplierUnvalidationBlocker::Paid`]: crate::errors::SupplierUnvalidationBlocker::Paid
pub async fn unvalidate_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use super::journal_entries::{DETACHED_SETTLEMENT_ENTRY, DETACHED_SETTLEMENT_TRACES};
    use crate::errors::{SupplierUnvalidationBlocker as B, SupplierUnvalidationHint as Hint};

    // (1) La facture, sous verrou.
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    // Revue P1, E-6 : un numéro vide ou blanc vaut absent (même règle que K8).
    // Story 33-1b (#926) : la forme courte, le numéro interne d'abord.
    let document_label = kesh_core::invoice_format::supplier_invoice_short_label(
        inv.internal_number.as_deref(),
        inv.supplier_invoice_number.as_deref(),
    );
    let refus = |blocker, document_id, hint| DbError::SupplierInvoiceNotUnvalidatable {
        blocker,
        document_id,
        document_label: document_label.clone(),
        hint,
    };
    //     Le statut : validée ou annulée passent ; payée → le règlement est-il
    //     rapproché ? (lecture verrouillante, C-v0172a-6) ; brouillon → refus.
    match inv.status.as_str() {
        "open" | "cancelled" => {}
        "paid" => {
            let reconciled = match inv.settlement_journal_entry_id {
                Some(settlement) => matched_transaction_locked(tx, settlement).await?,
                None => None,
            };
            return Err(match reconciled {
                Some((tx_id, booking_date)) => refus(
                    B::PaidReconciled,
                    Some(tx_id),
                    Hint {
                        transaction_date: Some(booking_date),
                        ..Hint::default()
                    },
                ),
                None => refus(B::Paid, None, Hint::default()),
            });
        }
        other => {
            return Err(DbError::IllegalStateTransition(format!(
                "seule une facture validée ou annulée repasse en brouillon (statut actuel : '{other}')"
            )));
        }
    }
    if inv.version != version {
        return Err(DbError::OptimisticLockConflict);
    }
    let je_id = inv.purchase_journal_entry_id.ok_or_else(|| {
        DbError::Invariant("facture fournisseur validée sans écriture d'achat".into())
    })?;

    // (2) Le lot `generated` (lecture verrouillante).
    let batch: Option<i64> = sqlx::query_scalar(
        "SELECT pb.id FROM payment_batch_items pbi \
         JOIN payment_batches pb ON pb.id = pbi.payment_batch_id \
         WHERE pbi.supplier_invoice_id = ? AND pb.status = 'generated' LIMIT 1 \
         LOCK IN SHARE MODE",
    )
    .bind(id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if let Some(batch_id) = batch {
        return Err(refus(
            B::InPaymentBatch,
            Some(batch_id),
            Hint {
                batch_id: Some(batch_id),
                ..Hint::default()
            },
        ));
    }

    // (3) Annulée — les verrous des deux écritures et de leurs exercices.
    let cancelled = inv.status == "cancelled";
    let reversal = if cancelled {
        let (purchase_fy_status,): (String,) = sqlx::query_as(
            "SELECT fy.status FROM journal_entries je \
             JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
             WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
        )
        .bind(je_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| DbError::Invariant("facture fournisseur sans écriture d'achat".into()))?;
        // ⛔ Lecture VERROUILLANTE sur `reverses_entry_id`, jamais
        // `journal_entries::reversed_by` (lecture ordinaire) : l'instantané ne
        // s'ouvre qu'après le dernier verrou (V1-1, M14).
        let found: Vec<(i64, i64, String)> = sqlx::query_as(
            "SELECT je.id, fy.id, fy.status FROM journal_entries je \
             JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
             WHERE je.reverses_entry_id = ? AND je.company_id = ? FOR UPDATE",
        )
        .bind(je_id)
        .bind(company_id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
        // C-v0172a-16 — une annulée a exactement une contre-passation liée ;
        // aucun geste ne produit un autre état.
        let [(cp_id, cp_fy_id, cp_fy_status)] = <[_; 1]>::try_from(found).map_err(|found| {
            DbError::Invariant(format!(
                "facture fournisseur annulée {id} : {} contre-passation(s) liée(s) à son achat",
                found.len()
            ))
        })?;
        Some((purchase_fy_status, cp_id, cp_fy_id, cp_fy_status))
    } else {
        None
    };
    //     Les transactions rapprochées de l'achat, puis de la contre-passation
    //     (lectures verrouillantes).
    let matched_purchase = matched_transaction_locked(tx, je_id).await?;
    let matched_reversal = match &reversal {
        Some((_, cp_id, _, _)) => matched_transaction_locked(tx, *cp_id).await?,
        None => None,
    };

    // (4) Annulée — lectures ORDINAIRES, après le dernier verrou : le paiement
    //     détaché que nomme chaque trace d'annulation (AC-6, C-v0172a-5). Le
    //     refus nomme la première écriture qui existe et n'est pas contre-passée.
    if cancelled {
        // `JSON_VALUE` rend un `LONGBLOB` : converti ici en entier (une valeur
        // illisible vaut 0, qu'aucune écriture ne porte) ; `NULL` pour une
        // annulation sans paiement.
        let traces: Vec<Option<i64>> = sqlx::query_scalar(&format!(
            "SELECT CAST({DETACHED_SETTLEMENT_ENTRY} AS SIGNED) FROM {DETACHED_SETTLEMENT_TRACES} \
               AND si.id = ? ORDER BY al.id"
        ))
        .bind(company_id)
        .bind(company_id)
        .bind(id)
        .fetch_all(&mut **tx)
        .await
        .map_err(map_db_error)?;
        for entry_id in traces.into_iter().flatten() {
            let entry: Option<(i64, chrono::NaiveDate)> = sqlx::query_as(
                "SELECT entry_number, entry_date FROM journal_entries WHERE id = ? AND company_id = ?",
            )
            .bind(entry_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
            let Some((entry_number, entry_date)) = entry else {
                continue;
            };
            // Sans verrou : une contre-passation ne s'annule pas ; une
            // contre-passation concurrente ne rend le refus que conservateur.
            if super::journal_entries::reversed_by(&mut **tx, company_id, entry_id)
                .await?
                .is_some()
            {
                continue;
            }
            let transaction_date: Option<chrono::NaiveDate> = sqlx::query_scalar(
                "SELECT booking_date FROM bank_transactions WHERE matched_entry_id = ? \
                 ORDER BY id LIMIT 1",
            )
            .bind(entry_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
            return Err(refus(
                B::DetachedPayment,
                Some(entry_id),
                Hint {
                    transaction_date,
                    entry_id: Some(entry_id),
                    entry_number: Some(entry_number),
                    entry_date: Some(entry_date),
                    ..Hint::default()
                },
            ));
        }
    }

    // (5) Le rapprochement : l'achat, puis la contre-passation.
    if let Some((tx_id, booking_date)) = matched_purchase.or(matched_reversal) {
        return Err(refus(
            B::MatchedBankTransaction,
            Some(tx_id),
            Hint {
                transaction_date: Some(booking_date),
                ..Hint::default()
            },
        ));
    }

    // (6) Annulée — pré-contrôle de l'exercice clos (C-v0172a-4) : le mode
    //     `System` de la dissolution suppose un exercice ouvert tenu.
    if let Some((purchase_fy_status, _, _, cp_fy_status)) = &reversal
        && (purchase_fy_status == "Closed" || cp_fy_status == "Closed")
    {
        return Err(DbError::FiscalYearClosed);
    }

    let lines = fetch_lines(tx, id).await?;
    let snapshot = snapshot_json(&inv, &lines);

    // (7) Annulée — les paires `reversal` de l'annulation se dissolvent, et
    //     elles seules (C-v0172a-3) ; l'exercice tenu est celui de la
    //     contre-passation, verrouillé en (3) et ouvert (6).
    if let Some((_, cp_id, cp_fy_id, _)) = &reversal {
        for key in letterings::reversal_groups_within_entries(tx, company_id, je_id, *cp_id).await?
        {
            letterings::dissolve_group_in_tx(
                tx,
                company_id,
                key,
                letterings::Mode::System {
                    held_open_fiscal_year_id: *cp_fy_id,
                },
                letterings::Actor {
                    user_id,
                    api_key_id: None,
                },
            )
            .await?;
        }
    }

    // (8) Un seul `UPDATE` : deux tiendraient un état qui viole
    //     `chk_supplier_invoices_entry_iff_not_draft`. Gardé par le statut LU
    //     en (1). `chk_supplier_invoices_draft_without_settlement` tient : une
    //     annulée n'a plus de colonne `settlement_*` (vidées par `cancel_in_tx`).
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET status = 'draft', purchase_journal_entry_id = NULL, \
         version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = ?",
    )
    .bind(id)
    .bind(company_id)
    .bind(version)
    .bind(&inv.status)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (9) Les écritures partent APRÈS que la facture a lâché sa référence
    //     (clé étrangère `RESTRICT`) : la contre-passation d'abord — elle n'est
    //     contre-passée par rien, et la clé `reverses_entry_id` est `RESTRICT` —,
    //     puis l'achat, dont l'étape 3-bis passe alors.
    if let Some((_, cp_id, _, _)) = &reversal {
        super::journal_entries::delete_in_tx(tx, company_id, *cp_id, user_id, None, false).await?;
    }
    super::journal_entries::delete_in_tx(tx, company_id, je_id, user_id, None, false).await?;

    // (10) La trace du geste (AC-9).
    let mut details = serde_json::json!({
        "supplierInvoice": snapshot,
        "previousStatus": inv.status,
        "purchaseJournalEntryId": je_id,
    });
    if let Some((_, cp_id, _, _)) = &reversal {
        details["reversalJournalEntryId"] = serde_json::json!(cp_id);
    }
    audit_log::insert_in_tx(
        &mut *tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.unvalidated".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(details),
        ),
    )
    .await?;
    reread(tx, company_id, id).await
}

/// La transaction bancaire rapprochée de l'écriture `entry_id`, et sa date de
/// comptabilisation — **lecture verrouillante** (`LOCK IN SHARE MODE`), pour
/// [`unvalidate_in_tx`].
async fn matched_transaction_locked(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    entry_id: i64,
) -> Result<Option<(i64, chrono::NaiveDate)>, DbError> {
    sqlx::query_as(
        "SELECT id, booking_date FROM bank_transactions WHERE matched_entry_id = ? \
         ORDER BY id LIMIT 1 LOCK IN SHARE MODE",
    )
    .bind(entry_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)
}

/// **Valide** un brouillon de facture fournisseur (Story v017-4a, #849, AC-10 ;
/// C-v017-4-7), dans la transaction fournie : l'écriture d'achat est passée —
/// **la même** que passait la création de la 0.17.0-rc1 pour les mêmes données
/// —, la facture devient `open` (« validée »), `version + 1`, audit
/// `supplier_invoice.validated` (`journalEntryId`).
///
/// Refus : facture absente → `NotFound` ; pas un brouillon →
/// `IllegalStateTransition` (lu **sous le verrou** : le perdant de deux
/// validations concurrentes relit `open` et rend ce code, non
/// `OptimisticLockConflict` — test piloté, M20) ; version périmée →
/// `OptimisticLockConflict` ; puis les refus de l'ancienne création, contrôles
/// de saisie refaits sous verrou compris (projet archivé depuis
/// l'enregistrement, compte devenu inactif ou non imputable) : exercice,
/// réglages, comptes désignés. Un refus n'écrit rien.
///
/// Aucun `begin`/`commit` interne. Réutilisé par [`create_validated_in_tx`].
///
/// # Ordre des verrous
///
/// **La facture `FOR UPDATE` d'abord**, puis l'ordre canonique côté achat
/// (Story 15-5e1, AC5), numéroté comme les étapes du code :
///
/// ```text
/// (0)    projects / companies   projet analytique : sentinelle companies puis
///                               projet FOR UPDATE (validate_taggable_in_tx),
///                               seulement si un projet est renseigné
/// (1)    contacts               fournisseur, lu SANS verrou
/// (2)    —                      passe de FORME des lignes : aucun verrou
///                               (purchase_lines_form : positivité, centime,
///                               TVA — v017-3b ; l'étape (0) l'a précédée)
/// (2 bis) company_invoice_settings  INSERT IGNORE puis FOR UPDATE
///                               (get_or_create_default_in_tx) — avancé AVANT les
///                               comptes de charge et l'exercice (choix C55)
/// (2, suite) accounts           passe des COMPTES de l'étape (2), après (2 bis) :
///                               chaque compte de charge FOR UPDATE, dans l'ordre
///                               des lignes (sans tri)
/// (2, désignés) accounts        comptes désignés (créanciers, TVA récupérable),
///                               LOCK IN SHARE MODE (partagé), ORDER BY id
///                               — Story 15-5d, lock_designated_accounts_in_tx ;
///                               tous les candidats, même la TVA récupérable
///                               d'une facture sans TVA
/// (3)    fiscal_years           find_open_covering_date
/// (4)    —                      exigence du compte créanciers
///                               (ConfigurationRequired) sur les réglages lus en
///                               (2 bis) : aucun verrou neuf
/// (3 bis) fiscal_years          numéro interne (Story 33-1a) : les AUTRES exercices
///                               de l'année de début, un par un FOR UPDATE, triés par
///                               date — aucun verrou neuf s'il n'y en a pas, le cas
///                               courant ; puis `supplier_invoices` : contrôle de
///                               collision FOR UPDATE, verrou d'intervalle sur l'UNIQUE
///                               quand la clé est absente
///                               (attribute_internal_number_in_tx)
/// (5)    journal_entries        create_in_tx — compteur des écritures FOR UPDATE ;
///                               l'insertion reprend des verrous PARTAGÉS sur
///                               companies et sur chaque compte écrit
///                               (fk_jel_account), donc APRÈS l'exercice
/// ```
///
/// L'exigence du compte créanciers (`ConfigurationRequired`) reste jugée
/// **après** l'exercice : seul l'**appel** des réglages est avancé, et il ne
/// refuse rien (hors erreur de base) — aucun refus ne change de place.
///
/// Cet ordre est une **convention qui réduit la fréquence** des interblocages,
/// pas une garantie : la ligne des réglages ordonne la plupart des saisies d'une
/// même société sans les sérialiser à coup sûr (un `INSERT IGNORE` en doublon y
/// pose un verrou partagé, mesuré sur MariaDB 10.11 — deux saisies qui le
/// tiennent puis demandent l'exclusif s'interbloquent), et l'insertion des
/// lignes reprend les comptes après l'exercice. Les comptes désignés de
/// `(2, désignés)` (passif, actif) et les comptes de charge (`Expense`) sont
/// disjoints **tant que les types désignés sont respectés** — ce qui n'est pas
/// contrôlé après la désignation (angle mort assumé, revue de code P1, B-3) :
/// un compte désigné retypé en charge et imputé par une ligne prend un verrou
/// exclusif en `(2, suite)` puis partagé en `(2, désignés)`, et deux saisies
/// croisées sur deux tels comptes peuvent s'interbloquer ; ce cycle étroit
/// relève du **rejeu des routes**, comme les autres ci-dessous. Le verrou des
/// comptes désignés est **partagé** (Story 15-5d) : il suffit
/// contre l'archivage et le passage à non imputable, qui écrivent la ligne du
/// compte, et reste compatible avec le verrou partagé que `fk_jel_account`
/// pose sur les créanciers dans un règlement fournisseur qui tient déjà
/// l'exercice ; il précède l'exercice, comme le compte interne d'un règlement
/// fournisseur. Les comptes de charge ne sont
/// pas triés : les cycles qui restent (deux saisies aux comptes croisés, une
/// charge imputée au compte d'arrondi contre un règlement client par compte
/// interne avec écart) sont couverts par le **rejeu des routes** — la
/// validation (`POST /supplier-invoices/{id}/validate`, Story v017-4a ; la
/// saisie jusqu'à la 0.17.0-rc1, Story 15-5e1). Cycle **connu** avec la
/// confirmation d'un lot et l'acceptation d'un lot de rapprochement (sentinelle
/// `companies` partagée, puis facture suivante) : rejoué de même. La règle
/// générale est écrite au doc-comment de [`super::invoices::validate_invoice`].
pub async fn validate_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{company_invoice_settings, fiscal_years, journal_entries};

    // (−1) La facture, sous verrou ; son statut, puis sa version.
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    if inv.status != "draft" {
        return Err(DbError::IllegalStateTransition(format!(
            "seul un brouillon se valide (statut actuel : '{}')",
            inv.status
        )));
    }
    if inv.version != version {
        return Err(DbError::OptimisticLockConflict);
    }
    let contact_id = inv.contact_id;
    let supplier_invoice_number = inv.supplier_invoice_number.clone();
    let invoice_date = inv.invoice_date;
    let project_id = inv.project_id;
    let lines: Vec<crate::entities::NewSupplierInvoiceLine> = fetch_lines(tx, id)
        .await?
        .into_iter()
        .map(|l| crate::entities::NewSupplierInvoiceLine {
            description: l.description,
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
            expense_account_id: l.expense_account_id,
        })
        .collect();

    if lines.is_empty() {
        return Err(DbError::IllegalStateTransition(
            "une facture fournisseur doit avoir au moins une ligne".into(),
        ));
    }

    // (0) Projet analytique optionnel (Story 19-3) : s'il est renseigné, il doit
    // exister, appartenir à la même company et ne pas être archivé (sinon la dépense
    // serait taguée sur un projet clos). Le helper prend le sentinel `companies`
    // AVANT le `FOR UPDATE` projets (Pattern 5, anti-deadlock ABBA avec
    // l'archivage d'un projet — code-review Pass 3 Opus 19-3) et ferme la race
    // d'archivage concurrent. Extrait en helper
    // partagé en Story 19-2 (réutilisé par le tag par-ligne des écritures manuelles).
    if let Some(pid) = project_id {
        super::projects::validate_taggable_in_tx(tx, company_id, &[pid]).await?;
    }

    // (1) Fournisseur valide (existe, company-scoped, is_supplier).
    let supplier: Option<(String, bool)> =
        sqlx::query_as("SELECT name, is_supplier FROM contacts WHERE id = ? AND company_id = ?")
            .bind(contact_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
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
    // Story 15-5a (choix C22) — DEUX passes : la FORME de toutes les lignes
    // d'abord (refus immédiat, dans l'ordre des lignes), puis les COMPTES. Les
    // comptes non imputables sont collectés pour être tous nommés par un seul
    // refus ; la forme d'une ligne ultérieure doit donc avoir été jugée avant.
    //
    // Story v017-3b (#687, D9) — la passe de forme est la fonction pure
    // `purchase_lines_form` (positivité → centime → TVA), que la complétion
    // d'une pièce importée appelle aussi. Elle ne prend aucun verrou ; ⚠️ l'étape
    // (0) du projet analytique, plus haut, en a déjà pris un (sentinelle
    // `companies`, projet `FOR UPDATE`) : un refus de forme n'en prend pas de
    // NOUVEAU, il ne les précède pas tous.
    let line_totals = purchase_lines_form(&lines)?;
    // (2 bis) Réglages de la société, AVANT les comptes de charge et l'exercice
    // (Story 15-5e1, AC5, choix C55) : l'ordre « réglages → exercice » devient
    // commun à la saisie et à la validation d'une facture client. Un refus de
    // forme (passe ci-dessus) ne prend ainsi aucun verrou de plus que ceux de
    // l'étape (0), quand un projet est renseigné (v017-3b). L'exigence du compte
    // créanciers reste jugée plus bas, après l'exercice.
    let settings =
        company_invoice_settings::get_or_create_default_in_tx(&mut *tx, company_id).await?;

    // (2, suite) Passe des COMPTES de l'étape (2), après les réglages.
    let mut non_postable: Vec<NonPostableAccount> = Vec::new();
    // Le `line_total` de chaque ligne vient de la passe de forme : aucun second
    // produit (v017-3b, AC-3).
    for (line, &line_total) in lines.iter().zip(&line_totals) {
        // Compte de charge : doit exister, être actif, company-scoped, de type
        // Expense (AC6 — sinon l'écriture débiterait un Passif/Actif) **et
        // imputable**.
        //
        // ⛔ `postable` ajouté après la passe 4 de revue de code de la Story 24-5
        // (#375). Le type ne suffit PAS : les comptes de clôture 9000/9100/9200
        // sont eux-mêmes typés `Expense` — c'est le fait fondateur de cette
        // story — donc `t == "Expense"` les laissait passer. Ce flux écrit avec
        // `enforce_postable = false`, ce SELECT est donc le seul contrôle.
        //
        // L'écran filtre déjà `active && postable`
        // (`supplier-invoices/+page.svelte:129`), si bien que le trou n'était
        // atteignable que par appel direct à l'API — comme pour le règlement
        // fournisseur, fermé au même titre : *une garde serveur ne se déduit pas
        // d'un filtre d'écran.*
        let acct: Option<(bool, bool, String, String)> = sqlx::query_as(
            "SELECT active, postable, account_type, number FROM accounts \
             WHERE id = ? AND company_id = ? FOR UPDATE",
        )
        .bind(line.expense_account_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
        // Story 15-5a — ordre des refus : (a) inconnu, autre société, archivé
        // ou d'un autre type que `Expense` → `InactiveOrInvalidAccounts`,
        // immédiatement et même si le compte est AUSSI non imputable ; (b)
        // sinon, non imputable → collecté, nommé après la passe si aucune
        // ligne n'est en (a).
        match acct {
            Some((true, true, ref t, _)) if t == "Expense" => {}
            Some((true, false, ref t, number)) if t == "Expense" => {
                non_postable.push(NonPostableAccount {
                    account_id: line.expense_account_id,
                    account_number: number,
                });
            }
            _ => return Err(DbError::InactiveOrInvalidAccounts),
        }
        pairs.push((line_total, line.vat_rate, line.expense_account_id));
    }
    if !non_postable.is_empty() {
        return Err(DbError::accounts_not_postable(non_postable));
    }

    // (2, désignés) Story 15-5d (#429) — les comptes DÉSIGNÉS que cette saisie
    // peut écrire (créanciers, TVA récupérable), lus dans les réglages de (2 bis),
    // verrouillés EN PARTAGÉ (`LOCK IN SHARE MODE`, `ORDER BY id`), après les
    // comptes de charge et AVANT l'exercice (3). Rien n'est refusé ici : le
    // contrôle des seuls rôles écrits vient après la génération (5). Un compte
    // créanciers `None` n'est simplement pas candidat ; son exigence (4) ne
    // bouge pas.
    let designated = company_invoice_settings::lock_designated_accounts_in_tx(
        &mut *tx,
        company_id,
        &DesignatedRole::candidate_ids(&DesignatedRole::PURCHASE, &settings),
    )
    .await?;

    // (3) Exercice ouvert couvrant la date de facture.
    let fy = fiscal_years::find_open_covering_date(&mut *tx, company_id, invoice_date)
        .await?
        .ok_or(DbError::FiscalYearInvalid)?;

    // (4) Config company (chargée en (2 bis)) : comptes créanciers 2000 (requis)
    // + 1171 récupérable (optionnel).
    let payable_account_id = settings
        .default_payable_account_id
        .ok_or_else(|| DbError::ConfigurationRequired("default_payable_account_id".into()))?;

    // (5) Écriture d'achat.
    let (generated, total_ttc) = generate_purchase_journal_lines(
        &pairs,
        payable_account_id,
        settings.default_vat_recoverable_account_id,
    )?;
    // Story 15-5d (#429) — la garde à l'usage : les seuls comptes de réglage
    // écrits (la TVA récupérable seulement si la TVA totale est positive), sur
    // l'instantané verrouillé en (2, désignés). Absent ou archivé →
    // `InactiveOrInvalidAccounts` ; non imputable →
    // `DesignatedAccountsNotPostable`.
    designated.check_written(&generated.roles, &settings)?;
    let entry_lines = generated.lines;
    // (3 bis) Le numéro interne (Story 33-1a, #926) : après le dernier refus de forme,
    // avant le libellé. Une facture qui en a déjà un le reprend sans lire ni écrire le
    // compteur (brouillon revalidé, C-e331-9).
    let internal_number = match inv.internal_number.clone() {
        Some(numero) => numero,
        None => {
            attribute_internal_number_in_tx(
                tx,
                company_id,
                &fy,
                &settings.supplier_internal_number_format,
            )
            .await?
        }
    };
    let description = format!(
        "Facture fournisseur {} - {contact_name}",
        document_long_label(
            id,
            Some(&internal_number),
            supplier_invoice_number.as_deref()
        )
    );
    let je = journal_entries::create_in_tx(
        &mut *tx,
        fy.id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: invoice_date,
            journal: Journal::Achats,
            description,
            // Propagation document-level : le projet de la facture est stampé sur
            // toutes les lignes de l'écriture d'achat (Story 19-3).
            project_id,
            lines: entry_lines,
        },
        // Flux automatique (facture fournisseur) : garde de postabilité
        // désactivée (Story 14-3b, D-A0).
        false,
    )
    .await?;

    // (6) La facture devient `open` (« validée ») ; son total est celui de la
    //     fonction pure (identique au brouillon, par construction) ; un seul
    //     `UPDATE`, sous la contrainte « écriture si et seulement si non
    //     brouillon ».
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET status = 'open', purchase_journal_entry_id = ?, \
         total_amount = ?, internal_number = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'draft'",
    )
    .bind(je.entry.id)
    .bind(total_ttc)
    .bind(&internal_number)
    .bind(id)
    .bind(company_id)
    .bind(version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows != 1 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (7) Relire + audit.
    let validated = reread(tx, company_id, id).await?;
    audit_log::insert_in_tx(
        &mut *tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.validated".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(serde_json::json!({
                "supplierInvoice": snapshot_json(&validated.invoice, &validated.lines),
                "journalEntryId": je.entry.id,
            })),
        ),
    )
    .await?;
    Ok(validated)
}

/// Garde [Story 12.3] : refuse une mutation directe (`pay`/`cancel`) si la
/// facture est engagée dans un lot de paiement `generated` (pain.001 en cours).
///
/// Le `SELECT … FOR UPDATE` sur la **ligne `supplier_invoices`** sérialise vis-à-vis
/// de `payment_batches::create_batch` (qui verrouille la même ligne). NE PAS appeler
/// depuis `confirm_batch` (qui règle des factures en lot `generated`).
async fn guard_not_in_generated_batch(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
) -> Result<(), DbError> {
    let exists: Option<i64> = sqlx::query_scalar(
        "SELECT id FROM supplier_invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if exists.is_none() {
        return Err(DbError::NotFound);
    }
    if in_generated_batch(tx, id).await?.is_some() {
        return Err(DbError::IllegalStateTransition(
            "facture engagée dans un lot de paiement en cours (annuler le lot d'abord)".into(),
        ));
    }
    Ok(())
}

/// Le lot de paiement **`generated`** qui engage la facture, s'il y en a un —
/// écrit une seule fois pour [`guard_not_in_generated_batch`] (`pay`) et pour
/// le dernier rang de [`supplier_invoice_cancel_blocker`] (Story 25-3-c).
///
/// Lecture **non verrouillante** : un geste qui s'en sert pour refuser doit
/// avoir verrouillé la facture avant — `payment_batches::create_batch`
/// verrouille la même ligne, ce qui sérialise les deux.
async fn in_generated_batch(
    conn: &mut sqlx::MySqlConnection,
    supplier_invoice_id: i64,
) -> Result<Option<i64>, DbError> {
    sqlx::query_scalar(
        "SELECT pbi.id FROM payment_batch_items pbi \
         JOIN payment_batches pb ON pb.id = pbi.payment_batch_id \
         WHERE pbi.supplier_invoice_id = ? AND pb.status = 'generated' LIMIT 1",
    )
    .bind(supplier_invoice_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)
}

/// La ligne de **crédit** de l'écriture d'achat — `(compte créanciers, TTC)` —
/// ou `None` si l'écriture n'en a pas (Story 15-6b, #474 ; choix C-15-6-11).
///
/// Lecteur sœur de `invoice_settlements::sale_receivable_account` : la dette se
/// lit **sur l'écriture d'achat**, jamais sur les réglages, pour que le compte
/// se solde exactement quoi qu'il soit arrivé à la configuration. Il diffère du
/// lecteur de la vente (ligne de crédit, et le montant). Deux appelants :
/// `pay_in_tx` (dont `None` est un `Invariant`) et la création d'un lot de
/// paiement (dont `None` est un refus de **cette** facture,
/// `SUPPLIER_INVOICE_PURCHASE_ENTRY_MALFORMED`). Portée par `je.company_id`.
/// Aucun verrou : les lignes d'une écriture d'achat sont gelées.
pub async fn purchase_payable_line(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    purchase_entry_id: i64,
) -> Result<Option<(i64, Decimal)>, DbError> {
    sqlx::query_as(
        "SELECT jel.account_id, jel.credit FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         WHERE jel.entry_id = ? AND je.company_id = ? AND jel.credit > 0 \
         ORDER BY jel.id LIMIT 1",
    )
    .bind(purchase_entry_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)
}

/// Règle une facture fournisseur `open` (choix binaire). Poste l'écriture de
/// règlement et passe en `paid`. Refuse si la facture est dans un lot `generated`.
pub async fn pay(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    choice: SettlementChoice,
    payment_date: chrono::NaiveDate,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let result = async {
        // Guard lot-membership dans le wrapper (JAMAIS dans pay_in_tx — H-FRESH-1).
        guard_not_in_generated_batch(&mut tx, company_id, id).await?;
        pay_in_tx(&mut tx, company_id, id, choice, payment_date, user_id).await
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

/// Cœur du règlement d'une facture, **tx-aware et GUARD-FREE** pour la
/// lot-membership [H-FRESH-1]. Réutilisé par `pay()` (wrapper) ET par
/// `payment_batches::confirm_batch` (qui appelle N fois dans UNE transaction
/// atomique, pendant que le lot est encore `generated` — d'où l'absence de
/// guard lot ici, sinon la confirmation se self-bloquerait).
///
/// Enveloppe d'une ligne de [`pay_in_tx_as`] (Story 30-1b, AC-b9) : l'acteur
/// est l'utilisateur, sans clé d'API — `pay` et `confirm_batch` ne changent pas.
pub async fn pay_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    choice: SettlementChoice,
    payment_date: chrono::NaiveDate,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    pay_in_tx_as(
        tx,
        company_id,
        id,
        choice,
        payment_date,
        letterings::Actor {
            user_id,
            api_key_id: None,
        },
    )
    .await
}

/// Le corps du règlement d'une facture fournisseur, **avec son acteur**
/// (Story 30-1b, #686, AC-b9) : l'audit `supplier_invoice.paid` passe par
/// [`NewAuditLogEntry::for_actor`] et le lettrage reçoit la clé d'API. Un seul
/// corps pour la fiche, la confirmation d'un lot ([`pay_in_tx`]) et
/// l'acceptation de la réconciliation (`accept_one_supplier_invoice`, qui
/// passe la clé de la requête) : aucun jumeau.
///
/// **`paid_at` est la date du paiement** (`payment_date` à minuit, Story
/// 30-1b, AC-b16, C-30-1-25) — la date même de l'écriture de règlement —, et
/// non plus l'heure de l'enregistrement (`CURRENT_TIMESTAMP(3)` jusqu'à la
/// v0.13.4). Même forme que le règlement client. Les factures déjà payées
/// gardent leur `paid_at` : rien n'est réécrit.
pub async fn pay_in_tx_as(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    choice: SettlementChoice,
    payment_date: chrono::NaiveDate,
    actor: letterings::Actor,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::entities::{Journal, NewJournalEntry};
    use crate::repositories::{fiscal_years, journal_entries};
    let user_id = actor.user_id;

    // (1) Verrou facture + garde statut.
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    if inv.status != "open" {
        return Err(DbError::IllegalStateTransition(format!(
            "seule une facture validée peut être réglée (statut actuel : '{}')",
            inv.status
        )));
    }
    let purchase_entry_id = inv.purchase_journal_entry_id.ok_or_else(|| {
        DbError::Invariant("facture fournisseur validée sans écriture d'achat".into())
    })?;

    // (2) Compte débité (créanciers) ET montant = ligne de crédit de l'écriture
    //     d'achat → solde 2000 garanti à 0 quelle que soit l'évolution des settings.
    //     Lecteur partagé avec la création d'un lot (Story 15-6b).
    let (payable_account_id, ttc) = purchase_payable_line(tx, company_id, purchase_entry_id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant("écriture d'achat sans ligne de crédit créanciers".into())
        })?;

    // (3) Contrepartie selon le choix binaire (company-scoped).
    let (counterparty_account_id, settlement_type, bank_account_id, internal_account_id, journal) =
        match choice {
            SettlementChoice::BankTransfer { bank_account_id } => {
                // Story 28-4a (#642, W27, AC6-bis) — `closed_on` et `archived`
                // lus DANS l'instruction verrouillée du sol (élargie de
                // colonnes, aucun verrou ajouté) : dernière version commitée,
                // sans fenêtre. Ordre : absent, autre société ou archivé →
                // `NotFound` ; clos → `BankAccountClosed` ; sans compte lié →
                // `ConfigurationRequired`.
                let row: Option<(Option<i64>, Option<chrono::NaiveDate>, bool)> = sqlx::query_as(
                    "SELECT journal_account_id, closed_on, archived FROM bank_accounts \
                     WHERE id = ? AND company_id = ? FOR UPDATE",
                )
                .bind(bank_account_id)
                .bind(company_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_db_error)?;
                let (journal_account_id, closed_on, archived) = row.ok_or(DbError::NotFound)?;
                if archived {
                    return Err(DbError::NotFound);
                }
                if closed_on.is_some() {
                    return Err(DbError::BankAccountClosed { bank_account_id });
                }
                let counterparty = journal_account_id.ok_or_else(|| {
                    DbError::ConfigurationRequired("bank_account.journal_account_id".into())
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
                // ⚠️ ACTIF **et IMPUTABLE** — jumeau exact de la garde de
                // `invoice_settlements_write.rs`. Ce flux écrit sans la garde de
                // postabilité de la 14-3b, donc ce SELECT est le seul contrôle.
                //
                // Ici l'écran filtre déjà `active && postable`
                // (`supplier-invoices/[id]/+page.svelte`, chargement des comptes
                // internes), si bien que le
                // trou n'était atteignable que par appel direct à l'API — à la
                // différence de son jumeau côté client, où l'écran offrait le
                // compte. Le fermer quand même : une garde serveur ne se déduit
                // pas d'un filtre d'écran, et le jumeau nous rappelle pourquoi.
                //
                // ⛔ Trouvé par le grep de propagation du finding P3-1 (passe 3
                // de revue de code de la Story 24-5, #375) — la lentille avait
                // nommé ce fichier pour SA validation de compte de charge, pas
                // pour ce site-ci.
                //
                // Jumeau aussi pour la garde qui suit l'étape (3) : la
                // contrepartie n'est pas le compte créanciers (Story 15-6b).
                let account: Option<(bool, bool, String)> = sqlx::query_as(
                    "SELECT active, postable, number FROM accounts WHERE id = ? AND company_id = ? \
                     FOR UPDATE",
                )
                .bind(account_id)
                .bind(company_id)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_db_error)?;
                // Story 15-5a — ordre des refus : (a) inconnu, autre société ou
                // archivé → `InactiveOrInvalidAccounts` ; (b) sinon, non
                // imputable → `AccountsNotPostable`, qui nomme le compte.
                match account {
                    Some((true, true, _)) => {}
                    Some((true, false, number)) => {
                        return Err(DbError::accounts_not_postable([NonPostableAccount {
                            account_id,
                            account_number: number,
                        }]));
                    }
                    _ => return Err(DbError::InactiveOrInvalidAccounts),
                }
                (
                    account_id,
                    "internal_account",
                    None,
                    Some(account_id),
                    Journal::OD,
                )
            }
        };

    // (3 bis) ⛔ Story 15-6b (#474) — la contrepartie n'est pas la dette :
    //         `D 2000 / C 2000` s'équilibre, et la facture passerait « payée »
    //         sans paiement. Comparée aussitôt résolue, après ses propres refus.
    //         À la confirmation d'un lot, `confirm_batch` intercepte ce refus pour
    //         le contextualiser.
    invoice_settlements::refuse_if_claim_account(
        tx,
        company_id,
        counterparty_account_id,
        payable_account_id,
        ClaimSubject::Counterparty(ClaimSide::Payable),
    )
    .await?;

    // (4) Exercice ouvert couvrant la date de règlement.
    let fy = fiscal_years::find_open_covering_date(tx, company_id, payment_date)
        .await?
        .ok_or(DbError::FiscalYearInvalid)?;

    // (5) Écriture de règlement : D 2000 / C contrepartie (TTC).
    let number_label = document_long_label(
        inv.id,
        inv.internal_number.as_deref(),
        inv.supplier_invoice_number.as_deref(),
    );
    let contact_name: String =
        sqlx::query_scalar("SELECT name FROM contacts WHERE id = ? AND company_id = ?")
            .bind(inv.contact_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?
            .unwrap_or_default();
    let entry_lines = vec![
        NewJournalEntryLine {
            account_id: payable_account_id,
            debit: ttc,
            credit: Decimal::ZERO,
            project_id: None,
        },
        NewJournalEntryLine {
            account_id: counterparty_account_id,
            debit: Decimal::ZERO,
            credit: ttc,
            project_id: None,
        },
    ];
    let je = journal_entries::create_in_tx(
        tx,
        fy.id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: payment_date,
            journal,
            description: format!("Règlement fournisseur {number_label} - {contact_name}"),
            // Le règlement hérite du projet de la facture (cohérence analytique).
            project_id: inv.project_id,
            lines: entry_lines,
        },
        // Flux automatique (règlement fournisseur) : garde de postabilité
        // désactivée (Story 14-3b, D-A0).
        false,
    )
    .await?;

    // (6) UPDATE facture → paid. `paid_at` = la date du paiement à minuit
    //     (Story 30-1b, AC-b16), liée en paramètre.
    let paid_at = payment_date
        .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("minuit est une heure valide"));
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET status = 'paid', settlement_type = ?, \
         settlement_bank_account_id = ?, settlement_account_id = ?, \
         settlement_journal_entry_id = ?, paid_at = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'open'",
    )
    .bind(settlement_type)
    .bind(bank_account_id)
    .bind(internal_account_id)
    .bind(je.entry.id)
    .bind(paid_at)
    .bind(id)
    .bind(company_id)
    .bind(inv.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (6 bis) Le lettrage de la pièce (Story 15-1a2-ii, P4) : une facture payée
    //     est lettrée `document` — l'achat et le paiement, sur la dette `B`.
    //     L'exercice tenu est celui du règlement, verrouillé en (4). ⛔ Un
    //     paiement n'est jamais refusé à cause du lettrage (compte non lettrable,
    //     périodes closes : abstention) ; une erreur structurelle se propage et
    //     annule le paiement — dans `confirm_batch`, le lot entier.
    letterings::sync_supplier_invoice_in_tx(tx, company_id, id, fy.id, actor).await?;

    // (7) Relire + audit — l'acteur, clé d'API comprise (Story 30-1b, AC-b9).
    let updated = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let inv_lines = fetch_lines(tx, id).await?;
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::for_actor(
            user_id,
            actor.api_key_id,
            "supplier_invoice.paid",
            "supplier_invoice",
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

// ---------------------------------------------------------------------------
// Story 30-1b (#686) — régler une facture fournisseur par la réconciliation
// ---------------------------------------------------------------------------

/// La facture fournisseur **verrouillée** et admise au règlement par une
/// transaction bancaire : rendue par [`lock_for_bank_settlement_in_tx`].
#[derive(Debug, Clone)]
pub struct LockedForSettlement {
    /// La facture, lue sous `FOR UPDATE` (statut `open`).
    pub invoice: SupplierInvoice,
}

/// Pourquoi une facture fournisseur ne peut pas être réglée par une
/// transaction bancaire du compte (Story 30-1b, #686, AC-b6, étape 3). Chaque
/// variante devient un refus **par proposition** dans `failed[]`
/// (`kesh-api`, `supplier_payment_error_to_failed_proposal`), jamais une
/// erreur globale (§ *Pattern batch* du `CLAUDE.md`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SupplierSettlementRefusal {
    /// Facture absente **ou d'une autre société** — indiscernables (KF-002).
    NotFound,
    /// Facture non ouverte (`paid`, `cancelled`).
    NotOpen { status: String },
    /// Compte bancaire absent, d'une autre société ou archivé.
    BankAccountNotFound,
    /// Compte bancaire clos (`closed_on` posé).
    BankAccountClosed,
    /// Compte bancaire sans compte au grand livre.
    BankAccountNotConfigured,
    /// La facture est engagée dans un lot de paiement `generated` (tout
    /// compte) : le fichier pain.001 est peut-être déjà à la banque.
    InGeneratedBatch { payment_batch_id: i64 },
    /// Une **autre** facture ouverte, au montant exact, est engagée dans un lot
    /// `generated` de ce compte — la garde (d), la même que la proposition.
    HeldByPaymentBatch {
        supplier_invoice_id: i64,
        payment_batch_id: i64,
    },
}

/// Le texte qui distingue, dans `PROCESSLIST`, la lecture verrouillante du
/// compte de [`lock_for_bank_settlement_in_tx`] — la sonde des tests de course
/// (`test_fixtures::attendre_une_requete_en_cours`). Macro, pour que la sonde
/// et la requête n'aient qu'une écriture ; le `FOR UPDATE` y figure, pour que
/// l'inventaire des gardes du compte clos
/// (`kesh-api/tests/bank_account_closed_inventory.rs`) ne lise pas ce
/// fragment comme une lecture non verrouillante de `closed_on`.
macro_rules! bank_settlement_account_lock_marker {
    () => {
        "closed_on, archived AS bank_settlement_lock FROM bank_accounts \
         WHERE id = ? AND company_id = ? FOR UPDATE"
    };
}

/// Cf. [`bank_settlement_account_lock_marker!`] — publique pour les tests.
pub const BANK_SETTLEMENT_ACCOUNT_LOCK_MARKER: &str = bank_settlement_account_lock_marker!();

/// L'étape (b) de [`lock_for_bank_settlement_in_tx`] : le compte de l'import,
/// en exclusif.
const BANK_SETTLEMENT_ACCOUNT_LOCK_SQL: &str = concat!(
    "SELECT journal_account_id, ",
    bank_settlement_account_lock_marker!()
);

/// **Verrous et éligibilité** du règlement d'une facture fournisseur par une
/// transaction bancaire du compte `bank_account_id` (Story 30-1b, #686, AC-b6
/// étape 3, AC-b8, AC-b10), dans la transaction de l'acceptation, **avant**
/// [`pay_in_tx_as`]. `amount` est la valeur absolue de la transaction
/// rechargée ; seule la garde (d) la lit.
///
/// Dans **cet ordre de verrous** — celui de `pay_in_tx` et de `create_batch` :
/// facture, puis compte, puis articles de lot :
///
/// - **(a)** la facture `FOR UPDATE` → absente ou d'une autre société :
///   [`SupplierSettlementRefusal::NotFound`] ; non `open` : `NotOpen` ;
/// - **(b)** le compte bancaire de l'import `FOR UPDATE` → absent ou archivé :
///   `BankAccountNotFound` ; clos : `BankAccountClosed` ; sans compte lié :
///   `BankAccountNotConfigured`. Le compte est pris **avant** la lecture des
///   articles (arbitrage de la P1) : une acceptation et un `create_batch` du
///   même compte se sérialisent au compte, sans interblocage (C3) ;
/// - **(c)** l'appartenance de la facture à un lot `generated`, en **lecture
///   verrouillante** (`LOCK IN SHARE MODE`) → `InGeneratedBatch`. ⛔ Une
///   lecture ordinaire lirait la vue de la transaction, **déjà née** (lecture
///   de la transaction bancaire, ou proposition précédente du lot) : un
///   `create_batch` validé entre-temps serait invisible — il verrouille la
///   facture sans la modifier —, et la facture serait réglée ici **puis payée
///   une seconde fois** par la banque sur le fichier pain.001. Valable en
///   MariaDB 10.11 (`innodb_snapshot_isolation = OFF`, #669). ⚠️ Sans article,
///   la lecture pose un **intervalle** partagé sur
///   `idx_payment_batch_items_invoice`, tenu jusqu'au commit du lot
///   d'acceptation : un `create_batch` d'**autres** factures dont un article y
///   tombe attend ; le seul cycle qui passe par là est C7 (entre comptes),
///   rejoué des deux côtés. [`in_generated_batch`], lu par les vues, reste non
///   verrouillant ;
/// - **(d)** la **même garde que la proposition** :
///   [`reconciliation::find_open_supplier_invoice_in_account_batch`], en
///   **lecture ordinaire** — une autre facture ouverte au montant exact,
///   engagée dans un lot `generated` de ce compte → `HeldByPaymentBatch`. Sans
///   elle, un appel d'API direct, ou un écran chargé avant la création du lot,
///   réglerait février avec le débit de janvier. ⚠️ Tolérance écrite : un lot
///   commis après la naissance de la vue n'est pas vu ; (d) n'ajoute aucun
///   verrou.
///
/// Les verrous (a) et (b) sont redemandés par [`pay_in_tx_as`], déjà tenus.
/// Une erreur de base est rendue en `Err` ; un refus, en `Ok(Err(_))`.
///
/// [`reconciliation::find_open_supplier_invoice_in_account_batch`]:
///     super::reconciliation::find_open_supplier_invoice_in_account_batch
pub async fn lock_for_bank_settlement_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    supplier_invoice_id: i64,
    bank_account_id: i64,
    amount: Decimal,
) -> Result<Result<LockedForSettlement, SupplierSettlementRefusal>, DbError> {
    // (a) La facture.
    let invoice = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(supplier_invoice_id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let Some(invoice) = invoice else {
        return Ok(Err(SupplierSettlementRefusal::NotFound));
    };
    if invoice.status != "open" {
        return Ok(Err(SupplierSettlementRefusal::NotOpen {
            status: invoice.status,
        }));
    }

    // (b) Le compte bancaire de l'import.
    let account: Option<(Option<i64>, Option<chrono::NaiveDate>, bool)> =
        sqlx::query_as(BANK_SETTLEMENT_ACCOUNT_LOCK_SQL)
            .bind(bank_account_id)
            .bind(company_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_db_error)?;
    match account {
        None | Some((_, _, true)) => {
            return Ok(Err(SupplierSettlementRefusal::BankAccountNotFound));
        }
        Some((_, Some(_), _)) => return Ok(Err(SupplierSettlementRefusal::BankAccountClosed)),
        Some((None, None, false)) => {
            return Ok(Err(SupplierSettlementRefusal::BankAccountNotConfigured));
        }
        Some((Some(_), None, false)) => {}
    }

    // (c) L'appartenance à un lot `generated`, en lecture verrouillante.
    let in_batch: Option<i64> = sqlx::query_scalar(
        "SELECT pb.id FROM payment_batch_items pbi \
         JOIN payment_batches pb ON pb.id = pbi.payment_batch_id \
         WHERE pbi.supplier_invoice_id = ? AND pb.status = 'generated' LIMIT 1 \
         LOCK IN SHARE MODE",
    )
    .bind(supplier_invoice_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;
    if let Some(payment_batch_id) = in_batch {
        return Ok(Err(SupplierSettlementRefusal::InGeneratedBatch {
            payment_batch_id,
        }));
    }

    // (d) La garde de la proposition : une autre facture du lot du compte.
    if let Some(held) = super::reconciliation::find_open_supplier_invoice_in_account_batch(
        &mut **tx,
        company_id,
        bank_account_id,
        amount,
    )
    .await?
    {
        return Ok(Err(SupplierSettlementRefusal::HeldByPaymentBatch {
            supplier_invoice_id: held.supplier_invoice_id,
            payment_batch_id: held.payment_batch_id,
        }));
    }

    Ok(Ok(LockedForSettlement { invoice }))
}

// ---------------------------------------------------------------------------
// Story 25-3-c (#454) — annuler une facture fournisseur, même payée
// ---------------------------------------------------------------------------

/// Ce qui empêche d'annuler la facture fournisseur `id` — le premier motif,
/// ou `None` (Story 25-3-c, #454).
///
/// ⚠️ **La précédence est composée ICI**, non par l'ordre de déclaration de
/// [`SettlementCancelBlocker`] : rang 1, la facture est un brouillon (Story
/// v017-4a), puis déjà `cancelled` (coupe court) ; rangs 2 à 5 (dont le 2 bis, le lettrage de pièce figé par la
/// période — Story 15-1a2-0), la **queue commune** évaluée sur l'écriture d'**achat**
/// ([`super::settlement_cancellation::settlement_entry_cancel_blocker`], aucun
/// jumeau) ; rang 6, **en dernier**, un lot de paiement `generated` — le lever
/// est le geste le plus lourd, et il serait vain avant un exercice clos.
///
/// Une seule fonction pour lire (l'écran masque le bouton avant le clic) et
/// pour refuser (le geste). **Lecture seule**, sans verrou : cf.
/// [`cancel_in_tx`] pour l'ordre des verrous.
///
/// Facture introuvable (ou d'une autre société) → [`DbError::NotFound`].
///
/// [`SettlementCancelBlocker`]: crate::errors::SettlementCancelBlocker
pub async fn supplier_invoice_cancel_blocker(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    id: i64,
) -> Result<Option<super::settlement_cancellation::SettlementCancelHit>, DbError> {
    use crate::errors::SettlementCancelBlocker;

    let row: Option<(String, Option<i64>)> = sqlx::query_as(
        "SELECT status, purchase_journal_entry_id FROM supplier_invoices \
         WHERE id = ? AND company_id = ?",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let (status, purchase_entry) = row.ok_or(DbError::NotFound)?;
    // Rang 1 (Story v017-4a, AC-13) : un brouillon n'a rien passé au journal,
    // il ne s'annule pas — il se corrige ou se supprime. Coupe court.
    if status == "draft" {
        return Ok(Some((
            SettlementCancelBlocker::SupplierInvoiceDraft,
            None,
            None,
        )));
    }
    if status == "cancelled" {
        return Ok(Some((
            SettlementCancelBlocker::SupplierInvoiceCancelled,
            None,
            None,
        )));
    }
    let purchase_entry = purchase_entry.ok_or_else(|| {
        DbError::Invariant("facture fournisseur validée sans écriture d'achat".into())
    })?;
    if let Some(hit) = super::settlement_cancellation::settlement_entry_cancel_blocker(
        conn,
        company_id,
        purchase_entry,
        None,
    )
    .await?
    {
        return Ok(Some(hit));
    }
    if in_generated_batch(conn, id).await?.is_some() {
        return Ok(Some((
            SettlementCancelBlocker::SupplierInvoiceInPaymentBatch,
            None,
            None,
        )));
    }
    Ok(None)
}

/// Annule une facture fournisseur, **ouverte ou payée**, dans une transaction
/// fournie, sans `BEGIN` ni `COMMIT` (Story 25-3-c, #454).
///
/// Contre-passe l'écriture d'**achat** par le socle
/// ([`journal_entries::reverse_owned_in_tx`], autorité `SupplierPurchase`) —
/// qui pose `reverses_entry_id` et consulte les motifs du grand livre — et passe
/// la facture en `cancelled`. Payée, son règlement **reste au grand livre** et
/// en est **détaché** (arbitrage du 2026-09-26) : les colonnes `settlement_*`
/// et `paid_at` reviennent à `NULL`, l'écriture de règlement n'a plus de
/// propriétaire ; l'audit garde le lien.
///
/// L'annulation n'est plus **définitive** (Story v0172-annulees, #868) : une
/// annulée saisie par erreur repasse en brouillon par [`unvalidate_in_tx`],
/// qui supprime l'achat et la contre-passation posée ici — sous les gardes de
/// la dévalidation, et tant que le règlement détaché n'est pas contre-passé,
/// jamais (son gel lit la facture par le statut `cancelled`).
///
/// ⛔ **Les verrous d'abord, tous, avant toute lecture non verrouillante**
/// (leçon de la Story 25-3-b) : sous `REPEATABLE READ`, la première lecture non
/// verrouillante fige l'instantané ; placée avant un verrou, elle ferait relire
/// après l'attente un exercice « ouvert » qu'une clôture concurrente vient de
/// fermer. ⚠️ Un appelant qui compose ne doit donc avoir fait **aucune**
/// lecture non verrouillante dans la transaction avant d'appeler ceci.
///
/// N'exécute **pas** de rollback : l'appelant, propriétaire de la
/// transaction, en est responsable.
///
/// [`journal_entries::reverse_owned_in_tx`]: crate::repositories::journal_entries::reverse_owned_in_tx
pub async fn cancel_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    use crate::errors::SettlementCancelBlocker;
    use crate::repositories::journal_entries::{self, ReversalAuthority};

    // (1) Verrou facture, puis verrou de l'écriture d'achat ET de son exercice.
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;
    //     Story v017-4a (AC-13) — un brouillon n'a pas d'écriture d'achat : le
    //     rang 1 le refuse avant toute lecture de l'écriture.
    if inv.status == "draft" {
        return Err(DbError::SupplierInvoiceNotCancellable {
            blocker: SettlementCancelBlocker::SupplierInvoiceDraft,
        });
    }
    let purchase_entry_id = inv.purchase_journal_entry_id.ok_or_else(|| {
        DbError::Invariant("facture fournisseur validée sans écriture d'achat".into())
    })?;
    //     Sa valeur est GARDÉE (Story 15-1a2-ii) : c'est l'exercice tenu de la
    //     dissolution de l'étape (2 bis).
    let purchase_fiscal_year_id: i64 = sqlx::query_scalar(
        "SELECT fy.id FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? FOR UPDATE",
    )
    .bind(purchase_entry_id)
    .bind(company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?
    .ok_or_else(|| DbError::Invariant("facture fournisseur sans écriture d'achat".into()))?;

    // Story 28-4a (#642, W28) — facture payée par virement vers un compte clos
    // (l'annulation en détache `settlement_bank_account_id`) : lecture non
    // verrouillante APRÈS le verrou de l'écriture d'achat et de son exercice,
    // avant les motifs de (2) — « les verrous d'abord, tous, avant toute
    // lecture non verrouillante » (doc ci-dessus) ; règle de placement de
    // `bank_accounts::ensure_open_in_tx`. Ne lit pas `archived`. Revérifiée sous
    // verrou après l'étape (4) (Story 29-4b, #651).
    if let Some(bank_account_id) = inv.settlement_bank_account_id {
        super::bank_accounts::ensure_open_in_tx(tx, company_id, bank_account_id).await?;
    }

    // (2) Les motifs. Têtes, exercice clos et lettrage figé par la période
    //     (rang 2 bis, Story 15-1a2-0) : refusés ici. Les autres : laissés au
    //     socle, qui les refuse avec son erreur canonique (400 qui NOMME les
    //     comptes archivés).
    if let Some((
        blocker @ (SettlementCancelBlocker::SupplierInvoiceDraft
        | SettlementCancelBlocker::SupplierInvoiceCancelled
        | SettlementCancelBlocker::FiscalYearClosed
        | SettlementCancelBlocker::DocumentLetteringInClosedPeriods
        | SettlementCancelBlocker::SupplierInvoiceInPaymentBatch),
        _,
        _,
    )) = supplier_invoice_cancel_blocker(tx, company_id, id).await?
    {
        return Err(DbError::SupplierInvoiceNotCancellable { blocker });
    }

    // (2 bis) Le lettrage de la pièce se défait AVANT la contre-passation
    //     (Story 15-1a2-ii, P4) — après les refus, pour que la cause rendue
    //     reste celle du refus. Facture payée : le groupe {achat, règlement} est
    //     dissous ; la contre-passation lettre ensuite l'achat avec son miroir,
    //     et le règlement DÉTACHÉ reste ouvert, lettrable à la main. Facture
    //     ouverte : aucun groupe, no-op. L'exercice tenu est celui de l'achat,
    //     verrouillé en (1) et ouvert (refus « exercice clos ») ; le rang 2 bis
    //     garantit qu'une ligne du groupe est en période ouverte.
    letterings::dissolve_supplier_invoice_document_group_in_tx(
        tx,
        company_id,
        id,
        purchase_fiscal_year_id,
        letterings::Actor {
            user_id,
            api_key_id: None,
        },
    )
    .await?;

    // (3) La contre-passation de l'achat, au titre de cette facture.
    let reversal = journal_entries::reverse_owned_in_tx(
        tx,
        company_id,
        purchase_entry_id,
        user_id,
        ReversalAuthority::SupplierPurchase {
            supplier_invoice_id: id,
        },
    )
    .await?;

    // (4) `cancelled`, règlement DÉTACHÉ (colonnes à NULL — déjà NULL pour une
    //     facture ouverte). ⛔ L'écriture de règlement n'est PAS contre-passée.
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET status = 'cancelled', settlement_type = NULL, \
         settlement_bank_account_id = NULL, settlement_account_id = NULL, \
         settlement_journal_entry_id = NULL, paid_at = NULL, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status IN ('open', 'paid')",
    )
    .bind(id)
    .bind(company_id)
    .bind(inv.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (4 bis) Story 29-4b (#651, W28, AC6) — facture payée par virement :
    //     revérification VERROUILLANTE du compte clos (`closed_on` seul, jamais
    //     `archived`), sur le compte lu en (1) — valeur de la ligne en mémoire,
    //     l'`UPDATE` ci-dessus l'ayant remis à `NULL` en base — en DERNIÈRE
    //     acquisition de verrou, après la dernière écriture, avant l'audit.
    //     Voir `bank_accounts::recheck_open_locked_in_tx` (dont son angle mort
    //     `innodb_snapshot_isolation`).
    if let Some(bank_account_id) = inv.settlement_bank_account_id {
        super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bank_account_id).await?;
    }

    // (5) L'audit du GESTE — seule trace du règlement détaché.
    let mut details = serde_json::json!({
        "previousStatus": inv.status,
        "purchaseJournalEntryId": purchase_entry_id,
        "reversalJournalEntryId": reversal.entry.id,
    });
    if let Some(settlement_entry) = inv.settlement_journal_entry_id {
        details["settlementJournalEntryId"] = serde_json::json!(settlement_entry);
        details["settlementType"] = serde_json::json!(inv.settlement_type);
        details["paidAt"] = serde_json::json!(inv.paid_at);
    }
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.cancelled".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(details),
        ),
    )
    .await?;

    let updated = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let lines = fetch_lines(tx, id).await?;
    Ok(SupplierInvoiceWithLines {
        invoice: updated,
        lines,
    })
}

/// Annule une facture fournisseur, transaction comprise — cf. [`cancel_in_tx`].
pub async fn cancel(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<SupplierInvoiceWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match cancel_in_tx(&mut tx, company_id, id, user_id).await {
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

// ---------------------------------------------------------------------------
// Story 25-3-a-2 (#414) — annuler un règlement fournisseur
// ---------------------------------------------------------------------------

/// Ce qui empêche d'annuler le règlement de la facture fournisseur
/// `supplier_invoice_id` — la **tête** fournisseur (rang 1 : la facture n'est
/// pas `paid`), puis la queue commune sur son écriture de règlement.
///
/// ⛔ **Aucun jumeau** : la queue est
/// [`super::settlement_cancellation::settlement_entry_cancel_blocker`], la même
/// que pour le client (Story 25-3-a-1). Une seule fonction pour lire (l'écran
/// masque le bouton avant le clic) et pour écrire (le geste refuse).
///
/// Facture introuvable (ou d'une autre société) → [`DbError::NotFound`].
pub async fn supplier_settlement_cancel_blocker(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    supplier_invoice_id: i64,
) -> Result<Option<super::settlement_cancellation::SettlementCancelHit>, DbError> {
    supplier_settlement_cancel_blocker_unlinking(conn, company_id, supplier_invoice_id, None).await
}

/// [`supplier_settlement_cancel_blocker`], avec l'**exemption étroite** du
/// rang 3 pour la transaction bancaire `unlinking` que l'on dé-rapproche
/// (Story 30-1a, #686, AC-a3) — patron exact de
/// [`super::invoice_settlements_write::settlement_cancel_blocker_unlinking`] ;
/// cf. [`super::settlement_cancellation::settlement_entry_cancel_blocker`].
///
/// ⛔ Le rang 1 n'est écrit qu'ici : le dé-rapprochement d'un règlement
/// fournisseur ([`super::reconciliation_cancel::cancel_in_tx`]) l'appelle, il
/// ne le recopie pas. Les annulations du règlement depuis la fiche passent
/// `None` (par l'enveloppe) : leur comportement est inchangé.
pub async fn supplier_settlement_cancel_blocker_unlinking(
    conn: &mut sqlx::MySqlConnection,
    company_id: i64,
    supplier_invoice_id: i64,
    unlinking: Option<i64>,
) -> Result<Option<super::settlement_cancellation::SettlementCancelHit>, DbError> {
    let row: Option<(String, Option<i64>)> = sqlx::query_as(
        "SELECT status, settlement_journal_entry_id FROM supplier_invoices \
         WHERE id = ? AND company_id = ?",
    )
    .bind(supplier_invoice_id)
    .bind(company_id)
    .fetch_optional(&mut *conn)
    .await
    .map_err(map_db_error)?;
    let (status, settlement_entry) = row.ok_or(DbError::NotFound)?;
    // Rang 1. ⚠️ `chk_supplier_invoices_paid_has_settlement` garantit qu'une
    // facture `paid` porte son écriture de règlement : l'absence ne peut venir
    // que d'une facture non `paid`.
    let Some(entry_id) = settlement_entry.filter(|_| status == "paid") else {
        return Ok(Some((
            crate::errors::SettlementCancelBlocker::SupplierInvoiceNotPaid,
            None,
            None,
        )));
    };
    super::settlement_cancellation::settlement_entry_cancel_blocker(
        conn, company_id, entry_id, unlinking,
    )
    .await
}

/// Ce que rend l'annulation d'un règlement fournisseur.
#[derive(Debug, Clone)]
pub struct SupplierSettlementCancellation {
    /// La facture relue, revenue à `open`.
    pub invoice: SupplierInvoiceWithLines,
    /// L'écriture inverse, datée du jour.
    pub reversal_journal_entry_id: i64,
}

/// Annule le règlement d'une facture fournisseur **dans une transaction
/// fournie**, sans `BEGIN` ni `COMMIT` (Story 25-3-a-2, #414) : contre-passe
/// l'écriture de règlement (datée du jour), ramène la facture à `open` et vide
/// ses colonnes de règlement.
///
/// Passe par le socle, qui pose `reverses_entry_id` — comme [`cancel`] depuis
/// la Story 25-3-c.
///
/// ⚠️ **Pas de `guard_not_in_generated_batch`** : une facture `paid` ne peut pas
/// être dans un lot `generated` (`create_batch` exige `open`, `pay` et `cancel`
/// refusent une facture en lot `generated`, et `confirm_batch` règle et confirme
/// dans la même transaction). Le lot confirmé qui l'a éventuellement réglée
/// **n'est pas modifié** (arbitrage : il reste l'historique de l'ordre transmis).
pub async fn cancel_settlement_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<SupplierSettlementCancellation, DbError> {
    use crate::errors::SettlementCancelBlocker;
    use crate::repositories::journal_entries::{self, ReversalAuthority};

    // (1) Verrou facture.
    let inv = sqlx::query_as::<_, SupplierInvoice>(&format!("{FIND_SCOPED_SQL} FOR UPDATE"))
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?
        .ok_or(DbError::NotFound)?;

    // (1-bis) ⛔ Verrou sur l'écriture de règlement ET son exercice — même
    // raison que l'étape 2-bis du geste client (Story 25-3-a-1, passe 1 de
    // revue) : la queue lit l'exercice SANS verrou, et le socle ne verrouille
    // l'exercice de l'origine qu'APRÈS ; une clôture concurrente passerait
    // entre les deux. Une facture non `paid` n'a pas d'écriture : le rang 1
    // la refuse juste après.
    //     Sa valeur est GARDÉE (Story 15-1a2-ii) : c'est l'exercice tenu de la
    //     dissolution de l'étape (2 bis).
    let mut settlement_fiscal_year_id = None;
    if let (true, Some(entry_id)) = (inv.status == "paid", inv.settlement_journal_entry_id) {
        let fy_id: i64 = sqlx::query_scalar(
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
        settlement_fiscal_year_id = Some(fy_id);
    }

    // Story 28-4a (#642, W29) — règlement par virement vers un compte clos :
    // lecture non verrouillante APRÈS (1-bis), avant les motifs de (2) — « la
    // queue lit l'exercice SANS verrou, et le socle ne verrouille l'exercice de
    // l'origine qu'APRÈS » (ci-dessus) ; règle de placement de
    // `bank_accounts::ensure_open_in_tx`. Ne lit pas `archived`. Revérifiée sous
    // verrou après l'étape (4) (Story 29-4b, #651).
    if let Some(bank_account_id) = inv.settlement_bank_account_id {
        super::bank_accounts::ensure_open_in_tx(tx, company_id, bank_account_id).await?;
    }

    // (2) Les motifs. Rang 1, « exercice clos » et lettrage figé par la
    //     période (rang 2 bis, Story 15-1a2-0) : refusés ici. Les autres :
    //     laissés au socle, qui les refuse avec son erreur canonique.
    if let Some((
        blocker @ (SettlementCancelBlocker::SupplierInvoiceNotPaid
        | SettlementCancelBlocker::FiscalYearClosed
        | SettlementCancelBlocker::DocumentLetteringInClosedPeriods),
        _,
        _,
    )) = supplier_settlement_cancel_blocker(tx, company_id, id).await?
    {
        return Err(DbError::SettlementNotCancellable { blocker });
    }
    let entry_id = inv
        .settlement_journal_entry_id
        .ok_or_else(|| DbError::Invariant("facture payée sans écriture de règlement".into()))?;
    let settlement_fiscal_year_id = settlement_fiscal_year_id.ok_or_else(|| {
        DbError::Invariant("facture payée : exercice du règlement non verrouillé".into())
    })?;

    // (2 bis) Le lettrage de la pièce se défait AVANT la contre-passation
    //     (Story 15-1a2-ii, P4) — après les refus : la contre-passation lettre
    //     ensuite le règlement avec son miroir (15-1a-ii R6), et la dette
    //     redevient ouverte. L'exercice tenu est celui du règlement, verrouillé
    //     en (1-bis) et ouvert (rang « exercice clos ») ; le rang 2 bis garantit
    //     qu'une ligne du groupe est en période ouverte.
    letterings::dissolve_supplier_invoice_document_group_in_tx(
        tx,
        company_id,
        id,
        settlement_fiscal_year_id,
        letterings::Actor {
            user_id,
            api_key_id: None,
        },
    )
    .await?;

    // (3) La contre-passation, au titre de cette facture. ⚠️ AVANT l'UPDATE qui
    //     vide la colonne : le socle y vérifie que l'écriture est bien celle du
    //     règlement.
    let reversal = journal_entries::reverse_owned_in_tx(
        tx,
        company_id,
        entry_id,
        user_id,
        ReversalAuthority::SupplierSettlement {
            supplier_invoice_id: id,
        },
    )
    .await?;

    // (4) Retour à `open`, colonnes de règlement vidées. Il n'y en a qu'un jeu ;
    //     le lien survit dans l'audit et au grand livre (`reverses_entry_id`).
    let rows = sqlx::query(
        "UPDATE supplier_invoices SET status = 'open', settlement_type = NULL, \
         settlement_bank_account_id = NULL, settlement_account_id = NULL, \
         settlement_journal_entry_id = NULL, paid_at = NULL, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'paid'",
    )
    .bind(id)
    .bind(company_id)
    .bind(inv.version)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();
    if rows == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    // (4 bis) Story 29-4b (#651, W29, AC6) — règlement par virement :
    //     revérification VERROUILLANTE du compte clos (`closed_on` seul, jamais
    //     `archived`), sur le compte lu en (1), en DERNIÈRE acquisition de
    //     verrou — après la dernière écriture, avant l'audit. Voir
    //     `bank_accounts::recheck_open_locked_in_tx` (dont son angle mort
    //     `innodb_snapshot_isolation`).
    if let Some(bank_account_id) = inv.settlement_bank_account_id {
        super::bank_accounts::recheck_open_locked_in_tx(tx, company_id, bank_account_id).await?;
    }

    // (5) L'audit du GESTE — il garde le lien que la colonne vient de perdre.
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry::user(
            user_id,
            "supplier_invoice.settlement_cancelled".to_string(),
            "supplier_invoice".to_string(),
            id,
            Some(serde_json::json!({
                "settlementType": inv.settlement_type,
                "settlementJournalEntryId": entry_id,
                "reversalJournalEntryId": reversal.entry.id,
                "paidAt": inv.paid_at,
            })),
        ),
    )
    .await?;

    let updated = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let lines = fetch_lines(tx, id).await?;
    Ok(SupplierSettlementCancellation {
        invoice: SupplierInvoiceWithLines {
            invoice: updated,
            lines,
        },
        reversal_journal_entry_id: reversal.entry.id,
    })
}

/// Annule le règlement d'une facture fournisseur, transaction comprise.
pub async fn cancel_settlement(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<SupplierSettlementCancellation, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match cancel_settlement_in_tx(&mut tx, company_id, id, user_id).await {
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

/// Une facture fournisseur, ses lignes, et ce qui décide de l'annulation de
/// son règlement — **lus dans un seul instantané** (Story 25-3-a-2, revue P1).
#[derive(Debug, Clone)]
pub struct SupplierInvoiceSettlementView {
    pub invoice: SupplierInvoice,
    pub lines: Vec<SupplierInvoiceLine>,
    /// Le premier motif qui refuse l'annulation du **règlement**, ou `None`.
    pub settlement_cancel_blocker: Option<super::settlement_cancellation::SettlementCancelHit>,
    /// Le premier motif qui refuse l'annulation de la **facture** (Story
    /// 25-3-c), ou `None` — cf. [`supplier_invoice_cancel_blocker`].
    pub invoice_cancel_blocker: Option<super::settlement_cancellation::SettlementCancelHit>,
    /// Cf. [`super::payment_batches::last_confirmed_batch_for_invoice`].
    pub last_confirmed_batch: Option<(i64, Option<chrono::NaiveDateTime>)>,
    /// Le document de la facture (Story 30-2c, AC18), `None` sans document —
    /// cf. [`super::imported_supplier_invoices::find_by_supplier_invoice_id_scoped`].
    pub source_document:
        Option<crate::entities::imported_supplier_invoice::ImportedSupplierInvoice>,
}

/// Lit une facture fournisseur **et** ses champs d'annulation dans une même
/// transaction de lecture, ou `None` si elle n'existe pas pour cette société.
///
/// ⛔ **Pourquoi une transaction pour lire.** Sous `REPEATABLE READ` (défaut de
/// MariaDB), toutes les lectures non verrouillantes d'une transaction voient
/// l'instantané pris à la première : la facture, son motif de refus et son
/// dernier lot décrivent donc le **même** état. Lus séparément — ce que faisait
/// la première version, après le `COMMIT` de `pay` ou de l'annulation —, un
/// règlement ou une annulation concurrents pouvaient s'intercaler, et la
/// réponse se contredire : `status: "paid"` à côté de
/// `SUPPLIER_INVOICE_NOT_PAID`. ⚠️ Aucune de ces lectures ne doit devenir
/// verrouillante (`FOR UPDATE`) : une lecture verrouillante lit le dernier état
/// validé, **hors** de l'instantané.
pub async fn get_settlement_view(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<SupplierInvoiceSettlementView>, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    let Some(invoice) = sqlx::query_as::<_, SupplierInvoice>(FIND_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?
    else {
        return Ok(None);
    };
    let lines = fetch_lines(&mut tx, invoice.id).await?;
    let settlement_cancel_blocker =
        supplier_settlement_cancel_blocker(&mut tx, company_id, id).await?;
    let invoice_cancel_blocker = supplier_invoice_cancel_blocker(&mut tx, company_id, id).await?;
    let last_confirmed_batch =
        super::payment_batches::last_confirmed_batch_for_invoice(&mut *tx, company_id, id).await?;
    // Story 30-2c (AC18) — le document, lu dans le même instantané (lecture
    // ordinaire : la seule lecture du staging de ce fichier, jamais verrouillante).
    let source_document = super::imported_supplier_invoices::find_by_supplier_invoice_id_scoped(
        &mut *tx, company_id, id,
    )
    .await?;
    tx.commit().await.map_err(map_db_error)?;
    Ok(Some(SupplierInvoiceSettlementView {
        invoice,
        lines,
        settlement_cancel_blocker,
        invoice_cancel_blocker,
        last_confirmed_batch,
        source_document,
    }))
}

// ---------------------------------------------------------------------------
// Story 25-5-a (#386) — lecture exhaustive pour l'export de souveraineté
// ---------------------------------------------------------------------------

/// Toutes les factures fournisseurs d'une société (Story 25-5-a, #386).
///
/// ⛔ **Non bornée** : cf. `credit_notes::list_all_by_company`.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<SupplierInvoice>, DbError> {
    sqlx::query_as::<_, SupplierInvoice>(
        "SELECT id, company_id, contact_id, supplier_invoice_number, internal_number, status, \
         invoice_date, due_date, total_amount, creditor_iban, creditor_qr_iban, payment_reference, \
         expected_payment_amount, project_id, purchase_journal_entry_id, settlement_type, \
         settlement_bank_account_id, settlement_account_id, settlement_journal_entry_id, \
         paid_at, version, created_at, updated_at \
         FROM supplier_invoices WHERE company_id = ? ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Toutes les lignes de factures fournisseurs d'une société (Story 25-5-a, #386).
///
/// ⚠️ Scoping par jointure : `supplier_invoice_lines` n'a pas de `company_id`.
pub async fn list_all_lines_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<SupplierInvoiceLine>, DbError> {
    sqlx::query_as::<_, SupplierInvoiceLine>(
        "SELECT sil.id, sil.supplier_invoice_id, sil.position, sil.description, sil.quantity, \
         sil.unit_price, sil.vat_rate, sil.line_total, sil.expense_account_id, sil.created_at \
         FROM supplier_invoice_lines sil \
         JOIN supplier_invoices si ON sil.supplier_invoice_id = si.id \
         WHERE si.company_id = ? \
         ORDER BY sil.supplier_invoice_id, sil.position",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn purchase_lines_single_rate() {
        // 1000 HT @8.1% → D 6000=1000, D 1171=81, C 2000=1081.
        let lines = vec![(dec!(1000), dec!(8.1), 6000)];
        let (generated, ttc) = generate_purchase_journal_lines(&lines, 2000, Some(1171)).unwrap();
        let je = generated.lines;
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
        let (generated, _) = generate_purchase_journal_lines(&lines, 2000, Some(1171)).unwrap();
        let je = generated.lines;
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
        let (generated, ttc) = generate_purchase_journal_lines(&lines, 2000, None).unwrap();
        let je = generated.lines;
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
