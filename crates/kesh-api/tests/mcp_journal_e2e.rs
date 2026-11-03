//! Story 32-6b (refs #723) — le journal d'audit des appels d'outil MCP
//! (AC-b1 à AC-b4, C-32-6-58).
//!
//! Tout passe par `tower::ServiceExt::oneshot` sur le routeur de production
//! (`build_router_avec_registre`), comme `mcp_transport_e2e.rs`. Les entrées
//! se lisent en base : `audit_log`, action `mcp.tool_called`.
//!
//! « Égal » veut dire égal : le détail de chaque entrée est comparé **entier**
//! au JSON attendu, `dureeMs` seul étant contrôlé par son type (entier ≥ 0)
//! avant d'être retiré de la comparaison.
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

mod common;

use std::sync::atomic::Ordering;
use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use kesh_api::mcp::textes::*;
use kesh_api::mcp::{ErreurOutil, Registre};
use kesh_api::services::acces::Portee;
use kesh_db::entities::{McpScope, SendValidation};
use serde_json::json;
use sqlx::MySqlPool;

use common::mcp::*;

/// Un banc : le routeur et son `static_dir`, gardé vivant.
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
    async fn mcp(&self, jeton: Option<&str>, corps: &str) -> Rendu {
        envoyer(&self.app, post_mcp(jeton, corps)).await
    }
}

/// Une société servie, son administrateur, une clé `books:read` dispensée et
/// une clé `books:read` soumise à la validation.
struct Livres {
    societe: i64,
    createur: i64,
    exempte: (i64, String),
    requise: (i64, String),
}

async fn livres_servis(pool: &MySqlPool) -> Livres {
    let (c, u) = livres(pool).await;
    activer(pool, c).await;
    let exempte = cle(pool, c, u, &[McpScope::BooksRead], SendValidation::Exempt).await;
    let requise = cle(pool, c, u, &[McpScope::BooksRead], SendValidation::Required).await;
    Livres {
        societe: c,
        createur: u,
        exempte,
        requise,
    }
}

