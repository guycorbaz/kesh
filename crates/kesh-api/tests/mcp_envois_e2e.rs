//! Story 32-8b (refs #723) — la poignée côté serveur de la validation avant
//! envoi : le dépôt sous une clé `required`, la relecture par `kesh_lire_envoi`,
//! les routes de décision de l'Administrateur, le journal et l'audit (AC-B2 à
//! AC-B9, AC-B12).
//!
//! Tout passe par `tower::ServiceExt::oneshot` sur le routeur de production
//! (`build_router_avec_registre`), comme les tests de la 32-6a : `/mcp` et les
//! routes `/api/v1/admin/mcp/sends` sont servis par **le même** routeur.
//!
//! ⛔ « Égal » veut dire égal octet pour octet : chaque réponse de `/mcp` est
//! comparée **entière** à la réponse attendue, construite depuis les textes de
//! `mcp::textes` et la valeur stockée ; l'absence d'un piège ne s'emploie qu'en
//! complément, pour un piège que le test a posé.
//!
//! Aucune donnée réelle : fixtures synthétiques et valeurs piégées
//! (`PIEGE_*_328`). Pré-requis : MariaDB démarré (`sqlx::test` crée une base
//! éphémère par test).

mod common;

use std::sync::atomic::Ordering;

use axum::Router;
use axum::body::Body;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use kesh_api::mcp::Registre;
use kesh_api::mcp::textes::*;
use kesh_api::services::acces::Portee;
use kesh_api::services::envois_mcp::{MCP_ENVOI_OCTETS_MAX, MCP_ENVOI_RESUME_OCTETS_MAX};
use kesh_db::entities::{ApiKeyScope, McpScope, NewApiKey, Role, SendValidation};
use kesh_db::repositories::api_keys;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::capture_trace::CaptureTrace;
use common::mcp::*;

const PIEGE_E1: &str = "PIEGE_E1_328";
const PIEGE_BLOC: &str = "PIEGE_BLOC1_328";
const PIEGE_STRUCT: &str = "PIEGE_STRUCT_328";
const PIEGE_RESUME: &str = "PIEGE_RESUME_328";
const PIEGE_MODIF: &str = "PIEGE_MODIF_328";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

struct Banc {
    app: Router,
    _statique: tempfile::TempDir,
}

fn banc(pool: &MySqlPool, registre: Registre) -> Banc {
    let statique = static_dir();
    Banc {
        app: routeur(
            pool.clone(),
            config(),
            statique.path(),
            registre,
            "127.0.0.1",
        ),
        _statique: statique,
    }
}

impl Banc {
    async fn mcp(&self, jeton: &str, corps: &str) -> Rendu {
        envoyer(&self.app, post_mcp(Some(jeton), corps)).await
    }

    async fn api(&self, methode: &str, chemin: &str, jeton: &str, corps: Option<Value>) -> Rendu {
        envoyer(&self.app, requete_api(methode, chemin, Some(jeton), corps)).await
    }

    /// Le détail d'un envoi, par la session `jwt`.
    async fn detail(&self, jwt: &str, n: i64) -> Rendu {
        self.api("GET", &format!("/api/v1/admin/mcp/sends/{n}"), jwt, None)
            .await
    }

    /// La décision `corps` sur l'envoi `n`.
    async fn decider(&self, jwt: &str, n: i64, corps: Value) -> Rendu {
        self.api(
            "POST",
            &format!("/api/v1/admin/mcp/sends/{n}/decision"),
            jwt,
            Some(corps),
        )
        .await
    }

    /// Lit la version au détail, puis décide : `contenu` `None` → tel quel.
    async fn valider(&self, jwt: &str, n: i64, contenu: Option<&str>) -> Rendu {
        let version = self.detail(jwt, n).await.json()["version"].clone();
        let mut corps = json!({ "version": version, "decision": "approve" });
        if let Some(c) = contenu {
            corps["content"] = json!(c);
        }
        self.decider(jwt, n, corps).await
    }

    async fn refuser(&self, jwt: &str, n: i64) -> Rendu {
        let version = self.detail(jwt, n).await.json()["version"].clone();
        self.decider(jwt, n, json!({ "version": version, "decision": "refuse" }))
            .await
    }

    /// Appelle l'outil `outil` sous `jeton` et rend le numéro déposé.
    async fn deposer(&self, jeton: &str, outil: &str) -> i64 {
        let r = self.mcp(jeton, &appel(1, outil, json!({}))).await;
        let n = numero_depose(&r);
        assert_eq!(r.json(), attendu_depose(1, n), "accusé de {outil}");
        n
    }
}

/// Une société servie, son Administrateur et sa session, une clé `books:read`
/// soumise à validation.
struct Livres {
    c: i64,
    u: i64,
    jwt: String,
    requise: (i64, String),
}

