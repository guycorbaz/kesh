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
use kesh_db::entities::address::StructuredAddress;
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

// ============================================================
// Story 16-1d (#281) — rejeu des backfills après restore, de bout en bout
// ============================================================
//
// # Pourquoi ces six cas ne sont pas substituables par des tests unitaires
//
// Le défaut que 16-1c ferme naît de l'**interaction** entre le chemin d'import
// et des migrations que la PR ne touche pas : `_sqlx_migrations` n'étant pas
// restaurée, une migration de backfill reste marquée appliquée et ne repasse
// jamais. Aucun test unitaire ne voit cette interaction — seul un import réel,
// depuis un backup réel, la met en scène.
//
// # Le montage, et pourquoi il passe par la base source et non par le fichier
//
// Deux gestes sont sûrs sur un `.keshbackup` réel :
//
// - **retirer une colonne des `columnNames` du manifeste** — les octets NDJSON
//   ne bougent pas, donc le `sha256` et le `rowCount` restent valides, et le
//   restore ne cite que les `columnNames` (`backup.rs:468`) : la colonne prend
//   son `DEFAULT` ou `NULL`. C'est ce que font déjà
//   `full_import_refuses_schema_mismatch_400` et son voisin ;
// - **muter la base AVANT l'export**, quand l'état visé est atteignable.
//
// **Supprimer** des lignes NDJSON ne l'est pas : `rezip` ne recalcule pas le
// `rowCount`, que `parse_and_verify` vérifie (`import.rs:163-166`).
//
// # Ce que ces tests N'ont pas le droit d'asserter
//
// Le corps de réponse HTTP de `full-import` est **inchangé** (16-1c, D-C7) : le
// rapport de rejeu vit dans le détail d'audit. Côté `audit_log`, `outcome` est
// le **code d'archive stable** de `ReplayOutcome::code()` — jamais un
// `format!("{:?}")` d'enum, qui dériverait au premier renommage de variant.

use std::collections::HashMap;

use kesh_db::entities::contact::Salutation;
use kesh_db::entities::{ContactType, NewContact, NewInvoice, NewInvoiceLine};
use kesh_db::repositories::{accounts, company_invoice_settings, contacts, invoices};
use rust_decimal_macros::dec;

/// Versions des deux entrées du registre, pour retrouver **l'entrée qu'un cas
/// discrimine** dans le rapport d'audit.
///
/// ⚠️ Chaque cas asserte l'entrée qui le concerne, **retrouvée par sa
/// `version`** — jamais la longueur du vecteur ni son ordre. Sans cette borne,
/// deux des cinq mutations de `T-D3` feraient varier le rapport de cas annoncés
/// verts : la mutation 1 bascule `ROLE_POSTABLE` en `Skipped` dans C3, et la
/// mutation 5 réduit le vecteur à un seul élément, observé par quatre cas.
const REVENUE_BACKFILL: i64 = 20260729000001;
const ROLE_POSTABLE: i64 = 20260722000001;

/// Un compte **titre** du plan PME — parent d'entrées du plan, donc jamais
/// imputable, et jamais mouvementé par la fixture.
///
/// Il est la sonde du backfill **structurel** de `postable` (`20260722000001:161`),
/// que le compte `2979` ne couvre pas : ce dernier n'exerce que le backfill par
/// numéro (`role = 'CurrentYearResult'`, `:177`).
const TITLE_ACCOUNT: &str = "10";

// ⚠️ **C4 n'est pas dans ce fichier, et c'est délibéré** — la numérotation des
// cas saute de C3 à C5. Le cas de transactionnalité (échec injecté *pendant* le
// rejeu) vit en test d'intégration `kesh-db` : cf. 16-1c et `replay_with_registry`.
// Par le chemin HTTP, `AppError::AdminFullImportFailed` rend un `500` générique
// dont le détail est loggé et jamais exposé — l'échec y serait indiscernable de
// celui d'un `INSERT` du restore. D'où **six** cas ici, et non sept.

