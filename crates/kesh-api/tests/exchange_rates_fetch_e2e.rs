//! Story 37-2 (AC-3, AC-3 bis, AC-4, AC-5, AC-7) — récupération des cours de
//! change par la route (`POST /api/v1/exchange-rates/fetch`,
//! `GET /api/v1/exchange-rates/status`), par la tâche automatique
//! (`exchange_rates::un_tour`) et à la connexion, contre la **source
//! factice** : aucun appel réseau.
//!
//! Horloge arrêtée sur le 16.11.2026, 10:00 UTC (lundi) : mois courant
//! novembre 2026, mois suivant décembre 2026.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use chrono::{DateTime, NaiveDate, TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::exchange_rates::source::{daily_xml, monthly_xml};
use kesh_api::exchange_rates::{
    Declencheur, Echec, MOCK_MONTHLY_RATES, MemoEntry, MockBehavior, MockRateSource, MockReply,
    SourceError, un_tour,
};
use kesh_api::{AppState, build_router};
use kesh_core::exchange_rates::{PublishedRate, RateKind, RateUnit};
use kesh_db::entities::AUDIT_ENTITY_ID_NONE;
use kesh_db::repositories::exchange_rates::{self as depot, FetchOutcomeStatus, SOURCE_AFC};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const FETCH: &str = "/api/v1/exchange-rates/fetch";
const STATUS: &str = "/api/v1/exchange-rates/status";

// ============================================================
// Harnais
// ============================================================

fn d(y: i32, m: u32, j: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, j).unwrap()
}

fn mois(y: i32, m: u32) -> RateUnit {
    RateUnit::month_of(d(y, m, 1))
}

/// L'instant de l'horloge des tests : lundi 16.11.2026, 10:00 UTC.
fn instant() -> DateTime<Utc> {
    "2026-11-16T10:00:00Z".parse().unwrap()
}

fn test_config(test_mode: bool) -> Config {
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
    .with_test_mode(test_mode)
}

struct Ctx {
    base_url: String,
    client: reqwest::Client,
    state: AppState,
    source: MockRateSource,
    s: SeededCompany,
    jwt: String,
}

