//! Story 32-10a-ii (refs #723) — le catalogue des prompts servi par `/mcp` :
//! `prompts/list` et `prompts/get` (AC-ii3 à AC-ii9).
//!
//! Tout passe par `tower::ServiceExt::oneshot` sur le routeur de production
//! (`build_router_avec_registre`), comme les tests de la 32-6a, ou par le
//! service `services::prompts_mcp` pour les règles R7 (refus avant toute
//! lecture, contrôle positif, exécution sur le pool en lecture seule).
//!
//! ⛔ « Égal » veut dire égal octet pour octet : chaque réponse de `/mcp` est
//! comparée **entière** à la réponse attendue ; l'absence d'un piège ne
//! s'emploie qu'en complément, pour un piège que le test a posé.
//!
//! Aucune donnée réelle : fixtures synthétiques et valeurs piégées
//! (`PIEGE_*_3210A`). Pré-requis : MariaDB démarré (`sqlx::test` crée une base
//! éphémère par test).

mod common;

use std::collections::BTreeSet;

use axum::Router;
use kesh_api::mcp::Registre;
use kesh_api::mcp::textes::*;
use kesh_api::services::acces::{ContexteAcces, Portee};
use kesh_api::services::poignees::LecturePrompts;
use kesh_api::services::prompts_mcp::{self, ErreurPrompts, marque_agent};
use kesh_db::entities::mcp_prompt::{Contenu, PLAFOND_A_VALIDER, VersionEcrite};
use kesh_db::entities::mcp_prompt_standards::{KESH_ASSISTANT, KESH_DEVELOPPEMENT, STANDARDS};
use kesh_db::entities::{McpScope, SendValidation};
use kesh_db::repositories::mcp_prompts as depot;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::capture_trace::CaptureTrace;
use common::lecture_seule::pool_lecture_seule;
use common::mcp::*;

const PIEGE_ARG: &str = "PIEGE_ARG_3210A";
const PIEGE_INVALIDE: &str = "PIEGE_INVALIDE_3210A";
const PIEGE_AUTRE: &str = "PIEGE_AUTRE_3210A";
const PIEGE_ATTENTE: &str = "PIEGE_ATTENTE_3210A";
const PIEGE_REFUSE: &str = "PIEGE_REFUSE_3210A";
const PIEGE_RETIRE: &str = "PIEGE_RETIRE_3210A";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

struct Banc {
    app: Router,
    _statique: tempfile::TempDir,
}

fn banc(pool: &MySqlPool) -> Banc {
    let statique = static_dir();
    Banc {
        app: routeur(
            pool.clone(),
            config(),
            statique.path(),
            Registre::standard(),
            "127.0.0.1",
        ),
        _statique: statique,
    }
}

impl Banc {
    async fn mcp(&self, jeton: &str, corps: &str) -> Rendu {
        envoyer(&self.app, post_mcp(Some(jeton), corps)).await
    }

