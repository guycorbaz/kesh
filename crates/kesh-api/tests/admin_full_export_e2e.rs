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
use kesh_db::entities::address::StructuredAddress;
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

// ============================================================
// AC1 — succès
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
    // + email_templates (Story 20-1, #224) → 34.
    // + dunning_levels / company_dunning_settings / invoice_reminders → 37.
    // + invoice_settlements (Story 24-2, #371) → 38.
    //
    // ⚠️ **L'export a repris la table tout seul** : il dérive de
    // `backup::TABLES_TO_TRUNCATE`. Ce nombre est donc un CONTRÔLE, pas une
    // déclaration — il rougit quand l'inventaire bouge, ce qui est exactement
    // son office. Il se met à jour en NOMMANT ce qui entre, jamais en alignant
    // le chiffre sur ce qu'on observe.
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
        data_count, 39,
        "39 fichiers data/<table>.ndjson (Story 24-2 : +invoice_settlements ; \
         Story 25-2-c : +journal_entry_number_sequences) : {names:?}"
    );

    // Lire manifest.json.
    let manifest: Value = {
        let mut f = archive.by_name("manifest.json").unwrap();
        let mut s = String::new();
        f.read_to_string(&mut s).unwrap();
        serde_json::from_str(&s).unwrap()
    };

    // AC4 — shape. Story 31-1a (#503) : format 2 ; une installation sans pièce
    // porte deux listes vides et garde l'entrée `files/`.
    assert_eq!(manifest["formatVersion"], 2);
    assert_eq!(manifest["files"], serde_json::json!([]));
    assert_eq!(manifest["filesNotIncluded"], serde_json::json!([]));
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

    // Story 25-1c-zero (refs #378) — `audit_log.company_id` voyage dans le backup.
    // Rien n'a été ajouté à l'export pour elle : il lit les colonnes par
    // `information_schema`. C'est ce que cette assertion prouve, plutôt que de le
    // supposer.
    let audit_cols = manifest["tables"]["audit_log"]["columnNames"]
        .as_array()
        .unwrap();
    assert!(
        audit_cols.iter().any(|c| c == "company_id"),
        "audit_log.company_id doit figurer au manifeste : {audit_cols:?}"
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
    // Story 22-4a (#167) — le code a changé, et le refus vient désormais de la
    // COUCHE posée sur `admin_routes`, non plus du `ensure_not_pat` du handler.
    // Cette garde-là subsiste (décision D3) mais n'est plus atteinte ; c'est
    // l'assertion de source de `admin_pat_denied_e2e` qui prouve son câblage.
    assert_eq!(
        body["error"]["code"], "API_KEY_ADMIN_FORBIDDEN",
        "code anti-PAT"
    );
}

// ============================================================
// AC6 — audit
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

// ============================================================
// Story 31-1a, AC10 — générateur de la fixture `.keshbackup` de la 0.14.0
// ============================================================

/// Chemin de la fixture de format 1 produite par le **vrai** exporteur de la
/// 0.14.0 (Story 31-1a, AC10, T-a0).
const FIXTURE_0_14_0: &str = "tests/fixtures/keshbackup-0.14.0-format1.keshbackup";

