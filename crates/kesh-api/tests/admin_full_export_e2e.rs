//! Tests E2E HTTP Story 17-3a — Export complet d'installation (`.keshbackup`).
//!
//! Couverture AC :
//! - AC1 : succès → 200 + `application/octet-stream` + Content-Disposition + ZIP PK\x03\x04
//! - AC1 : RBAC non-Admin → 403 ; non authentifié → 401
//! - AC2 : anti-PAT → 403 (Bearer kesh_pat_…)
//! - AC3/AC4 : structure ZIP (manifest.json root + data/<table>.ndjson ×26 + files/) + manifest shape
//! - AC4 : columnNames exclut la colonne générée `reconciliation_rules.active_uniq`
//! - AC5 : intégrité SHA-256 (recompute == manifest)
//! - AC6 : audit `admin.full_export` inséré
//! - AC7 : chemin streaming (plafond in-memory bas) délivre un ZIP valide
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).

use std::io::Read;
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
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

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

async fn spawn_app_with(pool: MySqlPool, config: Config) -> TestApp {
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

async fn spawn_app(pool: MySqlPool) -> TestApp {
    spawn_app_with(pool, test_config()).await
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
    #[allow(dead_code)]
    user_id: i64,
    jwt: String,
}

async fn seed(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
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

// ============================================================
// AC1 — succès
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_success_returns_keshbackup(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 200, "admin export → 200");
    assert_eq!(
        resp.headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/octet-stream")
    );
    let cd = resp
        .headers()
        .get("content-disposition")
        .and_then(|v| v.to_str().ok())
        .unwrap()
        .to_string();
    assert!(
        cd.contains("attachment"),
        "Content-Disposition attachment: {cd}"
    );
    assert!(cd.contains(".keshbackup"), "filename .keshbackup: {cd}");

    let bytes = resp.bytes().await.unwrap();
    assert_eq!(
        &bytes[0..4],
        &[0x50, 0x4B, 0x03, 0x04],
        "signature ZIP PK\\x03\\x04"
    );
}

// ============================================================
// AC3/AC4/AC5 — structure + manifest + intégrité
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_structure_manifest_and_integrity(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    // manifest.json au root + dossier files/ + 32 data/<table>.ndjson
    // (23 + 3 tables avoirs Story 12.1 : credit_notes, credit_note_lines,
    // credit_note_number_sequences + 2 tables factures fournisseurs Story 12.2 :
    // supplier_invoices, supplier_invoice_lines + 2 tables lots paiement Story 12.3 :
    // payment_batches, payment_batch_items + 1 table import Story 12.5b :
    // imported_supplier_invoices + 1 table projets Story 19-1 : projects).
    assert!(
        names.contains(&"manifest.json".to_string()),
        "manifest.json présent"
    );
    assert!(
        names.contains(&"files/".to_string()),
        "dossier files/ présent"
    );
    let data_count = names
        .iter()
        .filter(|n| n.starts_with("data/") && n.ends_with(".ndjson"))
        .count();
    assert_eq!(
        data_count, 32,
        "32 fichiers data/<table>.ndjson : {names:?}"
    );

    // Lire manifest.json.
    let manifest: Value = {
        let mut f = archive.by_name("manifest.json").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        serde_json::from_str(&s).unwrap()
    };

    // AC4 — shape.
    assert_eq!(manifest["formatVersion"], 1);
    assert_eq!(manifest["keshVersion"], env!("CARGO_PKG_VERSION"));
    assert!(manifest["keshVersionMinRequired"].is_string());
    assert!(
        manifest["instanceId"].as_i64().unwrap() >= 1,
        "instanceId ≥ 1"
    );
    assert!(manifest["exportDate"].as_str().unwrap().ends_with('Z'));

    // AC4 — columnNames exclut la colonne générée active_uniq.
    let rr_cols = manifest["tables"]["reconciliation_rules"]["columnNames"]
        .as_array()
        .unwrap();
    assert!(
        !rr_cols.iter().any(|c| c == "active_uniq"),
        "active_uniq (VIRTUAL) doit être exclue : {rr_cols:?}"
    );

    // companies a au moins 1 ligne (la company seedée).
    assert!(
        manifest["tables"]["companies"]["rowCount"]
            .as_u64()
            .unwrap()
            >= 1
    );

    // AC5 — intégrité SHA-256 : recompute chaque data/<table>.ndjson.
    let tables: Vec<String> = manifest["tables"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    for table in tables {
        let entry_name = format!("data/{table}.ndjson");
        let mut buf = Vec::new();
        archive
            .by_name(&entry_name)
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let recomputed = sha256_hex(&buf);
        let stored = manifest["tables"][&table]["sha256"].as_str().unwrap();
        assert_eq!(recomputed, stored, "SHA-256 mismatch pour {table}");
    }
}

// ============================================================
// AC1 — RBAC + auth
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_non_admin_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // AC1 — les deux rôles non-Admin sont refusés (Comptable ET Consultation).
    for (label, role) in [
        ("comptable", Role::Comptable),
        ("consultation", Role::Consultation),
    ] {
        let ctx = seed(&pool, label, role).await;
        let resp = app
            .client
            .get(app.url("/api/v1/admin/full-export"))
            .header("Authorization", format!("Bearer {}", ctx.jwt))
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status(), 403, "{label} → 403");
    }
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_unauthenticated_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let _ = seed(&pool, "Acme", Role::Admin).await;

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401, "sans token → 401");
}

// ============================================================
// AC2 — anti-PAT
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_via_pat_returns_403(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    // Crée une clé PAT read-write via l'endpoint HTTP (JWT admin).
    let create = app
        .client
        .post(app.url("/api/v1/settings/api-keys"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .json(&json!({ "name": "ci-pat", "scope": "read-write" }))
        .send()
        .await
        .unwrap();
    assert_eq!(create.status(), 201, "création PAT");
    let key = create.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(key.starts_with("kesh_pat_"));

    // L'export via PAT doit être refusé (403), même en read-write.
    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {key}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403, "export via PAT → 403");
    let body: Value = resp.json().await.unwrap();
    assert_eq!(
        body["error"]["code"], "API_KEY_MANAGEMENT_FORBIDDEN",
        "code anti-PAT"
    );
}

// ============================================================
// AC6 — audit
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_is_audited(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let _ = resp.bytes().await.unwrap();

    let (action, entity_type, actor): (String, String, String) = sqlx::query_as(
        "SELECT action, entity_type, actor_type FROM audit_log \
         WHERE action = 'admin.full_export' ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .expect("audit row admin.full_export");
    assert_eq!(action, "admin.full_export");
    assert_eq!(entity_type, "installation");
    assert_eq!(actor, "user", "export via JWT → actor_type user");
}

// ============================================================
// AC7 — chemin streaming (plafond bas force le spill fichier temporaire)
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn full_export_streaming_path_delivers_valid_zip(pool: MySqlPool) {
    let mut config = test_config();
    config.admin_export_inmem_mib = 0; // tout export > 0 octet → chemin streaming
    let app = spawn_app_with(pool.clone(), config).await;
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {}", ctx.jwt))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();
    assert_eq!(
        &bytes[0..4],
        &[0x50, 0x4B, 0x03, 0x04],
        "ZIP valide via streaming"
    );
    // Le ZIP streamé reste lisible.
    let archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    assert!(archive.len() >= 23, "23 data + files/ + manifest");
}
