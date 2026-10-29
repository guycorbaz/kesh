//! La seule sortie de `/mcp` : [`rendre`] (AC-a31, C-32-6-37, C-32-6-39).
//!
//! [`ReponseMcp`] est une énumération **propre au MCP** — ni `AppError`, ni
//! `Response` : une erreur de l'API ne peut pas y entrer, et son corps (qui
//! recopie le message d'une `Validation`) ne peut pas sortir par `/mcp`.

use axum::body::Body;
use axum::extract::Request;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::Response;
use tower::ServiceExt;

use crate::mcp::EtatMcp;
use crate::mcp::politique::DonneesEnvoyables;
use crate::mcp::prompts::{AccuseProposition, ListePrompts, PromptRendu};
use crate::mcp::textes;
use crate::services::envois_mcp::{AccuseEnvoi, ContenuValide};

/// Ce qu'un `tools/call` rend d'un envoi à valider (Story 32-8b, AC-B5,
/// AC-B6) : l'accusé d'un dépôt, ou l'issue d'une relecture. Seuls l'entier du
/// numéro, le booléen « modifié » et, pour un envoi validé, le texte validé en
/// sortent.
pub(crate) enum ReponseEnvoi {
    /// Dépôt réussi : un numéro, aucune donnée.
    Depose(AccuseEnvoi),
    /// Relecture : en attente.
    EnAttente(i64),
    /// Relecture : validé — le texte validé part, octet pour octet.
    Valide(ContenuValide),
    /// Relecture : refusé.
    Refuse(i64),
    /// Relecture : échu.
    Echu(i64),
    /// Relecture : inexistant, autre clé ou autre société.
    Inconnu(i64),
}

impl ReponseEnvoi {
    /// Le `result` d'un `tools/call` d'envoi — corps exacts de l'AC-B5 et de
    /// l'AC-B6. **Un seul canal, le texte** (v016-1, C-v016-1-1 ; #811) : les
    /// clés sont exactement `content` et `isError` ; le numéro, l'état et le
    /// booléen « modifié » se lisent dans la phrase de Kesh, le texte validé au
    /// second bloc.
    fn en_resultat(self) -> serde_json::Value {
        let un_bloc = |texte: String, erreur: bool| {
            serde_json::json!({
                "content": [{ "type": "text", "text": texte }],
                "isError": erreur,
            })
        };
        match self {
            Self::Depose(accuse) => un_bloc(textes::texte_envoi_depose(accuse.numero()), false),
            Self::EnAttente(n) => un_bloc(textes::texte_envoi_en_attente(n), false),
            Self::Valide(valide) => serde_json::json!({
                "content": [
                    {
                        "type": "text",
                        "text": textes::texte_envoi_valide(valide.numero(), valide.modifie()),
                    },
                    { "type": "text", "text": valide.texte() },
                ],
                "isError": false,
            }),
            Self::Refuse(n) => un_bloc(textes::texte_envoi_refuse(n), true),
            Self::Echu(n) => un_bloc(textes::texte_envoi_echu(n), true),
            Self::Inconnu(n) => un_bloc(textes::texte_envoi_inconnu(n), true),
        }
    }
}

/// L'`id` d'une requête JSON-RPC : une chaîne ou un nombre, tel que reçu.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct IdRequete(serde_json::Value);

impl IdRequete {
    /// `Some` si `valeur` est une chaîne ou un nombre.
    pub(crate) fn depuis(valeur: &serde_json::Value) -> Option<Self> {
        (valeur.is_string() || valeur.is_number()).then(|| Self(valeur.clone()))
    }
}

/// Ce que `/mcp` répond.
pub(crate) enum ReponseMcp {
    /// `/mcp` n'est pas servi à ce client : la requête **entière**, corps non
    /// lu, est remise au service statique du repli.
    RouteAbsente(Request),
    /// 405, `Allow: POST`, corps vide.
    MethodeNonPermise,
    /// 403, [`textes::TEXTE_ORIGINE_REFUSEE`].
    OrigineRefusee,
    /// 415.
    TypeNonAdmis,
    /// 400, `MCP-Protocol-Version` non servie.
    VersionNonServie,
    /// 401, [`textes::TEXTE_NON_AUTHENTIFIE`].
    NonAuthentifie,
    /// 500, [`textes::CORPS_ERREUR_INTERNE`].
    ErreurInterne,
    /// 413.
    CorpsTropGros,
    /// 400, corps interrompu.
    CorpsIllisible,
    /// 400, `-32700`, sans `id`.
    JsonInvalide,
    /// 400, `-32600`, sans `id`.
    RequeteInvalide,
    /// 202, corps vide (notification).
    Acceptee,
    /// 200, erreur JSON-RPC à l'`id` de la requête.
    Erreur {
        /// L'`id` reçu.
        id: IdRequete,
        /// Code JSON-RPC.
        code: i64,
        /// Message fixe.
        message: &'static str,
    },
    /// 200, `result` statique — construit des textes du binaire et des
    /// descripteurs du registre seulement.
    Resultat {
        /// L'`id` reçu.
        id: IdRequete,
        /// Le résultat.
        resultat: serde_json::Value,
    },
    /// 200, `result` d'un outil servi, tiré de la politique.
    Outil {
        /// L'`id` reçu.
        id: IdRequete,
        /// Les données que la politique laisse partir.
        donnees: DonneesEnvoyables,
    },
    /// 200, `result` d'un envoi à valider : accusé d'un dépôt ou issue d'une
    /// relecture (Story 32-8b).
    Envoi {
        /// L'`id` reçu.
        id: IdRequete,
        /// L'envoi.
        envoi: ReponseEnvoi,
    },
    /// 200, `result` de `prompts/list` — construit d'une [`ListePrompts`], que
    /// seuls des `PromptEnBase` composent (Story 32-10a-ii, AC-ii8).
    ListePrompts {
        /// L'`id` reçu.
        id: IdRequete,
        /// Les prompts servis.
        liste: ListePrompts,
    },
    /// 200, `result` de `prompts/get` — construit d'un [`PromptRendu`]
    /// (Story 32-10a-ii, AC-ii8).
    PromptRendu {
        /// L'`id` reçu.
        id: IdRequete,
        /// Le prompt servi.
        prompt: PromptRendu,
    },
    /// 200, `result` de `proposer_prompt` — l'accusé d'une proposition,
    /// construit d'un [`AccuseProposition`] (Story 32-10a-iii, AC-iii2) :
    /// aucune donnée lue.
    Accuse {
        /// L'`id` reçu.
        id: IdRequete,
        /// L'accusé.
        accuse: AccuseProposition,
    },
    /// 200, `result` d'un outil en erreur (`isError: true`), texte fixe.
    OutilEnErreur {
        /// L'`id` reçu.
        id: IdRequete,
        /// Le texte fixe.
        texte: &'static str,
    },
}

