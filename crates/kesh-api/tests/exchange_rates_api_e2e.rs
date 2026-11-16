//! Story 37-2 (AC-3 bis, AC-5, AC-9) — les routes des cours de la société :
//! cours effectif (`GET /exchange-rates/lookup`), cases d'un mois ou d'un jour
//! (`GET /exchange-rates`), saisie (`PUT /exchange-rates/rates`), ↑
//! (`POST /exchange-rates/rates/reset`), corrections de l'AFC
//! (`GET /exchange-rates/notices`, `POST …/notices/acknowledge`), tableau
//! annuel (`GET /fiscal-years/{id}/exchange-rates`), genres
//! (`PUT /fiscal-years/{id}/rate-kinds`) et la copie des cours récupérés aux
//! trois déclencheurs de 37-1 (création d'un exercice, compte qui reçoit une
//! devise) — contre la source factice : aucun appel réseau.
//!
//! Horloge arrêtée sur le 16.11.2026, 10:00 UTC.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::exchange_rates::MockRateSource;
use kesh_api::{AppState, build_router};
use kesh_core::exchange_rates::{PublishedRate, RateKind, RateUnit};
use kesh_db::repositories::exchange_rates::{self as depot, SOURCE_AFC};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const RATES: &str = "/api/v1/exchange-rates/rates";
const RESET: &str = "/api/v1/exchange-rates/rates/reset";
const NOTICES: &str = "/api/v1/exchange-rates/notices";
const ACK: &str = "/api/v1/exchange-rates/notices/acknowledge";

// ============================================================
// Harnais
// ============================================================

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

fn mois(y: i32, m: u32) -> RateUnit {
    RateUnit::month_of(d(y, m, 1))
}

fn instant() -> DateTime<Utc> {
    "2026-11-16T10:00:00Z".parse().unwrap()
}

fn test_config() -> Config {
    Config::from_fields_for_test(
        "mysql://test:test@localhost:3306/test".to_string(),
        "admin".to_string(),
        "e2e-test-admin-password".to_string(),
        String::from_utf8(TEST_JWT_SECRET.to_vec()).unwrap(),
        TimeDelta::minutes(15),
        TimeDelta::days(30),
        TimeDelta::minutes(15),
        100,
        TimeDelta::minutes(30),
        12,
    )
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
    base_url: String,
    client: reqwest::Client,
    s: SeededCompany,
    jwt: String,
}

impl Ctx {
    async fn envoyer(
        &self,
        methode: reqwest::Method,
        path: &str,
        jwt: &str,
        body: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .client
            .request(methode, format!("{}{}", self.base_url, path))
            .bearer_auth(jwt);
        if let Some(b) = body {
            r = r.json(&b);
        }
        let r = r.send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn get(&self, path: &str) -> (u16, Value) {
        self.envoyer(reqwest::Method::GET, path, &self.jwt, None)
            .await
    }

    async fn put(&self, path: &str, body: Value) -> (u16, Value) {
        self.envoyer(reqwest::Method::PUT, path, &self.jwt, Some(body))
            .await
    }

    async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        self.envoyer(reqwest::Method::POST, path, &self.jwt, Some(body))
            .await
    }

    fn fy(&self) -> i64 {
        self.s.fiscal_year_id
    }

    fn c(&self) -> i64 {
        self.s.company_id
    }
}

/// Société seedée ; `multi` : rendue multi-devises (réglage, comptes de
/// différence, exercice 2020–2030) avec un compte en USD (1020).
async fn monter(pool: &MySqlPool, multi: bool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    let mut ids = Vec::new();
    for (numero, genre) in [("6950", "Expense"), ("1090", "Asset")] {
        ids.push(
            sqlx::query(
                "INSERT INTO accounts (company_id, number, name, account_type) VALUES (?, ?, ?, ?)",
            )
            .bind(s.company_id)
            .bind(numero)
            .bind(format!("Compte {numero}"))
            .bind(genre)
            .execute(pool)
            .await
            .unwrap()
            .last_insert_id() as i64,
        );
    }
    if multi {
        sqlx::query(
            "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
             fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
        )
        .bind(ids[0])
        .bind(ids[0])
        .bind(ids[1])
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
        sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
            .bind(s.fiscal_year_id)
            .execute(pool)
            .await
            .unwrap();
        compte(pool, s.company_id, "1020", Some("USD"), true).await;
    }

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
    kesh_api::errors::init_error_i18n(i18n.clone(), config.locale);
    let mut state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
    state.rate_source = Arc::new(MockRateSource::new());
    let state = common::horloge::avec_horloge(state, instant());
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
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    Ctx {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
        s,
        jwt,
    }
}

/// Un compte d'actif, en devise ou non, actif ou archivé ; rend son id.
async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    devise: Option<&str>,
    actif: bool,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency, active) \
         VALUES (?, ?, ?, 'Asset', ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(devise)
    .bind(actif)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Une ligne `exchange_rates` posée par SQL ; rend son id.
#[allow(clippy::too_many_arguments)]
async fn ligne(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    kind: &str,
    date: &str,
    cours: &str,
    source: &str,
    updated_by: Option<i64>,
) -> i64 {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source, \
             updated_by) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(devise)
    .bind(kind)
    .bind(date)
    .bind(Decimal::from_str(cours).unwrap())
    .bind(source)
    .bind(updated_by)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Écrit des valeurs publiées (règle (a)(b)(c)) pour une unité.
async fn publier(pool: &MySqlPool, u: RateUnit, cours: &[(&str, i32, Decimal)]) {
    let rates: Vec<PublishedRate> = cours
        .iter()
        .map(|(code, unite, c)| PublishedRate {
            currency: kesh_core::currency::CurrencyCode::parse(code).unwrap(),
            published_unit: *unite,
            published_rate: *c,
            rate_per_unit: (*c / Decimal::from(*unite)).round_dp(10),
        })
        .collect();
    let mut tx = pool.begin().await.unwrap();
    let source_date = (u.kind == RateKind::Day).then_some(u.date);
    depot::write_fetched_unit_in_tx(
        &mut tx,
        SOURCE_AFC,
        u,
        source_date,
        &rates,
        d(2026, 11, 1).and_hms_opt(8, 0, 0).unwrap(),
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn compter(pool: &MySqlPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql).fetch_one(pool).await.unwrap()
}

/// Les détails des entrées d'audit d'une action, par id croissant.
async fn audits(pool: &MySqlPool, action: &str) -> Vec<Value> {
    let lignes: Vec<Option<String>> = sqlx::query_scalar(
        "SELECT CAST(details_json AS CHAR) FROM audit_log WHERE action = ? ORDER BY id",
    )
    .bind(action)
    .fetch_all(pool)
    .await
    .unwrap();
    lignes
        .into_iter()
        .map(|l| {
            l.map(|t| serde_json::from_str(&t).unwrap())
                .unwrap_or(Value::Null)
        })
        .collect()
}

/// Le cours retenu par une société (`None` sans ligne) et sa source.
async fn retenu(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    kind: &str,
    date: &str,
) -> Option<(Decimal, String)> {
    sqlx::query_as(
        "SELECT rate, source FROM exchange_rates WHERE company_id = ? AND currency = ? \
         AND kind = ? AND rate_date = ?",
    )
    .bind(company_id)
    .bind(devise)
    .bind(kind)
    .bind(date)
    .fetch_optional(pool)
    .await
    .unwrap()
}

fn dec_de(v: &Value) -> Decimal {
    Decimal::from_str(
        v.as_str()
            .unwrap_or_else(|| panic!("décimal attendu : {v}")),
    )
    .unwrap()
}

/// Seconde société multi-devises, avec un Administrateur ; rend `(id, jwt)`.
async fn societe_b(pool: &MySqlPool) -> (i64, String) {
    let c = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, \
             instance_language, multi_currency_enabled) \
         VALUES ('Société B', 'Rue B 1\n1000 Lausanne', 'Independant', 'FR', 'FR', TRUE)",
    )
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let u = sqlx::query(
        "INSERT INTO users (username, password_hash, role, active, company_id) \
         VALUES ('admin-b', REPEAT('x', 64), 'Admin', TRUE, ?)",
    )
    .bind(c)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    (c, forge_jwt(u, "Admin", c))
}

