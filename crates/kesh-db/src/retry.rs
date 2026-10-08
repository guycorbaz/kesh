//! Rejeu des transactions victimes d'un interblocage InnoDB (1213).
//!
//! **Ce qu'InnoDB fait d'un interblocage.** InnoDB détecte un cycle de verrous
//! **au moment de l'attente** (`innodb_deadlock_detect`, `ON` par défaut) et
//! annule **aussitôt** une transaction victime, qui reçoit l'erreur 1213
//! (`ER_LOCK_DEADLOCK`) — il n'attend pas `innodb_lock_wait_timeout`, que ce
//! cycle traverse une seule table ou plusieurs. La victime est la transaction
//! la plus **légère** (lignes modifiées et verrous tenus), non la plus jeune ;
//! les tests « route victime » de `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
//! et `accept_replays_the_batch_when_it_is_the_deadlock_victim`
//! (`reconciliation_e2e.rs`) reposent sur cette règle. Toute la transaction
//! de la victime est annulée : rien n'en est écrit, et la rejouer depuis son
//! `begin()` est sûr.
//!
//! **Pourquoi un rejeu, et non un ordre des verrous** (Story 15-5e1, choix
//! C54). Tout flux qui écrit au journal reprend, à l'insertion des lignes, un
//! verrou partagé sur chaque compte écrit (`fk_jel_account`) **après** le
//! verrou de l'exercice : aucun ordre « comptes avant exercice » ne tient de
//! bout en bout. L'ordre des verrous écrit aux doc-comments canoniques
//! (`invoices::validate_invoice`, `supplier_invoices::create_in_tx`) est une
//! convention qui **réduit la fréquence** des interblocages ; le rejeu des
//! routes les rend **invisibles** à l'utilisateur. Les routes rejouées passent
//! par l'enveloppe `DbError` de ce module ([`retry_on_deadlock`]) ou par
//! l'enveloppe `AppError` de `kesh_api::retry` ; l'inventaire des routes qui
//! écrivent au journal et leur statut de rejeu sont tenus par le registre
//! `crates/kesh-api/tests/audit_route_registry.rs`.
//!
//! **Ce qu'InnoDB ne détecte pas.** Un cycle qui passe par un verrou nommé
//! `GET_LOCK` (`kesh-reconciliation/src/mutex.rs`) **et** un verrou de ligne
//! n'est pas vu par le détecteur : il finit en 1205
//! (`innodb_lock_wait_timeout`) ou en `GET_LOCK = 0` (409
//! `RECONCILIATION_ACCOUNT_LOCKED`), jamais rejoué. Aucun ne se forme
//! aujourd'hui parce que **chaque flux de rapprochement prend son `GET_LOCK`
//! avant tout verrou de ligne** — invariant à garder. Et le rejeu suppose
//! `innodb_deadlock_detect = ON` : désactivé, chaque 1213 devient un 1205 que
//! rien ne rejoue.
//!
//! **Journalisation.** Chaque nouvelle tentative émet un `tracing::warn!` de
//! cible `kesh_db::retry` qui porte le nom de l'opération (`operation`, p. ex.
//! `"invoices::settle"`), le numéro de la tentative et le backoff : la ligne
//! dit quelle route a été rejouée, même sous `RUST_LOG=warn`.
//!
//! Cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 pour la convention
//! d'ordre des verrous, qui réduit la fréquence des interblocages sans les
//! exclure.

use crate::errors::DbError;
use std::future::Future;
use std::time::Duration;

/// Code d'erreur MariaDB / MySQL pour `ER_LOCK_DEADLOCK`.
///
/// InnoDB rollback automatiquement la tx victime et renvoie ce code à
/// l'application. Le retry redémarre la tx du début.
const MARIADB_DEADLOCK_ERROR_CODE: u16 = 1213;

/// Nombre maximum de tentatives par défaut (1 essai initial + 2 rejeux).
///
/// Choix conservateur :
/// - Sous charge nominale, la probabilité d'un interblocage répété est très
///   faible (chaque rejeu attend que les autres transactions finissent).
/// - 3 tentatives → 2 pauses (entre 1↔2 et 2↔3) ≈ 150 ms de latence au pire
///   (50 + 100 ms de backoff). La 3e tentative n'est suivie d'aucune pause —
///   l'enveloppe rend l'erreur si elle échoue. L'alternative au rejeu n'est
///   pas une attente : InnoDB annule la victime aussitôt, et sans rejeu
///   l'utilisateur reçoit un 500 immédiat.
/// - Au-delà de 3, le risque d'une boucle qui empile la latence dépasse
///   la valeur ajoutée du rejeu.
pub const DEFAULT_MAX_DEADLOCK_ATTEMPTS: u32 = 3;

/// Backoff initial entre la 1re et la 2e tentative.
///
/// Suffisamment court pour ne pas dégrader la latence en cas de retry
/// nécessaire, suffisamment long pour laisser la tx concurrente finir
/// (la durée typique d'une tx FOR UPDATE est < 50 ms).
const INITIAL_BACKOFF_MS: u64 = 50;

/// Borne supérieure sur le backoff (M-002 review remediation).
///
/// L'API publique `retry_with` accepte `max_attempts: u32`. Sans cap, un
/// caller exotique avec `max_attempts >= 65` produirait `2u64.pow(63+)`
/// → panic en debug, sleep absurde en release (jusqu'à >292 ans). Cette
/// borne saturente l'attente à 30 secondes, ce qui dépasse déjà
/// `innodb_lock_wait_timeout` par défaut (50 s) — au-delà ça n'a plus
/// de sens, mieux vaut surface l'erreur au caller.
const MAX_BACKOFF_MS: u64 = 30_000;