async fn livres_servis(pool: &MySqlPool) -> Livres {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    let requise = cle(pool, c, u, &[McpScope::BooksRead], SendValidation::Required).await;
    Livres {
        c,
        u,
        jwt: session(u, Role::Admin, c),
        requise,
    }
}

fn fixe(nom: &'static str, texte: &str) -> Box<Fixe> {
    Box::new(Fixe::texte(nom, texte).0)
}

/// Les lignes `mcp.tool_called`, dans l'ordre : (issue, détail sans durée).
async fn appels_journalises(pool: &MySqlPool) -> Vec<Value> {
    let lignes: Vec<String> = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = 'mcp.tool_called' \
         ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    lignes
        .into_iter()
        .map(|d| {
            let mut v: Value = serde_json::from_str(&d).unwrap();
            assert!(
                v.as_object_mut()
                    .unwrap()
                    .remove("dureeMs")
                    .unwrap()
                    .is_u64()
            );
            v
        })
        .collect()
}

/// Tout le journal d'audit, en texte.
async fn audit_en_texte(pool: &MySqlPool) -> String {
    let lignes: Vec<(String, Option<String>, String)> = sqlx::query_as(
        "SELECT action, CAST(details_json AS CHAR), actor_label FROM audit_log ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    format!("{lignes:?}")
}

fn code_d_erreur(r: &Rendu) -> String {
    r.json()["error"]["code"].as_str().unwrap_or("").to_string()
}

// ---------------------------------------------------------------------------
// AC-B2 — une seule représentation, le texte
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_seule_representation(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let texte = format!("{PIEGE_BLOC}\n\nbloc 2");
    let outil = Fixe {
        nom: "piege_struct",
        texte: texte.clone(),
        structure: Some(json!({ "x": PIEGE_STRUCT })),
        resume: Some(PIEGE_RESUME.to_string()),
        n: Default::default(),
    };
    let b = banc(&pool, Registre::vide().avec(Box::new(outil)));
    let n = b.deposer(&l.requise.1, "piege_struct").await;
    let (summary, content): (Option<String>, Option<String>) =
        sqlx::query_as("SELECT summary, content FROM mcp_sends WHERE id = ?")
            .bind(n)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(content.as_deref(), Some(texte.as_str()), "le texte seul");
    assert_eq!(summary.as_deref(), Some(PIEGE_RESUME), "le résumé, à part");
    // L'écran montre le résumé.
    assert_eq!(
        b.detail(&l.jwt, n).await.json()["summary"],
        json!(PIEGE_RESUME)
    );
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_valide(1, n, false, &texte));
    let corps = String::from_utf8_lossy(&r.corps);
    for piege in [PIEGE_STRUCT, PIEGE_RESUME] {
        assert!(!corps.contains(piege), "{piege} envoyé");
    }
}

// ---------------------------------------------------------------------------
// AC-B3 — rendre visible
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn depot_rend_visible(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(
        &pool,
        Registre::vide().avec(fixe("cache", "a\u{202E}b\rc\u{200B}d")),
    );
    let n = b.deposer(&l.requise.1, "cache").await;
    let attendu = "a\\u{202E}b\\u{000D}c\\u{200B}d";
    assert_eq!(contenu_en_base(&pool, n).await.as_deref(), Some(attendu));
    assert_eq!(b.detail(&l.jwt, n).await.json()["content"], json!(attendu));
}

// ---------------------------------------------------------------------------
// AC-B4 — la taille
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn taille_des_envois(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let deux_caches = format!(
        "{}\u{200B}\u{200B}",
        "a".repeat(MCP_ENVOI_OCTETS_MAX - 6 - 6)
    );
    // 24 564 octets bruts dont deux U+200B (3 octets) : 24 570 ≤ 24 576, mais
    // 24 580 une fois rendus visibles (8 octets chacun).
    assert_eq!(deux_caches.len(), MCP_ENVOI_OCTETS_MAX - 6);
    let resume_long = Fixe {
        nom: "resume_long",
        texte: "court".into(),
        structure: None,
        resume: Some("r".repeat(MCP_ENVOI_RESUME_OCTETS_MAX + 1)),
        n: Default::default(),
    };
    let resume_borne = Fixe {
        nom: "resume_borne",
        texte: "court".into(),
        structure: None,
        resume: Some("r".repeat(MCP_ENVOI_RESUME_OCTETS_MAX)),
        n: Default::default(),
    };
    let b = banc(
        &pool,
        Registre::vide()
            .avec(fixe("borne", &"a".repeat(MCP_ENVOI_OCTETS_MAX)))
            .avec(fixe("au_dela", &"a".repeat(MCP_ENVOI_OCTETS_MAX + 1)))
            .avec(fixe("caches", &deux_caches))
            .avec(Box::new(resume_long))
            .avec(Box::new(resume_borne)),
    );
    let n = b.deposer(&l.requise.1, "borne").await;
    assert_eq!(
        contenu_en_base(&pool, n).await.map(|c| c.len()),
        Some(MCP_ENVOI_OCTETS_MAX)
    );
    b.deposer(&l.requise.1, "resume_borne").await;
    for outil in ["au_dela", "caches", "resume_long"] {
        let avant = envois_en_base(&pool).await;
        let r = b.mcp(&l.requise.1, &appel(1, outil, json!({}))).await;
        assert_eq!(
            r.json(),
            attendu_outil_en_erreur(1, TEXTE_ENVOI_TROP_VOLUMINEUX),
            "{outil}"
        );
        assert_eq!(envois_en_base(&pool).await, avant, "{outil} : rien déposé");
    }
}

