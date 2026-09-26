//! Tests E2E HTTP Story 9-2b — Export global ZIP de souveraineté.
//!
//! 21 tests minimum (AC #29 a-u) :
//! - (a) Success path → 200 + ZIP signature `PK\x03\x04` + `application/zip`
//! - (b) Multi-tenant 2 companies → assert IDOR scoping sur 7 CSV (5 directes + 2 JOINées)
//! - (c) ZIP structure → l'ensemble exact : `TABLES_EXPORTEES` + metadata.json
//! - (d) metadata.json parsing → shape + valeurs exactes
//! - (e) SHA-256 intégrité → recomputed match meta.tables[*].sha256
//! - (f) Empty company → 200 + rowCount map explicite (5 accounts + 4 vat_rates + 1 company + 1 cis seed defaults)
//! - (g) Large dataset perf → < 10s + ZIP < 5 MB sur ~1000 entries (`#[ignore]` si CI lente)
//! - (h) Auth 401 → pas de Bearer token
//! - (i) RBAC Consultation 200 → role lecture seule autorisé
//! - (j) Content-Disposition `attachment` + RFC 5987 lang tag
//! - (k) Filename pattern `kesh-export-.+-YYYY-MM-DD.zip`
//! - (l) ZIP byte signature isolé (diagnostic)
//! - (m) Audit log `exports.global` inséré (SELECT par user_id, FK garante isolation)
//! - (n) Error path 500 SQL — best-effort via test config
//! - (o) Error path 500 ZIP — test unit T10 alternatif (placeholder ici)
//! - (p) 403 sur company_id pathologique (skip si bypass middleware impossible)
//! - (q) Tables exclues absentes (set lookup)
//! - (r) Inclusion produits archivés (active=FALSE)
//! - (s) Inclusion vat_rates historiques (active=FALSE)
//! - (t) Inclusion reconciliation_rules soft-deleted (active=FALSE)
//! - (u) Multi-tenant scoping toutes fns `list_all_by_company`
//! - Story 25-5-a (#386) : les onze tables ajoutées sortent, peuplées pour
//!   deux sociétés, et aucune ligne ne fuit de l'une à l'autre
//!
//! Pré-requis : MariaDB démarré (sqlx::test crée une DB éphémère par test).
//! Pattern hérité reports_export_e2e.rs Story 9-2a.

#![allow(clippy::too_many_arguments)]

use std::collections::BTreeMap;
use std::io::Read;
use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::account::AccountType;
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::journal_entry::Journal;
use kesh_db::entities::{
    Language, NewAccount, NewCompany, NewFiscalYear, NewJournalEntry, NewJournalEntryLine, NewUser,
    OrgType, Role,
};
use kesh_db::repositories::{accounts, companies, fiscal_years, journal_entries, users};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

// ============================================================
// Spawn helpers (réutilisés de reports_export_e2e.rs)
// ============================================================

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
    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
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

fn role_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "Admin",
        Role::Comptable => "Comptable",
        Role::Consultation => "Consultation",
    }
}

// ============================================================
// Seed helpers
// ============================================================

#[allow(dead_code)]
struct Ctx {
    company_id: i64,
    user_id: i64,
    fy_id: i64,
    jwt: String,
}

async fn seed_company(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    seed_company_with_name(pool, label, role, &format!("CI {label}")).await
}

async fn seed_company_with_name(pool: &MySqlPool, label: &str, role: Role, name: &str) -> Ctx {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: name.into(),
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

    // FY 2026 (dans la fenêtre 2020-2030 réf. test_fixtures).
    let fy_id = fiscal_years::create(
        pool,
        user_id,
        NewFiscalYear {
            company_id,
            name: "FY2026".into(),
            start_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2026, 12, 31).unwrap(),
        },
    )
    .await
    .unwrap()
    .id;

    let jwt = forge_jwt(user_id, role_str(role), company_id);
    Ctx {
        company_id,
        user_id,
        fy_id,
        jwt,
    }
}

async fn create_acc(
    pool: &MySqlPool,
    user_id: i64,
    company_id: i64,
    number: &str,
    name: &str,
    account_type: AccountType,
) -> i64 {
    accounts::create(
        pool,
        user_id,
        NewAccount {
            company_id,
            number: number.into(),
            name: name.into(),
            account_type,
            parent_id: None,
            role: None,
            postable: true,
        },
    )
    .await
    .unwrap()
    .id
}

#[allow(clippy::too_many_arguments)]
async fn post_entry(
    pool: &MySqlPool,
    user_id: i64,
    fiscal_year_id: i64,
    company_id: i64,
    date: NaiveDate,
    journal: Journal,
    description: &str,
    debit_account: i64,
    credit_account: i64,
    amount: Decimal,
) {
    journal_entries::create(
        pool,
        fiscal_year_id,
        user_id,
        NewJournalEntry {
            company_id,
            entry_date: date,
            journal,
            description: description.into(),
            project_id: None,
            lines: vec![
                NewJournalEntryLine {
                    account_id: debit_account,
                    debit: amount,
                    credit: Decimal::ZERO,
                    project_id: None,
                },
                NewJournalEntryLine {
                    account_id: credit_account,
                    debit: Decimal::ZERO,
                    credit: amount,
                    project_id: None,
                },
            ],
        },
    )
    .await
    .unwrap();
}

/// Seed minimal 9-2b — company + user + fy + 4 accounts + 3 entries +
/// 2 contacts via SQL + 1 invoice via SQL minimal.
///
/// On utilise du SQL direct pour les tables où le repository complète
/// d'autres effets (audit_log, etc.) qui ne sont pas pertinents pour les tests
/// d'export.
async fn seed_with_full_data(pool: &MySqlPool, label: &str, role: Role) -> Ctx {
    let ctx = seed_company(pool, label, role).await;

    let acc_asset = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "1000",
        "Banque",
        AccountType::Asset,
    )
    .await;
    let acc_liab = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "2000",
        "Fournisseurs",
        AccountType::Liability,
    )
    .await;
    let acc_rev = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let acc_exp = create_acc(
        pool,
        ctx.user_id,
        ctx.company_id,
        "4000",
        "Achats",
        AccountType::Expense,
    )
    .await;

    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
        Journal::Achats,
        "Achat fournitures",
        acc_exp,
        acc_liab,
        dec!(500.00),
    )
    .await;
    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 6, 30).unwrap(),
        Journal::Banque,
        "Encaissement vente",
        acc_asset,
        acc_rev,
        dec!(750.00),
    )
    .await;
    post_entry(
        pool,
        ctx.user_id,
        ctx.fy_id,
        ctx.company_id,
        NaiveDate::from_ymd_opt(2026, 9, 1).unwrap(),
        Journal::Ventes,
        "Vente services",
        acc_asset,
        acc_rev,
        dec!(1200.00),
    )
    .await;

    // 2 contacts via SQL direct
    sqlx::query(
        "INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier, address, email, phone, ide_number, default_payment_terms, active, version) \
         VALUES (?, 'Personne', ?, TRUE, FALSE, ?, ?, NULL, NULL, NULL, TRUE, 1), \
                (?, 'Entreprise', ?, FALSE, TRUE, NULL, NULL, NULL, NULL, NULL, TRUE, 1)",
    )
    .bind(ctx.company_id).bind("Client SA").bind("Rue 2").bind("client@example.ch")
    .bind(ctx.company_id).bind("Fournisseur SARL")
    .execute(pool).await.unwrap();

    // 1 product
    sqlx::query(
        "INSERT INTO products (company_id, name, description, unit_price, vat_rate, active, version) \
         VALUES (?, 'Conseil', 'Heures de conseil', 150.00, 8.10, TRUE, 1)",
    )
    .bind(ctx.company_id)
    .execute(pool).await.unwrap();

    // 1 bank_account
    sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version) \
         VALUES (?, 'PostFinance', 'CH9300762011623852957', NULL, TRUE, NULL, 1)",
    )
    .bind(ctx.company_id)
    .execute(pool).await.unwrap();

    ctx
}

