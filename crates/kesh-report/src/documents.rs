//! Les justificatifs d'une écriture (Story 55-3a, #1607) — ce qui la justifie :
//! le PDF d'une facture client, celui d'un avoir, le document d'une facture
//! fournisseur.
//!
//! Kesh n'a **aucune table de pièces jointes** : un justificatif se rattache à
//! une écriture par **cinq** colonnes, d'où les cinq branches d'une seule
//! union, définie **une fois** ici ([`union_des_rattachements`]) et lue par deux
//! fonctions :
//!
//! - [`entries_with_documents`] — l'ensemble des écritures d'une période qui ont
//!   au moins un justificatif : **une** requête par rapport (journal, grand
//!   livre), aucune requête par écriture ni par compte, aucune liste `IN` ;
//! - [`documents_of_entry`] — la liste des justificatifs d'une écriture, lue au
//!   clic par `GET /api/v1/journal-entries/{id}/documents`.
//!
//! Le même constructeur garantit que le booléen `hasDocuments` du rapport est
//! vrai **si et seulement si** la liste de l'écriture est non vide.
//!
//! # Les règles de rattachement
//!
//! | Branche | Colonne | Règle |
//! |---|---|---|
//! | facture de vente | `invoices.journal_entry_id` | validée, ou annulée **et** figée |
//! | facture réglée | `invoice_settlements.journal_entry_id` → `invoice_id` | même règle que la facture |
//! | avoir | `credit_notes.journal_entry_id` | non brouillon |
//! | document fournisseur (achat) | `supplier_invoices.purchase_journal_entry_id` ← `imported_supplier_invoices` | document `completed` |
//! | document fournisseur (règlement) | `supplier_invoices.settlement_journal_entry_id` ← idem | idem ; tous les documents d'un règlement groupé |
//!
//! Une facture **annulée jamais figée** n'a pas de PDF : pas de justificatif.
//! Une facture **validée non figée** en est un : l'écran la signale « PDF pas
//! encore émis », sans la lire (D12 : afficher n'est pas figer) — d'où
//! [`PdfStatus`], lu dans la même requête (`pdf_storage_path IS NOT NULL`, la
//! règle d'`is_frozen` de `kesh-api`).
//!
//! Le fournisseur suit deux branches plutôt qu'un `OR` : chacune garde l'index
//! de sa colonne.
//!
//! ⚠️ **Filtre de société sur chaque table de chaque branche** (défense en
//! profondeur, KF-002) : les clés étrangères ne visent que l'écriture, si bien
//! que le schéma admet une ligne d'une société qui pointe l'écriture d'une
//! autre.
//!
//! ⚠️ **Écart assumé avec le détail d'une écriture** : un règlement fournisseur
//! groupé rend ici **tous** ses documents ; le détail
//! (`find_source_document_of_entry`, `LIMIT 1`) n'en montre qu'un.

use std::collections::{BTreeMap, HashSet};

use chrono::NaiveDate;
use serde::Serialize;
use sqlx::MySqlPool;

use crate::errors::ReportError;

/// Le genre d'un justificatif ; l'ordre de déclaration est l'ordre
/// d'affichage (facture, avoir, document fournisseur).
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "camelCase")]
pub enum EntryDocumentKind {
    /// PDF d'une facture client (`GET /api/v1/invoices/{id}/pdf`).
    Invoice,
    /// PDF d'un avoir (`GET /api/v1/credit-notes/{id}/pdf`).
    CreditNote,
    /// Document importé d'une facture fournisseur
    /// (`GET /api/v1/supplier-invoices/{id}/source-document`).
    SupplierDocument,
}

impl EntryDocumentKind {
    /// Le code de la colonne `kind` de l'union (voir [`union_des_rattachements`]).
    fn depuis_code(code: i64) -> Result<Self, ReportError> {
        match code {
            0 => Ok(Self::Invoice),
            1 => Ok(Self::CreditNote),
            2 => Ok(Self::SupplierDocument),
            autre => Err(ReportError::CorruptData(format!(
                "genre de justificatif inconnu : {autre}"
            ))),
        }
    }
}