/// Redéfinit l'exercice seedé : dates et genres.
async fn exercice(pool: &MySqlPool, id: i64, debut: NaiveDate, fin: NaiveDate, o: &str, c: &str) {
    sqlx::query(
        "UPDATE fiscal_years SET start_date = ?, end_date = ?, opening_rate_kind = ?, \
         closing_rate_kind = ? WHERE id = ?",
    )
    .bind(debut)
    .bind(fin)
    .bind(o)
    .bind(c)
    .bind(id)
    .execute(pool)
    .await
    .unwrap();
}

/// Insère un exercice ; rend son id.
#[allow(clippy::too_many_arguments)]
async fn nouvel_exercice(
    pool: &MySqlPool,
    company_id: i64,
    nom: &str,
    debut: NaiveDate,
    fin: NaiveDate,
    statut: &str,
    multi: bool,
    ouverture: &str,
) -> i64 {
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
             multi_currency, opening_rate_kind) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(nom)
    .bind(debut)
    .bind(fin)
    .bind(statut)
    .bind(multi)
    .bind(ouverture)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

fn corps_put(devise: &str, kind: &str, date: &str, cours: &str, version: Option<i32>) -> Value {
    let mut b = json!({ "currency": devise, "kind": kind, "rateDate": date, "rate": cours });
    if let Some(v) = version {
        b["version"] = json!(v);
    }
    b
}

// ============================================================
// Cours effectif — lookup (AC-5, AC-9)
// ============================================================

/// Cours effectif : ligne `manual` seule (200, `publishedRate` `null`), aucune
/// ligne mais valeur publiée (404 `RATE_UNAVAILABLE`, forme `month`), puis 200
/// après le `POST /fetch` qui copie (`afc`) ; une correction de l'AFC ne change
/// pas le cours retenu ; forme `day` du 404 avec `details.date` ; refus
/// (`CHF`, `usd`, genre, date absente) ; aucune écriture.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn lookup_rend_le_cours_retenu_par_la_societe(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-01-01",
        "1.1",
        "manual",
        None,
    )
    .await;
    let audits_avant = compter(&pool, "SELECT COUNT(*) FROM audit_log").await;

    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=USD&date=2026-01-20")
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["kind"], "month");
    assert_eq!(b["rateDate"], "2026-01-01");
    assert_eq!(b["source"], "manual");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(1.1));
    assert_eq!(b["publishedRate"], Value::Null);
    assert_eq!(b["sourceDate"], Value::Null);

    // Valeur publiée sans ligne de la société : 404, forme mensuelle.
    publier(&pool, mois(2026, 2), &[("USD", 1, dec!(0.8700))]).await;
    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=USD&date=2026-02-10")
        .await;
    assert_eq!(s, 404, "{b}");
    assert_eq!(b["error"]["code"], "RATE_UNAVAILABLE");
    assert_eq!(
        b["error"]["details"],
        json!({ "currency": "USD", "month": "2026-02-01" })
    );
    // Lookup en lecture seule : aucune audit, aucune ligne.
    assert_eq!(
        compter(&pool, "SELECT COUNT(*) FROM audit_log").await,
        audits_avant
    );
    assert!(
        retenu(&pool, ctx.c(), "USD", "month", "2026-02-01")
            .await
            .is_none()
    );

    // Le POST /fetch copie, puis le lookup rend 200 `afc`.
    let (s, b) = ctx
        .post(
            "/api/v1/exchange-rates/fetch",
            json!({ "year": 2026, "month": 2 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=USD&date=2026-02-10&kind=month")
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "afc");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.88));
    assert_eq!(dec_de(&b["publishedRate"]), dec!(0.88));
    assert_eq!(b["publishedUnit"], 1);

    // Correction de l'AFC : le cours retenu reste l'ancien.
    publier(&pool, mois(2026, 2), &[("USD", 1, dec!(0.9000))]).await;
    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=USD&date=2026-02-10")
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.88));
    assert_eq!(dec_de(&b["publishedRate"]), dec!(0.9));

    // Jour sans cours : 404 avec `date`.
    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=USD&date=2026-03-10&kind=day")
        .await;
    assert_eq!(s, 404, "{b}");
    assert_eq!(
        b["error"]["details"],
        json!({ "currency": "USD", "month": "2026-03-01", "date": "2026-03-10" })
    );

    for (url, code) in [
        (
            "/api/v1/exchange-rates/lookup?currency=CHF&date=2026-01-10",
            "VALIDATION_ERROR",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=usd&date=2026-01-10",
            "CURRENCY_UNKNOWN",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=XYZ&date=2026-01-10",
            "CURRENCY_UNKNOWN",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=USD&date=2026-01-10&kind=week",
            "VALIDATION_ERROR",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=USD",
            "VALIDATION_ERROR",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=USD&date=2026-1-10",
            "VALIDATION_ERROR",
        ),
        (
            "/api/v1/exchange-rates/lookup?currency=USD&date=1999-12-31",
            "VALIDATION_ERROR",
        ),
    ] {
        let (s, b) = ctx.get(url).await;
        assert_eq!(s, 400, "{url} : {b}");
        assert_eq!(b["error"]["code"], code, "{url} : {b}");
    }
}

/// `GET /exchange-rates` : union des lignes de la société et des valeurs
/// publiées d'un mois ou d'un jour, triées, une devise à cours saisi seul
/// (`published` `null`), une devise publiée non retenue (`effectiveRate`,
/// `source`, `row` à `null`) ; filtre `currency` ; `month` et `date` ensemble
/// → 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_liste_d_un_mois_unit_lignes_et_valeurs_publiees(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-01-01",
        "1.1",
        "manual",
        Some(ctx.s.admin_user_id),
    )
    .await;
    publier(&pool, mois(2026, 1), &[("SEK", 100, dec!(8.4000))]).await;

    let (s, b) = ctx.get("/api/v1/exchange-rates?month=2026-01").await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["kind"], "month");
    assert_eq!(b["rateDate"], "2026-01-01");
    let rates = b["rates"].as_array().unwrap();
    assert_eq!(rates.len(), 2, "{b}");
    assert_eq!(rates[0]["currency"], "SEK");
    assert_eq!(rates[0]["effectiveRate"], Value::Null);
    assert_eq!(rates[0]["source"], Value::Null);
    assert_eq!(rates[0]["row"], Value::Null);
    assert_eq!(dec_de(&rates[0]["published"]["rate"]), dec!(0.084));
    assert_eq!(rates[0]["published"]["publishedUnit"], 100);
    assert_eq!(rates[1]["currency"], "USD");
    assert_eq!(rates[1]["source"], "manual");
    assert_eq!(rates[1]["published"], Value::Null);
    assert_eq!(rates[1]["row"]["updatedByKesh"], false);
    assert_eq!(rates[1]["corrected"], false);

    let (s, b) = ctx
        .get("/api/v1/exchange-rates?month=2026-01&currency=USD")
        .await;
    assert_eq!(s, 200);
    assert_eq!(b["rates"].as_array().unwrap().len(), 1);

    let (s, b) = ctx.get("/api/v1/exchange-rates?date=2026-01-15").await;
    assert_eq!(s, 200);
    assert_eq!(b["kind"], "day");
    assert!(b["rates"].as_array().unwrap().is_empty());

    for url in [
        "/api/v1/exchange-rates?month=2026-01&date=2026-01-15",
        "/api/v1/exchange-rates",
        "/api/v1/exchange-rates?month=2026-13",
        "/api/v1/exchange-rates?month=2026-01&currency=CHF",
    ] {
        let (s, _) = ctx.get(url).await;
        assert_eq!(s, 400, "{url}");
    }
}