/// Rejoue l'import du backup forgé et exige un `200`.
///
/// Le geste est identique dans les six cas — seul le manifeste diffère — d'où
/// la factorisation, qui évite six copies de la paire `rezip` + assertion de
/// statut. Les tests **antérieurs** du fichier gardent leur `post_import` nu :
/// ils attendent des `4xx`, pas un succès.
async fn import_ok(app: &TestApp, jwt: &str, manifest: &Value, data: &BTreeMap<String, Vec<u8>>) {
    let resp = post_import(app, jwt, rezip(manifest, data)).await;
    assert_eq!(resp.status(), 200, "l'import doit réussir");
}

/// Une société **complète** : plan comptable réel, exercice ouvert, réglages de
/// facturation, contact.
///
/// ⚠️ `seed_role` ne sait créer qu'une société et un utilisateur — ni plan
/// comptable, ni facture, ni écriture. Sans cette fixture, **C1 — le cas
/// nominal de l'issue #281 — porterait sur un ensemble vide et serait vert
/// pour rien.** C'est le mode d'échec que la mutation 5 de `T-D3` mesure.
struct Business {
    ctx: Ctx,
    accounts: HashMap<String, i64>,
    contact_id: i64,
}

/// Sème une société exploitable de bout en bout.
///
/// ⚠️ **Le plan vient de `bulk_create_from_chart`, et c'est essentiel.**
/// `seed_accounting_company` (`test_fixtures.rs`) insère **cinq comptes en dur**
/// (`1000`, `1100`, `2000`, `3000`, `4000`) **sans jamais renseigner `role`** et
/// **sans le compte `2979`** : C5 y serait inconstructible, son candidat
/// n'existant pas, et C2 n'aurait aucun rôle à réattribuer. Et il ne faut pas
/// davantage s'inspirer d'`accounts_role_backfill.rs`, dont le doc-module
/// **interdit** `bulk_create_from_chart` — sa technique repose sur un montage
/// **partiel** des migrations, impossible ici où `#[sqlx::test]` monte le
/// `MIGRATOR` entier.
async fn seed_business(pool: &MySqlPool, label: &str) -> Business {
    let ctx = seed_admin(pool, label).await;

    // Exercice ouvert LARGE : `validate_invoice` exige un exercice couvrant la
    // date de facture (`invoices.rs:1730-1733`, `FiscalYearInvalid` sinon).
    // Sans lui, la validation échoue, aucune écriture n'est produite, et C1
    // passe à vide.
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 16-1d', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(ctx.company_id)
    .execute(pool)
    .await
    .expect("exercice ouvert");

    let chart = kesh_core::chart_of_accounts::load_chart("Pme").expect("plan comptable PME");
    let created = accounts::bulk_create_from_chart(pool, ctx.company_id, &chart, "fr")
        .await
        .expect("semis du plan comptable");
    let accounts_map: HashMap<String, i64> =
        created.into_iter().map(|a| (a.number, a.id)).collect();

    // `insert_with_defaults` résout 1100 (créance) et 3000 (produit) depuis le
    // plan. Il ne renseigne PAS `default_vat_payable_account_id` — d'où des
    // factures montées SANS TVA, cf. `validated_invoice`.
    company_invoice_settings::insert_with_defaults(pool, ctx.company_id)
        .await
        .expect("réglages de facturation");

    let contact_id = contacts::create(
        pool,
        ctx.user_id,
        NewContact {
            company_id: ctx.company_id,
            contact_type: ContactType::Entreprise,
            name: format!("Client {label}"),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Rue du Test".into()),
            address_building: Some("2".into()),
            address_postal_code: Some("1000".into()),
            address_city: Some("Lausanne".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::default(),
        },
    )
    .await
    .expect("contact client")
    .id;

    Business {
        ctx,
        accounts: accounts_map,
        contact_id,
    }
}

/// Crée puis **valide** une facture par le vrai moteur, produisant son écriture.
///
/// ⚠️ **Deux lignes, et aucune TVA.** Deux lignes parce que le backfill pose le
/// compte de l'écriture sur **toutes** les lignes de la facture, ce qu'une
/// facture mono-ligne ne prouverait pas. Aucune TVA parce que
/// `insert_with_defaults` ne renseigne pas `default_vat_payable_account_id`, que
/// `validate_invoice` exige dès que la facture porte de la TVA
/// (`invoices.rs:1497-1500`, `ConfigurationRequired`). L'exonération est un cas
/// suisse légitime sous le seuil de CHF 100 000.
///
/// ⚠️ Cette combinaison — exonéré **et** validé par le vrai moteur — est
/// **inédite dans le dépôt** : les tests exonérés de 16-1a-bis montent leurs
/// écritures en SQL brut, et son seul test passant par le moteur utilise deux
/// taux non nuls. Ne pas la chercher toute faite.
async fn validated_invoice(pool: &MySqlPool, biz: &Business) -> i64 {
    let invoice_id = invoices::create(
        pool,
        biz.ctx.user_id,
        NewInvoice {
            company_id: biz.ctx.company_id,
            contact_id: biz.contact_id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
            due_date: None,
            payment_terms: None,
            project_id: None,
            lines: vec![
                NewInvoiceLine {
                    description: "Prestation".into(),
                    quantity: dec!(3),
                    unit_price: dec!(33.35),
                    vat_rate: dec!(0),
                    revenue_account_id: None,
                },
                NewInvoiceLine {
                    description: "Marchandise".into(),
                    quantity: dec!(7),
                    unit_price: dec!(12.55),
                    vat_rate: dec!(0),
                    revenue_account_id: None,
                },
            ],
        },
    )
    .await
    .map(|(inv, _)| inv.id)
    .expect("création de la facture par le vrai chemin applicatif");

    invoices::validate_invoice(pool, biz.ctx.company_id, invoice_id, biz.ctx.user_id)
        .await
        .expect("validation par le vrai chemin applicatif");

    invoice_id
}

/// Retire une colonne des `columnNames` du manifeste.
///
/// ⚠️ **Les octets NDJSON ne sont volontairement PAS touchés**, et c'est ce qui
/// rend le geste sûr : `sha256` et `rowCount` restent valides, et le restore ne
/// cite que les `columnNames` (`backup.rs:468`). La colonne prend donc son
/// `DEFAULT` ou `NULL` à la restauration — exactement l'état qu'un backup
/// antérieur à la migration produirait. *(D'où une signature à trois arguments
/// et non quatre : passer `data` laisserait croire qu'il est modifié.)*
fn strip_column(manifest: &mut Value, table: &str, column: &str) {
    let cols = manifest["tables"][table]["columnNames"]
        .as_array_mut()
        .unwrap_or_else(|| panic!("table {table} absente du manifeste"));
    let before = cols.len();
    cols.retain(|c| c != column);
    assert_eq!(
        cols.len(),
        before - 1,
        "la colonne {table}.{column} devait être présente au manifeste avant retrait"
    );
}

/// Le rapport de rejeu, relu depuis le détail d'audit de `admin.full_import`.
async fn backfill_report(pool: &MySqlPool) -> Vec<Value> {
    // `details_json` est de type `JSON` : MariaDB l'expose en `BLOB`, pas en
    // `VARCHAR` — le décoder en `String` échoue au niveau du protocole.
    let details: Vec<u8> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'admin.full_import' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(pool)
    .await
    .expect("entrée d'audit admin.full_import");
    let parsed: Value = serde_json::from_slice(&details).expect("details_json parsable");
    parsed["backfills_replayed"]
        .as_array()
        .expect("clé backfills_replayed présente au détail d'audit")
        .clone()
}

/// L'entrée du rapport **que le cas discrimine**, retrouvée par sa version.
fn entry(report: &[Value], version: i64) -> &Value {
    report
        .iter()
        .find(|e| e["version"] == version)
        .unwrap_or_else(|| panic!("entrée {version} absente du rapport de rejeu : {report:?}"))
}

/// Les `revenue_account_id` des lignes d'une facture, dans l'ordre.
async fn line_accounts(pool: &MySqlPool, invoice_id: i64) -> Vec<Option<i64>> {
    sqlx::query_scalar(
        "SELECT revenue_account_id FROM invoice_lines WHERE invoice_id = ? ORDER BY position",
    )
    .bind(invoice_id)
    .fetch_all(pool)
    .await
    .expect("lecture des comptes de produit des lignes")
}

/// Le rôle d'un compte, par son numéro.
async fn account_role(pool: &MySqlPool, number: &str) -> Option<String> {
    sqlx::query_scalar("SELECT role FROM accounts WHERE number = ?")
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("lecture du rôle de compte")
}

/// L'imputabilité d'un compte, par son numéro.
async fn account_postable(pool: &MySqlPool, number: &str) -> bool {
    sqlx::query_scalar::<_, i8>("SELECT postable FROM accounts WHERE number = ?")
        .bind(number)
        .fetch_one(pool)
        .await
        .expect("lecture de l'imputabilité de compte")
        != 0
}

/// **C1 — le cas nominal de l'issue #281.** Un backup produit avant la
/// migration `20260729000001` n'a pas la colonne `invoice_lines.revenue_account_id` ;
/// sans rejeu, l'import la laisserait `NULL` **définitivement**, la migration
/// restant marquée appliquée.
///
/// # Ce que ce test discrimine
///
/// Le mécanisme entier, de bout en bout. Il tomberait si le rejeu n'était pas
/// câblé, s'il tournait avant la garde de comptage, ou si l'entrée de classe A
/// disparaissait du registre.
///
/// ⚠️ **Sans facture validée dans la source, l'assertion porterait sur un
/// ensemble vide et ce test serait vert pour rien** — c'est précisément ce que
/// mesure la mutation 5 de `T-D3` (registre vidé de son entrée `20260729000001` :
/// C1 **et** C1-bis doivent rougir). D'où la pré-condition explicite ci-dessous.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_replays_revenue_account_backfill(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C1").await;
    let invoice_id = validated_invoice(&pool, &biz).await;
    let revenue = biz.accounts["3000"];

    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "pré-condition : 16-1a matérialise le compte de produit à la validation — \
         sans elle, ce test porterait sur un ensemble vide"
    );

    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (mut manifest, data) = unzip(&backup);
    strip_column(&mut manifest, "invoice_lines", "revenue_account_id");
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "le rejeu doit reposer, sur TOUTES les lignes, le compte que l'écriture a crédité"
    );

    let report = backfill_report(&pool).await;
    let e = entry(&report, REVENUE_BACKFILL);
    // ⚠️ **Ce cas n'épingle PAS la classe déclarée**, et c'est délibéré : il
    // asserte que l'entrée a été REJOUÉE, pas *à quel titre*. Discriminer la
    // classe A de la classe B est le rôle de **C1-bis**, et lui seul.
    //
    // Une rédaction antérieure comparait ici à `"REPLAYED_UNCONDITIONAL"`. La
    // mutation 1 de `T-D3` — classe A rendue conditionnelle — l'a révélé : elle
    // faisait rougir C1 **et** C1-bis, alors qu'elle ne doit tuer que C1-bis.
    // Les deux cas se recouvraient, et la mutation ne pouvait plus mesurer ce
    // qu'elle vise. Le protocole de mutation a donc corrigé le montage, pas
    // seulement constaté la couleur.
    assert!(
        e["outcome"] == "REPLAYED_UNCONDITIONAL" || e["outcome"] == "REPLAYED_SENTINELS_ABSENT",
        "l'entrée doit avoir été REJOUÉE (et non sautée), got {e:?}"
    );
    // Décompte **exact**, et non `>= 2` : la base est fraîche, la facture porte
    // exactement deux lignes, et l'entrée somme ses DEUX `UPDATE` — celui des
    // `invoice_lines` et son miroir `credit_note_lines`, qui ne trouve rien ici
    // faute d'avoir. Une borne lâche laisserait passer un `WHERE` élargi qui
    // toucherait des lignes hors périmètre.
    assert_eq!(
        e["rows_affected"].as_u64().unwrap(),
        2,
        "exactement les deux lignes de la facture, et rien d'autre, got {e:?}"
    );
}