/// Le nombre de lignes de `audit_log`, **toutes actions confondues** : un
/// appel ne doit en ajouter qu'une.
async fn lignes(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Une entrée `mcp.tool_called`, lue en base.
#[derive(Debug)]
struct Entree {
    action: String,
    entity_type: String,
    entity_id: i64,
    user_id: i64,
    actor_type: String,
    actor_api_key_id: Option<i64>,
    actor_label: String,
    details: serde_json::Value,
}

/// Une ligne brute : action, type d'entité, entité, utilisateur, type
/// d'auteur, clé d'API, libellé, détail en texte.
type LigneBrute = (
    String,
    String,
    i64,
    i64,
    String,
    Option<i64>,
    String,
    String,
);

/// Les entrées `mcp.tool_called`, dans l'ordre d'écriture.
async fn entrees(pool: &MySqlPool) -> Vec<Entree> {
    let lignes: Vec<LigneBrute> = sqlx::query_as(
        "SELECT action, entity_type, entity_id, user_id, CAST(actor_type AS CHAR), \
             actor_api_key_id, actor_label, CAST(details_json AS CHAR) \
             FROM audit_log WHERE action = 'mcp.tool_called' ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    lignes
        .into_iter()
        .map(
            |(
                action,
                entity_type,
                entity_id,
                user_id,
                actor_type,
                actor_api_key_id,
                actor_label,
                d,
            )| {
                Entree {
                    action,
                    entity_type,
                    entity_id,
                    user_id,
                    actor_type,
                    actor_api_key_id,
                    actor_label,
                    details: serde_json::from_str(&d).expect("détail JSON"),
                }
            },
        )
        .collect()
}

/// Le détail attendu, **sans** `dureeMs`.
fn attendu(cle_id: i64, outil: &str, issue: &str, validation: &str) -> serde_json::Value {
    json!({ "cleMcpId": cle_id, "outil": outil, "issue": issue, "validation": validation })
}

/// Contrôle `dureeMs` (un entier ≥ 0), le rend, et le retire du détail.
fn retirer_duree(details: &mut serde_json::Value) -> u64 {
    let d = details
        .as_object_mut()
        .expect("détail objet")
        .remove("dureeMs")
        .expect("dureeMs présent");
    d.as_u64()
        .unwrap_or_else(|| panic!("dureeMs n'est pas un entier ≥ 0 : {d}"))
}

/// Un appel ajoute **exactement une** ligne à `audit_log`, entrée
/// `mcp.tool_called` au nom du créateur de la clé, dont le détail est égal à
/// `attendu`. Rend la réponse et la durée journalisée.
async fn un_appel_une_ligne(
    pool: &MySqlPool,
    b: &Banc,
    l: &Livres,
    cle: &(i64, String),
    corps: &str,
    detail: serde_json::Value,
) -> (Rendu, u64) {
    let avant = lignes(pool).await;
    let r = b.mcp(Some(&cle.1), corps).await;
    assert_eq!(r.statut, StatusCode::OK, "{corps}");
    assert_eq!(
        lignes(pool).await,
        avant + 1,
        "une ligne par appel : {corps}"
    );
    let mut e = entrees(pool).await.pop().expect("une entrée");
    assert_eq!(e.action, "mcp.tool_called");
    assert_eq!(e.entity_type, "mcp_key");
    assert_eq!(e.entity_id, cle.0, "entity_id = la clé");
    assert_eq!(e.user_id, l.createur, "au nom du créateur de la clé");
    assert_eq!(e.actor_type, "user", "C-32-6-21");
    assert_eq!(e.actor_api_key_id, None);
    let duree = retirer_duree(&mut e.details);
    assert_eq!(e.details, detail, "détail de l'appel {corps}");
    (r, duree)
}

/// Le registre de test : `kesh_version`, le piégé, le refus de service, un
/// outil hors des portées de la clé, deux outils en erreur et le paniqué.
fn registre_de_test() -> Registre {
    let (hors_portee, _) = Compteur::nouveau("hors_portee", Portee::InvoicesRead);
    Registre::standard()
        .avec(Box::new(Piege))
        .avec(Box::new(RefusDeService {
            n: Default::default(),
        }))
        .avec(Box::new(hors_portee))
        .avec(Box::new(EnErreur {
            nom: "introuvable",
            erreur: ErreurOutil::Introuvable,
        }))
        .avec(Box::new(EnErreur {
            nom: "interne",
            erreur: ErreurOutil::Interne,
        }))
        .avec(Box::new(Panique))
}

// ---------------------------------------------------------------------------
// AC-b1 — chaque appel est journalisé
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn chaque_appel_est_journalise(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, registre_de_test());
    let (ke, kr) = (l.exempte.0, l.requise.0);

    // Chaque issue, avec la réponse que la 32-6a rend (inchangée).
    let cas: Vec<(&(i64, String), String, serde_json::Value, serde_json::Value)> = vec![
        (
            &l.exempte,
            appel(1, "piege", json!({})),
            attendu(ke, "piege", "servi", "exempt"),
            json!(null),
        ),
        // Story 32-8b (AC-B11) : `validation_requise` n'existe plus — sous une
        // clé `required`, la sortie est déposée ; les issues des envois sont
        // éprouvées après cette liste, une ligne chacune.
        (
            &l.exempte,
            appel(1, "n_existe_pas", json!({})),
            attendu(ke, "(inconnu)", "inconnu", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_OUTIL_INCONNU),
        ),
        (
            &l.exempte,
            appel(1, "hors_portee", json!({})),
            attendu(ke, "(inconnu)", "inconnu", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_OUTIL_INCONNU),
        ),
        (
            &l.exempte,
            appel(1, "refus_de_service", json!({})),
            attendu(ke, "refus_de_service", "inconnu", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_OUTIL_INCONNU),
        ),
        // `arguments_invalides`, première source : l'outil refuse l'argument.
        (
            &l.exempte,
            appel(1, "kesh_version", json!({ "inattendu": 1 })),
            attendu(ke, "kesh_version", "arguments_invalides", "exempt"),
            attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES),
        ),
        // Seconde source : la demande est mal formée (AC-a19) — `outil` vaut
        // toujours `(inconnu)`, même quand le nom est celui d'un outil servi.
        (
            &l.exempte,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call"}).to_string(),
            attendu(ke, "(inconnu)", "arguments_invalides", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_PARAMS_INVALIDES),
        ),
        (
            &l.exempte,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":7}}).to_string(),
            attendu(ke, "(inconnu)", "arguments_invalides", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_PARAMS_INVALIDES),
        ),
        (
            &l.exempte,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
                   "params":{"name":"kesh_version","arguments":[1]}})
            .to_string(),
            attendu(ke, "(inconnu)", "arguments_invalides", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_PARAMS_INVALIDES),
        ),
        (
            &l.exempte,
            json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":["kesh_version"]})
                .to_string(),
            attendu(ke, "(inconnu)", "arguments_invalides", "exempt"),
            attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_PARAMS_INVALIDES),
        ),
        (
            &l.exempte,
            appel(1, "introuvable", json!({})),
            attendu(ke, "introuvable", "introuvable", "exempt"),
            attendu_outil_en_erreur(1, TEXTE_INTROUVABLE),
        ),
        (
            &l.exempte,
            appel(1, "interne", json!({})),
            attendu(ke, "interne", "erreur", "exempt"),
            attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE),
        ),
        // Un panic de l'outil est une issue `erreur`, journalisée (C-32-6-58).
        (
            &l.exempte,
            appel(1, "panique", json!({})),
            attendu(ke, "panique", "erreur", "exempt"),
            attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE),
        ),
    ];
    for (cle, corps, detail, reponse) in &cas {
        let (r, _) = un_appel_une_ligne(&pool, &b, &l, cle, corps, detail.clone()).await;
        if reponse.is_null() {
            // Servi : la réponse porte les données (témoin que l'appel a lu).
            assert_eq!(r.json()["result"]["isError"], json!(false));
        } else {
            assert_eq!(&r.json(), reponse, "réponse de {corps}");
        }
    }
    assert_eq!(entrees(&pool).await.len(), cas.len());

    // `delai_depasse` : un registre au délai réduit ; la durée journalisée est
    // au moins ce délai.
    let lent = banc(
        &pool,
        Registre::vide()
            .avec(Box::new(Lent))
            .avec_delai(Duration::from_millis(100)),
    );
    let (r, duree) = un_appel_une_ligne(
        &pool,
        &lent,
        &l,
        &l.exempte,
        &appel(1, "lent", json!({})),
        attendu(ke, "lent", "delai_depasse", "exempt"),
    )
    .await;
    assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_DELAI_DEPASSE));
    assert!(duree >= 100, "dureeMs mesure l'appel : {duree}");

    // Story 32-8b (AC-B11) — les issues des envois, une ligne chacune, détail
    // égal : cinq clés, ou six avec `envoi` (le numéro déposé ou demandé).
    let (gros, _) = Fixe::texte(
        "gros",
        "a".repeat(kesh_api::services::envois_mcp::MCP_ENVOI_OCTETS_MAX + 1),
    );
    let b = banc(&pool, registre_de_test().avec(Box::new(gros)));
    let avec_envoi = |issue: &str, outil: &str, n: i64| {
        let mut d = attendu(kr, outil, issue, "required");
        d["envoi"] = json!(n);
        d
    };
    // Dépôt : l'entrée est écrite dans la transaction du dépôt.
    let avant = lignes(&pool).await;
    let r = b
        .mcp(Some(&l.requise.1), &appel(1, "piege", json!({})))
        .await;
    let n = numero_depose(&r);
    assert_eq!(r.json(), attendu_depose(1, n));
    assert_eq!(lignes(&pool).await, avant + 1, "un dépôt, une ligne");
    let mut e = entrees(&pool).await.pop().unwrap();
    retirer_duree(&mut e.details);
    assert_eq!(e.details, avec_envoi("envoi_depose", "piege", n));
    // Relecture en attente.
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &lire_envoi(1, n),
        avec_envoi("envoi_en_attente", "kesh_lire_envoi", n),
    )
    .await;
    assert_eq!(r.json(), attendu_en_attente(1, n));
    // Relecture validée : `servi`.
    decider_en_base(&pool, n, l.createur, true).await;
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &lire_envoi(1, n),
        avec_envoi("servi", "kesh_lire_envoi", n),
    )
    .await;
    assert_eq!(r.json()["result"]["isError"], json!(false));
    // Relecture refusée.
    let r = b
        .mcp(Some(&l.requise.1), &appel(1, "piege", json!({})))
        .await;
    let refuse = numero_depose(&r);
    decider_en_base(&pool, refuse, l.createur, false).await;
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &lire_envoi(1, refuse),
        avec_envoi("envoi_refuse", "kesh_lire_envoi", refuse),
    )
    .await;
    assert_eq!(r.json(), attendu_refuse(1, refuse));
    // Relecture échue.
    let r = b
        .mcp(Some(&l.requise.1), &appel(1, "piege", json!({})))
        .await;
    let echu = numero_depose(&r);
    echoir(&pool, echu).await;
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &lire_envoi(1, echu),
        avec_envoi("envoi_echu", "kesh_lire_envoi", echu),
    )
    .await;
    assert_eq!(r.json(), attendu_echu(1, echu));
    // Relecture d'un numéro inconnu : `inconnu`, le numéro demandé.
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &lire_envoi(1, 999_999),
        avec_envoi("inconnu", "kesh_lire_envoi", 999_999),
    )
    .await;
    assert_eq!(r.json(), attendu_inconnu(1, 999_999));
    // Relecture aux arguments invalides : cinq clés, sans `envoi`.
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &appel(1, "kesh_lire_envoi", json!({ "envoi": 0 })),
        attendu(kr, "kesh_lire_envoi", "arguments_invalides", "required"),
    )
    .await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ARGUMENTS_INVALIDES)
    );
    // Dépôt trop volumineux : journalisé après le refus, cinq clés.
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &appel(1, "gros", json!({})),
        attendu(kr, "gros", "envoi_trop_volumineux", "required"),
    )
    .await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ENVOI_TROP_VOLUMINEUX)
    );
    // Limite atteinte : vingt envois en attente pour la clé, puis un dépôt.
    remplir_en_attente(&pool, l.societe, kr).await;
    let avant_envois = envois_en_base(&pool).await;
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.requise,
        &appel(1, "piege", json!({})),
        attendu(kr, "piege", "limite_atteinte", "required"),
    )
    .await;
    assert_eq!(
        r.json(),
        attendu_outil_en_erreur(1, TEXTE_ENVOIS_TROP_NOMBREUX)
    );
    assert_eq!(envois_en_base(&pool).await, avant_envois, "rien déposé");
}

