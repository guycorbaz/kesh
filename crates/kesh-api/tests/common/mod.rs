//! Shared test utilities and fixtures for E2E tests

use kesh_db::entities::{Language, NewCompany, OrgType};
use kesh_db::repositories::companies;
use sqlx::MySqlPool;

/// Create a test company with default values (required by Story 6.2 for users.company_id FK).
/// Used across E2E tests: onboarding, profile, rbac, users, companies, i18n, etc.
pub async fn create_test_company(pool: &MySqlPool) {
    companies::create(
        pool,
        NewCompany {
            name: "Test Company".into(),
            address: "Test Address".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("create test company");
}