// ============================================================
// PUT /exchange-rates/rates
// ============================================================

/// Saisie : création (`exchange_rate.created`, avant `null`), remplacement
/// d'une ligne `afc` acquittée (devient `manual`, acquittement effacé,
/// `exchange_rate.updated` avec avant et après), version absente sur ligne
/// existante ou périmée → 409, bornes du cours → `FX_RATE_INVALID`, `rateDate`
/// du 15 en `month` → 400, `CHF` → 400, `usd` → `CURRENCY_UNKNOWN`, cours égal
/// à la valeur publiée conservé `manual`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_saisie_d_un_cours(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;

    let (s, b) = ctx
        .put(RATES, corps_put("EUR", "month", "2026-04-01", "0.95", None))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "manual");
    assert_eq!(b["kind"], "month");
    assert_eq!(b["rateDate"], "2026-04-01");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.95));
    assert_eq!(b["row"]["version"], 1);
    let a = audits(&pool, "exchange_rate.created").await;
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["before"], Value::Null);
    assert_eq!(a[0]["after"]["source"], "manual");
    assert_eq!(a[0]["currency"], "EUR");
    assert_eq!(a[0]["rateDate"], "2026-04-01");

    // Remplacement d'une ligne `afc` acquittée.
    let id = ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-05-01",
        "0.88",
        "afc",
        None,
    )
    .await;
    sqlx::query(
        "UPDATE exchange_rates SET acknowledged_rate = 0.9, acknowledged_at = '2026-06-01' \
         WHERE id = ?",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let (s, b) = ctx
        .put(
            RATES,
            corps_put("USD", "month", "2026-05-01", "0.91", Some(1)),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "manual");
    assert_eq!(b["row"]["version"], 2);
    let ack: (Option<Decimal>, Option<chrono::NaiveDateTime>) = sqlx::query_as(
        "SELECT acknowledged_rate, acknowledged_at FROM exchange_rates WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(ack, (None, None), "acquittement effacé");
    let a = audits(&pool, "exchange_rate.updated").await;
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["before"]["source"], "afc");
    assert_eq!(a[0]["after"]["source"], "manual");

    // Version absente sur ligne existante, puis périmée.
    for version in [None, Some(1)] {
        let (s, b) = ctx
            .put(
                RATES,
                corps_put("USD", "month", "2026-05-01", "0.92", version),
            )
            .await;
        assert_eq!(s, 409, "{b}");
        assert_eq!(b["error"]["code"], "OPTIMISTIC_LOCK_CONFLICT");
    }

    // Bornes du cours.
    for cours in ["0", "-1", "0.12345678901", "1000001"] {
        let (s, b) = ctx
            .put(RATES, corps_put("GBP", "month", "2026-04-01", cours, None))
            .await;
        assert_eq!(s, 400, "{cours} : {b}");
        assert_eq!(b["error"]["code"], "FX_RATE_INVALID", "{cours}");
        assert_eq!(b["error"]["details"], json!({ "currency": "GBP" }));
    }
    for (corps, code) in [
        (
            corps_put("GBP", "month", "2026-04-15", "1", None),
            "VALIDATION_ERROR",
        ),
        (
            corps_put("CHF", "month", "2026-04-01", "1", None),
            "VALIDATION_ERROR",
        ),
        (
            corps_put("usd", "month", "2026-04-01", "1", None),
            "CURRENCY_UNKNOWN",
        ),
        (
            corps_put("GBP", "month", "2026-04-01", "abc", None),
            "VALIDATION_ERROR",
        ),
        (
            corps_put("GBP", "week", "2026-04-01", "1", None),
            "VALIDATION_ERROR",
        ),
        (
            corps_put("GBP", "day", "2100-01-01", "1", None),
            "VALIDATION_ERROR",
        ),
        (json!({ "currency": "GBP" }), "VALIDATION_ERROR"),
    ] {
        let (s, b) = ctx.put(RATES, corps.clone()).await;
        assert_eq!(s, 400, "{corps} : {b}");
        assert_eq!(b["error"]["code"], code, "{corps} : {b}");
    }
    // `usd` refusé avant les bornes du cours (ordre des refus).
    let (_, b) = ctx
        .put(RATES, corps_put("usd", "month", "2026-04-01", "0", None))
        .await;
    assert_eq!(b["error"]["code"], "CURRENCY_UNKNOWN");

    // Cours saisi égal à la valeur publiée : conservé `manual`.
    publier(&pool, mois(2026, 6), &[("USD", 1, dec!(0.8800))]).await;
    let (s, b) = ctx
        .put(RATES, corps_put("USD", "month", "2026-06-01", "0.88", None))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "manual");
    assert_eq!(dec_de(&b["published"]["rate"]), dec!(0.88));
    assert_eq!(b["corrected"], false);
}

/// Deux créations simultanées du même cours : un 200 et un 409, jamais 500.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn deux_creations_simultanees(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let corps = corps_put("EUR", "day", "2026-03-31", "0.95", None);
    let (a, b) = tokio::join!(ctx.put(RATES, corps.clone()), ctx.put(RATES, corps.clone()));
    let mut statuts = [a.0, b.0];
    statuts.sort();
    assert_eq!(statuts, [200, 409], "{} / {}", a.1, b.1);
    assert_eq!(
        compter(&pool, "SELECT COUNT(*) FROM exchange_rates").await,
        1
    );
}

// ============================================================
// POST /exchange-rates/rates/reset (↑)
// ============================================================

