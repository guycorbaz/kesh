//! Story 32-10a-iii (refs #723) — les routes de gestion des prompts du
//! serveur MCP, `/api/v1/admin/mcp/prompts` (AC-iii6 à AC-iii9, AC-iii11 ;
//! amendements D1 et D2 de la fiche 32-10b).
//!
//! Tout passe par `tower::ServiceExt::oneshot` sur le routeur de production,
//! avec une session d'Administrateur (JWT forgé) ; `/mcp` sur le même routeur
//! pour ce qui est servi. Les refus d'un PAT, aux deux portées, sont tenus par
//! `admin_pat_denied_e2e`.
//!
//! ⛔ « Égal » veut dire égal octet pour octet ; l'absence d'un piège
//! (`PIEGE_*_3210A`) ne s'emploie qu'en complément. Aucune donnée réelle.
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par
//! test).

mod common;

use axum::Router;
use axum::http::StatusCode;
use kesh_api::mcp::Registre;
use kesh_api::mcp::textes::*;
use kesh_api::services::envois_mcp::rendre_visible;
use kesh_api::services::prompts_mcp::marque_agent;
use kesh_db::entities::mcp_prompt::{PLAFOND_A_VALIDER, VersionEcrite};
use kesh_db::entities::mcp_prompt_standards::{KESH_ASSISTANT, KESH_DEVELOPPEMENT, STANDARDS};
use kesh_db::entities::{McpScope, NewUser, Role, SendValidation};
use kesh_db::repositories::{mcp_prompts as depot, users};
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::mcp::*;

const PIEGE_TEXTE: &str = "PIEGE_TEXTE_3210A";
const PIEGE_DESC: &str = "PIEGE_DESC_3210A";
const RACINE: &str = "/api/v1/admin/mcp/prompts";

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
    async fn api(&self, methode: &str, chemin: &str, jwt: &str, corps: Option<Value>) -> Rendu {
        envoyer(&self.app, requete_api(methode, chemin, Some(jwt), corps)).await
    }

    async fn sans_auth(&self, methode: &str, chemin: &str, corps: Option<Value>) -> Rendu {
        envoyer(&self.app, requete_api(methode, chemin, None, corps)).await
    }

    async fn liste(&self, jwt: &str) -> Value {
        let r = self.api("GET", RACINE, jwt, None).await;
        assert_eq!(r.statut, StatusCode::OK);
        r.json()
    }

    async fn detail(&self, jwt: &str, id: i64) -> Rendu {
        self.api("GET", &format!("{RACINE}/{id}"), jwt, None).await
    }

    /// La `version` du prompt, lue au détail.
    async fn version(&self, jwt: &str, id: i64) -> Value {
        self.detail(jwt, id).await.json()["version"].clone()
    }

    async fn creer(&self, jwt: &str, nom: &str, d: &str, t: &str) -> Rendu {
        self.api(
            "POST",
            RACINE,
            jwt,
            Some(json!({"nom": nom, "description": d, "texte": t})),
        )
        .await
    }

    async fn ajouter(&self, jwt: &str, id: i64, corps: Value) -> Rendu {
        self.api("POST", &format!("{RACINE}/{id}/versions"), jwt, Some(corps))
            .await
    }

    async fn decider(&self, jwt: &str, id: i64, n: i32, verbe: &str, version: Value) -> Rendu {
        self.api(
            "POST",
            &format!("{RACINE}/{id}/versions/{n}/{verbe}"),
            jwt,
            Some(json!({ "version": version })),
        )
        .await
    }

    async fn standard(&self, jwt: &str, id: i64, version: Value) -> Rendu {
        self.api(
            "POST",
            &format!("{RACINE}/{id}/standard"),
            jwt,
            Some(json!({ "version": version })),
        )
        .await
    }

    async fn obtenir(&self, jeton: &str, nom: &str) -> Value {
        let corps = json!({"jsonrpc":"2.0","id":1,"method":"prompts/get","params":{"name":nom}});
        envoyer(&self.app, post_mcp(Some(jeton), &corps.to_string()))
            .await
            .json()
    }

    async fn noms_servis(&self, jeton: &str) -> Vec<String> {
        let corps = r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#;
        envoyer(&self.app, post_mcp(Some(jeton), corps))
            .await
            .json()["result"]["prompts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap().to_string())
            .collect()
    }
}

