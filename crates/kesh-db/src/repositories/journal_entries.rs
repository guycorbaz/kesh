//! Repository pour les écritures comptables en partie double.
//!
//! L'invariant central est l'**atomicité** : une écriture (en-tête +
//! lignes) est toujours créée ou rejetée en bloc. La numérotation
//! séquentielle par `(company_id, fiscal_year_id)` est garantie sans
//! trou par la combinaison `SELECT MAX(...) FOR UPDATE` + contrainte
//! `UNIQUE (company_id, fiscal_year_id, entry_number)`.
//!
//! # Defense in depth
//!
//! Trois niveaux de garde-fou empêchent une écriture déséquilibrée :
//!
//! 1. `kesh_core::accounting::validate()` (logique pure, côté route)
//! 2. Contrainte DB `chk_jel_debit_credit_exclusive` (par ligne)
//! 3. Re-calcul `SUM(debit) = SUM(credit)` après INSERT dans ce
//!    repository (rollback si mismatch)
//!
//! # Immutabilité post-clôture (FR24, CO art. 957-964)
//!
//! Un `SELECT fiscal_years FOR UPDATE` en tête de transaction verrouille
//! l'exercice contre toute clôture concurrente. Si `status = 'Closed'`,
//! la création est refusée avec `DbError::IllegalStateTransition`.

use std::str::FromStr;

use chrono::NaiveDate;
use rust_decimal::Decimal;
use sqlx::mysql::MySqlPool;
use sqlx::{QueryBuilder, Row};

use kesh_core::listing::{SortBy, SortDirection};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::{
    Journal, JournalEntry, JournalEntryLine, JournalEntryWithLines, NewJournalEntry,
};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const ENTRY_COLUMNS: &str = "id, company_id, fiscal_year_id, entry_number, entry_date, journal, description, \
     version, created_at, updated_at";

const LINE_COLUMNS: &str = "id, entry_id, account_id, line_order, debit, credit";

/// Crée une écriture comptable (en-tête + lignes) dans une transaction
/// atomique. Wrapper pool-level : ouvre sa propre transaction et la
/// valide/rollback selon le résultat. Délègue à [`create_in_tx`] pour
/// tout le travail métier.
///
/// Le `fiscal_year_id` doit être pré-validé par le caller via
/// [`fiscal_years::find_covering_date`](super::fiscal_years::find_covering_date)
/// — il est re-vérifié ici avec `FOR UPDATE` pour capturer les races
/// avec une clôture concurrente.
pub async fn create(
    pool: &MySqlPool,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;
    match create_in_tx(&mut tx, fiscal_year_id, user_id, new).await {
        Ok(result) => {
            tx.commit().await.map_err(map_db_error)?;
            Ok(result)
        }
        Err(e) => {
            // Best-effort rollback. Si le rollback échoue lui-même, on
            // privilégie l'erreur métier originale — l'appelant la verra
            // en premier et le drop-guard SQLx annulera la tx en arrière-plan.
            let _ = tx.rollback().await;
            Err(e)
        }
    }
}

/// Cœur métier de [`create`] — accepte une transaction ouverte par le
/// caller et **ne commit/rollback PAS** (responsabilité du caller).
///
/// Utilisée à deux endroits :
/// 1. [`create`] (wrapper pool-level) — cas standard.
/// 2. [`invoices::validate_invoice`](super::invoices::validate_invoice)
///    (Story 5.2) — pour garantir l'atomicité { numérotation facture +
///    insertion écriture comptable + UPDATE invoices.status } dans une
///    seule transaction (impossible si `create` ouvre sa propre tx).
///
/// Contrat : en cas de succès, retourne `Ok(JournalEntryWithLines)` et
/// la tx contient les inserts. En cas d'erreur, bubble-up sans toucher
/// à la tx — le caller doit rollback ou laisser le drop-guard agir.
pub async fn create_in_tx(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    fiscal_year_id: i64,
    user_id: i64,
    new: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    // Étape 1 : re-lock de l'exercice contre une clôture concurrente.
    let fy_row: Option<(i64, String)> = sqlx::query_as(
        "SELECT id, status FROM fiscal_years \
         WHERE id = ? AND company_id = ? FOR UPDATE",
    )
    .bind(fiscal_year_id)
    .bind(new.company_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_db_error)?;

    match fy_row {
        None => return Err(DbError::NotFound),
        Some((_, status)) if status == "Closed" => return Err(DbError::FiscalYearClosed),
        Some(_) => {}
    }

    // Étape 2 : vérifier que tous les comptes existent, appartiennent
    // à la company et sont actifs.
    if new.lines.is_empty() {
        return Err(DbError::Invariant(
            "NewJournalEntry sans lignes — devait être rejeté en amont".into(),
        ));
    }

    let account_ids: Vec<i64> = new.lines.iter().map(|l| l.account_id).collect();
    let placeholders = account_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let accounts_sql = format!(
        "SELECT id FROM accounts \
         WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_scalar::<_, i64>(&accounts_sql).bind(new.company_id);
    for id in &account_ids {
        q = q.bind(id);
    }
    let active_ids: Vec<i64> = q.fetch_all(&mut **tx).await.map_err(map_db_error)?;

    let mut unique_requested: Vec<i64> = account_ids.clone();
    unique_requested.sort_unstable();
    unique_requested.dedup();

    if active_ids.len() != unique_requested.len() {
        return Err(DbError::InactiveOrInvalidAccounts);
    }

    // Étape 3 : calculer le prochain entry_number (sérialisé par gap lock).
    let next_number: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries \
         WHERE company_id = ? AND fiscal_year_id = ? FOR UPDATE",
    )
    .bind(new.company_id)
    .bind(fiscal_year_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Étape 4 : INSERT de l'en-tête.
    let header_result = sqlx::query(
        "INSERT INTO journal_entries \
         (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(new.company_id)
    .bind(fiscal_year_id)
    .bind(next_number)
    .bind(new.entry_date)
    .bind(new.journal)
    .bind(&new.description)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = header_result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT journal_entries".into(),
        ));
    }
    let entry_id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    // Étape 5 : INSERT des lignes avec line_order séquentiel.
    for (idx, line) in new.lines.iter().enumerate() {
        let line_order = (idx as i32) + 1;
        sqlx::query(
            "INSERT INTO journal_entry_lines \
             (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(line.account_id)
        .bind(line_order)
        .bind(line.debit)
        .bind(line.credit)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
    }

    // Étape 6 : double-check balance applicative (defense in depth).
    let row = sqlx::query(
        "SELECT COALESCE(SUM(debit), 0) AS d, COALESCE(SUM(credit), 0) AS c \
         FROM journal_entry_lines WHERE entry_id = ?",
    )
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let total_debit: Decimal = row.try_get("d").map_err(map_db_error)?;
    let total_credit: Decimal = row.try_get("c").map_err(map_db_error)?;

    if total_debit != total_credit {
        return Err(DbError::Invariant(format!(
            "balance DB incohérente après INSERT : débit={total_debit}, crédit={total_credit}"
        )));
    }

    // Étape 7 : re-fetch entry + lines pour le retour.
    let entry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(entry_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(entry_id)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_db_error)?;

    // Étape 8 (Story 3.5) : INSERT audit_log avant le COMMIT.
    // Le caller qui appelle create_in_tx peut ajouter son propre audit
    // (ex. validate_invoice ajoute « invoice.validated » en complément).
    let snapshot = entry_snapshot_json(&entry, &lines);
    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "journal_entry.created".to_string(),
            entity_type: "journal_entry".to_string(),
            entity_id: entry_id,
            details_json: Some(snapshot),
        },
    )
    .await?;

    Ok(JournalEntryWithLines { entry, lines })
}

