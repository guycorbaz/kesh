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

// ---------------------------------------------------------------------------
// Story 32-6c — le journal du filtre d'adresse (AC-T1 à AC-T5)
// ---------------------------------------------------------------------------

/// Le fragment qui distingue un évènement de décision (le `debug!` d'une
/// entrée ignorée de `allowed_networks` n'en est pas un).
const DECISION: &str = "décision du filtre d'adresse";

/// Un mandataire déclaré, dans `172.16.0.0/12` (plages par défaut).
const MANDATAIRE: &str = r#"["172.18.0.1"]"#;

/// Les évènements de décision de la capture, dans l'ordre.
fn decisions(capture: &CaptureTrace) -> Vec<String> {
    capture
        .evenements()
        .into_iter()
        .filter(|e| e.contains(DECISION))
        .collect()
}

/// Un évènement de décision attendu — le message d'abord, puis les onze champs
/// dans l'ordre de l'AC-1 ; comparé par **égalité de chaîne complète**.
#[derive(Clone, Copy)]
struct Attendu {
    niveau: &'static str,
    company_id: i64,
    filtre: bool,
    plages: usize,
    pair: &'static str,
    pair_mandataire: bool,
    xff_lignes: usize,
    xff_entrees: usize,
    x_real_ip: bool,
    forwarded: bool,
    client: &'static str,
    verdict: &'static str,
}

impl Attendu {
    /// Un refus en `WARN` : filtre activé, sept plages, aucun en-tête de
    /// mandataire, le client étant le pair.
    fn refus(company_id: i64, pair: &'static str) -> Self {
        Self {
            niveau: "WARN",
            company_id,
            filtre: true,
            plages: 7,
            pair,
            pair_mandataire: false,
            xff_lignes: 0,
            xff_entrees: 0,
            x_real_ip: false,
            forwarded: false,
            client: pair,
            verdict: "refuse",
        }
    }

    /// Le même évènement, en admission (`DEBUG`).
    fn admis(self) -> Self {
        Self {
            niveau: "DEBUG",
            verdict: "admis",
            ..self
        }
    }

    fn texte(&self) -> String {
        format!(
            "{} kesh_api::mcp::reseau message=serveur MCP : {DECISION} company_id={} filtre={} \
             plages={} pair={} pair_mandataire={} xff_lignes={} xff_entrees={} x_real_ip={} \
             forwarded={} client={} verdict={}",
            self.niveau,
            self.company_id,
            self.filtre,
            self.plages,
            self.pair,
            self.pair_mandataire,
            self.xff_lignes,
            self.xff_entrees,
            self.x_real_ip,
            self.forwarded,
            self.client,
            self.verdict,
        )
    }
}

/// Un cas de l'AC-T1 : les réglages, la requête, la réponse et l'évènement.
struct Cas {
    n: u8,
    filtre: bool,
    reseaux: &'static str,
    mandataires: &'static str,
    pair: &'static str,
    en_tetes: Vec<(&'static str, &'static [u8])>,
    verdict: Verdict,
    attendu: Attendu,
}