/// Une société servie, son Administrateur et sa session, une clé
/// `books:read` + `prompts:write`, les standards amorcés.
struct Societe {
    c: i64,
    u: i64,
    jwt: String,
    cle: i64,
    jeton: String,
}

async fn societe_servie(pool: &MySqlPool) -> Societe {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    depot::amorcer_standards(pool, STANDARDS).await.unwrap();
    let (cle, jeton) = cle(
        pool,
        c,
        u,
        &[McpScope::BooksRead, McpScope::PromptsWrite],
        SendValidation::Exempt,
    )
    .await;
    Societe {
        c,
        u,
        jwt: session(u, Role::Admin, c),
        cle,
        jeton,
    }
}

/// Une proposition d'agent, laissée à valider (par le dépôt).
async fn proposer(pool: &MySqlPool, s: &Societe, nom: &str, d: &str, t: &str) -> VersionEcrite {
    let mut tx = pool.begin().await.unwrap();
    let v = depot::proposer_version_agent_in_tx(&mut tx, s.c, s.cle, nom, d, t, PLAFOND_A_VALIDER)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    v
}

async fn id_du_prompt(pool: &MySqlPool, c: i64, nom: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM mcp_prompts WHERE company_id = ? AND name = ?")
        .bind(c)
        .bind(nom)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn compte(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

/// Les détails d'audit d'une action, dans l'ordre.
async fn details(pool: &MySqlPool, action: &str) -> Vec<Value> {
    let lignes: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap();
    lignes
        .iter()
        .map(|d| serde_json::from_str(d).unwrap())
        .collect()
}

fn code(r: &Rendu) -> (StatusCode, String) {
    (
        r.statut,
        r.json()["error"]["code"].as_str().unwrap_or("").to_string(),
    )
}

fn erreur(r: &Rendu) -> (StatusCode, String, String) {
    let j = r.json();
    (
        r.statut,
        j["error"]["code"].as_str().unwrap_or("").to_string(),
        j["error"]["message"].as_str().unwrap_or("").to_string(),
    )
}

fn ecrite(
    id: i64,
    nom: &str,
    numero: i32,
    origine: &str,
    nature: &str,
    etat: &str,
    version: i32,
) -> Value {
    json!({"promptId": id, "nom": nom, "numero": numero, "origine": origine, "nature": nature,
           "etat": etat, "version": version})
}

fn texte_rendu(s: &str) -> Value {
    json!({"exact": s, "visible": rendre_visible(s)})
}

// ---------------------------------------------------------------------------
// AC-iii6 — les routes, par une session d'Administrateur
// ---------------------------------------------------------------------------

/// La liste (sans texte), le prompt neuf (201), la version d'administrateur
/// (201), l'approbation et le refus (200), le retour au standard (201) ; 404
/// d'une autre société, indiscernable d'un prompt absent ; 409 sur verrou
/// périmé (Miii9), version non la dernière, version décidée, retour au
/// standard d'un prompt non standard.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn routes_par_une_session_d_administrateur(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let assistant = id_du_prompt(&pool, s.c, KESH_ASSISTANT.nom).await;
    let developpement = id_du_prompt(&pool, s.c, KESH_DEVELOPPEMENT.nom).await;
    assert_eq!(
        b.liste(&s.jwt).await,
        json!([
            {"id": assistant, "nom": "kesh-assistant", "standard": true, "version": 1,
             "derniere": {"numero": 1, "origine": "standard", "etat": "approved", "nature": "text"},
             "enVigueur": {"numero": 1, "origine": "standard", "etat": "approved", "nature": "text"},
             "aValider": 0, "standardDiffere": false},
            {"id": developpement, "nom": "kesh-developpement", "standard": true, "version": 1,
             "derniere": {"numero": 1, "origine": "standard", "etat": "approved", "nature": "text"},
             "enVigueur": {"numero": 1, "origine": "standard", "etat": "approved", "nature": "text"},
             "aValider": 0, "standardDiffere": false},
        ])
    );

    // Prompt neuf.
    let r = b.creer(&s.jwt, "mon-prompt", "Description", "Texte").await;
    assert_eq!(r.statut, StatusCode::CREATED);
    let id = id_du_prompt(&pool, s.c, "mon-prompt").await;
    assert_eq!(
        r.json(),
        ecrite(id, "mon-prompt", 1, "admin", "text", "approved", 1)
    );
    // Version d'administrateur (texte), puis verrou périmé.
    let r = b
        .ajouter(
            &s.jwt,
            id,
            json!({"version": 1, "nature": "text", "description": "D2", "texte": "T2"}),
        )
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert_eq!(
        r.json(),
        ecrite(id, "mon-prompt", 2, "admin", "text", "approved", 2)
    );
    let r = b
        .ajouter(
            &s.jwt,
            id,
            json!({"version": 1, "nature": "text", "description": "D3", "texte": "T3"}),
        )
        .await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "OPTIMISTIC_LOCK_CONFLICT".into())
    );
    // Une proposition, puis une version d'administrateur : la proposition
    // n'est plus la dernière (409), puis elle se refuse (200).
    let p = proposer(&pool, &s, "mon-prompt", "Proposée", "Texte proposé").await;
    assert_eq!(p.numero, 3);
    let v = b.version(&s.jwt, id).await;
    let r = b
        .ajouter(
            &s.jwt,
            id,
            json!({"version": v, "nature": "text", "description": "D4", "texte": "T4"}),
        )
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    let v = b.version(&s.jwt, id).await;
    let r = b.decider(&s.jwt, id, 3, "approve", v.clone()).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into()),
        "non la dernière"
    );
    let r = b.decider(&s.jwt, id, 3, "refuse", v.clone()).await;
    assert_eq!(r.statut, StatusCode::OK);
    assert_eq!(
        r.json(),
        ecrite(id, "mon-prompt", 3, "agent", "text", "refused", 5)
    );
    let r = b.decider(&s.jwt, id, 3, "refuse", json!(5)).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into()),
        "déjà décidée"
    );
    // Retrait.
    let r = b
        .ajouter(&s.jwt, id, json!({"version": 5, "nature": "withdrawal"}))
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert_eq!(
        r.json(),
        ecrite(id, "mon-prompt", 5, "admin", "withdrawal", "approved", 6)
    );
    assert!(!b.noms_servis(&s.jeton).await.contains(&"mon-prompt".into()));
    // Retour au standard d'un prompt non standard : 409.
    let r = b.standard(&s.jwt, id, json!(6)).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into())
    );
    // Retour au standard d'un standard personnalisé : 201.
    let r = b
        .ajouter(
            &s.jwt,
            assistant,
            json!({"version": 1, "nature": "text", "description": "Perso", "texte": "Perso"}),
        )
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert_eq!(b.liste(&s.jwt).await[0]["standardDiffere"], json!(true));
    let r = b.standard(&s.jwt, assistant, json!(2)).await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert_eq!(
        r.json(),
        ecrite(
            assistant,
            "kesh-assistant",
            3,
            "standard",
            "text",
            "approved",
            3
        )
    );
    assert_eq!(b.liste(&s.jwt).await[0]["standardDiffere"], json!(false));
    // 404 : un prompt d'une autre société, indiscernable d'un prompt absent.
    let c2 = societe(&pool, "Autre société 3210a").await;
    let u2 = administrateur(&pool, c2, "admin-3210a-b").await;
    let autre = session(u2, Role::Admin, c2);
    let a = b.detail(&autre, id).await;
    let absent = b.detail(&s.jwt, 999_999).await;
    assert_eq!(a.statut, StatusCode::NOT_FOUND);
    assert_eq!(a.corps, absent.corps, "même corps");
    for r in [
        b.ajouter(&autre, id, json!({"version": 6, "nature": "withdrawal"}))
            .await,
        b.decider(&autre, id, 3, "approve", json!(6)).await,
        b.decider(&autre, id, 3, "refuse", json!(6)).await,
        b.standard(&autre, assistant, json!(3)).await,
    ] {
        assert_eq!(r.statut, StatusCode::NOT_FOUND);
    }
    assert_eq!(
        b.liste(&autre).await,
        json!([]),
        "la liste de l'autre société"
    );
}

