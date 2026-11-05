#![cfg(test)]
//! Story v0174-mise-a-jour (#847) — tests internes de la sauvegarde avant mise
//! à jour (T1 à T7-bis, T9 à T15 de la fiche ; T8, le binaire, vit dans
//! `tests/sauvegarde_avant_migration.rs`, T16 dans `kesh-db`).
//!
//! `#![cfg(test)]` en tête : sans lui, la garde `configuration_transmise` lirait
//! ce fichier comme du code de production, et la garde lexicale
//! `admin_backup::export::tests::chaque_lecture_n_a_qu_un_appelant` y compterait
//! les appels de `lire_installation(`.
//!
//! **Montage des bases partielles** : `#[sqlx::test(migrations = false)]`, puis
//! les migrations **jusqu'à** `20261027000001_mcp_prompts` exclue (résolue par
//! version, garde-fou P6 — [`montage::migrations_before`]), puis
//! `kesh_version_last_applied = '0.17.3'` (la base porte sinon `'0.1.0'`), puis
//! les données en SQL brut. Assertion de montage : le plan rend *à appliquer*
//! avec le nombre exact de migrations restantes. Ce fichier est inscrit à
//! `ALLOWED_REAL_MIGRATOR_FILES` (`crates/kesh-db/tests/test_schema_guard.rs`).

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{NaiveDate, NaiveDateTime, TimeDelta};
use sha2::{Digest, Sha256};
use sqlx::{MySqlPool, Row};

use kesh_db::backup::{TABLES_TO_TRUNCATE, restore_tables_in_tx_avec_inventaire};
use kesh_db::version::DowngradeCheckOutcome;

use super::*;
use crate::admin_backup::import::{
    ReadLimits, check_schema_compat, parse_and_verify, parse_and_verify_avec_inventaire,
};
use crate::auto_backup::WriteSeek;
use crate::config::test_helpers::make_test_config;
use crate::errors::AppError;

#[path = "../../../kesh-db/tests/common/mod.rs"]
mod montage;

/// La version du binaire sous test.
const BINAIRE: &str = env!("CARGO_PKG_VERSION");

/// La migration en attente qui crée des tables applicatives (`mcp_prompts`,
/// `mcp_prompt_versions`) : la base partielle s'arrête juste avant.
const PREMIERE_EN_ATTENTE: i64 = 20261027000001;

/// Les tables que crée la migration en attente : absentes de la base partielle.
const TABLES_EN_ATTENTE: &[&str] = &["mcp_prompt_versions", "mcp_prompts"];

/// L'inventaire de la version de départ, **littéral** (C-v0174m-17 (c)) :
/// oracle indépendant du prédicat que lit le module.
fn inventaire_de_depart() -> Vec<&'static str> {
    TABLES_TO_TRUNCATE
        .iter()
        .copied()
        .filter(|t| !TABLES_EN_ATTENTE.contains(t))
        .collect()
}

fn instant() -> NaiveDateTime {
    NaiveDate::from_ymd_opt(2026, 11, 5)
        .unwrap()
        .and_hms_milli_opt(10, 20, 30, 123)
        .unwrap()
}

// ---------------------------------------------------------------------------
// Environnement injecté
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Ecrivain {
    Normal,
    Plein,
    Panique,
}

struct EnvDeTest {
    libre: u64,
    ecrivain: Ecrivain,
    sync_refuse: bool,
}

impl EnvDeTest {
    fn normal() -> Arc<dyn BackupEnv> {
        Arc::new(Self {
            libre: u64::MAX,
            ecrivain: Ecrivain::Normal,
            sync_refuse: false,
        })
    }
}

/// Un écrivain dont chaque écriture rend `StorageFull`.
struct DisquePlein(File);

impl Write for DisquePlein {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::from(io::ErrorKind::StorageFull))
    }
    fn flush(&mut self) -> io::Result<()> {
        self.0.flush()
    }
}