impl Ctx {
    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    async fn post(&self, path: &str, jwt: &str, body: Value) -> (u16, Value) {
        let r = self
            .client
            .post(self.url(path))
            .bearer_auth(jwt)
            .json(&body)
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn fetch(&self, body: Value) -> (u16, Value) {
        self.post(FETCH, &self.jwt, body).await
    }

    async fn get(&self, path: &str, jwt: &str) -> (u16, Value) {
        let r = self
            .client
            .get(self.url(path))
            .bearer_auth(jwt)
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
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

/// Société seedée, rendue multi-devises (société et exercice 2020–2030), avec
/// un compte en USD ; source factice ; horloge arrêtée.
async fn monter(pool: &MySqlPool, test_mode: bool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
        .bind(s.company_id)
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    compte_en_devise(pool, s.company_id, "1020", "USD").await;

    let config = test_config(test_mode);
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
    let source = MockRateSource::new();
    let mut state =
        AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
    state.rate_source = Arc::new(source.clone());
    state.test_rate_source = test_mode.then(|| source.clone());
    let state = common::horloge::avec_horloge(state, instant());
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
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    Ctx {
        base_url: format!("http://{addr}"),
        client: reqwest::Client::new(),
        state,
        source,
        s,
        jwt,
    }
}

async fn compte_en_devise(pool: &MySqlPool, company_id: i64, numero: &str, devise: &str) {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency, active) \
         VALUES (?, ?, ?, 'Asset', ?, TRUE)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Banque {devise}"))
    .bind(devise)
    .execute(pool)
    .await
    .unwrap();
}

/// Redéfinit l'exercice seedé : dates et genres.
async fn exercice(pool: &MySqlPool, ctx: &Ctx, debut: NaiveDate, fin: NaiveDate) {
    sqlx::query(
        "UPDATE fiscal_years SET start_date = ?, end_date = ?, opening_rate_kind = 'month', \
         closing_rate_kind = 'month' WHERE id = ?",
    )
    .bind(debut)
    .bind(fin)
    .bind(ctx.s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
}

fn publie(code: &str, unite: i32, cours: Decimal) -> PublishedRate {
    PublishedRate {
        currency: kesh_core::currency::CurrencyCode::parse(code).unwrap(),
        published_unit: unite,
        published_rate: cours,
        rate_per_unit: cours / Decimal::from(unite),
    }
}

/// Écrit des valeurs publiées pour une unité, à l'instant donné.
async fn poser(pool: &MySqlPool, u: RateUnit, at: chrono::NaiveDateTime) {
    let mut tx = pool.begin().await.unwrap();
    let source_date = (u.kind == RateKind::Day).then_some(u.date);
    depot::write_fetched_unit_in_tx(
        &mut tx,
        SOURCE_AFC,
        u,
        source_date,
        &[publie("USD", 1, dec!(0.8800))],
        at,
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
}

async fn compte_lignes(pool: &MySqlPool, table: &str) -> i64 {
    sqlx::query_scalar(&format!("SELECT COUNT(*) FROM {table}"))
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn audits_fetched(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'exchange_rates.fetched'")
        .fetch_one(pool)
        .await
        .unwrap()
}

async fn etat(pool: &MySqlPool) -> Option<depot::FetchStatus> {
    let mut conn = pool.acquire().await.unwrap();
    depot::get_fetch_status(&mut conn).await.unwrap()
}

async fn retenu(pool: &MySqlPool, company_id: i64, devise: &str, u: RateUnit) -> Option<Decimal> {
    let mut conn = pool.acquire().await.unwrap();
    depot::find_company_rate_in_tx(&mut conn, company_id, devise, u.kind, u.date)
        .await
        .unwrap()
        .map(|r| r.rate)
}

fn erreur(source: SourceError) -> MockBehavior {
    MockBehavior::reply(MockReply::Error(source))
}

// ============================================================
// POST /fetch — refus
// ============================================================

/// Consultation → 403 ; Comptable admis.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_refuse_la_consultation(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let consultation = forge_jwt(ctx.s.admin_user_id, "Consultation", ctx.s.company_id);
    let (st, _) = ctx
        .post(FETCH, &consultation, json!({"year": 2026, "month": 11}))
        .await;
    assert_eq!(st, 403);
    assert_eq!(ctx.source.call_count(), 0);
    let comptable = forge_jwt(ctx.s.admin_user_id, "Comptable", ctx.s.company_id);
    let (st, _) = ctx
        .post(FETCH, &comptable, json!({"year": 2026, "month": 11}))
        .await;
    assert_eq!(st, 200);
    // `GET /status` : tout rôle authentifié.
    let (st, _) = ctx.get(STATUS, &consultation).await;
    assert_eq!(st, 200);
}

/// Société mono-devise → 409 `MULTI_CURRENCY_DISABLED`, avant les bornes (un
/// corps hors bornes rend aussi 409) ; `GET /status` aussi.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_refuse_la_societe_mono_devise(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    sqlx::query("UPDATE companies SET multi_currency_enabled = FALSE WHERE id = ?")
        .bind(ctx.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    for corps in [
        json!({"year": 2026, "month": 11}),
        json!({"year": 2030, "month": 1}),
    ] {
        let (st, b) = ctx.fetch(corps).await;
        assert_eq!(st, 409);
        assert_eq!(b["error"]["code"], "MULTI_CURRENCY_DISABLED");
    }
    let (st, b) = ctx.get(STATUS, &ctx.jwt).await;
    assert_eq!(st, 409);
    assert_eq!(b["error"]["code"], "MULTI_CURRENCY_DISABLED");
    assert_eq!(ctx.source.call_count(), 0);
}

/// Bornes de l'AC-5 → 400, sans appel : mois après le mois suivant, avant
/// 2000-01 ; jour d'aujourd'hui, de demain, avant 2000 ; deux formes ou
/// aucune ; corps illisible.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_bornes(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    for corps in [
        json!({"year": 2027, "month": 1}),
        json!({"year": 1999, "month": 12}),
        json!({"year": 2026, "month": 13}),
        json!({"date": "2026-11-16"}),
        json!({"date": "2026-11-17"}),
        json!({"date": "1999-12-31"}),
        json!({"date": "16.11.2026"}),
        json!({"year": 2026, "month": 11, "date": "2026-11-13"}),
        json!({}),
        json!({"year": "2026", "month": 11}),
    ] {
        let (st, b) = ctx.fetch(corps.clone()).await;
        assert_eq!(st, 400, "{corps}");
        assert_eq!(b["error"]["code"], "VALIDATION_ERROR", "{corps}");
    }
    assert_eq!(ctx.source.call_count(), 0);
    assert!(etat(&pool).await.is_none());
}

// ============================================================
// POST /fetch — succès, état, audit, copie
// ============================================================

/// Mois récupéré : 200, valeurs publiées écrites, copie pour la société (USD
/// seulement : la devise de ses comptes), état `manual`/`ok`, une ligne
/// d'audit sous l'utilisateur ; `GET /status` le dit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_mois_ecrit_copie_et_audite(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let (st, _) = ctx.get(STATUS, &ctx.jwt).await;
    assert_eq!(st, 200);
    let (st, b) = ctx.get(STATUS, &ctx.jwt).await;
    assert_eq!(st, 200);
    for champ in [
        "trigger",
        "lastRunAt",
        "status",
        "error",
        "kind",
        "rateDate",
    ] {
        assert!(
            b[champ].is_null(),
            "{champ} nul tant que rien n'est récupéré"
        );
    }

    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        b,
        json!({"kind": "month", "rateDate": "2026-11-01", "sourceDate": null,
               "count": MOCK_MONTHLY_RATES.len(), "rejected": 0, "corrected": 0})
    );
    assert_eq!(ctx.source.calls(), vec![mois(2026, 11)]);
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        MOCK_MONTHLY_RATES.len() as i64
    );
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88))
    );
    assert_eq!(
        compte_lignes(&pool, "exchange_rates").await,
        1,
        "seule la devise d'un compte de la société est copiée"
    );
    let e = etat(&pool).await.unwrap();
    assert_eq!(e.trigger, depot::FetchTrigger::Manual);
    assert_eq!(e.status, FetchOutcomeStatus::Ok);
    assert_eq!(e.error, None);
    let details = common::audit_details(
        &pool,
        "installation",
        AUDIT_ENTITY_ID_NONE,
        "exchange_rates.fetched",
    )
    .await
    .unwrap();
    assert_eq!(
        details,
        json!({"kind": "month", "rateDate": "2026-11-01", "sourceDate": null,
               "count": MOCK_MONTHLY_RATES.len(), "rejected": 0, "corrected": 0, "copied": 1})
    );
    let acteur: i64 =
        sqlx::query_scalar("SELECT user_id FROM audit_log WHERE action = 'exchange_rates.fetched'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(acteur, ctx.s.admin_user_id);

    let (st, b) = ctx.get(STATUS, &ctx.jwt).await;
    assert_eq!(st, 200);
    assert_eq!(b["trigger"], "manual");
    assert_eq!(b["status"], "ok");
    assert_eq!(b["kind"], "month");
    assert_eq!(b["rateDate"], "2026-11-01");
    assert!(b["lastRunAt"].as_str().unwrap().ends_with('Z'));
}

