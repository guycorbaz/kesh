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
//! - [`export_installation`] : la lecture de toute l'installation pour le
//!   `.keshbackup`, dans **un seul instantané** ([`ReadSnapshot`], Story 28-3,
//!   #633) — les lectures de la sauvegarde exigent `&mut ReadSnapshot` et ne
//!   peuvent plus s'exécuter sur le pool, table par table, à des instants
//!   différents.
//!
//! Le format NDJSON est figé par la spec parente 17-3 (§Format normatif) :
//! UTF-8, fin de ligne LF, NULL → `null`, ordre des clés = `column_names`,
//! `Decimal` → string, `NaiveDateTime` → ISO 8601 `YYYY-MM-DDTHH:MM:SS[.mmm]`
//! (précision sub-seconde préservée — colonnes `DATETIME(3)`), entiers → nombre JSON.

use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

use serde_json::Value;
use sqlx::{MySql, MySqlPool, Row, Transaction};

use crate::errors::{DbError, map_db_error};
use crate::snapshot::ReadSnapshot;

/// Liste canonique des tables applicatives Kesh, ordonnée **enfants → parents**
/// (ordre de suppression FK-safe). Exclut les tables système `_sqlx_migrations`,
/// `_kesh_version` et, depuis la Story 31-1c, `_kesh_backup_schedule` (réglage
/// et état de la sauvegarde automatique) — jamais exportées ni vidées : y
/// ajouter une table rendrait non importable toute sauvegarde antérieure.
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
    "invoice_reminders", // Story 21-5a (#231) — enfant de invoices (CASCADE) + companies (RESTRICT) → avant `invoices`.
    // Story 24-2 (#371) — enfant de invoices (CASCADE) + companies (RESTRICT) +
    // journal_entries (RESTRICT) → doit passer AVANT les trois. En particulier
    // avant `journal_entry_lines`/`journal_entries` : la FK y est RESTRICT
    // (une écriture référencée ne se supprime pas, elle se contre-passe).
    "invoice_settlements",
    "journal_entry_lines",
    "projects", // Story 19-1 (Epic 19) — enfant de companies + self-ref parent_id (RESTRICT) ; référencé par journal_entry_lines.project_id → APRÈS journal_entry_lines, AVANT companies.
    "credit_notes", // Story 12.1 — enfant de invoices/journal_entries/contacts (RESTRICT) → avant eux.
    "imported_supplier_invoices", // Story 12.5b — enfant de companies (RESTRICT) + supplier_invoices (SET NULL) → avant eux.
    "supplier_invoices", // Story 12.2 — enfant de journal_entries/contacts/bank_accounts/accounts (RESTRICT) → avant eux.
    "payment_batches",   // Story 12.3 — enfant de companies + bank_accounts (RESTRICT) → avant eux.
    "invoices",
    "invoice_number_sequences",
    "credit_note_number_sequences", // Story 12.1 — enfant de companies + fiscal_years (RESTRICT).
    // Story 25-2-c (#381) — enfant de companies + fiscal_years (RESTRICT), sans
    // enfant propre : sa place suit celle de ses deux sœurs.
    //
    // ⛔ **Son absence d'ici ferait échouer la story sur son propre but, et en
    // silence** : un compteur non sauvegardé revient VIDE d'une restauration,
    // repart de 1, et réattribue des numéros déjà portés — précisément le défaut
    // muet que #381 ferme. Une table de séquence se sauvegarde comme ses deux
    // voisines, pour la même raison qu'elles.
    "journal_entry_number_sequences",
    "journal_entries",
    "audit_log",
    "api_keys", // Story 17-2a (#100) — enfant de companies + users (RESTRICT).
    // Story 32-5a (#723) — clés du serveur MCP : enfant de companies + users
    // (RESTRICT), sans enfant propre ; sa place suit celle d'`api_keys`.
    "mcp_keys",
    "company_invoice_settings",
    // Story 32-5a (#723) — réglages du serveur MCP : une ligne par société au
    // plus, enfant de companies (RESTRICT) ; sa place suit celle de
    // `company_invoice_settings`, autre ligne de réglages de société.
    "mcp_server_settings",
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
    "email_templates", // Story 20-1 (#224) — enfant de companies (CASCADE), aucune table ne référence email_templates.
    "dunning_levels", // Story 21-3 (#231) — enfant de companies (RESTRICT) ; aucune FK ne le référence (invoice_reminders 21-5a snapshote level_number/fee_amount sans FK — hard-delete + renumérotation sûrs).
    "company_dunning_settings", // Story 21-3 (#231) — singleton enfant de companies (RESTRICT).
    "refresh_tokens",
    "password_reset_tokens", // Story 17-4a (#122) — enfant de users (CASCADE), tokens recovery éphémères.
    "onboarding_state",
    "users",
    "companies",
];

/// Les tables que la **remise à zéro** d'une installation (`kesh_seed::reset_demo`)
/// **conserve** — Story 15-7b2 (AC 3, choix C-15-7-10, #279).
///
/// La société (remise à l'état provisoire **en place**, #528), l'état
/// d'onboarding (remis à zéro **en place**), les utilisateurs et ce qui ne
/// dépend que d'eux (sessions, jetons de réinitialisation de mot de passe), et
/// les clés — d'API et, depuis la Story 32-5d (C-32-5-11), du serveur MCP
/// (`mcp_keys`) : un principal n'est pas une donnée de la société. Tout le
/// reste de [`TABLES_TO_TRUNCATE`] est vidé : [`reset_cleared_tables`] —
/// `mcp_server_settings` compris, si bien qu'après une remise à zéro le
/// serveur MCP est **désactivé** et ses clés demeurent.
///
/// ⚠️ **La partition est gardée sur le schéma**, et non sur cette liste : le test
/// `reset_partition_is_guarded_by_the_schema` lit les clés étrangères
/// (`information_schema.KEY_COLUMN_USAGE`) et fait rougir toute table neuve
/// enfant de `users` seulement — elle serait sinon vidée en silence.
pub const RESET_PRESERVED_TABLES: &[&str] = &[
    "api_keys",
    "companies",
    "mcp_keys",
    "onboarding_state",
    "password_reset_tokens",
    "refresh_tokens",
    "users",
];

