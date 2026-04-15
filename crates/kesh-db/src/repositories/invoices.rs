//! Repository CRUD pour `Invoice` + `InvoiceLine` (Story 5.1).
//!
//! Pattern strictement calqué sur `contacts.rs` / `products.rs` :
//! - Mutations avec audit log atomique (rollback explicite si audit échoue).
//! - Convention `details_json` : snapshot direct pour create/delete,
//!   wrapper `{before, after}` pour update.
//! - Liste paginée via deux `QueryBuilder` distincts (COUNT + SELECT).
//!
//! Spécificités factures :
//! - Relation 1-N avec `invoice_lines` (FK ON DELETE CASCADE).
//! - `total_amount` recalculé par le backend (source de vérité = lignes).
//! - `update` utilise le pattern **replace-all** sur les lignes (DELETE
//!   puis INSERT, dans la même transaction).
//! - `update` charge l'entité initiale sans `FOR UPDATE` (pattern optimiste
//!   products.rs). `delete` utilise `SELECT … FOR UPDATE` pour garantir
//!   l'atomicité snapshot + check statut + DELETE.

use chrono::{Duration, NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use kesh_core::listing::SortDirection;

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::invoice::{Invoice, InvoiceLine, InvoiceUpdate, NewInvoice, NewInvoiceLine};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const LINE_COLUMNS: &str = "id, invoice_id, position, description, quantity, unit_price, \
    vat_rate, line_total, created_at";

/// Toujours scopé par `company_id` (anti-IDOR multi-tenant).
const FIND_INVOICE_SCOPED_SQL: &str = "SELECT id, company_id, contact_id, invoice_number, \
    status, date, due_date, payment_terms, total_amount, journal_entry_id, paid_at, version, created_at, updated_at \
    FROM invoices WHERE id = ? AND company_id = ?";

/// Échappe pour `LIKE ? ESCAPE '\\'`. Dupliqué depuis contacts/products —
/// dette technique suivie (extraire si 4e duplication).
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Snapshot JSON d'une facture (entête + lignes) pour l'audit log.
fn invoice_snapshot_json(inv: &Invoice, lines: &[InvoiceLine]) -> serde_json::Value {
    let lines_json: Vec<serde_json::Value> = lines
        .iter()
        .map(|l| {
            serde_json::json!({
                "id": l.id,
                "position": l.position,
                "description": l.description,
                "quantity": l.quantity.to_string(),
                "unitPrice": l.unit_price.to_string(),
                "vatRate": l.vat_rate.to_string(),
                "lineTotal": l.line_total.to_string(),
            })
        })
        .collect();
    serde_json::json!({
        "id": inv.id,
        "companyId": inv.company_id,
        "contactId": inv.contact_id,
        "invoiceNumber": inv.invoice_number,
        "status": inv.status,
        "date": inv.date.to_string(),
        "dueDate": inv.due_date.map(|d| d.to_string()),
        "paymentTerms": inv.payment_terms,
        "totalAmount": inv.total_amount.to_string(),
        "paidAt": inv.paid_at.map(|dt| dt.to_string()),
        "version": inv.version,
        "lines": lines_json,
    })
}

/// Colonne de tri pour la liste des factures (whitelist anti-injection).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InvoiceSortBy {
    #[default]
    Date,
    DueDate,
    TotalAmount,
    ContactName,
    CreatedAt,
}

impl InvoiceSortBy {
    /// Colonne SQL qualifiée (compatible JOIN).
    pub fn as_sql_column(&self) -> &'static str {
        match self {
            Self::Date => "i.date",
            Self::DueDate => "i.due_date",
            Self::TotalAmount => "i.total_amount",
            Self::ContactName => "c.name",
            Self::CreatedAt => "i.created_at",
        }
    }
}

/// Filtre dérivé « statut de paiement » — non stocké en DB.
///
/// Toujours combiné avec `status = 'validated'` côté handler échéancier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaymentStatusFilter {
    #[default]
    All,
    Paid,
    Unpaid,
    Overdue,
}

/// Paramètres de recherche, tri et pagination.
#[derive(Debug, Clone, Default)]
pub struct InvoiceListQuery {
    pub search: Option<String>,
    pub status: Option<String>,
    pub contact_id: Option<i64>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    /// Filtre échéancier (dérivé de `paid_at` + `due_date`, Story 5.4).
    pub payment_status: Option<PaymentStatusFilter>,
    /// Plafond `due_date <= ?` (Story 5.4).
    pub due_before: Option<NaiveDate>,
    pub sort_by: InvoiceSortBy,
    pub sort_direction: SortDirection,
    pub limit: i64,
    pub offset: i64,
}

/// Projection légère (liste) : entête + `contact_name` via JOIN,
/// sans les lignes (optimisation liste paginée).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct InvoiceListItem {
    pub id: i64,
    pub company_id: i64,
    pub contact_id: i64,
    pub contact_name: String,
    pub invoice_number: Option<String>,
    pub status: String,
    pub date: NaiveDate,
    pub due_date: Option<NaiveDate>,
    pub payment_terms: Option<String>,
    pub total_amount: Decimal,
    pub paid_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug)]
pub struct InvoiceListResult {
    pub items: Vec<InvoiceListItem>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a InvoiceListQuery,
) {
    qb.push(" WHERE i.company_id = ");
    qb.push_bind(company_id);

    if let Some(ref status) = query.status {
        qb.push(" AND i.status = ");
        qb.push_bind(status.clone());
    }

    if let Some(cid) = query.contact_id {
        qb.push(" AND i.contact_id = ");
        qb.push_bind(cid);
    }

    if let Some(df) = query.date_from {
        qb.push(" AND i.date >= ");
        qb.push_bind(df);
    }

    if let Some(dt) = query.date_to {
        qb.push(" AND i.date <= ");
        qb.push_bind(dt);
    }

    // `due_before` : borne haute inclusive (`<=`), cohérente avec `date_to`.
    // Le nom historique « before » est conservé pour compatibilité API externe
    // (queryParam `dueBefore`) ; la sémantique est bien « jusqu'à et y compris ».
    if let Some(db) = query.due_before {
        qb.push(" AND i.due_date <= ");
        qb.push_bind(db);
    }

    // Story 5.4 — filtre dérivé du paid_at / due_date. UTC_DATE() garantit
    // la cohérence quelle que soit la TZ de la session SQL (convention
    // projet : tout en UTC naïf). Known limitation v0.1 : voir Story 5.4
    // « Dette technique v0.2 — fuseau horaire société ».
    //
    // P1 (review pass 1) : tout filtre de paiement non-All implique
    // `status='validated'` (une facture draft/cancelled n'a pas de sémantique
    // de paiement). Enforcé ici côté repository en défense en profondeur,
    // indépendamment du `query.status` passé par le caller.
    match query.payment_status.unwrap_or_default() {
        PaymentStatusFilter::All => {}
        PaymentStatusFilter::Paid => {
            qb.push(" AND i.status = 'validated' AND i.paid_at IS NOT NULL");
        }
        PaymentStatusFilter::Unpaid => {
            qb.push(" AND i.status = 'validated' AND i.paid_at IS NULL");
        }
        PaymentStatusFilter::Overdue => {
            // E6 (review pass 2) : `due_date IS NULL` → NULL < UTC_DATE() = NULL
            // → filtre false → la facture n'est jamais comptée comme overdue.
            // Comportement intentionnel : une facture sans échéance explicite
            // ne peut pas être « en retard ». Le défaut `due_date = invoice.date`
            // posé par le handler `create_invoice` (Story 5.2) garantit que ce
            // cas ne se produit pas en pratique pour les factures créées via
            // Kesh ; une éventuelle `due_date` NULL provient de données legacy.
            qb.push(" AND i.status = 'validated' AND i.paid_at IS NULL AND i.due_date < UTC_DATE()");
        }
    }

    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{}%", escape_like(trimmed));
            qb.push(" AND (COALESCE(i.invoice_number, '') LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" ESCAPE '\\\\' OR c.name LIKE ");
            qb.push_bind(pattern);
            qb.push(" ESCAPE '\\\\')");
        }
    }
}

