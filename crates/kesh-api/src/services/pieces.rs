//! Le service des pièces importées — la liste filtrée par statut (Story 32-9b,
//! refs #723, AC-b1, C-32-9-13), lue par
//! `GET /api/v1/imported-supplier-invoices?status=`, puis par l'outil MCP
//! `kesh_pieces`.
//!
//! Une lecture **sans trace** : elle rend directement (règle R5). **L'ordre
//! d'avant la 32-9b est conservé** (règle R6) : l'exigence, puis la société
//! (`Internal` si elle manque, ce que faisait `get_company_for`), puis la
//! validation du statut (requis et dans [`STATUTS`], même message), puis la
//! lecture `imported_supplier_invoices::list_by_status`, qui ne fait qu'un
//! `SELECT` (établi par lecture).
//!
//! **Deux formes rendues, et c'est le typage qui tient « aucun outil n'ouvre
//! un fichier »** (validation P1, F5) : [`lister_tout`] rend les
//! `ImportedSupplierInvoice` entières que l'API sérialise depuis la 12-5c —
//! pour l'API seulement, jamais nommée dans `src/mcp/` ; [`lister`] rend une
//! [`PagePieces`] de [`PiecePublique`], qui ne porte **ni** l'empreinte ni le
//! chemin du fichier archivé, **ni** aucune donnée lue du QR-code de la pièce
//! — ce contenu viendra en 0.17.0 (spike, E3) ; seul [`PiecePublique::qr_lu`]
//! en dit la présence.

use chrono::NaiveDateTime;
use kesh_db::entities::Role;
use kesh_db::entities::imported_supplier_invoice::ImportedSupplierInvoice;
use kesh_db::repositories::imported_supplier_invoices;

use crate::errors::AppError;
use crate::services::acces::{ContexteAcces, Exigence, Portee};
use crate::services::poignees::Lecture;

/// Taille de page par défaut de [`lister`] (§ *Les tailles mesurées* de la
/// fiche 32-9).
pub const LIMITE_DEFAUT: i64 = 35;
/// Plafond de la taille de page de [`lister`] (écrêtage, non refus).
pub const LIMITE_MAX: i64 = 65;

/// Le domaine du statut d'une pièce — la contrainte `chk_imported_si_status`
/// de la base (dette D1 de la 12-5c, déplacée de la route).
pub const STATUTS: [&str; 3] = ["to_complete", "completed", "discarded"];

/// Exigence de [`lister`] et de [`lister_tout`] : `Comptable` (le RBAC de
/// `comptable_routes`, où la route est montée) ; portée MCP `DocumentsRead`.
pub const EXIGENCE_LISTER: Exigence = Exigence {
    role_api: Role::Comptable,
    portee_mcp: Some(Portee::DocumentsRead),
};

/// La demande de [`lister`].
#[derive(Debug, Clone, Default)]
pub struct DemandeListePieces {
    /// Le statut — **requis**, dans [`STATUTS`].
    pub statut: Option<String>,
    /// Taille de page ; absente → [`LIMITE_DEFAUT`], ramenée entre 1 et
    /// [`LIMITE_MAX`].
    pub limit: Option<i64>,
    /// Décalage ; absent → 0, ramené à 0 s'il est négatif.
    pub offset: Option<i64>,
}

/// Une pièce **sans** son fichier ni ce qui a été lu de son QR-code : ce que
/// reçoit l'outil MCP `kesh_pieces`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PiecePublique {
    /// L'identifiant de la pièce.
    pub id: i64,
    /// `to_complete`, `completed` ou `discarded`.
    pub status: String,
    /// La facture fournisseur créée de la pièce, si elle l'a été.
    pub supplier_invoice_id: Option<i64>,
    /// Le nom d'origine du fichier (affichage seul).
    pub original_filename: String,
    /// Le type MIME du fichier.
    pub mime_type: String,
    /// La taille du fichier, en octets.
    pub byte_size: i64,
    /// L'import de la pièce.
    pub created_at: NaiveDateTime,
    /// La pièce porte-t-elle des données lues d'un QR-code ? (le contenu n'est
    /// pas rendu).
    pub qr_lu: bool,
}

