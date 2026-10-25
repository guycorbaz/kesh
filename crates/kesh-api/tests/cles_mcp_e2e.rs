//! Story 32-5b (refs #723) — l'authentification par clé MCP.
//!
//! - AC-b3 : `services::cles_mcp::authentifier` — succès (portées, politique,
//!   société, créateur, dernière utilisation), et le même 401 pour chaque
//!   refus ; le préfixe contrôlé **avant** tout accès à la base, prouvé par un
//!   pool vers un port fermé (α) et son témoin (β) ; une erreur de base n'est
//!   jamais un 401.
//! - AC-b4 : refus croisés — une clé MCP à `/api/v1` (en-tête et cookie) et à
//!   une route d'administration → 401, sans dernière utilisation ; un PAT
//!   actif à `authentifier` → 401, sans dernière utilisation ; un PAT recopié
//!   derrière le préfixe MCP → 401.
//! - AC-b5 : la séparation lecture / trace à l'exécution — `authentifier`
//!   réussit sur une `Lecture` en `READ ONLY`, la dernière utilisation passe
//!   par la `Trace` ; témoin : `touch_last_used` sur cette `Lecture` échoue en
//!   1792.
//!
//! L'AC-b6 (une clé restaurée authentifie) vit dans `admin_full_import_e2e.rs`,
//! dont il réutilise le montage (C-32-5-23).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une base éphémère par test).

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use chrono::{NaiveDateTime, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::api_key::{MCP_PREFIX, PAT_PREFIX, generate_mcp_key, generate_pat};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::errors::AppError;
use kesh_api::services::acces::{Canal, Portee};
use kesh_api::services::cles_mcp::{ROLE_CLE_MCP, authentifier};
use kesh_api::services::poignees::{Lecture, Trace};
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    ApiKeyScope, Language, McpScope, McpScopes, NewApiKey, NewCompany, NewMcpKey, NewUser, OrgType,
    Role, SendValidation,
};
use kesh_db::errors::DbError;
use kesh_db::repositories::{api_keys, companies, mcp_keys, users};
use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;

use common::lecture_seule::pool_lecture_seule;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ---------------------------------------------------------------------------
// Montage
// ---------------------------------------------------------------------------