// ---------------------------------------------------------------------------
// AC-B5 — le dépôt et son journal, en une transaction
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn depot_refuse_journalise(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (compteur, _) = Compteur::nouveau("compteur", Portee::BooksRead);
    let b = banc(
        &pool,
        Registre::vide()
            .avec(fixe("gros", &"a".repeat(MCP_ENVOI_OCTETS_MAX + 1)))
            .avec(Box::new(compteur)),
    );
    let k = l.requise.0;
    let r = b.mcp(&l.requise.1, &appel(1, "gros", json!({}))).await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ENVOI_TROP_VOLUMINEUX)
    );
    assert_eq!(
        appels_journalises(&pool).await,
        vec![
            json!({"cleMcpId": k, "outil": "gros", "issue": "envoi_trop_volumineux",
                    "validation": "required"})
        ]
    );
    assert_eq!(envois_en_base(&pool).await, 0);
    for _ in 0..20 {
        b.deposer(&l.requise.1, "compteur").await;
    }
    let r = b.mcp(&l.requise.1, &appel(1, "compteur", json!({}))).await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ENVOIS_TROP_NOMBREUX)
    );
    let journal = appels_journalises(&pool).await;
    assert_eq!(journal.len(), 22, "une ligne par appel : 1 + 20 + 1");
    assert_eq!(
        journal[21],
        json!({"cleMcpId": k, "outil": "compteur", "issue": "limite_atteinte",
               "validation": "required"})
    );
    assert_eq!(envois_en_base(&pool).await, 20);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn depot_et_journal_dans_la_meme_transaction(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (compteur, _) = Compteur::nouveau("compteur", Portee::BooksRead);
    let b = banc(&pool, Registre::vide().avec(Box::new(compteur)));
    sqlx::query("ALTER TABLE audit_log RENAME TO audit_log_cachee_328")
        .execute(&pool)
        .await
        .unwrap();
    let r = b.mcp(&l.requise.1, &appel(1, "compteur", json!({}))).await;
    assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE));
    assert_eq!(envois_en_base(&pool).await, 0, "le dépôt est annulé");
    sqlx::query("ALTER TABLE audit_log_cachee_328 RENAME TO audit_log")
        .execute(&pool)
        .await
        .unwrap();
    assert!(
        appels_journalises(&pool).await.is_empty(),
        "aucune ligne nulle part"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn limite_d_envois_en_attente(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (compteur, _) = Compteur::nouveau("compteur", Portee::BooksRead);
    let b = banc(&pool, Registre::vide().avec(Box::new(compteur)));
    let mut numeros = Vec::new();
    for _ in 0..20 {
        numeros.push(b.deposer(&l.requise.1, "compteur").await);
    }
    let r = b.mcp(&l.requise.1, &appel(1, "compteur", json!({}))).await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ENVOIS_TROP_NOMBREUX)
    );
    assert_eq!(
        b.valider(&l.jwt, numeros[3], None).await.statut,
        StatusCode::OK
    );
    b.deposer(&l.requise.1, "compteur").await;
}

