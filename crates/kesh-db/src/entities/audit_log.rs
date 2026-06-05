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
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Type d'acteur d'une entrée d'audit (Story 17-2a, DC5).
///
/// Stocké en DB en kebab/snake-case : `"user"`, `"api_key"`.
/// - `User` : action exécutée par un humain via l'UI web (chemin JWT). C'est
///   la sémantique **historique** — toute entrée d'audit pré-17-2a est `User`,
///   et la migration met `DEFAULT 'user'` (non-breaking).
/// - `ApiKey` : mutation exécutée via un PAT (`Authorization: Bearer
///   kesh_pat_…`). Dans ce cas `actor_api_key_id = Some(<id clé>)` et
///   `user_id = créateur de la clé` (imputabilité conservée).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ActorType {
    #[default]
    User,
    ApiKey,
}

impl ActorType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::ApiKey => "api_key",
        }
    }
}

impl std::str::FromStr for ActorType {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "user" => Ok(Self::User),
            "api_key" => Ok(Self::ApiKey),
            other => Err(format!("ActorType inconnu : {other}")),
        }
    }
}

impl Type<MySql> for ActorType {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for ActorType {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for ActorType {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

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
///
/// Story 17-2a (DC5) : `actor_type` + `actor_api_key_id` distinguent une
/// action UI web (`User`) d'une mutation via PAT (`ApiKey`). `user_id` reste
/// **toujours** renseigné (NOT NULL) — même via PAT, il porte le créateur de
/// la clé (imputabilité conservée).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AuditLogEntry {
    pub id: i64,
    pub user_id: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub details_json: Option<serde_json::Value>,
    /// Story 17-2a (DC5) — `User` (UI web / JWT) ou `ApiKey` (mutation via PAT).
    pub actor_type: ActorType,
    /// Story 17-2a (DC5) — id de la clé API si `actor_type = ApiKey`, sinon `None`.
    /// Pas de FK (pointeur logique — l'audit survit 10 ans à la révocation/suppression de la clé).
    pub actor_api_key_id: Option<i64>,
    pub created_at: NaiveDateTime,
}

/// Données de création d'une entrée d'audit.
///
/// **Ne pas construire par struct literal** : utiliser les constructeurs
/// [`NewAuditLogEntry::user`] (sémantique historique, `actor_type=User`) ou
/// [`NewAuditLogEntry::api_key`] (mutation via PAT). Le helper kesh-api
/// `NewAuditLogEntry::from_current_user` (trait `crate::audit::AuditActor`)
/// choisit l'un ou l'autre selon `CurrentUser.api_key_id` (Story 17-2a, DC5).
#[derive(Debug, Clone)]
pub struct NewAuditLogEntry {
    pub user_id: i64,
    pub action: String,
    pub entity_type: String,
    pub entity_id: i64,
    pub details_json: Option<serde_json::Value>,
    /// Story 17-2a (DC5).
    pub actor_type: ActorType,
    /// Story 17-2a (DC5).
    pub actor_api_key_id: Option<i64>,
}

impl NewAuditLogEntry {
    /// Constructeur **historique** : action exécutée par un utilisateur (UI web).
    /// `actor_type = User`, `actor_api_key_id = None`. Sémantique strictement
    /// identique au comportement pré-17-2a (invariant de non-régression).
    pub fn user(
        user_id: i64,
        action: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: i64,
        details_json: Option<serde_json::Value>,
    ) -> Self {
        Self {
            user_id,
            action: action.into(),
            entity_type: entity_type.into(),
            entity_id,
            details_json,
            actor_type: ActorType::User,
            actor_api_key_id: None,
        }
    }

    /// Constructeur « threadé » pour les call-sites *helper* (catégorie (ii),
    /// F-OPUS-1) qui ne disposent pas d'un `&CurrentUser` mais reçoivent
    /// `user_id: i64` + un `actor_api_key_id: Option<i64>` propagé depuis le
    /// handler appelant. `Some(id)` → [`Self::api_key`], `None` → [`Self::user`].
    pub fn for_actor(
        user_id: i64,
        actor_api_key_id: Option<i64>,
        action: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: i64,
        details_json: Option<serde_json::Value>,
    ) -> Self {
        match actor_api_key_id {
            Some(api_key_id) => Self::api_key(
                api_key_id,
                user_id,
                action,
                entity_type,
                entity_id,
                details_json,
            ),
            None => Self::user(user_id, action, entity_type, entity_id, details_json),
        }
    }

    /// Constructeur pour une mutation exécutée **via un PAT** (Story 17-2a, DC5).
    /// `actor_type = ApiKey`, `actor_api_key_id = Some(api_key_id)`,
    /// `user_id = creator_user_id` (le créateur de la clé — imputabilité).
    pub fn api_key(
        api_key_id: i64,
        creator_user_id: i64,
        action: impl Into<String>,
        entity_type: impl Into<String>,
        entity_id: i64,
        details_json: Option<serde_json::Value>,
    ) -> Self {
        Self {
            user_id: creator_user_id,
            action: action.into(),
            entity_type: entity_type.into(),
            entity_id,
            details_json,
            actor_type: ActorType::ApiKey,
            actor_api_key_id: Some(api_key_id),
        }
    }
}