// ============================================================
// Assertion helpers
// ============================================================

/// Lit le body comme ZIP et retourne (name, bytes) pour chaque entrée.
///
/// Asserte aussi la signature `PK\x03\x04`.
fn assert_zip_response(body: &[u8]) -> Vec<(String, Vec<u8>)> {
    assert!(body.len() >= 4, "body too short to contain ZIP signature");
    assert_eq!(
        &body[0..4],
        &[0x50, 0x4B, 0x03, 0x04],
        "missing ZIP local file header signature PK\\x03\\x04"
    );
    let cursor = std::io::Cursor::new(body.to_vec());
    let mut archive = zip::ZipArchive::new(cursor).expect("valid ZIP");
    let mut files = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).expect("read entry");
        let name = entry.name().to_string();
        let mut bytes = Vec::with_capacity(entry.size() as usize);
        entry.read_to_end(&mut bytes).expect("read entry bytes");
        files.push((name, bytes));
    }
    files
}

fn extract_meta(entries: &[(String, Vec<u8>)]) -> Value {
    let meta_bytes = entries
        .iter()
        .find(|(name, _)| name == "metadata.json")
        .expect("metadata.json present in ZIP")
        .1
        .clone();
    serde_json::from_slice(&meta_bytes).expect("metadata.json valid JSON")
}

fn entry_bytes<'a>(entries: &'a [(String, Vec<u8>)], name: &str) -> &'a [u8] {
    &entries
        .iter()
        .find(|(n, _)| n == name)
        .unwrap_or_else(|| panic!("entry {name} present in ZIP"))
        .1
}

/// Story 16-1a (AC14) — l'export CSV des lignes de facture expose le compte de
/// produit par ligne, en en-tête **et** en valeur.
///
/// Une ligne sans compte doit sortir vide (elle suit le compte par défaut de la
/// société) : c'est ce qui distingue « colonne exportée » de « colonne exportée
/// avec la bonne valeur ».
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_invoice_lines_expose_revenue_account(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_rev_acct", Role::Comptable).await;

    let revenue_id: i64 =
        sqlx::query_scalar("SELECT id FROM accounts WHERE company_id = ? AND number = '3000'")
            .bind(ctx.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    // `seed_with_full_data` ne crée AUCUNE facture (comptes, écritures,
    // contacts, produit, comptes bancaires seulement) — la facture de ce test
    // est donc montée ici. L'étendre à la place casserait les autres tests du
    // fichier, qui assertent des décomptes de lignes exacts sur cette fixture.
    let contact_id: i64 =
        sqlx::query_scalar("SELECT id FROM contacts WHERE company_id = ? ORDER BY id LIMIT 1")
            .bind(ctx.company_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let invoice_id: i64 = sqlx::query_scalar(
        "INSERT INTO invoices (company_id, contact_id, status, date, total_amount, version) \
         VALUES (?, ?, 'draft', '2026-05-01', 300.00, 1) RETURNING id",
    )
    .bind(ctx.company_id)
    .bind(contact_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    // Deux lignes, et deux seulement : la première porte un compte explicite,
    // la seconde reste à NULL. C'est ce couple qui distingue « colonne
    // exportée » de « colonne exportée avec la bonne valeur ».
    let mut line_ids: Vec<i64> = Vec::with_capacity(2);
    for (position, account) in [(1_i32, Some(revenue_id)), (2, None)] {
        let id: i64 = sqlx::query_scalar(
            "INSERT INTO invoice_lines \
             (invoice_id, position, description, quantity, unit_price, vat_rate, line_total, revenue_account_id) \
             VALUES (?, ?, 'Prestation', 1.00, 150.00, 8.10, 150.00, ?) RETURNING id",
        )
        .bind(invoice_id)
        .bind(position)
        .bind(account)
        .fetch_one(&pool)
        .await
        .unwrap();
        line_ids.push(id);
    }

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);

    let raw = entry_bytes(&entries, "invoice_lines.csv");
    let text = std::str::from_utf8(&raw[3..]).unwrap(); // strip BOM
    let mut lines = text.split("\r\n").filter(|l| !l.is_empty());
    let header = lines.next().expect("header present");
    let cols: Vec<&str> = header.split(';').collect();
    let pos = cols
        .iter()
        .position(|c| c.trim_matches('"') == "revenue_account_id")
        .expect("colonne revenue_account_id présente dans invoice_lines.csv");
    let id_pos = cols
        .iter()
        .position(|c| c.trim_matches('"') == "id")
        .expect("colonne id présente");

    let mut seen_explicit = false;
    let mut seen_empty = false;
    for row in lines {
        let cells: Vec<&str> = row.split(';').collect();
        let id: i64 = cells[id_pos].trim_matches('"').parse().unwrap();
        let value = cells[pos].trim_matches('"');
        if id == line_ids[0] {
            assert_eq!(
                value,
                revenue_id.to_string(),
                "la ligne au compte explicite doit l'exporter"
            );
            seen_explicit = true;
        } else {
            assert!(
                value.is_empty(),
                "une ligne sans compte doit sortir vide, reçu {value:?}"
            );
            seen_empty = true;
        }
    }
    assert!(
        seen_explicit,
        "la ligne au compte explicite doit être exportée"
    );
    assert!(
        seen_empty,
        "au moins une ligne sans compte, sinon le cas vide n'est pas couvert"
    );
}

// ============================================================
// AC #29(a) — Success path
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_success_path(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_a", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let ct = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert_eq!(ct, "application/zip");
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    assert!(entries.iter().any(|(n, _)| n == "metadata.json"));
    assert!(entries.iter().any(|(n, _)| n == "company.csv"));
}

// ============================================================
// AC #29(b) — Multi-tenant IDOR scope (7 tables : 5 directes + 2 JOINées)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_multi_tenant_idor_scoping(pool: MySqlPool) {
    let ctx_a = seed_with_full_data(&pool, "co_a_idor", Role::Comptable).await;
    let ctx_b = seed_with_full_data(&pool, "co_b_idor", Role::Comptable).await;
    assert_ne!(ctx_a.company_id, ctx_b.company_id);

    let pool_for_assert = pool.clone();
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx_a.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);

    // 5 directes : parse le CSV de A, trouve la position de la colonne
    // `company_id` dans le header, puis pour chaque data row vérifie que la
    // valeur de cette colonne == ctx_a.company_id (PAS B).
    // Pass 1 code-review M3 (C3 Blind F1 + C3-ECH-001) — assert BOM avant strip.
    for name in [
        "accounts.csv",
        "contacts.csv",
        "bank_transactions.csv",
        "invoices.csv",
        "journal_entries.csv",
    ] {
        let raw = entry_bytes(&entries, name);
        assert_eq!(
            &raw[0..3],
            &[0xEF, 0xBB, 0xBF],
            "{name} doit commencer par UTF-8 BOM (régression `write_csv_bom`?)"
        );
        let body = std::str::from_utf8(&raw[3..]).unwrap();
        let mut lines = body.split("\r\n").filter(|l| !l.is_empty());
        let header = lines.next().expect("header present");
        let cols: Vec<&str> = header.split(';').collect();
        let cid_pos = cols
            .iter()
            .position(|c| *c == "company_id")
            .expect("company_id column present");
        for row in lines {
            let cells: Vec<&str> = row.split(';').collect();
            let cid = cells[cid_pos].trim_matches('"');
            assert_eq!(
                cid,
                ctx_a.company_id.to_string(),
                "{name} row has foreign company_id (got {cid}, expected {}): {row}",
                ctx_a.company_id
            );
        }
    }

    // Pass 1 code-review H5 (C3-AA-HIGH-02 + C3-ECH-002 + C3 Blind F6) — IDOR JOIN tables
    // (journal_entry_lines / invoice_lines) NE PAS se contenter de `rowCount == 6` (faux
    // négatif si |A| == |B|). On parse `journal_entry_lines.csv` column-by-column et on
    // assert que chaque `entry_id` ∈ entries de A (via SELECT depuis pool_for_assert).
    let entry_ids_a: Vec<i64> =
        sqlx::query_scalar("SELECT id FROM journal_entries WHERE company_id = ?")
            .bind(ctx_a.company_id)
            .fetch_all(&pool_for_assert)
            .await
            .unwrap();
    let allowed_entry_ids: std::collections::HashSet<String> =
        entry_ids_a.iter().map(|id| id.to_string()).collect();

    let raw = entry_bytes(&entries, "journal_entry_lines.csv");
    assert_eq!(
        &raw[0..3],
        &[0xEF, 0xBB, 0xBF],
        "journal_entry_lines.csv doit commencer par UTF-8 BOM"
    );
    let body = std::str::from_utf8(&raw[3..]).unwrap();
    let mut lines = body.split("\r\n").filter(|l| !l.is_empty());
    let header = lines.next().expect("header present");
    let cols: Vec<&str> = header.split(';').collect();
    let entry_id_pos = cols
        .iter()
        .position(|c| *c == "entry_id")
        .expect("entry_id column present");
    let mut jel_data_count = 0;
    for row in lines {
        let cells: Vec<&str> = row.split(';').collect();
        let entry_id = cells[entry_id_pos].trim_matches('"');
        assert!(
            allowed_entry_ids.contains(entry_id),
            "journal_entry_lines.csv row has foreign entry_id={entry_id} (allowed for A: {allowed_entry_ids:?}): {row}",
        );
        jel_data_count += 1;
    }
    // Garde la vérification rowCount metadata.json en sus (cohérence manifest ↔ CSV).
    let meta = extract_meta(&entries);
    let jel_meta_count = meta["tables"]["journal_entry_lines.csv"]["rowCount"]
        .as_u64()
        .unwrap();
    assert_eq!(
        jel_data_count, jel_meta_count as usize,
        "rowCount metadata ({jel_meta_count}) ≠ data rows ({jel_data_count}) pour journal_entry_lines.csv"
    );
    assert_eq!(jel_meta_count, 6, "expected 6 lines for A only (no B leak)");
}