/// AC-T1 — un évènement, et un seul, par décision ; ses onze champs, son
/// niveau, et la réponse inchangée (refusé ⇔ route absente).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn decision_journalisee_par_cas(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    let r = |pair| Attendu::refus(c, pair);
    let cas = vec![
        Cas {
            n: 1,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "198.51.100.7",
            en_tetes: vec![],
            verdict: Refuse,
            attendu: r("198.51.100.7"),
        },
        Cas {
            n: 2,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "192.168.1.20",
            en_tetes: vec![],
            verdict: Admis,
            attendu: r("192.168.1.20").admis(),
        },
        Cas {
            n: 3,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"198.51.100.7")],
            verdict: Refuse,
            attendu: Attendu {
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 1,
                client: "198.51.100.7",
                ..r("172.18.0.1")
            },
        },
        Cas {
            n: 4,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"192.168.1.20"), ("x-real-ip", b"198.51.100.7")],
            verdict: Admis,
            attendu: Attendu {
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 1,
                x_real_ip: true,
                client: "192.168.1.20",
                ..r("172.18.0.1")
            }
            .admis(),
        },
        // Le cas DSM non déclaré : l'en-tête est posé, ignoré ; le pair est
        // le client, admis par les plages par défaut.
        Cas {
            n: 5,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "172.18.0.1",
            en_tetes: vec![
                (XFF, b"198.51.100.7, 192.0.2.10"),
                ("forwarded", b"for=192.0.2.10"),
            ],
            verdict: Admis,
            attendu: Attendu {
                xff_lignes: 1,
                xff_entrees: 2,
                forwarded: true,
                ..r("172.18.0.1")
            }
            .admis(),
        },
        Cas {
            n: 6,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"unknown")],
            verdict: Refuse,
            attendu: Attendu {
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 1,
                client: "aucun",
                ..r("172.18.0.1")
            },
        },
        Cas {
            n: 7,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"192.0.2.10, 192.168.1.20"), (XFF, b"198.51.100.7")],
            verdict: Refuse,
            attendu: Attendu {
                pair_mandataire: true,
                xff_lignes: 2,
                xff_entrees: 3,
                client: "198.51.100.7",
                ..r("172.18.0.1")
            },
        },
        Cas {
            n: 8,
            filtre: false,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "198.51.100.7",
            en_tetes: vec![],
            verdict: Admis,
            attendu: Attendu {
                filtre: false,
                ..r("198.51.100.7")
            }
            .admis(),
        },
        // Filtre coupé, en-tête illisible : le client est `aucun`, la
        // décision reste `Admis` (M15).
        Cas {
            n: 9,
            filtre: false,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"unknown")],
            verdict: Admis,
            attendu: Attendu {
                filtre: false,
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 1,
                client: "aucun",
                ..r("172.18.0.1")
            }
            .admis(),
        },
        Cas {
            n: 10,
            filtre: true,
            reseaux: "[]",
            mandataires: "[]",
            pair: "192.168.1.20",
            en_tetes: vec![],
            verdict: Refuse,
            attendu: Attendu {
                plages: 0,
                ..r("192.168.1.20")
            },
        },
        Cas {
            n: 11,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "2001:db8::7",
            en_tetes: vec![],
            verdict: Refuse,
            attendu: r("2001:db8::7"),
        },
        // Le pair est normalisé : l'IPv4 mappée s'écrit en IPv4.
        Cas {
            n: 12,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: "[]",
            pair: "::ffff:198.51.100.7",
            en_tetes: vec![],
            verdict: Refuse,
            attendu: r("198.51.100.7"),
        },
        // `plages` compte l'entrée invalide ; la ligne non ASCII se compte sur
        // ses octets.
        Cas {
            n: 13,
            filtre: true,
            reseaux: r#"["pas-un-réseau","192.0.2.0/24"]"#,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b"192.0.2.10, \xe9")],
            verdict: Refuse,
            attendu: Attendu {
                plages: 2,
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 2,
                client: "aucun",
                ..r("172.18.0.1")
            },
        },
        // Une ligne vide compte un morceau ; une virgule finale en ouvre un.
        Cas {
            n: 14,
            filtre: true,
            reseaux: RESEAUX_DEFAUT,
            mandataires: MANDATAIRE,
            pair: "172.18.0.1",
            en_tetes: vec![(XFF, b""), (XFF, b"192.0.2.10,")],
            verdict: Refuse,
            attendu: Attendu {
                pair_mandataire: true,
                xff_lignes: 2,
                xff_entrees: 3,
                client: "aucun",
                ..r("172.18.0.1")
            },
        },
    ];
    assert_eq!(cas.len(), 14, "quatorze cas");
    for k in cas {
        reglages(&pool, c, true, k.filtre, k.reseaux, k.mandataires, "[]").await;
        let capture = CaptureTrace::installer();
        assert_eq!(
            verdict(&pool, k.pair, &k.en_tetes).await,
            k.verdict,
            "cas {} : réponse",
            k.n
        );
        assert_eq!(
            decisions(&capture),
            vec![k.attendu.texte()],
            "cas {} : évènement de décision",
            k.n
        );
    }
}

/// AC-T2 — l'issue `Ecartee` n'a pas d'évènement de décision ; témoin : son
/// `error!` est dans la même capture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ecartee_sans_evenement_de_decision(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    reglages(
        &pool,
        c,
        true,
        true,
        RESEAUX_DEFAUT,
        r#"["pas-un-mandataire"]"#,
        "[]",
    )
    .await;
    let capture = CaptureTrace::installer();
    assert_eq!(verdict(&pool, "192.168.1.20", &[]).await, Refuse);
    capture.exiger_ensemble(&[
        "ERROR kesh_api::mcp::reseau",
        "trusted_proxies",
        &format!("company_id={c}"),
    ]);
    assert_eq!(decisions(&capture), Vec::<String>::new());
}

