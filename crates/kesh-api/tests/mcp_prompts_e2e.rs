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
use std::time::{Duration, Instant};

use axum::Router;
use axum::http::StatusCode;
use kesh_api::mcp::Registre;
use kesh_api::mcp::textes::*;
use kesh_api::services::acces::{ContexteAcces, Portee};
use kesh_api::services::cles_mcp::authentifier;
use kesh_api::services::poignees::{EcriturePrompts, Lecture, LecturePrompts, Trace};
use kesh_api::services::prompts_mcp::{self, ErreurPrompts, RefusProposition, marque_agent};
use kesh_db::entities::mcp_prompt::{Contenu, PLAFOND_A_VALIDER, VersionEcrite};
use kesh_db::entities::mcp_prompt_standards::{KESH_ASSISTANT, KESH_DEVELOPPEMENT, STANDARDS};
use kesh_db::entities::{McpScope, Role, SendValidation};
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

// ===========================================================================
// Story 32-10a-iii — la proposition d'un agent, le journal de `prompts/get`
// ===========================================================================

const PIEGE_TEXTE: &str = "PIEGE_TEXTE_3210A";
const PIEGE_DESC: &str = "PIEGE_DESC_3210A";
const PIEGE_NOM: &str = "PIEGE_NOM_3210A";

/// Les arguments d'une proposition.
fn args(nom: &str, description: &str, texte: &str) -> Value {
    json!({ "nom": nom, "description": description, "texte": texte })
}

/// L'accusé attendu d'une proposition de `nom` (AC-iii3) : le nom validé
/// seul.
fn accuse(nom: &str) -> Value {
    json!({"jsonrpc":"2.0","id":1,"result":{
        "content": [{"type": "text", "text": texte_accuse_proposition(nom)}],
        "structuredContent": {"nom": nom, "etat": "a_valider"},
        "isError": false,
    }})
}

/// Les lignes d'une action du journal d'audit, dans l'ordre, sans `dureeMs`
/// (dont le type est contrôlé).
async fn journal(pool: &MySqlPool, action: &str) -> Vec<Value> {
    let lignes: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap();
    lignes
        .into_iter()
        .map(|d| {
            let mut v: Value = serde_json::from_str(&d).unwrap();
            if let Some(duree) = v.as_object_mut().unwrap().remove("dureeMs") {
                assert!(duree.is_u64(), "dureeMs entier");
            }
            v
        })
        .collect()
}

/// Tout le journal d'audit, détails et libellés d'auteur, en texte.
async fn audit_en_texte(pool: &MySqlPool) -> String {
    let lignes: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT action, CAST(details_json AS CHAR), actor_label FROM audit_log ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    format!("{lignes:?}")
}

