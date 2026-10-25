//! Configuration du pool de connexions MariaDB via SQLx.

use std::time::Duration;

use sqlx::Executor;
use sqlx::mysql::{MySqlPool, MySqlPoolOptions};

use crate::errors::{DbError, map_db_error};

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

/// Le pool **en lecture seule** des outils du serveur MCP — Story 32-6a
/// (AC-a25, C-32-6-9, C-32-6-35 ; Q11, tranchée le 2026-10-24).
///
/// Copié de `depuis` (mêmes options de connexion, même délai d'acquisition) ;
/// **aucun autre réglage de session** que ces deux-là, le pool de production
/// n'en ayant pas (#669) : l'API et le MCP lisent la même chose.
///
/// - `after_connect` : `SET SESSION TRANSACTION READ ONLY` — MariaDB refuse
///   alors toute écriture et tout `SELECT … FOR UPDATE` (erreur 1792) ;
/// - `after_connect` encore : `SET SESSION max_statement_time = <delai_instruction>`
///   (secondes, au millième) — **MariaDB interrompt lui-même** toute instruction
///   qui dépasse ce délai (erreur 1969), compté **depuis le début de
///   l'instruction** côté serveur (revue P1 de la 32-6a, E1 = B2 ; test (g)).
///   ⚠️ Cette borne **ne suffit pas** à borner la charge : une instruction dont
///   la connexion est fermée en vol continue côté serveur jusqu'à
///   `delai_instruction` après **son** début — non après l'abandon —, alors que
///   le permis du pool est déjà rendu (revue P2, H1). Le répartiteur du MCP
///   l'évite par construction : l'outil s'exécute dans une tâche qui garde ses
///   connexions jusqu'à la fin de l'instruction (C-32-6-56) ;
/// - `after_release` rend **toujours** `Ok(false)` : **chaque connexion est
///   fermée à sa restitution** (`sqlx-core` 0.8.6, `pool/connection.rs`), si
///   bien que chaque emprunt part d'une connexion neuve, passée par
///   `after_connect`. Aucun état de session ne survit d'un emprunt à l'autre,
///   quelle que soit l'instruction qui l'aurait défait — la forme sans
///   `SESSION`, qui ne vaut que pour la transaction suivante et que `@@session`
///   ne montre pas, comprise. Coût : une connexion par emprunt.
///
/// ⚠️ **Ce que cette fabrique ne ferme pas** : une forme qui défait le mode de
/// session **dans** un emprunt, suivie d'une écriture dans le même emprunt. La
/// garde lexicale `lecture_seule_jamais_defaite`
/// (`kesh-api/tests/mcp_gardes_lexicales.rs`) lit le texte du dépôt ; un SQL
/// composé à l'exécution lui échappe.
///
/// ⚠️ `connect_lazy_with` lance les tâches d'entretien du pool : l'appel exige
/// un runtime Tokio.
///
/// # Panics
///
/// Si `delai_instruction` est nul : `max_statement_time = 0` voudrait dire
/// « aucune limite ».
pub fn pool_lecture_seule(
    depuis: &MySqlPool,
    max_connections: u32,
    delai_instruction: Duration,
) -> MySqlPool {
    let millis = delai_instruction.as_millis();
    assert!(millis > 0, "pool_lecture_seule : délai d'instruction nul");
    let borne = format!(
        "SET SESSION max_statement_time = {}.{:03}",
        millis / 1000,
        millis % 1000
    );
    MySqlPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(depuis.options().get_acquire_timeout())
        .after_connect(move |conn, _meta| {
            let borne = borne.clone();
            Box::pin(async move {
                conn.execute("SET SESSION TRANSACTION READ ONLY").await?;
                conn.execute(borne.as_str()).await?;
                Ok(())
            })
        })
        .after_release(|_conn, _meta| Box::pin(async move { Ok(false) }))
        .connect_lazy_with((*depuis.connect_options()).clone())
}
