//! Errors du module reconciliation.

use chrono::NaiveDate;

/// Erreurs spécifiques au flow de réconciliation Story 8-4.
#[derive(Debug, thiserror::Error)]
pub enum ReconciliationError {
    /// `GET_LOCK` timeout — un autre flow tient le lock sur ce compte.
    #[error(
        "advisory lock for bank_account_id={bank_account_id} not acquired \
         within {timeout_secs}s timeout"
    )]
    AccountLocked {
        bank_account_id: i64,
        timeout_secs: u32,
    },

    /// `RELEASE_LOCK` failure — connexion poisoned, lock potentiellement
    /// retenu jusqu'à fin de session MariaDB (HP3-1 Pass 3 + HP4-1
    /// Pass 4 wording correction). Cf. L22 + §mutex-account.
    #[error(
        "RELEASE_LOCK failed for bank_account_id={bank_account_id} \
         (lock may persist until session end)"
    )]
    LockReleaseFailed {
        bank_account_id: i64,
        #[source]
        source: sqlx::Error,
    },

    /// Erreur DB générique (mappage `From<sqlx::Error>`).
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Story 8-5a-base — l'exercice fiscal couvrant `entry_date` est
    /// soit inexistant soit `Closed` (helper
    /// `fiscal_years::find_open_covering_date` retourne `None` pour
    /// les deux cas, cf. L46). Le handler mappe vers
    /// `AppError::ReconciliationFiscalYearClosed { entry_date }` →
    /// HTTP 409 `RECONCILIATION_FISCAL_YEAR_CLOSED`.
    ///
    /// **Distinct de `Database(DbError::FiscalYearClosed)`** : ce
    /// variant est émis depuis la closure `with_account_lock` du flow
    /// manual quand le pré-flight `find_open_covering_date` retourne
    /// `None`. Si la race step 6 → step 7 frappe et `create_in_tx`
    /// retourne `DbError::FiscalYearClosed`, c'est le variant
    /// `Db(DbError)` qui prend le relais (mapping → 400 v0.1, cf.
    /// §validation-handler-side step 7).
    #[error("fiscal year closed/not found for entry_date={entry_date}")]
    FiscalYearClosed { entry_date: NaiveDate },

    /// Story 8-5a-base F1'''' Pass 6 Opus — wrapper typé pour
    /// `kesh_db::errors::DbError`. **Sans ce variant, les helpers
    /// `kesh-db` (qui retournent `Result<_, DbError>`) ne peuvent pas
    /// être appelés via `?` dans la closure `with_account_lock` car
    /// `DbError` n'est pas convertible en `sqlx::Error`** (`DbError`
    /// est un enum distinct, pas `From<DbError> for sqlx::Error`).
    ///
    /// Le handler `post_manual` (kesh-api) match ce variant en
    /// `Err(ReconciliationError::Db(db_err)) => Err(AppError::Database(db_err))`
    /// — le sous-match exhaustif sur `DbError` dans `errors.rs:922+`
    /// se charge ensuite du mapping HTTP fin :
    /// - `DbError::FiscalYearClosed` → 400 (race step 6 → step 7)
    /// - `DbError::OptimisticLockConflict` → 409 (race UPDATE bank_tx)
    /// - `DbError::Invariant`/`Sqlx` → 500 catch-all
    ///
    /// **Path-dep** : réutilisé par 8-5a-bis (split flow) et 8-5b
    /// (accept-with-rule flow) pour la même raison.
    ///
    /// **Non-régression 8-4** : les closures 8-4
    /// (`accept_batch`/`reject_batch`) continuent d'émettre
    /// `Database(sqlx::Error)` via `.map_err` manuel — ne PAS les
    /// refactorer en `?` (cela changerait le variant émis).
    /// Les match blocs handlers 8-4 (`post_accept`/`post_reject`)
    /// reçoivent malgré tout la branche `Db` pour exhaustivité du
    /// compilateur (unreachable en pratique).
    #[error("kesh-db error: {0}")]
    Db(#[from] kesh_db::errors::DbError),
}
