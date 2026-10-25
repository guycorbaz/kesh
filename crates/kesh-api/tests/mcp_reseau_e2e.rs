//! Story 32-6a (refs #723) — le filtre d'adresse, les mandataires de confiance
//! et `X-Forwarded-For` (AC-a7 à AC-a9, cas (a) à (s)).
//!
//! Par `tower::ServiceExt::oneshot`, le pair simulé par `MockConnectInfo`.
//! « Refusé » s'entend **réponse égale à celle d'une route absente** ;
//! « admis » s'entend **401 sans clé** (puis 200 avec clé, pour quelques cas).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

mod common;

use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use kesh_api::mcp::Registre;
use kesh_api::mcp::textes::TEXTE_NON_AUTHENTIFIE;
use kesh_db::entities::{McpScope, SendValidation};
use sqlx::MySqlPool;

use common::capture_trace::CaptureTrace;
use common::mcp::*;

const ABSENTE: &str = "/mcp-route-absente-326";
const LISTE: &str = r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#;

#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    Admis,
    Refuse,
}
use Verdict::{Admis, Refuse};

fn base(jeton: Option<&str>) -> HeaderMap {
    let mut h = HeaderMap::new();
    h.insert("content-type", HeaderValue::from_static("application/json"));
    if let Some(j) = jeton {
        h.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {j}")).unwrap(),
        );
    }
    h
}

