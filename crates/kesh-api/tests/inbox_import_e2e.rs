//! Tests E2E HTTP — Story 12.5c : import répertoire de factures (#194).
//!
//! Couvre AC2-AC11 :
//! - import inbox : staging + archivage + rapport ; échecs par catégorie
//!   (type/magic, symlink, doublon scopé, réactivation `discarded`, taille,
//!   répertoire ignoré, FIELD_TOO_LONG D2) ; verrou de run (409).
//! - complétion atomique DC6 : succès, exercice fermé (reste `to_complete`),
//!   réconciliation montant (exact + sous-centime F-OPUS-2), devise CHF-only
//!   (F-OPUS-1), routage QR-IBAN⇔QRR, statut non-pending (409).
//! - `discard` ; liste (validation `status` D1) ; download (404/410, anti-IDOR).
//!
//! Pré-requis : MariaDB démarré (`sqlx::test` crée une DB éphémère par test) +
//! `libpdfium.so` pour les fixtures PDF (aucune ici — fixtures PNG via rxing).
#![allow(clippy::too_many_arguments)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::imported_supplier_invoice::NewImportedSupplierInvoice;
use kesh_db::entities::{Language, NewCompany, NewUser, OrgType, Role};
use kesh_db::repositories::{companies, contacts, imported_supplier_invoices, users};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use kesh_qrbill::{Address, AddressType, Currency, QrBillData, Reference, build_payload};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================
// Harness
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    inbox: PathBuf,
    documents: PathBuf,
}

impl TestApp {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }
}

fn unique_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("kesh-{prefix}-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn test_config(inbox: &Path, documents: &Path, max_file_bytes: u64) -> Config {
    let mut config = Config::from_fields_for_test(
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
    );
    config.inbox_dir = inbox.to_string_lossy().to_string();
    config.documents_dir = documents.to_string_lossy().to_string();
    config.inbox_max_file_bytes = max_file_bytes;
    config
}

async fn spawn_app(pool: MySqlPool, max_file_bytes: u64) -> TestApp {
    let inbox = unique_dir("inbox");
    let documents = unique_dir("documents");
    let config = test_config(&inbox, &documents, max_file_bytes);
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
        inbox,
        documents,
    }
}