    async fn liste(&self, jeton: &str) -> Value {
        self.mcp(jeton, r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#)
            .await
            .json()
    }

    async fn obtenir(&self, jeton: &str, params: Value) -> Value {
        let corps = json!({"jsonrpc":"2.0","id":1,"method":"prompts/get","params":params});
        self.mcp(jeton, &corps.to_string()).await.json()
    }
}

/// Le `result` attendu d'une ligne de `prompts/list`.
fn ligne(nom: &str, description: &str) -> Value {
    json!({"name": nom, "description": description, "arguments": []})
}

fn liste_attendue(lignes: Vec<Value>) -> Value {
    json!({"jsonrpc":"2.0","id":1,"result":{"prompts": lignes}})
}

fn get_attendu(description: &str, texte: &str) -> Value {
    json!({"jsonrpc":"2.0","id":1,"result":{
        "description": description,
        "messages": [{"role": "user", "content": {"type": "text", "text": texte}}],
    }})
}

fn inconnu() -> Value {
    attendu_erreur(1, -32602, TEXTE_PROMPT_INCONNU)
}

/// Un prompt d'administrateur.
async fn admin(pool: &MySqlPool, c: i64, u: i64, nom: &str, d: &str, t: &str) -> VersionEcrite {
    let mut tx = pool.begin().await.unwrap();
    let v = depot::creer_prompt_admin_in_tx(&mut tx, c, u, nom, d, t)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    v
}

/// Une proposition d'agent, laissée à valider.
async fn proposer(
    pool: &MySqlPool,
    c: i64,
    cle: i64,
    nom: &str,
    d: &str,
    t: &str,
) -> VersionEcrite {
    let mut tx = pool.begin().await.unwrap();
    let v = depot::proposer_version_agent_in_tx(&mut tx, c, cle, nom, d, t, PLAFOND_A_VALIDER)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    v
}

/// Approuve (`true`) ou refuse la proposition `v`.
async fn decider(pool: &MySqlPool, c: i64, u: i64, v: &VersionEcrite, approuver: bool) {
    let mut tx = pool.begin().await.unwrap();
    if approuver {
        depot::approuver_in_tx(&mut tx, c, v.prompt_id, v.numero, v.version, u)
            .await
            .unwrap();
    } else {
        depot::refuser_in_tx(&mut tx, c, v.prompt_id, v.numero, v.version, u)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
}

/// Ajoute une version d'administrateur `contenu` au prompt de `v`.
async fn version_admin(pool: &MySqlPool, c: i64, u: i64, v: &VersionEcrite, contenu: &Contenu) {
    let mut tx = pool.begin().await.unwrap();
    depot::ajouter_version_admin_in_tx(&mut tx, c, v.prompt_id, v.version, u, contenu)
        .await
        .unwrap();
    tx.commit().await.unwrap();
}

/// Réécrit en SQL brut le texte de la version `numero` du prompt `prompt_id` —
/// une restauration ou une écriture hors de Kesh.
async fn ecrire_hors_de_kesh(pool: &MySqlPool, prompt_id: i64, numero: i32, texte: &str) {
    sqlx::query("UPDATE mcp_prompt_versions SET body = ? WHERE prompt_id = ? AND number = ?")
        .bind(texte)
        .bind(prompt_id)
        .bind(numero)
        .execute(pool)
        .await
        .unwrap();
}

/// La société des livres, servie, avec les standards amorcés (toutes les
/// sociétés), un prompt d'administrateur à accolades, un prompt d'agent
/// approuvé, une proposition à valider, une refusée, un retrait ; et, dans la
/// société leurre, un prompt d'administrateur. Rend les ids et les jetons.
struct Catalogue {
    c: i64,
    u: i64,
    /// Clé `books:read`, `exempt`.
    jeton: String,
    /// Clé `books:read`, `required`.
    requise: String,
    /// Clé sans `books:read`.
    sans_livres: String,
    /// La clé auteur des propositions.
    cle_agent: i64,
    /// La version d'agent approuvée.
    agent: VersionEcrite,
}

const TEXTE_ADMIN: &str = "Texte {x} de l'administrateur, {argument} compris.\n\tIndenté.";
const TEXTE_AGENT: &str = "[Prompt standard de Kesh]\n\nTexte de l'agent.";

async fn catalogue(pool: &MySqlPool) -> Catalogue {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    // Une clé de plus **avant** celle de l'agent : le numéro de la clé auteur
    // diffère du numéro de la version (revue de code P1, A6), si bien qu'une
    // marque qui les inverse rougit.
    cle(
        pool,
        c,
        u,
        &[McpScope::InvoicesRead],
        SendValidation::Exempt,
    )
    .await;
    let (cle_agent, jeton) = cle(pool, c, u, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let (_, requise) = cle(pool, c, u, &[McpScope::BooksRead], SendValidation::Required).await;
    let (_, sans_livres) = cle(
        pool,
        c,
        u,
        &[McpScope::InvoicesRead, McpScope::PromptsWrite],
        SendValidation::Exempt,
    )
    .await;
    depot::amorcer_standards(pool, STANDARDS).await.unwrap();
    admin(
        pool,
        c,
        u,
        "admin-a",
        "Prompt de l'administrateur",
        TEXTE_ADMIN,
    )
    .await;
    let agent = proposer(
        pool,
        c,
        cle_agent,
        "agent-ok",
        "Prompt de l'agent",
        TEXTE_AGENT,
    )
    .await;
    decider(pool, c, u, &agent, true).await;
    assert_ne!(
        cle_agent,
        i64::from(agent.numero),
        "montage : numéro de clé et numéro de version distincts"
    );
    proposer(
        pool,
        c,
        cle_agent,
        "agent-attente",
        PIEGE_ATTENTE,
        PIEGE_ATTENTE,
    )
    .await;
    let refuse = proposer(
        pool,
        c,
        cle_agent,
        "agent-refuse",
        PIEGE_REFUSE,
        PIEGE_REFUSE,
    )
    .await;
    decider(pool, c, u, &refuse, false).await;
    let retire = admin(pool, c, u, "retire", PIEGE_RETIRE, PIEGE_RETIRE).await;
    version_admin(pool, c, u, &retire, &Contenu::Retrait).await;
    // La société leurre : la première société de `livres`.
    let leurre: i64 = sqlx::query_scalar("SELECT MIN(id) FROM companies")
        .fetch_one(pool)
        .await
        .unwrap();
    assert_ne!(leurre, c);
    let u_leurre = administrateur(pool, leurre, "admin-leurre").await;
    admin(pool, leurre, u_leurre, "autre", PIEGE_AUTRE, PIEGE_AUTRE).await;
    Catalogue {
        c,
        u,
        jeton,
        requise,
        sans_livres,
        cle_agent,
        agent,
    }
}

fn marquee(cat: &Catalogue, s: &str) -> String {
    format!("{}\n\n{s}", marque_agent(cat.agent.numero, cat.cle_agent))
}

/// La liste attendue du catalogue : quatre prompts servis, triés par nom.
fn liste_du_catalogue(cat: &Catalogue) -> Value {
    liste_attendue(vec![
        ligne("admin-a", "Prompt de l'administrateur"),
        ligne("agent-ok", &marquee(cat, "Prompt de l'agent")),
        ligne(KESH_ASSISTANT.nom, KESH_ASSISTANT.description),
        ligne(KESH_DEVELOPPEMENT.nom, KESH_DEVELOPPEMENT.description),
    ])
}

fn acces(c: i64, u: i64, portees: &[Portee]) -> ContexteAcces {
    ContexteAcces::mcp(c, u, portees.iter().copied().collect::<BTreeSet<_>>())
}

// ---------------------------------------------------------------------------
// AC-ii5 — prompts/list
// ---------------------------------------------------------------------------

/// Les prompts en vigueur, approuvés, de nature texte, valides, de la société
/// de la clé ; ni la proposition, ni la version refusée, ni le retrait, ni le
/// prompt de l'autre société (Mii2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prompts_list_corps_egal(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let b = banc(&pool);
    let r = b
        .mcp(
            &cat.jeton,
            r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#,
        )
        .await;
    assert_eq!(r.json(), liste_du_catalogue(&cat));
    let corps = String::from_utf8_lossy(&r.corps).to_string();
    for piege in [PIEGE_ATTENTE, PIEGE_REFUSE, PIEGE_RETIRE, PIEGE_AUTRE] {
        assert!(!corps.contains(piege), "{piege} servi");
    }
    // Sous une clé `required` : la même liste (AC-ii9, hors politique).
    assert_eq!(b.liste(&cat.requise).await, liste_du_catalogue(&cat));
    // Une clé sans `books:read` : une liste vide, non une erreur.
    assert_eq!(b.liste(&cat.sans_livres).await, liste_attendue(vec![]));
    // Un `cursor` : -32602, la liste n'est pas paginée.
    let r = b
        .mcp(
            &cat.jeton,
            r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list","params":{"cursor":"x"}}"#,
        )
        .await;
    assert_eq!(r.json(), attendu_erreur(1, -32602, TEXTE_PARAMS_INVALIDES));
}

/// Le tri de la liste est l'ordre des **octets** (C-32-10a-91 ; revue de code
/// P1, E2) : `-` (0x2D) < chiffres < `_` (0x5F) < lettres minuscules, alors
/// que la collation `utf8mb4_unicode_ci` du `ORDER BY` du dépôt range `_` et
/// `-` autrement. Trois noms qui ne diffèrent qu'à ces caractères.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tri_octet_pour_octet(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    for nom in ["a_b", "a1", "a-b"] {
        admin(&pool, cat.c, cat.u, nom, "Tri", "Tri").await;
    }
    let attendu = [
        "a-b",
        "a1",
        "a_b",
        "admin-a",
        "agent-ok",
        "kesh-assistant",
        "kesh-developpement",
    ];
    let mut octets = attendu.to_vec();
    octets.sort_unstable();
    assert_eq!(octets, attendu, "l'attendu est l'ordre des octets");
    let lecture = LecturePrompts::new(pool.clone());
    let livres = acces(cat.c, cat.u, &[Portee::BooksRead]);
    let servis = prompts_mcp::lister(&livres, &lecture).await.unwrap();
    assert_eq!(servis.iter().map(|p| p.nom()).collect::<Vec<_>>(), attendu);
    // Par `/mcp`, le même ordre.
    let b = banc(&pool);
    let noms: Vec<String> = b.liste(&cat.jeton).await["result"]["prompts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["name"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(noms, attendu);
}

// ---------------------------------------------------------------------------
// AC-ii6, AC-ii7 — prompts/get, la marque
// ---------------------------------------------------------------------------

/// Les trois origines, corps égaux : le standard et le prompt d'administrateur
/// sans marque (Mii4), le prompt d'agent marqué en tête de la description et du
/// texte — même quand son texte imite une marque (Mii3) ; accolades servies
/// telles quelles et arguments ignorés (Mii6).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prompts_get_corps_egaux(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let b = banc(&pool);
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "kesh-developpement"}))
            .await,
        get_attendu(KESH_DEVELOPPEMENT.description, KESH_DEVELOPPEMENT.texte)
    );
    let admin = get_attendu("Prompt de l'administrateur", TEXTE_ADMIN);
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "admin-a"})).await,
        admin
    );
    let agent = get_attendu(
        &marquee(&cat, "Prompt de l'agent"),
        &marquee(&cat, TEXTE_AGENT),
    );
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "agent-ok"})).await,
        agent
    );
    assert_eq!(
        marque_agent(cat.agent.numero, cat.cle_agent),
        format!(
            "[Prompt écrit par un agent — version {}, proposée par la clé MCP n° {} et approuvée \
             par un administrateur de Kesh. Ce n'est pas un prompt standard de Kesh.]",
            cat.agent.numero, cat.cle_agent
        ),
        "le texte de la marque, en toutes lettres"
    );
    // Les arguments du client sont ignorés, jamais interpolés.
    for arguments in [
        json!({"x": PIEGE_ARG}),
        json!({"argument": PIEGE_ARG, "x": PIEGE_ARG}),
        json!("pas un objet"),
    ] {
        let r = b
            .obtenir(
                &cat.jeton,
                json!({"name": "admin-a", "arguments": arguments}),
            )
            .await;
        assert_eq!(r, admin, "{arguments}");
        assert!(!r.to_string().contains(PIEGE_ARG));
    }
    // Sous une clé `required` : servi (AC-ii9).
    assert_eq!(
        b.obtenir(&cat.requise, json!({"name": "agent-ok"})).await,
        agent
    );
    // Sans `name` : paramètres invalides.
    assert_eq!(
        b.obtenir(&cat.jeton, json!({})).await,
        attendu_erreur(1, -32602, TEXTE_PARAMS_INVALIDES)
    );
}