/// L'état du PDF d'une facture, à la lecture de la liste.
///
/// Énumération **ouverte** : l'epic « PDF marqués » (aperçu) et le PDF produit
/// hors de Kesh (`external`, issue #1609) pourront y ajouter une valeur ; un
/// client ne lit le PDF d'une facture que pour `frozen`.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PdfStatus {
    /// PDF émis, figé : `GET /invoices/{id}/pdf` le sert sans rien écrire.
    Frozen,
    /// Pas de PDF figé : `GET /invoices/{id}/pdf` le **figerait** ; l'écran
    /// affiche « PDF pas encore émis », sans rien lire.
    NotFrozen,
}

/// Un justificatif d'une écriture.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EntryDocument {
    /// `invoice` | `creditNote` | `supplierDocument`.
    pub kind: EntryDocumentKind,
    /// Identifiant pour la route de lecture : facture, avoir, facture
    /// fournisseur.
    pub id: i64,
    /// Numéro : facture, avoir ; numéro interne de la facture fournisseur, à
    /// défaut le sien.
    pub number: Option<String>,
    /// Nom du fichier importé (document fournisseur seulement).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_name: Option<String>,
    /// État du PDF d'une facture (facture seulement) : `frozen` ou `notFrozen`.
    /// Il vaut **à la lecture de la liste**.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pdf_status: Option<PdfStatus>,
}

/// Le filtre d'écritures d'une union : ce qui varie entre les deux lectures.
#[derive(Debug, Clone, Copy)]
enum FiltreEcritures {
    /// Les écritures datées dans `[from, to]` (index
    /// `idx_journal_entries_company_date`).
    Periode(NaiveDate, NaiveDate),
    /// Une seule écriture.
    Ecriture(i64),
}

impl FiltreEcritures {
    /// Le prédicat SQL sur `je`, ses paramètres liés par [`lier`].
    fn predicat(self) -> &'static str {
        match self {
            Self::Periode(..) => "je.entry_date BETWEEN ? AND ?",
            Self::Ecriture(_) => "je.id = ?",
        }
    }
}

/// Une ligne de l'union.
#[derive(Debug, sqlx::FromRow)]
struct LigneUnion {
    entry_id: i64,
    kind: i64,
    doc_id: i64,
    number: Option<String>,
    file_name: Option<String>,
    /// `1` figée, `0` non figée ; `NULL` hors facture.
    frozen: Option<i64>,
}

/// Règle d'une facture justificative : validée, ou annulée **et** figée (une
/// facture annulée jamais figée n'a pas de PDF).
const REGLE_FACTURE: &str =
    "(i.status = 'validated' OR (i.status = 'cancelled' AND i.pdf_storage_path IS NOT NULL))";

