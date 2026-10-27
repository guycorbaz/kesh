//! `kesh_postes_ouverts` — les postes ouverts d'un compte lettrable à une date
//! (Story 32-9a, AC-a10). Portée `books:read`. Un compte non lettrable rend
//! `TEXTE_COMPTE_NON_LETTRABLE` (AC-a22).

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{decalage, limite};
use crate::mcp::sortie::{
    Code, Date, Montant, Resume, TexteNonFiable, champ_enum, donnees, sortie,
};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::postes_ouverts::{
    self, DemandePostesOuverts, DocumentKind, DocumentOwner, DocumentState, OpenItem, OpenReason,
    Origin,
};

/// Postes par défaut (§ *Les tailles mesurées*).
pub const LIMITE_DEFAUT: i64 = 12;
/// Postes au plus, pour l'outil.
pub const LIMITE_MAX_MCP: i64 = 24;

/// L'outil.
pub struct KeshPostesOuverts;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Arguments {
    account_id: i64,
    as_of: Option<NaiveDate>,
    limit: Option<i64>,
    offset: Option<i64>,
}

/// Le type de la pièce qui possède l'écriture (miroir de `DocumentKind`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum NaturePiece {
    Invoice,
    CreditNote,
    SupplierInvoice,
    Settlement,
    BankTransaction,
}

/// Pourquoi une ligne est ouverte (miroir d'`OpenReason`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum MotifOuvert {
    Unlettered,
    LetteredAfterAsOf,
}

/// L'état de la facture d'une ligne (miroir de `DocumentState`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum EtatPiece {
    PaidWithoutSettlementEntry,
    NothingDue,
    PartiallySettled,
    Unpaid,
}

/// L'origine d'un lettrage (miroir d'`Origin`).
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
enum OrigineLettrage {
    Document,
    Reversal,
    Manual,
}

champ_enum!(NaturePiece, MotifOuvert, EtatPiece, OrigineLettrage);

fn nature(k: DocumentKind) -> NaturePiece {
    match k {
        DocumentKind::Invoice => NaturePiece::Invoice,
        DocumentKind::CreditNote => NaturePiece::CreditNote,
        DocumentKind::SupplierInvoice => NaturePiece::SupplierInvoice,
        DocumentKind::Settlement => NaturePiece::Settlement,
        DocumentKind::BankTransaction => NaturePiece::BankTransaction,
    }
}

fn motif(r: OpenReason) -> MotifOuvert {
    match r {
        OpenReason::Unlettered => MotifOuvert::Unlettered,
        OpenReason::LetteredAfterAsOf => MotifOuvert::LetteredAfterAsOf,
    }
}

fn etat(e: DocumentState) -> EtatPiece {
    match e {
        DocumentState::PaidWithoutSettlementEntry => EtatPiece::PaidWithoutSettlementEntry,
        DocumentState::NothingDue => EtatPiece::NothingDue,
        DocumentState::PartiallySettled => EtatPiece::PartiallySettled,
        DocumentState::Unpaid => EtatPiece::Unpaid,
    }
}

fn origine(o: Origin) -> OrigineLettrage {
    match o {
        Origin::Document => OrigineLettrage::Document,
        Origin::Reversal => OrigineLettrage::Reversal,
        Origin::Manual => OrigineLettrage::Manual,
    }
}

sortie! {
    /// La pièce qui possède l'écriture.
    struct Piece {
        nature: NaturePiece,
        id: i64,
        numero: Option<TexteNonFiable>,
        facture_id: Option<i64>,
        numero_facture: Option<TexteNonFiable>,
    }
}

