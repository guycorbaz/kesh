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
use crate::mcp::textes;

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
