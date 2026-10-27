//! `kesh_factures`, `kesh_facture`, `kesh_echeancier` — les factures clientes
//! (Story 32-9a, AC-a10). Portée `invoices:read`.
//!
//! Le nom du client sort, enveloppé, non masqué (C-32-9-9) ; l'adresse d'envoi
//! d'une facture est une coordonnée de tiers ([`Tiers`]), masquée sans
//! `contacts:read`. Ne sont pas rendus : le PDF figé (chemin, empreinte,
//! langue, instant), l'aperçu d'arrondi d'un brouillon, les règlements
//! (C-32-9-11). Le PDF figé n'atteint même pas l'outil :
//! `factures::detail_sans_fichier` le retire par le typage (C-32-9-49).

use chrono::NaiveDate;
use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{STATUTS_FACTURE, SensDuTri, StatutFacture, decalage, limite};
use crate::mcp::sortie::{
    Code, Date, Horodatage, Montant, Resume, TexteNonFiable, Tiers, donnees, sortie,
};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::factures::{
    self, DemandeEcheances, DemandeListeFactures, InvoiceListItem, InvoiceSortBy, PausedFilter,
    PaymentStatusFilter,
};

/// Taille de page par défaut (celle du service).
pub const LIMITE_DEFAUT: i64 = 20;
/// Taille de page maximale de l'outil (le service admet 100).
pub const LIMITE_MAX_MCP: i64 = 40;

/// La liste.
pub struct KeshFactures;
/// La fiche.
pub struct KeshFacture;
/// L'échéancier.
pub struct KeshEcheancier;

/// Le statut, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum StatutArg {
    Draft,
    Validated,
    Cancelled,
}

impl StatutArg {
    fn code(self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Validated => "validated",
            Self::Cancelled => "cancelled",
        }
    }
}

/// La suspension des rappels, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum SuspensionArg {
    All,
    Paused,
    NotPaused,
}

impl SuspensionArg {
    fn du_service(self) -> PausedFilter {
        match self {
            Self::All => PausedFilter::All,
            Self::Paused => PausedFilter::Paused,
            Self::NotPaused => PausedFilter::NotPaused,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Paused => "paused",
            Self::NotPaused => "notPaused",
        }
    }
}

/// Le statut de paiement, en argument (échéancier ; défaut : impayées).
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PaiementArg {
    All,
    Paid,
    Unpaid,
    Overdue,
}

impl PaiementArg {
    fn du_service(self) -> PaymentStatusFilter {
        match self {
            Self::All => PaymentStatusFilter::All,
            Self::Paid => PaymentStatusFilter::Paid,
            Self::Unpaid => PaymentStatusFilter::Unpaid,
            Self::Overdue => PaymentStatusFilter::Overdue,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Paid => "paid",
            Self::Unpaid => "unpaid",
            Self::Overdue => "overdue",
        }
    }
}

/// La colonne de tri, en argument.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "camelCase")]
enum TriFactures {
    Date,
    DueDate,
    TotalAmount,
    ContactName,
    CreatedAt,
}

impl TriFactures {
    fn du_service(self) -> InvoiceSortBy {
        match self {
            Self::Date => InvoiceSortBy::Date,
            Self::DueDate => InvoiceSortBy::DueDate,
            Self::TotalAmount => InvoiceSortBy::TotalAmount,
            Self::ContactName => InvoiceSortBy::ContactName,
            Self::CreatedAt => InvoiceSortBy::CreatedAt,
        }
    }

