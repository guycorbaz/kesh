//! Tests E2E HTTP Story 17-3c — Import complet d'installation (`.keshbackup`).
//!
//! Couverture AC :
//! - AC11 : succès → 200 + JSON `{ backupCreated, tablesRestored, rowsRestored,
//!   sourceVersion, sessionInvalidated:true }` ; RBAC non-Admin 403 ; anti-PAT 403
//! - AC12 : refus 409 version incompatible ; 400 SHA tamper ; 400 formatVersion ;
//!   400 IMPORT_SCHEMA_MISMATCH (colonne source inconnue)
//! - AC14/AC16/O-1 : round-trip — l'import **remplace** l'état destination,
//!   audit `admin.full_import` inséré avec `user_id = MIN(admin)` **source**
//!   (≠ caller, jamais de viol FK)
//! - AC17 : rollback transactionnel sur INSERT en échec → destination intacte
//! - DC11 : import sur instance onboardée → `onboarding_state` reste « done »
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).
//! ⚠️ `GET_LOCK('kesh_full_import')` est server-wide : en parallèle, les imports
//! se sérialisent (CI tourne `--test-threads=1`).

use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, users};
use reqwest::multipart;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

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
        TEST_ADMIN_PASSWORD.to_string(),
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
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if std::time::Instant::now() < deadline => {
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server not ready in 2s: {e}"),
        }
    }
    TestApp {
        base_url: format!("http://{}", addr),
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

struct Ctx {
    #[allow(dead_code)]
    company_id: i64,
    user_id: i64,
    jwt: String,
}

async fn seed_admin(pool: &MySqlPool, label: &str) -> Ctx {
    seed_role(pool, label, Role::Admin).await
}

async fn seed_role(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: format!("CI {label}"),
            address: "Rue Test 1".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;

    let user_id = users::create(
        pool,
        NewUser {
            username: format!("{label}_user"),
            password_hash: hash_password("password123").unwrap(),
            role,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;

    let jwt = forge_jwt(user_id, role.as_str(), company_id);
    Ctx {
        company_id,
        user_id,
        jwt,
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut s = String::with_capacity(64);
    for b in digest {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

async fn export_backup(app: &TestApp, jwt: &str) -> Vec<u8> {
    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "export préalable doit réussir");
    resp.bytes().await.unwrap().to_vec()
}

fn backup_form(bytes: Vec<u8>) -> multipart::Form {
    multipart::Form::new().part(
        "file",
        multipart::Part::bytes(bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    )
}

async fn post_import(app: &TestApp, jwt: &str, bytes: Vec<u8>) -> reqwest::Response {
    app.client
        .post(app.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {jwt}"))
        .multipart(backup_form(bytes))
        .send()
        .await
        .unwrap()
}

/// Décompose un `.keshbackup` en (manifest JSON, {table → ndjson bytes}).
fn unzip(bytes: &[u8]) -> (Value, BTreeMap<String, Vec<u8>>) {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
    let mut manifest = Value::Null;
    let mut data = BTreeMap::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).unwrap();
        let name = entry.name().to_string();
        if name == "manifest.json" {
            let mut s = String::new();
            entry.read_to_string(&mut s).unwrap();
            manifest = serde_json::from_str(&s).unwrap();
        } else if let Some(t) = name
            .strip_prefix("data/")
            .and_then(|n| n.strip_suffix(".ndjson"))
        {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();
            data.insert(t.to_string(), buf);
        }
    }
    (manifest, data)
}

/// Reconstruit un `.keshbackup` depuis (manifest, données) — pour forger des
/// backups invalides à partir d'un export réel.
fn rezip(manifest: &Value, data: &BTreeMap<String, Vec<u8>>) -> Vec<u8> {
    let mut cursor = std::io::Cursor::new(Vec::<u8>::new());
    {
        let mut zip = ZipWriter::new(&mut cursor);
        let opts: SimpleFileOptions =
            SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        for (table, ndjson) in data {
            zip.start_file(format!("data/{table}.ndjson"), opts)
                .unwrap();
            zip.write_all(ndjson).unwrap();
        }
        zip.add_directory("files/", opts).unwrap();
        zip.start_file("manifest.json", opts).unwrap();
        zip.write_all(&serde_json::to_vec_pretty(manifest).unwrap())
            .unwrap();
        zip.finish().unwrap();
    }
    cursor.into_inner()
}

// ============================================================
// AC14/AC16/O-1 — round-trip : remplacement + audit user_id source
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_round_trip_replaces_state_and_audits_source_admin(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // Source : 1 admin A + sa company. C'est l'état à sauvegarder.
    let a = seed_admin(&pool, "Source").await;
    let backup = export_backup(&app, &a.jwt).await;

    // Destination divergente : un 2e admin B (id > A) + une company « Ghost ».
    let b = seed_admin(&pool, "Dest").await;
    assert!(b.user_id != a.user_id, "B doit avoir un id distinct de A");
    let ghost = companies::create(
        &pool,
        NewCompany {
            name: "Ghost SA".into(),
            address: "À supprimer".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    let _ = ghost;

    // Import du backup source via le JWT de B (caller ≠ admin source).
    let resp = post_import(&app, &b.jwt, backup).await;
    assert_eq!(resp.status(), 200, "import → 200");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["sessionInvalidated"], true);
    assert_eq!(body["sourceVersion"], env!("CARGO_PKG_VERSION"));
    assert!(body["tablesRestored"].as_u64().unwrap() >= 1);
    assert_eq!(body["backupCreated"], true);

    // L'état destination est REMPLACÉ par la source : seule la company de A
    // subsiste (B + Ghost effacés).
    let company_names: Vec<String> = sqlx::query_scalar("SELECT name FROM companies ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(
        company_names,
        vec!["CI Source".to_string()],
        "seule la company source doit subsister : {company_names:?}"
    );

    // Seul l'admin A subsiste (B effacé).
    let admin_ids: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM users WHERE role = 'Admin' ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(admin_ids, vec![a.user_id], "seul l'admin source subsiste");

    // O-1 : l'audit import porte user_id = MIN(admin) SOURCE (= A), PAS le
    // caller B (qui n'existe plus → aurait violé la FK audit_log.user_id).
    let (audit_uid, actor): (i64, String) = sqlx::query_as(
        "SELECT user_id, actor_type FROM audit_log \
         WHERE action = 'admin.full_import' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("audit row admin.full_import");
    assert_eq!(audit_uid, a.user_id, "audit user_id = admin source (O-1)");
    assert_ne!(audit_uid, b.user_id, "audit user_id ≠ caller (O-1)");
    assert_eq!(actor, "user");

    // T-C7 : « login admin source possible » — le password_hash de A a été
    // fidèlement restauré ⇒ on peut se reconnecter avec ses identifiants.
    let login = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({"username": "Source_user", "password": "password123"}))
        .send()
        .await
        .unwrap();
    assert_eq!(
        login.status(),
        200,
        "login admin source possible après import (password_hash + FK intègres)"
    );
}

// ============================================================
// AC11 — RBAC + anti-PAT
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    for (label, role) in [
        ("comptable", Role::Comptable),
        ("consultation", Role::Consultation),
    ] {
        let ctx = seed_role(&pool, label, role).await;
        let resp = post_import(&app, &ctx.jwt, backup.clone()).await;
        assert_eq!(resp.status(), 403, "{label} → 403");
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_via_pat_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    // Crée un PAT read-write via l'API (JWT admin).
    let create = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {}", admin.jwt))
        .json(&serde_json::json!({ "name": "ci-pat", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 201);
    let key = create.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string();

    let resp = post_import(&app, &key, backup).await;
    assert_eq!(resp.status(), 403, "import via PAT → 403");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "API_KEY_MANAGEMENT_FORBIDDEN");
}

// ============================================================
// AC12 — refus version / SHA / format / schéma
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_refuses_incompatible_version_409(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    // Force keshVersionMinRequired très élevé → downgrade impossible.
    let (mut manifest, data) = unzip(&backup);
    manifest["keshVersionMinRequired"] = Value::from("99.0.0");
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &admin.jwt, forged).await;
    assert_eq!(resp.status(), 409, "version incompatible → 409");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "IMPORT_VERSION_INCOMPATIBLE");
    assert_eq!(body["error"]["details"]["sourceMinRequired"], "99.0.0");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_refuses_sha_tamper_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    // Altère le NDJSON companies sans mettre à jour le sha du manifeste.
    let (manifest, mut data) = unzip(&backup);
    let companies_ndjson = data.get_mut("companies").unwrap();
    companies_ndjson.extend_from_slice(b"{\"id\":424242,\"name\":\"Injected\"}\n");
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &admin.jwt, forged).await;
    assert_eq!(resp.status(), 400, "SHA tamper → 400");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_BACKUP_STRUCTURE");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_refuses_unknown_format_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    let (mut manifest, data) = unzip(&backup);
    manifest["formatVersion"] = Value::from(2);
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &admin.jwt, forged).await;
    assert_eq!(resp.status(), 400, "formatVersion 2 → 400");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INVALID_BACKUP_STRUCTURE");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_refuses_schema_mismatch_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    // Ajoute une colonne inexistante aux columnNames de companies (la source
    // « connaît » une colonne que la destination ignore) → unknownColumns.
    // Le sha du NDJSON reste valide (on ne touche pas aux bytes data).
    let (mut manifest, data) = unzip(&backup);
    let cols = manifest["tables"]["companies"]["columnNames"]
        .as_array_mut()
        .unwrap();
    cols.push(Value::from("colonne_fantome"));
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &admin.jwt, forged).await;
    assert_eq!(resp.status(), 400, "colonne inconnue → 400");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "IMPORT_SCHEMA_MISMATCH");
    assert_eq!(body["error"]["details"]["table"], "companies");
    assert!(
        body["error"]["details"]["unknownColumns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "colonne_fantome")
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_refuses_missing_required_column_400(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let backup = export_backup(&app, &admin.jwt).await;

    // Retire une colonne destination NOT NULL sans défaut (companies.name) des
    // columnNames source → chemin c2 `missingRequiredColumns`. Le NDJSON (bytes)
    // est inchangé donc le SHA reste valide ; le rejet vient du check schéma.
    let (mut manifest, data) = unzip(&backup);
    let cols = manifest["tables"]["companies"]["columnNames"]
        .as_array_mut()
        .unwrap();
    cols.retain(|c| c != "name");
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &admin.jwt, forged).await;
    assert_eq!(
        resp.status(),
        400,
        "colonne requise absente de la source → 400"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "IMPORT_SCHEMA_MISMATCH");
    assert_eq!(body["error"]["details"]["table"], "companies");
    assert!(
        body["error"]["details"]["missingRequiredColumns"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "name"),
        "missingRequiredColumns doit contenir 'name'"
    );
}

// ============================================================
// AC17 — rollback transactionnel : destination intacte sur échec restore
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_rolls_back_on_insert_failure(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let keep = seed_admin(&pool, "KeepMe").await;
    let _ = keep;
    let backup = export_backup(&app, &keep.jwt).await;

    // Forge un backup où la company contient un ide_number invalide (viole
    // chk_companies_ide_format, CHECK appliqué même sous FOREIGN_KEY_CHECKS=0)
    // → l'INSERT échoue en cours de restore → rollback. On conserve TOUTES les
    // colonnes réelles (sinon IMPORT_SCHEMA_MISMATCH 400 court-circuiterait
    // avant l'INSERT) — seul `ide_number` est rendu invalide.
    let (mut manifest, mut data) = unzip(&backup);
    let first_line = std::str::from_utf8(data.get("companies").unwrap())
        .unwrap()
        .lines()
        .next()
        .expect("au moins une company dans le backup")
        .to_string();
    let mut obj: serde_json::Map<String, Value> = serde_json::from_str(&first_line).unwrap();
    obj.insert("ide_number".to_string(), Value::from("PAS-UN-IDE"));
    let ndjson = format!("{}\n", serde_json::to_string(&Value::Object(obj)).unwrap()).into_bytes();
    data.insert("companies".to_string(), ndjson.clone());
    manifest["tables"]["companies"]["sha256"] = Value::from(sha256_hex(&ndjson));
    // rowCount et columnNames inchangés (1 ligne, colonnes réelles).
    let forged = rezip(&manifest, &data);

    let resp = post_import(&app, &keep.jwt, forged).await;
    assert_eq!(resp.status(), 500, "INSERT invalide → 500 (rollback)");

    // Destination intacte : la company KeepMe d'origine est toujours là.
    let names: Vec<String> = sqlx::query_scalar("SELECT name FROM companies ORDER BY name")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(
        names,
        vec!["CI KeepMe".to_string()],
        "rollback : l'état destination doit être intact, got {names:?}"
    );
    // L'admin d'origine subsiste aussi (pas d'état mi-effacé).
    let admin_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'Admin'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(admin_count, 1, "admin d'origine préservé après rollback");
}

// ============================================================
// DC11 — onboarding forcé « done » post-import (anti catch-22 #120)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_forces_onboarding_done(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let admin = seed_admin(&pool, "Acme").await;
    let _ = admin;

    // Destination « non onboardée » : onboarding_state à l'étape 0.
    sqlx::query("INSERT INTO onboarding_state (step_completed, is_demo) VALUES (0, FALSE)")
        .execute(&pool)
        .await
        .unwrap();

    let backup = export_backup(&app, &admin.jwt).await;
    let resp = post_import(&app, &admin.jwt, backup).await;
    assert_eq!(resp.status(), 200);

    // onboarding_state forcé « done » (>= 7) car la source a 1 company non-stub
    // + 1 admin → pas de réouverture du catch-22.
    let step: i32 = sqlx::query_scalar("SELECT step_completed FROM onboarding_state LIMIT 1")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        step >= 7,
        "onboarding forcé done (step_completed >= 7), got {step}"
    );
}