impl Seek for DisquePlein {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

impl BackupEnv for EnvDeTest {
    fn free_bytes(&self, _dir: &Path) -> io::Result<u64> {
        Ok(self.libre)
    }
    fn wrap_writer(&self, file: File) -> Box<dyn WriteSeek> {
        match self.ecrivain {
            Ecrivain::Normal => Box::new(file),
            Ecrivain::Plein => Box::new(DisquePlein(file)),
            Ecrivain::Panique => panic!("T15 : panique de l'écrivain (voulue)"),
        }
    }
    fn sync_dir(&self, dir: &Path) -> io::Result<()> {
        if self.sync_refuse {
            Err(io::Error::other("fsync de dossier refusé (T7-bis)"))
        } else {
            File::open(dir)?.sync_all()
        }
    }
}

// ---------------------------------------------------------------------------
// Montages
// ---------------------------------------------------------------------------

/// Dossiers de sauvegarde et de documents propres au test.
struct Dossiers {
    backup: tempfile::TempDir,
    documents: tempfile::TempDir,
}

impl Dossiers {
    fn neufs() -> Self {
        Self {
            backup: tempfile::tempdir().expect("dossier de sauvegarde"),
            documents: tempfile::tempdir().expect("dossier des documents"),
        }
    }

    fn pre_upgrade(&self) -> PathBuf {
        self.backup.path().join(PRE_UPGRADE_SUBDIR)
    }

    fn config(&self, skip: Option<&str>) -> Config {
        let mut config = make_test_config("admin", "v0174m-admin-password");
        config.admin_backup_dir = self.backup.path().to_string_lossy().to_string();
        config.documents_dir = self.documents.path().to_string_lossy().to_string();
        config.skip_pre_upgrade_backup = skip.map(str::to_owned);
        config
    }

