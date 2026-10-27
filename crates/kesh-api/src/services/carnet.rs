//! Le service du carnet d'adresses — la liste paginée des contacts et la fiche
//! d'un contact (Story 32-4a), lus par `GET /api/v1/contacts` et
//! `GET /api/v1/contacts/{id}`, puis par l'outil MCP de la 32-9b.
//!
//! Deux lectures **sans trace** : elles rendent directement (règle R5). Les
//! bornes de pagination vivent ici (C-32-4a-5) ; le dépôt
//! (`contacts::list_by_company_paginated`) lie `limit` et `offset` tels quels :
//! la borne de ce service est la **seule**. Le service rend la société avec le
//! contact : le libellé des conditions de paiement, qui en dépend, est de la
//! présentation et reste au handler (C-32-4a-8).

use kesh_core::listing::SortDirection;
use kesh_db::entities::Role;
use kesh_db::entities::company::Company;
use kesh_db::entities::contact::{Contact, ContactType};
use kesh_db::errors::DbError;
use kesh_db::repositories::contacts::{self, ContactListQuery, ContactListResult};

/// Réexporté pour l'outil MCP `kesh_contacts` de la 32-9b (C-32-9-25) :
/// `src/mcp/` ne nomme pas `kesh_db::repositories`.
pub use kesh_db::repositories::contacts::ContactSortBy;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`].
pub const LIMITE_DEFAUT: i64 = 20;
/// Plafond de la taille de page de [`lister`].
pub const LIMITE_MAX: i64 = 100;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/contacts`,
/// `authenticated_routes`) ; portée MCP `ContactsRead` (données personnelles).
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::ContactsRead),
};

/// Exigence de [`detail`] : tous rôles (`GET /api/v1/contacts/{id}`,
/// `authenticated_routes`) ; portée MCP `ContactsRead`.
pub const EXIGENCE_DETAIL: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::ContactsRead),
};

/// La demande de [`lister`] : les paramètres de `GET /api/v1/contacts`.
#[derive(Debug, Clone, Default)]
pub struct DemandeListeContacts {
    /// Recherche plein texte.
    pub search: Option<String>,
    /// Personne ou entreprise.
    pub contact_type: Option<ContactType>,
    /// Ne garder que les clients (ou les non-clients).
    pub is_client: Option<bool>,
    /// Ne garder que les fournisseurs (ou les non-fournisseurs).
    pub is_supplier: Option<bool>,
    /// Inclure les contacts archivés.
    pub include_archived: bool,
    /// Colonne de tri ; absente → celle par défaut du dépôt.
    pub sort_by: Option<ContactSortBy>,
    /// Sens du tri ; absent → ascendant.
    pub sort_direction: Option<SortDirection>,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], ramenée entre 1 et
    /// [`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, ramené à 0 s'il est négatif.
    pub offset: Option<i64>,
}

/// Une page de contacts, avec la société (pour la présentation).
#[derive(Debug)]
pub struct PageContacts {
    /// La page.
    pub page: ContactListResult,
    /// La société du contexte.
    pub societe: Company,
}

/// Un contact, avec la société (pour la présentation).
#[derive(Debug)]
pub struct FicheContact {
    /// Le contact.
    pub contact: Contact,
    /// La société du contexte.
    pub societe: Company,
}

/// La page de contacts de la société du contexte.
///
/// La société est lue **d'abord** (`Internal` si elle manque), puis les bornes
/// et les défauts du tri sont posés, puis la page est lue. Garde des défauts
/// du tri et de la recopie des filtres : `tri_par_defaut_et_filtres_du_carnet`
/// (`tests/services_rollout.rs`).
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeListeContacts,
) -> Result<PageContacts, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    // Société : contrôle défensif (fenêtre d'obsolescence du company_id) ET
    // langue d'instance pour le libellé des conditions de paiement (#245).
    let societe =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;

    let limit = demande.limit.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);
    let offset = demande.offset.unwrap_or(0).max(0);

    let query = ContactListQuery {
        search: demande.search,
        contact_type: demande.contact_type,
        is_client: demande.is_client,
        is_supplier: demande.is_supplier,
        include_archived: demande.include_archived,
        sort_by: demande.sort_by.unwrap_or_default(),
        sort_direction: demande.sort_direction.unwrap_or(SortDirection::Asc),
        limit,
        offset,
    };
    let page =
        contacts::list_by_company_paginated(lecture.pool(), acces.company_id(), query).await?;
    Ok(PageContacts { page, societe })
}

/// La fiche du contact `id` de la société du contexte.
///
/// Le contact est lu **avant** la société (ordre d'avant la 32-4a,
/// C-32-4a-6) : absent ou d'une autre société →
/// `AppError::Database(DbError::NotFound)` (404, sans fuite d'existence) ;
/// puis la société (`Internal` si elle manque).
pub async fn detail(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<FicheContact, AppError> {
    acces.exiger(EXIGENCE_DETAIL)?;
    let contact = contacts::find_by_id_in_company(lecture.pool(), id, acces.company_id())
        .await?
        .ok_or(AppError::Database(DbError::NotFound))?;
    // #245 : la société, pour la langue d'instance (fallback du libellé).
    let societe =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    Ok(FicheContact { contact, societe })
}
