//! Tests d'intégration du repo `dunning_levels` (Story 21-3, #231).

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{Language, NewCompany, NewDunningLevel, OrgType, UpdateDunningLevel};
use kesh_db::errors::DbError;
use kesh_db::repositories::{companies, dunning_levels};
use rust_decimal::Decimal;
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool, name: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: name.to_string(),
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
    .expect("create test company")
    .id
}

/// Crée un niveau via une tx (les tests n'ont pas besoin du sentinel lock —
/// DB éphémère sérialisée).
async fn create_level(pool: &MySqlPool, company_id: i64, delay: i32, fee: &str) -> i64 {
    let mut tx = pool.begin().await.unwrap();
    let created = dunning_levels::create_for_company(
        &mut tx,
        &NewDunningLevel {
            company_id,
            delay_days: delay,
            fee_amount: fee.parse::<Decimal>().unwrap(),
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    created.id
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_appends_at_next_level_number(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Append Co").await;
    create_level(&pool, company_id, 10, "0.00").await;
    create_level(&pool, company_id, 10, "20.00").await;
    create_level(&pool, company_id, 10, "40.00").await;

    let levels = dunning_levels::list_all_by_company(&pool, company_id)
        .await
        .unwrap();
    let nums: Vec<i16> = levels.iter().map(|l| l.level_number).collect();
    assert_eq!(nums, vec![1, 2, 3]);
    assert_eq!(
        dunning_levels::count_for_company(&pool, company_id)
            .await
            .unwrap(),
        3
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_stale_version_conflicts(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Update Co").await;
    let id = create_level(&pool, company_id, 10, "0.00").await;

    let mut tx = pool.begin().await.unwrap();
    let result = dunning_levels::update_for_company(
        &mut tx,
        company_id,
        id,
        &UpdateDunningLevel {
            delay_days: 15,
            fee_amount: Decimal::ZERO,
        },
        99, // version périmée
    )
    .await;
    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

/// H1 (validate P1) : la renumérotation à la suppression bumpe `version` — le
/// verrou optimiste n'est pas contournable pour un niveau déplacé.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_renumbers_and_bumps_version_of_shifted_levels(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Renumber Co").await;
    create_level(&pool, company_id, 10, "0.00").await; // niveau 1
    let id2 = create_level(&pool, company_id, 10, "20.00").await; // niveau 2
    create_level(&pool, company_id, 10, "40.00").await; // niveau 3

    // Snapshot du niveau 3 AVANT suppression du niveau 2.
    let before = dunning_levels::list_all_by_company(&pool, company_id)
        .await
        .unwrap();
    let level3 = before.iter().find(|l| l.level_number == 3).unwrap().clone();
    assert_eq!(level3.version, 0);

    // Supprime le niveau 2.
    let mut tx = pool.begin().await.unwrap();
    dunning_levels::delete_and_renumber(&mut tx, company_id, id2, 0)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // L'ex-niveau 3 est devenu niveau 2, contiguïté préservée...
    let after = dunning_levels::list_all_by_company(&pool, company_id)
        .await
        .unwrap();
    let nums: Vec<i16> = after.iter().map(|l| l.level_number).collect();
    assert_eq!(nums, vec![1, 2]);
    let shifted = after.iter().find(|l| l.id == level3.id).unwrap();
    assert_eq!(shifted.level_number, 2);
    // ...et sa version a été bumpée (H1).
    assert!(
        shifted.version > level3.version,
        "version doit être bumpée par la renumérotation"
    );

    // Un update avec l'ancien expected_version (0) est désormais rejeté.
    let mut tx = pool.begin().await.unwrap();
    let stale = dunning_levels::update_for_company(
        &mut tx,
        company_id,
        level3.id,
        &UpdateDunningLevel {
            delay_days: 99,
            fee_amount: Decimal::ZERO,
        },
        0,
    )
    .await;
    assert!(matches!(stale, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn fee_over_max_rejected_by_check(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Fee Co").await;
    let mut tx = pool.begin().await.unwrap();
    let result = dunning_levels::create_for_company(
        &mut tx,
        &NewDunningLevel {
            company_id,
            delay_days: 10,
            fee_amount: "10000.01".parse::<Decimal>().unwrap(),
        },
    )
    .await;
    assert!(
        result.is_err(),
        "frais > 10000 doit être rejeté par le CHECK DB"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn levels_isolated_per_company(pool: MySqlPool) {
    let a = create_test_company(&pool, "Co A").await;
    let b = create_test_company(&pool, "Co B").await;
    create_level(&pool, a, 10, "0.00").await;
    create_level(&pool, a, 10, "20.00").await;
    create_level(&pool, b, 7, "5.00").await;

    assert_eq!(
        dunning_levels::count_for_company(&pool, a).await.unwrap(),
        2
    );
    assert_eq!(
        dunning_levels::count_for_company(&pool, b).await.unwrap(),
        1
    );
    // b repart à level_number 1 (isolation).
    let b_levels = dunning_levels::list_all_by_company(&pool, b).await.unwrap();
    assert_eq!(b_levels[0].level_number, 1);
}