fn forge_jwt(user_id: i64, company_id: i64) -> String {
    let now = Utc::now().timestamp();
    let claims = Claims {
        sub: user_id.to_string(),
        role: "Comptable".to_string(),
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
    seeded: SeededCompany,
    supplier_id: i64,
    jwt: String,
}

async fn setup(pool: &MySqlPool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.unwrap();
    // Le seed pose default_receivable/revenue (ventes) mais PAS le compte
    // créanciers (achats) — requis par create_in_tx (cf. supplier_invoices repo test).
    sqlx::query(
        "UPDATE company_invoice_settings SET default_payable_account_id = ? WHERE company_id = ?",
    )
    .bind(seeded.accounts["2000"])
    .bind(seeded.company_id)
    .execute(pool)
    .await
    .unwrap();
    let supplier_id = contacts::create(
        pool,
        seeded.admin_user_id,
        NewContact {
            company_id: seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: "Fournisseur SA".into(),
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: Some("30".into()),
        },
    )
    .await
    .expect("create supplier")
    .id;
    let jwt = forge_jwt(seeded.admin_user_id, seeded.company_id);
    Ctx {
        seeded,
        supplier_id,
        jwt,
    }
}

/// Crée une **2e** company + user (username unique) et renvoie son JWT — pour les
/// tests anti-IDOR (`seed_accounting_company` hardcode le username `admin`, unique
/// global, donc non rappelable).
async fn other_company_jwt(pool: &MySqlPool) -> String {
    let company_id = companies::create(
        pool,
        NewCompany {
            name: "Autre SA".into(),
            address: "Rue 9\n1200 Genève".into(),
            ide_number: None,
            org_type: OrgType::Independant,
            accounting_language: Language::Fr,
            instance_language: Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    let username = format!("other-{}", uuid::Uuid::new_v4().simple());
    let user_id = users::create(
        pool,
        NewUser {
            username,
            password_hash: hash_password("password123").unwrap(),
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    forge_jwt(user_id, company_id)
}

// ============================================================
// Fixtures QR (rxing writer — pas d'accès au helper #[cfg(test)] de la lib)
// ============================================================

/// Rasterise un payload SPC en PNG via le writer rxing (dép runtime kesh-api).
fn qr_png_from_payload(payload: &str) -> Vec<u8> {
    use rxing::{BarcodeFormat, EncodeHints, Writer};
    let writer = rxing::qrcode::QRCodeWriter {};
    let matrix = writer
        .encode_with_hints(
            payload,
            &BarcodeFormat::QR_CODE,
            512,
            512,
            &EncodeHints::default(),
        )
        .expect("encode QR");
    let w = matrix.getWidth();
    let h = matrix.getHeight();
    let img = image::GrayImage::from_fn(w, h, |x, y| {
        if matrix.get(x, y) {
            image::Luma([0u8])
        } else {
            image::Luma([255u8])
        }
    });
    let mut png = Vec::new();
    image::DynamicImage::ImageLuma8(img)
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    png
}

/// PNG d'une facture QR standard (IBAN non-QR, référence None, montant `amount`).
/// `name` permet d'injecter un nom créancier sur-long pour le test D2.
fn qr_invoice_png(amount: Decimal, name: &str) -> Vec<u8> {
    let data = QrBillData {
        creditor_iban: "CH9300762011623852957".into(),
        creditor: Address {
            address_type: AddressType::Combined,
            name: "NAMEPLACEHOLDER".into(),
            line1: "Rue du Lac 1268".into(),
            line2: "2501 Biel".into(),
            country: "CH".into(),
        },
        ultimate_debtor: None,
        amount: Some(amount),
        currency: Currency::Chf,
        reference: Reference::None,
        unstructured_message: Some("Facture test".into()),
        billing_information: None,
    };
    // build_payload valide les longueurs (nom ≤ 70) ; on génère avec un token
    // court puis on substitue le nom dans le payload (le parseur 12-5a ne borne
    // pas les longueurs → le test D2 fait échouer l'INSERT staging en aval).
    let payload = build_payload(&data)
        .unwrap()
        .replace("NAMEPLACEHOLDER", name);
    qr_png_from_payload(&payload)
}

/// PNG blanc 64×64 sans aucun QR.
fn blank_png() -> Vec<u8> {
    let img =
        image::DynamicImage::ImageLuma8(image::GrayImage::from_pixel(64, 64, image::Luma([255u8])));
    let mut png = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .unwrap();
    png
}

// ============================================================
// Seed direct d'un staging (pour les tests complétion / download)
// ============================================================

#[allow(clippy::too_many_arguments)]
async fn seed_staging(
    pool: &MySqlPool,
    company_id: i64,
    currency: &str,
    is_qr_iban: bool,
    reference_type: &str,
    reference_value: Option<&str>,
    amount: Option<Decimal>,
    name: &str,
    storage_path: &str,
) -> i64 {
    let new = NewImportedSupplierInvoice {
        company_id,
        file_hash: format!("hash-{}", uuid::Uuid::new_v4().simple()),
        storage_path: storage_path.to_string(),
        original_filename: "facture.png".into(),
        mime_type: "image/png".into(),
        byte_size: 1234,
        creditor_iban: "CH9300762011623852957".into(),
        is_qr_iban,
        creditor_address_type: "K".into(),
        creditor_name: name.into(),
        creditor_line1: Some("Rue du Lac 1268".into()),
        creditor_line2: Some("2501 Biel".into()),
        creditor_postal_code: None,
        creditor_town: None,
        creditor_country: "CH".into(),
        reference_type: reference_type.into(),
        reference_value: reference_value.map(|s| s.to_string()),
        amount,
        currency: currency.into(),
        unstructured_message: None,
        billing_information: None,
    };
    imported_supplier_invoices::create(pool, &new)
        .await
        .expect("seed staging")
        .id
}

async fn staging_status(pool: &MySqlPool, id: i64) -> String {
    sqlx::query_scalar::<_, String>("SELECT status FROM imported_supplier_invoices WHERE id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn supplier_invoice_count(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM supplier_invoices WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

// ============================================================
// A. Import inbox
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_creates_staging_and_archives(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(
        app.inbox.join("facture1.png"),
        qr_invoice_png(dec!(100.00), "Robert SA"),
    )
    .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["accepted"].as_array().unwrap().len(), 1);
    assert!(body["failed"].as_array().unwrap().is_empty());

    // Staging créé `to_complete`.
    let items =
        imported_supplier_invoices::list_by_status(&pool, ctx.seeded.company_id, "to_complete")
            .await
            .unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].currency, "CHF");
    assert_eq!(items[0].amount, Some(dec!(100.00)));
    // Fichier inbox supprimé, justificatif archivé présent.
    assert!(!app.inbox.join("facture1.png").exists());
    assert!(app.documents.join(&items[0].storage_path).exists());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_rejects_unsupported_type_and_no_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(app.inbox.join("notes.txt"), b"pas une image").unwrap();
    std::fs::write(app.inbox.join("blank.png"), blank_png()).unwrap();

    let body: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2);
    let codes: Vec<&str> = failed
        .iter()
        .map(|f| f["errorCode"].as_str().unwrap())
        .collect();
    assert!(codes.contains(&"UNSUPPORTED_FILE_TYPE"));
    assert!(codes.contains(&"NO_QR_CODE_FOUND"));
    // Les deux déplacés dans failed/.
    assert_eq!(
        std::fs::read_dir(app.inbox.join("failed")).unwrap().count(),
        2
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_symlink_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    // Cible hors inbox + symlink dans l'inbox avec extension permise.
    let target = unique_dir("symtarget").join("real.pdf");
    std::fs::write(&target, b"%PDF-1.4 fake").unwrap();
    std::os::unix::fs::symlink(&target, app.inbox.join("link.pdf")).unwrap();

    let body: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["errorCode"], "SYMLINK_REJECTED");
    // La cible n'a jamais été suivie/ouverte.
    assert!(target.exists());
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_duplicate_scoped_and_reactivates_discarded(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let png = qr_invoice_png(dec!(50.00), "Dup SA");

    // 1er import → accepté.
    std::fs::write(app.inbox.join("dup.png"), &png).unwrap();
    let b1: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(b1["accepted"].as_array().unwrap().len(), 1);
    let staging_id = b1["accepted"][0]["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();

    // 2e import du même contenu → DUPLICATE.
    std::fs::write(app.inbox.join("dup2.png"), &png).unwrap();
    let b2: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(b2["failed"][0]["errorCode"], "DUPLICATE");

    // On écarte la row, puis on ré-importe → réactivation `to_complete`.
    sqlx::query("UPDATE imported_supplier_invoices SET status='discarded' WHERE id=?")
        .bind(staging_id)
        .execute(&pool)
        .await
        .unwrap();
    std::fs::write(app.inbox.join("dup3.png"), &png).unwrap();
    let b3: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        b3["accepted"][0]["importedSupplierInvoiceId"]
            .as_i64()
            .unwrap(),
        staging_id
    );
    assert_eq!(staging_status(&pool, staging_id).await, "to_complete");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_file_too_large_and_directory_ignored(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    // Cap minuscule (10 octets) → tout PNG dépasse.
    let app = spawn_app(pool.clone(), 10).await;
    std::fs::write(
        app.inbox.join("big.png"),
        qr_invoice_png(dec!(10.00), "Big SA"),
    )
    .unwrap();
    // Un sous-répertoire dans l'inbox doit être ignoré (ni accepted ni failed).
    std::fs::create_dir_all(app.inbox.join("subdir")).unwrap();

    let body: Value = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert!(body["accepted"].as_array().unwrap().is_empty());
    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 1);
    assert_eq!(failed[0]["errorCode"], "FILE_TOO_LARGE");
    assert_eq!(failed[0]["fileName"], "big.png");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_field_too_long_returns_failed_not_500(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    // Nom créancier > 70 chars (QR tiers non conforme SIX 2.2) → 1406 → FIELD_TOO_LONG.
    let long_name = "X".repeat(100);
    std::fs::write(
        app.inbox.join("long.png"),
        qr_invoice_png(dec!(20.00), &long_name),
    )
    .unwrap();

    let resp = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(
        resp.status(),
        200,
        "succès partiel reste HTTP 200 (PAS 500)"
    );
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["failed"][0]["errorCode"], "FIELD_TOO_LONG");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn import_already_running_returns_409(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;

    // On tient le verrou de run sur une connexion dédiée (même clé namespacée DB).
    let mut held = pool.acquire().await.unwrap();
    let db: Option<String> = sqlx::query_scalar("SELECT DATABASE()")
        .fetch_one(&mut *held)
        .await
        .unwrap();
    let key = format!("kesh_inbox_import:{}", db.unwrap_or_default());
    let got: Option<i64> = sqlx::query_scalar("SELECT GET_LOCK(?, 0)")
        .bind(&key)
        .fetch_one(&mut *held)
        .await
        .unwrap();
    assert_eq!(got, Some(1));

    let resp = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "INBOX_IMPORT_ALREADY_RUNNING");

    let _: Option<i64> = sqlx::query_scalar("SELECT RELEASE_LOCK(?)")
        .bind(&key)
        .fetch_one(&mut *held)
        .await
        .unwrap();
}

// ============================================================
// B. Complétion atomique (DC6)
// ============================================================

fn complete_body(
    supplier_id: i64,
    expense_account_id: i64,
    qty: Decimal,
    unit: Decimal,
    vat: Decimal,
) -> Value {
    json!({
        "contactId": supplier_id,
        "invoiceDate": "2026-06-15",
        "supplierInvoiceNumber": "FF-2026-100",
        "dueDate": "2026-07-15",
        "lines": [{
            "description": "Prestation",
            "quantity": qty,
            "unitPrice": unit,
            "vatRate": vat,
            "expenseAccountId": expense_account_id,
        }]
    })
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_creates_invoice_and_marks_completed(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let staging_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "Robert SA",
        "x.png",
    )
    .await;

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{staging_id}/complete"
        )))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(100.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "completion body: {body}");
    assert_eq!(body["status"], "open");
    assert_eq!(body["totalAmount"], "100.0000");
    assert_eq!(body["creditorIban"], "CH9300762011623852957");

    assert_eq!(staging_status(&pool, staging_id).await, "completed");
    assert_eq!(
        supplier_invoice_count(&pool, ctx.seeded.company_id).await,
        1
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_closed_fiscal_year_keeps_to_complete(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let staging_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "Robert SA",
        "x.png",
    )
    .await;

    // Date hors de tout exercice ouvert (le seed couvre 2020-2030).
    let mut body = complete_body(
        ctx.supplier_id,
        ctx.seeded.accounts["4000"],
        dec!(1),
        dec!(100.00),
        dec!(0),
    );
    body["invoiceDate"] = json!("2015-01-01");

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{staging_id}/complete"
        )))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    // Rollback total : aucune facture, staging reste `to_complete`.
    assert_eq!(staging_status(&pool, staging_id).await, "to_complete");
    assert_eq!(
        supplier_invoice_count(&pool, ctx.seeded.company_id).await,
        0
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_amount_mismatch_and_sub_centime_blocked(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;

    // (a) Montant simple non concordant.
    let s1 = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "A",
        "x.png",
    )
    .await;
    let r1 = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{s1}/complete")))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(99.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 400);
    let b1: Value = r1.json().await.unwrap();
    assert_eq!(b1["error"]["code"], "AMOUNT_MISMATCH");

    // (b) Sous-centime : 3 × 33.3333 = 99.9999 ≠ 100.00 (égalité EXACTE, F-OPUS-2).
    let s2 = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "B",
        "y.png",
    )
    .await;
    let r2 = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{s2}/complete")))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(3),
            dec!(33.3333),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 400);
    let b2: Value = r2.json().await.unwrap();
    assert_eq!(b2["error"]["code"], "AMOUNT_MISMATCH");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_currency_eur_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let staging_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "EUR",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "Euro SA",
        "x.png",
    )
    .await;

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{staging_id}/complete"
        )))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(100.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "CURRENCY_NOT_SUPPORTED");
    assert_eq!(staging_status(&pool, staging_id).await, "to_complete");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_iban_reference_consistency(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;

    // QR-IBAN sans QRR → rejet.
    let s1 = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        true,
        "NON",
        None,
        Some(dec!(100.00)),
        "A",
        "x.png",
    )
    .await;
    let r1 = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{s1}/complete")))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(100.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 400);
    assert_eq!(
        r1.json::<Value>().await.unwrap()["error"]["code"],
        "IBAN_REFERENCE_MISMATCH"
    );

    // IBAN normal + QRR → rejet inverse.
    let s2 = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "QRR",
        Some("210000000003139471430009017"),
        Some(dec!(100.00)),
        "B",
        "y.png",
    )
    .await;
    let r2 = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{s2}/complete")))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(100.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 400);
    assert_eq!(
        r2.json::<Value>().await.unwrap()["error"]["code"],
        "IBAN_REFERENCE_MISMATCH"
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn complete_not_pending_returns_409(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let staging_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        Some(dec!(100.00)),
        "A",
        "x.png",
    )
    .await;
    sqlx::query("UPDATE imported_supplier_invoices SET status='discarded' WHERE id=?")
        .bind(staging_id)
        .execute(&pool)
        .await
        .unwrap();

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{staging_id}/complete"
        )))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(100.00),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 409);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["error"]["code"], "IMPORT_NOT_PENDING_COMPLETION");
    assert_eq!(body["error"]["details"]["currentStatus"], "discarded");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn discard_marks_discarded(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let staging_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        None,
        "A",
        "x.png",
    )
    .await;

    let resp = app
        .client
        .post(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{staging_id}/discard"
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    assert_eq!(staging_status(&pool, staging_id).await, "discarded");
}