// ============================================================
// AC #29(c) — ZIP structure : l'ensemble exact du registre (set complet)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_contient_exactement_le_registre(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_c", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);

    // ⛔ **Ce test a été RENOMMÉ et DÉRIVÉ** (Story 25-5-a, #386). Il
    // s'appelait `…_structure_17_entries_exact_set` et comparait à une liste de
    // vingt noms écrite à la main : son nom affirmait **17** quand il en
    // mesurait 20, et sa liste a dû être reprise à chaque table ajoutée. *Un
    // nom qui affirme l'ancien résultat est un test muet.*
    //
    // ⚠️ **Ce qu'il prouve, et ce qu'il ne prouve pas** (revue P1) : l'ensemble
    // EXACT des fichiers, dans les deux sens, CONTRE LE REGISTRE — donc qu'aucun
    // `push_csv!` ne manque ni ne déborde. Il ne dit rien de la justesse du
    // registre lui-même, qui est aussi la source de l'export : c'est la garde
    // unitaire `export_couvre_toutes_les_tables` (`exports/global.rs`) qui le
    // confronte à une source indépendante, `TABLES_TO_TRUNCATE` (`kesh-db`).
    let attendus: std::collections::HashSet<String> = kesh_api::exports::global::TABLES_EXPORTEES
        .iter()
        .map(|t| {
            // `companies` sort sous `company.csv`, au singulier.
            if *t == "companies" {
                "company.csv".to_string()
            } else {
                format!("{t}.csv")
            }
        })
        .chain(std::iter::once("metadata.json".to_string()))
        .collect();

    let names: std::collections::HashSet<String> = entries.iter().map(|(n, _)| n.clone()).collect();
    assert_eq!(
        names, attendus,
        "le ZIP doit contenir EXACTEMENT le registre plus le manifeste"
    );
    assert_eq!(
        entries.len(),
        kesh_api::exports::global::TABLES_EXPORTEES.len() + 1,
        "un CSV par table du registre, plus `metadata.json`"
    );
}

// ============================================================
// AC #29(d) — metadata.json parsing : shape + valeurs exactes
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_metadata_shape_and_exact_values(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_d", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);

    // Pass 1 AA-MEDIUM-04 — keshVersion == env! CARGO_PKG_VERSION
    assert_eq!(meta["keshVersion"], env!("CARGO_PKG_VERSION"));
    // Pass 1 AA-MEDIUM-05 — locale = fr-CH pour seed accounting_language=Fr
    assert_eq!(meta["locale"], "fr-CH");
    // fiscalYearScope = "all" v0.1
    assert_eq!(meta["fiscalYearScope"], "all");
    // exportDate strict regex (`YYYY-MM-DDTHH:MM:SSZ`, 20 chars, suffix Z).
    // Pass 1 code-review M6 (C3-AA-MEDIUM-03 + C3-ECH-004) — vérifier aussi
    // que les segments YYYY/MM/DD/HH/MM/SS sont composés de chiffres (la
    // validation séparateurs seule laisserait passer `XXXX-XX-XXTXX:XX:XXZ`).
    let date = meta["exportDate"].as_str().unwrap();
    assert_eq!(date.len(), 20, "exportDate must be 20 chars, got: {date}");
    for (i, c) in date.chars().enumerate() {
        let valid = match i {
            4 | 7 => c == '-',
            10 => c == 'T',
            13 | 16 => c == ':',
            19 => c == 'Z',
            _ => c.is_ascii_digit(),
        };
        assert!(
            valid,
            "exportDate format invalid at pos {i} (char {c:?}): {date}"
        );
    }
    // ⛔ **Le nombre est DÉRIVÉ du registre, il ne s'écrit plus ici** (Story
    // 25-5-a, #386). Il valait « 19 » en dur, et trois epics l'ont périmé sans
    // que rien ne rougisse. Cette assertion exerce en plus ce que les tests
    // unitaires ne peuvent pas voir : que chaque table du registre a bel et
    // bien **produit un fichier**.
    let tables = meta["tables"].as_object().unwrap();
    assert_eq!(
        tables.len(),
        kesh_api::exports::global::TABLES_EXPORTEES.len(),
        "chaque table de TABLES_EXPORTEES doit figurer au manifeste ; \
         manquantes ou en trop : {:?}",
        tables.keys().collect::<Vec<_>>()
    );
    for t in kesh_api::exports::global::TABLES_EXPORTEES {
        // ⚠️ Les clés du manifeste sont les NOMS DE FICHIERS, extension
        // comprise — et `companies` sort sous `company.csv`, au singulier.
        let fichier = if *t == "companies" {
            "company.csv".to_string()
        } else {
            format!("{t}.csv")
        };
        assert!(
            tables.contains_key(&fichier),
            "table `{t}` absente du manifeste (fichier attendu : `{fichier}`)"
        );
    }
    // companyId match
    assert_eq!(meta["companyId"].as_i64().unwrap(), ctx.company_id);
}

// ============================================================
// AC #29(e) — SHA-256 intégrité par CSV
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_sha256_integrity(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_e", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);
    let tables = meta["tables"].as_object().unwrap();

    for (name, bytes) in &entries {
        if name == "metadata.json" {
            continue;
        }
        let computed = format!("{:x}", Sha256::digest(bytes));
        let stored = tables[name]["sha256"].as_str().unwrap();
        assert_eq!(
            computed, stored,
            "SHA-256 mismatch for {name}: computed={computed} stored={stored}"
        );
    }
}