sortie! {
    /// Un poste ouvert.
    struct Poste {
        ligne_id: i64,
        ecriture_id: i64,
        numero_ecriture: i64,
        exercice_id: i64,
        exercice: TexteNonFiable,
        date: Date,
        journal: Code<kesh_db::entities::Journal>,
        libelle: TexteNonFiable,
        debit: Montant,
        credit: Montant,
        piece: Option<Piece>,
        lettrage: Option<i64>,
        code_lettrage: Option<TexteNonFiable>,
        origine_lettrage: Option<OrigineLettrage>,
        lettre_le: Option<Date>,
        motif: MotifOuvert,
        etat_piece: Option<EtatPiece>,
        reste_du: Option<Montant>,
        lettrable_a_la_main: bool,
        en_periode_ouverte: bool,
    }
}

sortie! {
    /// La sortie.
    struct PostesOuverts {
        compte_id: i64,
        numero_compte: TexteNonFiable,
        au: Date,
        solde: Montant,
        total_ouvert: Montant,
        total: i64,
        limit: i64,
        offset: i64,
        postes: Vec<Poste>,
    }
}

fn piece(d: DocumentOwner) -> Piece {
    let DocumentOwner {
        kind,
        id,
        number,
        invoice_id,
        invoice_number,
    } = d;
    Piece {
        nature: nature(kind),
        id,
        numero: number.map(TexteNonFiable::de),
        facture_id: invoice_id,
        numero_facture: invoice_number.map(TexteNonFiable::de),
    }
}

fn poste(p: OpenItem) -> Poste {
    Poste {
        ligne_id: p.line_id,
        ecriture_id: p.entry_id,
        numero_ecriture: p.entry_number,
        exercice_id: p.fiscal_year_id,
        exercice: TexteNonFiable::de(p.fiscal_year_name),
        date: Date(p.date),
        journal: Code::depuis_str(p.journal),
        libelle: TexteNonFiable::de(p.description),
        debit: Montant(p.debit),
        credit: Montant(p.credit),
        piece: p.document.map(piece),
        lettrage: p.lettering_key,
        code_lettrage: p.lettering_code.map(TexteNonFiable::de),
        origine_lettrage: p.lettering_origin.map(origine),
        lettre_le: p.lettered_on.map(Date),
        motif: motif(p.reason),
        etat_piece: p.document_state.map(etat),
        reste_du: p.amount_due.map(Montant),
        lettrable_a_la_main: p.manually_letterable,
        en_periode_ouverte: p.in_open_period,
    }
}

impl McpTool for KeshPostesOuverts {
    fn nom(&self) -> &'static str {
        "kesh_postes_ouverts"
    }

    fn description(&self) -> &'static str {
        "Lit les postes ouverts (lignes non lettrées) d'un compte lettrable (accountId, \
         requis) à une date (asOf, défaut aujourd'hui) : solde, pièce, reste dû des factures. \
         Paginé par limit et offset ; si la réponse est trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"accountId":{"type":"integer","minimum":1},"asOf":{"type":"string","format":"date"},"limit":{"type":"integer","minimum":1,"maximum":24,"default":12},"offset":{"type":"integer","minimum":0}},"required":["accountId"],"additionalProperties":false}"#
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
            let a: Arguments = arguments(valeur)?;
            let limit = limite(a.limit, LIMITE_DEFAUT, LIMITE_MAX_MCP)?;
            let offset = decalage(a.offset)?;
            let resume = Resume::outil("kesh_postes_ouverts")
                .entier("accountId", Some(a.account_id))
                .date("asOf", a.as_of)
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandePostesOuverts {
                account_id: a.account_id,
                as_of: a.as_of.map(|d| d.to_string()),
                limit: Some(limit),
                offset: Some(offset),
            };
            let v = postes_ouverts::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = PostesOuverts {
                compte_id: v.account_id,
                numero_compte: TexteNonFiable::de(v.account_number),
                au: Date(v.as_of),
                solde: Montant(v.balance),
                total_ouvert: Montant(v.open_total),
                total: v.total,
                limit: v.limit,
                offset: v.offset,
                postes: v.items.into_iter().map(poste).collect(),
            };
            donnees(resume, &sortie)
        })
    }
}