// ---------------------------------------------------------------------------
// D1 — le texte servi, par la fonction du catalogue
// ---------------------------------------------------------------------------

/// Le détail rend, pour chaque version, le texte et la description **tels que
/// `prompts/get` les servirait** — égaux octet pour octet à la réponse de
/// `prompts/get` pour la version en vigueur —, `servable`, les formes rendues
/// visibles, le texte du code d'un standard, et les champs de l'écran.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn detail_texte_servi_egal_a_prompts_get(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let p = proposer(
        &pool,
        &s,
        "agent-x",
        "Description\u{A0}x",
        "Texte de l'agent",
    )
    .await;
    let r = b
        .decider(&s.jwt, p.prompt_id, 1, "approve", json!(p.version))
        .await;
    assert_eq!(r.statut, StatusCode::OK);
    let servi = b.obtenir(&s.jeton, "agent-x").await;
    let d = b.detail(&s.jwt, p.prompt_id).await.json();
    let v = &d["versions"][0];
    assert_eq!(
        v["texteServi"]["exact"], servi["result"]["messages"][0]["content"]["text"],
        "texte servi = prompts/get, octet pour octet"
    );
    assert_eq!(
        v["descriptionServie"]["exact"],
        servi["result"]["description"]
    );
    let marque = marque_agent(1, s.cle);
    let u: String = sqlx::query_scalar("SELECT username FROM users WHERE id = ?")
        .bind(s.u)
        .fetch_one(&pool)
        .await
        .unwrap();
    // Les trois dates : présentes (leur valeur est celle de la base, lue par
    // le dépôt ; leur forme est celle de `NaiveDateTime`).
    for date in [&v["creeLe"], &v["decideLe"], &d["modifieLe"]] {
        assert!(date.as_str().is_some_and(|x| x.starts_with("20")), "{date}");
    }
    let mut sans_dates = d.clone();
    for champ in ["creeLe", "decideLe"] {
        sans_dates["versions"][0]
            .as_object_mut()
            .unwrap()
            .remove(champ);
    }
    sans_dates.as_object_mut().unwrap().remove("modifieLe");
    assert_eq!(
        sans_dates,
        json!({
            "id": p.prompt_id, "nom": "agent-x", "standard": false, "version": 2,
            "numeroEnVigueur": 1, "standardDiffere": false, "standardDuCode": null,
            "versions": [{
                "numero": 1, "origine": "agent", "etat": "approved", "nature": "text",
                "description": texte_rendu("Description\u{A0}x"),
                "texte": texte_rendu("Texte de l'agent"),
                "servable": true,
                "descriptionServie": texte_rendu(&format!("{marque}\n\nDescription\u{A0}x")),
                "texteServi": texte_rendu(&format!("{marque}\n\nTexte de l'agent")),
                "auteurNom": null, "decideurNom": u, "cleAuteurId": s.cle, "cleNom": "agent",
                "cleRevoquee": false,
            }],
        })
    );
    // Un standard : son texte du code, servi tel quel, sans marque.
    let id = id_du_prompt(&pool, s.c, KESH_DEVELOPPEMENT.nom).await;
    let d = b.detail(&s.jwt, id).await.json();
    assert_eq!(
        d["standardDuCode"],
        json!({"description": texte_rendu(KESH_DEVELOPPEMENT.description),
               "texte": texte_rendu(KESH_DEVELOPPEMENT.texte)})
    );
    assert_eq!(
        d["versions"][0]["texteServi"]["exact"],
        b.obtenir(&s.jeton, KESH_DEVELOPPEMENT.nom).await["result"]["messages"][0]["content"]["text"]
    );
    assert_eq!(d["versions"][0]["servable"], json!(true));
    // Un caractère caché, écrit hors de Kesh : non servable, rendu visible.
    sqlx::query("UPDATE mcp_prompt_versions SET body = 'a\u{202E}b' WHERE prompt_id = ?")
        .bind(p.prompt_id)
        .execute(&pool)
        .await
        .unwrap();
    let v = b.detail(&s.jwt, p.prompt_id).await.json()["versions"][0].clone();
    assert_eq!(
        (
            v["servable"].clone(),
            v["texteServi"].clone(),
            v["descriptionServie"].clone(),
            v["texte"]["visible"].clone()
        ),
        (json!(false), Value::Null, Value::Null, json!("a\\u{202E}b"))
    );
}

