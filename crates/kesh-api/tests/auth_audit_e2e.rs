//! Story 33-7 (#435) — ce que le journal d'audit dit des gestes d'authentification :
//! le changement de mot de passe et la révocation de masse d'un vol de jeton
//! s'y inscrivent ; connexion, déconnexion, renouvellement et battement d'activité
//! ne s'y inscrivent pas (décision de Guy du 2026-11-06).
//!
//! Lus en base, par une table `audit_log` qui ne contient que ce que les gestes
//! du test y ont écrit.

use std::net::SocketAddr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::refresh_tokens::token_digest;
use kesh_db::repositories::users;
use serde_json::{Value, json};
use sqlx::MySqlPool;

mod common;

const MDP: &str = "ancien-mot-de-passe-33-7";
const NOUVEAU_MDP: &str = "nouveau-mot-de-passe-33-7";
const SECRET_JWT: &str = "test-secret-32-bytes-minimum-test-secret-padding";

const PASSWORD_CHANGED: &str = "auth.password_changed";
const SESSION_THEFT: &str = "auth.session_theft_detected";

struct App {
    base: String,
    client: reqwest::Client,
}

impl App {
    fn url(&self, chemin: &str) -> String {
        format!("{}{chemin}", self.base)
    }

    async fn login(&self, username: &str, mdp: &str) -> reqwest::Response {
        self.client
            .post(self.url("/api/v1/auth/login"))
            .json(&json!({ "username": username, "password": mdp }))
            .send()
            .await
            .unwrap()
    }

    /// Une connexion réussie : `(jeton d'accès, jeton de renouvellement)`.
    async fn session(&self, username: &str) -> (String, String) {
        let r = self.login(username, MDP).await;
        assert_eq!(r.status(), 200, "connexion");
        let b: Value = r.json().await.unwrap();
        (
            b["accessToken"].as_str().unwrap().to_string(),
            b["refreshToken"].as_str().unwrap().to_string(),
        )
    }

    async fn refresh(&self, jeton: &str) -> reqwest::Response {
        self.client
            .post(self.url("/api/v1/auth/refresh"))
            .json(&json!({ "refreshToken": jeton }))
            .send()
            .await
            .unwrap()
    }

    /// `PUT /auth/password` sous l'autorisation `bearer`.
    async fn changer_mdp(&self, bearer: &str, courant: &str, nouveau: &str) -> reqwest::Response {
        self.client
            .put(self.url("/api/v1/auth/password"))
            .bearer_auth(bearer)
            .json(&json!({ "currentPassword": courant, "newPassword": nouveau }))
            .send()
            .await
            .unwrap()
    }
}