/// Décide l'envoi `id` en base (validé tel quel ou refusé), sans passer par la
/// route — pour les tests du journal des appels.
async fn decider_en_base(pool: &MySqlPool, id: i64, user_id: i64, valider: bool) {
    let sql = if valider {
        "UPDATE mcp_sends SET state = 'approved', decided_at = NOW(3), \
         decided_by_user_id = ?, version = version + 1 WHERE id = ?"
    } else {
        "UPDATE mcp_sends SET state = 'refused', content = NULL, summary = NULL, \
         decided_at = NOW(3), decided_by_user_id = ?, version = version + 1 WHERE id = ?"
    };
    sqlx::query(sql)
        .bind(user_id)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
}

/// Complète à vingt les envois en attente de la clé `cle_id`.
async fn remplir_en_attente(pool: &MySqlPool, company_id: i64, cle_id: i64) {
    let en_attente: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM mcp_sends WHERE mcp_key_id = ? AND state = 'pending' \
         AND expires_at > NOW(3)",
    )
    .bind(cle_id)
    .fetch_one(pool)
    .await
    .unwrap();
    for _ in en_attente..20 {
        sqlx::query(
            "INSERT INTO mcp_sends (company_id, mcp_key_id, tool_name, state, content, \
             modified, expires_at) VALUES (?, ?, 'piege', 'pending', 'x', FALSE, \
             NOW(3) + INTERVAL 1 HOUR)",
        )
        .bind(company_id)
        .bind(cle_id)
        .execute(pool)
        .await
        .unwrap();
    }
}

