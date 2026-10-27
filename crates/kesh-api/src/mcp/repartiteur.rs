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
//!
//! # Les envois à valider (Story 32-8b, AC-B15)
//!
//! - le nom réservé `kesh_lire_envoi` est résolu **avant** le registre : la
//!   relecture n'est pas un `McpTool`, ne passe pas par la politique (pas de
//!   boucle) et ne fabrique aucun `DonneesEnvoyables` (pas d'exemption) ;
//! - sous une clé `required`, la sortie de l'outil est **déposée** : un dépôt
//!   réussi écrit l'entrée de son appel **dans** sa transaction, et la règle 1
//!   ci-dessus s'y tient — `journaliser_appel` n'est pas appelé en plus ; un
//!   dépôt refusé ou en échec n'a rien écrit, et l'appel est journalisé ici,
//!   après l'échec, comme toute autre issue (une seule entrée par appel).

use std::collections::BTreeSet;
use std::time::Instant;

use serde::Deserialize;

use crate::mcp::EtatMcp;
use crate::mcp::outil::{ContexteMcp, ErreurOutil, MotifArgument, arguments};
use crate::mcp::politique::{self, Sortie};
use crate::mcp::prompts::{self, ListePrompts};
use crate::mcp::registre::NOM_LIRE_ENVOI;
use crate::mcp::reponse::{IdRequete, ReponseEnvoi, ReponseMcp};
use crate::mcp::textes;
use crate::services::acces::{Canal, Portee};
use crate::services::cles_mcp::CleMcpAuthentifiee;
use crate::services::envois_mcp::{self, EnvoiLu, RefusDepot};
use crate::services::prompts_mcp::ErreurPrompts;
use crate::services::serveur_mcp::{self, IssueAppel, OUTIL_INCONNU};

/// Ce qu'un appel laisse au journal (règle 1).
enum Journal {
    /// L'entrée est à écrire par `journaliser_appel`.
    AEcrire {
        /// L'issue.
        issue: IssueAppel,
        /// Un nom du registre, le nom réservé, ou [`OUTIL_INCONNU`].
        outil: &'static str,
        /// Le numéro d'envoi : la relecture (le numéro demandé), sinon `None`.
        envoi: Option<i64>,
    },
    /// L'entrée est déjà écrite, dans la transaction d'un dépôt réussi.
    DejaEcrit,
}

fn a_ecrire(issue: IssueAppel, outil: &'static str, envoi: Option<i64>) -> Journal {
    Journal::AEcrire {
        issue,
        outil,
        envoi,
    }
}

/// Les arguments de `kesh_lire_envoi`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ArgsLireEnvoi {
    envoi: u64,
}

/// Le descripteur **statique** de `kesh_lire_envoi` (AC-B6) : textes du
/// binaire seulement.
fn descripteur_lire_envoi() -> serde_json::Value {
    serde_json::json!({
        "name": NOM_LIRE_ENVOI,
        "description": textes::DESCRIPTION_LIRE_ENVOI,
        "inputSchema": serde_json::from_str::<serde_json::Value>(textes::SCHEMA_LIRE_ENVOI)
            .expect("schéma de kesh_lire_envoi : un texte JSON du binaire"),
        "annotations": { "readOnlyHint": true },
    })
}

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
            // Les outils des portées de la clé, puis `kesh_lire_envoi`, pour
            // toute clé (AC-B6 ; amende l'AC-a17).
            let mut outils = etat.registre.descripteurs_pour(&portees(cle));
            outils.push(descripteur_lire_envoi());
            ReponseMcp::Resultat {
                id,
                resultat: serde_json::json!({ "tools": outils }),
            }
        }
        Methode::ToolsCall => appeler(etat, cle, id, params).await,
        Methode::PromptsList => {
            // La liste n'est pas paginée : un `cursor` est refusé, comme pour
            // `tools/list` (AC-ii5).
            if params.get("cursor").is_some() {
                return params_invalides(id);
            }
            // Hors politique (AC-a15, AC-ii9) : un prompt n'est pas une donnée
            // lue. Une clé sans `books:read` reçoit une liste vide, non une
            // erreur (aucun oracle).
            match prompts::lister(etat, cle).await {
                Ok(liste) => ReponseMcp::ListePrompts { id, liste },
                Err(ErreurPrompts::HorsPortee) => ReponseMcp::ListePrompts {
                    id,
                    liste: ListePrompts::vide(),
                },
                Err(ErreurPrompts::Interne) => erreur_interne(id),
            }
        }
        Methode::PromptsGet => {
            let Some(nom) = params.get("name").and_then(|n| n.as_str()) else {
                return params_invalides(id);
            };
            // Les `arguments` du client sont ignorés, jamais interpolés
            // (AC-ii6, C-32-10a-10). Inconnu, non servi, autre société, nom non
            // conforme, clé sans `books:read` : la même réponse.
            match prompts::obtenir(etat, cle, nom).await {
                Ok(Some(prompt)) => ReponseMcp::PromptRendu { id, prompt },
                Ok(None) | Err(ErreurPrompts::HorsPortee) => erreur(
                    id,
                    textes::CODE_PARAMS_INVALIDES,
                    textes::TEXTE_PROMPT_INCONNU,
                ),
                Err(ErreurPrompts::Interne) => erreur_interne(id),
            }
        }
    }
}