fn config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
        SECRET_JWT.to_string(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn lancer(pool: MySqlPool) -> App {
    let config = config();
    let limiteur = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(limiteur), i18n);
    let app = build_router(state, "dossier-statique-inexistant".to_string());
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = ecoute.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            ecoute,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    App {
        base: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

/// Un utilisateur de la société de test ; son identifiant.
async fn utilisateur(pool: &MySqlPool, nom: &str, role: Role) -> i64 {
    common::create_test_company(pool).await;
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    users::create(
        pool,
        NewUser {
            username: nom.to_string(),
            password_hash: hash_password(MDP).unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Les entrées de l'action : `(user_id, entity_type, entity_id, actor_type, details_json)`.
type Entree = (i64, String, i64, String, Option<Vec<u8>>);

async fn entrees(pool: &MySqlPool, action: &str) -> Vec<Entree> {
    sqlx::query_as(
        "SELECT user_id, entity_type, entity_id, actor_type, details_json \
         FROM audit_log WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap()
}

/// Recule la rotation d'un jeton au-delà de la grâce (sans `sleep`, patron de
/// `replay_after_rotation_aged`, `auth_e2e.rs`).
async fn reculer_rotation(pool: &MySqlPool, jeton: &str) {
    let r = sqlx::query(
        "UPDATE refresh_tokens SET revoked_at = NOW(3) - INTERVAL 60 SECOND WHERE token = ?",
    )
    .bind(token_digest(jeton))
    .execute(pool)
    .await
    .unwrap();
    assert_eq!(r.rows_affected(), 1, "la rotation est bien reculée");
}

/// Une rotation réussie de `jeton`, reculée hors de la grâce : le jeton est
/// désormais rejouable en vol.
async fn jeton_rejouable(app: &App, pool: &MySqlPool, jeton: &str) {
    assert_eq!(app.refresh(jeton).await.status(), 200, "rotation");
    reculer_rotation(pool, jeton).await;
}

fn code(b: &Value) -> &str {
    b["error"]["code"].as_str().unwrap_or("")
}

// ---------------------------------------------------------------------------
// AC-1 et AC-2 — le changement de mot de passe
// ---------------------------------------------------------------------------

/// AC-1 — une entrée, sur l'utilisateur, sans détail : aucun secret nulle part.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn changement_reussi_une_entree_sans_detail(pool: MySqlPool) {
    let u = utilisateur(&pool, "alice", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (acces, _) = app.session("alice").await;

    let r = app.changer_mdp(&acces, MDP, NOUVEAU_MDP).await;
    assert_eq!(r.status(), 200);

    let e = entrees(&pool, PASSWORD_CHANGED).await;
    assert_eq!(e.len(), 1, "une seule entrée");
    let (user_id, entity_type, entity_id, actor_type, details) = &e[0];
    assert_eq!(*user_id, u);
    assert_eq!(entity_type, "user");
    assert_eq!(*entity_id, u);
    assert_eq!(actor_type, "user");
    assert_eq!(*details, None, "details_json nul");

    // Aucun mot de passe ni condensat dans la ligne, toutes colonnes confondues.
    let condensat: String = sqlx::query_scalar("SELECT password_hash FROM users WHERE id = ?")
        .bind(u)
        .fetch_one(&pool)
        .await
        .unwrap();
    let ligne: String = sqlx::query_scalar(
        "SELECT CAST(CONCAT_WS('|', user_id, actor_label, action, entity_type, entity_id, \
         COALESCE(details_json, ''), actor_type) AS CHAR) FROM audit_log WHERE action = ?",
    )
    .bind(PASSWORD_CHANGED)
    .fetch_one(&pool)
    .await
    .unwrap();
    for secret in [MDP, NOUVEAU_MDP, condensat.as_str()] {
        assert!(!ligne.contains(secret), "secret dans la ligne : {ligne}");
    }
}

/// AC-2 — mot de passe courant faux, ou nouveau refusé : aucune entrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn changement_refuse_aucune_entree(pool: MySqlPool) {
    utilisateur(&pool, "bob", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (acces, _) = app.session("bob").await;

    let r = app
        .changer_mdp(&acces, "mauvais-mot-de-passe", NOUVEAU_MDP)
        .await;
    assert_eq!(r.status(), 401, "mot de passe courant faux");
    let r = app.changer_mdp(&acces, MDP, "court").await;
    assert_eq!(r.status(), 400, "nouveau refusé");

    assert!(entrees(&pool, PASSWORD_CHANGED).await.is_empty());
    assert_eq!(app.login("bob", MDP).await.status(), 200, "inchangé");
}

/// AC-2 — même transaction : si l'entrée ne s'écrit pas, le mot de passe reste
/// l'ancien, et les sessions restent ouvertes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn entree_impossible_le_mot_de_passe_reste_l_ancien(pool: MySqlPool) {
    let u = utilisateur(&pool, "carole", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (acces, renouvellement) = app.session("carole").await;
    sqlx::raw_sql(
        "CREATE TRIGGER t_337_mdp BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = 'auth.password_changed' THEN \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '33-7 atomicity'; END IF; END",
    )
    .execute(&pool)
    .await
    .unwrap();

    let r = app.changer_mdp(&acces, MDP, NOUVEAU_MDP).await;
    assert_eq!(r.status(), 500);

    sqlx::raw_sql("DROP TRIGGER t_337_mdp")
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(
        app.login("carole", MDP).await.status(),
        200,
        "l'ancien vaut"
    );
    assert_eq!(
        app.login("carole", NOUVEAU_MDP).await.status(),
        401,
        "le nouveau ne vaut pas"
    );
    let actifs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND token = ? AND revoked_at IS NULL",
    )
    .bind(u)
    .bind(token_digest(&renouvellement))
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actifs, 1, "la session d'origine reste ouverte");
}

/// AC-2 — une clé `read-write` qui présente le bon mot de passe : 403
/// `API_KEY_OPERATION_FORBIDDEN`, rien d'écrit, aucun cookie. Une clé `read`
/// reste arrêtée avant.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_d_api_ne_change_pas_le_mot_de_passe(pool: MySqlPool) {
    utilisateur(&pool, "denis", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (acces, _) = app.session("denis").await;
    let (_, rw) =
        common::create_key_via_http(&app.client, &app.base, &acces, "rw", "read-write").await;
    let (_, ro) = common::create_key_via_http(&app.client, &app.base, &acces, "ro", "read").await;

    let r = app.changer_mdp(&rw, MDP, NOUVEAU_MDP).await;
    assert_eq!(r.status(), 403);
    assert!(
        r.headers().get_all("set-cookie").iter().next().is_none(),
        "aucun cookie"
    );
    let b: Value = r.json().await.unwrap();
    assert_eq!(code(&b), "API_KEY_OPERATION_FORBIDDEN");

    let r = app.changer_mdp(&ro, MDP, NOUVEAU_MDP).await;
    assert_eq!(r.status(), 403);
    let b: Value = r.json().await.unwrap();
    assert_eq!(code(&b), "API_KEY_READ_ONLY");

    assert!(entrees(&pool, PASSWORD_CHANGED).await.is_empty());
    assert_eq!(app.login("denis", MDP).await.status(), 200, "inchangé");
    assert_eq!(app.login("denis", NOUVEAU_MDP).await.status(), 401);
}

/// AC-2 — le refus précède la lecture du corps : une clé au corps illisible reçoit le
/// même 403 (patron d'`activity`, #899). Une session au corps illisible reçoit,
/// elle, le refus du corps : c'est la sonde qui montre que ce corps est bien refusé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_cle_au_corps_illisible_recoit_le_403(pool: MySqlPool) {
    utilisateur(&pool, "lena", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (acces, _) = app.session("lena").await;
    let (_, rw) =
        common::create_key_via_http(&app.client, &app.base, &acces, "rw", "read-write").await;
    let illisible = |bearer: String| {
        app.client
            .put(app.url("/api/v1/auth/password"))
            .bearer_auth(bearer)
            .header("Content-Type", "application/json")
            .body("{pas du json")
            .send()
    };

    let r = illisible(rw).await.unwrap();
    assert_eq!(r.status(), 403);
    let b: Value = r.json().await.unwrap();
    assert_eq!(code(&b), "API_KEY_OPERATION_FORBIDDEN");

    let r = illisible(acces).await.unwrap();
    assert_eq!(r.status(), 400, "session : le refus du corps, inchangé");
}

// ---------------------------------------------------------------------------
// AC-3 — le vol de jeton
// ---------------------------------------------------------------------------

/// Fait échoir un jeton non révoqué : c'est ainsi que finit une session inactive, dont
/// la ligne reste à `revoked_at IS NULL` jusqu'à la purge du démarrage.
async fn faire_echoir(pool: &MySqlPool, jeton: &str) {
    let r = sqlx::query(
        "UPDATE refresh_tokens SET expires_at = NOW(3) - INTERVAL 1 HOUR WHERE token = ?",
    )
    .bind(token_digest(jeton))
    .execute(pool)
    .await
    .unwrap();
    assert_eq!(r.rows_affected(), 1, "le jeton est bien échu");
}

/// Les lignes de l'utilisateur encore à `revoked_at IS NULL`.
async fn non_revoquees(pool: &MySqlPool, u: i64) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(u)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// M-E1 — `revoked_sessions` compte les sessions vivantes : une ligne échue à côté de
/// deux vivantes est révoquée, sans être comptée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_session_echue_n_est_pas_comptee(pool: MySqlPool) {
    let u = utilisateur(&pool, "marc", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt_a) = app.session("marc").await;
    let (_, rt_echu) = app.session("marc").await;
    let (_, _rt_c) = app.session("marc").await;
    jeton_rejouable(&app, &pool, &rt_a).await;
    faire_echoir(&pool, &rt_echu).await;
    assert_eq!(non_revoquees(&pool, u).await, 3, "montage : trois lignes");

    assert_eq!(app.refresh(&rt_a).await.status(), 401);

    let e = entrees(&pool, SESSION_THEFT).await;
    assert_eq!(e.len(), 1, "une entrée");
    let d: Value = serde_json::from_slice(e[0].4.as_ref().unwrap()).unwrap();
    assert_eq!(
        d,
        json!({ "revoked_sessions": 2 }),
        "deux sessions vivantes"
    );
    assert_eq!(
        non_revoquees(&pool, u).await,
        0,
        "la ligne échue est révoquée aussi"
    );
}

/// M-E1 — aucune session vivante : rien à annoncer, aucune entrée ; la ligne échue est
/// tout de même révoquée, et le rejeu reste un 401.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sans_session_vivante_aucune_entree(pool: MySqlPool) {
    let u = utilisateur(&pool, "nina", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt) = app.session("nina").await;
    let r = app.refresh(&rt).await;
    assert_eq!(r.status(), 200, "rotation");
    let b: Value = r.json().await.unwrap();
    let neuf = b["refreshToken"].as_str().unwrap().to_string();
    reculer_rotation(&pool, &rt).await;
    faire_echoir(&pool, &neuf).await;
    assert_eq!(
        non_revoquees(&pool, u).await,
        1,
        "montage : une ligne échue"
    );

    assert_eq!(app.refresh(&rt).await.status(), 401);

    assert!(entrees(&pool, SESSION_THEFT).await.is_empty());
    assert_eq!(non_revoquees(&pool, u).await, 0, "révoquée quand même");
}

/// L-E5 — un rejeu après une reconnexion du titulaire révoque une session neuve : il
/// s'écrit donc de nouveau. La borne est d'une entrée par lot de sessions.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_rejeu_apres_reconnexion_s_ecrit_de_nouveau(pool: MySqlPool) {
    utilisateur(&pool, "olga", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt) = app.session("olga").await;
    jeton_rejouable(&app, &pool, &rt).await;
    assert_eq!(app.refresh(&rt).await.status(), 401);
    assert_eq!(entrees(&pool, SESSION_THEFT).await.len(), 1);

    let _ = app.session("olga").await;
    assert_eq!(app.refresh(&rt).await.status(), 401);

    let e = entrees(&pool, SESSION_THEFT).await;
    assert_eq!(e.len(), 2, "une entrée par lot de sessions");
    let d: Value = serde_json::from_slice(e[1].4.as_ref().unwrap()).unwrap();
    assert_eq!(d, json!({ "revoked_sessions": 1 }));
}

/// AC-3 — rejeu hors de la grâce : une entrée attribuée au titulaire, le nombre
/// de sessions révoquées juste, 401 inchangé. Second rejeu : 401, rien de plus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn vol_de_jeton_une_entree_puis_plus_rien(pool: MySqlPool) {
    let u = utilisateur(&pool, "eve", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt_a) = app.session("eve").await;
    let (_, _rt_b) = app.session("eve").await;
    // Deux sessions actives après la rotation : le jeton neuf de A, et B.
    jeton_rejouable(&app, &pool, &rt_a).await;

    let r = app.refresh(&rt_a).await;
    assert_eq!(r.status(), 401);
    let corps_premier: Value = r.json().await.unwrap();

    let e = entrees(&pool, SESSION_THEFT).await;
    assert_eq!(e.len(), 1, "une entrée");
    let (user_id, entity_type, entity_id, actor_type, details) = &e[0];
    assert_eq!(*user_id, u, "attribuée au titulaire de la session");
    assert_eq!(entity_type, "user");
    assert_eq!(*entity_id, u);
    assert_eq!(actor_type, "user");
    let d: Value = serde_json::from_slice(details.as_ref().unwrap()).unwrap();
    assert_eq!(d, json!({ "revoked_sessions": 2 }));

    // Second rejeu du même jeton : aucune session ne reste, rien ne s'écrit.
    let r = app.refresh(&rt_a).await;
    assert_eq!(r.status(), 401);
    let corps_second: Value = r.json().await.unwrap();
    assert_eq!(corps_premier, corps_second, "même réponse");
    assert_eq!(entrees(&pool, SESSION_THEFT).await.len(), 1, "rien de plus");
}

