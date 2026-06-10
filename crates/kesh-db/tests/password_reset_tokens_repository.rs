//! Tests d'intégration du repository `password_reset_tokens` + `find_by_email`
//! (Story 17-4a, recovery #122).
//!
//! Chaque test utilise `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]`
//! qui crée/détruit une DB temporaire par test.

use chrono::{Duration as ChronoDuration, Utc};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{password_reset_tokens, users};
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

async fn create_user(
    pool: &MySqlPool,
    username: &str,
    company_id: i64,
    email: Option<&str>,
) -> i64 {
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash:
                "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo"
                    .to_string(),
            role: Role::Admin,
            active: true,
            company_id,
            email: email.map(|s| s.to_string()),
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

fn hash(s: &str) -> String {
    // Hash factice 64 hex (le vrai SHA-256 est calculé par kesh-api).
    format!("{:0<64}", s)
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn create_then_find_valid_then_mark_used(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_user(&pool, "alice", company_id, Some("alice@example.com")).await;
    let token_hash = hash("abc");
    let expires_at = (Utc::now() + ChronoDuration::minutes(30)).naive_utc();

    let created = password_reset_tokens::create(&pool, user_id, &token_hash, expires_at)
        .await
        .expect("create should succeed");
    assert_eq!(created.user_id, user_id);
    assert_eq!(created.token_hash, token_hash);
    assert!(created.used_at.is_none());

    // find_valid retrouve le token non-utilisé non-expiré.
    let found = password_reset_tokens::find_valid_by_hash(&pool, &token_hash)
        .await
        .expect("find should succeed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);

    // mark_used consomme le token.
    password_reset_tokens::mark_used(&pool, created.id)
        .await
        .expect("mark_used should succeed");

    // find_valid retourne maintenant None (usage unique, DC8).
    let after = password_reset_tokens::find_valid_by_hash(&pool, &token_hash)
        .await
        .expect("find should succeed");
    assert!(after.is_none(), "token consommé ne doit plus être valide");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expired_token_is_not_valid(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_user(&pool, "bob", company_id, None).await;
    let token_hash = hash("expired");
    let expires_at = (Utc::now() - ChronoDuration::minutes(1)).naive_utc();

    password_reset_tokens::create(&pool, user_id, &token_hash, expires_at)
        .await
        .expect("create should succeed");

    let found = password_reset_tokens::find_valid_by_hash(&pool, &token_hash)
        .await
        .expect("find should succeed");
    assert!(found.is_none(), "token expiré ne doit pas être valide");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unknown_hash_returns_none(pool: MySqlPool) {
    let found = password_reset_tokens::find_valid_by_hash(&pool, &hash("nope"))
        .await
        .expect("find should succeed");
    assert!(found.is_none());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn invalidate_all_for_user_consumes_pending(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_user(&pool, "carol", company_id, None).await;
    let expires_at = (Utc::now() + ChronoDuration::minutes(30)).naive_utc();
    password_reset_tokens::create(&pool, user_id, &hash("t1"), expires_at)
        .await
        .unwrap();
    password_reset_tokens::create(&pool, user_id, &hash("t2"), expires_at)
        .await
        .unwrap();

    let invalidated = password_reset_tokens::invalidate_all_for_user(&pool, user_id)
        .await
        .expect("invalidate should succeed");
    assert_eq!(invalidated, 2);

    assert!(
        password_reset_tokens::find_valid_by_hash(&pool, &hash("t1"))
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        password_reset_tokens::find_valid_by_hash(&pool, &hash("t2"))
            .await
            .unwrap()
            .is_none()
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn cascade_delete_removes_tokens(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_user(&pool, "dave", company_id, None).await;
    let expires_at = (Utc::now() + ChronoDuration::minutes(30)).naive_utc();
    password_reset_tokens::create(&pool, user_id, &hash("d1"), expires_at)
        .await
        .unwrap();

    // Supprimer l'user purge ses tokens (FK ON DELETE CASCADE, DC11).
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("delete user should succeed");

    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens WHERE user_id = ?")
            .bind(user_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(count, 0, "CASCADE doit purger les tokens du user supprimé");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn find_by_email_returns_matches(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    create_user(&pool, "eve", company_id, Some("shared@example.com")).await;
    create_user(&pool, "frank", company_id, Some("shared@example.com")).await;
    create_user(&pool, "grace", company_id, Some("solo@example.com")).await;
    create_user(&pool, "heidi", company_id, None).await;

    // Non-unique : 2 matches pour l'email partagé (DC6).
    let shared = users::find_by_email(&pool, "shared@example.com")
        .await
        .expect("find_by_email should succeed");
    assert_eq!(shared.len(), 2);

    // Case-insensitive (collation utf8mb4_unicode_ci).
    let solo = users::find_by_email(&pool, "SOLO@example.com")
        .await
        .expect("find_by_email should succeed");
    assert_eq!(solo.len(), 1);
    assert_eq!(solo[0].username, "grace");

    // Email inconnu → vide.
    let none = users::find_by_email(&pool, "ghost@example.com")
        .await
        .expect("find_by_email should succeed");
    assert!(none.is_empty());
}
