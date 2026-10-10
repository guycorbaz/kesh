//! Tests E2E — Story 15-1b (#518) : les postes ouverts et les propositions à
//! travers la frontière HTTP, `letterable` sur `GET /accounts` et
//! `letteringCode` au JSON du Grand livre.
//!
//! Couvre les tests de T6 qui se jugent à HTTP : 5 (entrées et ordre des
//! refus), 13 (`limit` des propositions écrêté, 422 au-delà du plafond), 15 et
//! 20 (clés JSON figées des deux réponses, clé d'API en lecture), 16
//! (`letteringCode` du Grand livre), 17 (404 indiscernables, 409, rôle
//! Consultation), 18 (`letterable`). Le contenu de la vue et des propositions
//! est éprouvé au dépôt (`crates/kesh-db/tests/open_items.rs`), l'invariant
//! contre la Balance dans `crates/kesh-report/tests/open_items_invariant.rs`.

mod common;

use std::collections::BTreeSet;
use std::net::SocketAddr;
use std::sync::Arc;

use chrono::{NaiveDate, TimeDelta, Utc};
use common::create_test_company;
use kesh_api::auth::bootstrap::ensure_admin_user;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::NewFiscalYear;
use kesh_db::repositories::fiscal_years;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const TEST_ADMIN_PASSWORD: &str = "e2e-test-admin-password";

struct TestApp {
    base_url: String,
    client: reqwest::Client,
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
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
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

async fn login(app: &TestApp, user: &str, password: &str) -> String {
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/login", app.base_url))
        .json(&json!({ "username": user, "password": password }))
        .send()
        .await
        .unwrap();
    let body: Value = resp.json().await.unwrap();
    body["accessToken"].as_str().unwrap().to_string()
}

struct Monde {
    app: TestApp,
    token: String,
    company_id: i64,
    fy_id: i64,
    /// 1100, actif : lettrable.
    lettrable: i64,
    /// 3000, produit : contrepartie, non lettrable.
    produit: i64,
}

fn aujourdhui() -> NaiveDate {
    Utc::now().date_naive()
}

/// Insère un compte (montage) et rend son id.
async fn compte(pool: &MySqlPool, company_id: i64, numero: &str, type_: &str) -> i64 {
    sqlx::query("INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)")
        .bind(company_id)
        .bind(numero)
        .bind(format!("Compte {numero}"))
        .bind(type_)
        .execute(pool)
        .await
        .expect("compte")
        .last_insert_id() as i64
}

async fn setup(pool: &MySqlPool) -> Monde {
    let app = spawn_app(pool.clone()).await;
    create_test_company(pool).await;
    ensure_admin_user(pool, &test_config()).await.unwrap();
    let token = login(&app, "admin", TEST_ADMIN_PASSWORD).await;
    let company_id: i64 = sqlx::query_scalar("SELECT id FROM companies ORDER BY id LIMIT 1")
        .fetch_one(pool)
        .await
        .unwrap();
    let fy = fiscal_years::create(
        pool,
        1,
        NewFiscalYear {
            company_id,
            name: "Exercice 2020-2030".into(),
            start_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            end_date: NaiveDate::from_ymd_opt(2030, 12, 31).unwrap(),
        },
    )
    .await
    .expect("exercice");
    Monde {
        lettrable: compte(pool, company_id, "1100", "Asset").await,
        produit: compte(pool, company_id, "3000", "Revenue").await,
        app,
        token,
        company_id,
        fy_id: fy.id,
    }
}

/// Une écriture à deux lignes en SQL direct : `montant` sur `sur` (> 0 : débit),
/// la contrepartie sur le 3000 ; rend la ligne sur `sur`.
async fn ligne(pool: &MySqlPool, m: &Monde, sur: i64, date: NaiveDate, montant: Decimal) -> i64 {
    let numero: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(entry_number), 0) + 1 FROM journal_entries WHERE fiscal_year_id = ?",
    )
    .bind(m.fy_id)
    .fetch_one(pool)
    .await
    .unwrap();
    let entry = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
         journal, description) VALUES (?, ?, ?, ?, 'OD', 'postes ouverts')",
    )
    .bind(m.company_id)
    .bind(m.fy_id)
    .bind(numero)
    .bind(date)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let (d, c) = if montant > Decimal::ZERO {
        (montant, Decimal::ZERO)
    } else {
        (Decimal::ZERO, -montant)
    };
    let mut ids = Vec::new();
    for (ordre, (compte, debit, credit)) in [(sur, d, c), (m.produit, c, d)].into_iter().enumerate()
    {
        ids.push(
            sqlx::query(
                "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
                 VALUES (?, ?, ?, ?, ?)",
            )
            .bind(entry)
            .bind(compte)
            .bind(ordre as i32 + 1)
            .bind(debit)
            .bind(credit)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        );
    }
    ids[0]
}