/// AC-T3 — sans société activée, aucune décision, aucun évènement ; témoin :
/// la société activée, la même requête produit l'évènement attendu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn serveur_desactive_aucun_evenement(pool: MySqlPool) {
    let (c, _) = livres(&pool).await;
    let capture = CaptureTrace::installer();
    assert_eq!(
        verdict(&pool, "192.168.1.20", &[]).await,
        Refuse,
        "aucune ligne de réglages"
    );
    reglages(&pool, c, false, true, RESEAUX_DEFAUT, "[]", "[]").await;
    assert_eq!(
        verdict(&pool, "192.168.1.20", &[]).await,
        Refuse,
        "serveur désactivé"
    );
    assert_eq!(decisions(&capture), Vec::<String>::new());
    activer(&pool, c).await;
    assert_eq!(verdict(&pool, "192.168.1.20", &[]).await, Admis, "témoin");
    assert_eq!(
        decisions(&capture),
        vec![Attendu::refus(c, "192.168.1.20").admis().texte()]
    );
}

/// AC-T4 — ni clé, ni en-tête reçu, ni corps au journal : deux requêtes
/// piégées, l'une admise (jusqu'à l'authentification), l'autre refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn journal_du_filtre_sans_secret(pool: MySqlPool) {
    let (c, _) = societe_servie(&pool).await;
    reglages(&pool, c, true, true, RESEAUX_DEFAUT, MANDATAIRE, "[]").await;
    const CORPS: &str = r#"{"PIEGE_CORPS_326C":"PIEGE_CORPS_326C"}"#;
    let pieges = |refusee: bool| {
        let mut h = base(Some("kesh_mcp_PIEGE_JETON_326C"));
        h.insert(
            "cookie",
            HeaderValue::from_static("kesh_session=PIEGE_COOKIE_326C"),
        );
        h.insert("x-real-ip", HeaderValue::from_static("PIEGE_XRI_326C"));
        h.insert("forwarded", HeaderValue::from_static("for=PIEGE_FWD_326C"));
        if refusee {
            h.insert(XFF, HeaderValue::from_static("PIEGE_XFF_326C"));
            h.insert(
                "origin",
                HeaderValue::from_static("https://piege-origine-326c.example"),
            );
        }
        h
    };
    let statique = static_dir();
    let capture = CaptureTrace::installer();

    let admise = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "192.168.1.20",
    );
    let r = envoyer(
        &admise,
        requete("POST", "/mcp", &pieges(false), Body::from(CORPS)),
    )
    .await;
    assert_eq!(
        (r.statut, r.corps.as_slice()),
        (StatusCode::UNAUTHORIZED, TEXTE_NON_AUTHENTIFIE.as_bytes()),
        "admise : jusqu'à l'authentification"
    );

    let refusee = routeur(
        pool.clone(),
        config(),
        statique.path(),
        Registre::standard(),
        "172.18.0.1",
    );
    let mcp = envoyer(
        &refusee,
        requete("POST", "/mcp", &pieges(true), Body::from(CORPS)),
    )
    .await;
    let absente = envoyer(
        &refusee,
        requete("POST", ABSENTE, &pieges(true), Body::from(CORPS)),
    )
    .await;
    assert_eq!(mcp, absente, "refusée : route absente");

    // Témoin : les deux évènements de décision sont dans la capture.
    let tous = Attendu {
        x_real_ip: true,
        forwarded: true,
        ..Attendu::refus(c, "192.168.1.20")
    };
    assert_eq!(
        decisions(&capture),
        vec![
            tous.admis().texte(),
            Attendu {
                pair: "172.18.0.1",
                pair_mandataire: true,
                xff_lignes: 1,
                xff_entrees: 1,
                client: "aucun",
                ..tous
            }
            .texte(),
        ]
    );
    for piege in [
        "PIEGE_JETON_326C",
        "PIEGE_COOKIE_326C",
        "PIEGE_XFF_326C",
        "PIEGE_XRI_326C",
        "PIEGE_FWD_326C",
        "PIEGE_CORPS_326C",
        "piege-origine-326c",
        "kesh_mcp_",
    ] {
        capture.exiger_absent(piege);
    }
}

/// AC-T5 — deux sociétés activées, deux évènements, dans l'ordre des
/// `company_id` (`list_enabled` : `ORDER BY company_id`) : un refus pour l'une
/// n'empêche pas l'autre de servir.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deux_societes_deux_evenements(pool: MySqlPool) {
    let a = societe(&pool, "A").await;
    let b = societe(&pool, "B").await;
    assert!(a < b, "identifiants croissants");
    activer(&pool, a).await;
    reglages(&pool, b, true, true, r#"["192.0.2.0/24"]"#, "[]", "[]").await;
    let capture = CaptureTrace::installer();
    assert_eq!(
        verdict(&pool, "192.0.2.10", &[]).await,
        Admis,
        "servi par b"
    );
    assert_eq!(
        decisions(&capture),
        vec![
            Attendu::refus(a, "192.0.2.10").texte(),
            Attendu {
                plages: 1,
                ..Attendu::refus(b, "192.0.2.10")
            }
            .admis()
            .texte(),
        ]
    );
}