/// Correctif `fix-cours-afc-bgn` : le lev bulgare que l'AFC publie pour
/// 2024 (et la kuna de ses cours journaliers) n'est plus écarté — `rejected`
/// à 0, aucun `invalid_rates` à l'état ; la valeur publiée se lit et se
/// filtre par `GET /exchange-rates`, et un cours de la société se saisit dans
/// cette devise ; un compte, lui, ne s'ouvre toujours pas en BGN. Une
/// monnaie retirée que l'AFC ne publie pas (CUC) reste écartée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_retient_les_monnaies_retirees_publiees(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    ctx.source.set_for(
        mois(2024, 1),
        MockBehavior::reply(MockReply::Xml(monthly_xml(
            2024,
            1,
            &[("USD", 1, "0.8600"), ("BGN", 100, "44.0532")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"year": 2024, "month": 1})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["count"], 2, "{b}");
    assert_eq!(b["rejected"], 0, "{b}");
    assert_eq!(etat(&pool).await.unwrap().error, None);
    let publie: Option<Decimal> = sqlx::query_scalar(
        "SELECT rate_per_unit FROM _kesh_fetched_exchange_rates \
         WHERE currency = 'BGN' AND kind = 'month' AND rate_date = '2024-01-01'",
    )
    .fetch_optional(&pool)
    .await
    .unwrap();
    assert_eq!(publie, Some(dec!(0.440532)));

    let (st, b) = ctx
        .get(
            "/api/v1/exchange-rates?month=2024-01&currency=BGN",
            &ctx.jwt,
        )
        .await;
    assert_eq!(st, 200, "{b}");
    let cases = b["rates"].as_array().unwrap();
    assert_eq!(cases.len(), 1, "{b}");
    assert_eq!(cases[0]["currency"], "BGN");
    assert_eq!(cases[0]["published"]["publishedUnit"], 100, "{b}");

    // Un cours de la société se saisit en BGN (devise de cours).
    let r = ctx
        .client
        .put(ctx.url("/api/v1/exchange-rates/rates"))
        .bearer_auth(&ctx.jwt)
        .json(
            &json!({"currency": "BGN", "kind": "month", "rateDate": "2024-02-01",
                      "rate": "0.45"}),
        )
        .send()
        .await
        .unwrap();
    let st = r.status().as_u16();
    let b: Value = r.json().await.unwrap_or(Value::Null);
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "BGN", mois(2024, 2)).await,
        Some(dec!(0.45))
    );

    // Un compte ne s'ouvre pas dans une monnaie retirée.
    let (st, b) = ctx
        .post(
            "/api/v1/accounts",
            &ctx.jwt,
            json!({"number": "1031", "name": "Banque BGN", "accountType": "Asset",
                   "currency": "BGN"}),
        )
        .await;
    assert_eq!(st, 400, "{b}");
    assert_eq!(b["error"]["code"], "CURRENCY_UNKNOWN", "{b}");

    // Cours journalier : la kuna, encore publiée au 31.12.2024, est retenue.
    ctx.source.set_for(
        RateUnit::day(d(2024, 12, 31)),
        MockBehavior::reply(MockReply::Xml(daily_xml(
            d(2024, 12, 31),
            &[
                ("EUR", 1, "0.94000"),
                ("BGN", 100, "48.06000"),
                ("HRK", 100, "12.47000"),
            ],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"date": "2024-12-31"})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        (b["count"].clone(), b["rejected"].clone()),
        (json!(3), json!(0)),
        "{b}"
    );

    // Une monnaie retirée que l'AFC ne publie pas reste écartée.
    ctx.source.set_for(
        mois(2024, 3),
        MockBehavior::reply(MockReply::Xml(monthly_xml(
            2024,
            3,
            &[("USD", 1, "0.8800"), ("CUC", 1, "0.8800")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"year": 2024, "month": 3})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        (b["count"].clone(), b["rejected"].clone()),
        (json!(1), json!(1)),
        "{b}"
    );
    assert_eq!(
        etat(&pool).await.unwrap().error.as_deref(),
        Some("invalid_rates:1")
    );
}

/// Pas encore publié : 404 `RATE_NOT_PUBLISHED`, état `not_published`, aucun
/// audit ; mois (racine vide) comme jour (404 de `xmldaily`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_pas_encore_publie(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    ctx.source
        .set(RateKind::Month, MockBehavior::reply(MockReply::Empty));
    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 12})).await;
    assert_eq!(st, 404);
    assert_eq!(b["error"]["code"], "RATE_NOT_PUBLISHED");
    let e = etat(&pool).await.unwrap();
    assert_eq!(e.status, FetchOutcomeStatus::NotPublished);
    assert_eq!(e.rate_date, d(2026, 12, 1));
    ctx.source
        .set(RateKind::Day, erreur(SourceError::NotPublished));
    let (st, b) = ctx.fetch(json!({"date": "2026-11-13"})).await;
    assert_eq!(st, 404);
    assert_eq!(b["error"]["code"], "RATE_NOT_PUBLISHED");
    assert_eq!(audits_fetched(&pool).await, 0);
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        0
    );
}

