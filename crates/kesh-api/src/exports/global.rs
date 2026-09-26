//! Story 9-2b T3 — Orchestrateur `build_global_export` + builder ZIP +
//! struct `GlobalExportMeta` (retournée au handler pour audit + tracing).
//!
//! Pipeline (un CSV par entrée de [`TABLES_EXPORTEES`], plus un manifeste) :
//! 1. `Instant::now()` start.
//! 2. Pour chaque table exportée : appel repo (un SELECT par table, scopé
//!    par `company_id` — directement, ou par jointure sur le parent).
//! 3. Pour chaque table : `serialize_<table>_csv(rows, &mut Vec<u8>)` →
//!    `Vec<u8>` par CSV.
//! 4. SHA-256 par CSV (sur les bytes décompressés) → `TableMeta`.
//! 5. `build_metadata_json(...)` → `metadata.json` bytes (camelCase
//!    + BTreeMap tables alphabétique).
//! 6. `build_zip(&files)` — les CSV puis `metadata.json` (en dernier,
//!    Pass 1 ECH-LOW-02).
//! 7. `GlobalExportMeta { byte_size, csv_count, duration_ms }`.
//!
//! ⚠️ **Aucun nombre de tables n'est écrit ici** : ce doc-comment en portait
//! trois (19, 18, 18), tous faux après la 25-5-a (revue P1). Le seul nombre
//! qui compte est **dérivé** de [`TABLES_EXPORTEES`].
//!
//! ⚠️ **Lectures successives, sans instantané commun** : chaque table est lue
//! par sa propre requête sur le pool. Une écriture concurrente pendant
//! l'export peut donc y laisser une incohérence (ligne fille sans parent).
//! Défaut antérieur à la 25-5-a, tracé par #465.

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
// Story 25-5-a (#386) — les onze tables comptables ajoutées.
use crate::exports::csv_tables::{
    serialize_audit_log_csv, serialize_contact_persons_csv, serialize_credit_note_lines_csv,
    serialize_credit_notes_csv, serialize_imported_supplier_invoices_csv,
    serialize_invoice_settlements_csv, serialize_payment_batch_items_csv,
    serialize_payment_batches_csv, serialize_projects_csv, serialize_supplier_invoice_lines_csv,
    serialize_supplier_invoices_csv,
};
use crate::exports::metadata::{TableMeta, build_metadata_json, sha256_hex};

