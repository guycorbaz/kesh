//! Story 28-3 (#633) — Instantané de lecture : une connexion, une vue de la base.
//!
//! [`ReadSnapshot`] porte **toutes** les lectures de la sauvegarde
//! d'installation (`backup::export_installation`) : elles passent par une seule
//! connexion, dans une seule transaction, et voient donc la base à un seul
//! instant. Sans lui, chaque table était lue sur le pool, en autocommit, dans sa
//! propre vue : une suppression committée entre la lecture d'une table fille et
//! celle de sa table parente laissait dans le fichier des lignes sans parent.
//!
//! L'instantané s'ouvre, sur la connexion, par :
//!
//! ```sql
//! SET TRANSACTION ISOLATION LEVEL REPEATABLE READ;   -- transaction suivante seulement
//! START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY;
//! ```
//!
//! - `SET TRANSACTION` **sans** `SESSION` ne vaut que pour la transaction
//!   suivante : la connexion rendue au pool garde le niveau de sa session. Il
//!   est nécessaire même si le serveur est en `REPEATABLE READ` par défaut :
//!   sous `READ COMMITTED`, `WITH CONSISTENT SNAPSHOT` ne fige pas la vue.
//! - `WITH CONSISTENT SNAPSHOT` prend la vue **à l'ouverture**, non à la
//!   première lecture.
//! - `READ ONLY` : toute écriture dans l'instantané est refusée par le serveur.
//!
//! Les lectures sont cohérentes et **non verrouillantes** : aucune saisie
//! n'attend la sauvegarde. La garantie ne vaut que pour les tables InnoDB
//! (manuel d'administration, § *Exporter (sauvegarde complète)*).
//!
//! ⚠️ **Dépendance signalée** : l'ouverture passe par `sqlx::Transaction::begin`,
//! exportée mais `#[doc(hidden)]` (sqlx 0.8.6, `sqlx-core/src/transaction.rs:98`),
//! donc hors contrat semver. C'est la seule API qui rende une transaction
//! `'static` sur une connexion où l'on vient de poser le `SET TRANSACTION`
//! (`Connection::begin_with` emprunte la connexion ; `Pool::begin_with` en prend
//! une neuve). Non épinglée (`Cargo.toml` porte `0.8`, seul `Cargo.lock` tient
//! la 0.8.6) : à une montée de sqlx, les tests de `backup::tests` (session
//! `READ COMMITTED`, connexion rendue hors transaction, session inchangée) sont
//! ceux qui diraient qu'un comportement a changé.
//!
//! Réutilisable par l'export global CSV (#465), qui n'en bénéficie pas encore.

use sqlx::pool::PoolConnection;
use sqlx::{Executor, MySql, MySqlConnection, MySqlPool, Transaction};

use crate::errors::{DbError, map_db_error};

/// Niveau d'isolation de la seule transaction suivante (protocole texte).
const SET_ISOLATION: &str = "SET TRANSACTION ISOLATION LEVEL REPEATABLE READ";

/// Ouverture de l'instantané, passée à `sqlx::Transaction::begin`.
const START_SNAPSHOT: &str = "START TRANSACTION WITH CONSISTENT SNAPSHOT, READ ONLY";

/// Instantané de lecture cohérent sur une seule connexion du pool.
///
/// Se construit par [`ReadSnapshot::begin`] et se clôt par
/// [`ReadSnapshot::finish`] ; les requêtes s'exécutent sur
/// [`ReadSnapshot::conn`]. Le type ne donne accès ni au pool ni à la
/// `PoolConnection` : une lecture qui le reçoit ne peut pas s'en échapper.
///
/// Abandonné sans `finish` (chemin d'erreur), il est annulé par le `Drop` de
/// `sqlx::Transaction`, et la connexion revient au pool hors transaction.
pub struct ReadSnapshot {
    tx: Transaction<'static, MySql>,
}

impl ReadSnapshot {
    /// Acquiert une connexion du pool et y ouvre l'instantané.
    pub async fn begin(pool: &MySqlPool) -> Result<Self, DbError> {
        let conn = pool.acquire().await.map_err(map_db_error)?;
        Self::begin_on(conn).await
    }

    /// Ouvre l'instantané sur une connexion déjà acquise (tests de session).
    ///
    /// Échec du `SET TRANSACTION` : la connexion est marquée pour fermeture,
    /// elle ne revient pas au pool dans un état incertain. Échec du `START`
    /// (connexion déjà cédée à sqlx) : résidu assumé, le niveau « pour la
    /// prochaine transaction » peut rester posé (C-28-3-19, C-28-3-30).
    pub(crate) async fn begin_on(mut conn: PoolConnection<MySql>) -> Result<Self, DbError> {
        // `&str` passé à `Executor::execute` : protocole texte, comme le
        // `BEGIN` de sqlx — et non une requête préparée.
        if let Err(e) = (&mut *conn).execute(SET_ISOLATION).await {
            conn.close_on_drop();
            return Err(map_db_error(e));
        }
        // ⛔ Jamais un `START TRANSACTION` brut par `execute` : sqlx ne saurait
        // pas la connexion en transaction et la rendrait au pool ouverte.
        let tx = Transaction::begin(conn, Some(START_SNAPSHOT.into()))
            .await
            .map_err(map_db_error)?;
        Ok(Self { tx })
    }

    /// La connexion de l'instantané, pour y exécuter les lectures.
    pub fn conn(&mut self) -> &mut MySqlConnection {
        &mut self.tx
    }

    /// Clôt l'instantané (`ROLLBACK` : rien à valider) et rend la connexion au
    /// pool. Une erreur du `ROLLBACK` est rendue : l'appelant la propage
    /// (C-28-3-20).
    pub async fn finish(self) -> Result<(), DbError> {
        self.tx.rollback().await.map_err(map_db_error)
    }
}