/// L'erreur interne fixe d'une méthode des prompts (le détail est au journal
/// du serveur).
fn erreur_interne(id: IdRequete) -> ReponseMcp {
    erreur(
        id,
        textes::CODE_ERREUR_INTERNE,
        textes::TEXTE_ERREUR_INTERNE,
    )
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
    let (journal, reponse) = traiter_appel(etat, cle, id.clone(), params, debut).await;
    let Journal::AEcrire {
        issue,
        outil,
        envoi,
    } = journal
    else {
        // Un dépôt réussi : l'entrée est écrite dans sa transaction (AC-B5).
        return reponse;
    };
    let duree_ms = u64::try_from(debut.elapsed().as_millis()).unwrap_or(u64::MAX);
    match serveur_mcp::journaliser_appel(&etat.trace, cle, outil, issue, duree_ms, envoi).await {
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
        ErreurOutil::TropVolumineux => IssueAppel::EnvoiTropVolumineux,
        ErreurOutil::LimiteAtteinte => IssueAppel::LimiteAtteinte,
        // Story 32-9a (AC-a22) : les trois erreurs de demande gardent les
        // issues existantes.
        ErreurOutil::ExerciceIntrouvable => IssueAppel::Introuvable,
        ErreurOutil::PeriodeHorsExercice => IssueAppel::ArgumentsInvalides,
        ErreurOutil::CompteNonLettrable => IssueAppel::ArgumentsInvalides,
    }
}

/// L'erreur d'outil d'un dépôt refusé ou en échec (AC-B4, AC-B5).
fn erreur_de_depot(r: RefusDepot) -> ErreurOutil {
    match r {
        RefusDepot::TropVolumineux => ErreurOutil::TropVolumineux,
        RefusDepot::TropNombreux => ErreurOutil::LimiteAtteinte,
        RefusDepot::Interne => ErreurOutil::Interne,
    }
}

/// La réponse d'une erreur d'outil : `isError: true` à texte fixe, ou
/// l'erreur JSON-RPC de l'outil inconnu.
fn reponse_d_erreur(id: IdRequete, e: ErreurOutil) -> ReponseMcp {
    match textes::texte_erreur_outil(e) {
        Some(texte) => ReponseMcp::OutilEnErreur { id, texte },
        None => erreur(
            id,
            textes::CODE_PARAMS_INVALIDES,
            textes::TEXTE_OUTIL_INCONNU,
        ),
    }
}

/// `kesh_lire_envoi` : la relecture d'un envoi, **hors registre** (AC-B6) —
/// ni `autoriser`, ni `Donnees`, ni `DonneesEnvoyables`. Le numéro est un
/// entier de 1 à `i64::MAX` ; tout autre argument rend la même erreur qu'un
/// mauvais type.
async fn relire(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    arguments_recus: serde_json::Value,
) -> (Journal, ReponseMcp) {
    let numero = match arguments::<ArgsLireEnvoi>(arguments_recus) {
        Ok(a) if a.envoi >= 1 => i64::try_from(a.envoi).ok(),
        _ => None,
    };
    let Some(numero) = numero else {
        return (
            a_ecrire(IssueAppel::ArgumentsInvalides, NOM_LIRE_ENVOI, None),
            reponse_d_erreur(
                id,
                ErreurOutil::ArgumentsInvalides {
                    argument: "envoi",
                    motif: MotifArgument::Invalide,
                },
            ),
        );
    };
    let (issue, envoi) = match envois_mcp::lire(cle, &etat.lecture_envois, numero).await {
        Ok(EnvoiLu::Valide(c)) => (IssueAppel::Servi, ReponseEnvoi::Valide(c)),
        Ok(EnvoiLu::EnAttente { numero }) => {
            (IssueAppel::EnvoiEnAttente, ReponseEnvoi::EnAttente(numero))
        }
        Ok(EnvoiLu::Refuse { numero }) => (IssueAppel::EnvoiRefuse, ReponseEnvoi::Refuse(numero)),
        Ok(EnvoiLu::Echu { numero }) => (IssueAppel::EnvoiEchu, ReponseEnvoi::Echu(numero)),
        Ok(EnvoiLu::Inconnu { numero }) => (IssueAppel::Inconnu, ReponseEnvoi::Inconnu(numero)),
        Err(e) => {
            tracing::error!(
                mcp_key_id = cle.cle_id,
                "kesh_lire_envoi : relecture impossible, rien ne part : {e}"
            );
            return (
                a_ecrire(IssueAppel::Erreur, NOM_LIRE_ENVOI, Some(numero)),
                ReponseMcp::OutilEnErreur {
                    id,
                    texte: textes::TEXTE_ERREUR_INTERNE,
                },
            );
        }
    };
    (
        a_ecrire(issue, NOM_LIRE_ENVOI, Some(numero)),
        ReponseMcp::Envoi { id, envoi },
    )
}