/// Le ↑ : `manual` → `afc` à la valeur publiée ; `afc` corrigée → nouvelle
/// valeur ; case vide → ligne `afc` créée ; `manual` sans valeur publiée →
/// supprimée ; `afc` sans valeur publiée → 404 ; `publishedRate` ou `version`
/// périmés → 409 ; ligne existante sans `version` → 409 ; `version` sur une
/// case vide → 400 ; audit `exchange_rate.reset`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_reset_reprend_la_valeur_publiee(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let corps = |devise: &str, date: &str, version: Option<i32>, publiee: Option<&str>| {
        let mut b = json!({ "currency": devise, "kind": "month", "rateDate": date });
        if let Some(v) = version {
            b["version"] = json!(v);
        }
        if let Some(p) = publiee {
            b["publishedRate"] = json!(p);
        }
        b
    };
    publier(
        &pool,
        mois(2026, 1),
        &[
            ("USD", 1, dec!(0.9000)),
            ("EUR", 1, dec!(0.9300)),
            ("GBP", 1, dec!(1.1200)),
        ],
    )
    .await;

    // `manual` → `afc`.
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-01-01",
        "1.1",
        "manual",
        None,
    )
    .await;
    let (s, b) = ctx
        .post(RESET, corps("USD", "2026-01-01", Some(1), Some("0.9")))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "afc");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.9));
    assert_eq!(b["row"]["version"], 2);
    assert_eq!(b["row"]["updatedByKesh"], false);
    let a = audits(&pool, "exchange_rate.reset").await;
    assert_eq!(a.len(), 1);
    assert_eq!(a[0]["before"]["source"], "manual");
    assert_eq!(a[0]["after"]["source"], "afc");
    assert_eq!(dec_de(&a[0]["publishedRate"]), dec!(0.9));

    // `afc` corrigée → nouvelle valeur.
    ligne(
        &pool,
        ctx.c(),
        "EUR",
        "month",
        "2026-01-01",
        "0.92",
        "afc",
        None,
    )
    .await;
    let (s, b) = ctx
        .post(RESET, corps("EUR", "2026-01-01", Some(1), None))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.93));
    assert_eq!(b["corrected"], false);

    // Ligne existante sans version → 409 ; version sur case vide → 400.
    let (s, b) = ctx
        .post(RESET, corps("EUR", "2026-01-01", None, None))
        .await;
    assert_eq!(s, 409, "{b}");
    let (s, b) = ctx
        .post(RESET, corps("GBP", "2026-01-01", Some(1), None))
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], "VALIDATION_ERROR");
    // `publishedRate` périmé → 409 ; case vide → ligne `afc` créée.
    let (s, b) = ctx
        .post(RESET, corps("GBP", "2026-01-01", None, Some("1.1")))
        .await;
    assert_eq!(s, 409, "{b}");
    let (s, b) = ctx
        .post(RESET, corps("GBP", "2026-01-01", None, Some("1.12")))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "afc");
    assert_eq!(b["row"]["version"], 1);
    // Version périmée → 409.
    let (s, _) = ctx
        .post(RESET, corps("GBP", "2026-01-01", Some(7), None))
        .await;
    assert_eq!(s, 409);

    // Sans valeur publiée : `manual` supprimée, `afc` → 404, rien → 404.
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-02-01",
        "1.2",
        "manual",
        None,
    )
    .await;
    let (s, b) = ctx
        .post(RESET, corps("USD", "2026-02-01", Some(1), None))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["effectiveRate"], Value::Null);
    assert_eq!(b["row"], Value::Null);
    assert!(
        retenu(&pool, ctx.c(), "USD", "month", "2026-02-01")
            .await
            .is_none()
    );
    ligne(
        &pool,
        ctx.c(),
        "EUR",
        "month",
        "2026-02-01",
        "0.9",
        "afc",
        None,
    )
    .await;
    let (s, b) = ctx
        .post(RESET, corps("EUR", "2026-02-01", Some(1), None))
        .await;
    assert_eq!(s, 404, "{b}");
    assert_eq!(b["error"]["code"], "NOT_FOUND");
    let (s, _) = ctx
        .post(RESET, corps("GBP", "2026-02-01", None, None))
        .await;
    assert_eq!(s, 404);
    assert_eq!(audits(&pool, "exchange_rate.reset").await.len(), 4);
}

// ============================================================
// Mode, rôles
// ============================================================

/// Société mono-devise : `409 MULTI_CURRENCY_DISABLED` sur toutes les routes
/// des cours ; le tableau annuel et les genres répondent avec `fiscalYearId`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_societe_mono_devise_est_refusee(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let jwt = ctx.jwt.clone();
    for (methode, url, corps) in [
        (
            reqwest::Method::GET,
            "/api/v1/exchange-rates?month=2026-01".to_string(),
            None,
        ),
        (
            reqwest::Method::GET,
            "/api/v1/exchange-rates/lookup?currency=USD&date=2026-01-01".to_string(),
            None,
        ),
        (
            reqwest::Method::PUT,
            RATES.to_string(),
            Some(corps_put("USD", "month", "2026-01-01", "1", None)),
        ),
        (
            reqwest::Method::POST,
            RESET.to_string(),
            Some(json!({ "currency": "USD", "kind": "month", "rateDate": "2026-01-01" })),
        ),
        (reqwest::Method::GET, NOTICES.to_string(), None),
        (
            reqwest::Method::POST,
            ACK.to_string(),
            Some(json!({ "notices": [{ "currency": "USD", "kind": "month",
                "rateDate": "2026-01-01", "publishedRate": "1" }] })),
        ),
    ] {
        let (s, b) = ctx.envoyer(methode, &url, &jwt, corps).await;
        assert_eq!(s, 409, "{url} : {b}");
        assert_eq!(b["error"]["code"], "MULTI_CURRENCY_DISABLED", "{url}");
    }
    let fy = ctx.fy();
    let (s, b) = ctx
        .get(&format!("/api/v1/fiscal-years/{fy}/exchange-rates"))
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["details"], json!({ "fiscalYearId": fy }));
    let (s, b) = ctx
        .put(
            &format!("/api/v1/fiscal-years/{fy}/rate-kinds"),
            json!({ "openingRateKind": "month", "closingRateKind": "month" }),
        )
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["details"], json!({ "fiscalYearId": fy }));
}

/// Consultation : 403 sur les routes de `comptable_routes` (saisie, ↑,
/// notices et acquittement, genres) ; les lectures lui restent ouvertes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_consultation_lit_mais_n_ecrit_pas(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let jwt = forge_jwt(ctx.s.admin_user_id, "Consultation", ctx.c());
    let fy = ctx.fy();
    for (methode, url, corps) in [
        (
            reqwest::Method::PUT,
            RATES.to_string(),
            Some(corps_put("USD", "month", "2026-01-01", "1", None)),
        ),
        (
            reqwest::Method::POST,
            RESET.to_string(),
            Some(json!({ "currency": "USD", "kind": "month", "rateDate": "2026-01-01" })),
        ),
        (reqwest::Method::GET, NOTICES.to_string(), None),
        (
            reqwest::Method::POST,
            ACK.to_string(),
            Some(json!({ "notices": [] })),
        ),
        (
            reqwest::Method::PUT,
            format!("/api/v1/fiscal-years/{fy}/rate-kinds"),
            Some(json!({ "openingRateKind": "month", "closingRateKind": "month" })),
        ),
    ] {
        let (s, _) = ctx.envoyer(methode, &url, &jwt, corps).await;
        assert_eq!(s, 403, "{url}");
    }
    for url in [
        "/api/v1/exchange-rates?month=2026-01".to_string(),
        format!("/api/v1/fiscal-years/{fy}/exchange-rates"),
    ] {
        let (s, b) = ctx.envoyer(reqwest::Method::GET, &url, &jwt, None).await;
        assert_eq!(s, 200, "{url} : {b}");
    }
    let (s, _) = ctx
        .envoyer(
            reqwest::Method::GET,
            "/api/v1/exchange-rates/lookup?currency=USD&date=2026-01-01",
            &jwt,
            None,
        )
        .await;
    assert_eq!(
        s, 404,
        "lookup ouvert à la Consultation (aucun cours : 404)"
    );
}

// ============================================================
// Corrections de l'AFC (notices, acquittement), deux sociétés
// ============================================================

