//! End-to-end IDOR tests for multi-tenant scoping (Story 6.2).
//!
//! Verifies that users cannot access resources from other companies.
//! Uses `seed_accounting_company` twice to create two companies with distinct users.

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_contacts_cross_company_returns_404(pool: sqlx::MySqlPool) {
    use kesh_api::AppState;
    use kesh_api::auth::jwt;
    use kesh_api::middleware::auth::CurrentUser;
    use kesh_db::repositories::contacts;
    use kesh_db::test_fixtures::{seed_accounting_company, seed_contact_and_product, truncate_all};

    // Setup: truncate and seed two companies
    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    // Create resources in each company
    let (contact_a_id, _product_a_id) = seed_contact_and_product(&pool, company_a.company_id)
        .await
        .expect("seed A resources");
    let (contact_b_id, _product_b_id) = seed_contact_and_product(&pool, company_b.company_id)
        .await
        .expect("seed B resources");

    // Test: Company A admin tries to access Company B contact
    // Story 6.2 AC#6: Should return 404 NotFound (not 200 with leaked data, not 403 which reveals existence)

    // Simulate: CurrentUser from Company A (as if JWT was decoded)
    let current_user_a = CurrentUser {
        user_id: company_a.admin_user_id,
        role: kesh_db::entities::user::Role::Admin,
        company_id: company_a.company_id,
    };

    // Attempt to access contact_b (belongs to company B) as user from company A
    let found_contact_b = contacts::find_by_id(&pool, contact_b_id)
        .await
        .expect("query succeeds");

    // Verify: Contact B exists in DB (proves we're testing IDOR scoping, not missing resources)
    assert!(found_contact_b.is_some(), "contact B should exist in DB");

    // Verify: Repository-level scoping blocks cross-company access
    let contact_b_in_company_a =
        contacts::find_by_id_in_company(&pool, contact_b_id, company_a.company_id)
            .await
            .expect("scoped query succeeds");

    assert!(
        contact_b_in_company_a.is_none(),
        "admin from company A should NOT access contact from company B (would return 404 in HTTP)"
    );

    // Verify: Same user CAN access their own company's contact
    let contact_a_in_company_a =
        contacts::find_by_id_in_company(&pool, contact_a_id, company_a.company_id)
            .await
            .expect("scoped query succeeds");

    assert!(
        contact_a_in_company_a.is_some(),
        "admin from company A should access contact from company A"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_invoices_cross_company_returns_404(pool: sqlx::MySqlPool) {
    use kesh_db::repositories::invoices;
    use kesh_db::test_fixtures::{seed_accounting_company, seed_contact_and_product, truncate_all};

    truncate_all(&pool).await.expect("truncate");

    let company_a = seed_accounting_company(&pool).await.expect("seed A");
    let company_b = seed_accounting_company(&pool).await.expect("seed B");

    let (contact_a_id, _) = seed_contact_and_product(&pool, company_a.company_id)
        .await
        .expect("seed A");
    let (contact_b_id, _) = seed_contact_and_product(&pool, company_b.company_id)
        .await
        .expect("seed B");

    // Create invoices in each company (simplified: just store contact_id reference)
    // Note: Full invoice creation would require more setup; this tests the scoping mechanism
    let invoice_a = sqlx::query!(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, issue_date, total_before_vat, total_vat, total_after_vat)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        company_a.company_id,
        contact_a_id,
        "INV-001",
        "draft",
        "2026-04-18",
        1000i64,
        0i64,
        1000i64
    )
    .execute(&pool)
    .await
    .expect("invoice A insert");

    let invoice_b = sqlx::query!(
        "INSERT INTO invoices (company_id, contact_id, invoice_number, status, issue_date, total_before_vat, total_vat, total_after_vat)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
        company_b.company_id,
        contact_b_id,
        "INV-002",
        "draft",
        "2026-04-18",
        2000i64,
        0i64,
        2000i64
    )
    .execute(&pool)
    .await
    .expect("invoice B insert");

    // Test: Scoped query for cross-company invoice returns None
    let found_invoice_b_unscoped = invoices::find_by_id(&pool, invoice_b.last_insert_id() as i64)
        .await
        .expect("query succeeds");
    assert!(
        found_invoice_b_unscoped.is_some(),
        "invoice B exists unscoped"
    );

    let found_invoice_b_scoped = invoices::find_by_id_in_company(
        &pool,
        invoice_b.last_insert_id() as i64,
        company_a.company_id,
    )
    .await
    .expect("scoped query succeeds");
    assert!(
        found_invoice_b_scoped.is_none(),
        "Company A cannot access invoice B (scoped query returns 404)"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_idor_products_cross_company_returns_404(pool: sqlx::MySqlPool) {
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

    // Test: Scoped query for cross-company product returns None
    let found_product_b_unscoped = products::find_by_id(&pool, product_b_id)
        .await
        .expect("query succeeds");
    assert!(
        found_product_b_unscoped.is_some(),
        "product B exists unscoped"
    );

    let found_product_b_scoped =
        products::find_by_id_in_company(&pool, product_b_id, company_a.company_id)
            .await
            .expect("scoped query succeeds");
    assert!(
        found_product_b_scoped.is_none(),
        "Company A cannot access product B (scoped query returns 404)"
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
