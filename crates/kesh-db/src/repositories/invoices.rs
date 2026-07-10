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
//! - `update` charge l'entité initiale avec `SELECT ... FOR UPDATE` depuis
//!   KF-020 (closes #49) — divergence intentionnelle du pattern optimiste de
//!   `products.rs` car l'`update` invoice traverse un check no-op (`is_no_op_change`)
//!   qui pouvait fuiter un snapshot stale d'une tx parallèle (race KF-004
//!   résiduelle). `delete` utilise aussi `SELECT … FOR UPDATE` pour garantir
//!   l'atomicité snapshot + check statut + DELETE. Détails du lock-ordering
//!   d'`update()` dans son docblock fonction.

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
use crate::repositories::journal_entries;
use crate::util::search::{escape_boolean_ft, escape_like};

const LINE_COLUMNS: &str = "id, invoice_id, position, description, quantity, unit_price, \
    vat_rate, line_total, created_at";

/// Toujours scopé par `company_id` (anti-IDOR multi-tenant).
// pub(crate) : réutilisé par `repositories::credit_notes` (lock de la
// facture d'origine) — une seule liste de colonnes à maintenir.
pub(crate) const FIND_INVOICE_SCOPED_SQL: &str = "SELECT id, company_id, contact_id, invoice_number, \
    status, date, due_date, payment_terms, total_amount, journal_entry_id, paid_at, \
    emailed_at, emailed_to, project_id, \
    version, created_at, updated_at \
    FROM invoices WHERE id = ? AND company_id = ?";

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
        // P14 (review pass 3 A) : format RFC3339 pour round-trip ISO-8601
        // standard (parseurs externes compatibles). `paid_at` est stocké en
        // UTC naïf ; l'on affiche donc le suffixe 'Z' sans conversion.
        "paidAt": inv.paid_at.map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()),
        // P14 (review pass 3 A) : journal_entry_id tracé en audit pour
        // permettre de détecter une rupture du lien JE lors d'un mark_as_paid.
        "journalEntryId": inv.journal_entry_id,
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

    // N3 (review pass 3 B) : un filtre `payment_status` non-`All` impose
    // implicitement `status='validated'`. Si le caller passe aussi
    // `query.status = Some("draft"|"cancelled")`, la combinaison est
    // contradictoire et produirait une liste vide silencieuse. On émet
    // alors la garde `status='validated'` cohérente avec la sémantique
    // « payment_status l'emporte » et on ignore le `query.status` incohérent
    // (le caller a clairement voulu filtrer par statut de paiement).
    let payment_status_active = matches!(
        query.payment_status.unwrap_or_default(),
        PaymentStatusFilter::Paid | PaymentStatusFilter::Unpaid | PaymentStatusFilter::Overdue,
    );
    if let Some(ref status) = query.status
        && (!payment_status_active || status == "validated")
    {
        qb.push(" AND i.status = ");
        qb.push_bind(status.clone());
    }
    // sinon : ignoré silencieusement — la garde `status='validated'`
    // sera émise par le bloc `payment_status` ci-dessous.

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
            qb.push(
                " AND i.status = 'validated' AND i.paid_at IS NULL AND i.due_date < UTC_DATE()",
            );
        }
    }

    if let Some(ref search) = query.search {
        let trimmed = search.trim();
        if !trimmed.is_empty() {
            // Story 7-4 / KF-005 : `c.name` migré vers FULLTEXT BOOLEAN
            // MODE (bénéficie de l'index `ft_contacts_name`).
            // `invoice_number` et `payment_terms` restent LIKE (formats
            // structurés courts, peu volumineux).
            let escaped = escape_boolean_ft(trimmed);
            let like_pattern = format!("%{}%", escape_like(trimmed));
            if escaped.is_empty() {
                qb.push(" AND (COALESCE(i.invoice_number, '') LIKE ");
                qb.push_bind(like_pattern.clone());
                qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
                qb.push_bind(like_pattern);
                qb.push(" ESCAPE '\\\\')");
            } else {
                let bool_query = format!("{escaped}*");
                qb.push(" AND (MATCH(c.name) AGAINST(");
                qb.push_bind(bool_query);
                qb.push(" IN BOOLEAN MODE) OR COALESCE(i.invoice_number, '') LIKE ");
                qb.push_bind(like_pattern.clone());
                qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
                qb.push_bind(like_pattern);
                qb.push(" ESCAPE '\\\\')");
            }
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

    // Projet analytique optionnel (Story 19-4) : existe, même company, non
    // archivé. Sentinel `companies` puis FOR UPDATE projets (Pattern 5).
    if let Some(pid) = new.project_id
        && let Err(e) =
            super::projects::validate_taggable_in_tx(&mut tx, new.company_id, &[pid]).await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    let result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, due_date, payment_terms, \
         total_amount, project_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(new.contact_id)
    .bind(new.date)
    .bind(new.due_date)
    .bind(&new.payment_terms)
    .bind(total)
    .bind(new.project_id)
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
        NewAuditLogEntry::user(
            user_id,
            "invoice.created".to_string(),
            "invoice".to_string(),
            invoice.id,
            Some(invoice_snapshot_json(&invoice, &lines)),
        ),
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
    // P7 (review pass 3 A) : NULLs en fin de liste quel que soit le sort.
    // MariaDB trie NULL en premier en ASC et en dernier en DESC — on force
    // le comportement « NULLs last » de façon déterministe pour une
    // pagination stable (notamment InvoiceSortBy::DueDate où les factures
    // legacy sans `due_date` doivent finir en bas).
    items_qb.push(query.sort_by.as_sql_column());
    items_qb.push(" IS NULL, ");
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
/// AND paid_at IS NULL`), filtrées par les mêmes prédicats que
/// [`list_by_company_paginated`] *à l'exception* de `payment_status`
/// (volontairement ignoré — cf. doc de [`due_dates_summary`]).
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
/// **Filtres appliqués** (alignés avec spec §74) : `contact_id`, `search`,
/// `date_from`, `date_to`, `due_before` — le summary suit la même fenêtre
/// d'observation que la liste pour rester cohérent avec l'UX de filtrage.
/// **Filtres ignorés** : `payment_status` (le bascule d'onglet
/// `Paid/Unpaid/Overdue` ne doit pas remettre à zéro le KPI), `status`
/// (toujours `validated`), `paid_at IS NULL` (implicite).
///
/// P2 (review pass 3 B) : restauration des filtres temporels après que la
/// passe 2 a constaté que la spec §74 les inclut explicitement.
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
            // Story 7-4 / KF-005 : 2e callsite (cf. T6.3). Même logique
            // que list_by_company_paginated — `c.name` via FULLTEXT,
            // colonnes invoice structurées en LIKE.
            let escaped = escape_boolean_ft(trimmed);
            let like_pattern = format!("%{}%", escape_like(trimmed));
            if escaped.is_empty() {
                qb.push(" AND (COALESCE(i.invoice_number, '') LIKE ");
                qb.push_bind(like_pattern.clone());
                qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
                qb.push_bind(like_pattern);
                qb.push(" ESCAPE '\\\\')");
            } else {
                let bool_query = format!("{escaped}*");
                qb.push(" AND (MATCH(c.name) AGAINST(");
                qb.push_bind(bool_query);
                qb.push(" IN BOOLEAN MODE) OR COALESCE(i.invoice_number, '') LIKE ");
                qb.push_bind(like_pattern.clone());
                qb.push(" ESCAPE '\\\\' OR COALESCE(i.payment_terms, '') LIKE ");
                qb.push_bind(like_pattern);
                qb.push(" ESCAPE '\\\\')");
            }
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

/// Compare l'état persisté (header + lignes) au payload — `true` si aucun
/// champ métier ne diffère (KF-004 : court-circuit no-op pour ne pas bumper
/// version inutilement).
///
/// Comparaison ligne-par-ligne dans l'ordre — `line_order` est sémantique
/// (l'ordre détermine l'affichage PDF). `total_amount` n'est pas comparé
/// directement : il est dérivé des lignes via `compute_total`, donc lignes
/// identiques ⇒ total identique par construction.
fn is_no_op_change(
    before_inv: &Invoice,
    before_lines: &[InvoiceLine],
    changes: &InvoiceUpdate,
) -> bool {
    if before_inv.contact_id != changes.contact_id
        || before_inv.date != changes.date
        || before_inv.due_date != changes.due_date
        || before_inv.payment_terms != changes.payment_terms
        || before_inv.project_id != changes.project_id
    {
        return false;
    }
    if before_lines.len() != changes.lines.len() {
        return false;
    }
    before_lines.iter().zip(changes.lines.iter()).all(|(b, c)| {
        b.description == c.description
            && b.quantity == c.quantity
            && b.unit_price == c.unit_price
            && b.vat_rate == c.vat_rate
    })
}

/// Met à jour une facture brouillon : replace-all sur les lignes +
/// recalcul `total_amount` + audit wrapper `{before, after}`.
///
/// **Ordre des locks (canonique, KF-020 #49 closed)** :
///
/// 1. `invoices` : `SELECT ... FOR UPDATE` sur la facture (scope `company_id`),
///    fenêtre [BEGIN..COMMIT/ROLLBACK].
/// 2. `invoice_lines` : SELECT (lecture sans lock — `before_lines`) puis
///    DELETE+INSERT (locks ligne par ligne via FK CASCADE), fenêtre
///    [post-no-op-check..pré-UPDATE].
/// 3. `invoices` : UPDATE conditionnel (version-check), fenêtre [final].
/// 4. `audit_log` : INSERT.
///
/// Toutes les acquisitions séquentielles sur le même row d'`invoices` (id,
/// company_id), donc aucun deadlock self-induit. Si une autre tx prend les
/// mêmes locks dans le même ordre (ex. `delete()`, `validate_invoice()`,
/// `mark_as_paid()`), la sérialisation par X-lock invoices garantit l'absence
/// de deadlock inter-transactions sur les invoices/invoice_lines.
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

    // Projet analytique (Story 19-4) : validé UNIQUEMENT si la valeur change
    // (grandfathering du tag inchangé — un projet archivé après la pose du tag
    // ne bloque pas l'édition des autres champs du brouillon, leçon 19-2).
    // Pré-lecture NON verrouillée scopée company, AVANT le FOR UPDATE facture,
    // pour respecter l'ordre de verrous global companies → projects → invoices
    // (le sentinel du helper ne doit jamais être pris en détenant la ligne
    // facture — inversion ABBA avec create). Race bénigne : la valeur re-lue
    // sous lock plus bas fait foi pour le no-op check.
    let prior_project_id: Option<i64> =
        sqlx::query_scalar("SELECT project_id FROM invoices WHERE id = ? AND company_id = ?")
            .bind(id)
            .bind(company_id)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_db_error)?
            .flatten();
    if let Some(pid) = changes.project_id
        && changes.project_id != prior_project_id
        && let Err(e) = super::projects::validate_taggable_in_tx(&mut tx, company_id, &[pid]).await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    // KF-020 (closes #49) : SELECT … FOR UPDATE pour fermer la race no-op
    // résiduelle KF-004. Sous REPEATABLE READ + plain SELECT, une tx parallèle
    // qui commit entre notre BEGIN et le no-op check pouvait laisser fuiter un
    // snapshot stale en `200 OK`. Avec FOR UPDATE, le X-lock attend la fin de
    // la tx concurrente et la lecture post-lock retourne v=N+1 → la
    // version-check applicative déclenche le 409 attendu.
    let before_invoice_opt =
        sqlx::query_as::<_, Invoice>(&format!("{FIND_INVOICE_SCOPED_SQL} FOR UPDATE"))
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

    // KF-004 : court-circuit no-op AVANT le DELETE/INSERT.
    // CRITIQUE : ce check DOIT précéder le DELETE — sinon les IDs des lignes
    // seraient détruits puis régénérés, polluant le test no-op et bumpant
    // inutilement les compteurs AUTO_INCREMENT.
    // KF-020 closed (#49) : le SELECT FOR UPDATE ci-dessus garantit que
    // `before_invoice` ne peut pas être un snapshot stale d'une tx parallèle —
    // si une autre tx était en train de modifier l'invoice, notre lock attend
    // sa fin avant de lire. La version-check applicative ci-dessus rejette
    // alors v=N+1 != expected_version=N avec 409. La race no-op résiduelle
    // décrite dans Story 7-3 §race-condition est donc fermée pour invoices.
    if is_no_op_change(&before_invoice, &before_lines, &changes) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok((before_invoice, before_lines));
    }

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
         total_amount = ?, project_id = ?, version = version + 1 \
         WHERE id = ? AND company_id = ? AND version = ? AND status = 'draft'",
    )
    .bind(changes.contact_id)
    .bind(changes.date)
    .bind(changes.due_date)
    .bind(&changes.payment_terms)
    .bind(total)
    .bind(changes.project_id)
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
        NewAuditLogEntry::user(
            user_id,
            "invoice.updated".to_string(),
            "invoice".to_string(),
            id,
            Some(audit_details),
        ),
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
         payment_terms, total_amount, journal_entry_id, paid_at, emailed_at, emailed_to, \
         project_id, version, created_at, updated_at \
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
        // Brouillon : suppression inchangée (aucune écriture comptable).
        Some(inv) if inv.status == "draft" => inv,
        // Facture validée (#219) : suppression définitive AVEC garde-fous.
        // L'écriture comptable liée est supprimée plus bas (après la facture,
        // car FK `journal_entry_id ON DELETE RESTRICT`), et la garde
        // exercice-clos est assurée par `journal_entries::delete_in_tx`.
        Some(inv) if inv.status == "validated" => {
            // Garde 1 : facture payée → refus (un règlement/rapprochement
            // pointerait dans le vide).
            if inv.paid_at.is_some() {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::IllegalStateTransition(
                    "impossible de supprimer une facture payée — annuler le règlement d'abord"
                        .into(),
                ));
            }
            // Garde 2 : facture créditée par un avoir → refus. La FK
            // `credit_notes.invoice_id ON DELETE RESTRICT` est le backstop en
            // base ; ce pré-check donne un message métier propre.
            let credited: Option<(i64,)> =
                sqlx::query_as("SELECT id FROM credit_notes WHERE invoice_id = ? LIMIT 1")
                    .bind(id)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(map_db_error)?;
            if credited.is_some() {
                tx.rollback().await.map_err(map_db_error)?;
                return Err(DbError::IllegalStateTransition(
                    "impossible de supprimer une facture créditée par un avoir".into(),
                ));
            }
            inv
        }
        // Tout autre statut (ex. `cancelled`) reste non supprimable.
        Some(inv) => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::IllegalStateTransition(format!(
                "impossible de supprimer une facture de statut '{}'",
                inv.status
            )));
        }
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
        NewAuditLogEntry::user(
            user_id,
            "invoice.deleted".to_string(),
            "invoice".to_string(),
            id,
            Some(snapshot),
        ),
    )
    .await
    {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(e);
    }

    // #219 — facture validée : supprimer l'écriture comptable liée DANS la même
    // transaction (après la facture, car FK `journal_entry_id ON DELETE
    // RESTRICT`). `delete_in_tx` applique la garde exercice-clos (→
    // `FiscalYearClosed`, rollback atomique de tout) et journalise
    // `journal_entry.deleted`. Sur Err, le drop de `tx` rollback la suppression
    // de la facture aussi.
    if let Some(je_id) = current.journal_entry_id
        && let Err(e) = journal_entries::delete_in_tx(&mut tx, company_id, je_id, user_id).await
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

