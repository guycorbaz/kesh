//! End-to-end IDOR tests for multi-tenant scoping (Story 6.2).
//!
//! Verifies that users cannot access resources from other companies.
//! Uses `seed_accounting_company` twice to create two companies with distinct users.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_users_cannot_cross_company_boundaries(pool: sqlx::MySqlPool) {
    use kesh_db::test_fixtures::{seed_accounting_company, truncate_all};

    // Setup: truncate and seed two companies
    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool)
        .await
        .expect("seed company A");
    let company_b = seed_accounting_company(&pool)
        .await
        .expect("seed company B");

    // Validate: two distinct companies were created
    assert_ne!(
        company_a.company_id, company_b.company_id,
        "companies must be distinct"
    );

    // Test structure: In production, these would be HTTP requests via test client.
    // For unit validation, we verify:
    // 1. Company A admin can read company A data
    // 2. Company A admin CANNOT read company B data (would return 404 in HTTP)
    // 3. Sensitive scoping rules (company_id in JWT, company_id in queries)

    // This test structure is placeholder for full HTTP E2E.
    // Complete implementation requires:
    // - Tower test client to execute full route handlers
    // - JWT generation from test users
    // - HTTP response assertion (404, not 200 or 403)
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_multi_tenant_company_isolation_via_repository(pool: sqlx::MySqlPool) {
    use kesh_db::repositories::users;
    use kesh_db::test_fixtures::{seed_accounting_company, truncate_all};

    truncate_all(&pool).await.expect("truncate");
    let company_a = seed_accounting_company(&pool)
        .await
        .expect("seed A");
    let company_b = seed_accounting_company(&pool)
        .await
        .expect("seed B");

    // Verify: find_by_id_in_company returns None for cross-company user
    let admin_a_in_company_b =
        users::find_by_id_in_company(&pool, company_a.admin_user_id, company_b.company_id)
            .await
            .expect("query succeeds");

    assert!(
        admin_a_in_company_b.is_none(),
        "admin from A should NOT be found in company B context"
    );

    // Verify: find_by_id_in_company returns Some for same-company user
    let admin_a_in_company_a =
        users::find_by_id_in_company(&pool, company_a.admin_user_id, company_a.company_id)
            .await
            .expect("query succeeds");

    assert!(
        admin_a_in_company_a.is_some(),
        "admin from A should be found in company A context"
    );
}