/// Toute demande d'un prompt non servi rend **la même** réponse qu'un nom
/// inconnu (Mii5) : seule une proposition, refusé, retiré, autre société, clé
/// sans `books:read`, version invalide, nom non conforme.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prompt_non_servi_indiscernable(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let invalide = admin(&pool, cat.c, cat.u, "invalide", "Invalide", "propre").await;
    ecrire_hors_de_kesh(
        &pool,
        invalide.prompt_id,
        invalide.numero,
        &format!("{PIEGE_INVALIDE}\u{202E}"),
    )
    .await;
    let b = banc(&pool);
    for nom in [
        "inconnu",
        "agent-attente",
        "agent-refuse",
        "retire",
        "autre",
        "invalide",
        "KESH-ASSISTANT",
        "kesh-assistant ",
        "kesh-assistant\n",
    ] {
        let r = b.obtenir(&cat.jeton, json!({ "name": nom })).await;
        assert_eq!(r, inconnu(), "{nom:?}");
    }
    let r = b
        .obtenir(&cat.sans_livres, json!({"name": "kesh-assistant"}))
        .await;
    assert_eq!(r, inconnu(), "clé sans books:read");
    // Témoin : le même nom est servi à une clé `books:read`.
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "kesh-assistant"}))
            .await,
        get_attendu(KESH_ASSISTANT.description, KESH_ASSISTANT.texte)
    );
}

