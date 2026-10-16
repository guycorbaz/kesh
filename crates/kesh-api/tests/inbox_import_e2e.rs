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
use kesh_db::entities::address::StructuredAddress;
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
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: Some("Rue 2\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: Some("30".into()),
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
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
///
/// Story 30-2a (P4, F6) — le payload est encodé en **UTF-8 avec ECI** : sans
/// l'indication `CharacterSet`, un nom accentué serait relu en Latin-1 ou en
/// Shift_JIS, et le test du nom de 70 caractères accentués rougirait pour une
/// mauvaise raison.
fn qr_png_from_payload(payload: &str) -> Vec<u8> {
    use rxing::{BarcodeFormat, EncodeHints, Writer};
    let writer = rxing::qrcode::QRCodeWriter {};
    let hints = EncodeHints {
        CharacterSet: Some("UTF-8".to_string()),
        ..EncodeHints::default()
    };
    let matrix = writer
        .encode_with_hints(payload, &BarcodeFormat::QR_CODE, 512, 512, &hints)
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

// Index des lignes du payload SPC 0200 que les tests substituent (ordre SIX,
// identique aux constantes privées `L_*` de `kesh-qrbill/src/parser.rs`).
const SPC_LINE_IBAN: usize = 3;
const SPC_LINE_NAME: usize = 5;
const SPC_LINE_COUNTRY: usize = 10;
const SPC_LINE_AMOUNT: usize = 18;
const SPC_LINE_CURRENCY: usize = 19;
const SPC_LINE_EPD: usize = 30;

/// PNG d'une facture QR standard (IBAN non-QR, référence None, montant `amount`).
/// `name` permet d'injecter un nom créancier sur-long pour le test D2.
fn qr_invoice_png(amount: Decimal, name: &str) -> Vec<u8> {
    let amount = amount.to_string();
    qr_invoice_png_with(&[(SPC_LINE_AMOUNT, &amount), (SPC_LINE_NAME, name)])
}

/// PNG d'une facture QR dont des lignes du payload sont **substituées après**
/// `build_payload` — Story 30-2a (AC2), généralisation de la substitution du
/// nom (`NAMEPLACEHOLDER`) à toute ligne : montant, pays, devise, IBAN, trailer.
/// `build_payload` validerait ces valeurs et refuserait de générer ; le parseur
/// 12-5a, lui, ne borne ni les longueurs, ni le montant, ni le pays, ni la
/// devise — c'est exactement ce que le contrôle pur de l'import doit rattraper.
fn qr_invoice_png_with(overrides: &[(usize, &str)]) -> Vec<u8> {
    let data = QrBillData {
        creditor_iban: "CH9300762011623852957".into(),
        creditor: Address {
            address_type: AddressType::Combined,
            name: "NAMEPLACEHOLDER".into(),
            line1: "Rue du Lac 1268".into(),
            line2: "2501 Biel".into(),
            postal_code: String::new(),
            town: String::new(),
            country: "CH".into(),
        },
        ultimate_debtor: None,
        amount: Some(dec!(20.00)),
        currency: Currency::Chf,
        reference: Reference::None,
        unstructured_message: Some("Facture test".into()),
        billing_information: None,
    };
    // build_payload valide les longueurs (nom ≤ 70) ; on génère avec un token
    // court puis on substitue les lignes demandées dans le payload.
    let payload = build_payload(&data).unwrap();
    let mut lines: Vec<String> = payload.split('\n').map(str::to_string).collect();
    for (index, value) in overrides {
        lines[*index] = (*value).to_string();
    }
    qr_png_from_payload(&lines.join("\n"))
}

/// PDF minimal d'une page de texte, écrit à la main, **sans** QR (Story 30-2a,
/// fixture synthétique). Selon la présence de pdfium sur la station, l'import
/// le lit sans QR (`none`) ou ne peut le rendre (`unreadable`,
/// `PDF_RENDER_ERROR`) : les deux sont une acceptation sans données QR.
const PDF_SANS_QR: &[u8] = b"%PDF-1.4\n\
1 0 obj << /Type /Catalog /Pages 2 0 R >> endobj\n\
2 0 obj << /Type /Pages /Kids [3 0 R] /Count 1 >> endobj\n\
3 0 obj << /Type /Page /Parent 2 0 R /MediaBox [0 0 200 100] /Contents 4 0 R \
/Resources << /Font << /F1 5 0 R >> >> >> endobj\n\
4 0 obj << /Length 44 >> stream\n\
BT /F1 12 Tf 20 50 Td (Pas une facture) Tj ET\n\
endstream endobj\n\
5 0 obj << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >> endobj\n\
trailer << /Root 1 0 R >>\n\
%%EOF\n";

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
        creditor_iban: Some("CH9300762011623852957".into()),
        is_qr_iban: Some(is_qr_iban),
        creditor_address_type: Some("K".into()),
        creditor_name: Some(name.into()),
        creditor_line1: Some("Rue du Lac 1268".into()),
        creditor_line2: Some("2501 Biel".into()),
        creditor_postal_code: None,
        creditor_town: None,
        creditor_country: Some("CH".into()),
        reference_type: Some(reference_type.into()),
        reference_value: reference_value.map(|s| s.to_string()),
        amount,
        currency: Some(currency.into()),
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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
    assert_eq!(items[0].currency.as_deref(), Some("CHF"));
    assert_eq!(items[0].amount, Some(dec!(100.00)));
    // Fichier inbox supprimé, justificatif archivé présent.
    assert!(!app.inbox.join("facture1.png").exists());
    assert!(app.documents.join(&items[0].storage_path).exists());
}

/// Lance un run d'import et rend le rapport (HTTP 200 exigé).
async fn run_import(app: &TestApp, jwt: &str) -> Value {
    let resp = app
        .client
        .post(app.url("/api/v1/inbox-import"))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200, "un run d'import rend toujours 200");
    resp.json().await.unwrap()
}

/// L'entrée `accepted[]` d'un fichier, désignée par son nom (identifiant
/// business, jamais un index) ; échoue en citant le rapport si elle manque.
fn accepted_entry<'a>(report: &'a Value, file_name: &str) -> &'a Value {
    report["accepted"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["fileName"] == file_name)
        .unwrap_or_else(|| panic!("{file_name} absent de accepted[] : {report}"))
}

