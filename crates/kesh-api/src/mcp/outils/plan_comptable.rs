//! `kesh_plan_comptable` — le plan comptable (Story 32-9a, AC-a10). Le service
//! ne pagine pas : l'outil tranche (défaut 38, maximum 68 comptes, § *Les
//! tailles mesurées*). Portée `books:read`.
//!
//! Story 37-1 (AC-8) : avec le champ `devise`, un compte mesure 334 octets ;
//! 40 / 70 dépassaient les budgets de `tailles_mesurees` (13 729 o
//! au défaut pour 13 312, 23 779 o au maximum pour 23 552) — re-mesuré,
//! 38 / 68 (même révision que la balance âgée, C-32-9-44).

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::outils::commun::{decalage, limite, nombre, tranche};
use crate::mcp::sortie::{Code, Devise, Resume, TexteNonFiable, donnees, sortie};
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::plan_comptable;

/// Taille de page par défaut.
pub const LIMITE_DEFAUT: i64 = 38;
/// Taille de page maximale (aucun plafond de service).
pub const LIMITE_MAX_MCP: i64 = 68;

/// L'outil.
pub struct KeshPlanComptable;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Arguments {
    inclure_archives: Option<bool>,
    limit: Option<i64>,
    offset: Option<i64>,
}

sortie! {
    /// Un compte.
    struct Compte {
        id: i64,
        numero: TexteNonFiable,
        nom: TexteNonFiable,
        type_de_compte: kesh_db::entities::AccountType,
        parent_id: Option<i64>,
        actif: bool,
        role: Option<kesh_db::entities::AccountRole>,
        imputable: bool,
        lettrable: bool,
        // Story 37-1 (AC-8) : la devise du compte, `null` pour le CHF (clé
        // toujours présente). Les comptes de différence propres n'y sont pas.
        devise: Option<Code<Devise>>,
    }
}

sortie! {
    /// La sortie.
    struct PlanComptable { comptes: Vec<Compte>, total: i64, limit: i64, offset: i64 }
}

impl McpTool for KeshPlanComptable {
    fn nom(&self) -> &'static str {
        "kesh_plan_comptable"
    }

    fn description(&self) -> &'static str {
        "Lit le plan comptable de la société, trié par numéro : identifiant, numéro, nom, \
         type, rôle, devise (code ISO 4217, null pour la devise principale CHF), comptes \
         archivés sur demande (inclureArchives). Paginé par limit et offset ; si la \
         réponse est trop grosse, réduisez limit."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{"inclureArchives":{"type":"boolean"},"limit":{"type":"integer","minimum":1,"maximum":68,"default":38},"offset":{"type":"integer","minimum":0}},"additionalProperties":false}"#
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
            let comptes = plan_comptable::lister(
                ctx.acces(),
                ctx.lecture(),
                a.inclure_archives.unwrap_or(false),
            )
            .await
            .map_err(depuis_app_error)?;
            let total = nombre(comptes.len());
            let sortie = PlanComptable {
                comptes: tranche(comptes, offset, limit)
                    .into_iter()
                    .map(|c| Compte {
                        id: c.compte.id,
                        numero: TexteNonFiable::de(c.compte.number),
                        nom: TexteNonFiable::de(c.compte.name),
                        type_de_compte: c.compte.account_type,
                        parent_id: c.compte.parent_id,
                        actif: c.compte.active,
                        role: c.compte.role,
                        imputable: c.compte.postable,
                        lettrable: c.lettrable,
                        devise: c.compte.currency.map(Code::depuis_str),
                    })
                    .collect(),
                total,
                limit,
                offset,
            };
            let resume = Resume::outil("kesh_plan_comptable")
                .booleen("inclureArchives", a.inclure_archives)
                .entier("limit", Some(limit))
                .entier("offset", Some(offset));
            donnees(resume, &sortie)
        })
    }
}