/// Génère les lignes d'écriture comptable de la validation d'une facture de
/// vente, **TVA due comprise** (Story 18-1b, axe b).
///
/// Produit, dans cet **ordre canonique** (le `line_order` est posé séquentiellement
/// par [`journal_entries::create_in_tx`]) :
/// - (0) **débit créance** (compte `receivable`) = `total_ht + total_vat` (TTC) ;
/// - (1) **crédit produit** (compte `revenue`) = `total_ht` (HT, = `invoices.total_amount`, DC9) ;
/// - (2..N) **crédit TVA due** (compte `vat_payable`), **une ligne par taux** dont le
///   montant agrégé est strictement `> 0` (DC2 + F-OPUS-1), triées par taux croissant
///   (itération `BTreeMap` ASC).
///
/// `total_vat = Σ_taux Σ_lignes line_vat_amount(line_total, vat_rate)`, où chaque montant
/// est arrondi half-up **par ligne** (DC7, cf. [`kesh_core::accounting::vat::line_vat_amount`]).
///
/// # Équilibre par construction
///
/// `total_vat` est calculé sur **tous** les taux (avant le filtre `> 0`). Comme
/// `line_total >= 0` (hypothèse F-OPUS-2), chaque montant agrégé par taux est `>= 0`, donc
/// un taux filtré (montant `== 0`) contribue 0 au débit créance ET 0 aux crédits. Le débit
/// créance somme exactement les **mêmes** montants arrondis par ligne que la somme des
/// crédits (produit + lignes TVA) → `Σ debit == Σ credit` (vérifié par `create_in_tx`).
///
/// # Erreurs
///
/// - [`DbError::ConfigurationRequired`] (`"default_vat_payable_account_id"`) si au moins
///   une ligne TVA due doit être émise (`total_vat > 0`) mais `vat_payable_account_id` est
///   `None`. Une facture sans TVA > 0 (tout à taux 0/exempt) **n'exige pas** ce compte.
///
/// # Hypothèse (F-OPUS-2) — avoirs hors-scope
///
/// Suppose `line.line_total >= 0` (factures de vente). Les avoirs / notes de crédit
/// (Epic 12) devront passer par une **contre-passation** (swap débit↔crédit), PAS par un
/// montant négatif (les contraintes `chk_jel_*_nonneg` l'interdisent) — **ne pas réutiliser
/// ce helper tel quel** pour les avoirs.
fn generate_invoice_journal_lines(
    lines: &[InvoiceLine],
    receivable_account_id: i64,
    revenue_account_id: i64,
    vat_payable_account_id: Option<i64>,
) -> Result<Vec<crate::entities::NewJournalEntryLine>, DbError> {
    use crate::entities::NewJournalEntryLine;
    use kesh_core::accounting::vat::line_vat_amount;
    use std::collections::BTreeMap;

    let mut total_ht = Decimal::ZERO;
    // Montant TVA agrégé par taux. `BTreeMap` : itération ASC native (ordre AC6) +
    // `Decimal` Eq/Hash insensible à l'échelle (`8.1` et `8.10` = même clé).
    let mut vat_by_rate: BTreeMap<Decimal, Decimal> = BTreeMap::new();

    for line in lines {
        debug_assert!(
            line.line_total >= Decimal::ZERO,
            "F-OPUS-2 violé : line_total < 0 (ligne {}) — les avoirs passent par \
             contre-passation, pas ce helper",
            line.id
        );
        total_ht += line.line_total;
        let vat = line_vat_amount(line.line_total, line.vat_rate);
        *vat_by_rate.entry(line.vat_rate).or_insert(Decimal::ZERO) += vat;
    }

    // Somme des montants déjà arrondis par ligne (DC7) — NE PAS réarrondir.
    let total_vat: Decimal = vat_by_rate.values().copied().sum();

    let mut entry_lines = Vec::with_capacity(2 + vat_by_rate.len());
    // (0) Débit créance TTC = HT + TVA.
    entry_lines.push(NewJournalEntryLine {
        account_id: receivable_account_id,
        debit: total_ht + total_vat,
        credit: Decimal::ZERO,
        project_id: None,
    });
    // (1) Crédit produit HT.
    entry_lines.push(NewJournalEntryLine {
        account_id: revenue_account_id,
        debit: Decimal::ZERO,
        credit: total_ht,
        project_id: None,
    });

    // (2..N) Crédit TVA due par taux > 0. La config du compte TVA due n'est requise
    // QUE si une ligne doit réellement être émise (F-OPUS-1 + AC5).
    if total_vat > Decimal::ZERO {
        let vat_account = vat_payable_account_id.ok_or_else(|| {
            DbError::ConfigurationRequired("default_vat_payable_account_id".into())
        })?;
        for amount in vat_by_rate.values() {
            if *amount > Decimal::ZERO {
                entry_lines.push(NewJournalEntryLine {
                    account_id: vat_account,
                    debit: Decimal::ZERO,
                    credit: *amount,
                    project_id: None,
                });
            }
        }
    }

    Ok(entry_lines)
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
    use crate::entities::{Journal, NewJournalEntry};
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

        // (1 bis) Story 19-4 — re-validation du projet analytique AU POSTING :
        // le projet peut avoir été archivé entre le brouillon et la validation,
        // et toute nouvelle entrée au grand livre doit être sur projet actif
        // (équivalence avec 19-3 où création fournisseur = posting immédiat).
        // SELECT simple SANS verrou, volontairement : on détient déjà la ligne
        // facture (FOR UPDATE ci-dessus) — prendre le sentinel companies ici
        // créerait une inversion ABBA avec create/update (qui prennent
        // sentinel AVANT la ligne facture). La race d'archivage résiduelle
        // (fenêtre ms) est la même dette LOW acceptée qu'en 19-3 (LOW-1).
        // Existence/company déjà garanties (FK + validation à la pose du tag).
        if let Some(pid) = invoice_before.project_id {
            let archived: Option<bool> =
                sqlx::query_scalar("SELECT archived FROM projects WHERE id = ? AND company_id = ?")
                    .bind(pid)
                    .bind(company_id)
                    .fetch_optional(&mut *tx)
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

        // (7) Créer l'écriture comptable dans la même tx — TVA due comprise
        // (Story 18-1b). La génération des lignes (créance TTC + produit HT + N
        // lignes TVA due par taux) est centralisée dans le helper, pour qu'Epic 16
        // (compte produit par ligne) s'y branche sans 2e refactor de cette fonction.
        let journal: Journal = settings.default_sales_journal;

        let entry_lines = generate_invoice_journal_lines(
            &lines_before,
            receivable_account_id,
            revenue_account_id,
            settings.default_vat_payable_account_id,
        )?;

        let je = journal_entries::create_in_tx(
            &mut tx,
            fy.id,
            user_id,
            NewJournalEntry {
                company_id,
                entry_date: invoice_before.date,
                journal,
                description: entry_description,
                // Story 19-4 : le tag document-level de la facture est propagé
                // sur toutes les lignes de l'écriture de vente (via le bind
                // `line.project_id.or(new.project_id)` de create_in_tx).
                project_id: invoice_before.project_id,
                lines: entry_lines,
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
            NewAuditLogEntry::user(
                user_id,
                "invoice.validated".to_string(),
                "invoice".to_string(),
                invoice_id,
                Some(audit_details),
            ),
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
) -> Result<(Invoice, Vec<InvoiceLine>), DbError> {
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

        // P2 (review pass 2) : rejeter `unmark_paid` sur une facture jamais
        // marquée payée. Sans ce guard, l'UPDATE `SET paid_at = NULL` peut
        // retourner `rows_affected = 0` (MariaDB) → faux
        // `OPTIMISTIC_LOCK_CONFLICT`, ou bump de version + audit
        // `invoice.unpaid` spurious sur une transition logiquement invalide.
        if paid_at.is_none() && before.paid_at.is_none() {
            return Err(DbError::InvalidInput("alreadyUnpaid".to_string()));
        }

        // Validation calendaire paid_at vs invoice.date (défense en profondeur —
        // le handler pré-valide aussi). Comparaison sur la date (pas le datetime)
        // pour éviter qu'un paiement à 00:30 UTC soit rejeté comme « antérieur »
        // à une facture du même jour.
        // P2 (review pass 1) : tolérance 1 jour pour absorber l'écart entre
        // `paid_at` stocké en UTC naïf et `invoice.date` en date métier locale
        // (jusqu'à +/- 2h en CET/CEST). Sans cette tolérance, un paiement
        // saisi à 00:30 CET le jour même de la facture serait rejeté.
        if let Some(pa) = paid_at
            && pa.date() < before.date - Duration::days(1)
        {
            return Err(DbError::InvalidInput("paidAtBeforeInvoiceDate".to_string()));
        }
        // Pas de borne supérieure : `paid_at` représente la date d'exécution
        // bancaire, qui peut être dans le futur (ordre de virement programmé,
        // décalage week-end/jour férié réécrit par la banque). Voir N2 review
        // pass 3 B + AC#8 à amender dans la spec story 5.4.

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
            NewAuditLogEntry::user(
                user_id,
                action.to_string(),
                "invoice".to_string(),
                id,
                Some(audit_details),
            ),
        )
        .await?;

        // M1 (review pass 1 G2) : retourner lignes issues de la même tx pour
        // éviter une race DELETE/UPDATE entre mark_as_paid et le re-fetch du
        // handler (lignes inchangées par mark_as_paid → réutilise lines_before).
        Ok((after, lines_before))
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

/// Marque une facture « envoyée par e-mail » (Story 20-3b1, décision #16
/// epic-20) : pose `emailed_at = NOW(6)` + `emailed_to` (snapshot du
/// destinataire réellement utilisé) et écrit l'audit `invoice.emailed`
/// (avec `to` + `subject`) dans la même transaction.
///
/// **Pas de verrou optimiste** : l'envoi ne modifie pas l'état comptable ;
/// une course entre deux envois = deux renvois légitimes, tous deux audités
/// (le renvoi écrase `emailed_at`/`emailed_to`). Défense en profondeur : ne
/// s'applique qu'à une facture `validated` (le chemin d'envoi l'a déjà
/// garanti via le rendu PDF).
pub async fn mark_emailed(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    to: &str,
    subject: &str,
    user_id: i64,
    actor_api_key_id: Option<i64>,
) -> Result<Invoice, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    let result = async {
        let rows = sqlx::query(
            "UPDATE invoices SET emailed_at = NOW(6), emailed_to = ? \
             WHERE id = ? AND company_id = ? AND status = 'validated'",
        )
        .bind(to)
        .bind(id)
        .bind(company_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();

        if rows == 0 {
            // Facture absente / autre company / non-validée — le chemin
            // d'envoi a déjà validé tout ça ; refactor incomplet sinon.
            return Err(DbError::NotFound);
        }

        let after = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
            .bind(id)
            .bind(company_id)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_db_error)?;

        audit_log::insert_in_tx(
            &mut tx,
            NewAuditLogEntry::for_actor(
                user_id,
                actor_api_key_id,
                "invoice.emailed".to_string(),
                "invoice".to_string(),
                id,
                Some(serde_json::json!({ "to": to, "subject": subject })),
            ),
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
/// Tri : `due_date ASC, id ASC` (NULLs last).
///
/// # Retour
///
/// `(rows, truncated)` — `truncated = true` ssi le plafond de défense
/// (50_000) a silencieusement rabaissé le `max_rows` demandé par le caller.
/// Dans ce cas le handler doit alerter l'utilisateur plutôt qu'exploiter
/// un résultat partiel comme s'il était complet.
pub async fn list_for_export(
    pool: &MySqlPool,
    company_id: i64,
    query: &InvoiceListQuery,
    max_rows: i64,
) -> Result<(Vec<InvoiceListItem>, bool), DbError> {
    // P3 (review pass 1) : plafond absolu 50_000 en défense en profondeur.
    // La règle métier « > 10_000 → 400 » reste appliquée côté handler ;
    // ce clamp évite un scan complet si un caller passe une valeur aberrante.
    //
    // F3 (review pass 2) : borne basse à 1 (et non 0) pour éviter qu'un appel
    // avec `max_rows <= 0` se traduise silencieusement en `LIMIT 0` (retour
    // vide interprété à tort comme « aucune facture »).
    //
    // P3 (review pass 3 A) : `truncated` remonte au caller quand le clamp
    // haut a réellement rabaissé l'intent — plus de troncature silencieuse.
    const HARD_CAP: i64 = 50_000;
    let truncated = max_rows > HARD_CAP;
    let effective = max_rows.clamp(1, HARD_CAP);

    let mut items_qb: QueryBuilder<sqlx::MySql> = QueryBuilder::new(
        "SELECT i.id, i.company_id, i.contact_id, c.name AS contact_name, \
         i.invoice_number, i.status, i.date, i.due_date, i.payment_terms, \
         i.total_amount, i.paid_at, i.version, i.created_at, i.updated_at \
         FROM invoices i INNER JOIN contacts c ON c.id = i.contact_id",
    );
    push_where_clauses(&mut items_qb, company_id, query);
    // P13 (review pass 3 A) : défense en profondeur — l'export ne doit
    // jamais fuiter de drafts même si le handler oublie le filtre `status`.
    items_qb.push(" AND i.status = 'validated'");
    // NULLs last pour cohérence avec list_by_company_paginated (P7).
    items_qb.push(" ORDER BY i.due_date IS NULL, i.due_date ASC, i.id ASC LIMIT ");
    items_qb.push_bind(effective);

    let rows: Vec<InvoiceListItem> = items_qb
        .build_query_as()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    Ok((rows, truncated))
}

/// Story 9-2b T3.2.4 — Liste **toutes** les factures d'une company sans filtre
/// `status`, pour l'export global ZIP (souveraineté : drafts + validated +
/// paid + archived).
///
/// Distincte de [`list_for_export`] (qui force `status = 'validated'`) et de
/// [`list_by_company_paginated`]. Tri stable `id ASC`.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<Invoice>, DbError> {
    sqlx::query_as::<_, Invoice>(
        "SELECT id, company_id, contact_id, invoice_number, status, date, due_date, \
         payment_terms, total_amount, journal_entry_id, paid_at, emailed_at, emailed_to, \
         project_id, version, created_at, updated_at \
         FROM invoices \
         WHERE company_id = ? \
         ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Story 9-2b T3.2.5 — Liste **toutes** les lignes de facture d'une company
/// via un **single-query JOIN** (anti-N+1).
///
/// `InvoiceLine` n'a pas de `company_id` direct ground-truth
/// (`entities/invoice.rs:41-51`) — le scoping multi-tenant passe par
/// `invoices.company_id` via `JOIN`.
///
/// Tri `invoice_id, position` pour stabilité d'export (les lignes d'une même
/// facture sont contiguës et ordonnées).
pub async fn list_all_lines_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<InvoiceLine>, DbError> {
    sqlx::query_as::<_, InvoiceLine>(
        "SELECT il.id, il.invoice_id, il.position, il.description, il.quantity, \
         il.unit_price, il.vat_rate, il.line_total, il.created_at \
         FROM invoice_lines il \
         JOIN invoices i ON il.invoice_id = i.id \
         WHERE i.company_id = ? \
         ORDER BY il.invoice_id, il.position",
    )
    .bind(company_id)
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
    use crate::entities::NewJournalEntryLine;
    use crate::entities::contact::{ContactType, NewContact};
    use crate::repositories::contacts;
    use rust_decimal_macros::dec;
    use sqlx::QueryBuilder;
    use uuid::Uuid;

    // P9 (review pass 3 A) : unit tests pure `escape_like`. Sans Pool DB —
    // valide simplement que les métacaractères LIKE sont neutralisés avant
    // d'être passés à la requête paramétrée (complément aux tests d'intégration).
    #[test]
    fn test_escape_like_percent() {
        assert_eq!(escape_like("5%"), "5\\%");
    }

    #[test]
    fn test_escape_like_underscore() {
        assert_eq!(escape_like("a_b"), "a\\_b");
    }

    #[test]
    fn test_escape_like_backslash() {
        assert_eq!(escape_like("c:\\path"), "c:\\\\path");
    }

    #[test]
    fn test_escape_like_combined() {
        // Ordre critique : backslash d'abord sinon on double-échappe
        // les backslashes introduits par % et _.
        assert_eq!(escape_like("100%_done\\"), "100\\%\\_done\\\\");
    }

    #[test]
    fn test_escape_like_passthrough() {
        assert_eq!(escape_like("plain text"), "plain text");
    }

    // -----------------------------------------------------------------------
    // Tests unitaires `generate_invoice_journal_lines` (Story 18-1b, T-B3) —
    // sans DB : le helper est sans I/O, testable sur le `Vec` retourné.
    // -----------------------------------------------------------------------

    /// Construit une `InvoiceLine` minimale pour les tests du helper TVA
    /// (seuls `line_total` et `vat_rate` importent).
    fn make_line(line_total: Decimal, vat_rate: Decimal) -> InvoiceLine {
        InvoiceLine {
            id: 1,
            invoice_id: 1,
            position: 1,
            description: "ligne test".into(),
            quantity: dec!(1),
            unit_price: line_total,
            vat_rate,
            line_total,
            created_at: NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        }
    }

    const RECEIVABLE: i64 = 1100;
    const REVENUE: i64 = 3000;
    const VAT_DUE: i64 = 2200;

    fn sum_debit(lines: &[NewJournalEntryLine]) -> Decimal {
        lines.iter().map(|l| l.debit).sum()
    }
    fn sum_credit(lines: &[NewJournalEntryLine]) -> Decimal {
        lines.iter().map(|l| l.credit).sum()
    }

    /// (a) Facture mono-taux 8.1 % → 3 lignes (créance TTC, produit HT, 1×2200) + équilibre.
    #[test]
    fn gen_lines_single_rate() {
        let lines = [make_line(dec!(1000.00), dec!(8.10))];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(je.len(), 3, "créance + produit + 1 ligne TVA");
        // (0) créance TTC = 1000 + 81 = 1081.
        assert_eq!(je[0].account_id, RECEIVABLE);
        assert_eq!(je[0].debit, dec!(1081.00));
        assert_eq!(je[0].credit, Decimal::ZERO);
        // (1) produit HT = 1000.
        assert_eq!(je[1].account_id, REVENUE);
        assert_eq!(je[1].credit, dec!(1000.00));
        // (2) TVA due = 81.
        assert_eq!(je[2].account_id, VAT_DUE);
        assert_eq!(je[2].credit, dec!(81.00));
        assert_eq!(sum_debit(&je), sum_credit(&je), "équilibre");
    }

    /// (b) Facture entièrement à taux 0 → 2 lignes, AUCUNE ligne 2200 (F-OPUS-1).
    #[test]
    fn gen_lines_zero_rate_no_vat_line() {
        let lines = [
            make_line(dec!(500.00), dec!(0)),
            make_line(dec!(250.00), dec!(0)),
        ];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(je.len(), 2, "créance + produit, aucune TVA");
        assert!(
            je.iter().all(|l| l.account_id != VAT_DUE),
            "aucune ligne sur le compte TVA due"
        );
        assert_eq!(je[0].debit, dec!(750.00), "créance = HT (pas de TVA)");
        assert_eq!(je[1].credit, dec!(750.00));
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

    /// (c) Multi-taux 8.1 % + 2.6 % → 2 lignes 2200 distinctes, ordonnées par taux croissant.
    #[test]
    fn gen_lines_multi_rate_sorted_ascending() {
        // Volontairement ordre d'entrée inversé (8.1 avant 2.6) pour vérifier le tri.
        let lines = [
            make_line(dec!(1000.00), dec!(8.10)),
            make_line(dec!(500.00), dec!(2.60)),
        ];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(je.len(), 4, "créance + produit + 2 lignes TVA");
        // total TVA = 81.00 (8.1%) + 13.00 (2.6%) = 94 ; créance = 1500 + 94 = 1594.
        assert_eq!(je[0].debit, dec!(1594.00));
        assert_eq!(je[1].credit, dec!(1500.00));
        // Lignes TVA triées par taux croissant : 2.60 (13.00) AVANT 8.10 (81.00).
        assert_eq!(
            je[2].credit,
            dec!(13.00),
            "2.6 % en premier (taux croissant)"
        );
        assert_eq!(je[3].credit, dec!(81.00), "8.1 % en second");
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

    /// (d) Multi-taux 8.1 % + 0 % → 1 SEULE ligne 2200 (taux > 0 uniquement).
    #[test]
    fn gen_lines_mixed_positive_and_zero_rate() {
        let lines = [
            make_line(dec!(1000.00), dec!(8.10)),
            make_line(dec!(400.00), dec!(0)),
        ];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(
            je.len(),
            3,
            "créance + produit + 1 ligne TVA (taux 0 exclu)"
        );
        // créance = 1400 HT + 81 TVA = 1481 ; produit = 1400.
        assert_eq!(je[0].debit, dec!(1481.00));
        assert_eq!(je[1].credit, dec!(1400.00));
        assert_eq!(je[2].credit, dec!(81.00));
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

    /// (e) Taux > 0 mais base infime → arrondi 0.00 → AUCUNE ligne 2200 (F-OPUS-1).
    #[test]
    fn gen_lines_rate_rounds_to_zero() {
        // line_vat_amount(0.01, 8.1) = round(0.00081) = 0.00.
        let lines = [make_line(dec!(0.01), dec!(8.10))];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(je.len(), 2, "TVA arrondie à 0 → pas de ligne 2200");
        assert_eq!(je[0].debit, dec!(0.01), "créance = HT (TVA nulle)");
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

    /// (e2) DC7 — somme des montants arrondis PAR LIGNE (pas arrondi de la base agrégée).
    #[test]
    fn gen_lines_dc7_sum_of_per_line_rounding() {
        // 2 lignes 0.07 @ 8.1 % : round(0.00567) = 0.01 chacune → agrégat 0.02.
        // (Un arrondi de la base agrégée 0.14 @ 8.1 % donnerait round(0.01134) = 0.01 — incorrect.)
        let lines = [
            make_line(dec!(0.07), dec!(8.10)),
            make_line(dec!(0.07), dec!(8.10)),
        ];
        let je =
            generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, Some(VAT_DUE)).unwrap();
        assert_eq!(je.len(), 3, "une seule ligne TVA (même taux)");
        assert_eq!(
            je[2].credit,
            dec!(0.02),
            "0.01 + 0.01 (somme par ligne, DC7)"
        );
        // créance = 0.14 + 0.02 = 0.16.
        assert_eq!(je[0].debit, dec!(0.16));
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

    /// (f) TVA > 0 mais compte TVA due NON configuré (None) → ConfigurationRequired.
    #[test]
    fn gen_lines_config_required_when_vat_account_missing() {
        let lines = [make_line(dec!(1000.00), dec!(8.10))];
        let err = generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, None).unwrap_err();
        match err {
            DbError::ConfigurationRequired(field) => {
                assert_eq!(field, "default_vat_payable_account_id");
            }
            other => panic!("attendu ConfigurationRequired, reçu {other:?}"),
        }
    }

    /// (f-bis) Compte TVA due None mais facture SANS TVA → pas d'erreur (compte non requis).
    #[test]
    fn gen_lines_no_vat_account_ok_when_no_vat() {
        let lines = [make_line(dec!(1000.00), dec!(0))];
        let je = generate_invoice_journal_lines(&lines, RECEIVABLE, REVENUE, None).unwrap();
        assert_eq!(je.len(), 2, "pas de TVA → compte TVA due non requis");
        assert_eq!(sum_debit(&je), sum_credit(&je));
    }

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
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: Some("30 jours net".into()),
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
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
            project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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

    /// #219 — happy path : une facture validée, impayée, en exercice ouvert
    /// est supprimée définitivement AVEC son écriture comptable liée, et les
    /// deux suppressions sont audit-loggées.
    #[tokio::test]
    async fn test_delete_validated_unpaid_open_fy_removes_invoice_and_je() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, _v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            None,
            dec!(10.00),
        )
        .await;

        delete(&pool, company_id, id, admin_user_id).await.unwrap();

        // La facture a disparu.
        let inv: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoices WHERE id = ?")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(inv.is_none(), "la facture validée doit être supprimée");

        // L'écriture comptable liée a disparu (livres équilibrés, pas d'orphelin).
        let je: Option<(i64,)> = sqlx::query_as("SELECT id FROM journal_entries WHERE id = ?")
            .bind(je_id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(
            je.is_none(),
            "l'écriture comptable liée doit être supprimée"
        );

        // Les deux suppressions sont tracées.
        let inv_audit = audit_log::find_by_entity(&pool, "invoice", id, 10)
            .await
            .unwrap();
        assert!(
            inv_audit.iter().any(|e| e.action == "invoice.deleted"),
            "audit invoice.deleted attendu"
        );
        let je_audit = audit_log::find_by_entity(&pool, "journal_entry", je_id, 10)
            .await
            .unwrap();
        assert!(
            je_audit.iter().any(|e| e.action == "journal_entry.deleted"),
            "audit journal_entry.deleted attendu"
        );

        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// #219 garde 1 — une facture validée **payée** refuse la suppression
    /// (un règlement pointerait dans le vide). Rien n'est supprimé.
    #[tokio::test]
    async fn test_delete_validated_paid_is_rejected() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, _v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            None,
            dec!(10.00),
        )
        .await;
        sqlx::query("UPDATE invoices SET paid_at = ? WHERE id = ?")
            .bind(chrono::Utc::now().naive_utc())
            .bind(id)
            .execute(&pool)
            .await
            .unwrap();

        let err = delete(&pool, company_id, id, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        // Rollback atomique : facture + écriture toujours présentes.
        let inv: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoices WHERE id = ?")
            .bind(id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(inv.is_some(), "facture payée non supprimée");

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// #219 garde CO 958f — une facture validée dont l'écriture est dans un
    /// exercice **clos** refuse la suppression. Rien n'est supprimé.
    #[tokio::test]
    async fn test_delete_validated_in_closed_fy_is_rejected() {
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
                project_id: None,
            },
        )
        .await
        .unwrap();

        // Exercice clos dédié + écriture stub liée.
        let res = sqlx::query(
            "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
             VALUES (?, 'Closed219', '2019-01-01', '2019-12-31', 'Closed')",
        )
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
        let closed_fy = res.last_insert_id() as i64;
        let je_id = insert_stub_journal_entry(&pool, company_id, closed_fy).await;
        sqlx::query(
            "UPDATE invoices SET status = 'validated', journal_entry_id = ?, \
             version = version + 1 WHERE id = ?",
        )
        .bind(je_id)
        .bind(inv.id)
        .execute(&pool)
        .await
        .unwrap();

        let err = delete(&pool, company_id, inv.id, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::FiscalYearClosed));

        // Rollback atomique : facture toujours présente.
        let still: Option<(i64,)> = sqlx::query_as("SELECT id FROM invoices WHERE id = ?")
            .bind(inv.id)
            .fetch_optional(&pool)
            .await
            .unwrap();
        assert!(still.is_some(), "facture en exercice clos non supprimée");

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
            .bind(closed_fy)
            .execute(&pool)
            .await
            .ok();
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// #219 garde avoir — une facture validée **créditée par un avoir** refuse
    /// la suppression (pré-check métier avant le backstop FK RESTRICT).
    #[tokio::test]
    async fn test_delete_validated_credited_by_avoir_is_rejected() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        let (id, _v, je_id) = create_and_validate(
            &pool,
            company_id,
            admin_user_id,
            contact_id,
            today(),
            None,
            dec!(10.00),
        )
        .await;
        let cn = sqlx::query(
            "INSERT INTO credit_notes (company_id, contact_id, invoice_id, status, date) \
             VALUES (?, ?, ?, 'draft', CURDATE())",
        )
        .bind(company_id)
        .bind(contact_id)
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
        let cn_id = cn.last_insert_id() as i64;

        let err = delete(&pool, company_id, id, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        // Cleanup : l'avoir référence la facture (RESTRICT) → le supprimer d'abord.
        sqlx::query("DELETE FROM credit_notes WHERE id = ?")
            .bind(cn_id)
            .execute(&pool)
            .await
            .ok();
        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// #219 — un statut ni `draft` ni `validated` (ex. `cancelled`) reste
    /// non supprimable.
    #[tokio::test]
    async fn test_delete_rejects_cancelled() {
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
                project_id: None,
            },
        )
        .await
        .unwrap();
        sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
            .bind(inv.id)
            .execute(&pool)
            .await
            .unwrap();

        let err = delete(&pool, company_id, inv.id, admin_user_id)
            .await
            .unwrap_err();
        assert!(matches!(err, DbError::IllegalStateTransition(_)));

        cleanup_invoices(&pool, &[inv.id]).await;
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                    project_id: None,
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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
        let (after, lines_after) = mark_as_paid(&pool, admin_user_id, id, company_id, v, paid_at)
            .await
            .unwrap();
        assert!(after.paid_at.is_some());
        assert_eq!(after.version, v + 1);
        // M1 (review pass 1 G2) : mark_as_paid retourne maintenant les lignes
        // de la même transaction — pas de re-fetch post-commit côté handler.
        assert!(!lines_after.is_empty());

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
                project_id: None,
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

        let (after1, _) = mark_as_paid(
            &pool,
            admin_user_id,
            id,
            company_id,
            v,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();
        let (after2, _) = mark_as_paid(&pool, admin_user_id, id, company_id, after1.version, None)
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

    /// P4 (review pass 2) : spec T2.4 §715 — le statut `cancelled` doit
    /// aussi être rejeté avec `IllegalStateTransition`. Le test `rejects_draft`
    /// ne couvre que le cas `draft` ; la guard `status != "validated"` dans
    /// le code couvre tous les autres statuts, mais il faut un test pour
    /// capter toute régression si la condition devient un whitelist.
    #[tokio::test]
    async fn test_mark_as_paid_rejects_cancelled() {
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
                project_id: None,
            },
        )
        .await
        .unwrap();

        // Force le statut à 'cancelled' via SQL direct (pas d'API de cancel
        // dans v0.1 ; sera ajoutée Epic 10 avoirs).
        sqlx::query("UPDATE invoices SET status = 'cancelled' WHERE id = ?")
            .bind(inv.id)
            .execute(&pool)
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

    /// P2 (review pass 2) : `unmark_paid` sur une facture validée jamais
    /// marquée payée retourne `InvalidInput("alreadyUnpaid")` (transition
    /// logiquement invalide) au lieu d'un faux `OPTIMISTIC_LOCK_CONFLICT`
    /// ou d'un audit `invoice.unpaid` spurious.
    #[tokio::test]
    async fn test_unmark_never_paid_returns_invalid_input() {
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
            dec!(10.00),
        )
        .await;

        let err = mark_as_paid(&pool, admin_user_id, id, company_id, v, None)
            .await
            .unwrap_err();
        match err {
            DbError::InvalidInput(code) => assert_eq!(code, "alreadyUnpaid"),
            other => panic!("attendu InvalidInput(alreadyUnpaid), reçu {other:?}"),
        }

        // Vérifie qu'aucune entrée audit `invoice.unpaid` n'a été écrite
        // (la guard doit bloquer AVANT l'insert audit).
        let entries = audit_log::find_by_entity(&pool, "invoice", id, 10)
            .await
            .unwrap();
        assert!(!entries.iter().any(|e| e.action == "invoice.unpaid"));

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// N3 (review pass 3 B) : si `payment_status` est actif (non-`All`), un
    /// `query.status` contradictoire (ex. `draft` ou `cancelled`) est ignoré
    /// silencieusement au profit de la garde implicite `status='validated'`.
    /// Ce test verrouille la sémantique « payment_status l'emporte ».
    #[tokio::test]
    async fn test_payment_status_overrides_contradictory_status_filter() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;

        // Une facture validée + payée pour qu'elle apparaisse sous Paid.
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
        let _ = mark_as_paid(
            &pool,
            admin_user_id,
            id,
            company_id,
            v,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap();

        // status='draft' contradictoire avec payment_status=Paid : le repo
        // doit ignorer le `status` et émettre `status='validated'` via le
        // bloc payment_status, retournant 1 ligne au lieu de 0 silencieux.
        let res = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                status: Some("draft".into()),
                payment_status: Some(PaymentStatusFilter::Paid),
                contact_id: Some(contact_id),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            res.items.iter().any(|i| i.id == id),
            "facture validée+payée doit apparaître malgré status='draft' (payment_status l'emporte)"
        );

        cleanup_invoices(&pool, &[id]).await;
        cleanup_journal_entries(&pool, &[je_id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// P12 (review pass 3 A) : spec §184 — si l'insert audit échoue,
    /// la transaction complète est rollbackée et `paid_at` reste NULL.
    /// On force l'échec audit via un `user_id` inexistant (violation FK
    /// `fk_audit_log_user`).
    #[tokio::test]
    async fn test_mark_as_paid_atomic_rollback_on_audit_failure() {
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

        // user_id = -999 : FK `fk_audit_log_user` échoue → l'insert audit
        // lève une erreur DB qui force le rollback de l'UPDATE invoices.
        let err = mark_as_paid(
            &pool,
            -999,
            id,
            company_id,
            v,
            Some(chrono::Utc::now().naive_utc()),
        )
        .await
        .unwrap_err();
        // N4 (review pass 3 B) : assertion stricte sur `ForeignKeyViolation` —
        // l'invariant testé est précisément que l'échec de l'insert audit
        // (causé par la FK `fk_audit_log_user`) provoque le rollback.
        // Toute autre variante d'erreur indique que le test ne couvre plus
        // ce qu'il prétend couvrir (ex. FK désactivée, schéma changé).
        assert!(
            matches!(err, DbError::ForeignKeyViolation(_)),
            "attendu DbError::ForeignKeyViolation (FK audit_log.user_id), reçu {err:?}"
        );

        // Vérifie atomicité : `paid_at` est toujours NULL et `version`
        // n'a pas été bumpé (l'UPDATE a été annulé par le rollback).
        let (after, _) = find_by_id_with_lines(&pool, company_id, id)
            .await
            .unwrap()
            .unwrap();
        assert!(after.paid_at.is_none(), "paid_at aurait dû rester NULL");
        assert_eq!(after.version, v, "version n'aurait pas dû être bumpée");

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

    /// KF-020 (closes #49) : sous concurrence `update` (mutation) vs `update`
    /// no-op avec le même `expected_version`, le `SELECT FOR UPDATE` ajouté
    /// dans `update` doit garantir qu'aucun client ne reçoit un snapshot stale
    /// déguisé en `200 OK`.
    ///
    /// Avant le fix : la race documentée Story 7-3 §race-condition pouvait
    /// laisser fuiter un état v=N à B alors que A avait commit v=N+1 entre le
    /// BEGIN et le no-op check de B (cf. spec issue #49 T0-T6).
    ///
    /// Après le fix, les seules issues valides sous tokio::join sont :
    /// - **(A=Ok v=N+1, B=Ok v=N)** — B a acquis le X-lock en premier, no-op,
    ///   ROLLBACK ; A a ensuite acquis le lock, vu v=N (B n'a rien commité),
    ///   modifié, commit v=N+1.
    /// - **(A=Ok v=N+1, B=Err OptimisticLockConflict)** — A a acquis le lock
    ///   en premier, modifié, commit v=N+1 ; B a attendu, lu v=N+1,
    ///   version-check applicative N != N+1 → 409.
    ///
    /// L'issue **(A=Ok v=N+1, B=Ok v=N) où B a démarré APRÈS le commit de A**
    /// est éliminée par le X-lock (B serait bloqué par le lock de A jusqu'à
    /// son commit, puis lirait v=N+1 → 409). Le détecter empiriquement de
    /// l'extérieur exigerait des hooks DB-side (cf. KF-021 #50 pour le test
    /// déterministe via barrière SQL).
    ///
    /// Ce test est une **régression sanity-check** : 30 itérations tokio::join
    /// avec assertion d'invariants. Sans le fix, il pourrait passer
    /// occasionnellement (par chance d'ordering), mais devrait flaker quand
    /// le pool est bien chargé. Le test `KF-021 #50` couvrira la détection
    /// déterministe.
    #[tokio::test]
    async fn test_update_concurrent_no_op_vs_mutation_no_stale_snapshot_kf020() {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        // M-3 review remediation : 6 connexions = 2 pour tokio::join (A+B) + 1
        // pour create() séquentiel + 1 pour fetch_one final + 2 marge — évite
        // pool starvation si Tokio scheduling synchronise temporairement les
        // 4 phases sur la même boucle.
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(6)
            .connect(&url)
            .await
            .expect("pool connect");

        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;
        let contact_id = create_test_contact(&pool, company_id, admin_user_id).await;
        let mut invoices_to_clean: Vec<i64> = Vec::with_capacity(30);

        for iteration in 0..30 {
            // Setup : invoice v=1 avec 1 ligne « Item original ».
            let (inv, _lines) = create(
                &pool,
                admin_user_id,
                NewInvoice {
                    company_id,
                    contact_id,
                    date: today(),
                    due_date: None,
                    payment_terms: Some("30 jours net".into()),
                    lines: vec![sample_line("Item original", dec!(1), dec!(100.00))],
                    project_id: None,
                },
            )
            .await
            .expect("create invoice");
            invoices_to_clean.push(inv.id);
            assert_eq!(
                inv.version, 1,
                "iteration {iteration} : v=1 attendu après create"
            );

            let pool_a = pool.clone();
            let pool_b = pool.clone();
            let id = inv.id;
            let expected_version = inv.version;

            // Task A : mutation effective (description différente).
            let task_a = async move {
                update(
                    &pool_a,
                    company_id,
                    id,
                    expected_version,
                    admin_user_id,
                    InvoiceUpdate {
                        contact_id,
                        date: today(),
                        due_date: None,
                        payment_terms: Some("30 jours net".into()),
                        lines: vec![sample_line("Item modifié par A", dec!(1), dec!(100.00))],
                        project_id: None,
                    },
                )
                .await
            };

            // Task B : payload no-op strictement identique à l'état v=1.
            let task_b = async move {
                update(
                    &pool_b,
                    company_id,
                    id,
                    expected_version,
                    admin_user_id,
                    InvoiceUpdate {
                        contact_id,
                        date: today(),
                        due_date: None,
                        payment_terms: Some("30 jours net".into()),
                        lines: vec![sample_line("Item original", dec!(1), dec!(100.00))],
                        project_id: None,
                    },
                )
                .await
            };

            let (res_a, res_b) = tokio::join!(task_a, task_b);

            // Lecture finale de l'invoice pour vérifier l'invariant DB.
            let final_invoice = sqlx::query_as::<_, Invoice>(FIND_INVOICE_SCOPED_SQL)
                .bind(id)
                .bind(company_id)
                .fetch_one(&pool)
                .await
                .expect("fetch final invoice");

            // H-1 review remediation : MariaDB peut résoudre un deadlock en
            // rollback-ant l'une ou l'autre tx. Dans ce scenario à 2 acteurs sur
            // un seul row, c'est statistiquement très rare (gap-locks
            // invoice_lines uniquement), mais possible. On accepte
            // `DbError::Sqlx` (avec une trace explicite) au lieu de panic
            // brut — cela évite que CI fail spuriously sur une cause valide
            // mais transitoire. Si A *succeeds*, on assert v=N+1.
            match res_a {
                Ok((ref a_invoice, ref _a_lines)) => {
                    assert_eq!(
                        a_invoice.version, 2,
                        "iteration {iteration} : A.version doit être N+1=2 (mutation effective)"
                    );
                }
                Err(DbError::Sqlx(ref err)) => {
                    eprintln!(
                        "iteration {iteration} : A a perdu un deadlock (rare, accepté) : {err}"
                    );
                    // Si A a deadlock, le DB final reste à v=1 — l'invariant 3
                    // ci-dessous l'attrapera proprement. On continue.
                }
                Err(ref other) => {
                    panic!(
                        "iteration {iteration} : A doit être Ok ou Sqlx(deadlock), got {other:?}"
                    );
                }
            }

            // Invariant 2 : B est soit Ok(v=1, no-op préservé) soit
            // OptimisticLockConflict. Sous deadlock rare, B peut aussi être
            // Sqlx(deadlock) — accepté avec trace.
            match &res_b {
                Ok((b_invoice, _b_lines)) => {
                    assert_eq!(
                        b_invoice.version, 1,
                        "iteration {iteration} : B.version doit être 1 (no-op préserve la version)",
                    );
                    // B a acquis le lock en premier → ROLLBACK no-op ; A,
                    // bloquée pendant la durée de la tx de B, prend le lock
                    // ensuite, voit v=1, modifie, commit v=2.
                }
                Err(DbError::OptimisticLockConflict) => {
                    // A a acquis le lock en premier → commit v=2 → B a vu v=2 → 409. ✓
                }
                Err(DbError::Sqlx(err)) => {
                    eprintln!(
                        "iteration {iteration} : B a perdu un deadlock (rare, accepté) : {err}"
                    );
                }
                Err(other) => {
                    panic!(
                        "iteration {iteration} : B doit être Ok(v=1), OptimisticLockConflict, ou Sqlx(deadlock), got {other:?}"
                    );
                }
            }

            // Invariant 3 : DB final cohérent avec res_a.
            // - Si A=Ok : commit v=2 appliqué.
            // - Si A=Err(Sqlx(deadlock)) : pas de commit, DB reste v=1.
            // - B=Ok(v=1) ou Err(*) ne mute jamais le DB (no-op rollback ou échec).
            let expected_db_version = if res_a.is_ok() { 2 } else { 1 };
            assert_eq!(
                final_invoice.version,
                expected_db_version,
                "iteration {iteration} : DB final v={expected_db_version} attendu (res_a={:?})",
                res_a.as_ref().map(|(i, _)| i.version)
            );
            // total_amount = sum(qty × unit_price) HT, sans TVA (cf. compute_total).
            // Mutation A garde 1×100, donc total inchangé numériquement
            // quel que soit l'outcome — c'est version=2 qui prouve la mutation.
            assert_eq!(
                final_invoice.total_amount,
                dec!(100.0000),
                "iteration {iteration} : DB final total HT inchangé (1×100, sans TVA)"
            );
        }

        cleanup_invoices(&pool, &invoices_to_clean).await;
        cleanup_contacts(&pool, &[contact_id]).await;
        pool.close().await;
    }

    fn line_to_new(l: &InvoiceLine) -> NewInvoiceLine {
        NewInvoiceLine {
            description: l.description.clone(),
            quantity: l.quantity,
            unit_price: l.unit_price,
            vat_rate: l.vat_rate,
        }
    }

    /// KF-004 : payload identique (header + lignes même ordre) → pas de bump
    /// version, `updated_at` inchangé, **mêmes IDs DB pour les lignes** (pas de
    /// DELETE+INSERT), pas d'audit_log `invoice.updated`. `total_amount` reste
    /// cohérent avec les lignes par construction (compute_total est pure).
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_lines_churn() {
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
                payment_terms: Some("30 jours net".into()),
                lines: vec![
                    sample_line("Conseil", dec!(2), dec!(150.00)),
                    sample_line("Hébergement", dec!(1), dec!(50.00)),
                ],
                project_id: None,
            },
        )
        .await
        .unwrap();
        let version_initial = inv.version;
        let updated_at_initial = inv.updated_at;
        let total_initial = inv.total_amount;
        let line_ids_initial: Vec<i64> = lines.iter().map(|l| l.id).collect();

        let (result_inv, result_lines) = update(
            &pool,
            company_id,
            inv.id,
            version_initial,
            admin_user_id,
            InvoiceUpdate {
                contact_id: inv.contact_id,
                date: inv.date,
                due_date: inv.due_date,
                payment_terms: inv.payment_terms.clone(),
                lines: lines.iter().map(line_to_new).collect(),
                project_id: None,
            },
        )
        .await
        .unwrap();

        assert_eq!(result_inv.version, version_initial, "version inchangée");
        assert_eq!(
            result_inv.updated_at, updated_at_initial,
            "updated_at inchangé"
        );
        assert_eq!(result_inv.total_amount, total_initial);
        let line_ids_after: Vec<i64> = result_lines.iter().map(|l| l.id).collect();
        assert_eq!(
            line_ids_after, line_ids_initial,
            "no-op : pas de DELETE+INSERT, IDs lignes identiques"
        );

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'invoice' AND entity_id = ? AND action = 'invoice.updated'",
        )
        .bind(inv.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// KF-004 régression : modifier `unit_price` d'une ligne → bump version,
    /// DELETE+INSERT effectif (les IDs lignes changent), audit log présent.
    #[tokio::test]
    async fn update_line_change_bumps_version() {
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
                lines: vec![sample_line("Item A", dec!(1), dec!(100.00))],
                project_id: None,
            },
        )
        .await
        .unwrap();
        let version_initial = inv.version;

        let mut new_lines: Vec<NewInvoiceLine> = lines.iter().map(line_to_new).collect();
        new_lines[0].unit_price = dec!(120.00);

        let (result_inv, _result_lines) = update(
            &pool,
            company_id,
            inv.id,
            version_initial,
            admin_user_id,
            InvoiceUpdate {
                contact_id: inv.contact_id,
                date: inv.date,
                due_date: inv.due_date,
                payment_terms: inv.payment_terms.clone(),
                lines: new_lines,
                project_id: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(result_inv.version, version_initial + 1);

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'invoice' AND entity_id = ? AND action = 'invoice.updated'",
        )
        .bind(inv.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 1);

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    /// KF-004 régression : permuter l'ordre des lignes (mêmes contenus) → bump
    /// version. `line_order` est sémantique (détermine l'affichage PDF).
    #[tokio::test]
    async fn update_line_reorder_bumps_version() {
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
                    sample_line("Ligne A", dec!(1), dec!(100.00)),
                    sample_line("Ligne B", dec!(1), dec!(200.00)),
                ],
                project_id: None,
            },
        )
        .await
        .unwrap();
        let version_initial = inv.version;

        let reordered: Vec<NewInvoiceLine> = vec![line_to_new(&lines[1]), line_to_new(&lines[0])];

        let (result_inv, _) = update(
            &pool,
            company_id,
            inv.id,
            version_initial,
            admin_user_id,
            InvoiceUpdate {
                contact_id: inv.contact_id,
                date: inv.date,
                due_date: inv.due_date,
                payment_terms: inv.payment_terms.clone(),
                lines: reordered,
                project_id: None,
            },
        )
        .await
        .unwrap();
        assert_eq!(result_inv.version, version_initial + 1);

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }

    // -------------------------------------------------------------------
    // Story 7-4 / KF-005 — search inline tests (T6.4 + Pass 1 F1)
    // -------------------------------------------------------------------

    async fn create_named_contact(
        pool: &MySqlPool,
        company_id: i64,
        user_id: i64,
        name: &str,
    ) -> i64 {
        contacts::create(
            pool,
            user_id,
            NewContact {
                company_id,
                contact_type: ContactType::Entreprise,
                name: name.into(),
                first_name: None,
                last_name: None,
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                default_payment_terms: Some("30 jours net".into()),
                language: None,
                salutation: crate::entities::contact::Salutation::Neutre,
            },
        )
        .await
        .expect("create_named_contact")
        .id
    }

    /// T6.4 + Pass 1 F1 : couvre le path FULLTEXT `MATCH(c.name)` dans
    /// `list_by_company_paginated` ET `due_dates_summary` (les 2 callsites
    /// refactorés Story 7-4). Sans ce test, la query SQL générée pour
    /// `MATCH(c.name) AGAINST(...)` ne serait jamais exercée par la suite
    /// de tests — un bug dans la composition JOIN + multi-tenant + FULLTEXT
    /// passerait inaperçu.
    #[tokio::test]
    async fn test_filter_by_search_matches_contact_name_fulltext() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;

        // Marker unique 8 chars pour éviter la collision avec d'autres
        // tests qui partagent le même pool DB. Token ≥ 3 chars (indexable
        // via `innodb_ft_min_token_size = 3`).
        let suffix = short_uuid();
        let marker_marie = format!("KFFiveMa{suffix}");
        let marker_pierre = format!("KFFivePi{suffix}");

        let contact_marie = create_named_contact(
            &pool,
            company_id,
            admin_user_id,
            &format!("TestSearchInv {marker_marie} SARL"),
        )
        .await;
        let contact_pierre = create_named_contact(
            &pool,
            company_id,
            admin_user_id,
            &format!("TestSearchInv {marker_pierre} SARL"),
        )
        .await;

        let due = today();
        let (inv_marie, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id: contact_marie,
                date: today(),
                due_date: Some(due),
                payment_terms: None,
                lines: vec![sample_line("Conseil", dec!(1), dec!(100.00))],
                project_id: None,
            },
        )
        .await
        .unwrap();
        let (inv_pierre, _) = create(
            &pool,
            admin_user_id,
            NewInvoice {
                company_id,
                contact_id: contact_pierre,
                date: today(),
                due_date: Some(due),
                payment_terms: None,
                lines: vec![sample_line("Conseil", dec!(1), dec!(200.00))],
                project_id: None,
            },
        )
        .await
        .unwrap();

        // (1) list_by_company_paginated — search par marker Marie via
        //     `MATCH(c.name) AGAINST('marker*' IN BOOLEAN MODE)`.
        let result = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                search: Some(marker_marie.clone()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            result.items.iter().any(|i| i.id == inv_marie.id),
            "list_by_company_paginated : search par `{marker_marie}` doit retourner la facture Marie"
        );
        assert!(
            !result.items.iter().any(|i| i.id == inv_pierre.id),
            "list_by_company_paginated : la facture Pierre NE doit PAS matcher le marker Marie"
        );

        // (2) due_dates_summary — même search filter. On valide les
        //     deux factures pour qu'elles entrent dans le summary
        //     (`status='validated' AND paid_at IS NULL`).
        let (_, je_marie) = force_validate(&pool, company_id, inv_marie.id).await;
        let (_, je_pierre) = force_validate(&pool, company_id, inv_pierre.id).await;

        let summary = due_dates_summary(
            &pool,
            company_id,
            &InvoiceListQuery {
                search: Some(marker_marie.clone()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            summary.unpaid_count, 1,
            "due_dates_summary : search par `{marker_marie}` doit retourner unpaid_count=1 (Marie uniquement)"
        );
        assert_eq!(
            summary.unpaid_total,
            dec!(100.0000),
            "due_dates_summary : unpaid_total = total facture Marie uniquement"
        );

        cleanup_invoices(&pool, &[inv_marie.id, inv_pierre.id]).await;
        cleanup_journal_entries(&pool, &[je_marie, je_pierre]).await;
        cleanup_contacts(&pool, &[contact_marie, contact_pierre]).await;
    }

    /// Pass 1 F4 (régression) : un input non-vide entièrement composé
    /// d'opérateurs BOOLEAN MODE doit retourner 0 résultats côté invoices
    /// (le fallback LIKE sur `invoice_number`/`payment_terms` est actif,
    /// mais ne matche pas un input gibberish). Vérifie l'absence de skip
    /// silencieux (qui retournerait toutes les factures de la company).
    ///
    /// Pass 2 polish : sanity check positif en début de test pour
    /// distinguer un fail "search retourne 0 systématiquement" (bug grave)
    /// d'un fail "gibberish ne matche pas" (comportement attendu).
    #[tokio::test]
    async fn test_filter_by_search_pure_operators_returns_zero() {
        let pool = test_pool().await;
        let company_id = get_company_id(&pool).await;
        let admin_user_id = get_admin_user_id(&pool).await;

        let suffix = short_uuid();
        let marker = format!("KFFiveF4Inv{suffix}");
        let contact_id = create_named_contact(
            &pool,
            company_id,
            admin_user_id,
            &format!("TestF4Inv {marker} SARL"),
        )
        .await;
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
                project_id: None,
            },
        )
        .await
        .unwrap();

        // Sanity positive : le marker unique DOIT matcher la facture seedée
        // (sinon le path FULLTEXT MATCH(c.name) est cassé et le test
        // gibberish vacuously-pass avec 0 résultats systématiques).
        let sanity = list_by_company_paginated(
            &pool,
            company_id,
            InvoiceListQuery {
                search: Some(marker.clone()),
                limit: 100,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            sanity.items.iter().any(|i| i.id == inv.id),
            "sanity check : search par marker `{marker}` doit matcher la facture seedée — \
             si ça fail, le path search est cassé et les assertions gibberish ci-dessous sont vacuous"
        );

        for gibberish in ["+++", "***", "()()", "~~~"] {
            let result = list_by_company_paginated(
                &pool,
                company_id,
                InvoiceListQuery {
                    search: Some(gibberish.into()),
                    limit: 100,
                    ..Default::default()
                },
            )
            .await
            .unwrap();
            // L'invoice ne contient ni `+++` ni `***` etc. dans
            // invoice_number ou payment_terms (`30 jours net`) → 0 match.
            assert!(
                !result.items.iter().any(|i| i.id == inv.id),
                "input pure-opérateurs `{gibberish}` ne doit pas matcher la facture seedée"
            );
        }

        cleanup_invoices(&pool, &[inv.id]).await;
        cleanup_contacts(&pool, &[contact_id]).await;
    }
}