// ---------------------------------------------------------------------------
// D2 — l'approbation revalide le contenu
// ---------------------------------------------------------------------------

/// Une proposition que le serveur ne servirait pas — un caractère caché écrit
/// hors de Kesh — ne s'approuve pas : 400, le code de la création, rien
/// d'écrit, aucun audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn approbation_revalide_le_contenu(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let p = proposer(&pool, &s, "agent-y", "D", "Texte").await;
    sqlx::query("UPDATE mcp_prompt_versions SET body = 'a\u{2066}b' WHERE prompt_id = ?")
        .bind(p.prompt_id)
        .execute(&pool)
        .await
        .unwrap();
    let r = b
        .decider(&s.jwt, p.prompt_id, 1, "approve", json!(p.version))
        .await;
    assert_eq!(
        erreur(&r),
        (
            StatusCode::BAD_REQUEST,
            "VALIDATION_ERROR".into(),
            "Le texte du prompt contient un caractère invisible, bidirectionnel ou de contrôle : \
             retirez-le."
                .into()
        )
    );
    let etat: String = sqlx::query_scalar(
        "SELECT state FROM mcp_prompt_versions WHERE prompt_id = ? AND number = 1",
    )
    .bind(p.prompt_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(etat, "pending");
    assert!(
        details(&pool, "mcp_prompt.version_approved")
            .await
            .is_empty()
    );
    assert_eq!(
        b.obtenir(&s.jeton, "agent-y").await["error"]["message"],
        json!(TEXTE_PROMPT_INCONNU)
    );
    // Le refus, lui, reste possible.
    let r = b
        .decider(&s.jwt, p.prompt_id, 1, "refuse", json!(p.version))
        .await;
    assert_eq!(r.statut, StatusCode::OK);
}

