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

// `#[allow(dead_code)]` : ce fichier garde sa propre `apply_migrations_up_to`
// (qui porte la garde de frontière) et n'emprunte à `common` que
// `migrations_before`, par appel qualifié (Story 26-1, C-26-1-26).
#[allow(dead_code)]
mod common;

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
         `total - 57` (fenêtre d'upgrade, FRONTIÈRE figée à 34) reste \
         cohérent avec l'ajout de migrations futures. Si une migration a été ajoutée à \
         la branche, l'assertion `total == 91` du test upgrade_path_preserves_data \
         doit également échouer, c'est son rôle : elle signale qu'il faut décider \
         explicitement si la fenêtre s'élargit (bumper `total` seul) ou si la \
         frontière doit rester à 34 (bumper `total` ET la fenêtre). Cf. garde-fou \
         P6 de CLAUDE.md.",
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

/// AC #15a — cas générique upgrade path : `total - 57` migrations appliquées
/// (**34** à ce jour) + seed + `MIGRATOR.run()` final, qui applique les **57**
/// dernières. Assertion : seed préservé à travers la fenêtre d'upgrade.
///
/// ⚠️ Les nombres ci-dessus se recomptent, ils ne se relisent pas — cf. le
/// commentaire de la frontière dans le corps de la fonction, qui explique
/// pourquoi la fenêtre ne couvre **pas** la Story 10-2 malgré ce que ce
/// doc-comment a longtemps affirmé (il annonçait « 23 appliquées » et « les 5
/// dernières », trois nombres faux depuis plusieurs Epics ; corrigés ici en
/// revue de code 16-1a passe 5, la passe 2 n'ayant amendé que le commentaire
/// inline, 47 lignes plus bas, en laissant celui-ci le contredire).
#[sqlx::test(migrations = false)]
async fn upgrade_path_preserves_data(pool: MySqlPool) {
    // Total migrations attendues : 26 historiques + _kesh_version (Story 10-2)
    // + companies_is_stub (Story v011-2) + bank_accounts_archived (Story v014-1)
    // + api_keys & audit_log_actor (Story 17-2a)
    // + users_email & password_reset_tokens (Story 17-4a)
    // + vat_rates_crud (Story 11-1) + vat_accounts_config (Story 18-1a)
    // + credit_notes (Story 12-1) + supplier_invoices (Story 12-2) + payment_batches (Story 12-3)
    // + imported_supplier_invoices (Story 12-5b) + projects_analytics (Story 19-1) + supplier_invoices_project (Story 19-3)
    // + invoices_project (Story 19-4) + reconciliation_rules_default_project (Story 19-5) = 43
    // + structured_addresses + person_names + contact_persons (#213) = 46
    // + email_templates (Story 20-1, #224) = 47
    // + contacts_language_salutation + invoices_emailed + companies_email (Story 20-3b1) = 50
    // + contacts_default_payment_terms_days (Story 21-1, #245) = 51.
    // + dunning_config + email_templates_reminder (Story 21-3, #231) = 53.
    // + invoice_reminders (Story 21-5a, #231) = 54.
    // + accounts_role_postable (Story 14-3a, #269) = 55.
    // + invoice_lines_revenue_account (Story 16-1a, #152) = 56.
    // + invoice_lines_revenue_account_backfill (Story 16-1a-bis, #152) = 57.
    // + products_default_revenue_account (Story 16-2a, #144) = 58.
    // + companies_phone_website (Story 16-3a, #151) = 59.
    // + contacts_client_number (Story 16-3b, #151) = 60.
    // + contacts_client_number_canonical (Story 22-1, #294/#295) = 61.
    // + invoice_settlements (Story 24-2, #371) = 62.
    // + invoice_settlements_type (Story 24-3, #372) = 63.
    // + journal_entries_reversal (Story 24-4a, #380) = 64,
    // + companies_books_lock (Story 24-4c, #380) = 65,
    // + closing_accounts_not_postable (Story 24-5, #375) = 66,
    // + audit_log_actor_label (Story 25-1a, #376) = 67,
    // + audit_log_company_id (Story 25-1c-zero, refs #378) = 68,
    // + journal_entry_number_sequences (Story 25-2-c, refs #381) = 69,
    // + invoice_settings_rounding_account (Story 25-4-c3-a1, refs #476) = 70,
    // + invoice_rounding_5_centimes (Story 25-4-c4-a, refs #494) = 71,
    // + invoice_minimum_amount (Story 25-4-e, refs #495) = 72,
    // + invoice_settings_write_off_accounts (Story 25-4-d1, refs #384) = 73,
    // + invoice_settlements_write_off (Story 25-4-d2a, refs #384) = 74,
    // + invoices_frozen_pdf (Story 25-6-b, refs #387) = 75,
    // + journal_entry_lines_lettering (Story 15-1a-i, refs #518) = 76,
    // + lettering_documents_backfill + lettering_reversal_pairs_backfill
    //   (Story 15-1a2-ii, refs #518) = 78,
    // + bank_profiles_debit_sign (Story 26-1, refs #620 #621) = 79,
    // + bank_accounts_description (Story 26-2a, refs #622) = 80,
    // + bank_accounts_closed_on (Story 28-4a, refs #642) = 81,
    // + imported_documents_without_qr (Story 30-2a, refs #685) = 82,
    // + backup_schedule (Story 31-1c, refs #503) = 83.
    // + mcp_keys_and_server_settings (Story 32-5, refs #723) = 84.
    // + mcp_sends (Story 32-8a, refs #723) = 85.
    // + mcp_prompts (Story 32-10a-i, refs #723) = 86.
    // + bank_imports_manual_source (Story v016-2a, refs #816) = 87.
    // + supplier_invoices_draft (Story v017-4a, refs #849) = 88.
    // + installation_settings (Story v0172-session, refs #876, refs #891) = 89.
    // + supplier_invoice_internal_number (Story 33-1a, refs #926) = 90.
    // + bank_accounts_without_iban (Story 34-11a1, refs #972) = 91.
    let total = kesh_db::MIGRATOR.migrations.len();
    assert_eq!(
        total, 91,
        "91 migrations attendues (90 précédentes + Story 34-11a1 : bank_accounts_without_iban)"
    );

    // Étape 1 : applique toutes les migrations sauf les 57 dernières. La
    // fenêtre d'upgrade démarre donc à la 35ᵉ, `20260614000001_vat_accounts_config`
    // (Story 18-1a), et court jusqu'à la dernière du dépôt. Ne pas ré-énumérer
    // ici les migrations de la fenêtre : une liste nominative se périme à chaque
    // ajout sans que rien ne le signale — c'est précisément la dérive constatée
    // plus bas. Le seul nombre qui fait foi est celui du `total - N` ci-dessous.
    //
    // Note `total - N` : expression relative à la longueur totale, tandis que
    // l'assertion `total == 91` ci-dessus est INTENTIONNELLEMENT codée en dur
    // pour fail-loud sur toute évolution non revue. À chaque migration ajoutée,
    // le mainteneur doit (1) bumper ce compte (2) incrémenter `N` du même pas,
    // de sorte que `total - N` — la frontière — reste **constant**.
    //
    // Frontière actuelle : **34**. Le test applique donc les 34 premières
    // migrations (jusqu'à `20260613000001_vat_rates_crud` incluse), seede des
    // données, puis joue les 57 restantes comme « fenêtre d’upgrade ».
    //
    // ⚠️ `N` DOIT être incrémenté en même temps que `total`. Le laisser à 56
    // avec `total = 91` porterait la frontière à 35 : le test continuerait de
    // passer en testant une fenêtre plus étroite d'une migration.
    // Story 16-1a : 21 → 22, frontière inchangée (56 - 22 = 55 - 21 = 34).
    // Story 16-1a-bis : 22 → 23, frontière inchangée (57 - 23 = 34).
    // Story 16-2a : 23 → 24, frontière inchangée (58 - 24 = 34).
    // Story 16-3a : 24 → 25, frontière inchangée (59 - 25 = 34).
    // Story 16-3b : 25 → 26, frontière inchangée (60 - 26 = 34).
    // Story 22-1  : 26 → 27, frontière inchangée (61 - 27 = 34).
    // Story 24-2  : 27 → 28, frontière inchangée (62 - 28 = 34).
    // Story 24-3  : 28 → 29, frontière inchangée (63 - 29 = 34).
    // Story 24-4a : 29 → 30, frontière inchangée (64 - 30 = 34).
    // Story 24-4c : 30 → 31, frontière inchangée (65 - 31 = 34).
    // Story 24-5  : 31 → 32, frontière inchangée (66 - 32 = 34).
    // Story 25-1a : 32 → 33, frontière inchangée (67 - 33 = 34).
    // Story 25-1c-zero : 33 → 34, frontière inchangée (68 - 34 = 34).
    //
    // ⚠️ DÉRIVE DOCUMENTAIRE CONSTATÉE (revue de code 16-1a). Ce commentaire
    // affirmait simuler « l'état pré-Story-10-2 » et parlait d'une « frontière
    // de 23 migrations historiques », avec une assertion citée à `total == 39`.
    // Ces trois nombres sont faux et le sont
    // depuis plusieurs Epics : la migration de la Story 10-2
    // (`20260522000001_kesh_version`) est la **27ᵉ**, or la frontière est à 34
    // — la fenêtre d'upgrade ne couvre donc PAS la Story 10-2, elle démarre à
    // `20260614000001_vat_accounts_config` (Story 18-1a). Le test reste valide
    // (il exerce bien un upgrade sur 23 migrations avec préservation des
    // données), mais il n'exerce pas la fenêtre que son commentaire décrivait.
    // Restaurer l'intention d'origine supposerait de ramener la frontière à 26
    // — décision de périmètre, hors revue de la 16-1a.
    //
    // Chronologie des correctifs, parce que la dérive a survécu à ses propres
    // corrections : passe 2 rend CE commentaire factuel ; passe 5 corrige le
    // doc-comment de la fonction, resté 47 lignes plus haut à « 23 appliquées /
    // 5 dernières » ; passe 6 corrige le message d'assertion de
    // `apply_migrations_up_to`, resté à `total - 8` / `total == 39`. Trois sites
    // du même symptôme dans le même fichier, découverts un par passe. Ce qui
    // reste ouvert n'est plus documentaire, c'est la décision de périmètre
    // ci-dessus.
    // ⚠️ `- 34` et non `- 33` depuis la Story 25-1c-zero (même geste qu'en 24-2,
    // 24-3, 24-4a, 24-4c, 24-5 et 25-1a) : c'est la FRONTIÈRE (34) qui
    // est l'invariant voulu, pas la taille de la fenêtre. Garder `- 33` aurait
    // déplacé le point de départ à la 35ᵉ migration et changé le chemin
    // d'upgrade réellement testé — sans que rien ne le signale. La fenêtre
    // s'élargit donc d'un cran à chaque migration ajoutée, ce qui est le sens
    // voulu : « depuis un socle figé, jusqu'à la dernière du dépôt ».
    // Story 25-2-c : 34 → 35, frontière inchangée (69 - 35 = 34).
    // Story 25-4-c3-a1 : 35 → 36, frontière inchangée (70 - 36 = 34).
    // Story 25-4-c4-a : 36 → 37, frontière inchangée (71 - 37 = 34).
    // Story 25-4-e : 37 → 38, frontière inchangée (72 - 38 = 34).
    // Story 25-4-d1 : 38 → 39, frontière inchangée (73 - 39 = 34).
    // Story 25-4-d2a : 39 → 40, frontière inchangée (74 - 40 = 34).
    // Story 25-6-b (fusion avec la chaîne 25-4) : → 41, frontière inchangée (75 - 41 = 34).
    // Story 15-1a-i : 41 → 42, frontière inchangée (76 - 42 = 34).
    // Story 15-1a2-ii : 42 → 44, frontière inchangée (78 - 44 = 34).
    // Story 26-1 : 44 → 45, frontière inchangée (79 - 45 = 34).
    // Story 26-2a : 45 → 46, frontière inchangée (80 - 46 = 34).
    // Story 28-4a : 46 → 47, frontière inchangée (81 - 47 = 34).
    // Story 30-2a : 47 → 48, frontière inchangée (82 - 48 = 34).
    // Story 31-1c : 48 → 49, frontière inchangée (83 - 49 = 34).
    // Story 32-5 : 49 → 50, frontière inchangée (84 - 50 = 34).
    // Story 32-8a : 50 → 51, frontière inchangée (85 - 51 = 34).
    // Story 32-10a-i : 51 → 52, frontière inchangée (86 - 52 = 34).
    // Story v016-2a : 52 → 53, frontière inchangée (87 - 53 = 34).
    // Story v017-4a : 53 → 54, frontière inchangée (88 - 54 = 34).
    // Story v0172-session : 54 → 55, frontière inchangée (89 - 55 = 34).
    // Story 33-1a : 55 → 56, frontière inchangée (90 - 56 = 34).
    // Story 34-11a1 : 56 → 57, frontière inchangée (91 - 57 = 34).
    //
    // ⚠️ J'ai d'abord porté `total` à 69 SANS toucher à ce soustracteur : le socle
    // est alors passé de 34 à 35 migrations, la fenêtre a glissé d'un cran, et le
    // test a rougi non plus sur son compteur mais sur `COUNT(accounts) : expected
    // 4, got 2` — un échec qui ne ressemble en rien à sa cause. *Les deux nombres
    // se bougent du même pas, et le commentaire ci-dessus le disait.*
    let n_before_upgrade_window = total - 57;
    apply_migrations_up_to(&pool, n_before_upgrade_window)
        .await
        .expect("apply_migrations_up_to(total - 57) failed");

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

    // Étape 3 : appliquer les 57 migrations restantes via MIGRATOR.run().
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

    // Étape 5-bis (AC-G Story 14-3a) : le backfill de rôle a tourné sur les
    // comptes INSÉRÉS par une migration antérieure (1171 Impôt préalable et 2206
    // Décompte TVA, posés par 20260614000001_vat_accounts_config), pas seulement
    // sur ceux du seed manuel du fixture. C'est le seul cas que le test dédié
    // `accounts_role_backfill.rs` (montage sur charts JSON) ne couvre pas.
    let role_1171: Option<String> =
        sqlx::query_scalar("SELECT role FROM accounts WHERE company_id = ? AND number = '1171'")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .expect("compte 1171 introuvable après upgrade");
    assert_eq!(
        role_1171.as_deref(),
        Some("VatRecoverable"),
        "le backfill 14-3a doit taguer 1171 (inséré par vat_accounts_config)"
    );
    let role_2206: Option<String> =
        sqlx::query_scalar("SELECT role FROM accounts WHERE company_id = ? AND number = '2206'")
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .expect("compte 2206 introuvable après upgrade");
    assert_eq!(
        role_2206.as_deref(),
        Some("VatSettlement"),
        "le backfill 14-3a doit taguer 2206 (inséré par vat_accounts_config)"
    );

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
    // min_required = '0.17.0' depuis le bump breaking de la Story v016-2a
    // (bank_imports_manual_source.sql — 6e bump du repo, après '0.7.0' en
    // 21-3, '0.10.0' en 22-1, '0.13.0' en 15-1a-i, '0.13.1' en 26-1 et
    // '0.14.0' en 30-2a). Binary == db_min → Aligned.
    // Story 34-11a1 (#972) : 7e bump, '0.17.6' (compte sans IBAN).
    let result = check_downgrade_protection(&pool, "0.17.6").await;
    assert_eq!(
        result.unwrap(),
        DowngradeCheckOutcome::Aligned,
        "binary 0.17.6 == db_min 0.17.6 → Aligned"
    );
}

/// Story 34-11a1 (#972, AC-12) : un binaire 0.17.5 est **refusé** — il lit
/// `bank_accounts.iban` en `String`, et un seul compte sans IBAN ferait échouer
/// toute liste de comptes ; il ignorerait `account_kind` à l'import.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_refuses_0_17_5(pool: MySqlPool) {
    match check_downgrade_protection(&pool, "0.17.5").await {
        Err(VersionError::DowngradeRefused { db_min, binary }) => {
            assert_eq!(db_min.to_string(), "0.17.6");
            assert_eq!(binary.to_string(), "0.17.5");
        }
        other => panic!("Expected DowngradeRefused for 0.17.5, got {other:?}"),
    }
}