    /// Les fichiers (non `.partial`) de `pre-upgrade/` ; vide si le dossier
    /// n'existe pas.
    fn sauvegardes(&self) -> Vec<String> {
        let Ok(entrees) = std::fs::read_dir(self.pre_upgrade()) else {
            return Vec::new();
        };
        let mut noms: Vec<String> = entrees
            .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
            .filter(|n| !n.ends_with(".partial"))
            .collect();
        noms.sort();
        noms
    }
}

/// Monte une base **partielle** : migrations jusqu'à `PREMIERE_EN_ATTENTE`
/// exclue, `last_applied = '0.17.3'`. Rend le nombre de migrations restantes
/// et vérifie que le plan le rend (assertion de montage).
async fn monter_base_partielle(pool: &MySqlPool) -> usize {
    let appliquees = montage::migrations_before(PREMIERE_EN_ATTENTE, "mcp_prompts");
    montage::apply_migrations_up_to(pool, appliquees)
        .await
        .expect("migrations jusqu'à mcp_prompts exclue");
    sqlx::query("UPDATE _kesh_version SET kesh_version_last_applied = '0.17.3' WHERE id = 1")
        .execute(pool)
        .await
        .expect("version de départ");
    let restantes = kesh_db::MIGRATOR.migrations.len() - appliquees;
    assert!(
        restantes >= 1,
        "assertion de montage : migrations restantes"
    );
    assert_eq!(
        plan_de_migration(pool).await.expect("plan"),
        Plan::AAppliquer { n: restantes },
        "assertion de montage : la base partielle a {restantes} migration(s) à appliquer"
    );
    restantes
}

/// L'issue de l'étape, contrôle de version compris (comme `main.rs`).
async fn etape(
    pool: &MySqlPool,
    config: &Config,
    environnement: Arc<dyn BackupEnv>,
    now: NaiveDateTime,
) -> Result<Issue, Refus> {
    let outcome = kesh_db::version::check_downgrade_protection(pool, BINAIRE)
        .await
        .expect("contrôle de version");
    sauvegarder_avant_migration(pool, config, &outcome, BINAIRE, environnement, now).await
}

fn sha256_hex(octets: &[u8]) -> String {
    format!("{:x}", Sha256::digest(octets))
}

/// Peuple la base partielle : société, utilisateurs, exercice et comptes
/// (`seed_accounting_company`, SQL brut), une écriture à deux lignes, un
/// contact, une facture au PDF figé, une facture fournisseur importée — les
/// deux pièces écrites dans `documents`. Rend les deux noms de pièce.
async fn peupler(pool: &MySqlPool, documents: &Path) -> Vec<String> {
    let seed = kesh_db::test_fixtures::seed_accounting_company(pool)
        .await
        .expect("société de test");
    let entry = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, 1, '2026-01-15', 'OD', 'Écriture de test v0174m')",
    )
    .bind(seed.company_id)
    .bind(seed.fiscal_year_id)
    .execute(pool)
    .await
    .expect("écriture")
    .last_insert_id();
    for (ordre, compte, debit, credit) in [
        (1, seed.accounts["1000"], "100.00", "0"),
        (2, seed.accounts["2000"], "0", "100.00"),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry)
        .bind(compte)
        .bind(ordre)
        .bind(debit)
        .bind(credit)
        .execute(pool)
        .await
        .expect("ligne d'écriture");
    }
    let contact = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) \
         VALUES (?, 'Entreprise', 'Client de test v0174m SA', TRUE)",
    )
    .bind(seed.company_id)
    .execute(pool)
    .await
    .expect("contact")
    .last_insert_id();

    let pdf = b"%PDF-1.4 facture figee de test v0174m".to_vec();
    let pdf_sha = sha256_hex(&pdf);
    let pdf_nom = format!("{pdf_sha}.pdf");
    std::fs::write(documents.join(&pdf_nom), &pdf).expect("PDF figé");
    sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, date, pdf_storage_path, pdf_sha256, pdf_frozen_at, pdf_language) \
         VALUES (?, ?, '2026-01-20', ?, ?, '2026-01-20 10:00:00.000', 'FR')",
    )
    .bind(seed.company_id)
    .bind(contact)
    .bind(&pdf_nom)
    .bind(&pdf_sha)
    .execute(pool)
    .await
    .expect("facture au PDF figé");

    let scan = b"%PDF-1.4 facture fournisseur de test v0174m".to_vec();
    let scan_sha = sha256_hex(&scan);
    let scan_nom = format!("{scan_sha}.pdf");
    std::fs::write(documents.join(&scan_nom), &scan).expect("document fournisseur");
    sqlx::query(
        "INSERT INTO imported_supplier_invoices (company_id, file_hash, storage_path, original_filename, mime_type, byte_size) \
         VALUES (?, ?, ?, 'facture-fournisseur.pdf', 'application/pdf', ?)",
    )
    .bind(seed.company_id)
    .bind(&scan_sha)
    .bind(&scan_nom)
    .bind(scan.len() as i64)
    .execute(pool)
    .await
    .expect("facture fournisseur importée");

    let mut noms = vec![pdf_nom, scan_nom];
    noms.sort();
    noms
}

/// Le manifeste d'un `.keshbackup`.
fn manifeste(chemin: &Path) -> serde_json::Value {
    let mut archive = zip::ZipArchive::new(File::open(chemin).expect("sauvegarde")).expect("zip");
    let mut texte = String::new();
    archive
        .by_name("manifest.json")
        .expect("manifest.json")
        .read_to_string(&mut texte)
        .expect("lecture du manifeste");
    serde_json::from_str(&texte).expect("manifeste JSON")
}