/// `tools/call` : nom réservé, ou outil dans les portées → politique →
/// exécution sous délai → politique → envoi ou dépôt (AC-a15, AC-a18, AC-a20 ;
/// AC-B1, AC-B5, AC-B6). Rend ce qu'il reste à journaliser — l'issue et le nom
/// (un nom du registre, le nom réservé ou [`OUTIL_INCONNU`], jamais celui du
/// client), ou rien pour un dépôt réussi — et la réponse.
async fn traiter_appel(
    etat: &EtatMcp,
    cle: &CleMcpAuthentifiee,
    id: IdRequete,
    params: serde_json::Value,
    debut: Instant,
) -> (Journal, ReponseMcp) {
    let Some(nom) = params.get("name").and_then(|v| v.as_str()) else {
        return (
            a_ecrire(IssueAppel::ArgumentsInvalides, OUTIL_INCONNU, None),
            params_invalides(id),
        );
    };
    let arguments = match params.get("arguments") {
        None => serde_json::json!({}),
        Some(a) if a.is_object() => a.clone(),
        Some(_) => {
            return (
                a_ecrire(IssueAppel::ArgumentsInvalides, OUTIL_INCONNU, None),
                params_invalides(id),
            );
        }
    };
    if nom == NOM_LIRE_ENVOI {
        return relire(etat, cle, id, arguments).await;
    }
    let Some(outil) = etat.registre.outil_pour(nom, &portees(cle)) else {
        return (
            a_ecrire(IssueAppel::Inconnu, OUTIL_INCONNU, None),
            erreur(
                id,
                textes::CODE_PARAMS_INVALIDES,
                textes::TEXTE_OUTIL_INCONNU,
            ),
        );
    };
    let nom_outil = outil.nom();
    // Les deux politiques exécutent l'outil ; `autoriser` décide seulement de
    // ce que deviendra sa sortie (AC-B1).
    let autorisation = politique::autoriser(cle.validation);
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
    // Story 32-9a (AC-a7) : la trace de consultation des rapports, possédée
    // par la tâche comme le reste.
    let trace = etat.trace.clone();
    let tache = tokio::spawn(async move {
        let ctx = ContexteMcp::nouveau(&acces, &lecture, &trace);
        outil.executer(&ctx, arguments).await
    });
    let issue = tokio::time::timeout(etat.registre.delai(), tache).await;
    match issue {
        Err(_delai) => (
            a_ecrire(IssueAppel::DelaiDepasse, nom_outil, None),
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
                a_ecrire(IssueAppel::Erreur, nom_outil, None),
                ReponseMcp::OutilEnErreur {
                    id,
                    texte: textes::TEXTE_ERREUR_INTERNE,
                },
            )
        }
        Ok(Ok(Ok(donnees))) => match politique::appliquer(autorisation, donnees) {
            Sortie::Envoyer(donnees) => (
                a_ecrire(IssueAppel::Servi, nom_outil, None),
                ReponseMcp::Outil { id, donnees },
            ),
            Sortie::Deposer(a_valider) => {
                match a_valider
                    .deposer(cle, &etat.ecriture_envois, nom_outil, debut)
                    .await
                {
                    Ok(accuse) => (
                        Journal::DejaEcrit,
                        ReponseMcp::Envoi {
                            id,
                            envoi: ReponseEnvoi::Depose(accuse),
                        },
                    ),
                    Err(refus) => {
                        let e = erreur_de_depot(refus);
                        (
                            a_ecrire(issue_d_erreur(e), nom_outil, None),
                            reponse_d_erreur(id, e),
                        )
                    }
                }
            }
        },
        Ok(Ok(Err(e))) => (
            a_ecrire(issue_d_erreur(e), nom_outil, None),
            reponse_d_erreur(id, e),
        ),
    }
}