// ---------------------------------------------------------------------------
// AC-ii3 — revalidation au service
// ---------------------------------------------------------------------------

/// Une version écrite hors de Kesh avec un caractère refusé — U+202E
/// (bidirectionnel, Mii7) ou U+E0100 (sélecteur de variante, Mii10) — n'est
/// pas servie ; le journal du serveur le dit sans porter le texte. Un `\r\n`
/// restauré est servi **normalisé**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn version_invalide_non_servie(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let bidi = admin(&pool, cat.c, cat.u, "bidi", "Bidi", "propre").await;
    ecrire_hors_de_kesh(
        &pool,
        bidi.prompt_id,
        1,
        &format!("a\u{202E}{PIEGE_INVALIDE}"),
    )
    .await;
    let selecteur = admin(&pool, cat.c, cat.u, "selecteur", "Sélecteur", "propre").await;
    ecrire_hors_de_kesh(
        &pool,
        selecteur.prompt_id,
        1,
        &format!("a\u{E0100}{PIEGE_INVALIDE}"),
    )
    .await;
    let crlf = admin(&pool, cat.c, cat.u, "crlf", "Fins de ligne", "propre").await;
    ecrire_hors_de_kesh(&pool, crlf.prompt_id, 1, "ligne 1\r\nligne 2\r\n").await;
    let b = banc(&pool);
    let capture = CaptureTrace::installer();
    let mut attendue = liste_du_catalogue(&cat);
    attendue["result"]["prompts"]
        .as_array_mut()
        .unwrap()
        .insert(2, ligne("crlf", "Fins de ligne"));
    assert_eq!(b.liste(&cat.jeton).await, attendue);
    for nom in ["bidi", "selecteur"] {
        assert_eq!(
            b.obtenir(&cat.jeton, json!({ "name": nom })).await,
            inconnu(),
            "{nom}"
        );
    }
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "crlf"})).await,
        get_attendu("Fins de ligne", "ligne 1\nligne 2\n")
    );
    capture.exiger_ensemble(&[
        "prompt MCP non servi : contenu invalide",
        &format!("prompt_id={}", bidi.prompt_id),
        "numero=1",
    ]);
    capture.exiger(&format!("prompt_id={}", selecteur.prompt_id));
    capture.exiger_absent(PIEGE_INVALIDE);
}

