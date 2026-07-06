//! Story 17-3a — Brique DB du backup complet d'installation (`.keshbackup`).
//!
//! Ce module est la **source de vérité unique** de :
//! - [`TABLES_TO_TRUNCATE`] : la liste canonique ordonnée (enfants → parents)
//!   des tables applicatives Kesh. Promue ici depuis `test_fixtures.rs`
//!   (Story 17-3a, validate Pass 3 O-6/O-10) car l'export d'installation en a
//!   besoin pour énumérer les tables, et `test_fixtures::truncate_all` la
//!   réutilise. La sous-story import 17-3c y ajoutera le restore transactionnel.
//! - [`export_table`] : sérialiseur **générique dynamique** d'une table en
//!   NDJSON (1 objet JSON par ligne), avec liste de colonnes explicite
//!   **excluant les colonnes générées** (ex. `reconciliation_rules.active_uniq`
//!   `GENERATED … VIRTUAL`, non réinsérable à l'import).
//!
//! Le format NDJSON est figé par la spec parente 17-3 (§Format normatif) :
//! UTF-8, fin de ligne LF, NULL → `null`, ordre des clés = `column_names`,
//! `Decimal` → string, `NaiveDateTime` → ISO 8601 `YYYY-MM-DDTHH:MM:SS[.mmm]`
//! (précision sub-seconde préservée — colonnes `DATETIME(3)`), entiers → nombre JSON.

use std::collections::BTreeMap;

use serde_json::Value;
use sqlx::{MySql, MySqlPool, Row, Transaction};

use crate::errors::{DbError, map_db_error};

/// Liste canonique des tables applicatives Kesh, ordonnée **enfants → parents**
/// (ordre de suppression FK-safe). Exclut les tables système `_sqlx_migrations`
/// et `_kesh_version` (jamais exportées ni vidées).
///
/// Source de vérité unique : `test_fixtures::truncate_all` (vidage de test) et
/// l'export d'installation 17-3a la réutilisent. Le test
/// [`tests::backup_inventory_matches_schema`] garantit la synchro avec le schéma
/// réel à chaque migration.
pub const TABLES_TO_TRUNCATE: &[&str] = &[
    "credit_note_lines",      // Story 12.1 — enfant de credit_notes (CASCADE).
    "supplier_invoice_lines", // Story 12.2 — enfant de supplier_invoices (CASCADE).
    "payment_batch_items", // Story 12.3 — enfant de payment_batches (CASCADE) + supplier_invoices (RESTRICT) → avant eux.
    "invoice_lines",
    "journal_entry_lines",
    "projects", // Story 19-1 (Epic 19) — enfant de companies + self-ref parent_id (RESTRICT) ; référencé par journal_entry_lines.project_id → APRÈS journal_entry_lines, AVANT companies.
    "credit_notes", // Story 12.1 — enfant de invoices/journal_entries/contacts (RESTRICT) → avant eux.
    "imported_supplier_invoices", // Story 12.5b — enfant de companies (RESTRICT) + supplier_invoices (SET NULL) → avant eux.
    "supplier_invoices", // Story 12.2 — enfant de journal_entries/contacts/bank_accounts/accounts (RESTRICT) → avant eux.
    "payment_batches",   // Story 12.3 — enfant de companies + bank_accounts (RESTRICT) → avant eux.
    "invoices",
    "invoice_number_sequences",
    "credit_note_number_sequences", // Story 12.1 — enfant de companies + fiscal_years (RESTRICT).
    "journal_entries",
    "audit_log",
    "api_keys", // Story 17-2a (#100) — enfant de companies + users (RESTRICT).
    "company_invoice_settings",
    "bank_transactions", // Story 8-1b — enfant de bank_imports + bank_accounts.
    "bank_imports",      // Story 8-1b — enfant de bank_accounts + companies.
    "bank_profiles",     // Story 8-2 — enfant de companies (CASCADE).
    "reconciliation_rules", // Story 8-5b — enfant de companies + accounts (RESTRICT).
    "bank_accounts",
    "accounts", // FK self-ref via parent_id
    "products",
    "contact_persons", // #213 — enfant de contacts + companies (CASCADE) ; avant `contacts`.
    "contacts",
    "fiscal_years",
    "vat_rates", // Story 7.2 (KF-003) — table enfant de companies (FK fk_vat_rates_company), aucune table ne référence vat_rates.
    "refresh_tokens",
    "password_reset_tokens", // Story 17-4a (#122) — enfant de users (CASCADE), tokens recovery éphémères.
    "onboarding_state",
    "users",
    "companies",
];

