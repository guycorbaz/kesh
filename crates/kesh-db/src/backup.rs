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
//! `Decimal` → string, `NaiveDateTime` → ISO 8601 `YYYY-MM-DDTHH:MM:SS`,
//! entiers → nombre JSON.

use serde_json::Value;
use sqlx::{MySqlPool, Row};

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
    "invoice_lines",
    "journal_entry_lines",
    "invoices",
    "invoice_number_sequences",
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
    "contacts",
    "fiscal_years",
    "vat_rates", // Story 7.2 (KF-003) — table enfant de companies (FK fk_vat_rates_company), aucune table ne référence vat_rates.
    "refresh_tokens",
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
            Some(dt) => Value::String(dt.format("%Y-%m-%dT%H:%M:%S").to_string()),
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
        // created_at : DATETIME → ISO 8601 'YYYY-MM-DDTHH:MM:SS'
        let created = v["created_at"].as_str().expect("created_at string");
        assert_eq!(created.len(), 19, "datetime ISO sec-precision: {created}");
        assert_eq!(&created[10..11], "T", "séparateur T: {created}");
    }
}
