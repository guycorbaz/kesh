//! Errors du module reconciliation.

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
}
