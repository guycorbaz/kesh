//! End-to-end HTTP tests — Story 21-2a (#246) : montant TTC canonique sur les
//! surfaces API.
//!
//! Couvre l'AC 17 : `totalTtc` sur les réponses facture (détail + liste),
//! KPI de l'échéancier en TTC, export CSV échéancier en TTC, `{amount}` du
//! preview e-mail en TTC, et parité multi-taux avec le débit créance.
//!
//! Harnais calqué sur `contact_payment_terms_e2e.rs` (21-1).

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use chrono::TimeDelta;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::{users, vat_rates};
use serde_json::json;
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
        "e2e-test-password".to_string(),
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
    let state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);

    let app = build_router(state.clone(), "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind should succeed");
    let addr: SocketAddr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap();
    });

    let deadline = Duration::from_secs(2);
    let start = std::time::Instant::now();
    loop {
        match tokio::net::TcpStream::connect(addr).await {
            Ok(_) => break,
            Err(_) if start.elapsed() < deadline => {
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
            Err(e) => panic!("test server did not become ready: {e}"),
        }
    }

    TestApp {
        base_url: format!("http://{}", addr),
        client: reqwest::Client::new(),
    }
}

async fn login(app: &TestApp, username: &str, password: &str) -> String {
    let resp = app
        .client
        .post(app.url("/api/v1/auth/login"))
        .json(&json!({"username": username, "password": password}))
        .send()
        .await
        .expect("login should succeed");
    let body: serde_json::Value = resp.json().await.expect("json body");
    body["accessToken"]
        .as_str()
        .expect("accessToken present")
        .to_string()
}

async fn create_company_user(
    pool: &MySqlPool,
    company_id: i64,
    username: &str,
    password: &str,
) -> i64 {
    let hash = hash_password(password).expect("hash should succeed");
    let user = users::create(
        pool,
        NewUser {
            username: username.to_string(),
            password_hash: hash,
            role: Role::Comptable,
            active: true,
            company_id,
            email: None,
        },
    )
    .await
    .expect("user create should succeed");
    user.id
}

/// Société seedée (exercice + comptes + settings + taux TVA suisses).
async fn create_seeded_company(pool: &MySqlPool) -> i64 {
    let company_result = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('TTC Test Co', 'Test Address\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(pool)
    .await
    .expect("company insert");
    let company_id = company_result.last_insert_id() as i64;

    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2020-2030', '2020-01-01', '2030-12-31', 'Open')",
    )
    .bind(company_id)
    .execute(pool)
    .await
    .expect("fiscal_year insert");

    let mut accounts = std::collections::HashMap::new();
    for (code, name, account_type) in &[
        ("1100", "Créances", "Asset"),
        ("2000", "TVA due", "Liability"),
        ("3000", "Ventes", "Revenue"),
    ] {
        let result = sqlx::query(
            "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
        )
        .bind(company_id)
        .bind(code)
        .bind(name)
        .bind(account_type)
        .execute(pool)
        .await
        .expect("account insert");
        accounts.insert(*code, result.last_insert_id() as i64);
    }

    sqlx::query(
        "INSERT INTO company_invoice_settings \
         (company_id, default_receivable_account_id, default_revenue_account_id, \
          default_vat_payable_account_id, default_sales_journal) \
         VALUES (?, ?, ?, ?, 'Ventes')",
    )
    .bind(company_id)
    .bind(accounts["1100"])
    .bind(accounts["3000"])
    .bind(accounts["2000"])
    .execute(pool)
    .await
    .expect("company_invoice_settings insert");

    vat_rates::seed_default_swiss_rates(pool, company_id)
        .await
        .expect("vat_rates seed");

    company_id
}

async fn create_contact(app: &TestApp, token: &str, name: &str) -> i64 {
    let resp = app
        .client
        .post(app.url("/api/v1/contacts"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactType": "Entreprise",
            "name": name,
            "isClient": true,
            "isSupplier": false,
            "email": "client@example.ch",
        }))
        .send()
        .await
        .expect("contact create");
    assert_eq!(resp.status(), 201);
    resp.json::<serde_json::Value>().await.expect("json")["id"]
        .as_i64()
        .unwrap()
}

