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
//! **Portées en base** : la colonne `scopes` est un tableau JSON non vide
//! (`CHECK` de la migration `20261025000001`), écrit sans doublon dans l'ordre
//! de [`McpScope::ALL`]. L'**entrée** d'une API se lit strictement
//! ([`McpScopes::parse`], [`McpScopes::from_json`] : une chaîne inconnue est une
//! erreur, jamais ignorée). La **lecture** d'une ligne stockée est tolérante
//! ([`StoredMcpScopes`], C-32-5-37) : une portée que cette version ne connaît
//! pas est mise de côté et journalisée, jamais accordée.

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

    /// Lit une **entrée d'API** (tableau JSON de chaînes). **Strict** : JSON
    /// invalide, autre chose qu'un tableau de chaînes, chaîne inconnue ou
    /// tableau vide → erreur. Une ligne **stockée**, elle, se lit avec
    /// tolérance par [`StoredMcpScopes::from_stored_json`] (C-32-5-37).
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

/// Les portées d'une clé **telles que la base les porte** — lues avec
/// tolérance (C-32-5-37, revue P1 E1).
///
/// ⚠️ **Pourquoi pas [`McpScopes`] à la lecture.** Une ligne peut porter une
/// portée que cette version ne connaît pas — une sauvegarde d'une version
/// future qui n'ajoute aucune table s'importe ici, et l'import ne contrôle pas
/// le contenu des cellules. Un décodage strict rendait alors la **liste entière**
/// illisible (`list_by_company` est un `fetch_all`), la clé fautive impossible à
/// relire donc à révoquer, et l'authentification en erreur de base plutôt qu'en
/// refus. La lecture garde donc les portées **connues** et met les autres de
/// côté ([`StoredMcpScopes::unknown`]) ; le dépôt les journalise en nommant
/// l'id de la clé (`repositories::mcp_keys`). La colonne `JSON` s'y lit par
/// `CAST(scopes AS CHAR)` : MariaDB la rend sinon en `BLOB`.
///
/// Ce qui reste strict : l'**écriture** (on ne stocke qu'un [`McpScopes`]) et
/// la lecture d'une entrée d'API ([`McpScopes::parse`], [`McpScopes::from_json`]).
/// Une clé sans aucune portée connue se liste et se révoque, mais
/// [`StoredMcpScopes::usable`] rend `None` : elle n'authentifie jamais
/// (`repositories::mcp_keys::find_active_auth_by_key_hash`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoredMcpScopes {
    known: BTreeSet<McpScope>,
    unknown: Vec<String>,
}

impl StoredMcpScopes {
    /// Lit la colonne `scopes` sans jamais échouer : chaque chaîne connue est
    /// gardée ; une chaîne inconnue, un élément qui n'est pas une chaîne, ou un
    /// texte qui n'est pas un tableau JSON va dans [`Self::unknown`].
    pub fn from_stored_json(json: &str) -> Self {
        let mut stored = Self::default();
        match serde_json::from_str::<serde_json::Value>(json) {
            Ok(serde_json::Value::Array(items)) => {
                for item in items {
                    match item.as_str() {
                        Some(s) => match s.parse::<McpScope>() {
                            Ok(scope) => {
                                stored.known.insert(scope);
                            }
                            Err(_) => stored.unknown.push(s.to_string()),
                        },
                        None => stored.unknown.push(item.to_string()),
                    }
                }
            }
            _ => stored.unknown.push(json.to_string()),
        }
        stored
    }

    /// Les portées connues, dans l'ordre canonique.
    pub fn known(&self) -> &BTreeSet<McpScope> {
        &self.known
    }

    /// Ce que cette version ne reconnaît pas, tel que stocké.
    pub fn unknown(&self) -> &[String] {
        &self.unknown
    }

    /// Les portées connues, si elles forment un ensemble utilisable (non vide) ;
    /// `None` pour une clé sans aucune portée connue.
    pub fn usable(&self) -> Option<McpScopes> {
        McpScopes::new(self.known.iter().copied()).ok()
    }
}

impl From<McpScopes> for StoredMcpScopes {
    fn from(scopes: McpScopes) -> Self {
        Self {
            known: scopes.0,
            unknown: Vec::new(),
        }
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
/// masque `key_hash` (cf. note du module). Construite par le dépôt, qui lit
/// les portées avec tolérance ([`StoredMcpScopes`]).
#[derive(Clone)]
pub struct McpKey {
    pub id: i64,
    pub company_id: i64,
    pub created_by_user_id: i64,
    pub name: String,
    pub key_hash: String,
    pub scopes: StoredMcpScopes,
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
        // En entrée d'API : une chaîne inconnue parmi des connues fait échouer
        // TOUT l'ensemble (mutation Ma3). La lecture d'une ligne stockée, elle,
        // l'ignore (C-32-5-37, `stored_scopes_*`).
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
    fn stored_scopes_keep_the_known_and_set_aside_the_unknown() {
        // C-32-5-37 : à la lecture, une portée inconnue est mise de côté, jamais
        // une erreur ; les portées connues restent.
        let s =
            StoredMcpScopes::from_stored_json(r#"["books:write","books:read","prompts:write"]"#);
        assert_eq!(
            s.known().iter().copied().collect::<Vec<_>>(),
            vec![McpScope::BooksRead, McpScope::PromptsWrite]
        );
        assert_eq!(s.unknown(), ["books:write"]);
        assert_eq!(
            s.usable().map(|u| u.to_json()).as_deref(),
            Some(r#"["books:read","prompts:write"]"#)
        );
        // Une forme entièrement à jour n'a rien de côté.
        let ok = StoredMcpScopes::from_stored_json(r#"["books:read"]"#);
        assert!(ok.unknown().is_empty());
        assert_eq!(ok, McpScopes::new([McpScope::BooksRead]).unwrap().into());
    }

    #[test]
    fn stored_scopes_without_any_known_scope_are_not_usable() {
        // Revue P2 (R3) : la comparaison reste exacte — une portée connue en
        // majuscules ou entourée d'espaces n'est PAS reconnue.
        for j in [
            r#"["books:write"]"#,
            r#"["BOOKS:READ"]"#,
            r#"[" books:read"]"#,
            "[1, null]",
            "{}",
            "pas du json",
            "[]",
        ] {
            let s = StoredMcpScopes::from_stored_json(j);
            assert!(s.known().is_empty(), "{j}");
            assert_eq!(
                s.usable(),
                None,
                "{j} : une clé sans portée connue n'est pas utilisable"
            );
        }
        assert_eq!(
            StoredMcpScopes::from_stored_json("[1, null]").unknown(),
            ["1", "null"]
        );
        assert_eq!(StoredMcpScopes::from_stored_json("{}").unknown(), ["{}"]);
        assert_eq!(
            StoredMcpScopes::from_stored_json(r#"["BOOKS:READ"," books:read"]"#).unknown(),
            ["BOOKS:READ", " books:read"]
        );
        // L'entrée d'une API, elle, reste stricte (mutation Ma3).
        assert!(McpScopes::from_json(r#"["books:write","books:read"]"#).is_err());
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
            scopes: McpScopes::new([McpScope::BooksRead]).unwrap().into(),
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