/// Une erreur de lecture rend l'erreur interne fixe, sans détail.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn lecture_en_echec_erreur_interne(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    sqlx::query("RENAME TABLE mcp_prompt_versions TO mcp_prompt_versions_ailleurs")
        .execute(&pool)
        .await
        .unwrap();
    let b = banc(&pool);
    let interne = attendu_erreur(1, -32603, TEXTE_ERREUR_INTERNE);
    assert_eq!(b.liste(&cat.jeton).await, interne);
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "kesh-assistant"}))
            .await,
        interne
    );
    // Sans `books:read`, aucune lecture : la liste vide, le prompt inconnu.
    assert_eq!(b.liste(&cat.sans_livres).await, liste_attendue(vec![]));
    assert_eq!(
        b.obtenir(&cat.sans_livres, json!({"name": "kesh-assistant"}))
            .await,
        inconnu()
    );
}

// ---------------------------------------------------------------------------
// R7 — le service
// ---------------------------------------------------------------------------

/// Refus **avant toute lecture** : la table des versions renommée ne fait pas
/// échouer l'appel d'une clé sans `books:read` (`HorsPortee`), alors qu'elle
/// fait échouer celui d'une clé qui l'a (`Interne`, contrôle positif) ; un nom
/// non conforme rend `None` sans lire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn service_refus_avant_toute_lecture(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let lecture = LecturePrompts::new(pool.clone());
    let hors = acces(cat.c, cat.u, &[Portee::InvoicesRead, Portee::PromptsWrite]);
    let livres = acces(cat.c, cat.u, &[Portee::BooksRead]);
    // Témoin positif avant le renommage.
    assert_eq!(
        prompts_mcp::lister(&livres, &lecture).await.unwrap().len(),
        4
    );
    sqlx::query("RENAME TABLE mcp_prompt_versions TO mcp_prompt_versions_ailleurs")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        prompts_mcp::lister(&hors, &lecture).await.map(|v| v.len()),
        Err(ErreurPrompts::HorsPortee)
    );
    assert_eq!(
        prompts_mcp::obtenir(&hors, &lecture, "kesh-assistant")
            .await
            .map(|p| p.is_some()),
        Err(ErreurPrompts::HorsPortee)
    );
    assert_eq!(
        prompts_mcp::lister(&livres, &lecture)
            .await
            .map(|v| v.len()),
        Err(ErreurPrompts::Interne),
        "contrôle positif : la clé qui a la portée lit"
    );
    assert_eq!(
        prompts_mcp::obtenir(&livres, &lecture, "kesh-assistant")
            .await
            .map(|p| p.is_some()),
        Err(ErreurPrompts::Interne)
    );
    for nom in ["KESH-ASSISTANT", "kesh-assistant ", ""] {
        assert_eq!(
            prompts_mcp::obtenir(&livres, &lecture, nom)
                .await
                .map(|p| p.is_some()),
            Ok(false),
            "{nom:?} : aucune lecture"
        );
    }
}