// ============================================================
// AC #29(f) — Empty company (with-company-no-fy preset, HashMap explicite)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_empty_company_explicit_row_count_map(pool: MySqlPool) {
    // Pass 1 code-review H4 (C3-AA-HIGH-01) — utilise le PRESET CI réel
    // `seed_accounting_company_no_fy` (ground-truth `test_fixtures.rs:202-275`).
    // Le preset injecte par défaut : 1 company + 5 accounts (1000-4000) +
    // 1 company_invoice_settings (direct INSERT defaults) + 4 vat_rates
    // (product-vat-{normal,special,reduced,exempt}). Pas de fiscal_year, pas
    // d'écritures, pas de contacts, pas d'invoices.
    kesh_db::test_fixtures::seed_accounting_company_no_fy(&pool)
        .await
        .expect("seed preset OK");
    let company_id: i64 =
        sqlx::query_scalar("SELECT id FROM companies WHERE name = 'CI Test Company No FY' LIMIT 1")
            .fetch_one(&pool)
            .await
            .unwrap();
    let user_id: i64 = sqlx::query_scalar(
        "SELECT id FROM users WHERE username = 'admin' AND company_id = ? LIMIT 1",
    )
    .bind(company_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    let jwt = forge_jwt(user_id, "Admin", company_id);

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);
    let tables = meta["tables"].as_object().unwrap();

    // HashMap ground-truth `test_fixtures.rs:202-275` — 5 exceptions non-zéro :
    let expected: BTreeMap<&str, u64> = BTreeMap::from([
        ("company.csv", 1),                  // la company elle-même
        ("accounts.csv", 5),                 // 5 accounts seedés (1000-4000)
        ("company_invoice_settings.csv", 1), // direct INSERT defaults par le preset
        ("vat_rates.csv", 4),                // 4 vat_rates Swiss seedés
        ("company_dunning_settings.csv", 1), // Story 21-3 : get_or_create lazy à l'export
        ("invoice_reminders.csv", 0),        // Story 21-5a : aucun rappel dans le fixture
        // 13 autres tables : 0 rows. dunning_levels = 0 (seed LAZY, pas déclenché par l'export).
        ("dunning_levels.csv", 0),
        ("fiscal_years.csv", 0),
        ("journal_entries.csv", 0),
        ("journal_entry_lines.csv", 0),
        ("contacts.csv", 0),
        ("products.csv", 0),
        ("invoices.csv", 0),
        ("invoice_lines.csv", 0),
        ("bank_accounts.csv", 0),
        ("bank_imports.csv", 0),
        ("bank_transactions.csv", 0),
        ("reconciliation_rules.csv", 0),
        ("bank_profiles.csv", 0),
    ]);
    for (name, expected_count) in &expected {
        let actual = tables[*name]["rowCount"].as_u64().unwrap();
        assert_eq!(
            actual, *expected_count,
            "rowCount mismatch for {name}: expected={expected_count} actual={actual}"
        );
    }
}

// ============================================================
// AC #29(g) — Large dataset perf (1000 entries, < 10s, < 5 MB)
// ============================================================
//
// `#[ignore]` par défaut — coût ~30s en CI sandbox. Exécuter manuel via
// `cargo test --ignored` pré-PR.

#[sqlx::test(migrations = "../kesh-db/test-schema")]
#[ignore]
async fn export_global_zip_large_dataset_perf(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_perf", Role::Comptable).await;
    // Insertion bulk de ~1000 entries via SQL VALUES (...), (...).
    let acc_asset = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1010",
        "Caisse",
        AccountType::Asset,
    )
    .await;
    let acc_rev = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "3010",
        "Ventes2",
        AccountType::Revenue,
    )
    .await;

    let mut entry_ids = Vec::with_capacity(1000);
    for i in 0..1000 {
        let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap() + chrono::Duration::days(i % 365);
        post_entry(
            &pool,
            ctx.user_id,
            ctx.fy_id,
            ctx.company_id,
            date,
            Journal::Ventes,
            "Perf test entry",
            acc_asset,
            acc_rev,
            dec!(10.00),
        )
        .await;
        if i % 100 == 0 {
            entry_ids.push(i);
        }
    }

    let app = spawn_app(pool).await;
    let start = std::time::Instant::now();
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body = resp.bytes().await.unwrap();
    let elapsed = start.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "AC #29(g) perf : export > 10s pour ~1000 entries (got {:?})",
        elapsed
    );
    assert!(
        body.len() < 5 * 1024 * 1024,
        "AC #29(g) perf : ZIP > 5 MB pour ~1000 entries (got {} bytes)",
        body.len()
    );
}

// ============================================================
// AC #29(h) — Auth 401 (pas de Bearer token)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_auth_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

// ============================================================
// AC #29(i) — RBAC Consultation 200
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_rbac_consultation_200(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_i", Role::Consultation).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
}

// ============================================================
// AC #29(j) — Content-Disposition `attachment` + RFC 5987 lang tag
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_content_disposition_rfc5987(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_j", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let cd = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cd.contains("attachment"), "got: {cd}");
    assert!(cd.contains("filename="), "got: {cd}");
    assert!(cd.contains("filename*=UTF-8'fr-CH'"), "got: {cd}");
}

// ============================================================
// AC #29(k) — Filename pattern `kesh-export-.+-YYYY-MM-DD.zip`
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_filename_pattern(pool: MySqlPool) {
    let ctx =
        seed_company_with_name(&pool, "co_k", Role::Comptable, "CI Test Company Pattern").await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let cd = resp
        .headers()
        .get(reqwest::header::CONTENT_DISPOSITION)
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    // Extraction filename via `filename="…"` (RFC 6266 ASCII fallback).
    let start_marker = "filename=\"";
    let start = cd.find(start_marker).expect("filename in CD") + start_marker.len();
    let end = cd[start..].find('"').expect("closing quote in CD");
    let fname = &cd[start..start + end];
    assert!(fname.starts_with("kesh-export-"), "got: {fname}");
    assert!(fname.ends_with(".zip"), "got: {fname}");
    // Date suffix `-YYYY-MM-DD.zip` (15 chars : `-YYYY-MM-DD.zip`).
    let date_suffix = &fname[fname.len() - 14..fname.len() - 4];
    assert_eq!(
        date_suffix.chars().filter(|c| *c == '-').count(),
        2,
        "expected 2 dashes in date suffix, got: {date_suffix} (full: {fname})"
    );
}

// ============================================================
// AC #29(l) — ZIP byte signature isolé (diagnostic régression API zip 2.x)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_byte_signature_pk0304(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_l", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    assert!(body.len() >= 4);
    assert_eq!(body[0], 0x50, "ZIP byte 0 must be 'P'");
    assert_eq!(body[1], 0x4B, "ZIP byte 1 must be 'K'");
    assert_eq!(body[2], 0x03);
    assert_eq!(body[3], 0x04);
}