/// Story v016-2a (#816, AC-2) : un binaire 0.16.1 — la candidate
/// `v0.16.1-rc1` en porte la version Cargo, et c'est la dernière version
/// antérieure — est **refusé** : il ne décode pas `source_format = 'MANUAL'`
/// d'un relevé saisi à la main (`BankImportSourceFormat::from_str`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_refuses_0_16_1(pool: MySqlPool) {
    match check_downgrade_protection(&pool, "0.16.1").await {
        Err(VersionError::DowngradeRefused { db_min, binary }) => {
            // Story 34-11a1 : le seuil est passé à 0.17.6 ; 0.16.1 reste refusé.
            assert_eq!(db_min.to_string(), "0.17.6");
            assert_eq!(binary.to_string(), "0.16.1");
        }
        other => panic!("Expected DowngradeRefused for 0.16.1, got {other:?}"),
    }
}

/// Story 30-2a (#685) : un binaire 0.13.4 est désormais **refusé** — il lit
/// les sept colonnes QR de `imported_supplier_invoices` dans des types non
/// optionnels, et une seule ligne sans QR ferait échouer la liste « à
/// compléter » et l'export CSV.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_refuses_0_13_4(pool: MySqlPool) {
    match check_downgrade_protection(&pool, "0.13.4").await {
        Err(VersionError::DowngradeRefused { db_min, binary }) => {
            // Story v016-2a : le seuil est passé à 0.17.0 ; 0.13.4 reste refusé.
            // Story 34-11a1 : puis à 0.17.6.
            assert_eq!(db_min.to_string(), "0.17.6");
            assert_eq!(binary.to_string(), "0.13.4");
        }
        other => panic!("Expected DowngradeRefused for 0.13.4, got {other:?}"),
    }
}

/// Test additionnel : `check_downgrade_protection` retourne `BinaryAhead`
/// quand binary > db_min (upgrade legitime).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn downgrade_protection_binary_ahead_when_binary_greater(pool: MySqlPool) {
    // min_required = '0.17.0' depuis le bump breaking de la Story v016-2a,
    // binary 0.18.0 > 0.17.0 → BinaryAhead (upgrade légitime). *(Le binaire
    // était 0.15.0 jusqu'à la v016-2a : sous le seuil 0.17.0, il serait
    // devenu un refus — le binaire passe au-delà du seuil, la sémantique du
    // test reste « binaire en avance ».)*
    let result = check_downgrade_protection(&pool, "0.18.0").await;
    match result.unwrap() {
        DowngradeCheckOutcome::BinaryAhead { db_min, binary } => {
            // Story 34-11a1 : seuil 0.17.6, 0.18.0 reste en avance.
            assert_eq!(db_min.to_string(), "0.17.6");
            assert_eq!(binary.to_string(), "0.18.0");
        }
        other => panic!("Expected BinaryAhead, got {:?}", other),
    }
}

