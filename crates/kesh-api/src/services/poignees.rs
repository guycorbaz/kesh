//! Les deux poignées sur la base : [`Lecture`] pour lire les livres, [`Trace`]
//! pour écrire le journal d'audit.
//!
//! Deux types distincts, sans conversion de l'un à l'autre, à champ privé : un
//! service qui reçoit une `Lecture` ne peut pas en faire une `Trace`, et le pool
//! d'une poignée n'est lisible que dans `crate::services`. En production, elles
//! ne se fabriquent que par `AppState::lecture()`, `AppState::trace()` et, pour
//! la lecture des outils du MCP, au câblage de `build_router_avec_registre`
//! (Story 32-6a).
//!
//! La lecture seule du MCP (Q11, tranchée le 2026-10-24) est tenue par
//! l'application — les outils n'atteignent que des services qui reçoivent une
//! `Lecture` — et par le mode de session `READ ONLY` de la connexion des
//! outils (`kesh_db::pool::pool_lecture_seule`, 32-6a) ; aucun second compte
//! MariaDB. `AppState::lecture()` et `AppState::trace()` enveloppent le pool
//! ordinaire.

use sqlx::MySqlPool;

/// Poignée de **lecture** des livres : dépôts, générateurs de rapports.
///
/// Aucune écriture ne passe par elle (règle R4 de `services`).
#[derive(Clone)]
pub struct Lecture(MySqlPool);

impl Lecture {
    /// Enveloppe un pool. En production, `AppState::lecture()` et le câblage du
    /// MCP, `build_router_avec_registre` (sur le pool en lecture seule des
    /// outils), l'appellent.
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Le pool de lecture, pour les dépôts et les générateurs — visible de la
    /// seule couche de services.
    pub(in crate::services) fn pool(&self) -> &MySqlPool {
        &self.0
    }
}

/// Poignée d'**écriture de la trace** : le journal d'audit des lectures.
#[derive(Clone)]
pub struct Trace(MySqlPool);

impl Trace {
    /// Enveloppe un pool. En production, seul `AppState::trace()` l'appelle.
    pub fn new(pool: MySqlPool) -> Self {
        Self(pool)
    }

    /// Le pool de la trace, destiné aux seules fonctions d'audit de
    /// `services/rapports.rs` et à la dernière utilisation d'une clé MCP
    /// (`services/cles_mcp.rs`, Story 32-5b). ⚠️ La visibilité ne peut pas être
    /// plus étroite (`pub(in …)` n'accepte qu'un module ancêtre, E0742) :
    /// qu'aucun autre service ne l'appelle est tenu par la revue.
    pub(in crate::services) fn pool(&self) -> &MySqlPool {
        &self.0
    }
}
