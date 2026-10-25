//! L'outil de démonstration `kesh_version` (AC-a28) : la version du binaire, le
//! mode de la session qui l'a servi et sa borne d'instruction
//! (`delaiInstructionS`, C-32-6-54). **Aucune donnée des livres.**

use serde::Deserialize;

use crate::mcp::outil::{ContexteMcp, FuturOutil, McpTool, arguments};
use crate::mcp::politique::Donnees;
use crate::mcp::textes::depuis_app_error;
use crate::services::acces::Portee;
use crate::services::serveur_mcp;

/// L'outil — une unité sans champ, construite par `Registre::standard()`.
pub struct KeshVersion;

/// Aucun argument.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SansArgument {}

impl McpTool for KeshVersion {
    fn nom(&self) -> &'static str {
        "kesh_version"
    }

    fn description(&self) -> &'static str {
        "Version de Kesh, mode de la session de lecture (lecture seule ou non) et délai \
         au-delà duquel la base interrompt une instruction. Ne lit aucune donnée comptable."
    }

    fn schema_entree(&self) -> &'static str {
        r#"{"type":"object","properties":{},"additionalProperties":false}"#
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
            let SansArgument {} = arguments(valeur)?;
            let etat = serveur_mcp::etat(ctx.acces(), ctx.lecture())
                .await
                .map_err(depuis_app_error)?;
            let structure = serde_json::json!({
                "version": etat.version,
                "lectureSeule": etat.lecture_seule,
                "delaiInstructionS": etat.delai_instruction_s,
            });
            Ok(Donnees::nouvelles(structure.to_string(), Some(structure)))
        })
    }
}