/// Les cinq branches de l'union, chacune jointe à `journal_entries`, bornée par
/// `je.company_id = ?` et par `filtre`, avec le filtre de société sur **chaque**
/// table. Colonnes : `entry_id`, `kind` (0 facture, 1 avoir, 2 document
/// fournisseur), `doc_id`, `number`, `file_name`, `frozen`.
///
/// Paramètres, par branche et dans l'ordre : la société, puis ceux du filtre
/// ([`lier`]).
fn union_des_rattachements(filtre: FiltreEcritures) -> String {
    let p = filtre.predicat();
    let colonnes_facture = "je.id AS entry_id, CAST(0 AS SIGNED) AS kind, i.id AS doc_id, \
         i.invoice_number AS number, CAST(NULL AS CHAR(255)) AS file_name, \
         CAST(i.pdf_storage_path IS NOT NULL AS SIGNED) AS frozen";
    let colonnes_fournisseur = "je.id AS entry_id, CAST(2 AS SIGNED) AS kind, si.id AS doc_id, \
         COALESCE(si.internal_number, si.supplier_invoice_number) AS number, \
         isi.original_filename AS file_name, CAST(NULL AS SIGNED) AS frozen";
    let fournisseur = |colonne: &str| {
        format!(
            "SELECT {colonnes_fournisseur} \
             FROM supplier_invoices si \
             JOIN journal_entries je ON je.id = si.{colonne} \
             JOIN imported_supplier_invoices isi ON isi.supplier_invoice_id = si.id \
             WHERE je.company_id = ? AND {p} \
               AND si.company_id = je.company_id \
               AND isi.company_id = je.company_id \
               AND isi.status = 'completed'"
        )
    };
    [
        // 1. La facture de vente.
        format!(
            "SELECT {colonnes_facture} \
             FROM invoices i \
             JOIN journal_entries je ON je.id = i.journal_entry_id \
             WHERE je.company_id = ? AND {p} \
               AND i.company_id = je.company_id \
               AND {REGLE_FACTURE}"
        ),
        // 2. La facture réglée (règlement simple ou groupé, tout type).
        format!(
            "SELECT {colonnes_facture} \
             FROM invoice_settlements s \
             JOIN journal_entries je ON je.id = s.journal_entry_id \
             JOIN invoices i ON i.id = s.invoice_id \
             WHERE je.company_id = ? AND {p} \
               AND s.company_id = je.company_id \
               AND i.company_id = je.company_id \
               AND {REGLE_FACTURE}"
        ),
        // 3. L'avoir (un brouillon n'est pas une pièce comptable).
        format!(
            "SELECT je.id AS entry_id, CAST(1 AS SIGNED) AS kind, cn.id AS doc_id, \
                    cn.credit_note_number AS number, CAST(NULL AS CHAR(255)) AS file_name, \
                    CAST(NULL AS SIGNED) AS frozen \
             FROM credit_notes cn \
             JOIN journal_entries je ON je.id = cn.journal_entry_id \
             WHERE je.company_id = ? AND {p} \
               AND cn.company_id = je.company_id \
               AND cn.status <> 'draft'"
        ),
        // 4. et 5. Le document fournisseur, par l'achat puis par le règlement.
        fournisseur("purchase_journal_entry_id"),
        fournisseur("settlement_journal_entry_id"),
    ]
    .join(" UNION ALL ")
}

/// Le nombre de branches de [`union_des_rattachements`] : les paramètres se
/// lient une fois par branche.
const BRANCHES: usize = 5;

/// Lie les paramètres de l'union : pour chaque branche, la société puis le
/// filtre.
fn lier<'q, O>(
    mut requete: sqlx::query::QueryAs<'q, sqlx::MySql, O, sqlx::mysql::MySqlArguments>,
    company_id: i64,
    filtre: FiltreEcritures,
) -> sqlx::query::QueryAs<'q, sqlx::MySql, O, sqlx::mysql::MySqlArguments> {
    for _ in 0..BRANCHES {
        requete = requete.bind(company_id);
        requete = match filtre {
            FiltreEcritures::Periode(from, to) => requete.bind(from).bind(to),
            FiltreEcritures::Ecriture(id) => requete.bind(id),
        };
    }
    requete
}