/// Une seconde base éphémère (`_sqlx_test_…`, espace de noms autorisé au
/// compte de test), sur le même serveur que `pool`. Rend son pool et son nom.
async fn seconde_base(pool: &MySqlPool) -> (MySqlPool, String) {
    let nom = format!("_sqlx_test_v0174m_{}", uuid::Uuid::new_v4().simple());
    sqlx::raw_sql(&format!("CREATE DATABASE `{nom}`"))
        .execute(pool)
        .await
        .expect("création de la seconde base");
    let options = (*pool.connect_options()).clone().database(&nom);
    let second = sqlx::mysql::MySqlPoolOptions::new()
        .max_connections(2)
        .connect_with(options)
        .await
        .expect("connexion à la seconde base");
    (second, nom)
}

/// Les lignes d'une table, chaque valeur en hexadécimal (`NULL` marqué),
/// triées : comparaison brute, indépendante du type des colonnes. `audit_log`
/// sans `id` (la fusion en attribue de nouveaux).
async fn lignes_brutes(pool: &MySqlPool, table: &str) -> Vec<String> {
    let colonnes: Vec<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .expect("colonnes");
    let expressions: Vec<String> = colonnes
        .iter()
        .filter(|c| !(table == "audit_log" && c.as_str() == "id"))
        .map(|c| format!("IFNULL(HEX(CAST(`{c}` AS BINARY)), '-')"))
        .collect();
    assert!(!expressions.is_empty(), "table {table} sans colonnes");
    let sql = format!(
        "SELECT CONCAT_WS('|', {}) FROM `{table}`",
        expressions.join(", ")
    );
    let mut lignes: Vec<String> = sqlx::query(&sql)
        .fetch_all(pool)
        .await
        .unwrap_or_else(|e| panic!("lecture de {table} : {e}"))
        .iter()
        .map(|r| r.get::<String, _>(0))
        .collect();
    lignes.sort();
    lignes
}

// ---------------------------------------------------------------------------
// T1 — la sauvegarde est écrite avant la migration
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = false)]
async fn t1_sauvegarde_ecrite_puis_migration(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let pieces = peupler(&pool, dossiers.documents.path()).await;
    let config = dossiers.config(None);

    let issue = etape(&pool, &config, EnvDeTest::normal(), instant())
        .await
        .expect("sauvegarde écrite");
    let attendu_nom =
        format!("kesh-pre-upgrade-20261105T102030123Z-0.17.3-to-{BINAIRE}.keshbackup");
    let chemin = dossiers.pre_upgrade().join(&attendu_nom);
    assert_eq!(
        issue,
        Issue::Ecrite {
            chemin: chemin.clone(),
            echappatoire_ignoree: None,
            dossier_non_synchronise: None,
        }
    );
    assert_eq!(dossiers.sauvegardes(), vec![attendu_nom]);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode_dossier = std::fs::metadata(dossiers.pre_upgrade())
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode_dossier & 0o777, 0o700, "dossier pre-upgrade/ en 0700");
        let mode_fichier = std::fs::metadata(&chemin).unwrap().permissions().mode();
        assert_eq!(mode_fichier & 0o777, 0o600, "fichier en 0600");
    }

    let m = manifeste(&chemin);
    let mut tables: Vec<String> = m["tables"]
        .as_object()
        .expect("tables du manifeste")
        .keys()
        .cloned()
        .collect();
    tables.sort();
    let mut attendues: Vec<String> = inventaire_de_depart()
        .iter()
        .map(|t| t.to_string())
        .collect();
    attendues.sort();
    assert_eq!(
        tables, attendues,
        "tables du manifeste = inventaire de départ"
    );
    let mut fichiers: Vec<String> = m["files"]
        .as_array()
        .expect("files")
        .iter()
        .map(|f| f["path"].as_str().unwrap().to_string())
        .collect();
    fichiers.sort();
    assert_eq!(fichiers, pieces, "les deux pièces sont emportées");

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("la migration suit la sauvegarde");
}

