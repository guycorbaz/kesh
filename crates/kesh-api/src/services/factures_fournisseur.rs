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
use kesh_db::entities::supplier_invoice::{SupplierInvoice, SupplierInvoiceLine};
use kesh_db::errors::DbError;
use kesh_db::repositories::supplier_invoices::{
    self, SupplierInvoiceListFilter, SupplierInvoiceSettlementView, SupplierInvoiceSort,
};

/// Réexporté pour l'outil MCP de la 32-9a (C-32-9-25) : `src/mcp/` ne nomme pas
/// `kesh_db::repositories`. `SupplierInvoiceSettlementView`, qui porte le
/// chemin de la pièce, **n'est pas** réexporté.
pub use kesh_db::repositories::supplier_invoices::SupplierInvoiceListItem;

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
    /// Story v017-4a (#849, AC-15) — les statuts retenus, **vide = tous** ;
    /// construit par [`statuts_du_filtre`] (la route), laissé vide par l'outil
    /// MCP, qui n'en change pas.
    pub statuts: Vec<&'static str>,
    /// Story 33-1b (#926) — la recherche (numéro interne, numéro et nom du
    /// fournisseur), construite par [`recherche_du_filtre`] ; `None` par défaut,
    /// laissée ainsi par l'outil MCP.
    pub recherche: Option<String>,
    /// Story 33-1b (#926) — l'ordre, construit par [`tri_du_filtre`] ; la date par
    /// défaut, laissée ainsi par l'outil MCP.
    pub tri: SupplierInvoiceSort,
}

/// Longueur maximale de `?q=`, en caractères.
pub const RECHERCHE_MAX_CARACTERES: usize = 128;

/// Le filtre `?q=` (Story 33-1b, #926) : absent, vide ou blanc → aucune recherche ;
/// au-delà de [`RECHERCHE_MAX_CARACTERES`] → `400 VALIDATION_ERROR`.
pub fn recherche_du_filtre(param: Option<&str>) -> Result<Option<String>, AppError> {
    let Some(valeur) = param.map(str::trim).filter(|q| !q.is_empty()) else {
        return Ok(None);
    };
    if valeur.chars().count() > RECHERCHE_MAX_CARACTERES {
        return Err(AppError::Validation(format!(
            "q : {RECHERCHE_MAX_CARACTERES} caractères au plus"
        )));
    }
    Ok(Some(valeur.to_string()))
}

/// Les filtres `?sort=` et `?order=` (Story 33-1b, #926) : une liste fermée, jamais
/// d'interpolation. `sort` absent : l'ordre d'aujourd'hui ; `order` absent : croissant.
/// Une valeur hors liste → `400 VALIDATION_ERROR`.
pub fn tri_du_filtre(
    sort: Option<&str>,
    order: Option<&str>,
) -> Result<SupplierInvoiceSort, AppError> {
    let descendant = match order {
        None | Some("asc") => false,
        Some("desc") => true,
        Some(autre) => {
            return Err(AppError::Validation(format!(
                "order : valeur inconnue '{autre}' (attendu : asc, desc)"
            )));
        }
    };
    match sort {
        None => Ok(SupplierInvoiceSort::DateDesc),
        Some("internalNumber") if descendant => Ok(SupplierInvoiceSort::InternalNumberDesc),
        Some("internalNumber") => Ok(SupplierInvoiceSort::InternalNumberAsc),
        Some(autre) => Err(AppError::Validation(format!(
            "sort : valeur inconnue '{autre}' (attendu : internalNumber)"
        ))),
    }
}

/// Le filtre `?status=` de `GET /api/v1/supplier-invoices` (Story v017-4a,
/// AC-15) : absent → tous (vide) ; sinon une ou plusieurs valeurs de
/// [`supplier_invoices::SUPPLIER_INVOICE_STATUSES`], séparées par des
/// virgules (blancs autour ignorés, doublons fondus). Une valeur inconnue, ou
/// un filtre vide, → `400 VALIDATION_ERROR`.
pub fn statuts_du_filtre(param: Option<&str>) -> Result<Vec<&'static str>, AppError> {
    let Some(param) = param else {
        return Ok(Vec::new());
    };
    let mut statuts: Vec<&'static str> = Vec::new();
    for brut in param.split(',') {
        let valeur = brut.trim();
        let connu = supplier_invoices::SUPPLIER_INVOICE_STATUSES
            .iter()
            .find(|s| **s == valeur)
            .ok_or_else(|| {
                AppError::Validation(format!(
                    "status : valeur inconnue '{valeur}' (attendu : draft, open, paid, cancelled)"
                ))
            })?;
        if !statuts.contains(connu) {
            statuts.push(connu);
        }
    }
    Ok(statuts)
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
    let (items, total) = supplier_invoices::list_filtered(
        lecture.pool(),
        company.id,
        SupplierInvoiceListFilter {
            statuses: &demande.statuts,
            q: demande.recherche.as_deref(),
            sort: demande.tri,
        },
        limit,
        offset,
    )
    .await?;
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

/// La fiche d'une facture fournisseur **sans aucun chemin de fichier** (Story
/// 32-9a, C-32-9-13, AC-a10) : la facture, ses lignes, et de sa pièce le seul
/// identifiant.
#[derive(Debug)]
pub struct FicheFactureFournisseur {
    /// La facture.
    pub facture: SupplierInvoice,
    /// Ses lignes.
    pub lignes: Vec<SupplierInvoiceLine>,
    /// L'identifiant de la pièce importée rattachée, s'il y en a une.
    pub piece_id: Option<i64>,
}

/// [`detail`] — même exigence, même lecture —, projeté en
/// [`FicheFactureFournisseur`] : l'outil MCP ne reçoit ni `storage_path` ni
/// aucun autre champ de la pièce.
pub async fn detail_sans_fichier(
    acces: &ContexteAcces,
    lecture: &Lecture,
    id: i64,
) -> Result<FicheFactureFournisseur, AppError> {
    let vue = detail(acces, lecture, id).await?;
    Ok(FicheFactureFournisseur {
        facture: vue.invoice,
        lignes: vue.lines,
        piece_id: vue.source_document.map(|d| d.id),
    })
}
