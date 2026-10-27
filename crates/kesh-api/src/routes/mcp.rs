//! Le gestionnaire de `/mcp` — Story 32-6a (refs #723, AC-a1).
//!
//! Monté dans `lib.rs` par `route_service("/mcp", recevoir.with_state(etat))` :
//! un **service**, non un `MethodRouter`, si bien que toute méthode HTTP
//! arrive ici et qu'Axum n'ajoute ni `Allow` ni `content-length` de plus qu'au
//! repli statique (C-32-6-40). Aucun extracteur de corps : la requête entière
//! est remise au transport, qui ne lit le corps qu'après l'authentification.

use std::net::SocketAddr;

use axum::extract::rejection::ExtensionRejection;
use axum::extract::{ConnectInfo, Request, State};
use axum::response::Response;

use crate::mcp::{EtatMcp, reponse, transport};

/// Reçoit toute requête vers `/mcp`. Le pair vient de `ConnectInfo` (avec son
/// repli sur `MockConnectInfo` pour les tests) ; `Err` → route absente.
pub async fn recevoir(
    State(etat): State<EtatMcp>,
    pair: Result<ConnectInfo<SocketAddr>, ExtensionRejection>,
    requete: Request,
) -> Response {
    reponse::rendre(&etat, transport::traiter(&etat, pair, requete).await).await
}
