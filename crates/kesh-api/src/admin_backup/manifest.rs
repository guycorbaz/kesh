//! Story 17-3a — Manifeste `manifest.json` du `.keshbackup`.
//!
//! Schéma camelCase, pretty-printed (cohérent `exports/metadata.rs`). Source de
//! vérité du contrat export↔import (la sous-story 17-3c relira ce manifeste).

use std::collections::BTreeMap;

use chrono::{SecondsFormat, Utc};
use serde::Serialize;

use crate::errors::AppError;

/// Version du format `.keshbackup`. Figée à `1` (format NDJSON-par-table
/// introduit par 17-3a). L'import (17-3c) refusera `400` si `> 1`.
pub const BACKUP_FORMAT_VERSION: u32 = 1;

/// Manifeste du `.keshbackup` (au ROOT du ZIP).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub format_version: u32,
    pub kesh_version: String,
    pub kesh_version_min_required: String,
    pub instance_id: i64,
    pub export_date: String,
    pub tables: BTreeMap<String, BackupTableMeta>,
}

/// Métadonnées par table (clé du `tables` = nom de la table).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTableMeta {
    pub row_count: usize,
    pub sha256: String,
    /// Colonnes sérialisées (ordonnées, **hors colonnes générées**). Utilisée
    /// par l'import 17-3c pour construire les `INSERT` paramétrés.
    pub column_names: Vec<String>,
}

impl BackupManifest {
    /// Construit le manifeste en figeant `format_version`, `kesh_version`
    /// (build-time `CARGO_PKG_VERSION`) et `export_date` (ISO 8601 UTC,
    /// précision seconde).
    pub fn new(
        kesh_version_min_required: String,
        instance_id: i64,
        tables: BTreeMap<String, BackupTableMeta>,
    ) -> Self {
        Self {
            format_version: BACKUP_FORMAT_VERSION,
            kesh_version: env!("CARGO_PKG_VERSION").to_string(),
            kesh_version_min_required,
            instance_id,
            export_date: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            tables,
        }
    }
}

/// Sérialise le manifeste en JSON pretty (+ trailing `\n` implicite).
pub fn build_backup_manifest_json(manifest: &BackupManifest) -> Result<Vec<u8>, AppError> {
    serde_json::to_vec_pretty(manifest)
        .map_err(|e| AppError::AdminFullExportFailed(format!("manifest serialize: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_tables() -> BTreeMap<String, BackupTableMeta> {
        let mut t = BTreeMap::new();
        t.insert(
            "companies".to_string(),
            BackupTableMeta {
                row_count: 2,
                sha256: "abc123".into(),
                column_names: vec!["id".into(), "name".into()],
            },
        );
        t
    }

    #[test]
    fn manifest_json_has_canonical_shape() {
        let manifest = BackupManifest::new("0.1.0".into(), 1, sample_tables());
        let bytes = build_backup_manifest_json(&manifest).unwrap();
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parsed["formatVersion"], 1);
        assert_eq!(parsed["keshVersion"], env!("CARGO_PKG_VERSION"));
        assert_eq!(parsed["keshVersionMinRequired"], "0.1.0");
        assert_eq!(parsed["instanceId"], 1);
        // exportDate ISO 8601 UTC strict (Z, sec-precision = 20 chars).
        let date = parsed["exportDate"].as_str().unwrap();
        assert!(date.ends_with('Z'), "exportDate must end with Z: {date}");
        assert_eq!(date.len(), 20, "YYYY-MM-DDTHH:MM:SSZ: {date}");
        // tables[companies] : rowCount + sha256 + columnNames.
        let companies = &parsed["tables"]["companies"];
        assert_eq!(companies["rowCount"], 2);
        assert_eq!(companies["sha256"], "abc123");
        assert_eq!(companies["columnNames"][0], "id");
        assert_eq!(companies["columnNames"][1], "name");
    }
}
