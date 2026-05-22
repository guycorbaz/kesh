//! Tests d'intégration — Story 10-2 AC #15.
//!
//! Vérifie le upgrade path : une DB partiellement migrée préserve les
//! données quand `MIGRATOR.run()` applique les migrations restantes.
//! Vérifie aussi la downgrade protection (un binaire ancien refuse de
//! boot sur une DB avec `kesh_version_min_required` supérieur).
//!
//! ⚠️ Le test `upgrade_path_preserves_data` utilise
//! `#[sqlx::test(migrations = false)]` — pas `#[sqlx::test]` par défaut,
//! qui appliquerait `./migrations` automatiquement (`MigrationsOpt::
//! InferredPath`, cf. `sqlx-macros-core-0.8.6/src/test_attr.rs:196`).
//! Avec `migrations = false` (mapped à `MigrationsOpt::Disabled`, cf.
//! `test_attr.rs:265`), la DB éphémère reste vide pour que le test
//! contrôle l'application progressive des migrations.

use kesh_db::version::{
    DowngradeCheckOutcome, VersionError, check_downgrade_protection, record_boot_version,
};
use sqlx::MySqlPool;
use sqlx::migrate::Migrator;
use std::borrow::Cow;

/// Helper : applique les `n` premières migrations historiques via
/// sub-Migrator. Le champ `Migrator::migrations` est `pub Cow<'static,
/// [Migration]>` en sqlx 0.8.6 (`migrate/migrator.rs:14-22`, `#[doc(hidden)]`
/// + commentaire « semver-exempt »).
///
/// Préserve les checksums SHA-384 réels → `MIGRATOR.run()` final ne
/// déclenche pas `MigrateError::VersionMismatch`.
async fn apply_migrations_up_to(
    pool: &MySqlPool,
    n: usize,
) -> Result<(), sqlx::migrate::MigrateError> {
    let all = &kesh_db::MIGRATOR.migrations;
    assert!(
        n <= all.len(),
        "apply_migrations_up_to: n={} > total={}",
        n,
        all.len()
    );
    let sub = Migrator {
        migrations: Cow::Borrowed(&all[..n]),
        ignore_missing: kesh_db::MIGRATOR.ignore_missing,
        locking: kesh_db::MIGRATOR.locking,
        no_tx: kesh_db::MIGRATOR.no_tx,
    };
    sub.run(pool).await
}

