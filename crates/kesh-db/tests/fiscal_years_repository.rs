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
use kesh_db::retry::retry_on_deadlock;
use kesh_db::test_fixtures::attendre_une_requete_en_cours;
use sqlx::MySqlPool;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

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
/// appellent `audit_log::insert_in_tx`.
///
/// ⚠️ Ce n'est plus une contrainte de FK — retirée par la Story 25-1a (#376) —
/// mais le libellé d'acteur (`actor_label`) est résolu par sous-SELECT sur
/// `users` à l'insertion : sans utilisateur, l'entrée porterait « (inconnu) ».
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = fiscal_years::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
async fn close_fails_on_missing(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    let result = fiscal_years::close(&pool, user_id, company_id, 999_999).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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

#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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
///
/// ⚠️ Story 15-12a : « FY2 ouvert sous FY3 clos » est l'état **hérité** que la
/// clôture refuse désormais de produire — FY3 est donc clos **par SQL**. Le test
/// garde la garde LIFO pour les données qui portent déjà cet état.
#[sqlx::test(migrations = "./test-schema")]
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

    // FY1 clos, FY2 ouvert, FY3 clos — ce dernier par SQL (état hérité).
    fiscal_years::close(&pool, user_id, company_id, fy1.id)
        .await
        .unwrap();
    poser_clos(&pool, fy3.id).await;

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
#[sqlx::test(migrations = "./test-schema")]
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
#[sqlx::test(migrations = "./test-schema")]
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

// ---------------------------------------------------------------------------
// find_first_by_company() — Story 14-4 (bilan d'ouverture)
// ---------------------------------------------------------------------------

/// Plusieurs exercices → celui de `start_date` la plus ancienne (tri ASC,
/// l'inverse de `list_by_company` qui trie DESC — Piège 8).
#[sqlx::test(migrations = "./test-schema")]
async fn find_first_by_company_returns_oldest_start_date(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;

    // Création volontairement dans le désordre chronologique.
    for year in [2026, 2024, 2025] {
        let mut new = ny(&format!("Exercice {year}"), year);
        new.company_id = company_id;
        fiscal_years::create(&pool, user_id, new).await.unwrap();
    }

    let first = fiscal_years::find_first_by_company(&pool, company_id)
        .await
        .unwrap()
        .expect("premier exercice");
    assert_eq!(first.name, "Exercice 2024");
    assert_eq!(
        first.start_date,
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()
    );
}

/// Aucun exercice → `None` (le handler mappe en `NO_FISCAL_YEAR`).
#[sqlx::test(migrations = "./test-schema")]
async fn find_first_by_company_none_when_empty(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let result = fiscal_years::find_first_by_company(&pool, company_id)
        .await
        .unwrap();
    assert!(result.is_none());
}

/// Scope multi-tenant : les exercices d'une autre company ne fuient pas.
///
/// Note tie-break (P1-L-3) : le `ORDER BY start_date ASC, id ASC` porte un
/// tie-break `id ASC` défensif, mais l'état « deux exercices de même
/// `start_date` dans une company » est INATTEIGNABLE sous la contrainte
/// `uq_fiscal_years_company_start_date UNIQUE (company_id, start_date)` —
/// aucune fixture légitime ne peut l'exercer. Ce test vérifie à la place que
/// le scoping company isole bien deux exercices de même `start_date` posés
/// dans deux companies distinctes.
#[sqlx::test(migrations = "./test-schema")]
async fn find_first_by_company_is_tenant_scoped(pool: MySqlPool) {
    let company_a = create_company(&pool).await;
    let user_a = create_admin_user(&pool, company_a).await;
    let company_b = companies::create(
        &pool,
        NewCompany {
            name: "Autre SA".into(),
            ..sample_new_company()
        },
    )
    .await
    .unwrap()
    .id;
    let user_b = create_admin_user(&pool, company_b).await;

    // Company B possède un exercice PLUS ANCIEN (2020) que le premier de A (2025).
    let mut new_b = ny("Exercice B 2020", 2020);
    new_b.company_id = company_b;
    fiscal_years::create(&pool, user_b, new_b).await.unwrap();

    let mut new_a = ny("Exercice A 2025", 2025);
    new_a.company_id = company_a;
    fiscal_years::create(&pool, user_a, new_a).await.unwrap();

    let first_a = fiscal_years::find_first_by_company(&pool, company_a)
        .await
        .unwrap()
        .expect("premier exercice de A");
    assert_eq!(first_a.name, "Exercice A 2025");
    assert_eq!(first_a.company_id, company_a);
}

// ---------------------------------------------------------------------------
// Story 15-12a (#543) — les exercices se clôturent dans l'ordre (invariant I)
// ---------------------------------------------------------------------------

/// Pose `status = 'Closed'` **par SQL**, hors de `close` : le seul moyen, depuis
/// la Story 15-12a, de fabriquer l'état hérité « exercice ouvert suivi d'un
/// exercice clos ».
async fn poser_clos(pool: &MySqlPool, id: i64) {
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

/// Crée l'exercice `année` (1er janvier – 31 décembre) de la société.
async fn exercice(pool: &MySqlPool, user_id: i64, company_id: i64, annee: i32) -> i64 {
    let mut new = ny(&format!("Exercice {annee}"), annee);
    new.company_id = company_id;
    fiscal_years::create(pool, user_id, new).await.unwrap().id
}

async fn statut(pool: &MySqlPool, id: i64) -> Option<FiscalYearStatus> {
    fiscal_years::find_by_id(pool, id)
        .await
        .unwrap()
        .map(|fy| fy.status)
}

/// Nombre de lignes d'audit `action` pour l'exercice `id`.
async fn audits(pool: &MySqlPool, action: &str, id: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = ? AND entity_type = 'fiscal_year' \
         AND entity_id = ?",
    )
    .bind(action)
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// Nombre d'exercices de la société.
async fn nombre_exercices(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM fiscal_years WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Motif `attendre_une_requete_en_cours` de l'étape (c) de `close` — le verrou
/// de l'exercice par clé primaire (`LOCK_IN_COMPANY_SQL`) : sa liste de colonnes
/// l'oppose à l'étape (b'), `WHERE id = ` aux requêtes par société, `FOR UPDATE`
/// à l'étape (a).
const MOTIF_ETAPE_C: &[&str] = &["SELECT id, company_id", "WHERE id = ", "FOR UPDATE"];

/// Motif du pré-contrôle de chevauchement de la création (`find_overlapping`) —
/// `end_date >= ` l'oppose à toutes les requêtes de `close`.
const MOTIF_CHEVAUCHEMENT: &[&str] = &["end_date >= ", "LIMIT 1 FOR UPDATE"];

/// AC 1 — refus : un exercice antérieur est ouvert → `EarlierFiscalYearOpen`,
/// **rien d'écrit** (ni statut, ni audit `fiscal_year.closed`).
///
/// ⛔ **Tue la mutation (i)** (retirer la garde de `close`).
#[sqlx::test(migrations = "./test-schema")]
async fn close_is_refused_while_an_earlier_year_is_open(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y2025 = exercice(&pool, user_id, company_id, 2025).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;

    let result = fiscal_years::close(&pool, user_id, company_id, y2026).await;
    match result {
        Err(DbError::EarlierFiscalYearOpen {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            assert_eq!(fiscal_year_id, y2025);
            assert_eq!(fiscal_year_name, "Exercice 2025");
        }
        other => panic!("attendu EarlierFiscalYearOpen, obtenu {other:?}"),
    }
    assert_eq!(statut(&pool, y2026).await, Some(FiscalYearStatus::Open));
    assert_eq!(audits(&pool, "fiscal_year.closed", y2026).await, 0);

    // Dans l'ordre, les deux passent.
    fiscal_years::close(&pool, user_id, company_id, y2025)
        .await
        .unwrap();
    fiscal_years::close(&pool, user_id, company_id, y2026)
        .await
        .unwrap();
    assert_eq!(statut(&pool, y2026).await, Some(FiscalYearStatus::Closed));
}

/// AC 1 — le refus nomme le **plus ancien** antérieur ouvert, et le trouve par
/// la **date**, non par l'`id` : 2024 est créé **après** 2025, ses `id` sont
/// donc inversés par rapport aux dates.
#[sqlx::test(migrations = "./test-schema")]
async fn close_names_the_oldest_earlier_open_year_whatever_the_ids(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y2025 = exercice(&pool, user_id, company_id, 2025).await;
    let y2024 = exercice(&pool, user_id, company_id, 2024).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;
    assert!(y2024 > y2025, "montage : 2024 doit avoir le plus grand id");

    let result = fiscal_years::close(&pool, user_id, company_id, y2026).await;
    assert!(
        matches!(&result, Err(DbError::EarlierFiscalYearOpen { fiscal_year_id, fiscal_year_name })
            if *fiscal_year_id == y2024 && fiscal_year_name == "Exercice 2024"),
        "attendu EarlierFiscalYearOpen nommant 2024, obtenu {result:?}"
    );
    // 2025 se clôture-t-il ? Non : 2024 le précède, ouvert.
    let result = fiscal_years::close(&pool, user_id, company_id, y2025).await;
    assert!(
        matches!(&result, Err(DbError::EarlierFiscalYearOpen { fiscal_year_id, .. }) if *fiscal_year_id == y2024),
        "attendu EarlierFiscalYearOpen nommant 2024, obtenu {result:?}"
    );
}

/// AC 1 — les exercices d'une **autre société** sont ignorés.
#[sqlx::test(migrations = "./test-schema")]
async fn close_ignores_the_open_years_of_another_company(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let other = create_other_company(&pool).await;
    let other_user = create_admin_user(&pool, other).await;
    let _autre_2025 = exercice(&pool, other_user, other, 2025).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;

    fiscal_years::close(&pool, user_id, company_id, y2026)
        .await
        .unwrap();
    assert_eq!(statut(&pool, y2026).await, Some(FiscalYearStatus::Closed));
}

/// AC 2 — précédence, paire `NotFound` > `EarlierFiscalYearOpen` : un exercice
/// inexistant, ou d'une autre société, rend `NotFound` même si la société a un
/// exercice ouvert antérieur à tout.
#[sqlx::test(migrations = "./test-schema")]
async fn close_precedence_not_found_before_earlier_open(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let _y2020 = exercice(&pool, user_id, company_id, 2020).await;
    let other = create_other_company(&pool).await;
    let other_user = create_admin_user(&pool, other).await;
    let autre_2026 = exercice(&pool, other_user, other, 2026).await;

    let missing = fiscal_years::close(&pool, user_id, company_id, 999_999).await;
    assert!(matches!(missing, Err(DbError::NotFound)), "{missing:?}");
    let cross = fiscal_years::close(&pool, user_id, company_id, autre_2026).await;
    assert!(matches!(cross, Err(DbError::NotFound)), "{cross:?}");
    assert_eq!(
        statut(&pool, autre_2026).await,
        Some(FiscalYearStatus::Open)
    );
}

/// AC 2 — précédence, paire « déjà clos » > `EarlierFiscalYearOpen` : dans
/// l'état **hérité** (2025 ouvert, 2026 clos par SQL), re-clore 2026 répond
/// « déjà clos » — l'état de l'exercice lui-même parle d'abord.
#[sqlx::test(migrations = "./test-schema")]
async fn close_precedence_already_closed_before_earlier_open(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let _y2025 = exercice(&pool, user_id, company_id, 2025).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;
    poser_clos(&pool, y2026).await;

    let result = fiscal_years::close(&pool, user_id, company_id, y2026).await;
    assert!(
        matches!(result, Err(DbError::IllegalStateTransition(_))),
        "attendu IllegalStateTransition, obtenu {result:?}"
    );
}

/// AC 3 (c), F6 — l'exercice **disparaît** entre l'étape (a) et l'étape (c) :
/// `NotFound`, jamais une panique. W tient Y ; la clôture bute en (c) ; W
/// supprime Y (sans écriture) et valide.
#[sqlx::test(migrations = "./test-schema")]
async fn close_returns_not_found_when_the_year_vanishes_before_its_lock(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;

    let mut w = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(y2026)
        .execute(&mut *w)
        .await
        .unwrap();
    let cloture = {
        let pool = pool.clone();
        tokio::spawn(async move { fiscal_years::close(&pool, user_id, company_id, y2026).await })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, MOTIF_ETAPE_C, || cloture.is_finished()).await,
        "la clôture doit être vue en cours à l'étape (c)"
    );
    sqlx::query("DELETE FROM fiscal_years WHERE id = ?")
        .bind(y2026)
        .execute(&mut *w)
        .await
        .unwrap();
    w.commit().await.unwrap();

    let result = cloture.await.expect("la clôture ne doit pas paniquer");
    assert!(matches!(result, Err(DbError::NotFound)), "{result:?}");
}

/// AC 5 — création : refusée si un exercice **postérieur** est clos ; le refus
/// nomme le **plus proche** ; rien n'est inséré, rien n'est audité.
///
/// ⛔ **Tue la mutation (iv)** (retirer la garde de `create`).
#[sqlx::test(migrations = "./test-schema")]
async fn create_is_refused_before_a_closed_year(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;
    let y2027 = exercice(&pool, user_id, company_id, 2027).await;
    fiscal_years::close(&pool, user_id, company_id, y2026)
        .await
        .unwrap();
    fiscal_years::close(&pool, user_id, company_id, y2027)
        .await
        .unwrap();
    let avant = nombre_exercices(&pool, company_id).await;

    let mut new = ny("Exercice 2025", 2025);
    new.company_id = company_id;
    let result = fiscal_years::create(&pool, user_id, new).await;
    match result {
        Err(DbError::LaterFiscalYearClosed {
            fiscal_year_id,
            fiscal_year_name,
        }) => {
            assert_eq!(fiscal_year_id, y2026, "le plus proche postérieur clos");
            assert_eq!(fiscal_year_name, "Exercice 2026");
        }
        other => panic!("attendu LaterFiscalYearClosed, obtenu {other:?}"),
    }
    assert_eq!(nombre_exercices(&pool, company_id).await, avant);
    let audits_creation: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'fiscal_year.created' \
         AND JSON_UNQUOTE(JSON_EXTRACT(details_json, '$.name')) = 'Exercice 2025'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(audits_creation, 0);
}

/// AC 5 — un exercice postérieur **ouvert** ne gêne pas : créer un exercice
/// antérieur reste permis (l'invariant I ne porte que sur les clos).
#[sqlx::test(migrations = "./test-schema")]
async fn create_before_an_open_year_is_allowed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let _y2026 = exercice(&pool, user_id, company_id, 2026).await;
    let y2025 = exercice(&pool, user_id, company_id, 2025).await;
    assert_eq!(statut(&pool, y2025).await, Some(FiscalYearStatus::Open));
}

/// AC 5 — précédence : une demande qui **chevauche** un exercice et précède un
/// exercice clos rend le chevauchement (pré-contrôle existant, qui parle
/// d'abord).
#[sqlx::test(migrations = "./test-schema")]
async fn create_precedence_overlap_before_later_closed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y2024 = exercice(&pool, user_id, company_id, 2024).await;
    let y2026 = exercice(&pool, user_id, company_id, 2026).await;
    // 2025 n'existe pas : clôturer 2024 puis 2026 respecte l'ordre.
    fiscal_years::close(&pool, user_id, company_id, y2024)
        .await
        .unwrap();
    fiscal_years::close(&pool, user_id, company_id, y2026)
        .await
        .unwrap();

    let new = NewFiscalYear {
        company_id,
        name: "Exercice décalé".into(),
        start_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2025, 6, 30).unwrap(),
    };
    let result = fiscal_years::create(&pool, user_id, new).await;
    assert!(
        matches!(&result, Err(DbError::Invariant(k)) if k == FY_OVERLAP_KEY),
        "attendu FY_OVERLAP_KEY, obtenu {result:?}"
    );
}

