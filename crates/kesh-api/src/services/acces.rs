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
/// La 32-3 a posé les deux portées dont ses pilotes ont besoin, la 32-4a celle
/// du carnet d'adresses ; la 32-5 ajoute les autres, leurs chaînes et leur
/// persistance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Portee {
    /// Lecture des livres : plan comptable, écritures, rapports comptables.
    BooksRead,
    /// Lecture des factures, dont la balance âgée des débiteurs.
    InvoicesRead,
    /// Lecture du carnet d'adresses : contacts, données personnelles (Story
    /// 32-4a, C-32-4a-4).
    ContactsRead,
}

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
/// (l'API) ou [`ContexteAcces::mcp`] (les tests et la 32-6).
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
