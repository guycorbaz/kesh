//! `kesh_ecritures` et `kesh_ecriture` — la liste paginée des écritures et le
//! détail d'une écriture (Story 32-9a, AC-a10). Portée `books:read`.
//!
//! Les lignes ne rendent que l'identifiant de leur compte : le plan comptable
//! donne numéro et nom. Ne sont pas rendus : l'origine du lettrage (interne ;
//! le numéro de groupe l'est), les motifs de blocage (écran, C-32-9-22), le nom
//! du fichier de la pièce source (portée `documents:read`, 32-9b).

use chrono::NaiveDate;
use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{SensDuTri, decalage, limite};
use crate::mcp::sortie::{Date, Montant, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::ecritures::{self, DemandeListeEcritures};

/// Taille de page par défaut (§ *Les tailles mesurées*).
pub const LIMITE_DEFAUT: i64 = 12;
/// Taille de page maximale de l'outil.
pub const LIMITE_MAX_MCP: i64 = 22;

/// La liste.
pub struct KeshEcritures;
/// Le détail.
pub struct KeshEcriture;

/// Un journal, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
pub enum JournalArg {
    /// Achats.
    Achats,
    /// Ventes.
    Ventes,
    /// Banque.
    Banque,
    /// Caisse.
    Caisse,
    /// Opérations diverses.
    OD,
}

impl JournalArg {
    fn du_service(self) -> kesh_core::accounting::Journal {
        use kesh_core::accounting::Journal as J;
        match self {
            Self::Achats => J::Achats,
            Self::Ventes => J::Ventes,
            Self::Banque => J::Banque,
            Self::Caisse => J::Caisse,
            Self::OD => J::OD,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Achats => "Achats",
            Self::Ventes => "Ventes",
            Self::Banque => "Banque",
            Self::Caisse => "Caisse",
            Self::OD => "OD",
        }
    }
}

/// La colonne de tri, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TriEcritures {
    EntryDate,
    EntryNumber,
    Journal,
    Description,
}