// ---------------------------------------------------------------------------
// AC-iii7 — les gardes : Comptable, clé MCP, sans authentification
// ---------------------------------------------------------------------------

/// Sur **chaque** route : Comptable → `403 FORBIDDEN` ; clé MCP, même portant
/// `prompts:write` → 401 ; sans authentification → 401. L'agent ne peut
/// jamais approuver (P22).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptable_cle_mcp_et_anonyme_refuses(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let p = proposer(&pool, &s, "agent-z", "D", "T").await;
    let id = p.prompt_id;
    let comptable = {
        let u = users::create(
            &pool,
            NewUser {
                username: "comptable-3210a".into(),
                password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
                role: Role::Comptable,
                active: true,
                company_id: s.c,
                email: None,
            },
        )
        .await
        .unwrap()
        .id;
        session(u, Role::Comptable, s.c)
    };
    let v = json!({"version": 1});
    let chemins: Vec<(&str, String, Option<Value>)> = vec![
        ("GET", RACINE.to_string(), None),
        (
            "POST",
            RACINE.to_string(),
            Some(json!({"nom": "x", "description": "d", "texte": "t"})),
        ),
        ("GET", format!("{RACINE}/{id}"), None),
        (
            "POST",
            format!("{RACINE}/{id}/versions"),
            Some(json!({"version": 1, "nature": "withdrawal"})),
        ),
        (
            "POST",
            format!("{RACINE}/{id}/versions/1/approve"),
            Some(v.clone()),
        ),
        (
            "POST",
            format!("{RACINE}/{id}/versions/1/refuse"),
            Some(v.clone()),
        ),
        ("POST", format!("{RACINE}/{id}/standard"), Some(v.clone())),
    ];
    assert_eq!(chemins.len(), 7, "les sept routes");
    for (m, chemin, corps) in &chemins {
        let r = b.api(m, chemin, &comptable, corps.clone()).await;
        assert_eq!(
            code(&r),
            (StatusCode::FORBIDDEN, "FORBIDDEN".into()),
            "Comptable : {m} {chemin}"
        );
        let r = b.api(m, chemin, &s.jeton, corps.clone()).await;
        assert_eq!(r.statut, StatusCode::UNAUTHORIZED, "clé MCP : {m} {chemin}");
        let r = b.sans_auth(m, chemin, corps.clone()).await;
        assert_eq!(r.statut, StatusCode::UNAUTHORIZED, "anonyme : {m} {chemin}");
    }
    let etat: String =
        sqlx::query_scalar("SELECT state FROM mcp_prompt_versions WHERE prompt_id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(etat, "pending", "rien n'est décidé");
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_prompts WHERE name = 'x'").await,
        0
    );
}

// ---------------------------------------------------------------------------
// AC-iii8 — la validation des écritures de l'administrateur
// ---------------------------------------------------------------------------