/// Réponses inutilisables → 503 `RATE_SOURCE_UNAVAILABLE` (jamais 500), rien
/// écrit hors l'état, code classé dans `last_error`, message sans URL.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_reponses_inutilisables(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let cas: Vec<(u32, MockBehavior, &str)> = vec![
        (
            1,
            MockBehavior::reply(MockReply::Xml("<monatsmittelkurs><monat>2026-01".into())),
            "malformed",
        ),
        (
            2,
            MockBehavior::reply(MockReply::Xml(monthly_xml(
                2026,
                1,
                &[("USD", 1, "0.8800")],
            ))),
            "month_mismatch",
        ),
        (
            3,
            MockBehavior::reply(MockReply::Xml(monthly_xml(
                2026,
                3,
                &[("CHF", 1, "1.0000")],
            ))),
            "no_valid_rate",
        ),
        (4, erreur(SourceError::Timeout), "timeout"),
        (5, erreur(SourceError::HttpStatus(502)), "http_status:502"),
        (6, erreur(SourceError::TooLarge), "too_large"),
    ];
    for (m, behavior, code) in cas {
        ctx.source.set_for(mois(2026, m), behavior);
        let (st, b) = ctx.fetch(json!({"year": 2026, "month": m})).await;
        assert_eq!(st, 503, "{code}");
        assert_eq!(b["error"]["code"], "RATE_SOURCE_UNAVAILABLE", "{code}");
        assert!(
            !b["error"]["message"].as_str().unwrap().contains("http"),
            "aucune URL"
        );
        let e = etat(&pool).await.unwrap();
        assert_eq!(e.status, FetchOutcomeStatus::Error, "{code}");
        assert_eq!(e.error.as_deref(), Some(code));
    }
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        0
    );
    assert_eq!(compte_lignes(&pool, "exchange_rates").await, 0);
    assert_eq!(audits_fetched(&pool).await, 0);
}

/// Mémoire des 15 minutes : un échec n'est pas redemandé — même réponse, sans
/// appel, sans audit ni écriture de l'état ; un succès retire l'entrée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_memoire_des_echecs(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    ctx.source
        .set(RateKind::Month, erreur(SourceError::Connect));
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 503);
    let premier = etat(&pool).await.unwrap();
    ctx.source.reset();
    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 503, "réponse d'origine rendue");
    assert_eq!(b["error"]["code"], "RATE_SOURCE_UNAVAILABLE");
    assert_eq!(ctx.source.call_count(), 0, "pas redemandé à la source");
    assert_eq!(etat(&pool).await.unwrap(), premier, "état non réécrit");
    assert_eq!(audits_fetched(&pool).await, 0);

    // Une autre unité n'est pas touchée par la mémoire.
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 10})).await;
    assert_eq!(st, 200);

    // Un succès remplace l'entrée d'échec (ici expirée mais encore présente)
    // par une réussite (frein des réussites).
    if let Some(ancien) = Instant::now().checked_sub(Duration::from_secs(16 * 60)) {
        ctx.state.rate_fetch_memo.lock().unwrap().insert(
            (RateKind::Month, d(2026, 9, 1)),
            MemoEntry {
                depuis: ancien,
                echec: Some(Echec::Unavailable("timeout".into())),
            },
        );
        let (st, _) = ctx.fetch(json!({"year": 2026, "month": 9})).await;
        assert_eq!(st, 200);
        assert!(
            ctx.state
                .rate_fetch_memo
                .lock()
                .unwrap()
                .get(&(RateKind::Month, d(2026, 9, 1)))
                .is_some_and(|e| e.echec.is_none()),
            "réussite mémorisée"
        );
    }
    assert!(
        ctx.state
            .rate_fetch_memo
            .lock()
            .unwrap()
            .contains_key(&(RateKind::Month, d(2026, 11, 1))),
        "l'échec de novembre reste mémorisé"
    );
}

