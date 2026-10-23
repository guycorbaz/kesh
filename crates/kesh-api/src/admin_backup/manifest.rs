//! Story 17-3a — Manifeste `manifest.json` du `.keshbackup`.
//!
//! Schéma camelCase, pretty-printed (cohérent `exports/metadata.rs`). Source de
//! vérité du contrat export↔import (la sous-story 17-3c relira ce manifeste).
//!
//! Story 31-1a (#503) — **format 2** : le manifeste inventorie aussi les pièces
//! du dossier des documents emportées sous `files/` (`files`) et celles que la
//! base référence mais que la sauvegarde n'a pas pu emporter
//! (`filesNotIncluded`, avec leur raison).

use std::collections::BTreeMap;

use chrono::{SecondsFormat, Utc};
use serde::{Deserialize, Serialize};

use crate::errors::AppError;

/// Version du format `.keshbackup` écrite par l'export.
///
/// - `1` (17-3a à 0.14.x) : NDJSON par table, dossier `files/` **vide** ;
/// - `2` (31-1a, #503) : les pièces référencées par la base sous
///   `files/<storage_path>`, inventoriées par `files` et `filesNotIncluded`.
///
/// L'import accepte 1 et 2 et refuse au-delà (`400`). Une sauvegarde de
/// format 2 n'est pas importable par une 0.14.x : celle-ci refuse un `files/`
/// non vide, ou `formatVersion > 1`.
pub const BACKUP_FORMAT_VERSION: u32 = 2;

/// Manifeste du `.keshbackup` (au ROOT du ZIP). `Deserialize` ajouté en 17-3c
/// pour relire le manifeste à l'import.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupManifest {
    pub format_version: u32,
    pub kesh_version: String,
    pub kesh_version_min_required: String,
    pub instance_id: i64,
    pub export_date: String,
    pub tables: BTreeMap<String, BackupTableMeta>,
    /// Story 31-1a — pièces emportées sous `files/<path>`, triées par chemin.
    /// Absent d'un manifeste de format 1 : relu vide.
    #[serde(default)]
    pub files: Vec<BackupFileMeta>,
    /// Story 31-1a — pièces référencées par la base que la sauvegarde
    /// n'emporte pas, triées par chemin. Absent d'un format 1 : relu vide.
    #[serde(default)]
    pub files_not_included: Vec<FileNotIncluded>,
}

/// Métadonnées par table (clé du `tables` = nom de la table).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupTableMeta {
    pub row_count: usize,
    pub sha256: String,
    /// Colonnes sérialisées (ordonnées, **hors colonnes générées**). Utilisée
    /// par l'import 17-3c pour construire les `INSERT` paramétrés.
    pub column_names: Vec<String>,
}

/// Story 31-1a — une pièce emportée : entrée `files/<path>` de l'archive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupFileMeta {
    /// Nom dans le dossier des documents, `<sha256>.<ext>`.
    pub path: String,
    /// Empreinte SHA-256 (hex) du contenu — égale au préfixe du nom.
    pub sha256: String,
    /// Taille du contenu, en octets.
    pub byte_size: u64,
}

/// Story 31-1a — une pièce référencée que la sauvegarde n'emporte pas.
///
/// ⚠️ `path` est la valeur **telle que la base la porte**, éventuellement hors
/// forme (`../x`, retour à la ligne) : il ne sert jamais de chemin, et ne se
/// journalise qu'en `Debug`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileNotIncluded {
    pub path: String,
    pub reason: NotIncludedReason,
}

/// Story 31-1a (AC3) — pourquoi une pièce référencée n'est pas emportée.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotIncludedReason {
    /// Absente du dossier des documents (`NotFound`).
    Missing,
    /// Présente, mais son contenu n'a plus l'empreinte déclarée (ou sa taille
    /// a changé pendant la lecture).
    Altered,
    /// La ligne est incohérente : nom qui n'a pas la forme `<sha>.<ext>`,
    /// préfixe qui n'est pas l'empreinte déclarée, ou même chemin déclaré sous
    /// deux empreintes. Le fichier n'est pas lu.
    Mismatch,
    /// Toute autre erreur d'entrée-sortie à l'ouverture ou à la lecture
    /// (droits, `EIO`, dossier à la place du fichier).
    Unreadable,
}