impl From<ImportedSupplierInvoice> for PiecePublique {
    /// ⚠️ La déstructuration est **exhaustive**, sans `..` : un champ ajouté à
    /// l'entité ne compile pas ici tant qu'on n'a pas décidé s'il atteint
    /// l'outil (motif de `factures::detail_sans_fichier`, C-32-9-49).
    fn from(p: ImportedSupplierInvoice) -> Self {
        let ImportedSupplierInvoice {
            id,
            company_id: _,
            status,
            supplier_invoice_id,
            file_hash: _,
            storage_path: _,
            original_filename,
            mime_type,
            byte_size,
            creditor_iban,
            is_qr_iban: _,
            creditor_address_type: _,
            creditor_name: _,
            creditor_line1: _,
            creditor_line2: _,
            creditor_postal_code: _,
            creditor_town: _,
            creditor_country: _,
            reference_type: _,
            reference_value: _,
            amount: _,
            currency: _,
            unstructured_message: _,
            billing_information: _,
            version: _,
            created_at,
            updated_at: _,
        } = p;
        Self {
            id,
            status,
            supplier_invoice_id,
            original_filename,
            mime_type,
            byte_size,
            created_at,
            qr_lu: creditor_iban.is_some(),
        }
    }
}

/// Une page de pièces, les plus récentes d'abord (`created_at DESC, id DESC`,
/// l'ordre du dépôt).
#[derive(Debug)]
pub struct PagePieces {
    /// La page.
    pub items: Vec<PiecePublique>,
    /// Le nombre de pièces de ce statut dans la société.
    pub total: i64,
    /// La taille de page **écrêtée**.
    pub limit: i64,
    /// Le décalage **écrêté**.
    pub offset: i64,
}

/// Toutes les pièces de statut `statut` de la société du contexte, entières —
/// **pour l'API seulement** (`list_imported`).
///
/// L'ordre d'avant la 32-9b : l'exigence, la société (`Internal` si elle
/// manque), la validation du statut (`AppError::Validation`, message
/// inchangé), puis la lecture.
pub async fn lister_tout(
    acces: &ContexteAcces,
    lecture: &Lecture,
    statut: Option<String>,
) -> Result<Vec<ImportedSupplierInvoice>, AppError> {
    acces.exiger(EXIGENCE_LISTER)?;
    let company =
        crate::helpers::company_for_ids(lecture.pool(), acces.company_id(), acces.user_id())
            .await?;
    let statut = statut.unwrap_or_default();
    if !STATUTS.contains(&statut.as_str()) {
        return Err(AppError::Validation(format!(
            "Paramètre 'status' requis et ∈ {{to_complete, completed, discarded}} (reçu : '{statut}')"
        )));
    }
    Ok(imported_supplier_invoices::list_by_status(lecture.pool(), company.id, &statut).await?)
}

/// La page de pièces de statut `demande.statut` de la société du contexte,
/// **sans fichier ni donnée du QR** ([`PiecePublique`]) — pour l'outil MCP.
///
/// [`lister_tout`] (même exigence, même ordre), puis la tranche : `limit`
/// écrêté à `1..=`[`LIMITE_MAX`], `offset` ramené à 0 s'il est négatif.
pub async fn lister(
    acces: &ContexteAcces,
    lecture: &Lecture,
    demande: DemandeListePieces,
) -> Result<PagePieces, AppError> {
    let toutes = lister_tout(acces, lecture, demande.statut).await?;
    let limit = demande.limit.unwrap_or(LIMITE_DEFAUT).clamp(1, LIMITE_MAX);
    let offset = demande.offset.unwrap_or(0).max(0);
    let total = i64::try_from(toutes.len()).unwrap_or(i64::MAX);
    let items = toutes
        .into_iter()
        .skip(usize::try_from(offset).unwrap_or(usize::MAX))
        .take(usize::try_from(limit).unwrap_or(0))
        .map(PiecePublique::from)
        .collect();
    Ok(PagePieces {
        items,
        total,
        limit,
        offset,
    })
}