/// Frein des réussites (revue d'intégration de l'équipe de Guy) : un second
/// `POST` de la même unité dans les 15 minutes est servi en 200 sans appel à la
/// source, sans audit ni écriture de l'état, par ce que la base tient
/// (`count`, `rejected` et `corrected` à 0) — la copie faite pour la société de
/// l'appelant ; une autre unité n'est pas freinée ; passé le délai, la source
/// est rappelée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_reussite_freinee_quinze_minutes(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let (st, premier) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{premier}");
    assert_eq!(ctx.source.call_count(), 1);
    let etat_1 = etat(&pool).await.unwrap();
    let audits_1 = audits_fetched(&pool).await;

    // La source publierait autre chose : le frein ne la rappelle pas.
    ctx.source.set_for(
        mois(2026, 11),
        MockBehavior::reply(MockReply::Xml(monthly_xml(
            2026,
            11,
            &[("USD", 1, "0.8900")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(ctx.source.call_count(), 1, "pas rappelée");
    assert_eq!(b["kind"], "month");
    assert_eq!(b["rateDate"], "2026-11-01");
    assert_eq!(b["count"], premier["count"]);
    assert_eq!(b["rejected"], 0);
    assert_eq!(b["corrected"], 0);
    assert_eq!(etat(&pool).await.unwrap(), etat_1, "état non réécrit");
    assert_eq!(audits_fetched(&pool).await, audits_1, "aucun audit");

    // Une autre unité n'est pas freinée.
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 10})).await;
    assert_eq!(st, 200);
    assert_eq!(ctx.source.call_count(), 2);

    // Le délai passé, la source est rappelée et la correction comptée.
    if let Some(ancien) = Instant::now().checked_sub(Duration::from_secs(16 * 60)) {
        ctx.state
            .rate_fetch_memo
            .lock()
            .unwrap()
            .get_mut(&(RateKind::Month, d(2026, 11, 1)))
            .unwrap()
            .depuis = ancien;
        let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
        assert_eq!(st, 200, "{b}");
        assert_eq!(ctx.source.call_count(), 3, "rappelée après le délai");
        assert_eq!(b["corrected"], 1);
    }
}

/// Correction de l'AFC (règle (c)) comptée dans `corrected` ; les cours
/// retenus par la société (`afc` copié et `manual`) restent à leur valeur.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_correction_comptee_cours_retenus_intacts(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    compte_en_devise(&pool, ctx.s.company_id, "1021", "EUR").await;
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200);
    // EUR saisi à la main : `manual`.
    let mut conn = pool.acquire().await.unwrap();
    let saisi = depot::set_manual_rate(
        &mut conn,
        ctx.s.company_id,
        ctx.s.admin_user_id,
        "EUR",
        RateKind::Month,
        d(2026, 11, 1),
        dec!(0.95),
        Some(1),
    )
    .await
    .unwrap();
    drop(conn);
    let version = saisi.after.unwrap().version;

    ctx.source.set(
        RateKind::Month,
        MockBehavior::reply(MockReply::Xml(monthly_xml(
            2026,
            11,
            &[("USD", 1, "0.8900"), ("EUR", 1, "0.9400")],
        ))),
    );
    // La réussite de novembre freinerait le second `POST` (servi sans appel,
    // `fetch_reussite_freinee_quinze_minutes`) : on l'oublie, comme 15 minutes
    // écoulées.
    ctx.state.rate_fetch_memo.lock().unwrap().clear();
    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["count"], 2);
    assert_eq!(b["corrected"], 2);
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88)),
        "la ligne `afc` de la société n'est pas modifiée"
    );
    let mut conn = pool.acquire().await.unwrap();
    let eur = depot::find_company_rate_in_tx(
        &mut conn,
        ctx.s.company_id,
        "EUR",
        RateKind::Month,
        d(2026, 11, 1),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(eur.rate, dec!(0.95));
    assert_eq!(eur.version, version);
    assert_eq!(
        depot::list_notices(&mut conn, ctx.s.company_id)
            .await
            .unwrap()
            .len(),
        1,
        "la correction USD s'annonce"
    );
}

/// Cours journaliers : publication antérieure de 2 jours acceptée
/// (`sourceDate`), de plus de 7 jours refusée (503) ; 2000-01-01 servi
/// « 31.12.1999 » accepté.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_jours(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let samedi = RateUnit::day(d(2026, 11, 14));
    ctx.source.set_for(
        samedi,
        MockBehavior::reply(MockReply::Xml(daily_xml(
            d(2026, 11, 12),
            &[("USD", 1, "0.89120")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"date": "2026-11-14"})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["kind"], "day");
    assert_eq!(b["rateDate"], "2026-11-14");
    assert_eq!(b["sourceDate"], "2026-11-12");

    ctx.source.set_for(
        RateUnit::day(d(2026, 11, 10)),
        MockBehavior::reply(MockReply::Xml(daily_xml(
            d(2026, 11, 2),
            &[("USD", 1, "0.89120")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"date": "2026-11-10"})).await;
    assert_eq!(st, 503);
    assert_eq!(b["error"]["code"], "RATE_SOURCE_UNAVAILABLE");
    assert_eq!(
        etat(&pool).await.unwrap().error.as_deref(),
        Some("date_mismatch")
    );

    ctx.source.set_for(
        RateUnit::day(d(2000, 1, 1)),
        MockBehavior::reply(MockReply::Xml(daily_xml(
            d(1999, 12, 31),
            &[("USD", 1, "1.59000")],
        ))),
    );
    let (st, b) = ctx.fetch(json!({"date": "2000-01-01"})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["sourceDate"], "1999-12-31");
}

/// Deux `POST` simultanés (deux mois) : sérialisés par le verrou — la durée
/// totale couvre les deux délais de la source.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_deux_post_simultanes_serialises(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    ctx.source.set(
        RateKind::Month,
        MockBehavior {
            reply: MockReply::Default,
            delay: Some(Duration::from_millis(400)),
        },
    );
    let debut = Instant::now();
    let (a, b) = tokio::join!(
        ctx.fetch(json!({"year": 2026, "month": 11})),
        ctx.fetch(json!({"year": 2026, "month": 10}))
    );
    assert_eq!(a.0, 200);
    assert_eq!(b.0, 200);
    assert!(
        debut.elapsed() >= Duration::from_millis(800),
        "deux récupérations l'une après l'autre ({:?})",
        debut.elapsed()
    );
    assert_eq!(ctx.source.call_count(), 2);
}

/// Raccourci de l'AC-3 : un `POST` qui a attendu le verrou et trouve le mois
/// écrit par une récupération commencée après son arrivée rend 200 sans
/// rappeler la source, sans audit ni écriture de l'état, et fait la copie pour
/// sa société.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_raccourci_apres_attente(pool: MySqlPool) {
    let ctx = Arc::new(monter(&pool, false).await);
    let verrou = ctx.state.rate_fetch_lock.clone().lock_owned().await;
    let c = ctx.clone();
    let post = tokio::spawn(async move { c.fetch(json!({"year": 2026, "month": 11})).await });
    tokio::time::sleep(Duration::from_millis(300)).await;
    // Une récupération « commencée après l'arrivée » du POST écrit le mois.
    poser(&pool, mois(2026, 11), Utc::now().naive_utc()).await;
    drop(verrou);
    let (st, b) = post.await.unwrap();
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        b,
        json!({"kind": "month", "rateDate": "2026-11-01", "sourceDate": null,
               "count": 1, "rejected": 0, "corrected": 0})
    );
    assert_eq!(ctx.source.call_count(), 0);
    assert_eq!(audits_fetched(&pool).await, 0);
    assert!(etat(&pool).await.is_none());
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88)),
        "copie pour la société de l'utilisateur"
    );

    // Écrit AVANT l'arrivée : pas de raccourci, la source est rappelée.
    let verrou = ctx.state.rate_fetch_lock.clone().lock_owned().await;
    let c = ctx.clone();
    let post = tokio::spawn(async move { c.fetch(json!({"year": 2026, "month": 11})).await });
    tokio::time::sleep(Duration::from_millis(300)).await;
    drop(verrou);
    let (st, _) = post.await.unwrap();
    assert_eq!(st, 200);
    assert_eq!(ctx.source.call_count(), 1);
    assert_eq!(audits_fetched(&pool).await, 1);
}

