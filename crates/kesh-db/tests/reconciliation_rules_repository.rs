//! Tests d'intégration pour `repositories::reconciliation_rules`
//! (Story 8-5b T2.4 — AC #101-#112, #118).
//!
//! 8 tests `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` — DB éphémère
//! avec migrations auto-appliquées. Pattern hérité 8-1b/8-4/8-5a-zero.

use kesh_db::entities::account::AccountType;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewReconciliationRule, NewUser, OrgType,
    ReconciliationMatchType, Role, UpdateReconciliationRule,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{accounts, companies, reconciliation_rules, users};
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.into(),
            address: "Rue Test 1".into(),
            ide_number: None,
            org_type: OrgType::Pme,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("company create")
    .id
}

async fn create_test_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: username.into(),
            password_hash:
                "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo"
                    .into(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create")
    .id
}

async fn create_test_account(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    number: &str,
    name: &str,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type: AccountType::Expense,
            parent_id: None,
        },
    )
    .await
    .expect("account create")
    .id
}

fn make_rule(
    label: &str,
    match_type: ReconciliationMatchType,
    match_value: &str,
    counterparty_account_id: i64,
    priority: i32,
) -> NewReconciliationRule {
    NewReconciliationRule {
        label: label.into(),
        match_type,
        match_value: match_value.into(),
        counterparty_account_id,
        priority,
        default_project_id: None,
    }
}

/// Comme [`make_rule`] mais avec un projet analytique par défaut (Story 19-5).
fn make_rule_with_project(
    label: &str,
    match_type: ReconciliationMatchType,
    match_value: &str,
    counterparty_account_id: i64,
    priority: i32,
    default_project_id: Option<i64>,
) -> NewReconciliationRule {
    NewReconciliationRule {
        label: label.into(),
        match_type,
        match_value: match_value.into(),
        counterparty_account_id,
        priority,
        default_project_id,
    }
}

/// Crée un projet analytique (Story 19-5) et retourne son id. `archived`
/// contrôle l'état pour tester les rejets. Calqué sur
/// `supplier_invoices_repository.rs::make_project`.
async fn make_project(pool: &MySqlPool, company_id: i64, code: &str, archived: bool) -> i64 {
    let id: i64 = sqlx::query_scalar(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) \
         VALUES (?, NULL, ?, ?, ?) RETURNING id",
    )
    .bind(company_id)
    .bind(code)
    .bind(format!("Projet {code}"))
    .bind(archived)
    .fetch_one(pool)
    .await
    .expect("project insert");
    id
}

// ---------------------------------------------------------------------------
// Test 1 — AC #101 + AC #107 : create + find_by_id scoped multi-tenant.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_and_find_by_id_scopes_by_company(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Alpha SA").await;
    let company_b = create_test_company(&pool, "Beta SA").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let account_a = create_test_account(&pool, company_a, user_a, "6510", "Telecom").await;
    let account_b = create_test_account(&pool, company_b, user_b, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let created = reconciliation_rules::create_in_tx(
        &mut tx,
        company_a,
        user_a,
        &make_rule(
            "Swisscom auto",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_a,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert!(created.id > 0);
    assert_eq!(created.label, "Swisscom auto");
    assert_eq!(created.company_id, company_a);
    assert_eq!(
        created.match_type,
        ReconciliationMatchType::CounterpartyContains
    );
    assert_eq!(created.match_value, "Swisscom");
    assert_eq!(created.counterparty_account_id, account_a);
    assert_eq!(created.priority, 100);
    assert!(created.active);
    assert_eq!(created.applied_count, 0);
    assert!(created.last_applied_at.is_none());
    assert_eq!(created.version, 1);

    // Round-trip same company → found.
    let found = reconciliation_rules::find_by_id_for_company(&pool, company_a, created.id)
        .await
        .unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);

    // Cross-tenant : company_b doit voir None (pas leak d'existence).
    let cross = reconciliation_rules::find_by_id_for_company(&pool, company_b, created.id)
        .await
        .unwrap();
    assert!(cross.is_none(), "cross-tenant leak detected (KF-002)");

    // Sanity : un autre account côté B existe, n'interfère pas.
    let _ = account_b;
}

// ---------------------------------------------------------------------------
// Test 2 — AC #102 : UNIQUE (company_id, match_type, match_value) sur rules actives.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_match_type_value_per_company_when_active(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 1",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    let conflict = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 2 même match",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            200,
        ),
    )
    .await;

    let err = conflict.expect_err("duplicate active rule must violate UNIQUE");
    assert!(
        matches!(err, DbError::UniqueConstraintViolation(_)),
        "expected UniqueConstraintViolation, got {err:?}"
    );
    assert!(
        reconciliation_rules::is_duplicate_rule_constraint(&err),
        "is_duplicate_rule_constraint must recognize uq_reconciliation_rules_match_active in {err:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 3 — AC #103 + Q3 : UNIQUE partiel permet recreate après soft-delete.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_partial_allows_create_when_existing_inactive(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let r1 = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 1",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Soft-delete r1.
    let mut tx = pool.begin().await.unwrap();
    let transitioned =
        reconciliation_rules::soft_delete_by_id_for_company(&mut tx, company_id, user_id, r1.id)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    assert!(transitioned);

    // Recréer r2 avec le même match → doit réussir (active_uniq de r1 = NULL).
    let mut tx = pool.begin().await.unwrap();
    let r2 = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 2 reuse same match",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            150,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert!(r2.id > r1.id, "r2 must be a distinct row");
    assert!(r2.active);
    assert_eq!(r2.label, "Rule 2 reuse same match");
}