/// Les tables que la remise à zéro **vide** — Story 15-7b2 (AC 3, #279) :
/// [`TABLES_TO_TRUNCATE`] privé de [`RESET_PRESERVED_TABLES`], **dans l'ordre**
/// de [`TABLES_TO_TRUNCATE`] (enfants → parents), `audit_log` compris (la remise
/// à zéro l'efface puis y inscrit son propre geste, `installation.reset`).
///
/// **Dérivée**, jamais écrite en dur : une table ajoutée à la liste canonique
/// est vidée sans autre geste. Une fonction et non une `const` : une `const` ne
/// filtre pas une autre `const` en tranche de longueur inconnue ; le coût d'un
/// filtre de quelques dizaines de noms est nul pour un geste rare.
pub fn reset_cleared_tables() -> Vec<&'static str> {
    TABLES_TO_TRUNCATE
        .iter()
        .copied()
        .filter(|t| !RESET_PRESERVED_TABLES.contains(t))
        .collect()
}

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
    snap: &mut ReadSnapshot,
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
    .fetch_all(snap.conn())
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
///
/// Lit dans l'instantané reçu (Story 28-3, #633) : appelée par
/// [`export_installation`] pour chaque table, elle voit la base au même instant
/// que les autres tables de la sauvegarde.
pub async fn export_table(snap: &mut ReadSnapshot, table: &str) -> Result<TableExport, DbError> {
    let columns = non_generated_columns(snap, table).await?;
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
        .fetch_all(snap.conn())
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
pub async fn read_min_required(snap: &mut ReadSnapshot) -> Result<String, DbError> {
    sqlx::query_scalar::<_, String>(
        "SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(snap.conn())
    .await
    .map_err(map_db_error)
}

/// Identifiant informatif de l'installation source = `MIN(companies.id)`.
///
/// Invariant : ≥ 1 (l'endpoint export exige un Admin authentifié, et
/// `users.company_id` est `NOT NULL` ⇒ au moins une company existe). Retourne 0
/// défensivement si la table est vide (cas dégénéré impossible en pratique).
pub async fn read_instance_id(snap: &mut ReadSnapshot) -> Result<i64, DbError> {
    let v: Option<i64> = sqlx::query_scalar("SELECT MIN(id) FROM companies")
        .fetch_one(snap.conn())
        .await
        .map_err(map_db_error)?;
    Ok(v.unwrap_or(0))
}

/// Toutes les données d'une installation, lues dans **un seul instantané**
/// (Story 28-3, #633) — ce que `kesh-api` assemble en `.keshbackup`.
#[derive(Debug, Clone)]
pub struct InstallationExport {
    /// `kesh_version_min_required` de `_kesh_version`.
    pub min_required: String,
    /// Identifiant informatif de l'installation ([`read_instance_id`]).
    pub instance_id: i64,
    /// Une entrée par table de [`TABLES_TO_TRUNCATE`], dans son ordre.
    pub tables: Vec<(&'static str, TableExport)>,
    /// Story 31-1a (#503, AC1) — les fichiers du dossier des documents que la
    /// base désigne, lus **dans le même instantané** que les tables, après
    /// elles ([`read_referenced_files`]).
    pub referenced_files: Vec<ReferencedFile>,
}

/// Story 31-1a (#503, AC1) — un fichier du dossier des documents désigné par
/// une ligne de la base : un document fournisseur
/// (`imported_supplier_invoices.storage_path` / `file_hash`) ou le PDF figé
/// d'une facture (`invoices.pdf_storage_path` / `pdf_sha256`).
///
/// L'inventaire de ces colonnes est **clos** : le test
/// `tests::colonnes_storage_path_inventaire_clos` rougit si une colonne
/// `*storage_path*` apparaît ailleurs.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ReferencedFile {
    /// Nom relatif au dossier des documents, **tel que la base le porte** —
    /// éventuellement hors forme : c'est l'exporteur qui le contrôle.
    pub storage_path: String,
    /// Empreinte SHA-256 déclarée par la ligne (hex), chaîne vide si la ligne
    /// n'en porte pas.
    pub sha256: String,
}

/// Story 31-1a (#503, AC1) — lit les couples (chemin, empreinte) que la base
/// référence, dans l'instantané `snap`.
///
/// **Deux requêtes, jamais un `UNION` ni un `DISTINCT` SQL** : les colonnes
/// sont en `utf8mb4_unicode_ci`, insensible à la casse et aux espaces finaux,
/// qui fusionnerait `x.pdf` et `X.PDF` de deux lignes alors que l'import
/// compare les chemins octet à octet — la ligne fusionnée ne serait alors ni
/// sauvegardée ni déclarée manquante, et la sauvegarde authentique refusée.
/// Le dédoublonnage se fait ici, en Rust, sur l'égalité exacte des chaînes ;
/// le classement des anomalies (même chemin sous deux empreintes, préfixe qui
/// n'est pas l'empreinte, chemin hors forme) revient à l'exporteur.
///
/// Toutes les lignes de `imported_supplier_invoices` comptent, quel que soit
/// leur statut (`discarded` compris : la ligne reste restaurée et son document
/// téléchargeable). Rend la liste triée.
async fn read_referenced_files(snap: &mut ReadSnapshot) -> Result<Vec<ReferencedFile>, DbError> {
    let mut couples = std::collections::BTreeSet::new();
    let documents: Vec<(String, String)> =
        sqlx::query_as("SELECT storage_path, file_hash FROM imported_supplier_invoices")
            .fetch_all(snap.conn())
            .await
            .map_err(map_db_error)?;
    let pdf_figes: Vec<(String, Option<String>)> = sqlx::query_as(
        "SELECT pdf_storage_path, pdf_sha256 FROM invoices WHERE pdf_storage_path IS NOT NULL",
    )
    .fetch_all(snap.conn())
    .await
    .map_err(map_db_error)?;
    for (storage_path, sha256) in documents {
        couples.insert(ReferencedFile {
            storage_path,
            sha256,
        });
    }
    for (storage_path, sha256) in pdf_figes {
        couples.insert(ReferencedFile {
            storage_path,
            sha256: sha256.unwrap_or_default(),
        });
    }
    Ok(couples.into_iter().collect())
}

/// Futur boxé `Send`, alias local (std seule : `kesh-db` ne dépend pas du crate
/// `futures`).
type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Étape du cœur de la sauvegarde, signalée à l'observateur de
/// [`export_installation_in`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Etape {
    /// L'instantané est ouvert, aucune lecture n'a eu lieu.
    Ouvert,
    /// La table nommée vient d'être lue.
    ///
    /// Le champ n'est lu que par les tests (la production passe un observateur
    /// vide) : sans l'attribut, `clippy -D warnings` rougirait sur la cible lib.
    ApresTable(#[cfg_attr(not(test), allow(dead_code))] &'static str),
}

/// Lit toute l'installation — `min_required`, `instance_id`, puis chaque table
/// de [`TABLES_TO_TRUNCATE`] dans son ordre — dans **un seul**
/// [`ReadSnapshot`], clos avant de rendre la main (Story 28-3, #633).
///
/// Une saisie, une suppression ou une validation committée pendant la lecture
/// n'apparaît pas dans le résultat : aucune ligne fille n'y figure sans sa ligne
/// parente. La garantie vaut pour les tables InnoDB (manuel d'administration).
///
/// Une erreur de clôture de l'instantané (`finish`) fait échouer la lecture,
/// même si toutes les tables ont été lues (C-28-3-20).
pub async fn export_installation(pool: &MySqlPool) -> Result<InstallationExport, DbError> {
    let mut snap = ReadSnapshot::begin(pool).await?;
    let export = export_installation_in(&mut snap, &mut |_| Box::pin(async {})).await?;
    snap.finish().await?;
    Ok(export)
}

/// Cœur de [`export_installation`] : emprunte l'instantané (sans le clore) et
/// appelle l'observateur après l'ouverture puis après chaque table.
///
/// Il ne reçoit **aucun pool** : une lecture neuve écrite ici n'a d'autre
/// exécuteur que l'instantané. L'observateur est vide en production ; les
/// tests y placent une écriture concurrente, sur une autre connexion.
async fn export_installation_in(
    snap: &mut ReadSnapshot,
    obs: &mut (dyn FnMut(Etape) -> BoxFuture<'static, ()> + Send),
) -> Result<InstallationExport, DbError> {
    obs(Etape::Ouvert).await;
    let min_required = read_min_required(snap).await?;
    let instance_id = read_instance_id(snap).await?;
    let mut tables = Vec::with_capacity(TABLES_TO_TRUNCATE.len());
    for &table in TABLES_TO_TRUNCATE {
        let export = export_table(snap, table).await?;
        tables.push((table, export));
        obs(Etape::ApresTable(table)).await;
    }
    // Story 31-1a (AC1) : dans le même instantané, après les tables.
    let referenced_files = read_referenced_files(snap).await?;
    Ok(InstallationExport {
        min_required,
        instance_id,
        tables,
        referenced_files,
    })
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
    // DELETE enfants→parents (ordre TABLES_TO_TRUNCATE) — DEUX exclusions.
    //
    // `onboarding_state` : état local, jamais restauré (DC11, Story 17-3c).
    //
    // ⛔ `audit_log` **depuis la Story 25-1a (#376)** : la piste de contrôle n'est
    // plus remplacée, elle est **FUSIONNÉE**. Le motif n'est pas la perte d'un
    // historique — c'est le **BLANCHIMENT** : effacer ses traces en important un
    // backup. L'audit du 2026-08-26 le relève en III.3, et le module d'audit
    // promettait l'inamovibilité sans la tenir.
    //
    // ⚠️ La table **reste dans `TABLES_TO_TRUNCATE`**, et il le faut : la
    // constante sert aussi à produire l'export (la boucle de
    // `export_installation`), à compter les tables du manifeste
    // (`write_keshbackup`, `kesh-api/src/admin_backup/export.rs`) et à REJETER
    // un manifeste portant une table hors inventaire (`parse_and_verify`,
    // `kesh-api/src/admin_backup/import.rs`, contrôle « hors inventaire
    // applicatif »). L'en sortir rendrait **tout backup existant inimportable**.
    for &table in TABLES_TO_TRUNCATE {
        if table == "onboarding_state" || table == "audit_log" {
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
        // ⛔ FUSION de la piste de contrôle (Story 25-1a) — la colonne `id` de
        // `audit_log` est ÉCARTÉE, et l'auto-increment en attribue de nouveaux.
        //
        // Sans cela la fusion échouerait en doublon de clé primaire : les entrées
        // conservées et celles du backup viennent de **deux instances dont les
        // espaces d'identifiants se recouvrent**. Écarter `id` est licite ici et
        // ne l'est pour aucune autre table : sur une piste de contrôle, l'`id`
        // n'est pas une donnée métier et n'est référencé par rien — l'ordre
        // chronologique est porté par `created_at`, et les liens vers les objets
        // audités par `entity_type`/`entity_id`, qui sont des pointeurs logiques.
        let skip_id = table == "audit_log";
        let kept: Vec<usize> = (0..data.column_names.len())
            .filter(|&i| !(skip_id && data.column_names[i].eq_ignore_ascii_case("id")))
            .collect();
        let cols_sql = kept
            .iter()
            .map(|&i| format!("`{}`", data.column_names[i]))
            .collect::<Vec<_>>()
            .join(", ");
        let placeholders = vec!["?"; kept.len()].join(", ");
        let sql = format!("INSERT INTO `{table}` ({cols_sql}) VALUES ({placeholders})");
        for row in &data.rows {
            if row.len() != data.column_names.len() {
                return Err(DbError::Invariant(format!(
                    "restore: table '{table}' ligne à {} valeurs, {} colonnes attendues",
                    row.len(),
                    data.column_names.len()
                )));
            }
            // ⚠️ Les valeurs suivent `kept`, pas `row` : sur `audit_log` la
            // colonne `id` est écartée des DEUX côtés, faute de quoi le nombre de
            // paramètres ne correspondrait plus aux placeholders.
            let mut q = sqlx::query(&sql);
            for &i in &kept {
                q = bind_json_value(q, &row[i])?;
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
    #[sqlx::test(migrations = "./test-schema")]
    async fn backup_inventory_matches_schema(pool: MySqlPool) {
        let db_tables: Vec<String> = sqlx::query_scalar(
            "SELECT TABLE_NAME FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = DATABASE() \
               AND TABLE_NAME NOT IN ('_sqlx_migrations', '_kesh_version', \
                                      '_kesh_backup_schedule') \
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

    /// Story 15-7b2, test 10 — la partition de la remise à zéro : les sept tables
    /// conservées (six, plus `mcp_keys` depuis la 32-5d) sont des tables
    /// canoniques, et conservées ∪ vidées redonne [`TABLES_TO_TRUNCATE`] **dans
    /// son ordre**.
    ///
    /// ⚠️ Vrai en partie **par construction** ([`reset_cleared_tables`] est
    /// dérivée) : seule l'appartenance des sept noms mord (renommage d'une table).
    /// La garde de la partition est le test suivant, qui lit le schéma.
    #[test]
    fn reset_partition_covers_the_canonical_list_in_order() {
        for t in RESET_PRESERVED_TABLES {
            assert!(
                TABLES_TO_TRUNCATE.contains(t),
                "table conservée inconnue de la liste canonique : {t}"
            );
        }
        let cleared = reset_cleared_tables();
        assert_eq!(
            cleared.len() + RESET_PRESERVED_TABLES.len(),
            TABLES_TO_TRUNCATE.len()
        );
        let mut it = cleared.iter();
        let mut next = it.next();
        for t in TABLES_TO_TRUNCATE {
            if Some(t) == next {
                next = it.next();
            } else {
                assert!(
                    RESET_PRESERVED_TABLES.contains(t),
                    "{t} ni vidée ni conservée"
                );
            }
        }
        assert!(
            next.is_none(),
            "ordre des tables vidées différent de la liste canonique"
        );
        assert!(
            cleared.contains(&"audit_log"),
            "la piste est vidée puis réinscrite"
        );
    }

    /// Story 15-7b2, test 10b — **la partition de la remise à zéro est gardée sur
    /// le schéma** (AC 3, choix C-15-7-24, C-15-7-28).
    ///
    /// Relation : une table *rejoint* `companies` s'il existe une chaîne de clés
    /// étrangères qui y mène **sans traverser** une table de
    /// [`RESET_PRESERVED_TABLES`] autre que `companies` — en particulier sans
    /// passer par `users`. Une fermeture transitive ordinaire ne garderait rien :
    /// `refresh_tokens → users → companies` y « atteindrait » `companies`.
    ///
    /// - **Règle 1** : toute table vidée rejoint `companies`, sauf les exceptions
    ///   d'une liste fermée (`audit_log`, sans aucune clé étrangère), dont chacune
    ///   doit effectivement **ne pas** la rejoindre.
    /// - **Règle 2** : toute table canonique portant une clé vers une table
    ///   conservée autre que `companies`, et qui ne rejoint pas `companies`, est
    ///   conservée — aujourd'hui exactement `password_reset_tokens` et
    ///   `refresh_tokens`. Une future table enfant de `users` seulement rougit.
    /// - **Règle 3** : aucune clé d'une table conservée ne référence une table
    ///   vidée (le vidage se fait sous `FOREIGN_KEY_CHECKS=0` : une telle clé
    ///   laisserait une référence pendante).
    /// - **Règle 4** : les tables conservées qui portent une clé vers
    ///   `companies` sont exactement `users`, `api_keys` et `mcp_keys` (cette
    ///   dernière depuis la Story 32-5d) — le fait sur lequel repose la règle
    ///   des principaux orphelins
    ///   (`companies::reattach_orphan_principals_in_tx`), qui traite ces trois
    ///   tables et elles seules.
    ///
    /// Angle mort assumé : une future table enfant de `users` **et** de
    /// `companies` rejoint `companies` directement et serait vidée — voulu pour
    /// des données de société ; le choix se fait à sa création sinon.
    #[sqlx::test(migrations = "./test-schema")]
    async fn reset_partition_is_guarded_by_the_schema(pool: MySqlPool) {
        use std::collections::{BTreeMap, BTreeSet};

        let edges: Vec<(String, String)> = sqlx::query_as(
            "SELECT DISTINCT TABLE_NAME, REFERENCED_TABLE_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = DATABASE() AND REFERENCED_TABLE_NAME IS NOT NULL",
        )
        .fetch_all(&pool)
        .await
        .expect("clés étrangères du schéma");
        assert!(
            edges.len() > 30,
            "lecture des clés étrangères à vide ({} arêtes)",
            edges.len()
        );
        let mut graph: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
        for (from, to) in &edges {
            graph.entry(from.as_str()).or_default().insert(to.as_str());
        }
        let preserved_other = |t: &str| t != "companies" && RESET_PRESERVED_TABLES.contains(&t);
        let joins = |start: &str| -> bool {
            let mut seen: BTreeSet<&str> = BTreeSet::new();
            let mut stack = vec![start];
            while let Some(t) = stack.pop() {
                if !seen.insert(t) {
                    continue;
                }
                for &n in graph.get(t).into_iter().flatten() {
                    if n == "companies" {
                        return true;
                    }
                    if !preserved_other(n) {
                        stack.push(n);
                    }
                }
            }
            false
        };

        // Règle 1.
        const EXCEPTIONS: &[&str] = &["audit_log"];
        let cleared = reset_cleared_tables();
        for t in &cleared {
            if EXCEPTIONS.contains(t) {
                assert!(
                    !joins(t),
                    "exception devenue inutile : {t} rejoint companies"
                );
            } else {
                assert!(
                    joins(t),
                    "règle 1 : la table vidée {t} ne rejoint pas companies sans \
                     traverser une table conservée — à classer"
                );
            }
        }
        for e in EXCEPTIONS {
            assert!(
                cleared.contains(e),
                "exception hors des tables vidées : {e}"
            );
        }

        // Règle 2.
        let rule2: BTreeSet<&str> = TABLES_TO_TRUNCATE
            .iter()
            .copied()
            .filter(|t| *t != "companies")
            .filter(|t| {
                graph
                    .get(t)
                    .into_iter()
                    .flatten()
                    .any(|n| preserved_other(n))
            })
            .filter(|t| !joins(t))
            .collect();
        assert_eq!(
            rule2,
            BTreeSet::from(["password_reset_tokens", "refresh_tokens"]),
            "règle 2 : tables enfants de principaux seulement"
        );
        for t in &rule2 {
            assert!(
                RESET_PRESERVED_TABLES.contains(t),
                "règle 2 : {t} doit être conservée"
            );
        }

        // Règle 3.
        let dangling: Vec<(&str, &str)> = RESET_PRESERVED_TABLES
            .iter()
            .flat_map(|p| {
                graph
                    .get(p)
                    .into_iter()
                    .flatten()
                    .filter(|n| cleared.contains(n))
                    .map(move |n| (*p, *n))
            })
            .collect();
        assert!(
            dangling.is_empty(),
            "règle 3 : une table conservée référence une table vidée : {dangling:?}"
        );

        // Règle 4.
        let to_companies: BTreeSet<&str> = RESET_PRESERVED_TABLES
            .iter()
            .copied()
            .filter(|p| graph.get(p).is_some_and(|ns| ns.contains("companies")))
            .collect();
        assert_eq!(
            to_companies,
            BTreeSet::from(["api_keys", "mcp_keys", "users"]),
            "règle 4 : principaux portant une clé vers companies — toute table \
             conservée de plus doit entrer à la règle des principaux orphelins"
        );
    }

    /// `export_table` exclut les colonnes générées (`active_uniq` VIRTUAL).
    #[sqlx::test(migrations = "./test-schema")]
    async fn export_table_excludes_generated_columns(pool: MySqlPool) {
        let mut snap = ReadSnapshot::begin(&pool).await.expect("snapshot");
        let export = export_table(&mut snap, "reconciliation_rules")
            .await
            .expect("export reconciliation_rules");
        snap.finish().await.expect("finish");
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
    #[sqlx::test(migrations = "./test-schema")]
    async fn export_table_serializes_rows_faithfully(pool: MySqlPool) {
        // Insère une company minimale (ide_number NULL pour tester NULL → null).
        sqlx::query(
            "INSERT INTO companies (name, address, ide_number, org_type, accounting_language, instance_language) \
             VALUES ('Acme', 'Rue 1', NULL, 'Pme', 'FR', 'FR')",
        )
        .execute(&pool)
        .await
        .expect("insert company");

        let mut snap = ReadSnapshot::begin(&pool).await.expect("snapshot");
        let export = export_table(&mut snap, "companies")
            .await
            .expect("export companies");
        snap.finish().await.expect("finish");
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
    #[sqlx::test(migrations = "./test-schema")]
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

    /// Story 26-1 (#620), AC11 : `bank_profiles.debit_sign` est `NOT NULL`
    /// **avec** `DEFAULT 'positive'` → non requise. C'est ce qui rend
    /// importable une sauvegarde 0.13.0 (sans la colonne) : elle est
    /// restaurée à `positive`, la sémantique des profils d'avant.
    #[sqlx::test(migrations = "./test-schema")]
    async fn bank_profiles_debit_sign_is_not_required(pool: MySqlPool) {
        let cols = column_constraints(&pool, "bank_profiles")
            .await
            .expect("constraints bank_profiles");
        let debit_sign = cols
            .iter()
            .find(|c| c.name == "debit_sign")
            .expect("debit_sign column");
        assert!(!debit_sign.is_nullable, "debit_sign est NOT NULL");
        assert!(
            !debit_sign.is_required(),
            "debit_sign porte un DEFAULT : jamais requise"
        );
    }

    /// `reconciliation_rules.active_uniq` (GENERATED VIRTUAL) est marquée
    /// générée → jamais requise (cohérent avec son exclusion de l'export).
    #[sqlx::test(migrations = "./test-schema")]
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
    /// ⛔ **AC 2 de la Story 25-1a — la piste de contrôle SURVIT à un import, et
    /// nomme encore ses auteurs.**
    ///
    /// Le résultat exigé, mot pour mot : *après l'import d'un backup étranger,
    /// chaque entrée d'audit antérieure existe encore et nomme son auteur
    /// d'origine.* Le défaut fermé est le **blanchiment** — effacer ses traces en
    /// important une sauvegarde.
    ///
    /// Trois choses sont vérifiées d'un coup, et la troisième est celle qui
    /// manquait au dépôt : l'entrée locale **survit**, l'entrée du backup est
    /// **fusionnée** malgré un `id` en collision, et le libellé d'acteur reste
    /// lisible **alors que la table `users` a été remplacée**.
    #[sqlx::test(migrations = "./test-schema")]
    async fn restore_merges_audit_log_and_keeps_actor_names(pool: MySqlPool) {
        // Un utilisateur local, et une entrée d'audit qui lui est attribuée.
        // (`users.company_id` est NOT NULL sans défaut depuis 20260419000002.)
        sqlx::query(
            "INSERT INTO companies (id, name, address, org_type, accounting_language, instance_language) \
             VALUES (1, 'Locale SA', 'Rue 1', 'Pme', 'FR', 'FR')",
        )
        .execute(&pool)
        .await
        .expect("insert company locale");
        sqlx::query(
            "INSERT INTO users (id, company_id, username, password_hash, role) \
             VALUES (7, 1, 'alice', 'x0123456789012345678901234567890', 'Admin')",
        )
        .execute(&pool)
        .await
        .expect("insert user local");
        sqlx::query(
            "INSERT INTO audit_log (id, user_id, actor_label, action, entity_type, entity_id) \
             VALUES (100, 7, 'alice', 'books.locked', 'company', 1)",
        )
        .execute(&pool)
        .await
        .expect("insert audit local");

        // Le backup vient d'une AUTRE instance : son entrée porte le MÊME `id`
        // (les espaces d'identifiants se recouvrent) et un acteur qui n'existe
        // pas ici.
        let mut tables = BTreeMap::new();
        tables.insert(
            "audit_log".to_string(),
            TableRestore {
                column_names: vec![
                    "id".into(),
                    "user_id".into(),
                    "actor_label".into(),
                    "action".into(),
                    "entity_type".into(),
                    "entity_id".into(),
                ],
                rows: vec![vec![
                    serde_json::json!(100),
                    serde_json::json!(42),
                    serde_json::json!("bob"),
                    serde_json::json!("invoice.created"),
                    serde_json::json!("invoice"),
                    serde_json::json!(9),
                ]],
            },
        );
        // `users` est remplacée par celle du backup — alice DISPARAÎT.
        tables.insert(
            "users".to_string(),
            TableRestore {
                column_names: vec![
                    "id".into(),
                    "company_id".into(),
                    "username".into(),
                    "password_hash".into(),
                    "role".into(),
                ],
                rows: vec![vec![
                    serde_json::json!(42),
                    serde_json::json!(1),
                    serde_json::json!("bob"),
                    serde_json::json!("x0123456789012345678901234567890"),
                    serde_json::json!("Admin"),
                ]],
            },
        );

        let mut tx = pool.begin().await.expect("begin");
        restore_tables_in_tx(&mut tx, &tables)
            .await
            .expect("restore");
        tx.commit().await.expect("commit");

        let entries: Vec<(String, String)> =
            sqlx::query_as("SELECT action, actor_label FROM audit_log ORDER BY action")
                .fetch_all(&pool)
                .await
                .expect("select audit");

        assert_eq!(
            entries.len(),
            2,
            "la piste doit porter les DEUX entrées — la locale conservée et celle \
             du backup fusionnée ; got {entries:?}"
        );
        assert!(
            entries.contains(&("books.locked".to_string(), "alice".to_string())),
            "l'entrée LOCALE doit survivre à l'import et nommer encore alice — \
             c'est le blanchiment que cette story ferme ; got {entries:?}"
        );
        assert!(
            entries.contains(&("invoice.created".to_string(), "bob".to_string())),
            "l'entrée du BACKUP doit être fusionnée malgré son id en collision ; \
             got {entries:?}"
        );

        // alice n'existe plus dans `users` : sans `actor_label`, son entrée
        // serait devenue anonyme — ou pire, aurait désigné bob (id réattribué).
        let alice: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE username = 'alice'")
            .fetch_one(&pool)
            .await
            .expect("count alice");
        assert_eq!(
            alice, 0,
            "users doit bien avoir été remplacée par le backup"
        );
    }

    /// ⛔ **`actor_label` ne doit JAMAIS être plus étroit que `users.username`.**
    ///
    /// Le libellé est posé par un **sous-SELECT** dans l'`INSERT` du repository
    /// (`repositories/audit_log.rs`) : la valeur vient donc directement de
    /// `users.username`, sans passer par aucune validation Rust. Si la colonne
    /// source devenait plus large que la cible, MariaDB en mode strict —
    /// `STRICT_TRANS_TABLES`, actif en production — refuserait l'`INSERT` avec
    /// « Data too long ». Et comme l'écriture d'audit partage la transaction de
    /// l'opération auditée, **c'est l'opération métier entière qui échouerait**,
    /// sur un motif que rien ne rattacherait au nom de l'utilisateur.
    ///
    /// Le mode d'échec n'est donc pas « un libellé tronqué » mais « une facture
    /// qu'on ne peut plus valider parce qu'un administrateur porte un nom long ».
    /// D'où ce test, qui lit les deux largeurs dans `information_schema` plutôt
    /// que de répéter un nombre : il ne se périme pas, et il rougit en nommant
    /// exactement le couple qui a divergé.
    #[sqlx::test(migrations = "./test-schema")]
    async fn actor_label_is_at_least_as_wide_as_username(pool: MySqlPool) {
        let width = |table: &'static str, column: &'static str| {
            let pool = pool.clone();
            async move {
                sqlx::query_scalar::<_, u64>(
                    "SELECT CHARACTER_MAXIMUM_LENGTH FROM information_schema.COLUMNS \
                     WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? AND COLUMN_NAME = ?",
                )
                .bind(table)
                .bind(column)
                .fetch_one(&pool)
                .await
                .unwrap_or_else(|e| panic!("largeur de {table}.{column} introuvable : {e}"))
            }
        };

        let label = width("audit_log", "actor_label").await;
        let username = width("users", "username").await;
        assert!(
            label >= username,
            "audit_log.actor_label ({label}) est plus ÉTROIT que users.username \
             ({username}) : le sous-SELECT du repository ferait échouer l'INSERT \
             d'audit — donc l'opération métier qu'il accompagne — pour tout nom \
             d'utilisateur dépassant {label} caractères"
        );
    }

    #[sqlx::test(migrations = "./test-schema")]
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
        let mut snap = ReadSnapshot::begin(&pool).await.expect("snapshot");
        let export = export_table(&mut snap, "companies").await.expect("export");
        snap.finish().await.expect("finish");
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

    /// T-a11 (Story 28-4a, #642, AC2) — un compte bancaire **clos** revient
    /// clos, à la même date, d'un aller-retour par l'export générique et la
    /// restauration (rien à coder : `export_table` lit toutes les colonnes).
    #[sqlx::test(migrations = "./test-schema")]
    async fn restore_round_trips_a_closed_bank_account(pool: MySqlPool) {
        let seeded = crate::test_fixtures::seed_accounting_company(&pool)
            .await
            .expect("seed");
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, closed_on) \
             VALUES (?, 'UBS', 'CH4431999123000889012', FALSE, '2026-01-31')",
        )
        .bind(seeded.company_id)
        .execute(&pool)
        .await
        .expect("compte clos");

        let mut snap = ReadSnapshot::begin(&pool).await.expect("snapshot");
        let mut tables = BTreeMap::new();
        for table in ["companies", "bank_accounts"] {
            let export = export_table(&mut snap, table).await.expect("export");
            let rows = parse_ndjson_rows(&export.ndjson, &export.column_names).expect("parse");
            tables.insert(
                table.to_string(),
                TableRestore {
                    column_names: export.column_names.clone(),
                    rows,
                },
            );
        }
        snap.finish().await.expect("finish");
        assert!(
            tables["bank_accounts"]
                .column_names
                .iter()
                .any(|c| c == "closed_on"),
            "l'export porte closed_on"
        );

        let mut tx = pool.begin().await.expect("begin");
        restore_tables_in_tx(&mut tx, &tables)
            .await
            .expect("restore");
        tx.commit().await.expect("commit");

        let closed_on: Option<chrono::NaiveDate> =
            sqlx::query_scalar("SELECT closed_on FROM bank_accounts WHERE iban = ?")
                .bind("CH4431999123000889012")
                .fetch_one(&pool)
                .await
                .expect("compte restauré");
        assert_eq!(closed_on, chrono::NaiveDate::from_ymd_opt(2026, 1, 31));
    }

    // --- Story 28-3 (#633) : la sauvegarde lit ses tables dans un instantané ---

    /// Montage commun de l'AC5 : une société comptable, un compte bancaire, un
    /// import portant deux transactions. Rend `(company_id, import_id)`.
    async fn monter_import(pool: &MySqlPool) -> (i64, i64) {
        use crate::entities::{
            BankImportSourceFormat, NewBankAccount, NewBankImport, NewBankTransaction,
        };
        use crate::repositories::{bank_accounts, bank_imports};

        let seeded = crate::test_fixtures::seed_accounting_company(pool)
            .await
            .expect("seed société");
        let company_id = seeded.company_id;
        let bank_account_id = bank_accounts::create(
            pool,
            NewBankAccount {
                company_id,
                bank_name: "Banque test".into(),
                iban: "CH4431999123000889012".into(),
                qr_iban: None,
                is_primary: true,
            },
        )
        .await
        .expect("compte bancaire")
        .id;
        let jour = chrono::NaiveDate::from_ymd_opt(2026, 5, 15).expect("date");
        let txs: Vec<NewBankTransaction> = (1..=2)
            .map(|i| NewBankTransaction {
                company_id,
                bank_account_id,
                booking_date: jour,
                value_date: Some(jour),
                amount: rust_decimal::Decimal::new(100 * i, 0),
                currency: "CHF".into(),
                reference: Some(format!("REF-{i}")),
                details: format!("Transaction de test {i}"),
                end_to_end_id: None,
                transaction_id: None,
                counterparty_iban: None,
                counterparty_name: None,
            })
            .collect();
        let mut tx = pool.begin().await.expect("begin import");
        let (import, _) = bank_imports::create_with_transactions(
            &mut tx,
            NewBankImport {
                company_id,
                bank_account_id,
                filename: "releve.xml".into(),
                file_hash: "28".repeat(32),
                source_format: BankImportSourceFormat::Camt053V04,
                statement_id: Some("STMT-283".into()),
                period_from: jour,
                period_to: jour,
                opening_balance: None,
                closing_balance: None,
                transaction_count: 2,
                imported_by_user_id: seeded.admin_user_id,
            },
            txs,
        )
        .await
        .expect("import");
        tx.commit().await.expect("commit import");
        (company_id, import.id)
    }

    /// Les lignes NDJSON d'une table de la sauvegarde, décodées.
    fn lignes(export: &InstallationExport, table: &str) -> Vec<Value> {
        let (_, t) = export
            .tables
            .iter()
            .find(|(nom, _)| *nom == table)
            .unwrap_or_else(|| panic!("table {table} absente de la sauvegarde"));
        String::from_utf8(t.ndjson.clone())
            .expect("NDJSON UTF-8")
            .lines()
            .map(|l| serde_json::from_str(l).expect("ligne NDJSON"))
            .collect()
    }

    /// Lignes orphelines de la sauvegarde : pour chaque clé étrangère du schéma
    /// (`information_schema.KEY_COLUMN_USAGE`), toute valeur non nulle de la
    /// colonne fille doit figurer parmi celles de la colonne référencée, **dans
    /// le fichier**. Rend la liste des violations.
    async fn orphelins(pool: &MySqlPool, export: &InstallationExport) -> Vec<String> {
        let fks: Vec<(String, String, String, String)> = sqlx::query_as(
            "SELECT TABLE_NAME, COLUMN_NAME, REFERENCED_TABLE_NAME, REFERENCED_COLUMN_NAME \
             FROM information_schema.KEY_COLUMN_USAGE \
             WHERE TABLE_SCHEMA = DATABASE() AND REFERENCED_TABLE_NAME IS NOT NULL \
             ORDER BY TABLE_NAME, COLUMN_NAME",
        )
        .fetch_all(pool)
        .await
        .expect("clés étrangères");
        // Assertion de montage : sans la clé de l'import, le helper serait muet.
        assert!(
            fks.iter().any(|(t, c, rt, _)| t == "bank_transactions"
                && c == "import_id"
                && rt == "bank_imports"),
            "assertion de montage : orphelins() ne voit pas fk_bank_transactions_import \
             ({} clés lues)",
            fks.len()
        );
        let mut violations = Vec::new();
        for (table, col, ref_table, ref_col) in &fks {
            let parents: std::collections::HashSet<String> = lignes(export, ref_table)
                .iter()
                .map(|l| l[ref_col.as_str()].to_string())
                .collect();
            for ligne in lignes(export, table) {
                let v = &ligne[col.as_str()];
                if !v.is_null() && !parents.contains(&v.to_string()) {
                    violations.push(format!("{table}.{col}={v} → {ref_table}.{ref_col} absent"));
                }
            }
        }
        violations
    }

    /// Observateur des tests (a) et (b) : à l'étape `cible`, supprime l'import
    /// par l'écrivain réel (`delete_unreconciled_in_tx`), dans une transaction
    /// neuve du pool — donc sur une **autre** connexion que l'instantané —,
    /// validée ; compte ses déclenchements.
    fn observateur_qui_supprime(
        pool: MySqlPool,
        cible: Etape,
        company_id: i64,
        import_id: i64,
        compteur: std::sync::Arc<std::sync::atomic::AtomicUsize>,
    ) -> impl FnMut(Etape) -> BoxFuture<'static, ()> + Send {
        move |etape: Etape| -> BoxFuture<'static, ()> {
            if etape != cible {
                return Box::pin(async {});
            }
            let pool = pool.clone();
            let compteur = std::sync::Arc::clone(&compteur);
            Box::pin(async move {
                let mut tx = pool.begin().await.expect("begin suppression");
                let issue = crate::repositories::bank_imports::delete_unreconciled_in_tx(
                    &mut tx, company_id, import_id,
                )
                .await
                .expect("suppression de l'import");
                assert!(
                    matches!(
                        issue,
                        crate::repositories::bank_imports::DeleteBankImportOutcome::Deleted(_)
                    ),
                    "assertion de montage : l'import devait être supprimé"
                );
                tx.commit().await.expect("commit suppression");
                compteur.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            })
        }
    }

    /// Les tests (a) et (b) : sauvegarde par le cœur sur `snap`, suppression
    /// concurrente de l'import à l'étape `cible`, puis les trois attendus de
    /// l'AC5 (a) — montage, import et transactions présents, aucun orphelin.
    async fn sauvegarder_pendant_suppression(
        pool: &MySqlPool,
        mut snap: ReadSnapshot,
        cible: Etape,
        company_id: i64,
        import_id: i64,
    ) {
        let compteur = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut obs = observateur_qui_supprime(
            pool.clone(),
            cible,
            company_id,
            import_id,
            std::sync::Arc::clone(&compteur),
        );
        let export = export_installation_in(&mut snap, &mut obs)
            .await
            .expect("sauvegarde");
        snap.finish().await.expect("clôture de l'instantané");

        // 1. Assertion de montage : le geste concurrent a bien eu lieu.
        assert_eq!(
            compteur.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "assertion de montage : l'observateur n'a pas supprimé à l'étape {cible:?}"
        );
        let restants: i64 = sqlx::query_scalar(
            "SELECT (SELECT COUNT(*) FROM bank_imports WHERE id = ?) \
                  + (SELECT COUNT(*) FROM bank_transactions WHERE import_id = ?)",
        )
        .bind(import_id)
        .bind(import_id)
        .fetch_one(pool)
        .await
        .expect("comptage après suppression");
        assert_eq!(
            restants, 0,
            "assertion de montage : la base porte encore l'import ou ses transactions"
        );

        // 2. La sauvegarde montre la base d'avant la suppression ; 3. sans orphelin.
        let violations = orphelins(pool, &export).await;
        let imports = lignes(&export, "bank_imports")
            .iter()
            .filter(|l| l["id"] == import_id)
            .count();
        let txs = lignes(&export, "bank_transactions")
            .iter()
            .filter(|l| l["import_id"] == import_id)
            .count();
        assert_eq!(
            (imports, txs),
            (1, 2),
            "la sauvegarde doit porter l'import et ses deux transactions (orphelins : {violations:?})"
        );
        assert!(
            violations.is_empty(),
            "lignes orphelines dans la sauvegarde : {violations:?}"
        );
    }

    /// AC5 (a-défaut) — une suppression d'import committée entre la lecture de
    /// `bank_transactions` et celle de `bank_imports` n'apparaît pas dans la
    /// sauvegarde, et n'y laisse aucun orphelin.
    #[sqlx::test(migrations = "./test-schema")]
    async fn sauvegarde_ignore_une_suppression_entre_deux_tables(pool: MySqlPool) {
        let (company_id, import_id) = monter_import(&pool).await;
        let snap = ReadSnapshot::begin(&pool).await.expect("instantané");
        let cible = Etape::ApresTable("bank_transactions");
        sauvegarder_pendant_suppression(&pool, snap, cible, company_id, import_id).await;
    }

    /// AC5 (a-RC) — même scénario sur une connexion dont la **session** est en
    /// `READ COMMITTED` : le `SET TRANSACTION … REPEATABLE READ` de l'instantané
    /// est effectif.
    #[sqlx::test(migrations = "./test-schema")]
    async fn sauvegarde_ignore_une_suppression_en_session_read_committed(pool: MySqlPool) {
        let (company_id, import_id) = monter_import(&pool).await;
        let mut conn = pool.acquire().await.expect("connexion");
        sqlx::Executor::execute(
            &mut *conn,
            "SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED",
        )
        .await
        .expect("session READ COMMITTED");
        let snap = ReadSnapshot::begin_on(conn).await.expect("instantané");
        let cible = Etape::ApresTable("bank_transactions");
        sauvegarder_pendant_suppression(&pool, snap, cible, company_id, import_id).await;
    }

    /// AC5 (b) — l'instantané est pris à l'ouverture : une suppression faite
    /// avant la première lecture n'apparaît pas dans la sauvegarde.
    #[sqlx::test(migrations = "./test-schema")]
    async fn sauvegarde_prend_l_instantane_a_l_ouverture(pool: MySqlPool) {
        let (company_id, import_id) = monter_import(&pool).await;
        let snap = ReadSnapshot::begin(&pool).await.expect("instantané");
        sauvegarder_pendant_suppression(&pool, snap, Etape::Ouvert, company_id, import_id).await;
    }

    /// Numéro d'erreur MariaDB d'une erreur sqlx (`number()`, non le SQLSTATE
    /// que rend `code()`).
    fn numero_mariadb(err: &sqlx::Error) -> Option<u16> {
        err.as_database_error()
            .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
            .map(|d| d.number())
    }

    /// Un pool d'une seule connexion sur la base du test : la connexion
    /// réacquise après l'instantané est nécessairement celle qui l'a porté.
    async fn pool_d_une_connexion(pool: &MySqlPool) -> MySqlPool {
        sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(1)
            .connect_with((*pool.connect_options()).clone())
            .await
            .expect("pool d'une connexion")
    }

    /// `CONNECTION_ID()` (`BIGINT UNSIGNED`, d'où le `CAST`).
    async fn id_de_connexion(conn: &mut sqlx::MySqlConnection) -> i64 {
        sqlx::query_scalar("SELECT CAST(CONNECTION_ID() AS SIGNED)")
            .fetch_one(conn)
            .await
            .expect("CONNECTION_ID()")
    }

    /// `@@in_transaction` de la connexion : 1 dans une transaction ouverte.
    async fn en_transaction(conn: &mut sqlx::MySqlConnection) -> i64 {
        sqlx::query_scalar("SELECT CAST(@@in_transaction AS SIGNED)")
            .fetch_one(conn)
            .await
            .expect("@@in_transaction")
    }

    /// AC5 (d1)/(d2) : instantané ouvert sur la connexion unique d'un pool, une
    /// lecture, puis `finish` (d1) ou abandon (d2) ; la connexion réacquise est
    /// la même, et n'est plus en transaction.
    async fn connexion_rendue_hors_transaction(pool: &MySqlPool, avec_finish: bool) {
        let p1 = pool_d_une_connexion(pool).await;
        let mut conn = p1.acquire().await.expect("connexion");
        let id_avant = id_de_connexion(&mut conn).await;
        let mut snap = ReadSnapshot::begin_on(conn).await.expect("instantané");
        assert_eq!(
            en_transaction(snap.conn()).await,
            1,
            "assertion de montage : l'instantané est une transaction ouverte"
        );
        read_min_required(&mut snap).await.expect("lecture");
        if avec_finish {
            snap.finish().await.expect("finish");
        } else {
            drop(snap);
        }
        let mut conn = p1.acquire().await.expect("réacquisition");
        assert_eq!(
            id_de_connexion(&mut conn).await,
            id_avant,
            "assertion de montage : la connexion réacquise doit être celle de l'instantané"
        );
        assert_eq!(
            en_transaction(&mut conn).await,
            0,
            "la connexion revient au pool en transaction (finish = {avec_finish})"
        );
        drop(conn);
        p1.close().await;
    }

    /// AC5 (c) — une écriture dans l'instantané est refusée par le serveur
    /// (`1792`, `ER_CANT_EXECUTE_IN_READ_ONLY_TRANSACTION`).
    #[sqlx::test(migrations = "./test-schema")]
    async fn instantane_refuse_toute_ecriture(pool: MySqlPool) {
        crate::test_fixtures::seed_accounting_company(&pool)
            .await
            .expect("seed société");
        let mut snap = ReadSnapshot::begin(&pool).await.expect("instantané");
        let err = sqlx::query("UPDATE companies SET name = name")
            .execute(snap.conn())
            .await
            .expect_err("une écriture dans l'instantané doit être refusée");
        assert_eq!(
            numero_mariadb(&err),
            Some(1792),
            "erreur inattendue : {err}"
        );
        snap.finish().await.expect("finish");
    }

    /// AC5 (d1) — après `finish`, la connexion revient au pool hors transaction.
    #[sqlx::test(migrations = "./test-schema")]
    async fn instantane_clos_rend_une_connexion_hors_transaction(pool: MySqlPool) {
        connexion_rendue_hors_transaction(&pool, true).await;
    }

    /// AC5 (d2) — abandonné sans `finish` (chemin d'erreur), l'instantané est
    /// annulé et la connexion revient hors transaction.
    #[sqlx::test(migrations = "./test-schema")]
    async fn instantane_abandonne_rend_une_connexion_hors_transaction(pool: MySqlPool) {
        connexion_rendue_hors_transaction(&pool, false).await;
    }

    /// AC5 (e) — toutes les tables sauvegardées sont InnoDB sur la base de test
    /// (garde de test, non d'exécution : C-28-3-25), et `export_installation`
    /// les rend toutes, dans l'ordre de [`TABLES_TO_TRUNCATE`].
    #[sqlx::test(migrations = "./test-schema")]
    async fn tables_sauvegardees_innodb_et_toutes_exportees(pool: MySqlPool) {
        let moteurs: std::collections::HashMap<String, Option<String>> = sqlx::query_as(
            "SELECT TABLE_NAME, ENGINE FROM information_schema.TABLES \
             WHERE TABLE_SCHEMA = DATABASE()",
        )
        .fetch_all(&pool)
        .await
        .expect("information_schema.TABLES")
        .into_iter()
        .collect();
        let attendues: Vec<&str> = TABLES_TO_TRUNCATE
            .iter()
            .copied()
            .chain(["_kesh_version"])
            .collect();
        let trouvees = attendues
            .iter()
            .filter(|t| moteurs.contains_key(**t))
            .count();
        assert_eq!(
            trouvees,
            TABLES_TO_TRUNCATE.len() + 1,
            "assertion de montage : tables attendues trouvées dans le schéma"
        );
        for table in &attendues {
            let moteur = moteurs.get(*table).cloned().flatten();
            assert_eq!(
                moteur.as_deref(),
                Some("InnoDB"),
                "table {table} : moteur {moteur:?} — l'instantané ne la couvrirait pas"
            );
        }

        // Revue de code P1 (E3) : une société, pour que `instance_id` ait une
        // valeur à lire (sur une base vide, le repli `0` la masquerait).
        crate::test_fixtures::seed_accounting_company(&pool)
            .await
            .expect("seed société");
        let min_required_attendu: String =
            sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
                .fetch_one(&pool)
                .await
                .expect("_kesh_version");
        let instance_id_attendu: i64 = sqlx::query_scalar("SELECT MIN(id) FROM companies")
            .fetch_one(&pool)
            .await
            .expect("MIN(companies.id)");
        assert!(
            instance_id_attendu >= 1,
            "assertion de montage : une société existe"
        );

        let export = export_installation(&pool)
            .await
            .expect("export_installation");
        let noms: Vec<&str> = export.tables.iter().map(|(nom, _)| *nom).collect();
        assert_eq!(noms, TABLES_TO_TRUNCATE);
        assert_eq!(export.min_required, min_required_attendu, "min_required");
        assert_eq!(export.instance_id, instance_id_attendu, "instance_id");
    }

    /// AC5 (f) — le niveau d'isolation de la **session** n'est pas modifié par
    /// l'instantané : le `SET TRANSACTION` ne vaut que pour sa transaction.
    #[sqlx::test(migrations = "./test-schema")]
    async fn instantane_ne_change_pas_l_isolation_de_la_session(pool: MySqlPool) {
        let p1 = pool_d_une_connexion(&pool).await;
        let mut conn = p1.acquire().await.expect("connexion");
        sqlx::Executor::execute(
            &mut *conn,
            "SET SESSION TRANSACTION ISOLATION LEVEL READ COMMITTED",
        )
        .await
        .expect("session READ COMMITTED");
        let id_avant = id_de_connexion(&mut conn).await;
        let mut snap = ReadSnapshot::begin_on(conn).await.expect("instantané");
        read_min_required(&mut snap).await.expect("lecture");
        snap.finish().await.expect("finish");

        let mut conn = p1.acquire().await.expect("réacquisition");
        assert_eq!(
            id_de_connexion(&mut conn).await,
            id_avant,
            "assertion de montage : même connexion"
        );
        let niveau: String = sqlx::query_scalar("SELECT @@tx_isolation")
            .fetch_one(&mut *conn)
            .await
            .expect("@@tx_isolation");
        assert_eq!(niveau, "READ-COMMITTED", "niveau de session modifié");
        drop(conn);
        p1.close().await;
    }

    /// AC5 (h) — une erreur du `ROLLBACK` final est rendue par `finish` : la
    /// connexion de l'instantané est tuée (`KILL`) depuis une autre connexion.
    #[sqlx::test(migrations = "./test-schema")]
    async fn finish_rend_l_erreur_du_rollback(pool: MySqlPool) {
        let mut snap = ReadSnapshot::begin(&pool).await.expect("instantané");
        read_min_required(&mut snap).await.expect("lecture");
        let id = id_de_connexion(snap.conn()).await;
        sqlx::Executor::execute(&pool, format!("KILL {id}").as_str())
            .await
            .expect("assertion de montage : KILL de la connexion de l'instantané");
        assert!(
            snap.finish().await.is_err(),
            "finish doit rendre l'erreur du ROLLBACK sur une connexion tuée"
        );
    }

    /// Corps de production d'une fonction de `source` : code avant la première
    /// ligne `#[cfg(test)]`, lignes `//` (donc `///`) écartées, de la ligne qui
    /// commence par `entete` jusqu'à la première ligne réduite à `}`.
    fn corps_de_production(source: &str, entete: &str) -> String {
        let prod: Vec<&str> = source
            .lines()
            .take_while(|l| l.trim() != "#[cfg(test)]")
            .filter(|l| !l.trim_start().starts_with("//"))
            .collect();
        let debut = prod
            .iter()
            .position(|l| l.starts_with(entete))
            .unwrap_or_else(|| panic!("assertion de montage : `{entete}` introuvable"));
        let fin = debut
            + prod[debut..]
                .iter()
                .position(|l| *l == "}")
                .expect("assertion de montage : fin de fonction introuvable");
        assert!(fin > debut + 1, "assertion de montage : corps vide");
        prod[debut..=fin].join("\n")
    }

    /// AC5 (g)(i) — garde lexicale : `export_installation` ne nomme le pool que
    /// deux fois — son paramètre et l'unique `ReadSnapshot::begin(pool)` — et
    /// n'ouvre donc qu'un instantané (M11).
    #[test]
    fn export_installation_n_ouvre_qu_un_instantane() {
        let corps = corps_de_production(
            include_str!("backup.rs"),
            "pub async fn export_installation(",
        );
        let n = regex::Regex::new(r"\bpool\b")
            .expect("regex")
            .find_iter(&corps)
            .count();
        assert_eq!(
            n, 2,
            "`pool` doit apparaître exactement deux fois (paramètre, ReadSnapshot::begin) :\n{corps}"
        );
        // Revue de code P1 (E1, L1, A-5) : l'erreur de `finish` est propagée
        // par un `?` — `let _ = snap.finish().await;` compilerait et
        // l'avalerait, et aucun test d'exécution n'atteint ce point.
        assert!(
            corps.contains("snap.finish().await?"),
            "`export_installation` doit propager l'erreur de `finish` (`snap.finish().await?`) :\n{corps}"
        );
    }

    /// Revue de code P1 (A-4, E2) — échec du `SET TRANSACTION` : la connexion
    /// est fermée (`close_on_drop`), non rendue au pool. Le `SET` est provoqué
    /// par une transaction brute déjà ouverte sur la connexion (`1568`,
    /// `ER_CANT_CHANGE_TX_CHARACTERISTICS`), que sqlx ignore : rendue au pool,
    /// la connexion y reviendrait **transaction ouverte**.
    #[sqlx::test(migrations = "./test-schema")]
    async fn echec_du_set_ferme_la_connexion(pool: MySqlPool) {
        let p1 = pool_d_une_connexion(&pool).await;
        let mut conn = p1.acquire().await.expect("connexion");
        let id_avant = id_de_connexion(&mut conn).await;
        sqlx::Executor::execute(&mut *conn, "BEGIN")
            .await
            .expect("transaction brute");
        assert_eq!(
            en_transaction(&mut conn).await,
            1,
            "assertion de montage : la transaction brute est ouverte"
        );
        let err = match ReadSnapshot::begin_on(conn).await {
            Ok(_) => panic!("le SET TRANSACTION doit échouer dans une transaction ouverte"),
            Err(e) => e,
        };
        let numero = match &err {
            DbError::Sqlx(e) => numero_mariadb(e),
            _ => None,
        };
        assert_eq!(numero, Some(1568), "erreur inattendue : {err}");

        let mut conn = p1.acquire().await.expect("réacquisition");
        assert_ne!(
            id_de_connexion(&mut conn).await,
            id_avant,
            "la connexion du SET en échec est revenue au pool au lieu d'être fermée"
        );
        assert_eq!(
            en_transaction(&mut conn).await,
            0,
            "la connexion réacquise est en transaction"
        );
        drop(conn);
        p1.close().await;
    }

    // =======================================================================
    // Story 31-1a (#503) — fichiers référencés par la base (AC1, AC2)
    // =======================================================================

    /// Empreinte fictive de 64 caractères hexadécimaux, répétée de `c`.
    fn empreinte(c: char) -> String {
        std::iter::repeat_n(c, 64).collect()
    }

    /// Insère un document fournisseur sans QR (seules les colonnes du fichier).
    async fn inserer_document(pool: &MySqlPool, company_id: i64, chemin: &str, sha: &str) {
        sqlx::query(
            "INSERT INTO imported_supplier_invoices \
             (company_id, file_hash, storage_path, original_filename, mime_type, byte_size) \
             VALUES (?, ?, ?, 'doc.pdf', 'application/pdf', 10)",
        )
        .bind(company_id)
        .bind(sha)
        .bind(chemin)
        .execute(pool)
        .await
        .expect("document fournisseur");
    }

    /// Fige le PDF d'une facture : les quatre colonnes de la migration
    /// `20261003000001` (contrainte « tout ou rien »).
    async fn figer(pool: &MySqlPool, invoice_id: i64, chemin: &str, sha: &str) {
        sqlx::query(
            "UPDATE invoices SET pdf_storage_path = ?, pdf_sha256 = ?, \
             pdf_frozen_at = NOW(3), pdf_language = 'FR' WHERE id = ?",
        )
        .bind(chemin)
        .bind(sha)
        .bind(invoice_id)
        .execute(pool)
        .await
        .expect("facture figée");
    }

    /// Société comptable + un client + deux factures en brouillon (non figées) ;
    /// rend `(company_id, [facture_1, facture_2])`.
    async fn monter_factures(pool: &MySqlPool) -> (i64, [i64; 2]) {
        use crate::entities::contact::{ContactType, NewContact, Salutation};
        use crate::entities::invoice::{NewInvoice, NewInvoiceLine};
        use crate::repositories::{contacts, invoices};

        let seeded = crate::test_fixtures::seed_accounting_company(pool)
            .await
            .expect("seed société");
        let contact = contacts::create(
            pool,
            seeded.admin_user_id,
            NewContact {
                company_id: seeded.company_id,
                contact_type: ContactType::Personne,
                name: "Client Fictif".into(),
                first_name: Some("Client".into()),
                last_name: Some("Fictif".into()),
                is_client: true,
                is_supplier: false,
                address: None,
                address_street: None,
                address_building: None,
                address_postal_code: None,
                address_city: None,
                address_country: None,
                email: None,
                phone: None,
                ide_number: None,
                client_number: None,
                default_payment_terms: None,
                default_payment_terms_days: None,
                language: None,
                salutation: Salutation::Neutre,
            },
        )
        .await
        .expect("client");
        let mut ids = [0i64; 2];
        for id in &mut ids {
            let (invoice, _) = invoices::create(
                pool,
                seeded.admin_user_id,
                NewInvoice {
                    company_id: seeded.company_id,
                    contact_id: contact.id,
                    date: chrono::NaiveDate::from_ymd_opt(2026, 4, 14).expect("date"),
                    due_date: None,
                    payment_terms: None,
                    lines: vec![NewInvoiceLine {
                        revenue_account_id: None,
                        description: "Prestation".into(),
                        quantity: rust_decimal::Decimal::ONE,
                        unit_price: rust_decimal::Decimal::new(100, 0),
                        vat_rate: rust_decimal::Decimal::ZERO,
                    }],
                    project_id: None,
                },
            )
            .await
            .expect("facture");
            *id = invoice.id;
        }
        (seeded.company_id, ids)
    }

    /// Une seconde société (deux documents de même empreinte ne peuvent
    /// coexister que dans deux sociétés : `UNIQUE (company_id, file_hash)`).
    async fn seconde_societe(pool: &MySqlPool) -> i64 {
        use crate::entities::address::StructuredAddress;
        use crate::entities::{Language, NewCompany, OrgType};
        crate::repositories::companies::create(
            pool,
            NewCompany {
                name: "Seconde société".into(),
                first_name: None,
                last_name: None,
                address_structured: StructuredAddress {
                    street: "Rue Test".into(),
                    building: "2".into(),
                    postal_code: "1000".into(),
                    city: "Lausanne".into(),
                    country: "CH".into(),
                },
                ide_number: None,
                org_type: OrgType::Independant,
                accounting_language: Language::Fr,
                instance_language: Language::Fr,
            },
        )
        .await
        .expect("seconde société")
        .id
    }

    /// AC1 (T-a2) — les deux familles de pièces sont lues ; une facture non
    /// figée n'apparaît pas ; deux chemins qui ne diffèrent que par la casse
    /// (même empreinte, deux sociétés) restent **deux** fichiers — un `UNION`
    /// SQL, sous la collation `utf8mb4_unicode_ci`, les fusionnerait (M-a22) ;
    /// le résultat est trié.
    #[sqlx::test(migrations = "./test-schema")]
    async fn referenced_files_lit_documents_et_pdf_figes(pool: MySqlPool) {
        let (company_id, [figee, brouillon]) = monter_factures(&pool).await;
        let autre = seconde_societe(&pool).await;
        let h = empreinte('a');
        let minuscule = format!("{h}.pdf");
        let majuscule = format!("{}.pdf", h.to_uppercase());
        inserer_document(&pool, company_id, &minuscule, &h).await;
        inserer_document(&pool, autre, &majuscule, &h).await;
        let p = empreinte('b');
        figer(&pool, figee, &format!("{p}.pdf"), &p).await;
        let _ = brouillon;

        let export = export_installation(&pool).await.expect("export");
        let attendu = vec![
            ReferencedFile {
                storage_path: majuscule.clone(),
                sha256: h.clone(),
            },
            ReferencedFile {
                storage_path: minuscule.clone(),
                sha256: h.clone(),
            },
            ReferencedFile {
                storage_path: format!("{p}.pdf"),
                sha256: p.clone(),
            },
        ];
        assert_eq!(export.referenced_files, attendu);
    }

    /// AC1 (T-a2, M-a11) — les fichiers référencés se lisent dans l'instantané :
    /// une facture figée **après** l'ouverture (écriture committée sur une autre
    /// connexion, par l'observateur) n'apparaît pas ; un document inséré avant,
    /// si.
    #[sqlx::test(migrations = "./test-schema")]
    async fn referenced_files_lus_dans_l_instantane(pool: MySqlPool) {
        let (company_id, [figee, _]) = monter_factures(&pool).await;
        let h = empreinte('c');
        inserer_document(&pool, company_id, &format!("{h}.png"), &h).await;
        let p = empreinte('d');

        let compteur = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut obs = {
            let pool = pool.clone();
            let compteur = std::sync::Arc::clone(&compteur);
            let p = p.clone();
            move |etape: Etape| -> BoxFuture<'static, ()> {
                if etape != Etape::Ouvert {
                    return Box::pin(async {});
                }
                let pool = pool.clone();
                let compteur = std::sync::Arc::clone(&compteur);
                let p = p.clone();
                Box::pin(async move {
                    figer(&pool, figee, &format!("{p}.pdf"), &p).await;
                    compteur.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                })
            }
        };
        let mut snap = ReadSnapshot::begin(&pool).await.expect("instantané");
        let export = export_installation_in(&mut snap, &mut obs)
            .await
            .expect("sauvegarde");
        snap.finish().await.expect("clôture");

        assert_eq!(
            compteur.load(std::sync::atomic::Ordering::SeqCst),
            1,
            "assertion de montage : la facture n'a pas été figée pendant la sauvegarde"
        );
        let figees: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM invoices WHERE pdf_storage_path IS NOT NULL")
                .fetch_one(&pool)
                .await
                .expect("comptage");
        assert_eq!(
            figees, 1,
            "assertion de montage : la base porte la facture figée"
        );
        assert_eq!(
            export.referenced_files,
            vec![ReferencedFile {
                storage_path: format!("{h}.png"),
                sha256: h,
            }],
            "seul le document antérieur à l'instantané est référencé"
        );
    }

    /// AC2 — inventaire clos des colonnes qui désignent un fichier : toute
    /// colonne dont le nom contient `storage_path` est l'une des deux que lit
    /// [`read_referenced_files`]. Une colonne neuve fait rougir ce test :
    /// mettre à jour l'AC1 de la Story 31-1 (la lecture des fichiers
    /// référencés), puis cette liste.
    #[sqlx::test(migrations = "./test-schema")]
    async fn colonnes_storage_path_inventaire_clos(pool: MySqlPool) {
        let colonnes: Vec<(String, String)> = sqlx::query_as(
            "SELECT TABLE_NAME, COLUMN_NAME FROM information_schema.COLUMNS \
             WHERE TABLE_SCHEMA = DATABASE() AND COLUMN_NAME LIKE '%storage_path%' \
             ORDER BY TABLE_NAME, COLUMN_NAME",
        )
        .fetch_all(&pool)
        .await
        .expect("information_schema");
        assert_eq!(
            colonnes,
            vec![
                (
                    "imported_supplier_invoices".to_string(),
                    "storage_path".to_string()
                ),
                ("invoices".to_string(), "pdf_storage_path".to_string()),
            ],
            "colonne de fichier neuve : mettre à jour la lecture des fichiers référencés \
             (Story 31-1a, AC1 — read_referenced_files) puis cette liste"
        );
    }
}