/// AC-B11 — `entree_appel`, fonction pure : cinq clés sans envoi, six avec ;
/// l'acteur est le créateur de la clé (C-32-8-52 : la clé authentifiée ne se
/// construit que par `authentifier`, d'où ce test ici).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn entree_appel_cinq_ou_six_cles(pool: MySqlPool) {
    use kesh_api::services::poignees::{Lecture, Trace};
    use kesh_api::services::serveur_mcp::{IssueAppel, entree_appel};
    let l = livres_servis(&pool).await;
    let cle = kesh_api::services::cles_mcp::authentifier(
        &Lecture::new(pool.clone()),
        &Trace::new(pool.clone()),
        &l.requise.1,
    )
    .await
    .expect("clé authentifiée");
    let e = entree_appel(&cle, "kesh_version", IssueAppel::Servi, 12, None);
    assert_eq!(
        e.details_json,
        Some(json!({
            "cleMcpId": l.requise.0, "outil": "kesh_version", "issue": "servi",
            "dureeMs": 12, "validation": "required",
        }))
    );
    assert_eq!(
        (
            e.user_id,
            e.action.as_str(),
            e.entity_type.as_str(),
            e.entity_id
        ),
        (l.createur, "mcp.tool_called", "mcp_key", l.requise.0)
    );
    let e = entree_appel(&cle, "piege", IssueAppel::EnvoiDepose, 3, Some(42));
    assert_eq!(
        e.details_json,
        Some(json!({
            "cleMcpId": l.requise.0, "outil": "piege", "issue": "envoi_depose",
            "dureeMs": 3, "validation": "required", "envoi": 42,
        }))
    );
}

