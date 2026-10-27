//! `kesh_grand_livre` — le grand livre d'**un** compte sur une période
//! (Story 32-9a, AC-a10, C-32-9-8) ; la consultation s'inscrit au journal
//! d'audit (`ContexteMcp::afficher`). Portée `books:read`.

use chrono::NaiveDate;
use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{SENS, Sens, decalage, limite};
use crate::mcp::sortie::{Code, Date, Montant, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::rapports::{self, Affichage, DemandeGrandLivre};

/// Lignes par défaut (§ *Les tailles mesurées*).
pub const LIMITE_DEFAUT: i64 = 15;
/// Lignes au plus, pour l'outil (le service en admet `MAX_LEDGER_LIMIT`).
pub const LIMITE_MAX_MCP: i64 = 30;

/// L'outil.
pub struct KeshGrandLivre;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Arguments {
    account_id: i64,
    from: NaiveDate,
    to: NaiveDate,
    limit: Option<i64>,
    offset: Option<i64>,
}

sortie! {
    /// Une ligne du grand livre.
    struct LigneGrandLivre {
        ligne_id: i64,
        ecriture_id: i64,
        date: Date,
        exercice_id: i64,
        exercice: TexteNonFiable,
        numero_ecriture: i64,
        journal: Code<kesh_db::entities::Journal>,
        libelle: TexteNonFiable,
        contreparties: Vec<TexteNonFiable>,
        debit: Montant,
        credit: Montant,
        solde_courant: Montant,
        code_lettrage: Option<TexteNonFiable>,
    }
}

sortie! {
    /// Une rupture d'exercice dans la section.
    struct Rupture { date: Date, exercice_clos_id: i64, solde_de_cloture: Montant }
}

sortie! {
    /// La section d'un compte.
    struct Section {
        compte_id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        actif: bool,
        sens: Code<Sens>,
        ouverture: Montant,
        lignes: Vec<LigneGrandLivre>,
        total_debit: Montant,
        total_credit: Montant,
        cloture: Montant,
        solde_inhabituel: bool,
        ruptures: Vec<Rupture>,
        nombre_lignes: i64,
    }
}

sortie! {
    /// La sortie.
    struct GrandLivre { du: Date, au: Date, limit: i64, offset: i64, sections: Vec<Section> }
}

impl McpTool for KeshGrandLivre {
    fn nom(&self) -> &'static str {
        "kesh_grand_livre"
    }

    fn description(&self) -> &'static str {
        "Lit le grand livre d'un compte (accountId, requis) entre deux dates (from, to, \
         requises) : solde d'ouverture, lignes avec contreparties et solde courant, totaux. \
         Un compte à la fois ; lignes paginées par limit et offset (nombreLignes donne le \
         total). Si la réponse est trop grosse, réduisez limit. Sur un compte de produits ou \
         de charges, le solde courant repart de zéro à chaque rupture d'exercice (ruptures : \
         date, exercice clos, son solde), et cloture est le solde de l'exercice en cours à \
         la fin de la période : il vaut l'ouverture plus les totaux, sauf quand la période \
         franchit une fin d'exercice (le cumul reste alors dans les totaux). Sur un compte \
         d'actif ou de passif, cloture vaut toujours l'ouverture plus les totaux."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"accountId":{"type":"integer","minimum":1},"from":{"type":"string","format":"date"},"to":{"type":"string","format":"date"},"limit":{"type":"integer","minimum":1,"maximum":30,"default":15},"offset":{"type":"integer","minimum":0}},"required":["accountId","from","to"],"additionalProperties":false}"#
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
            let resume = Resume::outil("kesh_grand_livre")
                .entier("accountId", Some(a.account_id))
                .date("from", Some(a.from))
                .date("to", Some(a.to))
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            let demande = DemandeGrandLivre {
                from: a.from,
                to: a.to,
                account_ids: Some(a.account_id.to_string()),
                include_zero: true,
                offset: Some(offset),
                limit: Some(limit),
            };
            let rapport = rapports::grand_livre(ctx.acces(), ctx.lecture(), demande, Affichage)
                .await
                .map_err(depuis_app_error)?;
            let livre = ctx.afficher(rapport).await;
            let sortie = GrandLivre {
                du: Date(livre.period.from),
                au: Date(livre.period.to),
                limit,
                offset,
                sections: livre.sections.into_iter().map(section).collect(),
            };
            donnees(resume, &sortie)
        })
    }
}

fn section(s: kesh_report::general_ledger::LedgerSection) -> Section {
    Section {
        compte_id: s.account_id,
        numero: TexteNonFiable::de(s.account_number),
        nom: TexteNonFiable::de(s.account_name),
        type_de_compte: s.account_type,
        actif: s.active,
        sens: Code::parmi(s.balance_side.to_string(), SENS),
        ouverture: Montant(s.opening),
        lignes: s
            .lines
            .into_iter()
            .map(|l| LigneGrandLivre {
                ligne_id: l.line_id,
                ecriture_id: l.entry_id,
                date: Date(l.entry_date),
                exercice_id: l.fiscal_year_id,
                exercice: TexteNonFiable::de(l.fiscal_year_name),
                numero_ecriture: l.entry_number,
                journal: Code::depuis_str(l.journal),
                libelle: TexteNonFiable::de(l.description),
                contreparties: l.counterpart.into_iter().map(TexteNonFiable::de).collect(),
                debit: Montant(l.debit),
                credit: Montant(l.credit),
                solde_courant: Montant(l.running_balance),
                code_lettrage: l.lettering_code.map(TexteNonFiable::de),
            })
            .collect(),
        total_debit: Montant(s.total_debit),
        total_credit: Montant(s.total_credit),
        cloture: Montant(s.closing),
        solde_inhabituel: s.unnatural_balance,
        ruptures: s
            .fiscal_year_breaks
            .into_iter()
            .map(|r| Rupture {
                date: Date(r.date),
                exercice_clos_id: r.closing_fiscal_year_id,
                solde_de_cloture: Montant(r.closing_balance),
            })
            .collect(),
        nombre_lignes: s.line_count,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #778 — un agent qui additionne ouverture et totaux n'obtient pas `cloture`
    /// sur un compte de résultat à cheval sur deux exercices : la description le
    /// lui dit, et nomme `ruptures`. Revue P2 (P2-R-2) : elle ne le nie que dans ce
    /// cas — sur un seul exercice, ou sur un compte de bilan, l'identité vaut.
    #[test]
    fn la_description_definit_cloture_et_ruptures() {
        let d = KeshGrandLivre.description();
        assert!(
            d.contains("cloture est le solde de l'exercice en cours"),
            "{d}"
        );
        assert!(d.contains("ruptures"), "{d}");
        assert!(
            d.contains(
                "il vaut l'ouverture plus les totaux, sauf quand la période franchit une fin \
                 d'exercice"
            ),
            "la négation de l'identité doit rester conditionnée : {d}"
        );
        assert!(
            d.contains("cloture vaut toujours l'ouverture plus les totaux"),
            "compte de bilan : {d}"
        );
    }
}