/// Détails JSON de la dernière trace `action` d'une pièce importée.
async fn audit_details(pool: &MySqlPool, entity_id: i64, action: &str) -> Value {
    let raw: Option<Vec<u8>> = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE entity_type = 'imported_supplier_invoice' \
         AND entity_id = ? AND action = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(entity_id)
    .bind(action)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|e| panic!("trace {action} absente pour {entity_id} : {e}"));
    serde_json::from_slice(&raw.expect("détails présents")).expect("json valide")
}

/// Les sept colonnes QR « tout ou rien » d'une ligne de staging, lues en SQL
/// brut (indépendamment de l'entité) : `true` si toutes sont `NULL`.
async fn qr_columns_all_null(pool: &MySqlPool, id: i64) -> bool {
    let n: i64 = sqlx::query_scalar(
        "SELECT (creditor_iban IS NULL) + (is_qr_iban IS NULL) + \
         (creditor_address_type IS NULL) + (creditor_name IS NULL) + \
         (creditor_country IS NULL) + (reference_type IS NULL) + (currency IS NULL) \
         FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(pool)
    .await
    .unwrap();
    n == 7
}

/// Story 30-2a (AC1, AC3) — sens changé : le PNG **sans** QR n'est plus un
/// échec, il entre dans la liste sans données QR. Les deux refus de type
/// (extension hors liste blanche, octets magiques incohérents) restent.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_rejects_unsupported_type_and_accepts_no_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    // (a) Extension hors liste blanche → UNSUPPORTED_FILE_TYPE (check extension).
    std::fs::write(app.inbox.join("notes.txt"), b"pas une image").unwrap();
    // (b) Image PNG valide mais sans QR → ACCEPTÉE, `qr: "none"` (AC1).
    std::fs::write(app.inbox.join("blank.png"), blank_png()).unwrap();
    // (c) Extension permise (.pdf) MAIS magic bytes incohérents (contenu PNG) →
    //     UNSUPPORTED_FILE_TYPE via `magic_matches` (AA1 — chemin magic-bytes testé).
    std::fs::write(app.inbox.join("fake.pdf"), blank_png()).unwrap();

    let body = run_import(&app, &ctx.jwt).await;

    let failed = body["failed"].as_array().unwrap();
    assert_eq!(failed.len(), 2, "{body}");
    assert!(
        failed
            .iter()
            .all(|f| f["errorCode"] == "UNSUPPORTED_FILE_TYPE"),
        "{body}"
    );
    // Le magic-mismatch est bien rattaché au bon fichier (identifiant business).
    let fake = failed
        .iter()
        .find(|f| f["fileName"] == "fake.pdf")
        .expect("fake.pdf dans failed[]");
    assert_eq!(fake["errorCode"], "UNSUPPORTED_FILE_TYPE");
    // Le PNG sans QR est accepté, sans code d'erreur QR.
    let blank = accepted_entry(&body, "blank.png");
    assert_eq!(blank["qr"], "none", "{body}");
    assert!(blank.get("qrError").is_none(), "{body}");
    // Seuls les deux refus sont déplacés dans failed/.
    assert_eq!(
        std::fs::read_dir(app.inbox.join("failed")).unwrap().count(),
        2
    );
}

/// Story 30-2a (AC1) — un PNG sans QR entre en `to_complete`, les sept colonnes
/// QR à `NULL` ; il est archivé et quitte le dossier ; la trace de création a
/// la même forme qu'avec QR, ses clés présentes à `null`, `iban_present: false`
/// (tue M-a1 et M-a15).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_accepts_png_without_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let png = blank_png();
    std::fs::write(app.inbox.join("ticket.png"), &png).unwrap();

    let body = run_import(&app, &ctx.jwt).await;
    assert!(body["failed"].as_array().unwrap().is_empty(), "{body}");
    let entry = accepted_entry(&body, "ticket.png");
    assert_eq!(entry["qr"], "none");
    assert!(entry.get("qrError").is_none(), "{body}");
    let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();

    assert_eq!(staging_status(&pool, id).await, "to_complete");
    assert!(
        qr_columns_all_null(&pool, id).await,
        "sept colonnes QR à NULL"
    );
    let (amount_null, reference_null, line1_null): (bool, bool, bool) = sqlx::query_as(
        "SELECT amount IS NULL, reference_value IS NULL, creditor_line1 IS NULL \
         FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(amount_null && reference_null && line1_null);

    // Archivé sous son empreinte, supprimé du dossier d'entrée.
    use sha2::Digest;
    let archived = app
        .documents
        .join(format!("{:x}.png", sha2::Sha256::digest(&png)));
    assert!(archived.exists(), "document archivé");
    assert!(!app.inbox.join("ticket.png").exists(), "dossier vidé");

    let details = audit_details(&pool, id, "imported_supplier_invoice.created").await;
    assert_eq!(details["qr_present"], json!(false), "{details}");
    assert_eq!(details["iban_present"], json!(false), "{details}");
    for key in [
        "creditor_name",
        "amount",
        "currency",
        "reference_type",
        "is_qr_iban",
    ] {
        assert!(
            details.get(key).is_some_and(Value::is_null),
            "clé {key} présente à null : {details}"
        );
    }
    assert_eq!(details["original_filename"], "ticket.png");
}

/// Story 30-2a (AC1, AC2) — un document avec QR garde exactement son
/// comportement : `qr: "present"`, données stockées, trace `qr_present: true`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_with_qr_reports_present(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(
        app.inbox.join("qr.png"),
        qr_invoice_png(dec!(100.00), "Robert SA"),
    )
    .unwrap();
    let body = run_import(&app, &ctx.jwt).await;
    let entry = accepted_entry(&body, "qr.png");
    assert_eq!(entry["qr"], "present", "{body}");
    assert!(entry.get("qrError").is_none(), "{body}");
    let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();
    let details = audit_details(&pool, id, "imported_supplier_invoice.created").await;
    assert_eq!(details["qr_present"], json!(true), "{details}");
    assert_eq!(details["iban_present"], json!(true), "{details}");
    assert_eq!(details["creditor_name"], "Robert SA");
}

