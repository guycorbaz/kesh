//! Classement des erreurs de transport du client sortant, **bout en bout sur
//! le bouclage** (Story 38-12a). Ces tests lient des écouteurs : ils vivent ici
//! et non dans `src/`, où la garde `un_seul_ecouteur` n'admet que celui de
//! `main.rs`.

use std::time::Duration;

use kesh_api::http_client::{build_client, is_transient_transport};

/// Bout en bout, sur le seul bouclage : un TLS parlé à un serveur en clair
/// (comme un certificat refusé) est définitif ; un port fermé est
/// passager. L'hôte inconnu n'est pas joué ici (il interrogerait le
/// résolveur de la station) : son erreur, sans genre d'entrée-sortie
/// passager, est couverte par le test unitaire
/// `http_client::tests::connexion_coupee_passagere_certificat_et_hote_definitifs`.
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