impl BackupManifest {
    /// Construit le manifeste en figeant `format_version`, `kesh_version`
    /// (build-time `CARGO_PKG_VERSION`) et `export_date` (ISO 8601 UTC,
    /// précision seconde).
    pub fn new(
        kesh_version_min_required: String,
        instance_id: i64,
        tables: BTreeMap<String, BackupTableMeta>,
        files: Vec<BackupFileMeta>,
        files_not_included: Vec<FileNotIncluded>,
    ) -> Self {
        Self {
            format_version: BACKUP_FORMAT_VERSION,
            kesh_version: env!("CARGO_PKG_VERSION").to_string(),
            kesh_version_min_required,
            instance_id,
            export_date: Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            tables,
            files,
            files_not_included,
        }
    }
}

/// Sérialise le manifeste en JSON pretty **suivi d'un `\n` final** (le format
/// normatif `.keshbackup` l'exige ; `serde_json::to_vec_pretty` n'en ajoute
/// pas — review 17-3a Pass 1 F1).
pub fn build_backup_manifest_json(manifest: &BackupManifest) -> Result<Vec<u8>, AppError> {
    let mut bytes = serde_json::to_vec_pretty(manifest)
        .map_err(|e| AppError::AdminFullExportFailed(format!("manifest serialize: {e}")))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

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
        let manifest = BackupManifest::new(
            "0.1.0".into(),
            1,
            sample_tables(),
            vec![BackupFileMeta {
                path: "ab.pdf".into(),
                sha256: "ab".into(),
                byte_size: 12345,
            }],
            vec![FileNotIncluded {
                path: "cd.png".into(),
                reason: NotIncludedReason::Unreadable,
            }],
        );
        let bytes = build_backup_manifest_json(&manifest).unwrap();
        assert_eq!(
            bytes.last(),
            Some(&b'\n'),
            "manifest doit finir par \\n (format normatif)"
        );
        let parsed: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(parsed["formatVersion"], 2);
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
        // Story 31-1a (AC3) — les deux listes, en camelCase.
        assert_eq!(
            parsed["files"],
            serde_json::json!([{ "path": "ab.pdf", "sha256": "ab", "byteSize": 12345 }])
        );
        assert_eq!(
            parsed["filesNotIncluded"],
            serde_json::json!([{ "path": "cd.png", "reason": "unreadable" }])
        );
    }

    /// Story 31-1a (AC3, T-a4b) — un manifeste de format 1 (sans `files` ni
    /// `filesNotIncluded`) se relit avec deux listes vides.
    #[test]
    fn manifest_format_1_se_relit_avec_deux_listes_vides() {
        let json = serde_json::json!({
            "formatVersion": 1,
            "keshVersion": "0.14.0",
            "keshVersionMinRequired": "0.14.0",
            "instanceId": 1,
            "exportDate": "2026-10-20T00:00:00Z",
            "tables": {},
        });
        let manifest: BackupManifest = serde_json::from_value(json).unwrap();
        assert_eq!(manifest.format_version, 1);
        assert!(manifest.files.is_empty());
        assert!(manifest.files_not_included.is_empty());
    }

    /// Story 31-1a — les quatre raisons portent, au manifeste, le code de
    /// l'AC3, et se relisent.
    #[test]
    fn raisons_de_non_inclusion() {
        for (raison, code) in [
            (NotIncludedReason::Missing, "missing"),
            (NotIncludedReason::Altered, "altered"),
            (NotIncludedReason::Mismatch, "mismatch"),
            (NotIncludedReason::Unreadable, "unreadable"),
        ] {
            assert_eq!(serde_json::to_value(raison).unwrap(), code);
            assert_eq!(
                serde_json::from_value::<NotIncludedReason>(Value::from(code)).unwrap(),
                raison
            );
        }
    }
}
