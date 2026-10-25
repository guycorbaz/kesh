//! Story 32-6a (refs #723) — les gardes lexicales du serveur MCP (AC-a23,
//! AC-a24, AC-a27, AC-a30, AC-a31).
//!
//! **Règle commune** (validation P3 de la 32-6) : chaque garde dit sa
//! **propriété**, son **périmètre** et les **formes** qu'elle attrape ; son
//! motif a été greppé sur le dépôt réel à l'étape 0 du développement et rend
//! **vide** hors des sites écrits en liste blanche (fichier, ligne réduite de
//! ses blancs) ; chaque garde a un **témoin rouge** (une forme qu'elle doit
//! attraper) et un **témoin vert** (une forme légitime, existante au dépôt
//! quand il y en a une, qu'elle ne doit pas attraper), sur source factice, par
//! la même fonction pure que la garde.
//!
//! ⚠️ **Ce que ces gardes ne voient pas** : une macro qui fabriquerait un
//! identifiant, un alias de chemin (`use … as`) hors des formes refusées, une
//! forme coupée sur deux lignes, un SQL composé à l'exécution ou émis par une
//! dépendance. La revue les lit.

use std::path::{Path, PathBuf};

use regex::Regex;

// ---------------------------------------------------------------------------
// Lecture des sources
// ---------------------------------------------------------------------------

fn racine_crate() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn racine_src() -> PathBuf {
    racine_crate().join("src")
}

fn racine_crates() -> PathBuf {
    racine_crate().parent().unwrap().to_path_buf()
}

fn fichiers(dir: &Path, extensions: &[&str], v: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).expect("répertoire lisible") {
        let p = e.unwrap().path();
        if p.is_dir() {
            if p.file_name()
                .is_some_and(|n| n == "target" || n == "node_modules")
            {
                continue;
            }
            fichiers(&p, extensions, v);
        } else if p
            .extension()
            .is_some_and(|x| extensions.iter().any(|e| x == *e))
        {
            v.push(p);
        }
    }
}

/// Les `.rs` de `kesh-api/src` : (chemin relatif à `src/`, texte).
fn sources_src() -> Vec<(String, String)> {
    let mut v = Vec::new();
    fichiers(&racine_src(), &["rs"], &mut v);
    v.sort();
    let s: Vec<(String, String)> = v
        .into_iter()
        .map(|p| {
            let rel = p
                .strip_prefix(racine_src())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            (rel, std::fs::read_to_string(&p).unwrap())
        })
        .collect();
    assert!(
        s.len() >= 100,
        "balayage de src/ : {} fichiers, plancher 100",
        s.len()
    );
    assert!(
        s.iter().any(|(f, _)| f == "mcp/transport.rs"),
        "src/mcp/ lu"
    );
    s
}

/// Ligne de commentaire : son premier caractère non blanc ouvre `//`.
fn est_commentaire(l: &str) -> bool {
    l.trim_start().starts_with("//")
}

/// Les couples (fichier, ligne réduite) des lignes de code qui satisfont
/// `motif`, sur les sources retenues par `garder`.
fn sites(
    sources: &[(String, String)],
    garder: impl Fn(&str) -> bool,
    motif: &Regex,
) -> Vec<(String, String)> {
    let mut v = Vec::new();
    for (f, t) in sources.iter().filter(|(f, _)| garder(f)) {
        for l in t.lines().filter(|l| !est_commentaire(l)) {
            if motif.is_match(l) {
                v.push((f.clone(), l.trim().to_string()));
            }
        }
    }
    v.sort();
    v
}

/// La même chose, sur une source factice (témoins).
fn sites_factices(source: &str, motif: &Regex) -> usize {
    source
        .lines()
        .filter(|l| !est_commentaire(l) && motif.is_match(l))
        .count()
}

fn liste(v: &[(&str, &str)]) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = v
        .iter()
        .map(|(f, l)| (f.to_string(), l.to_string()))
        .collect();
    v.sort();
    v
}

fn re(s: &str) -> Regex {
    Regex::new(s).unwrap()
}

fn tout(_: &str) -> bool {
    true
}

fn dans_mcp(f: &str) -> bool {
    f.starts_with("mcp/") || f == "routes/mcp.rs"
}

// ---------------------------------------------------------------------------
// AC-a30 — un_seul_point_d_entree (N1)
// ---------------------------------------------------------------------------

/// `crate::mcp`, `super::mcp`, `use crate::mcp`, et `mcp::` **en tête de
/// chemin** (non précédé d'un identifiant ni de `::`), et la forme qualifiée par
/// le nom du crate, `kesh_api::mcp` (revue P1, E6 : `main.rs` est un autre
/// crate et la nommerait ainsi). Ne voit pas `ContexteAcces::mcp(`,
/// `portee_mcp`, `cles_mcp::`, `serveur_mcp::`.
const MOTIF_MODULE_MCP: &str =
    r"\bcrate\s*::\s*mcp\b|\bsuper\s*::\s*mcp\b|\bkesh_api\s*::\s*mcp\b|(^|[^A-Za-z0-9_:])mcp\s*::";

/// Les lignes de `lib.rs` qui nomment le module (déclaration et câblage).
const LIB_NOMME_MCP: &[(&str, &str)] = &[
    (
        "lib.rs",
        "build_router_avec_registre(state, static_dir, mcp::Registre::standard())",
    ),
    (
        "lib.rs",
        "fn service_statique(static_dir: &str) -> mcp::ServiceStatique {",
    ),
    ("lib.rs", "registre: mcp::Registre,"),
    (
        "lib.rs",
        "kesh_db::pool::pool_lecture_seule(&state.pool, mcp::MCP_POOL_CONNEXIONS, delai);",
    ),
    (
        "lib.rs",
        "mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils.clone()));",
    ),
    ("lib.rs", "let etat_mcp = mcp::EtatMcp {"),
    ("lib.rs", "origines_propres: mcp::origines_propres("),
];

#[test]
fn un_seul_point_d_entree() {
    let motif = re(MOTIF_MODULE_MCP);
    for rouge in [
        "use crate::mcp;",
        "    crate::mcp::transport::traiter(&e, p, r)",
        "use super::mcp::reponse;",
        "    mcp::reponse::rendre(&e, r)",
        "let x = (mcp::Registre::standard());",
        "    let r = kesh_api::mcp::Registre::standard();",
        "use kesh_api :: mcp;",
    ] {
        assert_eq!(sites_factices(rouge, &motif), 1, "rouge : {rouge}");
    }
    for vert in [
        "        acces: ContexteAcces::mcp(",
        "    pub portee_mcp: Option<Portee>,",
        "use kesh_db::repositories::mcp_keys;",
        "let cle = match cles_mcp::authentifier(&etat.garde, &etat.trace, jeton).await {",
        "    let (lues, ecartees) = mcp_server_settings::list_enabled(lecture.pool()).await?;",
        "    main_router = main_router.route_service(\"/mcp\", routes::mcp::recevoir.with_state(etat_mcp));",
        "/// crate::mcp::transport",
    ] {
        assert_eq!(sites_factices(vert, &motif), 0, "vert : {vert}");
    }
    let trouves = sites(&sources_src(), |f| !dans_mcp(f), &motif);
    assert_eq!(
        trouves,
        liste(LIB_NOMME_MCP),
        "le module mcp nommé hors de son point d'entrée"
    );

    // `routes::mcp::` n'apparaît qu'à la ligne de route.
    let routes_mcp = re(r"\broutes\s*::\s*mcp\s*::");
    assert_eq!(
        sites(&sources_src(), tout, &routes_mcp),
        liste(&[(
            "lib.rs",
            "main_router = main_router.route_service(\"/mcp\", routes::mcp::recevoir.with_state(etat_mcp));"
        )]),
    );
}