/// Recalcule `line_total` à partir de (quantity × unit_price), arrondi à 4 décimales.
fn compute_line_total(qty: Decimal, unit_price: Decimal) -> Decimal {
    (qty * unit_price).round_dp(4)
}

/// Somme les `line_total` d'un ensemble de `NewInvoiceLine`.
fn compute_total(lines: &[NewInvoiceLine]) -> Decimal {
    lines.iter().fold(Decimal::ZERO, |acc, l| {
        acc + compute_line_total(l.quantity, l.unit_price)
    })
}

/// Insère les lignes d'une facture dans la transaction.
async fn insert_lines(
    tx: &mut Transaction<'_, MySql>,
    invoice_id: i64,
    lines: &[NewInvoiceLine],
) -> Result<Vec<InvoiceLine>, DbError> {
    let mut out = Vec::with_capacity(lines.len());
    for (idx, l) in lines.iter().enumerate() {
        let line_total = compute_line_total(l.quantity, l.unit_price);
        let res = sqlx::query(
            "INSERT INTO invoice_lines (invoice_id, position, description, quantity, \
             unit_price, vat_rate, line_total) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(invoice_id)
        .bind(idx as i32)
        .bind(&l.description)
        .bind(l.quantity)
        .bind(l.unit_price)
        .bind(l.vat_rate)
        .bind(line_total)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

        let id = i64::try_from(res.last_insert_id())
            .map_err(|_| DbError::Invariant("last_insert_id dépasse i64::MAX".into()))?;

        let line = sqlx::query_as::<_, InvoiceLine>(&format!(
            "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE id = ?"
        ))
        .bind(id)
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
        out.push(line);
    }
    Ok(out)
}

/// Charge les lignes d'une facture ordonnées par position.
async fn fetch_lines(
    executor: &mut Transaction<'_, MySql>,
    invoice_id: i64,
) -> Result<Vec<InvoiceLine>, DbError> {
    sqlx::query_as::<_, InvoiceLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE invoice_id = ? ORDER BY position ASC"
    ))
    .bind(invoice_id)
    .fetch_all(&mut **executor)
    .await
    .map_err(map_db_error)
}

/// Crée une facture brouillon + ses lignes + audit log, atomiquement.
pub async fn create(
    pool: &MySqlPool,
    user_id: i64,
    new: NewInvoice,
) -> Result<(Invoice, Vec<InvoiceLine>), DbError> {
    if new.lines.is_empty() {
        // Défense en profondeur : le handler pré-valide (400 INVALID_INPUT).
        // Ici `Invariant` signale un appel incorrect du repository (bug interne).
        return Err(DbError::Invariant(
            "repository invoices::create appelé avec lines vide (handler doit pré-valider)".into(),
        ));
    }

    let total = compute_total(&new.lines);
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, due_date, payment_terms, \
         total_amount) VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.contact_id)
    .bind(new.date)
    .bind(new.due_date)
    .bind(&new.payment_terms)
    .bind(total)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT invoices".into(),
        ));
    }
    let invoice_id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    let lines = match insert_lines(&mut tx, invoice_id, &new.lines).await {
        Ok(l) => l,
        Err(e) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    };

    let invoice = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
        .bind(invoice_id)
        .bind(new.company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "invoice.created".to_string(),
            entity_type: "invoice".to_string(),
            entity_id: invoice.id,
            details_json: Some(invoice_snapshot_json(&invoice, &lines)),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok((invoice, lines))
}

/// Retourne une facture par ID avec ses lignes, scopée par `company_id`.
pub async fn find_by_id_with_lines(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<(Invoice, Vec<InvoiceLine>)>, DbError> {
    let invoice_opt = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(pool)
        .await
        .map_err(map_db_error)?;

    let Some(invoice) = invoice_opt else {
        return Ok(None);
    };

    let lines = sqlx::query_as::<_, InvoiceLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM invoice_lines WHERE invoice_id = ? ORDER BY position ASC"
    ))
    .bind(id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(Some((invoice, lines)))
}

/// Liste paginée + filtres dynamiques (JOIN contacts).
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: InvoiceListQuery,
) -> Result<InvoiceListResult, DbError> {
    let mut count_qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new(
        "SELECT COUNT(*) FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id",
    );
    push_where_clauses(&mut count_qb, company_id, &query);
    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    let mut items_qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new(
        "SELECT i.id, i.company_id, i.contact_id, c.name AS contact_name, \
         i.invoice_number, i.status, i.date, i.due_date, i.payment_terms, \
         i.total_amount, i.paid_at, i.version, i.created_at, i.updated_at \
         FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id",
    );
    push_where_clauses(&mut items_qb, company_id, &query);
    items_qb.push(" ORDER BY ");
    items_qb.push(query.sort_by.as_sql_column());
    items_qb.push(" ");
    items_qb.push(query.sort_direction.as_sql_keyword());
    items_qb.push(", i.id DESC");
    items_qb.push(" LIMIT ");
    items_qb.push_bind(query.limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(query.offset);

    let items: Vec<InvoiceListItem> = items_qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    Ok(InvoiceListResult {
        items,
        total,
        offset: query.offset,
        limit: query.limit,
    })
}

/// Agrégat pour la page « Échéancier » (Story 5.4).
///
/// Calculé sur les factures validées impayées (`status = 'validated'
/// AND paid_at IS NULL`), filtrées par contact/recherche/dates/due_before.
/// Le filtre `payment_status` de la query est **volontairement ignoré** :
/// le summary reflète toujours les créances en attente, indépendamment
/// de l'onglet actif côté UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DueDatesSummary {
    pub unpaid_count: i64,
    pub unpaid_total: Decimal,
    pub overdue_count: i64,
    pub overdue_total: Decimal,
}

