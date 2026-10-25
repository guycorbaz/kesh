//! Repository des réglages du serveur MCP (`mcp_server_settings`, Story
//! 32-5a, refs #723).
//!
//! - [`get`] **n'écrit jamais** (C-32-5-8) : `None` si la société n'a rien
//!   enregistré — l'appelant prend alors `McpServerSettings::defaults`
//!   (ou [`get_or_defaults`], qui le fait pour lui).
//! - [`save_in_tx`] : verrou optimiste où `version == 0` désigne « aucune
//!   ligne » — `INSERT`, `OptimisticLockConflict` si une ligne est apparue
//!   entre-temps ; `version ≥ 1` — `UPDATE … WHERE version = ?`,
//!   `OptimisticLockConflict` sinon.
//!
//! ⚠️ Les trois listes sont des colonnes `JSON`, lues par `CAST(… AS CHAR)`
//! (MariaDB rend une colonne `JSON` en `BLOB`). Le décodage est strict : une
//! valeur qui n'est pas un tableau de chaînes est une erreur d'invariant.

use chrono::NaiveDateTime;
use sqlx::mysql::MySqlPool;
use sqlx::{MySql, Transaction};

use crate::entities::mcp_server_settings::{McpServerSettings, McpServerSettingsUpdate};
use crate::errors::{DbError, map_db_error};

const COLUMNS: &str = "company_id, enabled, address_filter_enabled, \
     CAST(allowed_networks AS CHAR) AS allowed_networks, \
     CAST(trusted_proxies AS CHAR) AS trusted_proxies, \
     CAST(allowed_origins AS CHAR) AS allowed_origins, version, updated_at";

#[derive(sqlx::FromRow)]
struct Row {
    company_id: i64,
    enabled: bool,
    address_filter_enabled: bool,
    allowed_networks: String,
    trusted_proxies: String,
    allowed_origins: String,
    version: i32,
    updated_at: NaiveDateTime,
}

fn decode_list(colonne: &str, json: &str) -> Result<Vec<String>, DbError> {
    serde_json::from_str(json).map_err(|e| {
        DbError::Invariant(format!(
            "mcp_server_settings.{colonne} : tableau JSON de chaînes attendu ({e})"
        ))
    })
}

fn encode_list(list: &[String]) -> String {
    serde_json::to_string(list).expect("un tableau de chaînes se sérialise toujours")
}

impl TryFrom<Row> for McpServerSettings {
    type Error = DbError;
    fn try_from(r: Row) -> Result<Self, DbError> {
        Ok(Self {
            company_id: r.company_id,
            enabled: r.enabled,
            address_filter_enabled: r.address_filter_enabled,
            allowed_networks: decode_list("allowed_networks", &r.allowed_networks)?,
            trusted_proxies: decode_list("trusted_proxies", &r.trusted_proxies)?,
            allowed_origins: decode_list("allowed_origins", &r.allowed_origins)?,
            version: r.version,
            updated_at: Some(r.updated_at),
        })
    }
}

/// La ligne de réglages de la société, `None` s'il n'y en a pas. N'écrit rien.
pub async fn get(pool: &MySqlPool, company_id: i64) -> Result<Option<McpServerSettings>, DbError> {
    sqlx::query_as::<_, Row>(&format!(
        "SELECT {COLUMNS} FROM mcp_server_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_optional(pool)
    .await
    .map_err(map_db_error)?
    .map(McpServerSettings::try_from)
    .transpose()
}

/// La ligne de réglages, ou les défauts du code (`version == 0`) s'il n'y en
/// a pas. N'écrit rien.
pub async fn get_or_defaults(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<McpServerSettings, DbError> {
    Ok(get(pool, company_id)
        .await?
        .unwrap_or_else(|| McpServerSettings::defaults(company_id)))
}

/// Enregistre les réglages (remplacement complet), sous verrou optimiste, et
/// rend la ligne écrite.
///
/// - `version == 0` : aucune ligne attendue → `INSERT` ; si une ligne existe
///   déjà, `OptimisticLockConflict` (jamais d'écrasement).
/// - `version ≥ 1` : `UPDATE … WHERE company_id = ? AND version = ?` ;
///   aucune ligne touchée → `OptimisticLockConflict`.
/// - `version < 0` : `OptimisticLockConflict` (aucune ligne ne la porte).
pub async fn save_in_tx(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
    version: i32,
    settings: &McpServerSettingsUpdate,
) -> Result<McpServerSettings, DbError> {
    let networks = encode_list(&settings.allowed_networks);
    let proxies = encode_list(&settings.trusted_proxies);
    let origins = encode_list(&settings.allowed_origins);

    if version == 0 {
        let inserted = sqlx::query(
            "INSERT INTO mcp_server_settings \
             (company_id, enabled, address_filter_enabled, allowed_networks, trusted_proxies, \
              allowed_origins) \
             VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(settings.enabled)
        .bind(settings.address_filter_enabled)
        .bind(&networks)
        .bind(&proxies)
        .bind(&origins)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error);
        match inserted {
            Ok(_) => {}
            // Une ligne est apparue depuis la lecture (version 0) : conflit,
            // comme une version périmée — jamais un écrasement.
            Err(DbError::UniqueConstraintViolation(_)) => {
                return Err(DbError::OptimisticLockConflict);
            }
            Err(e) => return Err(e),
        }
    } else {
        if version < 0 {
            return Err(DbError::OptimisticLockConflict);
        }
        let rows = sqlx::query(
            "UPDATE mcp_server_settings \
             SET enabled = ?, address_filter_enabled = ?, allowed_networks = ?, \
                 trusted_proxies = ?, allowed_origins = ?, version = version + 1 \
             WHERE company_id = ? AND version = ?",
        )
        .bind(settings.enabled)
        .bind(settings.address_filter_enabled)
        .bind(&networks)
        .bind(&proxies)
        .bind(&origins)
        .bind(company_id)
        .bind(version)
        .execute(&mut **tx)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if rows == 0 {
            return Err(DbError::OptimisticLockConflict);
        }
    }

    sqlx::query_as::<_, Row>(&format!(
        "SELECT {COLUMNS} FROM mcp_server_settings WHERE company_id = ?"
    ))
    .bind(company_id)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_db_error)
    .and_then(McpServerSettings::try_from)
}