/// Story 26-1, AC9 — **le profil d'avant reste ce qu'il était**. Un profil
/// en mode débit/crédit séparés inséré AVANT la migration
/// `20261011000001_bank_profiles_debit_sign` prend `debit_sign = 'positive'`
/// (son comportement d'avant : débits sans signe, inversés par Kesh) et garde
/// son `header_row_count` et son `column_mapping`.
///
/// Index résolu **par version** (`common::migrations_before`, P6) ;
/// assertion de montage : la colonne n'existe pas avant (sans elle, le test
/// passerait à vide — précédent 16-1a).
#[sqlx::test(migrations = false)]
async fn bank_profiles_debit_sign_defaults_existing_profiles(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(20261011000001, "bank_profiles_debit_sign"),
    )
    .await
    .expect("apply_migrations_up_to(avant bank_profiles_debit_sign) failed");

    let before: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'bank_profiles' \
         AND COLUMN_NAME = 'debit_sign'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        before, 0,
        "montage : debit_sign ne doit pas exister avant la migration"
    );

    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société AC9', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("INSERT company failed");
    let mapping = r#"{"date":0,"debit_credit_split":[1,2],"reference":3}"#;
    sqlx::query(
        "INSERT INTO bank_profiles (company_id, bank_name, column_mapping, date_format, \
         decimal_separator, field_separator, header_row_count) \
         VALUES (?, 'Banque AC9', ?, '%d.%m.%Y', '.', ';', 5)",
    )
    .bind(company_id)
    .bind(mapping)
    .execute(&pool)
    .await
    .expect("INSERT bank_profile failed");

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("MIGRATOR.run() final failed");

    let (debit_sign, header_rows, column_mapping): (String, u8, String) = sqlx::query_as(
        "SELECT debit_sign, header_row_count, CAST(column_mapping AS CHAR) FROM bank_profiles \
         WHERE company_id = ?",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(debit_sign, "positive");
    assert_eq!(header_rows, 5);
    let got: serde_json::Value = serde_json::from_str(&column_mapping).unwrap();
    let want: serde_json::Value = serde_json::from_str(mapping).unwrap();
    assert_eq!(got, want, "column_mapping intact");

    let min_required: String =
        sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    // Toutes les migrations ont tourné : la valeur est celle du DERNIER bump
    // (Story v016-2a), non plus celui de cette migration-ci (0.13.1) — puis
    // celui de la Story 34-11a1 (0.17.6).
    assert_eq!(min_required, "0.17.6");
}

/// Story 30-2a (#685, AC4) — la migration `imported_documents_without_qr`
/// passe sur une base **peuplée** : une ligne `completed` liée à sa facture, une
/// `to_complete` et une `discarded` sont conservées telles quelles, le lien est
/// intact, et l'index de la clé étrangère est remplacé par l'unicité sans
/// erreur 1553 (le squash de test ne voit pas ce défaut, d'où ce test sur le
/// vrai `MIGRATOR`).
///
/// Index résolu **par version** (P6) ; assertion de montage : `creditor_iban`
/// est `NOT NULL` avant la migration (sans elle, le test passerait à vide).
#[sqlx::test(migrations = false)]
async fn imported_documents_without_qr_preserves_existing_rows(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(20261015000001, "imported_documents_without_qr"),
    )
    .await
    .expect("apply_migrations_up_to(avant imported_documents_without_qr) failed");

    let nullable_before: String = sqlx::query_scalar(
        "SELECT IS_NULLABLE FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'imported_supplier_invoices' AND COLUMN_NAME = 'creditor_iban'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        nullable_before, "NO",
        "montage : creditor_iban doit être NOT NULL avant la migration"
    );

    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société 30-2a', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("INSERT company failed");
    // Connexion dédiée : la facture 4242 n'existe pas, la clé étrangère est
    // levée le temps des INSERT — c'est la conservation du lien qui est sous test.
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    for (status, link, hash) in [
        ("completed", Some(4242_i64), "h-completed"),
        ("to_complete", None, "h-to-complete"),
        ("discarded", None, "h-discarded"),
    ] {
        sqlx::query(
            "INSERT INTO imported_supplier_invoices (company_id, status, supplier_invoice_id, \
             file_hash, storage_path, original_filename, mime_type, byte_size, creditor_iban, \
             is_qr_iban, creditor_address_type, creditor_name, creditor_country, reference_type, \
             amount, currency) VALUES (?, ?, ?, ?, 'x.png', 'x.png', 'image/png', 10, \
             'CH9300762011623852957', 0, 'K', 'Fournisseur SA', 'CH', 'NON', 12.5, 'CHF')",
        )
        .bind(company_id)
        .bind(status)
        .bind(link)
        .bind(hash)
        .execute(&mut *conn)
        .await
        .expect("INSERT staging failed");
    }
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("MIGRATOR.run() final failed — la migration doit passer sur une base peuplée");

    let rows: Vec<(String, Option<i64>, Option<String>)> = sqlx::query_as(
        "SELECT status, supplier_invoice_id, creditor_iban FROM imported_supplier_invoices \
         WHERE company_id = ? ORDER BY id",
    )
    .bind(company_id)
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rows,
        vec![
            (
                "completed".to_string(),
                Some(4242),
                Some("CH9300762011623852957".to_string())
            ),
            (
                "to_complete".to_string(),
                None,
                Some("CH9300762011623852957".to_string())
            ),
            (
                "discarded".to_string(),
                None,
                Some("CH9300762011623852957".to_string())
            ),
        ]
    );
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT INDEX_NAME FROM information_schema.STATISTICS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = 'imported_supplier_invoices' \
         AND COLUMN_NAME = 'supplier_invoice_id'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(indexes, vec!["uq_imported_supplier_invoice".to_string()]);
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

// ---------------------------------------------------------------------------
// Story v016-2a (#816) — la montée depuis une base 0.16.x peuplée (AC-2-bis),
// l'inventaire des instructions de la migration (AC-2-bis) et celui des
// tables (AC-3-bis). Contrainte du Project Lead du 2026-10-30 (C-v016-2-16) :
// les données de l'installation `v0.16.1-rc1` se gardent.
// ---------------------------------------------------------------------------

const MANUAL_SOURCE_VERSION: i64 = 20261030000001;
const MANUAL_SOURCE_LABEL: &str = "bank_imports_manual_source";

/// Chaque ligne d'une table, colonne par colonne (toutes les colonnes de
/// `information_schema`, `NULL` rendu `∅`), triée par `id` : la comparaison
/// avant/après ne choisit pas les colonnes qu'elle regarde.
async fn table_snapshot(pool: &MySqlPool, table: &str) -> Vec<String> {
    let columns: Vec<String> = sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap();
    assert!(!columns.is_empty(), "table {table} introuvable");
    let parts: Vec<String> = columns
        .iter()
        .map(|c| format!("'{c}=', IFNULL(CAST(`{c}` AS CHAR), '∅')"))
        .collect();
    sqlx::query_scalar(&format!(
        "SELECT CONCAT_WS('|', {}) FROM `{table}` ORDER BY id",
        parts.join(", ")
    ))
    .fetch_all(pool)
    .await
    .unwrap()
}

async fn application_tables(pool: &MySqlPool) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT TABLE_NAME FROM information_schema.TABLES \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_TYPE = 'BASE TABLE' \
           AND TABLE_NAME <> '_sqlx_migrations' ORDER BY TABLE_NAME",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// AC-2-bis — une base à l'état 0.16.x, **peuplée** d'un import CAMT à soldes