// ---------------------------------------------------------------------------
// T1-ter — importable par la version de départ (aller-retour)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = false)]
async fn t1_ter_importable_par_la_version_de_depart(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    peupler(&pool, dossiers.documents.path()).await;
    let config = dossiers.config(None);
    let Issue::Ecrite { chemin, .. } = etape(&pool, &config, EnvDeTest::normal(), instant())
        .await
        .expect("sauvegarde écrite")
    else {
        panic!("sauvegarde attendue");
    };
    let inventaire = inventaire_de_depart();
    let mut source = Vec::new();
    for t in &inventaire {
        if *t == "onboarding_state" {
            continue;
        }
        source.push((*t, lignes_brutes(&pool, t).await));
    }
    assert!(
        source
            .iter()
            .any(|(t, l)| *t == "journal_entry_lines" && l.len() == 2),
        "assertion de montage : la source porte les données peuplées"
    );

    let (seconde, nom) = seconde_base(&pool).await;
    montage::apply_migrations_up_to(
        &seconde,
        montage::migrations_before(PREMIERE_EN_ATTENTE, "mcp_prompts"),
    )
    .await
    .expect("seconde base au même état partiel");

    let limites = ReadLimits::production(2048);
    let parsed = parse_and_verify_avec_inventaire(
        File::open(&chemin).expect("sauvegarde"),
        &limites,
        &inventaire,
    )
    .expect("l'import de la version de départ accepte le fichier");
    check_schema_compat(&seconde, &parsed)
        .await
        .expect("colonnes compatibles avec la version de départ");
    let mut tx = seconde.begin().await.expect("transaction");
    restore_tables_in_tx_avec_inventaire(&mut tx, &parsed.tables, &inventaire)
        .await
        .expect("restauration");
    tx.commit().await.expect("commit");

    for (t, lignes) in &source {
        assert_eq!(
            &lignes_brutes(&seconde, t).await,
            lignes,
            "table {t} : lignes restaurées différentes de la source"
        );
    }

    let refus = parse_and_verify(File::open(&chemin).expect("sauvegarde"), &limites)
        .expect_err("l'inventaire du binaire refuse le fichier");
    match refus {
        AppError::InvalidBackupStructure(m) => assert!(
            m.contains("table 'mcp_prompt_versions' absente du manifeste"),
            "{m}"
        ),
        autre => panic!("refus inattendu : {autre:?}"),
    }

    seconde.close().await;
    sqlx::raw_sql(&format!("DROP DATABASE `{nom}`"))
        .execute(&pool)
        .await
        .expect("suppression de la seconde base");
}

// ---------------------------------------------------------------------------
// T2, T3, T4 — pas de sauvegarde sans migration
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t2_base_a_jour(pool: MySqlPool) {
    let dossiers = Dossiers::neufs();
    let issue = etape(
        &pool,
        &dossiers.config(None),
        EnvDeTest::normal(),
        instant(),
    )
    .await;
    assert_eq!(issue.expect("à jour"), Issue::AJour);
    assert!(!dossiers.pre_upgrade().exists(), "pre-upgrade/ non créé");
}