/// Deux sociétés A et B, ligne `afc` USD décembre dans chacune : correction →
/// notice dans A et B ; acquittement dans A → absente de A, présente dans B ;
/// ↑ dans B → B à la nouvelle valeur, A inchangée ; nouvelle correction →
/// notice de retour dans A ; ligne `manual` qui diffère → aucune notice ;
/// acquittement d'une valeur changée entre-temps → rien (aucun audit).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_corrections_s_annoncent_par_societe(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let (b_id, b_jwt) = societe_b(&pool).await;
    publier(
        &pool,
        mois(2025, 12),
        &[("USD", 1, dec!(0.8800)), ("EUR", 1, dec!(0.9300))],
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2025-12-01",
        "0.88",
        "afc",
        None,
    )
    .await;
    ligne(
        &pool,
        b_id,
        "USD",
        "month",
        "2025-12-01",
        "0.88",
        "afc",
        None,
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "EUR",
        "month",
        "2025-12-01",
        "0.95",
        "manual",
        None,
    )
    .await;

    let (s, b) = ctx.get(NOTICES).await;
    assert_eq!(s, 200);
    assert_eq!(
        b,
        json!({ "notices": [] }),
        "aucune correction, et `manual` ne s'annonce pas"
    );

    publier(
        &pool,
        mois(2025, 12),
        &[("USD", 1, dec!(0.9000)), ("EUR", 1, dec!(0.9400))],
    )
    .await;
    let notices_de = |jwt: String| {
        let ctx = &ctx;
        async move {
            let (s, b) = ctx.envoyer(reqwest::Method::GET, NOTICES, &jwt, None).await;
            assert_eq!(s, 200, "{b}");
            b["notices"].as_array().unwrap().clone()
        }
    };
    let a = notices_de(ctx.jwt.clone()).await;
    assert_eq!(a.len(), 1, "{a:?}");
    assert_eq!(a[0]["currency"], "USD");
    assert_eq!(a[0]["rateDate"], "2025-12-01");
    assert_eq!(dec_de(&a[0]["effectiveRate"]), dec!(0.88));
    assert_eq!(dec_de(&a[0]["publishedRate"]), dec!(0.9));
    assert_eq!(notices_de(b_jwt.clone()).await.len(), 1);

    // Acquittement d'une valeur périmée : rien, aucun audit.
    let ack = |publiee: &str| {
        json!({ "notices": [{ "currency": "USD", "kind": "month", "rateDate": "2025-12-01",
            "publishedRate": publiee }] })
    };
    let (s, b) = ctx.post(ACK, ack("0.89")).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b, json!({ "acknowledged": 0 }));
    assert!(
        audits(&pool, "exchange_rates.corrections_acknowledged")
            .await
            .is_empty()
    );

    let (s, b) = ctx.post(ACK, ack("0.90")).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b, json!({ "acknowledged": 1 }));
    assert!(notices_de(ctx.jwt.clone()).await.is_empty());
    assert_eq!(notices_de(b_jwt.clone()).await.len(), 1);
    let au = audits(&pool, "exchange_rates.corrections_acknowledged").await;
    assert_eq!(au.len(), 1);
    assert_eq!(au[0]["notices"][0]["currency"], "USD");
    let entite: (String, i64) = sqlx::query_as(
        "SELECT entity_type, entity_id FROM audit_log \
         WHERE action = 'exchange_rates.corrections_acknowledged'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(entite, ("company".to_string(), ctx.c()));
    // L'acquittement ne change ni le cours, ni la source, ni la version.
    let (rate, source, version): (Decimal, String, i32) = sqlx::query_as(
        "SELECT rate, source, version FROM exchange_rates WHERE company_id = ? AND currency = 'USD'",
    )
    .bind(ctx.c())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((rate, source.as_str(), version), (dec!(0.88), "afc", 1));

    // ↑ dans B : B à la nouvelle valeur, A inchangée.
    let (s, b) = ctx
        .envoyer(
            reqwest::Method::POST,
            RESET,
            &b_jwt,
            Some(
                json!({ "currency": "USD", "kind": "month", "rateDate": "2025-12-01",
                "version": 1, "publishedRate": "0.9" }),
            ),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        retenu(&pool, b_id, "USD", "month", "2025-12-01")
            .await
            .unwrap()
            .0,
        dec!(0.9)
    );
    assert_eq!(
        retenu(&pool, ctx.c(), "USD", "month", "2025-12-01")
            .await
            .unwrap()
            .0,
        dec!(0.88)
    );
    assert!(notices_de(b_jwt.clone()).await.is_empty());

    // Nouvelle correction : la notice revient dans A.
    publier(&pool, mois(2025, 12), &[("USD", 1, dec!(0.9100))]).await;
    assert_eq!(notices_de(ctx.jwt.clone()).await.len(), 1);

    // Bornes du corps.
    let (s, _) = ctx.post(ACK, json!({ "notices": [] })).await;
    assert_eq!(s, 400);
    let trop: Vec<Value> = (0..501)
        .map(|_| {
            json!({ "currency": "USD", "kind": "month", "rateDate": "2025-12-01",
            "publishedRate": "1" })
        })
        .collect();
    let (s, _) = ctx.post(ACK, json!({ "notices": trop })).await;
    assert_eq!(s, 400);
}

// ============================================================
// Source inconnue (A30)
// ============================================================