/// Génère `tests/fixtures/keshbackup-0.14.0-format1.keshbackup` — **une seule
/// fois, avec le code de la v0.14.0**, jamais ensuite.
///
/// ⚠️ **Pourquoi ce test est `#[ignore]` et doit le rester.** La fixture sert
/// à prouver qu'une sauvegarde produite par une installation 0.14.0 réelle
/// (format 1, `files/` vide) s'importe dans les versions suivantes (AC10).
/// Elle n'a de valeur que si elle sort du code de la 0.14.0 : régénérée par
/// une version ultérieure, elle deviendrait une sauvegarde de format 2 et le
/// test d'import ne prouverait plus rien de l'ancien format. Elle a été
/// produite le 2026-10-24 dans un worktree temporaire au tag `v0.14.0`
/// (`5a128aec`, chaîne Rust 1.96.1), par ce même test, puis versionnée telle
/// quelle ; son empreinte SHA-256 est au Dev Agent Record de la fiche 31-1.
///
/// Contenu, **entièrement synthétique** : la société de
/// `seed_accounting_company`, un client fictif, une facture validée dont le
/// PDF est **figé** par le premier téléchargement (`invoices.pdf_storage_path`,
/// `pdf_sha256`), et un document fournisseur sans QR écrit par
/// `store_document` (`imported_supplier_invoices.storage_path`, `file_hash`).
/// Les deux pièces restent dans le dossier des documents du générateur : la
/// sauvegarde de la 0.14.0 ne les emporte pas — c'est précisément ce que la
/// fixture doit montrer (deux pièces référencées, `files/` vide).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
#[ignore = "générateur de fixture : à n'exécuter qu'au tag v0.14.0 (Story 31-1a, AC10)"]
async fn generate_fixture_0_14_0(pool: MySqlPool) {
    use kesh_db::entities::bank_account::NewBankAccount;
    use kesh_db::entities::contact::{ContactType, NewContact, Salutation};
    use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
    use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
    use kesh_db::repositories::{bank_accounts, contacts, imported_supplier_invoices, invoices};
    use rust_decimal_macros::dec;

    let documents =
        std::env::temp_dir().join(format!("kesh-fixture-0-14-0-docs-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&documents).unwrap();
    let mut config = test_config();
    config.documents_dir = documents.to_string_lossy().to_string();

    let seeded = kesh_db::test_fixtures::seed_accounting_company(&pool)
        .await
        .expect("seed");
    bank_accounts::upsert_primary(
        &pool,
        NewBankAccount {
            company_id: seeded.company_id,
            bank_name: "Banque fictive".into(),
            iban: "CH9300762011623852957".into(),
            qr_iban: Some("CH4431999123000889012".into()),
            is_primary: true,
        },
    )
    .await
    .unwrap();
    let contact = contacts::create(
        &pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Personne,
            name: "Client Fictif".into(),
            first_name: Some("Client".into()),
            last_name: Some("Fictif".into()),
            is_client: true,
            is_supplier: false,
            address: None,
            address_street: Some("Rue de l'Exemple".into()),
            address_building: Some("1".into()),
            address_postal_code: Some("1000".into()),
            address_city: Some("Lausanne".into()),
            address_country: Some("CH".into()),
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: Salutation::Neutre,
        },
    )
    .await
    .unwrap();
    let (invoice, _) = invoices::create(
        &pool,
        seeded.admin_user_id,
        NewInvoice {
            company_id: seeded.company_id,
            contact_id: contact.id,
            date: chrono::NaiveDate::from_ymd_opt(2026, 4, 14).unwrap(),
            due_date: Some(chrono::NaiveDate::from_ymd_opt(2026, 5, 14).unwrap()),
            payment_terms: Some("30 jours net".into()),
            lines: vec![NewInvoiceLine {
                revenue_account_id: None,
                description: "Prestation fictive".into(),
                quantity: dec!(1),
                unit_price: dec!(100.00),
                vat_rate: dec!(8.10),
            }],
            project_id: None,
        },
    )
    .await
    .unwrap();
    invoices::validate_invoice(&pool, seeded.company_id, invoice.id, seeded.admin_user_id)
        .await
        .expect("validation");

    // Document fournisseur synthétique, écrit par la primitive de production.
    let doc = kesh_api::document_storage::store_document(
        &documents,
        b"%PDF-1.4\n% Document fournisseur fictif - fixture Kesh 0.14.0\n%%EOF\n",
        "pdf",
        "facture-fournisseur-fictive.pdf",
        "application/pdf",
    )
    .unwrap();
    imported_supplier_invoices::create(
        &pool,
        &NewImportedSupplierInvoice::without_qr(seeded.company_id, doc),
    )
    .await
    .unwrap();

    let app = spawn_app_with(pool.clone(), config).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    // Premier téléchargement : fige le PDF de la facture.
    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{}/pdf", invoice.id)))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "PDF figé");
    let (frozen,): (Option<String>,) =
        sqlx::query_as("SELECT pdf_storage_path FROM invoices WHERE id = ?")
            .bind(invoice.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(frozen.is_some(), "la facture doit être figée");

    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();

    let out = Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_0_14_0);
    std::fs::create_dir_all(out.parent().unwrap()).unwrap();
    std::fs::write(&out, &bytes).unwrap();
    std::fs::remove_dir_all(&documents).unwrap();
    eprintln!(
        "fixture écrite : {} ({} octets, sha256 {})",
        out.display(),
        bytes.len(),
        sha256_hex(&bytes)
    );
}

// ============================================================
// Story 31-1a (#503) — les pièces dans la sauvegarde (T-a4b)
// ============================================================

#[path = "support/pieces.rs"]
mod pieces;

/// Configuration de test dont le dossier des documents et le dossier de
/// sauvegarde sont propres au test.
fn config_docs(documents: &Path, backup: &Path) -> Config {
    let mut config = test_config();
    config.documents_dir = documents.to_string_lossy().to_string();
    config.admin_backup_dir = backup.to_string_lossy().to_string();
    config
}

async fn get_export(app: &TestApp, jwt: &str) -> reqwest::Response {
    app.client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap()
}

/// Décompose une archive : manifeste, noms des entrées **dans l'ordre**, et
/// contenu des entrées `files/<nom>`.
fn ouvrir(
    bytes: &[u8],
) -> (
    Value,
    Vec<String>,
    std::collections::BTreeMap<String, Vec<u8>>,
) {
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes.to_vec())).unwrap();
    let mut noms = Vec::new();
    let mut fichiers = std::collections::BTreeMap::new();
    let mut manifest = Value::Null;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).unwrap();
        let nom = entry.name().to_string();
        if nom == "manifest.json" {
            let mut s = String::new();
            entry.read_to_string(&mut s).unwrap();
            manifest = serde_json::from_str(&s).unwrap();
        } else if let Some(f) = nom.strip_prefix("files/").filter(|f| !f.is_empty()) {
            let mut buf = Vec::new();
            entry.read_to_end(&mut buf).unwrap();
            fichiers.insert(f.to_string(), buf);
        }
        noms.push(nom);
    }
    (manifest, noms, fichiers)
}

