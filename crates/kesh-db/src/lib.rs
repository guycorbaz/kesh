//! kesh-db — Couche de persistance MariaDB via SQLx.
//!
//! Repository pattern avec fonctions libres par entité. Aucune dépendance HTTP
//! ou réseau : ce crate se concentre sur le schéma, les entités et les accès
//! DB. Les types métier avec validation (ex: `CheNumber`) vivent dans
//! `kesh-core` et sont validés côté `kesh-api` avant l'appel au repository.

pub mod entities;
pub mod errors;
pub mod pool;
pub mod repositories;

/// Migrator SQLx chargé depuis `crates/kesh-db/migrations/`.
///
/// Utilisé par `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` pour appliquer
/// les migrations automatiquement à chaque test d'intégration, et par
/// l'application au démarrage pour initialiser/mettre à jour le schéma.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
