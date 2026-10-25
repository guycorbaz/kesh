//! Story 32-6a (refs #723) — l'endpoint `/mcp` : montage, règle « servi »,
//! protocole, clé, politique de validation, outils.
//!
//! Tout passe par `tower::ServiceExt::oneshot` sur le routeur de production
//! (`build_router_avec_registre`), le pair simulé par `MockConnectInfo` : sur
//! une vraie connexion, hyper pose lui-même `content-length: 0` sur une
//! réponse vide, ce qui masquerait la différence que l'AC-a3 éprouve
//! (validation P6, H4).
//!
//! « Égal » veut dire égal octet pour octet : chaque refus est comparé à sa
//! constante (`mcp::textes`) ou à la réponse d'une route absente.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use kesh_api::mcp::Registre;
use kesh_db::entities::{McpScope, SendValidation};
use sqlx::MySqlPool;

use common::mcp::*;

const ABSENTE: &str = "/mcp-route-absente-326";
const TROIS_MIO: usize = 3 * 1024 * 1024;

/// Une requête de la matrice de l'AC-a3.
fn requete_matrice(
    methode: &str,
    chemin: &str,
    jeton: Option<&str>,
    corps: usize,
) -> axum::http::Request<Body> {
    let mut h = HeaderMap::new();
    h.insert("content-type", HeaderValue::from_static("application/json"));
    if let Some(j) = jeton {
        h.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {j}")).unwrap(),
        );
    }
    let corps = if corps == 0 {
        Body::empty()
    } else {
        Body::from(vec![b'{'; corps])
    };
    requete(methode, chemin, &h, corps)
}

// ---------------------------------------------------------------------------
// AC-a3 — « désactivé » est indiscernable d'une route absente
// ---------------------------------------------------------------------------

/// Pour chaque méthode, avec et sans clé valide, avec et sans corps (3 Mio
/// compris), dans trois états — aucune ligne, ligne désactivée, ligne activée
/// dont le filtre refuse le pair —, la réponse de `/mcp` est **égale** à celle
/// d'une route absente : statut, tous les en-têtes hors `date` (dont `Allow` et
/// `content-length`), octets du corps.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn desactive_indiscernable_d_une_route_absente(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (_, jeton) = cle(&pool, c, u, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let statique = static_dir();

    // Témoin du `static_dir` réel : la route absente rend l'index, non un 404.
    let app = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "127.0.0.1",
    );
    let r = envoyer(&app, requete_matrice("GET", ABSENTE, None, 0)).await;
    assert_eq!(
        r.statut,
        StatusCode::OK,
        "le GET d'une route absente rend l'index"
    );
    assert_eq!(r.corps, INDEX_HTML.as_bytes());

    let etats: [(&str, &str); 3] = [
        ("aucune ligne de réglages", "127.0.0.1"),
        ("ligne enabled = FALSE", "127.0.0.1"),
        ("ligne activée, filtre qui refuse le pair", "8.8.8.8"),
    ];
    let mut cas = 0;
    for (i, (etat, pair)) in etats.iter().enumerate() {
        match i {
            0 => {}
            1 => reglages(&pool, c, false, true, RESEAUX_DEFAUT, "[]", "[]").await,
            _ => activer(&pool, c).await,
        }
        let app = routeur(
            pool.clone(),
            config(),
            statique.path(),
            Registre::standard(),
            pair,
        );
        for methode in ["POST", "GET", "HEAD", "DELETE", "OPTIONS"] {
            for j in [None, Some(jeton.as_str())] {
                for taille in [0usize, 64, TROIS_MIO] {
                    let mcp = envoyer(&app, requete_matrice(methode, "/mcp", j, taille)).await;
                    let absente = envoyer(&app, requete_matrice(methode, ABSENTE, j, taille)).await;
                    assert_eq!(
                        mcp,
                        absente,
                        "{etat} : {methode} /mcp (clé : {}, corps : {taille} o) doit égaler la \
                         route absente",
                        j.is_some()
                    );
                    cas += 1;
                }
            }
        }
    }
    assert_eq!(cas, 3 * 5 * 2 * 3, "toute la matrice a tourné");

    // Témoins : servi au pair admis, `/mcp` n'est plus une route absente.
    let app = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "127.0.0.1",
    );
    for taille in [0usize, TROIS_MIO] {
        let r = envoyer(&app, requete_matrice("POST", "/mcp", None, taille)).await;
        assert_eq!(
            r.statut,
            StatusCode::UNAUTHORIZED,
            "sans clé, corps de {taille} o : 401"
        );
        assert_eq!(
            r.corps,
            kesh_api::mcp::textes::TEXTE_NON_AUTHENTIFIE.as_bytes()
        );
        assert_eq!(
            r.en_tete("content-type").as_deref(),
            Some("application/json")
        );
    }
    let r = envoyer(
        &app,
        requete_matrice("POST", "/mcp", Some(&jeton), TROIS_MIO),
    )
    .await;
    assert_eq!(
        r.statut,
        StatusCode::PAYLOAD_TOO_LARGE,
        "clé valide, 3 Mio : 413"
    );
    let r = envoyer(&app, requete_matrice("GET", "/mcp", None, 0)).await;
    assert_eq!(r.statut, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(r.en_tete("allow").as_deref(), Some("POST"));
}
