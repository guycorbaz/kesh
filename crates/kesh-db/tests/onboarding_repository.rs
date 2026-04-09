//! Tests d'intégration pour `repositories::onboarding`.

use kesh_db::entities::onboarding::UiMode;
use kesh_db::errors::DbError;
use kesh_db::repositories::onboarding;
use sqlx::MySqlPool;

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_state_returns_none_when_empty(pool: MySqlPool) {
    let state = onboarding::get_state(&pool).await.expect("should succeed");
    assert!(state.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn init_state_creates_row_with_defaults(pool: MySqlPool) {
    let state = onboarding::init_state(&pool).await.expect("init should succeed");
    assert!(state.id > 0);
    assert_eq!(state.step_completed, 0);
    assert!(!state.is_demo);
    assert_eq!(state.ui_mode, None);
    assert_eq!(state.version, 1);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn get_state_returns_some_after_init(pool: MySqlPool) {
    onboarding::init_state(&pool).await.unwrap();
    let state = onboarding::get_state(&pool).await.unwrap();
    assert!(state.is_some());
    assert_eq!(state.unwrap().step_completed, 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_step_progresses_correctly(pool: MySqlPool) {
    let initial = onboarding::init_state(&pool).await.unwrap();
    assert_eq!(initial.step_completed, 0);
    assert_eq!(initial.version, 1);

    // Step 0 → 1 (langue choisie)
    let step1 = onboarding::update_step(&pool, 1, false, None, initial.version)
        .await
        .unwrap();
    assert_eq!(step1.step_completed, 1);
    assert_eq!(step1.version, 2);

    // Step 1 → 2 (mode choisi)
    let step2 = onboarding::update_step(&pool, 2, false, Some(UiMode::Guided), step1.version)
        .await
        .unwrap();
    assert_eq!(step2.step_completed, 2);
    assert_eq!(step2.ui_mode, Some(UiMode::Guided));
    assert_eq!(step2.version, 3);

    // Step 2 → 3 (chemin démo)
    let step3 = onboarding::update_step(&pool, 3, true, Some(UiMode::Guided), step2.version)
        .await
        .unwrap();
    assert_eq!(step3.step_completed, 3);
    assert!(step3.is_demo);
    assert_eq!(step3.version, 4);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_step_fails_on_stale_version(pool: MySqlPool) {
    let initial = onboarding::init_state(&pool).await.unwrap();

    // Première mise à jour réussit
    onboarding::update_step(&pool, 1, false, None, initial.version)
        .await
        .unwrap();

    // Deuxième mise à jour avec la même version échoue
    let result = onboarding::update_step(&pool, 2, false, None, initial.version).await;
    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn delete_state_removes_row(pool: MySqlPool) {
    onboarding::init_state(&pool).await.unwrap();
    assert!(onboarding::get_state(&pool).await.unwrap().is_some());

    onboarding::delete_state(&pool).await.unwrap();
    assert!(onboarding::get_state(&pool).await.unwrap().is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_step_with_expert_mode(pool: MySqlPool) {
    let initial = onboarding::init_state(&pool).await.unwrap();
    let updated = onboarding::update_step(&pool, 2, false, Some(UiMode::Expert), initial.version)
        .await
        .unwrap();
    assert_eq!(updated.ui_mode, Some(UiMode::Expert));
}
