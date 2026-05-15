//! Entité `AuditLogEntry` — journal d'audit des actions utilisateurs.
//!
//! Conformément au Code des obligations suisse (art. 957-964), les
//! entrées d'audit sont **inamovibles** : pas de repository `delete`.
//! La FK `users.id ON DELETE RESTRICT` empêche de supprimer un
//! utilisateur qui a laissé des traces d'audit (conservation 10 ans
//! obligatoire).
//!
//! Scope v0.1 (story 3.3) : `journal_entry.updated`, `journal_entry.deleted`.
//! Story 3.5 étendra avec `journal_entry.created` et l'UI de consultation.

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

/// Sentinelle `entity_id` pour les actions audit sans entité concrète
/// (rapports, consultations agrégées, exports, etc. — Story 9-1).
///
/// Garantie d'unicité sémantique : les `id` réels d'entités sont en `AUTO_INCREMENT`
/// qui démarre à 1 — `0` ne correspond à aucune entité réelle.
///
/// **IMPORTANT — utilisation correcte** : pour distinguer plusieurs actions audit
/// avec `entity_id = 0`, **toujours filtrer sur la combinaison `(entity_type, entity_id)`**,
/// jamais sur `entity_id` seul. Exemple :
///   SELECT * FROM audit_log WHERE entity_type = 'report' AND entity_id = AUDIT_ENTITY_ID_NONE;
pub const AUDIT_ENTITY_ID_NONE: i64 = 0;

/// Entrée du journal d'audit persistée en base.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub details_json: Option<serde_json::Value>,
    pub created_at: NaiveDateTime,
}

/// Données de création d'une entrée d'audit.
#[derive(Debug, Clone)]
pub struct NewAuditLogEntry {
    pub user_id: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub details_json: Option<serde_json::Value>,
}
