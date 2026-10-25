//! Le service des factures fournisseur — la liste paginée et la fiche
//! (Story 32-4b), lues par `GET /api/v1/supplier-invoices` et
//! `GET /api/v1/supplier-invoices/{id}`, puis par l'outil MCP de la 32-9a.
//!
//! Deux lectures **sans trace** : elles rendent directement (règle R5). La liste
//! **écrête** sa taille de page (défaut [`LIMITE_DEFAUT`], plafond
//! [`LIMITE_MAX`]) là où les listes clientes refusent (C-32-4b-6) : c'est le
//! comportement de l'écran, conservé. Le dépôt (`supplier_invoices::list`) lie
//! `limit` et `offset` tels quels : la borne de ce service est la **seule**.
//! La mise en forme de la fiche reste au handler
//! (`SupplierInvoiceResponse::from_view`, C-32-4b-5).

use kesh_db::entities::Role;
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices::{
    self, SupplierInvoiceListItem, SupplierInvoiceSettlementView,
};

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`].
pub const LIMITE_DEFAUT: i64 = 50;
/// Plafond de la taille de page de [`lister`] (écrêtage, non refus).
pub const LIMITE_MAX: i64 = 200;

/// Exigence de [`lister`] : tous rôles (`GET /api/v1/supplier-invoices`,
/// `authenticated_routes`) ; portée MCP `InvoicesRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// Exigence de [`detail`] : tous rôles (`GET /api/v1/supplier-invoices/{id}`,
/// `authenticated_routes`) ; portée MCP `InvoicesRead`.
pub const EXIGENCE_DETAIL: Exigence = Exigence {
    role_api: Role::Consultation,
    portee_mcp: Some(Portee::InvoicesRead),
};

/// La demande de [`lister`] : les paramètres de `GET /api/v1/supplier-invoices`.
#[derive(Debug, Clone, Default)]
pub struct DemandeListeFacturesFournisseur {
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], ramenée entre 1 et
    /// [`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, ramené à 0 s'il est négatif.
    pub offset: Option<i64>,
}

/// Une page de factures fournisseur, les plus récentes d'abord
/// (`invoice_date DESC, id DESC`, l'ordre du dépôt).
#[derive(Debug)]
pub struct PageFacturesFournisseur {
    /// La page.
    pub items: Vec<SupplierInvoiceListItem>,
    /// Le nombre total de factures fournisseur de la société.
    pub total: i64,
    /// La taille de page **écrêtée**, celle que la route rend.
    pub limit: i64,
    /// Le décalage **écrêté**, celui que la route rend.
    pub offset: i64,
}

/// La page de factures fournisseur de la société du contexte.
///
/// La société est lue **d'abord** (`Internal` si elle manque), puis les bornes
/// sont posées (écrêtage), puis la page est lue. Gardes :
/// `ac324b_b_les_bornes_des_listes_de_factures` et
/// `ac324b_d_f_la_liste_fournisseur_par_la_route` (`tests/services_rollout.rs`).
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeListeFacturesFournisseur,
) -> Result<PageFacturesFournisseur, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    let limit = demande.limit.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);
    let offset = demande.offset.unwrap_or(0).max(0);
    let (items, total) = supplier_invoices::list(lecture.pool(), company.id, limit, offset).await?;
    Ok(PageFacturesFournisseur {
        items,
        total,
        limit,
        offset,
    })
}

/// La fiche de la facture fournisseur `id` de la société du contexte, champs
/// d'annulation, dernier lot confirmé et document compris, **depuis un seul
/// instantané** (`supplier_invoices::get_settlement_view`, Story 25-3-a-2).
///
/// La société est lue **d'abord** (`Internal` si elle manque), puis la vue
/// (`AppError::Database(DbError::NotFound)` si la facture est absente ou d'une
/// autre société).
pub async fn detail(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<SupplierInvoiceSettlementView, AppError> {
    acces.exiger(EXIGENCE_DETAIL)?;
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    supplier_invoices::get_settlement_view(lecture.pool(), company.id, id)
        .await?
        .ok_or(AppError::Database(DbError::NotFound))
}