/// Retourne une écriture avec ses lignes, scopée à une company pour
/// éviter toute fuite cross-tenant.
///
/// **P10 defense in depth** : le paramètre `company_id` est obligatoire.
/// Une écriture d'une autre company retourne `None`, jamais de donnée.
/// Pattern à reproduire pour toute future route `GET /journal-entries/:id`.
pub async fn find_by_id(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
) -> Result<Option<JournalEntryWithLines>, DbError> {
    let entry_opt = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ? AND company_id = ?"
    ))
    .bind(id)
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?;

    let Some(entry) = entry_opt else {
        return Ok(None);
    };

    let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(entry.id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(Some(JournalEntryWithLines { entry, lines }))
}

/// Limites hard sur la pagination (borne défensive, la source canonique
/// est le route handler).
const MAX_LIMIT: i64 = 500;

/// Valeur maximale safe pour un filtre `amount_max` absent, exactement
/// alignée avec le maximum stockable en `DECIMAL(19,4)` : 15 chiffres
/// entiers + 4 décimales = `999'999'999'999'999.9999`.
///
/// Utilisé comme borne supérieure pour une sous-requête `HAVING SUM(debit)
/// BETWEEN ? AND ?` : tout `SUM(debit)` d'une écriture réelle sera
/// strictement inférieur à cette borne.
fn decimal_max_safe() -> Decimal {
    // `from_str` est infaillible pour un literal valide — le `expect`
    // ne panique jamais et sert de documentation d'invariant.
    Decimal::from_str("999999999999999.9999").expect("literal decimal constant must parse")
}

/// Échappe les caractères spéciaux `%` et `_` pour l'opérateur `LIKE`.
///
/// Pattern SQL utilisé : `LIKE ? ESCAPE '\\'`. Le backslash est le
/// caractère d'échappement. Attention au quadruple backslash en source
/// Rust (voir Dev Notes §Pièges story 3.4).
fn escape_like(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

/// Paramètres de recherche, tri et pagination pour `list_by_company_paginated`.
#[derive(Debug, Clone)]
pub struct JournalEntryListQuery {
    pub description: Option<String>,
    pub amount_min: Option<Decimal>,
    pub amount_max: Option<Decimal>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub journal: Option<Journal>,
    pub sort_by: SortBy,
    pub sort_dir: SortDirection,
    pub limit: i64,
    pub offset: i64,
}

impl Default for JournalEntryListQuery {
    fn default() -> Self {
        Self {
            description: None,
            amount_min: None,
            amount_max: None,
            date_from: None,
            date_to: None,
            journal: None,
            sort_by: SortBy::default(),
            sort_dir: SortDirection::default(),
            limit: 50,
            offset: 0,
        }
    }
}

/// Résultat paginé retourné par `list_by_company_paginated`.
#[derive(Debug)]
pub struct JournalEntryListResult {
    pub items: Vec<JournalEntryWithLines>,
    pub total: i64,
    pub offset: i64,
    pub limit: i64,
}

/// Pousse les clauses WHERE dynamiques dans un `QueryBuilder`.
///
/// **CRITIQUE** : cette fonction doit être appelée sur DEUX `QueryBuilder`
/// DISTINCTS (count + items) — un `QueryBuilder` encode un état mutable et
/// ne peut pas être réutilisé après un `build_*`.
///
/// Préconditions : `qb` vient d'être initialisé avec le SELECT préfixe
/// (ex: `QueryBuilder::new("SELECT COUNT(*) FROM journal_entries")`).
fn push_where_clauses<'a>(
    qb: &mut QueryBuilder<'a, sqlx::MySql>,
    company_id: i64,
    query: &'a JournalEntryListQuery,
) {
    qb.push(" WHERE company_id = ");
    qb.push_bind(company_id);

    if let Some(ref desc) = query.description {
        // Échappement `%` et `_` pour éviter la collision avec l'opérateur LIKE.
        qb.push(" AND description LIKE ");
        qb.push_bind(format!("%{}%", escape_like(desc)));
        qb.push(" ESCAPE '\\\\'");
    }

    if let Some(date_from) = query.date_from {
        qb.push(" AND entry_date >= ");
        qb.push_bind(date_from);
    }

    if let Some(date_to) = query.date_to {
        qb.push(" AND entry_date <= ");
        qb.push_bind(date_to);
    }

    if let Some(journal) = query.journal {
        qb.push(" AND journal = ");
        qb.push_bind(journal);
    }

    // Filtre par plage de montants — sous-requête sur la somme des débits
    // par écriture (en partie double, SUM(debit) == SUM(credit)).
    if query.amount_min.is_some() || query.amount_max.is_some() {
        let min_val = query.amount_min.unwrap_or(Decimal::ZERO);
        let max_val = query.amount_max.unwrap_or_else(decimal_max_safe);
        qb.push(" AND id IN (SELECT entry_id FROM journal_entry_lines GROUP BY entry_id HAVING SUM(debit) BETWEEN ");
        qb.push_bind(min_val);
        qb.push(" AND ");
        qb.push_bind(max_val);
        qb.push(")");
    }
}

