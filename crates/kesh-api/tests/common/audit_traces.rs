//! Le relevé « toutes traces » d'un geste fait par une clé d'API (Story 36-4,
//! #1131) et la création d'une clé par HTTP.
//!
//! Un test relève l'identifiant maximal d'`audit_log` **avant** le geste
//! ([`max_audit_id`]), puis lit **toutes** les entrées écrites depuis
//! ([`audit_depuis`]) et exige qu'elles portent la clé
//! ([`assert_toutes_a_la_cle`]). Un site futur du chemin qui perdrait la clé
//! rougit quelle que soit sa forme : l'assertion ne nomme pas les sites, elle
//! lit ce qui a été écrit.
//!
//! Inclus par `common/mod.rs` et, sous un module `common` en ligne, par les
//! binaires qui n'incluent pas `mod.rs` (`invoice_settlement_response_e2e.rs`,
//! `supplier_settlement_cancel_e2e.rs`) — d'où `dead_code` autorisé : chaque
//! binaire n'en emploie qu'une partie.
#![allow(dead_code)]

use sqlx::MySqlPool;

/// Une entrée d'audit telle que la lit [`audit_depuis`] :
/// `(action, actor_type, actor_api_key_id, user_id)`.
pub type Trace = (String, String, Option<i64>, i64);

/// L'identifiant maximal d'`audit_log`, `0` sur une table vide.
pub async fn max_audit_id(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(id) FROM audit_log")
        .fetch_one(pool)
        .await
        .expect("lecture de l'identifiant maximal d'audit")
        .unwrap_or(0)
}

/// Les entrées d'audit d'identifiant **strictement supérieur** à `id_min`,
/// dans l'ordre de leur écriture (`ORDER BY id`).
pub async fn audit_depuis(pool: &MySqlPool, id_min: i64) -> Vec<Trace> {
    sqlx::query_as(
        "SELECT action, actor_type, actor_api_key_id, user_id FROM audit_log \
         WHERE id > ? ORDER BY id",
    )
    .bind(id_min)
    .fetch_all(pool)
    .await
    .expect("lecture des entrées d'audit du geste")
}

/// Le créateur d'une clé d'API (`api_keys.created_by_user_id`) : l'`user_id`
/// qu'une entrée écrite par la clé doit porter.
pub async fn createur_de_la_cle(pool: &MySqlPool, cle: i64) -> i64 {
    sqlx::query_scalar("SELECT created_by_user_id FROM api_keys WHERE id = ?")
        .bind(cle)
        .fetch_one(pool)
        .await
        .expect("lecture du créateur de la clé")
}

/// L'identifiant de l'utilisateur `username` — l'auteur attendu des entrées
/// d'un geste fait par session.
pub async fn user_id_de(pool: &MySqlPool, username: &str) -> i64 {
    sqlx::query_scalar("SELECT id FROM users WHERE username = ?")
        .bind(username)
        .fetch_one(pool)
        .await
        .expect("lecture de l'utilisateur")
}

/// Le nombre de lignes d'écriture lettrées d'origine `document` — le lettrage
/// de la pièce qu'une annulation de règlement défait. Un test le relève avant
/// le geste : sans lui, le délettrage ne serait pas exercé.
pub async fn lignes_lettrees_document(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM journal_entry_lines WHERE lettering_origin = 'document'",
    )
    .fetch_one(pool)
    .await
    .expect("lecture des lignes lettrées")
}

/// ⛔ Toutes les `traces` portent la clé : `actor_type = 'api_key'`,
/// `actor_api_key_id = cle`, `user_id = createur` ; et leurs actions sont
/// **exactement** `actions_attendues`, **multiplicité comprise** (liste triée
/// contre liste triée) — une action manquante, en trop ou doublée échoue.
///
/// Échoue aussi sur une liste vide (un geste qui n'écrit rien ne prouve rien),
/// et nomme **toutes** les traces fautives, non la première seule.
pub fn assert_toutes_a_la_cle(
    traces: &[Trace],
    cle: i64,
    createur: i64,
    actions_attendues: &[&str],
) {
    assert_toutes_par(traces, ("api_key", Some(cle), createur), actions_attendues);
}

/// Le témoin de session (AC4 de la Story 36-4) : toutes les `traces` sont à
/// `('user', NULL, user_id)`, actions exactement `actions_attendues`, aux
/// mêmes conditions que [`assert_toutes_a_la_cle`].
pub fn assert_toutes_a_l_utilisateur(traces: &[Trace], user_id: i64, actions_attendues: &[&str]) {
    assert_toutes_par(traces, ("user", None, user_id), actions_attendues);
}

/// Le cœur des deux assertions : chaque trace porte exactement
/// `(actor_type, actor_api_key_id, user_id)` = `acteur`, et les actions sont
/// exactement `actions_attendues`, multiplicité comprise.
fn assert_toutes_par(
    traces: &[Trace],
    acteur: (&str, Option<i64>, i64),
    actions_attendues: &[&str],
) {
    assert!(
        !traces.is_empty(),
        "le geste n'a écrit aucune entrée d'audit"
    );
    let fautives: Vec<&Trace> = traces
        .iter()
        .filter(|(_, actor_type, key, user_id)| (actor_type.as_str(), *key, *user_id) != acteur)
        .collect();
    assert!(
        fautives.is_empty(),
        "entrées du geste qui ne sont pas à {acteur:?} : {fautives:?}"
    );
    let mut vues: Vec<&str> = traces.iter().map(|(a, ..)| a.as_str()).collect();
    vues.sort_unstable();
    let mut attendues: Vec<&str> = actions_attendues.to_vec();
    attendues.sort_unstable();
    assert_eq!(
        vues, attendues,
        "actions du geste (triées, multiplicité comprise)"
    );
}

/// Crée un jeton d'API par l'endpoint HTTP, **sous un JWT** (la route est
/// interdite aux jetons), et rend `(id, secret clair kesh_pat_…)` — Story
/// 15-7b1 (R2-1/F2-7 de sa validation), remonté de `api_keys_e2e.rs` pour être
/// partagé avec `onboarding_audit_e2e.rs` ; déplacé ici de `common/mod.rs`, qui
/// le réexporte, à la Story 36-4 (C-36-4-5).
///
/// Paramétré par le client et l'URL de base, chaque fichier de test ayant son
/// propre `TestApp`. `scope` vaut `"read"` ou `"read-write"` — un jeton en
/// lecture seule prend 403 sur toute route mutante.
pub async fn create_key_via_http(
    client: &reqwest::Client,
    base_url: &str,
    jwt: &str,
    name: &str,
    scope: &str,
) -> (i64, String) {
    let resp = client
        .post(format!("{base_url}/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {jwt}"))
        .json(&serde_json::json!({ "name": name, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé doit réussir");
    let body: serde_json::Value = resp.json().await.unwrap();
    let id = body["id"].as_i64().unwrap();
    let key = body["key"].as_str().unwrap().to_string();
    assert!(key.starts_with("kesh_pat_"), "secret au format attendu");
    (id, key)
}