async fn societe(pool: &MySqlPool, nom: &str) -> i64 {
    companies::create(
        pool,
        NewCompany {
            name: nom.into(),
            first_name: None,
            last_name: None,
            address_structured: StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .expect("société")
    .id
}

async fn utilisateur(pool: &MySqlPool, company_id: i64, nom: &str, role: Role) -> i64 {
    users::create(
        pool,
        NewUser {
            username: nom.into(),
            password_hash: "$argon2id$v=19$m=19456,t=2,p=1$dGVzdA$dGVzdA".into(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("utilisateur")
    .id
}

/// Une société et son administrateur.
async fn livres(pool: &MySqlPool) -> (i64, i64) {
    let c = societe(pool, "Livres MCP").await;
    let u = utilisateur(pool, c, "admin-mcp", Role::Admin).await;
    (c, u)
}

/// Crée une clé MCP par le dépôt ; rend `(id, jeton)`.
async fn cle_mcp(
    pool: &MySqlPool,
    company_id: i64,
    user_id: i64,
    portees: &[McpScope],
    validation: SendValidation,
    expires_at: Option<NaiveDateTime>,
) -> (i64, String) {
    let (jeton, key_hash) = generate_mcp_key();
    let mut tx = pool.begin().await.unwrap();
    let id = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id,
            created_by_user_id: user_id,
            name: "agent".into(),
            key_hash,
            scopes: McpScopes::new(portees.iter().copied()).unwrap(),
            send_validation: validation,
            expires_at,
        },
    )
    .await
    .expect("clé MCP")
    .id;
    tx.commit().await.unwrap();
    (id, jeton)
}

/// Une clé MCP `books:read`, politique par défaut, sans expiration.
async fn cle_livres(pool: &MySqlPool, c: i64, u: i64) -> (i64, String) {
    cle_mcp(
        pool,
        c,
        u,
        &[McpScope::BooksRead],
        SendValidation::Required,
        None,
    )
    .await
}

/// Crée un PAT `read-write` actif ; rend `(id, jeton)`.
async fn pat(pool: &MySqlPool, company_id: i64, user_id: i64) -> (i64, String) {
    let (jeton, key_hash) = generate_pat();
    let mut tx = pool.begin().await.unwrap();
    let id = api_keys::create_in_tx(
        &mut tx,
        NewApiKey {
            company_id,
            created_by_user_id: user_id,
            name: "intégration".into(),
            key_hash,
            scope: ApiKeyScope::ReadWrite,
            expires_at: None,
        },
    )
    .await
    .expect("PAT")
    .id;
    tx.commit().await.unwrap();
    (id, jeton)
}

async fn derniere_utilisation_mcp(pool: &MySqlPool, id: i64) -> Option<NaiveDateTime> {
    sqlx::query_scalar("SELECT last_used_at FROM mcp_keys WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn derniere_utilisation_pat(pool: &MySqlPool, id: i64) -> Option<NaiveDateTime> {
    sqlx::query_scalar("SELECT last_used_at FROM api_keys WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Les deux poignées de production sur le pool du test.
fn poignees(pool: &MySqlPool) -> (Lecture, Trace) {
    (Lecture::new(pool.clone()), Trace::new(pool.clone()))
}

fn est_401(r: &Result<impl std::fmt::Debug, AppError>) -> bool {
    matches!(r, Err(AppError::Unauthenticated(_)))
}

/// Pool paresseux vers un port **fermé** : toute requête échoue en erreur de
/// connexion, vite. (`127.0.0.1:3306`, le `stub_pool` de `middleware/auth.rs`,
/// est joignable sur la station : il ne convient pas.)
fn pool_injoignable() -> MySqlPool {
    MySqlPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy("mysql://x:x@127.0.0.1:1/x")
        .expect("pool paresseux")
}

// ---------------------------------------------------------------------------
// AC-b3 — authentifier
// ---------------------------------------------------------------------------

/// Succès : le contexte MCP porte la société, le créateur, les portées de la
/// clé ; l'id et la politique sont rendus ; la dernière utilisation est posée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn authentifier_rend_le_contexte_de_la_cle(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_mcp(
        &pool,
        c,
        u,
        &[McpScope::InvoicesRead, McpScope::BooksRead],
        SendValidation::Exempt,
        Some(Utc::now().naive_utc() + TimeDelta::days(30)),
    )
    .await;
    assert_eq!(derniere_utilisation_mcp(&pool, id).await, None, "montage");
    let (lecture, trace) = poignees(&pool);

    let cle = authentifier(&lecture, &trace, &jeton)
        .await
        .expect("authentifiée");

    assert_eq!(cle.cle_id, id);
    assert_eq!(cle.validation, SendValidation::Exempt);
    assert_eq!(cle.acces.company_id(), c);
    assert_eq!(cle.acces.user_id(), u);
    let attendues: BTreeSet<Portee> = [Portee::BooksRead, Portee::InvoicesRead].into();
    assert_eq!(cle.acces.canal(), &Canal::Mcp { portees: attendues });
    assert!(
        derniere_utilisation_mcp(&pool, id).await.is_some(),
        "dernière utilisation posée"
    );
}

/// La politique par défaut d'une clé est `required`, et c'est elle qui est
/// rendue.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn authentifier_rend_la_politique_par_defaut(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (_, jeton) = cle_livres(&pool, c, u).await;
    let (lecture, trace) = poignees(&pool);
    let cle = authentifier(&lecture, &trace, &jeton).await.unwrap();
    assert_eq!(cle.validation, SendValidation::Required);
}

/// Jeton inconnu (préfixe juste, condensat absent) → 401.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_inconnue_refusee(pool: MySqlPool) {
    livres(&pool).await;
    let (jeton, _) = generate_mcp_key();
    let (lecture, trace) = poignees(&pool);
    assert!(est_401(&authentifier(&lecture, &trace, &jeton).await));
}

/// Clé révoquée → 401, et sa dernière utilisation reste vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_revoquee_refusee(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    let mut tx = pool.begin().await.unwrap();
    mcp_keys::revoke_in_tx(&mut tx, c, id, 1).await.unwrap();
    tx.commit().await.unwrap();
    let (lecture, trace) = poignees(&pool);
    assert!(est_401(&authentifier(&lecture, &trace, &jeton).await));
    assert_eq!(derniere_utilisation_mcp(&pool, id).await, None);
}

/// Clé expirée → 401.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_expiree_refusee(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (_, jeton) = cle_mcp(
        &pool,
        c,
        u,
        &[McpScope::BooksRead],
        SendValidation::Required,
        Some(Utc::now().naive_utc() - TimeDelta::minutes(1)),
    )
    .await;
    let (lecture, trace) = poignees(&pool);
    assert!(est_401(&authentifier(&lecture, &trace, &jeton).await));
}

