//! Fixtures de test partagées — Story 6.4.
//!
//! Ce module fournit `seed_accounting_company`, un helper unique qui crée
//! une company « comptablement complète » utilisable par :
//!
//! - les tests d'intégration Rust (`crates/kesh-api/tests/*.rs`) — remplace
//!   les bypass SQL `seed_validated_invoice_via_sql` historiques (cf. KF-001) ;
//! - l'endpoint runtime `POST /api/v1/_test/seed` de `kesh-api`, gated par
//!   l'env var `KESH_TEST_MODE` (cf. Story 6.4 ACs #6-#10).
//!
//! **Sécurité** : ce module est compilé en permanence (pas de `#[cfg(test)]`)
//! car l'endpoint `kesh-api` en a besoin. La protection contre l'exposition
//! prod est portée par le gate runtime `KESH_TEST_MODE` dans `build_router` —
//! voir `crates/kesh-api/src/lib.rs`. Le module en lui-même ne contient
//! aucune logique métier sensible : juste des INSERTs + TRUNCATEs.
//!
//! **Pas de dépendance argon2** : kesh-db ne hash pas les passwords (c'est
//! le rôle de kesh-api). Les hashes Argon2id pour `admin/admin123` et
//! `changeme/changeme` sont pré-calculés en consts ci-dessous (générés via
//! `Argon2::default()` qui matche `crate::auth::password::hash_password`
//! côté kesh-api). Le password vérifie via `verify_password` quel que soit
//! le salt — donc les consts restent valides indéfiniment tant que les
//! paramètres Argon2 par défaut ne changent pas.

use sqlx::MySqlPool;
use std::collections::HashMap;
use thiserror::Error;

/// Hash Argon2id pré-calculé du password `admin123` (paramètres par défaut
/// `Argon2::default()` = m=19456, t=2, p=1, variant Argon2id). Vérifiable
/// via `crate::auth::password::verify_password("admin123", ADMIN_PASSWORD_HASH)`
/// côté kesh-api.
pub const ADMIN_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$wDaFUbAJuozHKhQshibCHw$T/DeYTKABHDpW7JM5MoiQciUad5Eb81Cfvh0aUvi2Z4";

/// Hash Argon2id pré-calculé du password `changeme`.
pub const CHANGEME_PASSWORD_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$81LfElCxe1hOPUgMpSeZgQ$PVGb49qpxsepIv9NC+1fms5ROMCD3jueZLVcrW5yud0";

/// Identifiants des rows seedées par `seed_accounting_company`. Les champs
/// pointent sur les `id` réels en base (auto-incrément MariaDB) pour que
/// les tests puissent immédiatement s'en servir comme FK.
///
/// **Note** : `company_invoice_settings` n'a pas de colonne `id` (PK =
/// `company_id`), donc cette table est accessible via `company_id` et
/// n'est pas exposée dans `SeededCompany`.
#[derive(Debug, Clone)]
pub struct SeededCompany {
    pub company_id: i64,
    pub fiscal_year_id: i64,
    pub admin_user_id: i64,
    pub changeme_user_id: i64,
    /// Map `code` → `id` pour les 5 comptes seedés (1000, 1100, 2000, 3000, 4000).
    pub accounts: HashMap<&'static str, i64>,
}

/// Erreurs spécifiques aux fixtures (`thiserror`-based, mappable vers
/// `AppError::Internal` côté kesh-api).
#[derive(Debug, Error)]
pub enum FixtureError {
    #[error("DB error during fixture seeding: {0}")]
    Db(#[from] sqlx::Error),
}

/// Seede une company comptablement complète dans la DB pointée par `pool`.
///
/// Idempotent : les call sites doivent truncate les tables AVANT d'appeler
/// ce helper s'ils veulent un état déterministe (utiliser `truncate_all`).
///
/// Crée :
/// - 1 `companies` : `'CI Test Company'`, org_type `Independant`, langues FR/FR
/// - 2 `users` Admin actifs : `admin/admin123` + `changeme/changeme`
/// - 1 `fiscal_years` 2020-2030 status `Open`
/// - 5 `accounts` : 1000 Caisse (Asset), 1100 Banque (Asset), 2000 Capital
///   (Liability), 3000 Ventes (Revenue), 4000 Charges (Expense)
/// - 1 `company_invoice_settings` : default_receivable_account_id = compte
///   1100, default_revenue_account_id = compte 3000, default_sales_journal
///   = `Ventes`
///
/// Retourne `SeededCompany` avec tous les IDs nécessaires pour les tests
/// downstream.
pub async fn seed_accounting_company(pool: &MySqlPool) -> Result<SeededCompany, FixtureError> {
    // Company — adresse sur 2 lignes (line1 = rue, line2 = zip + ville) car
    // la génération QR Bill exige les deux lignes (cf. test `invoice_pdf_e2e`).
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('CI Test Company', 'Test Address 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await?;
    let company_id = company_result.last_insert_id() as i64;

    // Users (admin + changeme)
    let admin_result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active) VALUES (?, ?, 'Admin', TRUE)",
    )
    .bind("admin")
    .bind(ADMIN_PASSWORD_HASH)
    .execute(pool)
    .await?;
    let admin_user_id = admin_result.last_insert_id() as i64;