// ---------------------------------------------------------------------------
// AC-b2 — ni arguments, ni données, ni nom fourni par le client
// ---------------------------------------------------------------------------

const PIEGE_ARG: &str = "PIEGE_ARG_326";
const PIEGE_NOM: &str = "PIEGE_NOM_326";

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn journal_sans_arguments_ni_donnees(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    compte(&pool, l.societe, l.createur, "1099", PIEGE_E1).await;
    let b = banc(&pool, registre_de_test());

    let servi = b
        .mcp(
            Some(&l.exempte.1),
            &appel(1, "piege", json!({ PIEGE_ARG: PIEGE_ARG })),
        )
        .await;
    // Témoin : l'outil a bien lu la donnée piégée, et l'a envoyée.
    assert!(
        String::from_utf8_lossy(&servi.corps).contains(PIEGE_E1),
        "témoin : la donnée piégée est partie au client"
    );
    let inconnu = b
        .mcp(
            Some(&l.exempte.1),
            &appel(1, PIEGE_NOM, json!({ PIEGE_ARG: PIEGE_ARG })),
        )
        .await;
    assert_eq!(
        inconnu.json(),
        attendu_erreur(1, CODE_PARAMS_INVALIDES, TEXTE_OUTIL_INCONNU)
    );

    // Story 32-8b (AC-B11) : un dépôt piégé puis sa relecture, validée — le
    // piège part au client à la relecture, jamais au journal.
    let depot = b
        .mcp(
            Some(&l.requise.1),
            &appel(2, "piege", json!({ PIEGE_ARG: PIEGE_ARG })),
        )
        .await;
    let n = numero_depose(&depot);
    decider_en_base(&pool, n, l.createur, true).await;
    let relu = b.mcp(Some(&l.requise.1), &lire_envoi(3, n)).await;
    assert!(
        String::from_utf8_lossy(&relu.corps).contains(PIEGE_E1),
        "témoin : la relecture validée porte la donnée piégée"
    );

    let es = entrees(&pool).await;
    assert_eq!(es.len(), 4);
    for (i, e) in es.iter().enumerate() {
        let texte = e.details.to_string();
        for piege in [PIEGE_ARG, PIEGE_NOM, PIEGE_E1] {
            assert!(!texte.contains(piege), "{piege} au détail : {texte}");
            assert!(!e.actor_label.contains(piege), "{piege} au libellé");
        }
        let mut cles: Vec<&str> = e
            .details
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        cles.sort_unstable();
        // Cinq clés, ou six avec `envoi` pour un dépôt et une relecture (AC-b2
        // de la 32-6b, amendée par l'AC-B11 de la 32-8b). L'attendu se fixe
        // par l'indice, non par la donnée contrôlée : les deux premiers appels
        // à cinq clés, le dépôt et la relecture à six (revue de code P1, A-L2).
        let attendues: &[&str] = if i >= 2 {
            &[
                "cleMcpId",
                "dureeMs",
                "envoi",
                "issue",
                "outil",
                "validation",
            ]
        } else {
            &["cleMcpId", "dureeMs", "issue", "outil", "validation"]
        };
        assert_eq!(cles, attendues, "les clés de l'AC-b1 : {texte}");
    }
    // Le second appel est écrit sous le nom constant, non sous le nom reçu.
    assert_eq!(es[1].details["outil"], json!("(inconnu)"));
    assert_eq!(
        (
            es[2].details["issue"].clone(),
            es[2].details["envoi"].clone()
        ),
        (json!("envoi_depose"), json!(n))
    );
    assert_eq!(
        (
            es[3].details["issue"].clone(),
            es[3].details["envoi"].clone()
        ),
        (json!("servi"), json!(n))
    );
}