/// Story 30-2a (AC1) — un PDF sans QR entre lui aussi, sans données QR. Selon
/// pdfium, le rapport dit `none` ou `unreadable` (`PDF_RENDER_ERROR`) : le test
/// n'exige que l'acceptation et l'absence de données QR.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_accepts_pdf_without_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(app.inbox.join("lettre.pdf"), PDF_SANS_QR).unwrap();
    let body = run_import(&app, &ctx.jwt).await;
    assert!(body["failed"].as_array().unwrap().is_empty(), "{body}");
    let entry = accepted_entry(&body, "lettre.pdf");
    match entry["qr"].as_str() {
        Some("none") => assert!(entry.get("qrError").is_none(), "{body}"),
        Some("unreadable") => assert_eq!(entry["qrError"], "PDF_RENDER_ERROR", "{body}"),
        other => panic!("qr inattendu {other:?} : {body}"),
    }
    let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();
    assert!(qr_columns_all_null(&pool, id).await);
}

/// Story 30-2a (AC2) — un QR décodé mais inexploitable (payload SPC invalide,
/// IBAN invalide) n'empêche pas l'entrée : `unreadable`, code d'origine en
/// `qrError`, aucune donnée QR stockée (tue M-a2).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_unreadable_qr_is_accepted_without_qr_data(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(
        app.inbox.join("spc.png"),
        qr_invoice_png_with(&[(SPC_LINE_EPD, "XXX")]),
    )
    .unwrap();
    std::fs::write(
        app.inbox.join("iban.png"),
        qr_invoice_png_with(&[(SPC_LINE_IBAN, "CH0000000000000000000")]),
    )
    .unwrap();
    let body = run_import(&app, &ctx.jwt).await;
    assert!(body["failed"].as_array().unwrap().is_empty(), "{body}");
    for (file, code) in [
        ("spc.png", "INVALID_SPC_PAYLOAD"),
        ("iban.png", "INVALID_IBAN"),
    ] {
        let entry = accepted_entry(&body, file);
        assert_eq!(entry["qr"], "unreadable", "{body}");
        assert_eq!(entry["qrError"], code, "{body}");
        let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();
        assert!(qr_columns_all_null(&pool, id).await, "{file}");
    }
}

/// Story 30-2a (AC2) — longueurs comptées en **caractères** : un nom accentué
/// de 70 caractères entre avec ses données, nom stocké égal (tue M-a12) ; de 71,
/// il entre sans (`FIELD_TOO_LONG`, tue M-a4) ; un pays de trois lettres aussi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_field_too_long_is_accepted_without_qr_data(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let name70 = "é".repeat(70);
    let name71 = "é".repeat(71);
    std::fs::write(
        app.inbox.join("n70.png"),
        qr_invoice_png(dec!(20.00), &name70),
    )
    .unwrap();
    std::fs::write(
        app.inbox.join("n71.png"),
        qr_invoice_png(dec!(21.00), &name71),
    )
    .unwrap();
    std::fs::write(
        app.inbox.join("pays.png"),
        qr_invoice_png_with(&[(SPC_LINE_COUNTRY, "CHE")]),
    )
    .unwrap();
    std::fs::write(
        app.inbox.join("devise.png"),
        qr_invoice_png_with(&[(SPC_LINE_CURRENCY, "CHFX")]),
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
    assert!(body["failed"].as_array().unwrap().is_empty(), "{body}");

    let n70 = accepted_entry(&body, "n70.png");
    assert_eq!(n70["qr"], "present", "{body}");
    let id70 = n70["importedSupplierInvoiceId"].as_i64().unwrap();
    let stored: Option<String> =
        sqlx::query_scalar("SELECT creditor_name FROM imported_supplier_invoices WHERE id = ?")
            .bind(id70)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(stored.as_deref(), Some(name70.as_str()), "nom stocké égal");

    for file in ["n71.png", "pays.png", "devise.png"] {
        let entry = accepted_entry(&body, file);
        assert_eq!(entry["qr"], "unreadable", "{file} : {body}");
        assert_eq!(entry["qrError"], "FIELD_TOO_LONG", "{file} : {body}");
        let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();
        assert!(qr_columns_all_null(&pool, id).await, "{file}");
    }
}

/// Story 30-2a (AC2) — le montant du QR est borné par le contrôle pur :
/// `amount_min..=amount_max`, deux décimales significatives. Hors bornes, le
/// document entre sans ses données (`QR_AMOUNT_OUT_OF_RANGE`, tue M-a11 par ses
/// deux volets) ; `20.000` et un montant vide gardent leurs données.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_amount_out_of_range_is_accepted_without_qr_data(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let cases_out = [
        ("seize.png", "1234567890123456.00"),
        ("negatif.png", "-5.00"),
        ("nul.png", "0.00"),
        ("decimales.png", "20.005"),
    ];
    for (file, amount) in cases_out {
        std::fs::write(
            app.inbox.join(file),
            qr_invoice_png_with(&[(SPC_LINE_AMOUNT, amount), (SPC_LINE_NAME, file)]),
        )
        .unwrap();
    }
    std::fs::write(
        app.inbox.join("trois-zeros.png"),
        qr_invoice_png_with(&[(SPC_LINE_AMOUNT, "20.000")]),
    )
    .unwrap();
    std::fs::write(
        app.inbox.join("ouvert.png"),
        qr_invoice_png_with(&[(SPC_LINE_AMOUNT, ""), (SPC_LINE_NAME, "Ouvert SA")]),
    )
    .unwrap();

    let body = run_import(&app, &ctx.jwt).await;
    assert!(body["failed"].as_array().unwrap().is_empty(), "{body}");
    for (file, _) in cases_out {
        let entry = accepted_entry(&body, file);
        assert_eq!(entry["qr"], "unreadable", "{file} : {body}");
        assert_eq!(
            entry["qrError"], "QR_AMOUNT_OUT_OF_RANGE",
            "{file} : {body}"
        );
        let id = entry["importedSupplierInvoiceId"].as_i64().unwrap();
        assert!(qr_columns_all_null(&pool, id).await, "{file}");
    }
    let zeros = accepted_entry(&body, "trois-zeros.png");
    assert_eq!(zeros["qr"], "present", "{body}");
    let open = accepted_entry(&body, "ouvert.png");
    assert_eq!(open["qr"], "present", "{body}");
    let open_id = open["importedSupplierInvoiceId"].as_i64().unwrap();
    let (amount_null, name): (bool, Option<String>) = sqlx::query_as(
        "SELECT amount IS NULL, creditor_name FROM imported_supplier_invoices WHERE id = ?",
    )
    .bind(open_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(amount_null, "montant ouvert : NULL, données gardées");
    assert_eq!(name.as_deref(), Some("Ouvert SA"));
}