/// **C1-bis — la colonne est PRÉSENTE et vide.** Le backup provient d'un binaire
/// qui portait déjà `revenue_account_id`, mais dont les lignes ne l'avaient pas
/// renseignée (montage : on l'efface dans la base source **avant** l'export).
///
/// # Ce que ce test discrimine
///
/// **La classe A — le rejeu inconditionnel.** Il tombe si l'entrée
/// `20260729000001` est traitée en **classe B** : une sentinelle
/// `(invoice_lines, revenue_account_id)` serait alors **présente**, l'entrée
/// serait `Skipped`, et les lignes resteraient `NULL`.
///
/// C'est le cas qui justifie la décision D-C1 de 16-1c : la colonne est créée
/// par `20260727000001` et remplie par `20260729000001`, deux migrations
/// distinctes — une sentinelle mentirait sur tout backup pris entre les deux.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_replays_class_a_even_when_column_is_present(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C1bis").await;
    let invoice_id = validated_invoice(&pool, &biz).await;
    let revenue = biz.accounts["3000"];

    // L'état visé est atteignable : on mute la base source PUIS on exporte.
    sqlx::query("UPDATE invoice_lines SET revenue_account_id = NULL WHERE invoice_id = ?")
        .bind(invoice_id)
        .execute(&pool)
        .await
        .expect("effacement du compte de produit dans la source");
    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![None, None],
        "pré-condition : la source exporte des lignes vides, colonne PRÉSENTE"
    );

    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (manifest, data) = unzip(&backup);
    assert!(
        manifest["tables"]["invoice_lines"]["columnNames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "revenue_account_id"),
        "montage : la colonne doit être PRÉSENTE au manifeste — c'est tout l'enjeu du cas"
    );
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![Some(revenue), Some(revenue)],
        "classe A : le rejeu est INCONDITIONNEL, la présence de la colonne ne le sautage pas"
    );
    assert_eq!(
        entry(&backfill_report(&pool).await, REVENUE_BACKFILL)["outcome"],
        "REPLAYED_UNCONDITIONAL"
    );
}