/// Liste paginée des écritures d'une company avec filtres et tri.
///
/// Deux queries séquentielles :
/// 1. `SELECT COUNT(*)` avec les filtres (pour le total).
/// 2. `SELECT ... ORDER BY ... LIMIT OFFSET` avec les filtres (pour les items).
///
/// Les lignes sont chargées ensuite via des SELECTs N+1 (acceptable pour
/// `limit <= 500` et un volume PME).
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    query: JournalEntryListQuery,
) -> Result<JournalEntryListResult, DbError> {
    // Clamp défensif — la source de vérité est le route handler.
    let clamped_limit = query.limit.clamp(1, MAX_LIMIT);
    let clamped_offset = query.offset.max(0);

    // --- Query 1 : count total (deux QueryBuilder distincts, critique) ---
    let mut count_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new("SELECT COUNT(*) FROM journal_entries");
    push_where_clauses(&mut count_qb, company_id, &query);

    let total: i64 = count_qb
        .build_query_scalar()
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    // --- Query 2 : items paginés ---
    let mut items_qb: QueryBuilder<sqlx::MySql> =
        QueryBuilder::new(format!("SELECT {ENTRY_COLUMNS} FROM journal_entries"));
    push_where_clauses(&mut items_qb, company_id, &query);

    // ORDER BY secondaire stable sur entry_number DESC pour éviter
    // les rangs instables en cas de dates/journaux identiques.
    let sort_col = query.sort_by.as_sql_column();
    let sort_dir_sql = query.sort_dir.as_sql_keyword();
    items_qb.push(format!(
        " ORDER BY {sort_col} {sort_dir_sql}, entry_number DESC LIMIT "
    ));
    items_qb.push_bind(clamped_limit);
    items_qb.push(" OFFSET ");
    items_qb.push_bind(clamped_offset);

    let entries: Vec<JournalEntry> = items_qb
        .build_query_as::<JournalEntry>()
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    // --- Charger les lignes de chaque entry (N+1 acceptable limit ≤ 500) ---
    let mut items = Vec::with_capacity(entries.len());
    for entry in entries {
        let lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
            "SELECT {LINE_COLUMNS} FROM journal_entry_lines \
             WHERE entry_id = ? ORDER BY line_order"
        ))
        .bind(entry.id)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
        items.push(JournalEntryWithLines { entry, lines });
    }

    Ok(JournalEntryListResult {
        items,
        total,
        offset: clamped_offset,
        limit: clamped_limit,
    })
}

/// Helper — produit un snapshot JSON d'une écriture (en-tête + lignes)
/// pour l'audit log (`before`/`after`).
///
/// Contient les champs utiles pour une reconstitution partielle :
/// id, entryNumber, entryDate, journal, description, version, lines.
/// Les montants sont sérialisés comme strings (évite erreurs d'arrondi JSON).
fn entry_snapshot_json(entry: &JournalEntry, lines: &[JournalEntryLine]) -> serde_json::Value {
    serde_json::json!({
        "id": entry.id,
        "entryNumber": entry.entry_number,
        "entryDate": entry.entry_date.to_string(),
        "journal": entry.journal.as_str(),
        "description": entry.description,
        "version": entry.version,
        "lines": lines.iter().map(|l| serde_json::json!({
            "lineOrder": l.line_order,
            "accountId": l.account_id,
            "debit": l.debit.to_string(),
            "credit": l.credit.to_string(),
        })).collect::<Vec<_>>()
    })
}

/// Met à jour une écriture existante avec verrouillage optimiste.
///
/// Stratégie « lock + check applicatif » :
/// 1. `SELECT ... FOR UPDATE` sur l'entry + jointure fiscal_years (exclusif)
/// 2. Check `fy.status == Open` sinon `FiscalYearClosed`
/// 3. Check `version_db == version_param` sinon `OptimisticLockConflict`
/// 4. Check `updated.entry_date` dans `[fy.start_date, fy.end_date]` sinon `DateOutsideFiscalYear`
/// 5. Vérifier tous les comptes actifs et appartenant à la company
/// 6. Snapshot "before" (SELECTs inline dans la tx)
/// 7. DELETE lines + UPDATE header (version += 1) + INSERT new lines
/// 8. Re-check balance applicatif
/// 9. Re-fetch + snapshot "after"
/// 10. INSERT audit_log avec `before`/`after`
/// 11. COMMIT
///
/// Compare l'état persisté (header + lignes) au payload — `true` si aucun
/// champ métier ne diffère (KF-004 : court-circuit no-op pour ne pas bumper
/// version inutilement).
///
/// Comparaison lignes en respectant `line_order` (la sémantique métier
/// d'une écriture comptable dépend de l'ordre — débit puis crédit, etc.).
fn is_no_op_change(
    before_entry: &JournalEntry,
    before_lines: &[JournalEntryLine],
    updated: &NewJournalEntry,
) -> bool {
    if before_entry.entry_date != updated.entry_date
        || before_entry.journal != updated.journal
        || before_entry.description != updated.description
    {
        return false;
    }
    if before_lines.len() != updated.lines.len() {
        return false;
    }
    before_lines
        .iter()
        .zip(updated.lines.iter())
        .all(|(b, c)| b.account_id == c.account_id && b.debit == c.debit && b.credit == c.credit)
}

