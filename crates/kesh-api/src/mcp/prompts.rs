//! Les prompts du serveur MCP — le catalogue servi par `prompts/list` et
//! `prompts/get` (Story 32-10a-ii, refs #723 ; socle AC-a24, C-32-6-23).
//!
//! # La chaîne des types (AC-ii4, AC-ii8)
//!
//! ```text
//! services::prompts_mcp::{lister, obtenir}  → PromptServi   (champs privés, construit par le service)
//! PromptEnBase::depuis(&PromptServi)        → PromptEnBase  (seule implémentation du trait scellé)
//! ListePrompts / PromptRendu                ← PromptEnBase  (seules charges de ReponseMcp pour les prompts)
//! ```
//!
//! Aucun pool ni aucune poignée n'entre ici : le catalogue lit par
//! `etat.lecture_prompts` à travers le service, qui exige la portée,
//! revalide chaque version et compose la marque d'un prompt d'agent. Une donnée
//! d'outil ne peut pas entrer dans ces types (`Donnees` est illisible hors de
//! `politique.rs`, AC-a23).
//!
//! Le trait [`McpPrompt`] est **scellé** — seul ce module peut l'implémenter
//! (garde `prompts_scelles`) — et `rendre` ne reçoit **aucun argument**
//! (C-32-10a-10) : un prompt est un texte fixe, les arguments d'un client sont
//! ignorés, jamais interpolés.

use std::time::Instant;

use crate::mcp::EtatMcp;
use crate::mcp::textes;
use crate::services::acces::Portee;
use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::prompts_mcp::{
    self, AccuseServi, ErreurPrompts, PromptServi, RefusProposition,
};

mod scelle {
    /// Le sceau : privé à `mcp::prompts`.
    pub trait Scelle {}
}

/// Un prompt servi par `prompts/get` — amendé en connaissance de cause par la
/// 32-10a-ii (C-32-6-23) : des textes lus en base, à travers le service.
pub trait McpPrompt: scelle::Scelle + Send + Sync + 'static {
    /// Nom unique dans la société.
    fn nom(&self) -> &str;
    /// Description servie à `prompts/list` et `prompts/get`.
    fn description(&self) -> &str;
    /// Les arguments déclarés : aucun en 0.16.0 (C-32-10a-10).
    fn arguments(&self) -> &'static [&'static str] {
        &[]
    }
    /// La portée exigée pour lister ou obtenir le prompt (C-32-10a-16).
    fn portee(&self) -> Portee {
        Portee::BooksRead
    }
    /// Le texte du prompt, premier message de `prompts/get` — sans argument.
    fn rendre(&self) -> &str;
}

/// Un prompt lu en base — **seule** implémentation de [`McpPrompt`]. Champs
/// privés, sans pool ni poignée ; construit par [`PromptEnBase::depuis`], à
/// partir d'un `&PromptServi` que seuls les services construisent.
pub struct PromptEnBase {
    nom: String,
    numero: i32,
    description: String,
    texte: String,
}

impl PromptEnBase {
    /// La seule fabrique : ce que le service sert, marque comprise.
    pub(in crate::mcp) fn depuis(p: &PromptServi) -> Self {
        Self {
            nom: p.nom().to_string(),
            numero: p.numero(),
            description: p.description().to_string(),
            texte: p.texte().to_string(),
        }
    }
}

impl scelle::Scelle for PromptEnBase {}

impl McpPrompt for PromptEnBase {
    fn nom(&self) -> &str {
        &self.nom
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn rendre(&self) -> &str {
        &self.texte
    }
}

/// La charge de `prompts/list` : des [`PromptEnBase`] seulement. Ni
/// `Serialize`, ni `Default`, ni `Debug`.
pub(crate) struct ListePrompts {
    prompts: Vec<PromptEnBase>,
}

impl ListePrompts {
    /// La liste vide — une clé sans `books:read` (aucun oracle, AC-ii5).
    pub(in crate::mcp) fn vide() -> Self {
        Self {
            prompts: Vec::new(),
        }
    }

