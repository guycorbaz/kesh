//! Le répartiteur JSON-RPC (AC-a12 à AC-a22) : la liste **close** des méthodes
//! servies, et le **seul** appelant de `McpTool::executer`, des points d'accès
//! du registre et de la politique.

use std::collections::BTreeSet;

use crate::mcp::EtatMcp;
use crate::mcp::outil::ContexteMcp;
use crate::mcp::politique;
use crate::mcp::reponse::{IdRequete, ReponseMcp};
use crate::mcp::textes;
use crate::services::acces::{Canal, Portee};
use crate::services::cles_mcp::CleMcpAuthentifiee;

/// Les méthodes servies — une énumération **fermée** ; tout autre nom rend
/// `-32601` (E2). Aucune `resources/*`, `logging/*`, `completion/*` ni
/// `tasks/*` (E3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Methode {
    Initialize,
    Ping,
    ToolsList,
    ToolsCall,
    PromptsList,
    PromptsGet,
}

impl Methode {
    fn depuis(nom: &str) -> Option<Self> {
        Some(match nom {
            "initialize" => Self::Initialize,
            "ping" => Self::Ping,
            "tools/list" => Self::ToolsList,
            "tools/call" => Self::ToolsCall,
            "prompts/list" => Self::PromptsList,
            "prompts/get" => Self::PromptsGet,
            _ => return None,
        })
    }
}

fn erreur(id: IdRequete, code: i64, message: &'static str) -> ReponseMcp {
    ReponseMcp::Erreur { id, code, message }
}

fn params_invalides(id: IdRequete) -> ReponseMcp {
    erreur(
        id,
        textes::CODE_PARAMS_INVALIDES,
        textes::TEXTE_PARAMS_INVALIDES,
    )
}

/// Les portées de la clé (canal MCP) ; vide pour tout autre canal.
fn portees(cle: &CleMcpAuthentifiee) -> BTreeSet<Portee> {
    match cle.acces.canal() {
        Canal::Mcp { portees } => portees.clone(),
        Canal::Api { .. } => BTreeSet::new(),
    }
}

/// Répartit une requête JSON-RPC **valide** d'une clé authentifiée.
pub(crate) async fn repartir(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    methode: &str,
    params: Option<serde_json::Value>,
) -> ReponseMcp {
    let Some(methode) = Methode::depuis(methode) else {
        return erreur(
            id,
            textes::CODE_METHODE_INCONNUE,
            textes::MESSAGE_METHODE_INCONNUE,
        );
    };
    if params.as_ref().is_some_and(|p| !p.is_object()) {
        return params_invalides(id);
    }
    let params = params.unwrap_or_else(|| serde_json::json!({}));
    match methode {
        Methode::Initialize => {
            let demandee = params.get("protocolVersion").and_then(|v| v.as_str());
            let version = crate::mcp::VERSIONS_SERVIES
                .iter()
                .find(|v| Some(**v) == demandee)
                .copied()
                .unwrap_or(crate::mcp::VERSIONS_SERVIES[0]);
            ReponseMcp::Resultat {
                id,
                resultat: serde_json::json!({
                    "protocolVersion": version,
                    "capabilities": {
                        "tools": { "listChanged": false },
                        "prompts": { "listChanged": false },
                    },
                    "serverInfo": { "name": "kesh", "version": env!("CARGO_PKG_VERSION") },
                    "instructions": textes::INSTRUCTIONS,
                }),
            }
        }
        Methode::Ping => ReponseMcp::Resultat {
            id,
            resultat: serde_json::json!({}),
        },
        Methode::ToolsList => {
            if params.get("cursor").is_some() {
                return params_invalides(id);
            }
            ReponseMcp::Resultat {
                id,
                resultat: serde_json::json!({
                    "tools": etat.registre.descripteurs_pour(&portees(cle)),
                }),
            }
        }
        Methode::ToolsCall => appeler(etat, cle, id, params).await,
        Methode::PromptsList => ReponseMcp::Resultat {
            id,
            resultat: serde_json::json!({ "prompts": [] }),
        },
        Methode::PromptsGet => erreur(
            id,
            textes::CODE_PARAMS_INVALIDES,
            textes::TEXTE_PROMPT_INCONNU,
        ),
    }
}

/// `tools/call` : outil dans les portées → politique → exécution sous délai →
/// politique (AC-a15, AC-a18, AC-a20).
async fn appeler(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    params: serde_json::Value,
) -> ReponseMcp {
    let Some(nom) = params.get("name").and_then(|v| v.as_str()) else {
        return params_invalides(id);
    };
    let arguments = match params.get("arguments") {
        None => serde_json::json!({}),
        Some(a) if a.is_object() => a.clone(),
        Some(_) => return params_invalides(id),
    };
    let Some(outil) = etat.registre.outil_pour(nom, &portees(cle)) else {
        return erreur(
            id,
            textes::CODE_PARAMS_INVALIDES,
            textes::TEXTE_OUTIL_INCONNU,
        );
    };
    let Ok(autorisation) = politique::autoriser(cle.validation) else {
        return ReponseMcp::OutilEnErreur {
            id,
            texte: textes::TEXTE_VALIDATION_REQUISE,
        };
    };
    // C-32-6-56 (revue P2, H1) : l'outil s'exécute dans une **tâche qui possède**
    // tout ce qu'il emprunte — l'outil, le contexte d'accès, la lecture des
    // outils, donc chaque connexion du pool qu'il prend. Abandonner l'attente
    // (délai de Kesh, ou futur du handler abandonné quand le client se
    // déconnecte) **n'annule pas** la tâche : une instruction en vol garde sa
    // connexion, donc son permis, jusqu'à ce que MariaDB l'ait finie ou
    // interrompue (`max_statement_time`). Le nombre d'instructions simultanées
    // côté serveur reste ainsi borné par les permis du pool
    // (`MCP_POOL_CONNEXIONS`) ; abandonner le futur de l'outil lui-même aurait
    // fermé la connexion et rendu le permis pendant que le serveur poursuivait.
    let acces = cle.acces.clone();
    let lecture = etat.lecture.clone();
    let tache = tokio::spawn(async move {
        let ctx = ContexteMcp::nouveau(&acces, &lecture);
        outil.executer(&ctx, arguments).await
    });
    let issue = tokio::time::timeout(etat.registre.delai(), tache).await;
    match issue {
        Err(_delai) => ReponseMcp::OutilEnErreur {
            id,
            texte: textes::TEXTE_DELAI_DEPASSE,
        },
        Ok(Err(panne)) => {
            // La tâche a paniqué (ou a été annulée par l'arrêt du runtime) :
            // le détail ne va qu'au journal du serveur (E5).
            tracing::error!("outil MCP : la tâche de l'outil a échoué : {panne}");
            ReponseMcp::OutilEnErreur {
                id,
                texte: textes::TEXTE_ERREUR_INTERNE,
            }
        }
        Ok(Ok(Ok(donnees))) => ReponseMcp::Outil {
            id,
            donnees: politique::appliquer(autorisation, donnees),
        },
        Ok(Ok(Err(e))) => match textes::texte_erreur_outil(e) {
            Some(texte) => ReponseMcp::OutilEnErreur { id, texte },
            None => erreur(
                id,
                textes::CODE_PARAMS_INVALIDES,
                textes::TEXTE_OUTIL_INCONNU,
            ),
        },
    }
}