// ---------------------------------------------------------------------------
// Test 4 — AC #109b Pass 1 P-H3 : réactivation d'une rule archivée
// alors qu'une rule active avec mêmes match existe → UNIQUE conflict.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_partial_conflicts_on_reactivation(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let r1 = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 1 (sera archivée)",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Soft-delete r1.
    let mut tx = pool.begin().await.unwrap();
    reconciliation_rules::soft_delete_by_id_for_company(&mut tx, company_id, user_id, r1.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Créer r2 active avec mêmes match → succès.
    let mut tx = pool.begin().await.unwrap();
    let r2 = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Rule 2 active",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            150,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // Tenter PATCH r1 { active: true } → doit violer UNIQUE (active_uniq
    // de r1 passe de NULL à "Swisscom", collide avec r2.active_uniq).
    let r1_after_delete = reconciliation_rules::find_by_id_for_company(&pool, company_id, r1.id)
        .await
        .unwrap()
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let res = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r1.id,
        r1_after_delete.version,
        &UpdateReconciliationRule {
            active: Some(true),
            ..Default::default()
        },
    )
    .await;
    drop(tx); // rollback implicite

    let err = res.expect_err("reactivation collides with r2 active_uniq");
    assert!(
        matches!(err, DbError::UniqueConstraintViolation(_)),
        "expected UniqueConstraintViolation on reactivation, got {err:?}"
    );
    assert!(
        reconciliation_rules::is_duplicate_rule_constraint(&err),
        "reactivation conflict must be recognized via is_duplicate_rule_constraint, got {err:?}"
    );

    // Sanity : r2 toujours présent.
    assert!(r2.active);
}

// ---------------------------------------------------------------------------
// Test 5 — AC #105 + #106 : list_by_company_paginated avec active_filter.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_filters_active(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_a = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;
    let account_b = create_test_account(&pool, company_id, user_id, "6520", "Postes").await;

    // 3 actives + 1 archivée.
    let mut tx = pool.begin().await.unwrap();
    let r1 = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "R1",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_a,
            100,
        ),
    )
    .await
    .unwrap();
    reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "R2",
            ReconciliationMatchType::CounterpartyContains,
            "Sunrise",
            account_a,
            200,
        ),
    )
    .await
    .unwrap();
    reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "R3",
            ReconciliationMatchType::ReferenceContains,
            "INV-",
            account_b,
            300,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    let mut tx = pool.begin().await.unwrap();
    reconciliation_rules::soft_delete_by_id_for_company(&mut tx, company_id, user_id, r1.id)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Sans filtre : 3 (R1 archivée + R2 + R3 actives).
    let (all, total_all) =
        reconciliation_rules::list_by_company_paginated(&pool, company_id, None, 1, 50)
            .await
            .unwrap();
    assert_eq!(total_all, 3);
    assert_eq!(all.len(), 3);

    // active_filter = Some(true) → 2.
    let (active, total_active) =
        reconciliation_rules::list_by_company_paginated(&pool, company_id, Some(true), 1, 50)
            .await
            .unwrap();
    assert_eq!(total_active, 2);
    assert_eq!(active.len(), 2);
    for r in &active {
        assert!(r.active, "{} doit être active", r.label);
    }

    // active_filter = Some(false) → 1 (R1 archivée).
    let (inactive, total_inactive) =
        reconciliation_rules::list_by_company_paginated(&pool, company_id, Some(false), 1, 50)
            .await
            .unwrap();
    assert_eq!(total_inactive, 1);
    assert_eq!(inactive.len(), 1);
    assert_eq!(inactive[0].label, "R1");
    assert!(!inactive[0].active);
}

