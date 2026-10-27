//! `kesh_factures_fournisseur` et `kesh_facture_fournisseur` — les factures
//! fournisseur (Story 32-9a, AC-a10). Portée `invoices:read`.
//!
//! L'IBAN et le QR-IBAN du créancier sont des coordonnées de tiers
//! ([`Tiers`]), masquées sans `contacts:read` ; le nom du fournisseur sort,
//! enveloppé. De la pièce importée, seul l'identifiant sort :
//! `detail_sans_fichier` ne transmet aucun chemin de fichier (C-32-9-13).

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{
    STATUTS_FACTURE_FOURNISSEUR, StatutFactureFournisseur, TYPES_REGLEMENT, TypeReglement,
    decalage, limite,
};
use crate::mcp::sortie::{
    Code, Date, Horodatage, Montant, Resume, TexteNonFiable, Tiers, donnees, sortie,
};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::factures_fournisseur::{
    self, DemandeListeFacturesFournisseur, FicheFactureFournisseur,
};

/// Taille de page par défaut (§ *Les tailles mesurées*).
pub const LIMITE_DEFAUT: i64 = 40;
/// Taille de page maximale de l'outil (le service admet 200).
pub const LIMITE_MAX_MCP: i64 = 70;

/// La liste.
pub struct KeshFacturesFournisseur;
/// La fiche.
pub struct KeshFactureFournisseur;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsListe {
    limit: Option<i64>,
    offset: Option<i64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgumentsDetail {
    id: i64,
}

sortie! {
    /// Une facture fournisseur de liste.
    struct FactureFournisseurListe {
        id: i64,
        fournisseur_id: i64,
        fournisseur: TexteNonFiable,
        numero: Option<TexteNonFiable>,
        statut: Code<StatutFactureFournisseur>,
        date: Date,
        echeance: Option<Date>,
        montant: Montant,
    }
}

sortie! {
    /// La liste.
    struct FacturesFournisseur {
        factures: Vec<FactureFournisseurListe>,
        total: i64,
        limit: i64,
        offset: i64,
    }
}

sortie! {
    /// Le règlement d'une facture payée.
    struct Reglement {
        r#type: Code<TypeReglement>,
        compte_bancaire_id: Option<i64>,
        compte_id: Option<i64>,
        ecriture_id: Option<i64>,
    }
}

sortie! {
    /// Une ligne.
    struct LigneFactureFournisseur {
        id: i64,
        position: i32,
        description: TexteNonFiable,
        quantite: Montant,
        prix_unitaire: Montant,
        taux_tva: Montant,
        total: Montant,
        compte_charge_id: i64,
    }
}

sortie! {
    /// La fiche.
    struct FicheSortie {
        id: i64,
        fournisseur_id: i64,
        numero: Option<TexteNonFiable>,
        statut: Code<StatutFactureFournisseur>,
        date: Date,
        echeance: Option<Date>,
        montant: Montant,
        iban_creancier: Tiers,
        qr_iban_creancier: Tiers,
        reference_paiement: Option<TexteNonFiable>,
        montant_attendu: Option<Montant>,
        projet_id: Option<i64>,
        ecriture_achat_id: i64,
        reglement: Option<Reglement>,
        payee_le: Option<Horodatage>,
        piece_id: Option<i64>,
        lignes: Vec<LigneFactureFournisseur>,
    }
}

impl McpTool for KeshFacturesFournisseur {
    fn nom(&self) -> &'static str {
        "kesh_factures_fournisseur"
    }

    fn description(&self) -> &'static str {
        "Liste les factures fournisseur, les plus récentes d'abord : fournisseur, numéro, \
         statut, dates, montant ; paginées (limit, offset). Si la réponse est trop grosse, \
         réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"limit":{"type":"integer","minimum":1,"maximum":70,"default":40},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#
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
            let resume = Resume::outil("kesh_factures_fournisseur")
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeListeFacturesFournisseur {
                limit: Some(limit),
                offset: Some(offset),
            };
            let page = factures_fournisseur::lister(ctx.acces(), ctx.lecture(), demande)
                .await
                .map_err(depuis_app_error)?;
            let sortie = FacturesFournisseur {
                factures: page
                    .items
                    .into_iter()
                    .map(|f| FactureFournisseurListe {
                        id: f.id,
                        fournisseur_id: f.contact_id,
                        fournisseur: TexteNonFiable::de(f.contact_name),
                        numero: f.supplier_invoice_number.map(TexteNonFiable::de),
                        statut: Code::parmi(f.status, STATUTS_FACTURE_FOURNISSEUR),
                        date: Date(f.invoice_date),
                        echeance: f.due_date.map(Date),
                        montant: Montant(f.total_amount),
                    })
                    .collect(),
                total: page.total,
                limit: page.limit,
                offset: page.offset,
            };
            donnees(resume, &sortie)
        })
    }
}

impl McpTool for KeshFactureFournisseur {
    fn nom(&self) -> &'static str {
        "kesh_facture_fournisseur"
    }

    fn description(&self) -> &'static str {
        "Lit une facture fournisseur par son identifiant : en-tête, coordonnées de paiement, \
         règlement, lignes, identifiant de la pièce importée. L'IBAN et le QR-IBAN du \
         créancier sont masqués sans la portée contacts:read."
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
            let FicheFactureFournisseur {
                facture: f,
                lignes,
                piece_id,
            } = factures_fournisseur::detail_sans_fichier(ctx.acces(), ctx.lecture(), id)
                .await
                .map_err(depuis_app_error)?;
            let reglement = f.settlement_type.map(|t| Reglement {
                r#type: Code::parmi(t, TYPES_REGLEMENT),
                compte_bancaire_id: f.settlement_bank_account_id,
                compte_id: f.settlement_account_id,
                ecriture_id: f.settlement_journal_entry_id,
            });
            let sortie = FicheSortie {
                id: f.id,
                fournisseur_id: f.contact_id,
                numero: f.supplier_invoice_number.map(TexteNonFiable::de),
                statut: Code::parmi(f.status, STATUTS_FACTURE_FOURNISSEUR),
                date: Date(f.invoice_date),
                echeance: f.due_date.map(Date),
                montant: Montant(f.total_amount),
                iban_creancier: Tiers::selon(ctx, f.creditor_iban),
                qr_iban_creancier: Tiers::selon(ctx, f.creditor_qr_iban),
                reference_paiement: f.payment_reference.map(TexteNonFiable::de),
                montant_attendu: f.expected_payment_amount.map(Montant),
                projet_id: f.project_id,
                ecriture_achat_id: f.purchase_journal_entry_id,
                reglement,
                payee_le: f.paid_at.map(Horodatage),
                piece_id,
                lignes: lignes
                    .into_iter()
                    .map(|l| LigneFactureFournisseur {
                        id: l.id,
                        position: l.position,
                        description: TexteNonFiable::de(l.description),
                        quantite: Montant(l.quantity),
                        prix_unitaire: Montant(l.unit_price),
                        taux_tva: Montant(l.vat_rate),
                        total: Montant(l.line_total),
                        compte_charge_id: l.expense_account_id,
                    })
                    .collect(),
            };
            donnees(
                Resume::outil("kesh_facture_fournisseur").entier("id", Some(id)),
                &sortie,
            )
        })
    }
}
