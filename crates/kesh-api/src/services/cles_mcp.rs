//! L'authentification d'une clé du serveur MCP — Story 32-5b (refs #723).
//!
//! Une clé MCP (`kesh_mcp_…`, `auth::api_key::generate_mcp_key`) est un second
//! type de clé, **distinct** des clés d'API : elle n'ouvre jamais `/api/v1`, et
//! une clé d'API n'ouvre jamais le MCP. [`authentifier`] est son seul chemin
//! de validation ; la garde de l'endpoint `/mcp` (32-6) l'appellera. Rien ne
//! l'appelle encore en production.
//!
//! # L'ordre (C-32-5-14)
//!
//! 1. **Préfixe exact** (`MCP_PREFIX`, comparaison d'octets), **avant tout
//!    accès à la base** : un jeton qui ne le porte pas — dont un PAT valide et
//!    actif — est refusé sans requête.
//! 2. Condensat `SHA-256(jeton)` et recherche de la clé **active** (non
//!    révoquée, non expirée, jointe à son créateur et à sa société,
//!    `mcp_keys::find_active_auth_by_key_hash`) sur la **`Lecture`** : une clé
//!    orpheline, ou sans aucune portée connue de cette version (C-32-5-37), n'y
//!    est jamais trouvée.
//! 3. Le créateur doit être **actif** et de rôle au moins [`ROLE_CLE_MCP`],
//!    relus à chaque requête (C-32-5-7, hypothèse Q3).
//! 4. Dernière utilisation *best-effort* sur la **`Trace`** : un échec est
//!    journalisé (id de la clé, jamais le condensat) et n'échoue pas
//!    l'authentification.
//!
//! Tout refus rend `AppError::Unauthenticated` — un 401 dont la réponse est
//! **la même** quel que soit le motif ; seul le journal du serveur le
//! distingue. Une erreur de la base n'est **jamais** un 401 : elle se propage
//! (`AppError::Database`), comme pour les clés d'API.
//!
//! La **politique de validation** de la clé ([`CleMcpAuthentifiee::validation`],
//! Q12) est rendue, non appliquée : le répartiteur de la 32-6 et la poignée de
//! la 32-8 la lisent.

use kesh_db::entities::{Role, SendValidation};
use kesh_db::repositories::mcp_keys;

use crate::auth::api_key::{MCP_PREFIX, sha256_hex};
use crate::errors::AppError;
use crate::services::acces::ContexteAcces;
use crate::services::poignees::{Lecture, Trace};

/// Rôle minimal du **créateur** d'une clé MCP pour qu'elle authentifie
/// (hypothèse Q3, C-32-5-7) : un administrateur rétrogradé coupe ses clés sur
/// le champ. Ne règle que l'authentification : la **création** est gardée par
/// `require_admin_role`, en dur, au bloc des routes d'administration (32-5c).
pub const ROLE_CLE_MCP: Role = Role::Admin;

/// Une clé MCP authentifiée : son identifiant, le contexte d'accès de ses
/// lectures (canal MCP, société et créateur de la clé, ses portées) et sa
/// politique de validation avant envoi.
///
/// Ne porte ni le jeton ni son condensat.
#[derive(Debug, Clone)]
pub struct CleMcpAuthentifiee {
    /// `mcp_keys.id`.
    pub cle_id: i64,
    /// Le contexte d'accès que reçoivent les services : `Canal::Mcp`, portées
    /// de la clé.
    pub acces: ContexteAcces,
    /// La politique de validation avant envoi de la clé (Q12), à appliquer par
    /// la 32-6 et la 32-8.
    pub validation: SendValidation,
}

/// Refus uniforme : le détail ne va qu'au journal du serveur.
fn refus(detail: &str) -> AppError {
    AppError::Unauthenticated(format!("mcp: {detail}"))
}

/// Authentifie un jeton de clé MCP (cf. l'ordre, en tête du module).
///
/// `Err(AppError::Unauthenticated)` pour : préfixe absent ou autre (dont un
/// PAT), clé inconnue, révoquée, expirée, orpheline ou sans portée connue,
/// créateur inactif, créateur de rôle inférieur à [`ROLE_CLE_MCP`].
/// `Err(AppError::Database)` si la recherche échoue.
pub async fn authentifier(
    lecture: &Lecture,
    trace: &Trace,
    jeton: &str,
) -> Result<CleMcpAuthentifiee, AppError> {
    if !jeton.as_bytes().starts_with(MCP_PREFIX.as_bytes()) {
        return Err(refus("missing or foreign prefix"));
    }

    let key_hash = sha256_hex(jeton);
    let row = mcp_keys::find_active_auth_by_key_hash(lecture.pool(), &key_hash)
        .await?
        .ok_or_else(|| refus("unknown, revoked, expired, orphan or scopeless key"))?;

    if !row.creator_active {
        return Err(refus("creator user is inactive"));
    }
    if row.creator_role < ROLE_CLE_MCP {
        return Err(refus("creator user is no longer an administrator"));
    }

    if let Err(e) = mcp_keys::touch_last_used(trace.pool(), &key_hash).await {
        tracing::warn!(
            mcp_key_id = row.mcp_key_id,
            "clé MCP : dernière utilisation non écrite (sans effet sur l'authentification) : {e}"
        );
    }

    Ok(CleMcpAuthentifiee {
        cle_id: row.mcp_key_id,
        acces: ContexteAcces::mcp(
            row.company_id,
            row.created_by_user_id,
            row.scopes.as_set().clone(),
        ),
        validation: row.send_validation,
    })
}
