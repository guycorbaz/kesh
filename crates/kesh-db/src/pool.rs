//! Configuration du pool de connexions MariaDB via SQLx.

use std::time::Duration;

use sqlx::mysql::{MySqlPool, MySqlPoolOptions};
use sqlx::Executor;

use crate::errors::{map_db_error, DbError};

/// Crée un pool SQLx connecté à MariaDB.
///
/// Chaque nouvelle connexion applique automatiquement :
/// - `sql_mode = 'STRICT_ALL_TABLES,NO_ZERO_DATE,NO_ZERO_IN_DATE,NO_ENGINE_SUBSTITUTION'`
///   pour rejeter les troncations silencieuses et les dates invalides
/// - `time_zone = '+00:00'` pour garantir la cohérence UTC avec `NaiveDateTime`
///
/// L'isolation reste celle par défaut de MariaDB (`REPEATABLE READ`), qui est
/// appropriée pour les transactions d'optimistic locking utilisées dans ce crate.
/// Passer à `READ COMMITTED` pourrait être envisagé ultérieurement si le volume
/// de lectures concurrentes pose problème, mais introduirait des phantoms dans
/// les lectures adjacentes aux updates.
///
/// # Arguments
///
/// - `database_url` : URL de connexion (`mysql://user:pass@host:port/db`)
/// - `max_connections` : taille maximale du pool (recommandé : 5 pour le MVP)
/// - `connect_timeout` : timeout d'acquisition d'une connexion (recommandé : 10 secondes)
pub async fn create_pool(
    database_url: &str,
    max_connections: u32,
    connect_timeout: Duration,
) -> Result<MySqlPool, DbError> {
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(connect_timeout)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                conn.execute(
                    "SET SESSION sql_mode = 'STRICT_ALL_TABLES,NO_ZERO_DATE,NO_ZERO_IN_DATE,NO_ENGINE_SUBSTITUTION', \
                         time_zone = '+00:00'",
                )
                .await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await
        .map_err(map_db_error)
}