fn non_inclus(manifest: &Value) -> Vec<(String, String)> {
    manifest["filesNotIncluded"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            (
                f["path"].as_str().unwrap().to_string(),
                f["reason"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn entete(resp: &reqwest::Response, nom: &str) -> Option<String> {
    resp.headers()
        .get(nom)
        .map(|v| v.to_str().unwrap().to_string())
}

async fn post_import(app: &TestApp, jwt: &str, bytes: Vec<u8>) -> reqwest::Response {
    let form = reqwest::multipart::Form::new().part(
        "file",
        reqwest::multipart::Part::bytes(bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    app.client
        .post(app.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {jwt}"))
        .multipart(form)
        .send()
        .await
        .unwrap()
}

/// AC3, AC5 — les deux familles de pièces entrent dans l'archive, sous
/// `files/<nom>`, avec leur empreinte et leur taille au manifeste ; l'entrée
/// `files/` précède les pièces ; en-têtes et audit portent les comptes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_format_2_emporte_les_pieces(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, invoice_id) = pieces::seed_company_with_invoice(&pool).await;
    let contenu = b"%PDF-1.4\n% document fournisseur fictif\n%%EOF\n".to_vec();
    let (_, doc_path) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), &contenu, "pdf").await;
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);
    let (pdf_path, pdf) =
        pieces::freeze_pdf(&pool, &app.client, &app.base_url, &jwt, invoice_id).await;

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(entete(&resp, "x-kesh-backup-files").as_deref(), Some("2"));
    assert_eq!(
        entete(&resp, "x-kesh-backup-files-not-included").as_deref(),
        Some("0")
    );
    assert_eq!(entete(&resp, "x-kesh-backup-exceeds-import-limit"), None);
    let bytes = resp.bytes().await.unwrap().to_vec();
    let (manifest, noms, fichiers) = ouvrir(&bytes);

    assert_eq!(manifest["formatVersion"], 2);
    let mut attendu = vec![
        json!({ "path": doc_path, "sha256": sha256_hex(&contenu), "byteSize": contenu.len() }),
        json!({ "path": pdf_path, "sha256": sha256_hex(&pdf), "byteSize": pdf.len() }),
    ];
    attendu.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    assert_eq!(manifest["files"], Value::Array(attendu));
    assert!(non_inclus(&manifest).is_empty());
    assert_eq!(fichiers.get(&doc_path), Some(&contenu));
    assert_eq!(fichiers.get(&pdf_path), Some(&pdf));
    let rang = |n: &str| noms.iter().position(|x| x == n).unwrap();
    assert!(
        rang("files/") < rang(&format!("files/{doc_path}"))
            && rang("files/") < rang(&format!("files/{pdf_path}")),
        "l'entrée files/ précède les pièces : {noms:?}"
    );

    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'admin.full_export' \
         ORDER BY id DESC LIMIT 1",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["file_count"], 2);
    assert_eq!(details["files_bytes"], (contenu.len() + pdf.len()) as u64);
    assert_eq!(details["files_not_included"], 0);
}

/// AC3 — une pièce absente du dossier est listée `missing`, l'export réussit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_piece_absente_listee_missing(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let (_, chemin) = pieces::seed_document(
        &pool,
        seeded.company_id,
        docs.path(),
        b"absent bientot",
        "pdf",
    )
    .await;
    std::fs::remove_file(docs.path().join(&chemin)).unwrap();
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(entete(&resp, "x-kesh-backup-files").as_deref(), Some("0"));
    assert_eq!(
        entete(&resp, "x-kesh-backup-files-not-included").as_deref(),
        Some("1")
    );
    let (manifest, _, fichiers) = ouvrir(&resp.bytes().await.unwrap());
    assert_eq!(non_inclus(&manifest), vec![(chemin, "missing".to_string())]);
    assert!(fichiers.is_empty());
}

/// AC3 (M-a2, C-31-1-53) — une pièce **altérée** suivie d'une pièce saine :
/// l'entrée altérée est abandonnée au milieu de l'archive (`abort_file`), la
/// saine y est, et l'archive **s'importe** sur une installation dont le
/// dossier des documents est vide (la saine est restaurée).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_piece_alteree_abandonnee_et_archive_importable(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let (_, a) = pieces::seed_document(
        &pool,
        seeded.company_id,
        docs.path(),
        b"premier document",
        "pdf",
    )
    .await;
    let (_, b) = pieces::seed_document(
        &pool,
        seeded.company_id,
        docs.path(),
        b"second document",
        "pdf",
    )
    .await;
    let (alteree, saine) = if a < b { (a, b) } else { (b, a) };
    std::fs::write(docs.path().join(&alteree), b"contenu altere apres coup").unwrap();
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();
    let (manifest, noms, fichiers) = ouvrir(&bytes);
    assert_eq!(
        non_inclus(&manifest),
        vec![(alteree.clone(), "altered".to_string())]
    );
    assert!(!noms.contains(&format!("files/{alteree}")), "{noms:?}");
    assert!(fichiers.contains_key(&saine), "{noms:?}");

    let docs_b = pieces::dossier();
    let app_b = spawn_app_with(pool.clone(), config_docs(docs_b.path(), backup.path())).await;
    let resp = post_import(&app_b, &jwt, bytes).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["filesRestored"], 1);
    assert_eq!(body["filesMissing"], 1);
    assert_eq!(
        std::fs::read(docs_b.path().join(&saine)).unwrap(),
        std::fs::read(docs.path().join(&saine)).unwrap(),
        "la pièce saine est restaurée à l'identique"
    );
    assert!(!docs_b.path().join(&alteree).exists());
}

