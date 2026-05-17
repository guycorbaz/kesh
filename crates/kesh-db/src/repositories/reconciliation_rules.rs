//! Repository pour `reconciliation_rules` — règles d'affectation
//! automatique (Story 8-5b FR47).
//!
//! Signatures `Executor` génériques pour les lectures (`find_*`) :
//! permettent un appel transaction-bound depuis le handler
//! `accept-with-rule` (cf. §accept-with-rule-flow step 7
//! re-validation match côté serveur anti-race).
//!
//! Soft-delete via `active=false` (Q3 décision Guy 2026-05-07) + UNIQUE
//! partiel via colonne `active_uniq` VIRTUAL (cf. migration
//! `20260513000001_reconciliation_rules.sql`). Le repository ne crée pas
//! de variant `DbError::DuplicateRule` (Pass 4 LOW#5) — la détection du
//! conflit UNIQUE sur `uq_reconciliation_rules_match_active` est faite
//! côté caller via [`is_duplicate_rule_constraint`] qui parse le message
//! de `DbError::UniqueConstraintViolation`.

use sqlx::mysql::MySqlPool;
use sqlx::{Executor, MySql, Transaction};

use crate::entities::audit_log::NewAuditLogEntry;
use crate::entities::reconciliation_rule::{
    NewReconciliationRule, ReconciliationRule, UpdateReconciliationRule,
};
use crate::errors::{DbError, map_db_error};
use crate::repositories::audit_log;

const COLUMNS: &str = "id, company_id, label, match_type, match_value, counterparty_account_id, \
     priority, active, applied_count, last_applied_at, version, created_at, updated_at";

/// Détecte si un [`DbError::UniqueConstraintViolation`] provient de la
/// contrainte `uq_reconciliation_rules_match_active`. Le message MariaDB
/// pour SQL 1062 contient le nom de la contrainte
/// (e.g. `Duplicate entry '...' for key 'uq_reconciliation_rules_match_active'`),
/// ce qui permet la discrimination depuis le caller pour traduire en
/// `409 RECONCILIATION_RULE_DUPLICATE` (vs autre `UniqueConstraintViolation`
/// inattendu qui serait 500).
pub fn is_duplicate_rule_constraint(err: &DbError) -> bool {
    matches!(
        err,
        DbError::UniqueConstraintViolation(msg)
            if msg.contains("uq_reconciliation_rules_match_active")
    )
}

/// Story 9-2b T3.2.9 — Liste **toutes** les règles de réconciliation d'une
/// company sans filtre `active`, pour l'export global ZIP (souveraineté :
/// règles soft-deleted incluses pour audit historique).
///
/// Distincte de [`find_active_for_company`] (filtre `active = TRUE`). Tri
/// stable `id ASC`.
pub async fn list_all_by_company(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<Vec<ReconciliationRule>, DbError> {
    sqlx::query_as::<_, ReconciliationRule>(&format!(
        "SELECT {COLUMNS} FROM reconciliation_rules \
         WHERE company_id = ? \
         ORDER BY id"
    ))
    .bind(company_id)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)
}

/// Liste toutes les rules **actives** d'une company, ordonnées par
/// `priority ASC, id ASC` (tiebreaker à 2 niveaux Pass 2 Q1, suppression
/// du 3e niveau `created_at ASC` régression Pass 1 — `id ASC` suffit
/// comme tiebreaker total puisque `id` est AUTO_INCREMENT unique).
///
/// Utilisé par `kesh_reconciliation::rules::first_matching_rule` au
/// flow `GET /api/v1/reconciliation/proposals`.
pub async fn find_active_for_company<'e, E>(
    executor: E,
    company_id: i64,
) -> Result<Vec<ReconciliationRule>, DbError>
where
    E: Executor<'e, Database = MySql>,
{
    let query = format!(
        "SELECT {COLUMNS} FROM reconciliation_rules \
         WHERE company_id = ? AND active = TRUE \
         ORDER BY priority ASC, id ASC"
    );
    sqlx::query_as::<_, ReconciliationRule>(&query)
        .bind(company_id)
        .fetch_all(executor)
        .await
        .map_err(map_db_error)
}