async fn get(m: &Monde, token: &str, path: &str) -> (reqwest::StatusCode, Value) {
    let resp = m
        .app
        .client
        .get(format!("{}{path}", m.app.base_url))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    let status = resp.status();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn lettrer(m: &Monde, ids: &[i64]) -> Value {
    let resp = m
        .app
        .client
        .post(format!("{}/api/v1/letterings", m.app.base_url))
        .bearer_auth(&m.token)
        .json(&json!({ "lineIds": ids }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "lettrage");
    resp.json().await.unwrap()
}

async fn creer_cle(m: &Monde, name: &str, scope: &str) -> String {
    let resp = m
        .app
        .client
        .post(format!("{}/api/v1/settings/api-keys", m.app.base_url))
        .bearer_auth(&m.token)
        .json(&json!({ "name": name, "scope": scope }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201, "création de clé");
    resp.json::<Value>().await.unwrap()["key"]
        .as_str()
        .unwrap()
        .to_string()
}

fn cles(v: &Value) -> BTreeSet<String> {
    v.as_object()
        .unwrap_or_else(|| panic!("objet attendu : {v}"))
        .keys()
        .cloned()
        .collect()
}

fn ensemble(noms: &[&str]) -> BTreeSet<String> {
    noms.iter().map(|s| s.to_string()).collect()
}

fn open_items_path(compte: i64, query: &str) -> String {
    format!("/api/v1/accounts/{compte}/open-items{query}")
}

// ---------------------------------------------------------------------------
// Test 5 — entrées et ordre des refus
// ---------------------------------------------------------------------------

/// Test 5 (AC1, AC7) — `asOf` absent = aujourd'hui ; mal formé → 400
/// `VALIDATION_ERROR` ; `limit=999999` → 500, `limit=0` → 1, `offset=-3` → 0 ;
/// un `limit` non numérique est rejeté par l'extracteur d'Axum ; et l'ordre
/// des refus : `asOf` mal formé sur un compte d'une autre société → 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn inputs_defaults_bounds_and_refusal_order(pool: MySqlPool) {
    let m = setup(&pool).await;
    ligne(
        &pool,
        &m,
        m.lettrable,
        aujourdhui() - TimeDelta::days(3),
        dec!(10),
    )
    .await;
    let base = open_items_path(m.lettrable, "");

    let (status, body) = get(&m, &m.token, &base).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(
        body["asOf"],
        aujourdhui().to_string(),
        "asOf absent = aujourd'hui"
    );
    assert_eq!(
        (body["limit"].as_i64(), body["offset"].as_i64()),
        (Some(50), Some(0))
    );
    assert_eq!(body["total"], 1);

    let (status, body) = get(&m, &m.token, &format!("{base}?asOf=2026-13-01")).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");

    // Hors de la plage des dates de MariaDB (revue P1, B-1 = E-1) : 400, jamais
    // un 500 ni un solde faux.
    for hors in ["%2B10000-01-01", "-0001-01-01", "0999-12-31"] {
        let (status, body) = get(&m, &m.token, &format!("{base}?asOf={hors}")).await;
        assert_eq!(status, 400, "asOf={hors} : {body}");
        assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    }
    let (status, body) = get(&m, &m.token, &format!("{base}?asOf=9999-12-31")).await;
    assert_eq!(status, 200, "{body}");
    assert_eq!(body["total"], 1);
    let (status, body) = get(&m, &m.token, &format!("{base}?asOf=")).await;
    assert_eq!(status, 200, "asOf vide = absent : {body}");
    assert_eq!(body["asOf"], aujourdhui().to_string());

    let (_, body) = get(&m, &m.token, &format!("{base}?limit=999999")).await;
    assert_eq!(body["limit"], 500);
    let (_, body) = get(&m, &m.token, &format!("{base}?limit=0&offset=-3")).await;
    assert_eq!(
        (body["limit"].as_i64(), body["offset"].as_i64()),
        (Some(1), Some(0))
    );

    let (status, body) = get(&m, &m.token, &format!("{base}?limit=abc")).await;
    assert_eq!(status, 400, "rejet de l'extracteur Query : {body}");
    assert_ne!(body["error"]["code"], "VALIDATION_ERROR");

    // Ordre des refus : la date d'abord, avant toute lecture.
    let autre = kesh_db::test_fixtures::seed_stub_company_only(&pool)
        .await
        .unwrap();
    let etranger = compte(&pool, autre, "1100", "Asset").await;
    let (status, body) = get(&m, &m.token, &open_items_path(etranger, "?asOf=2026-13-01")).await;
    assert_eq!(status, 400, "{body}");
    assert_eq!(body["error"]["code"], "VALIDATION_ERROR");
    let (status, _) = get(&m, &m.token, &open_items_path(etranger, "")).await;
    assert_eq!(status, 404);
}

// ---------------------------------------------------------------------------
// Tests 15 et 20 — contrats JSON figés, clé d'API en lecture
// ---------------------------------------------------------------------------

/// Tests 15 et 20 (AC1, AC5) — les clés JSON des deux réponses, figées, et
/// l'accès par une **clé d'API en lecture**.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn json_contracts_are_frozen_and_a_read_key_reaches_both_routes(pool: MySqlPool) {
    let m = setup(&pool).await;
    let j = aujourdhui() - TimeDelta::days(5);
    let a = ligne(&pool, &m, m.lettrable, j, dec!(25)).await;
    let b = ligne(&pool, &m, m.lettrable, j, dec!(-25)).await;
    // Une ligne lettrée après X, pour remplir les champs du lettrage.
    let c = ligne(&pool, &m, m.lettrable, j, dec!(7)).await;
    let d = ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(-7)).await;
    lettrer(&m, &[c, d]).await;
    let lecture = creer_cle(&m, "lecture-15-1b", "read").await;

    let veille = (aujourdhui() - TimeDelta::days(1)).to_string();
    let (status, body) = get(
        &m,
        &lecture,
        &open_items_path(m.lettrable, &format!("?asOf={veille}")),
    )
    .await;
    assert_eq!(status, 200, "une clé read lit la vue : {body}");
    assert_eq!(
        cles(&body),
        ensemble(&[
            "accountId",
            "accountNumber",
            "asOf",
            "balance",
            "openTotal",
            "total",
            "offset",
            "limit",
            "items"
        ])
    );
    assert_eq!(body["accountNumber"], "1100");
    assert_eq!(body["balance"], body["openTotal"]);
    let items = body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "{body}");
    let attendu = ensemble(&[
        "lineId",
        "entryId",
        "entryNumber",
        "fiscalYearName",
        "date",
        "journal",
        "description",
        "debit",
        "credit",
        "document",
        "letteringCode",
        "letteringOrigin",
        "letteredOn",
        "reason",
        "documentState",
        "amountDue",
        "manuallyLetterable",
        "inOpenPeriod",
    ]);
    for i in items {
        assert_eq!(cles(i), attendu, "{i}");
    }
    let lettree = items.iter().find(|i| i["lineId"] == c).unwrap();
    assert_eq!(lettree["reason"], "letteredAfterAsOf");
    assert_eq!(lettree["letteringOrigin"], "manual");
    assert_eq!(lettree["letteredOn"], aujourdhui().to_string());
    assert!(lettree["letteringCode"].is_string());
    assert_eq!(lettree["manuallyLetterable"], false);
    let libre = items.iter().find(|i| i["lineId"] == a).unwrap();
    assert_eq!(libre["reason"], "unlettered");
    assert!(libre["document"].is_null() && libre["documentState"].is_null());
    assert_eq!(libre["manuallyLetterable"], true);
    assert_eq!(libre["inOpenPeriod"], true);
    assert_eq!(libre["fiscalYearName"], "Exercice 2020-2030");

    let (status, body) = get(
        &m,
        &lecture,
        &format!("/api/v1/accounts/{}/lettering-proposals", m.lettrable),
    )
    .await;
    assert_eq!(status, 200, "une clé read lit les propositions : {body}");
    assert_eq!(
        cles(&body),
        ensemble(&["accountId", "candidateCount", "total", "limit", "items"])
    );
    assert_eq!(
        (body["candidateCount"].as_i64(), body["limit"].as_i64()),
        (Some(2), Some(100))
    );
    let p = &body["items"][0];
    assert_eq!(
        cles(p),
        ensemble(&["amount", "daysApart", "reversalPair", "debit", "credit"])
    );
    assert_eq!(p["debit"]["lineId"], a);
    assert_eq!(p["credit"]["lineId"], b);
    assert_eq!(p["daysApart"], 0);
    assert_eq!(p["reversalPair"], false);
    for cote in ["debit", "credit"] {
        assert_eq!(
            cles(&p[cote]),
            ensemble(&[
                "lineId",
                "entryId",
                "entryNumber",
                "fiscalYearName",
                "date",
                "journal",
                "description",
                "document",
                "inOpenPeriod"
            ])
        );
    }
}

