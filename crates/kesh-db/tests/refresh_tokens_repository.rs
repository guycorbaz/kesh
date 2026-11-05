//! Tests d'intégration du repository `refresh_tokens`.
//!
//! Chaque test utilise `#[sqlx::test(migrations = "./test-schema")]`
//! qui crée/détruit une DB temporaire par test — nécessite les droits sur
//! `\_sqlx\_test%`.* que pose `scripts/mariadb-init/01-dev-grants.sql`, aucun
//! droit global (voir le README du crate, § Prérequis).

use chrono::{Duration as ChronoDuration, Utc};
use kesh_db::entities::{NewRefreshToken, NewUser, Role};
use kesh_db::errors::DbError;
use kesh_db::repositories::{refresh_tokens, users};
use sqlx::MySqlPool;

/// Crée une company factice et retourne son id.
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

/// Crée un utilisateur factice et retourne son id.
async fn create_test_user(pool: &MySqlPool, username: &str, company_id: i64) -> i64 {
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash:
                "$argon2id$v=19$m=19456,t=2,p=1$dGVzdHNhbHQ$dGVzdGhhc2h0ZXN0aGFzaHRlc3RoYXNo"
                    .to_string(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

fn make_token_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_and_find_active_by_token(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "alice", company_id).await;
    let token_value = make_token_uuid();
    let expires_at = (Utc::now() + ChronoDuration::days(30)).naive_utc();

    let created = refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: token_value.clone(),
            expires_at,
        },
    )
    .await
    .expect("create should succeed");

    assert_eq!(created.user_id, user_id);
    // #890 : seule l'empreinte est stockée.
    assert_eq!(
        created.token_digest,
        refresh_tokens::token_digest(&token_value)
    );
    assert_ne!(created.token_digest, token_value);
    assert!(created.revoked_at.is_none());

    let found = refresh_tokens::find_active_by_token(&pool, &token_value)
        .await
        .expect("find should succeed");
    assert!(found.is_some());
    assert_eq!(found.unwrap().id, created.id);
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_active_by_token_returns_none_for_unknown_token(pool: MySqlPool) {
    let unknown = make_token_uuid();
    let result = refresh_tokens::find_active_by_token(&pool, &unknown)
        .await
        .expect("find should not error");
    assert!(result.is_none());
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_active_by_token_returns_none_for_expired_token(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "bob", company_id).await;
    let token_value = make_token_uuid();
    // Expiration dans le passé
    let expired_at = (Utc::now() - ChronoDuration::hours(1)).naive_utc();

    refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: token_value.clone(),
            expires_at: expired_at,
        },
    )
    .await
    .expect("create should succeed");

    let result = refresh_tokens::find_active_by_token(&pool, &token_value)
        .await
        .expect("find should not error");
    assert!(result.is_none(), "expired token should not be active");
}

#[sqlx::test(migrations = "./test-schema")]
async fn find_active_by_token_returns_none_for_revoked_token(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "carol", company_id).await;
    let token_value = make_token_uuid();
    let expires_at = (Utc::now() + ChronoDuration::days(30)).naive_utc();

    refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: token_value.clone(),
            expires_at,
        },
    )
    .await
    .expect("create should succeed");

    let revoked = refresh_tokens::revoke_by_token(&pool, &token_value, "logout")
        .await
        .expect("revoke should succeed");
    assert!(revoked, "first revoke should return true");

    let result = refresh_tokens::find_active_by_token(&pool, &token_value)
        .await
        .expect("find should not error");
    assert!(result.is_none(), "revoked token should not be active");
}

#[sqlx::test(migrations = "./test-schema")]
async fn revoke_by_token_is_idempotent(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "dave", company_id).await;
    let token_value = make_token_uuid();
    let expires_at = (Utc::now() + ChronoDuration::days(30)).naive_utc();

    refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: token_value.clone(),
            expires_at,
        },
    )
    .await
    .expect("create should succeed");

    let first = refresh_tokens::revoke_by_token(&pool, &token_value, "logout")
        .await
        .expect("first revoke should succeed");
    assert!(first);

    let second = refresh_tokens::revoke_by_token(&pool, &token_value, "logout")
        .await
        .expect("second revoke should not error");
    assert!(!second, "second revoke should return false (idempotent)");
}

#[sqlx::test(migrations = "./test-schema")]
async fn revoke_by_token_unknown_returns_false(pool: MySqlPool) {
    let unknown = make_token_uuid();
    let result = refresh_tokens::revoke_by_token(&pool, &unknown, "logout")
        .await
        .expect("revoke should not error on unknown token");
    assert!(!result);
}

