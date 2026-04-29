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
            default_sales_journal: settings.default_sales_journal,
            journal_entry_description_template: settings.journal_entry_description_template.clone(),
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
            default_sales_journal: Journal::Ventes,
            journal_entry_description_template: settings.journal_entry_description_template.clone(),
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
