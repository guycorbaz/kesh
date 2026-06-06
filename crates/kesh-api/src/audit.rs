//! Pont d'audit `CurrentUser` → `NewAuditLogEntry` (Story 17-2a, DC5).
//!
//! **Pourquoi ici et pas dans kesh-db** : `from_current_user` dépend de
//! [`CurrentUser`] (kesh-api). kesh-db ne peut PAS dépendre de kesh-api
//! (cycle de crates). Les constructeurs « bas niveau » `NewAuditLogEntry::user`
//! / `::api_key` (qui ne prennent que des `i64`) vivent dans kesh-db ; ce
//! trait d'extension les sélectionne selon `CurrentUser.api_key_id`
//! (correction F-OPUS-1 : la frontière `api_key` vs `user` est « a-t-on un
//! `&CurrentUser` en scope », pas kesh-api vs kesh-db).

use kesh_db::entities::audit_log::NewAuditLogEntry;

use crate::middleware::auth::CurrentUser;

/// Extension de [`NewAuditLogEntry`] pour construire une entrée d'audit à
/// partir du `CurrentUser` courant, en propageant automatiquement
/// l'attribution `actor_type` (`User` si JWT, `ApiKey` si PAT).
pub trait AuditActor {
    /// Construit une entrée d'audit attribuée au `CurrentUser` :
    /// - chemin JWT (`api_key_id = None`) → `actor_type = User` (sémantique
    ///   historique préservée — invariant de non-régression).
    /// - chemin PAT (`api_key_id = Some(id)`) → `actor_type = ApiKey`,
    ///   `actor_api_key_id = Some(id)`, `user_id = créateur de la clé`.
    fn from_current_user(
        user: &CurrentUser,
        action: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: i64,
        details_json: Option<serde_json::Value>,
    ) -> NewAuditLogEntry;
}

impl AuditActor for NewAuditLogEntry {
    fn from_current_user(
        user: &CurrentUser,
        action: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: i64,
        details_json: Option<serde_json::Value>,
    ) -> NewAuditLogEntry {
        match user.api_key_id {
            Some(api_key_id) => NewAuditLogEntry::api_key(
                api_key_id,
                user.user_id,
                action,
                entity_type,
                entity_id,
                details_json,
            ),
            None => {
                NewAuditLogEntry::user(user.user_id, action, entity_type, entity_id, details_json)
            }
        }
    }
}
