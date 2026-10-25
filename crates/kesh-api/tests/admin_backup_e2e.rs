//! Tests E2E HTTP Story 17-3e — Round-trip d'intégration export ↔ import
//! complet d'installation (`.keshbackup`).
//!
//! **Périmètre (delta, pas duplication)** : ce fichier couvre le round-trip
//! **exhaustif sur données riches multi-tables** (équivalence per-table après
//! export→remplacement→import, login admin source, intégrité FK, audit
//! préservé). Les cas de **refus** (409 version / 400 SHA tamper / 400 format /
//! 400 schema-mismatch) et de **rollback transactionnel** sont déjà couverts par
//! `admin_full_import_e2e.rs` (Story 17-3c) et ne sont **pas ré-implémentés** ici.
//! La structure ZIP / manifest / SHA / streaming est couverte par
//! `admin_full_export_e2e.rs` (Story 17-3a).
//!
//! Le test Playwright double-instance Docker (use-case migration cross-instance)
//! est une **dette v0.3 documentée** — trop lourd pour le MVP ; ce test
//! d'intégration Rust couvre l'équivalence fonctionnelle de bout en bout.
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::backup::TABLES_TO_TRUNCATE;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::{
    Language, McpScope, McpScopes, McpServerSettingsUpdate, NewCompany, NewMcpKey, OrgType,
    SendValidation,
};
use kesh_db::repositories::{companies, mcp_keys, mcp_server_settings};
use kesh_db::test_fixtures::seed_accounting_company;
use reqwest::multipart;
use serde_json::Value;
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

/// `COUNT(*)` par table applicative — **toutes** celles de `TABLES_TO_TRUNCATE`.
///
/// ⚠️ Ce commentaire annonçait « les 23 » alors que la liste en portait **38**.
/// Aucun test n'assertant ce nombre, la déclaration a dérivé de quinze sans que
/// rien ne rougisse. Elle ne porte plus de chiffre : la boucle parcourt la liste,
/// et c'est la liste qui fait foi. *(Relevé à la Story 25-2-c, qui y ajoute une
/// 39ᵉ table.)*
async fn table_counts(pool: &MySqlPool) -> BTreeMap<&'static str, i64> {
    let mut counts = BTreeMap::new();
    for &table in TABLES_TO_TRUNCATE {
        let c: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM `{table}`"))
            .fetch_one(pool)
            .await
            .unwrap_or_else(|e| panic!("count {table}: {e}"));
        counts.insert(table, c);
    }
    counts
}