    let changeme_result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active) VALUES (?, ?, 'Admin', TRUE)",
    )
    .bind("changeme")
    .bind(CHANGEME_PASSWORD_HASH)
    .execute(pool)
    .await?;
    let changeme_user_id = changeme_result.last_insert_id() as i64;

    // Fiscal year (2020-2030, large pour tolérer dérive d'horloge CI)
    let fy_result = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice CI 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await?;
    let fiscal_year_id = fy_result.last_insert_id() as i64;

    // Accounts (5 minimum pour couvrir les types Asset/Liability/Revenue/Expense)
    let mut accounts = HashMap::new();
    for (code, name, account_type) in &[
        ("1000", "Caisse CI", "Asset"),
        ("1100", "Banque CI", "Asset"),
        ("2000", "Capital CI", "Liability"),
        ("3000", "Ventes CI", "Revenue"),
        ("4000", "Charges CI", "Expense"),
    ] {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await?;
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    // Company invoice settings : default receivable = 1100 Banque, default
    // revenue = 3000 Ventes, default sales journal = Ventes (cf. AC #2 + #8).
    // Note : PK = company_id, pas d'AUTO_INCREMENT — on s'appuie sur la FK
    // pour retrouver la row.
    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, default_sales_journal) \
         VALUES (?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .execute(pool)
    .await?;

    Ok(SeededCompany {
        company_id,
        fiscal_year_id,
        admin_user_id,
        changeme_user_id,
        accounts,
    })
}

/// Truncate toutes les tables (sauf `_sqlx_migrations`) dans l'ordre
/// FK enfants → parents, avec `FOREIGN_KEY_CHECKS = 0` pour bypasser
/// l'ordre strict. Réinitialise aussi les `AUTO_INCREMENT`.
///
/// **Important** : utilise une connection unique acquise depuis le pool
/// pour que `SET FOREIGN_KEY_CHECKS = 0` (session-scoped) reste actif
/// sur toutes les requêtes TRUNCATE. Sans ça, sqlx peut multiplexer sur
/// des connections distinctes et MariaDB refuse TRUNCATE sur une table
/// référencée (erreur 1701).
///
/// **Inventaire validé** contre `crates/kesh-db/migrations/*.sql` au
/// 2026-04-16. Si une table est ajoutée par une future migration, elle
/// doit être ajoutée ici (le truncate ignorera silencieusement la
/// nouvelle table → polluera les fixtures suivantes).
pub async fn truncate_all(pool: &MySqlPool) -> Result<(), FixtureError> {
    let mut conn = pool.acquire().await?;

    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await?;

    // Ordre : enfants (FK) → parents. invoice_number_sequences avant invoices
    // (FK), invoice_lines avant invoices (FK), etc.
    let tables = [
        "invoice_lines",
        "journal_entry_lines",
        "invoices",
        "invoice_number_sequences",
        "journal_entries",
        "audit_log",
        "company_invoice_settings",
        "bank_accounts",
        "accounts", // FK self-ref via parent_id
        "products",
        "contacts",
        "fiscal_years",
        "refresh_tokens",
        "onboarding_state",
        "users",
        "companies",
    ];
    for table in tables {
        sqlx::query(&format!("TRUNCATE TABLE {table}"))
            .execute(&mut *conn)
            .await?;
    }

    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await?;

    Ok(())
}

/// Insère seulement le user `changeme/changeme` (preset `fresh` — cf. AC #7).
/// Aucune company, aucun account, aucun fiscal_year, aucun onboarding_state.
pub async fn seed_changeme_user_only(pool: &MySqlPool) -> Result<i64, FixtureError> {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active) VALUES (?, ?, 'Admin', TRUE)",
    )
    .bind("changeme")
    .bind(CHANGEME_PASSWORD_HASH)
    .execute(pool)
    .await?;
    Ok(result.last_insert_id() as i64)
}