/// et à `statement_id`, d'un import CSV sans solde, et de leurs transactions
/// (une `pending`, une `reconciled` liée à une écriture, une rejetée), monte en
/// 0.17.0 sans qu'aucune ligne ne change, et se relit par le code applicatif.
///
/// Index résolu **par version** (`common::migrations_before`, P6) ; assertion
/// de montage : `min_required` vaut `0.14.0` avant la migration et la base
/// porte bien les lignes semées (sans elles, le test passerait à vide).
#[sqlx::test(migrations = false)]
async fn bank_imports_manual_source_preserves_existing_rows(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(MANUAL_SOURCE_VERSION, MANUAL_SOURCE_LABEL),
    )
    .await
    .expect("apply_migrations_up_to(avant bank_imports_manual_source) failed");

    let (min_before, last_applied_before): (String, String) = sqlx::query_as(
        "SELECT kesh_version_min_required, kesh_version_last_applied FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(min_before, "0.14.0", "montage : état 0.16.x");

    // --- Semis, en SQL brut (le schéma de la 0.16.x) ---
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société v016-2a', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) \
         VALUES ('v0162a', ?, 'Comptable', ?) RETURNING id",
    )
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo")
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let bank_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary) \
         VALUES (?, 'Banque Exemple', 'CH9300762011623852957', TRUE) RETURNING id",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let fy_id: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut account_ids = Vec::new();
    for (number, name, kind) in [("1020", "Banque", "Asset"), ("3200", "Ventes", "Revenue")] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(company_id)
        .bind(number)
        .bind(name)
        .bind(kind)
        .fetch_one(&pool)
        .await
        .unwrap();
        account_ids.push(id);
    }
    let entry_id: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, 1, '2026-09-15', 'Banque', 'Encaissement') \
         RETURNING id",
    )
    .bind(company_id)
    .bind(fy_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    for (order, account, debit, credit) in [
        (1, account_ids[0], "50.00", "0"),
        (2, account_ids[1], "0", "50.00"),
    ] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(account)
        .bind(order)
        .bind(debit)
        .bind(credit)
        .execute(&pool)
        .await
        .unwrap();
    }
    let camt_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, statement_id, period_from, period_to, opening_balance, closing_balance, \
         transaction_count, imported_by_user_id) VALUES (?, ?, 'releve-09.xml', ?, \
         'CAMT053_V04', 'STMT-2026-09', '2026-09-01', '2026-09-30', 1000.00, 729.75, 3, ?) \
         RETURNING id",
    )
    .bind(company_id)
    .bind(bank_id)
    .bind("a".repeat(64))
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let csv_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, transaction_count, imported_by_user_id) \
         VALUES (?, ?, 'releve-10.csv', ?, 'CSV', '2026-10-01', '2026-10-31', 1, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(bank_id)
    .bind("b".repeat(64))
    .bind(user_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    // CAMT : une pending (avec date de valeur, référence, contrepartie), une
    // reconciled liée à l'écriture, une rejetée ; CSV : une pending.
    sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         value_date, amount, currency, reference, details, end_to_end_id, transaction_id, \
         counterparty_iban, counterparty_name, status, matched_entry_id, auto_match_rejected_at) \
         VALUES \
         (?, ?, ?, '2026-09-03', '2026-09-04', -320.25, 'CHF', 'RF18539007547034', \
          'Prélèvement assurance', 'E2E-1', 'TX-1', 'CH9300762011623852957', 'Assurance Exemple SA', \
          'pending', NULL, NULL), \
         (?, ?, ?, '2026-09-15', NULL, 50.00, 'CHF', 'F-2026-0042', 'Virement reçu', NULL, NULL, \
          NULL, NULL, 'reconciled', ?, NULL), \
         (?, ?, ?, '2026-09-20', NULL, -4.80, 'CHF', NULL, 'Frais', NULL, NULL, NULL, NULL, \
          'pending', NULL, '2026-10-01 08:00:00'), \
         (?, ?, ?, '2026-10-05', NULL, 12.50, 'CHF', NULL, 'Intérêts', NULL, NULL, NULL, NULL, \
          'pending', NULL, NULL)",
    )
    .bind(company_id)
    .bind(camt_id)
    .bind(bank_id)
    .bind(company_id)
    .bind(camt_id)
    .bind(bank_id)
    .bind(entry_id)
    .bind(company_id)
    .bind(camt_id)
    .bind(bank_id)
    .bind(company_id)
    .bind(csv_id)
    .bind(bank_id)
    .execute(&pool)
    .await
    .expect("INSERT bank_transactions failed");

    let watched = [
        "bank_imports",
        "bank_transactions",
        "journal_entries",
        "journal_entry_lines",
    ];
    let mut before = Vec::new();
    for t in watched {
        before.push(table_snapshot(&pool, t).await);
    }
    assert_eq!(before[0].len(), 2, "montage : deux imports semés");
    assert_eq!(before[1].len(), 4, "montage : quatre transactions semées");

    // --- La montée ---
    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("MIGRATOR.run() final failed — la montée doit passer sur une base peuplée");

    // 1. et 3. — chaque ligne se relit colonne par colonne à l'identique,
    // écriture liée et ses lignes comprises.
    for (t, b) in watched.iter().zip(&before) {
        assert_eq!(&table_snapshot(&pool, t).await, b, "table {t} modifiée");
    }

    // 2. — relecture par le code applicatif (décodage `from_str` compris).
    use kesh_db::entities::BankImportSourceFormat;
    use kesh_db::repositories::{bank_imports, bank_transactions};
    let camt = bank_imports::find_by_id_for_company(&pool, company_id, camt_id)
        .await
        .expect("relecture CAMT")
        .expect("import CAMT présent");
    assert_eq!(camt.source_format, BankImportSourceFormat::Camt053V04);
    assert_eq!(camt.statement_id.as_deref(), Some("STMT-2026-09"));
    assert_eq!(
        camt.closing_balance.map(|d| d.to_string()),
        Some("729.75".into())
    );
    let csv = bank_imports::find_by_id_for_company(&pool, company_id, csv_id)
        .await
        .expect("relecture CSV")
        .expect("import CSV présent");
    assert_eq!(csv.source_format, BankImportSourceFormat::Csv);
    assert_eq!(csv.closing_balance, None);
    let camt_txs = bank_transactions::list_by_import(&pool, company_id, camt_id)
        .await
        .expect("relecture des transactions CAMT");
    assert_eq!(camt_txs.len(), 3);
    assert_eq!(camt_txs[0].status.as_str(), "pending");
    assert_eq!(
        camt_txs[0].counterparty_name.as_deref(),
        Some("Assurance Exemple SA")
    );
    assert_eq!(camt_txs[1].status.as_str(), "reconciled");
    assert_eq!(camt_txs[1].matched_entry_id, Some(entry_id));
    assert!(camt_txs[2].auto_match_rejected_at.is_some());
    let csv_txs = bank_transactions::list_by_import(&pool, company_id, csv_id)
        .await
        .expect("relecture des transactions CSV");
    assert_eq!(csv_txs.len(), 1);

    // 4. — le seuil relevé, la version du dernier démarrage inchangée.
    let (min_after, last_applied_after): (String, String) = sqlx::query_as(
        "SELECT kesh_version_min_required, kesh_version_last_applied FROM _kesh_version WHERE id = 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    // Toutes les migrations ont tourné : le dernier bump (Story 34-11a1).
    assert_eq!(min_after, "0.17.6");
    assert_eq!(last_applied_after, last_applied_before);
}

/// AC-2-bis — inventaire décidable : la migration ne porte **qu'une**
/// instruction, le relèvement de `_kesh_version`. Le fichier est lu,
/// décommenté (`--` en fin de ligne et `/* … */`), découpé aux `;`.
#[test]
fn bank_imports_manual_source_only_raises_min_required() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("migrations/20261030000001_bank_imports_manual_source.sql");
    let raw = std::fs::read_to_string(&path).expect("migration lisible");
    let mut uncommented = String::new();
    let mut rest = raw.as_str();
    while let Some(i) = rest.find("/*") {
        uncommented.push_str(&rest[..i]);
        rest = rest[i..].split_once("*/").map(|(_, r)| r).unwrap_or("");
    }
    uncommented.push_str(rest);
    let code: String = uncommented
        .lines()
        .map(|l| l.split("--").next().unwrap_or(""))
        .collect::<Vec<_>>()
        .join("\n");
    let statements: Vec<String> = code
        .split(';')
        .map(|s| s.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|s| !s.is_empty())
        .collect();
    assert_eq!(
        statements,
        vec![
            "UPDATE _kesh_version SET kesh_version_min_required = '0.17.0' WHERE id = 1"
                .to_string()
        ],
        "la migration ne doit porter que le relèvement de _kesh_version"
    );
}

/// AC-3-bis — la 0.17.0 n'ajoute **aucune table** : l'inventaire des tables
/// après le `MIGRATOR` complet égale celui de l'état d'avant la migration.
/// Une sauvegarde `.keshbackup` de la 0.16.x a donc le même inventaire que
/// la 0.17.0 (condition de `parse_and_verify`).
#[sqlx::test(migrations = false)]
async fn bank_imports_manual_source_adds_no_table(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(MANUAL_SOURCE_VERSION, MANUAL_SOURCE_LABEL),
    )
    .await
    .unwrap();
    let before = application_tables(&pool).await;
    assert!(
        before.iter().any(|t| t == "bank_imports"),
        "montage : la base 0.16.x porte bank_imports"
    );
    // Story v0172-session : jusqu'à cette migration incluse, et non plus le
    // `MIGRATOR` complet — la 0.17.3 ajoute une table SYSTÈME
    // (`_kesh_installation_settings`), contrôlée par son propre test
    // (`installation_settings_adds_only_its_system_table`).
    apply_migrations_up_to(
        &pool,
        common::migrations_before(MANUAL_SOURCE_VERSION, MANUAL_SOURCE_LABEL) + 1,
    )
    .await
    .unwrap();
    assert_eq!(application_tables(&pool).await, before);
}

// ---------------------------------------------------------------------------
// Story v017-4a (#849, AC-2) — `20261031000001_supplier_invoices_draft` : la
// montée sans perte d'une base peuplée, depuis la 0.16.x et depuis la rc1.
// ---------------------------------------------------------------------------

const DRAFT_VERSION: i64 = 20261031000001;
const DRAFT_LABEL: &str = "supplier_invoices_draft";

/// Les colonnes d'une table, dans l'ordre du schéma.
async fn columns_of(pool: &MySqlPool, table: &str) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT COLUMN_NAME FROM information_schema.COLUMNS \
         WHERE TABLE_SCHEMA = DATABASE() AND TABLE_NAME = ? ORDER BY ORDINAL_POSITION",
    )
    .bind(table)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// [`table_snapshot`] sur une liste de colonnes **fixée avant la montée** : une
