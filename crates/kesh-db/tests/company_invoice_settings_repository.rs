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

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_insert_with_defaults_handles_missing_accounts(pool: MySqlPool) {
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
    let settings = company_invoice_settings::insert_with_defaults(&pool, company.id)
        .await
        .expect("Failed to insert with defaults");

    // Verify the settings were created with NULL for missing accounts
    assert_eq!(settings.company_id, company.id);
    assert_eq!(settings.default_receivable_account_id, None);
    assert_eq!(settings.default_revenue_account_id, None);
    assert_eq!(settings.invoice_number_format, "F-{YEAR}-{SEQ:04}");
}