// ---------------------------------------------------------------------------
// Test 6 — AC #108 : optimistic lock sur version.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_uses_optimistic_lock(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let r = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Initial",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(r.version, 1);

    // PATCH version=1 (correct) → version=2.
    let mut tx = pool.begin().await.unwrap();
    let r2 = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        1,
        &UpdateReconciliationRule {
            label: Some("Renamed".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(r2.version, 2);
    assert_eq!(r2.label, "Renamed");

    // PATCH version=1 (stale) → OptimisticLockConflict.
    let mut tx = pool.begin().await.unwrap();
    let res = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        1, // stale
        &UpdateReconciliationRule {
            label: Some("Other".into()),
            ..Default::default()
        },
    )
    .await;
    drop(tx);
    assert!(
        matches!(res, Err(DbError::OptimisticLockConflict)),
        "expected OptimisticLockConflict, got {res:?}"
    );
}

// ---------------------------------------------------------------------------
// Test 7 — AC #110 + #111 : soft_delete + idempotence.
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn soft_delete_sets_active_false_idempotent(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let r = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "To delete",
            ReconciliationMatchType::CounterpartyExact,
            "Acme",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert!(r.active);

    // Premier soft_delete → transition active=true→false (AC #110).
    let mut tx = pool.begin().await.unwrap();
    let transitioned_1 =
        reconciliation_rules::soft_delete_by_id_for_company(&mut tx, company_id, user_id, r.id)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    assert!(transitioned_1, "première transition doit être true→false");

    let after_1 = reconciliation_rules::find_by_id_for_company(&pool, company_id, r.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!after_1.active);
    assert_eq!(after_1.version, r.version + 1);

    // Second soft_delete → idempotent (AC #111), pas de nouvelle entrée audit.
    let mut tx = pool.begin().await.unwrap();
    let transitioned_2 =
        reconciliation_rules::soft_delete_by_id_for_company(&mut tx, company_id, user_id, r.id)
            .await
            .unwrap();
    tx.commit().await.unwrap();
    assert!(!transitioned_2, "second soft_delete must be idempotent");

    let after_2 = reconciliation_rules::find_by_id_for_company(&pool, company_id, r.id)
        .await
        .unwrap()
        .unwrap();
    assert!(!after_2.active);
    assert_eq!(
        after_2.version, after_1.version,
        "second soft_delete ne doit pas incrémenter version (NOP)"
    );

    // Audit log : exactement 2 entrées pour cette rule
    // (rule.created + rule.deleted), pas de doublon `deleted` (idempotence).
    let audit_deleted_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log \
         WHERE entity_type = 'reconciliation_rules' AND entity_id = ? \
         AND action = 'reconciliation_rule.deleted'",
    )
    .bind(r.id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        audit_deleted_count, 1,
        "exactement 1 audit deleted (pas 2 ; idempotence préservée)"
    );
}

// ---------------------------------------------------------------------------
// Test 8 — AC #118 partiel : increment_applied_count_in_tx atomique.
// Pass 1 code review HIGH AA1 fix : version est bumped dans le SET
// (pas de WHERE version=? clause — différent de l'optimistic lock CRUD).
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn increment_applied_count_atomic(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account_id = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;

    let mut tx = pool.begin().await.unwrap();
    let r = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Counter test",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account_id,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(r.applied_count, 0);
    assert!(r.last_applied_at.is_none());
    let initial_version = r.version;

    // 5 incréments atomiques séquentiels. La fonction retourne maintenant
    // applied_count post-incrément (Pass 1 HIGH AA2 fix pour audit
    // applied_count_after field).
    for expected in 1..=5 {
        let mut tx = pool.begin().await.unwrap();
        let count = reconciliation_rules::increment_applied_count_in_tx(&mut tx, company_id, r.id)
            .await
            .unwrap();
        tx.commit().await.unwrap();
        assert_eq!(
            count, expected,
            "applied_count_after doit refléter post-incrément"
        );
    }

    let after = reconciliation_rules::find_by_id_for_company(&pool, company_id, r.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(after.applied_count, 5);
    assert!(
        after.last_applied_at.is_some(),
        "last_applied_at doit être posée après le premier increment"
    );
    assert_eq!(
        after.version,
        initial_version + 5,
        "version DOIT être bumpée à chaque increment (Pass 1 HIGH AA1 fix — \
         requis par spec step 14 + scope-locked §5 + T2.2)"
    );
}

// ---------------------------------------------------------------------------
// Story 19-5 — projet analytique par défaut sur une règle (AC #3-#5, #20).
// ---------------------------------------------------------------------------

/// Create avec `default_project_id` valide → relu correctement.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_default_project_persists(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;
    let project_id = make_project(&pool, company_id, "RENOV", false).await;

    let mut tx = pool.begin().await.unwrap();
    let created = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule_with_project(
            "Swisscom projet",
            ReconciliationMatchType::CounterpartyContains,
            "Swisscom",
            account,
            100,
            Some(project_id),
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    assert_eq!(created.default_project_id, Some(project_id));

    let reloaded = reconciliation_rules::find_by_id_for_company(&pool, company_id, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(reloaded.default_project_id, Some(project_id));
}

/// Create avec projet archivé → `IllegalStateTransition` (mappé 409).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_archived_project_rejected(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;
    let project_id = make_project(&pool, company_id, "OLD", true).await;

    let mut tx = pool.begin().await.unwrap();
    let res = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule_with_project(
            "Règle projet clos",
            ReconciliationMatchType::CounterpartyContains,
            "X",
            account,
            100,
            Some(project_id),
        ),
    )
    .await;
    drop(tx);

    assert!(
        matches!(res, Err(DbError::IllegalStateTransition(_))),
        "projet archivé doit être rejeté, got {res:?}"
    );
}