// ---------------------------------------------------------------------------
// AC-b3 — le journal fail closed
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn journal_indisponible_rien_ne_part(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    compte(&pool, l.societe, l.createur, "1099", PIEGE_E1).await;
    let b = banc(&pool, registre_de_test());

    // Témoin : journal disponible, l'appel est servi avec la donnée piégée.
    let servi = b
        .mcp(Some(&l.exempte.1), &appel(1, "piege", json!({})))
        .await;
    assert!(String::from_utf8_lossy(&servi.corps).contains(PIEGE_E1));

    // L'authentification (`mcp_keys`) et l'outil (`accounts`) restent
    // possibles ; seule l'écriture du journal échoue.
    sqlx::query("ALTER TABLE audit_log RENAME TO audit_log_cachee_326")
        .execute(&pool)
        .await
        .unwrap();
    let r = b
        .mcp(Some(&l.exempte.1), &appel(1, "piege", json!({})))
        .await;
    assert_eq!(r.statut, StatusCode::OK);
    assert_eq!(r.json(), attendu_outil_en_erreur(1, TEXTE_ERREUR_INTERNE));
    assert!(!String::from_utf8_lossy(&r.corps).contains(PIEGE_E1));
}

// ---------------------------------------------------------------------------
// AC-b4 — ce qui n'est pas journalisé
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ce_qui_n_est_pas_journalise(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(&pool, registre_de_test());
    let avant = lignes(&pool).await;

    // Un 401 : clé inconnue, puis aucune clé.
    let r = b
        .mcp(Some("kesh_mcp_inconnue"), &appel(1, "piege", json!({})))
        .await;
    assert_eq!(r.statut, StatusCode::UNAUTHORIZED);
    let r = b.mcp(None, &appel(1, "piege", json!({}))).await;
    assert_eq!(r.statut, StatusCode::UNAUTHORIZED);
    // Les métadonnées.
    for corps in [
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/list"}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"ping"}"#,
        r#"{"jsonrpc":"2.0","id":1,"method":"prompts/list"}"#,
    ] {
        let r = b.mcp(Some(&l.exempte.1), corps).await;
        assert_eq!(r.statut, StatusCode::OK, "{corps}");
    }
    // Une notification `tools/call` : jamais remise au répartiteur.
    let r = b
        .mcp(
            Some(&l.exempte.1),
            &json!({"jsonrpc":"2.0","method":"tools/call","params":{"name":"piege"}}).to_string(),
        )
        .await;
    assert_eq!(r.statut, StatusCode::ACCEPTED);
    assert_eq!(lignes(&pool).await, avant, "aucune ligne ajoutée");

    // Témoin : le compte voit bien un appel d'outil.
    b.mcp(Some(&l.exempte.1), &appel(1, "piege", json!({})))
        .await;
    assert_eq!(
        lignes(&pool).await,
        avant + 1,
        "témoin : un tools/call compte"
    );
}