/// Test 6 à HTTP (AC3) — `document` est sérialisé par `DocumentKind::as_str()`,
/// avec ses cinq clés : une ligne seulement rapprochée d'une transaction.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn document_object_shape(pool: MySqlPool) {
    let m = setup(&pool).await;
    let l = ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(-15)).await;
    let entry: i64 = sqlx::query_scalar("SELECT entry_id FROM journal_entry_lines WHERE id = ?")
        .bind(l)
        .fetch_one(&pool)
        .await
        .unwrap();
    let banque = sqlx::query(
        "INSERT INTO bank_accounts (company_id, bank_name, iban) VALUES (?, 'B', 'CH9300762011623852957')",
    )
    .bind(m.company_id)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let import = sqlx::query(
        "INSERT INTO bank_imports (company_id, bank_account_id, filename, file_hash, \
         source_format, period_from, period_to, imported_by_user_id) \
         VALUES (?, ?, 'l.xml', REPEAT('c', 64), 'camt053', ?, ?, 1)",
    )
    .bind(m.company_id)
    .bind(banque)
    .bind(aujourdhui())
    .bind(aujourdhui())
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let tx = sqlx::query(
        "INSERT INTO bank_transactions (company_id, import_id, bank_account_id, booking_date, \
         amount, currency, details, matched_entry_id, status) \
         VALUES (?, ?, ?, ?, 15, 'CHF', 'x', ?, 'reconciled')",
    )
    .bind(m.company_id)
    .bind(import)
    .bind(banque)
    .bind(aujourdhui())
    .bind(entry)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let (_, body) = get(&m, &m.token, &open_items_path(m.lettrable, "")).await;
    let doc = &body["items"][0]["document"];
    assert_eq!(
        cles(doc),
        ensemble(&["type", "id", "number", "invoiceId", "invoiceNumber"])
    );
    assert_eq!(doc["type"], "bankTransaction");
    assert_eq!(doc["id"], tx);
    assert!(doc["number"].is_null() && doc["invoiceId"].is_null());
    assert_eq!(body["items"][0]["manuallyLetterable"], true);
}