/// Résultat de la sérialisation NDJSON d'une table.
#[derive(Debug, Clone)]
pub struct TableExport {
    /// Colonnes sérialisées (ordonnées, **hors colonnes générées**). C'est la
    /// liste reportée dans le manifeste (`columnNames`) et utilisée par l'import
    /// 17-3c pour construire les `INSERT` paramétrés.
    pub column_names: Vec<String>,
    /// Nombre de lignes exportées.
    pub row_count: usize,
    /// Contenu NDJSON (UTF-8, LF) — 0 octet si la table est vide.
    pub ndjson: Vec<u8>,
}

/// Lit les colonnes **non générées** d'une table, ordonnées par position.
///
/// `EXTRA NOT LIKE '%GENERATED%'` exclut les colonnes `VIRTUAL`/`STORED`
/// GENERATED (réinsertion interdite). Les colonnes `auto_increment` sont
/// conservées (on exporte/réimporte les `id` explicites).
async fn non_generated_columns(
    pool: &MySqlPool,
    table: &str,
) -> Result<Vec<(String, String)>, DbError> {
    let rows = sqlx::query_as::<_, (String, String)>(
        "SELECT COLUMN_NAME, DATA_TYPE \
         FROM INFORMATION_SCHEMA.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? \
           AND (EXTRA IS NULL OR EXTRA NOT LIKE '%GENERATED%') \
         ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;
    Ok(rows)
}

/// Sérialise toutes les lignes d'une table applicative en NDJSON.
///
/// - Énumère les colonnes non générées (ordre `ORDINAL_POSITION`).
/// - `SELECT <colonnes> FROM <table>` (identifiants back-quotés ; `table` issu
///   de [`TABLES_TO_TRUNCATE`], `column_name` de `INFORMATION_SCHEMA` — aucune
///   donnée utilisateur dans le SQL).
/// - Décode chaque cellule selon son `DATA_TYPE` (fidélité de type, NULL → `null`).
///
/// L'ordre des clés de chaque objet JSON suit `column_names` (déterministe,
/// garantit la stabilité du SHA-256 entre export et import).
pub async fn export_table(pool: &MySqlPool, table: &str) -> Result<TableExport, DbError> {
    let columns = non_generated_columns(pool, table).await?;
    if columns.is_empty() {
        return Err(DbError::Invariant(format!(
            "table '{table}' has no non-generated columns (unexpected)"
        )));
    }
    let column_names: Vec<String> = columns.iter().map(|(name, _)| name.clone()).collect();

    // Défense en profondeur (review 17-3a Pass 1) : bien que les noms viennent
    // d'INFORMATION_SCHEMA (source SGBD) et que MariaDB n'autorise pas de
    // back-quote nu dans un identifiant, on rejette explicitement tout `\`` qui
    // casserait le back-quoting du SELECT (panne, pas injection).
    if let Some((name, _)) = columns.iter().find(|(n, _)| n.contains('`')) {
        return Err(DbError::Invariant(format!(
            "table '{table}' column '{name}' contains a backtick (unsupported)"
        )));
    }

    // SELECT explicite (back-quote des identifiants).
    let select_cols = column_names
        .iter()
        .map(|c| format!("`{c}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!("SELECT {select_cols} FROM `{table}`");

    let rows = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .map_err(map_db_error)?;
    let row_count = rows.len();

    let mut ndjson = Vec::<u8>::new();
    for row in &rows {
        let mut line = String::from("{");
        for (idx, (name, data_type)) in columns.iter().enumerate() {
            if idx > 0 {
                line.push(',');
            }
            let value = decode_cell(row, idx, data_type)?;
            // Clé JSON correctement échappée + valeur sérialisée serde_json.
            line.push_str(
                &serde_json::to_string(name)
                    .map_err(|e| DbError::Invariant(format!("json key encode: {e}")))?,
            );
            line.push(':');
            line.push_str(
                &serde_json::to_string(&value)
                    .map_err(|e| DbError::Invariant(format!("json value encode: {e}")))?,
            );
        }
        line.push('}');
        line.push('\n');
        ndjson.extend_from_slice(line.as_bytes());
    }

    Ok(TableExport {
        column_names,
        row_count,
        ndjson,
    })
}

/// Décode une cellule d'une ligne MySQL en [`serde_json::Value`] selon son
/// `DATA_TYPE` (INFORMATION_SCHEMA, minuscule).
///
/// Utilise `try_get_unchecked` : on connaît le type via INFORMATION_SCHEMA, on
/// contourne donc la vérification de compatibilité de sqlx (qui rejetterait par
/// ex. un `TINYINT` lu en `i64`). NULL → [`Value::Null`].
fn decode_cell(row: &sqlx::mysql::MySqlRow, idx: usize, data_type: &str) -> Result<Value, DbError> {
    let val = match data_type {
        "bigint" | "int" | "integer" | "smallint" | "mediumint" | "tinyint" => {
            match row
                .try_get_unchecked::<Option<i64>, _>(idx)
                .map_err(map_db_error)?
            {
                Some(n) => Value::from(n),
                None => Value::Null,
            }
        }
        "decimal" | "numeric" => {
            match row
                .try_get_unchecked::<Option<rust_decimal::Decimal>, _>(idx)
                .map_err(map_db_error)?
            {
                Some(d) => Value::String(d.to_string()),
                None => Value::Null,
            }
        }
        "double" | "float" => {
            match row
                .try_get_unchecked::<Option<f64>, _>(idx)
                .map_err(map_db_error)?
            {
                Some(f) => serde_json::Number::from_f64(f)
                    .map(Value::Number)
                    .unwrap_or(Value::Null),
                None => Value::Null,
            }
        }
        "date" => match row
            .try_get_unchecked::<Option<chrono::NaiveDate>, _>(idx)
            .map_err(map_db_error)?
        {
            Some(d) => Value::String(d.format("%Y-%m-%d").to_string()),
            None => Value::Null,
        },
        "datetime" | "timestamp" => match row
            .try_get_unchecked::<Option<chrono::NaiveDateTime>, _>(idx)
            .map_err(map_db_error)?
        {
            // `%.f` préserve la précision sub-seconde sans trailing zeros et
            // l'omet si nulle. **Toutes** les colonnes timestamp Kesh sont
            // `DATETIME(3)` (millisecondes) — un format sec-only tronquerait
            // (review 17-3a Pass 1, fidélité backup). Round-trip : MariaDB
            // reparse `…:SS.mmm` ou `…:SS` indifféremment.
            Some(dt) => Value::String(dt.format("%Y-%m-%dT%H:%M:%S%.f").to_string()),
            None => Value::Null,
        },
        // varchar, char, text*, enum, set, et JSON (reporté `longtext` par
        // MariaDB) → string (round-trip exact, l'import re-bind la string).
        _ => match row
            .try_get_unchecked::<Option<String>, _>(idx)
            .map_err(map_db_error)?
        {
            Some(s) => Value::String(s),
            None => Value::Null,
        },
    };
    Ok(val)
}

/// Lit `kesh_version_min_required` de la row singleton `_kesh_version`.
pub async fn read_min_required(pool: &MySqlPool) -> Result<String, DbError> {
    sqlx::query_scalar::<_, String>(
        "SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(pool)
    .await
    .map_err(map_db_error)
}

/// Identifiant informatif de l'installation source = `MIN(companies.id)`.
///
/// Invariant : ≥ 1 (l'endpoint export exige un Admin authentifié, et
/// `users.company_id` est `NOT NULL` ⇒ au moins une company existe). Retourne 0
/// défensivement si la table est vide (cas dégénéré impossible en pratique).
pub async fn read_instance_id(pool: &MySqlPool) -> Result<i64, DbError> {
    let v: Option<i64> = sqlx::query_scalar("SELECT MIN(id) FROM companies")
        .fetch_one(pool)
        .await
        .map_err(map_db_error)?;
    Ok(v.unwrap_or(0))
}

// ===========================================================================
// Story 17-3c — Brique import (lecture NDJSON, contraintes colonnes, restore
// transactionnel). Symétrique de l'export ci-dessus.
// ===========================================================================

/// Parse un NDJSON (1 objet JSON par ligne) en lignes de valeurs **ordonnées
/// selon `column_names`** — symétrique de [`export_table`] (qui écrit les clés
/// dans cet ordre). Une clé absente d'une ligne → [`Value::Null`] (tolérance
/// forward-compat). Les lignes vides sont ignorées.
pub fn parse_ndjson_rows(
    ndjson: &[u8],
    column_names: &[String],
) -> Result<Vec<Vec<Value>>, DbError> {
    let text = std::str::from_utf8(ndjson)
        .map_err(|e| DbError::Invariant(format!("NDJSON non-UTF8: {e}")))?;
    let mut rows = Vec::new();
    for (lineno, line) in text.lines().enumerate() {
        if line.is_empty() {
            continue;
        }
        let obj: serde_json::Map<String, Value> = serde_json::from_str(line).map_err(|e| {
            DbError::Invariant(format!("NDJSON ligne {} invalide: {e}", lineno + 1))
        })?;
        let values = column_names
            .iter()
            .map(|c| obj.get(c).cloned().unwrap_or(Value::Null))
            .collect();
        rows.push(values);
    }
    Ok(rows)
}

/// Contrainte d'une colonne destination (Story 17-3c, AC12c), lue de
/// `INFORMATION_SCHEMA.COLUMNS`. Sert au check de compatibilité de schéma
/// avant tout DELETE à l'import.
#[derive(Debug, Clone)]
pub struct ColumnConstraint {
    pub name: String,
    /// `IS_NULLABLE = 'YES'`.
    pub is_nullable: bool,
    /// `COLUMN_DEFAULT IS NOT NULL` (la colonne a une valeur par défaut).
    pub has_default: bool,
    /// `EXTRA` contient `auto_increment`.
    pub is_auto_increment: bool,
    /// `EXTRA` contient `GENERATED` (VIRTUAL/STORED).
    pub is_generated: bool,
}

impl ColumnConstraint {
    /// Une colonne destination est **obligatoire dans la source** (doit figurer
    /// dans `columnNames` du backup) ssi elle est `NOT NULL`, sans `DEFAULT`,
    /// ni auto-increment, ni générée : sinon l'`INSERT` du restore (qui ne cite
    /// que `columnNames`) planterait sur cette colonne non fournie.
    pub fn is_required(&self) -> bool {
        !self.is_nullable && !self.has_default && !self.is_auto_increment && !self.is_generated
    }
}

/// Lit les contraintes de toutes les colonnes d'une table destination.
pub async fn column_constraints(
    pool: &MySqlPool,
    table: &str,
) -> Result<Vec<ColumnConstraint>, DbError> {
    let rows = sqlx::query_as::<_, (String, String, Option<String>, String)>(
        "SELECT COLUMN_NAME, IS_NULLABLE, COLUMN_DEFAULT, EXTRA \
         FROM INFORMATION_SCHEMA.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? \
         ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;

    Ok(rows
        .into_iter()
        .map(|(name, is_nullable, col_default, extra)| {
            let extra_lc = extra.to_ascii_lowercase();
            ColumnConstraint {
                name,
                is_nullable: is_nullable.eq_ignore_ascii_case("YES"),
                has_default: col_default.is_some(),
                is_auto_increment: extra_lc.contains("auto_increment"),
                is_generated: extra_lc.contains("generated"),
            }
        })
        .collect())
}

/// Données d'une table à restaurer depuis un `.keshbackup` (Story 17-3c).
#[derive(Debug, Clone)]
pub struct TableRestore {
    /// Colonnes ciblées par l'`INSERT` (= `columnNames` du manifeste, ordonné,
    /// exclut les colonnes générées).
    pub column_names: Vec<String>,
    /// Lignes : chaque ligne = valeurs ordonnées selon `column_names`.
    pub rows: Vec<Vec<Value>>,
}

/// Restaure toutes les tables applicatives dans la transaction fournie (DC6).
///
/// Procédure sur **une seule connexion** (la transaction — `FOREIGN_KEY_CHECKS`
/// est session-scoped) :
/// 1. `SET FOREIGN_KEY_CHECKS = 0` (tolère la FK self-réf `accounts.parent_id`
///    et l'ordre d'insertion) ;
/// 2. `DELETE FROM <table>` pour les tables applicatives — **`onboarding_state`
///    exclue** (DC11, état local destination) — ordre enfants→parents ;
/// 3. `INSERT` paramétrés (colonnes explicites du manifeste) parents→enfants ;
/// 4. `SET FOREIGN_KEY_CHECKS = 1` — **rétabli systématiquement**, succès ou
///    erreur, pour ne jamais rendre une connexion au pool avec les FK
///    désactivées (corruption silencieuse).
///
/// Retourne `(tables_restaurées, lignes_restaurées)`. `_kesh_version`,
/// `_sqlx_migrations` et `onboarding_state` ne sont jamais touchées. Le caller
/// (handler 17-3c) insère l'audit + le COMMIT dans la **même** transaction.
pub async fn restore_tables_in_tx(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<(usize, usize), DbError> {
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;

    let result = restore_body(tx, tables).await;

    // Rétablissement systématique (même sur erreur du corps).
    let reset = sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut **tx)
        .await;
    match (result, reset) {
        (Ok(counts), Ok(_)) => Ok(counts),
        // Le corps a échoué : on propage son erreur (le reset a été tenté).
        (Err(e), _) => Err(e),
        // Le corps a réussi mais le reset a échoué : erreur dure (la connexion
        // ne doit pas committer avec FK désactivées).
        (Ok(_), Err(e)) => {
            tracing::error!(error = ?e, "restore: échec rétablissement FOREIGN_KEY_CHECKS=1");
            Err(map_db_error(e))
        }
    }
}

/// Corps du restore (DELETE + INSERT), hors gestion `FOREIGN_KEY_CHECKS`.
async fn restore_body(
    tx: &mut Transaction<'_, MySql>,
    tables: &BTreeMap<String, TableRestore>,
) -> Result<(usize, usize), DbError> {
    // DELETE enfants→parents (ordre TABLES_TO_TRUNCATE), onboarding_state exclue.
    for &table in TABLES_TO_TRUNCATE {
        if table == "onboarding_state" {
            continue;
        }
        sqlx::query(&format!("DELETE FROM `{table}`"))
            .execute(&mut **tx)
            .await
            .map_err(map_db_error)?;
    }

    // INSERT parents→enfants (ordre inverse) — indifférent sous FK=0, mais
    // cohérent/lisible.
    let mut tables_restored = 0usize;
    let mut rows_restored = 0usize;
    for &table in TABLES_TO_TRUNCATE.iter().rev() {
        if table == "onboarding_state" {
            continue;
        }
        let Some(data) = tables.get(table) else {
            // Dans le flux réel, `parse_and_verify` garantit la couverture (toute
            // table de TABLES_TO_TRUNCATE figure au manifeste). En usage isolé
            // (tests bas-niveau avec map partielle), une table absente est
            // simplement vidée (le DELETE a déjà eu lieu) — **non comptée** comme
            // restaurée (review Pass 3 : évite le sur-comptage de tables_restored
            // sans imposer la couverture complète à cette fonction).
            continue;
        };
        tables_restored += 1;
        if data.column_names.is_empty() {
            return Err(DbError::Invariant(format!(
                "restore: table '{table}' sans colonnes dans le manifeste"
            )));
        }
        if data.rows.is_empty() {
            continue;
        }
        let cols_sql = data
            .column_names
            .iter()
            .map(|c| format!("`{c}`"))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; data.column_names.len()].join(", ");
        let sql = format!("INSERT INTO `{table}` ({cols_sql}) VALUES ({placeholders})");
        for row in &data.rows {
            if row.len() != data.column_names.len() {
                return Err(DbError::Invariant(format!(
                    "restore: table '{table}' ligne à {} valeurs, {} colonnes attendues",
                    row.len(),
                    data.column_names.len()
                )));
            }
            let mut q = sqlx::query(&sql);
            for v in row {
                q = bind_json_value(q, v)?;
            }
            q.execute(&mut **tx).await.map_err(map_db_error)?;
            rows_restored += 1;
        }
    }
    Ok((tables_restored, rows_restored))
}

/// Bind une valeur JSON (issue du NDJSON d'export) sur une requête paramétrée,
/// en préservant la fidélité de type (DC1). MariaDB recoerce les strings vers
/// `DECIMAL`/`DATETIME`/etc. à l'`INSERT` (l'export sérialise ces types en
/// string fidèle).
fn bind_json_value<'q>(
    q: sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments>,
    v: &'q Value,
) -> Result<sqlx::query::Query<'q, MySql, sqlx::mysql::MySqlArguments>, DbError> {
    let bound = match v {
        Value::Null => q.bind(Option::<String>::None),
        Value::Bool(b) => q.bind(if *b { 1_i64 } else { 0_i64 }),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                q.bind(i)
            } else if let Some(u) = n.as_u64() {
                // Review Pass 1 : `u64 > i64::MAX` n'est pas représentable dans
                // le schéma Kesh (tous les entiers sont signés BIGINT/INT).
                // Rejeter explicitement plutôt que wrapper silencieusement vers
                // un négatif (`u64::MAX as i64 == -1`) qui corromprait la donnée.
                let i = i64::try_from(u).map_err(|_| {
                    DbError::Invariant(format!("entier {u} hors borne i64 (backup corrompu ?)"))
                })?;
                q.bind(i)
            } else {
                // Nombre non-entier : binder sa forme **string décimale exacte**.
                // MariaDB parse exactement vers DECIMAL (fidélité financière,
                // zéro arrondi binaire) et coerce aussi pour DOUBLE/FLOAT. Évite
                // le `19.95 → 19.9499…` d'un bind `f64` sur une colonne DECIMAL
                // si un backup (forgé/étranger) porte un nombre JSON au lieu de
                // la string produite par l'export (review Pass 3). `to_string()`
                // d'un `serde_json::Number` est sa représentation décimale exacte.
                q.bind(n.to_string())
            }
        }
        Value::String(s) => q.bind(s.as_str()),
        // Objet/array : une colonne JSON est exportée en string (longtext) par
        // l'export, donc ce cas est théorique — on re-sérialise par sécurité.
        other => q.bind(other.to_string()),
    };
    Ok(bound)
}