use kesh_db::entities::Company;
use kesh_db::repositories::{
    accounts, bank_accounts, bank_imports, bank_profiles, bank_transactions,
    company_dunning_settings, company_invoice_settings, contacts, dunning_levels, fiscal_years,
    invoice_reminders, invoices, journal_entries, reconciliation_rules, vat_rates,
};
// Story 25-5-a (#386).
use kesh_db::repositories::{
    audit_log, contact_persons, credit_notes, imported_supplier_invoices, invoice_settlements,
    payment_batches, projects, supplier_invoices,
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
/// Les tables que l'export de souveraineté **porte** (Story 25-5-a, #386).
///
/// ⛔ **Ce registre n'est pas une commodité : c'est un des deux membres de la
/// garde `export_couvre_toutes_les_tables`.** L'autre est
/// [`kesh_db::backup::TABLES_TO_TRUNCATE`], tenu à jour et lui-même contrôlé
/// contre `information_schema`. Une table qui apparaît au schéma sans entrer
/// ici ni dans [`TABLES_HORS_EXPORT`] fait **rougir le gate**.
///
/// *Sans cette garde, l'export s'est périmé trois fois en silence : écrit à la
/// Story 9-2b, étendu une fois à l'Epic 21, et jamais rattrapé sur les Epics 12
/// (fournisseurs, avoirs), 19 (projets) ni 24 (règlements).*
pub const TABLES_EXPORTEES: &[&str] = &[
    // Les 19 d'origine (Stories 9-2b, 21-5a, v014-1).
    //
    // ⚠️ `companies` est exportée sous le nom de fichier **`company.csv`**, au
    // singulier, et ne porte QUE la société courante — l'export est scopé. Ce
    // registre nomme la TABLE, non le fichier ; confondre les deux l'a fait
    // compter à tort parmi les absentes lors de la spécification.
    "companies",
    "fiscal_years",
    "accounts",
    "journal_entries",
    "journal_entry_lines",
    "contacts",
    "products",
    "invoices",
    "invoice_lines",
    "bank_accounts",
    "bank_imports",
    "bank_transactions",
    "vat_rates",
    "company_invoice_settings",
    "dunning_levels",
    "company_dunning_settings",
    "invoice_reminders",
    "reconciliation_rules",
    "bank_profiles",
    // Les onze de la 25-5-a (#386).
    "credit_notes",
    "credit_note_lines",
    "supplier_invoices",
    "supplier_invoice_lines",
    "payment_batches",
    "payment_batch_items",
    "invoice_settlements",
    "projects",
    "contact_persons",
    "imported_supplier_invoices",
    "audit_log",
];

/// Les tables **délibérément absentes** de l'export, et pourquoi.
///
/// ⛔ **Le motif est OBLIGATOIRE, et c'est tout l'intérêt.** Le patron est celui
/// des exemptions de rejeu (`kesh_db::post_restore::EXEMPT_MIGRATIONS`, P7) :
/// une exclusion sans justification écrite est une omission qui se déguise en
/// décision. La garde refuse un motif vide.
///
/// ⚠️ **Critère de tri, arbitré le 2026-09-23** : *« l'export CSV pour migrer
/// vers une autre application n'a besoin que des données comptables ; seuls les
/// backups de Kesh pour restaurer dans Kesh ont besoin de toutes les
/// informations »*. Les dix ci-dessous sont **dans la sauvegarde**, qui porte
/// les 39 tables.
pub const TABLES_HORS_EXPORT: &[(&str, &str)] = &[
    (
        "invoice_number_sequences",
        "Compteur interne à Kesh : les numéros attribués sont déjà portés par les \
         pièces exportées (`invoices.invoice_number`). Le compteur lui-même n'a aucun \
         sens dans un autre logiciel.",
    ),
    (
        "credit_note_number_sequences",
        "Compteur interne, cf. `invoice_number_sequences` — les numéros sont sur les avoirs.",
    ),
    (
        "journal_entry_number_sequences",
        "Compteur interne, cf. `invoice_number_sequences` — les numéros sont sur les écritures.",
    ),
    (
        "api_keys",
        "SECRET. Contient les condensés des clés d'API. Les exporter dans un fichier \
         destiné à être transmis à un tiers serait une fuite, et le condensé ne permet \
         de toute façon pas de restaurer la clé.",
    ),
    (
        "refresh_tokens",
        "SECRET et éphémère : jetons de session en cours. Aucune valeur comptable, et \
         leur export élargirait la surface d'attaque sans rien apporter.",
    ),
    (
        "password_reset_tokens",
        "SECRET et éphémère, cf. `refresh_tokens`.",
    ),
    (
        "users",
        "Données personnelles, et SANS PERTE pour la comptabilité : depuis la 25-1a, \
         chaque entrée d'audit porte le nom de son auteur en INSTANTANÉ \
         (`audit_log.actor_label`), figé au moment de l'écriture. Qui a fait quoi reste \
         donc lisible dans `audit_log.csv` sans exporter le registre des comptes.",
    ),
    (
        "email_templates",
        "Configuration de l'instance, non comptabilité : des gabarits d'e-mail n'ont \
         aucun sens dans un autre logiciel.",
    ),
    (
        "onboarding_state",
        "État d'installation de l'instance, non comptabilité.",
    ),
];

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

    // -------- 1. Une requête par table exportée --------
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

    // -------- 1-bis. Story 25-5-a (#386) : les onze tables comptables qui
    // manquaient. Toutes ces lectures sont NON BORNÉES et scopées `company_id` ;
    // cf. leurs doc-comments côté dépôt.
    let credit_notes_rows = credit_notes::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let credit_note_lines_rows = credit_notes::list_all_lines_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let supplier_invoices_rows = supplier_invoices::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let supplier_invoice_lines_rows =
        supplier_invoices::list_all_lines_by_company(pool, company_id)
            .await
            .map_err(map_db)?;
    let payment_batches_rows = payment_batches::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let payment_batch_items_rows = payment_batches::list_all_items_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let invoice_settlements_rows = invoice_settlements::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    // include_archived = true : souveraineté, cf. accounts/contacts plus haut.
    let projects_rows =
        projects::list_by_company(pool, company_id, /*include_archived=*/ true)
            .await
            .map_err(map_db)?;
    let contact_persons_rows = contact_persons::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;
    let imported_supplier_invoices_rows =
        imported_supplier_invoices::list_all_by_company(pool, company_id)
            .await
            .map_err(map_db)?;
    let audit_log_rows = audit_log::list_all_by_company(pool, company_id)
        .await
        .map_err(map_db)?;

    // -------- 2 & 3 & 4. Serialize + SHA-256 + assemblage ordonné --------
    //
    // Ordre du Vec = ordre d'écriture dans le ZIP ; `metadata.json` en dernier.
    let mut tables_meta: BTreeMap<String, TableMeta> = BTreeMap::new();
    let mut files: Vec<(String, Vec<u8>)> = Vec::with_capacity(TABLES_EXPORTEES.len() + 1);

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

    // -------- Story 25-5-a (#386) : les onze tables ajoutées --------
    push_csv!(
        "credit_notes.csv",
        credit_notes_rows,
        serialize_credit_notes_csv
    );
    push_csv!(
        "credit_note_lines.csv",
        credit_note_lines_rows,
        serialize_credit_note_lines_csv
    );
    push_csv!(
        "supplier_invoices.csv",
        supplier_invoices_rows,
        serialize_supplier_invoices_csv
    );
    push_csv!(
        "supplier_invoice_lines.csv",
        supplier_invoice_lines_rows,
        serialize_supplier_invoice_lines_csv
    );
    push_csv!(
        "payment_batches.csv",
        payment_batches_rows,
        serialize_payment_batches_csv
    );
    push_csv!(
        "payment_batch_items.csv",
        payment_batch_items_rows,
        serialize_payment_batch_items_csv
    );
    push_csv!(
        "invoice_settlements.csv",
        invoice_settlements_rows,
        serialize_invoice_settlements_csv
    );
    push_csv!("projects.csv", projects_rows, serialize_projects_csv);
    push_csv!(
        "contact_persons.csv",
        contact_persons_rows,
        serialize_contact_persons_csv
    );
    push_csv!(
        "imported_supplier_invoices.csv",
        imported_supplier_invoices_rows,
        serialize_imported_supplier_invoices_csv
    );
    push_csv!("audit_log.csv", audit_log_rows, serialize_audit_log_csv);

    // ⛔ **Le nombre est DÉRIVÉ, il ne s'écrit plus.** Il était codé en dur à
    // sept endroits — dont deux `debug_assert_eq!` qui **paniquaient en test**
    // et un `csv_count` littéral qui aurait fait mentir le manifeste. Chacun se
    // périmait à chaque table ajoutée, et trois epics l'ont prouvé.
    // *Un compteur qu'aucun calcul ne tient se périme en silence.*
    debug_assert_eq!(
        files.len(),
        TABLES_EXPORTEES.len(),
        "chaque table de TABLES_EXPORTEES doit avoir produit un CSV"
    );
    debug_assert_eq!(tables_meta.len(), TABLES_EXPORTEES.len());

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
            csv_count: TABLES_EXPORTEES.len(),
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
    use std::collections::BTreeSet;

    use super::*;
    use kesh_db::backup::TABLES_TO_TRUNCATE;

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

    // -----------------------------------------------------------------------
    // Story 25-5-a (#386) — LA GARDE D'EXHAUSTIVITÉ
    // -----------------------------------------------------------------------

    /// ⛔ **Chaque table du schéma est soit EXPORTÉE, soit EXCLUE AVEC MOTIF.**
    ///
    /// C'est la vraie livraison de la 25-5-a, et non les onze exports qu'elle
    /// ajoute. Ceux-ci réparent une fois ; celle-ci empêche que cela se
    /// recasse. *L'export a été écrit à la Story 9-2b, étendu une fois à
    /// l'Epic 21, et jamais rattrapé sur les Epics 12 (fournisseurs, avoirs),
    /// 19 (projets) ni 24 (règlements) : **trois epics ont rouvert le trou sans
    /// que rien ne rougisse**.*
    ///
    /// ⛔ **Les deux membres viennent de SOURCES DIFFÉRENTES, et c'est la
    /// condition pour que l'assertion veuille dire quelque chose** :
    /// `TABLES_TO_TRUNCATE` est tenue par `kesh-db` et contrôlée contre
    /// `information_schema` ; `TABLES_EXPORTEES` est tenue ici. Tirer les deux
    /// de la même source rendrait le test vert **par construction** — c'est
    /// l'`AC7` de l'Epic 23, et il ne mesurait rien.
    #[test]
    fn export_couvre_toutes_les_tables() {
        let exportees: BTreeSet<&str> = TABLES_EXPORTEES.iter().copied().collect();
        let exclues: BTreeSet<&str> = TABLES_HORS_EXPORT.iter().map(|(t, _)| *t).collect();

        // ⛔ On inventorie les NON RÉSOLUS, on n'énumère pas ce qui marche
        // (D4-ter) : une table oubliée ne peut pas contourner cette liste.
        let orphelines: Vec<&str> = TABLES_TO_TRUNCATE
            .iter()
            .copied()
            .filter(|t| !exportees.contains(t) && !exclues.contains(t))
            .collect();

        assert!(
            orphelines.is_empty(),
            "\nCes tables ne sont NI exportées NI exclues : {orphelines:?}\n\
             → soit les ajouter à `TABLES_EXPORTEES` (avec leur sérialiseur et leur \
             `push_csv!`), soit à `TABLES_HORS_EXPORT` AVEC UN MOTIF ÉCRIT.\n\
             ⚠️ Ne pas choisir l'exclusion par facilité : c'est le chemin le moins \
             coûteux, donc celui qu'il faut justifier."
        );
    }

    /// Une exclusion sans motif est une omission déguisée en décision.
    #[test]
    fn toute_exclusion_porte_un_motif() {
        let muettes: Vec<&str> = TABLES_HORS_EXPORT
            .iter()
            .filter(|(_, motif)| motif.trim().len() < 20)
            .map(|(t, _)| *t)
            .collect();
        assert!(
            muettes.is_empty(),
            "motif absent ou trop court pour : {muettes:?} — \
             écrire POURQUOI la table ne sort pas, pas seulement qu'elle ne sort pas"
        );
    }

    /// Les deux registres ne se recouvrent pas, et ne citent rien d'inconnu.
    ///
    /// ⚠️ Sans ce contrôle, une table pourrait figurer **des deux côtés** — et
    /// la garde ci-dessus resterait verte en la croyant traitée deux fois.
    #[test]
    fn les_deux_registres_sont_disjoints_et_connus() {
        let connues: BTreeSet<&str> = TABLES_TO_TRUNCATE.iter().copied().collect();
        let exclues: BTreeSet<&str> = TABLES_HORS_EXPORT.iter().map(|(t, _)| *t).collect();

        let doublons: Vec<&&str> = TABLES_EXPORTEES
            .iter()
            .filter(|t| exclues.contains(*t))
            .collect();
        assert!(
            doublons.is_empty(),
            "tables à la fois exportées ET exclues : {doublons:?}"
        );

        // ⚠️ `company` (singulier) est le NOM DU FICHIER, pas celui de la table :
        // le registre porte `companies`, et c'est bien elle qui est exportée.
        let inconnues: Vec<&&str> = TABLES_EXPORTEES
            .iter()
            .chain(exclues.iter())
            .filter(|t| !connues.contains(*t))
            .collect();
        assert!(
            inconnues.is_empty(),
            "tables citées mais absentes du schéma : {inconnues:?} — \
             un nom périmé rend la garde muette sur la table réelle"
        );
    }
}