/// colonne ajoutée par la migration (`link_origin`) n'entre pas dans la
/// comparaison, toutes les autres, si.
async fn table_snapshot_on(pool: &MySqlPool, table: &str, columns: &[String]) -> Vec<String> {
    let parts: Vec<String> = columns
        .iter()
        .map(|c| format!("'{c}=', IFNULL(CAST(`{c}` AS CHAR), '∅')"))
        .collect();
    sqlx::query_scalar(&format!(
        "SELECT CONCAT_WS('|', {}) FROM `{table}` ORDER BY id",
        parts.join(", ")
    ))
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Le décor d'une base de la 0.16.x ou de la rc1, semé par le SQL et par les
/// fonctions du dépôt dont le chemin ne touche aucune colonne neuve (paiement,
/// annulation, lots) — comme les a écrits le binaire de ces versions.
struct DecorMontee {
    company_id: i64,
    user_id: i64,
    fy_id: i64,
    expense: i64,
    open_id: i64,
    open_vat_id: i64,
    paid_id: i64,
    cancelled_id: i64,
    paid_cancelled_id: i64,
    completed_id: i64,
    attached_id: i64,
    batch_id: i64,
    completed_doc: i64,
    attached_doc: i64,
}

/// Une facture **ouverte** telle que la créait la rc1 : son écriture d'achat
/// (journal Achats), puis la ligne `open`, en une transaction.
#[allow(clippy::too_many_arguments)]
async fn facture_ouverte_rc1(
    pool: &MySqlPool,
    d: &DecorMontee,
    contact: i64,
    payable: i64,
    vat_account: i64,
    numero: &str,
    ht: rust_decimal::Decimal,
    taux: rust_decimal::Decimal,
) -> i64 {
    use kesh_db::entities::{Journal, NewJournalEntry, NewJournalEntryLine};
    use rust_decimal::Decimal;
    let tva = kesh_core::accounting::line_vat_amount(ht, taux);
    let ttc = ht + tva;
    let mut lines = vec![NewJournalEntryLine {
        account_id: d.expense,
        debit: ht,
        credit: Decimal::ZERO,
        project_id: None,
    }];
    if tva > Decimal::ZERO {
        lines.push(NewJournalEntryLine {
            account_id: vat_account,
            debit: tva,
            credit: Decimal::ZERO,
            project_id: None,
        });
    }
    lines.push(NewJournalEntryLine {
        account_id: payable,
        debit: Decimal::ZERO,
        credit: ttc,
        project_id: None,
    });
    let mut tx = pool.begin().await.unwrap();
    let je = kesh_db::repositories::journal_entries::create_in_tx(
        &mut tx,
        d.fy_id,
        d.user_id,
        NewJournalEntry {
            company_id: d.company_id,
            entry_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
            journal: Journal::Achats,
            description: format!("Facture fournisseur {numero} - Fournisseur montée SA"),
            project_id: None,
            lines,
        },
        false,
    )
    .await
    .expect("écriture d'achat de la rc1");
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, due_date, total_amount, creditor_iban, purchase_journal_entry_id) \
         VALUES (?, ?, ?, 'open', '2026-06-15', '2026-07-15', ?, 'CH5604835012345678009', ?) \
         RETURNING id",
    )
    .bind(d.company_id)
    .bind(contact)
    .bind(numero)
    .bind(ttc)
    .bind(je.entry.id)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO supplier_invoice_lines (supplier_invoice_id, position, description, \
         quantity, unit_price, vat_rate, line_total, expense_account_id) \
         VALUES (?, 0, 'Fourniture', 1, ?, ?, ?, ?)",
    )
    .bind(id)
    .bind(ht)
    .bind(taux)
    .bind(ht)
    .bind(d.expense)
    .execute(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

/// Monte le schéma jusqu'avant `frontiere`, sème le décor, applique le reste, et
/// vérifie les cinq points de l'AC-2.
async fn draft_migration_preserves_a_populated_base(pool: MySqlPool, frontiere: (i64, &str)) {
    use kesh_db::entities::{NewPaymentBatch, SettlementChoice};
    use kesh_db::repositories::{payment_batches, supplier_invoices};
    use rust_decimal_macros::dec;

    apply_migrations_up_to(&pool, common::migrations_before(frontiere.0, frontiere.1))
        .await
        .expect("montage du schéma d'avant");
    // Story 33-1a : le paiement, l'annulation et les lots du semis, passés par le dépôt
    // courant, lisent deux colonnes de la migration 33-1a. Elles sont posées le temps du
    // semis, et retirées avant la montée : `MIGRATOR.run()` les ajoute lui-même.
    sqlx::query("ALTER TABLE supplier_invoices ADD COLUMN internal_number VARCHAR(64) NULL")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "ALTER TABLE company_invoice_settings ADD COLUMN supplier_internal_number_format \
         VARCHAR(64) NOT NULL DEFAULT 'FF-{YEAR}-{SEQ:03}'",
    )
    .execute(&pool)
    .await
    .unwrap();
    let entry_nullable: String = sqlx::query_scalar(
        "SELECT IS_NULLABLE FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'supplier_invoices' AND COLUMN_NAME = 'purchase_journal_entry_id'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        entry_nullable, "NO",
        "montage : schéma d'avant la migration"
    );

    // --- Semis (SQL, fixtures synthétiques) ---
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Société v017-4a', 'Rue 1', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) \
         VALUES ('v0174a', ?, 'Comptable', ?) RETURNING id",
    )
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo")
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let fy_id: i64 = sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date) \
         VALUES (?, 'Exercice 2026', '2026-01-01', '2026-12-31') RETURNING id",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let mut acc = std::collections::HashMap::new();
    for (number, name, kind) in [
        ("1020", "Banque", "Asset"),
        ("1170", "TVA récupérable", "Asset"),
        ("2000", "Créanciers", "Liability"),
        ("4000", "Achats", "Expense"),
    ] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(company_id)
        .bind(number)
        .bind(name)
        .bind(kind)
        .fetch_one(&pool)
        .await
        .unwrap();
        acc.insert(number, id);
    }
    sqlx::query(
        "INSERT INTO company_invoice_settings (company_id, default_payable_account_id, \
         default_vat_recoverable_account_id) VALUES (?, ?, ?)",
    )
    .bind(company_id)
    .bind(acc["2000"])
    .bind(acc["1170"])
    .execute(&pool)
    .await
    .unwrap();
    let contact: i64 = sqlx::query_scalar(
        "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
         VALUES (?, 'Entreprise', 'Fournisseur montée SA', TRUE) RETURNING id",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let bank_id: i64 = sqlx::query_scalar(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, is_primary, journal_account_id) \
         VALUES (?, 'Banque Exemple', 'CH9300762011623852957', TRUE, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(acc["1020"])
    .fetch_one(&pool)
    .await
    .unwrap();

    let mut d = DecorMontee {
        company_id,
        user_id,
        fy_id,
        expense: acc["4000"],
        open_id: 0,
        open_vat_id: 0,
        paid_id: 0,
        cancelled_id: 0,
        paid_cancelled_id: 0,
        completed_id: 0,
        attached_id: 0,
        batch_id: 0,
        completed_doc: 0,
        attached_doc: 0,
    };
    let (p, v) = (acc["2000"], acc["1170"]);
    d.open_id = facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-O", dec!(100), dec!(0)).await;
    d.open_vat_id =
        facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-T", dec!(1000), dec!(8.1)).await;
    d.paid_id = facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-P", dec!(200), dec!(0)).await;
    d.cancelled_id =
        facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-A", dec!(300), dec!(0)).await;
    d.paid_cancelled_id =
        facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-PA", dec!(400), dec!(0)).await;
    d.completed_id =
        facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-C", dec!(150), dec!(0)).await;
    d.attached_id = facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-J", dec!(250), dec!(0)).await;
    let batched = facture_ouverte_rc1(&pool, &d, contact, p, v, "FF-L", dec!(500), dec!(0)).await;
    let day = chrono::NaiveDate::from_ymd_opt(2026, 6, 20).unwrap();
    let virement = SettlementChoice::BankTransfer {
        bank_account_id: bank_id,
    };
    supplier_invoices::pay(&pool, company_id, d.paid_id, virement, day, user_id)
        .await
        .expect("payée (lettrage de pièce)");
    supplier_invoices::cancel(&pool, company_id, d.cancelled_id, user_id)
        .await
        .expect("annulée (contre-passation)");
    supplier_invoices::pay(
        &pool,
        company_id,
        d.paid_cancelled_id,
        virement,
        day,
        user_id,
    )
    .await
    .expect("payée");
    supplier_invoices::cancel(&pool, company_id, d.paid_cancelled_id, user_id)
        .await
        .expect("payée puis annulée (règlement détaché)");
    for (hash, invoice) in [("h-c", d.completed_id), ("h-j", d.attached_id)] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO imported_supplier_invoices (company_id, file_hash, storage_path, \
             original_filename, mime_type, byte_size, status, supplier_invoice_id) \
             VALUES (?, ?, ?, ?, 'application/pdf', 10, 'completed', ?) RETURNING id",
        )
        .bind(company_id)
        .bind(hash)
        .bind(format!("{hash}.pdf"))
        .bind(format!("{hash}.pdf"))
        .bind(invoice)
        .fetch_one(&pool)
        .await
        .unwrap();
        if hash == "h-c" {
            d.completed_doc = id;
        } else {
            d.attached_doc = id;
        }
    }
    let outcome = payment_batches::create_batch(
        &pool,
        NewPaymentBatch {
            company_id,
            bank_account_id: bank_id,
            requested_execution_date: day,
            supplier_invoice_ids: vec![batched],
        },
        user_id,
    )
    .await
    .expect("lot");
    d.batch_id = outcome.batch.expect("lot créé").batch.id;
    payment_batches::confirm_batch(&pool, company_id, d.batch_id, day, user_id)
        .await
        .expect("lot confirmé");

    sqlx::query("ALTER TABLE supplier_invoices DROP COLUMN internal_number")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("ALTER TABLE company_invoice_settings DROP COLUMN supplier_internal_number_format")
        .execute(&pool)
        .await
        .unwrap();

    let watched = [
        "supplier_invoices",
        "supplier_invoice_lines",
        "imported_supplier_invoices",
        "payment_batches",
        "payment_batch_items",
        "journal_entries",
        "journal_entry_lines",
        "audit_log",
    ];
    let mut before = Vec::new();
    for t in watched {
        let cols = columns_of(&pool, t).await;
        before.push((t, cols.clone(), table_snapshot_on(&pool, t, &cols).await));
    }
    let statuses: Vec<String> =
        sqlx::query_scalar("SELECT status FROM supplier_invoices ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        statuses,
        [
            "open",
            "open",
            "paid",
            "cancelled",
            "cancelled",
            "open",
            "open",
            "paid"
        ],
        "montage : les quatre statuts semés"
    );

    // --- La montée ---
    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("la montée doit passer sur une base peuplée");

    // 1. — colonne par colonne, à l'identique (`version`, `updated_at` compris).
    for (t, cols, b) in &before {
        assert_eq!(
            &table_snapshot_on(&pool, t, cols).await,
            b,
            "table {t} modifiée par la montée"
        );
    }

    // 2. — relue par le code : get, list, vue de règlement.
    let (items, total) = supplier_invoices::list(&pool, company_id, &[], 50, 0)
        .await
        .expect("liste");
    assert_eq!(total, 8);
    assert_eq!(items.len(), 8);
    for (id, status) in [
        (d.open_id, "open"),
        (d.paid_id, "paid"),
        (d.cancelled_id, "cancelled"),
        (d.paid_cancelled_id, "cancelled"),
        (batched, "paid"),
    ] {
        let (inv, _) = supplier_invoices::get(&pool, company_id, id)
            .await
            .expect("relecture")
            .expect("présente");
        assert_eq!(inv.status, status);
        assert!(inv.purchase_journal_entry_id.is_some());
        let view = supplier_invoices::get_settlement_view(&pool, company_id, id)
            .await
            .expect("vue")
            .expect("présente");
        assert_eq!(view.invoice.status, status);
    }

    // 4. — le seuil et l'origine des liens.
    let min: String =
        sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    // Toutes les migrations ont tourné : le dernier bump (Story 34-11a1).
    assert_eq!(min, "0.17.6");
    let origins: Vec<Option<String>> =
        sqlx::query_scalar("SELECT link_origin FROM imported_supplier_invoices ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        origins,
        vec![None, None],
        "lien antérieur à la 0.17.0 : nul"
    );

    // 3. et 5. — le cas de Guy : la facture ouverte, puis celle à document joint,
    // se dévalident, se modifient, se revalident.
    for (id, numero) in [(d.open_id, "FF-O"), (d.attached_id, "FF-J")] {
        let (inv, _) = supplier_invoices::get(&pool, company_id, id)
            .await
            .unwrap()
            .unwrap();
        let u = supplier_invoices::unvalidate(&pool, company_id, id, inv.version, user_id)
            .await
            .expect("dévalidée");
        assert_eq!(u.invoice.status, "draft");
        assert_eq!(u.invoice.purchase_journal_entry_id, None);
        let m = supplier_invoices::update_draft(
            &pool,
            company_id,
            id,
            u.invoice.version,
            kesh_db::entities::NewSupplierInvoice {
                company_id,
                contact_id: contact,
                supplier_invoice_number: Some(numero.into()),
                invoice_date: chrono::NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
                due_date: None,
                creditor_iban: Some("CH9300762011623852957".into()),
                creditor_qr_iban: None,
                payment_reference: None,
                expected_payment_amount: None,
                project_id: None,
                lines: vec![kesh_db::entities::NewSupplierInvoiceLine {
                    description: "Total corrigé".into(),
                    quantity: dec!(1),
                    unit_price: dec!(99.95),
                    vat_rate: dec!(0),
                    expense_account_id: d.expense,
                }],
            },
            user_id,
        )
        .await
        .expect("modifiée");
        assert_eq!(m.invoice.total_amount, dec!(99.95));
        let r = supplier_invoices::validate(&pool, company_id, id, m.invoice.version, user_id)
            .await
            .expect("revalidée");
        assert_eq!(r.invoice.status, "open");
        assert!(r.invoice.purchase_journal_entry_id.is_some());
    }
    let _ = (
        d.open_vat_id,
        d.completed_doc,
        d.attached_doc,
        d.completed_id,
    );
}