/// AC-3 — l'entrée est au mieux : si elle ne s'écrit pas, le 401 et la
/// révocation tiennent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn vol_de_jeton_entree_impossible_la_revocation_tient(pool: MySqlPool) {
    let u = utilisateur(&pool, "fanny", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt) = app.session("fanny").await;
    jeton_rejouable(&app, &pool, &rt).await;
    sqlx::raw_sql(
        "CREATE TRIGGER t_337_vol BEFORE INSERT ON audit_log FOR EACH ROW \
         BEGIN IF NEW.action = 'auth.session_theft_detected' THEN \
         SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = '33-7 best effort'; END IF; END",
    )
    .execute(&pool)
    .await
    .unwrap();

    assert_eq!(app.refresh(&rt).await.status(), 401);

    sqlx::raw_sql("DROP TRIGGER t_337_vol")
        .execute(&pool)
        .await
        .unwrap();
    assert!(entrees(&pool, SESSION_THEFT).await.is_empty());
    let actifs: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens WHERE user_id = ? AND revoked_at IS NULL",
    )
    .bind(u)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(actifs, 0, "révocation faite");
}

/// AC-3 — rejeu dans la grâce : aucune entrée (et sessions intactes).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rejeu_dans_la_grace_aucune_entree(pool: MySqlPool) {
    utilisateur(&pool, "gaston", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt) = app.session("gaston").await;
    assert_eq!(app.refresh(&rt).await.status(), 200, "rotation");

    assert_eq!(app.refresh(&rt).await.status(), 401, "rejeu dans la grâce");
    assert!(entrees(&pool, SESSION_THEFT).await.is_empty());
}

