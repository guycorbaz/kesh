//! Story 28-3 (#633) — Instantané de lecture : une connexion, une vue de la base.
//!
//! [`ReadSnapshot`] porte **toutes** les lectures de la sauvegarde
//! d'installation (`backup::export_installation`) : elles passent par une seule
//! connexion, dans une seule transaction, et voient donc la base à un seul
//! instant. Sans lui, chaque table était lue sur le pool, en autocommit, dans sa
//! propre vue : une suppression committée entre la lecture d'une table fille et
//! celle de sa table parente laissait dans le fichier des lignes sans parent.
//!
//! Réutilisable par l'export global CSV (#465), qui n'en bénéficie pas encore.
//!
//! ⚠️ **T1 — forme dégradée** : ce commit pose l'API seule (constructeurs, accès
//! à la connexion, `finish`) sans ouvrir de transaction, pour que les tests de
//! l'instantané rougissent d'abord sur ce même texte (C-28-3-16).

use sqlx::pool::PoolConnection;
use sqlx::{MySql, MySqlConnection, MySqlPool};

use crate::errors::{DbError, map_db_error};

/// Instantané de lecture cohérent sur une seule connexion du pool.
///
/// Se construit par [`ReadSnapshot::begin`] et se clôt par
/// [`ReadSnapshot::finish`] ; les requêtes s'exécutent sur
/// [`ReadSnapshot::conn`]. Le type ne donne accès ni au pool ni à la
/// `PoolConnection` : une lecture qui le reçoit ne peut pas s'en échapper.
pub struct ReadSnapshot {
    conn: PoolConnection<MySql>,
}

impl ReadSnapshot {
    /// Acquiert une connexion du pool et y ouvre l'instantané.
    pub async fn begin(pool: &MySqlPool) -> Result<Self, DbError> {
        let conn = pool.acquire().await.map_err(map_db_error)?;
        Self::begin_on(conn).await
    }

    /// Ouvre l'instantané sur une connexion déjà acquise (tests de session).
    pub(crate) async fn begin_on(conn: PoolConnection<MySql>) -> Result<Self, DbError> {
        Ok(Self { conn })
    }

    /// La connexion de l'instantané, pour y exécuter les lectures.
    pub fn conn(&mut self) -> &mut MySqlConnection {
        &mut self.conn
    }

    /// Clôt l'instantané et rend la connexion au pool.
    pub async fn finish(self) -> Result<(), DbError> {
        drop(self.conn);
        Ok(())
    }
}
