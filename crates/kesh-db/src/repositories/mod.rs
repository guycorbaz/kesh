//! Repositories : fonctions libres par entité pour les opérations CRUD.
//!
//! Un fichier par entité. Pattern : `create`, `find_by_id`, `update`, `list`
//! comme API standard. Les méthodes spécifiques (`find_by_username`,
//! `list_by_company`, `close`) s'ajoutent là où c'est nécessaire.

pub mod accounts;
pub mod bank_accounts;
pub mod companies;
pub mod fiscal_years;
pub mod onboarding;
pub mod refresh_tokens;
pub mod users;

/// Limite haute pour les appels `list()` : évite les OOM par `fetch_all`
/// sur de gros résultats. S'applique à toutes les entités.
pub const MAX_LIST_LIMIT: i64 = 1000;
