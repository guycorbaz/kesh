//! Entité `McpKey` : clé du serveur MCP (Story 32-5a, refs #723).
//!
//! Second type de clé de Kesh, **distinct** des clés d'API (`api_key.rs`,
//! préfixe `kesh_pat_`) : une clé MCP (préfixe `kesh_mcp_`, généré par
//! `kesh-api`) n'ouvre que le serveur MCP, jamais `/api/v1`, et une clé d'API
//! n'ouvre jamais le serveur MCP. Elle porte des **portées fines**
//! ([`McpScope`]) et une **politique de validation avant envoi**
//! ([`SendValidation`], Q12 de Guy : `required` par défaut).
//!
//! **Sécurité** (motif `api_key.rs`, DC1) : ni [`McpKey`] ni [`NewMcpKey`] ne
//! dérivent `Serialize`/`Deserialize`, et leur `Debug` est écrit à la main pour
//! masquer `key_hash`. Seul le condensat SHA-256 hex du jeton est persisté ; le
//! secret n'existe qu'en mémoire, à la création.
//!
//! **Portées en base** : la colonne `scopes` est un tableau JSON trié, sans
//! doublon, non vide (`CHECK` de la migration `20261025000001`). Le décodage
//! est **strict** : une chaîne inconnue est une erreur, jamais ignorée — une
//! portée qu'on ignorerait en silence serait un droit qu'on croit retiré.

use std::collections::BTreeSet;

use chrono::NaiveDateTime;
use sqlx::{Decode, Encode, MySql, Type, encode::IsNull, error::BoxDynError, mysql::MySqlTypeInfo};

/// Portée d'une clé MCP — ce qu'elle a le droit de faire.
///
/// Aucune portée d'administration ni d'écriture des livres n'existe
/// (C-32-0-7) ; `prompts:write` ne vise que les prompts de la 32-10a.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum McpScope {
    /// `books:read` — plan comptable, écritures, rapports comptables.
    BooksRead,
    /// `invoices:read` — factures, balance âgée des débiteurs.
    InvoicesRead,
    /// `contacts:read` — carnet d'adresses.
    ContactsRead,
    /// `documents:read` — pièces (PDF figés, documents fournisseur).
    DocumentsRead,
    /// `prompts:write` — prompts enregistrés (32-10a).
    PromptsWrite,
}

impl McpScope {
    /// Toutes les portées, dans l'ordre canonique (celui de `Ord`).
    pub const ALL: [McpScope; 5] = [
        McpScope::BooksRead,
        McpScope::InvoicesRead,
        McpScope::ContactsRead,
        McpScope::DocumentsRead,
        McpScope::PromptsWrite,
    ];

    /// Chaîne stockée en base et échangée par l'API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BooksRead => "books:read",
            Self::InvoicesRead => "invoices:read",
            Self::ContactsRead => "contacts:read",
            Self::DocumentsRead => "documents:read",
            Self::PromptsWrite => "prompts:write",
        }
    }
}

impl std::fmt::Display for McpScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Erreur de lecture d'une portée ou d'un ensemble de portées.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum McpScopeError {
    /// Chaîne qui ne nomme aucune portée (comparaison exacte, sensible à la casse).
    #[error("portée MCP inconnue : {0}")]
    Unknown(String),
    /// Ensemble vide : une clé sans portée ne sert à rien et n'est pas admise.
    #[error("une clé MCP porte au moins une portée")]
    Empty,
    /// Valeur stockée qui n'est pas un tableau JSON de chaînes.
    #[error("portées MCP : tableau JSON de chaînes attendu ({0})")]
    InvalidJson(String),
}

impl std::str::FromStr for McpScope {
    type Err = McpScopeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        McpScope::ALL
            .into_iter()
            .find(|p| p.as_str() == s)
            .ok_or_else(|| McpScopeError::Unknown(s.to_string()))
    }
}

/// Ensemble **non vide**, trié et sans doublon de portées.
///
/// L'invariant « non vide » est tenu par les seuls constructeurs
/// ([`McpScopes::new`], [`McpScopes::from_json`]) ; les doublons sont
/// normalisés (un ensemble ne les connaît pas), l'ordre est celui de
/// [`McpScope::ALL`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct McpScopes(BTreeSet<McpScope>);

impl McpScopes {
    /// Construit l'ensemble ; refuse un ensemble vide.
    pub fn new(scopes: impl IntoIterator<Item = McpScope>) -> Result<Self, McpScopeError> {
        let set: BTreeSet<McpScope> = scopes.into_iter().collect();
        if set.is_empty() {
            return Err(McpScopeError::Empty);
        }
        Ok(Self(set))
    }