/// Créateur inactif → 401 (mutation Mb5), et sa dernière utilisation reste
/// vide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn createur_inactif_refuse(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    sqlx::query("UPDATE users SET active = FALSE WHERE id = ?")
        .bind(u)
        .execute(&pool)
        .await
        .unwrap();
    let (lecture, trace) = poignees(&pool);
    assert!(est_401(&authentifier(&lecture, &trace, &jeton).await));
    assert_eq!(derniere_utilisation_mcp(&pool, id).await, None);
}

/// Créateur qui n'est plus Administrateur → 401, pour chaque rôle inférieur
/// (hypothèse Q3, mutation Mb4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn createur_non_administrateur_refuse(pool: MySqlPool) {
    assert_eq!(ROLE_CLE_MCP, Role::Admin, "hypothèse Q3");
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    let (lecture, trace) = poignees(&pool);
    for role in [Role::Comptable, Role::Consultation] {
        sqlx::query("UPDATE users SET role = ? WHERE id = ?")
            .bind(role.as_str())
            .bind(u)
            .execute(&pool)
            .await
            .unwrap();
        let r = authentifier(&lecture, &trace, &jeton).await;
        assert!(est_401(&r), "créateur {role:?} : {:?}", r.err());
    }
    assert_eq!(derniere_utilisation_mcp(&pool, id).await, None);
    // Contrôle : rendu Administrateur, le créateur rouvre sa clé.
    sqlx::query("UPDATE users SET role = ? WHERE id = ?")
        .bind(Role::Admin.as_str())
        .bind(u)
        .execute(&pool)
        .await
        .unwrap();
    assert!(authentifier(&lecture, &trace, &jeton).await.is_ok());
}

/// Société absente (clé orpheline, montée sous `FOREIGN_KEY_CHECKS = 0`) → 401.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn societe_absente_refusee(pool: MySqlPool) {
    let (_, u) = livres(&pool).await;
    let (jeton, key_hash) = generate_mcp_key();
    let mut conn = pool.acquire().await.unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(&mut *conn)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes) \
         VALUES (999999, ?, 'orpheline', ?, '[\"books:read\"]')",
    )
    .bind(u)
    .bind(&key_hash)
    .execute(&mut *conn)
    .await
    .unwrap();
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(&mut *conn)
        .await
        .unwrap();
    drop(conn);
    let (lecture, trace) = poignees(&pool);
    assert!(est_401(&authentifier(&lecture, &trace, &jeton).await));
}

/// Une clé dont aucune portée n'est connue de cette version → 401 (le dépôt ne
/// la rend pas, C-32-5-37) ; une clé **mixte** authentifie par ses seules
/// portées connues.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn portees_inconnues(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (lecture, trace) = poignees(&pool);
    for (scopes, attendu) in [
        (r#"["books:write"]"#, None),
        (
            r#"["books:write", "contacts:read"]"#,
            Some(BTreeSet::from([Portee::ContactsRead])),
        ),
    ] {
        let (jeton, key_hash) = generate_mcp_key();
        sqlx::query(
            "INSERT INTO mcp_keys (company_id, created_by_user_id, name, key_hash, scopes) \
             VALUES (?, ?, 'venue d''ailleurs', ?, ?)",
        )
        .bind(c)
        .bind(u)
        .bind(&key_hash)
        .bind(scopes)
        .execute(&pool)
        .await
        .unwrap();
        let r = authentifier(&lecture, &trace, &jeton).await;
        match attendu {
            None => assert!(est_401(&r), "{scopes} : {:?}", r.err()),
            Some(portees) => assert_eq!(
                r.expect("clé mixte").acces.canal(),
                &Canal::Mcp { portees },
                "{scopes}"
            ),
        }
    }
}