#[sqlx::test(migrations = "./test-schema")]
async fn revoke_all_for_user_revokes_all_active_tokens(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "eve", company_id).await;
    let expires_at = (Utc::now() + ChronoDuration::days(30)).naive_utc();

    // 3 tokens actifs pour ce user
    for _ in 0..3 {
        refresh_tokens::create(
            &pool,
            NewRefreshToken {
                user_id,
                token: make_token_uuid(),
                expires_at,
            },
        )
        .await
        .expect("create should succeed");
    }

    let revoked_count = refresh_tokens::revoke_all_for_user(&pool, user_id, "password_change")
        .await
        .expect("revoke_all should succeed");
    assert_eq!(revoked_count, 3);

    // Un second appel ne doit rien révoquer
    let second_call = refresh_tokens::revoke_all_for_user(&pool, user_id, "password_change")
        .await
        .expect("second revoke_all should succeed");
    assert_eq!(second_call, 0);
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_with_unknown_user_id_fails_with_fk_violation(pool: MySqlPool) {
    let result = refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id: 99999,
            token: make_token_uuid(),
            expires_at: (Utc::now() + ChronoDuration::days(30)).naive_utc(),
        },
    )
    .await;

    assert!(matches!(result, Err(DbError::ForeignKeyViolation(_))));
}

#[sqlx::test(migrations = "./test-schema")]
async fn create_with_invalid_token_format_fails_check_constraint(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "frank", company_id).await;

    // #890 : `create` hache toute entrée, si bien que la contrainte de la
    // colonne ne se voit plus par lui ; elle reste la garde de la colonne, et
    // se constate par un INSERT brut.
    let result =
        sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind("not-a-uuid")
            .bind((Utc::now() + ChronoDuration::days(30)).naive_utc())
            .execute(&pool)
            .await
            .map_err(kesh_db::errors::map_db_error);

    assert!(matches!(result, Err(DbError::CheckConstraintViolation(_))));
}

/// #890 — `create` d'une valeur qui n'est pas un UUID réussit : seule son
/// empreinte, toujours conforme à la contrainte, est stockée.
#[sqlx::test(migrations = "./test-schema")]
async fn create_hashes_any_value_into_a_valid_digest(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "frida", company_id).await;

    let created = refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: "not-a-uuid".to_string(),
            expires_at: (Utc::now() + ChronoDuration::days(30)).naive_utc(),
        },
    )
    .await
    .expect("create hache la valeur");
    assert_eq!(
        created.token_digest,
        refresh_tokens::token_digest("not-a-uuid")
    );
    assert_eq!(&created.token_digest[14..15], "0");
}

#[sqlx::test(migrations = "./test-schema")]
async fn deleting_user_cascades_to_refresh_tokens(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "grace", company_id).await;
    let token_value = make_token_uuid();

    refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: token_value.clone(),
            expires_at: (Utc::now() + ChronoDuration::days(30)).naive_utc(),
        },
    )
    .await
    .expect("create should succeed");

    // DELETE direct du user via SQL (le repo users n'expose pas de delete)
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(user_id)
        .execute(&pool)
        .await
        .expect("delete user should succeed");

    // Le refresh_token doit avoir été cascaded
    let found = refresh_tokens::find_active_by_token(&pool, &token_value)
        .await
        .expect("find should not error");
    assert!(
        found.is_none(),
        "refresh_token should be cascaded when user is deleted"
    );
}

/// Story v0174-session-1 (#890, AC-4) — le nettoyage du démarrage efface une
/// ligne forme 0.17.3 (UUID v4 clair, active, échéance future) et garde une
/// ligne créée par `create` (empreinte, active, échéance future).
#[sqlx::test(migrations = "./test-schema")]
async fn cleanup_deletes_legacy_clear_rows_and_keeps_digests(pool: MySqlPool) {
    let company_id = create_test_company(&pool).await;
    let user_id = create_test_user(&pool, "henri", company_id).await;
    let future = (Utc::now() + ChronoDuration::minutes(30)).naive_utc();

    let legacy = make_token_uuid();
    let inserted =
        sqlx::query("INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES (?, ?, ?)")
            .bind(user_id)
            .bind(&legacy)
            .bind(future)
            .execute(&pool)
            .await
            .expect("ligne forme 0.17.3");
    assert_eq!(inserted.rows_affected(), 1);
    let kept = refresh_tokens::create(
        &pool,
        NewRefreshToken {
            user_id,
            token: make_token_uuid(),
            expires_at: future,
        },
    )
    .await
    .expect("create should succeed");

    let cutoff = (Utc::now() - ChronoDuration::days(7)).naive_utc();
    let removed = refresh_tokens::delete_expired_and_revoked(&pool, cutoff)
        .await
        .expect("cleanup");
    assert_eq!(removed, 1, "seule la ligne forme 0.17.3 est effacée");

    let ids: Vec<i64> = sqlx::query_scalar("SELECT id FROM refresh_tokens WHERE user_id = ?")
        .bind(user_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(ids, vec![kept.id], "la ligne à empreinte reste");
}