/// Revue de code P1 (37-2-CR-P1-2) — une écriture de cours attend la fin
/// d'une restauration (`backup_lock` tenu par `full_import` jusqu'à son
/// commit) : rien n'est écrit pendant qu'elle le tient, et la récupération
/// aboutit une fois le verrou rendu. L'appel sortant, lui, n'attend pas.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_attend_la_fin_d_une_restauration(pool: MySqlPool) {
    let ctx = Arc::new(monter(&pool, false).await);
    let restauration = ctx.state.backup_lock.clone().lock_owned().await;
    let c = ctx.clone();
    let post = tokio::spawn(async move { c.fetch(json!({"year": 2026, "month": 11})).await });
    let limite = Instant::now() + Duration::from_secs(10);
    while ctx.source.call_count() == 0 && Instant::now() < limite {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert_eq!(ctx.source.call_count(), 1, "l'appel sortant n'attend pas");
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert!(!post.is_finished(), "l'écriture attend la restauration");
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        None
    );
    assert_eq!(audits_fetched(&pool).await, 0);
    drop(restauration);
    let (st, b) = post.await.unwrap();
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88))
    );
}

/// Revue de code P2 (37-2-CR-P2-3) — l'audit du `POST /fetch` ne compte que
/// les copies de la société de l'utilisateur : une seconde société multi-devises
/// reçoit sa copie, sans entrer dans le `copied` de l'audit de la première.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_audit_ne_compte_que_la_societe_de_l_utilisateur(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    let b = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, \
             instance_language, multi_currency_enabled) \
         VALUES ('Société B', 'Rue B 1\n1000 Lausanne', 'Independant', 'FR', 'FR', TRUE)",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, \
             multi_currency, opening_rate_kind) \
         VALUES (?, 'B 2026', '2026-01-01', '2026-12-31', 'Open', TRUE, 'month')",
    )
    .bind(b)
    .execute(&pool)
    .await
    .unwrap();
    compte_en_devise(&pool, b, "1020", "USD").await;
    let (st, body) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{body}");
    assert_eq!(
        retenu(&pool, b, "USD", mois(2026, 11)).await,
        Some(dec!(0.88)),
        "la seconde société reçoit sa copie"
    );
    let details = common::audit_details(
        &pool,
        "installation",
        AUDIT_ENTITY_ID_NONE,
        "exchange_rates.fetched",
    )
    .await
    .unwrap();
    assert_eq!(details["copied"], 1, "{details}");
}

/// Exercice clos : `POST /fetch` copie pour la société de l'utilisateur
/// (A7), la tâche, elle, ne vise que les exercices ouverts.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fetch_copie_pour_un_exercice_clos(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let (st, b) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88))
    );
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan.statut, None);
    assert_eq!(
        ctx.source.call_count(),
        1,
        "aucun exercice ouvert : aucun appel"
    );
}

// ============================================================
// Mode test : pilotage de la source, vidages
// ============================================================