// ---------------------------------------------------------------------------
// C-32-6-58, règle 3 — un appel abandonné par le client est journalisé
// ---------------------------------------------------------------------------

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn appel_abandonne_journalise(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let b = banc(
        &pool,
        Registre::vide()
            .avec(Box::new(LecteurRetarde {
                attente: Duration::from_millis(2000),
            }))
            .avec_delai(Duration::from_secs(10)),
    );
    // Le client abandonne la requête au bout de 1 s (le futur du
    // gestionnaire est abandonné, comme quand hyper voit la connexion coupée).
    // La coupure doit tomber APRÈS le lancement de la tâche de l'appel —
    // authentification comprise — et AVANT la fin de l'outil (2 s). *(Marges
    // élargies, issue #761 : coupure 100 ms → 1 s, attente 500 ms → 2 s ; à
    // 100 ms, un abandon tombé avant le lancement de la tâche a fait rougir le
    // test à tort en CI, comme pour #759.)*
    let coupe = tokio::time::timeout(
        Duration::from_secs(1),
        b.mcp(Some(&l.exempte.1), &appel(1, "lecteur_retarde", json!({}))),
    )
    .await;
    assert!(coupe.is_err(), "la requête est abandonnée en vol");
    assert!(
        entrees(&pool).await.is_empty(),
        "témoin : l'appel n'est pas encore allé au bout"
    );
    // La tâche de l'appel va au bout et écrit son entrée.
    let debut = std::time::Instant::now();
    let mut es = entrees(&pool).await;
    while es.is_empty() && debut.elapsed() < Duration::from_secs(10) {
        tokio::time::sleep(Duration::from_millis(50)).await;
        es = entrees(&pool).await;
    }
    assert_eq!(es.len(), 1, "l'appel abandonné est journalisé, une fois");
    let mut e = es.pop().unwrap();
    let duree = retirer_duree(&mut e.details);
    assert_eq!(
        e.details,
        attendu(l.exempte.0, "lecteur_retarde", "servi", "exempt")
    );
    assert!(
        duree >= 2000,
        "l'issue est celle de l'outil allé au bout : {duree}"
    );
}

/// Le compteur du registre : un appel journalisé n'exécute l'outil qu'une fois
/// (aucune ré-exécution pour écrire le journal).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_journal_n_execute_pas_l_outil_deux_fois(pool: MySqlPool) {
    let l = livres_servis(&pool).await;
    let (compteur, n) = Compteur::nouveau("compteur", Portee::BooksRead);
    let b = banc(&pool, Registre::vide().avec(Box::new(compteur)));
    let (r, _) = un_appel_une_ligne(
        &pool,
        &b,
        &l,
        &l.exempte,
        &appel(1, "compteur", json!({})),
        attendu(l.exempte.0, "compteur", "servi", "exempt"),
    )
    .await;
    // Le texte de l'outil `Compteur` : `{"n":1}` (v016-1 : `content` seul).
    assert_eq!(
        r.json()["result"]["content"][0]["text"],
        json!(json!({ "n": 1 }).to_string())
    );
    assert_eq!(n.load(Ordering::SeqCst), 1);
}