/// Les écritures de la société datées dans `[from, to]` qui ont **au moins un**
/// justificatif — **une** requête, quelle que soit la taille de la période.
///
/// Appelée une fois par `journal_report::generate` et une fois par
/// `general_ledger::generate`, hors de toute boucle ; l'ensemble est lu ensuite
/// (`contains`).
pub async fn entries_with_documents(
    pool: &MySqlPool,
    company_id: i64,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<HashSet<i64>, ReportError> {
    let filtre = FiltreEcritures::Periode(from, to);
    let sql = format!(
        "SELECT DISTINCT d.entry_id FROM ({}) AS d",
        union_des_rattachements(filtre)
    );
    let ids: Vec<(i64,)> = lier(sqlx::query_as(&sql), company_id, filtre)
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;
    Ok(ids.into_iter().map(|(id,)| id).collect())
}

/// Les justificatifs de l'écriture `entry_id` de la société, **dédoublonnés**
/// par `(genre, identifiant)` et **triés** (genre, puis identifiant) ; vide si
/// elle n'en a aucun, ou si elle n'est pas de la société.
///
/// Ne lit que la base : aucun PDF n'est lu ni figé. `pdf_status` vaut à la
/// lecture.
pub async fn documents_of_entry(
    pool: &MySqlPool,
    company_id: i64,
    entry_id: i64,
) -> Result<Vec<EntryDocument>, ReportError> {
    let filtre = FiltreEcritures::Ecriture(entry_id);
    let sql = union_des_rattachements(filtre);
    let lignes: Vec<LigneUnion> = lier(sqlx::query_as(&sql), company_id, filtre)
        .fetch_all(pool)
        .await
        .map_err(kesh_db::errors::map_db_error)?;

    let mut uniques: BTreeMap<(EntryDocumentKind, i64), EntryDocument> = BTreeMap::new();
    for l in lignes {
        debug_assert_eq!(l.entry_id, entry_id);
        let kind = EntryDocumentKind::depuis_code(l.kind)?;
        let pdf_status = match (kind, l.frozen) {
            (EntryDocumentKind::Invoice, Some(f)) if f != 0 => Some(PdfStatus::Frozen),
            (EntryDocumentKind::Invoice, _) => Some(PdfStatus::NotFrozen),
            _ => None,
        };
        uniques
            .entry((kind, l.doc_id))
            .or_insert_with(|| EntryDocument {
                kind,
                id: l.doc_id,
                number: l.number,
                file_name: l.file_name,
                pdf_status,
            });
    }
    Ok(uniques.into_values().collect())
}

#[cfg(test)]
mod tests {
    //! Gardes de construction de l'union ; les appels réels vivent dans
    //! `tests/documents.rs` (AC-5 : aucun appel d'`entries_with_documents` dans
    //! `src/` hors des deux rapports).
    use super::*;

    /// Chaque branche lie la société puis son filtre : autant de `?` que
    /// `BRANCHES` × (1 + paramètres du filtre).
    #[test]
    fn l_union_a_autant_de_parametres_que_lier_en_pose() {
        let jour = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let periode = union_des_rattachements(FiltreEcritures::Periode(jour, jour));
        assert_eq!(periode.matches('?').count(), BRANCHES * 3);
        let ecriture = union_des_rattachements(FiltreEcritures::Ecriture(1));
        assert_eq!(ecriture.matches('?').count(), BRANCHES * 2);
        assert_eq!(ecriture.matches(" UNION ALL ").count(), BRANCHES - 1);
    }

    /// Le filtre de société de chaque table de chaque branche (KF-002).
    #[test]
    fn chaque_table_porte_le_filtre_de_societe() {
        let sql = union_des_rattachements(FiltreEcritures::Ecriture(1));
        for filtre in [
            "i.company_id = je.company_id",
            "s.company_id = je.company_id",
            "cn.company_id = je.company_id",
            "si.company_id = je.company_id",
            "isi.company_id = je.company_id",
        ] {
            assert!(sql.contains(filtre), "{filtre} absent");
        }
        assert_eq!(sql.matches("WHERE je.company_id = ?").count(), BRANCHES);
    }

    #[test]
    fn genres_dans_l_ordre_d_affichage() {
        assert!(EntryDocumentKind::Invoice < EntryDocumentKind::CreditNote);
        assert!(EntryDocumentKind::CreditNote < EntryDocumentKind::SupplierDocument);
        assert!(EntryDocumentKind::depuis_code(3).is_err());
    }
}