/// **C2 — classe B, déclenchement.** Backup sans `accounts.role` ni
/// `accounts.postable` : les deux sentinelles manquent, l'entrée se rejoue.
///
/// # Ce que ce test discrimine
///
/// Le **déclenchement** de la classe B. Sans rejeu, le restore laisserait
/// `role = NULL` partout et `postable` à son `DEFAULT TRUE` — la perte
/// silencieuse décrite par l'issue, qui casse la présentation des fonds propres
/// par rôle **et** la condition d'imputabilité du backfill de 16-1a-bis.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_replays_role_and_postable_when_both_columns_absent(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C2").await;

    assert_eq!(
        account_role(&pool, "1100").await.as_deref(),
        Some("Receivable"),
        "pré-condition : le plan semé porte bien les rôles"
    );
    assert!(
        !account_postable(&pool, TITLE_ACCOUNT).await,
        "pré-condition : un compte titre naît NON imputable ({TITLE_ACCOUNT}) — \
         `is_postable` écarte tout numéro parent d'une entrée du plan"
    );

    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (mut manifest, data) = unzip(&backup);
    strip_column(&mut manifest, "accounts", "role");
    strip_column(&mut manifest, "accounts", "postable");
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    assert_eq!(
        account_role(&pool, "1100").await.as_deref(),
        Some("Receivable"),
        "le rejeu doit réattribuer le rôle de créance au compte 1100"
    );
    assert!(
        !account_postable(&pool, "2979").await,
        "le compte de résultat de l'exercice doit être rendu NON imputable par le rejeu"
    );
    // ⚠️ **Le second `UPDATE` de `postable` ne suffit pas à couvrir l'entrée.**
    // `20260722000001` en porte DEUX : le backfill #2, par numéro via
    // `role = 'CurrentYearResult'` (le `2979` ci-dessus), et le backfill #1,
    // **purement structurel** (`:161`), qui rend non imputable tout compte à
    // enfants actifs jamais mouvementé. La colonne est `NOT NULL DEFAULT TRUE`
    // (`:77`) : retirée du manifeste, elle revient donc à `TRUE` sur TOUT le
    // plan, comptes titres compris. Sans cette assertion, une régression du
    // backfill #1 — ou sa disparition au découpage en statements — laisserait un
    // compte de regroupement imputable après restore sans qu'aucun cas ne rougisse.
    assert!(
        !account_postable(&pool, TITLE_ACCOUNT).await,
        "le backfill STRUCTUREL de `postable` doit aussi avoir été rejoué : \
         {TITLE_ACCOUNT} a des enfants actifs et aucune écriture"
    );

    let report = backfill_report(&pool).await;
    let e = entry(&report, ROLE_POSTABLE);
    assert_eq!(e["outcome"], "REPLAYED_SENTINELS_ABSENT");
    assert!(
        e["rows_affected"].as_u64().unwrap() > 0,
        "un rejeu déclenché qui ne touche AUCUNE ligne serait muet, got {e:?}"
    );
    let missing: Vec<&str> = e["missing_sentinels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(
        missing,
        vec!["accounts.role", "accounts.postable"],
        "les DEUX sentinelles manquent"
    );
}