/// Le nombre de versions à valider de la base.
async fn a_valider(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM mcp_prompt_versions WHERE state = 'pending'")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Le nombre de versions de la base.
async fn versions(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM mcp_prompt_versions")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Une société servie, son Administrateur, une clé `books:read` +
/// `prompts:write` (`required`), sa session, et les standards amorcés.
struct Ecrivain {
    c: i64,
    u: i64,
    cle: i64,
    jeton: String,
    jwt: String,
}

async fn ecrivain(pool: &MySqlPool) -> Ecrivain {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    depot::amorcer_standards(pool, STANDARDS).await.unwrap();
    let (cle, jeton) = cle(
        pool,
        c,
        u,
        &[McpScope::BooksRead, McpScope::PromptsWrite],
        SendValidation::Required,
    )
    .await;
    Ecrivain {
        c,
        u,
        cle,
        jeton,
        jwt: session(u, Role::Admin, c),
    }
}

impl Banc {
    async fn proposer(&self, jeton: &str, a: Value) -> Rendu {
        self.mcp(jeton, &appel(1, OUTIL_PROPOSER_PROMPT, a)).await
    }

    async fn api(&self, methode: &str, chemin: &str, jwt: &str, corps: Option<Value>) -> Rendu {
        envoyer(&self.app, requete_api(methode, chemin, Some(jwt), corps)).await
    }
}

// ---------------------------------------------------------------------------
// AC-iii3 — la proposition, son accusé et son audit
// ---------------------------------------------------------------------------

/// Une proposition servie, **sous une clé `required`** (la politique ne
/// s'applique pas, Miii10) : l'accusé, égal ; une version `pending` à la clé,
/// au contenu **normalisé** (`\r\n` → `\n`) ; **exactement une** ligne au
/// journal, `mcp.tool_called`, détail égal ; aucun piège dans la réponse ni au
/// journal (Miii4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn proposition_servie_accuse_et_audit(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    let b = banc(&pool);
    let texte = format!("Ligne 1 {PIEGE_TEXTE}\r\nLigne 2");
    let r = b
        .proposer(&e.jeton, args("mon-prompt", PIEGE_DESC, &texte))
        .await;
    assert_eq!(r.json(), accuse("mon-prompt"));
    let corps = String::from_utf8_lossy(&r.corps).to_string();
    assert!(!corps.contains(PIEGE_TEXTE) && !corps.contains(PIEGE_DESC));
    let (origine, etat, cle, description, enregistre): (
        String,
        String,
        Option<i64>,
        String,
        String,
    ) = sqlx::query_as(
        "SELECT v.origin, v.state, v.author_mcp_key_id, v.description, v.body \
             FROM mcp_prompt_versions v JOIN mcp_prompts p ON p.id = v.prompt_id \
             WHERE p.name = 'mon-prompt' AND p.company_id = ?",
    )
    .bind(e.c)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        (origine.as_str(), etat.as_str(), cle),
        ("agent", "pending", Some(e.cle))
    );
    assert_eq!(description, PIEGE_DESC);
    assert_eq!(
        enregistre,
        format!("Ligne 1 {PIEGE_TEXTE}\nLigne 2"),
        "le contenu normalisé"
    );
    assert_eq!(
        journal(&pool, "mcp.tool_called").await,
        vec![json!({
            "cleMcpId": e.cle,
            "outil": "proposer_prompt",
            "issue": "servi",
            "validation": "required",
        })]
    );
    let audit = audit_en_texte(&pool).await;
    assert!(!audit.contains(PIEGE_TEXTE) && !audit.contains(PIEGE_DESC));
    // Une seconde proposition sur le même nom : une nouvelle version à valider
    // du même prompt (C-32-10a-27 : un nom non standard existant).
    let r = b
        .proposer(&e.jeton, args("mon-prompt", "Autre", "Autre texte"))
        .await;
    assert_eq!(r.json(), accuse("mon-prompt"));
    assert_eq!(a_valider(&pool).await, 2);
}

