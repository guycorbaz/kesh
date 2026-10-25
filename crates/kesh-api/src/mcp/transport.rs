//! Le transport (étapes 1 à 10 de l'ordre des contrôles, `mcp/mod.rs`) : rend
//! une [`ReponseMcp`], jamais une erreur vers le framework. Aucune donnée
//! d'outil ne s'y construit.
//!
//! ⛔ Le jeton, l'en-tête `Authorization` et le corps ne sont **jamais**
//! journalisés (test `jeton_jamais_journalise`).

use std::net::SocketAddr;

use axum::extract::rejection::ExtensionRejection;
use axum::extract::{ConnectInfo, Request};
use axum::http::{HeaderMap, Method, header};
use kesh_db::entities::McpServerSettings;

use crate::errors::AppError;
use crate::mcp::reponse::{IdRequete, ReponseMcp};
use crate::mcp::reseau::{self, Decision};
use crate::mcp::{EtatMcp, MCP_CORPS_MAX, VERSIONS_SERVIES, repartiteur};
use crate::services::{cles_mcp, serveur_mcp};

/// `Content-Type` est-il `application/json` (paramètres admis, casse ignorée) ?
fn est_json(en_tetes: &HeaderMap) -> bool {
    en_tetes
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .is_some_and(|t| t.trim().eq_ignore_ascii_case("application/json"))
}

/// Le jeton de `Authorization: Bearer <jeton>` (schéma insensible à la casse) ;
/// `None` pour tout le reste — le cookie de session n'est jamais lu.
fn jeton_bearer(en_tetes: &HeaderMap) -> Option<&str> {
    let mut valeurs = en_tetes.get_all(header::AUTHORIZATION).iter();
    let valeur = valeurs.next()?;
    if valeurs.next().is_some() {
        return None;
    }
    let (schema, jeton) = valeur.to_str().ok()?.split_once(' ')?;
    let jeton = jeton.trim();
    (schema.eq_ignore_ascii_case("bearer") && !jeton.is_empty()).then_some(jeton)
}

/// Le corps a-t-il dépassé la limite (`LengthLimitError` dans la chaîne des
/// causes) ?
fn depasse_la_limite(e: &axum::Error) -> bool {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = std::error::Error::source(e);
    while let Some(c) = cause {
        if c.is::<http_body_util::LengthLimitError>() {
            return true;
        }
        cause = c.source();
    }
    false
}

/// Traite une requête vers `/mcp`.
pub(crate) async fn traiter(
    etat: &EtatMcp,
    pair: Result<ConnectInfo<SocketAddr>, ExtensionRejection>,
    requete: Request,
) -> ReponseMcp {
    // 1. Le pair TCP.
    let Ok(ConnectInfo(pair)) = pair else {
        return ReponseMcp::RouteAbsente(requete);
    };
    let pair = pair.ip().to_canonical();

    // 2. S : les sociétés activées dont le filtre admet ce client.
    let reglages = match serveur_mcp::reglages_actifs(&etat.garde).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("serveur MCP : réglages illisibles, /mcp n'est pas servi : {e}");
            return ReponseMcp::RouteAbsente(requete);
        }
    };
    let servies: Vec<McpServerSettings> = reglages
        .into_iter()
        .filter(|s| reseau::decider(s, pair, requete.headers()) == Decision::Admis)
        .collect();
    if servies.is_empty() {
        return ReponseMcp::RouteAbsente(requete);
    }

    // 3. La méthode.
    if requete.method() != Method::POST {
        return ReponseMcp::MethodeNonPermise;
    }
    let en_tetes = requete.headers();

    // 4. Origin, contre les origines de S.
    if !reseau::origine_admise(
        en_tetes,
        &etat.origines_propres,
        servies.iter().flat_map(|s| s.allowed_origins.iter()),
    ) {
        return ReponseMcp::OrigineRefusee;
    }

    // 5. Content-Type.
    if !est_json(en_tetes) {
        return ReponseMcp::TypeNonAdmis;
    }

    // 6. MCP-Protocol-Version.
    match en_tetes.get("mcp-protocol-version") {
        None => tracing::debug!("serveur MCP : requête sans MCP-Protocol-Version"),
        Some(v) => {
            if !v.to_str().is_ok_and(|v| VERSIONS_SERVIES.contains(&v)) {
                return ReponseMcp::VersionNonServie;
            }
        }
    }

    // 7. La clé.
    let Some(jeton) = jeton_bearer(en_tetes) else {
        return ReponseMcp::NonAuthentifie;
    };
    let cle = match cles_mcp::authentifier(&etat.garde, &etat.trace, jeton).await {
        Ok(cle) => cle,
        Err(AppError::Unauthenticated(_)) => return ReponseMcp::NonAuthentifie,
        Err(e) => {
            tracing::error!("serveur MCP : authentification impossible : {e}");
            return ReponseMcp::ErreurInterne;
        }
    };

    // 8. La société de la clé doit servir ce client.
    let Some(societe) = servies
        .iter()
        .find(|s| s.company_id == cle.acces.company_id())
    else {
        return ReponseMcp::NonAuthentifie;
    };

    // 9. Origin, contre les origines de la société de la clé.
    if !reseau::origine_admise(
        en_tetes,
        &etat.origines_propres,
        societe.allowed_origins.iter(),
    ) {
        return ReponseMcp::OrigineRefusee;
    }

    // 10. Le corps, borné, puis le message JSON-RPC.
    let octets = match axum::body::to_bytes(requete.into_body(), MCP_CORPS_MAX).await {
        Ok(o) => o,
        Err(e) if depasse_la_limite(&e) => return ReponseMcp::CorpsTropGros,
        Err(_) => return ReponseMcp::CorpsIllisible,
    };
    let Ok(message) = serde_json::from_slice::<serde_json::Value>(&octets) else {
        return ReponseMcp::JsonInvalide;
    };
    let Some(objet) = message.as_object() else {
        // Un lot (tableau) ou un scalaire : jamais traité.
        return ReponseMcp::RequeteInvalide;
    };
    if objet.get("jsonrpc").and_then(|v| v.as_str()) != Some("2.0") {
        return ReponseMcp::RequeteInvalide;
    }
    let Some(methode) = objet.get("method") else {
        // Un message de type réponse (`result` ou `error`) : jamais traité.
        return ReponseMcp::RequeteInvalide;
    };
    let Some(methode) = methode.as_str() else {
        return ReponseMcp::RequeteInvalide;
    };
    let Some(id) = objet.get("id") else {
        // Une notification : acceptée, jamais remise au répartiteur.
        return ReponseMcp::Acceptee;
    };
    let Some(id) = IdRequete::depuis(id) else {
        return ReponseMcp::RequeteInvalide;
    };
    repartiteur::repartir(etat, &cle, id, methode, objet.get("params").cloned()).await
}
