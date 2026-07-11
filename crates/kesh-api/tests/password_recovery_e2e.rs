//! Tests d'intégration du recovery de mot de passe self-service (Story 17-4e).
//!
//! Couvre AC23 a-m : flux complets `POST /api/v1/auth/forgot-password` +
//! `POST /api/v1/auth/reset-password` (contrats figés 17-4c) — happy path avec
//! révocation des refresh tokens et audits, token expiré/réutilisé,
//! anti-énumération (zéro trace), user sans email, rate-limit partagé,
//! SMTP down (toujours 200), compte inactif (émission ET consommation),
//! email dupliqué actif/inactif, username avec `@`, trim du token,
//! VALIDATION_ERROR qui ne brûle pas le token, feature off → 404.
//!
//! Synchronisation avec la tâche détachée (DE-3) : le handler forgot-password
//! répond AVANT que la task (audit + create token + envoi) ait tourné — les
//! assertions positives passent par un polling borné (`wait_until`), les
//! assertions négatives par une fenêtre de settle bornée puis vérification.

use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::mail::MockMailer;
use kesh_api::middleware::rate_limit::RateLimiter;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{audit_log, password_reset_tokens, users};
use kesh_db::test_fixtures::{seed_stub_company_only, truncate_all};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const OLD_PASSWORD: &str = "old-password-12chars";
const NEW_PASSWORD: &str = "new-password-12chars";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn post_forgot(&self, identifier: &str) -> reqwest::Response {
        self.client
            .post(self.url("/api/v1/auth/forgot-password"))
            .json(&json!({ "identifier": identifier }))
            .send()
            .await
            .expect("POST forgot-password")
    }

    async fn post_reset(&self, token: &str, new_password: &str) -> reqwest::Response {
        self.client
            .post(self.url("/api/v1/auth/reset-password"))
            .json(&json!({ "token": token, "newPassword": new_password }))
            .send()
            .await
            .expect("POST reset-password")
    }

    async fn post_login(&self, username: &str, password: &str) -> reqwest::Response {
        self.client
            .post(self.url("/api/v1/auth/login"))
            .json(&json!({ "username": username, "password": password }))
            .send()
            .await
            .expect("POST login")
    }
}

/// Config de test feature-on (DE-5) : `forgot_password_enabled` + `public_base_url`
/// mutés après construction. PAS de SMTP — le mailer est remplacé par MockMailer.
fn recovery_config() -> Config {
    let mut config = Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "test-admin-password-12chars".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    );
    config.forgot_password_enabled = true;
    config.public_base_url = Some("http://127.0.0.1".to_string());
    config
}

/// Config feature OFF (AC23-m) : défauts de `from_fields_for_test`
/// (`forgot_password_enabled = false`).
fn feature_off_config() -> Config {
    let mut config = recovery_config();
    config.forgot_password_enabled = false;
    config.public_base_url = None;
    config
}

