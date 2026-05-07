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
    // H5 Pass 1 — i32 binding explicite (MariaDB attend signed int) +
    // distinguer NULL vs 0 vs valeur inattendue.
    let acquired: Option<i32> = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name)
        .bind(timeout_secs as i32)
        .fetch_one(&mut **tx)
        .await
        .map_err(ReconciliationError::Database)?;
    match acquired {
        Some(1) => { /* lock acquis, on continue */ }
        Some(0) => {
            return Err(ReconciliationError::AccountLocked {
                bank_account_id,
                timeout_secs,
            });
        }
        Some(other) => {
            tracing::error!(
                ?other,
                ?bank_account_id,
                "GET_LOCK returned unexpected non-0/1 value"
            );
            return Err(ReconciliationError::Database(sqlx::Error::Protocol(
                format!("GET_LOCK unexpected return: {other}"),
            )));
        }
        None => {
            tracing::error!(
                ?bank_account_id,
                "GET_LOCK returned NULL — DB internal error"
            );
            return Err(ReconciliationError::Database(sqlx::Error::Protocol(
                "GET_LOCK returned NULL".to_string(),
            )));
        }
    }

    let result = f(tx).await;

    // HP3-1 + HP6-1 + C4 Pass 1 — RELEASE_LOCK explicite avant retour.
    // M3 Pass 1 : utiliser query_scalar pour récupérer la valeur de
    // retour (Some(1) = released, Some(0) = was not held, None =
    // lock didn't exist). C4 Pass 1 : préserver l'erreur business
    // (`result`) si elle existe, prioritaire sur une erreur RELEASE.
    let release_outcome: Result<Option<i32>, sqlx::Error> =
        sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
            .bind(&lock_name)
            .fetch_one(&mut **tx)
            .await;

    match (result, release_outcome) {
        // Cas nominal : business OK + release OK Some(1).
        (Ok(value), Ok(Some(1))) => Ok(value),
        // Business OK mais release problématique (Some(0)/None ou Err) :
        // remonter LockReleaseFailed en log + Err. La valeur business
        // est sacrifiée — le caller voit l'erreur et drop tx_outer.
        (Ok(_value), release) => match release {
            Ok(release_status) => {
                tracing::error!(
                    ?release_status,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK returned non-1 after successful business op; \
                     lock will leak until session end (cf. L22)"
                );
                Err(ReconciliationError::LockReleaseFailed {
                    bank_account_id,
                    source: sqlx::Error::Protocol(format!(
                        "RELEASE_LOCK returned {release_status:?}"
                    )),
                })
            }
            Err(e) => {
                tracing::error!(
                    ?e,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK failed after successful business op; \
                     lock will leak until session end (cf. L22)"
                );
                Err(ReconciliationError::LockReleaseFailed {
                    bank_account_id,
                    source: e,
                })
            }
        },
        // Erreur business prime ; warning si release a aussi échoué.
        (Err(business_err), release) => {
            if let Err(release_err) = release {
                tracing::warn!(
                    ?release_err,
                    ?bank_account_id,
                    lock_name = %lock_name,
                    "RELEASE_LOCK also failed during error path (business error preserved)"
                );
            } else if let Ok(release_status) = release {
                if release_status != Some(1) {
                    tracing::warn!(
                        ?release_status,
                        ?bank_account_id,
                        lock_name = %lock_name,
                        "RELEASE_LOCK returned non-1 during error path \
                         (business error preserved)"
                    );
                }
            }
            Err(business_err)
        }
    }
}