/// Liste paginée des rules d'une company, avec filtre optionnel sur
/// `active`. Retourne `(rules, count_total)` pour exposer le total au
/// client (cf. spec §pagination AC #105).
///
/// `per_page` est borné par [`MAX_LIST_LIMIT`](crate::repositories::MAX_LIST_LIMIT).
/// `page` est 1-indexé (offset = (page-1) * per_page).
pub async fn list_by_company_paginated(
    pool: &MySqlPool,
    company_id: i64,
    active_filter: Option<bool>,
    page: i64,
    per_page: i64,
) -> Result<(Vec<ReconciliationRule>, i64), DbError> {
    let per_page = per_page.clamp(1, crate::repositories::MAX_LIST_LIMIT);
    let page = page.max(1);
    let offset = (page - 1) * per_page;

    let where_active = match active_filter {
        Some(true) => " AND active = TRUE",
        Some(false) => " AND active = FALSE",
        None => "",
    };

    let select_sql = format!(
        "SELECT {COLUMNS} FROM reconciliation_rules \
         WHERE company_id = ?{where_active} \
         ORDER BY priority ASC, id ASC LIMIT ? OFFSET ?"
    );
    let count_sql =
        format!("SELECT COUNT(*) FROM reconciliation_rules WHERE company_id = ?{where_active}");

    let rules = sqlx::query_as::<_, ReconciliationRule>(&select_sql)
        .bind(company_id)
        .bind(per_page)
        .bind(offset)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;

    let count: i64 = sqlx::query_scalar(&count_sql)
        .bind(company_id)
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;

    Ok((rules, count))
}

/// Cherche une rule par id **scopée multi-tenant** (pattern KF-002 —
/// `None` pour cross-tenant, pas de leak d'existence). Signature
/// `Executor` générique pour appel transaction-bound (re-validation
/// match au flow accept-with-rule).
pub async fn find_by_id_for_company<'e, E>(
    executor: E,
    company_id: i64,
    id: i64,
) -> Result<Option<ReconciliationRule>, DbError>
where
    E: Executor<'e, Database = MySql>,
{
    let query =
        format!("SELECT {COLUMNS} FROM reconciliation_rules WHERE id = ? AND company_id = ?");
    sqlx::query_as::<_, ReconciliationRule>(&query)
        .bind(id)
        .bind(company_id)
        .fetch_optional(executor)
        .await
        .map_err(map_db_error)
}

/// Crée une rule dans la transaction `tx` + écrit l'entrée audit_log
/// associée. Le caller commit la transaction.
///
/// # Erreurs
///
/// - `DbError::UniqueConstraintViolation(msg)` quand la contrainte
///   `uq_reconciliation_rules_match_active` est violée (i.e. une rule
///   active existe déjà avec mêmes `(company_id, match_type,
///   match_value)`). Le caller utilise [`is_duplicate_rule_constraint`]
///   pour discriminer et traduire en 409 `RECONCILIATION_RULE_DUPLICATE`.
/// - `DbError::CheckConstraintViolation` sur CHECKs DB (label/match_value
///   empty, priority hors range) — défense-en-profondeur, normalement
///   filtré par le handler.
/// - `DbError::ForeignKeyViolation` si `counterparty_account_id`
///   référence un account inexistant.
pub async fn create_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    user_id: i64,
    new_rule: &NewReconciliationRule,
) -> Result<ReconciliationRule, DbError> {
    let result = sqlx::query(
        "INSERT INTO reconciliation_rules \
         (company_id, label, match_type, match_value, counterparty_account_id, priority) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(&new_rule.label)
    .bind(new_rule.match_type)
    .bind(&new_rule.match_value)
    .bind(new_rule.counterparty_account_id)
    .bind(new_rule.priority)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let last_id = result.last_insert_id();
    if last_id == 0 {
        return Err(DbError::Invariant(
            "last_insert_id == 0 après INSERT reconciliation_rules".into(),
        ));
    }
    let id = i64::try_from(last_id)
        .map_err(|_| DbError::Invariant(format!("last_insert_id {last_id} dépasse i64::MAX")))?;

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "reconciliation_rule.created".to_string(),
            entity_type: "reconciliation_rules".to_string(),
            entity_id: id,
            details_json: Some(serde_json::json!({
                "rule_id": id,
                "label": new_rule.label,
                "match_type": new_rule.match_type.as_str(),
                "match_value": new_rule.match_value,
                "counterparty_account_id": new_rule.counterparty_account_id,
                "priority": new_rule.priority,
                "active": true,
            })),
        },
    )
    .await?;

    find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!("reconciliation_rule {id} introuvable après INSERT"))
        })
}