/// AC #15a — cas générique upgrade path : 23 migrations appliquées + seed
/// + MIGRATOR.run() final (qui applique les 3 dernières historiques +
/// `20260522000001_kesh_version.sql` = 4 migrations totales par le full
/// run). Assertion : seed préservé.
#[sqlx::test(migrations = false)]
async fn upgrade_path_preserves_data(pool: MySqlPool) {
    // Total migrations attendues : 26 historiques + 1 nouvelle = 27.
    let total = kesh_db::MIGRATOR.migrations.len();
    assert_eq!(
        total, 27,
        "Story 10-2 attend 27 migrations totales (26 historiques + _kesh_version)"
    );

    // Étape 1 : appliquer les 23 premières migrations historiques (= 26 − 3).
    apply_migrations_up_to(&pool, 23)
        .await
        .expect("apply_migrations_up_to(23) failed");

    // Étape 2 : seed 1 company + 1 user + 2 accounts + 1 invoice + 1 contact.
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind("Upgrade Path SA")
    .bind("Rue de l'Upgrade 1, 1000 Lausanne")
    .bind("Pme")
    .bind("FR")
    .bind("FR")
    .fetch_one(&pool)
    .await
    .expect("INSERT company failed");

    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind("upgrade_admin")
    .bind("$argon2id$v=19$m=19456,t=2,p=1$mock_salt$mock_hash_for_schema_test_only")
    .bind("Admin")
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .expect("INSERT user failed");

    let account_caisse_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("1000")
    .bind("Caisse")
    .bind("Asset")
    .fetch_one(&pool)
    .await
    .expect("INSERT account 1000 failed");

    let account_ventes_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("3000")
    .bind("Ventes")
    .bind("Revenue")
    .fetch_one(&pool)
    .await
    .expect("INSERT account 3000 failed");

    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("Personne")
    .bind("Client Upgrade")
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("INSERT contact failed");

    let invoice_id: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, date) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 5, 22).unwrap())
    .fetch_one(&pool)
    .await
    .expect("INSERT invoice failed");

    // Étape 3 : appliquer les 4 migrations restantes via MIGRATOR.run().
    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("MIGRATOR.run() final failed — upgrade path broken");

    // Étape 4 : assertions COUNT(*) sur les 5 tables seedées inchangé.
    let counts: Vec<(String, i64)> = vec![
        (
            "companies".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM companies")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
        (
            "users".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
        (
            "accounts".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
        (
            "contacts".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
        (
            "invoices".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM invoices")
                .fetch_one(&pool)
                .await
                .unwrap(),
        ),
    ];
    let expected = [
        ("companies", 1),
        ("users", 1),
        ("accounts", 2),
        ("contacts", 1),
        ("invoices", 1),
    ];
    for ((table, actual), (_, exp)) in counts.iter().zip(expected.iter()) {
        assert_eq!(
            *actual, *exp,
            "COUNT({}) après upgrade : expected {}, got {}",
            table, exp, actual
        );
    }

    // Étape 5 : assertion seed rows préservées (colonnes scalaires).
    let c_name: String = sqlx::query_scalar("SELECT name FROM companies WHERE id = ?")
        .bind(company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(c_name, "Upgrade Path SA");

    let u_username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(u_username, "upgrade_admin");

    let a_caisse_number: String = sqlx::query_scalar("SELECT number FROM accounts WHERE id = ?")
        .bind(account_caisse_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(a_caisse_number, "1000");

    let a_ventes_number: String = sqlx::query_scalar("SELECT number FROM accounts WHERE id = ?")
        .bind(account_ventes_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(a_ventes_number, "3000");

    let i_status: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(i_status, "draft");

    // Étape 6 : assertion `_kesh_version` créée + row initiale, last_boot_at NULL.
    let (last_applied, last_boot_at): (String, Option<chrono::NaiveDateTime>) = sqlx::query_as(
        "SELECT kesh_version_last_applied, last_boot_at FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("SELECT _kesh_version failed après upgrade");
    assert_eq!(last_applied, "0.1.0");
    assert!(
        last_boot_at.is_none(),
        "last_boot_at NULL avant record_boot_version() — le test n'invoque pas la boot integration"
    );
}

/// AC #15b — cas downgrade détecté : DB migrée full, on simule un futur
/// binaire qui aurait bumped `kesh_version_min_required` à `0.99.0`, puis
/// on appelle `check_downgrade_protection` avec un binaire `0.1.0` →
/// assertion `Err(VersionError::DowngradeRefused)`.
///
/// Utilise `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` (full migrator
/// avant le test — comportement souhaité ici contrairement à AC #15a) :
/// la table `_kesh_version` doit exister pour l'UPDATE.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_rejects_old_binary(pool: MySqlPool) {
    // Simule un binaire futur qui aurait bumped min_required.
    sqlx::query("UPDATE _kesh_version SET kesh_version_min_required = '0.99.0' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("UPDATE _kesh_version failed");

    // Tente le boot avec binaire 0.1.0 (plus ancien que 0.99.0) → refus.
    let result = check_downgrade_protection(&pool, "0.1.0").await;

    match result {
        Err(VersionError::DowngradeRefused { db_min, binary }) => {
            assert_eq!(db_min.to_string(), "0.99.0");
            assert_eq!(binary.to_string(), "0.1.0");
        }
        other => panic!(
            "Expected Err(DowngradeRefused {{ db_min: 0.99.0, binary: 0.1.0 }}), got {:?}",
            other
        ),
    }
}

/// Test additionnel : `check_downgrade_protection` retourne `Aligned`
/// quand binary == db_min (cas nominal upgrade-then-boot).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_aligned_when_binary_equals_min(pool: MySqlPool) {
    // Default migration : min_required = '0.1.0'.
    let result = check_downgrade_protection(&pool, "0.1.0").await;
    assert_eq!(
        result.unwrap(),
        DowngradeCheckOutcome::Aligned,
        "binary 0.1.0 == db_min 0.1.0 → Aligned"
    );
}

/// Test additionnel : `check_downgrade_protection` retourne `BinaryAhead`
/// quand binary > db_min (upgrade legitime).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_binary_ahead_when_binary_greater(pool: MySqlPool) {
    // Default migration : min_required = '0.1.0', binary 0.2.0 > 0.1.0.
    let result = check_downgrade_protection(&pool, "0.2.0").await;
    match result.unwrap() {
        DowngradeCheckOutcome::BinaryAhead { db_min, binary } => {
            assert_eq!(db_min.to_string(), "0.1.0");
            assert_eq!(binary.to_string(), "0.2.0");
        }
        other => panic!("Expected BinaryAhead, got {:?}", other),
    }
}

/// Test additionnel : `record_boot_version` met à jour `last_boot_at` à
/// non-NULL et `kesh_version_last_applied` à la version passée.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn record_boot_version_updates_row(pool: MySqlPool) {
    // Pre-state : last_boot_at NULL (set par la migration).
    let pre_last_boot: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT last_boot_at FROM _kesh_version WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(pre_last_boot.is_none());

    let test_start = chrono::Utc::now().naive_utc();

    record_boot_version(&pool, "0.1.0")
        .await
        .expect("record_boot_version failed");

    let (last_applied, last_boot_at): (String, Option<chrono::NaiveDateTime>) = sqlx::query_as(
        "SELECT kesh_version_last_applied, last_boot_at FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(last_applied, "0.1.0");
    let last_boot = last_boot_at.expect("last_boot_at devrait être non-NULL après record");
    // Assertion borne (pas égalité exacte — cf. AC #17).
    let test_end = chrono::Utc::now().naive_utc();
    assert!(
        last_boot >= test_start - chrono::TimeDelta::seconds(2)
            && last_boot <= test_end + chrono::TimeDelta::seconds(2),
        "last_boot_at hors fenêtre test : start={:?}, last_boot={:?}, end={:?}",
        test_start,
        last_boot,
        test_end
    );
}