/// Règle stricte : `tx.rollback()` explicite avant chaque `return Err`.
pub async fn update(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    version: i32,
    user_id: i64,
    updated: NewJournalEntry,
) -> Result<JournalEntryWithLines, DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Étape 1 : SELECT FOR UPDATE join fiscal_year.
    #[derive(sqlx::FromRow)]
    struct LockedRow {
        entry_version: i32,
        fy_status: String,
        fy_start: NaiveDate,
        fy_end: NaiveDate,
    }

    let locked: Option<LockedRow> = sqlx::query_as(
        "SELECT je.version AS entry_version, \
                fy.status AS fy_status, \
                fy.start_date AS fy_start, \
                fy.end_date AS fy_end \
         FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? \
         FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let locked = match locked {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(row) => row,
    };

    // Étape 2 : statut fiscal_year.
    if locked.fy_status == "Closed" {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::FiscalYearClosed);
    }

    // Étape 3 : version check applicatif.
    if locked.entry_version != version {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }

    // Étape 4 : date dans l'exercice courant (anti-TOCTOU, M4 passe 1).
    if updated.entry_date < locked.fy_start || updated.entry_date > locked.fy_end {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::DateOutsideFiscalYear);
    }

    // Étape 5 : comptes actifs appartenant à la company.
    if updated.lines.is_empty() {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(
            "NewJournalEntry sans lignes — devait être rejeté en amont".into(),
        ));
    }

    let account_ids: Vec<i64> = updated.lines.iter().map(|l| l.account_id).collect();
    let placeholders = account_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let accounts_sql = format!(
        "SELECT id FROM accounts \
         WHERE company_id = ? AND active = TRUE AND id IN ({placeholders})"
    );
    let mut q = sqlx::query_scalar::<_, i64>(&accounts_sql).bind(company_id);
    for aid in &account_ids {
        q = q.bind(aid);
    }
    let active_ids: Vec<i64> = q.fetch_all(&mut *tx).await.map_err(map_db_error)?;

    let mut unique_requested: Vec<i64> = account_ids.clone();
    unique_requested.sort_unstable();
    unique_requested.dedup();

    if active_ids.len() != unique_requested.len() {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::InactiveOrInvalidAccounts);
    }

    // Étape 6 : snapshot "before" (SELECTs inline dans la tx — M2 tranché).
    let before_entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before_lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before_json = entry_snapshot_json(&before_entry, &before_lines);

    // KF-004 : court-circuit no-op AVANT le DELETE/UPDATE/INSERT.
    // Tous les guards (FY status, version check, date dans FY, comptes
    // actifs) ont déjà passé — un payload identique avec un état env
    // valide retourne l'entry inchangée. Le verrou `FOR UPDATE` est
    // libéré par le `tx.rollback()` (équivalent à commit côté locks
    // InnoDB pour ce qui est de leur libération).
    // NOTE concurrence (KF-004): grâce à `SELECT ... FOR UPDATE` étape 1,
    // cette fonction n'est PAS exposée à la race REPEATABLE READ décrite
    // dans la spec §race-condition. Les commits parallèles attendent le
    // verrou X-lock, donc le snapshot post-lock est forcément à jour.
    if is_no_op_change(&before_entry, &before_lines, &updated) {
        tx.rollback().await.map_err(map_db_error)?;
        return Ok(JournalEntryWithLines {
            entry: before_entry,
            lines: before_lines,
        });
    }

    // Étape 7 : DELETE old lines + UPDATE header + INSERT new lines.
    sqlx::query("DELETE FROM journal_entry_lines WHERE entry_id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    let update_result = sqlx::query(
        "UPDATE journal_entries SET entry_date = ?, journal = ?, description = ?, \
         version = version + 1, updated_at = CURRENT_TIMESTAMP(3) \
         WHERE id = ?",
    )
    .bind(updated.entry_date)
    .bind(updated.journal)
    .bind(&updated.description)
    .bind(id)
    .execute(&mut *tx)
    .await;

    if let Err(e) = update_result {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(map_db_error(e));
    }

    for (idx, line) in updated.lines.iter().enumerate() {
        let line_order = (idx as i32) + 1;
        let insert = sqlx::query(
            "INSERT INTO journal_entry_lines \
             (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(line.account_id)
        .bind(line_order)
        .bind(line.debit)
        .bind(line.credit)
        .execute(&mut *tx)
        .await;

        if let Err(e) = insert {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(map_db_error(e));
        }
    }

    // Étape 8 : re-check balance applicatif.
    let row = sqlx::query(
        "SELECT COALESCE(SUM(debit), 0) AS d, COALESCE(SUM(credit), 0) AS c \
         FROM journal_entry_lines WHERE entry_id = ?",
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let total_debit: Decimal = row.try_get("d").map_err(map_db_error)?;
    let total_credit: Decimal = row.try_get("c").map_err(map_db_error)?;

    if total_debit != total_credit {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::Invariant(format!(
            "balance DB incohérente après UPDATE : débit={total_debit}, crédit={total_credit}"
        )));
    }

    // Étape 9 : re-fetch pour le retour + snapshot "after".
    let after_entry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let after_lines = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let after_json = entry_snapshot_json(&after_entry, &after_lines);

    // Étape 10 : INSERT audit_log dans la même tx.
    let audit_details = serde_json::json!({
        "before": before_json,
        "after": after_json,
    });
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "journal_entry.updated".to_string(),
            entity_type: "journal_entry".to_string(),
            entity_id: id,
            details_json: Some(audit_details),
        },
    )
    .await?;

    tx.commit().await.map_err(map_db_error)?;

    Ok(JournalEntryWithLines {
        entry: after_entry,
        lines: after_lines,
    })
}