// ---------------------------------------------------------------------------
// AC-a30 — executer, outil_pour, descripteurs_pour, autoriser, appliquer (N3,
// N4, N16, N27) ; ContexteMcp::nouveau
// ---------------------------------------------------------------------------

/// Un **appel** (méthode ou forme qualifiée) ; les lignes de définition (`fn
/// executer`…) sont écartées par [`est_definition`].
const MOTIF_APPELS_REPARTITEUR: &str =
    r"\b(executer|outil_pour|descripteurs_pour|autoriser|appliquer)\s*\(";

fn est_definition(l: &str) -> bool {
    re(r"\bfn\s+(executer|outil_pour|descripteurs_pour|autoriser|appliquer)\b").is_match(l)
}

fn appels(source: &str, motif: &Regex) -> usize {
    source
        .lines()
        .filter(|l| !est_commentaire(l) && motif.is_match(l) && !est_definition(l))
        .count()
}

#[test]
fn executer_appele_par_le_repartiteur_seul() {
    let motif = re(MOTIF_APPELS_REPARTITEUR);
    for rouge in [
        "    let r = outil.executer(&ctx, arguments).await;",
        "    McpTool::executer(&*o, &ctx, a)",
        "    Registre::outil_pour(&r, nom, &p)",
        "    etat.registre.descripteurs_pour(&portees)",
        "    let a = politique::autoriser(v)?;",
        "    politique::appliquer(a, d)",
    ] {
        assert_eq!(appels(rouge, &motif), 1, "rouge : {rouge}");
    }
    for vert in [
        "    fn executer<'a>(",
        "pub(crate) fn autoriser(validation: SendValidation) -> Autorisation {",
        "pub(crate) fn appliquer(autorisation: Autorisation, donnees: Donnees) -> Sortie {",
        "    pub(in crate::mcp) fn outil_pour(",
        "    pub(in crate::mcp) fn descripteurs_pour(",
    ] {
        assert_eq!(appels(vert, &motif), 0, "vert : {vert}");
    }
    let mut hors = Vec::new();
    for (f, t) in sources_src() {
        for l in t.lines() {
            if !est_commentaire(l)
                && motif.is_match(l)
                && !est_definition(l)
                && f != "mcp/repartiteur.rs"
            {
                hors.push((f.clone(), l.trim().to_string()));
            }
        }
    }
    assert!(hors.is_empty(), "appel hors du répartiteur : {hors:?}");
}

#[test]
fn contexte_mcp_construit_par_le_repartiteur_seul() {
    let motif = re(r"\bContexteMcp\s*::\s*nouveau\b|\bContexteMcp\s*\{");
    assert_eq!(
        sites_factices("    let c = ContexteMcp::nouveau(&a, &l);", &motif),
        1
    );
    assert_eq!(
        sites_factices("    let c = ContexteMcp { acces, lecture };", &motif),
        1
    );
    assert_eq!(
        sites_factices("    ctx: &'a ContexteMcp<'a>,", &motif),
        0,
        "vert"
    );
    assert_eq!(
        sites(&sources_src(), tout, &motif),
        liste(&[(
            "mcp/repartiteur.rs",
            // dans la tâche de l'outil, sur les copies de `cle.acces` et
            // `etat.lecture` qu'elle possède (C-32-6-56)
            "let ctx = ContexteMcp::nouveau(&acces, &lecture);"
        )])
    );
}

// ---------------------------------------------------------------------------
// AC-a30 — contexte_acces_mcp_construit_par_authentifier_seul (N24)
// ---------------------------------------------------------------------------