/// Insère une ligne de document fournisseur **telle quelle** (chemin et
/// empreinte au choix du test, éventuellement incohérents).
async fn inserer_document_brut(pool: &MySqlPool, company_id: i64, chemin: &str, sha: &str) {
    sqlx::query(
        "INSERT INTO imported_supplier_invoices \
         (company_id, file_hash, storage_path, original_filename, mime_type, byte_size) \
         VALUES (?, ?, ?, 'doc.pdf', 'application/pdf', 10)",
    )
    .bind(company_id)
    .bind(sha)
    .bind(chemin)
    .execute(pool)
    .await
    .expect("ligne de document");
}

/// Une seconde société (deux lignes de même empreinte n'existent que dans
/// deux sociétés : `UNIQUE (company_id, file_hash)`).
async fn seconde_societe(pool: &MySqlPool) -> i64 {
    seed(pool, "Seconde", Role::Comptable).await.company_id
}

/// AC1 (i), (ii) — un préfixe qui n'est pas l'empreinte de la ligne, et un
/// même chemin déclaré sous deux empreintes : `mismatch`, une seule fois par
/// chemin, aucune entrée dans l'archive — même si un fichier de ce nom existe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_lignes_incoherentes_listees_mismatch(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let autre = seconde_societe(&pool).await;
    // (i) préfixe ≠ empreinte de la ligne ; le fichier existe sous ce nom.
    let faux = format!("{}.pdf", sha256_hex(b"contenu x"));
    std::fs::write(docs.path().join(&faux), b"contenu x").unwrap();
    inserer_document_brut(&pool, seeded.company_id, &faux, &sha256_hex(b"contenu y")).await;
    // (ii) même chemin, deux empreintes (dont la bonne), deux sociétés.
    let (_, double) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"double", "pdf").await;
    inserer_document_brut(&pool, autre, &double, &sha256_hex(b"autre chose")).await;
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let (manifest, _, fichiers) = ouvrir(&resp.bytes().await.unwrap());
    let mut attendu = vec![
        (faux.clone(), "mismatch".to_string()),
        (double.clone(), "mismatch".to_string()),
    ];
    attendu.sort();
    assert_eq!(non_inclus(&manifest), attendu);
    assert!(fichiers.is_empty(), "{:?}", fichiers.keys());
}

