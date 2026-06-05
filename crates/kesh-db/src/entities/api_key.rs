//! Entité `ApiKey` : clé d'accès API externe (Personal Access Token).
//!
//! Story 17-2a (#100) — permet à une IA externe ou un logiciel tiers de
//! consommer les routes `/api/v1/*` via un header `Authorization: Bearer
//! kesh_pat_…`, sans partager les identifiants utilisateur.
//!
//! **Sécurité (DC1)** : `ApiKey` ne dérive PAS `Serialize`/`Deserialize`
//! (défense en profondeur — une clé ne doit jamais fuiter en JSON), et
//! `Debug` est implémenté manuellement pour masquer `key_hash`. Le secret
//! en clair n'est JAMAIS stocké : seul `SHA-256(token)` hex (`key_hash`)
//! est persisté. Le secret clair n'existe qu'en mémoire au moment de la
//! création (retourné une seule fois par le repo / la route).

use chrono::NaiveDateTime;
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Portée d'une clé API (DC3).
///
/// Stocké en DB en kebab-case : `"read"`, `"read-write"`.
/// - `Read` → seules les méthodes GET/HEAD/OPTIONS sont autorisées.
/// - `ReadWrite` → toutes les méthodes (sous réserve du RBAC rôle existant).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApiKeyScope {
    Read,
    ReadWrite,
}

impl ApiKeyScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Read => "read",
            Self::ReadWrite => "read-write",
        }
    }
}

impl std::str::FromStr for ApiKeyScope {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "read" => Ok(Self::Read),
            "read-write" => Ok(Self::ReadWrite),
            other => Err(format!("ApiKeyScope inconnu : {other}")),
        }
    }
}

impl Type<MySql> for ApiKeyScope {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for ApiKeyScope {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for ApiKeyScope {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Clé API persistée en base.
///
/// `key_hash` contient le `SHA-256(token)` hex (64 chars), jamais le secret
/// en clair. Pas de `Serialize`/`Deserialize` (cf. note module). `Debug`
/// manuel masquant `key_hash`.
#[derive(Clone, sqlx::FromRow)]
pub struct ApiKey {
    pub id: i64,
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub name: String,
    pub key_hash: String,
    pub scope: ApiKeyScope,
    pub expires_at: Option<NaiveDateTime>,
    pub last_used_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl std::fmt::Debug for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ApiKey")
            .field("id", &self.id)
            .field("company_id", &self.company_id)
            .field("created_by_user_id", &self.created_by_user_id)
            .field("name", &self.name)
            .field("key_hash", &"***")
            .field("scope", &self.scope)
            .field("expires_at", &self.expires_at)
            .field("last_used_at", &self.last_used_at)
            .field("revoked_at", &self.revoked_at)
            .field("version", &self.version)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// Données de création d'une clé API.
///
/// `key_hash` doit être fourni **déjà haché** (SHA-256 hex) par l'appelant
/// (`kesh-api::auth::api_key::generate_pat`). Ce crate ne hache jamais
/// lui-même. `Debug` manuel masquant `key_hash`.
#[derive(Clone)]
pub struct NewApiKey {
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub name: String,
    pub key_hash: String,
    pub scope: ApiKeyScope,
    pub expires_at: Option<NaiveDateTime>,
}

impl std::fmt::Debug for NewApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewApiKey")
            .field("company_id", &self.company_id)
            .field("created_by_user_id", &self.created_by_user_id)
            .field("name", &self.name)
            .field("key_hash", &"***")
            .field("scope", &self.scope)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn scope_roundtrip_str() {
        assert_eq!(ApiKeyScope::Read.as_str(), "read");
        assert_eq!(ApiKeyScope::ReadWrite.as_str(), "read-write");
        assert_eq!(ApiKeyScope::from_str("read").unwrap(), ApiKeyScope::Read);
        assert_eq!(
            ApiKeyScope::from_str("read-write").unwrap(),
            ApiKeyScope::ReadWrite
        );
        assert!(ApiKeyScope::from_str("write").is_err());
        assert!(ApiKeyScope::from_str("READ").is_err());
    }

    #[test]
    fn debug_masks_key_hash() {
        let key = ApiKey {
            id: 1,
            company_id: 2,
            created_by_user_id: 3,
            name: "ci".to_string(),
            key_hash: "deadbeef".repeat(8),
            scope: ApiKeyScope::Read,
            expires_at: None,
            last_used_at: None,
            revoked_at: None,
            version: 1,
            created_at: chrono::NaiveDateTime::default(),
            updated_at: chrono::NaiveDateTime::default(),
        };
        let dbg = format!("{key:?}");
        assert!(dbg.contains("***"), "key_hash doit être masqué");
        assert!(
            !dbg.contains("deadbeef"),
            "le hash ne doit jamais apparaître en Debug"
        );
    }
}
