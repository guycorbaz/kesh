//! Le répartiteur JSON-RPC (AC-a12 à AC-a22) : la liste **close** des méthodes
//! servies, et le **seul** appelant de `McpTool::executer`, des points d'accès
//! du registre et de la politique.
//!
//! # Le journal des appels (Story 32-6b, C-32-6-58)
//!
//! Chaque `tools/call` — demande mal formée comprise — est traité dans une
//! **tâche de l'appel** (`tokio::spawn`) qui va de la résolution de l'outil à
//! l'écriture de son entrée au journal d'audit. La règle :
//!
//! 1. l'entrée s'écrit **dans cette tâche**, une fois, quand l'issue est connue
//!    (délai dépassé et panic de l'outil compris), **avant** que la réponse ne
//!    soit rendue au gestionnaire ;
//! 2. une réponse ne part que si l'écriture a réussi ; sinon, elle devient
//!    `isError: true` à texte fixe, sans donnée (fail closed, C-32-6-30) ;
//! 3. le gestionnaire n'attend que la tâche : si le client abandonne la
//!    requête, la tâche va au bout et **journalise quand même** l'appel, avec
//!    son issue réelle — la réponse est jetée ;
//! 4. si la tâche de l'appel elle-même échoue (panic hors de l'outil, arrêt du
//!    runtime), aucune donnée ne part et l'appel n'a pas d'entrée : le journal
//!    du serveur seul en garde la trace.

use std::collections::BTreeSet;
use std::time::Instant;

use crate::mcp::EtatMcp;
use crate::mcp::outil::{ContexteMcp, ErreurOutil};
use crate::mcp::politique;
use crate::mcp::reponse::{IdRequete, ReponseMcp};
use crate::mcp::textes;
use crate::services::acces::{Canal, Portee};
use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::serveur_mcp::{self, AppelOutil, IssueAppel, OUTIL_INCONNU};

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
    let params = match params {
        None => serde_json::json!({}),
        Some(p) if p.is_object() => p,
        // Un `tools/call` mal formé est journalisé (AC-a19, AC-b1) : il va à
        // `appeler`, qui le refuse de la même réponse.
        Some(_) if methode == Methode::ToolsCall => serde_json::Value::Null,
        Some(_) => return params_invalides(id),
    };
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

/// `tools/call` : la tâche de l'appel, attendue par le gestionnaire (règle du
/// journal en tête du module, C-32-6-58).
async fn appeler(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    params: serde_json::Value,
) -> ReponseMcp {
    let (etat_appel, cle_appel, id_appel) = (etat.clone(), cle.clone(), id.clone());
    let appel = tokio::spawn(async move {
        appeler_et_journaliser(&etat_appel, &cle_appel, id_appel, params).await
    });
    match appel.await {
        Ok(reponse) => reponse,
        Err(panne) => {
            // Règle 4 : aucune donnée, aucune entrée ; le détail au journal du
            // serveur seulement (E5).
            tracing::error!("outil MCP : la tâche de l'appel a échoué : {panne}");
            ReponseMcp::OutilEnErreur {
                id,
                texte: textes::TEXTE_ERREUR_INTERNE,
            }
        }
    }
}

/// Traite l'appel, puis écrit son entrée au journal ; la réponse ne sort que
/// si l'entrée est écrite (règles 1 et 2, C-32-6-58 ; C-32-6-30).
async fn appeler_et_journaliser(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    params: serde_json::Value,
) -> ReponseMcp {
    let debut = Instant::now();
    let (issue, outil, reponse) = traiter_appel(etat, cle, id.clone(), params).await;
    let appel = AppelOutil {
        cle_id: cle.cle_id,
        createur: cle.acces.user_id(),
        outil,
        issue,
        duree_ms: u64::try_from(debut.elapsed().as_millis()).unwrap_or(u64::MAX),
        validation: cle.validation,
    };
    match serveur_mcp::journaliser_appel(&etat.trace, &appel).await {
        Ok(()) => reponse,
        Err(e) => {
            tracing::error!(
                mcp_key_id = cle.cle_id,
                "outil MCP : journal de l'appel non écrit, aucune donnée ne part : {e}"
            );
            ReponseMcp::OutilEnErreur {
                id,
                texte: textes::TEXTE_ERREUR_INTERNE,
            }
        }
    }
}

/// L'issue au journal d'une erreur d'outil.
fn issue_d_erreur(e: ErreurOutil) -> IssueAppel {
    match e {
        ErreurOutil::ArgumentsInvalides { .. } => IssueAppel::ArgumentsInvalides,
        ErreurOutil::Inconnu => IssueAppel::Inconnu,
        ErreurOutil::Introuvable => IssueAppel::Introuvable,
        ErreurOutil::Interne => IssueAppel::Erreur,
    }
}

/// `tools/call` : outil dans les portées → politique → exécution sous délai →
/// politique (AC-a15, AC-a18, AC-a20). Rend l'issue, le nom à journaliser — un
/// nom du registre ou [`OUTIL_INCONNU`], jamais celui du client — et la
/// réponse.
async fn traiter_appel(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    params: serde_json::Value,
) -> (IssueAppel, &'static str, ReponseMcp) {
    let Some(nom) = params.get("name").and_then(|v| v.as_str()) else {
        return (
            IssueAppel::ArgumentsInvalides,
            OUTIL_INCONNU,
            params_invalides(id),
        );
    };
    let arguments = match params.get("arguments") {
        None => serde_json::json!({}),
        Some(a) if a.is_object() => a.clone(),
        Some(_) => {
            return (
                IssueAppel::ArgumentsInvalides,
                OUTIL_INCONNU,
                params_invalides(id),
            );
        }
    };
    let Some(outil) = etat.registre.outil_pour(nom, &portees(cle)) else {
        return (
            IssueAppel::Inconnu,
            OUTIL_INCONNU,
            erreur(
                id,
                textes::CODE_PARAMS_INVALIDES,
                textes::TEXTE_OUTIL_INCONNU,
            ),
        );
    };
    let nom_outil = outil.nom();
    let Ok(autorisation) = politique::autoriser(cle.validation) else {
        return (
            IssueAppel::ValidationRequise,
            nom_outil,
            ReponseMcp::OutilEnErreur {
                id,
                texte: textes::TEXTE_VALIDATION_REQUISE,
            },
        );
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
    let (issue, reponse) = match issue {
        Err(_delai) => (
            IssueAppel::DelaiDepasse,
            ReponseMcp::OutilEnErreur {
                id,
                texte: textes::TEXTE_DELAI_DEPASSE,
            },
        ),
        Ok(Err(panne)) => {
            // La tâche a paniqué (ou a été annulée par l'arrêt du runtime) :
            // le détail ne va qu'au journal du serveur (E5).
            tracing::error!("outil MCP : la tâche de l'outil a échoué : {panne}");
            (
                IssueAppel::Erreur,
                ReponseMcp::OutilEnErreur {
                    id,
                    texte: textes::TEXTE_ERREUR_INTERNE,
                },
            )
        }
        Ok(Ok(Ok(donnees))) => (
            IssueAppel::Servi,
            ReponseMcp::Outil {
                id,
                donnees: politique::appliquer(autorisation, donnees),
            },
        ),
        Ok(Ok(Err(e))) => (
            issue_d_erreur(e),
            match textes::texte_erreur_outil(e) {
                Some(texte) => ReponseMcp::OutilEnErreur { id, texte },
                None => erreur(
                    id,
                    textes::CODE_PARAMS_INVALIDES,
                    textes::TEXTE_OUTIL_INCONNU,
                ),
            },
        ),
    };
    (issue, nom_outil, reponse)
}
