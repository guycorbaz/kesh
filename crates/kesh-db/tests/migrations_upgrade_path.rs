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
        "apply_migrations_up_to: n={} > total={} — vérifier que le calcul \
         `total - 8` (fenêtre d'upgrade Story 10-2 + companies_is_stub v011-2 \
         + bank_accounts_archived v014-1 + api_keys/audit_log_actor 17-2a) reste \
         cohérent avec l'ajout de migrations futures. Si une migration a été ajoutée à \
         la branche, l'assertion `total == 39` du test upgrade_path_preserves_data \
         devrait également fail pour signaler le besoin de mise à jour.",
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
/// + MIGRATOR.run() final (qui applique les 5 dernières : 4 de la fenêtre
/// Story 10-2 jusqu'à `20260522000001_kesh_version.sql` + `20260528000001_companies_is_stub.sql`
/// de Story v011-2). Assertion : seed préservé.
#[sqlx::test(migrations = false)]
async fn upgrade_path_preserves_data(pool: MySqlPool) {
    // Total migrations attendues : 26 historiques + _kesh_version (Story 10-2)
    // + companies_is_stub (Story v011-2) + bank_accounts_archived (Story v014-1)
    // + api_keys & audit_log_actor (Story 17-2a)
    // + users_email & password_reset_tokens (Story 17-4a)
    // + vat_rates_crud (Story 11-1) + vat_accounts_config (Story 18-1a)
    // + credit_notes (Story 12-1) + supplier_invoices (Story 12-2) + payment_batches (Story 12-3)
    // + imported_supplier_invoices (Story 12-5b) = 39.
    let total = kesh_db::MIGRATOR.migrations.len();
    assert_eq!(
        total, 39,
        "39 migrations attendues (26 historiques + _kesh_version Story 10-2 + companies_is_stub Story v011-2 + bank_accounts_archived Story v014-1 + api_keys & audit_log_actor Story 17-2a + users_email & password_reset_tokens Story 17-4a + vat_rates_crud Story 11-1 + vat_accounts_config Story 18-1a + credit_notes Story 12-1 + supplier_invoices Story 12-2 + payment_batches Story 12-3 + imported_supplier_invoices Story 12-5b)"
    );

    // Étape 1 : simule l'état pré-Story-10-2 en appliquant toutes les
    // migrations sauf les 13 dernières (reconciliation_8_4,
    // bank_account_journal_link, reconciliation_rules, _kesh_version,
    // companies_is_stub ajoutée Story v011-2, bank_accounts_archived
    // ajoutée Story v014-1, api_keys + audit_log_actor ajoutées Story 17-2a,
    // users_email + password_reset_tokens ajoutées Story 17-4a,
    // vat_rates_crud ajoutée Story 11-1, vat_accounts_config ajoutée Story 18-1a,
    // et credit_notes ajoutée Story 12-1).
    //
    // Note `total - N` : expression relative à la longueur totale, mais
    // l'assertion `total == 39` ci-dessus est INTENTIONNELLEMENT hardcode
    // pour fail-loud sur toute évolution non-revue. À chaque ajout d'une
    // migration future, le mainteneur doit (1) bumper le compte à la nouvelle
    // longueur (2) revoir si `total - N` cible toujours la bonne fenêtre.
    // La frontière reste à 23 (état pré-10-2) : les migrations ajoutées APRÈS
    // la fenêtre Story 10-2 (companies_is_stub v011-2, bank_accounts_archived
    // v014-1, api_keys + audit_log_actor 17-2a, users_email + password_reset_tokens
    // 17-4a, vat_rates_crud 11-1, vat_accounts_config 18-1a, credit_notes 12-1,
    // supplier_invoices 12-2, payment_batches 12-3, imported_supplier_invoices 12-5b)
    // élargissent la fenêtre d'upgrade et incrémentent donc le `N` soustrait
    // (de 4 à 6 à 8 à 10 à 11 à 12 à 13 à 14 à 15 à 16),
    // sans déplacer la frontière des 23 migrations historiques.
    let n_before_upgrade_window = total - 16;
    apply_migrations_up_to(&pool, n_before_upgrade_window)
        .await
        .expect("apply_migrations_up_to(total - 16) failed");

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

    // Étape 3 : appliquer les 10 migrations restantes via MIGRATOR.run().
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
                .expect("SELECT COUNT(*) FROM companies failed"),
        ),
        (
            "users".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM users")
                .fetch_one(&pool)
                .await
                .expect("SELECT COUNT(*) FROM users failed"),
        ),
        (
            "accounts".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
                .fetch_one(&pool)
                .await
                .expect("SELECT COUNT(*) FROM accounts failed"),
        ),
        (
            "contacts".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM contacts")
                .fetch_one(&pool)
                .await
                .expect("SELECT COUNT(*) FROM contacts failed"),
        ),
        (
            "invoices".into(),
            sqlx::query_scalar("SELECT COUNT(*) FROM invoices")
                .fetch_one(&pool)
                .await
                .expect("SELECT COUNT(*) FROM invoices failed"),
        ),
    ];
    let expected = [
        ("companies", 1),
        ("users", 1),
        // Story 18-1a : la migration vat_accounts_config ajoute 2 comptes TVA
        // (1171 Impôt préalable + 2206 Décompte TVA) par company existante.
        // Le seed de 2 comptes (1000 Caisse + 3000 Ventes) reste préservé → 2 + 2 = 4.
        // (parent 10/20 absent de ce fixture minimal → comptes créés orphelins, toléré.)
        ("accounts", 4),
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
        .expect("SELECT name FROM companies failed");
    assert_eq!(c_name, "Upgrade Path SA");

    let u_username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT username FROM users failed");
    assert_eq!(u_username, "upgrade_admin");

    let a_caisse_number: String = sqlx::query_scalar("SELECT number FROM accounts WHERE id = ?")
        .bind(account_caisse_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT number FROM accounts (caisse) failed");
    assert_eq!(a_caisse_number, "1000");

    let a_ventes_number: String = sqlx::query_scalar("SELECT number FROM accounts WHERE id = ?")
        .bind(account_ventes_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT number FROM accounts (ventes) failed");
    assert_eq!(a_ventes_number, "3000");

    let i_status: String = sqlx::query_scalar("SELECT status FROM invoices WHERE id = ?")
        .bind(invoice_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT status FROM invoices failed");
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

/// Couvre explicitement le bras `FreshInstall` de `check_downgrade_protection`
/// — ER_NO_SUCH_TABLE 1146 → `Ok(FreshInstall)`. Sans ce test, le pattern
/// `try_downcast_ref::<MySqlDatabaseError>().is_some_and(|e| e.number() == 1146)`
/// n'est exercé runtime par aucun autre test (tous les autres utilisent
/// `#[sqlx::test(migrator = ...)]` qui crée la table avant le corps du test).
/// Un futur bump sqlx qui modifierait le downcast passerait silencieusement
/// jusqu'à un premier déploiement sur DB vierge.
#[sqlx::test(migrations = false)]
async fn check_downgrade_protection_fresh_install_on_empty_db(pool: MySqlPool) {
    // DB éphémère sans aucune migration appliquée → table `_kesh_version` absente.
    let result = check_downgrade_protection(&pool, "0.1.0").await;
    assert_eq!(
        result.expect("check_downgrade_protection on empty DB should succeed"),
        DowngradeCheckOutcome::FreshInstall,
        "DB vierge sans _kesh_version → FreshInstall via ER_NO_SUCH_TABLE 1146"
    );
}

/// Couvre explicitement le bras `VersionError::RowMissing` de
/// `check_downgrade_protection` — table `_kesh_version` présente mais
/// row id=1 absente (TRUNCATE accidentel ou restore partiel). Ce variant
/// existe comme défense contre une corruption silencieuse, sans test
/// runtime il serait dead code (un bump sqlx 0.9+ modifiant le mapping
/// de `fetch_one` sur empty result casserait la branche silencieusement).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_downgrade_protection_row_missing_after_truncate(pool: MySqlPool) {
    // Pre-state : MIGRATOR appliqué → _kesh_version existe + row id=1 présente.
    // On simule un TRUNCATE / DELETE manuel qui vide la table sans la dropper.
    sqlx::query("DELETE FROM _kesh_version WHERE id = 1")
        .execute(&pool)
        .await
        .expect("DELETE _kesh_version row failed");

    let result = check_downgrade_protection(&pool, "0.1.0").await;
    match result {
        Err(VersionError::RowMissing) => {
            // OK — la table existe (pas FreshInstall) mais la row est
            // absente → diagnostic explicite pour l'opérateur.
        }
        other => panic!(
            "Expected Err(RowMissing) après DELETE row id=1, got {:?}",
            other
        ),
    }
}

/// Couvre le bras `VersionError::InvalidSemver { origin: "base de données", .. }`
/// — la colonne `kesh_version_min_required` contient une string non-SemVer
/// (corruption manuelle, downgrade depuis un binaire qui aurait écrit
/// n'importe quoi). Ce variant inclut la valeur fautive (tronquée 64
/// chars) pour aider l'opérateur à diagnostiquer.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_downgrade_protection_invalid_db_semver(pool: MySqlPool) {
    // Injecte une string non-SemVer dans la colonne.
    sqlx::query("UPDATE _kesh_version SET kesh_version_min_required = 'not-a-semver' WHERE id = 1")
        .execute(&pool)
        .await
        .expect("UPDATE _kesh_version with invalid semver failed");

    let result = check_downgrade_protection(&pool, "0.1.0").await;
    match result {
        Err(VersionError::InvalidSemver {
            origin,
            value,
            error: _,
        }) => {
            assert_eq!(origin, "base de données");
            assert_eq!(value, "not-a-semver");
        }
        other => panic!(
            "Expected Err(InvalidSemver {{ origin: \"base de données\", value: \"not-a-semver\", .. }}), got {:?}",
            other
        ),
    }
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
///
/// AC #17 autorise explicitement une assertion `IS NOT NULL` plutôt qu'une
/// fenêtre temporelle. On préfère cette forme : la comparaison
/// `chrono::Utc::now().naive_utc()` (côté Rust UTC) vs `NOW()` (côté
/// MariaDB, dépend de `time_zone`) introduit une flakiness sur les
/// installations locales où MariaDB n'est pas configuré en UTC (typique
/// NAS Synology Europe/Zurich). L'objectif du test est de prouver que
/// l'UPDATE met bien le champ à une valeur — l'exactitude horloge relève
/// d'un test d'horloge, pas d'un test de migration.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn record_boot_version_updates_row(pool: MySqlPool) {
    // Pre-state : last_boot_at NULL (set par la migration).
    let pre_last_boot: Option<chrono::NaiveDateTime> =
        sqlx::query_scalar("SELECT last_boot_at FROM _kesh_version WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("SELECT last_boot_at pre-record failed");
    assert!(pre_last_boot.is_none());

    // Utilise une valeur DIFFÉRENTE du pré-state ('0.1.0' figé par la migration
    // initiale) pour prouver que l'UPDATE écrit bien kesh_version_last_applied
    // — sinon assertion `'0.1.0' == '0.1.0'` serait toujours vraie et ne
    // détecterait pas une régression où la fonction oublierait d'écrire la
    // colonne (Pass 3 BH3-2 catch).
    record_boot_version(&pool, "9.9.9")
        .await
        .expect("record_boot_version failed");

    let (last_applied, last_boot_at): (String, Option<chrono::NaiveDateTime>) = sqlx::query_as(
        "SELECT kesh_version_last_applied, last_boot_at FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("SELECT _kesh_version post-record failed");

    assert_eq!(
        last_applied, "9.9.9",
        "record_boot_version doit écrire kesh_version_last_applied (pré-state '0.1.0' devait être remplacé par '9.9.9')"
    );
    assert!(
        last_boot_at.is_some(),
        "last_boot_at devrait être non-NULL après record_boot_version()"
    );
}
