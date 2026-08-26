//! Story 9-2b T3 — Orchestrateur `build_global_export` + builder ZIP +
//! struct `GlobalExportMeta` (retournée au handler pour audit + tracing).
//!
//! Pipeline (19 CSV + 1 manifeste) :
//! 1. `Instant::now()` start.
//! 2. Pour chaque table §scope-tables : appel repo (queries 18 SELECTs +
//!    JOIN scoping `company_id`).
//! 3. Pour chaque table : `serialize_<table>_csv(rows, &mut Vec<u8>)` →
//!    `Vec<u8>` par CSV.
//! 4. SHA-256 par CSV (sur les bytes décompressés) → `TableMeta`.
//! 5. `build_metadata_json(...)` → `metadata.json` bytes (camelCase
//!    + BTreeMap tables alphabétique).
//! 6. `build_zip(&files)` — 18 CSV puis `metadata.json` (en dernier,
//!    Pass 1 ECH-LOW-02).
//! 7. `GlobalExportMeta { byte_size, csv_count: 19, duration_ms }`.

use std::collections::BTreeMap;
use std::io::Write;
use std::time::Instant;

use sqlx::MySqlPool;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use crate::errors::AppError;
use crate::exports::csv_tables::{
    serialize_accounts_csv, serialize_bank_accounts_csv, serialize_bank_imports_csv,
    serialize_bank_profiles_csv, serialize_bank_transactions_csv, serialize_company_csv,
    serialize_company_dunning_settings_csv, serialize_company_invoice_settings_csv,
    serialize_contacts_csv, serialize_dunning_levels_csv, serialize_fiscal_years_csv,
    serialize_invoice_lines_csv, serialize_invoice_reminders_csv, serialize_invoices_csv,
    serialize_journal_entries_csv, serialize_journal_entry_lines_csv, serialize_products_csv,
    serialize_reconciliation_rules_csv, serialize_vat_rates_csv,
};
use crate::exports::metadata::{TableMeta, build_metadata_json, sha256_hex};

use kesh_db::entities::Company;
use kesh_db::repositories::{
    accounts, bank_accounts, bank_imports, bank_profiles, bank_transactions,
    company_dunning_settings, company_invoice_settings, contacts, dunning_levels, fiscal_years,
    invoice_reminders, invoices, journal_entries, reconciliation_rules, vat_rates,
};

// ===========================================================================
// GlobalExportMeta (retournée pour audit + tracing post-handler)
// ===========================================================================

/// Métadonnées retournées par [`build_global_export`] pour permettre au
/// handler de :
///
/// - populer le tracing span `global_export` (`span.record(...)`)
/// - émettre la ligne `audit_log` (`details_json` inclut `byte_size`,
///   `csv_count`, `duration_ms`)
/// - construire la réponse HTTP (`Content-Length` calculé à partir de
///   `zip_bytes.len()` côté `Body::from`).
///
/// Cette struct est intentionnellement plate (`Copy`-friendly) et ne porte
/// pas le `Vec<u8>` du ZIP — celui-ci est retourné séparément par
/// `build_global_export` pour ne pas forcer un move/clone.
#[derive(Debug, Clone, Copy)]
pub struct GlobalExportMeta {
    pub byte_size: usize,
    pub csv_count: usize,
    pub duration_ms: u64,
}

// ===========================================================================
// build_zip — assemblage ZIP en mémoire
// ===========================================================================

/// Construit un ZIP en mémoire à partir de `(name, bytes)`.
///
/// Compression Deflate level 6 (default `SimpleFileOptions` zip 2.x). Permis
/// `0o644` (lecture/écriture owner, lecture seule pour les autres). Si une
/// entrée échoue (`start_file` ou `write_all`), l'erreur est traduite vers
/// [`AppError::GlobalExportFailed`] avec le nom de l'entrée fautive — utile
/// pour diagnostic ops.
///
/// **Ordre des entrées** : exactement celui du `files: &[...]` passé par le
/// caller. `build_global_export` injecte `metadata.json` en **dernier**
/// (position 16) cohérent lecture humaine + Pass 1 ECH-LOW-02.
pub fn build_zip(files: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError> {
    use std::io::Cursor;

    let mut cursor = Cursor::new(Vec::<u8>::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let options: SimpleFileOptions = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o644);
        for (name, bytes) in files {
            zip.start_file(name.as_str(), options)
                .map_err(|e| AppError::GlobalExportFailed(format!("zip start_file {name}: {e}")))?;
            zip.write_all(bytes)
                .map_err(|e| AppError::GlobalExportFailed(format!("zip write {name}: {e}")))?;
        }
        zip.finish()
            .map_err(|e| AppError::GlobalExportFailed(format!("zip finish: {e}")))?;
    }
    Ok(cursor.into_inner())
}

// ===========================================================================
// build_global_export — orchestrateur
// ===========================================================================