/// Story 30-2a (prérequis des copies E2E) — un PNG à QR suffixé d'octets
/// après son bloc `IEND` reste décodé : les copies uniques des parcours E2E
/// reposent sur ce suffixe.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_png_with_suffix_after_iend_still_decoded(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let mut png = qr_invoice_png(dec!(30.00), "Suffixe SA");
    png.extend_from_slice(b"kesh-e2e-copie-0123456789abcdef");
    std::fs::write(app.inbox.join("suffixe.png"), &png).unwrap();
    let body = run_import(&app, &ctx.jwt).await;
    let entry = accepted_entry(&body, "suffixe.png");
    assert_eq!(entry["qr"], "present", "{body}");
}

/// Story 30-2a (AC3) — une image aux octets magiques valides mais au corps
/// tronqué reste refusée (`UNSUPPORTED_FILE_TYPE`, dans `failed/`) : tue M-a3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_corrupt_png_is_still_rejected(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let png = blank_png();
    std::fs::write(app.inbox.join("casse.png"), &png[..png.len() / 2]).unwrap();
    let body = run_import(&app, &ctx.jwt).await;
    assert!(body["accepted"].as_array().unwrap().is_empty(), "{body}");
    assert_eq!(body["failed"][0]["fileName"], "casse.png", "{body}");
    assert_eq!(body["failed"][0]["errorCode"], "UNSUPPORTED_FILE_TYPE");
    assert_eq!(
        std::fs::read_dir(app.inbox.join("failed")).unwrap().count(),
        1
    );
}

/// Story 30-2a (AC3) — un document sans QR déposé deux fois est un doublon ;
/// retiré puis redéposé, il est réactivé avec `qr: "none"`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_no_qr_duplicate_and_reactivation(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let png = blank_png();
    std::fs::write(app.inbox.join("a.png"), &png).unwrap();
    let b1 = run_import(&app, &ctx.jwt).await;
    let id = accepted_entry(&b1, "a.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();

    std::fs::write(app.inbox.join("b.png"), &png).unwrap();
    let b2 = run_import(&app, &ctx.jwt).await;
    assert_eq!(b2["failed"][0]["fileName"], "b.png", "{b2}");
    assert_eq!(b2["failed"][0]["errorCode"], "DUPLICATE", "{b2}");

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{id}/discard")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);

    std::fs::write(app.inbox.join("c.png"), &png).unwrap();
    let b3 = run_import(&app, &ctx.jwt).await;
    let entry = accepted_entry(&b3, "c.png");
    assert_eq!(entry["importedSupplierInvoiceId"].as_i64(), Some(id));
    assert_eq!(entry["qr"], "none", "{b3}");
    assert_eq!(staging_status(&pool, id).await, "to_complete");
}

/// Story 30-2a (AC2) — la réactivation d'un document avec QR dit `present`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn import_reactivation_with_qr_reports_present(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let png = qr_invoice_png(dec!(40.00), "Reactive SA");
    std::fs::write(app.inbox.join("r1.png"), &png).unwrap();
    let b1 = run_import(&app, &ctx.jwt).await;
    let id = accepted_entry(&b1, "r1.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();
    sqlx::query("UPDATE imported_supplier_invoices SET status='discarded' WHERE id=?")
        .bind(id)
        .execute(&pool)
        .await
        .unwrap();
    std::fs::write(app.inbox.join("r2.png"), &png).unwrap();
    let b2 = run_import(&app, &ctx.jwt).await;
    let entry = accepted_entry(&b2, "r2.png");
    assert_eq!(entry["qr"], "present", "{b2}");
}

/// Story 30-2a (AC4, AC5) — la liste rend les champs QR d'un document sans QR
/// à `null`, sans 500.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn list_returns_nulls_for_document_without_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(app.inbox.join("liste.png"), blank_png()).unwrap();
    run_import(&app, &ctx.jwt).await;
    let resp = app
        .client
        .get(app.url("/api/v1/imported-supplier-invoices?status=to_complete"))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let items: Value = resp.json().await.unwrap();
    let item = items
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["originalFilename"] == "liste.png")
        .unwrap_or_else(|| panic!("liste.png absent : {items}"));
    for key in [
        "creditorIban",
        "isQrIban",
        "creditorAddressType",
        "creditorName",
        "creditorCountry",
        "referenceType",
        "currency",
        "amount",
    ] {
        assert!(item[key].is_null(), "{key} à null : {item}");
    }
}

/// Story 30-2a (AC7) — un document sans QR se complète : ni contrôle de devise
/// (tue M-a7), ni routage IBAN, ni contrôle de montant ; sans coordonnées dans
/// le corps, facture sans IBAN, ni référence, ni montant attendu (la 30-2b les
/// prend dans le corps : `complete_without_qr_takes_payment_details_and_project_from_body`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complete_document_without_qr(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(app.inbox.join("sansqr.png"), blank_png()).unwrap();
    let report = run_import(&app, &ctx.jwt).await;
    let id = accepted_entry(&report, "sansqr.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();

    let resp = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{id}/complete")))
        .bearer_auth(&ctx.jwt)
        .json(&complete_body(
            ctx.supplier_id,
            ctx.seeded.accounts["4000"],
            dec!(1),
            dec!(73.40),
            dec!(0),
        ))
        .send()
        .await
        .unwrap();
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 200, "{body}");
    assert!(body["creditorIban"].is_null(), "{body}");
    assert!(body["creditorQrIban"].is_null(), "{body}");
    assert!(body["paymentReference"].is_null(), "{body}");
    assert!(body["expectedPaymentAmount"].is_null(), "{body}");
    assert_eq!(body["totalAmount"], "73.4000");
    assert_eq!(staging_status(&pool, id).await, "completed");
}