/// AC-2 (a) — depuis l'état 0.16.x (avant `20261030000001`).
#[sqlx::test(migrations = false)]
async fn draft_migration_preserves_a_populated_016_base(pool: MySqlPool) {
    draft_migration_preserves_a_populated_base(pool, (MANUAL_SOURCE_VERSION, MANUAL_SOURCE_LABEL))
        .await;
}

/// AC-2 (b) — depuis l'état de la rc1 (avant `20261031000001`).
#[sqlx::test(migrations = false)]
async fn draft_migration_preserves_a_populated_rc1_base(pool: MySqlPool) {
    draft_migration_preserves_a_populated_base(pool, (DRAFT_VERSION, DRAFT_LABEL)).await;
}

/// AC-3 (c) — la migration n'ajoute aucune table.
#[sqlx::test(migrations = false)]
async fn draft_migration_adds_no_table(pool: MySqlPool) {
    apply_migrations_up_to(&pool, common::migrations_before(DRAFT_VERSION, DRAFT_LABEL))
        .await
        .unwrap();
    let before = application_tables(&pool).await;
    // Story v0172-session : jusqu'à cette migration incluse (cf.
    // `bank_imports_manual_source_adds_no_table`).
    apply_migrations_up_to(
        &pool,
        common::migrations_before(DRAFT_VERSION, DRAFT_LABEL) + 1,
    )
    .await
    .unwrap();
    assert_eq!(application_tables(&pool).await, before);
}

// ---------------------------------------------------------------------------
// Story v0172-session (0.17.3, #876, CR #891) — `20261104000001_installation_settings`
// ---------------------------------------------------------------------------

const INSTALLATION_SETTINGS_VERSION: i64 = 20261104000001;
const INSTALLATION_SETTINGS_LABEL: &str = "installation_settings";

/// La migration de la 0.17.3 n'ajoute qu'une table **système**,
/// `_kesh_installation_settings`, vide (les défauts vivent dans le code), et
/// laisse intactes les lignes existantes — dont `refresh_tokens`, dont
/// l'échéance change de sens sans être réécrite. L'inventaire des tables
/// applicatives (celui d'une sauvegarde `.keshbackup`) ne change pas : une
/// sauvegarde 0.17.2 reste importable.
#[sqlx::test(migrations = false)]
async fn installation_settings_adds_only_its_system_table(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(INSTALLATION_SETTINGS_VERSION, INSTALLATION_SETTINGS_LABEL),
    )
    .await
    .unwrap();
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Session SA', 'Rue 1, 1000 Lausanne', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) VALUES (?, ?, 'Admin', ?) \
         RETURNING id",
    )
    .bind("session_admin")
    .bind("$argon2id$v=19$m=19456,t=2,p=1$mock_salt$mock_hash_for_schema_test_only")
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) \
         VALUES (?, '00000000-0000-4000-8000-000000000001', '2026-11-04 10:00:00.123')",
    )
    .bind(user_id)
    .execute(&pool)
    .await
    .unwrap();
    let before = application_tables(&pool).await;
    let row_before: (i64, String, chrono::NaiveDateTime) =
        sqlx::query_as("SELECT user_id, token, expires_at FROM refresh_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();

    kesh_db::MIGRATOR.run(&pool).await.unwrap();

    let mut expected = before.clone();
    expected.push("_kesh_installation_settings".to_string());
    expected.sort();
    let mut after = application_tables(&pool).await;
    after.sort();
    assert_eq!(after, expected, "une seule table neuve, système");
    assert!(
        !kesh_db::backup::TABLES_TO_TRUNCATE.contains(&"_kesh_installation_settings"),
        "table système : hors de l'inventaire d'une sauvegarde"
    );
    let rows: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _kesh_installation_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        rows, 0,
        "aucune ligne posée : les défauts vivent dans le code"
    );
    let row_after: (i64, String, chrono::NaiveDateTime) =
        sqlx::query_as("SELECT user_id, token, expires_at FROM refresh_tokens")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(row_after, row_before, "ligne refresh_tokens intacte");
    let settings = kesh_db::repositories::installation_settings::get(&pool)
        .await
        .unwrap();
    assert_eq!(settings.session_idle_minutes, 60);
    assert_eq!(settings.version, 0);
}

// ---------------------------------------------------------------------------
// Story 33-1a (#926) — `20261106000001_supplier_invoice_internal_number`
// ---------------------------------------------------------------------------

const NUMERO_INTERNE_VERSION: i64 = 20261106000001;
const NUMERO_INTERNE_LABEL: &str = "supplier_invoice_internal_number";
/// Date passée épinglée sur les `updated_at` du décor : sans elle, une migration
/// qui les toucherait survivrait à l'AC4 si elle tombait dans la même
/// milliseconde que l'insertion (`DATETIME(3)`).
const EPINGLE: &str = "2020-02-02 02:02:02.000";

/// Le décor d'une société à l'état 0.17.4, en SQL brut : le dépôt lit désormais
/// une colonne absente de la base partielle.
struct SocieteAvantNumero {
    company_id: i64,
    user_id: i64,
    expense: i64,
    payable: i64,
    bank: i64,
    fournisseur_a: i64,
    fournisseur_b: i64,
}

async fn societe_avant_numero(pool: &MySqlPool, nom: &str) -> SocieteAvantNumero {
    let company_id: i64 = sqlx::query_scalar(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, 'Rue 1, 1000 Lausanne', 'Pme', 'FR', 'FR') RETURNING id",
    )
    .bind(nom)
    .fetch_one(pool)
    .await
    .unwrap();
    let user_id: i64 = sqlx::query_scalar(
        "INSERT INTO users (username, password_hash, role, company_id) \
         VALUES (?, ?, 'Comptable', ?) RETURNING id",
    )
    .bind(format!("u_{company_id}"))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo")
    .bind(company_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let mut comptes = Vec::new();
    for (numero, libelle, genre) in [
        ("4000", "Charges", "Expense"),
        ("2000", "Créanciers", "Liability"),
        ("1020", "Banque", "Asset"),
    ] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO accounts (company_id, number, name, account_type) \
             VALUES (?, ?, ?, ?) RETURNING id",
        )
        .bind(company_id)
        .bind(numero)
        .bind(libelle)
        .bind(genre)
        .fetch_one(pool)
        .await
        .unwrap();
        comptes.push(id);
    }
    let mut fournisseurs = Vec::new();
    for nom_f in ["Fournisseur A SA", "Fournisseur B SA"] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO contacts (company_id, contact_type, name, is_supplier) \
             VALUES (?, 'Entreprise', ?, TRUE) RETURNING id",
        )
        .bind(company_id)
        .bind(nom_f)
        .fetch_one(pool)
        .await
        .unwrap();
        fournisseurs.push(id);
    }
    SocieteAvantNumero {
        company_id,
        user_id,
        expense: comptes[0],
        payable: comptes[1],
        bank: comptes[2],
        fournisseur_a: fournisseurs[0],
        fournisseur_b: fournisseurs[1],
    }
}