// ---------------------------------------------------------------------------
// Test 13 à HTTP — propositions : `limit` et 422
// ---------------------------------------------------------------------------

/// Test 13 (AC5) — `limit=999999` écrêté à 500, `limit=0` → 1 ; au-delà de
/// 2 000 candidates, 422 `LETTERING_PROPOSALS_TOO_MANY_LINES` avec le plafond
/// dans le message.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn proposals_limit_and_cap(pool: MySqlPool) {
    let m = setup(&pool).await;
    ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(3)).await;
    let base = format!("/api/v1/accounts/{}/lettering-proposals", m.lettrable);
    let (_, body) = get(&m, &m.token, &format!("{base}?limit=999999")).await;
    assert_eq!(body["limit"], 500);
    let (_, body) = get(&m, &m.token, &format!("{base}?limit=0")).await;
    assert_eq!(body["limit"], 1);

    let entry: i64 = sqlx::query_scalar("SELECT MAX(id) FROM journal_entries")
        .fetch_one(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         SELECT ?, ?, 10 + seq, 1, 0 FROM seq_1_to_2000",
    )
    .bind(entry)
    .bind(m.lettrable)
    .execute(&pool)
    .await
    .unwrap();
    let (status, body) = get(&m, &m.token, &base).await;
    assert_eq!(status, 422, "{body}");
    assert_eq!(body["error"]["code"], "LETTERING_PROPOSALS_TOO_MANY_LINES");
    assert!(
        body["error"]["message"].as_str().unwrap().contains("2000"),
        "le plafond dans le message : {body}"
    );
}