/// Chaque refus : corps égal, rien d'écrit, une ligne au journal (fail
/// closed), le piège nulle part (AC-iii2, point 1 ; AC-iii3 ; Miii1, Miii3,
/// Miii11).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn proposition_refus_corps_egaux(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    let b = banc(&pool);
    let (_, lecteur) = cle(
        &pool,
        e.c,
        e.u,
        &[McpScope::BooksRead],
        SendValidation::Exempt,
    )
    .await;
    let avant = versions(&pool).await;
    let invalides = attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES);
    let reserve = attendu_outil_en_erreur(1, TEXTE_NOM_RESERVE);
    let cas: Vec<(&str, &str, Value, Value)> = vec![
        // Sans `prompts:write` : le chemin d'un outil inconnu.
        (
            "sans portée",
            &lecteur,
            args("p", "d", PIEGE_TEXTE),
            attendu_erreur(1, -32602, TEXTE_OUTIL_INCONNU),
        ),
        (
            "champ en trop",
            &e.jeton,
            json!({"nom": "p", "description": "d", "texte": PIEGE_TEXTE, "x": 1}),
            invalides.clone(),
        ),
        (
            "texte absent",
            &e.jeton,
            json!({"nom": "p", "description": "d"}),
            invalides.clone(),
        ),
        (
            "description sur deux lignes",
            &e.jeton,
            args("p", "a\nb", PIEGE_TEXTE),
            invalides.clone(),
        ),
        (
            "texte bidirectionnel",
            &e.jeton,
            args("p", "d", &format!("{PIEGE_TEXTE}\u{202E}")),
            invalides.clone(),
        ),
        (
            "nom non conforme",
            &e.jeton,
            args("Mon-Prompt", "d", PIEGE_TEXTE),
            invalides.clone(),
        ),
        (
            "kesh-assistant",
            &e.jeton,
            args(KESH_ASSISTANT.nom, "d", PIEGE_TEXTE),
            reserve.clone(),
        ),
        (
            "kesh-developpement",
            &e.jeton,
            args(KESH_DEVELOPPEMENT.nom, "d", PIEGE_TEXTE),
            reserve.clone(),
        ),
        (
            "keshx",
            &e.jeton,
            args("keshx", "d", PIEGE_TEXTE),
            reserve.clone(),
        ),
    ];
    for (motif, jeton, a, attendu) in &cas {
        let r = b.proposer(jeton, a.clone()).await;
        assert_eq!(&r.json(), attendu, "{motif}");
        assert!(
            !String::from_utf8_lossy(&r.corps).contains(PIEGE_TEXTE),
            "{motif}"
        );
    }
    assert_eq!(versions(&pool).await, avant, "aucun refus n'écrit");
    let issues: Vec<(String, String)> = journal(&pool, "mcp.tool_called")
        .await
        .into_iter()
        .map(|d| {
            (
                d["outil"].as_str().unwrap().to_string(),
                d["issue"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    let attendues: Vec<(String, String)> = std::iter::once(("(inconnu)", "inconnu"))
        .chain(std::iter::repeat_n(
            ("proposer_prompt", "arguments_invalides"),
            8,
        ))
        .map(|(o, i)| (o.to_string(), i.to_string()))
        .collect();
    assert_eq!(issues, attendues);
    // Le standard est toujours servi tel quel.
    assert_eq!(
        b.obtenir(&e.jeton, json!({"name": KESH_ASSISTANT.nom}))
            .await,
        get_attendu(KESH_ASSISTANT.description, KESH_ASSISTANT.texte)
    );

    // Le plafond : vingt propositions à valider, la vingt et unième refusée.
    for n in 0..PLAFOND_A_VALIDER {
        let nom = format!("p{n:02}");
        assert_eq!(
            b.proposer(&e.jeton, args(&nom, "d", "t")).await.json(),
            accuse(&nom)
        );
    }
    assert_eq!(a_valider(&pool).await, i64::from(PLAFOND_A_VALIDER));
    let r = b.proposer(&e.jeton, args("p99", "d", PIEGE_TEXTE)).await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_PROPOSITIONS_TROP_NOMBREUSES)
    );
    assert_eq!(a_valider(&pool).await, i64::from(PLAFOND_A_VALIDER));
    let derniere = journal(&pool, "mcp.tool_called").await.pop().unwrap();
    assert_eq!(
        derniere,
        json!({"cleMcpId": e.cle, "outil": "proposer_prompt", "issue": "limite_atteinte",
               "validation": "required"})
    );
    assert!(!audit_en_texte(&pool).await.contains(PIEGE_TEXTE));
}

/// AC-iii3 — la version et son audit dans **une** transaction (Miii12) :
/// `audit_log` cachée, la proposition rend l'erreur interne fixe et **aucune**
/// version n'est enregistrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn proposition_et_audit_dans_la_meme_transaction(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    let b = banc(&pool);
    let avant = versions(&pool).await;
    sqlx::query("ALTER TABLE audit_log RENAME TO audit_log_cachee_3210a")
        .execute(&pool)
        .await
        .unwrap();
    let r = b.proposer(&e.jeton, args("p", "d", PIEGE_TEXTE)).await;
    assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE));
    sqlx::query("ALTER TABLE audit_log_cachee_3210a RENAME TO audit_log")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(versions(&pool).await, avant, "aucune version enregistrée");
    assert!(journal(&pool, "mcp.tool_called").await.is_empty());
}

/// #783 point 1 — la précondition de lecture d'`EcriturePrompts::proposer` :
/// rien n'est lu dans sa transaction avant le verrou de société. Dix-neuf
/// propositions ; une transaction du test tient le verrou et pose la
/// vingtième ; la proposition du service **attend** ce verrou, puis voit la
/// vingtième validée et rend « plafond atteint ». Une lecture simple faite
/// avant le verrou figerait la vue d'avant (dix-neuf) : la vingt et unième
/// passerait.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn plafond_juste_apres_l_attente_du_verrou(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    for n in 0..PLAFOND_A_VALIDER - 1 {
        proposer(&pool, e.c, e.cle, &format!("p{n:02}"), "d", "t").await;
    }
    let cle_auth = authentifier(
        &Lecture::new(pool.clone()),
        &Trace::new(pool.clone()),
        &e.jeton,
    )
    .await
    .unwrap();
    let mut tenue = pool.begin().await.unwrap();
    depot::proposer_version_agent_in_tx(
        &mut tenue,
        e.c,
        e.cle,
        "p-vingt",
        "d",
        "t",
        PLAFOND_A_VALIDER,
    )
    .await
    .unwrap();
    let ecriture = EcriturePrompts::new(pool.clone());
    let tache = tokio::spawn(async move {
        prompts_mcp::proposer(
            &cle_auth,
            &ecriture,
            Instant::now(),
            args("p-vingt-et-un", "d", "t"),
        )
        .await
        .map(|a| a.nom().to_string())
    });
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert!(
        !tache.is_finished(),
        "la proposition attend le verrou de société"
    );
    tenue.commit().await.unwrap();
    let issue = tache.await.unwrap();
    assert_eq!(issue, Err(RefusProposition::PlafondAtteint));
    assert_eq!(a_valider(&pool).await, i64::from(PLAFOND_A_VALIDER));
}

