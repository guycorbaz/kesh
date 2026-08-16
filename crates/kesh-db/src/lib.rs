//! kesh-db — Couche de persistance MariaDB via SQLx.
//!
//! Repository pattern avec fonctions libres par entité. Aucune dépendance HTTP
//! ou réseau : ce crate se concentre sur le schéma, les entités et les accès
//! DB. Les types métier avec validation (ex: `CheNumber`) vivent dans
//! `kesh-core` et sont validés côté `kesh-api` avant l'appel au repository.

pub mod backfill;
pub mod backup;
pub mod entities;
pub mod errors;
pub mod pool;
pub mod post_restore;
pub mod repositories;
pub mod retry;
pub mod test_fixtures;
pub mod util;
pub mod version;

/// Migrator SQLx chargé depuis `crates/kesh-db/migrations/`.
///
/// Utilisé par l'application au démarrage pour initialiser/mettre à jour le
/// schéma, et par les seuls tests qui exercent **le chemin des migrations
/// lui-même** (installation fraîche, fenêtres d'upgrade, backfills, garde-fou
/// de schéma). Le reste de la suite d'intégration monte le squash
/// `test-schema/` — cf. Story 22-5 / issue #251.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");