/// **C2-bis — sémantique OU des sentinelles.** Backup portant `accounts.role`
/// mais **pas** `accounts.postable` : **une seule** sentinelle manque, et cela
/// suffit à déclencher le rejeu.
///
/// # Ce que ce test discrimine
///
/// Le **OU** entre sentinelles. Il tombe si le déclencheur est implémenté en
/// **ET** — l'entrée serait alors sautée et `postable` resterait à son
/// `DEFAULT TRUE`, y compris sur le compte de résultat de l'exercice.
///
/// ⚠️ **Discriminant synthétique assumé.** Les deux colonnes sont ajoutées par
/// le **même** `ALTER TABLE` (`20260722000001:74-77`), donc co-présentes dans
/// tout backup réel : cet état n'est pas atteignable en production et se
/// construit par `strip_column`. Le cas ne décrit donc aucun scénario vécu — il
/// **verrouille la règle** pour les entrées futures, qui pourront porter des
/// sentinelles sur des colonnes de migrations différentes.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_replays_when_only_one_sentinel_is_absent(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C2bis").await;

    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (mut manifest, data) = unzip(&backup);
    strip_column(&mut manifest, "accounts", "postable");
    assert!(
        manifest["tables"]["accounts"]["columnNames"]
            .as_array()
            .unwrap()
            .iter()
            .any(|c| c == "role"),
        "montage : `role` doit RESTER au manifeste — sans quoi le cas testerait C2"
    );
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    let report = backfill_report(&pool).await;
    let e = entry(&report, ROLE_POSTABLE);
    assert_eq!(
        e["outcome"], "REPLAYED_SENTINELS_ABSENT",
        "UNE sentinelle absente suffit — un ET rendrait ici `SKIPPED`"
    );
    let missing: Vec<&str> = e["missing_sentinels"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    assert_eq!(missing, vec!["accounts.postable"]);

    assert!(
        !account_postable(&pool, "2979").await,
        "le rejeu déclenché doit recalculer l'imputabilité du compte de résultat"
    );
}

