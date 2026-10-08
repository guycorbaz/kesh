//! Enveloppe `AppError` du rejeu sur interblocage (Story 15-5e1, AC2).
//!
//! Le rejeu lui-même — la règle, le backoff, la journalisation — est celui de
//! [`kesh_db::retry`] : ce module n'en est que la variante pour les routes
//! dont la transaction est ouverte **dans le handler** (fonction « une
//! tentative » qui rend `Result<_, AppError>`), là où l'enveloppe `DbError`
//! [`kesh_db::retry::retry_on_deadlock`] sert les routes dont l'écriture est
//! une seule fonction de dépôt.
//!
//! Les règles de l'enveloppe `DbError` valent ici à l'identique : une
//! tentative = une transaction neuve (`pool.begin()` … `commit()`), fermeture
//! `Fn` qui clone par tentative ce qu'elle consomme, aucun effet de bord hors
//! transaction, et aucune conversion qui ferait sortir une erreur sqlx de
//! `AppError::Database(DbError::Sqlx(_))` avant le prédicat — sans quoi le
//! rejeu deviendrait muet. Les contrôles qui lisent ce que la transaction
//! verrouille restent **dans** la tentative, dans le même ordre.

use std::future::Future;

use kesh_db::retry::{DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_deadlock_error, retry_with};

use crate::errors::AppError;

/// Vrai si `err` est un interblocage InnoDB (1213) remonté par la base.
///
/// Seul `AppError::Database(db)` peut l'être, et seulement si
/// [`kesh_db::retry::is_deadlock_error`] reconnaît `db` : un dépassement
/// d'attente (1205) ou une erreur métier ne le sont pas.
pub fn is_app_deadlock(err: &AppError) -> bool {
    matches!(err, AppError::Database(db) if is_deadlock_error(db))
}

/// **Enveloppe `AppError`** : exécute `f` et la rejoue, jusqu'à
/// `DEFAULT_MAX_DEADLOCK_ATTEMPTS` tentatives, tant qu'elle échoue sur un
/// interblocage ([`is_app_deadlock`]).
///
/// `operation` nomme la route (p. ex. `"reconciliation::manual"`) ; il est
/// porté par le `warn!` de cible `kesh_db::retry` émis avant chaque nouvelle
/// tentative. Toute autre erreur est rendue au premier essai.
pub async fn retry_app_on_deadlock<F, Fut, T>(operation: &'static str, f: F) -> Result<T, AppError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    retry_with(operation, DEFAULT_MAX_DEADLOCK_ATTEMPTS, is_app_deadlock, f).await
}