/// Nom non conforme, préfixe `kesh`, nom pris, description ou texte vides,
/// trop longs, caractère refusé → 400 au message traduit (repli fr-CH), rien
/// d'écrit, aucun audit. Le contenu enregistré est le contenu **normalisé** :
/// un texte de 16 000 caractères après normalisation, reçu avec des `\r\n`,
/// passe (aucune erreur 500 du `CHECK`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validation_des_ecritures(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    b.creer(&s.jwt, "pris", "D", "T").await;
    let versions_avant = compte(&pool, "SELECT COUNT(*) FROM mcp_prompt_versions").await;
    let audit_avant = compte(&pool, "SELECT COUNT(*) FROM audit_log").await;
    let long = "é".repeat(16_001);
    let description_longue = "d".repeat(501);
    let cas: Vec<(&str, &str, String, &str)> = vec![
        (
            "Majuscule",
            "D",
            "T".into(),
            "Nom de prompt invalide : de 1 à 64 caractères parmi les minuscules non accentuées, les chiffres, le tiret et le souligné.",
        ),
        (
            "nom ",
            "D",
            "T".into(),
            "Nom de prompt invalide : de 1 à 64 caractères parmi les minuscules non accentuées, les chiffres, le tiret et le souligné.",
        ),
        (
            "kesh-x",
            "D",
            "T".into(),
            "Nom de prompt réservé : un nom commençant par « kesh » est réservé aux prompts standard de Kesh.",
        ),
        (
            "keshx",
            "D",
            "T".into(),
            "Nom de prompt réservé : un nom commençant par « kesh » est réservé aux prompts standard de Kesh.",
        ),
        ("pris", "D", "T".into(), "Un prompt de ce nom existe déjà."),
        ("ok", " ", "T".into(), "La description du prompt est vide."),
        (
            "ok",
            &description_longue,
            "T".into(),
            "La description du prompt dépasse 500 caractères.",
        ),
        (
            "ok",
            "a\nb",
            "T".into(),
            "La description du prompt contient un saut de ligne, une tabulation ou un caractère invisible, bidirectionnel ou de contrôle : retirez-le.",
        ),
        ("ok", "D", " \n".into(), "Le texte du prompt est vide."),
        (
            "ok",
            "D",
            long.clone(),
            "Le texte du prompt dépasse 16 000 caractères.",
        ),
        (
            "ok",
            "D",
            format!("{PIEGE_TEXTE}\u{200B}"),
            "Le texte du prompt contient un caractère invisible, bidirectionnel ou de contrôle : retirez-le.",
        ),
    ];
    for (nom, d, t, message) in &cas {
        let r = b.creer(&s.jwt, nom, d, t).await;
        assert_eq!(
            erreur(&r),
            (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                message.to_string()
            ),
            "{nom:?}"
        );
        assert!(!String::from_utf8_lossy(&r.corps).contains(PIEGE_TEXTE));
    }
    let id = id_du_prompt(&pool, s.c, "pris").await;
    for (corps, message) in [
        (
            json!({"version": 0, "nature": "withdrawal"}),
            "Version invalide : un entier supérieur ou égal à 1 est attendu.",
        ),
        (
            json!({"version": 1, "nature": "withdrawal", "texte": "t"}),
            "Un retrait ne porte ni description ni texte.",
        ),
        (
            json!({"version": 1, "nature": "text", "texte": "t"}),
            "Une version de texte porte une description et un texte.",
        ),
        (
            json!({"version": 1, "nature": "text", "description": "d", "texte": "a\u{202E}"}),
            "Le texte du prompt contient un caractère invisible, bidirectionnel ou de contrôle : \
             retirez-le.",
        ),
    ] {
        let r = b.ajouter(&s.jwt, id, corps.clone()).await;
        assert_eq!(
            erreur(&r),
            (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR".to_string(),
                message.to_string()
            ),
            "{corps}"
        );
    }
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_prompt_versions").await,
        versions_avant,
        "rien d'écrit"
    );
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM audit_log").await,
        audit_avant,
        "aucun audit"
    );
    // Le contenu normalisé : 15 995 caractères et cinq `\r\n` (16 005 reçus,
    // 16 000 après normalisation) passent, et c'est la forme normalisée qui
    // est enregistrée.
    let recu = format!("{}{}", "a".repeat(15_995), "\r\n".repeat(5));
    assert_eq!(recu.chars().count(), 16_005);
    let r = b.creer(&s.jwt, "normalise", "D", &recu).await;
    assert_eq!(r.statut, StatusCode::CREATED);
    let enregistre: String = sqlx::query_scalar(
        "SELECT v.body FROM mcp_prompt_versions v JOIN mcp_prompts p ON p.id = v.prompt_id \
         WHERE p.name = 'normalise'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        enregistre,
        format!("{}{}", "a".repeat(15_995), "\n".repeat(5))
    );
}