    /// Le `result` de `prompts/list` : `{"prompts": [{"name", "description",
    /// "arguments": []}…]}` (type `Prompt` de la révision `2025-11-25` ; `title`
    /// non servi).
    pub(in crate::mcp) fn en_resultat(&self) -> serde_json::Value {
        let prompts: Vec<serde_json::Value> = self
            .prompts
            .iter()
            .map(|p| {
                serde_json::json!({
                    "name": p.nom(),
                    "description": p.description(),
                    "arguments": p.arguments(),
                })
            })
            .collect();
        serde_json::json!({ "prompts": prompts })
    }
}

/// La charge de `prompts/get` : un [`PromptEnBase`]. Ni `Serialize`, ni
/// `Default`, ni `Debug`.
pub(crate) struct PromptRendu {
    prompt: PromptEnBase,
}

impl PromptRendu {
    /// Le nom du prompt servi — celui du catalogue, que le service a
    /// revalidé (journal de `prompts/get`, AC-iii5).
    pub(in crate::mcp) fn nom(&self) -> &str {
        self.prompt.nom()
    }

    /// Le numéro de la version servie (journal de `prompts/get`, AC-iii5).
    pub(in crate::mcp) fn numero(&self) -> i32 {
        self.prompt.numero
    }

    /// Le `result` de `prompts/get` : `{"description", "messages": [{"role":
    /// "user", "content": {"type": "text", "text"}}]}` (`GetPromptResult` de la
    /// révision `2025-11-25`).
    pub(in crate::mcp) fn en_resultat(&self) -> serde_json::Value {
        serde_json::json!({
            "description": self.prompt.description(),
            "messages": [{
                "role": "user",
                "content": { "type": "text", "text": self.prompt.rendre() },
            }],
        })
    }
}

/// `prompts/list` : les prompts servis de la société de la clé.
pub(in crate::mcp) async fn lister(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
) -> Result<ListePrompts, ErreurPrompts> {
    let servis = prompts_mcp::lister(&cle.acces, &etat.lecture_prompts).await?;
    Ok(ListePrompts {
        prompts: servis.iter().map(PromptEnBase::depuis).collect(),
    })
}

/// `prompts/get` : le prompt `nom` de la société de la clé, s'il est servi.
pub(in crate::mcp) async fn obtenir(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    nom: &str,
) -> Result<Option<PromptRendu>, ErreurPrompts> {
    let servi = prompts_mcp::obtenir(&cle.acces, &etat.lecture_prompts, nom).await?;
    Ok(servi.map(|p| PromptRendu {
        prompt: PromptEnBase::depuis(&p),
    }))
}

// ---------------------------------------------------------------------------
// L'accusé de `proposer_prompt` — Story 32-10a-iii (AC-iii2, AC-iii3)
// ---------------------------------------------------------------------------

/// La charge de l'accusé d'une proposition : le nom **validé** seul. Construit
/// à partir d'un [`AccuseServi`] — que seul le service construit — par
/// [`AccuseProposition::depuis`] ; ni `Serialize`, ni `Default`, ni `Debug`, ni
/// `Clone`. Aucune donnée lue n'y entre (spike § 5.3.1, E2 : l'accusé ne passe
/// pas par la validation).
pub(crate) struct AccuseProposition {
    nom: String,
}

impl AccuseProposition {
    /// La seule fabrique.
    pub(in crate::mcp) fn depuis(a: &AccuseServi) -> Self {
        Self {
            nom: a.nom().to_string(),
        }
    }

    /// Le `result` du `tools/call` : un bloc de texte constant où n'entre que
    /// le nom validé, `structuredContent` `{"nom", "etat": "a_valider"}`,
    /// `isError: false` (C-32-10a-17).
    pub(in crate::mcp) fn en_resultat(&self) -> serde_json::Value {
        serde_json::json!({
            "content": [{
                "type": "text",
                "text": textes::texte_accuse_proposition(&self.nom),
            }],
            "structuredContent": { "nom": self.nom, "etat": "a_valider" },
            "isError": false,
        })
    }
}

/// `proposer_prompt` : la proposition de l'agent de la clé, par le service
/// (portée `prompts:write`, contenu, nom, version et audit en une transaction).
pub(in crate::mcp) async fn proposer(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    debut: Instant,
    arguments: serde_json::Value,
) -> Result<AccuseProposition, RefusProposition> {
    let accuse = prompts_mcp::proposer(cle, &etat.ecriture_prompts, debut, arguments).await?;
    Ok(AccuseProposition::depuis(&accuse))
}
