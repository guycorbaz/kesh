//! Story 17-3c — Lecture + validation d'un `.keshbackup` à l'import.
//!
//! Toute la validation se fait **avant tout DELETE** (la DB n'est jamais mutée
//! si le backup est rejeté) :
//! 1. structure ZIP (`manifest.json` au root + `data/<table>.ndjson` + `files/`
//!    vide) → [`AppError::InvalidBackupStructure`] ;
//! 2. désérialisation du manifeste + `formatVersion ≤ 1` ;
//! 3. intégrité SHA-256 par table (tamper) ;
//! 4. couverture (toute table applicative attendue est présente) ;
//! 5. compat colonnes bidirectionnelle vs schéma destination (AC12c).
//!
//! ⚠️ Le `.keshbackup` est un **secret** : ne jamais logger son contenu.

use std::collections::{BTreeMap, HashSet};
use std::io::{Cursor, Read};

use sqlx::MySqlPool;
use zip::ZipArchive;

use kesh_db::backup::{TABLES_TO_TRUNCATE, TableRestore, column_constraints, parse_ndjson_rows};

use crate::admin_backup::manifest::{BACKUP_FORMAT_VERSION, BackupManifest};
use crate::errors::AppError;
use crate::exports::metadata::sha256_hex;

/// Backup parsé + vérifié (structure + intégrité), prêt pour le restore.
#[derive(Debug)]
pub struct ParsedBackup {
    pub manifest: BackupManifest,
    /// Données par table, dans l'ordre des `column_names` du manifeste.
    pub tables: BTreeMap<String, TableRestore>,
    pub total_rows: usize,
}

/// Parse le conteneur ZIP et vérifie structure + intégrité SHA-256.
///
/// Ne touche **pas** la DB. Retourne [`AppError::InvalidBackupStructure`] (→400)
/// sur toute anomalie de structure, format ou intégrité.
pub fn parse_and_verify(bytes: &[u8]) -> Result<ParsedBackup, AppError> {
    let mut zip = ZipArchive::new(Cursor::new(bytes))
        .map_err(|e| AppError::InvalidBackupStructure(format!("conteneur ZIP illisible : {e}")))?;

    let mut manifest_bytes: Option<Vec<u8>> = None;
    let mut ndjson: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    let mut files_non_empty = false;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| AppError::InvalidBackupStructure(format!("entrée ZIP {i} : {e}")))?;
        let name = entry.name().to_string();

        if name == "manifest.json" {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| AppError::InvalidBackupStructure(format!("lecture manifest : {e}")))?;
            manifest_bytes = Some(buf);
        } else if let Some(table) = name
            .strip_prefix("data/")
            .and_then(|n| n.strip_suffix(".ndjson"))
        {
            // Nom de table = base du fichier ; rejeter les sous-chemins.
            if table.is_empty() || table.contains('/') {
                return Err(AppError::InvalidBackupStructure(format!(
                    "entrée data/ invalide : {name}"
                )));
            }
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| AppError::InvalidBackupStructure(format!("lecture {name} : {e}")))?;
            ndjson.insert(table.to_string(), buf);
        } else if name == "files/" {
            // Dossier forward-compat attendu, vide en v0.2.
        } else if name.starts_with("files/") {
            // Toute entrée fichier (non-dossier) sous files/ → non-vide → refus.
            if !name.ends_with('/') {
                files_non_empty = true;
            }
        } else {
            return Err(AppError::InvalidBackupStructure(format!(
                "entrée inattendue dans le backup : {name}"
            )));
        }
    }

    if files_non_empty {
        return Err(AppError::InvalidBackupStructure(
            "le dossier files/ doit être vide (aucun binaire en v0.2)".into(),
        ));
    }

    let manifest_bytes = manifest_bytes
        .ok_or_else(|| AppError::InvalidBackupStructure("manifest.json absent".into()))?;
    let manifest: BackupManifest = serde_json::from_slice(&manifest_bytes)
        .map_err(|e| AppError::InvalidBackupStructure(format!("manifest.json illisible : {e}")))?;

    if manifest.format_version > BACKUP_FORMAT_VERSION {
        return Err(AppError::InvalidBackupStructure(format!(
            "formatVersion {} non supporté (maximum {})",
            manifest.format_version, BACKUP_FORMAT_VERSION
        )));
    }

    // Couverture : chaque table applicative attendue doit figurer au manifeste
    // (sinon on ne saurait pas la restaurer correctement).
    for &t in TABLES_TO_TRUNCATE {
        if !manifest.tables.contains_key(t) {
            return Err(AppError::InvalidBackupStructure(format!(
                "table '{t}' absente du manifeste"
            )));
        }
    }

    // Intégrité SHA-256 + parsing NDJSON par table (avant tout DELETE).
    let mut tables = BTreeMap::new();
    let mut total_rows = 0usize;
    for (table, meta) in &manifest.tables {
        let data = ndjson.get(table).ok_or_else(|| {
            AppError::InvalidBackupStructure(format!("data/{table}.ndjson absent"))
        })?;
        let sha = sha256_hex(data);
        if sha != meta.sha256 {
            return Err(AppError::InvalidBackupStructure(format!(
                "intégrité SHA-256 invalide pour la table '{table}' (fichier altéré ?)"
            )));
        }
        let rows = parse_ndjson_rows(data, &meta.column_names).map_err(|e| {
            AppError::InvalidBackupStructure(format!("NDJSON '{table}' invalide : {e}"))
        })?;
        if rows.len() != meta.row_count {
            return Err(AppError::InvalidBackupStructure(format!(
                "table '{table}' : {} lignes lues, {} déclarées au manifeste",
                rows.len(),
                meta.row_count
            )));
        }
        total_rows += rows.len();
        tables.insert(
            table.clone(),
            TableRestore {
                column_names: meta.column_names.clone(),
                rows,
            },
        );
    }

    Ok(ParsedBackup {
        manifest,
        tables,
        total_rows,
    })
}

