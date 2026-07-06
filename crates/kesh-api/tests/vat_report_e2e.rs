//! Tests E2E HTTP Story 11-2 — Rapport TVA (TVA due / vente).
//!
//! Couvre : GET /api/v1/reports/vat (JSON) + /export?format=pdf|csv.
//! - agrégation par taux, factures `validated` uniquement (draft/cancelled
//!   exclus, hors période exclu),
//! - arrondi par ligne (FR55),
//! - ligne 0 %/exonérée présente,
//! - export PDF/CSV (content-type + content-disposition),
//! - format invalide → 400, période hors exercice → 400,
//! - anti-IDOR cross-tenant.
//!
//! Pré-requis : MariaDB (sqlx::test crée une DB éphémère par test).
//! Harness hérité de `invoice_echeancier_e2e.rs`.

use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta};
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::address::StructuredAddress;
use kesh_db::entities::company::NewCompany;
use kesh_db::entities::contact::{ContactType, NewContact};
use kesh_db::entities::invoice::{NewInvoice, NewInvoiceLine};
use kesh_db::entities::{Language, OrgType};
use kesh_db::repositories::{companies, contacts, invoices};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "admin123";

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
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .unwrap()
                .join("kesh-i18n/locales")
                .as_path(),
        )
        .expect("load test i18n"),
    );
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);

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

async fn login(app: &TestApp) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({ "username": "admin", "password": TEST_ADMIN_PASSWORD }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

async fn seed_base(pool: &MySqlPool) -> (i64, i64, i64) {
    let seeded = seed_accounting_company(pool)
        .await
        .expect("seed_accounting_company");
    (
        seeded.admin_user_id,
        seeded.company_id,
        seeded.fiscal_year_id,
    )
}

async fn seed_contact(pool: &MySqlPool, company_id: i64, admin_id: i64, name: &str) -> i64 {
    contacts::create(
        pool,
        admin_id,
        NewContact {
            company_id,
            contact_type: ContactType::Personne,
            name: name.into(),
            first_name: None,
            last_name: None,
            is_client: true,
            is_supplier: false,
            address: Some("Rue 1\n1000 Lausanne".into()),
            address_street: None,
            address_building: None,
            address_postal_code: None,
            address_city: None,
            address_country: None,
            email: None,
            phone: None,
            ide_number: None,
            default_payment_terms: None,
        },
    )
    .await
    .unwrap()
    .id
}

/// Crée une facture (lignes `(vat_rate, unit_price)` à quantité 1) puis la valide
/// via le flow normal `validate_invoice`.
async fn create_validated_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    admin_id: i64,
    date: NaiveDate,
    lines: &[(Decimal, Decimal)],
) -> i64 {
    let new = NewInvoice {
        company_id,
        contact_id,
        date,
        due_date: Some(date),
        payment_terms: None,
        lines: lines
            .iter()
            .map(|(rate, price)| NewInvoiceLine {
                description: "Ligne".into(),
                quantity: dec!(1),
                unit_price: *price,
                vat_rate: *rate,
            })
            .collect(),
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();
    let validated = invoices::validate_invoice(pool, company_id, inv.id, admin_id)
        .await
        .expect("validate_invoice");
    validated.invoice.id
}

/// Crée une facture brouillon (non validée).
async fn create_draft_invoice(
    pool: &MySqlPool,
    company_id: i64,
    contact_id: i64,
    admin_id: i64,
    date: NaiveDate,
    rate: Decimal,
    price: Decimal,
) -> i64 {
    let new = NewInvoice {
        company_id,
        contact_id,
        date,
        due_date: Some(date),
        payment_terms: None,
        lines: vec![NewInvoiceLine {
            description: "Brouillon".into(),
            quantity: dec!(1),
            unit_price: price,
            vat_rate: rate,
        }],
        project_id: None,
    };
    let (inv, _) = invoices::create(pool, admin_id, new).await.unwrap();
    inv.id
}

fn row_by_rate<'a>(rows: &'a [Value], rate: &str) -> Option<&'a Value> {
    rows.iter().find(|r| r["rate"].as_str() == Some(rate))
}

fn dec_field(v: &Value, key: &str) -> Decimal {
    Decimal::from_str(v[key].as_str().unwrap()).unwrap()
}

const FY_PERIOD: &str = "periodStart=2026-01-01&periodEnd=2026-12-31";

