//! Helpers de retry pour les erreurs DB transitoires.
//!
//! Usage typique : envelopper un handler ou une section qui acquiert
//! plusieurs `SELECT ... FOR UPDATE` et qui pourrait deadlocker contre une
//! tx concurrente prenant les mêmes locks dans l'ordre inverse.
//!
//! KF-002-H-002 (#43) : MariaDB ne détecte pas les deadlocks cross-table
//! avant `innodb_lock_wait_timeout` (50s par défaut), surfaçant comme un
//! `500 Internal Server Error` côté user. En enveloppant les sections
//! sensibles dans `retry_on_deadlock`, on rejoue automatiquement la tx
//! après un backoff exponentiel court — la deadlock-victim est typiquement
//! choisie par InnoDB (rollback de la tx la plus jeune), donc le retry
//! repart proprement.
//!
//! Cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 pour la doc des
//! ordres de locks canoniques.

use crate::errors::DbError;
use std::future::Future;
use std::time::Duration;

/// Code d'erreur MariaDB / MySQL pour `ER_LOCK_DEADLOCK`.
///
/// InnoDB rollback automatiquement la tx victime et renvoie ce code à
/// l'application. Le retry redémarre la tx du début.
const MARIADB_DEADLOCK_ERROR_CODE: u16 = 1213;

/// Nombre maximum de tentatives par défaut (1 essai initial + 2 retries).
///
/// Choix conservateur :
/// - Sous charge nominale, la probabilité de deadlock répété est très
///   faible (chaque retry attend que les autres tx finissent).
/// - 3 tentatives ≈ 350 ms de latence pire cas (50 + 100 + 200 ms backoff)
///   AVANT que l'erreur surface au client — acceptable vs. un 500 sur
///   `innodb_lock_wait_timeout`.
/// - Au-delà de 3, le risque d'une boucle qui empile la latence dépasse
///   la valeur ajoutée du retry.
pub const DEFAULT_MAX_DEADLOCK_ATTEMPTS: u32 = 3;

/// Backoff initial entre la 1re et la 2e tentative.
///
/// Suffisamment court pour ne pas dégrader la latence en cas de retry
/// nécessaire, suffisamment long pour laisser la tx concurrente finir
/// (la durée typique d'une tx FOR UPDATE est < 50 ms).
const INITIAL_BACKOFF_MS: u64 = 50;

/// Détermine si une `sqlx::Error` est un deadlock MariaDB (code 1213).
///
/// Les autres erreurs DB (contrainte unique, FK, syntax, etc.) ne sont PAS
/// considérées comme retryable — un retry sur ces erreurs masquerait des
/// bugs réels.
fn is_deadlock_sqlx(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db_err| db_err.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
        .map(|my_err| my_err.number() == MARIADB_DEADLOCK_ERROR_CODE)
        .unwrap_or(false)
}

/// Variante publique : détermine si une `DbError` est un deadlock retryable.
///
/// Exposé pour permettre aux callers `kesh-api` de construire leur propre
/// prédicat sur leur type d'erreur (ex. `AppError::Database(db) if
/// is_deadlock_error(db)`).
pub fn is_deadlock_error(err: &DbError) -> bool {
    matches!(err, DbError::Sqlx(sqlx_err) if is_deadlock_sqlx(sqlx_err))
}

/// Exécute une closure asynchrone avec retry automatique en cas de
/// deadlock MariaDB. Utilise `DEFAULT_MAX_DEADLOCK_ATTEMPTS` (3).
///
/// La closure DOIT être idempotente : elle sera ré-appelée intégralement
/// si le 1er essai deadlock. En pratique cela signifie qu'elle commence
/// par un `pool.begin()` et finit par `tx.commit()` (le ROLLBACK est
/// automatique sur deadlock côté InnoDB).
///
/// # Examples
///
/// ```ignore
/// retry_on_deadlock(|| async {
///     let mut tx = pool.begin().await?;
///     // ... SELECT FOR UPDATE + UPDATE ...
///     tx.commit().await?;
///     Ok(())
/// }).await?
/// ```
pub async fn retry_on_deadlock<F, Fut, T>(f: F) -> Result<T, DbError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, DbError>>,
{
    retry_on_deadlock_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, f).await
}

/// Variante de `retry_on_deadlock` avec `max_attempts` paramétrable.
///
/// `max_attempts` doit être ≥ 1 (1 = pas de retry, équivaut à appeler
/// la closure une fois). Une valeur `0` est traitée comme `1`.
pub async fn retry_on_deadlock_with<F, Fut, T>(max_attempts: u32, f: F) -> Result<T, DbError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, DbError>>,
{
    retry_with(max_attempts, |e: &DbError| is_deadlock_error(e), f).await
}