/// Story 30-2a (AC7) — le retrait d'un document sans QR trace
/// `iban_present: false` (le test historique ne vérifiait que la présence de
/// la clé) : tue M-a6.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn discard_document_without_qr_traces_iban_absent(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    std::fs::write(app.inbox.join("retrait.png"), blank_png()).unwrap();
    let report = run_import(&app, &ctx.jwt).await;
    let id = accepted_entry(&report, "retrait.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{id}/discard")))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let details = audit_details(&pool, id, "imported_supplier_invoice.discarded").await;
    assert_eq!(details["iban_present"], json!(false), "{details}");
    assert!(details["creditor_name"].is_null(), "{details}");
    assert_eq!(details["original_filename"], "retrait.png");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

    // ⛔ Story 25-1b (AC 3) — LE CHEMIN LE PLUS SÉVÈRE DE LA STORY, et il n'avait
    // aucun test avant la passe 1 de revue de code.
    //
    // La réactivation rend `Accepted` SANS rien créer. Sans sa propre trace, la
    // piste garderait `discarded` pour DERNIÈRE trace d'une pièce redevenue
    // `to_complete` : elle dirait le contraire de la base. C'est cet ordre-là
    // qu'on vérifie, pas seulement la présence des actions.
    let actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE entity_type = 'imported_supplier_invoice' \
         AND entity_id = ? ORDER BY id",
    )
    .bind(staging_id)
    .fetch_all(&pool)
    .await
    .expect("lecture des actions");
    assert_eq!(
        actions,
        vec![
            "imported_supplier_invoice.created".to_string(),
            "imported_supplier_invoice.reactivated".to_string(),
        ],
        "l'entrée initiale puis la réactivation — et la DERNIÈRE trace doit \
         concorder avec le statut `to_complete`, non le démentir. Le doublon \
         rejeté entre les deux n'écrit rien : trois imports, deux traces."
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

    // Story 25-1b (AC 3) — la transition de la PIÈCE porte sa propre trace.
    // ⚠️ Son absence était une ASYMÉTRIE : le cycle de vie disait
    // `created → discarded` quand une pièce est rejetée, et `created → rien`
    // quand elle aboutit. Deux traces s'écrivent ici, dans la même transaction :
    // celle de la facture créée et celle de la pièce.
    let actions: Vec<String> = sqlx::query_scalar(
        "SELECT action FROM audit_log WHERE entity_type = 'imported_supplier_invoice' \
         AND entity_id = ? ORDER BY id",
    )
    .bind(staging_id)
    .fetch_all(&pool)
    .await
    .expect("lecture des actions");
    assert_eq!(
        actions,
        vec!["imported_supplier_invoice.completed".to_string()],
        "la pièce a été semée directement en base, d'où l'absence de `.created`"
    );

    // La facture fournisseur créée est tracée de son côté — la trace de la
    // pièce ne la remplace pas, elle la complète.
    let invoice_traces: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_log WHERE action = 'supplier_invoice.created'",
    )
    .fetch_one(&pool)
    .await
    .expect("comptage");
    assert_eq!(invoice_traces, 1, "les deux traces coexistent");
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

    // Id inexistant → 404 IMPORTED_INVOICE_NOT_FOUND (code distinct du download,
    // code-review 12-5c EC1/BH2/AA4).
    let r404 = app
        .client
        .post(app.url("/api/v1/imported-supplier-invoices/999999/complete"))
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
    assert_eq!(r404.status(), 404);
    assert_eq!(
        r404.json::<Value>().await.unwrap()["error"]["code"],
        "IMPORTED_INVOICE_NOT_FOUND"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

    // Story 25-1b (AC 3, 10) — le rejet laisse une trace, et l'IBAN n'y figure
    // JAMAIS : seule sa présence est journalisée (convention du dépôt).
    let row: (String, Option<Vec<u8>>) = sqlx::query_as(
        "SELECT action, details_json FROM audit_log          WHERE entity_type = 'imported_supplier_invoice' AND entity_id = ? ORDER BY id DESC LIMIT 1",
    )
    .bind(staging_id)
    .fetch_one(&pool)
    .await
    .expect("une trace de rejet");
    assert_eq!(row.0, "imported_supplier_invoice.discarded");
    let details: serde_json::Value =
        serde_json::from_slice(&row.1.expect("détails présents")).expect("json valide");
    assert!(
        details.get("iban_present").is_some(),
        "la présence de l'IBAN est journalisée"
    );
    assert!(
        details.get("creditor_iban").is_none(),
        "⛔ l'IBAN lui-même ne doit JAMAIS figurer dans la piste"
    );
    // Le détail nomme la pièce : une trace qui ne dit pas ce qui a été rejeté
    // ne répond à personne.
    assert!(details.get("original_filename").is_some());
}

// ============================================================
// C. Liste (validation status D1) + download
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

#[sqlx::test(migrations = "../kesh-db/test-schema")]
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

// ============================================================
// Story 15-5d (#429) — la garde À L'USAGE à la complétion
// ============================================================

/// La complétion d'une facture importée crée la facture fournisseur par le
/// même chemin que la saisie (`supplier_invoices::create_in_tx`) : un compte
/// créanciers désigné **devenu non imputable** la refuse en 400
/// `ACCOUNT_NOT_POSTABLE` ; rollback total — le staging reste `to_complete`,
/// aucune facture fournisseur n'est créée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complete_refuses_a_designated_payable_made_non_postable(pool: MySqlPool) {
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
    // Le compte créanciers, désigné par `setup`, puis rendu non imputable.
    let payable = ctx.seeded.accounts["2000"];
    sqlx::query("UPDATE accounts SET postable = FALSE WHERE id = ?")
        .bind(payable)
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
    let status = resp.status();
    let body: Value = resp.json().await.unwrap();
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "ACCOUNT_NOT_POSTABLE", "{body}");
    assert_eq!(
        body["error"]["details"]["rejected"],
        json!([{ "accountId": payable, "accountNumber": "2000" }]),
        "{body}"
    );
    assert_eq!(staging_status(&pool, staging_id).await, "to_complete");
    assert_eq!(
        supplier_invoice_count(&pool, ctx.seeded.company_id).await,
        0
    );
}

// ============================================================
// C. Story 30-2b (#685) — un seul corps, coordonnées de paiement validées,
//    fournisseurs proposés par IBAN
// ============================================================

/// IBAN suisse ordinaire, synthétique et valide (MOD-97).
const IBAN_CH: &str = "CH9300762011623852957";
/// QR-IBAN synthétique valide (QR-IID 31999).
const QR_IBAN_CH: &str = "CH4431999123000889012";
/// IBAN allemand valide : un IBAN étranger d'un pays de la table reste accepté.
const IBAN_DE: &str = "DE89370400440532013000";
/// QRR valide (somme de contrôle modulo 10 récursive).
const QRR_OK: &str = "210000000003139471430009017";