/// **C3 — classe B, conditionnement.** Backup complet, dont le rôle du compte
/// `1100` a été **délibérément effacé**. Les deux sentinelles sont présentes :
/// la donnée du backup fait foi, l'entrée est **sautée**, et le rôle effacé le
/// reste.
///
/// # Ce que ce test discrimine
///
/// Le **conditionnement** de la classe B — il tombe si l'entrée est rendue
/// inconditionnelle, un rejeu réécrivant alors `'Receivable'` sur un champ que
/// l'administrateur avait vidé.
///
/// ⚠️ **« Poser `role` / `postable` à la main sur des valeurs non standard » ne
/// suffirait PAS** — le montage littéral produirait un test **muet**. Les dix
/// `UPDATE` de rôle sont gardés `role IS NULL`, donc no-op sur un rôle
/// renseigné ; et les deux `UPDATE` de `postable` visent des états que l'API ne
/// peut pas produire, `effective_postable` (`accounts.rs:126-131`) forçant
/// `postable = false` sur un compte à rôle `CurrentYearResult` ou à enfants
/// actifs, **à la création comme à la mise à jour**. Un rejeu inconditionnel
/// serait alors entièrement no-op et la mutation 3 ne rougirait pas.
///
/// Le montage discriminant est **atteignable par l'API** : `PUT /accounts/{id}`
/// documente `role: null` comme l'acte de retrait (`routes/accounts.rs:70`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_skips_role_backfill_when_sentinels_are_present(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C3").await;

    // Le retrait délibéré, dans la base source.
    sqlx::query("UPDATE accounts SET role = NULL WHERE company_id = ? AND number = '1100'")
        .bind(biz.ctx.company_id)
        .execute(&pool)
        .await
        .expect("retrait délibéré du rôle du compte 1100");
    assert_eq!(
        account_role(&pool, "1100").await,
        None,
        "pré-condition : le rôle est effacé dans la source"
    );

    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (manifest, data) = unzip(&backup);
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    assert_eq!(
        account_role(&pool, "1100").await,
        None,
        "la donnée du backup fait foi : un rôle DÉLIBÉRÉMENT effacé doit le rester"
    );

    let report = backfill_report(&pool).await;
    let e = entry(&report, ROLE_POSTABLE);
    assert_eq!(
        e["outcome"], "SKIPPED",
        "sentinelles présentes → skip strict, aucun statement envoyé à la base"
    );
    assert_eq!(
        e["missing_sentinels"].as_array().unwrap().len(),
        0,
        "aucune sentinelle manquante"
    );
    assert_eq!(
        e["rows_affected"].as_u64().unwrap(),
        0,
        "un skip strict ne touche aucune ligne"
    );
}

