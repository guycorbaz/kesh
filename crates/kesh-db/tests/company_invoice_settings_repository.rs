//! Tests pour le repository company_invoice_settings (Story 2.6).

use kesh_db::entities::{Language, NewCompany, OrgType};
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
        first.default_revenue_account_id, second.default_revenue_account_id,
    );
}

/// P8 / P16 — if the FK accounts have been deactivated after the row was inserted,
/// the idempotent path must fail with InactiveOrInvalidAccounts (FK liveness check).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_rejects_when_referenced_accounts_inactive(
    pool: MySqlPool,
) {
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
    sqlx::query("UPDATE accounts SET active = FALSE WHERE company_id = ? AND number IN ('1100', '3000')")
        .bind(company.id)
        .execute(&pool)
        .await
        .expect("Failed to deactivate accounts");

    // Re-call must reject because the JOIN on accounts.active=TRUE finds no row.
    let result = company_invoice_settings::insert_with_defaults(&pool, company.id).await;
    assert!(
        matches!(result, Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts)),
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
    let result = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await;

    // Verify the error is correctly rejected
    assert!(result.is_err(), "Expected Err when accounts are missing");
    match result {
        Err(kesh_db::errors::DbError::InactiveOrInvalidAccounts) => {
            // Expected behavior: fail-fast when accounts don't exist
        }
        _ => panic!("Expected InactiveOrInvalidAccounts error, got: {:?}", result),
    }
}