async fn exercice_avant_numero(
    pool: &MySqlPool,
    s: &SocieteAvantNumero,
    nom: &str,
    debut: &str,
    fin: &str,
) -> i64 {
    sqlx::query_scalar(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, updated_at) \
         VALUES (?, ?, ?, ?, 'Open', ?) RETURNING id",
    )
    .bind(s.company_id)
    .bind(nom)
    .bind(debut)
    .bind(fin)
    .bind(EPINGLE)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Une facture telle que l'écrivait la 0.17.4 : son écriture d'achat dans
/// l'exercice `fy` (sauf brouillon), puis la ligne, `updated_at` épinglé.
/// `date_ecriture` et `date_facture` peuvent différer : c'est l'écriture qui donne
/// l'exercice (C-e331-6).
#[allow(clippy::too_many_arguments)]
async fn facture_avant_numero(
    pool: &MySqlPool,
    s: &SocieteAvantNumero,
    fy: i64,
    contact: i64,
    statut: &str,
    date_facture: &str,
    date_ecriture: &str,
    numero: Option<&str>,
) -> i64 {
    use kesh_db::entities::{Journal, NewJournalEntry, NewJournalEntryLine};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    let mut tx = pool.begin().await.unwrap();
    let achat = if statut == "draft" {
        None
    } else {
        let je = kesh_db::repositories::journal_entries::create_in_tx(
            &mut tx,
            fy,
            s.user_id,
            NewJournalEntry {
                company_id: s.company_id,
                entry_date: date_ecriture.parse().unwrap(),
                journal: Journal::Achats,
                description: format!(
                    "Facture fournisseur {} - libellé figé",
                    numero.unwrap_or("")
                ),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: s.expense,
                        debit: dec!(100),
                        credit: Decimal::ZERO,
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: s.payable,
                        debit: Decimal::ZERO,
                        credit: dec!(100),
                        project_id: None,
                    },
                ],
            },
            false,
        )
        .await
        .expect("écriture d'achat de la 0.17.4");
        Some(je.entry.id)
    };
    let (reglement, payee_le) = if statut == "paid" {
        let je = kesh_db::repositories::journal_entries::create_in_tx(
            &mut tx,
            fy,
            s.user_id,
            NewJournalEntry {
                company_id: s.company_id,
                entry_date: date_ecriture.parse().unwrap(),
                journal: Journal::Banque,
                description: "Règlement fournisseur figé".into(),
                project_id: None,
                lines: vec![
                    NewJournalEntryLine {
                        account_id: s.payable,
                        debit: dec!(100),
                        credit: Decimal::ZERO,
                        project_id: None,
                    },
                    NewJournalEntryLine {
                        account_id: s.bank,
                        debit: Decimal::ZERO,
                        credit: dec!(100),
                        project_id: None,
                    },
                ],
            },
            false,
        )
        .await
        .expect("écriture de règlement de la 0.17.4");
        (Some(je.entry.id), Some(format!("{date_ecriture} 00:00:00")))
    } else {
        (None, None)
    };
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, status, \
         invoice_date, total_amount, purchase_journal_entry_id, settlement_type, \
         settlement_account_id, settlement_journal_entry_id, paid_at, updated_at) \
         VALUES (?, ?, ?, ?, ?, 100, ?, ?, ?, ?, ?, ?) RETURNING id",
    )
    .bind(s.company_id)
    .bind(contact)
    .bind(numero)
    .bind(statut)
    .bind(date_facture)
    .bind(achat)
    .bind(reglement.map(|_| "internal_account"))
    .bind(reglement.map(|_| s.bank))
    .bind(reglement)
    .bind(payee_le)
    .bind(EPINGLE)
    .fetch_one(&mut *tx)
    .await
    .unwrap();
    tx.commit().await.unwrap();
    id
}