/// Patch partiel + optimistic lock + audit log atomique.
///
/// # Erreurs
///
/// - `DbError::NotFound` si la rule n'existe pas (ou cross-tenant).
/// - `DbError::OptimisticLockConflict` si `expected_version != current_version`
///   (i.e. le row a été modifié entre le SELECT du client et le UPDATE).
/// - `DbError::UniqueConstraintViolation` si la patch déclenche un
///   conflit UNIQUE de réactivation (cf. AC #109b — R1 soft-deleted +
///   R2 active créée entre-temps + PATCH R1 `active=true`). Le caller
///   discrimine via [`is_duplicate_rule_constraint`].
pub async fn update_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    user_id: i64,
    id: i64,
    expected_version: i32,
    patch: &UpdateReconciliationRule,
) -> Result<ReconciliationRule, DbError> {
    let before = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or(DbError::NotFound)?;

    let mut set_clauses: Vec<&'static str> = Vec::new();
    if patch.label.is_some() {
        set_clauses.push("label = ?");
    }
    if patch.match_value.is_some() {
        set_clauses.push("match_value = ?");
    }
    if patch.counterparty_account_id.is_some() {
        set_clauses.push("counterparty_account_id = ?");
    }
    if patch.priority.is_some() {
        set_clauses.push("priority = ?");
    }
    if patch.active.is_some() {
        set_clauses.push("active = ?");
    }
    // version est toujours incrémentée (sauf si patch totalement vide,
    // auquel cas on évite un UPDATE inutile). On n'incrémente pas non
    // plus si rien à patcher pour ne pas perturber l'optimistic lock
    // côté client suivant.
    if set_clauses.is_empty() {
        return Ok(before);
    }
    set_clauses.push("version = version + 1");

    let set_sql = set_clauses.join(", ");
    let sql = format!(
        "UPDATE reconciliation_rules SET {set_sql} \
         WHERE id = ? AND company_id = ? AND version = ?"
    );

    let mut q = sqlx::query(&sql);
    if let Some(ref v) = patch.label {
        q = q.bind(v);
    }
    if let Some(ref v) = patch.match_value {
        q = q.bind(v);
    }
    if let Some(v) = patch.counterparty_account_id {
        q = q.bind(v);
    }
    if let Some(v) = patch.priority {
        q = q.bind(v);
    }
    if let Some(v) = patch.active {
        q = q.bind(v);
    }
    q = q.bind(id).bind(company_id).bind(expected_version);

    let result = q.execute(&mut **tx).await.map_err(map_db_error)?;
    if result.rows_affected() == 0 {
        return Err(DbError::OptimisticLockConflict);
    }

    let after = find_by_id_for_company(&mut **tx, company_id, id)
        .await?
        .ok_or_else(|| {
            DbError::Invariant(format!("reconciliation_rule {id} disparue post-UPDATE"))
        })?;

    audit_log::insert_in_tx(
        tx,
        NewAuditLogEntry {
            user_id,
            action: "reconciliation_rule.updated".to_string(),
            entity_type: "reconciliation_rules".to_string(),
            entity_id: id,
            details_json: Some(serde_json::json!({
                "rule_id": id,
                "before": {
                    "label": before.label,
                    "match_value": before.match_value,
                    "priority": before.priority,
                    "active": before.active,
                },
                "after": {
                    "label": after.label,
                    "match_value": after.match_value,
                    "priority": after.priority,
                    "active": after.active,
                },
            })),
        },
    )
    .await?;

    Ok(after)
}