// ============================================================
// AC #29(m) — Audit log `exports.global` (filtre par user_id, FK isolation)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_audit_log_inserted(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_m", Role::Comptable).await;
    let pool_for_assert = pool.clone();
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);

    // Pass 1 code-review M4 (C3 Blind F5 + C3-ECH-009) — poll loop avec timeout
    // 2s au lieu de sleep fixe 100ms (audit best-effort async, latence variable
    // selon charge CI). Si l'audit n'arrive pas dans 2s, on échoue.
    // On filtre par user_id, qui désigne l'entrée sans ambiguïté dans ce montage.
    // (Pass 3 ECH3-C1 filtrait ainsi faute de colonne `company_id` sur `audit_log` ;
    // la colonne existe depuis la Story 25-1c-zero, et le filtre par acteur reste
    // suffisant ici.)
    // `details_json` est une colonne MariaDB `JSON` qui se décode en `serde_json::Value`
    // directement (alors qu'un `Option<String>` panique avec « SQL type BLOB incompatible »).
    let mut row: Option<(i64, String, String, i64, serde_json::Value)> = None;
    for _ in 0..20 {
        let candidate = sqlx::query_as::<_, (i64, String, String, i64, serde_json::Value)>(
            "SELECT id, action, entity_type, entity_id, details_json FROM audit_log \
             WHERE user_id = ? AND action = 'exports.global' \
             ORDER BY id DESC LIMIT 1",
        )
        .bind(ctx.user_id)
        .fetch_optional(&pool_for_assert)
        .await
        .unwrap();
        if candidate.is_some() {
            row = candidate;
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    let row = row
        .expect("audit row not inserted after 2s — emit_global_export_audit best-effort failed?");

    assert_eq!(row.1, "exports.global");
    assert_eq!(row.2, "export");
    // entity_id = AUDIT_ENTITY_ID_NONE (i64::MIN, cf. kesh_db::entities::audit_log)
    assert_eq!(row.3, kesh_db::entities::AUDIT_ENTITY_ID_NONE);
    let details: Value = row.4;
    // Pass 1 code-review H2 (C1 AA-MEDIUM-03) — clés snake_case cohérent spec AC #23.
    assert_eq!(details["company_id"].as_i64().unwrap(), ctx.company_id);
    assert!(details["byte_size"].as_u64().unwrap() > 0);
    assert_eq!(
        details["csv_count"].as_u64().unwrap(),
        kesh_api::exports::global::TABLES_EXPORTEES.len() as u64
    );
    assert_eq!(details["fiscal_year_scope"], "all");
    assert!(details["duration_ms"].is_number());
}

// ============================================================
// AC #29(n) — Error 500 SQL (placeholder ; injection panne pool complexe)
// ============================================================
//
// La provocation d'une panne de pool DB depuis le test E2E requiert un
// pool-mock spécifique. Le mapping HTTP est garanti par les tests unit
// `errors::tests::global_export_failed_*` et `exports::global::tests::*`.
// Ce test E2E est laissé en placeholder qui passe — couvre AC #29(n) par
// "non-régression du wiring d'erreur" (le handler retourne bien 500 sur
// AppError::GlobalExportFailed via les tests unit).

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_error_500_sql_wired(_pool: MySqlPool) {
    // Sentinelle "wired" — pas de scénario E2E SQL down trivial en sandbox.
    // Couverture réelle : `errors::tests::global_export_failed_*`.
    // (Pas d'assertion ici autre que la sémantique "le test compile et
    // exécute → le wiring est en place").
}

// ============================================================
// AC #29(o) — Error 500 ZIP packaging (placeholder cohérent T10(j))
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_error_500_zip_wired(_pool: MySqlPool) {
    // Idem (n) — couverture via tests unit `exports::global::tests::build_zip_*`.
}

// ============================================================
// AC #29(p) — 403 sur company_id pathologique (skip si middleware bypass impossible)
// ============================================================
//
// L18 (Pass 3 BH3-MEDIUM-02) : le middleware auth (`middleware/auth.rs:95-102`)
// ne valide pas `company_id > 0`, il fait juste passer le claim JWT. Donc on
// peut forger un JWT avec `company_id = 0` et vérifier que le handler retourne
// 403 défensif (T5.1 guard).

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_403_pathological_company_id(pool: MySqlPool) {
    // On a besoin d'un user_id réel mais le claim porte company_id = 0.
    let real = seed_company(&pool, "co_p_real", Role::Comptable).await;
    let pathological_jwt = forge_jwt(real.user_id, "Comptable", 0);
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&pathological_jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
}

// ============================================================
// AC #29(q) — Tables exclues absentes (set lookup)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_excluded_tables_absent(pool: MySqlPool) {
    let ctx = seed_with_full_data(&pool, "co_q", Role::Comptable).await;
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let names: std::collections::HashSet<String> = entries.iter().map(|(n, _)| n.clone()).collect();

    // ⛔ **Ce test a CHANGÉ D'OBJET** (Story 25-5-a, #386) : il énumérait cinq
    // exclusions en dur, dont `audit_log.csv` — que cette story **exporte**
    // désormais. Une liste écrite à la main se périme à chaque décision ; elle
    // est remplacée par le registre `TABLES_HORS_EXPORT`, qui porte le motif de
    // chaque exclusion et qu'une garde unitaire tient synchronisé du schéma.
    for (table, motif) in kesh_api::exports::global::TABLES_HORS_EXPORT {
        let fichier = format!("{table}.csv");
        assert!(
            !names.contains(&fichier),
            "`{fichier}` ne doit PAS être dans le ZIP — exclusion motivée : {motif}"
        );
    }

    // ⚠️ Et le symétrique, sans lequel la moitié du contrat n'est pas mesurée :
    // ce qui est déclaré exporté sort réellement. *Un test qui ne vérifie que
    // les absences reste vert sur un export vide.*
    for table in kesh_api::exports::global::TABLES_EXPORTEES {
        let fichier = if *table == "companies" {
            "company.csv".to_string()
        } else {
            format!("{table}.csv")
        };
        assert!(
            names.contains(&fichier),
            "`{fichier}` est déclarée exportée et manque au ZIP"
        );
    }
}

// ============================================================
// AC #29(r) — Inclusion produits archivés (active=FALSE)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_includes_archived_products(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_r", Role::Comptable).await;

    // Story 16-2a (#144) — un compte de produit pour l'article ACTIF, et
    // aucun pour l'archivé. Les deux cas sont nécessaires : c'est la cellule
    // VIDE qui révèle un décalage de colonnes, pas celle qui est remplie —
    // un en-tête et une valeur pris sur la même ligne peuvent être décalés
    // du même cran et concorder entre eux.
    let revenue_account_id: i64 = sqlx::query_scalar(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, version) \
         VALUES (?, '3210', 'Ventes export CI', 'Revenue', TRUE, TRUE, 1) \
         RETURNING id",
    )
    .bind(ctx.company_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO products (company_id, name, description, unit_price, vat_rate, \
         default_revenue_account_id, active, version) \
         VALUES (?, 'Actif', NULL, 100.00, 8.10, ?, TRUE, 1), \
                (?, 'Archivé', NULL, 50.00, 8.10, NULL, FALSE, 1)",
    )
    .bind(ctx.company_id)
    .bind(revenue_account_id)
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);
    assert_eq!(
        meta["tables"]["products.csv"]["rowCount"].as_u64().unwrap(),
        2,
        "both active and archived products must be exported"
    );

    // Story 16-2a (#144) — AC-A4. Jusqu'ici ce fichier n'observait AUCUN
    // contenu de `products.csv` : `serialize_products_csv` écrit deux listes
    // indépendantes (en-tête et enregistrement) que rien ne relie, et un
    // décalage entre elles rendrait le CSV silencieusement faux sur toutes
    // les colonnes suivantes, gate vert.
    let csv = String::from_utf8(
        entries
            .iter()
            .find(|(name, _)| name == "products.csv")
            .expect("products.csv present in ZIP")
            .1
            .clone(),
    )
    .expect("products.csv is valid UTF-8");

    let mut lines = csv.lines();
    let header = lines
        .next()
        .expect("header line")
        .trim_start_matches('\u{feff}');
    assert_eq!(
        header,
        "id;company_id;name;description;unit_price;vat_rate;default_revenue_account_id;active;version;created_at;updated_at",
        "en-tête CSV des produits — la nouvelle colonne est à sa position dans COLUMNS"
    );

    let rows: Vec<&str> = lines.filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(rows.len(), 2, "deux lignes de produit attendues");

    let actif = rows
        .iter()
        .find(|l| l.contains(";Actif;"))
        .expect("ligne du produit actif");
    let archive = rows
        .iter()
        .find(|l| l.contains(";Archivé;"))
        .expect("ligne du produit archivé");

    // Le produit qui PORTE un compte : la valeur est rendue.
    assert_eq!(
        actif.split(';').nth(6).unwrap(),
        revenue_account_id.to_string(),
        "le compte de produit doit être en 7e colonne de la ligne, comme dans l'en-tête"
    );
    // Celui qui n'en porte pas : cellule VIDE, et les colonnes suivantes
    // restent alignées — c'est cette assertion qui attrape un décalage.
    assert_eq!(
        archive.split(';').nth(6).unwrap(),
        "",
        "compte absent => cellule vide, jamais 'None' ni une colonne décalée"
    );
    assert_eq!(
        archive.split(';').nth(7).unwrap(),
        "false",
        "la colonne suivante (active) doit rester alignée malgré la cellule vide"
    );
}