/// AC1 (iii), AC6 (M-a18) — des chemins hors forme en base (`/etc/hostname`,
/// `../x.pdf`) sont listés `mismatch` tels quels, sans lecture hors du
/// dossier ; la sauvegarde **s'importe** (200), et ces lignes comptent dans
/// `filesMissing`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_chemins_hors_forme_puis_aller_retour(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let autre = seconde_societe(&pool).await;
    let h = sha256_hex(b"hors forme");
    inserer_document_brut(&pool, seeded.company_id, "/etc/hostname", &h).await;
    inserer_document_brut(&pool, autre, "../x.pdf", &h).await;
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();
    let (manifest, _, fichiers) = ouvrir(&bytes);
    assert_eq!(
        non_inclus(&manifest),
        vec![
            ("../x.pdf".to_string(), "mismatch".to_string()),
            ("/etc/hostname".to_string(), "mismatch".to_string()),
        ]
    );
    assert!(fichiers.is_empty());

    let resp = post_import(&app, &jwt, bytes).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["filesRestored"], 0);
    assert_eq!(body["filesMissing"], 2);
}

/// AC1 (M-a13) — un même contenu importé par deux sociétés sous deux
/// extensions (`x.jpg`, `x.jpeg`) : deux chemins de même empreinte, **deux**
/// pièces incluses.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_deux_chemins_meme_empreinte_tous_deux_inclus(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let autre = seconde_societe(&pool).await;
    let contenu = b"\xFF\xD8\xFF image fictive";
    let (_, jpg) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), contenu, "jpg").await;
    let (_, jpeg) = pieces::seed_document(&pool, autre, docs.path(), contenu, "jpeg").await;
    assert_ne!(jpg, jpeg, "assertion de montage : deux chemins");
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let (manifest, _, fichiers) = ouvrir(&resp.bytes().await.unwrap());
    assert!(non_inclus(&manifest).is_empty(), "{manifest}");
    assert_eq!(fichiers.keys().cloned().collect::<Vec<_>>(), {
        let mut v = vec![jpg, jpeg];
        v.sort();
        v
    });
}