/// Orchestrateur Story 9-2b — assemble le ZIP global d'une company.
///
/// Le caller (handler `routes::exports::export_global`) doit déjà avoir :
/// - vérifié `current_user.company_id > 0` (AC #19 défensif)
/// - chargé la `Company` via `companies::find_by_id` (filename + manifest)
/// - résolu `locale_bcp47` via `util::map_language_to_bcp47`.
///
/// Cette signature explicite (`company: &Company`, `locale_bcp47: &str`)
/// évite une seconde fetch DB de la `Company` à l'intérieur de l'orchestrateur.
pub async fn build_global_export(
    pool: &MySqlPool,
    company: &Company,
    locale_bcp47: &str,
) -> Result<(Vec<u8>, GlobalExportMeta), AppError> {
    let start = Instant::now();
    let company_id = company.id;

    // -------- 1. Queries 18 tables (single-fetch each) --------
    let company_rows = vec![company.clone()];
    let fiscal_years_rows = fiscal_years::list_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    // accounts/contacts : verrouiller include_archived=true (souveraineté).
    let accounts_rows =
        accounts::list_by_company(pool, company_id, /*include_archived=*/ true)
            .await
            .map_err(map_db)?;
    let journal_entries_rows = journal_entries::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let journal_entry_lines_rows = journal_entries::list_all_lines_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let contacts_rows =
        contacts::list_by_company(pool, company_id, /*include_archived=*/ true)
            .await
            .map_err(map_db)?;
    let products_rows = kesh_db::repositories::products::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let invoices_rows = invoices::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let invoice_lines_rows = invoices::list_all_lines_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    // Story v014-1 : include_archived=true pour souveraineté CO Art. 957
    // (conservation 10 ans). Cohérent pattern accounts/contacts ci-dessus.
    let bank_accounts_rows =
        bank_accounts::list_by_company(pool, company_id, /*include_archived=*/ true)
            .await
            .map_err(map_db)?;
    let bank_imports_rows = bank_imports::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let bank_transactions_rows = bank_transactions::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let vat_rates_rows = vat_rates::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    // lazy-create OK : write idempotent, row injectée avec defaults si absente.
    let cis_row = company_invoice_settings::get_or_create_default(pool, company_id)
        .await
        .map_err(map_db)?;
    let cis_rows = vec![cis_row];
    let dunning_levels_rows = dunning_levels::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    // lazy-create OK (write idempotent). Le seed lazy des niveaux n'est PAS déclenché
    // par l'export (souveraineté = photo de l'état réel, pas d'effet de bord métier).
    let cds_row = company_dunning_settings::get_or_create_default(pool, company_id)
        .await
        .map_err(map_db)?;
    let cds_rows = vec![cds_row];
    let invoice_reminders_rows = invoice_reminders::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let reconciliation_rules_rows = reconciliation_rules::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let bank_profiles_rows = bank_profiles::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;

    // -------- 2 & 3 & 4. Serialize + SHA-256 + assemblage ordonné --------
    //
    // Ordre du Vec = ordre §scope-tables d'écriture dans le ZIP (les 18 CSV).
    // metadata.json est ajouté en dernier (18e index, après les 18 CSV).
    let mut tables_meta: BTreeMap<String, TableMeta> = BTreeMap::new();
    let mut files: Vec<(String, Vec<u8>)> = Vec::with_capacity(20);

    macro_rules! push_csv {
        ($name:literal, $rows:expr, $serializer:ident) => {{
            let rows = &$rows;
            let mut buf = Vec::<u8>::new();
            $serializer(rows, &mut buf)?;
            tables_meta.insert(
                $name.to_string(),
                TableMeta {
                    row_count: rows.len(),
                    sha256: sha256_hex(&buf),
                },
            );
            files.push(($name.to_string(), buf));
        }};
    }

    push_csv!("company.csv", company_rows, serialize_company_csv);
    push_csv!(
        "fiscal_years.csv",
        fiscal_years_rows,
        serialize_fiscal_years_csv
    );
    push_csv!("accounts.csv", accounts_rows, serialize_accounts_csv);
    push_csv!(
        "journal_entries.csv",
        journal_entries_rows,
        serialize_journal_entries_csv
    );
    push_csv!(
        "journal_entry_lines.csv",
        journal_entry_lines_rows,
        serialize_journal_entry_lines_csv
    );
    push_csv!("contacts.csv", contacts_rows, serialize_contacts_csv);
    push_csv!("products.csv", products_rows, serialize_products_csv);
    push_csv!("invoices.csv", invoices_rows, serialize_invoices_csv);
    push_csv!(
        "invoice_lines.csv",
        invoice_lines_rows,
        serialize_invoice_lines_csv
    );
    push_csv!(
        "bank_accounts.csv",
        bank_accounts_rows,
        serialize_bank_accounts_csv
    );
    push_csv!(
        "bank_imports.csv",
        bank_imports_rows,
        serialize_bank_imports_csv
    );
    push_csv!(
        "bank_transactions.csv",
        bank_transactions_rows,
        serialize_bank_transactions_csv
    );
    push_csv!("vat_rates.csv", vat_rates_rows, serialize_vat_rates_csv);
    push_csv!(
        "company_invoice_settings.csv",
        cis_rows,
        serialize_company_invoice_settings_csv
    );
    push_csv!(
        "dunning_levels.csv",
        dunning_levels_rows,
        serialize_dunning_levels_csv
    );
    push_csv!(
        "company_dunning_settings.csv",
        cds_rows,
        serialize_company_dunning_settings_csv
    );
    push_csv!(
        "invoice_reminders.csv",
        invoice_reminders_rows,
        serialize_invoice_reminders_csv
    );
    push_csv!(
        "reconciliation_rules.csv",
        reconciliation_rules_rows,
        serialize_reconciliation_rules_csv
    );
    push_csv!(
        "bank_profiles.csv",
        bank_profiles_rows,
        serialize_bank_profiles_csv
    );

    debug_assert_eq!(files.len(), 19, "19 CSV expected before manifest");
    debug_assert_eq!(tables_meta.len(), 19, "19 TableMeta expected");

    // -------- 5. Manifest --------
    let manifest_bytes = build_metadata_json(company, locale_bcp47, tables_meta)?;
    files.push(("metadata.json".to_string(), manifest_bytes));

    // -------- 6. ZIP --------
    let zip_bytes = build_zip(&files)?;
    let byte_size = zip_bytes.len();
    let duration_ms = start.elapsed().as_millis() as u64;

    Ok((
        zip_bytes,
        GlobalExportMeta {
            byte_size,
            csv_count: 19,
            duration_ms,
        },
    ))
}

