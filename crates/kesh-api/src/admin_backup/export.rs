//! Story 17-3a — Cœur de l'export complet d'installation (`.keshbackup`).
//!
//! [`build_keshbackup`] assemble le conteneur ZIP **sans émettre d'audit** :
//! l'audit `admin.full_export` est ajouté par le handler `routes::admin`, et
//! la sous-story 17-3c réutilisera ce cœur pour le backup pré-import (qui, lui,
//! ne doit PAS auditer).
//!
//! Depuis la Story 28-3 (#633), la base est lue par **un seul** appel,
//! `kesh_db::backup::export_installation`, qui lit toutes les tables dans un
//! instantané commun (`kesh_db::snapshot::ReadSnapshot`) : le fichier
//! photographie la base à un seul instant, et une saisie concurrente ne peut
//! plus y laisser une ligne sans son parent. Ce module n'assemble plus que le
//! ZIP, après la clôture de l'instantané.
//!
//! Structure du ZIP (cf. spec parente §Format normatif) :
//! ```text
//! manifest.json              # au ROOT
//! data/<table>.ndjson        # une par table de `TABLES_TO_TRUNCATE`
//! files/                     # dossier vide (forward-compat)
//! ```

use std::collections::BTreeMap;
use std::io::{Cursor, Write};

use sqlx::MySqlPool;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::admin_backup::manifest::{BackupManifest, BackupTableMeta, build_backup_manifest_json};
use crate::errors::AppError;
use crate::exports::metadata::sha256_hex;

use kesh_db::backup::{InstallationExport, TABLES_TO_TRUNCATE, export_installation};

/// Métadonnées retournées au handler (audit + tracing).
#[derive(Debug, Clone, Copy)]
pub struct KeshBackupMeta {
    pub byte_size: usize,
    pub table_count: usize,
    pub total_rows: usize,
}

/// Assemble le `.keshbackup` complet **en mémoire**, sans audit.
///
/// Toutes les lectures passent par `export_installation` : un seul instantané,
/// en lecture seule, clos avant la compression (Story 28-3, #633). La garde
/// `tests::build_keshbackup_ne_lit_la_base_qu_une_fois` y tient.
///
/// DC8 : l'assemblage est in-memory ; le handler décide ensuite de la
/// livraison (in-memory vs spill fichier temporaire + streaming) selon le
/// plafond `KESH_ADMIN_EXPORT_INMEM_MB`.
///
/// Erreurs : [`AppError::AdminFullExportFailed`]. ⚠️ L'import l'appelle pour
/// **produire** la sauvegarde pré-import : il convertit l'échec par
/// `routes::admin::avant_sauvegarde` en `AdminPreImportBackupFailed`
/// (Story 15-13b, #576) ; l'export, lui, le rend tel quel.
pub async fn build_keshbackup(pool: &MySqlPool) -> Result<(Vec<u8>, KeshBackupMeta), AppError> {
    let InstallationExport {
        min_required,
        instance_id,
        tables,
    } = export_installation(pool).await.map_err(map_db)?;

    let mut tables_meta: BTreeMap<String, BackupTableMeta> = BTreeMap::new();
    let mut total_rows = 0usize;

    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let opts: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);

        // Une entrée NDJSON par table (ordre TABLES_TO_TRUNCATE).
        for (table, export) in tables {
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

#[cfg(test)]
mod tests {
    /// Corps de production de `build_keshbackup` : code avant la première ligne
    /// `#[cfg(test)]`, lignes `//` (donc `///`) écartées, de la ligne
    /// `pub async fn build_keshbackup(` à la première ligne réduite à `}`.
    /// Même découpe que `routes::admin::tests::code_de_production`.
    fn corps_de_build_keshbackup() -> String {
        let prod: Vec<&str> = include_str!("export.rs")
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect();
        let debut = prod
            .iter()
            .position(|l| l.starts_with("pub async fn build_keshbackup("))
            .expect("assertion de montage : `pub async fn build_keshbackup(` introuvable");
        let fin = debut
            + prod[debut..]
                .iter()
                .position(|l| *l == "}")
                .expect("assertion de montage : fin de fonction introuvable");
        assert!(fin > debut + 1, "assertion de montage : corps vide");
        prod[debut..=fin].join("\n")
    }

    /// Story 28-3 (#633), AC5 (g)(ii) — garde lexicale : `build_keshbackup` ne
    /// nomme le pool que deux fois — son paramètre et l'unique
    /// `export_installation(pool)` —, si bien qu'aucune lecture n'y échappe à
    /// l'instantané commun (M10 : une relecture dans un second instantané
    /// compile, et seule cette garde la voit).
    #[test]
    fn build_keshbackup_ne_lit_la_base_qu_une_fois() {
        let corps = corps_de_build_keshbackup();
        let n = regex::Regex::new(r"\bpool\b")
            .expect("regex")
            .find_iter(&corps)
            .count();
        assert_eq!(
            n, 2,
            "`pool` doit apparaître exactement deux fois (paramètre, export_installation) :\n{corps}"
        );
    }
}