/// Calcule le résumé échéancier (1 requête SQL, 4 colonnes).
///
/// Le filtre `payment_status` de `query` est ignoré — voir [`DueDatesSummary`].
pub async fn due_dates_summary(
    pool: &MySqlPool,
    company_id: i64,
    query: &InvoiceListQuery,
) -> Result<DueDatesSummary, DbError> {
    let mut qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new(
        // CAST AS SIGNED pour forcer BIGINT : MariaDB SUM(CASE…) retourne
        // DECIMAL par défaut, incompatible avec Rust i64.
        "SELECT \
            COUNT(*) AS unpaid_count, \
            COALESCE(SUM(i.total_amount), CAST(0 AS DECIMAL(19,4))) AS unpaid_total, \
            CAST(COALESCE(SUM(CASE WHEN i.due_date < UTC_DATE() THEN 1 ELSE 0 END), 0) AS SIGNED) AS overdue_count, \
            COALESCE(SUM(CASE WHEN i.due_date < UTC_DATE() THEN i.total_amount ELSE 0 END), CAST(0 AS DECIMAL(19,4))) AS overdue_total \
         FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id",
    );
    qb.push(" WHERE i.company_id = ");
    qb.push_bind(company_id);
    qb.push(" AND i.status = 'validated' AND i.paid_at IS NULL");

    if let Some(cid) = query.contact_id {
        qb.push(" AND i.contact_id = ");
        qb.push_bind(cid);
    }
    if let Some(df) = query.date_from {
        qb.push(" AND i.date >= ");
        qb.push_bind(df);
    }
    if let Some(dt) = query.date_to {
        qb.push(" AND i.date <= ");
        qb.push_bind(dt);
    }
    if let Some(db) = query.due_before {
        qb.push(" AND i.due_date <= ");
        qb.push_bind(db);
    }
    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            let pattern = format!("%{}%", escape_like(trimmed));
            qb.push(" AND (COALESCE(i.invoice_number, '') LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
            qb.push_bind(pattern.clone());
            qb.push(" ESCAPE '\\\\' OR c.name LIKE ");
            qb.push_bind(pattern);
            qb.push(" ESCAPE '\\\\')");
        }
    }

    let row: (i64, Decimal, i64, Decimal) = qb
        .build_query_as()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    Ok(DueDatesSummary {
        unpaid_count: row.0,
        unpaid_total: row.1,
        overdue_count: row.2,
        overdue_total: row.3,
    })
}

/// Met à jour une facture brouillon : replace-all sur les lignes +
/// recalcul `total_amount` + audit wrapper `{before, after}`.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    expected_version: i32,
    user_id: i64,
    changes: InvoiceUpdate,
) -> Result<(Invoice, Vec<InvoiceLine>), DbError> {
    if changes.lines.is_empty() {
        return Err(DbError::Invariant(
            "repository invoices::update appelé avec lines vide (handler doit pré-valider)".into(),
        ));
    }

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Pattern optimiste (pas de FOR UPDATE), comme products.rs.
    let before_invoice_opt = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let before_invoice = match before_invoice_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(inv) if inv.status != "draft" => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(format!(
                "impossible de modifier une facture de statut '{}'",
                inv.status
            )));
        }
        Some(inv) if inv.version != expected_version => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::OptimisticLockConflict);
        }
        Some(inv) => inv,
    };

    let before_lines = match fetch_lines(&mut tx, id).await {
        Ok(l) => l,
        Err(e) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    };

    // Replace-all : DELETE anciennes lignes puis INSERT nouvelles.
    sqlx::query("DELETE FROM invoice_lines WHERE invoice_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let new_lines = match insert_lines(&mut tx, id, &changes.lines).await {
        Ok(l) => l,
        Err(e) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    };

    let total = compute_total(&changes.lines);

    let rows = sqlx::query(
        "UPDATE invoices SET contact_id = ?, date = ?, due_date = ?, payment_terms = ?, \
         total_amount = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'draft'",
    )
    .bind(changes.contact_id)
    .bind(changes.date)
    .bind(changes.due_date)
    .bind(&changes.payment_terms)
    .bind(total)
    .bind(id)
    .bind(company_id)
    .bind(expected_version)
    .execute(&mut *tx)
    .await
    .map_err(map_db_error)?
    .rows_affected();

    if rows == 0 {
        // Re-query pour distinguer NotFound (ligne supprimée entre SELECT et UPDATE)
        // vs OptimisticLockConflict (version changée).
        let still_exists = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?;
        tx.rollback().await.map_err(map_db_error)?;
        return Err(match still_exists {
            None => DbError::NotFound,
            Some(_) => DbError::OptimisticLockConflict,
        });
    }

    let after_invoice = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
        .bind(id)
        .bind(company_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let audit_details = serde_json::json!({
        "before": invoice_snapshot_json(&before_invoice, &before_lines),
        "after": invoice_snapshot_json(&after_invoice, &new_lines),
    });

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "invoice.updated".to_string(),
            entity_type: "invoice".to_string(),
            entity_id: id,
            details_json: Some(audit_details),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok((after_invoice, new_lines))
}

/// Supprime une facture brouillon (CASCADE sur lignes) + audit.
/// Utilise `SELECT … FOR UPDATE` pour garantir l'atomicité snapshot/DELETE.
pub async fn delete(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let current_opt = sqlx::query_as::<_, Invoice>(
        "SELECT id, company_id, contact_id, invoice_number, status, date, due_date, \
         payment_terms, total_amount, journal_entry_id, paid_at, version, created_at, updated_at \
         FROM invoices WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let current = match current_opt {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(inv) if inv.status != "draft" => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(format!(
                "impossible de supprimer une facture de statut '{}'",
                inv.status
            )));
        }
        Some(inv) => inv,
    };

    let lines = match fetch_lines(&mut tx, id).await {
        Ok(l) => l,
        Err(e) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(e);
        }
    };
    let snapshot = invoice_snapshot_json(&current, &lines);

    let rows = sqlx::query("DELETE FROM invoices WHERE id = ? AND company_id = ?")
        .bind(id)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();

    if rows == 0 {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::NotFound);
    }

    if let Err(e) = audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "invoice.deleted".to_string(),
            entity_type: "invoice".to_string(),
            entity_id: id,
            details_json: Some(snapshot),
        },
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Story 5.2 — Validation & numérotation
// ---------------------------------------------------------------------------

/// Résultat d'une validation réussie (facture validée + lignes + écriture
/// comptable générée). Les lignes sont retournées pour permettre au
/// handler HTTP de construire la réponse sans re-fetch post-commit
/// (review P3 — évite une fenêtre de race sur les lignes).
#[derive(Debug)]
pub struct ValidatedInvoice {
    pub invoice: Invoice,
    pub lines: Vec<InvoiceLine>,
    pub journal_entry: crate::entities::JournalEntryWithLines,
}