/// Spawn l'app avec un AppState littéral (pattern `setup_admin_e2e.rs:81`) :
/// `mailer` = le MockMailer fourni (cloné — garder l'original comme poignée de
/// lecture), `rate_limiter_recovery` = seuils explicites (DE-4 : permissif par
/// défaut, bas pour le test 429).
async fn spawn_app(
    pool: MySqlPool,
    config: Config,
    mailer: MockMailer,
    recovery_max_attempts: u32,
) -> TestApp {
    let rate_limiter = RateLimiter::new(&config);
    let i18n = Arc::new(
        kesh_i18n::I18nBundle::load(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

    let state = AppState {
        pool,
        config: Arc::new(config),
        rate_limiter: Arc::new(rate_limiter),
        // DE-4 — instance dédiée par test : permissive (1000) pour les tests
        // fonctionnels, seuil bas pour le test rate-limit dédié.
        rate_limiter_recovery: Arc::new(RateLimiter::with_thresholds(
            recovery_max_attempts,
            Duration::from_secs(15 * 60),
            Duration::from_secs(30 * 60),
        )),
        i18n,
        users_exist: Arc::new(AtomicBool::new(true)),
        mailer: Arc::new(mailer),
        // Story 20-3b1 — champs hors scope recovery (défauts).
        rate_limiter_send_email: Arc::new(kesh_api::build_send_email_rate_limiter()),
        smtp_ready: false,
        test_mock_mailer: None,
    };

    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let client = reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("client");

    // Attente active de la connectivité (pattern auth_e2e.rs) + assertion
    // finale (Pass 1 ECH) : un serveur jamais prêt doit échouer ICI avec un
    // message clair, pas en ECONNREFUSED opaque dans le test.
    let mut ready = false;
    for _ in 0..50 {
        if tokio::net::TcpStream::connect(addr).await.is_ok() {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert!(ready, "serveur de test pas prêt après 500 ms ({addr})");

    TestApp {
        base_url: format!("http://{}", addr),
        client,
    }
}

/// Truncate + seed d'une company stub (FK `users.company_id` NOT NULL) —
/// préambule commun de chaque test. Retourne le `company_id`.
async fn reset_db(pool: &MySqlPool) -> i64 {
    truncate_all(pool).await.expect("truncate");
    seed_stub_company_only(pool)
        .await
        .expect("seed stub company")
}

/// Crée un user de test directement via le repo (hash Argon2id réel pour que
/// les assertions de login fonctionnent).
async fn create_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    email: Option<&str>,
    active: bool,
) -> kesh_db::entities::User {
    let hash = kesh_api::auth::password::hash_password_async(OLD_PASSWORD.to_string())
        .await
        .expect("hash");
    users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active,
            company_id,
            email: email.map(|e| e.to_string()),
        },
    )
    .await
    .expect("create user")
}

/// Polling borné (DE-3) : ~2 s max (100 × 20 ms). Panique avec `label` si la
/// condition n'est jamais vraie.
async fn wait_until<F>(label: &str, mut cond: F)
where
    F: FnMut() -> bool,
{
    for _ in 0..100 {
        if cond() {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("wait_until timeout: {label}");
}

/// Variante DB : attend que `SELECT COUNT(*)` atteigne `expected`.
async fn wait_for_token_count(pool: &MySqlPool, expected: i64) {
    for _ in 0..100 {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens")
            .fetch_one(pool)
            .await
            .expect("count tokens");
        if count == expected {
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let last: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens")
        .fetch_one(pool)
        .await
        .unwrap_or(-1);
    panic!("wait_for_token_count timeout (expected {expected}, last seen {last})");
}

/// Fenêtre de settle pour les assertions NÉGATIVES (anti-énum) : laisse à la
/// task détachée le temps de tourner si elle avait été lancée à tort, puis
/// l'appelant vérifie l'absence d'effet. 300 ms >> latence d'un spawn + 2
/// requêtes DB sur loopback.
async fn settle() {
    tokio::time::sleep(Duration::from_millis(300)).await;
}

fn extract_token(reset_url: &str) -> String {
    reset_url
        .split("token=")
        .nth(1)
        .expect("reset_url contient ?token=")
        .to_string()
}

async fn audit_actions(pool: &MySqlPool, user_id: i64) -> Vec<kesh_db::entities::AuditLogEntry> {
    audit_log::find_by_entity(pool, "user", user_id, 50)
        .await
        .expect("find audit")
}

/// Polling borné (~2 s) jusqu'à l'entrée d'audit `auth.password_reset_requested`
/// du user, retournée pour assertions. Pass 1 BH-F3/ECH-F1 : les assertions
/// POSITIVES sur la task détachée ne doivent JAMAIS reposer sur un `settle()`
/// fixe (faux rouge sous charge CI) — toujours ce polling.
async fn wait_for_requested_audit(
    pool: &MySqlPool,
    user_id: i64,
) -> kesh_db::entities::AuditLogEntry {
    for _ in 0..100 {
        if let Some(e) = audit_actions(pool, user_id)
            .await
            .into_iter()
            .find(|e| e.action == "auth.password_reset_requested")
        {
            return e;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("wait_for_requested_audit timeout (user {user_id})");
}

// === AC23-a : happy path complet ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn happy_path_forgot_then_reset_revokes_and_audits(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    let user = create_user(&pool, cid, "alice", Some("alice@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    // 2 logins AVANT le reset → 2 refresh tokens actifs : la révocation doit
    // toucher TOUS les tokens, pas juste un (Pass 1 BH-F2 — avec 1 seule
    // session, `revoked >= 1` serait trivialement vrai même si un bug n'en
    // révoquait qu'un sur N).
    let login = app.post_login("alice", OLD_PASSWORD).await;
    assert_eq!(login.status(), 200, "login initial");
    let login2 = app.post_login("alice", OLD_PASSWORD).await;
    assert_eq!(login2.status(), 200, "2e session");

    // 1. Forgot → 200 immédiat.
    let res = app.post_forgot("alice").await;
    assert_eq!(res.status(), 200);

    // 2. La task détachée envoie le mail (polling DE-3).
    wait_until("mail capturé", || !mock.sent().is_empty()).await;
    let sent = mock.sent();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].to, "alice@example.ch");
    let token = extract_token(&sent[0].reset_url);
    assert_eq!(token.len(), 27, "token base62 largeur fixe");

    // 3. Audit de la demande, recoverable:true (l'audit précède l'envoi dans
    // la task — le mail capturé garantit qu'il est écrit).
    let entries = audit_actions(&pool, user.id).await;
    let requested = entries
        .iter()
        .find(|e| e.action == "auth.password_reset_requested")
        .expect("audit password_reset_requested");
    assert_eq!(
        requested
            .details_json
            .as_ref()
            .and_then(|d| d.get("recoverable"))
            .and_then(Value::as_bool),
        Some(true)
    );

    // 4. Reset → 200 {"status":"ok"}.
    let res = app.post_reset(&token, NEW_PASSWORD).await;
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.expect("json");
    assert_eq!(body["status"], "ok");

    // 5. Ancien mot de passe refusé, nouveau accepté.
    let old_login = app.post_login("alice", OLD_PASSWORD).await;
    assert_eq!(old_login.status(), 401, "ancien mdp doit être refusé");
    let new_login = app.post_login("alice", NEW_PASSWORD).await;
    assert_eq!(new_login.status(), 200, "nouveau mdp doit fonctionner");

    // 6. TOUTES les sessions pré-reset sont révoquées "password_change"
    // (Pass 1 BH-F2 : 2 sessions créées en tête de test — `revoked == 2`
    // prouve la révocation totale, pas juste « au moins une »).
    let revoked: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM refresh_tokens \
         WHERE user_id = ? AND revoked_reason = 'password_change' AND revoked_at IS NOT NULL",
    )
    .bind(user.id)
    .fetch_one(&pool)
    .await
    .expect("count revoked");
    assert_eq!(
        revoked, 2,
        "les 2 sessions pré-reset doivent être révoquées 'password_change'"
    );

    // 7. Audit de la complétion.
    let entries = audit_actions(&pool, user.id).await;
    assert!(
        entries
            .iter()
            .any(|e| e.action == "auth.password_reset_completed"),
        "audit password_reset_completed attendu"
    );
}

// === AC23-b : token expiré ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn expired_token_returns_generic_400(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    let user = create_user(&pool, cid, "bob", Some("bob@example.ch"), true).await;
    let app = spawn_app(pool.clone(), recovery_config(), MockMailer::new(), 1000).await;

    // Token fabriqué direct en DB, expiré depuis 1 h (marge large, defer D3).
    let (token_clear, token_hash) = kesh_api::auth::api_key::generate_reset_token();
    let expired_at = (chrono::Utc::now() - TimeDelta::hours(1)).naive_utc();
    password_reset_tokens::create(&pool, user.id, &token_hash, expired_at)
        .await
        .expect("create expired token");

    let res = app.post_reset(&token_clear, NEW_PASSWORD).await;
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.expect("json");
    assert_eq!(body["error"]["code"], "INVALID_OR_EXPIRED_TOKEN");

    // Mot de passe inchangé.
    let login = app.post_login("bob", OLD_PASSWORD).await;
    assert_eq!(login.status(), 200, "ancien mdp toujours valide");
}

// === AC23-c : double-consume ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reused_token_returns_generic_400(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "carol", Some("carol@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    app.post_forgot("carol").await;
    wait_until("mail capturé", || !mock.sent().is_empty()).await;
    let token = extract_token(&mock.sent()[0].reset_url);

    let first = app.post_reset(&token, NEW_PASSWORD).await;
    assert_eq!(first.status(), 200);

    let second = app.post_reset(&token, "another-password-12chars").await;
    assert_eq!(second.status(), 400, "token déjà consommé");
    let body: Value = second.json().await.expect("json");
    assert_eq!(body["error"]["code"], "INVALID_OR_EXPIRED_TOKEN");
}

// === AC23-d : identifiant inexistant — anti-énumération zéro trace ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn unknown_identifier_returns_200_and_leaves_no_trace(pool: MySqlPool) {
    reset_db(&pool).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    let res = app.post_forgot("ghost").await;
    assert_eq!(res.status(), 200, "anti-énum : 200 même pour un inconnu");
    let res = app.post_forgot("ghost@nowhere.ch").await;
    assert_eq!(res.status(), 200);

    settle().await;
    let tokens: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(tokens, 0, "aucun token créé");
    assert!(mock.sent().is_empty(), "aucun mail envoyé");
    let audits: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .expect("count audit");
    assert_eq!(audits, 0, "aucune entrée d'audit (pas de match)");
}