/// Le service s'exécute sur une `LecturePrompts` posée sur le pool en lecture
/// seule du MCP, et rend la société de la clé seulement.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn service_sur_lecture_seule(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let lecture = LecturePrompts::new(pool_lecture_seule(&pool).await);
    let livres = acces(cat.c, cat.u, &[Portee::BooksRead]);
    let servis = prompts_mcp::lister(&livres, &lecture).await.unwrap();
    assert_eq!(
        servis.iter().map(|p| p.nom()).collect::<Vec<_>>(),
        [
            "admin-a",
            "agent-ok",
            "kesh-assistant",
            "kesh-developpement"
        ]
    );
    let agent = &servis[1];
    assert_eq!(agent.numero(), cat.agent.numero);
    assert_eq!(agent.cle_auteur(), Some(cat.cle_agent));
    assert_eq!(agent.texte(), marquee(&cat, TEXTE_AGENT));
    let p = prompts_mcp::obtenir(&livres, &lecture, "kesh-developpement")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        (p.description(), p.texte(), p.cle_auteur()),
        (
            KESH_DEVELOPPEMENT.description,
            KESH_DEVELOPPEMENT.texte,
            None
        )
    );
    assert!(
        prompts_mcp::obtenir(&livres, &lecture, "autre")
            .await
            .unwrap()
            .is_none(),
        "le prompt de l'autre société"
    );
}
