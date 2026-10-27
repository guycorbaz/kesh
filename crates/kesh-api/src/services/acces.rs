//! Le contrôle d'accès des services : qui lit, par quel canal, et ce qu'une
//! opération exige.
//!
//! Un service commence par `acces.exiger(EXIGENCE)?` (règle R2 de
//! `services`) : le refus précède la validation de la demande et toute lecture.
//! Pour le canal API, l'exigence **réplique** le RBAC de la route — le
//! middleware refuse toujours le premier ; le service est la défense en
//! profondeur, et le seul contrôle pour le canal MCP.

use std::collections::BTreeSet;

use kesh_db::entities::Role;

use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;

/// Portée d'une clé MCP — ce qu'elle a le droit de lire.
///
/// **Une seule énumération** (Story 32-5b, C-32-5-4) : celle que la base
/// persiste, `kesh_db::entities::McpScope`, réexportée sous le nom que lui
/// donnent les services. Ce que chaque portée ouvre aux services :
///
/// - `BooksRead` — plan comptable, écritures, postes ouverts, rapports
///   comptables (32-3, 32-4a) ; et les prompts servis par `prompts/list` et
///   `prompts/get` (`prompts_mcp`, 32-10a-ii, C-32-10a-16) ;
/// - `InvoicesRead` — factures clientes et fournisseur, échéancier,
///   règlements, balance âgée des débiteurs (32-3, 32-4b, C-32-4b-3) ;
/// - `ContactsRead` — carnet d'adresses, données personnelles (32-4a,
///   C-32-4a-4) ;
/// - `DocumentsRead`, `PromptsWrite` — aucun service encore (les pièces et la
///   proposition d'un prompt viennent avec les stories suivantes de l'Epic 32).
pub use kesh_db::entities::McpScope as Portee;

/// Canal par lequel arrive une lecture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Canal {
    /// L'API HTTP, authentifiée par JWT ou clé d'API : le rôle de l'utilisateur.
    Api {
        /// Rôle de l'utilisateur (ou du créateur de la clé).
        role: Role,
    },
    /// Le serveur MCP (32-6) : les portées de la clé.
    Mcp {
        /// Portées accordées à la clé MCP.
        portees: BTreeSet<Portee>,
    },
}

/// Ce qu'une opération exige, pour chacun des deux canaux.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exigence {
    /// Rôle minimal exigé sur le canal API (comparaison `Role::level`, la même
    /// que `middleware/rbac.rs`).
    pub role_api: Role,
    /// Portée exigée sur le canal MCP ; `None` : l'opération n'est jamais
    /// servie au MCP.
    pub portee_mcp: Option<Portee>,
}

/// Le contexte d'accès d'une lecture : société, utilisateur, canal.
///
/// Champs privés : un contexte ne se construit que par [`ContexteAcces::depuis_api`]
/// (l'API) ou [`ContexteAcces::mcp`] — en production par
/// `services::cles_mcp::authentifier` (Story 32-5b), qu'appelle la garde de
/// `/mcp` (Story 32-6a), et par les tests (garde
/// `contexte_acces_mcp_construit_par_authentifier_seul`).
#[derive(Debug, Clone)]
pub struct ContexteAcces {
    company_id: i64,
    user_id: i64,
    canal: Canal,
}

impl ContexteAcces {
    /// Contexte d'une requête de l'API : la société, l'utilisateur et le rôle
    /// que le middleware d'authentification a établis.
    pub fn depuis_api(u: &CurrentUser) -> Self {
        Self {
            company_id: u.company_id,
            user_id: u.user_id,
            canal: Canal::Api { role: u.role },
        }
    }

    /// Contexte d'une requête du serveur MCP, avec les portées de sa clé.
    pub fn mcp(company_id: i64, user_id: i64, portees: BTreeSet<Portee>) -> Self {
        Self {
            company_id,
            user_id,
            canal: Canal::Mcp { portees },
        }
    }

    /// Société dont on lit les livres.
    pub fn company_id(&self) -> i64 {
        self.company_id
    }

    /// Utilisateur au nom duquel on lit — celui de la trace d'audit.
    pub fn user_id(&self) -> i64 {
        self.user_id
    }

    /// Canal de la requête.
    pub fn canal(&self) -> &Canal {
        &self.canal
    }

    /// Vérifie l'exigence d'une opération ; `Err(AppError::Forbidden)` sinon.
    ///
    /// - canal API : refus si le rôle est inférieur à `e.role_api` (même
    ///   comparaison que `middleware/rbac.rs`) ;
    /// - canal MCP : refus si l'opération n'est pas servie au MCP
    ///   (`e.portee_mcp` à `None`) ou si sa portée manque à la clé.
    pub fn exiger(&self, e: Exigence) -> Result<(), AppError> {
        let permis = match &self.canal {
            Canal::Api { role } => *role >= e.role_api,
            Canal::Mcp { portees } => e.portee_mcp.is_some_and(|p| portees.contains(&p)),
        };
        if permis {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}