// === AC23-e : user sans email ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn user_without_email_gets_audit_recoverable_false(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    let user = create_user(&pool, cid, "dave", None, true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    let res = app.post_forgot("dave").await;
    assert_eq!(res.status(), 200);

    // L'audit recoverable:false EST écrit (match) — polling positif (helper
    // partagé, Pass 1 refactor DRY).
    let requested = wait_for_requested_audit(&pool, user.id).await;
    assert_eq!(
        requested
            .details_json
            .as_ref()
            .and_then(|d| d.get("recoverable"))
            .and_then(Value::as_bool),
        Some(false)
    );

    settle().await;
    let tokens: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(tokens, 0, "aucun token pour un compte sans email");
    assert!(mock.sent().is_empty(), "aucun mail");
}

// === AC23-f : rate-limit partagé forgot + reset ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn rate_limit_applies_to_both_endpoints(pool: MySqlPool) {
    reset_db(&pool).await;
    let app = spawn_app(pool.clone(), recovery_config(), MockMailer::new(), 3).await;

    // 2 forgot + 1 reset = 3 slots consommés (limiter PARTAGÉ).
    assert_eq!(app.post_forgot("nobody-1").await.status(), 200);
    assert_eq!(app.post_forgot("nobody-2").await.status(), 200);
    assert_eq!(
        app.post_reset("un-token-quelconque", NEW_PASSWORD)
            .await
            .status(),
        400
    );

    // 4e requête (seuil 3) → 429, sur les DEUX endpoints.
    let res = app.post_forgot("nobody-3").await;
    assert_eq!(res.status(), 429, "forgot bloqué après le seuil");
    let res = app.post_reset("un-autre-token", NEW_PASSWORD).await;
    assert_eq!(res.status(), 429, "reset bloqué par le même limiter");
}