/// `POST /_test/exchange-rates/source` pilote la source factice ;
/// `_test/reset` et `_test/seed` vident les tables système et la mémoire et
/// remettent la source à son défaut.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mode_test_pilotage_et_vidages(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    let (st, b) = ctx
        .post(
            "/api/v1/_test/exchange-rates/source",
            &ctx.jwt,
            json!({"kind": "month", "reply": "error", "error": "timeout"}),
        )
        .await;
    assert_eq!(st, 200, "{b}");
    assert_eq!(b["calls"], 0);
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 503);
    let (st, _) = ctx
        .post(
            "/api/v1/_test/exchange-rates/source",
            &ctx.jwt,
            json!({"kind": "month", "reply": "error", "error": "bizarre"}),
        )
        .await;
    assert_eq!(st, 400);
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 10})).await;
    assert_eq!(st, 503, "pilotage du genre");
    let (st, _) = ctx
        .post(
            "/api/v1/_test/exchange-rates/source",
            &ctx.jwt,
            json!({"kind": "month", "reply": "default"}),
        )
        .await;
    assert_eq!(st, 200);
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 9})).await;
    assert_eq!(st, 200);
    assert!(compte_lignes(&pool, "_kesh_fetched_exchange_rates").await > 0);
    assert!(compte_lignes(&pool, "_kesh_exchange_rate_fetch").await > 0);
    assert!(!ctx.state.rate_fetch_memo.lock().unwrap().is_empty());

    let r = ctx
        .client
        .post(ctx.url("/api/v1/_test/reset"))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        0
    );
    assert_eq!(compte_lignes(&pool, "_kesh_exchange_rate_fetch").await, 0);
    assert!(ctx.state.rate_fetch_memo.lock().unwrap().is_empty());
    assert_eq!(ctx.source.call_count(), 0, "source remise à son défaut");
    assert!(ctx.state.rate_login_trigger.lock().unwrap().is_none());

    // `_test/seed` aussi.
    poser(&pool, mois(2026, 11), Utc::now().naive_utc()).await;
    let r = ctx
        .client
        .post(ctx.url("/api/v1/_test/seed"))
        .json(&json!({"preset": "with-company"}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200);
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        0
    );
}

// ============================================================
// Tâche automatique : un_tour
// ============================================================

/// Aucun exercice multi-devises ouvert → aucun appel, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_sans_exercice_multi_devises(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(ctx.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan, Default::default());
    assert_eq!(ctx.source.call_count(), 0);
    assert!(etat(&pool).await.is_none());
}

/// Source injoignable → état `error` (`auto`), rien d'autre touché ; un
/// `POST` qui suit rend l'échec mémorisé par le tour.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_source_injoignable(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    ctx.source
        .set(RateKind::Month, erreur(SourceError::Timeout));
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(
        bilan.demandees,
        vec![mois(2026, 10), mois(2026, 11), mois(2026, 12)]
    );
    assert_eq!(bilan.reverifiee, None, "le mois courant manquait");
    assert_eq!(bilan.statut, Some(FetchOutcomeStatus::Error));
    let e = etat(&pool).await.unwrap();
    assert_eq!(e.trigger, depot::FetchTrigger::Auto);
    assert_eq!(e.error.as_deref(), Some("timeout"));
    assert_eq!(
        e.rate_date,
        d(2026, 12, 1),
        "à égalité, l'unité la plus récente"
    );
    assert_eq!(
        compte_lignes(&pool, "_kesh_fetched_exchange_rates").await,
        0
    );
    assert_eq!(compte_lignes(&pool, "exchange_rates").await, 0);
    assert_eq!(
        compte_lignes(&pool, "audit_log WHERE action LIKE 'exchange%'").await,
        0
    );
    ctx.source.reset();
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 503, "la mémoire est alimentée par la tâche");
    assert_eq!(ctx.source.call_count(), 0);
}

/// Pire statut : le mois suivant pas encore publié, les autres récupérés →
/// `not_published` ; plus un échec → `error`, sur l'unité en cause.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_pire_statut(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    ctx.source
        .set_for(mois(2026, 12), MockBehavior::reply(MockReply::Empty));
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan.statut, Some(FetchOutcomeStatus::NotPublished));
    let e = etat(&pool).await.unwrap();
    assert_eq!(e.rate_date, d(2026, 12, 1));
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 10)).await,
        Some(dec!(0.88)),
        "copie du mois d'ouverture par la tâche"
    );
    assert_eq!(
        compte_lignes(&pool, "audit_log WHERE action LIKE 'exchange%'").await,
        0
    );

    // Tour suivant (mémoire vidée) : décembre en échec, novembre revérifié.
    ctx.state.rate_fetch_memo.lock().unwrap().clear();
    ctx.source
        .set_for(mois(2026, 12), erreur(SourceError::HttpStatus(500)));
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan.demandees, vec![mois(2026, 12)]);
    assert_eq!(bilan.reverifiee, Some(mois(2026, 11)));
    assert_eq!(bilan.statut, Some(FetchOutcomeStatus::Error));
    assert_eq!(
        etat(&pool).await.unwrap().error.as_deref(),
        Some("http_status:500")
    );
}

/// Revérification du mois courant (A10) : mois suivant manquant, mois courant
/// présent → deux appels (le suivant, puis le courant) ; une autre valeur
/// rendue met à jour la valeur publiée, la ligne `afc` de la société reste,
/// la correction s'annonce, aucune ligne d'audit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_reverification_du_mois_courant(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 10})).await;
    assert_eq!(st, 200);
    let (st, _) = ctx.fetch(json!({"year": 2026, "month": 11})).await;
    assert_eq!(st, 200);
    let audits = audits_fetched(&pool).await;
    ctx.source.reset();
    ctx.source.set_for(
        mois(2026, 11),
        MockBehavior::reply(MockReply::Xml(monthly_xml(
            2026,
            11,
            &[("USD", 1, "0.9000")],
        ))),
    );
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(ctx.source.calls(), vec![mois(2026, 12), mois(2026, 11)]);
    assert_eq!(bilan.statut, Some(FetchOutcomeStatus::Ok));
    let mut conn = pool.acquire().await.unwrap();
    let publiee = depot::find_fetched_rate_in_tx(
        &mut conn,
        SOURCE_AFC,
        "USD",
        RateKind::Month,
        d(2026, 11, 1),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(publiee.rate_per_unit, dec!(0.9));
    assert_eq!(
        retenu(&pool, ctx.s.company_id, "USD", mois(2026, 11)).await,
        Some(dec!(0.88))
    );
    assert_eq!(
        depot::list_notices(&mut conn, ctx.s.company_id)
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        audits_fetched(&pool).await,
        audits,
        "tour automatique non audité"
    );

    // Rien ne manque → aucun appel, revérification comprise.
    drop(conn);
    ctx.source.reset();
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan, Default::default());
    assert_eq!(ctx.source.call_count(), 0);
}