/// Corps de `POST /supplier-invoices` (ou de la complétion) : `complete_body`
/// complété des champs de paiement donnés.
fn body_with(ctx: &Ctx, extra: Value) -> Value {
    let mut body = complete_body(
        ctx.supplier_id,
        ctx.seeded.accounts["4000"],
        dec!(1),
        dec!(100.00),
        dec!(0),
    );
    for (k, v) in extra.as_object().unwrap() {
        body[k] = v.clone();
    }
    body
}

async fn post_supplier_invoice(app: &TestApp, jwt: &str, body: &Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url("/api/v1/supplier-invoices"))
        .bearer_auth(jwt)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

async fn post_complete(app: &TestApp, jwt: &str, id: i64, body: &Value) -> (u16, Value) {
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/imported-supplier-invoices/{id}/complete")))
        .bearer_auth(jwt)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

/// AC12-bis — un cas par motif de refus, sur `POST /supplier-invoices` ; aucun
/// ne crée de facture. Tue M-b10 (validation retirée de la conversion commune).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_refuses_each_invalid_payment_detail(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let ref_65 = "R".repeat(65);
    let cases: Vec<(Value, &str, &str)> = vec![
        (
            json!({ "creditorIban": "CH9300762011623852958" }),
            "creditorIban",
            "INVALID_IBAN",
        ),
        (
            json!({ "creditorIban": QR_IBAN_CH }),
            "creditorIban",
            "QR_IBAN_IN_IBAN_FIELD",
        ),
        (
            json!({ "creditorQrIban": IBAN_CH, "paymentReference": QRR_OK }),
            "creditorQrIban",
            "NOT_A_QR_IBAN",
        ),
        (
            json!({ "creditorQrIban": "CH00 PAS UN IBAN", "paymentReference": QRR_OK }),
            "creditorQrIban",
            "INVALID_IBAN",
        ),
        (
            json!({ "creditorIban": IBAN_CH, "creditorQrIban": QR_IBAN_CH, "paymentReference": QRR_OK }),
            "creditorIban",
            "BOTH_IBANS",
        ),
        (
            json!({ "creditorQrIban": QR_IBAN_CH }),
            "paymentReference",
            "QRR_REQUIRED",
        ),
        // Revue P1, E3 : une référence BLANCHE avec un QR-IBAN vaut absente — et non
        // une QRR invalide (tue M-b23 : `reference` lu brut, sans `present()`).
        (
            json!({ "creditorQrIban": QR_IBAN_CH, "paymentReference": " \u{00A0}\u{200B}" }),
            "paymentReference",
            "QRR_REQUIRED",
        ),
        (
            json!({ "creditorQrIban": QR_IBAN_CH, "paymentReference": "210000000003139471430009018" }),
            "paymentReference",
            "INVALID_QRR",
        ),
        (
            json!({ "creditorIban": IBAN_CH, "paymentReference": ref_65 }),
            "paymentReference",
            "REFERENCE_TOO_LONG",
        ),
        // Revue P1, E2 : la limite se compte en CARACTÈRES — 65 « é » (130 octets).
        (
            json!({ "creditorIban": IBAN_CH, "paymentReference": "é".repeat(65) }),
            "paymentReference",
            "REFERENCE_TOO_LONG",
        ),
    ];
    for (extra, field, reason) in cases {
        let (status, body) =
            post_supplier_invoice(&app, &ctx.jwt, &body_with(&ctx, extra.clone())).await;
        assert_eq!(status, 400, "{extra} → {body}");
        assert_eq!(
            body["error"]["code"], "INVALID_PAYMENT_DETAILS",
            "{extra} → {body}"
        );
        assert_eq!(body["error"]["details"]["field"], field, "{extra} → {body}");
        assert_eq!(
            body["error"]["details"]["reason"], reason,
            "{extra} → {body}"
        );
    }
    assert_eq!(
        supplier_invoice_count(&pool, ctx.seeded.company_id).await,
        0,
        "aucun refus ne crée de facture"
    );
}