/// DC11 (Story 17-3c) — après un restore, force `onboarding_state` à l'état
/// terminé (`step_completed = 8`, valeur post-finalize) **si** le dataset
/// restauré contient au moins une company non-stub et au moins un admin. Évite
/// de rouvrir le catch-22 #120 quand on importe sur une instance fraîche /
/// non-onboardée. `onboarding_state` n'étant pas restaurée (état local), on
/// agit sur la row destination.
///
/// **Upsert** (review Pass 1) : couvre les deux cas que l'`UPDATE` ratait —
/// (a) **row absente** (instance fraîche jamais onboardée : `get_state` renvoie
/// `None` ⇒ le frontend rouvrirait le wizard malgré l'admin présent) ;
/// (b) row à un `step_completed` quelconque < terminal. `GREATEST` évite tout
/// downgrade. Clé unique `uq_onboarding_singleton (singleton)`. Idempotent.
/// Retourne `true` si éligible (état forcé), `false` sinon.
pub async fn force_onboarding_done_if_eligible(
    tx: &mut Transaction<'_, MySql>,
) -> Result<bool, DbError> {
    let companies: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies WHERE is_stub = FALSE")
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    let admins: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'Admin'")
        .fetch_one(&mut **tx)
        .await
        .map_err(map_db_error)?;
    if companies >= 1 && admins >= 1 {
        sqlx::query(
            // `is_demo = FALSE` aussi à l'UPDATE (review Pass 3) : après import
            // d'un backup réel, l'instance n'est plus en mode démo même si la
            // row destination l'était (les seeds démo viennent d'être effacés).
            "INSERT INTO onboarding_state (singleton, step_completed, is_demo) \
             VALUES (TRUE, 8, FALSE) \
             ON DUPLICATE KEY UPDATE step_completed = GREATEST(step_completed, 8), is_demo = FALSE",
        )
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::MySqlPool;

    /// Garantit que [`TABLES_TO_TRUNCATE`] reste synchro avec le schéma réel.
    /// Si une future migration ajoute/retire une table applicative, ce test
    /// échoue avec le delta, forçant la mise à jour de la const avant merge.
    /// (Promu de `test_fixtures.rs`, Story 17-3a — la const vit désormais ici.)
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn backup_inventory_matches_schema(pool: MySqlPool) {
        let db_tables: Vec<String> = sqlx::query_scalar(
            "SELECT TABLE_NAME FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = DATABASE() \
               AND TABLE_NAME NOT IN ('_sqlx_migrations', '_kesh_version') \
             ORDER BY TABLE_NAME",
        )
        .fetch_all(&pool)
        .await
        .expect("information_schema query");

        let mut hardcoded: Vec<&str> = TABLES_TO_TRUNCATE.to_vec();
        hardcoded.sort();
        let mut from_db: Vec<String> = db_tables.clone();
        from_db.sort();
        let hardcoded_str: Vec<String> = hardcoded.iter().map(|s| s.to_string()).collect();

        assert_eq!(
            hardcoded_str, from_db,
            "\nTABLES_TO_TRUNCATE désynchronisé avec information_schema :\n\
             - tables DB : {from_db:?}\n\
             - hardcoded : {hardcoded_str:?}\n\
             → mettre à jour `TABLES_TO_TRUNCATE` dans `crates/kesh-db/src/backup.rs`"
        );
    }

    /// `export_table` exclut les colonnes générées (`active_uniq` VIRTUAL).
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn export_table_excludes_generated_columns(pool: MySqlPool) {
        let export = export_table(&pool, "reconciliation_rules")
            .await
            .expect("export reconciliation_rules");
        assert!(
            !export.column_names.iter().any(|c| c == "active_uniq"),
            "colonne générée active_uniq doit être exclue, colonnes : {:?}",
            export.column_names
        );
        // Table vide en DB de test fraîche → NDJSON 0 octet.
        assert_eq!(export.row_count, 0);
        assert!(export.ndjson.is_empty());
    }

    /// NDJSON fidèle : 1 ligne JSON par row, ordre des clés = column_names,
    /// types respectés (id entier, date ISO, NULL → null).
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn export_table_serializes_rows_faithfully(pool: MySqlPool) {
        // Insère une company minimale (ide_number NULL pour tester NULL → null).
        sqlx::query(
            "INSERT INTO companies (name, address, ide_number, org_type, accounting_language, instance_language) \
             VALUES ('Acme', 'Rue 1', NULL, 'Pme', 'FR', 'FR')",
        )
        .execute(&pool)
        .await
        .expect("insert company");

        let export = export_table(&pool, "companies")
            .await
            .expect("export companies");
        assert_eq!(export.row_count, 1);

        let line = String::from_utf8(export.ndjson.clone()).unwrap();
        let line = line.trim_end();
        let v: serde_json::Value = serde_json::from_str(line).expect("valid ndjson line");

        assert!(v["id"].is_i64(), "id doit être un entier JSON");
        assert_eq!(v["name"], "Acme");
        assert!(v["ide_number"].is_null(), "ide_number NULL → null JSON");
        // created_at : DATETIME(3) → ISO 8601 'YYYY-MM-DDTHH:MM:SS[.mmm]'
        // (précision sub-seconde préservée). Doit reparse en NaiveDateTime.
        let created = v["created_at"].as_str().expect("created_at string");
        assert_eq!(&created[10..11], "T", "séparateur T: {created}");
        assert!(
            chrono::NaiveDateTime::parse_from_str(created, "%Y-%m-%dT%H:%M:%S%.f").is_ok(),
            "created_at doit reparse (round-trip) : {created}"
        );
    }

    // --- Story 17-3c : import (parse NDJSON, contraintes, restore) ---

    #[test]
    fn parse_ndjson_rows_orders_by_column_names_and_handles_null() {
        let cols = vec!["id".to_string(), "name".to_string(), "note".to_string()];
        // 2e ligne : clé `note` absente → null ; clés dans le désordre.
        let ndjson = b"{\"name\":\"Acme\",\"id\":1,\"note\":\"x\"}\n{\"id\":2,\"name\":\"Beta\"}\n";
        let rows = parse_ndjson_rows(ndjson, &cols).expect("parse");
        assert_eq!(rows.len(), 2);
        assert_eq!(
            rows[0],
            vec![Value::from(1), Value::from("Acme"), Value::from("x")]
        );
        assert_eq!(
            rows[1],
            vec![Value::from(2), Value::from("Beta"), Value::Null]
        );
    }

    #[test]
    fn parse_ndjson_rows_empty_input_is_empty() {
        let cols = vec!["id".to_string()];
        assert!(parse_ndjson_rows(b"", &cols).unwrap().is_empty());
    }

    /// `column_constraints` détecte `companies.id` comme auto-increment (donc
    /// non requise) et `companies.name` comme NOT NULL sans défaut (requise).
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn column_constraints_flags_required_and_optional(pool: MySqlPool) {
        let cols = column_constraints(&pool, "companies")
            .await
            .expect("constraints companies");
        let by_name = |n: &str| cols.iter().find(|c| c.name == n).cloned();

        let id = by_name("id").expect("id column");
        assert!(id.is_auto_increment, "id doit être auto_increment");
        assert!(
            !id.is_required(),
            "id (auto_increment) ne doit pas être requise"
        );

        let name = by_name("name").expect("name column");
        assert!(!name.is_nullable, "name est NOT NULL");
        assert!(
            name.is_required(),
            "name NOT NULL sans défaut doit être requise"
        );

        // ide_number est nullable → jamais requise.
        let ide = by_name("ide_number").expect("ide_number column");
        assert!(!ide.is_required());
    }

    /// `reconciliation_rules.active_uniq` (GENERATED VIRTUAL) est marquée
    /// générée → jamais requise (cohérent avec son exclusion de l'export).
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn column_constraints_marks_generated_column(pool: MySqlPool) {
        let cols = column_constraints(&pool, "reconciliation_rules")
            .await
            .expect("constraints reconciliation_rules");
        let active_uniq = cols
            .iter()
            .find(|c| c.name == "active_uniq")
            .expect("active_uniq present in schema");
        assert!(active_uniq.is_generated, "active_uniq doit être générée");
        assert!(!active_uniq.is_required());
    }

    /// Round-trip restore : insère 2 companies → exporte (export_table) →
    /// restore_tables_in_tx (DELETE+INSERT) → les 2 companies sont rétablies
    /// avec les mêmes valeurs (fidélité de type).
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn restore_tables_in_tx_round_trips_companies(pool: MySqlPool) {
        for (name, ide) in [("Acme SA", Some("CHE123456789")), ("Beta GmbH", None)] {
            let ide_sql = match ide {
                Some(v) => format!("'{v}'"),
                None => "NULL".to_string(),
            };
            sqlx::query(&format!(
                "INSERT INTO companies (name, address, ide_number, org_type, accounting_language, instance_language) \
                 VALUES ('{name}', 'Rue 1', {ide_sql}, 'Pme', 'FR', 'FR')"
            ))
            .execute(&pool)
            .await
            .expect("insert company");
        }

        // Snapshot via export_table.
        let export = export_table(&pool, "companies").await.expect("export");
        let rows = parse_ndjson_rows(&export.ndjson, &export.column_names).expect("parse");
        assert_eq!(rows.len(), 2);

        let mut tables = BTreeMap::new();
        tables.insert(
            "companies".to_string(),
            TableRestore {
                column_names: export.column_names.clone(),
                rows,
            },
        );

        // Restore (DELETE-all + INSERT companies) dans une transaction.
        let mut tx = pool.begin().await.expect("begin");
        let (n_tables, n_rows) = restore_tables_in_tx(&mut tx, &tables)
            .await
            .expect("restore");
        tx.commit().await.expect("commit");

        // Seule `companies` est fournie dans la map de test → 1 table comptée
        // (les autres sont vidées par le DELETE mais non re-restaurées, donc
        // non comptées — cf. skip-sans-incrément). 2 lignes companies.
        assert_eq!(n_tables, 1);
        assert_eq!(n_rows, 2);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("count");
        assert_eq!(count, 2, "les 2 companies doivent être rétablies");

        let names: Vec<String> = sqlx::query_scalar("SELECT name FROM companies ORDER BY name")
            .fetch_all(&pool)
            .await
            .expect("names");
        assert_eq!(names, vec!["Acme SA".to_string(), "Beta GmbH".to_string()]);

        // FOREIGN_KEY_CHECKS rétabli sur la connexion (vérif via une nouvelle
        // requête : la valeur de session est 1 par défaut sur une conn fraîche
        // du pool — ce test garantit surtout que le restore a committé propre).
    }
}