/// Ligne `source = 'ecb'` posée par SQL : lue telle quelle par `lookup` et la
/// grille (aucune erreur interne), devise sans compte présente dans la grille
/// sans `all=true` ; son `PUT` → `manual`, son ↑ → `afc`, son ↑ sans valeur
/// publiée → supprimée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_source_inconnue_se_lit_telle_quelle(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    exercice(
        &pool,
        ctx.fy(),
        d(2026, 1, 1),
        d(2026, 12, 31),
        "month",
        "month",
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "NOK",
        "month",
        "2026-03-01",
        "0.08",
        "ecb",
        None,
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "NOK",
        "month",
        "2026-04-01",
        "0.08",
        "ecb",
        None,
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "NOK",
        "month",
        "2026-05-01",
        "0.08",
        "ecb",
        None,
    )
    .await;

    let (s, b) = ctx
        .get("/api/v1/exchange-rates/lookup?currency=NOK&date=2026-03-05")
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "ecb");
    let fy = ctx.fy();
    let (s, b) = ctx
        .get(&format!("/api/v1/fiscal-years/{fy}/exchange-rates"))
        .await;
    assert_eq!(s, 200, "{b}");
    let nok = b["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == "NOK")
        .expect("ligne NOK sans all=true");
    assert_eq!(nok["hasAccount"], false);
    assert_eq!(nok["months"][2]["source"], "ecb");

    let (s, b) = ctx
        .put(
            RATES,
            corps_put("NOK", "month", "2026-03-01", "0.09", Some(1)),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "manual");
    publier(&pool, mois(2026, 4), &[("NOK", 100, dec!(8.5000))]).await;
    let (s, b) = ctx
        .post(
            RESET,
            json!({ "currency": "NOK", "kind": "month", "rateDate": "2026-04-01", "version": 1 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["source"], "afc");
    assert_eq!(dec_de(&b["effectiveRate"]), dec!(0.085));
    let (s, b) = ctx
        .post(
            RESET,
            json!({ "currency": "NOK", "kind": "month", "rateDate": "2026-05-01", "version": 1 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert!(
        retenu(&pool, ctx.c(), "NOK", "month", "2026-05-01")
            .await
            .is_none()
    );
}

// ============================================================
// Tableau annuel (AC-5, R6)
// ============================================================

/// Colonnes, `openingDate`, `closingDate` d'exercices au 1er janvier, au 1er
/// avril, au 1er juillet, au 15.03, de six et de dix-huit mois, dans les deux
/// genres ; ordre de l'exercice.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn le_tableau_suit_la_duree_de_l_exercice(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let fy = ctx.fy();
    // (début, fin, genres, nb colonnes, première, dernière, ouverture, clôture)
    let cas = [
        (
            d(2026, 1, 1),
            d(2026, 12, 31),
            "month",
            12,
            "2026-01-01",
            "2026-12-01",
            "2025-12-01",
            "2026-12-01",
        ),
        (
            d(2026, 1, 1),
            d(2026, 12, 31),
            "day",
            12,
            "2026-01-01",
            "2026-12-01",
            "2025-12-31",
            "2026-12-31",
        ),
        (
            d(2026, 4, 1),
            d(2027, 3, 31),
            "month",
            12,
            "2026-04-01",
            "2027-03-01",
            "2026-03-01",
            "2027-03-01",
        ),
        (
            d(2026, 7, 1),
            d(2027, 6, 30),
            "month",
            12,
            "2026-07-01",
            "2027-06-01",
            "2026-06-01",
            "2027-06-01",
        ),
        (
            d(2026, 7, 1),
            d(2027, 6, 30),
            "day",
            12,
            "2026-07-01",
            "2027-06-01",
            "2026-06-30",
            "2027-06-30",
        ),
        (
            d(2024, 3, 15),
            d(2025, 3, 14),
            "month",
            13,
            "2024-03-01",
            "2025-03-01",
            "2024-03-01",
            "2025-03-01",
        ),
        (
            d(2024, 3, 15),
            d(2025, 3, 14),
            "day",
            13,
            "2024-03-01",
            "2025-03-01",
            "2024-03-14",
            "2025-03-14",
        ),
        (
            d(2026, 1, 1),
            d(2026, 6, 30),
            "month",
            6,
            "2026-01-01",
            "2026-06-01",
            "2025-12-01",
            "2026-06-01",
        ),
        (
            d(2026, 1, 1),
            d(2027, 6, 30),
            "month",
            18,
            "2026-01-01",
            "2027-06-01",
            "2025-12-01",
            "2027-06-01",
        ),
        (
            d(2026, 1, 1),
            d(2027, 6, 30),
            "day",
            18,
            "2026-01-01",
            "2027-06-01",
            "2025-12-31",
            "2027-06-30",
        ),
    ];
    for (debut, fin, genre, n, premier, dernier, ouverture, cloture) in cas {
        exercice(&pool, fy, debut, fin, genre, genre).await;
        let (s, b) = ctx
            .get(&format!("/api/v1/fiscal-years/{fy}/exchange-rates"))
            .await;
        assert_eq!(s, 200, "{b}");
        let months = b["months"].as_array().unwrap();
        assert_eq!(months.len(), n, "{debut} → {fin}");
        assert_eq!(months[0], premier, "{debut}");
        assert_eq!(months[n - 1], dernier, "{debut}");
        let tries: Vec<&str> = months.iter().map(|m| m.as_str().unwrap()).collect();
        let mut ordre = tries.clone();
        ordre.sort();
        assert_eq!(tries, ordre, "ordre de l'exercice");
        assert_eq!(b["openingDate"], ouverture, "{debut} {genre}");
        assert_eq!(b["closingDate"], cloture, "{debut} {genre}");
        assert_eq!(b["openingRateKind"], genre);
        let usd = &b["rows"][0];
        assert_eq!(usd["currency"], "USD");
        assert_eq!(usd["months"].as_array().unwrap().len(), n);
        assert_eq!(usd["opening"]["rateDate"], ouverture);
        assert_eq!(usd["opening"]["kind"], genre);
        assert_eq!(usd["closing"]["rateDate"], cloture);
    }
}

/// Lignes du tableau : devise d'un compte actif (`hasAccount`), devise à
/// ligne `manual` sans compte, compte archivé qui a une ligne (présent) ou
/// n'en a pas (absent), devise publiée seule (présente avec `all=true`) ;
/// case `corrected` et `acknowledged` ; cases d'Ouverture et de Clôture
/// partagées par deux exercices consécutifs ; exercice d'une autre société →
/// 404 ; exercice mono-devise d'une société multi-devises → 409.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_lignes_et_cases_du_tableau(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let fy = ctx.fy();
    exercice(&pool, fy, d(2026, 1, 1), d(2026, 12, 31), "month", "month").await;
    compte(&pool, ctx.c(), "1031", Some("JPY"), false).await;
    compte(&pool, ctx.c(), "1032", Some("CAD"), false).await;
    ligne(
        &pool,
        ctx.c(),
        "JPY",
        "month",
        "2026-03-01",
        "0.0058",
        "afc",
        None,
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "EUR",
        "month",
        "2026-02-01",
        "0.95",
        "manual",
        None,
    )
    .await;
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2026-01-01",
        "0.88",
        "afc",
        None,
    )
    .await;
    publier(
        &pool,
        mois(2026, 1),
        &[("USD", 1, dec!(0.9000)), ("SEK", 100, dec!(8.4000))],
    )
    .await;
    sqlx::query(
        "UPDATE exchange_rates SET acknowledged_rate = 0.9, \
             acknowledged_at = '2026-02-01 10:00:00' WHERE company_id = ? AND currency = 'USD'",
    )
    .bind(ctx.c())
    .execute(&pool)
    .await
    .unwrap();

    let url = format!("/api/v1/fiscal-years/{fy}/exchange-rates");
    let (s, b) = ctx.get(&url).await;
    assert_eq!(s, 200, "{b}");
    let devises: Vec<(&str, bool)> = b["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["currency"].as_str().unwrap(),
                r["hasAccount"].as_bool().unwrap(),
            )
        })
        .collect();
    assert_eq!(devises, vec![("EUR", false), ("JPY", true), ("USD", true)]);
    let usd = &b["rows"][2];
    let janvier = &usd["months"][0];
    assert_eq!(janvier["source"], "afc");
    assert_eq!(janvier["corrected"], true);
    assert_eq!(janvier["acknowledged"], true);
    assert_eq!(janvier["row"]["updatedByKesh"], true);
    assert_eq!(dec_de(&janvier["published"]["rate"]), dec!(0.9));
    assert_eq!(usd["months"][1]["effectiveRate"], Value::Null);

    let (s, b) = ctx.get(&format!("{url}?all=true")).await;
    assert_eq!(s, 200);
    let toutes: Vec<&str> = b["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["currency"].as_str().unwrap())
        .collect();
    assert_eq!(toutes, vec!["EUR", "JPY", "SEK", "USD"]);

    // Exercices consécutifs : la Clôture `month` de 2026 et l'Ouverture
    // `month` de 2027 sont une seule case.
    let suivant = nouvel_exercice(
        &pool,
        ctx.c(),
        "Exercice 2027",
        d(2027, 1, 1),
        d(2027, 12, 31),
        "Open",
        true,
        "month",
    )
    .await;
    sqlx::query("UPDATE fiscal_years SET end_date = '2026-12-31' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();
    let (s, _) = ctx
        .put(RATES, corps_put("USD", "month", "2026-12-01", "0.87", None))
        .await;
    assert_eq!(s, 200);
    let (_, b1) = ctx.get(&url).await;
    let (_, b2) = ctx
        .get(&format!("/api/v1/fiscal-years/{suivant}/exchange-rates"))
        .await;
    let usd2 = b2["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == "USD")
        .unwrap()
        .clone();
    assert_eq!(
        b1["rows"][2]["closing"]["row"]["id"],
        usd2["opening"]["row"]["id"]
    );
    assert_eq!(dec_de(&usd2["opening"]["effectiveRate"]), dec!(0.87));

    // Autre société → 404 ; exercice mono-devise d'une société multi → 409.
    let (b_id, _) = societe_b(&pool).await;
    let autre = nouvel_exercice(
        &pool,
        b_id,
        "B 2026",
        d(2026, 1, 1),
        d(2026, 12, 31),
        "Open",
        true,
        "day",
    )
    .await;
    let (s, _) = ctx
        .get(&format!("/api/v1/fiscal-years/{autre}/exchange-rates"))
        .await;
    assert_eq!(s, 404);
    let mono = nouvel_exercice(
        &pool,
        ctx.c(),
        "Exercice 2015",
        d(2015, 1, 1),
        d(2015, 12, 31),
        "Open",
        false,
        "day",
    )
    .await;
    let (s, b) = ctx
        .get(&format!("/api/v1/fiscal-years/{mono}/exchange-rates"))
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["code"], "MULTI_CURRENCY_DISABLED");
    assert_eq!(b["error"]["details"], json!({ "fiscalYearId": mono }));
    let (s, b) = ctx
        .put(
            &format!("/api/v1/fiscal-years/{mono}/rate-kinds"),
            json!({ "openingRateKind": "month", "closingRateKind": "month" }),
        )
        .await;
    assert_eq!(s, 409, "{b}");
    assert_eq!(b["error"]["details"], json!({ "fiscalYearId": mono }));
}

// ============================================================
// PUT /fiscal-years/{id}/rate-kinds (AC-5, U4)
// ============================================================

/// Instantané des écritures (en-têtes et lignes), pour prouver qu'aucune n'est
/// modifiée.
async fn ecritures(pool: &MySqlPool) -> Vec<String> {
    let mut v: Vec<String> = sqlx::query_scalar(
        "SELECT CONCAT_WS('|', id, fiscal_year_id, entry_number, entry_date, description, \
             updated_at) FROM journal_entries ORDER BY id",
    )
    .fetch_all(pool)
    .await
    .unwrap();
    v.extend(
        sqlx::query_scalar::<_, String>(
            "SELECT CONCAT_WS('|', id, entry_id, account_id, debit, credit) \
             FROM journal_entry_lines ORDER BY id",
        )
        .fetch_all(pool)
        .await
        .unwrap(),
    );
    v
}

/// Genres : clôture `day` → `month` avec exercice suivant ouvert et
/// `applyToNextOpening` → ouverture du suivant en `month`, une ligne d'audit
/// avec les deux exercices, aucune écriture modifiée ; même geste à `false`
/// ou sans le champ → suivant inchangé ; suivant clos → `applied: false` ;
/// exercice séparé par un trou → pas de suivant ; ouverture seule → rien au
/// suivant ; no-op → 200 sans audit ; clos → `FISCAL_YEAR_CLOSED` ; corps
/// illisible → 400 ; les unités neuves reçoivent la copie.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn les_genres_et_l_exercice_suivant(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let fy = ctx.fy();
    exercice(&pool, fy, d(2025, 1, 1), d(2025, 12, 31), "month", "day").await;
    let suivant = nouvel_exercice(
        &pool,
        ctx.c(),
        "Exercice 2026",
        d(2026, 1, 1),
        d(2026, 12, 31),
        "Open",
        true,
        "day",
    )
    .await;
    // Une écriture, témoin de « aucune écriture modifiée ».
    let je = sqlx::query(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, \
             journal, description) VALUES (?, ?, 1, '2025-03-01', 'OD', 'Témoin')",
    )
    .bind(ctx.c())
    .bind(fy)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    for (ordre, compte, debit, credit) in [(1, "1000", "10", "0"), (2, "3000", "0", "10")] {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je)
        .bind(ctx.s.accounts[compte])
        .bind(ordre)
        .bind(Decimal::from_str(debit).unwrap())
        .bind(Decimal::from_str(credit).unwrap())
        .execute(&pool)
        .await
        .unwrap();
    }
    let avant = ecritures(&pool).await;
    // Un cours d'ouverture `day` déjà publié (la veille du 01.01.2025) : unité
    // neuve quand l'ouverture passe en `day`, copiée par le changement.
    publier(
        &pool,
        RateUnit::day(d(2024, 12, 31)),
        &[("USD", 1, dec!(0.8800)), ("SEK", 100, dec!(8.4))],
    )
    .await;
    let url = format!("/api/v1/fiscal-years/{fy}/rate-kinds");
    let genre_suivant = |pool: MySqlPool| async move {
        sqlx::query_scalar::<_, String>(
            "SELECT CAST(opening_rate_kind AS CHAR) FROM fiscal_years WHERE id = ?",
        )
        .bind(suivant)
        .fetch_one(&pool)
        .await
        .unwrap()
    };

    // Sans le champ : le suivant ne bouge pas, même si la clôture change.
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "day", "closingRateKind": "month" }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["fiscalYear"]["closingRateKind"], "month");
    assert_eq!(b["fiscalYear"]["openingRateKind"], "day");
    assert_eq!(b["nextFiscalYear"], Value::Null);
    assert_eq!(genre_suivant(pool.clone()).await, "day");
    let a = audits(&pool, "fiscal_year.rate_kinds_updated").await;
    assert_eq!(a.len(), 1);
    assert_eq!(
        a[0]["before"],
        json!({ "openingRateKind": "month", "closingRateKind": "day" })
    );
    assert_eq!(
        a[0]["after"],
        json!({ "openingRateKind": "day", "closingRateKind": "month" })
    );
    assert_eq!(a[0]["nextFiscalYear"], Value::Null);
    assert_eq!(
        a[0]["copied"], 1,
        "USD du 31.12.2024 copié, SEK sans compte non"
    );
    assert!(
        retenu(&pool, ctx.c(), "USD", "day", "2024-12-31")
            .await
            .is_some()
    );
    assert!(
        retenu(&pool, ctx.c(), "SEK", "day", "2024-12-31")
            .await
            .is_none()
    );

    // No-op : 200, aucun audit.
    let (s, _) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "day", "closingRateKind": "month",
                "applyToNextOpening": true }),
        )
        .await;
    assert_eq!(s, 200);
    assert_eq!(
        audits(&pool, "fiscal_year.rate_kinds_updated").await.len(),
        1
    );

    // Retour en `day` à `false` : suivant inchangé.
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "day", "closingRateKind": "day",
                "applyToNextOpening": false }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(genre_suivant(pool.clone()).await, "day");

    // Clôture `day` → `month`, appliquée au suivant ouvert.
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "day", "closingRateKind": "month",
                "applyToNextOpening": true }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["nextFiscalYear"],
        json!({ "id": suivant, "openingRateKind": "month", "applied": true })
    );
    assert_eq!(genre_suivant(pool.clone()).await, "month");
    let a = audits(&pool, "fiscal_year.rate_kinds_updated").await;
    assert_eq!(a.len(), 3);
    assert_eq!(
        a[2]["nextFiscalYear"],
        json!({ "id": suivant, "before": { "openingRateKind": "day" },
            "after": { "openingRateKind": "month" } })
    );

    // Ouverture seule : rien au suivant.
    sqlx::query("UPDATE fiscal_years SET opening_rate_kind = 'day' WHERE id = ?")
        .bind(suivant)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "month", "closingRateKind": "month",
                "applyToNextOpening": true }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["nextFiscalYear"], Value::Null);
    assert_eq!(genre_suivant(pool.clone()).await, "day");

    // Suivant clos : inchangé, `applied: false`.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(suivant)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "month", "closingRateKind": "day",
                "applyToNextOpening": true }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        b["nextFiscalYear"],
        json!({ "id": suivant, "openingRateKind": "day", "applied": false, "reason": "closed" })
    );

    // Trou : pas de suivant.
    sqlx::query("UPDATE fiscal_years SET start_date = '2026-01-02', status = 'Open' WHERE id = ?")
        .bind(suivant)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "month", "closingRateKind": "month",
                "applyToNextOpening": true }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["nextFiscalYear"], Value::Null);
    assert_eq!(genre_suivant(pool.clone()).await, "day");

    assert_eq!(ecritures(&pool).await, avant, "aucune écriture modifiée");

    // Corps illisible, exercice clos, autre société.
    for corps in [
        json!({ "openingRateKind": "month" }),
        json!({ "openingRateKind": "week", "closingRateKind": "day" }),
    ] {
        let (s, b) = ctx.put(&url, corps).await;
        assert_eq!(s, 400, "{b}");
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR");
    }
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(fy)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .put(
            &url,
            json!({ "openingRateKind": "day", "closingRateKind": "day" }),
        )
        .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(b["error"]["code"], "FISCAL_YEAR_CLOSED");
    let (s, _) = ctx
        .put(
            "/api/v1/fiscal-years/999999/rate-kinds",
            json!({ "openingRateKind": "day", "closingRateKind": "day" }),
        )
        .await;
    assert_eq!(s, 404);
}