impl TriEcritures {
    fn du_service(self) -> kesh_core::listing::SortBy {
        use kesh_core::listing::SortBy as S;
        match self {
            Self::EntryDate => S::EntryDate,
            Self::EntryNumber => S::EntryNumber,
            Self::Journal => S::Journal,
            Self::Description => S::Description,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::EntryDate => "entryDate",
            Self::EntryNumber => "entryNumber",
            Self::Journal => "journal",
            Self::Description => "description",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsListe {
    description: Option<String>,
    account_id: Option<i64>,
    amount_min: Option<String>,
    amount_max: Option<String>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    journal: Option<JournalArg>,
    sort_by: Option<TriEcritures>,
    sort_dir: Option<SensDuTri>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsDetail {
    id: i64,
}

sortie! {
    /// Une ligne d'écriture.
    struct Ligne {
        id: i64,
        compte_id: i64,
        ordre: i32,
        debit: Montant,
        credit: Montant,
        projet_id: Option<i64>,
        lettrage: Option<i64>,
    }
}

sortie! {
    /// Une écriture et ses lignes.
    struct Ecriture {
        id: i64,
        numero: i64,
        exercice_id: i64,
        date: Date,
        journal: kesh_db::entities::Journal,
        libelle: TexteNonFiable,
        contre_passe: Option<i64>,
        lignes: Vec<Ligne>,
    }
}

sortie! {
    /// La liste.
    struct Ecritures { ecritures: Vec<Ecriture>, total: i64, limit: i64, offset: i64 }
}

sortie! {
    /// La pièce source : la facture fournisseur dont l'écriture est l'achat
    /// ou le règlement.
    struct PieceSource {
        facture_fournisseur_id: i64,
        numero_facture_fournisseur: Option<TexteNonFiable>,
    }
}

sortie! {
    /// Le détail.
    struct FicheEcriture {
        ecriture: Ecriture,
        contre_passee_par: Option<i64>,
        piece_source: Option<PieceSource>,
    }
}

fn ecriture(e: kesh_db::entities::JournalEntryWithLines) -> Ecriture {
    let kesh_db::entities::JournalEntryWithLines { entry, lines } = e;
    Ecriture {
        id: entry.id,
        numero: entry.entry_number,
        exercice_id: entry.fiscal_year_id,
        date: Date(entry.entry_date),
        journal: entry.journal,
        libelle: TexteNonFiable::de(entry.description),
        contre_passe: entry.reverses_entry_id,
        lignes: lines
            .into_iter()
            .map(|l| Ligne {
                id: l.id,
                compte_id: l.account_id,
                ordre: l.line_order,
                debit: Montant(l.debit),
                credit: Montant(l.credit),
                projet_id: l.project_id,
                lettrage: l.lettering_key,
            })
            .collect(),
    }
}

impl McpTool for KeshEcritures {
    fn nom(&self) -> &'static str {
        "kesh_ecritures"
    }

    fn description(&self) -> &'static str {
        "Liste les écritures comptables et leurs lignes, filtrées par libellé, compte \
         (accountId), montants, dates, journal ; triées et paginées (limit, offset). Les \
         lignes donnent l'identifiant du compte (kesh_plan_comptable donne numéro et nom). Si \
         la réponse est trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"description":{"type":"string"},"accountId":{"type":"integer","minimum":1},"amountMin":{"type":"string"},"amountMax":{"type":"string"},"dateFrom":{"type":"string","format":"date"},"dateTo":{"type":"string","format":"date"},"journal":{"type":"string","enum":["Achats","Ventes","Banque","Caisse","OD"]},"sortBy":{"type":"string","enum":["entryDate","entryNumber","journal","description"]},"sortDir":{"type":"string","enum":["asc","desc"]},"limit":{"type":"integer","minimum":1,"maximum":22,"default":12},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsListe = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            let resume = Resume::outil("kesh_ecritures")
                .recherche_fournie("description", a.description.is_some())
                .entier("accountId", a.account_id)
                .recherche_fournie("amountMin", a.amount_min.is_some())
                .recherche_fournie("amountMax", a.amount_max.is_some())
                .date("dateFrom", a.date_from)
                .date("dateTo", a.date_to)
                .code("journal", a.journal.map(JournalArg::code))
                .code("sortBy", a.sort_by.map(TriEcritures::code))
                .code("sortDir", a.sort_dir.map(SensDuTri::code))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeListeEcritures {
                description: a.description,
                account_id: a.account_id,
                amount_min: a.amount_min,
                amount_max: a.amount_max,
                date_from: a.date_from.map(|d| d.to_string()),
                date_to: a.date_to.map(|d| d.to_string()),
                journal: a.journal.map(JournalArg::du_service),
                sort_by: a.sort_by.map_or(
                    kesh_core::listing::SortBy::EntryDate,
                    TriEcritures::du_service,
                ),
                sort_dir: a.sort_dir.map_or(
                    kesh_core::listing::SortDirection::Desc,
                    SensDuTri::du_service,
                ),
                limit: Some(limit),
                offset: Some(offset),
            };
            let page = ecritures::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = Ecritures {
                ecritures: page.items.into_iter().map(ecriture).collect(),
                total: page.total,
                limit: page.limit,
                offset: page.offset,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshEcriture {
    fn nom(&self) -> &'static str {
        "kesh_ecriture"
    }

    fn description(&self) -> &'static str {
        "Lit une écriture comptable par son identifiant : en-tête, lignes, écriture qui la \
         contre-passe, et la facture fournisseur dont elle est l'achat ou le règlement."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"id":{"type":"integer","minimum":1}},"required":["id"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::BooksRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let ArgumentsDetail { id } = arguments(valeur)?;
            let d = ecritures::detail(ctx.acces(), ctx.lecture(), id)
                .await
                .map_err(depuis_app_error)?;
            let ecritures::DetailEcriture {
                ecriture: e,
                contre_passee_par,
                blocage_contre_passation: _,
                blocage_modification: _,
                piece_source,
            } = d;
            let sortie = FicheEcriture {
                ecriture: ecriture(e),
                contre_passee_par,
                piece_source: piece_source.map(|p| {
                    let ecritures::EntrySourceDocument {
                        supplier_invoice_id,
                        supplier_invoice_number,
                        original_filename: _,
                    } = p;
                    PieceSource {
                        facture_fournisseur_id: supplier_invoice_id,
                        numero_facture_fournisseur: supplier_invoice_number.map(TexteNonFiable::de),
                    }
                }),
            };
            donnees(
                Resume::outil("kesh_ecriture").entier("id", Some(id)),
                &sortie,
            )
        })
    }
}