/// Story 32-5a (AC-a7) — peuple les deux tables du serveur MCP : une clé
/// active (dispensée, deux portées, dernière utilisation à la milliseconde),
/// une clé révoquée, et des réglages **serveur activé** aux trois listes non
/// vides. Écrit par les dépôts, puis horodaté à la main (`last_used_at`,
/// `revoked_at`) pour éprouver la milliseconde.
async fn seed_mcp(pool: &MySqlPool, company_id: i64, user_id: i64) {
    let mut tx = pool.begin().await.unwrap();
    let active = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id,
            created_by_user_id: user_id,
            name: "Agent comptable".into(),
            key_hash: "a".repeat(64),
            scopes: McpScopes::new([McpScope::DocumentsRead, McpScope::BooksRead]).unwrap(),
            send_validation: SendValidation::Exempt,
            expires_at: Some(
                chrono::NaiveDate::from_ymd_opt(2999, 1, 1)
                    .unwrap()
                    .and_hms_milli_opt(12, 0, 0, 250)
                    .unwrap(),
            ),
        },
    )
    .await
    .unwrap();
    let revoked = mcp_keys::create_in_tx(
        &mut tx,
        NewMcpKey {
            company_id,
            created_by_user_id: user_id,
            name: "Ancienne clé".into(),
            key_hash: "b".repeat(64),
            scopes: McpScopes::new([McpScope::PromptsWrite]).unwrap(),
            send_validation: SendValidation::Required,
            expires_at: None,
        },
    )
    .await
    .unwrap();
    mcp_server_settings::save_in_tx(
        &mut tx,
        company_id,
        0,
        &McpServerSettingsUpdate {
            enabled: true,
            address_filter_enabled: false,
            allowed_networks: vec!["10.0.0.0/8".into(), "fd00::/8".into()],
            trusted_proxies: vec!["192.168.1.1/32".into()],
            allowed_origins: vec!["http://nas.local:3000".into()],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    sqlx::query("UPDATE mcp_keys SET last_used_at = '2026-10-20 10:11:12.345' WHERE id = ?")
        .bind(active.id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE mcp_keys SET revoked_at = '2026-10-21 08:09:10.678', version = version + 1 \
         WHERE id = ?",
    )
    .bind(revoked.id)
    .execute(pool)
    .await
    .unwrap();
}

/// Instantané colonne par colonne d'une table, chaque cellule rendue en texte
/// par MariaDB (`CAST(… AS CHAR)` : les `DATETIME(3)` gardent leurs
/// millisecondes, les `JSON` leur texte exact).
async fn column_snapshot(
    pool: &MySqlPool,
    table: &str,
    columns: &[&str],
    order_by: &str,
) -> Vec<Vec<Option<String>>> {
    use sqlx::Row;
    let select = columns
        .iter()
        .map(|c| format!("CAST(`{c}` AS CHAR)"))
        .collect::<Vec<_>>()
        .join(", ");
    sqlx::query(&format!(
        "SELECT {select} FROM `{table}` ORDER BY {order_by}"
    ))
    .fetch_all(pool)
    .await
    .unwrap_or_else(|e| panic!("instantané {table}: {e}"))
    .iter()
    .map(|row| {
        (0..columns.len())
            .map(|i| row.get::<Option<String>, _>(i))
            .collect()
    })
    .collect()
}

const MCP_KEYS_COLUMNS: [&str; 13] = [
    "id",
    "company_id",
    "created_by_user_id",
    "name",
    "key_hash",
    "scopes",
    "send_validation",
    "expires_at",
    "last_used_at",
    "revoked_at",
    "version",
    "created_at",
    "updated_at",
];

const MCP_SERVER_SETTINGS_COLUMNS: [&str; 8] = [
    "company_id",
    "enabled",
    "address_filter_enabled",
    "allowed_networks",
    "trusted_proxies",
    "allowed_origins",
    "version",
    "updated_at",
];

/// Export d'installation par HTTP ; rend les octets du `.keshbackup`.
async fn export_backup(app: &TestApp, jwt: &str) -> Vec<u8> {
    let resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "export → 200");
    resp.bytes().await.unwrap().to_vec()
}

/// Import d'installation par HTTP ; exige un 200.
async fn import_backup(app: &TestApp, jwt: &str, bytes: Vec<u8>) {
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    let resp = app
        .client
        .post(app.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {jwt}"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "import → 200");
}

// ============================================================
// Story 32-5a (AC-a7) — les tables du serveur MCP font l'aller-retour
// ============================================================

/// Les deux lignes de chaque table MCP reviennent **identiques, colonne par
/// colonne** — portées, politique, `last_used_at` et `revoked_at` à la
/// milliseconde, listes JSON à l'identique — et un serveur **activé** dans la
/// sauvegarde l'est après l'import (hypothèse Q7, C-32-5-11). Entre l'export
/// et l'import, tout est altéré : clé active supprimée, réglages vidés et
/// désactivés ; l'import doit tout rendre. Tue la mutation Ma6 (une colonne
/// JSON rebindée mal à l'import).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mcp_tables_roundtrip_column_by_column_and_enabled_server_stays_enabled(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    seed_mcp(&pool, seeded.company_id, seeded.admin_user_id).await;
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let keys_before = column_snapshot(&pool, "mcp_keys", &MCP_KEYS_COLUMNS, "id").await;
    let settings_before = column_snapshot(
        &pool,
        "mcp_server_settings",
        &MCP_SERVER_SETTINGS_COLUMNS,
        "company_id",
    )
    .await;
    assert_eq!(keys_before.len(), 2, "deux clés au montage");
    assert_eq!(settings_before.len(), 1, "une ligne de réglages au montage");
    // Le montage porte bien ce que le test prétend éprouver (sans quoi la
    // comparaison passerait à vide sur ces valeurs).
    let active = &keys_before[0];
    assert_eq!(
        active[5].as_deref(),
        Some(r#"["books:read","documents:read"]"#),
        "portées stockées sous forme canonique"
    );
    assert_eq!(active[6].as_deref(), Some("exempt"));
    assert_eq!(active[8].as_deref(), Some("2026-10-20 10:11:12.345"));
    assert_eq!(
        keys_before[1][9].as_deref(),
        Some("2026-10-21 08:09:10.678")
    );
    assert_eq!(
        settings_before[0][1].as_deref(),
        Some("1"),
        "serveur activé"
    );
    assert!(
        settings_before[0][3]
            .as_deref()
            .unwrap()
            .contains("fd00::/8")
    );

    let backup = export_backup(&app, &jwt).await;

    // Altération entre l'export et l'import.
    sqlx::query("DELETE FROM mcp_keys WHERE key_hash = ?")
        .bind("a".repeat(64))
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "UPDATE mcp_server_settings SET enabled = FALSE, allowed_networks = '[]', \
         trusted_proxies = '[]', allowed_origins = '[]', version = version + 1",
    )
    .execute(&pool)
    .await
    .unwrap();

    import_backup(&app, &jwt, backup).await;

    let keys_after = column_snapshot(&pool, "mcp_keys", &MCP_KEYS_COLUMNS, "id").await;
    let settings_after = column_snapshot(
        &pool,
        "mcp_server_settings",
        &MCP_SERVER_SETTINGS_COLUMNS,
        "company_id",
    )
    .await;
    assert_eq!(keys_after, keys_before, "mcp_keys : colonne par colonne");
    assert_eq!(
        settings_after, settings_before,
        "mcp_server_settings : colonne par colonne"
    );

    // Et par le dépôt : le serveur activé l'est encore (Q7), la clé active se
    // retrouve à l'authentification avec ses portées et sa politique.
    let settings = mcp_server_settings::get(&pool, seeded.company_id)
        .await
        .unwrap()
        .expect("réglages restaurés");
    assert!(
        settings.enabled,
        "un serveur activé dans la sauvegarde l'est après l'import"
    );
    let auth = mcp_keys::find_active_auth_by_key_hash(&pool, &"a".repeat(64))
        .await
        .unwrap()
        .expect("clé active restaurée");
    assert_eq!(
        auth.scopes.iter().collect::<Vec<_>>(),
        vec![McpScope::BooksRead, McpScope::DocumentsRead]
    );
    assert_eq!(auth.send_validation, SendValidation::Exempt);
    assert!(
        mcp_keys::find_active_auth_by_key_hash(&pool, &"b".repeat(64))
            .await
            .unwrap()
            .is_none(),
        "la clé révoquée avant l'export reste révoquée"
    );
}

