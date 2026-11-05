//! Story v0174-correctifs-2 (#842, #540 ; AC-2.9, AC-2.10) — la session d'une
//! connexion est remise à zéro **au retour au pool** : un verrou nommé
//! (`GET_LOCK`) laissé tenu par une requête abandonnée est rendu, et
//! `FOREIGN_KEY_CHECKS` est rétabli. Sans le crochet (témoins), la connexion
//! revient au pool avec son état de session, et le pool la garde.
//!
//! Chaque montage prend un pool **d'une** connexion, pour que la connexion
//! réacquise soit celle qui a servi, et observe depuis une connexion
//! indépendante (le pool du test). Les noms de verrou portent le nom de la base
//! éphémère : `GET_LOCK` est global au serveur, et les tests tournent en
//! parallèle.

use std::time::{Duration, Instant};

use kesh_db::pool::avec_remise_a_zero_de_session;
use sqlx::mysql::MySqlPoolOptions;
use sqlx::{MySqlPool, Row};

/// Un pool d'une connexion sur la base du test, avec ou sans le crochet.
async fn pool_of_one(pool: &MySqlPool, with_reset: bool) -> MySqlPool {
    let options = MySqlPoolOptions::new().max_connections(1);
    let options = if with_reset {
        avec_remise_a_zero_de_session(options)
    } else {
        options
    };
    options
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("pool d'une connexion")
}

async fn lock_name(pool: &MySqlPool) -> String {
    let db: String = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(pool)
        .await
        .unwrap();
    format!("kesh-test-{db}")
}

/// `true` si le verrou devient libre, vu d'une connexion indépendante, au plus
/// tard `within` après l'appel (sondage à 50 ms : la restitution au pool est
/// asynchrone).
async fn becomes_free(observer: &MySqlPool, name: &str, within: Duration) -> bool {
    let deadline = Instant::now() + within;
    loop {
        let free: Option<i64> = sqlx::query_scalar("SELECT IS_FREE_LOCK(?)")
            .bind(name)
            .fetch_one(observer)
            .await
            .unwrap();
        if free == Some(1) {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// Prend le verrou sur la connexion du pool d'une connexion, puis l'abandonne
/// **sans** `RELEASE_LOCK` (retour au pool).
async fn take_lock_and_drop(p: &MySqlPool, name: &str) {
    let mut conn = p.acquire().await.unwrap();
    let got: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
        .bind(name)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(got, Some(1), "verrou pris");
    drop(conn);
}

/// AC-2.9 — avec le crochet, le verrou de session est rendu au retour au pool.
#[sqlx::test(migrations = "./test-schema")]
async fn session_lock_is_released_when_the_connection_returns_to_the_pool(pool: MySqlPool) {
    let name = lock_name(&pool).await;
    let p = pool_of_one(&pool, true).await;
    take_lock_and_drop(&p, &name).await;
    assert!(
        becomes_free(&pool, &name, Duration::from_secs(2)).await,
        "le verrou doit être rendu au retour de la connexion au pool"
    );
    p.close().await;
}

/// AC-2.9, témoin — sans le crochet, la connexion revient au pool en tenant le
/// verrou, au-delà de 2 s.
#[sqlx::test(migrations = "./test-schema")]
async fn without_the_hook_the_lock_outlives_the_request(pool: MySqlPool) {
    let name = lock_name(&pool).await;
    let p = pool_of_one(&pool, false).await;
    take_lock_and_drop(&p, &name).await;
    assert!(
        !becomes_free(&pool, &name, Duration::from_secs(2)).await,
        "témoin : sans crochet, le verrou reste tenu par la connexion du pool"
    );
    p.close().await;
    assert!(
        becomes_free(&pool, &name, Duration::from_secs(2)).await,
        "la fermeture de la connexion libère le verrou"
    );
}

/// `@@session.foreign_key_checks` d'une connexion réacquise après qu'une autre
/// requête l'a posé à 0, et l'identifiant des deux connexions.
async fn fk_checks_after_reacquire(p: &MySqlPool) -> (i64, u64, u64) {
    let mut conn = p.acquire().await.unwrap();
    sqlx::query("SET SESSION foreign_key_checks = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    let first: u64 = sqlx::query_scalar("SELECT CONNECTION_ID()")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    let mut conn = p.acquire().await.unwrap();
    let row = sqlx::query("SELECT @@session.foreign_key_checks AS fk, CONNECTION_ID() AS id")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    (row.get::<i64, _>("fk"), first, row.get::<u64, _>("id"))
}

/// AC-2.10 (#540) — avec le crochet, `FOREIGN_KEY_CHECKS` est rétabli à 1.
#[sqlx::test(migrations = "./test-schema")]
async fn foreign_key_checks_is_restored_when_the_connection_returns(pool: MySqlPool) {
    let p = pool_of_one(&pool, true).await;
    let (fk, first, second) = fk_checks_after_reacquire(&p).await;
    assert_eq!(first, second, "la même connexion, réacquise");
    assert_eq!(fk, 1, "FOREIGN_KEY_CHECKS rétabli au retour au pool");
    p.close().await;
}

/// AC-2.10, témoin — sans le crochet, la connexion revient avec
/// `FOREIGN_KEY_CHECKS = 0`.
#[sqlx::test(migrations = "./test-schema")]
async fn without_the_hook_foreign_key_checks_stays_off(pool: MySqlPool) {
    let p = pool_of_one(&pool, false).await;
    let (fk, first, second) = fk_checks_after_reacquire(&p).await;
    assert_eq!(first, second, "la même connexion, réacquise");
    assert_eq!(fk, 0, "témoin : l'état de session survit sans crochet");
    p.close().await;
}
