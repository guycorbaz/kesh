//! Tests d'intégration — Story 10-2 AC #14.
//!
//! Vérifie qu'un fresh install (DB vierge → `MIGRATOR.run()` complet)
//! produit le schéma attendu : toutes les tables présentes, un seed
//! minimal qui round-trip OK, et la row initiale `_kesh_version`.
//!
//! Utilise `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` qui provisionne
//! une DB éphémère puis applique le migrator complet AVANT le corps du
//! test (cf. AC #18 + Dev Notes §"Pattern test migrations").

use sqlx::MySqlPool;
use sqlx::Row;

/// AC #14a — toutes les tables attendues existent après MIGRATOR.run() complet.
///
/// Liste minimale de 23 tables (à la date de Story 10-2). Sémantique
/// **subset** : assertion que chaque table de la liste minimale existe
/// dans `SHOW TABLES` (pas d'égalité stricte qui casserait sur ajout
/// futur d'une nouvelle table par un Epic ultérieur).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migrations_apply_all_tables_present(pool: MySqlPool) {
    let rows = sqlx::query("SHOW TABLES")
        .fetch_all(&pool)
        .await
        .expect("SHOW TABLES failed");

    let present: std::collections::HashSet<String> =
        rows.into_iter().map(|r| r.get::<String, _>(0)).collect();

    let expected = [
        "_kesh_version",
        "_sqlx_migrations",
        "accounts",
        "audit_log",
        "bank_accounts",
        "bank_imports",
        "bank_profiles",
        "bank_transactions",
        "companies",
        "company_dunning_settings",
        "company_invoice_settings",
        "contacts",
        "dunning_levels",
        "fiscal_years",
        "invoice_lines",
        "invoice_number_sequences",
        "invoices",
        "journal_entries",
        "journal_entry_lines",
        "onboarding_state",
        "products",
        "reconciliation_rules",
        "refresh_tokens",
        "users",
        "vat_rates",
    ];

    let missing: Vec<&str> = expected
        .iter()
        .copied()
        .filter(|t| !present.contains(*t))
        .collect();

    assert!(
        missing.is_empty(),
        "Tables manquantes après MIGRATOR.run() complet : {:?}. Présentes : {:?}",
        missing,
        present
    );
}