/// AC1 (T-a2, M-a22) — `x….pdf` et `X….pdf` déclarés par deux lignes de même
/// empreinte : deux chemins (la collation de la base les confondrait sous un
/// `UNION`) ; le premier est inclus, le second — préfixe hors `[0-9a-f]` — est
/// `mismatch` ; l'aller-retour rend 200.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_chemins_de_casse_differente_aller_retour(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let autre = seconde_societe(&pool).await;
    let (_, minuscule) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"casse", "pdf").await;
    let h = sha256_hex(b"casse");
    let majuscule = format!("{}.pdf", h.to_uppercase());
    inserer_document_brut(&pool, autre, &majuscule, &h).await;
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap().to_vec();
    let (manifest, _, fichiers) = ouvrir(&bytes);
    assert_eq!(
        fichiers.keys().cloned().collect::<Vec<_>>(),
        vec![minuscule]
    );
    assert_eq!(
        non_inclus(&manifest),
        vec![(majuscule, "mismatch".to_string())]
    );

    let resp = post_import(&app, &jwt, bytes).await;
    assert_eq!(resp.status(), 200, "{}", resp.text().await.unwrap());
}

/// Sous `root`, les droits ne refusent rien : un test qui compte sur un dossier
/// non inscriptible passerait à vide. Il échoue alors **bruyamment**.
fn exiger_un_refus_de_droits(dossier: &Path) {
    assert!(
        std::fs::File::create(dossier.join("sonde")).is_err(),
        "le test s'exécute avec des droits qui ignorent les permissions (root ?) : \
         il ne prouverait rien — relancer sous un utilisateur ordinaire"
    );
}

/// AC3 (M-a20) — une pièce **illisible** (droits `000`) est listée
/// `unreadable` ; l'export rend 200 et les autres pièces sont incluses.
#[cfg(unix)]
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_piece_illisible_listee_unreadable(pool: MySqlPool) {
    use std::os::unix::fs::PermissionsExt;
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let (_, illisible) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"illisible", "pdf").await;
    let (_, lisible) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"lisible", "pdf").await;
    let chemin = docs.path().join(&illisible);
    std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o000)).unwrap();
    assert!(
        std::fs::read(&chemin).is_err(),
        "le test s'exécute avec des droits qui ignorent les permissions (root ?) : \
         il ne prouverait rien — relancer sous un utilisateur ordinaire"
    );
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    std::fs::set_permissions(&chemin, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(resp.status(), 200);
    let (manifest, _, fichiers) = ouvrir(&resp.bytes().await.unwrap());
    assert_eq!(
        non_inclus(&manifest),
        vec![(illisible, "unreadable".to_string())]
    );
    assert_eq!(fichiers.keys().cloned().collect::<Vec<_>>(), vec![lisible]);
}

/// AC3 (M-a21) — un **dossier** à la place d'une pièce s'ouvre puis échoue à
/// la lecture, donc **après** le début de l'entrée : `unreadable`, l'entrée est
/// abandonnée, la pièce saine qui suit est incluse et l'archive se relit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_dossier_a_la_place_de_la_piece(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    let (_, a) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"piece a", "pdf").await;
    let (_, b) =
        pieces::seed_document(&pool, seeded.company_id, docs.path(), b"piece b", "pdf").await;
    let (dossier, saine) = if a < b { (a, b) } else { (b, a) };
    std::fs::remove_file(docs.path().join(&dossier)).unwrap();
    std::fs::create_dir(docs.path().join(&dossier)).unwrap();
    let app = spawn_app_with(pool.clone(), config_docs(docs.path(), backup.path())).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    let (manifest, noms, fichiers) = ouvrir(&resp.bytes().await.unwrap());
    assert_eq!(
        non_inclus(&manifest),
        vec![(dossier.clone(), "unreadable".to_string())]
    );
    assert!(!noms.contains(&format!("files/{dossier}")), "{noms:?}");
    assert_eq!(fichiers.keys().cloned().collect::<Vec<_>>(), vec![saine]);
}

