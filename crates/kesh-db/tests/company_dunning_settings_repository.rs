//! Tests d'intégration du repo `company_dunning_settings` + seed (Story 21-3, #231).

use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{CompanyDunningSettingsUpdate, Language, NewCompany, OrgType};
use kesh_db::errors::DbError;
use kesh_db::repositories::{companies, company_dunning_settings, dunning_levels};
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

async fn create_admin(pool: &MySqlPool, company_id: i64) -> i64 {
    let result = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES (?, ?, 'Admin', TRUE, ?)",
    )
    .bind(format!("admin_{company_id}"))
    .bind("$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams")
    .bind(company_id)
    .execute(pool)
    .await
    .expect("create admin user for test");
    result.last_insert_id() as i64
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_or_create_is_idempotent(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "GOC Co").await;
    let s1 = company_dunning_settings::get_or_create_default(&pool, company_id)
        .await
        .unwrap();
    let s2 = company_dunning_settings::get_or_create_default(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(s1.company_id, s2.company_id);
    assert_eq!(s1.version, s2.version);
    assert!(
        s1.seeded_at.is_none(),
        "get_or_create ne seede PAS (seeded_at reste NULL)"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_no_op_and_version_and_conflict(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Upd Co").await;
    let admin = create_admin(&pool, company_id).await;
    let s = company_dunning_settings::get_or_create_default(&pool, company_id)
        .await
        .unwrap();

    // No-op (même grâce) → version inchangée.
    let noop = company_dunning_settings::update(
        &pool,
        company_id,
        s.version,
        admin,
        CompanyDunningSettingsUpdate {
            grace_period_days: s.grace_period_days,
        },
    )
    .await
    .unwrap();
    assert_eq!(noop.version, s.version);

    // Vrai changement → version +1.
    let bumped = company_dunning_settings::update(
        &pool,
        company_id,
        s.version,
        admin,
        CompanyDunningSettingsUpdate {
            grace_period_days: s.grace_period_days + 3,
        },
    )
    .await
    .unwrap();
    assert_eq!(bumped.version, s.version + 1);
    assert_eq!(bumped.grace_period_days, s.grace_period_days + 3);

    // Version périmée → conflit.
    let stale = company_dunning_settings::update(
        &pool,
        company_id,
        s.version, // périmé maintenant
        admin,
        CompanyDunningSettingsUpdate {
            grace_period_days: 99,
        },
    )
    .await;
    assert!(matches!(stale, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_creates_three_levels_and_stamps_seeded_at(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Seed Co").await;

    let mut tx = pool.begin().await.unwrap();
    let settings = company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    assert!(settings.seeded_at.is_some(), "seed pose seeded_at");
    assert_eq!(settings.grace_period_days, 5);
    let levels = dunning_levels::list_all_by_company(&pool, company_id)
        .await
        .unwrap();
    assert_eq!(levels.len(), 3);
    assert_eq!(levels[0].level_number, 1);
    assert_eq!(levels[2].fee_amount.to_string(), "40.00");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_is_idempotent(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "Idem Co").await;
    for _ in 0..2 {
        let mut tx = pool.begin().await.unwrap();
        company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }
    // Toujours 3 niveaux (pas de doublon).
    assert_eq!(
        dunning_levels::count_for_company(&pool, company_id)
            .await
            .unwrap(),
        3
    );
}

/// D7 : une fois seedé, vider `dunning_levels` = désactivation VOLONTAIRE →
/// un nouvel appel au seed NE ressuscite PAS les défauts.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn seed_does_not_resurrect_after_manual_empty(pool: MySqlPool) {
    let company_id = create_test_company(&pool, "NoResurrect Co").await;

    let mut tx = pool.begin().await.unwrap();
    company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // L'utilisateur vide volontairement les niveaux.
    sqlx::query("DELETE FROM dunning_levels WHERE company_id = ?")
        .bind(company_id)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        dunning_levels::count_for_company(&pool, company_id)
            .await
            .unwrap(),
        0
    );

    // Nouvel accès config → PAS de résurrection (seeded_at déjà posé).
    let mut tx = pool.begin().await.unwrap();
    company_dunning_settings::ensure_seeded_in_tx(&mut tx, company_id)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(
        dunning_levels::count_for_company(&pool, company_id)
            .await
            .unwrap(),
        0,
        "vide = désactivé, pas de résurrection"
    );
}
