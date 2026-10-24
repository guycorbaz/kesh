//! Les deux poignées sur la base : [`Lecture`] pour lire les livres, [`Trace`]
//! pour écrire le journal d'audit.
//!
//! Deux types distincts, sans conversion de l'un à l'autre, à champ privé : un
//! service qui reçoit une `Lecture` ne peut pas en faire une `Trace`, et le pool
//! d'une poignée n'est lisible que dans `crate::services`. En production, elles
//! ne se fabriquent que par `AppState::lecture()` et `AppState::trace()`.
//!
//! La fabrication d'une `Lecture` en lecture seule — mode de session,
//! utilisateur MariaDB limité au `SELECT`, ou les deux — est la 32-6, selon la
//! décision de Guy sur Q11 ; rien ici n'en préjuge : en 32-3, les deux poignées
//! enveloppent le même pool.

use sqlx::MySqlPool;

/// Poignée de **lecture** des livres : dépôts, générateurs de rapports.
///
/// Aucune écriture ne passe par elle (règle R4 de `services`).
#[derive(Clone)]
pub struct Lecture(MySqlPool);

impl Lecture {
    /// Enveloppe un pool. En production, seul `AppState::lecture()` l'appelle.
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
    /// `services/rapports.rs`. ⚠️ La visibilité ne peut pas être plus étroite
    /// (`pub(in …)` n'accepte qu'un module ancêtre, E0742) : qu'aucun autre
    /// service ne l'appelle est tenu par la revue.
    pub(in crate::services) fn pool(&self) -> &MySqlPool {
        &self.0
    }
}