/// AC #14b — seed minimal round-trip OK.
///
/// INSERT 1 company → 1 user (Argon2 mock) → 1 contact (client) → 1 account
/// (Asset) → 1 fiscal_year (Open) → 1 journal_entry → 1 invoice (draft).
/// SELECT chaque ligne avec assertion sur colonnes clés.
///
/// Aucune validation business comptable lourde (pas d'invariant partie-
/// double sur 1 ligne, pas d'entry_lines) — test de schéma uniquement
/// (l'objectif AC #14b est de prouver que le schéma accepte les INSERTs
/// principaux, pas de tester la logique applicative).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migrations_minimal_seed_roundtrips(pool: MySqlPool) {
    // 1 company (org_type / accounting_language / instance_language NOT NULL)
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) VALUES (?, ?, ?, ?, ?) RETURNING id",
    )
    .bind("Test Migrations SA")
    .bind("Rue du Test 1, 1000 Lausanne")
    .bind("Pme")
    .bind("FR")
    .bind("FR")
    .fetch_one(&pool)
    .await
    .expect("INSERT company failed");

    // 1 fiscal_year (Open par défaut — requis pour journal_entries FK)
    let fiscal_year_id: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("Exercice 2026")
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 12, 31).unwrap())
    .fetch_one(&pool)
    .await
    .expect("INSERT fiscal_year failed");

    // 1 user (Argon2 mock — schéma uniquement, pas un vrai hash)
    // users.company_id NOT NULL depuis migration 20260419000002 (Story 6-2 multi-tenant).
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind("test_admin")
    .bind("$argon2id$v=19$m=19456,t=2,p=1$mock_salt$mock_hash_for_schema_test_only")
    .bind("Admin")
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .expect("INSERT user failed");

    // 1 contact (requis pour invoice FK)
    let contact_id: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_client) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("Personne")
    .bind("Client Test")
    .bind(true)
    .fetch_one(&pool)
    .await
    .expect("INSERT contact failed");

    // 1 account (number + name + account_type)
    let account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind("1000")
    .bind("Caisse test")
    .bind("Asset")
    .fetch_one(&pool)
    .await
    .expect("INSERT account failed");

    // 1 journal_entry (entry_number = 1, journal = OD)
    let je_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) VALUES (?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(fiscal_year_id)
    .bind(1_i64)
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 5, 22).unwrap())
    .bind("OD")
    .bind("Test entry")
    .fetch_one(&pool)
    .await
    .expect("INSERT journal_entry failed");

    // 1 invoice (draft, total_amount=0 défaut)
    let invoice_id: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, date) VALUES (?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(contact_id)
    .bind(chrono::NaiveDate::from_ymd_opt(2026, 5, 22).unwrap())
    .fetch_one(&pool)
    .await
    .expect("INSERT invoice failed");

    // Round-trip : SELECT chaque ligne et assert sur colonnes clés.
    let (c_name, c_org_type): (String, String) =
        sqlx::query_as("SELECT name, org_type FROM companies WHERE id = ?")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .expect("SELECT company failed");
    assert_eq!(c_name, "Test Migrations SA");
    assert_eq!(c_org_type, "Pme");

    let (fy_name, fy_status): (String, String) =
        sqlx::query_as("SELECT name, status FROM fiscal_years WHERE id = ?")
            .bind(fiscal_year_id)
            .fetch_one(&pool)
            .await
            .expect("SELECT fiscal_year failed");
    assert_eq!(fy_name, "Exercice 2026");
    assert_eq!(fy_status, "Open", "fiscal_year status default = 'Open'");

    let u_username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(user_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT user failed");
    assert_eq!(u_username, "test_admin");

    let a_number: String = sqlx::query_scalar("SELECT number FROM accounts WHERE id = ?")
        .bind(account_id)
        .fetch_one(&pool)
        .await
        .expect("SELECT account failed");
    assert_eq!(a_number, "1000");

    let (je_journal, je_entry_number): (String, i64) =
        sqlx::query_as("SELECT journal, entry_number FROM journal_entries WHERE id = ?")
            .bind(je_id)
            .fetch_one(&pool)
            .await
            .expect("SELECT journal_entry failed");
    assert_eq!(je_journal, "OD");
    assert_eq!(je_entry_number, 1);

    let (i_status, i_total): (String, rust_decimal::Decimal) =
        sqlx::query_as("SELECT status, total_amount FROM invoices WHERE id = ?")
            .bind(invoice_id)
            .fetch_one(&pool)
            .await
            .expect("SELECT invoice failed");
    assert_eq!(i_status, "draft", "invoice status default = 'draft'");
    assert_eq!(
        i_total,
        rust_decimal::Decimal::ZERO,
        "invoice total_amount default = 0"
    );
}

/// AC #14c — la row initiale `_kesh_version` est créée par la migration
/// `20260522000001_kesh_version.sql` avec `('0.1.0', '0.1.0')`, puis
/// `kesh_version_min_required` est bumpé à `'0.7.0'` par le 1er bump breaking
/// du repo (Story 21-3, `email_templates_reminder.sql`) — `kesh_version_last_applied`
/// reste `'0.1.0'` (non touché par la migration, écrit au boot seulement).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migrations_kesh_version_initial_row(pool: MySqlPool) {
    let (min_required, last_applied, last_boot_at): (String, String, Option<chrono::NaiveDateTime>) =
        sqlx::query_as(
            "SELECT kesh_version_min_required, kesh_version_last_applied, last_boot_at FROM _kesh_version WHERE id = 1",
        )
        .fetch_one(&pool)
        .await
        .expect("SELECT _kesh_version failed");

    assert_eq!(min_required, "0.7.0");
    assert_eq!(last_applied, "0.1.0");
    // last_boot_at reste NULL tant que record_boot_version() n'a pas été
    // appelé (le test n'invoque pas la boot integration de main.rs).
    assert!(
        last_boot_at.is_none(),
        "last_boot_at devrait être NULL avant le premier record_boot_version"
    );

    // Vérifie que la singleton-row constraint est appliquée : impossible
    // d'insérer une 2e row avec id=2. On vérifie en plus le SQLSTATE
    // (classe 23xxx — integrity constraint violation) pour s'assurer que
    // l'erreur vient bien d'une violation de contrainte et non d'une autre
    // cause (table absente, pool fermé, etc.) qui passerait l'assertion
    // `is_err()` silencieusement.
    let err = sqlx::query(
        "INSERT INTO _kesh_version (id, kesh_version_min_required, kesh_version_last_applied) VALUES (2, '0.2.0', '0.2.0')",
    )
    .execute(&pool)
    .await
    .expect_err("CHECK (id = 1) devrait empêcher INSERT id=2, got Ok");

    let sqlstate = err
        .as_database_error()
        .and_then(|db_err| db_err.code())
        .map(|c| c.to_string())
        .unwrap_or_default();
    assert!(
        sqlstate.starts_with("23"),
        "INSERT id=2 devrait échouer avec SQLSTATE 23xxx (integrity constraint violation), got {:?} ({})",
        sqlstate,
        err
    );
}