/// AC 13 a — **la preuve de l'invariant I sous concurrence** : une réouverture
/// de N **en cours** contre `close(L)` (N clos, L ouvert, L > N).
///
/// W refait les gestes de `reopen` sans les valider (verrou de N, lecture des
/// postérieurs clos, `UPDATE … 'Open'`). La clôture est lancée dans son
/// enveloppe et vue **en cours** (l'aide voit une requête qui s'exécute ; c'est
/// le montage — W tient N, que l'étape (b') demande — qui en fait une attente).
/// W valide : la relecture verrouillante (d) lit N **ouvert** → refus.
///
/// ⛔ **Tue la mutation (ii)** (relecture (d) non verrouillante) : la clôture
/// lirait (d) dans sa vue, fixée en (a) avant la validation de W — N clos —, et
/// passerait : état final « N ouvert, L clos ». Remplace l'ancien
/// `reopen_close_concurrent_is_serialized`, course libre qui admettait cet état.
#[sqlx::test(migrations = "./test-schema")]
async fn close_sees_a_concurrent_reopening_of_an_earlier_year(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let n = exercice(&pool, user_id, company_id, 2025).await;
    let l = exercice(&pool, user_id, company_id, 2026).await;
    fiscal_years::close(&pool, user_id, company_id, n)
        .await
        .unwrap();
    let n_start = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();

    let mut w = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(n)
        .execute(&mut *w)
        .await
        .unwrap();
    assert!(
        fiscal_years::find_later_closed_in_tx(&mut w, company_id, n_start)
            .await
            .unwrap()
            .is_none()
    );
    sqlx::query("UPDATE fiscal_years SET status = 'Open' WHERE id = ?")
        .bind(n)
        .execute(&mut *w)
        .await
        .unwrap();

    let cloture = {
        let pool = pool.clone();
        tokio::spawn(async move {
            retry_on_deadlock("fiscal_years::close", || {
                fiscal_years::close(&pool, user_id, company_id, l)
            })
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["fiscal_years", "FOR UPDATE"], || cloture
            .is_finished())
        .await,
        "la clôture doit être vue en cours, bloquée sur N"
    );
    w.commit().await.unwrap();

    let result = cloture.await.unwrap();
    assert!(
        matches!(&result, Err(DbError::EarlierFiscalYearOpen { fiscal_year_id, .. }) if *fiscal_year_id == n),
        "attendu EarlierFiscalYearOpen nommant N, obtenu {result:?}"
    );
    assert_eq!(statut(&pool, n).await, Some(FiscalYearStatus::Open));
    assert_eq!(statut(&pool, l).await, Some(FiscalYearStatus::Open));
    assert_eq!(audits(&pool, "fiscal_year.closed", l).await, 0);
}

/// AC 13 c — une clôture de N **en cours** contre `close(N+1)` (N et N+1
/// ouverts) : garde contre un refus **parasite**. W pose `'Closed'` sur N sans
/// valider ; `close(N+1)` est vue en cours, bloquée sur N à l'étape (b') ; W
/// valide ; `close(N+1)` **réussit**. Forme ordonnée : lancées librement,
/// `close(N+1)` pourrait lire N ouvert la première et refuser légitimement.
///
/// ⛔ **Tue la mutation (ii)** : sous une relecture (d) non verrouillante, la
/// vue fixée en (a) montre N ouvert → refus parasite.
#[sqlx::test(migrations = "./test-schema")]
async fn close_after_a_concurrent_close_of_the_previous_year_succeeds(pool: MySqlPool) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let n = exercice(&pool, user_id, company_id, 2025).await;
    let suivant = exercice(&pool, user_id, company_id, 2026).await;

    let mut w = pool.begin().await.unwrap();
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(n)
        .execute(&mut *w)
        .await
        .unwrap();
    let cloture = {
        let pool = pool.clone();
        tokio::spawn(async move {
            retry_on_deadlock("fiscal_years::close", || {
                fiscal_years::close(&pool, user_id, company_id, suivant)
            })
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, &["fiscal_years", "FOR UPDATE"], || cloture
            .is_finished())
        .await,
        "la clôture de N+1 doit être vue en cours, bloquée sur N"
    );
    w.commit().await.unwrap();

    let result = cloture.await.unwrap();
    assert!(result.is_ok(), "close(N+1) doit réussir : {result:?}");
    assert_eq!(statut(&pool, n).await, Some(FiscalYearStatus::Closed));
    assert_eq!(statut(&pool, suivant).await, Some(FiscalYearStatus::Closed));
}

/// Déroule le test 13 b : `create(X)` contre `close(Y)`, X antérieur à Y, tous
/// deux dans leur enveloppe de rejeu. Entrelacement forcé : W tient Y ; la
/// clôture est vue en étape (c) ; la création est vue à l'étape où elle bute
/// (`motif_creation` — dans les deux configurations mesurées, son pré-contrôle
/// de chevauchement : sur Y en b1, sur M en b2) ; W valide. Rend les deux
/// issues et le nombre de tentatives de chaque côté.
async fn creation_contre_cloture(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    y: i64,
    x_annee: i32,
    motif_creation: &[&str],
) -> (
    Result<kesh_db::entities::FiscalYear, DbError>,
    Result<kesh_db::entities::FiscalYear, DbError>,
    u32,
    u32,
) {
    let mut w = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(y)
        .execute(&mut *w)
        .await
        .unwrap();

    let essais_cloture = Arc::new(AtomicU32::new(0));
    let cloture = {
        let pool = pool.clone();
        let essais = essais_cloture.clone();
        tokio::spawn(async move {
            retry_on_deadlock("fiscal_years::close", || {
                essais.fetch_add(1, Ordering::SeqCst);
                fiscal_years::close(&pool, user_id, company_id, y)
            })
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(pool, MOTIF_ETAPE_C, || cloture.is_finished()).await,
        "la clôture doit être vue à l'étape (c), bloquée sur Y"
    );

    let essais_creation = Arc::new(AtomicU32::new(0));
    let creation = {
        let pool = pool.clone();
        let essais = essais_creation.clone();
        let mut new = ny(&format!("Exercice {x_annee}"), x_annee);
        new.company_id = company_id;
        tokio::spawn(async move {
            retry_on_deadlock("fiscal_years::create", || {
                essais.fetch_add(1, Ordering::SeqCst);
                fiscal_years::create(&pool, user_id, new.clone())
            })
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(pool, motif_creation, || creation.is_finished()).await,
        "la création doit être vue en cours, bloquée ({motif_creation:?})"
    );
    w.commit().await.unwrap();

    let c = cloture.await.unwrap();
    let k = creation.await.unwrap();
    (
        c,
        k,
        essais_cloture.load(Ordering::SeqCst),
        essais_creation.load(Ordering::SeqCst),
    )
}

/// Asserte la propriété du 13 b : **jamais** « X ouvert, Y clos », et des
/// issues cohérentes de part et d'autre.
async fn jamais_x_ouvert_sous_y_clos(
    pool: &MySqlPool,
    company_id: i64,
    y: i64,
    x_annee: i32,
    cloture: &Result<kesh_db::entities::FiscalYear, DbError>,
    creation: &Result<kesh_db::entities::FiscalYear, DbError>,
) {
    let x: Option<i64> =
        sqlx::query_scalar("SELECT id FROM fiscal_years WHERE company_id = ? AND name = ?")
            .bind(company_id)
            .bind(format!("Exercice {x_annee}"))
            .fetch_optional(pool)
            .await
            .unwrap();
    let y_clos = statut(pool, y).await == Some(FiscalYearStatus::Closed);
    assert!(
        !(y_clos && x.is_some()),
        "état fautif atteint : X ouvert sous Y clos (clôture {cloture:?}, création {creation:?})"
    );
    match (cloture, creation) {
        (Ok(_), Err(DbError::LaterFiscalYearClosed { fiscal_year_id, .. })) => {
            assert_eq!(*fiscal_year_id, y);
        }
        (Err(DbError::EarlierFiscalYearOpen { fiscal_year_id, .. }), Ok(fy)) => {
            assert_eq!(*fiscal_year_id, fy.id);
        }
        other => panic!("issues incohérentes : {other:?}"),
    }
}

/// AC 13 b1 — `create(X)` contre `close(Y)`, **aucun exercice antérieur à X**.
///
/// **Mécanisme observé** (ce test, trois exécutions) : **sérialisation** — le
/// pré-contrôle `find_overlapping` de la création lit Y (la borne de son
/// parcours) et l'attend derrière la clôture ; la clôture valide ; la création
/// reprend, lit Y clos dans sa garde et refuse (`LaterFiscalYearClosed`), en une
/// seule tentative. (À la main, en T0, la création avait passé ce pré-contrôle
/// et buté dans sa garde : même issue, autre point d'arrêt — le test fait foi.)
/// Aucune mutation propre : la propriété est tenue par les gardes (i) et (iv)
/// ensemble.
#[sqlx::test(migrations = "./test-schema")]
async fn create_against_close_without_earlier_year_never_leaves_an_open_year_under_a_closed_one(
    pool: MySqlPool,
) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let y = exercice(&pool, user_id, company_id, 2026).await;

    let (c, k, essais_c, essais_k) =
        creation_contre_cloture(&pool, user_id, company_id, y, 2025, MOTIF_CHEVAUCHEMENT).await;
    eprintln!("13 b1 : tentatives clôture = {essais_c}, création = {essais_k}");
    jamais_x_ouvert_sous_y_clos(&pool, company_id, y, 2025, &c, &k).await;
}

/// AC 13 b2 — `create(X)` contre `close(Y)`, **un exercice clos M antérieur à
/// X**.
///
/// **Mécanisme observé** (T0, puis ce test) : **interblocage** — la clôture
/// tient M (étape (b')) ; la création lit M dans `find_overlapping` et l'attend
/// (c'est là qu'elle est vue en cours, après avoir posé le verrou d'index de
/// M) ; la clôture, Y obtenu, demande en étape (d) ce verrou d'index : cycle. Victime observée : la création, qui, rejouée, lit Y
/// clos et refuse. Aucune mutation propre (gardes (i) et (iv) ensemble).
#[sqlx::test(migrations = "./test-schema")]
async fn create_against_close_with_a_closed_earlier_year_never_leaves_an_open_year_under_a_closed_one(
    pool: MySqlPool,
) {
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let m = exercice(&pool, user_id, company_id, 2020).await;
    fiscal_years::close(&pool, user_id, company_id, m)
        .await
        .unwrap();
    let y = exercice(&pool, user_id, company_id, 2026).await;

    let (c, k, essais_c, essais_k) =
        creation_contre_cloture(&pool, user_id, company_id, y, 2025, MOTIF_CHEVAUCHEMENT).await;
    eprintln!("13 b2 : tentatives clôture = {essais_c}, création = {essais_k}");
    jamais_x_ouvert_sous_y_clos(&pool, company_id, y, 2025, &c, &k).await;
}

/// AC 13 d — `close(N)` contre une **contre-passation** (cycle de l'AC 3 ; M
/// clos, N ouvert, T ouvert couvrant le jour du serveur, M < N < T).
///
/// W refait les deux temps de `reverse_in_tx_inner` : verrou de N par clé
/// primaire, puis `find_open_covering_date(today)`, dont le parcours part du
/// premier exercice et contient M. La clôture est lancée entre les deux et vue
/// en étape (c) — ce qui prouve que l'étape (b') a rendu et tient M. W demande
/// alors M : **interblocage**. Victime mesurée (T0) : la clôture, que W soit
/// alourdie ou non — W obtient M. ⚠️ La clôture rejouée rebuterait en (b') sur
/// M, que W tient : le test **annule W d'abord**, puis attend la clôture.
///
/// **Objet** : le cycle existe et se résout — la clôture, dans son enveloppe,
/// finit acceptée. Il ne tue **aucune** mutation : l'enveloppe y est écrite par
/// le test ; c'est le test HTTP de rejeu (`rejeu_interblocage_e2e.rs`) qui tue
/// la mutation (ix).
#[sqlx::test(migrations = "./test-schema")]
async fn close_against_a_reversal_deadlocks_and_the_replay_resolves_it(pool: MySqlPool) {
    use chrono::Datelike;
    let company_id = create_company(&pool).await;
    let user_id = create_admin_user(&pool, company_id).await;
    let today = chrono::Utc::now().date_naive();
    let m = exercice(&pool, user_id, company_id, today.year() - 6).await;
    let n = exercice(&pool, user_id, company_id, today.year() - 5).await;
    let _t = exercice(&pool, user_id, company_id, today.year()).await;
    fiscal_years::close(&pool, user_id, company_id, m)
        .await
        .unwrap();

    let mut w = pool.begin().await.unwrap();
    sqlx::query("SELECT id FROM fiscal_years WHERE id = ? FOR UPDATE")
        .bind(n)
        .execute(&mut *w)
        .await
        .unwrap();

    let essais = Arc::new(AtomicU32::new(0));
    let cloture = {
        let pool = pool.clone();
        let essais = essais.clone();
        tokio::spawn(async move {
            retry_on_deadlock("fiscal_years::close", || {
                essais.fetch_add(1, Ordering::SeqCst);
                fiscal_years::close(&pool, user_id, company_id, n)
            })
            .await
        })
    };
    assert!(
        attendre_une_requete_en_cours(&pool, MOTIF_ETAPE_C, || cloture.is_finished()).await,
        "la clôture doit être vue à l'étape (c), bloquée sur N"
    );

    // Second temps de la contre-passation : l'exercice du jour.
    let w_issue = fiscal_years::find_open_covering_date(&mut w, company_id, today).await;
    let w_victime = match &w_issue {
        Ok(fy) => {
            assert!(fy.is_some(), "T couvre le jour");
            false
        }
        Err(e) => {
            assert!(
                kesh_db::retry::is_deadlock_error(e),
                "W ne peut échouer que par un interblocage : {e:?}"
            );
            true
        }
    };
    w.rollback().await.unwrap();

    let result = cloture.await.unwrap();
    assert!(
        result.is_ok(),
        "la clôture doit finir acceptée : {result:?}"
    );
    assert_eq!(statut(&pool, n).await, Some(FiscalYearStatus::Closed));
    // Le cycle a bien eu lieu : l'une des deux transactions en a été victime.
    let tentatives = essais.load(Ordering::SeqCst);
    assert!(
        w_victime || tentatives == 2,
        "un interblocage était attendu (tentatives de la clôture : {tentatives}, W victime : {w_victime})"
    );
}
