//! Tests d'intégration pour `repositories::fiscal_years`.

use chrono::NaiveDate;
use kesh_db::entities::{FiscalYearStatus, Language, NewCompany, NewFiscalYear, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::{companies, fiscal_years};
use sqlx::MySqlPool;


fn sample_new_company() -> NewCompany {
    NewCompany {
        name: "Test SA".into(),
        address: "Rue Test 1".into(),
        ide_number: None,
        org_type: OrgType::Pme,
        accounting_language: Language::Fr,
        instance_language: Language::Fr,
    }
}

async fn create_company(pool: &MySqlPool) -> i64 {
    companies::create(pool, sample_new_company()).await.unwrap().id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_and_find_by_id(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let new = NewFiscalYear {
        company_id,
        name: "Exercice 2026".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let created = fiscal_years::create(&pool, new).await.unwrap();
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
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = fiscal_years::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fk_violation_on_missing_company(pool: MySqlPool) {
    let new = NewFiscalYear {
        company_id: 999_999,
        name: "Exercice fantôme".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let result = fiscal_years::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::ForeignKeyViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_name_per_company(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let year = NewFiscalYear {
        company_id,
        name: "Exercice 2026".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    fiscal_years::create(&pool, year).await.unwrap();

    // Même nom sur la même company → UNIQUE violation
    let duplicate = NewFiscalYear {
        company_id,
        name: "Exercice 2026".into(),
        start_date: NaiveDate::from_ymd_opt(2027, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2027, 12, 31).unwrap(),
    };
    let result = fiscal_years::create(&pool, duplicate).await;
    assert!(matches!(result, Err(DbError::UniqueConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_start_date_per_company(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let year1 = NewFiscalYear {
        company_id,
        name: "Exercice A".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    fiscal_years::create(&pool, year1).await.unwrap();

    // Même start_date sur la même company → UNIQUE violation
    let year2 = NewFiscalYear {
        company_id,
        name: "Exercice B".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
    };
    let result = fiscal_years::create(&pool, year2).await;
    assert!(matches!(result, Err(DbError::UniqueConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_constraint_rejects_equal_dates(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    // end_date == start_date doit aussi être rejeté (contrainte: end > start, strict)
    let bad = NewFiscalYear {
        company_id,
        name: "Zero-length".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 6, 15).unwrap(),
    };
    let result = fiscal_years::create(&pool, bad).await;
    assert!(
        matches!(result, Err(DbError::CheckConstraintViolation(_))),
        "end_date == start_date doit violer CHECK, got {result:?}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn check_constraint_end_date_must_be_after_start(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let bad = NewFiscalYear {
        company_id,
        name: "Invalid".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
    };
    let result = fiscal_years::create(&pool, bad).await;
    assert!(
        matches!(result, Err(DbError::CheckConstraintViolation(_))),
        "end_date < start_date doit retourner CheckConstraintViolation, got {result:?}"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_by_company(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    // Créer 3 exercices dans le désordre
    for year in [2027, 2025, 2026] {
        let new = NewFiscalYear {
            company_id,
            name: format!("Exercice {year}"),
            start_date: NaiveDate::from_ymd_opt(year, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(year, 12, 31).unwrap(),
        };
        fiscal_years::create(&pool, new).await.unwrap();
    }

    let list = fiscal_years::list_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(list.len(), 3);
    // Triés par start_date ASC
    assert_eq!(list[0].name, "Exercice 2025");
    assert_eq!(list[1].name, "Exercice 2026");
    assert_eq!(list[2].name, "Exercice 2027");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_open_to_closed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let new = NewFiscalYear {
        company_id,
        name: "Exercice 2026".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let created = fiscal_years::create(&pool, new).await.unwrap();
    assert_eq!(created.status, FiscalYearStatus::Open);

    let closed = fiscal_years::close(&pool, created.id).await.unwrap();
    assert_eq!(closed.status, FiscalYearStatus::Closed);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_fails_on_missing(pool: MySqlPool) {
    let result = fiscal_years::close(&pool, 999_999).await;
    assert!(matches!(result, Err(DbError::NotFound)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn close_fails_on_already_closed(pool: MySqlPool) {
    let company_id = create_company(&pool).await;

    let new = NewFiscalYear {
        company_id,
        name: "Exercice 2026".into(),
        start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
        end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
    };
    let created = fiscal_years::create(&pool, new).await.unwrap();

    // Première clôture : OK
    fiscal_years::close(&pool, created.id).await.unwrap();

    // Deuxième clôture : doit échouer — pas de réouverture possible
    let result = fiscal_years::close(&pool, created.id).await;
    assert!(matches!(result, Err(DbError::IllegalStateTransition(_))));
}