/// La réponse à `pair` pour `en_tetes`, comparée à la route absente.
async fn verdict(pool: &MySqlPool, pair: &str, en_tetes: &[(&'static str, &[u8])]) -> Verdict {
    let mut h = base(None);
    for (n, v) in en_tetes {
        h.append(*n, HeaderValue::from_bytes(v).unwrap());
    }
    let statique = static_dir();
    let app = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        pair,
    );
    let mcp = envoyer(&app, requete("POST", "/mcp", &h, Body::from(LISTE))).await;
    let absente = envoyer(&app, requete("POST", ABSENTE, &h, Body::from(LISTE))).await;
    if mcp == absente {
        return Refuse;
    }
    assert_eq!(
        mcp.statut,
        StatusCode::UNAUTHORIZED,
        "{pair} : ni refusé ni 401"
    );
    assert_eq!(mcp.corps, TEXTE_NON_AUTHENTIFIE.as_bytes());
    Admis
}

const XFF: &str = "x-forwarded-for";

async fn societe_servie(pool: &MySqlPool) -> (i64, i64) {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    (c, u)
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn a_b_c_d_le_pair_et_le_filtre(pool: MySqlPool) {
    let (c, u) = societe_servie(&pool).await;
    assert_eq!(verdict(&pool, "8.8.8.8", &[]).await, Refuse, "(a)");
    assert_eq!(verdict(&pool, "192.168.1.10", &[]).await, Admis, "(b)");
    assert_eq!(
        verdict(&pool, "::ffff:8.8.8.8", &[]).await,
        Refuse,
        "(c) mappée publique"
    );
    assert_eq!(
        verdict(&pool, "::ffff:192.168.1.10", &[]).await,
        Admis,
        "(c) mappée privée"
    );
    reglages(&pool, c, true, false, RESEAUX_DEFAUT, "[]", "[]").await;
    assert_eq!(
        verdict(&pool, "8.8.8.8", &[]).await,
        Admis,
        "(d) filtre désactivé"
    );
    // « Admis » va jusqu'au 200 avec une clé.
    let (_, jeton) = cle(&pool, c, u, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let statique = static_dir();
    let app = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "8.8.8.8",
    );
    let r = envoyer(
        &app,
        requete("POST", "/mcp", &base(Some(&jeton)), Body::from(LISTE)),
    )
    .await;
    assert_eq!(r.statut, StatusCode::OK, "(d) avec clé");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn e_f_x_forwarded_for_d_un_pair_non_declare_est_ignore(pool: MySqlPool) {
    societe_servie(&pool).await;
    assert_eq!(
        verdict(&pool, "8.8.8.8", &[(XFF, b"192.168.1.10")]).await,
        Refuse,
        "(e)"
    );
    assert_eq!(
        verdict(&pool, "192.168.1.10", &[(XFF, b"8.8.8.8")]).await,
        Admis,
        "(f)"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn g_a_l_mandataires_declares(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["172.18.0.2/32"]"#,
        "[]",
    )
    .await;
    let m = "172.18.0.2";
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"192.168.1.20")]).await,
        Admis,
        "(g)"
    );
    assert_eq!(verdict(&pool, m, &[(XFF, b"8.8.8.8")]).await, Refuse, "(h)");
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"192.168.1.20, 8.8.8.8")]).await,
        Refuse,
        "(i)"
    );
    assert_eq!(
        verdict(&pool, m, &[]).await,
        Admis,
        "(k) en-tête absent : le pair"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"unknown")]).await,
        Refuse,
        "(l) unknown"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"8.8.8.8:443")]).await,
        Refuse,
        "(l) port"
    );
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["172.18.0.2/32","172.18.0.3/32"]"#,
        "[]",
    )
    .await;
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"192.168.1.20, 172.18.0.3")]).await,
        Admis,
        "(j)"
    );
    reglages(
        &pool,
        c,
        true,
        true,
        r#"["192.168.0.0/16"]"#,
        r#"["172.18.0.2/32"]"#,
        "[]",
    )
    .await;
    assert_eq!(
        verdict(&pool, m, &[]).await,
        Refuse,
        "(k') le pair, hors des plages"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn m_x_real_ip_et_forwarded_jamais_lus(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    assert_eq!(
        verdict(&pool, "8.8.8.8", &[("x-real-ip", b"192.168.1.10")]).await,
        Refuse,
        "(m) X-Real-IP"
    );
    assert_eq!(
        verdict(&pool, "8.8.8.8", &[("forwarded", b"for=192.168.1.10")]).await,
        Refuse,
        "(m) Forwarded"
    );
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["172.18.0.2/32"]"#,
        "[]",
    )
    .await;
    assert_eq!(
        verdict(
            &pool,
            "172.18.0.2",
            &[(XFF, b"8.8.8.8"), ("x-real-ip", b"192.168.1.10")]
        )
        .await,
        Refuse,
        "(m') X-Real-IP dans la branche mandataire"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn n_o_entrees_stockees_invalides_et_liste_vide(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    reglages(
        &pool,
        c,
        true,
        true,
        r#"["pas-un-réseau","192.168.0.0/16"]"#,
        "[]",
        "[]",
    )
    .await;
    assert_eq!(
        verdict(&pool, "192.168.1.10", &[]).await,
        Admis,
        "(n) entrée ignorée"
    );
    assert_eq!(
        verdict(&pool, "8.8.8.8", &[]).await,
        Refuse,
        "(n) jamais élargissante"
    );
    reglages(&pool, c, true, true, "[]", "[]", "[]").await;
    assert_eq!(
        verdict(&pool, "127.0.0.1", &[]).await,
        Refuse,
        "(o) filtre activé, liste vide : personne"
    );
}

/// (n') — une entrée invalide de `trusted_proxies` écarte la société de S,
/// **quel que soit l'état du filtre** (P7-H3), avec un `error!` ; témoin : sans
/// l'entrée invalide, les deux requêtes sont admises. Avec une seconde société
/// servie, la clé de la société écartée reçoit le 401 fixe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn n_prime_mandataire_invalide_ecarte_la_societe(pool: MySqlPool) {
    let (c, u) = societe_servie(&pool).await;
    let proxies_invalides = r#"["pas-un-mandataire","172.18.0.2"]"#;
    for filtre in [true, false] {
        reglages(
            &pool,
            c,
            true,
            filtre,
            RESEAUX_DEFAUT,
            proxies_invalides,
            "[]",
        )
        .await;
        let capture = CaptureTrace::installer();
        assert_eq!(
            verdict(&pool, "172.18.0.2", &[(XFF, b"192.168.1.20")]).await,
            Refuse,
            "(n') filtre {filtre} : mandataire"
        );
        assert_eq!(
            verdict(&pool, "192.168.1.10", &[]).await,
            Refuse,
            "(n') filtre {filtre} : sans en-tête"
        );
        capture.exiger_ensemble(&[
            "ERROR kesh_api::mcp::reseau",
            "trusted_proxies",
            &format!("company_id={c}"),
        ]);
        capture.exiger_absent("pas-un-mandataire");
        drop(capture);
        reglages(
            &pool,
            c,
            true,
            filtre,
            RESEAUX_DEFAUT,
            r#"["172.18.0.2"]"#,
            "[]",
        )
        .await;
        assert_eq!(
            verdict(&pool, "172.18.0.2", &[(XFF, b"192.168.1.20")]).await,
            Admis,
            "témoin {filtre}"
        );
        assert_eq!(
            verdict(&pool, "192.168.1.10", &[]).await,
            Admis,
            "témoin {filtre}"
        );
    }

    // Deux sociétés : la clé de la société écartée reçoit le 401 fixe.
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        proxies_invalides,
        "[]",
    )
    .await;
    let b_id = societe(&pool, "B").await;
    activer(&pool, b_id).await;
    let (_, jeton) = cle(&pool, c, u, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let statique = static_dir();
    let app = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "192.168.1.10",
    );
    let r = envoyer(
        &app,
        requete("POST", "/mcp", &base(Some(&jeton)), Body::from(LISTE)),
    )
    .await;
    assert_eq!(
        (r.statut, r.corps.as_slice()),
        (StatusCode::UNAUTHORIZED, TEXTE_NON_AUTHENTIFIE.as_bytes())
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn p_q_r_s_lignes_multiples_mappees_non_ascii(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["172.18.0.2/32"]"#,
        "[]",
    )
    .await;
    let m = "172.18.0.2";
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"192.168.1.20"), (XFF, b"8.8.8.8")]).await,
        Refuse,
        "(p) deux lignes : toutes lues"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"8.8.8.8"), (XFF, b"172.18.0.2")]).await,
        Refuse,
        "(p') la dernière ligne seule ne suffit pas"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"::ffff:8.8.8.8")]).await,
        Refuse,
        "(q) publique mappée"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"::ffff:192.168.1.20")]).await,
        Admis,
        "(q) privée mappée"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, "192.168.1.2\u{e9}".as_bytes())]).await,
        Refuse,
        "(r) non ASCII"
    );
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["::ffff:172.18.0.2/128"]"#,
        "[]",
    )
    .await;
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"192.168.1.20")]).await,
        Admis,
        "(s) mandataire stocké mappé"
    );
    assert_eq!(
        verdict(&pool, m, &[(XFF, b"8.8.8.8")]).await,
        Refuse,
        "(s) témoin : il est bien mandataire"
    );
}