    /// Lit des chaînes (l'entrée d'une route) ; une chaîne inconnue est une
    /// erreur, jamais ignorée.
    pub fn parse<'a>(scopes: impl IntoIterator<Item = &'a str>) -> Result<Self, McpScopeError> {
        let parsed = scopes
            .into_iter()
            .map(str::parse)
            .collect::<Result<Vec<McpScope>, _>>()?;
        Self::new(parsed)
    }

    /// Décode la colonne `scopes` (tableau JSON de chaînes). **Strict** : JSON
    /// invalide, autre chose qu'un tableau de chaînes, chaîne inconnue ou
    /// tableau vide → erreur.
    pub fn from_json(json: &str) -> Result<Self, McpScopeError> {
        let raw: Vec<String> =
            serde_json::from_str(json).map_err(|e| McpScopeError::InvalidJson(e.to_string()))?;
        Self::parse(raw.iter().map(String::as_str))
    }

    /// Encode en tableau JSON trié (forme canonique stockée).
    pub fn to_json(&self) -> String {
        let names: Vec<&str> = self.0.iter().map(McpScope::as_str).collect();
        serde_json::to_string(&names).expect("un Vec<&str> se sérialise toujours")
    }

    /// Vrai si l'ensemble contient la portée.
    pub fn contains(&self, scope: McpScope) -> bool {
        self.0.contains(&scope)
    }

    /// Les portées, dans l'ordre canonique.
    pub fn iter(&self) -> impl Iterator<Item = McpScope> + '_ {
        self.0.iter().copied()
    }

    /// L'ensemble sous-jacent.
    pub fn as_set(&self) -> &BTreeSet<McpScope> {
        &self.0
    }
}

impl Type<MySql> for McpScopes {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for McpScopes {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <String as Encode<MySql>>::encode_by_ref(&self.to_json(), buf)
    }
}

/// ⚠️ La colonne `JSON` se lit par `CAST(scopes AS CHAR)` : MariaDB la rend
/// sinon en `BLOB` (motif `bank_profiles.column_mapping`).
impl<'r> Decode<'r, MySql> for McpScopes {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        McpScopes::from_json(&s).map_err(Into::into)
    }
}

/// Politique de validation avant envoi d'une clé (Q12 de Guy) : `required`
/// par défaut ; `exempt` pour une clé que l'administrateur désigne. La 32-5
/// porte le réglage ; son application relève des 32-6 et 32-8.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SendValidation {
    /// `required` — tout envoi passe par une validation humaine.
    #[default]
    Required,
    /// `exempt` — la clé est dispensée de validation (chaque envoi journalisé).
    Exempt,
}

impl SendValidation {
    /// Chaîne stockée en base (`CHECK send_validation IN ('required', 'exempt')`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Required => "required",
            Self::Exempt => "exempt",
        }
    }
}

impl std::str::FromStr for SendValidation {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "required" => Ok(Self::Required),
            "exempt" => Ok(Self::Exempt),
            other => Err(format!("SendValidation inconnue : {other}")),
        }
    }
}

impl Type<MySql> for SendValidation {
    fn type_info() -> MySqlTypeInfo {
        <String as Type<MySql>>::type_info()
    }
    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <String as Type<MySql>>::compatible(ty) || <str as Type<MySql>>::compatible(ty)
    }
}

impl<'q> Encode<'q, MySql> for SendValidation {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode_by_ref(&self.as_str(), buf)
    }
}

impl<'r> Decode<'r, MySql> for SendValidation {
    fn decode(value: <MySql as sqlx::Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <String as Decode<MySql>>::decode(value)?;
        s.parse().map_err(Into::into)
    }
}

/// Clé MCP persistée. Pas de `Serialize`/`Deserialize` ; `Debug` manuel qui
/// masque `key_hash` (cf. note du module).
#[derive(Clone, sqlx::FromRow)]
pub struct McpKey {
    pub id: i64,
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub name: String,
    pub key_hash: String,
    pub scopes: McpScopes,
    pub send_validation: SendValidation,
    pub expires_at: Option<NaiveDateTime>,
    pub last_used_at: Option<NaiveDateTime>,
    pub revoked_at: Option<NaiveDateTime>,
    pub version: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl std::fmt::Debug for McpKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpKey")
            .field("id", &self.id)
            .field("company_id", &self.company_id)
            .field("created_by_user_id", &self.created_by_user_id)
            .field("name", &self.name)
            .field("key_hash", &"***")
            .field("scopes", &self.scopes)
            .field("send_validation", &self.send_validation)
            .field("expires_at", &self.expires_at)
            .field("last_used_at", &self.last_used_at)
            .field("revoked_at", &self.revoked_at)
            .field("version", &self.version)
            .field("created_at", &self.created_at)
            .field("updated_at", &self.updated_at)
            .finish()
    }
}