/// Create avec projet inexistant OU cross-company → `NotFound` (mappé 404).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_with_unknown_or_cross_company_project_rejected(pool: MySqlPool) {
    let company_a = create_test_company(&pool, "Alpha SA").await;
    let company_b = create_test_company(&pool, "Beta SA").await;
    let user_a = create_test_user(&pool, "alice", company_a).await;
    let user_b = create_test_user(&pool, "bob", company_b).await;
    let account_a = create_test_account(&pool, company_a, user_a, "6510", "Telecom").await;
    // Projet appartenant à B, utilisé (à tort) par une règle de A.
    let project_b = make_project(&pool, company_b, "BPROJ", false).await;
    let _ = user_b;

    // Inexistant.
    let mut tx = pool.begin().await.unwrap();
    let res = reconciliation_rules::create_in_tx(
        &mut tx,
        company_a,
        user_a,
        &make_rule_with_project(
            "Règle projet fantôme",
            ReconciliationMatchType::CounterpartyContains,
            "X",
            account_a,
            100,
            Some(999_999),
        ),
    )
    .await;
    drop(tx);
    assert!(
        matches!(res, Err(DbError::NotFound)),
        "projet inexistant doit être rejeté NotFound, got {res:?}"
    );

    // Cross-company.
    let mut tx = pool.begin().await.unwrap();
    let res = reconciliation_rules::create_in_tx(
        &mut tx,
        company_a,
        user_a,
        &make_rule_with_project(
            "Règle projet d'autrui",
            ReconciliationMatchType::CounterpartyExact,
            "Y",
            account_a,
            100,
            Some(project_b),
        ),
    )
    .await;
    drop(tx);
    assert!(
        matches!(res, Err(DbError::NotFound)),
        "projet cross-company doit être rejeté NotFound, got {res:?}"
    );
}

/// Update qui change le projet vers un projet actif → validé et persisté ;
/// update qui touche un autre champ sans changer le projet dont le projet
/// stocké a été archivé entre-temps → passe (grandfathering, DC4).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_validates_new_project_but_grandfathers_unchanged(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Alpha SA").await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let account = create_test_account(&pool, company_id, user_id, "6510", "Telecom").await;
    let project_a = make_project(&pool, company_id, "PA", false).await;
    let project_b = make_project(&pool, company_id, "PB", false).await;

    // Règle sans projet.
    let mut tx = pool.begin().await.unwrap();
    let r = reconciliation_rules::create_in_tx(
        &mut tx,
        company_id,
        user_id,
        &make_rule(
            "Base",
            ReconciliationMatchType::CounterpartyContains,
            "Z",
            account,
            100,
        ),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();

    // (a) Update affecte project_a → validé.
    let mut tx = pool.begin().await.unwrap();
    let updated = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        r.version,
        &UpdateReconciliationRule {
            default_project_id: Some(Some(project_a)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(updated.default_project_id, Some(project_a));

    // (b) Archive project_a « après coup » puis update le libellé seul :
    // grandfathering — le projet inchangé n'est pas re-validé → pas d'erreur.
    sqlx::query("UPDATE projects SET archived = TRUE WHERE id = ?")
        .bind(project_a)
        .execute(&pool)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let relabeled = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        updated.version,
        &UpdateReconciliationRule {
            label: Some("Base renommée".into()),
            ..Default::default()
        },
    )
    .await
    .expect("update label seul ne re-valide pas le projet inchangé (grandfathering)");
    tx.commit().await.unwrap();
    assert_eq!(relabeled.label, "Base renommée");
    assert_eq!(relabeled.default_project_id, Some(project_a));

    // (c) Changer vers project_b (actif) reste possible.
    let mut tx = pool.begin().await.unwrap();
    let moved = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        relabeled.version,
        &UpdateReconciliationRule {
            default_project_id: Some(Some(project_b)),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(moved.default_project_id, Some(project_b));

    // (d) Effacer le projet (Some(None)) → NULL, pas de validation.
    let mut tx = pool.begin().await.unwrap();
    let cleared = reconciliation_rules::update_in_tx(
        &mut tx,
        company_id,
        user_id,
        r.id,
        moved.version,
        &UpdateReconciliationRule {
            default_project_id: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(cleared.default_project_id, None);
}