/// Helper de retry générique pour tout type d'erreur `E`.
///
/// Le caller fournit un prédicat `should_retry: Fn(&E) -> bool` qui décide
/// de la nature retryable de chaque erreur. Utilisé par `kesh-api` pour
/// retryer sur `AppError::Database(DbError::Sqlx(deadlock))` sans devoir
/// convertir l'erreur en `DbError` côté handler.
///
/// Backoff exponentiel : 50 ms × 2^(attempt-1). `max_attempts` clampé à ≥ 1.
pub async fn retry_with<F, Fut, T, E, P>(max_attempts: u32, should_retry: P, f: F) -> Result<T, E>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    P: Fn(&E) -> bool,
{
    let attempts = max_attempts.max(1);
    let mut attempt: u32 = 0;
    loop {
        attempt += 1;
        let result = f().await;
        match result {
            Ok(value) => return Ok(value),
            Err(err) => {
                if !should_retry(&err) || attempt >= attempts {
                    return Err(err);
                }
                let backoff = Duration::from_millis(INITIAL_BACKOFF_MS * 2u64.pow(attempt - 1));
                tracing::debug!(
                    target: "kesh_db::retry",
                    attempt,
                    max_attempts = attempts,
                    backoff_ms = backoff.as_millis() as u64,
                    "retryable error detected, retrying after backoff"
                );
                tokio::time::sleep(backoff).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// Erreur sqlx synthétique factice (PAS un deadlock) pour vérifier que
    /// les erreurs non-retryables passent immédiatement.
    fn fake_non_deadlock_error() -> DbError {
        DbError::OptimisticLockConflict
    }

    #[tokio::test]
    async fn returns_ok_on_first_success_no_retry() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<i32, DbError> = retry_on_deadlock(|| {
            let calls = calls_clone.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(42)
            }
        })
        .await;
        assert_eq!(result.unwrap(), 42);
        assert_eq!(calls.load(Ordering::SeqCst), 1, "1 seul appel attendu");
    }

    #[tokio::test]
    async fn passes_non_deadlock_errors_through_immediately() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<i32, DbError> = retry_on_deadlock(|| {
            let calls = calls_clone.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(fake_non_deadlock_error())
            }
        })
        .await;
        assert!(matches!(result, Err(DbError::OptimisticLockConflict)));
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "OptimisticLockConflict ne doit PAS être retryé"
        );
    }

    #[tokio::test]
    async fn max_attempts_one_means_no_retry() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<i32, DbError> = retry_on_deadlock_with(1, || {
            let calls = calls_clone.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(fake_non_deadlock_error())
            }
        })
        .await;
        assert!(result.is_err());
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn max_attempts_zero_treated_as_one() {
        let calls = Arc::new(AtomicU32::new(0));
        let calls_clone = calls.clone();
        let _: Result<i32, DbError> = retry_on_deadlock_with(0, || {
            let calls = calls_clone.clone();
            async move {
                calls.fetch_add(1, Ordering::SeqCst);
                Err(fake_non_deadlock_error())
            }
        })
        .await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "max_attempts=0 doit être traité comme 1, pas une boucle infinie"
        );
    }

    /// Test d'intégration : trigger un vrai deadlock InnoDB via 2 tx
    /// concurrentes prenant des locks dans l'ordre inverse, et vérifie que
    /// `retry_on_deadlock` fait passer la victime. Sans le wrapper, l'une
    /// des tx surfacerait `Err(Sqlx)` avec code 1213.
    ///
    /// **Stratégie sans barrière** : un `tokio::sync::Barrier` ne survit pas
    /// à un retry (la closure rappelée tenterait `barrier.wait()` une 2e
    /// fois alors que l'autre task a déjà avancé). On utilise donc un délai
    /// court (`tokio::time::sleep`) entre la 1re et la 2e acquisition de
    /// lock — suffisant pour que les deux tx tiennent leur 1er lock
    /// simultanément et entrent en cycle de deadlock dès la 2e tentative.
    ///
    /// **Outcome attendu** : InnoDB détecte le cycle (instantané, pas
    /// d'attente sur `innodb_lock_wait_timeout`) et rollback une des tx
    /// avec ER_LOCK_DEADLOCK (1213). `retry_on_deadlock` rappelle la
    /// closure ; au 2e essai la tx victorieuse a commit → la victime
    /// acquiert ses locks normalement (sans contention) et commit.
    ///
    /// **Si le deadlock ne se reproduit pas** (timing variable, OS scheduler),
    /// les deux tx commit séquentiellement sans deadlock ; le test passe
    /// quand même car les invariants finaux (val_1==1, val_2==1) restent
    /// vrais. Un deadlock garanti exigerait un point de synchro côté DB
    /// (SELECT GET_LOCK p.ex.), out-of-scope pour ce sanity test.
    #[tokio::test]
    async fn integration_real_deadlock_recovers_via_retry() {
        dotenvy::dotenv().ok();
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required for DB tests");
        let pool = sqlx::mysql::MySqlPoolOptions::new()
            .max_connections(4)
            .connect(&url)
            .await
            .expect("pool connect");

        // Table sentinelle dédiée — évite de polluer une table métier.
        // DROP+CREATE plutôt que CREATE IF NOT EXISTS + DELETE : évite de
        // bloquer sur un metadata lock si une exécution précédente a laissé
        // une connexion zombie qui détient des row-locks (vu une fois en dev,
        // résolu en killant la connexion idle). DROP libère implicitement
        // les row-locks de la victime (DDL prend le metadata lock après
        // attente du lock-wait-timeout, mais sans table c'est immédiat).
        sqlx::query("DROP TABLE IF EXISTS retry_test_sentinel")
            .execute(&pool)
            .await
            .expect("drop sentinel (cleanup pre-run)");
        sqlx::query(
            "CREATE TABLE retry_test_sentinel (\
                id BIGINT PRIMARY KEY,\
                value INT NOT NULL\
             ) ENGINE=InnoDB",
        )
        .execute(&pool)
        .await
        .expect("create sentinel table");
        sqlx::query("INSERT INTO retry_test_sentinel (id, value) VALUES (1, 0), (2, 0)")
            .execute(&pool)
            .await
            .expect("seed sentinel");

        let pool_a = pool.clone();
        let pool_b = pool.clone();

        let task_a = tokio::spawn(async move {
            retry_on_deadlock(|| {
                let pool = pool_a.clone();
                async move {
                    let mut tx = pool.begin().await.map_err(crate::errors::map_db_error)?;
                    sqlx::query("SELECT id FROM retry_test_sentinel WHERE id = 1 FOR UPDATE")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    // Pause courte : laisse à task_b le temps d'acquérir SON
                    // 1er lock, ce qui maximise la probabilité de deadlock
                    // sur la 2e acquisition. Si la pause expire avant que B
                    // ait son lock, A passe en série (pas de deadlock, c'est
                    // ok — le test reste valide).
                    tokio::time::sleep(Duration::from_millis(80)).await;
                    sqlx::query("SELECT id FROM retry_test_sentinel WHERE id = 2 FOR UPDATE")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    sqlx::query("UPDATE retry_test_sentinel SET value = value + 1 WHERE id = 1")
                        .execute(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    tx.commit().await.map_err(crate::errors::map_db_error)?;
                    Ok::<_, DbError>(())
                }
            })
            .await
        });

        let task_b = tokio::spawn(async move {
            retry_on_deadlock(|| {
                let pool = pool_b.clone();
                async move {
                    let mut tx = pool.begin().await.map_err(crate::errors::map_db_error)?;
                    sqlx::query("SELECT id FROM retry_test_sentinel WHERE id = 2 FOR UPDATE")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    tokio::time::sleep(Duration::from_millis(80)).await;
                    sqlx::query("SELECT id FROM retry_test_sentinel WHERE id = 1 FOR UPDATE")
                        .fetch_one(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    sqlx::query("UPDATE retry_test_sentinel SET value = value + 1 WHERE id = 2")
                        .execute(&mut *tx)
                        .await
                        .map_err(crate::errors::map_db_error)?;
                    tx.commit().await.map_err(crate::errors::map_db_error)?;
                    Ok::<_, DbError>(())
                }
            })
            .await
        });

        let res_a = task_a.await.expect("task A panicked");
        let res_b = task_b.await.expect("task B panicked");

        // Avec le retry wrapper, les deux tx doivent finir Ok — la deadlock
        // victime est replay-ée après que l'autre ait commit. Si pas de
        // deadlock effectif (timing manqué), elles commit séquentiellement.
        assert!(
            res_a.is_ok(),
            "task A doit réussir via retry, got {res_a:?}"
        );
        assert!(
            res_b.is_ok(),
            "task B doit réussir via retry, got {res_b:?}"
        );

        // Vérifie que les deux UPDATE ont effectivement été appliqués.
        let (val_1,): (i32,) = sqlx::query_as("SELECT value FROM retry_test_sentinel WHERE id = 1")
            .fetch_one(&pool)
            .await
            .expect("read sentinel 1");
        let (val_2,): (i32,) = sqlx::query_as("SELECT value FROM retry_test_sentinel WHERE id = 2")
            .fetch_one(&pool)
            .await
            .expect("read sentinel 2");
        assert_eq!(val_1, 1, "task A doit avoir updaté la ligne 1");
        assert_eq!(val_2, 1, "task B doit avoir updaté la ligne 2");

        // Cleanup : drop la table sentinelle (idempotent).
        sqlx::query("DROP TABLE IF EXISTS retry_test_sentinel")
            .execute(&pool)
            .await
            .ok();
        pool.close().await;
    }
}
