//! Advisory lock per bank account (Story 8-4 §mutex-account).
//!
//! Helper [`with_account_lock`] acquiert un advisory lock MariaDB
//! `GET_LOCK('reconcile:{company_id}:{bank_account_id}', timeout_secs)`
//! pour serializer les flows accept/reject sur le même compte bancaire.
//!
//! **Choix vs alternatives** :
//! - ❌ `SELECT ... FOR UPDATE` row-level : sémantique floue, plus
//!   long que nécessaire.
//! - ❌ Tokio Mutex en mémoire : pas multi-instance.
//! - ✅ Advisory lock MariaDB : portable multi-instance, nommé,
//!   libère à fin de session si app crash.
//!
//! **HP3-1 + HP4-1 + HP5-1 — caller pattern** : sur
//! `Err(LockReleaseFailed)`, le handler **DOIT** :
//!
//! ```ignore
//! match with_account_lock(&mut tx_outer, ...).await {
//!     Ok(result) => result,
//!     Err(ReconciliationError::LockReleaseFailed { bank_account_id, .. }) => {
//!         drop(tx_outer); // Drop impl rollback + retour pool
//!         return Err(AppError::ReconciliationLockReleaseFailed { bank_account_id });
//!     }
//!     Err(e) => return Err(map_reconciliation_error(e)),
//! }
//! ```
//!
//! **Anti-patterns** : `pool.close().await` (ferme TOUT le pool, outage),
//! `connection.detach()` (méthode sur PoolConnection inaccessible via
//! `&mut Transaction`), `tx_outer.rollback().await` (redondant avec Drop).

use crate::errors::ReconciliationError;

/// Acquiert un advisory lock MariaDB sur `(company_id, bank_account_id)`,
/// exécute la closure `f` dans la transaction sous le lock, puis
/// libère explicitement le lock.
///
/// Retourne :
/// - `Ok(T)` si la closure réussit ET le lock est libéré sans erreur.
/// - `Err(AccountLocked)` si l'acquisition timeout.
/// - `Err(LockReleaseFailed)` si la closure réussit mais
///   `RELEASE_LOCK` échoue (cas pathologique connexion poisoned —
///   le caller doit drop tx_outer pour rollback + retour pool, le
///   lock advisory sera libéré à fin de session MariaDB cf. L22).
/// - `Err(...)` propagé tel quel depuis la closure.
pub async fn with_account_lock<F, T>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    timeout_secs: u32,
    f: F,
) -> Result<T, ReconciliationError>
where
    F: AsyncFnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T, ReconciliationError>,
{
    let lock_name = format!("reconcile:{company_id}:{bank_account_id}");
    let acquired: i32 = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name)
        .bind(timeout_secs)
        .fetch_one(&mut **tx)
        .await?;
    if acquired != 1 {
        return Err(ReconciliationError::AccountLocked {
            bank_account_id,
            timeout_secs,
        });
    }
    let result = f(tx).await;
    // HP3-1 + HP6-1 — RELEASE_LOCK explicite avant retour. En cas
    // d'erreur, retourner LockReleaseFailed (le caller laissera
    // tx_outer drop pour rollback + retour pool). Ne PAS appeler
    // pool.close() ni connection.detach() (cf. doc-comment).
    let release_result = sqlx::query("SELECT RELEASE_LOCK(?)")
        .bind(&lock_name)
        .execute(&mut **tx)
        .await;
    if let Err(e) = release_result {
        tracing::error!(
            ?e,
            lock_name = %lock_name,
            "RELEASE_LOCK failed — connection returns to pool with \
             lock potentially held ; advisory lock will release at \
             session end (cf. L22)"
        );
        return Err(ReconciliationError::LockReleaseFailed {
            bank_account_id,
            source: e,
        });
    }
    result
}