// ============================================================
// AC22 — round-trip exhaustif sur données riches
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn full_roundtrip_rich_dataset_preserves_all_tables(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;

    // 1. Seed riche multi-tables (companies/users/fiscal_years/accounts/
    //    company_invoice_settings/vat_rates + contacts/products).
    let seeded = seed_accounting_company(&pool)
        .await
        .expect("seed accounting");
    kesh_db::test_fixtures::seed_contact_and_product(&pool, seeded.company_id)
        .await
        .expect("seed contact+product");
    // Story 32-5a (AC-a7) : les deux tables du serveur MCP, peuplées, pour que
    // l'équivalence ne passe pas à vide sur elles.
    seed_mcp(&pool, seeded.company_id, seeded.admin_user_id).await;

    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    // 2. Baseline : COUNT(*) par table AVANT export.
    let baseline = table_counts(&pool).await;
    // Sanity : plusieurs tables non vides (le round-trip est significatif).
    let populated = baseline.values().filter(|&&c| c > 0).count();
    assert!(
        populated >= 6,
        "le seed riche doit peupler ≥ 6 tables, got {populated} : {baseline:?}"
    );
    // Équivalence NON-TRIVIALE (review P1) : les 10 tables peuplées par le seed
    // (8, puis les deux du serveur MCP à la Story 32-5a)
    // doivent avoir un baseline > 0, sinon `after == baseline` serait vacuement
    // `0 == 0`. Les autres tables (journal/bank/invoices) restent vides — leur
    // sérialisation par type est couverte par les tests unitaires de
    // `kesh-db/src/backup.rs` (peupler les 23 tables = enhancement v0.3).
    for t in [
        "companies",
        "users",
        "fiscal_years",
        "accounts",
        "company_invoice_settings",
        "vat_rates",
        "contacts",
        "products",
        "mcp_keys",
        "mcp_server_settings",
    ] {
        assert!(
            baseline[t] > 0,
            "le seed doit peupler '{t}' pour une équivalence non-triviale (baseline={})",
            baseline[t]
        );
    }
    // Fidélité de VALEUR (pas seulement de count, review P1) : snapshot des
    // DECIMAL `vat_rates.rate` avant export pour vérifier le round-trip EXACT
    // (anti-arrondi binaire — la régression classique du bind f64).
    let vat_before: Vec<String> =
        sqlx::query_scalar("SELECT CAST(rate AS CHAR) FROM vat_rates ORDER BY rate DESC, label")
            .fetch_all(&pool)
            .await
            .unwrap();

    // 3. Export via GET /admin/full-export (JWT admin seedé).
    let export_resp = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(export_resp.status(), 200, "export → 200");
    let backup_bytes = export_resp.bytes().await.unwrap().to_vec();
    assert_eq!(&backup_bytes[0..4], &[0x50, 0x4b, 0x03, 0x04], "ZIP valide");

    // 4. Mutation entre export et import : company « Ghost » (doit disparaître
    //    après le restore — preuve du remplacement).
    let ghost_id = companies::create(
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
    let ghost_present: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies WHERE id = ?")
        .bind(ghost_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(ghost_present, 1, "Ghost présente avant import");

    // 5. Import via POST /admin/full-import (multipart, JWT admin).
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(backup_bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    let import_resp = app
        .client
        .post(app.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {jwt}"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(import_resp.status(), 200, "import → 200");
    let body: Value = import_resp.json().await.unwrap();
    assert_eq!(body["sessionInvalidated"], true);
    assert_eq!(body["backupCreated"], true);

    // 6. Équivalence per-table : chaque COUNT == baseline (Ghost supprimée,
    //    tout restauré exactement, fidélité de sérialisation prouvée).
    let after = table_counts(&pool).await;
    for &table in TABLES_TO_TRUNCATE {
        // `audit_log` et `onboarding_state` ont des invariants DC dédiés
        // (vérifiés juste après) → exclues de l'équivalence stricte.
        if table == "audit_log" || table == "onboarding_state" {
            continue;
        }
        assert_eq!(
            after[table], baseline[table],
            "table '{table}' : {} après import != {} baseline (équivalence stricte)",
            after[table], baseline[table]
        );
    }
    // ⛔ `audit_log` : **FUSIONNÉE, non remplacée** — Story 25-1a (#376).
    //
    // L'ancienne assertion était `baseline + 1` : la table était effacée, celle du
    // backup restaurée, puis `admin.full_import` ajoutée. Elle rend désormais **2**,
    // et l'écart dit exactement ce que la story a changé : la trace de l'EXPORT —
    // `admin.full_export`, émise APRÈS que `baseline` a été relevé — **survit
    // maintenant à l'import**. Auparavant, importer effaçait la preuve qu'on avait
    // exporté.
    //
    // ⚠️ On assert les ACTIONS et non un compte : un compte se périme dès qu'une
    // opération tracée s'ajoute au scénario, et l'invariant à tenir n'est pas
    // « combien » mais « rien n'a disparu ».
    let actions: Vec<String> = sqlx::query_scalar("SELECT action FROM audit_log ORDER BY id")
        .fetch_all(&pool)
        .await
        .expect("actions audit après import");
    assert!(
        actions.iter().any(|a| a == "admin.full_export"),
        "la trace de l'export doit SURVIVRE à l'import — c'est le blanchiment que \
         la Story 25-1a ferme ; got {actions:?}"
    );
    assert!(
        actions.iter().any(|a| a == "admin.full_import"),
        "l'import doit se tracer lui-même ; got {actions:?}"
    );
    assert!(
        after["audit_log"] > baseline["audit_log"],
        "la piste ne DIMINUE jamais : {} après import, {} au baseline",
        after["audit_log"],
        baseline["audit_log"]
    );
    // `onboarding_state` (DC11) : exclue du restore puis **forcée à l'état
    // terminé** (`step_completed == 8`, valeur post-finalize) car le dataset
    // restauré est éligible (company non-stub + admin). La row est créée si
    // absente (seed = 0 row). On assert la valeur EXACTE (review P1 : `>= 7`
    // laisserait passer une régression mettant 7 = « finalize en cours »).
    let onb_step: Option<i32> =
        sqlx::query_scalar("SELECT step_completed FROM onboarding_state LIMIT 1")
            .fetch_optional(&pool)
            .await
            .unwrap();
    assert_eq!(
        onb_step,
        Some(8),
        "onboarding_state doit être forcée done (step_completed == 8, DC11), got {onb_step:?}"
    );

    // Fidélité DECIMAL end-to-end (review P1) : les taux TVA round-trippent
    // EXACTEMENT (mêmes strings « 8.10 »/« 0.00 » qu'avant export).
    let vat_after: Vec<String> =
        sqlx::query_scalar("SELECT CAST(rate AS CHAR) FROM vat_rates ORDER BY rate DESC, label")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(
        vat_after, vat_before,
        "round-trip DECIMAL vat_rates.rate doit être exact (anti-arrondi)"
    );
    let ghost_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies WHERE id = ?")
        .bind(ghost_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(
        ghost_after, 0,
        "Ghost doit avoir été supprimée par le restore"
    );
}

// ============================================================
// AC22 — login admin source possible après restore
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn full_roundtrip_source_admin_can_login_after_import(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let export = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(export.status(), 200, "export → 200");
    let backup_bytes = export.bytes().await.unwrap().to_vec();

    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(backup_bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    let import_resp = app
        .client
        .post(app.url("/api/v1/admin/full-import"))
        .header("Authorization", format!("Bearer {jwt}"))
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(import_resp.status(), 200);

    // Login avec les identifiants de l'admin SOURCE (admin/admin123, cf.
    // test_fixtures::ADMIN_PASSWORD_HASH) → prouve la fidélité de
    // users.password_hash + la restauration des users + FK users→companies.
    let login = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&serde_json::json!({ "username": "admin", "password": "admin123" }))
        .send()
        .await
        .unwrap();
    assert_eq!(
        login.status(),
        200,
        "login admin source (admin/admin123) doit réussir après restore"
    );
}

// ============================================================
// AC22 — intégrité FK + audit préservé / entrée import présente
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn full_roundtrip_fk_integrity_and_audit_after_import(pool: MySqlPool) {
    let app = spawn_app(pool.clone()).await;
    let seeded = seed_accounting_company(&pool).await.expect("seed");
    let jwt = forge_jwt(seeded.admin_user_id, "Admin", seeded.company_id);

    let audit_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();

    let export = app
        .client
        .get(app.url("/api/v1/admin/full-export"))
        .header("Authorization", format!("Bearer {jwt}"))
        .send()
        .await
        .unwrap();
    assert_eq!(export.status(), 200, "export → 200");
    let backup_bytes = export.bytes().await.unwrap().to_vec();
    let form = multipart::Form::new().part(
        "file",
        multipart::Part::bytes(backup_bytes)
            .file_name("installation.keshbackup")
            .mime_str("application/octet-stream")
            .unwrap(),
    );
    assert_eq!(
        app.client
            .post(app.url("/api/v1/admin/full-import"))
            .header("Authorization", format!("Bearer {jwt}"))
            .multipart(form)
            .send()
            .await
            .unwrap()
            .status(),
        200
    );

    // FK : company_invoice_settings.default_receivable_account_id pointe sur un
    // accounts.id réel restauré. On compare le COUNT joint au COUNT total de la
    // table → **TOUTES** les rows ont une FK valide (review P1 : le restore se
    // fait sous FOREIGN_KEY_CHECKS=0, donc une FK pendante pourrait commiter ;
    // `>= 1` la masquerait, `== total` la détecte).
    let settings_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM company_invoice_settings")
        .fetch_one(&pool)
        .await
        .unwrap();
    let settings_fk_ok: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM company_invoice_settings cis \
         INNER JOIN accounts a ON a.id = cis.default_receivable_account_id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(settings_total >= 1, "le seed crée company_invoice_settings");
    assert_eq!(
        settings_fk_ok, settings_total,
        "TOUTES les company_invoice_settings.default_receivable_account_id doivent pointer sur un account restauré (pas de FK pendante)"
    );

    // FK : fiscal_years → companies (toutes les rows, pas seulement ≥ 1).
    let fy_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM fiscal_years")
        .fetch_one(&pool)
        .await
        .unwrap();
    let fy_fk_ok: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM fiscal_years fy \
         INNER JOIN companies c ON c.id = fy.company_id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(fy_total >= 1, "le seed crée un fiscal_year");
    assert_eq!(
        fy_fk_ok, fy_total,
        "TOUS les fiscal_years.company_id doivent référencer une company restaurée"
    );

    // Audit : l'entrée admin.full_import (post-restore) est présente, et le
    // total a augmenté (historique source préservé + nouvelle entrée).
    let import_audit: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'admin.full_import' AND entity_type = 'installation'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        import_audit, 1,
        "une entrée audit admin.full_import attendue"
    );

    let audit_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM audit_log")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(
        audit_after > audit_before,
        "audit_log post-import ({audit_after}) doit inclure l'entrée import (baseline {audit_before})"
    );
}
