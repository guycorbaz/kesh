//! Story 17-3a — Cœur de l'export complet d'installation (`.keshbackup`).
//!
//! [`build_keshbackup`] assemble le conteneur ZIP **sans émettre d'audit** :
//! l'audit `admin.full_export` est ajouté par le handler `routes::admin`, et
//! la sous-story 17-3c réutilisera ce cœur pour le backup pré-import (qui, lui,
//! ne doit PAS auditer).
//!
//! Structure du ZIP (cf. spec parente §Format normatif) :
//! ```text
//! manifest.json              # au ROOT
//! data/<table>.ndjson        # 1 par table applicative (22)
//! files/                     # dossier vide (forward-compat)
//! ```

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use sqlx::MySqlPool;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::admin_backup::manifest::{BackupManifest, BackupTableMeta, build_backup_manifest_json};
use crate::errors::AppError;
use crate::exports::metadata::sha256_hex;

use kesh_db::backup::{TABLES_TO_TRUNCATE, export_table, read_instance_id, read_min_required};

/// Métadonnées retournées au handler (audit + tracing).
#[derive(Debug, Clone, Copy)]
pub struct KeshBackupMeta {
    pub byte_size: usize,
    pub table_count: usize,
    pub total_rows: usize,
}

/// Assemble le `.keshbackup` complet **en mémoire**, sans audit.
///
/// DC8 : l'assemblage est in-memory ; le handler décide ensuite de la
/// livraison (in-memory vs spill fichier temporaire + streaming) selon le
/// plafond `KESH_ADMIN_EXPORT_INMEM_MB`.
pub async fn build_keshbackup(pool: &MySqlPool) -> Result<(Vec<u8>, KeshBackupMeta), AppError> {
    let min_required = read_min_required(pool).await.map_err(map_db)?;
    let instance_id = read_instance_id(pool).await.map_err(map_db)?;

    let mut tables_meta: BTreeMap<String, BackupTableMeta> = BTreeMap::new();
    let mut total_rows = 0usize;

    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let opts: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // Une entrée NDJSON par table (ordre TABLES_TO_TRUNCATE).
        for &table in TABLES_TO_TRUNCATE {
            let export = export_table(pool, table).await.map_err(map_db)?;
            let sha = sha256_hex(&export.ndjson);
            total_rows += export.row_count;
            tables_meta.insert(
                table.to_string(),
                BackupTableMeta {
                    row_count: export.row_count,
                    sha256: sha,
                    column_names: export.column_names,
                },
            );
            zip.start_file(format!("data/{table}.ndjson"), opts)
                .map_err(|e| {
                    AppError::AdminFullExportFailed(format!("zip start data/{table}: {e}"))
                })?;
            zip.write_all(&export.ndjson).map_err(|e| {
                AppError::AdminFullExportFailed(format!("zip write data/{table}: {e}"))
            })?;
        }

        // Dossier vide files/ (forward-compat — aucun binaire stocké en v0.2).
        zip.add_directory("files/", opts).map_err(|e| {
            AppError::AdminFullExportFailed(format!("zip add_directory files/: {e}"))
        })?;

        // Manifest en dernier (cohérent lecture humaine + pattern 9-2b).
        let manifest = BackupManifest::new(min_required, instance_id, tables_meta);
        let manifest_bytes = build_backup_manifest_json(&manifest)?;
        zip.start_file("manifest.json", opts)
            .map_err(|e| AppError::AdminFullExportFailed(format!("zip start manifest: {e}")))?;
        zip.write_all(&manifest_bytes)
            .map_err(|e| AppError::AdminFullExportFailed(format!("zip write manifest: {e}")))?;

        zip.finish()
            .map_err(|e| AppError::AdminFullExportFailed(format!("zip finish: {e}")))?;
    }

    let bytes = cursor.into_inner();
    let meta = KeshBackupMeta {
        byte_size: bytes.len(),
        table_count: TABLES_TO_TRUNCATE.len(),
        total_rows,
    };
    Ok((bytes, meta))
}

/// Mappe une `DbError` vers [`AppError::AdminFullExportFailed`].
fn map_db(e: kesh_db::errors::DbError) -> AppError {
    AppError::AdminFullExportFailed(format!("db: {e}"))
}