fn brut(statut: StatusCode, type_contenu: &'static str, corps: &'static str) -> Response {
    let mut r = Response::new(Body::from(corps));
    *r.status_mut() = statut;
    r.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static(type_contenu));
    r
}

fn vide(statut: StatusCode) -> Response {
    let mut r = Response::new(Body::empty());
    *r.status_mut() = statut;
    r
}

fn jsonrpc(id: IdRequete, cle: &'static str, valeur: serde_json::Value) -> Response {
    let mut message = serde_json::Map::new();
    message.insert("jsonrpc".into(), serde_json::Value::from("2.0"));
    message.insert("id".into(), id.0);
    message.insert(cle.into(), valeur);
    let octets = serde_json::to_vec(&serde_json::Value::Object(message))
        .expect("une valeur JSON se sérialise toujours");
    let mut r = Response::new(Body::from(octets));
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json"),
    );
    r.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    r
}

const JSON: &str = "application/json";
const TEXTE: &str = "text/plain; charset=utf-8";

/// Produit la réponse HTTP — **seule** fonction qui le fasse pour `/mcp`.
pub(crate) async fn rendre(etat: &EtatMcp, r: ReponseMcp) -> Response {
    match r {
        ReponseMcp::RouteAbsente(requete) => match etat.statique.clone().oneshot(requete).await {
            Ok(reponse) => reponse.map(Body::new),
            Err(jamais) => match jamais {},
        },
        ReponseMcp::MethodeNonPermise => {
            let mut r = vide(StatusCode::METHOD_NOT_ALLOWED);
            r.headers_mut()
                .insert(header::ALLOW, HeaderValue::from_static("POST"));
            r
        }
        ReponseMcp::OrigineRefusee => {
            brut(StatusCode::FORBIDDEN, JSON, textes::TEXTE_ORIGINE_REFUSEE)
        }
        ReponseMcp::TypeNonAdmis => brut(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            TEXTE,
            textes::CORPS_TYPE_NON_ADMIS,
        ),
        ReponseMcp::VersionNonServie => brut(
            StatusCode::BAD_REQUEST,
            TEXTE,
            textes::CORPS_VERSION_NON_SERVIE,
        ),
        ReponseMcp::NonAuthentifie => brut(
            StatusCode::UNAUTHORIZED,
            JSON,
            textes::TEXTE_NON_AUTHENTIFIE,
        ),
        ReponseMcp::ErreurInterne => brut(
            StatusCode::INTERNAL_SERVER_ERROR,
            JSON,
            textes::CORPS_ERREUR_INTERNE,
        ),
        ReponseMcp::CorpsTropGros => brut(
            StatusCode::PAYLOAD_TOO_LARGE,
            TEXTE,
            textes::CORPS_TROP_GROS,
        ),
        ReponseMcp::CorpsIllisible => brut(StatusCode::BAD_REQUEST, TEXTE, textes::CORPS_ILLISIBLE),
        ReponseMcp::JsonInvalide => {
            brut(StatusCode::BAD_REQUEST, JSON, textes::CORPS_JSON_INVALIDE)
        }
        ReponseMcp::RequeteInvalide => brut(
            StatusCode::BAD_REQUEST,
            JSON,
            textes::CORPS_REQUETE_INVALIDE,
        ),
        ReponseMcp::Acceptee => vide(StatusCode::ACCEPTED),
        ReponseMcp::Erreur { id, code, message } => jsonrpc(
            id,
            "error",
            serde_json::json!({ "code": code, "message": message }),
        ),
        ReponseMcp::Resultat { id, resultat } => jsonrpc(id, "result", resultat),
        ReponseMcp::Outil { id, donnees } => jsonrpc(id, "result", donnees.en_resultat()),
        ReponseMcp::Envoi { id, envoi } => jsonrpc(id, "result", envoi.en_resultat()),
        ReponseMcp::ListePrompts { id, liste } => jsonrpc(id, "result", liste.en_resultat()),
        ReponseMcp::PromptRendu { id, prompt } => jsonrpc(id, "result", prompt.en_resultat()),
        ReponseMcp::Accuse { id, accuse } => jsonrpc(id, "result", accuse.en_resultat()),
        ReponseMcp::OutilEnErreur { id, texte } => jsonrpc(
            id,
            "result",
            serde_json::json!({
                "content": [{ "type": "text", "text": texte }],
                "isError": true,
            }),
        ),
    }
}
