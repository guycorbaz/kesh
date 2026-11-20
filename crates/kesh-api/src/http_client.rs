//! Client HTTP sortant commun — socle S5 (Story 38-12a), extrait de la source
//! des cours de change (Story 37-2, `exchange_rates/source.rs`).
//!
//! Tout appel de Kesh vers un service extérieur (cours de l'AFC, API PayPal,
//! puis Wise et Stripe) passe par [`build_client`] et [`read_body_bounded`] :
//!
//! - **rustls**, aucun magasin de cookies ;
//! - **aucune redirection suivie** : un jeton d'accès ne part jamais vers un
//!   autre hôte que celui qu'on a appelé ;
//! - **HTTPS seul** hors du bouclage des tests ;
//! - délais de connexion et total, réglables (les tests les raccourcissent) ;
//! - `User-Agent` `Kesh/<version>`, sans donnée personnelle ;
//! - corps de réponse **borné** : `Content-Length` contrôlé s'il est présent,
//!   puis lecture par morceaux bornée (une source n'envoie pas toujours de
//!   `Content-Length`).
//!
//! Le texte d'une `reqwest::Error` porte l'URL : il ne sort jamais tel quel ;
//! l'appelant classe l'erreur (`is_timeout`).

use std::time::Duration;

/// Construit le client sortant : rustls, aucune redirection, délais,
/// `User-Agent` sans donnée personnelle, aucun magasin de cookies.
pub fn build_client(
    https_only: bool,
    connect: Duration,
    total: Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .use_rustls_tls()
        .redirect(reqwest::redirect::Policy::none())
        .https_only(https_only)
        .connect_timeout(connect)
        .timeout(total)
        .user_agent(concat!("Kesh/", env!("CARGO_PKG_VERSION")))
        .build()
}

/// Échec de la lecture bornée d'un corps.
#[derive(Debug)]
pub enum BodyError {
    /// Le corps dépasse la borne (annoncée ou lue).
    TooLarge,
    /// Erreur de transport pendant la lecture (délai compris).
    Transport(reqwest::Error),
}

/// Lit le corps d'une réponse, au plus `max` octets : `Content-Length` refusé
/// d'emblée s'il dépasse, puis lecture par morceaux arrêtée au premier
/// dépassement.
pub async fn read_body_bounded(
    mut response: reqwest::Response,
    max: usize,
) -> Result<Vec<u8>, BodyError> {
    if response.content_length().is_some_and(|n| n > max as u64) {
        return Err(BodyError::TooLarge);
    }
    let mut body: Vec<u8> = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(BodyError::Transport)? {
        if body.len() + chunk.len() > max {
            return Err(BodyError::TooLarge);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}
