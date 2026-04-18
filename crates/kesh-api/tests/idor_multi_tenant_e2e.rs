//! End-to-end IDOR tests for multi-tenant scoping (Story 6.2).
//!
//! Verifies that users cannot access resources from other companies via repository-level scoping.
//! Uses `seed_accounting_company` twice to create two companies with distinct users.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_invoices_cross_company_scoped(pool: sqlx::MySqlPool) {
    use kesh_db::repositories::invoices;
    use kesh_db::test_fixtures::{seed_accounting_company, seed_contact_and_product, truncate_all};

    // Setup: truncate and seed two companies
    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let (contact_a_id, _) = seed_contact_and_product(&pool, company_a.company_id)
        .await
        .expect("seed A");
    let (contact_b_id, _) = seed_contact_and_product(&pool, company_b.company_id)
        .await
        .expect("seed B");

    // Create invoices in each company
    let invoice_a_result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, issue_date, total_before_vat, total_vat, total_after_vat)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(company_a.company_id)
    .bind(contact_a_id)
    .bind("INV-001")
    .bind("draft")
    .bind("2026-04-18")
    .bind(1000i64)
    .bind(0i64)
    .bind(1000i64)
    .execute(&pool)
    .await
    .expect("invoice A insert");

    let invoice_b_result = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, issue_date, total_before_vat, total_vat, total_after_vat)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(company_b.company_id)
    .bind(contact_b_id)
    .bind("INV-002")
    .bind("draft")
    .bind("2026-04-18")
    .bind(2000i64)
    .bind(0i64)
    .bind(2000i64)
    .execute(&pool)
    .await
    .expect("invoice B insert");

    let invoice_a_id = invoice_a_result.last_insert_id() as i64;
    let invoice_b_id = invoice_b_result.last_insert_id() as i64;

    // Test: Scoped query for cross-company invoice (company A trying to access company B's invoice)
    // Story 6.2 AC#6: Repository-level scoping should block cross-company access
    let found_invoice_b_in_company_a = invoices::find_by_id_with_lines(&pool, company_a.company_id, invoice_b_id)
        .await
        .expect("scoped query succeeds");

    assert!(
        found_invoice_b_in_company_a.is_none(),
        "Company A cannot access invoice B via scoped query (would return 404 in HTTP handler)"
    );

    // Test: Same-company access should succeed
    let found_invoice_a_in_company_a =
        invoices::find_by_id_with_lines(&pool, company_a.company_id, invoice_a_id)
            .await
            .expect("scoped query succeeds");

    assert!(
        found_invoice_a_in_company_a.is_some(),
        "Company A can access its own invoice A"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_products_cross_company_scoped(pool: sqlx::MySqlPool) {
    use kesh_db::repositories::products;
    use kesh_db::test_fixtures::{seed_accounting_company, seed_contact_and_product, truncate_all};

    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let (_, product_a_id) = seed_contact_and_product(&pool, company_a.company_id)
        .await
        .expect("seed A");
    let (_, product_b_id) = seed_contact_and_product(&pool, company_b.company_id)
        .await
        .expect("seed B");

    // Test: Scoped query for cross-company product (company A trying to access company B's product)
    let found_product_b_in_company_a = products::find_by_id(&pool, company_a.company_id, product_b_id)
        .await
        .expect("scoped query succeeds");

    assert!(
        found_product_b_in_company_a.is_none(),
        "Company A cannot access product B via scoped query (would return 404)"
    );

    // Test: Same-company access should succeed
    let found_product_a_in_company_a = products::find_by_id(&pool, company_a.company_id, product_a_id)
        .await
        .expect("scoped query succeeds");

    assert!(
        found_product_a_in_company_a.is_some(),
        "Company A can access its own product A"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_multi_tenant_company_isolation_via_repository(pool: sqlx::MySqlPool) {
    use kesh_db::repositories::users;
    use kesh_db::test_fixtures::{seed_accounting_company, truncate_all};

    truncate_all(&pool).await.expect("truncate");
    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

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