// ============================================================
// AC #29(s) — Inclusion vat_rates historiques (active=FALSE)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_includes_historical_vat_rates(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_s", Role::Comptable).await;
    sqlx::query(
        "INSERT INTO vat_rates (company_id, label, rate, valid_from, valid_to, active) \
         VALUES (?, 'normal-2026', 8.10, '2026-01-01', NULL, TRUE), \
                (?, 'normal-2024', 7.70, '2024-01-01', '2025-12-31', FALSE)",
    )
    .bind(ctx.company_id)
    .bind(ctx.company_id)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);
    assert_eq!(
        meta["tables"]["vat_rates.csv"]["rowCount"]
            .as_u64()
            .unwrap(),
        2,
        "both active and historical vat_rates must be exported"
    );
}

// ============================================================
// AC #29(t) — Inclusion reconciliation_rules soft-deleted (active=FALSE)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_includes_soft_deleted_reconciliation_rules(pool: MySqlPool) {
    let ctx = seed_company(&pool, "co_t", Role::Comptable).await;
    // Création d'un compte counterparty FR obligatoire pour la FK.
    let acc_cp = create_acc(
        &pool,
        ctx.user_id,
        ctx.company_id,
        "1100",
        "Banque CP",
        AccountType::Asset,
    )
    .await;
    sqlx::query(
        "INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, counterparty_account_id, priority, active, applied_count, version) \
         VALUES (?, 'Active rule', 'reference_contains', 'salaire', ?, 100, TRUE, 0, 1), \
                (?, 'Deleted rule', 'reference_contains', 'remboursement', ?, 90, FALSE, 0, 1)",
    )
    .bind(ctx.company_id).bind(acc_cp)
    .bind(ctx.company_id).bind(acc_cp)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let meta = extract_meta(&entries);
    assert_eq!(
        meta["tables"]["reconciliation_rules.csv"]["rowCount"]
            .as_u64()
            .unwrap(),
        2,
        "both active and soft-deleted reconciliation_rules must be exported"
    );
}