// ---------------------------------------------------------------------------
// Test 16 — `letteringCode` au Grand livre
// ---------------------------------------------------------------------------

/// Test 16 (AC6) — le JSON du Grand livre porte `letteringCode` : le code du
/// groupe pour une ligne lettrée, `null` pour une ligne ouverte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn general_ledger_json_carries_the_lettering_code(pool: MySqlPool) {
    let m = setup(&pool).await;
    let a = ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(40)).await;
    let b = ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(-40)).await;
    let libre = ligne(&pool, &m, m.lettrable, aujourdhui(), dec!(2)).await;
    let groupe = lettrer(&m, &[a, b]).await;
    let (status, body) = get(
        &m,
        &m.token,
        &format!(
            "/api/v1/reports/general-ledger?from=2020-01-01&to=2030-12-31&accountIds={}",
            m.lettrable
        ),
    )
    .await;
    assert_eq!(status, 200, "{body}");
    let lignes = body["sections"][0]["lines"].as_array().unwrap();
    let code = |id: i64| {
        lignes
            .iter()
            .find(|l| l["lineId"] == id)
            .unwrap_or_else(|| panic!("ligne {id} absente : {body}"))["letteringCode"]
            .clone()
    };
    assert_eq!(code(a), groupe["code"]);
    assert_eq!(code(b), groupe["code"]);
    assert!(code(libre).is_null());
}

// ---------------------------------------------------------------------------
// Test 17 — 404, 409, rôle
// ---------------------------------------------------------------------------

