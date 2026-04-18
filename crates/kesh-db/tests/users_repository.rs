//! Tests d'intégration pour `repositories::users`.

use kesh_db::entities::{NewUser, Role, UserUpdate};
use kesh_db::errors::DbError;
use kesh_db::repositories::users;
use sqlx::MySqlPool;

async fn create_test_company(pool: &MySqlPool) -> i64 {
    let result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind("Test Company")
    .bind("Test Address")
    .bind("Independant")
    .bind("FR")
    .bind("FR")
    .execute(pool)
    .await
    .expect("company insert should succeed");
    result.last_insert_id() as i64
}

fn sample_new_user(company_id: i64) -> NewUser {
    NewUser {
        username: "alice".into(),
        // Hash factice Argon2id (le vrai hachage est fait par kesh-api story 1.5).
        // Longueur >= 20 pour respecter la contrainte CHECK chk_users_password_hash_len.
        password_hash: "$argon2id$v=19$m=19456,t=2,p=1$QUJDRA$YWJjZGVmZ2hpams".into(),
        role: Role::Comptable,
        active: true,
        company_id,
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_and_find_by_id(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let created = users::create(&pool, sample_new_user(company_id)).await.unwrap();
    assert!(created.id > 0);
    assert_eq!(created.username, "alice");
    assert_eq!(created.role, Role::Comptable);
    assert!(created.active);
    assert_eq!(created.version, 1);

    let found = users::find_by_id(&pool, created.id).await.unwrap().unwrap();
    assert_eq!(found.id, created.id);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_id_returns_none_for_missing(pool: MySqlPool) {
    let result = users::find_by_id(&pool, 999_999).await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_username(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    users::create(&pool, sample_new_user(company_id)).await.unwrap();

    let found = users::find_by_username(&pool, "alice").await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().username, "alice");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_username_returns_none_for_missing(pool: MySqlPool) {
    let result = users::find_by_username(&pool, "bob").await.unwrap();
    assert!(result.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_username_is_case_insensitive(pool: MySqlPool) {
    // La collation utf8mb4_unicode_ci est case-insensitive :
    // find_by_username("ALICE") matche la ligne "alice".
    // Ce comportement est documenté dans repositories/users.rs.
    let company_id = create_test_company(&pool).await;
    users::create(&pool, sample_new_user(company_id)).await.unwrap();

    let upper = users::find_by_username(&pool, "ALICE").await.unwrap();
    assert!(
        upper.is_some(),
        "find_by_username doit être case-insensitive"
    );
    assert_eq!(upper.unwrap().username, "alice");

    let mixed = users::find_by_username(&pool, "Alice").await.unwrap();
    assert!(mixed.is_some());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unique_constraint_on_username(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    users::create(&pool, sample_new_user(company_id)).await.unwrap();

    // Deuxième user avec même username → UNIQUE violation
    let result = users::create(&pool, sample_new_user(company_id)).await;
    assert!(matches!(result, Err(DbError::UniqueConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_role_and_active(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let created = users::create(&pool, sample_new_user(company_id)).await.unwrap();

    let changes = UserUpdate {
        role: Role::Admin,
        active: false,
    };
    let updated = users::update_role_and_active(&pool, created.id, created.version, changes)
        .await
        .unwrap();
    assert!(!updated.username.is_empty());

    assert_eq!(updated.role, Role::Admin);
    assert!(!updated.active);
    assert_eq!(updated.version, created.version + 1);
    // Le password_hash reste inchangé
    assert_eq!(updated.password_hash, created.password_hash);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn update_fails_on_stale_version(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let created = users::create(&pool, sample_new_user(company_id)).await.unwrap();

    // Premier update
    users::update_role_and_active(
        &pool,
        created.id,
        1,
        UserUpdate {
            role: Role::Admin,
            active: true,
        },
    )
    .await
    .unwrap();

    // Deuxième avec version stale
    let result = users::update_role_and_active(
        &pool,
        created.id,
        1,
        UserUpdate {
            role: Role::Consultation,
            active: true,
        },
    )
    .await;
    assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_with_pagination(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    for i in 0..4 {
        let mut new = sample_new_user(company_id);
        new.username = format!("user{i}");
        users::create(&pool, new).await.unwrap();
    }

    let page1 = users::list(&pool, 2, 0).await.unwrap();
    assert_eq!(page1.len(), 2);

    let page2 = users::list(&pool, 2, 2).await.unwrap();
    assert_eq!(page2.len(), 2);

    let empty = users::list(&pool, 10, 100).await.unwrap();
    assert_eq!(empty.len(), 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn debug_masks_password_hash(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let created = users::create(&pool, sample_new_user(company_id)).await.unwrap();
    let debug_output = format!("{created:?}");
    assert!(!debug_output.contains("argon2id"));
    assert!(!debug_output.contains("QUJDRA"));
    assert!(debug_output.contains("***"));
}

#[test]
fn debug_masks_password_hash_on_new_user() {
    // Vérifie que NewUser::Debug masque aussi le hash — pas de DB nécessaire
    let new = sample_new_user(1);
    let debug_output = format!("{new:?}");
    assert!(!debug_output.contains("argon2id"));
    assert!(debug_output.contains("***"));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn username_empty_rejected(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let mut new = sample_new_user(company_id);
    new.username = String::new();
    let result = users::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn password_hash_too_short_rejected(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let mut new = sample_new_user(company_id);
    new.password_hash = "short".into();
    let result = users::create(&pool, new).await;
    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

// Note : Rust ne permet pas d'exprimer "NOT impl Trait" au compile-time sans
// specialization. Pour protéger l'invariant "User ne doit JAMAIS dériver
// Serialize", on s'appuie sur :
// 1. Le commentaire SÉCURITÉ au-dessus de la struct User (entities/user.rs)
// 2. La revue de code (le reviewer doit explicitement approuver un ajout)
// 3. Ce test d'intégration qui vérifie que le `Debug` masque bien le hash —
//    au moins on détecte une fuite via logs/debug même si Serialize est ajouté
//
// Pour une garantie plus forte, une story future pourrait ajouter un test
// `trybuild` vérifiant qu'un use de `serde_json::to_string(&user)` ne compile
// pas.