/// Valide une facture brouillon : lui attribue un numéro définitif,
/// génère l'écriture comptable associée, et bascule son statut en
/// `validated`. Le tout dans une transaction atomique.
///
/// # Ordre des locks (canonique — Story 5.2 section Concurrence)
///
/// 1. `invoices` (`SELECT ... FOR UPDATE` sur la facture à valider).
/// 2. `fiscal_years` (via [`fiscal_years::find_open_covering_date`]).
/// 3. `invoice_number_sequences` (via [`invoice_number_sequences::next_number_for`]).
/// 4. `journal_entries` (via [`journal_entries::create_in_tx`]).
/// 5. INSERTs + UPDATE invoices + INSERT audit.
///
/// **Toute divergence de cet ordre = risque de deadlock** avec des
/// créations manuelles concurrentes de `journal_entries`.
///
/// # Erreurs
///
/// - [`DbError::NotFound`] : facture absente ou hors scope company.
/// - [`DbError::IllegalStateTransition`] : statut ≠ `draft`.
/// - [`DbError::FiscalYearInvalid`] : aucun exercice ouvert pour `invoice.date`.
/// - [`DbError::ConfigurationRequired`] : comptes par défaut absents.
/// - [`DbError::OptimisticLockConflict`] : race sur l'UPDATE final (défensif).
pub async fn validate_invoice(
    pool: &MySqlPool,
    company_id: i64,
    invoice_id: i64,
    user_id: i64,
) -> Result<ValidatedInvoice, DbError> {
    use crate::entities::{Journal, NewJournalEntry, NewJournalEntryLine};
    use crate::repositories::{
        company_invoice_settings, fiscal_years, invoice_number_sequences, journal_entries,
    };
    use kesh_core::invoice_format;

    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        // (1) Lock facture + check draft.
        let invoice_before =
            sqlx::query_as::<_, Invoice>(&format!("{FIND_INVOICE_SCOPED_SQL} FOR UPDATE"))
                .bind(invoice_id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;

        let invoice_before = match invoice_before {
            None => return Err(DbError::NotFound),
            Some(inv) if inv.status != "draft" => {
                return Err(DbError::IllegalStateTransition(format!(
                    "impossible de valider une facture de statut '{}'",
                    inv.status
                )));
            }
            Some(inv) => inv,
        };

        let lines_before = fetch_lines(&mut tx, invoice_id).await?;

        // (2) Config company (lazy create si absente).
        let settings =
            company_invoice_settings::get_or_create_default_in_tx(&mut tx, company_id).await?;

        let receivable_account_id = settings.default_receivable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_receivable_account_id".into())
        })?;
        let revenue_account_id = settings
            .default_revenue_account_id
            .ok_or_else(|| DbError::ConfigurationRequired("default_revenue_account_id".into()))?;

        // (3) Fiscal year ouvert couvrant invoice.date.
        let fy = fiscal_years::find_open_covering_date(&mut tx, company_id, invoice_before.date)
            .await?
            .ok_or(DbError::FiscalYearInvalid)?;

        // (4) Sequence : lock + incrément.
        let seq = invoice_number_sequences::next_number_for(&mut tx, company_id, fy.id).await?;

        // (5) Render numéro de facture.
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
        let invoice_number =
            invoice_format::render(&settings.invoice_number_format, year, &fy.name, seq).map_err(
                |e| {
                    DbError::Invariant(format!(
                        "rendu numéro facture échoué (config invalide ?) : {e}"
                    ))
                },
            )?;

        // (6) Contact name pour le libellé écriture.
        let contact_name: String =
            sqlx::query_scalar("SELECT name FROM contacts WHERE id = ? AND company_id = ?")
                .bind(invoice_before.contact_id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?
                .ok_or_else(|| {
                    // Review P10 : la FK contacts n'est pas ON DELETE CASCADE
                    // côté invoices, mais un contact archivé/supprimé par une
                    // voie directe (maintenance, cross-company bug) remonterait
                    // 500 Invariant. On préfère 404 NotFound (surface client
                    // actionnable) — le handler mappe déjà NotFound → 404.
                    let _ = invoice_before.contact_id; // ID présent dans le log log au niveau handler
                    DbError::NotFound
                })?;

        let entry_description = invoice_format::render_journal_entry_description(
            &settings.journal_entry_description_template,
            year,
            &invoice_number,
            &contact_name,
        );

        // (7) Créer l'écriture comptable dans la même tx.
        let total = invoice_before.total_amount;
        let journal: Journal = settings.default_sales_journal;

        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: invoice_before.date,
                journal,
                description: entry_description,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: receivable_account_id,
                        debit: total,
                        credit: Decimal::ZERO,
                    },
                    NewJournalEntryLine {
                        account_id: revenue_account_id,
                        debit: Decimal::ZERO,
                        credit: total,
                    },
                ],
            },
        )
        .await?;

        // (8) UPDATE invoices → validated.
        // Review Q5 (pass 2) : le check `status = 'draft'` dans le WHERE est
        // redondant avec le `SELECT ... FOR UPDATE` de (1) qui garantit que
        // `invoice_before.status = 'draft'` jusqu'au commit. Conservé en
        // défense en profondeur contre un refactor futur qui retirerait le
        // FOR UPDATE initial (le check version suffit mais est moins robuste
        // si le lock disparaît).
        let rows = sqlx::query(
            "UPDATE invoices SET status = 'validated', invoice_number = ?, \
             journal_entry_id = ?, version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'draft'",
        )
        .bind(&invoice_number)
        .bind(je.entry.id)
        .bind(invoice_id)
        .bind(company_id)
        .bind(invoice_before.version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();

        if rows == 0 {
            // Défensif : race entre SELECT FOR UPDATE et UPDATE (ne devrait pas arriver).
            return Err(DbError::OptimisticLockConflict);
        }

        let invoice_after = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
            .bind(invoice_id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

        // (9) Audit log.
        let audit_details = serde_json::json!({
            "before": invoice_snapshot_json(&invoice_before, &lines_before),
            "after": invoice_snapshot_json(&invoice_after, &lines_before),
            "journalEntry": {
                "id": je.entry.id,
                "entryNumber": je.entry.entry_number,
                "journal": je.entry.journal.as_str(),
                "entryDate": je.entry.entry_date.to_string(),
                "description": je.entry.description,
                "lines": je.lines.iter().map(|l| serde_json::json!({
                    "accountId": l.account_id,
                    "lineOrder": l.line_order,
                    "debit": l.debit.to_string(),
                    "credit": l.credit.to_string(),
                })).collect::<Vec<_>>(),
            },
        });

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: "invoice.validated".to_string(),
                entity_type: "invoice".to_string(),
                entity_id: invoice_id,
                details_json: Some(audit_details),
            },
        )
        .await?;

        Ok(ValidatedInvoice {
            invoice: invoice_after,
            lines: lines_before,
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

// ---------------------------------------------------------------------------
// Story 5.4 — Échéancier : mark_as_paid / unmark_as_paid
// ---------------------------------------------------------------------------

/// Marque une facture validée comme payée (`paid_at.is_some()`) ou l'annule
/// (`paid_at.is_none()` → unmark).
///
/// # Concurrence
///
/// Transaction atomique :
/// 1. `SELECT ... FOR UPDATE` sur la facture (scope company).
/// 2. Vérifie `status = 'validated'` (sinon [`DbError::IllegalStateTransition`]).
/// 3. Vérifie `version == expected_version` (sinon [`DbError::OptimisticLockConflict`]).
/// 4. `UPDATE invoices SET paid_at = ?, version = version + 1`.
/// 5. Insert audit log wrapper `{before, after}` avec action `invoice.paid`
///    (si `paid_at.is_some()`) ou `invoice.unpaid` (si `None`).
///
/// # Note écriture comptable
///
/// **Ne crée AUCUNE écriture comptable** en v0.1. L'écriture d'encaissement
/// sera générée par la réconciliation automatique (Epic 6). Ici `paid_at`
/// est un simple marqueur opérationnel.
pub async fn mark_as_paid(
    pool: &MySqlPool,
    user_id: i64,
    id: i64,
    company_id: i64,
    expected_version: i32,
    paid_at: Option<NaiveDateTime>,
) -> Result<Invoice, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        let before_opt =
            sqlx::query_as::<_, Invoice>(&format!("{FIND_INVOICE_SCOPED_SQL} FOR UPDATE"))
                .bind(id)
                .bind(company_id)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_db_error)?;

        let before = match before_opt {
            None => return Err(DbError::NotFound),
            Some(inv) if inv.status != "validated" => {
                return Err(DbError::IllegalStateTransition(format!(
                    "impossible de marquer payée une facture de statut '{}'",
                    inv.status
                )));
            }
            Some(inv) if inv.version != expected_version => {
                return Err(DbError::OptimisticLockConflict);
            }
            Some(inv) => inv,
        };

        // Validation calendaire paid_at vs invoice.date (défense en profondeur —
        // le handler pré-valide aussi). Comparaison sur la date (pas le datetime)
        // pour éviter qu'un paiement à 00:30 UTC soit rejeté comme « antérieur »
        // à une facture du même jour.
        // P2 (review pass 1) : tolérance 1 jour pour absorber l'écart entre
        // `paid_at` stocké en UTC naïf et `invoice.date` en date métier locale
        // (jusqu'à +/- 2h en CET/CEST). Sans cette tolérance, un paiement
        // saisi à 00:30 CET le jour même de la facture serait rejeté.
        if let Some(pa) = paid_at {
            if pa.date() < before.date - Duration::days(1) {
                return Err(DbError::InvalidInput("paidAtBeforeInvoiceDate".to_string()));
            }
        }

        let lines_before = fetch_lines(&mut tx, id).await?;

        let rows = sqlx::query(
            "UPDATE invoices SET paid_at = ?, version = version + 1 \
             WHERE id = ? AND company_id = ? AND version = ? AND status = 'validated'",
        )
        .bind(paid_at)
        .bind(id)
        .bind(company_id)
        .bind(expected_version)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();

        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }

        let after = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

        let audit_details = serde_json::json!({
            "before": invoice_snapshot_json(&before, &lines_before),
            "after": invoice_snapshot_json(&after, &lines_before),
        });

        let action = if paid_at.is_some() {
            "invoice.paid"
        } else {
            "invoice.unpaid"
        };

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry {
                user_id,
                action: action.to_string(),
                entity_type: "invoice".to_string(),
                entity_id: id,
                details_json: Some(audit_details),
            },
        )
        .await?;

        Ok(after)
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