// ============================================================
// Copie (AC-3 bis) aux déclencheurs de 37-1
// ============================================================

/// Création d'un exercice multi-devises : les cours déjà récupérés de ses
/// unités utiles (un mois, l'Ouverture `month` du mois précédant l'exercice)
/// sont copiés pour les devises des comptes, compte archivé en JPY compris,
/// non pour SEK (aucun compte) ; `copied` au détail de `fiscal_year.created` ;
/// `updated_by` `NULL`, aucune ligne d'audit propre ; une ligne `manual`
/// présente est laissée telle quelle.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_creation_d_un_exercice_copie_les_cours(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    sqlx::query("UPDATE fiscal_years SET closing_rate_kind = 'month' WHERE id = ?")
        .bind(ctx.fy())
        .execute(&pool)
        .await
        .unwrap();
    compte(&pool, ctx.c(), "1031", Some("JPY"), false).await;
    publier(&pool, mois(2030, 12), &[("USD", 1, dec!(0.8800))]).await;
    publier(
        &pool,
        mois(2031, 3),
        &[
            ("USD", 1, dec!(0.8700)),
            ("JPY", 100, dec!(0.5800)),
            ("SEK", 100, dec!(8.4)),
        ],
    )
    .await;
    publier(&pool, mois(2031, 4), &[("USD", 1, dec!(0.8600))]).await;
    ligne(
        &pool,
        ctx.c(),
        "USD",
        "month",
        "2031-04-01",
        "1.5",
        "manual",
        None,
    )
    .await;

    let (s, b) = ctx
        .post(
            "/api/v1/fiscal-years",
            json!({ "name": "Exercice 2031", "startDate": "2031-01-01", "endDate": "2031-12-31" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["multiCurrency"], true);
    assert_eq!(b["openingRateKind"], "month");
    let a = audits(&pool, "fiscal_year.created").await;
    assert_eq!(a.last().unwrap()["copied"], 3, "{a:?}");
    let copiees: Vec<(String, NaiveDate, Option<i64>)> = sqlx::query_as(
        "SELECT currency, rate_date, updated_by FROM exchange_rates WHERE source = 'afc' \
         ORDER BY rate_date, currency",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(
        copiees,
        vec![
            ("USD".to_string(), d(2030, 12, 1), None),
            ("JPY".to_string(), d(2031, 3, 1), None),
            ("USD".to_string(), d(2031, 3, 1), None),
        ]
    );
    assert_eq!(
        retenu(&pool, ctx.c(), "USD", "month", "2031-04-01").await,
        Some((dec!(1.5), "manual".to_string())),
        "une ligne présente n'est jamais modifiée"
    );
    assert_eq!(
        compter(
            &pool,
            "SELECT COUNT(*) FROM audit_log WHERE entity_type = 'exchange_rate'"
        )
        .await,
        0,
        "aucune ligne d'audit propre à la copie"
    );
}

/// Un compte qui reçoit une devise : création d'un compte SEK (cours SEK
/// copiés, ceux d'une autre devise non), modification d'un compte CHF en SEK
/// (rien de neuf : `copied` 0), d'USD en EUR sur un compte sans ligne (EUR
/// copiés), `PUT` no-op (aucune copie, `copied` absent) ; sans exercice
/// multi-devises ouvert → rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_qui_recoit_une_devise_copie_ses_cours(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    publier(
        &pool,
        mois(2026, 5),
        &[
            ("SEK", 100, dec!(8.4)),
            ("EUR", 1, dec!(0.93)),
            ("GBP", 1, dec!(1.12)),
        ],
    )
    .await;

    let (s, b) = ctx
        .post(
            "/api/v1/accounts",
            json!({ "number": "1040", "name": "Banque SEK", "accountType": "Asset",
                "currency": "SEK" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(
        audits(&pool, "account.created").await.last().unwrap()["copied"],
        1
    );
    assert!(
        retenu(&pool, ctx.c(), "SEK", "month", "2026-05-01")
            .await
            .is_some()
    );
    assert!(
        retenu(&pool, ctx.c(), "EUR", "month", "2026-05-01")
            .await
            .is_none()
    );

    // CHF → SEK : SEK déjà portée, rien de neuf.
    let chf = compte(&pool, ctx.c(), "1041", None, true).await;
    let maj = |devise: Value, version: i32| {
        json!({ "name": "Compte 1041", "accountType": "Asset", "role": null, "postable": true,
            "currency": devise, "version": version })
    };
    let (s, b) = ctx
        .put(&format!("/api/v1/accounts/{chf}"), maj(json!("SEK"), 1))
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        audits(&pool, "account.updated").await.last().unwrap()["copied"],
        0
    );

    // USD → EUR sur un compte sans ligne : EUR copiés.
    let usd = compte(&pool, ctx.c(), "1042", Some("USD"), true).await;
    let (s, b) = ctx
        .put(
            &format!("/api/v1/accounts/{usd}"),
            json!({ "name": "Compte 1042", "accountType": "Asset", "role": null,
                "postable": true, "currency": "EUR", "version": 1 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        audits(&pool, "account.updated").await.last().unwrap()["copied"],
        1
    );
    assert!(
        retenu(&pool, ctx.c(), "EUR", "month", "2026-05-01")
            .await
            .is_some()
    );
    assert!(
        retenu(&pool, ctx.c(), "GBP", "month", "2026-05-01")
            .await
            .is_none()
    );

    // Renommage sans changement de devise : pas de `copied`.
    let (s, b) = ctx
        .put(
            &format!("/api/v1/accounts/{usd}"),
            json!({ "name": "Compte 1042 bis", "accountType": "Asset", "role": null,
                "postable": true, "version": 2 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert!(
        audits(&pool, "account.updated")
            .await
            .last()
            .unwrap()
            .get("copied")
            .is_none()
    );

    // Aucun exercice multi-devises ouvert : rien.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.fy())
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = ctx
        .post(
            "/api/v1/accounts",
            json!({ "number": "1043", "name": "Banque GBP", "accountType": "Asset",
                "currency": "GBP" }),
        )
        .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(
        audits(&pool, "account.created").await.last().unwrap()["copied"],
        0
    );
    assert!(
        retenu(&pool, ctx.c(), "GBP", "month", "2026-05-01")
            .await
            .is_none()
    );
}

/// L'activation du mode multi-devises ne copie rien (aucun compte ne porte de
/// devise) et son audit n'a pas de `copied`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn l_activation_ne_copie_rien(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    publier(&pool, mois(2026, 5), &[("USD", 1, dec!(0.88))]).await;
    let ids: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM accounts WHERE company_id = ? AND number IN ('6950', '1090') \
         ORDER BY number DESC",
    )
    .bind(ctx.c())
    .fetch_all(&pool)
    .await
    .unwrap();
    let (s, b) = ctx
        .put(
            "/api/v1/companies/current/currency-settings",
            json!({ "multiCurrencyEnabled": true, "fxGainAccountId": ids[0],
                "fxLossAccountId": ids[0], "fxTransitAccountId": ids[1], "version": 1 }),
        )
        .await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(
        compter(&pool, "SELECT COUNT(*) FROM exchange_rates").await,
        0
    );
    let a = audits(&pool, "company.currency_settings_updated").await;
    assert_eq!(a.len(), 1);
    assert!(a[0].get("copied").is_none(), "{}", a[0]);
}