#[sqlx::test(migrations = false)]
async fn t3_base_neuve(pool: MySqlPool) {
    assert_eq!(plan_de_migration(&pool).await.expect("plan"), Plan::Neuve);
    let dossiers = Dossiers::neufs();
    let config = dossiers.config(None);

    let issue = sauvegarder_avant_migration(
        &pool,
        &config,
        &DowngradeCheckOutcome::FreshInstall,
        BINAIRE,
        EnvDeTest::normal(),
        instant(),
    )
    .await;
    assert_eq!(issue.expect("installation neuve"), Issue::InstallationNeuve);
    assert!(!dossiers.pre_upgrade().exists());

    // *neuve* hors `FreshInstall` : incohérent (V5-4).
    let refus = sauvegarder_avant_migration(
        &pool,
        &config,
        &DowngradeCheckOutcome::Aligned,
        BINAIRE,
        EnvDeTest::normal(),
        instant(),
    )
    .await
    .expect_err("plan incohérent refusé");
    assert!(matches!(refus, Refus::Plan { erreur: None }), "{refus:?}");
    assert!(!dossiers.pre_upgrade().exists());

    // L'échappatoire ne franchit pas le plan incohérent.
    let refus = sauvegarder_avant_migration(
        &pool,
        &dossiers.config(Some(BINAIRE)),
        &DowngradeCheckOutcome::Aligned,
        BINAIRE,
        EnvDeTest::normal(),
        instant(),
    )
    .await
    .expect_err("plan incohérent refusé malgré l'échappatoire");
    assert!(matches!(refus, Refus::Plan { erreur: None }), "{refus:?}");
    assert!(!dossiers.pre_upgrade().exists());
    let message = message_de_refus(&refus, BINAIRE);
    assert!(
        message.contains("état des migrations incohérent"),
        "{message}"
    );
    assert!(
        !message.contains("KESH_SKIP_PRE_UPGRADE_BACKUP"),
        "le refus incohérent ne conseille pas l'échappatoire : {message}"
    );
}

#[sqlx::test(migrations = false)]
async fn t4_migration_interrompue(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    sqlx::query(
        "UPDATE _sqlx_migrations SET success = 0 \
         WHERE version = (SELECT v FROM (SELECT MAX(version) AS v FROM _sqlx_migrations) AS m)",
    )
    .execute(&pool)
    .await
    .expect("ligne interrompue");
    let dossiers = Dossiers::neufs();
    let issue = etape(
        &pool,
        &dossiers.config(None),
        EnvDeTest::normal(),
        instant(),
    )
    .await;
    assert_eq!(issue.expect("interrompue"), Issue::Interrompue);
    assert!(!dossiers.pre_upgrade().exists());
}

// ---------------------------------------------------------------------------
// T5 à T10, T15 — un échec refuse la migration
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = false)]
async fn t5_dossier_non_creable(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let fichier = dossiers.backup.path().join("un-fichier");
    std::fs::write(&fichier, b"pas un dossier").unwrap();
    let mut config = dossiers.config(None);
    config.admin_backup_dir = fichier.to_string_lossy().to_string();
    let refus = etape(&pool, &config, EnvDeTest::normal(), instant())
        .await
        .expect_err("dossier non créable");
    assert!(matches!(refus, Refus::Dossier { .. }), "{refus:?}");
    let message = message_de_refus(&refus, BINAIRE);
    assert!(message.contains("mise à jour refusée"), "{message}");
    assert!(message.contains("dossier"), "{message}");
    assert!(
        message.contains(&format!("KESH_SKIP_PRE_UPGRADE_BACKUP={BINAIRE}")),
        "{message}"
    );
}

#[sqlx::test(migrations = false)]
async fn t6_espace_insuffisant(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let env: Arc<dyn BackupEnv> = Arc::new(EnvDeTest {
        libre: 1024,
        ecrivain: Ecrivain::Normal,
        sync_refuse: false,
    });
    let refus = etape(&pool, &dossiers.config(None), env, instant())
        .await
        .expect_err("espace insuffisant");
    assert!(
        matches!(
            refus,
            Refus::Espace {
                libre: Ok(1024),
                ..
            }
        ),
        "{refus:?}"
    );
    assert!(dossiers.sauvegardes().is_empty(), "rien d'écrit");
    let message = message_de_refus(&refus, BINAIRE);
    assert!(message.contains("espace insuffisant"), "{message}");
}

#[sqlx::test(migrations = false)]
async fn t7_ecriture_disque_plein(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let env: Arc<dyn BackupEnv> = Arc::new(EnvDeTest {
        libre: u64::MAX,
        ecrivain: Ecrivain::Plein,
        sync_refuse: false,
    });
    let refus = etape(&pool, &dossiers.config(None), env, instant())
        .await
        .expect_err("écriture refusée");
    assert!(matches!(refus, Refus::Ecriture { .. }), "{refus:?}");
    assert!(dossiers.sauvegardes().is_empty());
}