/// Charge jusqu'à `max_rows` factures validées filtrées (pour l'export CSV).
///
/// Contrairement à [`list_by_company_paginated`], pas de LIMIT/OFFSET exposé :
/// le handler passe `max_rows + 1` pour détecter le dépassement (> 10_000).
/// Tri : `due_date ASC, id ASC`.
pub async fn list_for_export(
    pool: &MySqlPool,
    company_id: i64,
    query: &InvoiceListQuery,
    max_rows: i64,
) -> Result<Vec<InvoiceListItem>, DbError> {
    // P3 (review pass 1) : plafond absolu 50_000 en défense en profondeur.
    // La règle métier « > 10_000 → 400 » reste appliquée côté handler ;
    // ce clamp évite un scan complet si un caller passe une valeur aberrante.
    //
    // F3 (review pass 2) : borne basse à 1 (et non 0) pour éviter qu'un appel
    // avec `max_rows <= 0` se traduise silencieusement en `LIMIT 0` (retour
    // vide interprété à tort comme « aucune facture »).
    let max_rows = max_rows.clamp(1, 50_000);
    let mut items_qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new(
        "SELECT i.id, i.company_id, i.contact_id, c.name AS contact_name, \
         i.invoice_number, i.status, i.date, i.due_date, i.payment_terms, \
         i.total_amount, i.paid_at, i.version, i.created_at, i.updated_at \
         FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id",
    );
    push_where_clauses(&mut items_qb, company_id, query);
    items_qb.push(" ORDER BY i.due_date ASC, i.id ASC LIMIT ");
    items_qb.push_bind(max_rows);

    items_qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)
}