/// AC-3 — refus simple (jeton révoqué par une déconnexion) : aucune entrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn jeton_de_session_fermee_aucune_entree(pool: MySqlPool) {
    utilisateur(&pool, "helene", Role::Comptable).await;
    let app = lancer(pool.clone()).await;
    let (_, rt) = app.session("helene").await;
    let r = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({ "refreshToken": rt }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "déconnexion");

    assert_eq!(app.refresh(&rt).await.status(), 401);
    assert!(entrees(&pool, SESSION_THEFT).await.is_empty());
}

// ---------------------------------------------------------------------------
// AC-4 — ce qui ne se trace pas, avec sa sonde positive
// ---------------------------------------------------------------------------

/// AC-4 — connexion, renouvellement, battement, déconnexion : aucune écriture au
/// journal. **Sonde positive** : le vol de jeton, sur la même table et la même
/// requête de comptage, en écrit une — sans quoi « 0 » pourrait venir d'un
/// comptage mort.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_gestes_de_session_ne_se_tracent_pas(pool: MySqlPool) {
    utilisateur(&pool, "ines", Role::Comptable).await;
    let app = lancer(pool.clone()).await;

    let (_, rt) = app.session("ines").await;
    let r = app.refresh(&rt).await;
    assert_eq!(r.status(), 200, "renouvellement ordinaire");
    let b: Value = r.json().await.unwrap();
    let acces = b["accessToken"].as_str().unwrap().to_string();
    let rt2 = b["refreshToken"].as_str().unwrap().to_string();
    let r = app
        .client
        .post(app.url("/api/v1/auth/activity"))
        .header(
            "Cookie",
            format!("kesh_access_token={acces}; kesh_refresh_token={rt2}"),
        )
        .json(&json!({ "idleMs": 0 }))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "battement : {}", r.status());
    let r = app
        .client
        .post(app.url("/api/v1/auth/logout"))
        .json(&json!({ "refreshToken": rt2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 204, "déconnexion");
    assert_eq!(common::audit_count(&pool).await, 0, "aucune entrée");

    // La sonde positive.
    let (_, rt3) = app.session("ines").await;
    jeton_rejouable(&app, &pool, &rt3).await;
    assert_eq!(app.refresh(&rt3).await.status(), 401);
    assert_eq!(common::audit_count(&pool).await, 1, "le vol, lui, s'écrit");
}

// ---------------------------------------------------------------------------
// AC-5 — le vocabulaire
// ---------------------------------------------------------------------------

/// AC-5 — les deux actions se filtrent à l'écran : la route de vocabulaire les
/// rend, traduites.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_deux_actions_sont_au_vocabulaire(pool: MySqlPool) {
    utilisateur(&pool, "julie", Role::Admin).await;
    let app = lancer(pool.clone()).await;
    let (acces, _) = app.session("julie").await;

    let r = app
        .client
        .get(app.url("/api/v1/audit-log/vocabulary"))
        .bearer_auth(&acces)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200);
    let b: Value = r.json().await.unwrap();
    let actions = b["actions"].as_array().unwrap();
    for code in [PASSWORD_CHANGED, SESSION_THEFT] {
        let a = actions
            .iter()
            .find(|a| a["code"] == code)
            .unwrap_or_else(|| panic!("{code} absente du vocabulaire"));
        assert_ne!(a["label"], code, "{code} : un libellé, non le code brut");
    }
}