// ============================================================
// Tests
// ============================================================

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_requires_auth_returns_401(pool: MySqlPool) {
    let app = spawn_app(pool).await;
    let resp = app
        .client
        .get(app.url("/api/v1/reports/vat?fiscalYearId=1"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 401);
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_aggregates_by_rate_validated_only(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;

    // Validée 1 : deux lignes (8.10 sur 1000, 2.60 sur 500).
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 3, 15).unwrap(),
        &[(dec!(8.10), dec!(1000)), (dec!(2.60), dec!(500))],
    )
    .await;
    // Validée 2 : une ligne (8.10 sur 2000).
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 6, 20).unwrap(),
        &[(dec!(8.10), dec!(2000))],
    )
    .await;
    // Brouillon : exclu.
    create_draft_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 4, 1).unwrap(),
        dec!(8.10),
        dec!(9999),
    )
    .await;
    // Validée hors période (novembre, mais période demandée janv-juin) : exclue.
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 11, 1).unwrap(),
        &[(dec!(8.10), dec!(7000))],
    )
    .await;

    let app = spawn_app(pool).await;
    let token = login(&app).await;

    // Période janvier→juin : la facture de novembre est exclue.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&periodStart=2026-01-01&periodEnd=2026-06-30"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 2, "deux taux attendus (2.60 et 8.10)");

    // Tri par taux ASC.
    assert_eq!(rows[0]["rate"].as_str().unwrap(), "2.60");
    assert_eq!(rows[1]["rate"].as_str().unwrap(), "8.10");

    let r260 = row_by_rate(rows, "2.60").unwrap();
    assert_eq!(dec_field(r260, "baseHt"), dec!(500));
    assert_eq!(dec_field(r260, "vatDue"), dec!(13.00));
    assert!(r260["category"].is_null());

    let r810 = row_by_rate(rows, "8.10").unwrap();
    assert_eq!(dec_field(r810, "baseHt"), dec!(3000));
    // Arrondi par ligne : 81.00 + 162.00 = 243.00.
    assert_eq!(dec_field(r810, "vatDue"), dec!(243.00));

    assert_eq!(dec_field(&body, "totalBaseHt"), dec!(3500));
    assert_eq!(dec_field(&body, "totalVatDue"), dec!(256.00));
    assert_eq!(dec_field(&body, "totalVatRecoverable"), dec!(0));
    assert_eq!(dec_field(&body, "vatBalance"), dec!(256.00));
}

/// AC#3 — arrondi PAR LIGNE ≠ global : 3 lignes 0.10 @ 8 % → 0.03 (pas 0.02).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_rounds_per_line_not_global(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 2, 2).unwrap(),
        &[
            (dec!(8.00), dec!(0.10)),
            (dec!(8.00), dec!(0.10)),
            (dec!(8.00), dec!(0.10)),
        ],
    )
    .await;

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(dec_field(&rows[0], "vatDue"), dec!(0.03));
    assert_ne!(dec_field(&rows[0], "vatDue"), dec!(0.02));
}

/// OPUS-5 — une ligne 0 %/exonérée figure dans le rapport (base > 0, TVA = 0).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_includes_zero_rate_exempt_row(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 5, 5).unwrap(),
        &[(dec!(0.00), dec!(1500))],
    )
    .await;

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    let r0 = row_by_rate(rows, "0.00").expect("ligne 0 % présente");
    assert_eq!(dec_field(r0, "baseHt"), dec!(1500));
    assert_eq!(dec_field(r0, "vatDue"), dec!(0));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_export_pdf(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat/export?format=pdf&fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/pdf"
    );
    let cd = resp
        .headers()
        .get("content-disposition")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();
    assert!(cd.contains("attachment"));
    assert!(
        cd.contains("decompte-tva"),
        "slug localisé fr-CH attendu : {cd}"
    );
    let bytes = resp.bytes().await.unwrap();
    assert!(bytes.starts_with(b"%PDF"), "magic PDF attendu");
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_export_csv(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat/export?format=csv&fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "text/csv; charset=utf-8"
    );
    let bytes = resp.bytes().await.unwrap();
    // BOM UTF-8 en tête.
    assert_eq!(&bytes[0..3], &[0xEF, 0xBB, 0xBF]);
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("Taux;ChiffreAffairesHT;TVADue"));
    assert!(text.contains("8.10;1000.00;81.00"));
    assert!(text.contains("Total chiffre d'affaires HT;1000.00;"));
    assert!(text.contains("Total TVA due;;81.00"));
    assert!(text.contains("TVA récupérable;;0.00"));
    assert!(text.contains("Solde;;81.00"));
}