// ============================================================
// C. Liste (validation status D1) + download
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn list_validates_status_d1(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        None,
        "A",
        "x.png",
    )
    .await;

    // Status invalide → 400.
    let r1 = app
        .client
        .get(app.url("/api/v1/imported-supplier-invoices?status=invalid"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r1.status(), 400);
    // Sans status → 400.
    let r2 = app
        .client
        .get(app.url("/api/v1/imported-supplier-invoices"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r2.status(), 400);
    // Status valide → 200 + liste scopée.
    let r3 = app
        .client
        .get(app.url("/api/v1/imported-supplier-invoices?status=to_complete"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r3.status(), 200);
    assert_eq!(
        r3.json::<Value>().await.unwrap().as_array().unwrap().len(),
        1
    );
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn download_source_document_404_410_and_idor(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;

    // (a) Justificatif présent → 200 + Content-Disposition.
    let bytes = qr_invoice_png(dec!(10.00), "A");
    std::fs::write(app.documents.join("present.png"), &bytes).unwrap();
    let ok_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        None,
        "A",
        "present.png",
    )
    .await;
    let r_ok = app
        .client
        .get(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{ok_id}/source-document"
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r_ok.status(), 200);
    assert!(
        r_ok.headers()
            .get("content-disposition")
            .unwrap()
            .to_str()
            .unwrap()
            .contains("attachment")
    );

    // (b) Métadonnée présente mais fichier disque absent → 410 Gone.
    let gone_id = seed_staging(
        &pool,
        ctx.seeded.company_id,
        "CHF",
        false,
        "NON",
        None,
        None,
        "A",
        "missing.png",
    )
    .await;
    let r_gone = app
        .client
        .get(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{gone_id}/source-document"
        )))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r_gone.status(), 410);

    // (c) Row inexistante → 404.
    let r_404 = app
        .client
        .get(app.url("/api/v1/imported-supplier-invoices/999999/source-document"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r_404.status(), 404);

    // (d) Anti-IDOR : un user d'une autre company ne voit pas le justificatif.
    let other_jwt = other_company_jwt(&pool).await;
    let r_idor = app
        .client
        .get(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{ok_id}/source-document"
        )))
        .bearer_auth(&other_jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(r_idor.status(), 404);
}