/// AC12-bis — ce qui reste accepté, et la forme enregistrée : IBAN étranger,
/// blancs Unicode retirés (la même liste qu'au Vitest de `iban.ts`), référence
/// libre et ancienne référence de 27 chiffres avec un IBAN ordinaire, QRR en
/// groupes de cinq avec un QR-IBAN, chaîne vide = absente.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn create_accepts_and_normalizes_payment_details(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;

    // IBAN allemand, saisi en groupes de quatre.
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": "DE89 3704 0044 0532 0130 00" }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["creditorIban"], IBAN_DE);

    // IBAN suisse truffé des sept blancs de l'ensemble écrit (AC12-bis) :
    // U+0020, U+0009, U+00A0, U+202F, U+0085, U+200B, U+FEFF.
    let blanc = "CH93\u{0020}0076\u{0009}2011\u{00A0}6238\u{202F}5295\u{0085}7\u{200B}\u{FEFF}";
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": blanc, "paymentReference": "Facture E2E" }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["creditorIban"], IBAN_CH, "blancs retirés");
    assert_eq!(
        b["paymentReference"], "Facture E2E",
        "référence libre gardée"
    );

    // Ancienne référence BVR/ESR (27 chiffres) avec un IBAN ordinaire : acceptée.
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": IBAN_CH, "paymentReference": QRR_OK }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["paymentReference"], QRR_OK);
    assert!(b["creditorQrIban"].is_null());

    // Référence libre rognée, 64 caractères tout juste.
    let ref_64 = "R".repeat(64);
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": IBAN_CH, "paymentReference": format!("  {ref_64}  ") }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["paymentReference"], ref_64);

    // Revue P1, E2 : 64 caractères non ASCII (128 octets) sont acceptés — la limite
    // se compte en caractères, comme la colonne `VARCHAR(64)` (tue M-b21 : `.len()`).
    let ref_64_e = "é".repeat(64);
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": IBAN_CH, "paymentReference": ref_64_e }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["paymentReference"], ref_64_e);

    // Revue P1, B3 : 64 caractères visibles suivis d'un U+200B et précédés d'un
    // U+FEFF (copier-coller d'un PDF) — rognés comme les autres blancs, ni stockés
    // ni comptés (tue M-b22 : rognage `trim()` seul).
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": IBAN_CH, "paymentReference": format!("\u{FEFF}{ref_64}\u{200B}") }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["paymentReference"], ref_64);

    // QR-IBAN + QRR en groupes de cinq : stockée sans blancs.
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorQrIban": "CH44 3199 9123 0008 8901 2", "paymentReference": "21 00000 00003 13947 14300 09017" }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["creditorQrIban"], QR_IBAN_CH);
    assert!(b["creditorIban"].is_null());
    assert_eq!(b["paymentReference"], QRR_OK);

    // Chaînes vides ou blanches = absentes.
    let (s, b) = post_supplier_invoice(
        &app,
        &ctx.jwt,
        &body_with(
            &ctx,
            json!({ "creditorIban": "", "creditorQrIban": " \u{00A0}", "paymentReference": "" }),
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert!(b["creditorIban"].is_null(), "{b}");
    assert!(b["creditorQrIban"].is_null(), "{b}");
    assert!(b["paymentReference"].is_null(), "{b}");
}

/// AC12, AC13 — la complétion d'un document sans QR prend IBAN, référence,
/// montant attendu et projet dans le corps, validés et normalisés par la
/// conversion commune ; la réponse est relue par la fonction du GET (ses champs
/// calculés ne sont pas `null`). Tue M-b3 (projet ignoré).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complete_without_qr_takes_payment_details_and_project_from_body(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    let project_id: i64 = sqlx::query(
        "INSERT INTO projects (company_id, parent_id, code, name, archived) VALUES (?, NULL, 'P302B', 'Projet 30-2b', FALSE)",
    )
    .bind(ctx.seeded.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    std::fs::write(app.inbox.join("sansqr.png"), blank_png()).unwrap();
    let report = run_import(&app, &ctx.jwt).await;
    let id = accepted_entry(&report, "sansqr.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();

    // Un IBAN faux est refusé, et le document reste à traiter.
    let (s, b) = post_complete(
        &app,
        &ctx.jwt,
        id,
        &body_with(&ctx, json!({ "creditorIban": "CH9300762011623852958" })),
    )
    .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], "INVALID_PAYMENT_DETAILS");
    assert_eq!(b["error"]["details"]["reason"], "INVALID_IBAN");
    assert_eq!(staging_status(&pool, id).await, "to_complete");

    let (s, b) = post_complete(
        &app,
        &ctx.jwt,
        id,
        &body_with(
            &ctx,
            json!({
                "creditorIban": "CH93 0076 2011 6238 5295 7",
                "paymentReference": " Facture 4711 ",
                "expectedPaymentAmount": "100.00",
                "projectId": project_id,
            }),
        ),
    )
    .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["creditorIban"], IBAN_CH);
    assert_eq!(b["paymentReference"], "Facture 4711");
    assert_eq!(b["expectedPaymentAmount"], "100.0000");
    assert_eq!(b["projectId"], project_id, "projet honoré à la complétion");
    assert_eq!(
        b["cancellable"], true,
        "réponse relue par la fonction du GET : {b}"
    );
    assert_eq!(staging_status(&pool, id).await, "completed");
}

/// AC11 — avec QR, le corps ne porte aucune coordonnée de paiement : `400
/// QR_FIELDS_FROM_DOCUMENT` pour chacun des quatre champs, le document reste à
/// traiter ; une chaîne vide vaut absente, et un client actuel (sans ces
/// champs) reste accepté, les valeurs venant du document. Tue M-b1.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complete_with_qr_refuses_payment_fields_in_body(pool: MySqlPool) {
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
    for extra in [
        json!({ "creditorIban": IBAN_DE }),
        json!({ "creditorQrIban": QR_IBAN_CH }),
        json!({ "paymentReference": "Autre" }),
        json!({ "expectedPaymentAmount": "100.00" }),
    ] {
        let (s, b) =
            post_complete(&app, &ctx.jwt, staging_id, &body_with(&ctx, extra.clone())).await;
        assert_eq!(s, 400, "{extra} → {b}");
        assert_eq!(
            b["error"]["code"], "QR_FIELDS_FROM_DOCUMENT",
            "{extra} → {b}"
        );
        assert_eq!(staging_status(&pool, staging_id).await, "to_complete");
    }

    let (s, b) = post_complete(
        &app,
        &ctx.jwt,
        staging_id,
        &body_with(&ctx, json!({ "creditorIban": "", "paymentReference": " " })),
    )
    .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["creditorIban"], IBAN_CH, "l'IBAN vient du document");
    assert_eq!(b["expectedPaymentAmount"], "100.0000");
}

// --- AC14 : fournisseurs proposés par IBAN ---

async fn make_contact(pool: &MySqlPool, ctx: &Ctx, name: &str) -> i64 {
    contacts::create(
        pool,
        ctx.seeded.admin_user_id,
        NewContact {
            company_id: ctx.seeded.company_id,
            contact_type: ContactType::Entreprise,
            name: name.into(),
            first_name: None,
            last_name: None,
            is_client: false,
            is_supplier: true,
            address: None,
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            client_number: None,
            default_payment_terms: None,
            default_payment_terms_days: None,
            language: None,
            salutation: kesh_db::entities::contact::Salutation::Neutre,
        },
    )
    .await
    .expect("create contact")
    .id
}

/// Une facture fournisseur du contact, à la date donnée, portant `IBAN_CH`
/// (forme normalisée : la validation l'exige), réécrite ensuite en SQL brut si
/// `stored` est donné (une facture antérieure à la validation peut porter des
/// espaces).
async fn invoice_with_iban(
    app: &TestApp,
    pool: &MySqlPool,
    ctx: &Ctx,
    contact_id: i64,
    date: &str,
    stored: Option<(&str, &str)>,
) -> i64 {
    let mut body = body_with(ctx, json!({ "creditorIban": IBAN_CH, "invoiceDate": date }));
    body["contactId"] = json!(contact_id);
    let (s, b) = post_supplier_invoice(app, &ctx.jwt, &body).await;
    assert_eq!(s, 201, "{b}");
    let id = b["id"].as_i64().unwrap();
    if let Some((column, value)) = stored {
        sqlx::query(&format!(
            "UPDATE supplier_invoices SET creditor_iban = NULL, {column} = ? WHERE id = ?"
        ))
        .bind(value)
        .bind(id)
        .execute(pool)
        .await
        .unwrap();
    }
    id
}