// ---------------------------------------------------------------------------
// Tests d'intégration DB (Story 5.1)
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::contact::{ContactType, NewContact};
    use crate::repositories::contacts;
    use rust_decimal_macros::dec;
    use sqlx::QueryBuilder;
    use uuid::Uuid;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    async fn get_company_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests");
        row.0
    }

    async fn get_admin_user_id(pool: &MySqlPool) -> i64 {
        let row: (i64,) = sqlx::query_as("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one Admin user in DB for tests");
        row.0
    }

    fn short_uuid() -> String {
        Uuid::new_v4().simple().to_string()[..8].to_string()
    }

    async fn create_test_contact(pool: &MySqlPool, company_id: i64, user_id: i64) -> i64 {
        let suffix = short_uuid();
        let contact = contacts::create(
            pool,
            user_id,
            NewContact {
                company_id,
                contact_type: ContactType::Entreprise,
                name: format!("TestInvoiceContact_{suffix}"),
                is_client: true,
                is_supplier: false,
                address: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: Some("30 jours net".into()),
            },
        )
        .await
        .expect("create_test_contact");
        contact.id
    }

    async fn cleanup_invoices(pool: &MySqlPool, ids: &[i64]) {
        if ids.is_empty() {
            return;
        }
        let mut qb: QueryBuilder<sqlx::MySql> =
            QueryBuilder::new("DELETE FROM invoices WHERE id IN (");
        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(*id);
        }
        sep.push_unseparated(")");
        qb.build().execute(pool).await.ok();
    }

    async fn cleanup_contacts(pool: &MySqlPool, ids: &[i64]) {
        if ids.is_empty() {
            return;
        }
        let mut qb: QueryBuilder<sqlx::MySql> =
            QueryBuilder::new("DELETE FROM contacts WHERE id IN (");
        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(*id);
        }
        sep.push_unseparated(")");
        qb.build().execute(pool).await.ok();
    }

    fn sample_line(desc: &str, qty: Decimal, price: Decimal) -> NewInvoiceLine {
        NewInvoiceLine {
            description: desc.to_string(),
            quantity: qty,
            unit_price: price,
            vat_rate: dec!(8.10),
        }
    }

    fn today() -> NaiveDate {
        chrono::Utc::now().naive_utc().date()
    }

    #[tokio::test]
    async fn test_create_with_lines_computes_total() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let new = NewInvoice {
            company_id,
            contact_id,
            date: today(),
            due_date: None,
            payment_terms: Some("30 jours net".into()),
            lines: vec![
                sample_line("Conseil", dec!(4.5), dec!(200.00)),
                sample_line("Logo", dec!(1), dec!(500.00)),
            ],
        };
        let (inv, lines) = create(&pool, admin_user_id, new).await.unwrap();
        assert_eq!(inv.status, "draft");
        assert_eq!(inv.version, 1);
        assert_eq!(inv.total_amount, dec!(1400.0000));
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].line_total, dec!(900.0000));
        assert_eq!(lines[0].position, 0);
        assert_eq!(lines[1].position, 1);

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_create_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("Item A", dec!(2), dec!(100.00))],
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "invoice", inv.id, 10)
            .await
            .unwrap();
        let created = entries
            .iter()
            .find(|e| e.action == "invoice.created")
            .expect("invoice.created audit entry");
        let details = created.details_json.as_ref().unwrap();
        assert_eq!(
            details.get("totalAmount").and_then(|v| v.as_str()),
            Some("200.0000")
        );
        assert!(details.get("lines").and_then(|v| v.as_array()).is_some());

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_update_replaces_all_lines() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![
                    sample_line("Old A", dec!(1), dec!(100.00)),
                    sample_line("Old B", dec!(1), dec!(100.00)),
                ],
            },
        )
        .await
        .unwrap();

        let (updated, new_lines) = update(
            &pool,
            company_id,
            inv.id,
            inv.version,
            admin_user_id,
            InvoiceUpdate {
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![
                    sample_line("New 1", dec!(1), dec!(50.00)),
                    sample_line("New 2", dec!(1), dec!(50.00)),
                    sample_line("New 3", dec!(1), dec!(50.00)),
                ],
            },
        )
        .await
        .unwrap();
        assert_eq!(updated.version, 2);
        assert_eq!(updated.total_amount, dec!(150.0000));
        assert_eq!(new_lines.len(), 3);
        assert_eq!(new_lines[0].position, 0);
        assert_eq!(new_lines[2].position, 2);

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_update_optimistic_lock_conflict() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();

        let err = update(
            &pool,
            company_id,
            inv.id,
            999,
            admin_user_id,
            InvoiceUpdate {
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X2", dec!(1), dec!(20.00))],
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_update_rejects_non_draft() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();

        // Bascule validated + journal_entry stub (respecte la CHECK
        // chk_invoices_validated_has_je de la Story 5.2).
        let (_v, je_id) = force_validate(&pool, company_id, inv.id).await;

        let err = update(
            &pool,
            company_id,
            inv.id,
            inv.version,
            admin_user_id,
            InvoiceUpdate {
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("Y", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_update_writes_audit_log_wrapper() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("Before", dec!(1), dec!(100.00))],
            },
        )
        .await
        .unwrap();

        let _ = update(
            &pool,
            company_id,
            inv.id,
            inv.version,
            admin_user_id,
            InvoiceUpdate {
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("After", dec!(2), dec!(50.00))],
            },
        )
        .await
        .unwrap();

        let entries = audit_log::find_by_entity(&pool, "invoice", inv.id, 10)
            .await
            .unwrap();
        let upd = entries
            .iter()
            .find(|e| e.action == "invoice.updated")
            .expect("invoice.updated audit entry");
        let details = upd.details_json.as_ref().unwrap();
        assert!(details.get("before").is_some());
        assert!(details.get("after").is_some());

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_delete_cascades_lines() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, lines) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![
                    sample_line("L1", dec!(1), dec!(10.00)),
                    sample_line("L2", dec!(1), dec!(20.00)),
                ],
            },
        )
        .await
        .unwrap();
        let line_id = lines[0].id;

        delete(&pool, company_id, inv.id, admin_user_id)
            .await
            .unwrap();

        let found: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoice_lines WHERE id = ?")
            .bind(line_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(found.is_none(), "CASCADE must delete lines");

        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_delete_rejects_non_draft() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();
        let (_v, je_id) = force_validate(&pool, company_id, inv.id).await;

        let err = delete(&pool, company_id, inv.id, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_delete_writes_audit_log() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();
        delete(&pool, company_id, inv.id, admin_user_id)
            .await
            .unwrap();

        let entries = audit_log::find_by_entity(&pool, "invoice", inv.id, 10)
            .await
            .unwrap();
        assert!(entries.iter().any(|e| e.action == "invoice.deleted"));

        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_list_filters_by_status_and_date_range() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("L", dec!(1), dec!(50.00))],
            },
        )
        .await
        .unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                status: Some("draft".into()),
                contact_id: Some(contact_id),
                date_from: Some(today()),
                date_to: Some(today()),
                limit: 100,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(result.items.iter().any(|i| i.id == inv.id));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_find_by_id_returns_lines_ordered_by_position() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![
                    sample_line("A", dec!(1), dec!(10.00)),
                    sample_line("B", dec!(1), dec!(20.00)),
                    sample_line("C", dec!(1), dec!(30.00)),
                ],
            },
        )
        .await
        .unwrap();

        let (_, lines) = find_by_id_with_lines(&pool, company_id, inv.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(lines.len(), 3);
        for (idx, l) in lines.iter().enumerate() {
            assert_eq!(l.position, idx as i32);
        }

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_db_rejects_quantity_zero_via_direct_insert() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();

        let err = sqlx::query(
            "INSERT INTO invoice_lines (invoice_id, position, description, quantity, \
             unit_price, vat_rate, line_total) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(inv.id)
        .bind(99)
        .bind("bad")
        .bind(dec!(0))
        .bind(dec!(10.00))
        .bind(dec!(8.10))
        .bind(dec!(0))
        .execute(&pool)
        .await
        .unwrap_err();
        assert!(matches!(
            map_db_error(err),
            DbError::CheckConstraintViolation(_)
        ));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_db_rejects_invalid_status_via_direct_update() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();
        let err = sqlx::query("UPDATE invoices SET status = 'bogus' WHERE id = ?")
            .bind(inv.id)
            .execute(&pool)
            .await
            .unwrap_err();
        assert!(matches!(
            map_db_error(err),
            DbError::CheckConstraintViolation(_)
        ));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_list_orders_by_date_desc_by_default() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let old_date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let mid_date = NaiveDate::from_ymd_opt(2026, 2, 15).unwrap();
        let new_date = NaiveDate::from_ymd_opt(2026, 4, 10).unwrap();

        let mut ids = Vec::new();
        for d in [old_date, mid_date, new_date] {
            let (inv, _) = create(
                &pool,
                admin_user_id,
                NewInvoice {
                    company_id,
                    contact_id,
                    date: d,
                    due_date: None,
                    payment_terms: None,
                    lines: vec![sample_line("L", dec!(1), dec!(10.00))],
                },
            )
            .await
            .unwrap();
            ids.push(inv.id);
        }

        let result = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                contact_id: Some(contact_id),
                limit: 100,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Filtre contact restreint les résultats à nos 3 factures → tri date DESC.
        let dates: Vec<NaiveDate> = result.items.iter().map(|i| i.date).collect();
        assert_eq!(dates, vec![new_date, mid_date, old_date]);

        cleanup_invoices(&pool, &ids).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_list_filter_excludes_out_of_range_dates() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let inside = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let outside = NaiveDate::from_ymd_opt(2025, 12, 31).unwrap();

        let (inv_in, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: inside,
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("L", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();
        let (inv_out, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: outside,
                due_date: None,
                payment_terms: None,
                lines: vec![sample_line("L", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();

        let result = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                contact_id: Some(contact_id),
                date_from: Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap()),
                date_to: Some(NaiveDate::from_ymd_opt(2026, 12, 31).unwrap()),
                limit: 100,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert!(result.items.iter().any(|i| i.id == inv_in.id));
        assert!(
            !result.items.iter().any(|i| i.id == inv_out.id),
            "out-of-range invoice must be excluded by date filter"
        );

        cleanup_invoices(&pool, &[inv_in.id, inv_out.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    // -----------------------------------------------------------------------
    // Story 5.4 — Échéancier factures
    // -----------------------------------------------------------------------

    /// Assure un `fiscal_years` couvrant une plage large pour la company de
    /// test (idempotent). Crée aussi un `journal_entries` stub et bascule
    /// l'invoice en `validated` avec la FK — nécessaire pour satisfaire la
    /// CHECK `chk_invoices_validated_has_je` (Story 5.2 mig 20260417000002).
    async fn ensure_fiscal_year(pool: &MySqlPool, company_id: i64) -> i64 {
        if let Some((id,)) =
            sqlx::query_as::<_, (i64,)>("SELECT id FROM fiscal_years WHERE company_id = ? LIMIT 1")
                .bind(company_id)
                .fetch_optional(pool)
                .await
                .unwrap()
        {
            return id;
        }
        let res = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, 'Test5.4Exercice', '2020-01-01', '2030-12-31', 'Open')",
        )
        .bind(company_id)
        .execute(pool)
        .await
        .unwrap();
        res.last_insert_id() as i64
    }

    async fn insert_stub_journal_entry(pool: &MySqlPool, company_id: i64, fy_id: i64) -> i64 {
        let (max_n,): (Option<i64>,) = sqlx::query_as(
            "SELECT MAX(entry_number) FROM journal_entries \
             WHERE company_id = ? AND fiscal_year_id = ?",
        )
        .bind(company_id)
        .bind(fy_id)
        .fetch_one(pool)
        .await
        .unwrap();
        let next_n = max_n.unwrap_or(0) + 1;
        let res = sqlx::query(
            "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, \
             entry_date, journal, description) VALUES (?, ?, ?, CURDATE(), 'Ventes', 'stub-5.4')",
        )
        .bind(company_id)
        .bind(fy_id)
        .bind(next_n)
        .execute(pool)
        .await
        .unwrap();
        res.last_insert_id() as i64
    }

    /// Bascule une facture brouillon en `validated` avec un journal_entry
    /// stub lié, sans passer par validate_invoice (qui requiert la config
    /// complète company_invoice_settings + sequences). Retourne
    /// `(new_version, journal_entry_id)` — ce dernier est utile pour le
    /// cleanup car `ON DELETE RESTRICT` empêche la purge tant qu'une
    /// invoice référence la je.
    async fn force_validate(pool: &MySqlPool, company_id: i64, invoice_id: i64) -> (i32, i64) {
        let fy_id = ensure_fiscal_year(pool, company_id).await;
        let je_id = insert_stub_journal_entry(pool, company_id, fy_id).await;
        sqlx::query(
            "UPDATE invoices SET status = 'validated', journal_entry_id = ?, \
             version = version + 1 WHERE id = ?",
        )
        .bind(je_id)
        .bind(invoice_id)
        .execute(pool)
        .await
        .unwrap();
        let (v,): (i32,) = sqlx::query_as("SELECT version FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(pool)
            .await
            .unwrap();
        (v, je_id)
    }

    async fn cleanup_journal_entries(pool: &MySqlPool, ids: &[i64]) {
        if ids.is_empty() {
            return;
        }
        let mut qb: QueryBuilder<sqlx::MySql> =
            QueryBuilder::new("DELETE FROM journal_entries WHERE id IN (");
        let mut sep = qb.separated(", ");
        for id in ids {
            sep.push_bind(*id);
        }
        sep.push_unseparated(")");
        qb.build().execute(pool).await.ok();
    }

    async fn create_and_validate(
        pool: &MySqlPool,
        company_id: i64,
        admin_user_id: i64,
        contact_id: i64,
        date: NaiveDate,
        due_date: Option<NaiveDate>,
        amount: Decimal,
    ) -> (i64, i32, i64) {
        let (inv, _) = create(
            pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date,
                due_date,
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), amount)],
            },
        )
        .await
        .unwrap();
        let (v, je_id) = force_validate(pool, company_id, inv.id).await;
        (inv.id, v, je_id)
    }

    #[tokio::test]
    async fn test_mark_as_paid_nominal() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(100.00),
        )
        .await;

        let paid_at = Some(chrono::Utc::now().naive_utc());
        let after = mark_as_paid(&pool, admin_user_id, id, company_id, v, paid_at)
            .await
            .unwrap();
        assert!(after.paid_at.is_some());
        assert_eq!(after.version, v + 1);

        let entries = audit_log::find_by_entity(&pool, "invoice", id, 10)
            .await
            .unwrap();
        assert!(entries.iter().any(|e| e.action == "invoice.paid"));

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_mark_as_paid_rejects_draft() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (inv, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id,
                date: today(),
                due_date: Some(today()),
                payment_terms: None,
                lines: vec![sample_line("X", dec!(1), dec!(10.00))],
            },
        )
        .await
        .unwrap();

        let err = mark_as_paid(
            &pool,
            admin_user_id,
            inv.id,
            company_id,
            inv.version,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_mark_as_paid_optimistic_lock() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(50.00),
        )
        .await;

        let err = mark_as_paid(
            &pool,
            admin_user_id,
            id,
            company_id,
            v + 42,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap_err();
        assert!(matches!(err, DbError::OptimisticLockConflict));

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_unmark_paid_writes_audit_unpaid() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(75.00),
        )
        .await;

        let after1 = mark_as_paid(
            &pool,
            admin_user_id,
            id,
            company_id,
            v,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();
        let after2 = mark_as_paid(&pool, admin_user_id, id, company_id, after1.version, None)
            .await
            .unwrap();
        assert!(after2.paid_at.is_none());

        let entries = audit_log::find_by_entity(&pool, "invoice", id, 10)
            .await
            .unwrap();
        assert!(entries.iter().any(|e| e.action == "invoice.unpaid"));

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_mark_as_paid_rejects_paid_at_before_invoice_date() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let invoice_date = NaiveDate::from_ymd_opt(2026, 3, 15).unwrap();
        let (id, v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            invoice_date,
            Some(invoice_date),
            dec!(10.00),
        )
        .await;

        let before = NaiveDate::from_ymd_opt(2026, 3, 10)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap();
        let err = mark_as_paid(&pool, admin_user_id, id, company_id, v, Some(before))
            .await
            .unwrap_err();
        match err {
            DbError::InvalidInput(code) => assert_eq!(code, "paidAtBeforeInvoiceDate"),
            other => panic!("attendu InvalidInput, reçu {other:?}"),
        }

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_list_filter_overdue() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let t = today();
        let past = t - chrono::Duration::days(30);
        let future = t + chrono::Duration::days(30);

        // 3 validated invoices : overdue-unpaid / overdue-paid / future-unpaid.
        let (id_overdue_unpaid, _, je1) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            past,
            Some(past),
            dec!(100.00),
        )
        .await;
        let (id_overdue_paid, v_op, je2) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            past,
            Some(past),
            dec!(200.00),
        )
        .await;
        let _ = mark_as_paid(
            &pool,
            admin_user_id,
            id_overdue_paid,
            company_id,
            v_op,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();
        let (id_future_unpaid, _, je3) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            t,
            Some(future),
            dec!(300.00),
        )
        .await;

        let result = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                status: Some("validated".into()),
                contact_id: Some(contact_id),
                payment_status: Some(PaymentStatusFilter::Overdue),
                limit: 100,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();

        let ids: Vec<i64> = result.items.iter().map(|i| i.id).collect();
        assert!(ids.contains(&id_overdue_unpaid));
        assert!(
            !ids.contains(&id_overdue_paid),
            "une facture overdue mais payée doit être exclue"
        );
        assert!(
            !ids.contains(&id_future_unpaid),
            "une facture future non payée n'est pas overdue"
        );

        cleanup_invoices(
            &pool,
            &[id_overdue_unpaid, id_overdue_paid, id_future_unpaid],
        )
        .await;
        cleanup_journal_entries(&pool, &[je1, je2, je3]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_due_dates_summary_computes_correct_totals() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let t = today();
        let past = t - chrono::Duration::days(10);
        let future = t + chrono::Duration::days(10);

        let (id_overdue, _, je1) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            past,
            Some(past),
            dec!(100.00),
        )
        .await;
        let (id_unpaid_not_overdue, _, je2) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            t,
            Some(future),
            dec!(50.00),
        )
        .await;
        let (id_paid, v_p, je3) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            t,
            Some(future),
            dec!(500.00),
        )
        .await;
        let _ = mark_as_paid(
            &pool,
            admin_user_id,
            id_paid,
            company_id,
            v_p,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();

        let summary = due_dates_summary(
            &pool,
            company_id,
            &InvoiceListQuery {
                contact_id: Some(contact_id),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        // Summary = unpaid only. Les 2 impayées (overdue + not-overdue), pas la payée.
        assert_eq!(summary.unpaid_count, 2);
        assert_eq!(summary.unpaid_total, dec!(150.0000));
        assert_eq!(summary.overdue_count, 1);
        assert_eq!(summary.overdue_total, dec!(100.0000));

        cleanup_invoices(&pool, &[id_overdue, id_unpaid_not_overdue, id_paid]).await;
        cleanup_journal_entries(&pool, &[je1, je2, je3]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    #[tokio::test]
    async fn test_due_dates_summary_ignores_payment_status_filter() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id_unpaid_a, _, je1) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(10.00),
        )
        .await;
        let (id_unpaid_b, _, je2) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(20.00),
        )
        .await;
        let (id_paid, v_p, je3) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(999.00),
        )
        .await;
        let _ = mark_as_paid(
            &pool,
            admin_user_id,
            id_paid,
            company_id,
            v_p,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();

        // Avec payment_status = Paid → summary doit toujours compter les impayées.
        let summary = due_dates_summary(
            &pool,
            company_id,
            &InvoiceListQuery {
                contact_id: Some(contact_id),
                payment_status: Some(PaymentStatusFilter::Paid),
                ..Default::default()
            },
        )
        .await
        .unwrap();

        assert_eq!(summary.unpaid_count, 2);
        assert_eq!(summary.unpaid_total, dec!(30.0000));

        cleanup_invoices(&pool, &[id_unpaid_a, id_unpaid_b, id_paid]).await;
        cleanup_journal_entries(&pool, &[je1, je2, je3]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// Concurrence : deux `mark_as_paid` en parallèle → un réussit, l'autre
    /// reçoit `OptimisticLockConflict`. Pool dédié à 4 connexions pour éviter
    /// le deadlock pool-timeout (chaque tx garde sa connexion jusqu'au COMMIT).
    #[tokio::test]
    async fn test_mark_as_paid_concurrent_one_succeeds_other_409() {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await
            .expect("pool connect");

        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            Some(today()),
            dec!(42.00),
        )
        .await;

        let pool_a = pool.clone();
        let pool_b = pool.clone();
        let date_a = chrono::Utc::now().naive_utc();
        let date_b = date_a;

        let (res_a, res_b) = tokio::join!(
            async move { mark_as_paid(&pool_a, admin_user_id, id, company_id, v, Some(date_a)).await },
            async move { mark_as_paid(&pool_b, admin_user_id, id, company_id, v, Some(date_b)).await },
        );

        let successes = [&res_a, &res_b].iter().filter(|r| r.is_ok()).count();
        assert_eq!(successes, 1, "exactement un mark_as_paid doit réussir");
        let conflicts = [&res_a, &res_b]
            .iter()
            .filter(|r| matches!(r, Err(DbError::OptimisticLockConflict)))
            .count();
        assert_eq!(conflicts, 1, "l'autre doit recevoir OptimisticLockConflict");

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;

        // P8 (review pass 1) : fermer le pool dédié pour libérer les 4
        // connexions (éviter l'accumulation face à `max_connections` serveur
        // sur une suite de tests avec pools multiples).
        pool.close().await;
    }
}