/// Supprime une écriture et ses lignes (CASCADE), avec enregistrement
/// audit atomique.
///
/// Étapes :
/// 1. BEGIN tx
/// 2. SELECT FOR UPDATE join fiscal_year (lock entry + FY)
/// 3. Si `Closed` → rollback + `FiscalYearClosed`
/// 4. Snapshot "before" (re-fetch lines)
/// 5. INSERT audit_log (AVANT le DELETE pour préserver la trace)
/// 6. DELETE FROM journal_entries → lignes suivent par CASCADE
/// 7. COMMIT
pub async fn delete_by_id(
    pool: &MySqlPool,
    company_id: i64,
    id: i64,
    user_id: i64,
) -> Result<(), DbError> {
    let mut tx = pool.begin().await.map_err(map_db_error)?;

    // Étape 2 : lock entry + fiscal_year.
    let locked: Option<(i64, String)> = sqlx::query_as(
        "SELECT je.fiscal_year_id, fy.status \
         FROM journal_entries je \
         JOIN fiscal_years fy ON fy.id = je.fiscal_year_id \
         WHERE je.id = ? AND je.company_id = ? \
         FOR UPDATE",
    )
    .bind(id)
    .bind(company_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let (_fy_id, fy_status) = match locked {
        None => {
            tx.rollback().await.map_err(map_db_error)?;
            return Err(DbError::NotFound);
        }
        Some(row) => row,
    };

    // Étape 3 : statut FY.
    if fy_status == "Closed" {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::FiscalYearClosed);
    }

    // Étape 4 : snapshot avant suppression.
    let before_entry: JournalEntry = sqlx::query_as::<_, JournalEntry>(&format!(
        "SELECT {ENTRY_COLUMNS} FROM journal_entries WHERE id = ?"
    ))
    .bind(id)
    .fetch_one(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let before_lines: Vec<JournalEntryLine> = sqlx::query_as::<_, JournalEntryLine>(&format!(
        "SELECT {LINE_COLUMNS} FROM journal_entry_lines WHERE entry_id = ? ORDER BY line_order"
    ))
    .bind(id)
    .fetch_all(&mut *tx)
    .await
    .map_err(map_db_error)?;

    let snapshot = entry_snapshot_json(&before_entry, &before_lines);

    // Étape 5 : INSERT audit_log AVANT le DELETE (ordre critique — la
    // trace doit exister avant que la source disparaisse).
    audit_log::insert_in_tx(
        &mut tx,
        NewAuditLogEntry {
            user_id,
            action: "journal_entry.deleted".to_string(),
            entity_type: "journal_entry".to_string(),
            entity_id: id,
            details_json: Some(snapshot),
        },
    )
    .await?;

    // Étape 6 : DELETE (les lignes suivent par CASCADE).
    sqlx::query("DELETE FROM journal_entries WHERE id = ?")
        .bind(id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

    tx.commit().await.map_err(map_db_error)?;
    Ok(())
}

/// Supprime toutes les écritures d'une company (utilisé par `reset_demo`).
///
/// Les lignes suivent par `ON DELETE CASCADE`.
pub async fn delete_all_by_company(pool: &MySqlPool, company_id: i64) -> Result<u64, DbError> {
    let rows = sqlx::query("DELETE FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .execute(pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{NewJournalEntry, NewJournalEntryLine};
    use crate::repositories::{accounts, fiscal_years};
    use chrono::{Datelike, NaiveDate};
    use kesh_core::accounting::Journal as CoreJournal;
    use rust_decimal_macros::dec;

    async fn test_pool() -> MySqlPool {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        MySqlPool::connect(&url).await.expect("DB connect failed")
    }

    /// Nettoie les écritures de test puis retourne
    /// `(company_id, fiscal_year_id, admin_user_id)`.
    ///
    /// Story 3.5 : le `admin_user_id` est récupéré ici et propagé aux
    /// tests pour satisfaire la nouvelle signature de `create` qui
    /// requiert un `user_id` pour l'audit log.
    async fn setup(pool: &MySqlPool) -> (i64, i64, i64) {
        // Récupérer la première company (créée par seed_demo).
        let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies LIMIT 1")
            .fetch_one(pool)
            .await
            .expect("need at least one company in DB for tests (run seed-demo)");

        // Nettoyer les écritures existantes pour éviter les interférences.
        delete_all_by_company(pool, company_id).await.unwrap();

        // Récupérer l'exercice ouvert courant.
        let today = chrono::Utc::now().naive_utc().date();
        let fy = fiscal_years::find_covering_date(pool, company_id, today)
            .await
            .expect("find_covering_date")
            .expect("need fiscal year for today (run seed-demo)");

        // Récupérer l'admin user pour l'audit log (dupliqué depuis
        // audit_log::tests story 3.3 — voir spec 3.5 Dev Notes L1).
        let admin_user_id: i64 =
            sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' LIMIT 1")
                .fetch_one(pool)
                .await
                .expect("need at least one admin user (run seed-demo or bootstrap)");

        (company_id, fy.id, admin_user_id)
    }

    /// Récupère 2 comptes actifs pour les tests (premier actif puis un autre).
    async fn two_accounts(pool: &MySqlPool, company_id: i64) -> (i64, i64) {
        let accs = accounts::list_by_company(pool, company_id, false)
            .await
            .unwrap();
        assert!(accs.len() >= 2, "need ≥ 2 active accounts (run seed-demo)");
        (accs[0].id, accs[1].id)
    }

    fn mk_entry(
        company_id: i64,
        date: NaiveDate,
        lines: Vec<NewJournalEntryLine>,
    ) -> NewJournalEntry {
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal: CoreJournal::Banque.into(),
            description: "Test entry".to_string(),
            lines,
        }
    }

    /// Story 3.5 — vérifie que `create` insère bien une entrée `audit_log`
    /// avec `action = "journal_entry.created"` et un `details_json`
    /// contenant le snapshot direct (PAS de wrapper `{before, after}`).
    #[tokio::test]
    async fn test_create_writes_audit_log() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(42),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(42),
                },
            ],
        );
        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();

        let audit_entries = audit_log::find_by_entity(&pool, "journal_entry", created.entry.id, 10)
            .await
            .unwrap();

        let created_audit = audit_entries
            .iter()
            .find(|e| e.action == "journal_entry.created")
            .expect("audit entry with action journal_entry.created must exist");

        assert_eq!(created_audit.user_id, admin_user_id);
        assert_eq!(created_audit.entity_type, "journal_entry");
        assert_eq!(created_audit.entity_id, created.entry.id);

        let details = created_audit
            .details_json
            .as_ref()
            .expect("details_json must be present");

        // Convention projet : snapshot direct (pas de wrapper {before, after}).
        assert!(
            details.get("before").is_none(),
            "create audit must NOT wrap in {{before, after}} — expected direct snapshot"
        );
        assert!(
            details.get("after").is_none(),
            "create audit must NOT wrap in {{before, after}} — expected direct snapshot"
        );

        // Le snapshot doit contenir les champs clés de l'écriture.
        assert_eq!(
            details.get("description").and_then(|v| v.as_str()),
            Some("Test entry")
        );
        let lines = details
            .get("lines")
            .and_then(|v| v.as_array())
            .expect("lines array must be present in snapshot");
        assert_eq!(lines.len(), 2);
    }

    #[tokio::test]
    async fn test_create_balanced_entry() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;

        let today = chrono::Utc::now().naive_utc().date();
        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                },
            ],
        );

        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
        assert_eq!(created.entry.entry_number, 1);
        assert_eq!(created.lines.len(), 2);
        assert_eq!(created.lines[0].line_order, 1);
        assert_eq!(created.lines[1].line_order, 2);
        assert_eq!(created.lines[0].debit, dec!(100));
        assert_eq!(created.lines[1].credit, dec!(100));
    }

    #[tokio::test]
    async fn test_create_sequential_numbering() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        for expected in 1..=3 {
            let new = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(50),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(50),
                    },
                ],
            );
            let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
            assert_eq!(created.entry.entry_number, expected);
        }
    }

    #[tokio::test]
    async fn test_create_rejects_closed_fiscal_year() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Clore l'exercice (Story 3.7 : signature audit-aware avec user_id +
        // company_id pour défense en profondeur multi-tenant — Code Review F2).
        fiscal_years::close(&pool, admin_user_id, company_id, fy_id)
            .await
            .unwrap();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                },
            ],
        );

        let result = create(&pool, fy_id, admin_user_id, new).await;
        assert!(
            matches!(result, Err(DbError::FiscalYearClosed)),
            "expected FiscalYearClosed, got {:?}",
            result
        );

        // Nettoyer : impossible de rouvrir un exercice clos — on doit
        // supprimer et recréer. Passer par SQL direct pour ce test.
        // P13 : supprimer d'abord les éventuelles écritures référençant
        // cet exercice pour éviter un échec FK RESTRICT si un test
        // concurrent en a inséré (garde-fou défensif).
        delete_all_by_company(&pool, company_id).await.unwrap();
        sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
            .bind(fy_id)
            .execute(&pool)
            .await
            .unwrap();

        // Recréer pour les tests suivants (Story 3.7 : pas d'audit log, contexte test).
        let year = chrono::Utc::now().naive_utc().date().year();
        fiscal_years::create_for_seed(
            &pool,
            crate::entities::NewFiscalYear {
                company_id,
                name: format!("Exercice {year}"),
                start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            },
        )
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_find_covering_date_none() {
        let pool = test_pool().await;
        let (company_id, _fy_id, _admin_user_id) = setup(&pool).await;

        // Date très ancienne — aucun exercice ne la couvre.
        let old = NaiveDate::from_ymd_opt(1900, 1, 1).unwrap();
        let result = fiscal_years::find_covering_date(&pool, company_id, old)
            .await
            .unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_find_covering_date_open() {
        let pool = test_pool().await;
        let (company_id, fy_id, _admin_user_id) = setup(&pool).await;
        let today = chrono::Utc::now().naive_utc().date();

        let result = fiscal_years::find_covering_date(&pool, company_id, today)
            .await
            .unwrap();
        let fy = result.expect("fiscal year should cover today");
        assert_eq!(fy.id, fy_id);
    }

    #[tokio::test]
    async fn test_find_by_id_returns_lines_in_order() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(30),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(50),
                },
            ],
        );

        let created = create(&pool, fy_id, admin_user_id, new).await.unwrap();
        let fetched = find_by_id(&pool, company_id, created.entry.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.lines.len(), 3);
        assert_eq!(fetched.lines[0].line_order, 1);
        assert_eq!(fetched.lines[1].line_order, 2);
        assert_eq!(fetched.lines[2].line_order, 3);
    }

    #[tokio::test]
    async fn test_list_paginated_default() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 3 écritures à la même date.
        for _ in 0..3 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        let result = list_by_company_paginated(&pool, company_id, JournalEntryListQuery::default())
            .await
            .unwrap();

        assert!(result.items.len() >= 3);
        assert!(result.total >= 3);
        assert_eq!(result.offset, 0);
        assert_eq!(result.limit, 50);
        // Tri par entry_number DESC à date égale (secondary sort stable).
        assert!(result.items[0].entry.entry_number > result.items[1].entry.entry_number);
    }

    #[tokio::test]
    async fn test_list_filter_description() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 2 écritures avec descriptions distinctes.
        let mut entry1 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(100),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(100),
                },
            ],
        );
        entry1.description = "Facture fournisseur ABC".to_string();
        create(&pool, fy_id, admin_user_id, entry1).await.unwrap();

        let mut entry2 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(50),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(50),
                },
            ],
        );
        entry2.description = "Virement bancaire XYZ".to_string();
        create(&pool, fy_id, admin_user_id, entry2).await.unwrap();

        // Filtre par « facture ».
        let query = JournalEntryListQuery {
            description: Some("facture".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
        assert!(result.items[0].entry.description.contains("Facture"));

        // Filtre par « virement ».
        let query = JournalEntryListQuery {
            description: Some("virement".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
        assert!(result.items[0].entry.description.contains("Virement"));
    }

    #[tokio::test]
    async fn test_list_filter_description_escapes_percent() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 2 écritures : une avec "50%" dans la description, une avec "50X".
        let mut e1 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(10),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(10),
                },
            ],
        );
        e1.description = "Remise 50% client".to_string();
        create(&pool, fy_id, admin_user_id, e1).await.unwrap();

        let mut e2 = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(20),
                },
            ],
        );
        e2.description = "Compte 50X fournisseur".to_string();
        create(&pool, fy_id, admin_user_id, e2).await.unwrap();

        // Recherche « 50% » — doit matcher UNIQUEMENT la première (le % est
        // échappé et devient un caractère littéral, pas un wildcard).
        let query = JournalEntryListQuery {
            description: Some("50%".to_string()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(
            result.total, 1,
            "Le % user input doit être échappé et ne pas matcher comme wildcard"
        );
        assert!(result.items[0].entry.description.contains("50%"));
    }

    #[tokio::test]
    async fn test_list_filter_amount_range() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // 3 écritures à 100, 500, 1000.
        for amount in [dec!(100), dec!(500), dec!(1000)] {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: amount,
                            credit: dec!(0),
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: amount,
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        // Filtre [200, 800] — doit retourner uniquement 500.
        let query = JournalEntryListQuery {
            amount_min: Some(dec!(200)),
            amount_max: Some(dec!(800)),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 1);
    }

    #[tokio::test]
    async fn test_list_filter_journal() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 2 écritures Banque + 1 Ventes.
        for _ in 0..2 {
            let mut e = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(10),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(10),
                    },
                ],
            );
            e.journal = CoreJournal::Banque.into();
            create(&pool, fy_id, admin_user_id, e).await.unwrap();
        }
        let mut ventes = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(20),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a2,
                    debit: dec!(0),
                    credit: dec!(20),
                },
            ],
        );
        ventes.journal = CoreJournal::Ventes.into();
        create(&pool, fy_id, admin_user_id, ventes).await.unwrap();

        // Filtre Banque → 2 écritures.
        let query = JournalEntryListQuery {
            journal: Some(CoreJournal::Banque.into()),
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert_eq!(result.total, 2);
    }

    #[tokio::test]
    async fn test_list_pagination_offset_limit() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 5 écritures.
        for _ in 0..5 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        // Page 1 : limit=2, offset=0.
        let page1 = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                limit: 2,
                offset: 0,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page1.items.len(), 2);
        assert_eq!(page1.total, 5);
        assert_eq!(page1.limit, 2);
        assert_eq!(page1.offset, 0);

        // Page 2 : limit=2, offset=2.
        let page2 = list_by_company_paginated(
            &pool,
            company_id,
            JournalEntryListQuery {
                limit: 2,
                offset: 2,
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.offset, 2);

        // Les écritures ne se chevauchent pas.
        let page1_ids: Vec<i64> = page1.items.iter().map(|i| i.entry.id).collect();
        let page2_ids: Vec<i64> = page2.items.iter().map(|i| i.entry.id).collect();
        for id in &page2_ids {
            assert!(!page1_ids.contains(id), "pages se chevauchent");
        }
    }

    #[tokio::test]
    async fn test_list_sort_by_entry_number_asc() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        for _ in 0..3 {
            create(
                &pool,
                fy_id,
                admin_user_id,
                mk_entry(
                    company_id,
                    today,
                    vec![
                        NewJournalEntryLine {
                            account_id: a1,
                            debit: dec!(10),
                            credit: dec!(0),
                        },
                        NewJournalEntryLine {
                            account_id: a2,
                            debit: dec!(0),
                            credit: dec!(10),
                        },
                    ],
                ),
            )
            .await
            .unwrap();
        }

        let query = JournalEntryListQuery {
            sort_by: SortBy::EntryNumber,
            sort_dir: SortDirection::Asc,
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        assert!(result.items.len() >= 3);
        // Tri ascendant : 1, 2, 3...
        for i in 0..result.items.len() - 1 {
            assert!(
                result.items[i].entry.entry_number <= result.items[i + 1].entry.entry_number,
                "Tri ascendant cassé"
            );
        }
    }

    #[tokio::test]
    async fn test_list_count_accurate_after_filter() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        // Créer 3 écritures : 2 matchent le filtre, 1 non.
        for desc in ["Match 1", "Match 2", "Autre"] {
            let mut e = mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(10),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(10),
                    },
                ],
            );
            e.description = desc.to_string();
            create(&pool, fy_id, admin_user_id, e).await.unwrap();
        }

        let query = JournalEntryListQuery {
            description: Some("Match".to_string()),
            limit: 1, // limit petit pour forcer la pagination
            ..Default::default()
        };
        let result = list_by_company_paginated(&pool, company_id, query)
            .await
            .unwrap();
        // Total doit refléter TOUTES les matches, pas seulement la page.
        assert_eq!(result.total, 2);
        assert_eq!(result.items.len(), 1);
    }

    #[tokio::test]
    async fn test_check_constraint_rejects_debit_and_credit_same_line() {
        let pool = test_pool().await;
        let (company_id, _fy_id, admin_user_id) = setup(&pool).await;
        let (a1, _a2) = two_accounts(&pool, company_id).await;

        // Créer d'abord une entry valide pour récupérer un entry_id.
        // On va ensuite tenter un INSERT direct d'une ligne invalide.
        let today = chrono::Utc::now().naive_utc().date();
        let new = mk_entry(
            company_id,
            today,
            vec![
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(10),
                    credit: dec!(0),
                },
                NewJournalEntryLine {
                    account_id: a1,
                    debit: dec!(0),
                    credit: dec!(10),
                },
            ],
        );
        let created = create(&pool, _fy_id, admin_user_id, new).await.unwrap();

        // Tentative d'INSERT direct d'une ligne avec debit > 0 ET credit > 0.
        let direct_result = sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, 99, 5, 5)",
        )
        .bind(created.entry.id)
        .bind(a1)
        .execute(&pool)
        .await;

        assert!(direct_result.is_err(), "CHECK constraint should reject");
        let err = map_db_error(direct_result.unwrap_err());
        assert!(
            matches!(err, DbError::CheckConstraintViolation(_)),
            "expected CheckConstraintViolation, got {:?}",
            err
        );
    }

    /// KF-004 : payload identique (header + lignes même ordre + comptes
    /// toujours actifs + FY ouvert) → pas de bump version, `updated_at`
    /// inchangé, mêmes IDs DB pour les lignes (pas de DELETE+INSERT),
    /// pas d'audit_log `journal_entry.updated`.
    #[tokio::test]
    async fn update_no_op_returns_unchanged_entity_no_lines_churn() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(100),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(100),
                    },
                ],
            ),
        )
        .await
        .unwrap();
        let version_initial = created.entry.version;
        let updated_at_initial = created.entry.updated_at;
        let line_ids_initial: Vec<i64> = created.lines.iter().map(|l| l.id).collect();

        // Payload strictement identique reconstruit depuis les `before` lines.
        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            version_initial,
            admin_user_id,
            identical,
        )
        .await
        .unwrap();

        assert_eq!(result.entry.version, version_initial);
        assert_eq!(result.entry.updated_at, updated_at_initial);
        let line_ids_after: Vec<i64> = result.lines.iter().map(|l| l.id).collect();
        assert_eq!(
            line_ids_after, line_ids_initial,
            "no-op : pas de DELETE+INSERT, IDs lignes identiques"
        );

        let count: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'journal_entry' AND entity_id = ? AND action = 'journal_entry.updated'",
        )
        .bind(created.entry.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(count.0, 0);
    }

    /// KF-004 : si l'exercice est clôturé entre la création et l'update no-op,
    /// le check `FiscalYearClosed` rejette AVANT le no-op check (pas de leak).
    #[tokio::test]
    async fn update_no_op_in_closed_fy_returns_fiscal_year_closed() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(50),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(50),
                    },
                ],
            ),
        )
        .await
        .unwrap();

        fiscal_years::close(&pool, admin_user_id, company_id, fy_id)
            .await
            .unwrap();

        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            identical,
        )
        .await;
        assert!(
            matches!(result, Err(DbError::FiscalYearClosed)),
            "expected FiscalYearClosed, got {:?}",
            result
        );

        // Nettoyage (cf. test_create_rejects_closed_fiscal_year).
        delete_all_by_company(&pool, company_id).await.unwrap();
        sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
            .bind(fy_id)
            .execute(&pool)
            .await
            .unwrap();
        let year = chrono::Utc::now().naive_utc().date().year();
        fiscal_years::create_for_seed(
            &pool,
            crate::entities::NewFiscalYear {
                company_id,
                name: format!("Exercice {year}"),
                start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
                end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
            },
        )
        .await
        .unwrap();
    }

    /// KF-004 : si un compte référencé par l'écriture a été archivé entre la
    /// création et l'update no-op, le check d'intégrité rejette AVANT le no-op
    /// check (pas de leak via no-op).
    #[tokio::test]
    async fn update_no_op_with_inactive_account_returns_inactive_error() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(75),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(75),
                    },
                ],
            ),
        )
        .await
        .unwrap();

        // Archiver a1 directement en SQL (la fonction archive() exige de ne
        // pas avoir de sous-comptes ; on évite cette vérification ici car
        // elle est orthogonale au scope du test).
        sqlx::query("UPDATE accounts SET active = FALSE, version = version + 1 WHERE id = ?")
            .bind(a1)
            .execute(&pool)
            .await
            .unwrap();

        let identical = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                })
                .collect(),
        };

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            created.entry.version,
            admin_user_id,
            identical,
        )
        .await;
        assert!(
            matches!(result, Err(DbError::InactiveOrInvalidAccounts)),
            "expected InactiveOrInvalidAccounts, got {:?}",
            result
        );

        // Réactiver le compte pour les tests suivants.
        sqlx::query("UPDATE accounts SET active = TRUE WHERE id = ?")
            .bind(a1)
            .execute(&pool)
            .await
            .unwrap();
        delete_all_by_company(&pool, company_id).await.unwrap();
    }

    /// KF-004 régression : modifier la `description` → bump version.
    #[tokio::test]
    async fn update_partial_change_bumps_version() {
        let pool = test_pool().await;
        let (company_id, fy_id, admin_user_id) = setup(&pool).await;
        let (a1, a2) = two_accounts(&pool, company_id).await;
        let today = chrono::Utc::now().naive_utc().date();

        let created = create(
            &pool,
            fy_id,
            admin_user_id,
            mk_entry(
                company_id,
                today,
                vec![
                    NewJournalEntryLine {
                        account_id: a1,
                        debit: dec!(33),
                        credit: dec!(0),
                    },
                    NewJournalEntryLine {
                        account_id: a2,
                        debit: dec!(0),
                        credit: dec!(33),
                    },
                ],
            ),
        )
        .await
        .unwrap();
        let version_initial = created.entry.version;

        let mut payload = NewJournalEntry {
            company_id,
            entry_date: created.entry.entry_date,
            journal: created.entry.journal,
            description: created.entry.description.clone(),
            lines: created
                .lines
                .iter()
                .map(|l| NewJournalEntryLine {
                    account_id: l.account_id,
                    debit: l.debit,
                    credit: l.credit,
                })
                .collect(),
        };
        payload.description = "Description modifiée".into();

        let result = update(
            &pool,
            company_id,
            created.entry.id,
            version_initial,
            admin_user_id,
            payload,
        )
        .await
        .unwrap();
        assert_eq!(result.entry.version, version_initial + 1);
        assert_eq!(result.entry.description, "Description modifiée");
    }
}