/// Échec de la revérification, l'unité manquante réussie → `error`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_echec_de_la_reverification(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    let at = Utc::now().naive_utc();
    poser(&pool, mois(2026, 10), at).await;
    poser(&pool, mois(2026, 11), at).await;
    ctx.source
        .set_for(mois(2026, 11), erreur(SourceError::Connect));
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan.demandees, vec![mois(2026, 12)]);
    assert_eq!(bilan.reverifiee, Some(mois(2026, 11)));
    assert_eq!(bilan.statut, Some(FetchOutcomeStatus::Error));
    let e = etat(&pool).await.unwrap();
    assert_eq!(e.rate_date, d(2026, 11, 1));
    assert_eq!(e.error.as_deref(), Some("connect"));
}

/// Plafond : 36 unités manquantes par tour, plus la revérification (hors
/// plafond), dans l'ordre chronologique.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn tour_plafond_trente_six_plus_un(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2023, 1, 1), d(2026, 12, 31)).await;
    poser(&pool, mois(2026, 11), Utc::now().naive_utc()).await;
    let bilan = un_tour(&ctx.state, Declencheur::Tache).await.unwrap();
    assert_eq!(bilan.demandees.len(), 36);
    assert_eq!(
        bilan.demandees[0],
        mois(2022, 12),
        "le mois d'ouverture d'abord"
    );
    // Revue de code P1 (37-2-CR-P1-4) : le mois suivant, qui manque, est
    // toujours demandé — il prend la place de 2025-11, ordre chronologique
    // gardé.
    assert_eq!(bilan.demandees[34], mois(2025, 10));
    assert_eq!(bilan.demandees[35], mois(2026, 12));
    assert_eq!(bilan.reverifiee, Some(mois(2026, 11)));
    assert_eq!(ctx.source.call_count(), 37);
}

// ============================================================
// Connexion
// ============================================================

async fn connexion(ctx: &Ctx) -> (u16, Duration) {
    let debut = Instant::now();
    let r = ctx
        .client
        .post(ctx.url("/api/v1/auth/login"))
        .json(&json!({"username": "admin", "password": "admin123"}))
        .send()
        .await
        .unwrap();
    (r.status().as_u16(), debut.elapsed())
}

/// La réponse de connexion ne dépend pas de la source (délai de 30 s) ; une
/// unité manquante autre que le mois suivant lance un tour détaché.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn connexion_lance_un_tour_detache(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    *ctx.state.rate_login_trigger.lock().unwrap() = None;
    ctx.source.set(
        RateKind::Month,
        MockBehavior {
            reply: MockReply::Default,
            delay: Some(Duration::from_secs(30)),
        },
    );
    let (st, duree) = connexion(&ctx).await;
    assert_eq!(st, 200);
    assert!(duree < Duration::from_secs(10), "connexion en {duree:?}");
    let limite = Instant::now() + Duration::from_secs(10);
    while ctx.source.call_count() == 0 && Instant::now() < limite {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert_eq!(ctx.source.calls().first(), Some(&mois(2026, 10)));
    assert!(ctx.state.rate_login_trigger.lock().unwrap().is_some());
}

/// Seul le mois suivant manque → aucun tour (aucun appel, aucun état) ; et
/// en `KESH_TEST_MODE`, rien du tout.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn connexion_seul_le_mois_suivant_manque(pool: MySqlPool) {
    let ctx = monter(&pool, false).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    let at = Utc::now().naive_utc();
    poser(&pool, mois(2026, 10), at).await;
    poser(&pool, mois(2026, 11), at).await;
    *ctx.state.rate_login_trigger.lock().unwrap() = None;
    let (st, _) = connexion(&ctx).await;
    assert_eq!(st, 200);
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(ctx.source.call_count(), 0);
    assert!(etat(&pool).await.is_none());
    assert!(ctx.state.rate_login_trigger.lock().unwrap().is_none());
}

/// Mode test : aucune tâche détachée à la connexion.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn connexion_en_mode_test_ne_declenche_rien(pool: MySqlPool) {
    let ctx = monter(&pool, true).await;
    exercice(&pool, &ctx, d(2026, 11, 1), d(2026, 12, 31)).await;
    *ctx.state.rate_login_trigger.lock().unwrap() = None;
    assert!(
        kesh_api::exchange_rates::declencher_a_la_connexion(&ctx.state, ctx.s.company_id).is_none()
    );
    let (st, _) = connexion(&ctx).await;
    assert_eq!(st, 200);
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(ctx.source.call_count(), 0);
}