/// Test 17 (AC7) — 404 indiscernables (autre société, inexistant) sur les deux
/// routes ; 409 `LETTERING_ACCOUNT_NOT_LETTERABLE` pour un compte de charge, et
/// pour un compte RETYPÉ après lettrage (C104) ; le rôle Consultation lit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn not_found_not_letterable_and_consultation(pool: MySqlPool) {
    use kesh_db::entities::{NewUser, Role};
    let m = setup(&pool).await;
    let autre = kesh_db::test_fixtures::seed_stub_company_only(&pool)
        .await
        .unwrap();
    let etranger = compte(&pool, autre, "1100", "Asset").await;
    let charge = compte(&pool, m.company_id, "4000", "Expense").await;
    // Un compte lettré, puis retypé en charge.
    let retype = compte(&pool, m.company_id, "1105", "Asset").await;
    let a = ligne(&pool, &m, retype, aujourdhui(), dec!(5)).await;
    let b = ligne(&pool, &m, retype, aujourdhui(), dec!(-5)).await;
    lettrer(&m, &[a, b]).await;
    sqlx::query("UPDATE accounts SET account_type = 'Expense' WHERE id = ?")
        .bind(retype)
        .execute(&pool)
        .await
        .unwrap();

    for route in ["open-items", "lettering-proposals"] {
        let (s1, b1) = get(
            &m,
            &m.token,
            &format!("/api/v1/accounts/{etranger}/{route}"),
        )
        .await;
        let (s2, b2) = get(&m, &m.token, &format!("/api/v1/accounts/999999999/{route}")).await;
        assert_eq!((s1.as_u16(), s2.as_u16()), (404, 404), "{route}");
        assert_eq!(b1, b2, "404 indiscernables ({route})");
        for c in [charge, retype, m.produit] {
            let (s, body) = get(&m, &m.token, &format!("/api/v1/accounts/{c}/{route}")).await;
            assert_eq!(s, 409, "{route}, compte {c} : {body}");
            assert_eq!(body["error"]["code"], "LETTERING_ACCOUNT_NOT_LETTERABLE");
        }
    }

    let password = "consultation-test-pw-12345";
    kesh_db::repositories::users::create(
        &pool,
        NewUser {
            username: "consultation".into(),
            password_hash: kesh_api::auth::password::hash_password(password).unwrap(),
            role: Role::Consultation,
            active: true,
            company_id: m.company_id,
            email: None,
        },
    )
    .await
    .unwrap();
    let lecteur = login(&m.app, "consultation", password).await;
    for route in ["open-items", "lettering-proposals"] {
        let (s, body) = get(
            &m,
            &lecteur,
            &format!("/api/v1/accounts/{}/{route}", m.lettrable),
        )
        .await;
        assert_eq!(s, 200, "Consultation lit {route} : {body}");
    }
}

// ---------------------------------------------------------------------------
// Test 18 — `letterable`
// ---------------------------------------------------------------------------

