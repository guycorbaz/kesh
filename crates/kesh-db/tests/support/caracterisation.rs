//! Story 48-14 (#1255, AC9) — l'état que laisse un dé-rapprochement groupé,
//! relevé champ par champ pour les tests de caractérisation des bras client
//! (`grouped_settlements.rs`) et fournisseur (`grouped_settlements_supplier.rs`).
//!
//! Chaque relevé rend une `serde_json::Value` que le test compare **en entier**
//! à une valeur écrite en dur : la réécriture du dé-rapprochement (#1255) doit
//! laisser exactement l'état que laissait l'ancien code. Les montants passent
//! par `Decimal::to_string` (échelle de la colonne comprise), les dates par
//! leur forme ISO.
//!
//! `tests/support/` n'a pas de `mod.rs` : un binaire l'inclut par
//! `#[path = "support/caracterisation.rs"] mod caracterisation;`.

use chrono::{NaiveDate, NaiveDateTime};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sqlx::MySqlPool;

/// La transaction bancaire : `status`, `matched_entry_id`,
/// `auto_match_rejected_at`, `version`.
pub async fn transaction_bancaire(pool: &MySqlPool, id: i64) -> Value {
    let (status, matched, rejected, version): (String, Option<i64>, Option<NaiveDateTime>, i32) =
        sqlx::query_as(
            "SELECT status, matched_entry_id, auto_match_rejected_at, version \
             FROM bank_transactions WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .expect("transaction bancaire");
    json!({
        "status": status,
        "matched_entry_id": matched,
        "auto_match_rejected_at": rejected.map(|d| d.to_string()),
        "version": version,
    })
}

/// Une écriture : son en-tête (numéro, date, exercice, journal, libellé,
/// écriture contre-passée, version) et ses lignes dans l'ordre (numéro du
/// compte, débit, crédit, projet, clé et origine du lettrage).
pub async fn ecriture(pool: &MySqlPool, id: i64) -> Value {
    #[allow(clippy::type_complexity)]
    let (number, date, fy, journal, description, reverses, version): (
        i64,
        NaiveDate,
        i64,
        String,
        String,
        Option<i64>,
        i32,
    ) = sqlx::query_as(
        "SELECT entry_number, entry_date, fiscal_year_id, journal, description, \
                reverses_entry_id, version \
         FROM journal_entries WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .expect("écriture");
    #[allow(clippy::type_complexity)]
    let lignes: Vec<(i32, String, Decimal, Decimal, Option<i64>, Option<i64>, Option<String>)> =
        sqlx::query_as(
            "SELECT l.line_order, a.number, l.debit, l.credit, l.project_id, \
                    l.lettering_key, l.lettering_origin \
             FROM journal_entry_lines l JOIN accounts a ON a.id = l.account_id \
             WHERE l.entry_id = ? ORDER BY l.line_order",
        )
        .bind(id)
        .fetch_all(pool)
        .await
        .expect("lignes");
    json!({
        "entry_number": number,
        "entry_date": date.to_string(),
        "fiscal_year_id": fy,
        "journal": journal,
        "description": description,
        "reverses_entry_id": reverses,
        "version": version,
        "lines": lignes
            .into_iter()
            .map(|(order, account, debit, credit, project, key, origin)| json!({
                "line_order": order,
                "account": account,
                "debit": debit.to_string(),
                "credit": credit.to_string(),
                "project_id": project,
                "lettering_key": key,
                "lettering_origin": origin,
            }))
            .collect::<Vec<_>>(),
    })
}

/// L'identifiant d'une ligne d'écriture, par sa position.
pub async fn ligne(pool: &MySqlPool, entry_id: i64, line_order: i32) -> i64 {
    sqlx::query_scalar("SELECT id FROM journal_entry_lines WHERE entry_id = ? AND line_order = ?")
        .bind(entry_id)
        .bind(line_order)
        .fetch_one(pool)
        .await
        .expect("ligne d'écriture")
}

/// La plus grande ligne d'audit.
pub async fn dernier_audit(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COALESCE(MAX(id), 0) FROM audit_log")
        .fetch_one(pool)
        .await
        .expect("dernier audit")
}

/// **Toutes** les lignes d'audit d'identifiant supérieur à `avant`, dans
/// l'ordre des identifiants, entières hors horodatage et identifiant.
pub async fn audits_depuis(pool: &MySqlPool, avant: i64) -> Vec<Value> {
    #[allow(clippy::type_complexity)]
    let lignes: Vec<(
        String,
        String,
        i64,
        i64,
        String,
        Option<i64>,
        String,
        Option<i64>,
        Option<Value>,
    )> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, user_id, CAST(actor_type AS CHAR), \
                actor_api_key_id, actor_label, company_id, details_json \
         FROM audit_log WHERE id > ? ORDER BY id",
    )
    .bind(avant)
    .fetch_all(pool)
    .await
    .expect("audits");
    lignes
        .into_iter()
        .map(
            |(action, entity_type, entity_id, user, actor_type, key, label, company, details)| {
                json!({
                    "action": action,
                    "entity_type": entity_type,
                    "entity_id": entity_id,
                    "user_id": user,
                    "actor_type": actor_type,
                    "actor_api_key_id": key,
                    "actor_label": label,
                    "company_id": company,
                    "details_json": details,
                })
            },
        )
        .collect()
}

/// Une clé d'API de la société, créée par `user_id` : l'acteur du geste, pour
/// que `actor_api_key_id` soit non nul.
pub async fn cle_api(pool: &MySqlPool, company_id: i64, user_id: i64) -> i64 {
    sqlx::query(
        "INSERT INTO api_keys (company_id, created_by_user_id, name, key_hash, scope) \
         VALUES (?, ?, 'cle-48-14', SHA2('cle-48-14', 256), 'read-write')",
    )
    .bind(company_id)
    .bind(user_id)
    .execute(pool)
    .await
    .expect("clé d'API")
    .last_insert_id() as i64
}