/// Marque l'`onboarding_state` singleton à `step_completed = 10` (preset
/// `post-onboarding` / `with-company` — cf. AC #8). À appeler APRÈS
/// `seed_accounting_company`.
pub async fn mark_onboarding_complete(pool: &MySqlPool) -> Result<(), FixtureError> {
    // INSERT IGNORE : si la row existe déjà (singleton), on l'ignore puis on
    // UPDATE. Singleton garanti par `uq_onboarding_singleton` UNIQUE constraint.
    sqlx::query(
        "INSERT IGNORE INTO onboarding_state (singleton, step_completed, is_demo, ui_mode) \
         VALUES (TRUE, 10, FALSE, 'guided')",
    )
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE onboarding_state SET step_completed = 10, ui_mode = 'guided' WHERE singleton = TRUE",
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Seede 1 contact + 1 product (preset `with-data` — cf. AC #10). À appeler
/// APRÈS `seed_accounting_company`. **Pas de facture pré-seedée** (cf.
/// décision H3 review pass 3 — `invoices_echeancier.spec.ts` crée ses
/// fixtures dynamiquement).
pub async fn seed_contact_and_product(
    pool: &MySqlPool,
    company_id: i64,
) -> Result<(i64, i64), FixtureError> {
    let contact_result = sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier) \
         VALUES (?, 'Entreprise', 'CI Contact SA', TRUE, FALSE)",
    )
    .bind(company_id)
    .execute(pool)
    .await?;
    let contact_id = contact_result.last_insert_id() as i64;

    let product_result = sqlx::query(
        "INSERT INTO products (company_id, name, unit_price, vat_rate) \
         VALUES (?, 'CI Product', '100.00', '8.10')",
    )
    .bind(company_id)
    .execute(pool)
    .await?;
    let product_id = product_result.last_insert_id() as i64;

    Ok((contact_id, product_id))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste que `seed_accounting_company` produit un état complet et
    /// cohérent. Utilise `sqlx::test` pour DB éphémère.
    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn seed_accounting_company_creates_complete_state(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.expect("seed");

        // Company
        let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(company_count, 1, "1 company expected");

        // Users
        let user_count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'Admin' AND active = TRUE")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(user_count, 2, "2 Admin users expected (admin + changeme)");

        // Fiscal year
        let fy_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM fiscal_years WHERE status = 'Open' AND start_date = '2020-01-01' AND end_date = '2030-12-31'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(fy_count, 1, "1 fiscal_year 2020-2030 Open expected");

        // Accounts
        let account_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(account_count, 5, "5 accounts expected");
        assert_eq!(seeded.accounts.len(), 5);
        assert!(seeded.accounts.contains_key("1100"));
        assert!(seeded.accounts.contains_key("3000"));

        // Company invoice settings : FK cohérence (PK = company_id).
        let cis_receivable: i64 = sqlx::query_scalar(
            "SELECT default_receivable_account_id FROM company_invoice_settings WHERE company_id = ?",
        )
        .bind(seeded.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            cis_receivable, seeded.accounts["1100"],
            "default_receivable_account_id must point to account 1100"
        );

        let cis_revenue: i64 = sqlx::query_scalar(
            "SELECT default_revenue_account_id FROM company_invoice_settings WHERE company_id = ?",
        )
        .bind(seeded.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            cis_revenue, seeded.accounts["3000"],
            "default_revenue_account_id must point to account 3000"
        );
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn truncate_all_clears_all_tables(pool: MySqlPool) {
        // Seed quelque chose, puis truncate, puis vérifie tout vide.
        seed_accounting_company(&pool).await.unwrap();

        truncate_all(&pool).await.unwrap();

        let counts: Vec<(&str, i64)> = vec![
            (
                "companies",
                sqlx::query_scalar("SELECT COUNT(*) FROM companies")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
            ),
            (
                "users",
                sqlx::query_scalar("SELECT COUNT(*) FROM users")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
            ),
            (
                "accounts",
                sqlx::query_scalar("SELECT COUNT(*) FROM accounts")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
            ),
            (
                "fiscal_years",
                sqlx::query_scalar("SELECT COUNT(*) FROM fiscal_years")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
            ),
            (
                "company_invoice_settings",
                sqlx::query_scalar("SELECT COUNT(*) FROM company_invoice_settings")
                    .fetch_one(&pool)
                    .await
                    .unwrap(),
            ),
        ];
        for (table, count) in counts {
            assert_eq!(count, 0, "table {table} should be empty after truncate");
        }
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn seed_changeme_user_only_creates_single_user(pool: MySqlPool) {
        let user_id = seed_changeme_user_only(&pool).await.unwrap();
        assert!(user_id > 0);

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 1);

        let username: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(username, "changeme");
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn mark_onboarding_complete_sets_step_10(pool: MySqlPool) {
        seed_accounting_company(&pool).await.unwrap();
        mark_onboarding_complete(&pool).await.unwrap();

        let step: i32 = sqlx::query_scalar(
            "SELECT step_completed FROM onboarding_state WHERE singleton = TRUE",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(step, 10);
    }

    #[sqlx::test(migrator = "crate::MIGRATOR")]
    async fn seed_contact_and_product_creates_both(pool: MySqlPool) {
        let seeded = seed_accounting_company(&pool).await.unwrap();
        let (contact_id, product_id) = seed_contact_and_product(&pool, seeded.company_id)
            .await
            .unwrap();
        assert!(contact_id > 0);
        assert!(product_id > 0);

        let contact_name: String = sqlx::query_scalar("SELECT name FROM contacts WHERE id = ?")
            .bind(contact_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(contact_name, "CI Contact SA");

        let product_name: String = sqlx::query_scalar("SELECT name FROM products WHERE id = ?")
            .bind(product_id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(product_name, "CI Product");
    }
}