// === AC23-g : SMTP down → toujours 200, token créé ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn smtp_failure_still_returns_200_and_creates_token(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "erin", Some("erin@example.ch"), true).await;
    let app = spawn_app(pool.clone(), recovery_config(), MockMailer::failing(), 1000).await;

    let res = app.post_forgot("erin").await;
    assert_eq!(res.status(), 200, "échec SMTP jamais propagé (oracle DC4)");

    // Le token EST créé (l'échec ne concerne que l'envoi).
    wait_for_token_count(&pool, 1).await;
}

// === AC23-h : compte inactif — émission ET consommation ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn inactive_account_is_not_recoverable(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    let user = create_user(&pool, cid, "frank", Some("frank@example.ch"), false).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    // Émission : match username inactif → audit recoverable:false, pas de mail.
    let res = app.post_forgot("frank").await;
    assert_eq!(res.status(), 200);
    // Polling positif sur l'audit (Pass 1 BH-F3 — pas de settle()+expect),
    // puis settle pour l'assertion NÉGATIVE no-mail (la task early-return
    // juste après l'audit pour un compte inactif).
    let requested = wait_for_requested_audit(&pool, user.id).await;
    assert_eq!(
        requested
            .details_json
            .as_ref()
            .and_then(|d| d.get("recoverable"))
            .and_then(Value::as_bool),
        Some(false)
    );
    settle().await;
    assert!(mock.sent().is_empty(), "pas de mail pour un compte inactif");

    // Consommation : un token valide émis AVANT désactivation → 400 (re-check P3).
    let (token_clear, token_hash) = kesh_api::auth::api_key::generate_reset_token();
    let valid_until = (chrono::Utc::now() + TimeDelta::minutes(30)).naive_utc();
    password_reset_tokens::create(&pool, user.id, &token_hash, valid_until)
        .await
        .expect("create token");
    let res = app.post_reset(&token_clear, NEW_PASSWORD).await;
    assert_eq!(res.status(), 400, "compte inactif → même 400 générique");
    let body: Value = res.json().await.expect("json");
    assert_eq!(body["error"]["code"], "INVALID_OR_EXPIRED_TOKEN");
}