async fn get_candidates(app: &TestApp, jwt: &str, id: i64) -> (u16, Value) {
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/imported-supplier-invoices/{id}/supplier-candidates"
        )))
        .bearer_auth(jwt)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap())
}

/// AC14 — `supplier-candidates` : IBAN lu sur le document, comparé normalisé
/// (espaces et insécables en base retrouvés), dans `creditor_iban` comme dans
/// `creditor_qr_iban` ; factures annulées, contacts archivés ou non
/// fournisseurs, autre société exclus ; ordre `MAX(invoice_date) DESC` puis
/// `contacts.id` ; trois au plus ; vide sans QR ; 404 inconnu ou autre société ;
/// l'IBAN ne sort pas dans la réponse. Tue M-b13 côté serveur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn supplier_candidates_by_iban(pool: MySqlPool) {
    let ctx = setup(&pool).await;
    let app = spawn_app(pool.clone(), 25 * 1024 * 1024).await;
    // Document avec QR (IBAN_CH, cf. `seed_staging`).
    let doc = seed_staging(
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

    let a = make_contact(&pool, &ctx, "Alpha espaces").await;
    let g = make_contact(&pool, &ctx, "Gamma récent").await;
    let b = make_contact(&pool, &ctx, "Beta QR").await;
    let h = make_contact(&pool, &ctx, "Hors plafond").await;
    let annulee = make_contact(&pool, &ctx, "Annulée").await;
    let archive = make_contact(&pool, &ctx, "Archivé").await;
    let client = make_contact(&pool, &ctx, "Plus fournisseur").await;
    let ailleurs = make_contact(&pool, &ctx, "Facture ailleurs").await;
    let sans = make_contact(&pool, &ctx, "Autre IBAN").await;

    invoice_with_iban(
        &app,
        &pool,
        &ctx,
        a,
        "2026-05-01",
        Some(("creditor_iban", "ch93 0076\u{00A0}2011\u{202F}6238 5295 7")),
    )
    .await;
    invoice_with_iban(&app, &pool, &ctx, g, "2026-05-01", None).await;
    invoice_with_iban(
        &app,
        &pool,
        &ctx,
        b,
        "2026-04-01",
        Some(("creditor_qr_iban", IBAN_CH)),
    )
    .await;
    invoice_with_iban(&app, &pool, &ctx, h, "2026-02-01", None).await;
    let inv_annulee = invoice_with_iban(&app, &pool, &ctx, annulee, "2026-06-01", None).await;
    sqlx::query("UPDATE supplier_invoices SET status = 'cancelled' WHERE id = ?")
        .bind(inv_annulee)
        .execute(&pool)
        .await
        .unwrap();
    invoice_with_iban(&app, &pool, &ctx, archive, "2026-06-01", None).await;
    sqlx::query("UPDATE contacts SET active = FALSE WHERE id = ?")
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    invoice_with_iban(&app, &pool, &ctx, client, "2026-06-01", None).await;
    sqlx::query("UPDATE contacts SET is_supplier = FALSE, is_client = TRUE WHERE id = ?")
        .bind(client)
        .execute(&pool)
        .await
        .unwrap();
    let inv_ailleurs = invoice_with_iban(&app, &pool, &ctx, ailleurs, "2026-06-01", None).await;
    let other_company = companies::create(
        &pool,
        NewCompany {
            name: "Ailleurs SA".into(),
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
    sqlx::query("UPDATE supplier_invoices SET company_id = ? WHERE id = ?")
        .bind(other_company)
        .bind(inv_ailleurs)
        .execute(&pool)
        .await
        .unwrap();
    // Revue P1, E4 — la portée société côté CONTACT : une facture de la société
    // courante, la plus récente de toutes et au bon IBAN, mais dont le contact
    // appartient à une autre société (état incohérent posé en SQL). Il ne sort pas
    // (tue M-b24 : `contacts.company_id = ?` retiré).
    let contact_ailleurs = make_contact(&pool, &ctx, "Contact ailleurs").await;
    invoice_with_iban(&app, &pool, &ctx, contact_ailleurs, "2026-07-01", None).await;
    sqlx::query("UPDATE contacts SET company_id = ? WHERE id = ?")
        .bind(other_company)
        .bind(contact_ailleurs)
        .execute(&pool)
        .await
        .unwrap();
    let mut autre = body_with(
        &ctx,
        json!({ "creditorIban": IBAN_DE, "invoiceDate": "2026-06-01" }),
    );
    autre["contactId"] = json!(sans);
    assert_eq!(post_supplier_invoice(&app, &ctx.jwt, &autre).await.0, 201);

    let (s, body) = get_candidates(&app, &ctx.jwt, doc).await;
    assert_eq!(s, 200, "{body}");
    let ids: Vec<i64> = body["candidates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_i64().unwrap())
        .collect();
    assert_eq!(ids, vec![a, g, b], "ordre, filtres et plafond : {body}");
    assert!(
        !body.to_string().contains("CH93") && !body.to_string().contains("ch93"),
        "l'IBAN ne sort pas dans la réponse : {body}"
    );

    // Document sans QR : liste vide.
    std::fs::write(app.inbox.join("sansqr.png"), blank_png()).unwrap();
    let report = run_import(&app, &ctx.jwt).await;
    let sans_qr = accepted_entry(&report, "sansqr.png")["importedSupplierInvoiceId"]
        .as_i64()
        .unwrap();
    let (s, body) = get_candidates(&app, &ctx.jwt, sans_qr).await;
    assert_eq!(s, 200, "{body}");
    assert_eq!(body["candidates"], json!([]));

    // Inconnu, puis autre société : le même 404.
    let (s, body) = get_candidates(&app, &ctx.jwt, doc + 1000).await;
    assert_eq!(s, 404, "{body}");
    assert_eq!(body["error"]["code"], "IMPORTED_INVOICE_NOT_FOUND");
    let other_jwt = other_company_jwt(&pool).await;
    let (s, body) = get_candidates(&app, &other_jwt, doc).await;
    assert_eq!(s, 404, "{body}");
    assert_eq!(body["error"]["code"], "IMPORTED_INVOICE_NOT_FOUND");
}