/// Le préfixe se compare aux octets : casse, espace de tête, préfixe tronqué,
/// jeton vide → 401.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn prefixe_exact(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (_, jeton) = cle_livres(&pool, c, u).await;
    let corps = jeton.strip_prefix(MCP_PREFIX).unwrap();
    let (lecture, trace) = poignees(&pool);
    for faux in [
        format!("KESH_MCP_{corps}"),
        format!(" {jeton}"),
        format!("kesh_mcp{corps}"),
        corps.to_string(),
        String::new(),
    ] {
        let r = authentifier(&lecture, &trace, &faux).await;
        assert!(est_401(&r), "{faux:?} : {:?}", r.err());
    }
    assert!(
        authentifier(&lecture, &trace, &jeton).await.is_ok(),
        "contrôle"
    );
}

/// AC-b3 (α) — sans le préfixe, le refus précède **tout** accès à la base : sur
/// un pool injoignable, c'est un 401, non une erreur de base (mutation Mb6).
#[tokio::test]
async fn alpha_sans_prefixe_aucun_acces_a_la_base() {
    let pool = pool_injoignable();
    let (lecture, trace) = poignees(&pool);
    let (pat_jeton, _) = generate_pat();
    for jeton in ["n'importe quoi", pat_jeton.as_str()] {
        let r = authentifier(&lecture, &trace, jeton).await;
        assert!(est_401(&r), "{jeton:?} : {:?}", r.err());
    }
}

/// AC-b3 (β), témoin d'(α) — avec le préfixe, le même pool injoignable rend
/// une erreur de base, **pas** un 401 : sans ce témoin, une implémentation qui
/// rendrait 401 pour tout échec passerait (α) sans contrôler le préfixe
/// d'abord. Et une erreur de base n'est jamais un 401.
#[tokio::test]
async fn beta_avec_prefixe_l_erreur_de_base_se_propage() {
    let pool = pool_injoignable();
    let (lecture, trace) = poignees(&pool);
    let (jeton, _) = generate_mcp_key();
    let r = authentifier(&lecture, &trace, &jeton).await;
    assert!(
        matches!(
            r,
            Err(AppError::Database(DbError::ConnectionUnavailable(_)))
        ),
        "erreur de base attendue : {:?}",
        r.err()
    );
}

// ---------------------------------------------------------------------------
// AC-b4 — refus croisés
// ---------------------------------------------------------------------------

/// AC-b4 (c) — un PAT **actif** `read-write` passé à `authentifier` → 401, et
/// sa dernière utilisation (table `api_keys`) reste inchangée (mutation Mb1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn b4_c_un_pat_n_authentifie_pas_au_mcp(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (pat_id, pat_jeton) = pat(&pool, c, u).await;
    assert!(pat_jeton.starts_with(PAT_PREFIX));
    let avant = derniere_utilisation_pat(&pool, pat_id).await;
    let (lecture, trace) = poignees(&pool);
    let r = authentifier(&lecture, &trace, &pat_jeton).await;
    assert!(est_401(&r), "{:?}", r.err());
    assert_eq!(derniere_utilisation_pat(&pool, pat_id).await, avant);
}

/// AC-b4 (d) — le jeton d'un PAT recopié derrière le préfixe MCP (en entier,
/// ou à la place du préfixe PAT) → 401 : son condensat n'est pas dans
/// `mcp_keys`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn b4_d_un_pat_deguise_n_authentifie_pas(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (pat_id, pat_jeton) = pat(&pool, c, u).await;
    let (lecture, trace) = poignees(&pool);
    for deguise in [
        format!("{MCP_PREFIX}{pat_jeton}"),
        format!(
            "{MCP_PREFIX}{}",
            pat_jeton.strip_prefix(PAT_PREFIX).unwrap()
        ),
    ] {
        let r = authentifier(&lecture, &trace, &deguise).await;
        assert!(est_401(&r), "{deguise} : {:?}", r.err());
    }
    assert_eq!(derniere_utilisation_pat(&pool, pat_id).await, None);
}

// --- AC-b4 (a) et (b), par HTTP ---------------------------------------------

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
}