// ---------------------------------------------------------------------------
// AC-B6 — kesh_lire_envoi, ses arguments
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn arguments_de_kesh_lire_envoi(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::standard());
    for args in [
        json!({ "envoi": 0 }),
        json!({ "envoi": -1 }),
        json!({ "envoi": 1.5 }),
        json!({ "envoi": "1" }),
        json!({ "envoi": 18_446_744_073_709_551_615_u64 }),
        json!({}),
        json!({ "envoi": 1, "x": 1 }),
    ] {
        let r = b
            .mcp(&l.requise.1, &appel(1, "kesh_lire_envoi", args.clone()))
            .await;
        assert_eq!(
            r.json(),
            attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES),
            "{args}"
        );
    }
    assert_eq!(envois_en_base(&pool).await, 0, "aucun dépôt");
    // Aucune relecture n'est tentée : chaque appel est journalisé en
    // `arguments_invalides`, sans `envoi`. Le nombre d'abord, pour que la
    // boucle ne puisse pas passer à vide (revue de code P1, A-L4 = B-5).
    let journal = appels_journalises(&pool).await;
    assert_eq!(journal.len(), 7, "une entrée par appel");
    for d in journal {
        assert_eq!(
            d,
            json!({"cleMcpId": l.requise.0, "outil": "kesh_lire_envoi",
                   "issue": "arguments_invalides", "validation": "required"})
        );
    }
}

// ---------------------------------------------------------------------------
// AC-B7 — la propriété centrale, éprouvée
// ---------------------------------------------------------------------------