// === AC23-i : email dupliqué ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn duplicate_email_two_actives_is_noop(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "grace1", Some("shared@example.ch"), true).await;
    create_user(&pool, cid, "grace2", Some("shared@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    let res = app.post_forgot("shared@example.ch").await;
    assert_eq!(res.status(), 200);
    settle().await;
    assert!(mock.sent().is_empty(), "2 actifs → comptage ≠ 1 → no-op");
    let tokens: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM password_reset_tokens")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(tokens, 0);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn duplicate_email_active_plus_inactive_recovers_the_active(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    let active = create_user(&pool, cid, "heidi-active", Some("dup@example.ch"), true).await;
    create_user(&pool, cid, "heidi-old", Some("dup@example.ch"), false).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    let res = app.post_forgot("dup@example.ch").await;
    assert_eq!(res.status(), 200);
    wait_until("mail capturé", || !mock.sent().is_empty()).await;
    assert_eq!(mock.sent().len(), 1, "exactement 1 mail (le compte actif)");

    // Le token appartient bien au compte actif (retain P4).
    let owner: i64 = sqlx::query_scalar("SELECT user_id FROM password_reset_tokens LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("owner");
    assert_eq!(owner, active.id);
}

// === AC23-j : username avec `@` (legacy) routé vers le lookup email ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn legacy_username_with_at_is_routed_to_email_lookup(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    // User legacy inséré direct en DB (la garde `@` 17-4c bloque la création
    // via l'API, pas le repo).
    create_user(&pool, cid, "jean@dupont", Some("jean@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    // L'identifiant `jean@dupont` contient `@` → lookup EMAIL (`jean@dupont`
    // ne matche aucun email) → no-op silencieux. Le username n'est jamais tenté.
    let res = app.post_forgot("jean@dupont").await;
    assert_eq!(res.status(), 200);
    settle().await;
    assert!(
        mock.sent().is_empty(),
        "username avec @ non-recouvrable (DC6)"
    );

    // Son EMAIL réel reste la voie de recovery.
    let res = app.post_forgot("jean@example.ch").await;
    assert_eq!(res.status(), 200);
    wait_until("mail via email réel", || !mock.sent().is_empty()).await;
}

// === AC23-k : trim du token ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn reset_token_is_trimmed(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "ivan", Some("ivan@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    app.post_forgot("ivan").await;
    wait_until("mail capturé", || !mock.sent().is_empty()).await;
    let token = extract_token(&mock.sent()[0].reset_url);

    // Copier-coller depuis un client mail qui wrappe : espaces + retour-ligne.
    let padded = format!("  {token}\n");
    let res = app.post_reset(&padded, NEW_PASSWORD).await;
    assert_eq!(res.status(), 200, "le token doit être trimé (P6)");
}

// === AC23-l : VALIDATION_ERROR ne brûle pas le token ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn validation_error_does_not_consume_token(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "judy", Some("judy@example.ch"), true).await;
    let mock = MockMailer::new();
    let app = spawn_app(pool.clone(), recovery_config(), mock.clone(), 1000).await;

    app.post_forgot("judy").await;
    wait_until("mail capturé", || !mock.sent().is_empty()).await;
    let token = extract_token(&mock.sent()[0].reset_url);

    // Mot de passe trop court → 400 VALIDATION_ERROR (la validation précède
    // mark_used).
    let res = app.post_reset(&token, "court").await;
    assert_eq!(res.status(), 400);
    let body: Value = res.json().await.expect("json");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    // Le MÊME token reste consommable.
    let res = app.post_reset(&token, NEW_PASSWORD).await;
    assert_eq!(res.status(), 200, "le token ne doit pas avoir été brûlé");
}

// === AC23-m : feature off → 404 ===

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn feature_off_routes_are_not_mounted(pool: MySqlPool) {
    let cid = reset_db(&pool).await;
    create_user(&pool, cid, "kim", Some("kim@example.ch"), true).await;
    let app = spawn_app(pool.clone(), feature_off_config(), MockMailer::new(), 1000).await;

    // Routes non montées → le POST tombe sur le fallback statique SPA
    // (`fallback_service`, GET-only) → 405 empiriquement (Pass 1 BH-F4 : la
    // valeur exacte est un détail d'implémentation de tower_http::ServeDir —
    // on asserte la PROPRIÉTÉ : 404/405, et AUCUNE sémantique recovery
    // (ni 200 anti-énum, ni 400 token, ni 429 rate-limit).
    let res = app.post_forgot("kim").await;
    assert!(
        [404u16, 405].contains(&res.status().as_u16()),
        "forgot-password non monté (feature off), got {}",
        res.status()
    );
    let res = app.post_reset("token-quelconque", NEW_PASSWORD).await;
    assert!(
        [404u16, 405].contains(&res.status().as_u16()),
        "reset-password non monté (feature off), got {}",
        res.status()
    );
}