/// `(numéro interne, version, updated_at)` d'une facture ; `updated_at` en texte.
async fn etat_facture(pool: &MySqlPool, id: i64) -> (Option<String>, i32, String) {
    sqlx::query_as(
        "SELECT internal_number, version, DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s.%f') \
         FROM supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

async fn compteur_exercice(pool: &MySqlPool, fy: i64) -> i64 {
    sqlx::query_scalar("SELECT supplier_invoice_next_number FROM fiscal_years WHERE id = ?")
        .bind(fy)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Empreinte de toutes les écritures : description et `updated_at`, à l'octet.
async fn empreinte_ecritures(pool: &MySqlPool) -> Vec<(i64, String, String)> {
    sqlx::query_as(
        "SELECT id, description, DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s.%f') \
         FROM journal_entries ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// AC2 à AC5 — une base 0.17.4 peuplée monte : chaque facture non brouillon reçoit
/// son numéro, par société et année de début d'exercice, dans l'ordre
/// `fy.start_date, invoice_date, id` ; les compteurs suivent ; rien d'autre ne bouge.
#[sqlx::test(migrations = false)]
async fn internal_number_migration_numbers_a_populated_base(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(NUMERO_INTERNE_VERSION, NUMERO_INTERNE_LABEL),
    )
    .await
    .expect("montage du schéma d'avant");
    let colonne: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'supplier_invoices' AND COLUMN_NAME = 'internal_number'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(colonne, 0, "montage : la base partielle n'a pas la colonne");

    let a = societe_avant_numero(&pool, "Société A").await;
    let b = societe_avant_numero(&pool, "Société B").await;
    // Deux exercices commençant en 2025 (dont un « 2025/2026 » commencé le 1.7),
    // un en 2026 commencé tard, un en 2027 sans facture.
    let fy_s1 = exercice_avant_numero(&pool, &a, "2025-S1", "2025-01-01", "2025-06-30").await;
    let fy_cheval = exercice_avant_numero(&pool, &a, "2025/2026", "2025-07-01", "2026-06-30").await;
    let fy_2026 = exercice_avant_numero(&pool, &a, "2026-S2", "2026-07-01", "2026-12-31").await;
    let fy_vide = exercice_avant_numero(&pool, &a, "2027", "2027-01-01", "2027-12-31").await;
    let fy_b = exercice_avant_numero(&pool, &b, "2026", "2026-01-01", "2026-12-31").await;

    let (fa, fb) = (a.fournisseur_a, a.fournisseur_b);
    // Trois factures à la même date : ordre par `id`.
    let s1 = facture_avant_numero(
        &pool,
        &a,
        fy_s1,
        fa,
        "open",
        "2025-03-01",
        "2025-03-01",
        Some("X-1"),
    )
    .await;
    let s2 = facture_avant_numero(
        &pool,
        &a,
        fy_s1,
        fa,
        "paid",
        "2025-03-01",
        "2025-03-01",
        None,
    )
    .await;
    // Même numéro du fournisseur chez un autre fournisseur.
    let s3 = facture_avant_numero(
        &pool,
        &a,
        fy_s1,
        fb,
        "cancelled",
        "2025-03-01",
        "2025-03-01",
        Some("X-1"),
    )
    .await;
    // Date antérieure, insérée après : ordre par date.
    let s4 = facture_avant_numero(
        &pool,
        &a,
        fy_s1,
        fa,
        "open",
        "2025-02-01",
        "2025-02-01",
        Some("X-4"),
    )
    .await;
    let s5 = facture_avant_numero(
        &pool,
        &a,
        fy_cheval,
        fa,
        "open",
        "2025-08-01",
        "2025-08-01",
        Some("X-5"),
    )
    .await;
    // Sa date tomberait dans l'exercice 2026 : l'exercice est celui de l'écriture.
    let s8 = facture_avant_numero(
        &pool,
        &a,
        fy_cheval,
        fa,
        "open",
        "2026-08-01",
        "2026-03-01",
        Some("X-8"),
    )
    .await;
    let s6 = facture_avant_numero(
        &pool,
        &a,
        fy_2026,
        fa,
        "open",
        "2026-09-01",
        "2026-09-01",
        None,
    )
    .await;
    let s7 = facture_avant_numero(
        &pool,
        &a,
        fy_2026,
        fb,
        "paid",
        "2026-09-02",
        "2026-09-02",
        Some("X-7"),
    )
    .await;
    let brouillon = facture_avant_numero(
        &pool,
        &a,
        fy_2026,
        fa,
        "draft",
        "2026-10-01",
        "2026-10-01",
        Some("X-9"),
    )
    .await;
    let sb = facture_avant_numero(
        &pool,
        &b,
        fy_b,
        b.fournisseur_a,
        "open",
        "2026-05-01",
        "2026-05-01",
        Some("X-1"),
    )
    .await;

    let ecritures_avant = empreinte_ecritures(&pool).await;
    assert_eq!(
        ecritures_avant.len(),
        11,
        "montage : neuf achats, deux règlements"
    );

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("la migration passe");

    for (id, attendu) in [
        (s4, "FF-2025-001"),
        (s1, "FF-2025-002"),
        (s2, "FF-2025-003"),
        (s3, "FF-2025-004"),
        (s5, "FF-2025-005"),
        (s8, "FF-2025-006"),
        (s6, "FF-2026-001"),
        (s7, "FF-2026-002"),
        (sb, "FF-2026-001"),
    ] {
        let (numero, version, maj) = etat_facture(&pool, id).await;
        assert_eq!(numero.as_deref(), Some(attendu), "facture {id}");
        assert_eq!(version, 1, "AC4 : version intacte, facture {id}");
        assert_eq!(
            maj,
            format!("{EPINGLE}000"),
            "AC4 : updated_at intact, facture {id}"
        );
    }
    let (numero, version, maj) = etat_facture(&pool, brouillon).await;
    assert_eq!(numero, None, "un brouillon n'a pas de numéro");
    assert_eq!((version, maj), (1, format!("{EPINGLE}000")));

    for (fy, attendu) in [
        (fy_s1, 5),
        (fy_cheval, 7),
        (fy_2026, 3),
        (fy_vide, 1),
        (fy_b, 2),
    ] {
        assert_eq!(
            compteur_exercice(&pool, fy).await,
            attendu,
            "compteur de l'exercice {fy}"
        );
        let maj: String = sqlx::query_scalar(
            "SELECT DATE_FORMAT(updated_at, '%Y-%m-%d %H:%i:%s.%f') FROM fiscal_years WHERE id = ?",
        )
        .bind(fy)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            maj,
            format!("{EPINGLE}000"),
            "AC4 : updated_at de l'exercice {fy}"
        );
    }

    assert_eq!(
        empreinte_ecritures(&pool).await,
        ecritures_avant,
        "AC4 : aucune écriture ne bouge"
    );
    let sans_numero: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM supplier_invoices WHERE status <> 'draft' AND internal_number IS NULL",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(sans_numero, 0, "aucune facture non brouillon sans numéro");
    // Lu au schéma : le décor ne pose aucune ligne de réglages (revue P1, A-L6).
    let format: String = sqlx::query_scalar(
        "SELECT TRIM(BOTH '''' FROM COLUMN_DEFAULT) FROM information_schema.columns \
         WHERE table_schema = DATABASE() AND table_name = 'company_invoice_settings' \
         AND column_name = 'supplier_internal_number_format'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(format, "FF-{YEAR}-{SEQ:03}");
}

/// AC7 — la millième facture : `LPAD` seul tronquerait à `FF-2026-100` et l'`UNIQUE`
/// échouerait à la montée.
#[sqlx::test(migrations = false)]
async fn internal_number_migration_keeps_the_thousandth(pool: MySqlPool) {
    apply_migrations_up_to(
        &pool,
        common::migrations_before(NUMERO_INTERNE_VERSION, NUMERO_INTERNE_LABEL),
    )
    .await
    .unwrap();
    let s = societe_avant_numero(&pool, "Société mille").await;
    let fy = exercice_avant_numero(&pool, &s, "2026", "2026-01-01", "2026-12-31").await;
    // L'écriture d'achat peut être partagée : aucune unicité sur la colonne.
    let premiere = facture_avant_numero(
        &pool,
        &s,
        fy,
        s.fournisseur_a,
        "open",
        "2026-01-01",
        "2026-01-01",
        None,
    )
    .await;
    sqlx::query(
        "INSERT INTO supplier_invoices (company_id, contact_id, status, invoice_date, total_amount, \
         purchase_journal_entry_id) \
         SELECT company_id, contact_id, 'open', invoice_date, total_amount, purchase_journal_entry_id \
         FROM supplier_invoices, seq_1_to_1000 WHERE id = ?",
    )
    .bind(premiere)
    .execute(&pool)
    .await
    .unwrap();

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("la migration passe à 1 001 factures");

    let rangs: Vec<(String,)> = sqlx::query_as(
        "SELECT internal_number FROM supplier_invoices \
         WHERE internal_number IN ('FF-2026-999', 'FF-2026-1000', 'FF-2026-1001', 'FF-2026-100') \
         ORDER BY internal_number",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        rangs.into_iter().map(|r| r.0).collect::<Vec<_>>(),
        ["FF-2026-100", "FF-2026-1000", "FF-2026-1001", "FF-2026-999"]
    );
    assert_eq!(compteur_exercice(&pool, fy).await, 1002);
}

/// AC1 — la migration n'ajoute aucune table : `TABLES_TO_TRUNCATE` inchangé, donc
/// les sauvegardes 0.17.4 restent importables.
#[sqlx::test(migrations = false)]
async fn internal_number_migration_adds_no_table(pool: MySqlPool) {
    let n = common::migrations_before(NUMERO_INTERNE_VERSION, NUMERO_INTERNE_LABEL);
    apply_migrations_up_to(&pool, n).await.unwrap();
    let before = application_tables(&pool).await;
    apply_migrations_up_to(&pool, n + 1).await.unwrap();
    assert_eq!(application_tables(&pool).await, before);
}

// ===========================================================================
// Story 34-11a1 (#972, AC-1, AC-12) — `20261110000001_bank_accounts_without_iban`
// ===========================================================================

const SANS_IBAN_VERSION: i64 = 20261110000001;
const SANS_IBAN_LABEL: &str = "bank_accounts_without_iban";

/// Les colonnes de `bank_accounts` d'avant la migration, lues ligne à ligne
/// (horodatages compris) : l'« empreinte » que la montée ne doit pas toucher.
async fn bank_accounts_avant_colonnes(pool: &MySqlPool) -> Vec<String> {
    sqlx::query_scalar(
        "SELECT CONCAT_WS('|', id, company_id, bank_name, iban, COALESCE(qr_iban, '∅'), \
         COALESCE(journal_account_id, '∅'), is_primary, version, archived, \
         COALESCE(description, '∅'), COALESCE(closed_on, '∅'), created_at, updated_at) \
         FROM bank_accounts ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap()
}

/// AC-1 — une base 0.17.5 **peuplée** monte sans écart : chaque compte garde
/// ses colonnes d'avant à l'identique, devient `bank` sans quatre chiffres, et
/// les contraintes neuves sont actives après la montée. Montage **résolu par
/// version** (P6), assertion de montage sur la colonne `account_kind` absente.
#[sqlx::test(migrations = false)]
async fn sans_iban_la_montee_garde_chaque_compte_bancaire(pool: MySqlPool) {
    use kesh_db::entities::address::StructuredAddress;
    use kesh_db::entities::{Language, NewCompany, OrgType};

    apply_migrations_up_to(
        &pool,
        common::migrations_before(SANS_IBAN_VERSION, SANS_IBAN_LABEL),
    )
    .await
    .expect("apply_migrations_up_to(avant bank_accounts_without_iban) failed");
    let colonne: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.COLUMNS WHERE TABLE_SCHEMA = DATABASE() \
         AND TABLE_NAME = 'bank_accounts' AND COLUMN_NAME = 'account_kind'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        colonne, 0,
        "assertion de montage : la migration n'a pas tourné"
    );

    let company_id = kesh_db::repositories::companies::create(
        &pool,
        NewCompany {
            name: "Montée SA".into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    // Trois comptes de forme 0.17.5 : principal avec QR-IBAN, secondaire avec
    // description, clos et archivé.
    for (iban, qr, primaire, description, clos, archive) in [
        (
            "CH9300762011623852957",
            Some("CH4431999123000889012"),
            true,
            None,
            None,
            false,
        ),
        (
            "CH5604835012345678009",
            None,
            false,
            Some("Épargne"),
            None,
            false,
        ),
        (
            "CH3908704016075473007",
            None,
            false,
            None,
            Some("2026-01-31"),
            true,
        ),
    ] {
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, \
             description, closed_on, archived) VALUES (?, 'Banque', ?, ?, ?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(iban)
        .bind(qr)
        .bind(primaire)
        .bind(description)
        .bind(clos)
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    }
    let avant = bank_accounts_avant_colonnes(&pool).await;
    assert_eq!(avant.len(), 3, "assertion de montage : trois comptes");

    kesh_db::MIGRATOR
        .run(&pool)
        .await
        .expect("MIGRATOR.run() final failed — la montée doit passer sur une base peuplée");

    assert_eq!(
        bank_accounts_avant_colonnes(&pool).await,
        avant,
        "les colonnes d'avant sont intactes, IBAN compris"
    );
    let neuves: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT account_kind, card_last4 FROM bank_accounts ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(neuves, vec![("bank".to_string(), None); 3]);
    // L'entité relit chaque compte (décodage de l'IBAN `Option`, du type).
    let lus = kesh_db::repositories::bank_accounts::list_by_company(
        &pool,
        company_id,
        kesh_db::repositories::bank_accounts::BankAccountListFilter {
            include_archived: true,
            include_closed: true,
        },
    )
    .await
    .unwrap();
    assert_eq!(lus.len(), 3);
    assert!(lus.iter().all(|a| a.iban.is_some()
        && a.account_kind == kesh_db::entities::BankAccountKind::Bank));
    // Les contraintes neuves sont actives : un compte bancaire sans IBAN est
    // refusé, une carte sans IBAN est admise.
    assert!(
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'X', NULL)"
        )
        .bind(company_id)
        .execute(&pool)
        .await
        .is_err(),
        "un compte bancaire sans IBAN doit être refusé"
    );
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, account_kind, iban, card_last4) \
         VALUES (?, 'Visa', 'credit_card', NULL, '1234')",
    )
    .bind(company_id)
    .execute(&pool)
    .await
    .expect("une carte sans IBAN est admise après la montée");
    let min: String =
        sqlx::query_scalar("SELECT kesh_version_min_required FROM _kesh_version WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(min, "0.17.6");
}

/// AC-12 — la migration n'ajoute aucune table : l'inventaire des tables d'une
/// sauvegarde 0.17.5 reste celui de la 0.17.6, qui l'importe donc.
#[sqlx::test(migrations = false)]
async fn sans_iban_la_migration_n_ajoute_aucune_table(pool: MySqlPool) {
    let n = common::migrations_before(SANS_IBAN_VERSION, SANS_IBAN_LABEL);
    apply_migrations_up_to(&pool, n).await.unwrap();
    let before = application_tables(&pool).await;
    apply_migrations_up_to(&pool, n + 1).await.unwrap();
    assert_eq!(application_tables(&pool).await, before);
}