fn texte_piege() -> String {
    format!("ligne 1\n{PIEGE_E1} ligne piégée\nligne 3 — é")
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn modification_recue_octet_pour_octet(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let c1 = b.detail(&l.jwt, n).await.json()["content"]
        .as_str()
        .unwrap()
        .to_string();
    let mut c2: String = c1
        .split_inclusive('\n')
        .filter(|ligne| !ligne.contains(PIEGE_E1))
        .collect();
    c2.push_str("\tligne ajoutée — é ✓\n");
    assert_ne!(c1, c2);
    let r = b.valider(&l.jwt, n, Some(&c2)).await;
    assert_eq!(r.statut, StatusCode::OK);
    assert_eq!(r.json()["modified"], json!(true));
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_valide(1, n, true, &c2));
    assert!(!String::from_utf8_lossy(&r.corps).contains(PIEGE_E1));
    assert_eq!(
        contenu_en_base(&pool, n).await.as_deref(),
        Some(c2.as_str())
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tel_quel_recu_octet_pour_octet(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let c1 = b.detail(&l.jwt, n).await.json()["content"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_valide(1, n, false, &c1));
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_ne_transmet_rien(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.refuser(&l.jwt, n).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_refuse(1, n));
    assert_eq!(contenu_en_base(&pool, n).await, None);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn echeance_ne_transmet_rien(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let en_attente = b.deposer(&l.requise.1, "piege").await;
    let valide = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.valider(&l.jwt, valide, None).await.statut, StatusCode::OK);
    let version = b.detail(&l.jwt, en_attente).await.json()["version"].clone();
    echoir(&pool, en_attente).await;
    echoir(&pool, valide).await;
    for n in [en_attente, valide] {
        let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
        assert_eq!(r.json(), attendu_echu(1, n));
    }
    let r = b
        .decider(
            &l.jwt,
            en_attente,
            json!({ "version": version, "decision": "approve" }),
        )
        .await;
    assert_eq!(
        (r.statut, code_d_erreur(&r)),
        (StatusCode::CONFLICT, "ILLEGAL_STATE_TRANSITION".to_string())
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn relu_deux_fois_memes_octets(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r1 = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    let r2 = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r1.corps, r2.corps, "mêmes octets");
    let servis: Vec<Value> = appels_journalises(&pool)
        .await
        .into_iter()
        .filter(|d| d["outil"] == json!("kesh_lire_envoi"))
        .collect();
    assert_eq!(
        servis,
        vec![
            json!({"cleMcpId": l.requise.0, "outil": "kesh_lire_envoi", "issue": "servi",
                   "validation": "required", "envoi": n});
            2
        ]
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn autre_cle_et_autre_societe_indiscernables(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (_, autre_cle) = cle(
        &pool,
        l.c,
        l.u,
        &[McpScope::BooksRead],
        SendValidation::Required,
    )
    .await;
    let c2 = societe(&pool, "Autre société 328").await;
    let u2 = administrateur(&pool, c2, "admin-328-b").await;
    activer(&pool, c2).await;
    let (_, cle_c2) = cle(
        &pool,
        c2,
        u2,
        &[McpScope::BooksRead],
        SendValidation::Required,
    )
    .await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    // Témoin : la clé de l'envoi le relit.
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json()["result"]["isError"], json!(false));
    // Une ligne incohérente : la clé de l'appel, la société d'autrui (AC-A7).
    let incoherent = sqlx::query(
        "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, content, modified, \
         expires_at, decided_at, decided_by_user_id) VALUES (?, ?, 'piege', 'approved', ?, \
         FALSE, NOW(3) + INTERVAL 1 HOUR, NOW(3), ?)",
    )
    .bind(c2)
    .bind(l.requise.0)
    .bind(PIEGE_E1)
    .bind(u2)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (jeton, numero) in [
        (&autre_cle, n),
        (&cle_c2, n),
        (&l.requise.1, 999_999),
        (&l.requise.1, incoherent),
    ] {
        let r = b.mcp(jeton, &lire_envoi(1, numero)).await;
        assert_eq!(r.json(), attendu_inconnu(1, numero), "envoi {numero}");
        assert!(!String::from_utf8_lossy(&r.corps).contains(PIEGE_E1));
    }
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn politique_changee_entre_depot_et_relecture(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let c1 = contenu_en_base(&pool, n).await.unwrap();
    let politique = |v: &'static str| {
        let (b, jwt, k, p) = (&b, l.jwt.clone(), l.requise.0, pool.clone());
        async move {
            let version: i32 = sqlx::query_scalar("SELECT version FROM mcp_keys WHERE id = ?")
                .bind(k)
                .fetch_one(&p)
                .await
                .unwrap();
            let r = b
                .api(
                    "PUT",
                    &format!("/api/v1/admin/mcp/keys/{k}/send-validation"),
                    &jwt,
                    Some(json!({ "version": version, "sendValidation": v })),
                )
                .await;
            assert_eq!(r.statut, StatusCode::OK, "politique {v}");
        }
    };
    politique("exempt").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_valide(1, n, false, &c1), "sous exempt");
    politique("required").await;
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(
        r.json(),
        attendu_valide(1, n, false, &c1),
        "revenu à required"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn survit_au_redemarrage(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let n = {
        let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
        b.deposer(&l.requise.1, "piege").await
    };
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(
        r.json(),
        attendu_valide(1, n, false, &texte_piege()),
        "la seconde application relit l'envoi de la première"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn kesh_version_passe_par_la_validation(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::standard());
    let n = b.deposer(&l.requise.1, "kesh_version").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let attendu = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "lectureSeule": true,
        "delaiInstructionS": 30.0,
    })
    .to_string();
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_valide(1, n, false, &attendu));
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn relecture_sans_nouvelle_validation(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (compteur, appels) = Compteur::nouveau("compteur", Portee::BooksRead);
    let b = banc(&pool, Registre::vide().avec(Box::new(compteur)));
    let n = b.deposer(&l.requise.1, "compteur").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(
        r.json(),
        attendu_valide(1, n, false, &json!({ "n": 1 }).to_string())
    );
    assert_eq!(envois_en_base(&pool).await, 1, "aucune ligne neuve");
    assert_eq!(
        appels.load(Ordering::SeqCst),
        1,
        "l'outil n'est pas rappelé"
    );
}

/// Règle R7 : la relecture s'exécute sur une `LectureEnvois` en lecture seule.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn lire_pour_cle_en_lecture_seule(pool: MySqlPool) {
    use kesh_api::services::envois_mcp::{EnvoiLu, lire};
    use kesh_api::services::poignees::{Lecture, LectureEnvois, Trace};
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let cle = kesh_api::services::cles_mcp::authentifier(
        &Lecture::new(pool.clone()),
        &Trace::new(pool.clone()),
        &l.requise.1,
    )
    .await
    .unwrap();
    let seule = kesh_db::pool::pool_lecture_seule(&pool, 1, std::time::Duration::from_secs(5));
    let mode: i64 = sqlx::query_scalar("SELECT @@session.tx_read_only")
        .fetch_one(&seule)
        .await
        .unwrap();
    assert_eq!(mode, 1, "témoin : la connexion est en lecture seule");
    match lire(&cle, &LectureEnvois::new(seule), n).await.unwrap() {
        EnvoiLu::Valide(c) => assert_eq!(c.texte(), texte_piege()),
        _ => panic!("validé attendu"),
    }
}

// ---------------------------------------------------------------------------
// AC-B8 — les routes de décision
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn routes_de_decision_par_une_session_d_administrateur(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let liste = b.api("GET", "/api/v1/admin/mcp/sends", &l.jwt, None).await;
    assert_eq!(liste.statut, StatusCode::OK);
    let lignes = liste.json();
    assert_eq!(lignes.as_array().unwrap().len(), 1);
    let ligne = &lignes[0];
    assert_eq!(
        (
            ligne["id"].clone(),
            ligne["state"].clone(),
            ligne["toolName"].clone()
        ),
        (json!(n), json!("pending"), json!("piege"))
    );
    assert!(
        ligne.get("content").is_none() && ligne.get("summary").is_none(),
        "la liste ne porte ni contenu ni résumé : {ligne}"
    );
    let d = b.detail(&l.jwt, n).await;
    assert_eq!(d.statut, StatusCode::OK);
    let d = d.json();
    assert_eq!(d["content"], json!(texte_piege()));
    assert_eq!(d["contentBytes"], json!(texte_piege().len()));
    assert_eq!(d["version"], json!(1));
    let r = b.valider(&l.jwt, n, None).await;
    assert_eq!(r.statut, StatusCode::OK);
    assert_eq!(
        (r.json()["state"].clone(), r.json()["version"].clone()),
        (json!("approved"), json!(2))
    );
    // Introuvable, ou d'une autre société : le même 404.
    let c2 = societe(&pool, "Autre société 328").await;
    let u2 = administrateur(&pool, c2, "admin-328-b").await;
    let autre = b.detail(&session(u2, Role::Admin, c2), n).await;
    let absent = b.detail(&l.jwt, 999_999).await;
    assert_eq!(autre.statut, StatusCode::NOT_FOUND);
    assert_eq!(autre.corps, absent.corps, "même corps");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptable_cle_mcp_et_pat_refuses(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let comptable = {
        let id = kesh_db::repositories::users::create(
            &pool,
            kesh_db::entities::NewUser {
                username: "comptable-328".into(),
                password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
                role: Role::Comptable,
                active: true,
                company_id: l.c,
                email: None,
            },
        )
        .await
        .unwrap()
        .id;
        session(id, Role::Comptable, l.c)
    };
    let decision = json!({ "version": 1, "decision": "approve" });
    let chemins = [
        ("GET", "/api/v1/admin/mcp/sends".to_string(), None),
        ("GET", format!("/api/v1/admin/mcp/sends/{n}"), None),
        (
            "POST",
            format!("/api/v1/admin/mcp/sends/{n}/decision"),
            Some(decision.clone()),
        ),
    ];
    for (m, chemin, corps) in &chemins {
        let r = b.api(m, chemin, &comptable, corps.clone()).await;
        assert_eq!(
            (r.statut, code_d_erreur(&r)),
            (StatusCode::FORBIDDEN, "FORBIDDEN".to_string()),
            "Comptable : {m} {chemin}"
        );
        let r = b.api(m, chemin, &l.requise.1, corps.clone()).await;
        assert_eq!(r.statut, StatusCode::UNAUTHORIZED, "clé MCP : {m} {chemin}");
    }
    // `pat_ne_valide_pas` : un PAT `read-write` d'un Administrateur.
    let (jeton, key_hash) = kesh_api::auth::api_key::generate_pat();
    let mut tx = pool.begin().await.unwrap();
    api_keys::create_in_tx(
        &mut tx,
        NewApiKey {
            company_id: l.c,
            created_by_user_id: l.u,
            name: "intégration 328".into(),
            key_hash,
            scope: ApiKeyScope::ReadWrite,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    let r = b
        .api(
            "POST",
            &format!("/api/v1/admin/mcp/sends/{n}/decision"),
            &jeton,
            Some(decision),
        )
        .await;
    assert_eq!(
        (r.statut, code_d_erreur(&r)),
        (StatusCode::FORBIDDEN, "API_KEY_ADMIN_FORBIDDEN".to_string()),
        "pat_ne_valide_pas"
    );
    let etat: String = sqlx::query_scalar("SELECT state FROM mcp_sends WHERE id = ?")
        .bind(n)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(etat, "pending", "rien n'est décidé");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn decisions_concurrentes(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    let corps = json!({ "version": 1, "decision": "approve" });
    let (r1, r2) = tokio::join!(
        b.decider(&l.jwt, n, corps.clone()),
        b.decider(&l.jwt, n, corps.clone())
    );
    let mut statuts = vec![r1.statut, r2.statut];
    statuts.sort();
    assert_eq!(statuts, vec![StatusCode::OK, StatusCode::CONFLICT]);
    let decisions: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'mcp_send.approved'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(decisions, 1, "une seule décision écrite");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn decision_et_audit_dans_la_meme_transaction(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", &texte_piege())));
    let n = b.deposer(&l.requise.1, "piege").await;
    sqlx::query("ALTER TABLE audit_log RENAME TO audit_log_cachee_328")
        .execute(&pool)
        .await
        .unwrap();
    let r = b
        .decider(&l.jwt, n, json!({ "version": 1, "decision": "approve" }))
        .await;
    assert_eq!(r.statut, StatusCode::INTERNAL_SERVER_ERROR);
    let (etat, version): (String, i32) =
        sqlx::query_as("SELECT state, version FROM mcp_sends WHERE id = ?")
            .bind(n)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!((etat.as_str(), version), ("pending", 1), "décision annulée");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn decision_journalisee_sans_contenu(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let outil = Fixe {
        nom: "piege",
        texte: texte_piege(),
        structure: None,
        resume: Some(PIEGE_RESUME.to_string()),
        n: Default::default(),
    };
    let b = banc(&pool, Registre::vide().avec(Box::new(outil)));
    let n = b.deposer(&l.requise.1, "piege").await;
    let refuse = b.deposer(&l.requise.1, "piege").await;
    let modifie = format!("{PIEGE_MODIF} ligne modifiée");
    assert_eq!(
        b.valider(&l.jwt, n, Some(&modifie)).await.statut,
        StatusCode::OK
    );
    assert_eq!(b.refuser(&l.jwt, refuse).await.statut, StatusCode::OK);
    let entrees: Vec<(String, String, i64, i64, String)> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, user_id, CAST(details_json AS CHAR) \
         FROM audit_log WHERE action LIKE 'mcp_send.%' ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    let detail = |s: &str| serde_json::from_str::<Value>(s).unwrap();
    assert_eq!(entrees.len(), 2);
    assert_eq!(
        (
            entrees[0].0.as_str(),
            entrees[0].1.as_str(),
            entrees[0].2,
            entrees[0].3,
            detail(&entrees[0].4)
        ),
        (
            "mcp_send.approved",
            "mcp_send",
            n,
            l.u,
            json!({"envoiId": n, "cleMcpId": l.requise.0, "outil": "piege", "modifie": true})
        )
    );
    assert_eq!(
        (entrees[1].0.as_str(), detail(&entrees[1].4)),
        (
            "mcp_send.refused",
            json!({"envoiId": refuse, "cleMcpId": l.requise.0, "outil": "piege",
                   "modifie": false})
        )
    );
    let audit = audit_en_texte(&pool).await;
    for piege in [PIEGE_MODIF, PIEGE_RESUME, PIEGE_E1] {
        assert!(!audit.contains(piege), "{piege} au journal d'audit");
    }
}

// ---------------------------------------------------------------------------
// AC-B9 — le contenu modifié, contrôlé, jamais corrigé (par la route)
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn contenu_modifie_controle_par_la_route(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", "texte déposé")));
    for refuse in [
        "a\r\nb".to_string(),
        "x\u{202E}y".to_string(),
        "\u{0378}".to_string(),
        "a".repeat(MCP_ENVOI_OCTETS_MAX + 1),
    ] {
        let n = b.deposer(&l.requise.1, "piege").await;
        let r = b.valider(&l.jwt, n, Some(&refuse)).await;
        assert_eq!(
            (r.statut, code_d_erreur(&r)),
            (StatusCode::BAD_REQUEST, "VALIDATION_ERROR".to_string()),
            "{:?}",
            &refuse[..refuse.len().min(12)]
        );
        assert_eq!(
            contenu_en_base(&pool, n).await.as_deref(),
            Some("texte déposé"),
            "rien d'écrit"
        );
        // Libère la place de l'envoi pour le plafond.
        assert_eq!(b.refuser(&l.jwt, n).await.statut, StatusCode::OK);
    }
    // Admis, et stockés égaux — dont la chaîne vide (point 3 de #765) et un
    // blanc final.
    for admis in [
        "⚠ é".to_string(),
        String::new(),
        "blanc final ".to_string(),
        "a".repeat(MCP_ENVOI_OCTETS_MAX),
    ] {
        let n = b.deposer(&l.requise.1, "piege").await;
        assert_eq!(
            b.valider(&l.jwt, n, Some(&admis)).await.statut,
            StatusCode::OK
        );
        assert_eq!(
            contenu_en_base(&pool, n).await.as_deref(),
            Some(admis.as_str())
        );
        let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
        assert_eq!(r.json(), attendu_valide(1, n, true, &admis));
    }
    // Un refus ne porte pas de contenu.
    let n = b.deposer(&l.requise.1, "piege").await;
    let r = b
        .decider(
            &l.jwt,
            n,
            json!({ "version": 1, "decision": "refuse", "content": "x" }),
        )
        .await;
    assert_eq!(r.statut, StatusCode::BAD_REQUEST);
}

// ---------------------------------------------------------------------------
// #765, point 1 — l'ordre des refus de la décision, à deux causes
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ordre_des_refus_de_la_decision(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", "texte")));
    let deja = t_message("mcp-send-already-decided");
    let echu = t_message("mcp-send-expired");
    let essai = |n: i64| {
        let b = &b;
        let jwt = l.jwt.clone();
        async move {
            let r = b
                .decider(&jwt, n, json!({ "version": 99, "decision": "approve" }))
                .await;
            (
                r.statut,
                code_d_erreur(&r),
                r.json()["error"]["message"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            )
        }
    };
    // (a) refusé, échu, mauvaise version → déjà décidé.
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.refuser(&l.jwt, n).await.statut, StatusCode::OK);
    echoir(&pool, n).await;
    assert_eq!(
        essai(n).await,
        (
            StatusCode::CONFLICT,
            "ILLEGAL_STATE_TRANSITION".into(),
            deja.clone()
        ),
        "refusé et échu"
    );
    // (b) validé, échu, mauvaise version → déjà décidé.
    let n = b.deposer(&l.requise.1, "piege").await;
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    echoir(&pool, n).await;
    assert_eq!(
        essai(n).await,
        (
            StatusCode::CONFLICT,
            "ILLEGAL_STATE_TRANSITION".into(),
            deja.clone()
        ),
        "validé et échu"
    );
    // (c) en attente, échu, mauvaise version → échu, avant la version.
    let n = b.deposer(&l.requise.1, "piege").await;
    echoir(&pool, n).await;
    assert_eq!(
        essai(n).await,
        (
            StatusCode::CONFLICT,
            "ILLEGAL_STATE_TRANSITION".into(),
            echu
        ),
        "échu à mauvaise version"
    );
    // (d) en attente, vivant, mauvaise version → version périmée.
    let n = b.deposer(&l.requise.1, "piege").await;
    let (statut, code, _) = essai(n).await;
    assert_eq!(
        (statut, code.as_str()),
        (StatusCode::CONFLICT, "OPTIMISTIC_LOCK_CONFLICT")
    );
}

/// La valeur fr-CH d'une clé de message (repli égal, garde G9).
fn t_message(cle: &str) -> String {
    let ftl = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../kesh-i18n/locales/fr-CH/messages.ftl"),
    )
    .unwrap();
    ftl.lines()
        .find_map(|l| l.strip_prefix(&format!("{cle} = ")))
        .unwrap_or_else(|| panic!("{cle} absente du catalogue fr-CH"))
        .to_string()
}

// ---------------------------------------------------------------------------
// #765, point 3 — valider tel quel une ligne sans contenu
// ---------------------------------------------------------------------------

/// Une ligne en attente **sans** contenu ne naît jamais d'un dépôt (il écrit
/// toujours un texte) ; posée par le pool du test, sa validation tel quel est
/// admise par le dépôt, et sa relecture **échoue fermée** : erreur interne,
/// rien ne part (C-32-8-42, C-32-8-55). Le texte vide (`Some("")`), lui, est un
/// contenu : `contenu_modifie_controle_par_la_route`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn validation_tel_quel_sans_contenu_echoue_fermee(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, Registre::vide().avec(fixe("piege", "texte")));
    let n = b.deposer(&l.requise.1, "piege").await;
    sqlx::query("UPDATE mcp_sends SET content = NULL WHERE id = ?")
        .bind(n)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(b.valider(&l.jwt, n, None).await.statut, StatusCode::OK);
    let r = b.mcp(&l.requise.1, &lire_envoi(1, n)).await;
    assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE));
    assert_eq!(
        appels_journalises(&pool).await.last().unwrap(),
        &json!({"cleMcpId": l.requise.0, "outil": "kesh_lire_envoi", "issue": "erreur",
                "validation": "required", "envoi": n})
    );
}

// ---------------------------------------------------------------------------
// AC-B12 — jamais au journal du serveur
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn contenu_jamais_dans_le_journal_du_serveur(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let outil = Fixe {
        nom: "piege",
        texte: texte_piege(),
        structure: None,
        resume: Some(PIEGE_RESUME.to_string()),
        n: Default::default(),
    };
    let b = banc(&pool, Registre::vide().avec(Box::new(outil)));
    let capture = CaptureTrace::installer();
    // Requêtes sans `MCP-Protocol-Version` : le `debug!` du transport est le
    // témoin que la capture voit les évènements du serveur.
    let sans_version = |corps: String| {
        let mut h = HeaderMap::new();
        h.insert("content-type", HeaderValue::from_static("application/json"));
        h.insert(
            "authorization",
            HeaderValue::from_str(&format!("Bearer {}", l.requise.1)).unwrap(),
        );
        requete("POST", "/mcp", &h, Body::from(corps))
    };
    let r = envoyer(&b.app, sans_version(appel(1, "piege", json!({})))).await;
    let n = numero_depose(&r);
    capture.exiger("requête sans MCP-Protocol-Version");
    let modifie = format!("{PIEGE_MODIF}\n");
    assert_eq!(
        b.valider(&l.jwt, n, Some(&modifie)).await.statut,
        StatusCode::OK
    );
    let r = envoyer(&b.app, sans_version(lire_envoi(2, n))).await;
    assert!(
        String::from_utf8_lossy(&r.corps).contains(PIEGE_MODIF),
        "témoin : relu"
    );
    for piege in [PIEGE_E1, PIEGE_RESUME, PIEGE_MODIF] {
        capture.exiger_absent(piege);
    }
}