/// Soft-delete : `UPDATE active = FALSE` + audit log. **Idempotent** —
/// si la rule est déjà inactive, retourne `Ok(false)` sans erreur et
/// sans nouvelle entrée audit (AC #111).
///
/// Retourne `Ok(true)` quand la transition active→inactive a eu lieu
/// (AC #110), `Ok(false)` quand la rule était déjà inactive ou n'existe
/// pas (les deux cas sont indiscernables côté repository ; le handler
/// `DELETE /rules/{id}` doit faire un `find_by_id_for_company` au
/// préalable s'il veut distinguer 204 vs 404).
pub async fn soft_delete_by_id_for_company(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    user_id: i64,
    id: i64,
) -> Result<bool, DbError> {
    let result = sqlx::query(
        "UPDATE reconciliation_rules SET active = FALSE, version = version + 1 \
         WHERE id = ? AND company_id = ? AND active = TRUE",
    )
    .bind(id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let transitioned = result.rows_affected() > 0;
    if transitioned {
        audit_log::insert_in_tx(
            tx,
            NewAuditLogEntry {
                user_id,
                action: "reconciliation_rule.deleted".to_string(),
                entity_type: "reconciliation_rules".to_string(),
                entity_id: id,
                details_json: Some(serde_json::json!({
                    "rule_id": id,
                    "soft_delete": true,
                    "before": { "active": true },
                    "after": { "active": false },
                })),
            },
        )
        .await?;
    }

    Ok(transitioned)
}

/// Incrémente `applied_count` + `last_applied_at = NOW(3)` + bumps
/// `version`. **Pas de WHERE clause sur version** (Pass 1 P-M ECH-10) :
/// `applied_count` est un compteur statistique, donc l'UPDATE ne doit
/// jamais échouer pour cause d'optimistic lock conflict avec un PATCH
/// /rules concurrent. **Mais `version` doit être bumped** (Pass 1 code
/// review HIGH AA1 — corrigé) : spec step 14 + scope-locked §5 + T2.2
/// confirment 3 fois que `version=version+1` est requis dans le SET
/// pour invalider les sessions édition utilisateur en cours et garder
/// les audit logs cohérents. Sans le bump, un PATCH après un accept
/// verrait une version stale et ne déclencherait pas OPTIMISTIC_LOCK_CONFLICT.
///
/// Retourne `applied_count` post-incrément pour le caller (utilisé
/// dans l'audit `reconciliation_rule.applied` field `applied_count_after`).
pub async fn increment_applied_count_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    rule_id: i64,
) -> Result<i64, DbError> {
    sqlx::query(
        "UPDATE reconciliation_rules \
         SET applied_count = applied_count + 1, \
             last_applied_at = NOW(3), \
             version = version + 1 \
         WHERE id = ? AND company_id = ?",
    )
    .bind(rule_id)
    .bind(company_id)
    .execute(&mut **tx)
    .await
    .map_err(map_db_error)?;

    let applied_count: i64 = sqlx::query_scalar(
        "SELECT applied_count FROM reconciliation_rules WHERE id = ? AND company_id = ?",
    )
    .bind(rule_id)
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)?;
    Ok(applied_count)
}