/// Crée une facture 1 ligne 100.00 @ 8.1 % et retourne la réponse JSON.
async fn create_invoice_81(app: &TestApp, token: &str, contact_id: i64) -> serde_json::Value {
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-07-01",
            "lines": [
                {"description": "Conseil", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"}
            ]
        }))
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 201);
    resp.json().await.expect("invoice json")
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn total_ttc_on_detail_list_summary_and_csv(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ttc_user", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ttc_user", "password-12345").await;

    let contact_id = create_contact(&app, &token, "TTC Client").await;
    let invoice = create_invoice_81(&app, &token, contact_id).await;

    // (a) Détail : totalTtc = 108.10, totalAmount = 100.00 (HT, compat).
    // Les Decimal sérialisent avec leur scale (DECIMAL(19,4) → 4 décimales).
    assert_eq!(invoice["totalAmount"], "100.0000");
    assert_eq!(invoice["totalTtc"], "108.1000");

    // Liste générale : idem sur l'item.
    let list: serde_json::Value = app
        .client
        .get(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("list")
        .json()
        .await
        .expect("json");
    let item = &list["items"][0];
    assert_eq!(item["totalTtc"], "108.1000");
    assert_eq!(item["totalAmount"], "100.0000");

    // Valider la facture pour l'échéancier (unpaid = validated + non payée).
    let id = invoice["id"].as_i64().unwrap();
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/validate")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("validate");
    assert_eq!(resp.status(), 200);

    // (b) KPI échéancier en TTC.
    let due: serde_json::Value = app
        .client
        .get(app.url("/api/v1/invoices/due-dates"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("due-dates")
        .json()
        .await
        .expect("json");
    assert_eq!(due["summary"]["unpaidCount"], 1);
    assert_eq!(due["summary"]["unpaidTotal"], "108.1000");
    assert_eq!(due["items"][0]["totalTtc"], "108.1000");

    // (c) Export CSV échéancier : colonne total en TTC.
    let csv = app
        .client
        .get(app.url("/api/v1/invoices/due-dates/export.csv"))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("csv");
    assert_eq!(csv.status(), 200);
    let text = String::from_utf8_lossy(&csv.bytes().await.unwrap()).to_string();
    assert!(
        text.contains("108.10"),
        "CSV doit contenir le TTC 108.10, got: {text}"
    );
    assert!(
        !text.contains(";100.00;"),
        "le HT nu ne doit plus apparaître comme total dû, got: {text}"
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn email_preview_amount_is_ttc(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ttc_mail", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ttc_mail", "password-12345").await;

    let contact_id = create_contact(&app, &token, "TTC Mail Client").await;
    let invoice = create_invoice_81(&app, &token, contact_id).await;
    let id = invoice["id"].as_i64().unwrap();

    let resp = app
        .client
        .get(app.url(&format!("/api/v1/invoices/{id}/email-preview")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("preview");
    assert_eq!(resp.status(), 200);
    let preview: serde_json::Value = resp.json().await.expect("json");
    let body = preview["body"].as_str().expect("body");
    assert!(
        body.contains("108.10"),
        "{{amount}} du preview doit être le TTC 108.10, got body: {body}"
    );
    assert!(
        !body.contains("100.00"),
        "le HT ne doit pas apparaître comme montant dû, got body: {body}"
    );
}

/// Parité multi-taux : totalTtc de l'API == débit créance de l'écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn multi_rate_total_ttc_matches_journal_receivable(pool: MySqlPool) {
    let company_id = create_seeded_company(&pool).await;
    create_company_user(&pool, company_id, "ttc_multi", "password-12345").await;
    let app = spawn_app(pool.clone()).await;
    let token = login(&app, "ttc_multi", "password-12345").await;

    let contact_id = create_contact(&app, &token, "TTC Multi").await;
    let resp = app
        .client
        .post(app.url("/api/v1/invoices"))
        .header("Authorization", format!("Bearer {token}"))
        // Taux réellement seedés (Swiss VAT) — le handler valide les taux.
        .json(&json!({
            "contactId": contact_id,
            "date": "2026-07-01",
            "lines": [
                {"description": "A", "quantity": "1", "unitPrice": "100.00", "vatRate": "8.10"},
                {"description": "B", "quantity": "1", "unitPrice": "200.00", "vatRate": "2.60"},
                {"description": "C", "quantity": "1", "unitPrice": "50.00", "vatRate": "3.80"},
                {"description": "D", "quantity": "1", "unitPrice": "10.00", "vatRate": "0.00"}
            ]
        }))
        .send()
        .await
        .expect("invoice create");
    assert_eq!(resp.status(), 201);
    let invoice: serde_json::Value = resp.json().await.expect("json");
    let id = invoice["id"].as_i64().unwrap();
    // HT 360.00 ; TVA 8.10 + 5.20 + 1.90 + 0 = 15.20 ; TTC 375.20.
    assert_eq!(invoice["totalTtc"], "375.2000");

    // #151 : récap TVA par taux — 0 % exclu, trié décroissant, arrondi par ligne.
    let vb = invoice["vatBreakdown"]
        .as_array()
        .expect("vatBreakdown array");
    assert_eq!(vb.len(), 3, "3 taux positifs (0 % exclu)");
    let rates: Vec<&str> = vb
        .iter()
        .map(|v| v["ratePercent"].as_str().unwrap())
        .collect();
    assert_eq!(rates, ["8.10", "3.80", "2.60"], "tri taux décroissant");
    let parse = |v: &serde_json::Value| {
        v.as_str()
            .unwrap()
            .parse::<rust_decimal::Decimal>()
            .unwrap()
    };
    // Σ TVA == 15.20 (= TTC − HT).
    let sum_vat: rust_decimal::Decimal = vb.iter().map(|v| parse(&v["vatAmount"])).sum();
    assert_eq!(sum_vat, "15.20".parse::<rust_decimal::Decimal>().unwrap());
    // Taux normal 8.10 % : base 100.00, TVA 8.10.
    assert_eq!(parse(&vb[0]["baseHt"]), "100.00".parse().unwrap());
    assert_eq!(parse(&vb[0]["vatAmount"]), "8.10".parse().unwrap());
    // Taux réduit 2.60 % sur 200.00 → 5.20.
    assert_eq!(parse(&vb[2]["vatAmount"]), "5.20".parse().unwrap());

    // Validation → le débit créance porte exactement le même TTC.
    let resp = app
        .client
        .post(app.url(&format!("/api/v1/invoices/{id}/validate")))
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .await
        .expect("validate");
    assert_eq!(resp.status(), 200);
    let receivable_debit: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT jel.debit FROM journal_entry_lines jel \
         JOIN journal_entries je ON je.id = jel.entry_id \
         JOIN invoices i ON i.journal_entry_id = je.id \
         JOIN accounts a ON a.id = jel.account_id \
         WHERE i.id = ? AND a.number = '1100'",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .expect("receivable debit");
    assert_eq!(receivable_debit.to_string(), "375.2000");
}