/// Vérifie la compatibilité de schéma **bidirectionnelle** (AC12c) entre le
/// backup (source) et la base destination, via `INFORMATION_SCHEMA.COLUMNS`.
///
/// - (c1) chaque colonne source ⊆ colonnes destination (sinon `unknownColumns`) ;
/// - (c2) chaque colonne destination `NOT NULL` sans défaut, non-générée,
///   non-auto-increment ⊆ colonnes source (sinon `missingRequiredColumns`).
///
/// Retourne [`AppError::ImportSchemaMismatch`] (→400) au premier écart.
pub async fn check_schema_compat(pool: &MySqlPool, parsed: &ParsedBackup) -> Result<(), AppError> {
    for (table, data) in &parsed.tables {
        let dest = column_constraints(pool, table).await.map_err(|e| {
            AppError::AdminFullImportFailed(format!("lecture schéma '{table}' : {e}"))
        })?;

        let dest_names: HashSet<&str> = dest.iter().map(|c| c.name.as_str()).collect();
        let src_names: HashSet<&str> = data.column_names.iter().map(|s| s.as_str()).collect();

        // (c1) colonnes source inconnues côté destination.
        let mut unknown: Vec<String> = data
            .column_names
            .iter()
            .filter(|c| !dest_names.contains(c.as_str()))
            .cloned()
            .collect();
        unknown.sort();

        // (c2) colonnes destination obligatoires absentes de la source.
        let mut missing: Vec<String> = dest
            .iter()
            .filter(|c| c.is_required() && !src_names.contains(c.name.as_str()))
            .map(|c| c.name.clone())
            .collect();
        missing.sort();

        if !unknown.is_empty() || !missing.is_empty() {
            return Err(AppError::ImportSchemaMismatch {
                table: table.clone(),
                unknown_columns: unknown,
                missing_required_columns: missing,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

    /// Construit un `.keshbackup` minimal en mémoire pour les tests de parsing.
    /// `tables` : (nom, column_names, ndjson_bytes).
    fn build_test_backup(
        format_version_override: Option<u32>,
        tables: &[(&str, Vec<&str>, &[u8])],
        include_all_required: bool,
        files_extra: Option<&str>,
    ) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts: SimpleFileOptions =
                SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

            let mut meta = serde_json::Map::new();
            // Émet les tables fournies.
            for (name, cols, ndjson) in tables {
                zip.start_file(format!("data/{name}.ndjson"), opts).unwrap();
                zip.write_all(ndjson).unwrap();
                let row_count = std::str::from_utf8(ndjson)
                    .unwrap()
                    .lines()
                    .filter(|l| !l.is_empty())
                    .count();
                meta.insert(
                    name.to_string(),
                    serde_json::json!({
                        "rowCount": row_count,
                        "sha256": sha256_hex(ndjson),
                        "columnNames": cols,
                    }),
                );
            }
            // Complète les tables applicatives manquantes (vides) si demandé,
            // pour passer le check de couverture.
            if include_all_required {
                for &t in TABLES_TO_TRUNCATE {
                    if !meta.contains_key(t) {
                        zip.start_file(format!("data/{t}.ndjson"), opts).unwrap();
                        // 0 octet.
                        meta.insert(
                            t.to_string(),
                            serde_json::json!({
                                "rowCount": 0,
                                "sha256": sha256_hex(b""),
                                "columnNames": ["id"],
                            }),
                        );
                    }
                }
            }

            zip.add_directory("files/", opts).unwrap();
            if let Some(extra) = files_extra {
                zip.start_file(format!("files/{extra}"), opts).unwrap();
                zip.write_all(b"binaire interdit").unwrap();
            }

            let manifest = serde_json::json!({
                "formatVersion": format_version_override.unwrap_or(BACKUP_FORMAT_VERSION),
                "keshVersion": "0.1.8",
                "keshVersionMinRequired": "0.1.0",
                "instanceId": 1,
                "exportDate": "2026-06-09T00:00:00Z",
                "tables": meta,
            });
            zip.start_file("manifest.json", opts).unwrap();
            zip.write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
                .unwrap();
            zip.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn parse_valid_backup_succeeds() {
        let ndjson = b"{\"id\":1,\"name\":\"Acme\"}\n";
        let backup = build_test_backup(
            None,
            &[("companies", vec!["id", "name"], ndjson)],
            true,
            None,
        );
        let parsed = parse_and_verify(&backup).expect("valid backup");
        assert_eq!(parsed.manifest.format_version, BACKUP_FORMAT_VERSION);
        assert_eq!(parsed.tables["companies"].rows.len(), 1);
        assert_eq!(parsed.total_rows, 1);
    }

    #[test]
    fn parse_rejects_future_format_version() {
        let backup = build_test_backup(Some(2), &[("companies", vec!["id"], b"")], true, None);
        let err = parse_and_verify(&backup).unwrap_err();
        assert!(matches!(err, AppError::InvalidBackupStructure(_)));
    }

    #[test]
    fn parse_rejects_sha_tamper() {
        // Construit un backup valide, puis on falsifie un NDJSON sans MAJ du
        // SHA en re-zippant à la main : plus simple — on altère via un backup
        // où le sha déclaré ne matche pas. On force un mismatch en réécrivant.
        let good = b"{\"id\":1}\n";
        let backup = build_test_backup(None, &[("companies", vec!["id"], good)], true, None);
        // Re-parse OK d'abord.
        assert!(parse_and_verify(&backup).is_ok());

        // Falsification : on reconstruit avec un ndjson différent mais un sha
        // figé de l'ancien contenu via un manifeste forgé.
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
            zip.start_file("data/companies.ndjson", opts).unwrap();
            zip.write_all(b"{\"id\":999}\n").unwrap(); // contenu altéré
            let mut meta = serde_json::Map::new();
            meta.insert(
                "companies".into(),
                serde_json::json!({
                    "rowCount": 1,
                    "sha256": sha256_hex(good), // SHA de l'ancien contenu → mismatch
                    "columnNames": ["id"],
                }),
            );
            for &t in TABLES_TO_TRUNCATE {
                if t != "companies" {
                    zip.start_file(format!("data/{t}.ndjson"), opts).unwrap();
                    meta.insert(
                        t.into(),
                        serde_json::json!({ "rowCount": 0, "sha256": sha256_hex(b""), "columnNames": ["id"] }),
                    );
                }
            }
            zip.add_directory("files/", opts).unwrap();
            let manifest = serde_json::json!({
                "formatVersion": 1, "keshVersion": "0.1.8", "keshVersionMinRequired": "0.1.0",
                "instanceId": 1, "exportDate": "2026-06-09T00:00:00Z", "tables": meta,
            });
            zip.start_file("manifest.json", opts).unwrap();
            zip.write_all(&serde_json::to_vec_pretty(&manifest).unwrap())
                .unwrap();
            zip.finish().unwrap();
        }
        let tampered = cursor.into_inner();
        let err = parse_and_verify(&tampered).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("SHA-256")),
            "expected SHA mismatch, got {err:?}"
        );
    }

    #[test]
    fn parse_rejects_non_empty_files_dir() {
        let backup = build_test_backup(
            None,
            &[("companies", vec!["id"], b"")],
            true,
            Some("rogue.bin"),
        );
        let err = parse_and_verify(&backup).unwrap_err();
        assert!(matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("files/")));
    }

    #[test]
    fn parse_rejects_missing_table_coverage() {
        // Un seul fichier table, sans compléter les autres → couverture KO.
        let backup = build_test_backup(
            None,
            &[("companies", vec!["id"], b"{\"id\":1}\n")],
            false,
            None,
        );
        let err = parse_and_verify(&backup).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("absente du manifeste"))
        );
    }

    #[test]
    fn parse_rejects_missing_manifest() {
        // ZIP sans manifest.json.
        let mut cursor = Cursor::new(Vec::<u8>::new());
        {
            let mut zip = ZipWriter::new(&mut cursor);
            let opts = SimpleFileOptions::default();
            zip.add_directory("files/", opts).unwrap();
            zip.finish().unwrap();
        }
        let err = parse_and_verify(&cursor.into_inner()).unwrap_err();
        assert!(
            matches!(err, AppError::InvalidBackupStructure(ref m) if m.contains("manifest.json absent"))
        );
    }
}