/// AC#5 — une facture `cancelled` (insérée en SQL direct, aucun endpoint ne la
/// crée) est exclue du rapport.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_excludes_cancelled_invoices(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;
    // Annulée (cancelled, pas de journal_entry_id requis) avec un taux distinctif.
    let inv_c = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, status, date, total_amount, version) \
         VALUES (?, ?, 'cancelled', '2026-04-04', 5000.0000, 0)",
    )
    .bind(company_id)
    .bind(contact_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'annulee', 1, 5000.0000, 77.77, 5000.0000)",
    )
    .bind(inv_c)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert!(
        row_by_rate(rows, "77.77").is_none(),
        "le taux de la facture annulée ne doit pas apparaître"
    );
    assert_eq!(dec_field(&body, "totalBaseHt"), dec!(1000));
}

#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_export_invalid_format_returns_400(pool: MySqlPool) {
    let (_admin_id, _company_id, fy_id) = seed_base(&pool).await;
    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat/export?format=xlsx&fiscalYearId={fy_id}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

/// Période hors exercice → 400 (couvre aussi le cas « chevauchant 2 exercices »,
/// même mécanisme de bornage `PeriodOutOfFiscalYear`).
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_period_out_of_fiscal_year_returns_400(pool: MySqlPool) {
    let (_admin_id, _company_id, fy_id) = seed_base(&pool).await;
    let app = spawn_app(pool).await;
    let token = login(&app).await;
    // FY seedé = 2020-2030 ; periodEnd 2031 est hors bornes.
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&periodStart=2026-01-01&periodEnd=2031-12-31"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

/// Anti-IDOR : les factures d'une autre entreprise n'apparaissent jamais dans le
/// rapport de l'entreprise courante.
#[sqlx::test(migrator = "kesh_db::MIGRATOR")]
async fn vat_report_excludes_cross_tenant_invoices(pool: MySqlPool) {
    let (admin_id, company_id, fy_id) = seed_base(&pool).await;
    let contact_id = seed_contact(&pool, company_id, admin_id, "Client A").await;
    create_validated_invoice(
        &pool,
        company_id,
        contact_id,
        admin_id,
        NaiveDate::from_ymd_opt(2026, 3, 3).unwrap(),
        &[(dec!(8.10), dec!(1000))],
    )
    .await;

    // Company B (autre tenant) + facture validée insérée en SQL direct
    // (taux distinctif 99.99) — ne doit jamais apparaître chez A.
    let company_b = companies::create(
        &pool,
        NewCompany {
            name: "Company B".into(),
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
    .unwrap();
    let contact_b = seed_contact(&pool, company_b.id, admin_id, "Client B").await;
    // Fiscal year + journal entry pour B (la facture validée exige un
    // journal_entry_id — CHECK chk_invoices_validated_has_je).
    let fy_b = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, '2026', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_b.id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let je_b = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, 1, '2026-04-04', 'Ventes', 'B')",
    )
    .bind(company_b.id)
    .bind(fy_b)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let res_b = sqlx::query(
        "INSERT INTO invoices (company_id, contact_id, status, date, total_amount, journal_entry_id, version) \
         VALUES (?, ?, 'validated', '2026-04-04', 7777.0000, ?, 0)",
    )
    .bind(company_b.id)
    .bind(contact_b)
    .bind(je_b)
    .execute(&pool)
    .await
    .unwrap();
    let inv_b = res_b.last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO invoice_lines (invoice_id, position, description, quantity, unit_price, vat_rate, line_total) \
         VALUES (?, 1, 'B line', 1, 7777.0000, 99.99, 7777.0000)",
    )
    .bind(inv_b)
    .execute(&pool)
    .await
    .unwrap();

    let app = spawn_app(pool).await;
    let token = login(&app).await;
    let resp = app
        .client
        .get(app.url(&format!(
            "/api/v1/reports/vat?fiscalYearId={fy_id}&{FY_PERIOD}"
        )))
        .bearer_auth(&token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    let rows = body["rows"].as_array().unwrap();
    assert!(
        row_by_rate(rows, "99.99").is_none(),
        "le taux cross-tenant 99.99 ne doit pas apparaître"
    );
    assert_eq!(dec_field(&body, "totalBaseHt"), dec!(1000));
}