// ---------------------------------------------------------------------------
// AC-iii9 — l'audit, sans texte
// ---------------------------------------------------------------------------

/// Chaque écriture a son entrée, dans sa transaction, au détail exact — jamais
/// la description ni le texte (Miii6) ; le refus d'une proposition d'un prompt
/// qui n'a aucune version approuvée ne porte pas le nom (P14, F8).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn audit_sans_texte(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let r = b.creer(&s.jwt, "audite", PIEGE_DESC, PIEGE_TEXTE).await;
    let id = r.json()["promptId"].as_i64().unwrap();
    b.ajouter(
        &s.jwt,
        id,
        json!({"version": 1, "nature": "text", "description": PIEGE_DESC, "texte": PIEGE_TEXTE}),
    )
    .await;
    let p = proposer(&pool, &s, "audite", PIEGE_DESC, PIEGE_TEXTE).await;
    b.decider(&s.jwt, id, p.numero, "approve", json!(p.version))
        .await;
    let neuf = proposer(&pool, &s, "jamais-approuve", PIEGE_DESC, PIEGE_TEXTE).await;
    b.decider(&s.jwt, neuf.prompt_id, 1, "refuse", json!(neuf.version))
        .await;
    let p2 = proposer(&pool, &s, "audite", PIEGE_DESC, PIEGE_TEXTE).await;
    b.decider(&s.jwt, id, p2.numero, "refuse", json!(p2.version))
        .await;
    let assistant = id_du_prompt(&pool, s.c, KESH_ASSISTANT.nom).await;
    b.ajouter(
        &s.jwt,
        assistant,
        json!({"version": 1, "nature": "withdrawal"}),
    )
    .await;
    b.standard(&s.jwt, assistant, json!(2)).await;

    assert_eq!(
        details(&pool, "mcp_prompt.created").await,
        vec![json!({"promptId": id, "nom": "audite", "numero": 1})]
    );
    assert_eq!(
        details(&pool, "mcp_prompt.version_created").await,
        vec![
            json!({"promptId": id, "nom": "audite", "numero": 2, "origine": "admin",
                   "nature": "text"}),
            json!({"promptId": assistant, "nom": "kesh-assistant", "numero": 2,
                   "origine": "admin", "nature": "withdrawal"}),
        ]
    );
    assert_eq!(
        details(&pool, "mcp_prompt.version_approved").await,
        vec![json!({"promptId": id, "nom": "audite", "numero": 3})]
    );
    assert_eq!(
        details(&pool, "mcp_prompt.version_refused").await,
        vec![
            json!({"promptId": neuf.prompt_id, "numero": 1}),
            json!({"promptId": id, "nom": "audite", "numero": 4}),
        ]
    );
    assert_eq!(
        details(&pool, "mcp_prompt.standard_restored").await,
        vec![json!({"promptId": assistant, "nom": "kesh-assistant", "numero": 3})]
    );
    let (entite, types): (i64, String) = sqlx::query_as(
        "SELECT COUNT(*), GROUP_CONCAT(DISTINCT entity_type) FROM audit_log \
         WHERE action LIKE 'mcp\\_prompt.%'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((entite, types.as_str()), (7, "mcp_prompt"));
    let tout: Vec<(Option<String>, String)> =
        sqlx::query_as("SELECT CAST(details_json AS CHAR), actor_label FROM audit_log")
            .fetch_all(&pool)
            .await
            .unwrap();
    let tout = format!("{tout:?}");
    assert!(!tout.contains(PIEGE_TEXTE) && !tout.contains(PIEGE_DESC));
    assert!(
        !tout.contains("jamais-approuve"),
        "le nom refusé n'est pas écrit"
    );
}

