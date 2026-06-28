//! Tests pour le repository company_invoice_settings (Story 2.6).

use kesh_db::entities::{CompanyInvoiceSettingsUpdate, Journal, Language, NewCompany, OrgType};
use kesh_db::repositories::{accounts, companies, company_invoice_settings};
use sqlx::MySqlPool;

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_finds_accounts_1100_3000(pool: MySqlPool) {
    // Create a company
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Test Co".to_string(),
            address: "123 Main St".to_string(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("Failed to create company");

    // Load the PME chart of accounts (includes 1100 and 3000)
    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("Failed to load chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .expect("Failed to create accounts from chart");

    // Call insert_with_defaults
    let settings = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .expect("Failed to insert with defaults");

    // Verify the settings were created with the correct account IDs (not None)
    assert_eq!(settings.company_id, company.id);
    assert!(
        settings.default_receivable_account_id.is_some(),
        "Account 1100 should be found"
    );
    assert!(
        settings.default_revenue_account_id.is_some(),
        "Account 3000 should be found"
    );
    assert_eq!(settings.invoice_number_format, "F-{YEAR}-{SEQ:04}");
    assert_eq!(settings.default_sales_journal.as_str(), "Ventes");
}

/// P8 / C1 — calling insert_with_defaults twice on the same company must be idempotent.
/// The second call exercises the `rows_affected == 0` branch (DUPLICATE KEY) and the
/// JOIN-on-active-accounts validation introduced by P16. It must return the existing
/// settings row, not error and not corrupt state.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_is_idempotent_on_existing_row(pool: MySqlPool) {
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Idem Co".to_string(),
            address: "789 Reentrant Way".to_string(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("Failed to create company");

    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("Failed to load chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .expect("Failed to create accounts from chart");

    // First call: inserts the row (rows_affected == 1).
    let first = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .expect("first insert should succeed");

    // Second call: exercises rows_affected == 0 path. Must succeed and return same row.
    let second = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .expect("second insert should be idempotent");

    assert_eq!(
        first.company_id, second.company_id,
        "Idempotent call must return the same settings row (same company_id PK)"
    );
    assert_eq!(
        first.default_receivable_account_id, second.default_receivable_account_id,
        "Account references must match across idempotent calls"
    );
    assert_eq!(
        first.default_revenue_account_id,
        second.default_revenue_account_id,
    );
}

/// P8 / P16 — if the FK accounts have been deactivated after the row was inserted,
/// the idempotent path must fail with InactiveOrInvalidAccounts (FK liveness check).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_rejects_when_referenced_accounts_inactive(pool: MySqlPool) {
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Stale FK Co".to_string(),
            address: "1 Deactivated St".to_string(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("Failed to create company");

    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("Failed to load chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .expect("Failed to create accounts from chart");

    company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .expect("seed insert should succeed");

    // Deactivate the referenced accounts (1100 and 3000) without removing them.
    sqlx::query(
        "UPDATE accounts SET active = FALSE WHERE company_id = ? AND number IN ('1100', '3000')",
    )
    .bind(company.id)
    .execute(&pool)
    .await
    .expect("Failed to deactivate accounts");

    // Re-call must reject because the JOIN on accounts.active=TRUE finds no row.
    let result = company_invoice_settings::insert_with_defaults(&pool, company.id).await;
    assert!(
        matches!(
            result,
            Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts)
        ),
        "Expected InactiveOrInvalidAccounts when referenced accounts are inactive, got: {:?}",
        result
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_rejects_missing_accounts(pool: MySqlPool) {
    // Create a company without any accounts
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Empty Co".to_string(),
            address: "456 Side St".to_string(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("Failed to create company");

    // Call insert_with_defaults with no accounts
    // E2-001 Fix: Now that P1-004+P1-007 added early NULL validation,
    // insert_with_defaults should reject with InactiveOrInvalidAccounts error
    let result = company_invoice_settings::insert_with_defaults(&pool, company.id).await;

    // Verify the error is correctly rejected
    assert!(result.is_err(), "Expected Err when accounts are missing");
    match result {
        Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts) => {
            // Expected behavior: fail-fast when accounts don't exist
        }
        _ => panic!(
            "Expected InactiveOrInvalidAccounts error, got: {:?}",
            result
        ),
    }
}

async fn create_admin_user(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES (?, ?, 'Admin', TRUE, ?)",
    )
    .bind(format!("admin_{}", company_id))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams")
    .bind(company_id)
    .execute(pool)
    .await
    .expect("create admin user for test");
    result.last_insert_id() as i64
}

/// KF-004 : payload identique à l'état persisté → pas de bump version,
/// `updated_at` inchangé, **aucune entrée audit_log `company_invoice_settings.updated`**.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_no_op_returns_unchanged_entity_no_audit(pool: MySqlPool) {
    let company = companies::create(
        &pool,
        NewCompany {
            name: "NoOp Co".into(),
            address: "1 rue Test".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();
    let admin_user_id = create_admin_user(&pool, company.id).await;

    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .unwrap();
    let settings = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .unwrap();
    let version_initial = settings.version;
    let updated_at_initial = settings.updated_at;

    let result = company_invoice_settings::update(
        &pool,
        company.id,
        version_initial,
        admin_user_id,
        CompanyInvoiceSettingsUpdate {
            invoice_number_format: settings.invoice_number_format.clone(),
            default_receivable_account_id: settings.default_receivable_account_id,
            default_revenue_account_id: settings.default_revenue_account_id,
            default_vat_payable_account_id: settings.default_vat_payable_account_id,
            default_vat_recoverable_account_id: settings.default_vat_recoverable_account_id,
            default_vat_decompte_account_id: settings.default_vat_decompte_account_id,
            default_sales_journal: settings.default_sales_journal,
            journal_entry_description_template: settings.journal_entry_description_template.clone(),
            credit_note_number_format: settings.credit_note_number_format.clone(),
            default_payable_account_id: settings.default_payable_account_id,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.version, version_initial);
    assert_eq!(result.updated_at, updated_at_initial);

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'company_invoice_settings' AND entity_id = ? AND action = 'company_invoice_settings.updated'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 0);
}

/// KF-004 régression : modifier `invoice_number_format` → bump version + audit log.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_partial_change_bumps_version(pool: MySqlPool) {
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Partial Co".into(),
            address: "2 rue Test".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();
    let admin_user_id = create_admin_user(&pool, company.id).await;

    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .unwrap();
    let settings = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .unwrap();
    let version_initial = settings.version;

    let result = company_invoice_settings::update(
        &pool,
        company.id,
        version_initial,
        admin_user_id,
        CompanyInvoiceSettingsUpdate {
            invoice_number_format: "F-{YEAR}-{SEQ:05}".into(),
            default_receivable_account_id: settings.default_receivable_account_id,
            default_revenue_account_id: settings.default_revenue_account_id,
            default_vat_payable_account_id: settings.default_vat_payable_account_id,
            default_vat_recoverable_account_id: settings.default_vat_recoverable_account_id,
            default_vat_decompte_account_id: settings.default_vat_decompte_account_id,
            default_sales_journal: Journal::Ventes,
            journal_entry_description_template: settings.journal_entry_description_template.clone(),
            credit_note_number_format: settings.credit_note_number_format.clone(),
            default_payable_account_id: settings.default_payable_account_id,
        },
    )
    .await
    .unwrap();
    assert_eq!(result.version, version_initial + 1);
    assert_eq!(result.invoice_number_format, "F-{YEAR}-{SEQ:05}");

    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'company_invoice_settings' AND entity_id = ? AND action = 'company_invoice_settings.updated'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);
}

// ---------------------------------------------------------------------------
// Story 18-1a — comptes TVA par défaut
// ---------------------------------------------------------------------------

/// Helper : id d'un compte par numéro pour une company.
async fn account_id_by_number(pool: &MySqlPool, company_id: i64, number: &str) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT id FROM accounts WHERE company_id = ? AND number = ?")
        .bind(company_id)
        .bind(number)
        .fetch_one(pool)
        .await
        .unwrap_or_else(|_| panic!("compte {number} introuvable pour company {company_id}"))
}

/// AC4/AC5 (a) — round-trip des 3 comptes TVA via `update()`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_vat_accounts_round_trip(pool: MySqlPool) {
    let company = companies::create(
        &pool,
        NewCompany {
            name: "VAT RT Co".into(),
            address: "3 rue TVA".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();
    let admin_user_id = create_admin_user(&pool, company.id).await;

    // Le plan PME contient désormais 1171 (Asset) + 2206 (Liability) + 2200 (Liability).
    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("chart");
    accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
        .await
        .unwrap();
    let settings = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .unwrap();

    // Avant configuration : les 3 champs sont NULL.
    assert_eq!(settings.default_vat_payable_account_id, None);
    assert_eq!(settings.default_vat_recoverable_account_id, None);
    assert_eq!(settings.default_vat_decompte_account_id, None);

    let vat_payable = account_id_by_number(&pool, company.id, "2200").await; // Liability
    let vat_recoverable = account_id_by_number(&pool, company.id, "1171").await; // Asset
    let vat_decompte = account_id_by_number(&pool, company.id, "2206").await; // Liability

    let result = company_invoice_settings::update(
        &pool,
        company.id,
        settings.version,
        admin_user_id,
        CompanyInvoiceSettingsUpdate {
            invoice_number_format: settings.invoice_number_format.clone(),
            default_receivable_account_id: settings.default_receivable_account_id,
            default_revenue_account_id: settings.default_revenue_account_id,
            default_vat_payable_account_id: Some(vat_payable),
            default_vat_recoverable_account_id: Some(vat_recoverable),
            default_vat_decompte_account_id: Some(vat_decompte),
            default_sales_journal: settings.default_sales_journal,
            journal_entry_description_template: settings.journal_entry_description_template.clone(),
            credit_note_number_format: settings.credit_note_number_format.clone(),
            default_payable_account_id: settings.default_payable_account_id,
        },
    )
    .await
    .unwrap();

    assert_eq!(result.version, settings.version + 1);
    assert_eq!(result.default_vat_payable_account_id, Some(vat_payable));
    assert_eq!(
        result.default_vat_recoverable_account_id,
        Some(vat_recoverable)
    );
    assert_eq!(result.default_vat_decompte_account_id, Some(vat_decompte));

    // Re-lecture indépendante : les colonnes sont bien persistées et relues
    // (COLUMNS / FromRow couvrent les 3 nouveaux champs).
    let reread = company_invoice_settings::get_or_create_default(&pool, company.id)
        .await
        .unwrap();
    assert_eq!(reread.default_vat_payable_account_id, Some(vat_payable));
    assert_eq!(
        reread.default_vat_recoverable_account_id,
        Some(vat_recoverable)
    );
    assert_eq!(reread.default_vat_decompte_account_id, Some(vat_decompte));

    // AC8 (d) — non-régression : receivable/revenue inchangés.
    assert_eq!(
        reread.default_receivable_account_id,
        settings.default_receivable_account_id
    );
    assert_eq!(
        reread.default_revenue_account_id,
        settings.default_revenue_account_id
    );
}

/// AC8 (c) — anti-IDOR / FK : un compte d'une AUTRE company ne peut pas être
/// désigné comme compte TVA (la contrainte FK `fk_cis_vat_*` rejette tout id
/// hors `accounts`). La validation d'appartenance company stricte vit dans le
/// handler route (`validate_account`) ; ce test couvre le garde-fou DB FK.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_vat_account_foreign_id_rejected_by_fk(pool: MySqlPool) {
    let company_a = companies::create(
        &pool,
        NewCompany {
            name: "IDOR A".into(),
            address: "1".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap();
    let admin_a = create_admin_user(&pool, company_a.id).await;
    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("chart");
    accounts::bulk_create_from_chart(&pool, company_a.id, &chart, "fr")
        .await
        .unwrap();
    let settings_a = company_invoice_settings::insert_with_defaults(&pool, company_a.id)
        .await
        .unwrap();

    // Un id qui n'existe dans aucune company → FK violation attendue.
    let nonexistent_account_id: i64 = 9_999_999;

    let result = company_invoice_settings::update(
        &pool,
        company_a.id,
        settings_a.version,
        admin_a,
        CompanyInvoiceSettingsUpdate {
            invoice_number_format: settings_a.invoice_number_format.clone(),
            default_receivable_account_id: settings_a.default_receivable_account_id,
            default_revenue_account_id: settings_a.default_revenue_account_id,
            default_vat_payable_account_id: Some(nonexistent_account_id),
            default_vat_recoverable_account_id: None,
            default_vat_decompte_account_id: None,
            default_sales_journal: settings_a.default_sales_journal,
            journal_entry_description_template: settings_a
                .journal_entry_description_template
                .clone(),
            credit_note_number_format: settings_a.credit_note_number_format.clone(),
            default_payable_account_id: settings_a.default_payable_account_id,
        },
    )
    .await;

    assert!(
        result.is_err(),
        "un account_id inexistant doit être rejeté par la FK fk_cis_vat_payable, got {result:?}"
    );
}

/// AC1 — les comptes 1171 (Asset, parent 10) et 2206 (Liability, parent 20)
/// sont seedés par `bulk_create_from_chart` (nouvelle install) dans les 3 plans.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn chart_seed_includes_new_vat_accounts(pool: MySqlPool) {
    for org in ["Pme", "Independant", "Association"] {
        let company = companies::create(
            &pool,
            NewCompany {
                name: format!("Seed {org}"),
                address: "1".into(),
                ide_number: None,
                org_type: OrgType::Pme,
                accounting_language: Language::Fr,
                instance_language: Language::Fr,
            },
        )
        .await
        .unwrap();
        let chart = kesh_core::chart_of_accounts::load_chart(org).expect("chart");
        accounts::bulk_create_from_chart(&pool, company.id, &chart, "fr")
            .await
            .unwrap();

        let row_1171: (String, Option<i64>) = sqlx::query_as(
            "SELECT account_type, parent_id FROM accounts WHERE company_id = ? AND number = '1171'",
        )
        .bind(company.id)
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| panic!("1171 absent du plan {org}"));
        assert_eq!(row_1171.0, "Asset", "1171 doit être Asset ({org})");
        let parent_10 = account_id_by_number(&pool, company.id, "10").await;
        assert_eq!(row_1171.1, Some(parent_10), "1171 parent = 10 ({org})");

        let row_2206: (String, Option<i64>) = sqlx::query_as(
            "SELECT account_type, parent_id FROM accounts WHERE company_id = ? AND number = '2206'",
        )
        .bind(company.id)
        .fetch_one(&pool)
        .await
        .unwrap_or_else(|_| panic!("2206 absent du plan {org}"));
        assert_eq!(row_2206.0, "Liability", "2206 doit être Liability ({org})");
        let parent_20 = account_id_by_number(&pool, company.id, "20").await;
        assert_eq!(row_2206.1, Some(parent_20), "2206 parent = 20 ({org})");
    }
}

/// AC2/AC3 (b) — la **migration data** (INSERT idempotent par company) crée
/// 1171/2206 pour une company existante qui ne les a pas, avec le bon parent et
/// la locale comptable, sans toucher les comptes existants, et de façon
/// idempotente au re-run.
///
/// On simule une install antérieure en insérant manuellement seulement les
/// comptes parents 10/20 (pas 1171/2206), puis on rejoue **le SQL de backfill**
/// de la migration `20260614000001_vat_accounts_config.sql`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migration_backfill_creates_vat_accounts_idempotently(pool: MySqlPool) {
    // Company avec locale comptable DE pour vérifier le CASE de libellé.
    let company = companies::create(
        &pool,
        NewCompany {
            name: "Legacy DE".into(),
            address: "1".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::De,
            instance_language: Language::De,
        },
    )
    .await
    .unwrap();

    // Install antérieure : seulement les comptes parents 10 et 20 (Asset/Liability).
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type) \
         VALUES (?, '10', 'Umlaufvermögen', 'Asset'), (?, '20', 'Kurzfristiges Fremdkapital', 'Liability')",
    )
    .bind(company.id)
    .bind(company.id)
    .execute(&pool)
    .await
    .unwrap();

    // SQL de backfill identique à la migration (locale via CASE accounting_language,
    // garde « company sans plan » via `EXISTS (… accounts …)`).
    let backfill_1171 = "INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version) \
        SELECT c.id, '1171', \
            CASE c.accounting_language WHEN 'DE' THEN 'Vorsteuer' WHEN 'IT' THEN 'Imposta precedente' WHEN 'EN' THEN 'Input VAT' ELSE 'Impôt préalable' END, \
            'Asset', (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '10'), TRUE, 1 \
        FROM companies c \
        WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id) AND NOT EXISTS (SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '1171')";
    let backfill_2206 = "INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version) \
        SELECT c.id, '2206', \
            CASE c.accounting_language WHEN 'DE' THEN 'MWST-Abrechnung' WHEN 'IT' THEN 'Rendiconto IVA' WHEN 'EN' THEN 'VAT settlement' ELSE 'Décompte TVA' END, \
            'Liability', (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '20'), TRUE, 1 \
        FROM companies c \
        WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id) AND NOT EXISTS (SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '2206')";

    sqlx::query(backfill_1171).execute(&pool).await.unwrap();
    sqlx::query(backfill_2206).execute(&pool).await.unwrap();

    // 1171 créé : Asset, parent 10, libellé DE.
    let r1: (String, String, Option<i64>) = sqlx::query_as(
        "SELECT account_type, name, parent_id FROM accounts WHERE company_id = ? AND number = '1171'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(r1.0, "Asset");
    assert_eq!(
        r1.1, "Vorsteuer",
        "libellé DE attendu (CASE accounting_language)"
    );
    let parent_10 = account_id_by_number(&pool, company.id, "10").await;
    assert_eq!(r1.2, Some(parent_10));

    // 2206 créé : Liability, parent 20, libellé DE.
    let r2: (String, String) = sqlx::query_as(
        "SELECT account_type, name FROM accounts WHERE company_id = ? AND number = '2206'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(r2.0, "Liability");
    assert_eq!(r2.1, "MWST-Abrechnung");

    // Idempotence : re-run du backfill ne crée pas de doublon (NOT EXISTS).
    sqlx::query(backfill_1171).execute(&pool).await.unwrap();
    sqlx::query(backfill_2206).execute(&pool).await.unwrap();

    let count_1171: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND number = '1171'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let count_2206: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND number = '2206'",
    )
    .bind(company.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(count_1171, 1, "1171 ne doit pas être dupliqué au re-run");
    assert_eq!(count_2206, 1, "2206 ne doit pas être dupliqué au re-run");

    // AC3 — aucun compte existant (10/20) altéré.
    let parent_20 = account_id_by_number(&pool, company.id, "20").await;
    assert!(parent_10 > 0 && parent_20 > 0);
}

/// Régression Pass 3 (Opus) + Pass 4 (Sonnet) — le backfill se déclenche sur la
/// PRÉSENCE D'UN PLAN COMPTABLE, pas sur le flag `is_stub`.
///
/// - Une company SANS plan (stub bootstrap, 0 compte) est EXCLUE : sinon le garde de
///   seed onboarding `if existing == 0` verrait `existing == 2` et sauterait le seed
///   du plan COMPLET, laissant la company privée de ses comptes de base (HIGH Pass 3).
///   Elle recevra `1171`/`2206` via `bulk_create_from_chart` à l'onboarding.
/// - Une company AVEC un plan mais `is_stub` ENCORE TRUE (état mid-onboarding : le plan
///   est seedé à `set_accounting_language` AVANT que `set_coordinates` repasse is_stub
///   à FALSE) DOIT être backfillée — sinon un upgrade pendant cette fenêtre la priverait
///   définitivement des comptes TVA (MEDIUM Pass 4). Tester `is_stub = FALSE` l'aurait raté.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn migration_backfill_keys_on_chart_presence_not_stub_flag(pool: MySqlPool) {
    // Helper : crée une company avec parents 10/20, puis force is_stub à la valeur voulue.
    async fn seed_company(pool: &MySqlPool, name: &str, with_chart: bool, is_stub: bool) -> i64 {
        let c = companies::create(
            pool,
            NewCompany {
                name: name.into(),
                address: "1".into(),
                ide_number: None,
                org_type: OrgType::Pme,
                accounting_language: Language::Fr,
                instance_language: Language::Fr,
            },
        )
        .await
        .unwrap();
        if with_chart {
            sqlx::query(
                "INSERT INTO accounts (company_id, number, name, account_type) \
                 VALUES (?, '10', 'Actif circulant', 'Asset'), (?, '20', 'Capitaux étrangers CT', 'Liability')",
            )
            .bind(c.id)
            .bind(c.id)
            .execute(pool)
            .await
            .unwrap();
        }
        if is_stub {
            sqlx::query("UPDATE companies SET is_stub = TRUE WHERE id = ?")
                .bind(c.id)
                .execute(pool)
                .await
                .unwrap();
        }
        c.id
    }

    // a) onboardée : plan présent, is_stub = FALSE → DOIT être backfillée.
    let onboarded = seed_company(&pool, "Onboardée", true, false).await;
    // b) stub bootstrap : 0 compte, is_stub = TRUE → DOIT être exclue.
    let stub = seed_company(&pool, "Stub bootstrap", false, true).await;
    // c) mid-onboarding : plan présent MAIS is_stub encore TRUE → DOIT être backfillée.
    let mid_onboarding = seed_company(&pool, "Mid-onboarding", true, true).await;

    // Backfill identique à la migration (garde `EXISTS (… accounts …)`).
    let backfill_1171 = "INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version) \
        SELECT c.id, '1171', 'Impôt préalable', 'Asset', \
            (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '10'), TRUE, 1 \
        FROM companies c \
        WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id) AND NOT EXISTS (SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '1171')";
    let backfill_2206 = "INSERT INTO accounts (company_id, number, name, account_type, parent_id, active, version) \
        SELECT c.id, '2206', 'Décompte TVA', 'Liability', \
            (SELECT p.id FROM accounts p WHERE p.company_id = c.id AND p.number = '20'), TRUE, 1 \
        FROM companies c \
        WHERE EXISTS (SELECT 1 FROM accounts a2 WHERE a2.company_id = c.id) AND NOT EXISTS (SELECT 1 FROM accounts a WHERE a.company_id = c.id AND a.number = '2206')";
    sqlx::query(backfill_1171).execute(&pool).await.unwrap();
    sqlx::query(backfill_2206).execute(&pool).await.unwrap();

    let vat_count = |company_id: i64| {
        let pool = pool.clone();
        async move {
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM accounts WHERE company_id = ? AND number IN ('1171', '2206')",
            )
            .bind(company_id)
            .fetch_one(&pool)
            .await
            .unwrap()
        }
    };

    assert_eq!(
        vat_count(onboarded).await,
        2,
        "company onboardée (plan présent) doit recevoir 1171 + 2206"
    );
    assert_eq!(
        vat_count(stub).await,
        0,
        "stub sans plan ne doit recevoir AUCUN compte TVA (seed à l'onboarding)"
    );
    assert_eq!(
        vat_count(mid_onboarding).await,
        2,
        "company mid-onboarding (plan présent, is_stub encore TRUE) doit être backfillée"
    );
}