async fn spawn_app(pool: MySqlPool) -> TestApp {
    let config = test_config();
    let rate_limiter = kesh_api::middleware::rate_limit::RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });
    TestApp {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

fn forge_jwt(user_id: i64, role: &str, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        company_id,
        iat: now,
        exp: now + 3600,
    };
    jsonwebtoken::encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(TEST_JWT_SECRET),
    )
    .unwrap()
}

/// AC-b4 (a) et (b) — une clé MCP **active** `books:read` présentée à
/// `GET /api/v1/accounts` (en-tête `Authorization: Bearer`, puis cookie
/// `kesh_access_token`, deux requêtes distinctes) et à une route
/// d'administration → 401 chaque fois, et sa dernière utilisation reste vide
/// (mutation Mb2). Contrôle : un JWT de l'administrateur ouvre les deux routes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn b4_a_b_une_cle_mcp_n_ouvre_pas_l_api(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(u, "Admin", c);

    for chemin in ["/api/v1/accounts", "/api/v1/admin/backup-schedule"] {
        let controle = app
            .client
            .get(app.url(chemin))
            .header("Authorization", format!("Bearer {jwt}"))
            .send()
            .await
            .unwrap();
        assert_eq!(controle.status(), 200, "contrôle JWT sur {chemin}");

        let par_entete = app
            .client
            .get(app.url(chemin))
            .header("Authorization", format!("Bearer {jeton}"))
            .send()
            .await
            .unwrap();
        assert_eq!(par_entete.status(), 401, "clé MCP en en-tête sur {chemin}");

        let par_cookie = app
            .client
            .get(app.url(chemin))
            .header("Cookie", format!("kesh_access_token={jeton}"))
            .send()
            .await
            .unwrap();
        assert_eq!(par_cookie.status(), 401, "clé MCP en cookie sur {chemin}");
    }
    assert_eq!(
        derniere_utilisation_mcp(&pool, id).await,
        None,
        "l'API ne lit jamais la table des clés MCP"
    );
    // Contrôle : la clé était bien valable — au service MCP, elle authentifie.
    let (lecture, trace) = poignees(&pool);
    assert!(authentifier(&lecture, &trace, &jeton).await.is_ok());
}

// ---------------------------------------------------------------------------
// AC-b5 — la séparation lecture / trace, à l'exécution
// ---------------------------------------------------------------------------

/// `authentifier` réussit sur une `Lecture` en `READ ONLY` et une `Trace`
/// ordinaire, et pose la dernière utilisation (mutation Mb3 : écrite sur la
/// `Lecture`, elle échouerait en 1792 et resterait vide).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn b5_lecture_seule_et_trace(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    let lecture = Lecture::new(pool_lecture_seule(&pool).await);
    let trace = Trace::new(pool.clone());
    authentifier(&lecture, &trace, &jeton)
        .await
        .expect("authentifiée sur une Lecture en lecture seule");
    assert!(
        derniere_utilisation_mcp(&pool, id).await.is_some(),
        "dernière utilisation écrite par la Trace"
    );
}

/// Témoin de `b5_lecture_seule_et_trace` : sur la connexion en lecture seule,
/// l'`UPDATE` de `touch_last_used` **lui-même** échoue en 1792 — sans quoi le
/// test ci-dessus ne prouverait rien de la poignée qu'emploie l'écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn b5_temoin_touch_last_used_refuse_en_lecture_seule(pool: MySqlPool) {
    let (c, u) = livres(&pool).await;
    let (id, jeton) = cle_livres(&pool, c, u).await;
    let ro = pool_lecture_seule(&pool).await;
    let e = mcp_keys::touch_last_used(&ro, &kesh_api::auth::api_key::sha256_hex(&jeton))
        .await
        .expect_err("l'UPDATE doit échouer sur la connexion en lecture seule");
    let numero = match &e {
        DbError::Sqlx(s) => s
            .as_database_error()
            .and_then(|d| d.try_downcast_ref::<sqlx::mysql::MySqlDatabaseError>())
            .map(|m| m.number()),
        _ => None,
    };
    assert_eq!(numero, Some(1792), "erreur attendue 1792 : {e:?}");
    assert_eq!(derniere_utilisation_mcp(&pool, id).await, None);
}