/// Données de création d'une clé MCP. `key_hash` arrive **déjà haché**
/// (SHA-256 hex, `kesh-api`) : ce crate ne hache jamais. `Debug` manuel.
#[derive(Clone)]
pub struct NewMcpKey {
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub name: String,
    pub key_hash: String,
    pub scopes: McpScopes,
    pub send_validation: SendValidation,
    pub expires_at: Option<NaiveDateTime>,
}

impl std::fmt::Debug for NewMcpKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewMcpKey")
            .field("company_id", &self.company_id)
            .field("created_by_user_id", &self.created_by_user_id)
            .field("name", &self.name)
            .field("key_hash", &"***")
            .field("scopes", &self.scopes)
            .field("send_validation", &self.send_validation)
            .field("expires_at", &self.expires_at)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn every_scope_roundtrips_through_its_string() {
        let attendu = [
            "books:read",
            "invoices:read",
            "contacts:read",
            "documents:read",
            "prompts:write",
        ];
        assert_eq!(McpScope::ALL.len(), attendu.len());
        for (scope, s) in McpScope::ALL.into_iter().zip(attendu) {
            assert_eq!(scope.as_str(), s);
            assert_eq!(McpScope::from_str(s).unwrap(), scope);
        }
    }

    #[test]
    fn unknown_scope_string_is_refused() {
        for s in [
            "books:write",
            "admin",
            "read",
            "BOOKS:READ",
            "",
            " books:read",
        ] {
            assert_eq!(
                McpScope::from_str(s),
                Err(McpScopeError::Unknown(s.to_string())),
                "{s:?} doit être refusée"
            );
        }
        // Au décodage de la colonne : une chaîne inconnue parmi des connues
        // fait échouer TOUT l'ensemble — jamais ignorée (mutation Ma3).
        assert_eq!(
            McpScopes::from_json(r#"["books:read","books:write"]"#),
            Err(McpScopeError::Unknown("books:write".into()))
        );
    }

    #[test]
    fn empty_set_is_refused() {
        assert_eq!(McpScopes::new([]), Err(McpScopeError::Empty));
        assert_eq!(McpScopes::from_json("[]"), Err(McpScopeError::Empty));
        assert_eq!(McpScopes::parse([]), Err(McpScopeError::Empty));
    }

    #[test]
    fn non_array_json_is_refused() {
        for j in ["{}", r#""books:read""#, "[1]", "nope", "null"] {
            assert!(
                matches!(McpScopes::from_json(j), Err(McpScopeError::InvalidJson(_))),
                "{j:?} doit être refusé"
            );
        }
    }

    #[test]
    fn duplicates_are_normalised_and_order_is_canonical() {
        let s = McpScopes::from_json(r#"["prompts:write","books:read","prompts:write"]"#).unwrap();
        assert_eq!(s.to_json(), r#"["books:read","prompts:write"]"#);
        assert_eq!(
            s.iter().collect::<Vec<_>>(),
            vec![McpScope::BooksRead, McpScope::PromptsWrite]
        );
        let tout = McpScopes::new(McpScope::ALL.into_iter().rev()).unwrap();
        assert_eq!(tout.iter().collect::<Vec<_>>(), McpScope::ALL.to_vec());
        assert!(tout.contains(McpScope::DocumentsRead));
        assert_eq!(McpScopes::from_json(&tout.to_json()).unwrap(), tout);
    }

    #[test]
    fn send_validation_roundtrip_and_default() {
        assert_eq!(SendValidation::default(), SendValidation::Required);
        for v in [SendValidation::Required, SendValidation::Exempt] {
            assert_eq!(SendValidation::from_str(v.as_str()).unwrap(), v);
        }
        assert!(SendValidation::from_str("maybe").is_err());
        assert!(SendValidation::from_str("Required").is_err());
    }

    #[test]
    fn debug_masks_key_hash() {
        let key = McpKey {
            id: 1,
            company_id: 2,
            created_by_user_id: 3,
            name: "agent".into(),
            key_hash: "deadbeef".repeat(8),
            scopes: McpScopes::new([McpScope::BooksRead]).unwrap(),
            send_validation: SendValidation::Required,
            expires_at: None,
            last_used_at: None,
            revoked_at: None,
            version: 1,
            created_at: NaiveDateTime::default(),
            updated_at: NaiveDateTime::default(),
        };
        let dbg = format!("{key:?}");
        assert!(dbg.contains("***"));
        assert!(
            !dbg.contains("deadbeef"),
            "le condensat ne paraît jamais en Debug"
        );
        let new = NewMcpKey {
            company_id: 2,
            created_by_user_id: 3,
            name: "agent".into(),
            key_hash: "cafebabe".repeat(8),
            scopes: McpScopes::new([McpScope::BooksRead]).unwrap(),
            send_validation: SendValidation::Exempt,
            expires_at: None,
        };
        let dbg = format!("{new:?}");
        assert!(!dbg.contains("cafebabe"));
    }
}