#[sqlx::test(migrations = false)]
async fn t7_bis_dossier_non_synchronise(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let env: Arc<dyn BackupEnv> = Arc::new(EnvDeTest {
        libre: u64::MAX,
        ecrivain: Ecrivain::Normal,
        sync_refuse: true,
    });
    let issue = etape(&pool, &dossiers.config(None), env, instant())
        .await
        .expect("la sauvegarde reste écrite");
    match issue {
        Issue::Ecrite {
            chemin,
            dossier_non_synchronise: Some(_),
            echappatoire_ignoree: None,
        } => assert!(chemin.is_file(), "fichier présent"),
        autre => panic!("Ecrite avec dossier non synchronisé attendue : {autre:?}"),
    }
}

#[sqlx::test(migrations = false)]
async fn t9_plan_illisible(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    pool.close().await;
    assert!(
        plan_de_migration(&pool).await.is_err(),
        "plan sur pool fermé"
    );
    let refus = sauvegarder_avant_migration(
        &pool,
        &dossiers.config(None),
        &DowngradeCheckOutcome::Aligned,
        BINAIRE,
        EnvDeTest::normal(),
        instant(),
    )
    .await
    .expect_err("plan illisible refusé");
    assert!(
        matches!(refus, Refus::Plan { erreur: Some(_) }),
        "{refus:?}"
    );
    let message = message_de_refus(&refus, BINAIRE);
    assert!(
        message.contains("état des migrations illisible"),
        "{message}"
    );
    assert!(dossiers.sauvegardes().is_empty());
}

#[sqlx::test(migrations = false)]
async fn t10_lecture_de_la_base(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    pool.close().await;
    let refus = ecrire_la_sauvegarde(
        &pool,
        &dossiers.config(None),
        EnvDeTest::normal(),
        "0.17.3",
        BINAIRE,
        instant(),
    )
    .await
    .expect_err("lecture refusée");
    assert!(matches!(refus, Refus::Lecture { .. }), "{refus:?}");
    assert!(dossiers.sauvegardes().is_empty());
}

#[sqlx::test(migrations = false)]
async fn t15_panique_de_la_tache(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let env: Arc<dyn BackupEnv> = Arc::new(EnvDeTest {
        libre: u64::MAX,
        ecrivain: Ecrivain::Panique,
        sync_refuse: false,
    });
    let refus = etape(&pool, &dossiers.config(None), env, instant())
        .await
        .expect_err("tâche interrompue");
    assert!(matches!(refus, Refus::Tache { .. }), "{refus:?}");
    assert!(dossiers.sauvegardes().is_empty(), "aucun fichier final");
}