/// **C5 — l'ordre de rejeu est un invariant.** Backup sans `accounts.role`, sans
/// `accounts.postable` et sans `invoice_lines.revenue_account_id`, dont l'unique
/// candidat du backfill de produit est le **compte de résultat de l'exercice**
/// n° `2979`.
///
/// # Ce que ce test discrimine
///
/// L'**ordre croissant** des versions. En ordre correct, `20260722000001` rend
/// `2979` non imputable **avant** que `20260729000001` ne cherche ses candidats,
/// qu'il restreint à `a.postable = TRUE` : plus aucun candidat, la ligne reste
/// `NULL`. En ordre inversé, le backfill de produit verrait `postable = TRUE`
/// partout — la valeur `DEFAULT` que le restore vient de poser — retiendrait
/// `2979` et l'écrirait sur les lignes.
///
/// ⚠️ **Retirer `postable` du manifeste est INDISPENSABLE.** S'il y restait, il
/// vaudrait déjà `FALSE` sur `2979` : le restore reposerait la valeur telle
/// quelle et le test passerait **dans les deux ordres**, sans rien discriminer.
///
/// ⚠️ **NE PAS AJOUTER de ligne d'écriture : cela INVERSE le test.**
/// `HAVING COUNT(*) = 1` (`20260729000001:264`) compte les candidats de
/// l'écriture **entière**, et une facture canonique en produit déjà exactement
/// un. Avec une ligne de plus : en ordre **correct**, `2979` est écarté et il
/// reste un candidat → la ligne reçoit `3000` et l'attendu échoue ; en ordre
/// **inversé**, deux candidats → `HAVING` échoue → la ligne reste `NULL` et le
/// test passe. Rouge sur l'implémentation correcte, vert sur la fautive.
///
/// ⚠️ **NE PAS créer d'écriture séparée** : `jel.entry_id = i.journal_entry_id`
/// (`:254`) la rendrait invisible au backfill — muet dans les deux ordres.
///
/// Le candidat s'obtient donc en **REPOINTANT** l'unique ligne de crédit de
/// produit, par SQL direct sur la base source. Aucun chemin applicatif ne peut
/// produire cet état : `POST /invoices` refuse un compte de produit non
/// imputable ; `journal_entries::update` passe `enforce_postable = true` en dur
/// (`:936`) et son *grandfather* n'exempte que les comptes **déjà référencés**
/// par l'écriture (`:926-928`) ; et `2979` **naît** non imputable, le seed
/// appliquant `is_postable`, jumelle pure d'`effective_postable`. Le repointage
/// préserve en outre l'équilibre débit/crédit, qu'aucun `CHECK` ne protège.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_import_replays_backfills_in_increasing_version_order(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let biz = seed_business(&pool, "C5").await;
    let invoice_id = validated_invoice(&pool, &biz).await;
    let result_account = biz.accounts["2979"];

    // Repointage de l'unique ligne de crédit de produit vers `2979`.
    let repointed = sqlx::query(
        "UPDATE journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         SET jel.account_id = ? \
         WHERE i.id = ? AND jel.credit > 0 AND jel.credit = i.total_amount",
    )
    .bind(result_account)
    .bind(invoice_id)
    .execute(&pool)
    .await
    .expect("repointage de la ligne de crédit de produit")
    .rows_affected();
    assert_eq!(repointed, 1, "exactement une ligne de crédit à repointer");

    // Assertion de montage : l'écriture compte EXACTEMENT un candidat au sens du
    // critère, et c'est bien `2979`. Sans elle, un montage décalé rendrait le
    // test muet plutôt que discriminant.
    let candidates: Vec<i64> = sqlx::query_scalar(
        "SELECT jel.account_id FROM journal_entry_lines jel \
         JOIN invoices i ON i.journal_entry_id = jel.entry_id \
         WHERE i.id = ? AND jel.credit > 0 AND jel.credit = i.total_amount",
    )
    .bind(invoice_id)
    .fetch_all(&pool)
    .await
    .expect("candidats du critère d'unicité");
    assert_eq!(
        candidates,
        vec![result_account],
        "montage : UN seul candidat, et c'est le compte de résultat de l'exercice"
    );

    // L'état « pré-16-1a » des lignes s'obtient par le seul `strip_column` :
    // retirée des `columnNames`, la colonne n'est pas citée par l'`INSERT` du
    // restore et reprend son défaut, `NULL`. Un `UPDATE … SET NULL` sur la base
    // source avant export serait redondant — c'est déjà ainsi que procède C1.
    let backup = export_backup(&app, &biz.ctx.jwt).await;
    let (mut manifest, data) = unzip(&backup);
    strip_column(&mut manifest, "accounts", "role");
    strip_column(&mut manifest, "accounts", "postable");
    strip_column(&mut manifest, "invoice_lines", "revenue_account_id");
    import_ok(&app, &biz.ctx.jwt, &manifest, &data).await;

    assert!(
        !account_postable(&pool, "2979").await,
        "montage : le rejeu de `20260722000001` doit avoir rendu 2979 non imputable"
    );
    assert_eq!(
        line_accounts(&pool, invoice_id).await,
        vec![None, None],
        "ordre CROISSANT : 2979 est rendu non imputable AVANT que le backfill de produit \
         ne cherche ses candidats, il n'en reste donc aucun et la ligne reste NULL. \
         En ordre inversé, `postable` vaudrait TRUE (valeur DEFAULT posée par le restore) \
         et 2979 serait écrit sur les lignes."
    );
}