// ---------------------------------------------------------------------------
// AC-iii4 — jamais servi avant l'approbation
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn jamais_servi_avant_l_approbation(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    let b = banc(&pool);
    let avant = b.liste(&e.jeton).await;
    let r = b
        .proposer(&e.jeton, args("neuf", "Description neuve", "Texte neuf"))
        .await;
    assert_eq!(r.json(), accuse("neuf"));
    assert_eq!(b.liste(&e.jeton).await, avant, "rien de neuf à la liste");
    assert_eq!(
        b.obtenir(&e.jeton, json!({"name": "neuf"})).await,
        inconnu()
    );
    // Les standards ne se proposent pas, et restent servis.
    for s in [KESH_ASSISTANT, KESH_DEVELOPPEMENT] {
        let r = b.proposer(&e.jeton, args(s.nom, "d", "t")).await;
        assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_NOM_RESERVE));
        assert_eq!(
            b.obtenir(&e.jeton, json!({"name": s.nom})).await,
            get_attendu(s.description, s.texte)
        );
    }
    let prompt_id: i64 = sqlx::query_scalar("SELECT id FROM mcp_prompts WHERE name = 'neuf'")
        .fetch_one(&pool)
        .await
        .unwrap();
    // L'approbation par la route : servi, avec sa marque.
    let detail = b
        .api(
            "GET",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}"),
            &e.jwt,
            None,
        )
        .await
        .json();
    let r = b
        .api(
            "POST",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}/versions/1/approve"),
            &e.jwt,
            Some(json!({"version": detail["version"]})),
        )
        .await;
    assert_eq!(r.statut, StatusCode::OK);
    let marque = marque_agent(1, e.cle);
    assert_eq!(
        b.obtenir(&e.jeton, json!({"name": "neuf"})).await,
        get_attendu(
            &format!("{marque}\n\nDescription neuve"),
            &format!("{marque}\n\nTexte neuf")
        )
    );
    // Une proposition suivante, refusée : rien ne change.
    let servi = b.obtenir(&e.jeton, json!({"name": "neuf"})).await;
    b.proposer(&e.jeton, args("neuf", "Autre", "Autre")).await;
    let detail = b
        .api(
            "GET",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}"),
            &e.jwt,
            None,
        )
        .await
        .json();
    let r = b
        .api(
            "POST",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}/versions/2/refuse"),
            &e.jwt,
            Some(json!({"version": detail["version"]})),
        )
        .await;
    assert_eq!(r.statut, StatusCode::OK);
    assert_eq!(b.obtenir(&e.jeton, json!({"name": "neuf"})).await, servi);
}

// ---------------------------------------------------------------------------
// AC-iii5 — le journal de `prompts/get`
// ---------------------------------------------------------------------------

/// Une ligne `mcp.prompt_served` par `prompts/get`, détail **exactement**
/// `{cleMcpId, prompt, numero, issue}` ; le nom reçu du client n'est jamais
/// écrit ; `prompts/list` n'est pas journalisé (Miii5).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prompts_get_journalise(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let b = banc(&pool);
    b.liste(&cat.jeton).await;
    assert!(journal(&pool, "mcp.prompt_served").await.is_empty());
    let cle_lecteur: i64 = cat.cle_agent;
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "agent-ok"})).await,
        get_attendu(
            &marquee(&cat, "Prompt de l'agent"),
            &marquee(&cat, TEXTE_AGENT)
        )
    );
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": PIEGE_NOM})).await,
        inconnu()
    );
    assert_eq!(
        b.obtenir(&cat.jeton, json!({"name": "agent-attente"}))
            .await,
        inconnu()
    );
    let (sans_id, sans_jeton) = cle(
        &pool,
        cat.c,
        cat.u,
        &[McpScope::InvoicesRead],
        SendValidation::Exempt,
    )
    .await;
    assert_eq!(
        b.obtenir(&sans_jeton, json!({"name": "agent-ok"})).await,
        inconnu()
    );
    assert_eq!(
        journal(&pool, "mcp.prompt_served").await,
        vec![
            json!({"cleMcpId": cle_lecteur, "prompt": "agent-ok", "numero": cat.agent.numero,
                   "issue": "servi"}),
            json!({"cleMcpId": cle_lecteur, "prompt": "(inconnu)", "numero": null,
                   "issue": "inconnu"}),
            json!({"cleMcpId": cle_lecteur, "prompt": "(inconnu)", "numero": null,
                   "issue": "inconnu"}),
            json!({"cleMcpId": sans_id, "prompt": "(inconnu)", "numero": null,
                   "issue": "inconnu"}),
        ]
    );
    let (entity_type, entity_id): (String, i64) = sqlx::query_as(
        "SELECT entity_type, entity_id FROM audit_log WHERE action = 'mcp.prompt_served' \
         ORDER BY id LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((entity_type.as_str(), entity_id), ("mcp_key", cle_lecteur));
    assert!(!audit_en_texte(&pool).await.contains(PIEGE_NOM));
}

