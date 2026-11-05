//! Story v0174-correctifs-2 (#842, AC-2.11) — une requête abandonnée (client
//! déconnecté : Axum abandonne la future du gestionnaire) pendant qu'elle tient
//! le verrou d'un compte bancaire (`with_account_lock`) ne laisse plus ce
//! verrou tenu par la connexion rendue au pool : le crochet du pool
//! (`kesh_db::pool::avec_remise_a_zero_de_session`) le rend au retour de la
//! connexion. Témoin sans crochet : le verrou reste tenu au-delà de 2 s.

use std::time::{Duration, Instant};

use kesh_reconciliation::errors::ReconciliationError;
use kesh_reconciliation::mutex::with_account_lock;
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

const COMPANY: i64 = 842;
const ACCOUNT: i64 = 7;

async fn pool_of_one(pool: &MySqlPool, with_reset: bool) -> MySqlPool {
    let options = MySqlPoolOptions::new().max_connections(1);
    let options = if with_reset {
        kesh_db::pool::avec_remise_a_zero_de_session(options)
    } else {
        options
    };
    options
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("pool d'une connexion")
}

/// Le nom du verrou que prend `with_account_lock` (`mutex.rs`).
async fn account_lock_name(pool: &MySqlPool) -> String {
    let db: String = sqlx::query_scalar("SELECT COALESCE(DATABASE(), '')")
        .fetch_one(pool)
        .await
        .unwrap();
    format!("reconcile:{db}:{COMPANY}:{ACCOUNT}")
}

/// Une requête qui prend le verrou du compte, puis attend indéfiniment, est
/// abandonnée au bout de 100 ms ; sa transaction est abandonnée avec elle.
async fn abandoned_request(p: &MySqlPool) {
    let mut tx = p.begin().await.unwrap();
    let r = tokio::time::timeout(
        Duration::from_millis(100),
        with_account_lock(&mut tx, COMPANY, ACCOUNT, 0, async |_tx| {
            std::future::pending::<Result<(), ReconciliationError>>().await
        }),
    )
    .await;
    assert!(r.is_err(), "la requête doit être abandonnée");
    drop(tx);
}

/// `true` si une connexion indépendante prend le verrou au plus tard `within`
/// après l'appel (sondage à 50 ms), et le rend aussitôt.
async fn lock_taken_by_another(pool: &MySqlPool, name: &str, within: Duration) -> bool {
    let deadline = Instant::now() + within;
    loop {
        let mut conn = pool.acquire().await.unwrap();
        let got: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
            .bind(name)
            .fetch_one(&mut *conn)
            .await
            .unwrap();
        if got == Some(1) {
            let _: Option<i32> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
                .bind(name)
                .fetch_one(&mut *conn)
                .await
                .unwrap();
            return true;
        }
        drop(conn);
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}

/// AC-2.11 — avec le crochet, le verrou du compte est rendu après l'abandon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn abandoned_request_does_not_keep_the_account_locked_842(pool: MySqlPool) {
    let name = account_lock_name(&pool).await;
    let p = pool_of_one(&pool, true).await;
    abandoned_request(&p).await;
    assert!(
        lock_taken_by_another(&pool, &name, Duration::from_secs(2)).await,
        "le verrou du compte doit être rendu au retour de la connexion au pool"
    );
    p.close().await;
}

/// AC-2.11, témoin — sans le crochet, le verrou reste tenu par la connexion
/// du pool, au-delà de 2 s.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn without_the_hook_the_account_stays_locked_842(pool: MySqlPool) {
    let name = account_lock_name(&pool).await;
    let p = pool_of_one(&pool, false).await;
    abandoned_request(&p).await;
    assert!(
        !lock_taken_by_another(&pool, &name, Duration::from_secs(2)).await,
        "témoin : sans crochet, le verrou survit à la requête"
    );
    p.close().await;
}