// ---------------------------------------------------------------------------
// AC-iii11 — la version d'agent approuvée par mégarde
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn approuvee_par_megarde(pool: MySqlPool) {
    let s = societe_servie(&pool).await;
    let b = banc(&pool);
    let p = proposer(&pool, &s, "megarde", "Proposée", "Texte de l'agent").await;
    let id = p.prompt_id;
    let r = b.decider(&s.jwt, id, 1, "approve", json!(p.version)).await;
    assert_eq!(r.statut, StatusCode::OK);
    // (f) approuver deux fois → 409.
    let r = b.decider(&s.jwt, id, 1, "approve", json!(2)).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into())
    );
    // (a) à l'historique, approuvée, avec son approbateur et sa date ; aucune
    // route ne la « désapprouve » (le refus d'une version décidée est refusé).
    let r = b.decider(&s.jwt, id, 1, "refuse", json!(2)).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into())
    );
    let v = b.detail(&s.jwt, id).await.json()["versions"][0].clone();
    assert_eq!(v["etat"], json!("approved"));
    assert!(v["decideurNom"].is_string() && v["decideLe"].is_string());
    // (b) une version d'administrateur la remplace : servie sans marque, la
    // version d'agent toujours à l'historique.
    let r = b
        .ajouter(
            &s.jwt,
            id,
            json!({"version": 2, "nature": "text", "description": "Corrigée", "texte": "Texte corrigé"}),
        )
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    let servi = b.obtenir(&s.jeton, "megarde").await;
    assert_eq!(
        servi["result"],
        json!({"description": "Corrigée",
               "messages": [{"role": "user", "content": {"type": "text", "text": "Texte corrigé"}}]})
    );
    let d = b.detail(&s.jwt, id).await.json();
    assert_eq!(
        d["versions"][0]["texte"]["exact"],
        json!("Texte de l'agent")
    );
    assert_eq!(d["numeroEnVigueur"], json!(2));
    // (e) une proposition, puis une version d'administrateur : la proposition
    // n'est plus la dernière, son approbation → 409, rien d'écrit.
    let p3 = proposer(&pool, &s, "megarde", "Encore", "Encore").await;
    let v = b.version(&s.jwt, id).await;
    b.ajouter(
        &s.jwt,
        id,
        json!({"version": v, "nature": "text", "description": "Corrigée", "texte": "Texte corrigé"}),
    )
    .await;
    let v = b.version(&s.jwt, id).await;
    let r = b.decider(&s.jwt, id, p3.numero, "approve", v.clone()).await;
    assert_eq!(
        code(&r),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".into())
    );
    assert_eq!(b.version(&s.jwt, id).await, v, "rien d'écrit");
    // (c) un retrait ôte le prompt de `prompts/list`.
    let r = b
        .ajouter(&s.jwt, id, json!({"version": v, "nature": "withdrawal"}))
        .await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert!(!b.noms_servis(&s.jeton).await.contains(&"megarde".into()));
    // (d) sur un standard : une version d'agent ne peut s'y proposer
    // (C-32-10a-27) ; une version d'administrateur le personnalise, le retour
    // au standard sert de nouveau le texte du code, et le prompt redevient non
    // personnalisé (l'amorçage suivant ne le touche pas, puisqu'il sert le
    // texte du code).
    let dev = id_du_prompt(&pool, s.c, KESH_DEVELOPPEMENT.nom).await;
    b.ajouter(
        &s.jwt,
        dev,
        json!({"version": 1, "nature": "text", "description": "Perso", "texte": "Perso"}),
    )
    .await;
    let r = b.standard(&s.jwt, dev, json!(2)).await;
    assert_eq!(r.statut, StatusCode::CREATED);
    assert_eq!(
        b.obtenir(&s.jeton, KESH_DEVELOPPEMENT.nom).await["result"]["messages"][0]["content"]["text"],
        json!(KESH_DEVELOPPEMENT.texte)
    );
    let ligne = b
        .liste(&s.jwt)
        .await
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["nom"] == json!(KESH_DEVELOPPEMENT.nom))
        .cloned()
        .unwrap();
    assert_eq!(ligne["standardDiffere"], json!(false));
    let avant = compte(&pool, "SELECT COUNT(*) FROM mcp_prompt_versions").await;
    depot::amorcer_standards(&pool, STANDARDS).await.unwrap();
    assert_eq!(
        compte(&pool, "SELECT COUNT(*) FROM mcp_prompt_versions").await,
        avant,
        "l'amorçage suivant n'écrit rien"
    );
}