/// Test 18 (AC11) — `letterable` : compte de bilan vrai, compte bancaire faux,
/// compte ARCHIVÉ dont le compte bancaire est archivé faux (C127), produit faux
/// — dans la liste (archivés compris) et dans une réponse unitaire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn letterable_flag_on_accounts(pool: MySqlPool) {
    let m = setup(&pool).await;
    let banque = compte(&pool, m.company_id, "1020", "Asset").await;
    let banque_archivee = compte(&pool, m.company_id, "1021", "Asset").await;
    let passif = compte(&pool, m.company_id, "2000", "Liability").await;
    for (c, archive) in [(banque, false), (banque_archivee, true)] {
        sqlx::query(
            "INSERT INTO bank_accounts (company_id, bank_name, iban, journal_account_id, archived) \
             VALUES (?, 'B', 'CH9300762011623852957', ?, ?)",
        )
        .bind(m.company_id)
        .bind(c)
        .bind(archive)
        .execute(&pool)
        .await
        .unwrap();
    }
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(banque_archivee)
        .execute(&pool)
        .await
        .unwrap();

    let (status, body) = get(&m, &m.token, "/api/v1/accounts?includeArchived=true").await;
    assert_eq!(status, 200, "{body}");
    let drapeau = |id: i64| {
        body.as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == id)
            .unwrap_or_else(|| panic!("compte {id} absent : {body}"))["letterable"]
            .clone()
    };
    assert_eq!(drapeau(m.lettrable), true, "actif");
    assert_eq!(drapeau(passif), true, "passif");
    assert_eq!(drapeau(banque), false, "compte bancaire");
    assert_eq!(
        drapeau(banque_archivee),
        false,
        "compte bancaire archivé (C127)"
    );
    assert_eq!(drapeau(m.produit), false, "produit");

    // Réponses unitaires : création (bilan → vrai) et réactivation du compte
    // bancaire archivé (→ faux).
    let resp = m
        .app
        .client
        .post(format!("{}/api/v1/accounts", m.app.base_url))
        .bearer_auth(&m.token)
        .json(&json!({ "number": "1110", "name": "Débiteurs bis", "accountType": "Asset" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let cree: Value = resp.json().await.unwrap();
    assert_eq!(cree["letterable"], true, "{cree}");
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(banque_archivee)
        .fetch_one(&pool)
        .await
        .unwrap();
    let resp = m
        .app
        .client
        .put(format!(
            "{}/api/v1/accounts/{banque_archivee}/reactivate",
            m.app.base_url
        ))
        .bearer_auth(&m.token)
        .json(&json!({ "version": version }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let reactive: Value = resp.json().await.unwrap();
    assert_eq!(reactive["letterable"], false, "{reactive}");
    assert_eq!(reactive["active"], true);

    // Modification (passif → passif) et archivage du 2000 : vrai (revue P1, A).
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(passif)
        .fetch_one(&pool)
        .await
        .unwrap();
    let resp = m
        .app
        .client
        .put(format!("{}/api/v1/accounts/{passif}", m.app.base_url))
        .bearer_auth(&m.token)
        .json(
            &json!({ "name": "Passif modifié", "accountType": "Liability", "role": null,
                       "postable": true, "version": version }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let modifie: Value = resp.json().await.unwrap();
    assert_eq!(modifie["letterable"], true, "{modifie}");
    let resp = m
        .app
        .client
        .put(format!(
            "{}/api/v1/accounts/{passif}/archive",
            m.app.base_url
        ))
        .bearer_auth(&m.token)
        .json(&json!({ "version": modifie["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let archive: Value = resp.json().await.unwrap();
    assert_eq!(
        archive["letterable"], true,
        "archivé, toujours lettrable : {archive}"
    );
    assert_eq!(archive["active"], false);
    // Et le compte bancaire, modifié : faux.
    let version: i32 = sqlx::query_scalar("SELECT version FROM accounts WHERE id = ?")
        .bind(banque)
        .fetch_one(&pool)
        .await
        .unwrap();
    let resp = m
        .app
        .client
        .put(format!("{}/api/v1/accounts/{banque}", m.app.base_url))
        .bearer_auth(&m.token)
        .json(
            &json!({ "name": "Banque modifiée", "accountType": "Asset", "role": null,
                       "postable": true, "version": version }),
        )
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let banque_modifiee: Value = resp.json().await.unwrap();
    assert_eq!(banque_modifiee["letterable"], false, "{banque_modifiee}");

    // Le cas opposé de chaque réponse unitaire (revue P2, E2-2) : création d'une
    // charge → faux ; archivage du compte bancaire → faux ; réactivation d'un
    // passif → vrai.
    let resp = m
        .app
        .client
        .post(format!("{}/api/v1/accounts", m.app.base_url))
        .bearer_auth(&m.token)
        .json(&json!({ "number": "4100", "name": "Charges bis", "accountType": "Expense" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 201);
    let charge: Value = resp.json().await.unwrap();
    assert_eq!(charge["letterable"], false, "{charge}");
    let resp = m
        .app
        .client
        .put(format!(
            "{}/api/v1/accounts/{banque}/archive",
            m.app.base_url
        ))
        .bearer_auth(&m.token)
        .json(&json!({ "version": banque_modifiee["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let banque_archivee_rep: Value = resp.json().await.unwrap();
    assert_eq!(
        banque_archivee_rep["letterable"], false,
        "{banque_archivee_rep}"
    );
    let resp = m
        .app
        .client
        .put(format!(
            "{}/api/v1/accounts/{passif}/reactivate",
            m.app.base_url
        ))
        .bearer_auth(&m.token)
        .json(&json!({ "version": archive["version"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let passif_reactive: Value = resp.json().await.unwrap();
    assert_eq!(passif_reactive["letterable"], true, "{passif_reactive}");
}