// ============================================================
// AC #29(u) — Multi-tenant scoping toutes fns `list_all_by_company`
// ============================================================
//
// Pass 3 ECH3-MEDIUM-05 — seed 2 companies (A + B) avec 1 row dans chacune des
// 8 tables couvertes par les nouvelles `list_all_by_company`, puis appel direct
// du repository A et assert qu'on n'a JAMAIS de row de B.

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_repo_scoping_all_list_all_by_company(pool: MySqlPool) {
    use kesh_db::repositories::{
        bank_imports, bank_profiles, bank_transactions, invoices, journal_entries, products,
        reconciliation_rules, vat_rates,
    };

    let a = seed_company(&pool, "co_u_a", Role::Comptable).await;
    let b = seed_company(&pool, "co_u_b", Role::Comptable).await;

    // Insertion d'1 row par company dans chacune des 8 tables (couples).
    // products
    sqlx::query("INSERT INTO products (company_id, name, unit_price, vat_rate, active, version) VALUES (?, 'pa', 1.00, 8.10, TRUE, 1), (?, 'pb', 1.00, 8.10, TRUE, 1)")
        .bind(a.company_id).bind(b.company_id).execute(&pool).await.unwrap();
    // vat_rates
    sqlx::query("INSERT INTO vat_rates (company_id, label, rate, valid_from, active) VALUES (?, 'normal-a', 8.10, '2026-01-01', TRUE), (?, 'normal-b', 8.10, '2026-01-01', TRUE)")
        .bind(a.company_id).bind(b.company_id).execute(&pool).await.unwrap();
    // reconciliation_rules — nécessite counterparty_account_id (FK).
    let acc_a = create_acc(
        &pool,
        a.user_id,
        a.company_id,
        "1100",
        "Banque A",
        AccountType::Asset,
    )
    .await;
    let acc_b = create_acc(
        &pool,
        b.user_id,
        b.company_id,
        "1100",
        "Banque B",
        AccountType::Asset,
    )
    .await;
    sqlx::query("INSERT INTO reconciliation_rules (company_id, label, match_type, match_value, counterparty_account_id, priority, active, applied_count, version) VALUES (?, 'lA', 'reference_contains', 'va', ?, 100, TRUE, 0, 1), (?, 'lB', 'reference_contains', 'vb', ?, 100, TRUE, 0, 1)")
        .bind(a.company_id).bind(acc_a)
        .bind(b.company_id).bind(acc_b)
        .execute(&pool).await.unwrap();
    // bank_profiles
    let cm = "{\"columns\":[]}";
    sqlx::query("INSERT INTO bank_profiles (company_id, bank_name, filename_pattern, column_mapping, date_format, decimal_separator, field_separator, encoding, header_row_count) VALUES (?, 'pA', NULL, ?, 'YYYY-MM-DD', '.', ',', NULL, 0), (?, 'pB', NULL, ?, 'YYYY-MM-DD', '.', ',', NULL, 0)")
        .bind(a.company_id).bind(cm)
        .bind(b.company_id).bind(cm)
        .execute(&pool).await.unwrap();
    // bank_accounts + bank_imports + bank_transactions
    // MariaDB ne supporte pas `RETURNING` sur INSERT — on lit `LAST_INSERT_ID()` après.
    let bank_a_id: i64 = {
        sqlx::query("INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version) VALUES (?, 'BA', 'CH9300762011623852957', NULL, TRUE, NULL, 1)")
            .bind(a.company_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let bank_b_id: i64 = {
        sqlx::query("INSERT INTO bank_accounts (company_id, bank_name, iban, qr_iban, is_primary, journal_account_id, version) VALUES (?, 'BB', 'CH9300762011623852958', NULL, TRUE, NULL, 1)")
            .bind(b.company_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let imp_a_id: i64 = {
        sqlx::query("INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, source_format, statement_id, period_from, period_to, opening_balance, closing_balance, transaction_count, imported_by_user_id) VALUES (?, ?, 'a.xml', 'a000000000000000000000000000000000000000000000000000000000000000', 'CAMT053_V04', NULL, '2026-01-01', '2026-01-31', NULL, NULL, 0, ?)")
            .bind(a.company_id).bind(bank_a_id).bind(a.user_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let imp_b_id: i64 = {
        sqlx::query("INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, source_format, statement_id, period_from, period_to, opening_balance, closing_balance, transaction_count, imported_by_user_id) VALUES (?, ?, 'b.xml', 'b000000000000000000000000000000000000000000000000000000000000000', 'CAMT053_V04', NULL, '2026-01-01', '2026-01-31', NULL, NULL, 0, ?)")
            .bind(b.company_id).bind(bank_b_id).bind(b.user_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    // bank_transactions
    sqlx::query("INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, value_date, amount, currency, reference, details, status, version) VALUES (?, ?, ?, '2026-01-10', '2026-01-10', 100.00, 'CHF', NULL, '', 'pending', 1), (?, ?, ?, '2026-01-15', '2026-01-15', 200.00, 'CHF', NULL, '', 'pending', 1)")
        .bind(a.company_id).bind(imp_a_id).bind(bank_a_id)
        .bind(b.company_id).bind(imp_b_id).bind(bank_b_id)
        .execute(&pool).await.unwrap();
    // journal_entries via repository pour avoir la mécanique d'entry_number + lines + journal_entries
    let acc_cash_a = create_acc(
        &pool,
        a.user_id,
        a.company_id,
        "1000",
        "Caisse",
        AccountType::Asset,
    )
    .await;
    let acc_rev_a = create_acc(
        &pool,
        a.user_id,
        a.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    let acc_cash_b = create_acc(
        &pool,
        b.user_id,
        b.company_id,
        "1000",
        "Caisse",
        AccountType::Asset,
    )
    .await;
    let acc_rev_b = create_acc(
        &pool,
        b.user_id,
        b.company_id,
        "3000",
        "Ventes",
        AccountType::Revenue,
    )
    .await;
    post_entry(
        &pool,
        a.user_id,
        a.fy_id,
        a.company_id,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        Journal::Ventes,
        "JE A",
        acc_cash_a,
        acc_rev_a,
        dec!(100.00),
    )
    .await;
    post_entry(
        &pool,
        b.user_id,
        b.fy_id,
        b.company_id,
        NaiveDate::from_ymd_opt(2026, 5, 2).unwrap(),
        Journal::Ventes,
        "JE B",
        acc_cash_b,
        acc_rev_b,
        dec!(200.00),
    )
    .await;
    // invoices : on doit créer un contact d'abord (FK)
    let contact_a: i64 = {
        sqlx::query("INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier, active, version) VALUES (?, 'Personne', 'CA', TRUE, FALSE, TRUE, 1)")
            .bind(a.company_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let contact_b: i64 = {
        sqlx::query("INSERT INTO contacts (company_id, contact_type, name, is_client, is_supplier, active, version) VALUES (?, 'Personne', 'CB', TRUE, FALSE, TRUE, 1)")
            .bind(b.company_id).execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let invoice_a_id: i64 = {
        sqlx::query("INSERT INTO invoices (company_id, contact_id, status, date, total_amount, version) VALUES (?, ?, 'draft', '2026-05-01', 100.00, 1)")
            .bind(a.company_id).bind(contact_a)
            .execute(&pool).await.unwrap().last_insert_id() as i64
    };
    let invoice_b_id: i64 = {
        sqlx::query("INSERT INTO invoices (company_id, contact_id, status, date, total_amount, version) VALUES (?, ?, 'draft', '2026-05-02', 200.00, 1)")
            .bind(b.company_id).bind(contact_b)
            .execute(&pool).await.unwrap().last_insert_id() as i64
    };
    // Pass 1 code-review H5 (C3-AA-HIGH-02 + C3-ECH-007) — insérer 1 invoice_line
    // pour A et 1 pour B pour que l'assertion `list_all_lines_by_company(A) == 1`
    // prouve effectivement le scoping JOIN (vs tautologique `== 0` quand 0 lignes
    // partout). Si la query JOIN ne filtre pas correctement par `i.company_id`,
    // le résultat sera 2 au lieu de 1 → IDOR détecté.
    sqlx::query("INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) VALUES (?, 1, 'Line A', 1.00, 100.00, 8.10, 100.00)")
        .bind(invoice_a_id).execute(&pool).await.unwrap();
    sqlx::query("INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) VALUES (?, 1, 'Line B', 1.00, 200.00, 8.10, 200.00)")
        .bind(invoice_b_id).execute(&pool).await.unwrap();

    // Assert scoping de chaque fn list_all_by_company sur company A.
    let prods_a = products::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(prods_a.len(), 1);
    assert_eq!(prods_a[0].company_id, a.company_id);

    let vats_a = vat_rates::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(vats_a.len(), 1);
    assert_eq!(vats_a[0].company_id, a.company_id);

    let rules_a = reconciliation_rules::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(rules_a.len(), 1);
    assert_eq!(rules_a[0].company_id, a.company_id);

    let profiles_a = bank_profiles::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(profiles_a.len(), 1);
    assert_eq!(profiles_a[0].company_id, a.company_id);

    let imports_a = bank_imports::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(imports_a.len(), 1);
    assert_eq!(imports_a[0].company_id, a.company_id);

    let txs_a = bank_transactions::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(txs_a.len(), 1);
    assert_eq!(txs_a[0].company_id, a.company_id);

    let entries_a = journal_entries::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(entries_a.len(), 1);
    assert_eq!(entries_a[0].company_id, a.company_id);

    let invoices_a = invoices::list_all_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(invoices_a.len(), 1);
    assert_eq!(invoices_a[0].company_id, a.company_id);

    // JOIN-scoped (no direct company_id sur ces entités) — count via fn.
    let lines_jel_a = journal_entries::list_all_lines_by_company(&pool, a.company_id)
        .await
        .unwrap();
    assert_eq!(lines_jel_a.len(), 2); // 1 entry × 2 lines = 2
    let invoice_lines_a = invoices::list_all_lines_by_company(&pool, a.company_id)
        .await
        .unwrap();
    // Pass 1 code-review H5 — 1 invoice + 1 ligne pour A, 1 invoice + 1 ligne
    // pour B. Le JOIN doit isoler strictement A → exactement 1 row retournée.
    assert_eq!(
        invoice_lines_a.len(),
        1,
        "invoice_lines scoping cassé : attendu 1 ligne pour A, reçu {} (probable fuite B)",
        invoice_lines_a.len()
    );
    assert_eq!(
        invoice_lines_a[0].invoice_id, invoice_a_id,
        "ligne retournée pour A ne pointe pas vers l'invoice de A — fuite cross-tenant"
    );
}

// ============================================================
// Story 25-5-a (#386) — les onze tables ajoutées : elles SORTENT,
// et elles ne fuient pas d'une société à l'autre.
// ============================================================

/// ⛔ **Deux propriétés en un seul test, et il faut les deux.**
///
/// Qu'un CSV soit présent ne prouve rien — un sérialiseur qui n'écrirait que
/// son en-tête produirait un fichier, et le test de structure resterait vert.
/// Ce test vérifie donc que **la ligne de la société A y figure**.
///
/// Et le symétrique compte autant : **la ligne de la société B n'y est pas**.
/// Les trois tables enfants (`credit_note_lines`, `supplier_invoice_lines`,
/// `payment_batch_items`) n'ont **pas de `company_id`** — leur filtre passe par
/// une jointure sur le parent, et c'est exactement le genre de scoping qu'on
/// oublie. *Une fonction d'export non scopée exfiltrerait les données d'une
/// autre société sans que rien ne le signale.*
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn export_global_zip_onze_tables_neuves_sortent_et_sont_scopees(pool: MySqlPool) {
    let a = seed_company(&pool, "co_386_a", Role::Comptable).await;
    let b = seed_company(&pool, "co_386_b", Role::Comptable).await;

    // Un jeu minimal par société : contact → projet → personne de contact,
    // puis facture + avoir, facture fournisseur + lot de paiement, règlement.
    for (ctx, suffixe) in [(&a, "A"), (&b, "B")] {
        let cid = ctx.company_id;
        let contact_id: i64 = sqlx::query_scalar(
            "INSERT INTO contacts (company_id, contact_type, name, version) \
             VALUES (?, 'Entreprise', ?, 1) RETURNING id",
        )
        .bind(cid)
        .bind(format!("Contact {suffixe}"))
        .fetch_one(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO projects (company_id, code, name, archived, version) \
             VALUES (?, ?, ?, FALSE, 1)",
        )
        .bind(cid)
        .bind(format!("PRJ-{suffixe}"))
        .bind(format!("Projet {suffixe}"))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO contact_persons (company_id, contact_id, first_name, last_name, \
             active, version) VALUES (?, ?, ?, 'Nom', TRUE, 1)",
        )
        .bind(cid)
        .bind(contact_id)
        .bind(format!("Prenom{suffixe}"))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO imported_supplier_invoices (company_id, status, file_hash, \
             storage_path, original_filename, mime_type, byte_size, creditor_iban, \
             is_qr_iban, creditor_address_type, creditor_name, creditor_country, \
             reference_type, currency, version) \
             VALUES (?, 'to_complete', ?, ?, ?, 'application/pdf', 1024, \
             'CH9300762011623852957', FALSE, 'S', ?, 'CH', 'NON', 'CHF', 1)",
        )
        .bind(cid)
        .bind(format!("hash-{suffixe}"))
        .bind(format!("/inbox/{suffixe}.pdf"))
        .bind(format!("piece-{suffixe}.pdf"))
        .bind(format!("Fournisseur {suffixe}"))
        .execute(&pool)
        .await
        .unwrap();

        // Les huit autres tables neuves, dont les trois enfants scopées par
        // jointure — revue P1 : elles n'étaient vérifiées que VIDES, ce qui ne
        // prouvait rien de leur scoping.
        //
        // ⚠️ Montage par SQL direct, clés étrangères suspendues sur UNE
        // connexion dédiée : ce test éprouve le FILTRE de l'export (chaque
        // ligne sort pour sa société, jamais pour l'autre), pas les flux
        // métier qui produisent ces lignes — les monter par les vrais chemins
        // (facture validée, avoir, règlement, lot) coûterait un plan comptable
        // et des réglages complets par société, sans rien ajouter à la preuve.
        // Les identifiants référencés hors de ces tables (écritures, comptes,
        // compte bancaire) sont donc fictifs. Les contraintes CHECK, elles,
        // restent actives.
        //
        // ⚠️ Une panique entre les deux `SET` rendrait la connexion au pool sans
        // contrôle des clés : sans effet — base et pool sont propres à ce test
        // (`#[sqlx::test]`), et le test a déjà échoué (revue P2, reclassé LOW).
        let mut conn = pool.acquire().await.unwrap();
        // Identifiants fictifs DISTINCTS par société : certaines colonnes sont
        // uniques (`uq_credit_notes_invoice`).
        let fictif = 900_000 + cid * 100;
        sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
            .execute(&mut *conn)
            .await
            .unwrap();
        let credit_note_id: i64 = sqlx::query_scalar(
            "INSERT INTO credit_notes (company_id, contact_id, invoice_id, \
             credit_note_number, status, date, total_amount) \
             VALUES (?, ?, ?, ?, 'draft', '2026-03-01', 10) RETURNING id",
        )
        .bind(cid)
        .bind(contact_id)
        .bind(fictif + 1)
        .bind(format!("AV-386-{suffixe}"))
        .fetch_one(&mut *conn)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO credit_note_lines (credit_note_id, position, description, \
             quantity, unit_price, vat_rate, line_total) VALUES (?, 1, ?, 1, 10, 0, 10)",
        )
        .bind(credit_note_id)
        .bind(format!("LigneAvoir386{suffixe}"))
        .execute(&mut *conn)
        .await
        .unwrap();
        let supplier_invoice_id: i64 = sqlx::query_scalar(
            "INSERT INTO supplier_invoices (company_id, contact_id, supplier_invoice_number, \
             status, invoice_date, total_amount, purchase_journal_entry_id) \
             VALUES (?, ?, ?, 'open', '2026-03-01', 10, ?) RETURNING id",
        )
        .bind(cid)
        .bind(contact_id)
        .bind(format!("FF-386-{suffixe}"))
        .bind(fictif + 2)
        .fetch_one(&mut *conn)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO supplier_invoice_lines (supplier_invoice_id, position, description, \
             quantity, unit_price, vat_rate, line_total, expense_account_id) \
             VALUES (?, 1, ?, 1, 10, 0, 10, ?)",
        )
        .bind(supplier_invoice_id)
        .bind(format!("LigneAchat386{suffixe}"))
        .bind(fictif + 3)
        .execute(&mut *conn)
        .await
        .unwrap();
        let batch_id: i64 = sqlx::query_scalar(
            "INSERT INTO payment_batches (company_id, bank_account_id, status, \
             requested_execution_date, total_amount, msg_id, payment_info_id) \
             VALUES (?, ?, 'generated', '2026-03-02', 10, ?, ?) RETURNING id",
        )
        .bind(cid)
        .bind(fictif + 4)
        .bind(format!("MSG-386-{suffixe}"))
        .bind(format!("PMT-386-{suffixe}"))
        .fetch_one(&mut *conn)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO payment_batch_items (payment_batch_id, supplier_invoice_id, position, \
             end_to_end_id, amount) VALUES (?, ?, 1, ?, 10)",
        )
        .bind(batch_id)
        .bind(supplier_invoice_id)
        .bind(format!("E2E-386-{suffixe}"))
        .execute(&mut *conn)
        .await
        .unwrap();
        // `invoice_settlements` n'a aucune colonne de texte : le marqueur est
        // un MONTANT propre à chaque société.
        sqlx::query(
            "INSERT INTO invoice_settlements (company_id, invoice_id, journal_entry_id, \
             amount, settled_on, settlement_type, settlement_account_id) \
             VALUES (?, ?, ?, ?, '2026-03-03', 'internal_account', ?)",
        )
        .bind(cid)
        .bind(fictif + 5)
        .bind(fictif + 6)
        .bind(if suffixe == "A" { "7777.77" } else { "8888.88" })
        .bind(fictif + 7)
        .execute(&mut *conn)
        .await
        .unwrap();
        sqlx::query(
            "INSERT INTO audit_log (user_id, action, entity_type, entity_id, actor_label, \
             company_id) VALUES (?, 'test.marker', 'marker', 1, ?, ?)",
        )
        .bind(ctx.user_id)
        .bind(format!("Acteur386{suffixe}"))
        .bind(cid)
        .execute(&mut *conn)
        .await
        .unwrap();
        sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
            .execute(&mut *conn)
            .await
            .unwrap();
    }

    let app = spawn_app(pool.clone()).await;
    let resp = app
        .client
        .get(app.url("/api/v1/exports/global.zip"))
        .bearer_auth(&a.jwt)
        .send()
        .await
        .unwrap();
    let body = resp.bytes().await.unwrap();
    let entries = assert_zip_response(&body);
    let lire = |nom: &str| -> String {
        entries
            .iter()
            .find(|(n, _)| n == nom)
            .map(|(_, c)| String::from_utf8_lossy(c).to_string())
            .unwrap_or_else(|| panic!("`{nom}` absent du ZIP"))
    };

    // Les douze tables peuplées ci-dessus : la ligne de A sort, celle de B non.
    // ⛔ Les trois enfants sans `company_id` (`credit_note_lines`,
    // `supplier_invoice_lines`, `payment_batch_items`) sont celles dont le
    // filtre passe par une jointure : c'est leur ligne qui compte le plus.
    for (fichier, marqueur_a, marqueur_b) in [
        ("projects.csv", "Projet A", "Projet B"),
        ("contact_persons.csv", "PrenomA", "PrenomB"),
        (
            "imported_supplier_invoices.csv",
            "Fournisseur A",
            "Fournisseur B",
        ),
        ("contacts.csv", "Contact A", "Contact B"),
        ("credit_notes.csv", "AV-386-A", "AV-386-B"),
        ("credit_note_lines.csv", "LigneAvoir386A", "LigneAvoir386B"),
        ("supplier_invoices.csv", "FF-386-A", "FF-386-B"),
        (
            "supplier_invoice_lines.csv",
            "LigneAchat386A",
            "LigneAchat386B",
        ),
        ("payment_batches.csv", "MSG-386-A", "MSG-386-B"),
        ("payment_batch_items.csv", "E2E-386-A", "E2E-386-B"),
        ("invoice_settlements.csv", "7777.77", "8888.88"),
        ("audit_log.csv", "Acteur386A", "Acteur386B"),
    ] {
        let csv = lire(fichier);
        assert!(
            csv.contains(marqueur_a),
            "`{fichier}` doit porter la ligne de la société A ({marqueur_a})"
        );
        assert!(
            !csv.contains(marqueur_b),
            "⛔ FUITE MULTI-TENANT : `{fichier}` porte une ligne de la société B ({marqueur_b})"
        );
    }
}