/// Détermine si une `sqlx::Error` est un deadlock MariaDB (code 1213).
///
/// Les autres erreurs DB (contrainte unique, FK, syntax, etc.) ne sont PAS
/// considérées comme retryable — un retry sur ces erreurs masquerait des
/// bugs réels. **Note** : MariaDB surface le deadlock via le numéro
/// d'erreur 1213, jamais via le SQLSTATE ANSI `40001` seul. On match donc
/// strictement sur `MySqlDatabaseError::number()` pour éviter les faux
/// positifs (cf. L-004 review).
///
/// **Hors scope** : `ER_LOCK_WAIT_TIMEOUT` (code 1205) n'est PAS retried.
/// Ce code surface après `innodb_lock_wait_timeout` (50 s) — différent
/// d'un cycle de deadlock détecté instantanément. Un retry sur 1205
/// risquerait simplement de re-déclencher la même attente.
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

/// **Enveloppe `DbError`** : exécute une fermeture asynchrone et la rejoue
/// si elle échoue sur un interblocage (1213). Utilise
/// `DEFAULT_MAX_DEADLOCK_ATTEMPTS` (3).
///
/// Elle sert les routes dont l'écriture est **une** fonction de dépôt qui
/// ouvre et conclut sa propre transaction et rend `Result<_, DbError>`
/// (Story 15-5e1, AC2). Règles :
/// - **une tentative = une transaction neuve** : la fermeture commence par
///   `pool.begin()` (ou appelle une fonction qui le fait) et finit par le
///   `commit()` ; InnoDB a déjà annulé la transaction de la victime ;
/// - la fermeture est `Fn` : ce que la route consomme par déplacement (le
///   corps de la requête, un `New…` construit à partir de lui) se **clone
///   dans** la fermeture, à chaque tentative ;
/// - aucun effet de bord hors transaction (e-mail, fichier, réseau) dans la
///   fermeture ;
/// - la conversion en `AppError` se fait **après** le rejeu, jamais dans la
///   fermeture : une erreur sqlx convertie en une autre variante rendrait le
///   rejeu muet.
///
/// `operation` nomme la route (p. ex. `"invoices::settle"`) ; il est porté
/// par le `warn!` émis avant chaque nouvelle tentative.
///
/// # Examples
///
/// ```ignore
/// let entry = retry_on_deadlock("journal_entries::create", || {
///     let new = new.clone();
///     async move { journal_entries::create(&pool, fy_id, user_id, new).await }
/// })
/// .await?;
/// ```
pub async fn retry_on_deadlock<F, Fut, T>(operation: &'static str, f: F) -> Result<T, DbError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, DbError>>,
{
    retry_on_deadlock_with(operation, DEFAULT_MAX_DEADLOCK_ATTEMPTS, f).await
}

/// Variante de `retry_on_deadlock` avec `max_attempts` paramétrable.
///
/// `max_attempts` doit être ≥ 1 (1 = pas de retry, équivaut à appeler
/// la closure une fois). Une valeur `0` est traitée comme `1`.
pub async fn retry_on_deadlock_with<F, Fut, T>(
    operation: &'static str,
    max_attempts: u32,
    f: F,
) -> Result<T, DbError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, DbError>>,
{
    retry_with(
        operation,
        max_attempts,
        |e: &DbError| is_deadlock_error(e),
        f,
    )
    .await
}

/// Helper de retry générique pour tout type d'erreur `E`.
///
/// Le caller fournit un prédicat `should_retry: Fn(&E) -> bool` qui décide
/// de la nature retryable de chaque erreur. Utilisé par `kesh-api` pour
/// retryer sur `AppError::Database(DbError::Sqlx(deadlock))` sans devoir
/// convertir l'erreur en `DbError` côté handler.
///
/// Backoff exponentiel : 50 ms × 2^(attempt-1). `max_attempts` clampé à ≥ 1.
///
/// `operation` nomme l'opération rejouée (p. ex. `"reconciliation::accept"`) :
/// avant chaque nouvelle tentative, un `tracing::warn!` de cible
/// `kesh_db::retry` porte ce nom (champ `operation`), le numéro de la
/// tentative, le nombre maximal et le backoff. Une erreur épuisée n'est pas
/// journalisée ici : elle remonte à l'appelant (500 journalisé en erreur).
pub async fn retry_with<F, Fut, T, E, P>(
    operation: &'static str,
    max_attempts: u32,
    should_retry: P,
    f: F,
) -> Result<T, E>
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
                // M-002 review remediation : `2u64.pow(attempt-1)` panic en
                // debug pour `attempt >= 65` ; en release ça wrap silencieusement.
                // Saturating shift + cap à `MAX_BACKOFF_MS` rend l'API safe pour
                // tout `max_attempts: u32` sans changer le comportement réel
                // dans la plage utile (DEFAULT=3 → 50, 100 ms).
                let multiplier = 1u64.checked_shl(attempt - 1).unwrap_or(u64::MAX);
                let backoff_ms = INITIAL_BACKOFF_MS
                    .saturating_mul(multiplier)
                    .min(MAX_BACKOFF_MS);
                let backoff = Duration::from_millis(backoff_ms);
                tracing::warn!(
                    target: "kesh_db::retry",
                    operation,
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
        let result: Result<i32, DbError> = retry_on_deadlock("test::retry", || {
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
        let result: Result<i32, DbError> = retry_on_deadlock("test::retry", || {
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
        let result: Result<i32, DbError> = retry_on_deadlock_with("test::retry", 1, || {
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
        let _: Result<i32, DbError> = retry_on_deadlock_with("test::retry", 0, || {
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
            retry_on_deadlock("test::retry", || {
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
            retry_on_deadlock("test::retry", || {
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