    fn code(self) -> &'static str {
        match self {
            Self::Date => "date",
            Self::DueDate => "dueDate",
            Self::TotalAmount => "totalAmount",
            Self::ContactName => "contactName",
            Self::CreatedAt => "createdAt",
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsListe {
    search: Option<String>,
    status: Option<StatutArg>,
    contact_id: Option<i64>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    paused: Option<SuspensionArg>,
    sort_by: Option<TriFactures>,
    sort_direction: Option<SensDuTri>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct ArgumentsEcheancier {
    search: Option<String>,
    contact_id: Option<i64>,
    date_from: Option<NaiveDate>,
    date_to: Option<NaiveDate>,
    due_before: Option<NaiveDate>,
    payment_status: Option<PaiementArg>,
    sort_by: Option<TriFactures>,
    sort_direction: Option<SensDuTri>,
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsDetail {
    id: i64,
}

sortie! {
    /// Une facture de liste.
    struct FactureListe {
        id: i64,
        client_id: i64,
        client: TexteNonFiable,
        numero: Option<TexteNonFiable>,
        statut: Code<StatutFacture>,
        date: Date,
        echeance: Option<Date>,
        conditions: Option<TexteNonFiable>,
        montant_ht: Montant,
        total_ttc: Montant,
        montant_regle: Montant,
        reste_du: Montant,
        payee_le: Option<Horodatage>,
        suspendue_le: Option<Horodatage>,
        note_suspension: Option<TexteNonFiable>,
    }
}

sortie! {
    /// La liste.
    struct Factures { factures: Vec<FactureListe>, total: i64, limit: i64, offset: i64 }
}

sortie! {
    /// Le résumé de l'échéancier (impayées et échues, sur les mêmes filtres).
    struct ResumeEcheances {
        impayees_nombre: i64,
        impayees_total: Montant,
        echues_nombre: i64,
        echues_total: Montant,
    }
}

sortie! {
    /// L'échéancier.
    struct Echeancier {
        factures: Vec<FactureListe>,
        total: i64,
        limit: i64,
        offset: i64,
        resume: ResumeEcheances,
    }
}

sortie! {
    /// Une ligne de facture.
    struct LigneFacture {
        id: i64,
        position: i32,
        description: TexteNonFiable,
        quantite: Montant,
        prix_unitaire: Montant,
        taux_tva: Montant,
        total: Montant,
        compte_produit_id: Option<i64>,
    }
}

sortie! {
    /// La fiche d'une facture.
    struct FicheFacture {
        id: i64,
        client_id: i64,
        numero: Option<TexteNonFiable>,
        statut: Code<StatutFacture>,
        date: Date,
        echeance: Option<Date>,
        conditions: Option<TexteNonFiable>,
        montant_ht: Montant,
        arrondi: Montant,
        ecriture_id: Option<i64>,
        payee_le: Option<Horodatage>,
        envoyee_le: Option<Horodatage>,
        envoyee_a: Tiers,
        projet_id: Option<i64>,
        suspendue_le: Option<Horodatage>,
        note_suspension: Option<TexteNonFiable>,
        montant_regle: Montant,
        reste_du: Montant,
        lignes: Vec<LigneFacture>,
    }
}

fn facture_liste(f: InvoiceListItem) -> FactureListe {
    FactureListe {
        id: f.id,
        client_id: f.contact_id,
        client: TexteNonFiable::de(f.contact_name),
        numero: f.invoice_number.map(TexteNonFiable::de),
        statut: Code::parmi(f.status, STATUTS_FACTURE),
        date: Date(f.date),
        echeance: f.due_date.map(Date),
        conditions: f.payment_terms.map(TexteNonFiable::de),
        montant_ht: Montant(f.total_amount),
        total_ttc: Montant(f.total_ttc),
        montant_regle: Montant(f.amount_settled),
        reste_du: Montant(f.amount_due),
        payee_le: f.paid_at.map(Horodatage),
        suspendue_le: f.dunning_paused_at.map(Horodatage),
        note_suspension: f.dunning_paused_note.map(TexteNonFiable::de),
    }
}

const SCHEMA_LISTE: &str = r#"{"type":"object","properties":{"search":{"type":"string","maxLength":100},"status":{"type":"string","enum":["draft","validated","cancelled"]},"contactId":{"type":"integer","minimum":1},"dateFrom":{"type":"string","format":"date"},"dateTo":{"type":"string","format":"date"},"paused":{"type":"string","enum":["all","paused","notPaused"]},"sortBy":{"type":"string","enum":["date","dueDate","totalAmount","contactName","createdAt"]},"sortDirection":{"type":"string","enum":["asc","desc"]},"limit":{"type":"integer","minimum":1,"maximum":40,"default":20},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#;

const SCHEMA_ECHEANCIER: &str = r#"{"type":"object","properties":{"search":{"type":"string","maxLength":100},"contactId":{"type":"integer","minimum":1},"dateFrom":{"type":"string","format":"date"},"dateTo":{"type":"string","format":"date"},"dueBefore":{"type":"string","format":"date"},"paymentStatus":{"type":"string","enum":["all","paid","unpaid","overdue"]},"sortBy":{"type":"string","enum":["date","dueDate","totalAmount","contactName","createdAt"]},"sortDirection":{"type":"string","enum":["asc","desc"]},"limit":{"type":"integer","minimum":1,"maximum":40,"default":20},"offset":{"type":"integer","minimum":0,"maximum":1000000}},"additionalProperties":false}"#;

impl McpTool for KeshFactures {
    fn nom(&self) -> &'static str {
        "kesh_factures"
    }

    fn description(&self) -> &'static str {
        "Liste les factures clientes : client, numéro, statut, dates, montants, réglé et \
         reste dû ; filtrées (recherche, statut, client, dates, suspension des rappels), \
         triées et paginées (limit, offset). Si la réponse est trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_LISTE
    }

    fn portee(&self) -> Portee {
        Portee::InvoicesRead
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
            let resume = Resume::outil("kesh_factures")
                .recherche_fournie("search", a.search.is_some())
                .code("status", a.status.map(StatutArg::code))
                .entier("contactId", a.contact_id)
                .date("dateFrom", a.date_from)
                .date("dateTo", a.date_to)
                .code("paused", a.paused.map(SuspensionArg::code))
                .code("sortBy", a.sort_by.map(TriFactures::code))
                .code("sortDirection", a.sort_direction.map(SensDuTri::code))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeListeFactures {
                search: a.search,
                status: a.status.map(|s| s.code().to_string()),
                contact_id: a.contact_id,
                date_from: a.date_from,
                date_to: a.date_to,
                paused: a.paused.map(SuspensionArg::du_service),
                sort_by: a.sort_by.map(TriFactures::du_service),
                sort_direction: a.sort_direction.map(SensDuTri::du_service),
                limit: Some(limit),
                offset: Some(offset),
            };
            let page = factures::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = Factures {
                factures: page.items.into_iter().map(facture_liste).collect(),
                total: page.total,
                limit: page.limit,
                offset: page.offset,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshEcheancier {
    fn nom(&self) -> &'static str {
        "kesh_echeancier"
    }

    fn description(&self) -> &'static str {
        "L'échéancier des factures clientes validées : par défaut les impayées \
         (paymentStatus : all, paid, unpaid, overdue), avec le nombre et le total des \
         impayées et des échues ; filtré, trié et paginé (limit, offset). Si la réponse est \
         trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        SCHEMA_ECHEANCIER
    }

    fn portee(&self) -> Portee {
        Portee::InvoicesRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let a: ArgumentsEcheancier = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            let resume = Resume::outil("kesh_echeancier")
                .recherche_fournie("search", a.search.is_some())
                .entier("contactId", a.contact_id)
                .date("dateFrom", a.date_from)
                .date("dateTo", a.date_to)
                .date("dueBefore", a.due_before)
                .code("paymentStatus", a.payment_status.map(PaiementArg::code))
                .code("sortBy", a.sort_by.map(TriFactures::code))
                .code("sortDirection", a.sort_direction.map(SensDuTri::code))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeEcheances {
                search: a.search,
                contact_id: a.contact_id,
                date_from: a.date_from,
                date_to: a.date_to,
                due_before: a.due_before,
                payment_status: a.payment_status.map(PaiementArg::du_service),
                sort_by: a.sort_by.map(TriFactures::du_service),
                sort_direction: a.sort_direction.map(SensDuTri::du_service),
                limit: Some(limit),
                offset: Some(offset),
            };
            let e = factures::echeances(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = Echeancier {
                factures: e.page.items.into_iter().map(facture_liste).collect(),
                total: e.page.total,
                limit: e.page.limit,
                offset: e.page.offset,
                resume: ResumeEcheances {
                    impayees_nombre: e.resume.unpaid_count,
                    impayees_total: Montant(e.resume.unpaid_total),
                    echues_nombre: e.resume.overdue_count,
                    echues_total: Montant(e.resume.overdue_total),
                },
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshFacture {
    fn nom(&self) -> &'static str {
        "kesh_facture"
    }

    fn description(&self) -> &'static str {
        "Lit une facture cliente par son identifiant : en-tête, lignes, réglé et reste dû, \
         envoi par courriel. L'adresse d'envoi est masquée sans la portée contacts:read."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"id":{"type":"integer","minimum":1}},"required":["id"],"additionalProperties":false}"#
    }

    fn portee(&self) -> Portee {
        Portee::InvoicesRead
    }

    fn executer<'a>(
        &'a self,
        ctx: &'a ContexteMcp<'a>,
        valeur: serde_json::Value,
    ) -> FuturOutil<'a> {
        Box::pin(async move {
            let ArgumentsDetail { id } = arguments(valeur)?;
            let fiche = factures::detail_sans_fichier(ctx.acces(), ctx.lecture(), id)
                .await
                .map_err(depuis_app_error)?;
            let f = fiche.facture;
            let sortie = FicheFacture {
                id: f.id,
                client_id: f.contact_id,
                numero: f.invoice_number.map(TexteNonFiable::de),
                statut: Code::parmi(f.status, STATUTS_FACTURE),
                date: Date(f.date),
                echeance: f.due_date.map(Date),
                conditions: f.payment_terms.map(TexteNonFiable::de),
                montant_ht: Montant(f.total_amount),
                arrondi: Montant(f.rounding_amount),
                ecriture_id: f.journal_entry_id,
                payee_le: f.paid_at.map(Horodatage),
                envoyee_le: f.emailed_at.map(Horodatage),
                envoyee_a: Tiers::selon(ctx, f.emailed_to),
                projet_id: f.project_id,
                suspendue_le: f.dunning_paused_at.map(Horodatage),
                note_suspension: f.dunning_paused_note.map(TexteNonFiable::de),
                montant_regle: Montant(fiche.montant_regle),
                reste_du: Montant(fiche.reste_du),
                lignes: fiche
                    .lignes
                    .into_iter()
                    .map(|l| LigneFacture {
                        id: l.id,
                        position: l.position,
                        description: TexteNonFiable::de(l.description),
                        quantite: Montant(l.quantity),
                        prix_unitaire: Montant(l.unit_price),
                        taux_tva: Montant(l.vat_rate),
                        total: Montant(l.line_total),
                        compte_produit_id: l.revenue_account_id,
                    })
                    .collect(),
            };
            donnees(
                Resume::outil("kesh_facture").entier("id", Some(id)),
                &sortie,
            )
        })
    }
}
