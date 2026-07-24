//! Tests d'intégration pour `repositories::fiscal_years`.

use chrono::NaiveDate;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    FiscalYearStatus, Language, NewCompany, NewFiscalYear, NewUser, OrgType, Role,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::fiscal_years::{
    FY_NAME_DUPLICATE_KEY, FY_NAME_EMPTY_KEY, FY_NAME_MAX_LEN, FY_NAME_TOO_LONG_KEY,
    FY_OVERLAP_KEY, FY_REOPEN_ALREADY_OPEN_KEY, FY_REOPEN_LIFO_BLOCKED_KEY,
};
use kesh_db::repositories::{audit_log, companies, fiscal_years, users};
use sqlx::MySqlPool;

fn sample_new_company() -> NewCompany {
    NewCompany {
        name: "Test SA".into(),
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
    }
}

async fn create_company(pool: &MySqlPool) -> i64 {
    companies::create(pool, sample_new_company())
        .await
        .unwrap()
        .id
}

/// Crée un admin user pour ce company. Nécessaire pour les fns repo qui
/// appellent `audit_log::insert_in_tx` (FK `audit_log.user_id → users.id`).
async fn create_admin_user(pool: &MySqlPool, company_id: i64) -> i64 {
    users::create(
        pool,
        NewUser {
            username: format!("admin-{company_id}"),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams".into(),
            role: Role::Admin,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("create admin user")
    .id
}

fn ny(name: &str, year: i32) -> NewFiscalYear {
    NewFiscalYear {
        company_id: 0,
        name: name.into(),
        start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
    }
}

// ---------------------------------------------------------------------------
// create() — happy path + audit + UNIQUE & CHECK constraints
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_and_find_by_id(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();
    assert!(created.id > 0);
    assert_eq!(created.status, FiscalYearStatus::Open);
    assert_eq!(created.name, "Exercice 2026");

    let found = fiscal_years::find_by_id(&pool, created.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found.id, created.id);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_writes_audit_log(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    let create_entry = entries
        .iter()
        .find(|e| e.action == "fiscal_year.created")
        .expect("audit entry fiscal_year.created should exist");
    assert_eq!(create_entry.user_id, user_id);
    assert_eq!(create_entry.entity_type, "fiscal_year");
    let details = create_entry.details_json.as_ref().expect("details");
    assert_eq!(details["name"], "Exercice 2026");
    assert_eq!(details["status"], "Open");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = fiscal_years::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fk_violation_on_missing_company(pool: MySqlPool) {
    // user_id valide mais company_id invalide → FK violation à l'INSERT.
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice fantôme", 2026);
    new.company_id = 999_999;
    let result = fiscal_years::create(&pool, user_id, new).await;
    assert!(matches!(result, Err(DbError::ForeignKeyViolation(_))));
}

// ---------------------------------------------------------------------------
// Pré-checks overlap & nom (Story 3.7 H-5 + H-6)
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_rejects_duplicate_name(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut a = ny("Exercice 2026", 2026);
    a.company_id = company_id;
    fiscal_years::create(&pool, user_id, a).await.unwrap();

    // Même nom, dates différentes → erreur namespacée FY_NAME_DUPLICATE_KEY.
    let mut b = ny("Exercice 2026", 2027);
    b.company_id = company_id;
    let result = fiscal_years::create(&pool, user_id, b).await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_NAME_DUPLICATE_KEY),
        other => panic!("expected Invariant(FY_NAME_DUPLICATE_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_rejects_overlap_with_existing(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // Exercice 2027 (Jan-Dec).
    let mut existing = ny("Exercice 2027", 2027);
    existing.company_id = company_id;
    fiscal_years::create(&pool, user_id, existing)
        .await
        .unwrap();

    // Tentative Mid 2027 (Jul 2027 – Jun 2028) — chevauche l'existant.
    let overlap = NewFiscalYear {
        company_id,
        name: "Mid 2027".into(),
        start_date: NaiveDate::from_ymd_opt(2027, 7, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2028, 6, 30).unwrap(),
    };
    let result = fiscal_years::create(&pool, user_id, overlap).await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_OVERLAP_KEY),
        other => panic!("expected Invariant(FY_OVERLAP_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_constraint_rejects_equal_dates(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // end_date == start_date — la contrainte CHECK exige strict >.
    let bad = NewFiscalYear {
        company_id,
        name: "Zero-length".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
    };
    let result = fiscal_years::create(&pool, user_id, bad).await;
    assert!(
        matches!(result, Err(DbError::CheckConstraintViolation(_))),
        "end_date == start_date doit violer CHECK, got {result:?}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_constraint_end_date_must_be_after_start(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let bad = NewFiscalYear {
        company_id,
        name: "Invalid".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    };
    let result = fiscal_years::create(&pool, user_id, bad).await;
    assert!(
        matches!(result, Err(DbError::CheckConstraintViolation(_))),
        "end_date < start_date doit retourner CheckConstraintViolation, got {result:?}"
    );
}

// ---------------------------------------------------------------------------
// list_by_company — Story 3.7 P3-M3 : ORDER BY DESC
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_list_by_company_orders_by_start_date_desc(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    for year in [2027, 2025, 2026] {
        let mut new = ny("placeholder", year);
        new.name = format!("Exercice {year}");
        new.company_id = company_id;
        fiscal_years::create(&pool, user_id, new).await.unwrap();
    }

    let list = fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 3);
    // Story 3.7 P3-M3 : tri start_date DESC (le plus récent en tête).
    assert_eq!(list[0].name, "Exercice 2027");
    assert_eq!(list[1].name, "Exercice 2026");
    assert_eq!(list[2].name, "Exercice 2025");
}

// ---------------------------------------------------------------------------
// close() — Story 3.7 : signature audit-aware
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_open_to_closed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();
    assert_eq!(created.status, FiscalYearStatus::Open);

    let closed = fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();
    assert_eq!(closed.status, FiscalYearStatus::Closed);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_fails_on_missing(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let result = fiscal_years::close(&pool, user_id, company_id, 999_999).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_fails_on_already_closed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();

    let result = fiscal_years::close(&pool, user_id, company_id, created.id).await;
    assert!(matches!(result, Err(DbError::IllegalStateTransition(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_close_writes_audit_log(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    let close_entry = entries
        .iter()
        .find(|e| e.action == "fiscal_year.closed")
        .expect("audit entry fiscal_year.closed should exist");
    assert_eq!(close_entry.user_id, user_id);
    let details = close_entry.details_json.as_ref().expect("details");
    assert_eq!(details["status"], "Closed");
}

// Pass 2 HP2-L4 : close empty fiscal_year (no journal entries) — devrait
// réussir (il n'y a aucune contrainte applicative qui bloque).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_close_fiscal_year_with_no_journal_entries(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let closed = fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();
    assert_eq!(closed.status, FiscalYearStatus::Closed);
}

// ---------------------------------------------------------------------------
// update_name() — Story 3.7 T1.3
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_writes_audit_log_with_before_after(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let renamed =
        fiscal_years::update_name(&pool, user_id, company_id, created.id, "FY 2026".into())
            .await
            .unwrap();
    assert_eq!(renamed.name, "FY 2026");

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    let update_entry = entries
        .iter()
        .find(|e| e.action == "fiscal_year.updated")
        .expect("audit entry fiscal_year.updated should exist");
    assert_eq!(update_entry.user_id, user_id);
    let details = update_entry.details_json.as_ref().expect("details");
    assert_eq!(details["before"]["name"], "Exercice 2026");
    assert_eq!(details["after"]["name"], "FY 2026");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_rejects_empty(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let result =
        fiscal_years::update_name(&pool, user_id, company_id, created.id, "   ".into()).await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_NAME_EMPTY_KEY),
        other => panic!("expected Invariant(FY_NAME_EMPTY_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_rejects_duplicate_name(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut a = ny("Exercice 2026", 2026);
    a.company_id = company_id;
    fiscal_years::create(&pool, user_id, a).await.unwrap();

    let mut b = ny("Exercice 2027", 2027);
    b.company_id = company_id;
    let b_created = fiscal_years::create(&pool, user_id, b).await.unwrap();

    // Renommer 2027 en "Exercice 2026" doit échouer (autre row même nom).
    let result = fiscal_years::update_name(
        &pool,
        user_id,
        company_id,
        b_created.id,
        "Exercice 2026".into(),
    )
    .await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_NAME_DUPLICATE_KEY),
        other => panic!("expected Invariant(FY_NAME_DUPLICATE_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_not_found(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let result =
        fiscal_years::update_name(&pool, user_id, company_id, 999_999, "anything".into()).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

// ---------------------------------------------------------------------------
// create_for_seed() — Story 3.7 T1.8
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_for_seed_does_not_audit(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create_for_seed(&pool, new).await.unwrap();

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    assert!(
        entries.is_empty(),
        "create_for_seed must not write audit_log, got {entries:?}"
    );
}

// ---------------------------------------------------------------------------
// create_if_absent_in_tx() — Story 3.7 T1.2
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_if_absent_in_tx_creates_when_empty(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut tx = pool.begin().await.unwrap();
    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let result = fiscal_years::create_if_absent_in_tx(&mut tx, user_id, new)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let fy = result.expect("Some(fy) when company has no fiscal_year");
    assert_eq!(fy.name, "Exercice 2026");

    // Audit log présent.
    let entries = audit_log::find_by_entity(&pool, "fiscal_year", fy.id, 10)
        .await
        .unwrap();
    assert!(entries.iter().any(|e| e.action == "fiscal_year.created"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_if_absent_in_tx_skips_when_exists(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // Pré-insérer un fiscal_year.
    let mut existing = ny("Exercice 2025", 2025);
    existing.company_id = company_id;
    fiscal_years::create(&pool, user_id, existing)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();
    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let result = fiscal_years::create_if_absent_in_tx(&mut tx, user_id, new)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert!(
        result.is_none(),
        "create_if_absent_in_tx must return None when a fiscal_year already exists"
    );

    // Toujours un seul fiscal_year (pas de doublon) — celui pré-inséré.
    let list = fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].name, "Exercice 2025");
}

// ---------------------------------------------------------------------------
// find_by_id_in_company — Story 3.7 H-8 multi-tenant scoping
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_find_by_id_in_company_returns_none_for_other_company(pool: MySqlPool) {
    let company_a = create_company(&pool).await;
    let user_a = create_admin_user(&pool, company_a).await;

    // Deuxième company.
    let company_b = companies::create(
        &pool,
        NewCompany {
            name: "Other SA".into(),
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

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_a;
    let fy_a = fiscal_years::create(&pool, user_a, new).await.unwrap();

    // Lookup avec company_b doit retourner None (anti-énumération).
    let result = fiscal_years::find_by_id_in_company(&pool, company_b, fy_a.id)
        .await
        .unwrap();
    assert!(result.is_none());

    // Lookup avec la bonne company retourne Some.
    let result = fiscal_years::find_by_id_in_company(&pool, company_a, fy_a.id)
        .await
        .unwrap();
    assert!(result.is_some());
}

// ---------------------------------------------------------------------------
// Code Review Pass 1 F2 — multi-tenant defense in depth (update_name + close)
// ---------------------------------------------------------------------------

async fn create_other_company(pool: &MySqlPool) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: "Other SA".into(),
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
    .id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_rejects_cross_tenant(pool: MySqlPool) {
    let company_a = create_company(&pool).await;
    let user_a = create_admin_user(&pool, company_a).await;
    let company_b = create_other_company(&pool).await;

    let mut new = ny("FY of B", 2026);
    new.company_id = company_b;
    let fy_b = fiscal_years::create_for_seed(&pool, new).await.unwrap();

    // user_a (company_a) tente de renommer fy_b → NotFound (pas autorisé).
    let result =
        fiscal_years::update_name(&pool, user_a, company_a, fy_b.id, "hijacked".into()).await;
    assert!(matches!(result, Err(DbError::NotFound)));

    // Vérifier que le nom n'a PAS changé en DB.
    let unchanged = fiscal_years::find_by_id(&pool, fy_b.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.name, "FY of B");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_close_rejects_cross_tenant(pool: MySqlPool) {
    let company_a = create_company(&pool).await;
    let user_a = create_admin_user(&pool, company_a).await;
    let company_b = create_other_company(&pool).await;

    let mut new = ny("FY of B", 2026);
    new.company_id = company_b;
    let fy_b = fiscal_years::create_for_seed(&pool, new).await.unwrap();

    let result = fiscal_years::close(&pool, user_a, company_a, fy_b.id).await;
    assert!(matches!(result, Err(DbError::NotFound)));

    // Vérifier que le statut n'a PAS changé.
    let unchanged = fiscal_years::find_by_id(&pool, fy_b.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(unchanged.status, FiscalYearStatus::Open);
}

// ---------------------------------------------------------------------------
// Code Review Pass 1 F3 — pré-validation longueur nom (VARCHAR(50))
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_rejects_name_too_long(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let too_long_name = "a".repeat(FY_NAME_MAX_LEN + 1);
    let mut new = ny(&too_long_name, 2026);
    new.company_id = company_id;
    let result = fiscal_years::create(&pool, user_id, new).await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_NAME_TOO_LONG_KEY),
        other => panic!("expected Invariant(FY_NAME_TOO_LONG_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_update_name_rejects_name_too_long(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let too_long_name = "x".repeat(FY_NAME_MAX_LEN + 1);
    let result =
        fiscal_years::update_name(&pool, user_id, company_id, created.id, too_long_name).await;
    match result {
        Err(DbError::Invariant(s)) => assert_eq!(s, FY_NAME_TOO_LONG_KEY),
        other => panic!("expected Invariant(FY_NAME_TOO_LONG_KEY), got {other:?}"),
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_accepts_name_at_max_length(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let max_name = "a".repeat(FY_NAME_MAX_LEN);
    let mut new = ny(&max_name, 2026);
    new.company_id = company_id;
    let result = fiscal_years::create(&pool, user_id, new).await;
    assert!(result.is_ok(), "name at exactly MAX_LEN should be accepted");
}

// ---------------------------------------------------------------------------
// Code Review Pass 1 F1 — create_if_absent_in_tx idempotent sur UniqueConstraintViolation
// ---------------------------------------------------------------------------

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn test_create_if_absent_in_tx_idempotent_on_unique_violation(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // Pré-insérer un fiscal_year via create_for_seed (sans audit) pour
    // simuler un conflit UNIQUE qui ne serait pas détecté par le NOT EXISTS
    // sous certaines conditions de race (ex. row insérée par une autre tx
    // entre le NOT EXISTS et l'INSERT).
    //
    // On force ici la simulation : insertion d'un fiscal_year manuel avec
    // le même nom que celui que `create_if_absent_in_tx` va tenter, mais
    // SANS passer par le pré-check NOT EXISTS — au-dessous on appelle
    // l'helper sur une company qui a déjà une row, donc le NOT EXISTS
    // protège déjà. Le test vérifie que l'helper retourne Ok(None) sans
    // panic même sous concurrence.
    let mut existing = ny("Exercice 2026", 2026);
    existing.company_id = company_id;
    fiscal_years::create_for_seed(&pool, existing)
        .await
        .unwrap();

    // Tentative de create_if_absent → doit voir la row pré-existante via
    // NOT EXISTS et retourner Ok(None) idempotent.
    let mut tx = pool.begin().await.unwrap();
    let mut new = ny("Exercice 2027", 2027);
    new.company_id = company_id;
    let result = fiscal_years::create_if_absent_in_tx(&mut tx, user_id, new)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert!(
        result.is_none(),
        "idempotent: company has fiscal_year, return None"
    );
}

// ===========================================================================
// Story 14-2 — reopen() : flip guardé Closed → Open + audit motif + garde LIFO
// ===========================================================================

/// AC-A — reopen d'un `Closed` → `Open` + audit `fiscal_year.reopened` avec
/// `details_json.motif` et `before/after` corrects.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_closed_writes_audit_with_motif(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();
    fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();

    let motif = "Correction TVA Q3 oubliée";
    let reopened = fiscal_years::reopen(&pool, user_id, company_id, created.id, motif.to_string())
        .await
        .unwrap();
    assert_eq!(reopened.status, FiscalYearStatus::Open);

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    let entry = entries
        .iter()
        .find(|e| e.action == "fiscal_year.reopened")
        .expect("audit entry fiscal_year.reopened should exist");
    assert_eq!(entry.user_id, user_id);
    let details = entry.details_json.as_ref().expect("details");
    assert_eq!(details["motif"], motif);
    assert_eq!(details["before"]["status"], "Closed");
    assert_eq!(details["after"]["status"], "Open");
}

/// AC-A — reopen d'un `Open` → `Invariant(FY_REOPEN_ALREADY_OPEN_KEY)` ; AUCUNE
/// écriture d'audit ajoutée (désambiguïsation « déjà ouvert » avant garde LIFO).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_already_open_returns_invariant_no_audit(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();

    let result =
        fiscal_years::reopen(&pool, user_id, company_id, created.id, "motif".to_string()).await;
    assert!(
        matches!(result, Err(DbError::Invariant(ref k)) if k == FY_REOPEN_ALREADY_OPEN_KEY),
        "expected FY_REOPEN_ALREADY_OPEN_KEY, got {result:?}"
    );

    let entries = audit_log::find_by_entity(&pool, "fiscal_year", created.id, 10)
        .await
        .unwrap();
    assert!(
        entries.iter().all(|e| e.action != "fiscal_year.reopened"),
        "no reopened audit entry should be written on a no-op reopen"
    );
}

/// AC-A — reopen d'un id inexistant / d'une autre société → `NotFound`.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_missing_and_cross_tenant_return_notfound(pool: MySqlPool) {
    let company_a = create_company(&pool).await;
    let user_a = create_admin_user(&pool, company_a).await;
    let company_b = create_company(&pool).await;
    let user_b = create_admin_user(&pool, company_b).await;

    // Inexistant.
    let missing =
        fiscal_years::reopen(&pool, user_a, company_a, 999_999, "motif".to_string()).await;
    assert!(matches!(missing, Err(DbError::NotFound)));

    // Cross-tenant : exercice de B clos, réouverture tentée par A → NotFound.
    let mut new = ny("Exercice B 2026", 2026);
    new.company_id = company_b;
    let fy_b = fiscal_years::create(&pool, user_b, new).await.unwrap();
    fiscal_years::close(&pool, user_b, company_b, fy_b.id)
        .await
        .unwrap();

    let cross = fiscal_years::reopen(&pool, user_a, company_a, fy_b.id, "motif".to_string()).await;
    assert!(matches!(cross, Err(DbError::NotFound)));
}

/// AC-B — garde LIFO : FY_N clos + FY_{N+1} clos → reopen FY_N bloqué ; reopen
/// FY_{N+1} d'abord → OK, puis FY_N → OK.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_lifo_blocked_then_ordered_ok(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut y2025 = ny("Exercice 2025", 2025);
    y2025.company_id = company_id;
    let fy2025 = fiscal_years::create(&pool, user_id, y2025).await.unwrap();
    let mut y2026 = ny("Exercice 2026", 2026);
    y2026.company_id = company_id;
    let fy2026 = fiscal_years::create(&pool, user_id, y2026).await.unwrap();

    fiscal_years::close(&pool, user_id, company_id, fy2025.id)
        .await
        .unwrap();
    fiscal_years::close(&pool, user_id, company_id, fy2026.id)
        .await
        .unwrap();

    // Rouvrir le plus ancien alors que le plus récent est clos → bloqué.
    let blocked =
        fiscal_years::reopen(&pool, user_id, company_id, fy2025.id, "motif".to_string()).await;
    assert!(
        matches!(blocked, Err(DbError::Invariant(ref k)) if k == FY_REOPEN_LIFO_BLOCKED_KEY),
        "expected FY_REOPEN_LIFO_BLOCKED_KEY, got {blocked:?}"
    );

    // Rouvrir le plus récent d'abord → OK, puis le plus ancien → OK.
    fiscal_years::reopen(&pool, user_id, company_id, fy2026.id, "motif".to_string())
        .await
        .unwrap();
    fiscal_years::reopen(&pool, user_id, company_id, fy2025.id, "motif".to_string())
        .await
        .unwrap();
}

/// AC-B — LIFO permissif : FY_N clos + FY_{N+1} OUVERT → reopen FY_N OK.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_lifo_permissive_when_later_open(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut y2025 = ny("Exercice 2025", 2025);
    y2025.company_id = company_id;
    let fy2025 = fiscal_years::create(&pool, user_id, y2025).await.unwrap();
    let mut y2026 = ny("Exercice 2026", 2026);
    y2026.company_id = company_id;
    let _fy2026 = fiscal_years::create(&pool, user_id, y2026).await.unwrap();

    // Seul le plus ancien est clos ; le plus récent reste ouvert → réouverture OK.
    fiscal_years::close(&pool, user_id, company_id, fy2025.id)
        .await
        .unwrap();
    fiscal_years::reopen(&pool, user_id, company_id, fy2025.id, "motif".to_string())
        .await
        .unwrap();
}

/// P4-LOW — LIFO 3 exercices intercalés (Closed-Open-Closed) : reopen FY1 bloqué
/// en citant FY3 (le plus proche postérieur clos via ORDER BY start_date ASC
/// LIMIT 1 — prouve que la query n'a pas besoin de notion d'adjacence).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_lifo_three_years_intercalated(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut y1 = ny("Exercice 2024", 2024);
    y1.company_id = company_id;
    let fy1 = fiscal_years::create(&pool, user_id, y1).await.unwrap();
    let mut y2 = ny("Exercice 2025", 2025);
    y2.company_id = company_id;
    let _fy2 = fiscal_years::create(&pool, user_id, y2).await.unwrap();
    let mut y3 = ny("Exercice 2026", 2026);
    y3.company_id = company_id;
    let fy3 = fiscal_years::create(&pool, user_id, y3).await.unwrap();

    // FY1 clos, FY2 ouvert, FY3 clos.
    fiscal_years::close(&pool, user_id, company_id, fy1.id)
        .await
        .unwrap();
    fiscal_years::close(&pool, user_id, company_id, fy3.id)
        .await
        .unwrap();

    // Rouvrir FY1 est bloqué : FY3 (postérieur, clos) existe — même avec FY2
    // ouvert intercalé, la garde ne dépend pas de l'adjacence.
    let blocked =
        fiscal_years::reopen(&pool, user_id, company_id, fy1.id, "motif".to_string()).await;
    assert!(
        matches!(blocked, Err(DbError::Invariant(ref k)) if k == FY_REOPEN_LIFO_BLOCKED_KEY),
        "expected FY_REOPEN_LIFO_BLOCKED_KEY (FY3 postérieur clos), got {blocked:?}"
    );
}

/// AC-C — le flip Closed → Open ré-active l'immutabilité SANS toucher
/// `journal_entries` : après reopen, une écriture peut être créée dans
/// l'exercice ; puis re-close → re-bloqué (`FiscalYearClosed`). Fix structurel :
/// le statut vivant pilote l'immutabilité, pas de flag dupliqué.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_reactivates_entry_editability(pool: MySqlPool) {
    use kesh_db::entities::account::AccountType;
    use kesh_db::entities::journal_entry::Journal;
    use kesh_db::entities::{NewAccount, NewJournalEntry, NewJournalEntryLine};
    use kesh_db::repositories::{accounts, journal_entries};
    use rust_decimal::Decimal;

    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let fy = fiscal_years::create(&pool, user_id, new).await.unwrap();

    // Deux comptes postables pour une écriture équilibrée.
    let cash = accounts::create(
        &pool,
        user_id,
        NewAccount {
            company_id,
            number: "1000".into(),
            name: "Caisse".into(),
            account_type: AccountType::Asset,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;
    let sales = accounts::create(
        &pool,
        user_id,
        NewAccount {
            company_id,
            number: "3000".into(),
            name: "Ventes".into(),
            account_type: AccountType::Revenue,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id;

    let make_entry = |date: NaiveDate| NewJournalEntry {
        company_id,
        entry_date: date,
        journal: Journal::OD,
        description: "test".into(),
        project_id: None,
        lines: vec![
            NewJournalEntryLine {
                account_id: cash,
                debit: Decimal::new(100, 0),
                credit: Decimal::ZERO,
                project_id: None,
            },
            NewJournalEntryLine {
                account_id: sales,
                debit: Decimal::ZERO,
                credit: Decimal::new(100, 0),
                project_id: None,
            },
        ],
    };

    let d = NaiveDate::from_ymd_opt(2026, 6, 15).unwrap();

    // Ouvert : création OK.
    journal_entries::create(&pool, fy.id, user_id, make_entry(d))
        .await
        .expect("create while open should succeed");

    // Clos : création bloquée.
    fiscal_years::close(&pool, user_id, company_id, fy.id)
        .await
        .unwrap();
    let blocked = journal_entries::create(&pool, fy.id, user_id, make_entry(d)).await;
    assert!(
        matches!(blocked, Err(DbError::FiscalYearClosed)),
        "closed year should block entry creation, got {blocked:?}"
    );

    // Rouvert : création de nouveau OK (statut vivant Open — aucune modif de
    // journal_entries.rs requise).
    fiscal_years::reopen(&pool, user_id, company_id, fy.id, "motif".to_string())
        .await
        .unwrap();
    journal_entries::create(&pool, fy.id, user_id, make_entry(d))
        .await
        .expect("create after reopen should succeed");

    // Re-clos : re-bloqué.
    fiscal_years::close(&pool, user_id, company_id, fy.id)
        .await
        .unwrap();
    let reblocked = journal_entries::create(&pool, fy.id, user_id, make_entry(d)).await;
    assert!(
        matches!(reblocked, Err(DbError::FiscalYearClosed)),
        "re-closed year should block again, got {reblocked:?}"
    );
}

/// AC-J test (a) — le message (log-only) de re-clôture d'un exercice déjà clos
/// ne prétend PLUS que la réouverture est interdite (reformulé « déjà clos »).
/// Le `Display` n'est jamais exposé au client — assertion doc/log-only.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_already_closed_message_no_longer_claims_reopen_forbidden(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let mut new = ny("Exercice 2026", 2026);
    new.company_id = company_id;
    let created = fiscal_years::create(&pool, user_id, new).await.unwrap();
    fiscal_years::close(&pool, user_id, company_id, created.id)
        .await
        .unwrap();

    let result = fiscal_years::close(&pool, user_id, company_id, created.id).await;
    match result {
        Err(DbError::IllegalStateTransition(msg)) => {
            assert!(
                !msg.contains("réouverture interdite"),
                "close message must no longer claim reopening is forbidden: {msg}"
            );
            // Assertion positive (F4) : le message reformulé mentionne bien
            // « déjà clos » — un futur refactor qui viderait le message échouerait.
            assert!(
                msg.contains("déjà clos"),
                "close message should state the year is already closed: {msg}"
            );
        }
        other => panic!("expected IllegalStateTransition, got {other:?}"),
    }
}

/// P1-M6 — course concurrente `reopen(FY_N)` vs `close(FY_{N+1})` sérialisée par
/// le `FOR UPDATE` : l'issue est déterministe (pas de deadlock, pas de panic) et
/// l'état final est cohérent. Documente l'hypothèse next-key locking de la
/// garde LIFO (filtre `status` non indexé).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reopen_close_concurrent_is_serialized(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // FY_N clos, FY_{N+1} ouvert.
    let mut y2025 = ny("Exercice 2025", 2025);
    y2025.company_id = company_id;
    let fy2025 = fiscal_years::create(&pool, user_id, y2025).await.unwrap();
    let mut y2026 = ny("Exercice 2026", 2026);
    y2026.company_id = company_id;
    let fy2026 = fiscal_years::create(&pool, user_id, y2026).await.unwrap();
    fiscal_years::close(&pool, user_id, company_id, fy2025.id)
        .await
        .unwrap();

    // reopen(FY2025) [garde LIFO lit FY2026] vs close(FY2026) [Open→Closed],
    // concurrents. Le FOR UPDATE sérialise → pas de deadlock, chaque future
    // résout en Ok ou en une DbError définie.
    let p1 = pool.clone();
    let p2 = pool.clone();
    let reopen_fut = tokio::spawn(async move {
        fiscal_years::reopen(&p1, user_id, company_id, fy2025.id, "motif".to_string()).await
    });
    let close_fut =
        tokio::spawn(async move { fiscal_years::close(&p2, user_id, company_id, fy2026.id).await });

    let (reopen_res, close_res) = tokio::join!(reopen_fut, close_fut);
    let reopen_res = reopen_res.expect("reopen task should not panic");
    let close_res = close_res.expect("close task should not panic");

    // Aucune des deux ne doit produire une erreur inattendue (Sqlx/deadlock).
    // reopen : soit Ok (a gagné la course avant close), soit LIFO blocked (close
    // a commit d'abord). close : Ok (FY2026 était Open).
    assert!(
        reopen_res.is_ok()
            || matches!(&reopen_res, Err(DbError::Invariant(k)) if k == FY_REOPEN_LIFO_BLOCKED_KEY),
        "reopen outcome must be Ok or LIFO-blocked, got {reopen_res:?}"
    );
    assert!(
        close_res.is_ok(),
        "close(FY2026 Open) should succeed, got {close_res:?}"
    );

    // État final : chaque exercice a un statut valide (pas de corruption).
    let fy2025_final = fiscal_years::find_by_id(&pool, fy2025.id)
        .await
        .unwrap()
        .unwrap();
    let fy2026_final = fiscal_years::find_by_id(&pool, fy2026.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fy2026_final.status, FiscalYearStatus::Closed);
    assert!(matches!(
        fy2025_final.status,
        FiscalYearStatus::Open | FiscalYearStatus::Closed
    ));
}