#[test]
fn contexte_acces_mcp_construit_par_authentifier_seul() {
    // (1) `ContexteAcces::mcp` (et `Self::mcp`) nommé en `services/cles_mcp.rs`
    // seulement — la définition, `pub fn mcp(`, ne le nomme pas.
    let fabrique =
        re(r"\bContexteAcces\s*::\s*mcp\b|\bSelf\s*::\s*mcp\b|<\s*ContexteAcces\s*>\s*::\s*mcp\b");
    for rouge in [
        "    let a = ContexteAcces::mcp(1, 2, toutes);",
        "    let a = <ContexteAcces>::mcp(1, 2, toutes);",
        "        Self::mcp(1, 2, p)",
    ] {
        assert_eq!(sites_factices(rouge, &fabrique), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices(
            "    pub fn mcp(company_id: i64, user_id: i64, portees: BTreeSet<Portee>) -> Self {",
            &fabrique
        ),
        0
    );
    assert_eq!(
        sites(&sources_src(), tout, &fabrique),
        liste(&[("services/cles_mcp.rs", "acces: ContexteAcces::mcp(")])
    );
    // (2) dans `src/mcp/`, le jeton `ContexteAcces` en `mcp/outil.rs` seulement.
    let jeton = re(r"\bContexteAcces\b");
    let trouves: Vec<String> = sites(&sources_src(), |f| f.starts_with("mcp/"), &jeton)
        .into_iter()
        .map(|(f, _)| f)
        .filter(|f| f != "mcp/outil.rs")
        .collect();
    assert!(
        trouves.is_empty(),
        "ContexteAcces nommé dans src/mcp/ : {trouves:?}"
    );
    assert_eq!(
        sites_factices("    fn f(a: &ContexteAcces) {}", &jeton),
        1,
        "rouge"
    );
    assert_eq!(sites_factices("    acces: &cle.acces,", &jeton), 0, "vert");
    // (3) `impl ContexteAcces` en `services/acces.rs` seulement.
    let imp = re(r"\bimpl\b[^{]*\bContexteAcces\b");
    assert_eq!(sites_factices("impl ContexteAcces {", &imp), 1);
    assert_eq!(sites_factices("impl Trait for ContexteAcces {", &imp), 1);
    let fichiers: Vec<String> = sites(&sources_src(), tout, &imp)
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    assert_eq!(fichiers, vec!["services/acces.rs".to_string()]);
}

// ---------------------------------------------------------------------------
// AC-a30 — registre_standard_en_un_lieu, un_seul_ecouteur,
// pool_lecture_seule_en_un_lieu
// ---------------------------------------------------------------------------

#[test]
fn registre_standard_en_un_lieu() {
    let motif = re(r"\bRegistre\s*::\s*standard\b");
    assert_eq!(
        sites_factices("    let r = mcp::Registre::standard();", &motif),
        1
    );
    assert_eq!(
        sites_factices("    pub fn standard() -> Self {", &motif),
        0,
        "vert : la définition"
    );
    assert_eq!(
        sites(&sources_src(), tout, &motif),
        liste(&[(
            "lib.rs",
            "build_router_avec_registre(state, static_dir, mcp::Registre::standard())"
        )])
    );
    let main = re(r"\bbuild_router_avec_registre\b|\bRegistre\s*::");
    assert!(
        sites(&sources_src(), |f| f == "main.rs", &main).is_empty(),
        "main.rs"
    );
    assert_eq!(
        sites(
            &sources_src(),
            |f| f == "main.rs",
            &re(r"\bbuild_router\s*\(")
        )
        .len(),
        1,
        "témoin : main.rs appelle build_router"
    );
}

#[test]
fn un_seul_ecouteur() {
    // Périmètre : `kesh-api/src` ; les fichiers de test qui lient un écouteur
    // sont hors du compte. ⚠️ Angle mort écrit (revue P1, A6) : un écouteur
    // construit par un crate que `kesh-api` ne dépend pas aujourd'hui
    // (`socket2`, un descripteur hérité passé à `hyper_util`) — `socket2` et
    // `from_raw_fd` sont attrapés, les autres formes relèvent de la revue d'une
    // dépendance ajoutée.
    let motif = re(r"\bTcpListener\b|\bTcpSocket\b|\bUnixListener\b|\bsocket2\b|\bfrom_raw_fd\b");
    for rouge in [
        "    let l = tokio::net::TcpListener::bind(a).await?;",
        "    let l = std::net::TcpListener::bind(a)?;",
        "    let s = tokio::net::TcpSocket::new_v4()?;",
        "    let u = tokio::net::UnixListener::bind(p)?;",
        "    let s = socket2::Socket::new(d, t, None)?;",
        "    let l = unsafe { std::net::TcpListener::from_raw_fd(3) };",
    ] {
        assert_eq!(sites_factices(rouge, &motif), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices("    let addr: SocketAddr = x;", &motif),
        0,
        "vert"
    );
    assert_eq!(
        sites(&sources_src(), tout, &motif),
        liste(&[(
            "main.rs",
            "let listener = tokio::net::TcpListener::bind(&bind_addr)"
        )])
    );
}

#[test]
fn pool_lecture_seule_en_un_lieu() {
    // Périmètre : `kesh-api/src` ; `tests/common/lecture_seule.rs` et
    // `kesh-db/tests/pool_lecture_seule.rs` en sont hors.
    let motif = re(r"\bpool_lecture_seule\s*\(");
    assert_eq!(
        sites_factices(
            "    let p = kesh_db::pool::pool_lecture_seule(&p, 3);",
            &motif
        ),
        1
    );
    assert_eq!(
        sites_factices("pub fn pool_lecture_seule_x() {}", &motif),
        0,
        "vert"
    );
    assert_eq!(
        sites(&sources_src(), tout, &motif),
        liste(&[(
            "lib.rs",
            "kesh_db::pool::pool_lecture_seule(&state.pool, mcp::MCP_POOL_CONNEXIONS, delai);"
        )])
    );
}

// ---------------------------------------------------------------------------
// AC-a30 — mcp_sans_pool (N19, N27)
// ---------------------------------------------------------------------------

/// Les noms d'un pool et de ce qui en fabrique un ; `kesh_db::{…, repositories…}`
/// et `kesh_db::{…, pool…}` compris quand l'import groupé tient **sur une
/// ligne** (revue P1, A6).
///
/// ⚠️ **Angles morts écrits** (revue P2, H4) : la garde lit ligne par ligne ;
/// l'import groupé que rustfmt **coupe** (`use kesh_db::{` / `repositories::x,`
/// / `};`) lui échappe, la ligne qui nomme `repositories` ne nommant pas
/// `kesh_db` ; un groupe **imbriqué** (`kesh_db::{entities::{A, B},
/// repositories::x}`) aussi, `[^}]*` s'arrêtant au premier `}`. Le chemin utile
/// reste fermé par le compilateur : `Lecture::pool` est
/// `pub(in crate::services)`, si bien qu'un module de `mcp/` ne peut tirer un
/// pool d'aucune `Lecture`.
const MOTIF_SANS_POOL: &str = r"\bMySqlPool\b|\bsqlx\b|\bAppState\b|\.\s*pool\b|\bkesh_db\s*::\s*repositories\b|\bkesh_db\s*::\s*pool\b|\bkesh_db\s*::\s*\{[^}]*\b(repositories|pool)\b|\bcreate_pool\b|\bdatabase_url\b|\bConfig\b";

#[test]
fn mcp_sans_pool() {
    let motif = re(MOTIF_SANS_POOL);
    for rouge in [
        "    pub(crate) app: AppState,",
        "use sqlx::MySqlPool;",
        "    let p = etat.app.pool.clone();",
        "    let p = kesh_db::pool::pool_lecture_seule(&p, 3);",
        "use kesh_db::repositories::accounts;",
        "    config: Arc<Config>,",
        "    let u = cfg.database_url.clone();",
        "    let p = kesh_db::pool::create_pool(u, 5, t);",
        "use kesh_db::{entities::McpServerSettings, repositories::accounts};",
        "use kesh_db::{entities::X, pool::pool_lecture_seule};",
    ] {
        assert_eq!(sites_factices(rouge, &motif), 1, "rouge : {rouge}");
    }
    for vert in [
        "use axum::extract::{ConnectInfo, Request};",
        "use kesh_db::entities::McpServerSettings;",
        "pub const MCP_POOL_CONNEXIONS: u32 = 3;",
    ] {
        assert_eq!(sites_factices(vert, &motif), 0, "vert : {vert}");
    }
    assert!(
        sites(&sources_src(), dans_mcp, &motif).is_empty(),
        "{:?}",
        sites(&sources_src(), dans_mcp, &motif)
    );

    let outils = re(r"\bEtatMcp\b|\bContexteAcces\s*::");
    assert_eq!(sites_factices("    let e: &EtatMcp = x;", &outils), 1);
    assert_eq!(
        sites_factices("    let a = ContexteAcces::mcp(1, 2, p);", &outils),
        1
    );
    assert_eq!(
        sites_factices(
            "            let etat = serveur_mcp::etat(ctx.acces(), ctx.lecture())",
            &outils
        ),
        0
    );
    assert!(sites(&sources_src(), |f| f.starts_with("mcp/outils/"), &outils).is_empty());
}

// ---------------------------------------------------------------------------
// AC-a30 — outils_et_politique_en_leur_lieu (N27, N28)
// ---------------------------------------------------------------------------

#[test]
fn outils_et_politique_en_leur_lieu() {
    let imp = re(r"\bimpl\b[^{]*\bMcpTool\s+for\b");
    assert_eq!(sites_factices("impl McpTool for X {", &imp), 1);
    assert_eq!(
        sites_factices("impl<'a> crate::mcp::McpTool for X<'a> {", &imp),
        1
    );
    assert_eq!(
        sites_factices("pub trait McpTool: Send + Sync + 'static {", &imp),
        0,
        "vert"
    );
    let hors: Vec<_> = sites(&sources_src(), |f| !f.starts_with("mcp/outils/"), &imp);
    assert!(
        hors.is_empty(),
        "impl McpTool hors de src/mcp/outils/ : {hors:?}"
    );
    assert_eq!(
        sites(&sources_src(), |f| f.starts_with("mcp/outils/"), &imp).len(),
        1,
        "témoin : kesh_version"
    );

    let lecture = re(r"\bLectureOutils\s*::\s*nouvelle\b|\bLectureOutils\s*\(");
    assert_eq!(
        sites_factices("    let l = LectureOutils::nouvelle(x);", &lecture),
        1
    );
    assert_eq!(sites_factices("    let l = LectureOutils(x);", &lecture), 1);
    assert_eq!(
        sites_factices("    lecture: &'a LectureOutils,", &lecture),
        0
    );
    assert_eq!(
        sites(&sources_src(), |f| f != "mcp/outil.rs", &lecture),
        liste(&[(
            "lib.rs",
            "mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils.clone()));"
        )])
    );

    // Story 32-8b (AC-B1) : `.deposer(` d'un `EnvoiAValider` n'est appelé, dans
    // `src/mcp/`, qu'au répartiteur ; la définition (`fn deposer`) n'est pas un
    // appel. Hors de `src/mcp/`, `.deposer(` est la poignée des envois
    // (`services/`), hors du périmètre.
    let deposer = re(r"\.\s*deposer\s*\(");
    assert_eq!(
        sites_factices(
            "        .deposer(cle, &etat.ecriture_envois, nom, debut)",
            &deposer
        ),
        1
    );
    assert_eq!(
        sites_factices("    pub(in crate::mcp) async fn deposer(", &deposer),
        0,
        "vert : la définition"
    );
    assert_eq!(
        sites_factices("        envois_mcp::deposer(", &deposer),
        0,
        "vert : l'appel du service par la méthode, en politique.rs"
    );
    let trouves: Vec<String> = sites(&sources_src(), |f| f.starts_with("mcp/"), &deposer)
        .into_iter()
        .map(|(f, _)| f)
        .collect();
    assert_eq!(
        trouves,
        vec!["mcp/repartiteur.rs".to_string()],
        "`.deposer(` d'un EnvoiAValider hors du répartiteur"
    );

    let donnees = re(r"\bDonnees\s*::\s*nouvelles\b|\bDonnees\s*\{");
    assert_eq!(
        sites_factices("    Ok(Donnees::nouvelles(t, None))", &donnees),
        1
    );
    assert_eq!(
        sites_factices("    let d = Donnees { texte, structure };", &donnees),
        1
    );
    assert_eq!(sites_factices("    donnees: Donnees,", &donnees), 0);
    let hors: Vec<_> = sites(
        &sources_src(),
        |f| !f.starts_with("mcp/outils/") && f != "mcp/politique.rs",
        &donnees,
    );
    assert!(
        hors.is_empty(),
        "Donnees construite hors des outils : {hors:?}"
    );
}

// ---------------------------------------------------------------------------
// AC-a23 — donnees_construites_par_la_politique_seule
// ---------------------------------------------------------------------------

/// Les attributs `#[derive(…)]` qui précèdent `struct <nom>` ou `enum <nom>`
/// (commentaires de doc sautés) — `enum` depuis la 32-8b, `Autorisation` et
/// `Sortie` étant des énumérations.
fn derives_de(source: &str, nom: &str) -> Vec<String> {
    let lignes: Vec<&str> = source.lines().collect();
    let tete = re(&format!(
        r"^\s*pub(\([^)]*\))?\s+(struct|enum)\s+{nom}\b|^\s*(struct|enum)\s+{nom}\b"
    ));
    let mut v = Vec::new();
    for (i, l) in lignes.iter().enumerate() {
        if tete.is_match(l) {
            let mut j = i;
            while j > 0 {
                j -= 1;
                let p = lignes[j].trim();
                if p.starts_with("#[") {
                    v.push(p.to_string());
                } else if !p.starts_with("///") {
                    break;
                }
            }
        }
    }
    v
}

fn interdits_dans(derives: &[String], interdits: &[&str]) -> Vec<String> {
    derives
        .iter()
        .filter(|d| {
            interdits
                .iter()
                .any(|i| re(&format!(r"\b{i}\b")).is_match(d))
        })
        .cloned()
        .collect()
}

#[test]
fn donnees_construites_par_la_politique_seule() {
    let construction = re(
        r"\bDonneesEnvoyables\s*\{|\bDonneesEnvoyables\s*::|\bimpl\b[^{]*\bfor\s+DonneesEnvoyables\b|\bimpl\s+DonneesEnvoyables\b",
    );
    for rouge in [
        "    let d = DonneesEnvoyables { donnees };",
        "    let d = DonneesEnvoyables::default();",
        "impl Default for DonneesEnvoyables {",
    ] {
        assert_eq!(sites_factices(rouge, &construction), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices("        donnees: DonneesEnvoyables,", &construction),
        0,
        "vert"
    );
    let hors: Vec<_> = sites(&sources_src(), |f| f != "mcp/politique.rs", &construction);
    assert!(
        hors.is_empty(),
        "DonneesEnvoyables hors de politique.rs : {hors:?}"
    );

    let politique = std::fs::read_to_string(racine_src().join("mcp/politique.rs")).unwrap();
    let factice = "/// doc\n#[derive(Clone, Debug)]\npub struct Donnees {\n";
    assert_eq!(
        interdits_dans(&derives_de(factice, "Donnees"), &["Debug"]).len(),
        1,
        "rouge"
    );
    assert_eq!(
        derives_de(&politique, "Donnees"),
        vec!["#[derive(Clone)]".to_string()],
        "vert existant"
    );
    let tous = &["Serialize", "Default", "Debug", "Clone", "Copy"][..];
    for (nom, interdits) in [
        ("Donnees", &["Serialize", "Default", "Debug"][..]),
        ("DonneesEnvoyables", &["Serialize", "Default", "Debug"][..]),
        ("Autorisation", tous),
        // Story 32-8b (AC-B1) : les deux jetons, l'envoi à valider, la sortie.
        ("JetonEnvoi", tous),
        ("JetonDepot", tous),
        ("EnvoiAValider", tous),
        ("Sortie", tous),
    ] {
        let fautifs = interdits_dans(&derives_de(&politique, nom), interdits);
        assert!(fautifs.is_empty(), "{nom} : {fautifs:?}");
    }
    // Témoin : `derives_de` voit une énumération (sans quoi `Autorisation` et
    // `Sortie` passeraient à vide).
    assert_eq!(
        derives_de("#[derive(Debug)]\npub(crate) enum Sortie {\n", "Sortie"),
        vec!["#[derive(Debug)]".to_string()]
    );
    let imp = re(
        r"\bimpl\b[^{]*\b(Default|Serialize|Debug)\b[^{]*\bfor\s+(Donnees|DonneesEnvoyables|Autorisation|JetonEnvoi|JetonDepot|EnvoiAValider|Sortie)\b|\bimpl\b[^{]*\b(Clone|Copy)\b[^{]*\bfor\s+(Autorisation|JetonEnvoi|JetonDepot|EnvoiAValider|Sortie)\b",
    );
    for rouge in [
        "impl Default for Donnees {",
        "impl serde::Serialize for DonneesEnvoyables {",
        "impl std::fmt::Debug for Autorisation {",
        "impl Clone for Autorisation {",
        "impl Clone for EnvoiAValider {",
        "impl std::fmt::Debug for JetonDepot {",
    ] {
        assert_eq!(sites_factices(rouge, &imp), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices("impl Clone for Donnees {", &imp),
        0,
        "vert : permis"
    );
    assert!(sites(&sources_src(), tout, &imp).is_empty());

    // Story 32-8b (AC-B1) — les jetons, l'envoi à valider et les variantes de
    // l'autorisation ne se construisent qu'en `politique.rs` ; `Sortie` s'y
    // construit aussi, et le répartiteur la DÉFAIT par un `match` — ses deux
    // lignes de motif sont la liste blanche, lexicalement indiscernables d'une
    // construction.
    let construction = re(
        r"\b(JetonEnvoi|JetonDepot|EnvoiAValider)\s*\{|\bEnvoiAValider\s*::|\bAutorisation\s*::\s*(Envoi|Depot)\b|\bSortie\s*::\s*(Envoyer|Deposer)\b|\bimpl\s+EnvoiAValider\b",
    );
    for rouge in [
        "    let j = JetonDepot { _prive: () };",
        "    let e = EnvoiAValider { resume: None, contenu: c };",
        "    Autorisation::Envoi(j)",
        "    Sortie::Envoyer(d)",
        "impl EnvoiAValider {",
    ] {
        assert_eq!(sites_factices(rouge, &construction), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices("    a_valider: EnvoiAValider,", &construction),
        0,
        "vert"
    );
    assert_eq!(
        sites(&sources_src(), |f| f != "mcp/politique.rs", &construction),
        liste(&[
            ("mcp/repartiteur.rs", "Sortie::Envoyer(donnees) => ("),
            ("mcp/repartiteur.rs", "Sortie::Deposer(a_valider) => {"),
        ]),
        "jeton, envoi à valider ou sortie construits hors de politique.rs"
    );
}

// ---------------------------------------------------------------------------
// AC-a24 — prompts_scelles
// ---------------------------------------------------------------------------

#[test]
fn prompts_scelles() {
    let motif = re(r"\bimpl\b[^{]*\b(McpPrompt|Scelle)\s+for\b");
    assert_eq!(sites_factices("impl McpPrompt for Bilan {", &motif), 1);
    assert_eq!(
        sites_factices("impl scelle::Scelle for Bilan {}", &motif),
        1
    );
    assert_eq!(
        sites_factices(
            "pub trait McpPrompt: scelle::Scelle + Send + Sync + 'static {",
            &motif
        ),
        0
    );
    // Périmètre : `impl … McpPrompt for` sur tout `src/` ; `impl … Scelle for`
    // sur `src/mcp/` seulement — `services/rapports.rs` a son propre sceau,
    // homonyme (`impl Scelle for super::Affichage {}`), témoin vert existant.
    let prompt = re(r"\bimpl\b[^{]*\bMcpPrompt\s+for\b");
    assert!(
        sites(&sources_src(), tout, &prompt).is_empty(),
        "aucun prompt en 32-6"
    );
    assert!(
        sites(&sources_src(), |f| f.starts_with("mcp/"), &motif).is_empty(),
        "aucun sceau posé dans src/mcp/"
    );
    assert_eq!(
        sites(&sources_src(), |f| f == "services/rapports.rs", &motif).len(),
        2,
        "témoin vert existant : le sceau des rapports, hors du périmètre"
    );
}

// ---------------------------------------------------------------------------
// AC-a27 — lecture_seule_jamais_defaite
// ---------------------------------------------------------------------------

/// Les quatre formes de la spike (§ 4.1) qui défont `SET SESSION TRANSACTION
/// READ ONLY` et le nom de la variable de session, quelle que soit la casse.
/// Les commentaires exécutables de MariaDB (`/*!…*/`, `/*M!…*/`) ne sont **pas**
/// effacés (ils sont exécutés) : une forme qu'ils logent est vue (revue P1, A1).
///
/// S'y ajoute `max_statement_time` (revue P2, H3) : la borne que pose
/// `pool_lecture_seule` à côté du `READ ONLY` se défait de la même manière
/// (`SET SESSION max_statement_time = 0`, `SET STATEMENT max_statement_time = 0
/// FOR …`) ; tout site qui la nomme est une exception écrite.
const MOTIF_LECTURE_SEULE: &str =
    r"(?i)\bread\s+write\b|\btx_read_only\b|\btransaction_read_only\b|\bmax_statement_time\b";

/// En plus, dans le **Rust** : tout commentaire exécutable, refusé en lui-même
/// — aucun code de Kesh n'en émet, et un SQL émis sur la connexion du MCP n'a
/// pas à en porter. Les `.sql` en sont exemptés : le squash de test
/// (`test-schema/0001_schema_squash.sql`, produit par `mysqldump`) en porte des
/// dizaines (`/*!40101 SET … */`), qui ne passent jamais sur la connexion du
/// MCP ; leur contenu reste lu par [`MOTIF_LECTURE_SEULE`].
///
/// ⚠️ **Faux positif écrit** (revue P2, H5) : en Rust, `/*! … */` est aussi un
/// commentaire de doc **interne** de bloc ; la garde le refuserait. Écrire la
/// doc interne en `//!` (lignes écartées d'abord, jamais vues ici). Aucun
/// `/*!` dans le Rust de `crates/` aujourd'hui, hors de ce fichier.
const MOTIF_COMMENTAIRE_EXECUTABLE: &str = r"/\*M?!";

/// Les lignes d'un texte : lignes de commentaire (`//`, `--`, `#` en SQL)
/// écartées **d'abord** — un `/*` de glob dans un commentaire de doc
/// (`crates/*/src`) n'ouvre donc pas de bloc —, puis commentaires de bloc
/// remplacés **par une espace** (comme le fait MariaDB), **sauf** les
/// commentaires exécutables `/*!` et `/*M!`, gardés tels quels pour que le
/// motif les voie.
///
/// ⚠️ **Limite écrite** (revue P1, A1) : l'aide ne connaît pas les littéraux.
/// Un `/*` **dans une chaîne** d'une ligne de code ouvre un bloc qui avale tout
/// jusqu'au `*/` suivant, sur plusieurs lignes ; une forme placée là
/// échapperait. Même limite que `sans_commentaires` de `services_motif.rs`.
/// La garde ne peut pas mesurer ce qu'elle avale : le fait qu'aucune forme ne
/// s'y cache aujourd'hui s'établit **hors d'elle**, par un `grep` brut qui ne
/// retire aucun commentaire (revue P2, H6) — `grep -rniE
/// 'read\s+write|tx_read_only|transaction_read_only|max_statement_time'
/// --include=*.rs crates/`, hors de ce fichier, ne rend que des lignes de
/// commentaire et les cinq exceptions de la liste blanche ci-dessous (relevé
/// du 2026-10-25).
fn lignes_decommentees(texte: &str, sql: bool) -> Vec<String> {
    let sans_lignes: String = texte
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("//") || (sql && (t.starts_with("--") || t.starts_with('#'))))
        })
        .collect::<Vec<_>>()
        .join("\n");
    re(r"(?s)/\*.*?\*/")
        .replace_all(&sans_lignes, |c: &regex::Captures| {
            let bloc = &c[0];
            if bloc.starts_with("/*!") || bloc.starts_with("/*M!") {
                bloc.to_string()
            } else {
                " ".to_string()
            }
        })
        .lines()
        .map(str::to_string)
        .collect()
}

fn formes_interdites(texte: &str, sql: bool) -> Vec<String> {
    let motif = re(MOTIF_LECTURE_SEULE);
    let executable = re(MOTIF_COMMENTAIRE_EXECUTABLE);
    lignes_decommentees(texte, sql)
        .into_iter()
        .filter(|l| motif.is_match(l) || (!sql && executable.is_match(l)))
        .map(|l| l.trim().to_string())
        .collect()
}

#[test]
fn lecture_seule_jamais_defaite() {
    for rouge in [
        "SET SESSION TRANSACTION READ WRITE",
        "START TRANSACTION READ WRITE",
        "SET TRANSACTION READ WRITE",
        "SET SESSION tx_read_only=0",
        "SET @@session.tx_read_only := 0",
        "SET SESSION `tx_read_only`=0",
        "SET TRANSACTION READ/**/WRITE",
        "SET SESSION transaction_read_only = 0",
        "set transaction read  write",
        // une forme logée dans un commentaire exécutable (revue P1, A1)
        "SET TRANSACTION /*M!100000 READ WRITE */",
        "SET /*!50000 SESSION TRANSACTION READ WRITE */",
        // la borne d'instruction défaite (revue P2, H3)
        "SET SESSION max_statement_time = 0",
        "SET STATEMENT MAX_STATEMENT_TIME=0 FOR SELECT 1",
    ] {
        assert_eq!(
            formes_interdites(rouge, true).len(),
            1,
            "rouge (SQL) : {rouge}"
        );
        assert_eq!(
            formes_interdites(&format!("q(\"{rouge}\")"), false).len(),
            1,
            "rouge (Rust) : {rouge}"
        );
    }
    for vert in [
        // existants au dépôt
        "    ReadWrite => \"read-write\",",
        "use std::io::{Read, Write};",
        "        sqlx::query(\"SET SESSION TRANSACTION READ ONLY\")",
        // factice : le retrait des commentaires
        "/// la forme READ WRITE est interdite",
        "-- READ WRITE en commentaire SQL",
        "/* READ WRITE */ SELECT 1",
        "/// la borne max_statement_time en commentaire",
        "    let max_statement_timeout = 0;",
    ] {
        assert!(formes_interdites(vert, true).is_empty(), "vert : {vert}");
    }
    // Un `/*` de glob dans un commentaire de doc (forme existante :
    // `configuration_transmise.rs:16`, « `crates/*/src` ») n'avale plus le code
    // qui suit : la forme interdite placée entre lui et un `*/` lointain est vue.
    let glob = "//! (`crates/*/src`) : chaque occurrence\nq(\"SET TRANSACTION READ WRITE\")\n/// `crates/*/src/**/*.rs`\n";
    assert_eq!(
        formes_interdites(glob, false).len(),
        1,
        "rouge : sous un glob"
    );
    // Dans le Rust, un commentaire exécutable est refusé en lui-même ; dans le
    // SQL, seul son contenu est lu (témoin vert existant : le squash de test).
    let execute = "q(\"SELECT /*!50000 1 */\")";
    assert_eq!(formes_interdites(execute, false).len(), 1, "rouge (Rust)");
    assert!(
        formes_interdites(
            "/*!40101 SET @saved_cs_client     = @@character_set_client */;",
            true
        )
        .is_empty(),
        "vert (SQL) : la ligne existante du squash"
    );

    let mut v = Vec::new();
    fichiers(&racine_crates(), &["rs", "sql"], &mut v);
    v.sort();
    assert!(v.len() >= 400, "balayage de crates/ : {} fichiers", v.len());
    let ce_fichier = racine_crate().join("tests/mcp_gardes_lexicales.rs");
    let mut trouves = Vec::new();
    for p in v.iter().filter(|p| **p != ce_fichier) {
        let texte = std::fs::read_to_string(p).unwrap();
        let sql = p.extension().is_some_and(|x| x == "sql");
        for l in formes_interdites(&texte, sql) {
            let rel = p
                .strip_prefix(racine_crates())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            trouves.push((rel, l));
        }
    }
    trouves.sort();
    assert_eq!(
        trouves,
        liste(&[
            // AC-a25 (e) et (e bis) : les deux exécutions volontaires.
            (
                "kesh-db/tests/pool_lecture_seule.rs",
                "sqlx::query(\"SET SESSION TRANSACTION READ WRITE\")"
            ),
            (
                "kesh-db/tests/pool_lecture_seule.rs",
                "sqlx::query(\"SET TRANSACTION READ WRITE\")"
            ),
            // AC-a28 : la lecture du mode par `kesh_version`.
            (
                "kesh-api/src/services/serveur_mcp.rs",
                "sqlx::query_as(\"SELECT @@session.tx_read_only, @@session.max_statement_time\")"
            ),
            // Story 32-8b (règle R7) : le témoin de `lire_pour_cle_en_lecture_seule`
            // lit le mode de la connexion, il ne le change pas.
            (
                "kesh-api/tests/mcp_envois_e2e.rs",
                "let mode: i64 = sqlx::query_scalar(\"SELECT @@session.tx_read_only\")"
            ),
            // C-32-6-54 : la pose de la borne, et sa lecture par le test (g).
            (
                "kesh-db/src/pool.rs",
                "\"SET SESSION max_statement_time = {}.{:03}\","
            ),
            (
                "kesh-db/tests/pool_lecture_seule.rs",
                "sqlx::query_scalar(\"SELECT @@session.max_statement_time\")"
            ),
        ]),
        "forme qui défait le mode de session hors des exceptions écrites"
    );
}

// ---------------------------------------------------------------------------
// AC-a31 — reponses_par_rendre_seul (N25)
// ---------------------------------------------------------------------------

const MOTIF_REPONSES: &str = r"\binto_response\b|\bIntoResponse\b|\bAppError\b";

/// L'inventaire des sites — non des formes : chaque ligne de code de
/// `src/mcp/**` et `routes/mcp.rs` qui nomme `into_response`, `IntoResponse`
/// ou `AppError`.
const SITES_REPONSES: &[(&str, &str)] = &[
    ("mcp/textes.rs", "use crate::errors::AppError;"),
    (
        "mcp/textes.rs",
        "pub fn depuis_app_error(e: AppError) -> ErreurOutil {",
    ),
    (
        "mcp/textes.rs",
        "AppError::Forbidden => ErreurOutil::Inconnu,",
    ),
    (
        "mcp/textes.rs",
        "AppError::Database(DbError::NotFound) => ErreurOutil::Introuvable,",
    ),
    (
        "mcp/textes.rs",
        "AppError::Validation(_) => ErreurOutil::ArgumentsInvalides {",
    ),
    ("mcp/transport.rs", "use crate::errors::AppError;"),
    (
        "mcp/transport.rs",
        "Err(AppError::Unauthenticated(_)) => return ReponseMcp::NonAuthentifie,",
    ),
];

#[test]
fn reponses_par_rendre_seul() {
    let motif = re(MOTIF_REPONSES);
    let rouge = "    match traiter(e, p, r).await { Ok(j) => j.into_response(), Err(e) => e.into_response() }";
    assert_eq!(
        sites_factices(rouge, &motif),
        1,
        "rouge : la forme de la P2"
    );
    assert_eq!(
        sites_factices(
            "pub(crate) async fn rendre(etat: &EtatMcp, r: ReponseMcp) -> Response {",
            &motif
        ),
        0,
        "vert"
    );
    assert_eq!(
        sites(&sources_src(), dans_mcp, &motif),
        liste(SITES_REPONSES)
    );

    // Le gestionnaire : aucun extracteur de corps, et un corps exact.
    let gestionnaire = std::fs::read_to_string(racine_src().join("routes/mcp.rs")).unwrap();
    let corps = gestionnaire
        .split("pub async fn recevoir(")
        .nth(1)
        .expect("recevoir défini");
    let (signature, reste) = corps.split_once(") -> Response {").expect("signature");
    assert!(
        !re(r"\b(Bytes|String|Json|Form)\b").is_match(signature),
        "extracteur de corps dans recevoir : {signature}"
    );
    let corps_fn: Vec<&str> = reste
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    assert_eq!(
        &corps_fn[..2],
        &[
            "reponse::rendre(&etat, transport::traiter(&etat, pair, requete).await).await",
            "}"
        ]
    );
}

// ===========================================================================
// Story 32-8b (refs #723) — les gardes des envois à valider (AC-B12, AC-B14)
// ===========================================================================

/// Le texte **aplati** d'une source Rust, débarrassé de ses commentaires et
/// de ses littéraux, et la liste de ses littéraux de chaîne.
///
/// Dans le squelette, chaque littéral de chaîne (`"…"`, `b"…"`, brut
/// `r#"…"#`) et de caractère (`'('`) est remplacé par une espace — sauf les
/// **interpolations** `{ident…}` d'un littéral, rendues comme l'identifiant
/// seul (`{{` est un échappement, non une interpolation) ; une durée de vie
/// (`'a`) reste du code. Les sauts de ligne deviennent des espaces : une forme
/// coupée sur plusieurs lignes se lit d'un tenant.
///
/// ⚠️ **Angles morts écrits** : une chaîne produite par une macro autre que
/// celles de `tracing` (`concat!`, `stringify!`) n'est pas interprétée ; un
/// identifiant fabriqué par une macro, un alias (`use … as`), un argument
/// renommé (`let c = contenu; warn!("{c}")`) et un type porteur formaté par
/// `Debug` ne sont pas vus — les identifiants visés sont exacts, non des
/// sous-chaînes. Les types de `kesh-db` qui portent contenu ou résumé masquent
/// l'un et l'autre dans leur `Debug` (C-32-8-40). Trois formes encore, écrites
/// à la revue de code P1 (B-4) et non traitées : un commentaire de bloc
/// **imbriqué** (`/* a /* b */ contenu */`, que Rust admet) se ferme au
/// premier `*/` ; une macro d'évènement invoquée par `{…}` ou `[…]`
/// (`warn!{…}`, `error![…]`) échappe à `appels_d_evenement`, qui n'apparie
/// que la parenthèse ; les préfixes `c"…"` et `cr"…"` ne sont pas reconnus
/// comme tels : l'un et l'autre se lisent comme un identifiant suivi d'une
/// chaîne **ordinaire** — sans effet pour `c"…"`, dont les échappements sont
/// ceux d'une chaîne ordinaire, mais une `cr#"…"#` est lue avec les règles
/// d'échappement d'une chaîne ordinaire, non d'une chaîne brute.
fn squelette(source: &str) -> (String, Vec<String>) {
    let c: Vec<char> = source.chars().collect();
    let mut code = String::with_capacity(source.len());
    let mut litteraux = Vec::new();
    let mut i = 0;
    let ident = |ch: char| ch.is_alphanumeric() || ch == '_';
    while i < c.len() {
        // Commentaire de ligne (doc comprise) et de bloc.
        if c[i] == '/' && c.get(i + 1) == Some(&'/') {
            while i < c.len() && c[i] != '\n' {
                i += 1;
            }
            continue;
        }
        if c[i] == '/' && c.get(i + 1) == Some(&'*') {
            i += 2;
            while i + 1 < c.len() && !(c[i] == '*' && c[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            code.push(' ');
            continue;
        }
        let precedent_ident = i > 0 && ident(c[i - 1]);
        // Chaîne brute : `r"…"`, `r#"…"#`, `br#"…"#`.
        let debut_brut = if c[i] == 'r' && !precedent_ident {
            Some(i + 1)
        } else if c[i] == 'b' && c.get(i + 1) == Some(&'r') && !precedent_ident {
            Some(i + 2)
        } else {
            None
        };
        if let Some(mut j) = debut_brut {
            let mut dieses = 0;
            while c.get(j) == Some(&'#') {
                dieses += 1;
                j += 1;
            }
            if c.get(j) == Some(&'"') {
                j += 1;
                let mut contenu = String::new();
                loop {
                    if j >= c.len() {
                        break;
                    }
                    if c[j] == '"' && (1..=dieses).all(|k| c.get(j + k) == Some(&'#')) {
                        j += 1 + dieses;
                        break;
                    }
                    contenu.push(c[j]);
                    j += 1;
                }
                code.push(' ');
                code.push_str(&interpolations(&contenu));
                code.push(' ');
                litteraux.push(contenu);
                i = j;
                continue;
            }
        }
        // Chaîne ordinaire (et `b"…"`).
        if c[i] == '"' {
            let mut j = i + 1;
            let mut contenu = String::new();
            while j < c.len() && c[j] != '"' {
                if c[j] == '\\' && j + 1 < c.len() {
                    contenu.push(c[j]);
                    contenu.push(c[j + 1]);
                    j += 2;
                    continue;
                }
                contenu.push(c[j]);
                j += 1;
            }
            code.push(' ');
            code.push_str(&interpolations(&contenu));
            code.push(' ');
            litteraux.push(contenu);
            i = j + 1;
            continue;
        }
        // Caractère (`'('`, `'\''`) — une durée de vie (`'a`) reste du code.
        if c[i] == '\'' {
            if c.get(i + 1) == Some(&'\\') {
                // `i + 3` : le caractère échappé (`i + 2`) peut être lui-même
                // une apostrophe, `'\''` (revue de code P1, B-3).
                let mut j = i + 3;
                while j < c.len() && c[j] != '\'' {
                    j += 1;
                }
                code.push(' ');
                i = j + 1;
                continue;
            }
            if c.get(i + 2) == Some(&'\'') {
                code.push(' ');
                i += 3;
                continue;
            }
        }
        code.push(if c[i] == '\n' { ' ' } else { c[i] });
        i += 1;
    }
    (code, litteraux)
}

/// Les interpolations `{ident…}` d'un littéral de format, séparées par des
/// espaces (`{{` sauté).
fn interpolations(litteral: &str) -> String {
    let c: Vec<char> = litteral.chars().collect();
    let mut v = Vec::new();
    let mut i = 0;
    while i < c.len() {
        if c[i] == '{' {
            if c.get(i + 1) == Some(&'{') {
                i += 2;
                continue;
            }
            let nom: String = c[i + 1..]
                .iter()
                .take_while(|ch| ch.is_alphanumeric() || **ch == '_')
                .collect();
            if !nom.is_empty() {
                v.push(nom);
            }
        }
        i += 1;
    }
    v.join(" ")
}

/// Les arguments (de la parenthèse ouvrante à celle qui lui répond) de
/// chaque appel d'une macro d'évènement de `tracing` du squelette.
fn appels_d_evenement(code: &str) -> Vec<String> {
    let tete = re(r"\b(trace|debug|info|warn|error|event)\s*!\s*\(");
    let octets: Vec<char> = code.chars().collect();
    let mut v = Vec::new();
    for m in tete.find_iter(code) {
        // `find_iter` donne des positions en octets ; on repart en caractères.
        let debut = code[..m.end()].chars().count();
        let mut profondeur = 1;
        let mut j = debut;
        while j < octets.len() && profondeur > 0 {
            match octets[j] {
                '(' => profondeur += 1,
                ')' => profondeur -= 1,
                _ => {}
            }
            j += 1;
        }
        v.push(octets[debut..j.saturating_sub(1)].iter().collect());
    }
    v
}

/// Les identifiants que ni un évènement ni un span ne doivent nommer.
const IDENTS_DU_CONTENU: &str = r"\b(content|contenu|resume|summary|texte|body)\b";

/// Les appels d'évènement de `source` qui nomment le contenu.
fn evenements_fautifs(source: &str) -> Vec<String> {
    let (code, _) = squelette(source);
    let vises = re(IDENTS_DU_CONTENU);
    appels_d_evenement(&code)
        .into_iter()
        .filter(|a| vises.is_match(a))
        .collect()
}

/// Le périmètre des gardes du chemin des envois : tout `src/mcp/`, le service,
/// les routes et les poignées des envois — et, constat P3-F3, l'entité et le
/// dépôt de `kesh-db`. Rend (chemin lisible, texte).
fn chemin_des_envois() -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = sources_src()
        .into_iter()
        .filter(|(f, _)| {
            f.starts_with("mcp/")
                || f == "services/envois_mcp.rs"
                || f == "routes/mcp_envois.rs"
                || f == "services/poignees.rs"
        })
        .collect();
    for f in [
        "kesh-db/src/entities/mcp_send.rs",
        "kesh-db/src/repositories/mcp_sends.rs",
    ] {
        v.push((
            f.to_string(),
            std::fs::read_to_string(racine_crates().join(f)).unwrap(),
        ));
    }
    assert!(
        v.iter().any(|(f, _)| f == "services/envois_mcp.rs")
            && v.iter().any(|(f, _)| f == "mcp/repartiteur.rs")
            && v.len() >= 16,
        "périmètre lu : {:?}",
        v.iter().map(|(f, _)| f).collect::<Vec<_>>()
    );
    v
}

/// AC-B12 — aucun évènement `tracing` du chemin des envois ne nomme le contenu
/// ni le résumé (les chemins d'erreur, que le test par capture n'exécute pas).
#[test]
fn aucun_evenement_ne_nomme_le_contenu() {
    // Rouges factices : chaque forme de l'AC-B12.
    for rouge in [
        "warn!(contenu = %c, \"x\");",
        "tracing::error!(\"{}\", contenu);",
        "info!(\"purge : {contenu}\");",
        "tracing::warn!(\n    company_id,\n    \"purge : {}\",\n    contenu\n);",
        // l'appariement saute les littéraux : une parenthèse orpheline dans
        // le message ne ferme pas l'appel (P2, F12)
        "warn!(\"échec :) {}\", contenu)",
        // constat P3-F4 : un caractère `)`, une durée de vie, une chaîne brute,
        // des délimiteurs `[]` et `{}`
        "warn!(x = ')', \"y {}\", contenu)",
        "fn f<'a>(s: &'a str) { warn!(\"{}\", contenu); }",
        "warn!(r#\"a ) b\"#, contenu)",
        "warn!(\"{}\", [contenu][0])",
        "warn!(\"{}\", { body })",
        "debug!(\"{:?}\", summary)",
        // revue de code P1, B-3 : `'\''` collé à un caractère `')'` — lue
        // `'\'` puis `','`, l'appariement fermait la macro avant `contenu`
        "warn!(a = ('\\'',')'), contenu)",
    ] {
        assert_eq!(evenements_fautifs(rouge).len(), 1, "rouge : {rouge}");
    }
    // Verts factices : la prose d'un littéral ne compte pas.
    for vert in [
        "warn!(\"le contenu est purgé\")",
        "warn!(r#\"le contenu ( purgé\"#)",
        "error!(\"{{contenu}} littéral\")",
        "let contenu = 1; warn!(\"x\");",
    ] {
        assert!(evenements_fautifs(vert).is_empty(), "vert : {vert}");
    }
    // Témoin vert existant : l'évènement multi-ligne de `cles_mcp.rs`.
    let cles = std::fs::read_to_string(racine_src().join("services/cles_mcp.rs")).unwrap();
    let (code, _) = squelette(&cles);
    assert_eq!(appels_d_evenement(&code).len(), 1, "cles_mcp.rs lu");
    assert!(evenements_fautifs(&cles).is_empty());

    let mut appels = 0;
    let mut fautifs = Vec::new();
    for (f, t) in chemin_des_envois() {
        let (code, _) = squelette(&t);
        appels += appels_d_evenement(&code).len();
        fautifs.extend(evenements_fautifs(&t).into_iter().map(|a| (f.clone(), a)));
    }
    assert!(appels >= 10, "témoin : {appels} appels d'évènement lus");
    assert!(
        fautifs.is_empty(),
        "évènement qui nomme le contenu : {fautifs:?}"
    );
}

/// AC-B12 — aucun span, ni `#[instrument]`, dans le chemin des envois : un span
/// enregistrerait par défaut chaque argument de la fonction (P2, F4).
#[test]
fn aucun_span_dans_le_chemin_des_envois() {
    let motif = re(
        r"#\s*\[\s*(tracing\s*::\s*)?instrument\b|\b(span|trace_span|debug_span|info_span|warn_span|error_span)\s*!\s*\(",
    );
    let compte = |source: &str| motif.find_iter(&squelette(source).0).count();
    assert_eq!(
        compte("#[tracing::instrument]\nfn deposer(contenu: &str) {}"),
        1,
        "rouge factice"
    );
    assert_eq!(
        compte("#[instrument(skip_all)]\nfn f() {}"),
        1,
        "rouge factice"
    );
    let exports = std::fs::read_to_string(racine_src().join("routes/exports.rs")).unwrap();
    assert!(compte(&exports) >= 1, "rouge existant : routes/exports.rs");
    assert_eq!(compte("use tracing::Instrument;"), 0, "vert existant");
    assert_eq!(
        compte("    fut.instrument(span.clone()).await"),
        0,
        "vert existant"
    );
    let fautifs: Vec<String> = chemin_des_envois()
        .into_iter()
        .filter(|(_, t)| compte(t) > 0)
        .map(|(f, _)| f)
        .collect();
    assert!(
        fautifs.is_empty(),
        "span dans le chemin des envois : {fautifs:?}"
    );
}

/// AC-B14 — le dépôt des envois n'est importé ou nommé qu'en ses deux lieux :
/// les poignées (`services/poignees.rs`) et les routes de décision
/// (`routes/mcp_envois.rs`) ; et le nom de la table n'est écrit dans aucun
/// littéral hors de l'entrée de `TABLES_HORS_EXPORT`.
#[test]
fn envois_lus_par_leurs_lieux() {
    let chemin = re(r"\brepositories\s*::\s*mcp_sends\b");
    let groupe = re(r"\brepositories\s*::\s*\{[^;]*\bmcp_sends\b");
    let etoile = re(r"\brepositories\s*::\s*\*");
    let nomme = |source: &str| {
        let (code, _) = squelette(source);
        chemin.is_match(&code) || groupe.is_match(&code) || etoile.is_match(&code)
    };
    for rouge in [
        "use kesh_db::repositories::{accounts, mcp_sends};",
        "use kesh_db::repositories::{\n    accounts,\n    mcp_sends,\n};",
        "use kesh_db::repositories::{invoices::{self}, mcp_sends};",
        "    kesh_db::repositories::mcp_sends::trouver(p, 1, 2)",
        "use kesh_db::repositories::*;",
    ] {
        assert!(nomme(rouge), "rouge : {rouge}");
    }
    for vert in [
        "use kesh_db::repositories::mcp_keys;",
        "use kesh_db::repositories::{api_keys, audit_log};",
        "use kesh_db::entities::mcp_send::NouvelEnvoi;",
    ] {
        assert!(!nomme(vert), "vert : {vert}");
    }
    let lieux: Vec<String> = sources_src()
        .into_iter()
        .filter(|(_, t)| nomme(t))
        .map(|(f, _)| f)
        .collect();
    assert_eq!(
        lieux,
        vec![
            "routes/mcp_envois.rs".to_string(),
            "services/poignees.rs".to_string()
        ],
        "le dépôt des envois nommé hors de ses lieux"
    );
    // Le nom de la table dans un littéral (P2, F11).
    let table = re(r"\bmcp_sends\b");
    let dans_litteraux = |source: &str| {
        squelette(source)
            .1
            .iter()
            .filter(|l| table.is_match(l))
            .count()
    };
    assert_eq!(
        dans_litteraux("sqlx::query(\"SELECT content FROM mcp_sends WHERE id = ?\")"),
        1,
        "rouge factice"
    );
    assert_eq!(
        dans_litteraux("let x = \"mcp_sends_purges\"; let y = \"mcp_keys_revoked\";"),
        0,
        "vert : mot entier"
    );
    let trouves: Vec<String> = sources_src()
        .into_iter()
        .filter(|(_, t)| dans_litteraux(t) > 0)
        .map(|(f, _)| f)
        .collect();
    assert_eq!(
        trouves,
        vec!["exports/global.rs".to_string()],
        "le nom de la table hors de l'entrée de TABLES_HORS_EXPORT"
    );
    let global = std::fs::read_to_string(racine_src().join("exports/global.rs")).unwrap();
    assert_eq!(
        dans_litteraux(&global),
        1,
        "une seule entrée, celle de TABLES_HORS_EXPORT"
    );
}

/// AC-B14 — `ContenuValide` et `AccuseEnvoi` ne se construisent, ne
/// s'implémentent et ne se dérivent qu'en `services/envois_mcp.rs`, sans
/// `Serialize`, `Default`, `Debug` ni `Clone`.
#[test]
fn envoi_valide_construit_par_le_service_seul() {
    let motif = re(
        r"\b(ContenuValide|AccuseEnvoi)\s*\{|\b(ContenuValide|AccuseEnvoi)\s*::|\bimpl\b[^{]*\b(ContenuValide|AccuseEnvoi)\b",
    );
    assert_eq!(
        sites_factices(
            "    let c = ContenuValide { numero: 1, modifie: false, texte: s };",
            &motif
        ),
        1,
        "rouge factice"
    );
    assert_eq!(
        sites_factices("impl Clone for AccuseEnvoi {", &motif),
        1,
        "rouge factice"
    );
    assert_eq!(
        sites_factices("    Valide(ContenuValide),", &motif),
        0,
        "vert : une variante qui le porte"
    );
    assert_eq!(
        sites_factices("    pub fn texte(&self) -> &str {", &motif),
        0,
        "vert factice : un accesseur dans l'impl"
    );
    let hors: Vec<_> = sites(&sources_src(), |f| f != "services/envois_mcp.rs", &motif);
    assert!(hors.is_empty(), "construit hors du service : {hors:?}");
    let service = std::fs::read_to_string(racine_src().join("services/envois_mcp.rs")).unwrap();
    assert!(
        sites(
            &[("services/envois_mcp.rs".to_string(), service.clone())],
            tout,
            &motif
        )
        .len()
            >= 3,
        "témoin : le service les construit"
    );
    for nom in ["ContenuValide", "AccuseEnvoi"] {
        let fautifs = interdits_dans(
            &derives_de(&service, nom),
            &["Serialize", "Default", "Debug", "Clone"],
        );
        assert!(fautifs.is_empty(), "{nom} : {fautifs:?}");
    }
}