/// Fail closed (Miii5) : le journal indisponible, rien n'est servi — l'erreur
/// interne fixe, sans le texte du prompt.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prompts_get_journal_indisponible_rien_ne_part(pool: MySqlPool) {
    let cat = catalogue(&pool).await;
    let b = banc(&pool);
    sqlx::query("ALTER TABLE audit_log RENAME TO audit_log_cachee_3210a")
        .execute(&pool)
        .await
        .unwrap();
    let r = b
        .mcp(
            &cat.jeton,
            &json!({"jsonrpc":"2.0","id":1,"method":"prompts/get",
                    "params":{"name":"admin-a"}})
            .to_string(),
        )
        .await;
    sqlx::query("ALTER TABLE audit_log_cachee_3210a RENAME TO audit_log")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        r.json(),
        attendu_erreur(1, -32603, TEXTE_ERREUR_INTERNE),
        "rien ne part sans journal"
    );
    assert!(!String::from_utf8_lossy(&r.corps).contains("Texte"));
}

// ---------------------------------------------------------------------------
// AC-iii12 — aucun texte de prompt au journal du serveur
// ---------------------------------------------------------------------------

/// Une proposition, une proposition refusée par la validation, une
/// approbation, un `prompts/get` et une version invalide servie : aucun piège
/// (texte, description) dans les évènements capturés ; **témoins** : le
/// `debug!` de la décision du filtre d'adresse, émis par le serveur pendant
/// chaque requête vers `/mcp`, et l'`error!` de la version invalide sont
/// captés (Miii8). *(Le témoin de `jeton_jamais_journalise`, la requête sans
/// `MCP-Protocol-Version`, n'est pas émis ici : `post_mcp` pose l'en-tête.)*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn textes_jamais_journalises(pool: MySqlPool) {
    let e = ecrivain(&pool).await;
    let b = banc(&pool);
    let capture = CaptureTrace::installer();
    b.proposer(&e.jeton, args("piege", PIEGE_DESC, PIEGE_TEXTE))
        .await;
    let r = b
        .proposer(
            &e.jeton,
            args("piege-bis", PIEGE_DESC, &format!("{PIEGE_TEXTE}\u{202E}")),
        )
        .await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES)
    );
    let prompt_id: i64 = sqlx::query_scalar("SELECT id FROM mcp_prompts WHERE name = 'piege'")
        .fetch_one(&pool)
        .await
        .unwrap();
    let detail = b
        .api(
            "GET",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}"),
            &e.jwt,
            None,
        )
        .await
        .json();
    let r = b
        .api(
            "POST",
            &format!("/api/v1/admin/mcp/prompts/{prompt_id}/versions/1/approve"),
            &e.jwt,
            Some(json!({"version": detail["version"]})),
        )
        .await;
    assert_eq!(r.statut, StatusCode::OK);
    b.obtenir(&e.jeton, json!({"name": "piege"})).await;
    // Une version invalide en base (écrite hors de Kesh) : non servie,
    // journalisée sans son texte.
    ecrire_hors_de_kesh(&pool, prompt_id, 1, &format!("{PIEGE_TEXTE}\u{202E}")).await;
    assert_eq!(
        b.obtenir(&e.jeton, json!({"name": "piege"})).await,
        inconnu()
    );
    capture.exiger("décision du filtre d'adresse");
    capture.exiger(&format!("prompt_id={prompt_id}"));
    capture.exiger_absent(PIEGE_TEXTE);
    capture.exiger_absent(PIEGE_DESC);
}