/// AC12 — l'en-tête `X-Kesh-Backup-Exceeds-Import-Limit` est posé quand
/// l'archive dépasse le plafond d'import (ici 1 Mio), absent sinon.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_previent_si_l_archive_depasse_le_plafond_d_import(pool: MySqlPool) {
    let docs = pieces::dossier();
    let backup = pieces::dossier();
    let (seeded, _) = pieces::seed_company_with_invoice(&pool).await;
    // 1,1 Mio de contenu peu compressible (pièce stockée sans compression).
    let gros: Vec<u8> = (0..1_150_000u32)
        .map(|i| (i.wrapping_mul(2_654_435_761) >> 13) as u8)
        .collect();
    pieces::seed_document(&pool, seeded.company_id, docs.path(), &gros, "pdf").await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let mut config = config_docs(docs.path(), backup.path());
    config.admin_import_max_mib = 1;
    let app = spawn_app_with(pool.clone(), config).await;
    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        entete(&resp, "x-kesh-backup-exceeds-import-limit").as_deref(),
        Some("1")
    );

    let mut config = config_docs(docs.path(), backup.path());
    config.admin_import_max_mib = 2;
    let app = spawn_app_with(pool.clone(), config).await;
    let resp = get_export(&app, &jwt).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(entete(&resp, "x-kesh-backup-exceeds-import-limit"), None);
}

/// AC4 (M-a19) — l'archive déborde **dans `KESH_ADMIN_BACKUP_DIR`**, et non
/// dans `/tmp` : ce dossier, propre au test et rendu non inscriptible, fait
/// échouer un export qui doit déborder (seuil 0) et laisse passer un export
/// qui tient en mémoire. Le temporaire étant anonyme, c'est l'effet qui se
/// mesure.
#[cfg(unix)]
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_deborde_dans_le_dossier_de_sauvegarde(pool: MySqlPool) {
    use std::os::unix::fs::PermissionsExt;
    let docs = pieces::dossier();
    let racine = pieces::dossier();
    let backup = racine.path().join("sauvegardes");
    std::fs::create_dir(&backup).unwrap();
    assert_ne!(backup, std::env::temp_dir());
    std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(0o500)).unwrap();
    exiger_un_refus_de_droits(&backup);
    let ctx = seed(&pool, "Acme", Role::Admin).await;

    let mut config = config_docs(docs.path(), &backup);
    config.admin_export_inmem_mib = 0;
    let app = spawn_app_with(pool.clone(), config).await;
    let resp = get_export(&app, &ctx.jwt).await;
    assert_eq!(resp.status(), 500, "débordement impossible → 500");

    let mut config = config_docs(docs.path(), &backup);
    config.admin_export_inmem_mib = 50;
    let app = spawn_app_with(pool.clone(), config).await;
    let resp = get_export(&app, &ctx.jwt).await;
    std::fs::set_permissions(&backup, std::fs::Permissions::from_mode(0o700)).unwrap();
    assert_eq!(resp.status(), 200, "en mémoire : rien à écrire");
}

/// AC4 — un dossier de sauvegarde absent est créé par l'export.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_cree_le_dossier_de_sauvegarde_absent(pool: MySqlPool) {
    let docs = pieces::dossier();
    let racine = pieces::dossier();
    let backup = racine.path().join("absent").join("sous-dossier");
    let ctx = seed(&pool, "Acme", Role::Admin).await;
    let mut config = config_docs(docs.path(), &backup);
    config.admin_export_inmem_mib = 0;
    let app = spawn_app_with(pool.clone(), config).await;
    let resp = get_export(&app, &ctx.jwt).await;
    assert_eq!(resp.status(), 200);
    let bytes = resp.bytes().await.unwrap();
    assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    assert!(backup.is_dir(), "dossier de sauvegarde créé");
}