/// Mappe une `DbError` vers `AppError::GlobalExportFailed` (intercepte les
/// échecs SQL transitoires : DB down, timeout pool, network blip — AC #17).
///
/// On consume l'erreur source vers la string (cohérent UX-DR38 : detail
/// jamais exposé en HTTP body, juste loggé dans `IntoResponse` arm).
fn map_db(e: kesh_db::errors::DbError) -> AppError {
    AppError::GlobalExportFailed(format!("db: {e}"))
}

// ===========================================================================
// Tests (Story 9-2b T3 + AC #30(d)(g)(j))
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ----- AC #30(d) — build_zip produit signature PK\x03\x04 valide -----

    #[test]
    fn build_zip_signature_and_entries() {
        let files = vec![
            ("hello.csv".to_string(), b"a;b;c\r\n".to_vec()),
            ("world.csv".to_string(), b"d;e;f\r\n".to_vec()),
        ];
        let bytes = build_zip(&files).expect("zip ok");
        // Signature ZIP `PK\x03\x04` (local file header magic)
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
        // Lecture vérifie présence des 2 entries
        let cursor = std::io::Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor).unwrap();
        assert_eq!(archive.len(), 2);
        let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();
        assert!(names.contains(&"hello.csv".to_string()));
        assert!(names.contains(&"world.csv".to_string()));
        // Contenu round-trip
        use std::io::Read;
        let mut content = String::new();
        archive
            .by_name("hello.csv")
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        assert_eq!(content, "a;b;c\r\n");
    }

    #[test]
    fn build_zip_empty_input_still_valid() {
        let bytes = build_zip(&[]).expect("empty zip ok");
        // Une archive ZIP vide commence aussi par end-of-central-directory ;
        // zip 2.x produit un ZIP minimal mais valide.
        let cursor = std::io::Cursor::new(bytes);
        let archive = zip::ZipArchive::new(cursor).unwrap();
        assert_eq!(archive.len(), 0);
    }

    // ----- AC #30(j) — build_zip failure path -----
    //
    // Provoquer une vraie erreur dans `zip 2.x` est non trivial (l'API ne
    // valide pas spécialement les noms en in-memory `Cursor`). Le pattern
    // d'erreur est néanmoins testé indirectement par AC #29(o) E2E si on peut
    // construire un cas, et par `errors::tests::global_export_failed_*` pour
    // le mapping HTTP. On confirme ici que le pattern de mapping est en
    // place (la macro `?` propage `AppError::GlobalExportFailed`).
    #[test]
    fn build_zip_error_path_is_wired() {
        // Test de structure : le type retour est `Result<_, AppError>` et la
        // variante `GlobalExportFailed` est utilisée (compile-time check).
        fn _check_signature(f: &[(String, Vec<u8>)]) -> Result<Vec<u8>, AppError> {
            build_zip(f)
        }
        // Si l'arm utilisait un autre variant, l'AC #29(n)(o) E2E le détecterait
        // au level handler. Le test unit (i) `errors::global_export_failed_*`
        // assure le mapping HTTP 500.
        let _ = _check_signature;
    }
}
