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
/// chemin** (non précédé d'un identifiant ni de `::`). Ne voit pas
/// `ContexteAcces::mcp(`, `portee_mcp`, `cles_mcp::`, `serveur_mcp::`.
const MOTIF_MODULE_MCP: &str =
    r"\bcrate\s*::\s*mcp\b|\bsuper\s*::\s*mcp\b|(^|[^A-Za-z0-9_:])mcp\s*::";

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
        "let pool_outils = kesh_db::pool::pool_lecture_seule(&state.pool, mcp::MCP_POOL_CONNEXIONS);",
    ),
    (
        "lib.rs",
        "mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils));",
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
        "pub(crate) fn autoriser(validation: SendValidation) -> Result<Autorisation, Refus> {",
        "pub(crate) fn appliquer(_autorisation: Autorisation, donnees: Donnees) -> DonneesEnvoyables {",
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
            "let ctx = ContexteMcp::nouveau(&cle.acces, &etat.lecture);"
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
    // sont hors du compte.
    let motif = re(r"\bTcpListener\b|\bTcpSocket\b|\bUnixListener\b");
    for rouge in [
        "    let l = tokio::net::TcpListener::bind(a).await?;",
        "    let l = std::net::TcpListener::bind(a)?;",
        "    let s = tokio::net::TcpSocket::new_v4()?;",
        "    let u = tokio::net::UnixListener::bind(p)?;",
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
            "let pool_outils = kesh_db::pool::pool_lecture_seule(&state.pool, mcp::MCP_POOL_CONNEXIONS);"
        )])
    );
}

// ---------------------------------------------------------------------------
// AC-a30 — mcp_sans_pool (N19, N27)
// ---------------------------------------------------------------------------

const MOTIF_SANS_POOL: &str = r"\bMySqlPool\b|\bsqlx\b|\bAppState\b|\.\s*pool\b|\bkesh_db\s*::\s*repositories\b|\bkesh_db\s*::\s*pool\b|\bcreate_pool\b|\bdatabase_url\b|\bConfig\b";

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
            "mcp::LectureOutils::nouvelle(services::poignees::Lecture::new(pool_outils));"
        )])
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

/// Les attributs `#[derive(…)]` qui précèdent `struct <nom>` (commentaires de
/// doc sautés).
fn derives_de(source: &str, nom: &str) -> Vec<String> {
    let lignes: Vec<&str> = source.lines().collect();
    let tete = re(&format!(
        r"^\s*pub(\([^)]*\))?\s+struct\s+{nom}\b|^\s*struct\s+{nom}\b"
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
    for (nom, interdits) in [
        ("Donnees", &["Serialize", "Default", "Debug"][..]),
        ("DonneesEnvoyables", &["Serialize", "Default", "Debug"][..]),
        (
            "Autorisation",
            &["Serialize", "Default", "Debug", "Clone", "Copy"][..],
        ),
    ] {
        let fautifs = interdits_dans(&derives_de(&politique, nom), interdits);
        assert!(fautifs.is_empty(), "{nom} : {fautifs:?}");
    }
    let imp = re(
        r"\bimpl\b[^{]*\b(Default|Serialize|Debug)\b[^{]*\bfor\s+(Donnees|DonneesEnvoyables|Autorisation)\b|\bimpl\b[^{]*\b(Clone|Copy)\b[^{]*\bfor\s+Autorisation\b",
    );
    for rouge in [
        "impl Default for Donnees {",
        "impl serde::Serialize for DonneesEnvoyables {",
        "impl std::fmt::Debug for Autorisation {",
        "impl Clone for Autorisation {",
    ] {
        assert_eq!(sites_factices(rouge, &imp), 1, "rouge : {rouge}");
    }
    assert_eq!(
        sites_factices("impl Clone for Donnees {", &imp),
        0,
        "vert : permis"
    );
    assert!(sites(&sources_src(), tout, &imp).is_empty());
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
/// READ ONLY`, et le nom de la variable de session, quelle que soit la casse.
const MOTIF_LECTURE_SEULE: &str =
    r"(?i)\bread\s+write\b|\btx_read_only\b|\btransaction_read_only\b";

/// Les lignes d'un texte, commentaires de bloc remplacés **par une espace**
/// (comme le fait MariaDB), lignes de commentaire (`//`, `--`, `#` en SQL)
/// écartées.
fn lignes_decommentees(texte: &str, sql: bool) -> Vec<String> {
    let sans_blocs = re(r"(?s)/\*.*?\*/").replace_all(texte, " ").into_owned();
    sans_blocs
        .lines()
        .filter(|l| {
            let t = l.trim_start();
            !(t.starts_with("//") || (sql && (t.starts_with("--") || t.starts_with('#'))))
        })
        .map(str::to_string)
        .collect()
}

fn formes_interdites(texte: &str, sql: bool) -> Vec<String> {
    let motif = re(MOTIF_LECTURE_SEULE);
    lignes_decommentees(texte, sql)
        .into_iter()
        .filter(|l| motif.is_match(l))
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
    ] {
        assert!(formes_interdites(vert, true).is_empty(), "vert : {vert}");
    }

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
                "let mode: i64 = sqlx::query_scalar(\"SELECT @@session.tx_read_only\")"
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