// ---------------------------------------------------------------------------
// T11, T11-bis — l'échappatoire
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = false)]
async fn t11_echappatoire_sur_base_partielle(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let issue = etape(
        &pool,
        &dossiers.config(Some(BINAIRE)),
        EnvDeTest::normal(),
        instant(),
    )
    .await;
    assert_eq!(issue.expect("contournée"), Issue::Contournee);
    assert!(dossiers.sauvegardes().is_empty(), "aucun fichier");

    let issue = etape(
        &pool,
        &dossiers.config(Some("0.0.1")),
        EnvDeTest::normal(),
        instant(),
    )
    .await
    .expect("sauvegarde écrite malgré la valeur ignorée");
    match issue {
        Issue::Ecrite {
            chemin,
            echappatoire_ignoree: Some(v),
            dossier_non_synchronise: None,
        } => {
            assert_eq!(v, "0.0.1");
            assert!(chemin.is_file());
        }
        autre => panic!("Ecrite avec échappatoire ignorée attendue : {autre:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn t11_echappatoire_sur_base_a_jour(pool: MySqlPool) {
    let dossiers = Dossiers::neufs();
    for skip in [BINAIRE, "0.0.1"] {
        let issue = etape(
            &pool,
            &dossiers.config(Some(skip)),
            EnvDeTest::normal(),
            instant(),
        )
        .await;
        assert_eq!(
            issue.expect("à jour"),
            Issue::AJour,
            "skip = {skip} : aucune migration contournée"
        );
    }
    assert!(dossiers.sauvegardes().is_empty());
    assert!(!dossiers.pre_upgrade().exists());
}

#[sqlx::test(migrations = false)]
async fn t11_bis_echappatoire_avant_le_plan(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    pool.close().await;
    let issue = sauvegarder_avant_migration(
        &pool,
        &dossiers.config(Some(BINAIRE)),
        &DowngradeCheckOutcome::Aligned,
        BINAIRE,
        EnvDeTest::normal(),
        instant(),
    )
    .await;
    assert_eq!(
        issue.expect("l'échappatoire passe avant le plan"),
        Issue::Contournee
    );
}

// ---------------------------------------------------------------------------
// T13, T14 — aucune suppression ; le nom
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = false)]
async fn t13_deux_executions_deux_fichiers(pool: MySqlPool) {
    monter_base_partielle(&pool).await;
    let dossiers = Dossiers::neufs();
    let config = dossiers.config(None);
    etape(&pool, &config, EnvDeTest::normal(), instant())
        .await
        .expect("première");
    etape(
        &pool,
        &config,
        EnvDeTest::normal(),
        instant() + TimeDelta::milliseconds(1),
    )
    .await
    .expect("seconde");
    assert_eq!(
        dossiers.sauvegardes().len(),
        2,
        "{:?}",
        dossiers.sauvegardes()
    );
}

#[test]
fn t14_nom_du_fichier() {
    assert_eq!(
        nom_de_fichier(instant(), "0.17.3", "0.17.4"),
        "kesh-pre-upgrade-20261105T102030123Z-0.17.3-to-0.17.4.keshbackup"
    );
    assert_eq!(
        nom_de_fichier(instant(), "0.17.0-rc1", "1.0.0+build/x"),
        "kesh-pre-upgrade-20261105T102030123Z-0.17.0-rc1-to-1.0.0buildx.keshbackup"
    );
    assert_eq!(
        nom_de_fichier(instant(), "+++", ""),
        "kesh-pre-upgrade-20261105T102030123Z-inconnue-to-inconnue.keshbackup"
    );
    let long = "1".repeat(41);
    let quarante = "1".repeat(40);
    assert_eq!(
        nom_de_fichier(instant(), &long, "0.17.4"),
        format!("kesh-pre-upgrade-20261105T102030123Z-{quarante}-to-0.17.4.keshbackup")
    );
}

/// Les messages de refus : cause, conseil de l'échappatoire à la version du
/// binaire, jamais `DATABASE_URL`.
#[test]
fn messages_de_refus() {
    let refus = Refus::Espace {
        dir: PathBuf::from("/data/backup/pre-upgrade"),
        libre: Ok(10 * 1024 * 1024 + 1),
        seuil: 300 * 1024 * 1024 + 1,
    };
    let m = message_de_refus(&refus, "0.17.4");
    assert!(
        m.contains(
            "espace insuffisant dans /data/backup/pre-upgrade (10 Mio libres, 301 Mio requis)"
        ),
        "{m}"
    );
    assert!(m.starts_with("FATAL : mise à jour refusée"), "{m}");
    assert!(
        m.contains("posez KESH_SKIP_PRE_UPGRADE_BACKUP=0.17.4 dans le .env"),
        "{m}"
    );
    assert!(!m.contains("DATABASE_URL"));
    let illisible = Refus::Espace {
        dir: PathBuf::from("/d"),
        libre: Err(io::Error::other("statvfs")),
        seuil: 1,
    };
    assert!(
        message_de_refus(&illisible, "0.17.4").contains("espace libre de /d illisible : statvfs")
    );
}
