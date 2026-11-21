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
//! l'appelant classe l'erreur ([`is_transient_transport`]).
//!
//! S'y ajoutent, pour tous les clients de prestataire (revue P1 de la 38-12a,
//! F2, F5) : [`is_transient_transport`] (une erreur de transport se reprend-elle
//! ?) et [`retry_after`] (l'en-tête `Retry-After` tel que reçu).

use std::io::ErrorKind;
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

/// L'en-tête `Retry-After` d'une réponse, tel que reçu (`None` s'il manque ou
/// n'est pas du texte) ; [`crate::provider_download::pause_for`] le lit.
pub fn retry_after(response: &reqwest::Response) -> Option<String> {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string)
}

/// Une erreur de transport est-elle **passagère** (à reprendre) ? Oui pour un
/// délai dépassé (`is_timeout`) et pour une erreur d'entrée-sortie de
/// connexion — refusée, coupée, interrompue, réseau ou hôte injoignable —
/// trouvée dans la chaîne des causes ([`is_transient_io_chain`]). Non pour le
/// reste, **définitif** : certificat refusé (TLS), hôte inconnu (résolution),
/// schéma refusé (`https_only`), requête mal formée — reprendre six fois ne
/// ferait qu'attendre deux minutes pour dire « réessayez plus tard » d'une
/// faute de configuration (revue P1 de la 38-12a, F2).
///
/// Limite assumée : une panne **passagère** de la résolution de noms se lit
/// comme un hôte inconnu, donc définitive (l'erreur du résolveur ne distingue
/// pas les deux de façon portable) ; relancer le téléchargement suffit.
pub fn is_transient_transport(e: &reqwest::Error) -> bool {
    e.is_timeout() || is_transient_io_chain(e)
}

/// Cherche dans la chaîne des causes de `e` une erreur d'entrée-sortie d'un
/// genre passager (connexion refusée, coupée, interrompue, délai, réseau ou
/// hôte injoignable, fin de flux inattendue), ou une erreur `hyper` de connexion
/// fermée avant la fin du message.
pub fn is_transient_io_chain(e: &(dyn std::error::Error + 'static)) -> bool {
    let mut cause: Option<&(dyn std::error::Error + 'static)> = Some(e);
    while let Some(err) = cause {
        if let Some(io) = err.downcast_ref::<std::io::Error>()
            && matches!(
                io.kind(),
                ErrorKind::ConnectionRefused
                    | ErrorKind::ConnectionReset
                    | ErrorKind::ConnectionAborted
                    | ErrorKind::NotConnected
                    | ErrorKind::BrokenPipe
                    | ErrorKind::TimedOut
                    | ErrorKind::UnexpectedEof
                    | ErrorKind::NetworkUnreachable
                    | ErrorKind::HostUnreachable
                    | ErrorKind::NetworkDown
            )
        {
            return true;
        }
        // Connexion fermée par le serveur après l'envoi de la requête (course
        // du keep-alive) : hyper rend un message incomplet **sans** cause
        // d'entrée-sortie (revue P2, G1).
        if let Some(h) = err.downcast_ref::<hyper::Error>()
            && (h.is_incomplete_message() || h.is_closed() || h.is_canceled())
        {
            return true;
        }
        cause = std::error::Error::source(err);
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Une erreur qui en enveloppe une autre, comme `hyper` enveloppe l'erreur
    /// d'entrée-sortie d'une connexion.
    #[derive(Debug)]
    struct Enveloppe(std::io::Error);

    impl std::fmt::Display for Enveloppe {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("connexion")
        }
    }

    impl std::error::Error for Enveloppe {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[test]
    fn connexion_coupee_passagere_certificat_et_hote_definitifs() {
        for kind in [
            ErrorKind::ConnectionRefused,
            ErrorKind::ConnectionReset,
            ErrorKind::TimedOut,
            ErrorKind::UnexpectedEof,
        ] {
            assert!(is_transient_io_chain(&Enveloppe(kind.into())), "{kind:?}");
        }
        // Certificat refusé (rustls rend une erreur `InvalidData`), hôte
        // inconnu (le résolveur rend une erreur sans genre), autre.
        for kind in [
            ErrorKind::InvalidData,
            ErrorKind::Other,
            ErrorKind::InvalidInput,
        ] {
            assert!(!is_transient_io_chain(&Enveloppe(kind.into())), "{kind:?}");
        }
    }

    /// Bout en bout, sur le seul bouclage : un TLS parlé à un serveur en clair
    /// (comme un certificat refusé) est définitif ; un port fermé est
    /// passager. L'hôte inconnu n'est pas joué ici (il interrogerait le
    /// résolveur de la station) : son erreur, sans genre d'entrée-sortie
    /// passager, est couverte par le test précédent.
    #[tokio::test]
    async fn erreurs_reelles_de_reqwest_classees() {
        let client = build_client(false, Duration::from_secs(2), Duration::from_secs(4)).unwrap();
        // Port fermé : on lie puis on libère un port du bouclage.
        let libre = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let port = libre.local_addr().unwrap().port();
        drop(libre);
        let e = client
            .get(format!("http://127.0.0.1:{port}/"))
            .send()
            .await
            .unwrap_err();
        assert!(is_transient_transport(&e), "port fermé : {e:?}");
        // TLS vers un serveur qui parle en clair.
        let clair = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adresse = clair.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((mut s, _)) = clair.accept().await {
                use tokio::io::AsyncWriteExt;
                let _ = s
                    .write_all(b"HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\n\r\n")
                    .await;
            }
        });
        let e = client
            .get(format!("https://{adresse}/"))
            .send()
            .await
            .unwrap_err();
        assert!(!is_transient_transport(&e), "TLS refusé : {e:?}");
    }

    /// Revue P2, G1 — un serveur qui lit la requête puis ferme sans répondre
    /// (la course classique du keep-alive) rend à hyper une erreur de message
    /// incomplet, **sans** cause d'entrée-sortie : elle est passagère.
    #[tokio::test]
    async fn connexion_fermee_apres_la_requete_est_passagere() {
        let client = build_client(false, Duration::from_secs(2), Duration::from_secs(4)).unwrap();
        let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let adresse = ecoute.local_addr().unwrap();
        tokio::spawn(async move {
            use tokio::io::AsyncReadExt;
            while let Ok((mut flux, _)) = ecoute.accept().await {
                let mut tampon = [0u8; 4096];
                let _ = flux.read(&mut tampon).await;
                drop(flux);
            }
        });
        let e = client
            .get(format!("http://{adresse}/"))
            .send()
            .await
            .unwrap_err();
        assert!(
            is_transient_transport(&e),
            "fermée après la requête : {e:?}"
        );
    }
}
