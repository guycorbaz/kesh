//! Story 37-3a (AC-6, AC-7) — les écritures en devise par l'API :
//! `POST`/`PUT /api/v1/journal-entries` avec `foreign`, analyse sans calcul du
//! client (cours déduit du CHF, CHF posé selon `side`), statuts et codes de
//! chaque refus, forme des réponses (`foreign` omis sur une ligne CHF).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

// ============================================================
// Harnais (patron account_currency_e2e.rs)
// ============================================================

struct TestApp {
    base_url: String,
    client: reqwest::Client,
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
    app: TestApp,
    jwt: String,
    s: SeededCompany,
    usd: i64,
    eur: i64,
}

async fn compte(pool: &MySqlPool, company_id: i64, numero: &str, devise: Option<&str>) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, 'Asset', ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Société multi-devises (SQL), exercice multi-devises, comptes USD et EUR.
async fn monter(pool: &MySqlPool) -> Ctx {
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
    let usd = compte(pool, s.company_id, "1020", Some("USD")).await;
    let eur = compte(pool, s.company_id, "1030", Some("EUR")).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    Ctx {
        app,
        jwt,
        s,
        usd,
        eur,
    }
}

async fn cours(pool: &MySqlPool, ctx: &Ctx, devise: &str, rate: &str, source: &str) {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, ?, 'month', '2026-03-01', ?, ?) \
         ON DUPLICATE KEY UPDATE rate = VALUES(rate), source = VALUES(source)",
    )
    .bind(ctx.s.company_id)
    .bind(devise)
    .bind(rate)
    .bind(source)
    .execute(pool)
    .await
    .unwrap();
}

/// Corps d'une écriture du 15.03.2026 : une ligne en devise sur `compte`, et sa
/// contrepartie CHF (ventes, au crédit du CHF `contre`).
fn corps(ctx: &Ctx, compte: i64, ligne: Value, contre: &str) -> Value {
    let mut l = ligne;
    l["accountId"] = json!(compte);
    json!({
        "entryDate": "2026-03-15",
        "journal": "OD",
        "description": "Écriture en devise",
        "lines": [
            l,
            { "accountId": ctx.s.accounts["3000"], "debit": "0", "credit": contre },
        ],
    })
}

async fn post(ctx: &Ctx, body: &Value) -> (u16, Value) {
    let r = ctx
        .app
        .client
        .post(format!("{}/api/v1/journal-entries", ctx.app.base_url))
        .bearer_auth(&ctx.jwt)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

async fn put(ctx: &Ctx, id: i64, body: &Value) -> (u16, Value) {
    let r = ctx
        .app
        .client
        .put(format!("{}/api/v1/journal-entries/{id}", ctx.app.base_url))
        .bearer_auth(&ctx.jwt)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or("")
}

// ============================================================
// AC-6 — forme des réponses
// ============================================================

/// Ligne sans `currency` sur un compte USD → 201, `foreign.currency` = `USD`
/// (lue sur le compte) ; la ligne CHF n'a pas de clé `foreign`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ligne_usd_sans_devise_annoncee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "debit": "1125.83", "credit": "0", "foreign": { "amount": "1234.56", "rate": "0.91193", "rateSource": "manual" } }),
            "1125.83",
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(
        b["lines"][0]["foreign"],
        json!({ "currency": "USD", "amount": "1234.56", "rate": "0.91193", "rateSource": "manual" })
    );
    assert!(b["lines"][1].get("foreign").is_none(), "{b}");
}

// ============================================================
// AC-6 — un test HTTP par code (statuts)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn statuts_des_refus(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let fx = |f: Value| json!({ "debit": "9.12", "credit": "0", "foreign": f });
    // (libellé, compte, ligne, contrepartie CHF, statut, code)
    type Cas<'a> = (&'a str, i64, Value, &'a str, u16, &'a str);
    let cas: Vec<Cas> = vec![
        (
            "hors bornes",
            ctx.usd,
            fx(json!({ "amount": "10.00001", "rate": "0.912", "rateSource": "manual" })),
            "9.12",
            400,
            "FX_VALUE_OUT_OF_RANGE",
        ),
        (
            "arrondi nul",
            ctx.usd,
            json!({ "debit": "0", "credit": "0", "foreign": { "amount": "0.001", "rate": "0.912", "rateSource": "manual" } }),
            "0.01",
            400,
            "FX_LINE_ROUNDS_TO_ZERO",
        ),
        (
            "écart CHF",
            ctx.usd,
            fx(json!({ "amount": "10", "rate": "0.913", "rateSource": "manual" })),
            "9.12",
            400,
            "FX_AMOUNT_MISMATCH",
        ),
        (
            "compte CHF",
            ctx.s.accounts["1000"],
            fx(json!({ "amount": "10", "rate": "0.912", "rateSource": "manual" })),
            "9.12",
            400,
            "FX_ACCOUNT_IN_CHF",
        ),
        (
            "compte CHF, CHF annoncé",
            ctx.s.accounts["1000"],
            fx(
                json!({ "currency": "CHF", "amount": "10", "rate": "0.912", "rateSource": "manual" }),
            ),
            "9.12",
            400,
            "FX_ACCOUNT_IN_CHF",
        ),
        (
            "devise annoncée EUR",
            ctx.usd,
            fx(
                json!({ "currency": "EUR", "amount": "10", "rate": "0.912", "rateSource": "manual" }),
            ),
            "9.12",
            400,
            "FX_CURRENCY_MISMATCH",
        ),
        (
            "devise annoncée usd",
            ctx.usd,
            fx(
                json!({ "currency": "usd", "amount": "10", "rate": "0.912", "rateSource": "manual" }),
            ),
            "9.12",
            400,
            "FX_CURRENCY_MISMATCH",
        ),
        (
            "sans foreign",
            ctx.usd,
            json!({ "debit": "9.12", "credit": "0" }),
            "9.12",
            400,
            "ACCOUNT_REQUIRES_CURRENCY",
        ),
        (
            "afc sans cours retenu",
            ctx.usd,
            fx(json!({ "amount": "10", "rate": "0.912", "rateSource": "afc" })),
            "9.12",
            404,
            "RATE_UNAVAILABLE",
        ),
        (
            "revaluation",
            ctx.usd,
            fx(json!({ "amount": "0", "rate": "0.912", "rateSource": "revaluation" })),
            "9.12",
            400,
            "VALIDATION_ERROR",
        ),
        (
            "côté inconnu",
            ctx.usd,
            json!({ "foreign": { "amount": "10", "rate": "0.912", "side": "x" } }),
            "9.12",
            400,
            "VALIDATION_ERROR",
        ),
    ];
    for (libelle, compte, ligne, contre, statut, attendu) in cas {
        let (s, b) = post(&ctx, &corps(&ctx, compte, ligne, contre)).await;
        assert_eq!((s, code(&b)), (statut, attendu), "{libelle} : {b}");
    }
    // FX_AMOUNT_MISMATCH : détails `{ line, expected, provided }`.
    let (_, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            fx(json!({ "amount": "10", "rate": "0.913", "rateSource": "manual" })),
            "9.12",
        ),
    )
    .await;
    assert_eq!(
        b["error"]["details"],
        json!({ "line": 1, "expected": "9.13", "provided": "9.12" })
    );
    // FX_RATE_NOT_AFC (cours retenu à un autre cours) : `expectedRate`.
    cours(&pool, &ctx, "USD", "0.91193", "afc").await;
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            fx(json!({ "amount": "10", "rate": "0.912", "rateSource": "afc" })),
            "9.12",
        ),
    )
    .await;
    assert_eq!((s, code(&b)), (400, "FX_RATE_NOT_AFC"), "{b}");
    assert_eq!(
        b["error"]["details"],
        json!({ "line": 1, "expectedRate": "0.91193" })
    );
    // RATE_UNAVAILABLE : forme de 37-2.
    let (_, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.eur,
            fx(json!({ "amount": "10", "rate": "0.912", "rateSource": "afc" })),
            "9.12",
        ),
    )
    .await;
    assert_eq!(
        b["error"]["details"],
        json!({ "currency": "EUR", "month": "2026-03-01" })
    );
    // FX_MULTIPLE_CURRENCIES : `details.currencies` triées.
    let body = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": "Deux devises",
        "lines": [
            { "accountId": ctx.usd, "debit": "9.12", "credit": "0", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } },
            { "accountId": ctx.eur, "debit": "0", "credit": "9.12", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } },
        ],
    });
    let (s, b) = post(&ctx, &body).await;
    assert_eq!((s, code(&b)), (400, "FX_MULTIPLE_CURRENCIES"), "{b}");
    assert_eq!(b["error"]["details"]["currencies"], json!(["EUR", "USD"]));
    // Ligne CHF sans `debit` → 400 (et non plus 422).
    let body = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": "CHF",
        "lines": [
            { "accountId": ctx.s.accounts["1000"], "credit": "0" },
            { "accountId": ctx.s.accounts["3000"], "debit": "0", "credit": "10" },
        ],
    });
    let (s, b) = post(&ctx, &body).await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR"), "{b}");
}

/// 409 `MULTI_CURRENCY_DISABLED` — exercice mono-devise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn exercice_mono_devise_rend_409(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(ctx.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let (s, b) = post(
        &ctx,
        &corps(&ctx, ctx.usd, json!({ "debit": "9.12", "credit": "0", "foreign": { "amount": "10", "rate": "0.912" } }), "9.12"),
    )
    .await;
    assert_eq!((s, code(&b)), (409, "MULTI_CURRENCY_DISABLED"), "{b}");
    assert_eq!(
        b["error"]["details"]["fiscalYearId"],
        json!(ctx.s.fiscal_year_id)
    );
    // Revue de code P1 : le message ne dit pas pourquoi l'exercice est
    // mono-devise — la même variante sort de la route des genres de cours
    // dans une société jamais passée en multi-devises.
    let message = b["error"]["message"].as_str().unwrap_or("");
    assert!(message.contains("mono-devise"), "{b}");
    assert!(!message.contains("clos"), "{b}");
}

// ============================================================
// AC-6 — analyse de `foreign` (V1, V2)
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cours_deduit_et_cote(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // `{ amount, chf }` sans cours ni débit/crédit, côté débit → cours de la
    // fixture, CHF au débit ; sans cours retenu → `manual`.
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "foreign": { "amount": "1234.56", "chf": "1125.83", "side": "debit" } }),
            "1125.83",
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][0]["debit"], "1125.8300");
    assert_eq!(b["lines"][0]["foreign"]["rate"], "0.9119281363");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "manual");
    // CHF tiré de débit/crédit (un seul fourni : l'autre vaut 0).
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "debit": "1125.83", "foreign": { "amount": "1234.56" } }),
            "1125.83",
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rate"], "0.9119281363");
    // Cours déduit (A32) : le mois est retenu `afc` à 0.91193, qui redonne le
    // CHF → cours retenu, provenance `afc`.
    cours(&pool, &ctx, "USD", "0.91193", "afc").await;
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "foreign": { "amount": "1234.56", "chf": "1125.83", "side": "debit" } }),
            "1125.83",
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rate"], "0.91193");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "afc");
    // Côté crédit, CHF calculé par le produit (cours fourni, sans débit/crédit).
    let body = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": "Crédit",
        "lines": [
            { "accountId": ctx.s.accounts["4000"], "debit": "9.12", "credit": "0" },
            { "accountId": ctx.usd, "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual", "side": "credit" } },
        ],
    });
    let (s, b) = post(&ctx, &body).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][1]["credit"], "9.1200");
    // Refus de l'analyse.
    let refus: Vec<(&str, Value, &str)> = vec![
        (
            "chf ≠ débit + crédit",
            json!({ "debit": "9.12", "foreign": { "amount": "10", "chf": "9.13" } }),
            "FX_AMOUNT_MISMATCH",
        ),
        (
            "ni cours ni CHF",
            json!({ "foreign": { "amount": "10", "side": "debit" } }),
            "VALIDATION_ERROR",
        ),
        (
            "sans côté",
            json!({ "foreign": { "amount": "10", "rate": "0.912" } }),
            "VALIDATION_ERROR",
        ),
        (
            "côté contraire",
            json!({ "debit": "9.12", "foreign": { "amount": "10", "rate": "0.912", "side": "credit" } }),
            "VALIDATION_ERROR",
        ),
        (
            "aucun cours ne convient",
            json!({ "foreign": { "amount": "10000000000000", "chf": "9123456789012.34", "side": "debit" } }),
            "FX_AMOUNT_MISMATCH",
        ),
        (
            "montant 0 sans cours",
            json!({ "foreign": { "amount": "0", "chf": "1", "side": "debit" } }),
            "FX_VALUE_OUT_OF_RANGE",
        ),
        (
            "CHF 0 sans cours",
            json!({ "foreign": { "amount": "10", "chf": "0", "side": "debit" } }),
            "FX_VALUE_OUT_OF_RANGE",
        ),
        (
            "montant à 5 décimales",
            json!({ "foreign": { "amount": "1.00001", "chf": "1", "side": "debit" } }),
            "FX_VALUE_OUT_OF_RANGE",
        ),
        (
            "CHF négatif",
            json!({ "foreign": { "amount": "10", "chf": "-9.12", "side": "debit" } }),
            "VALIDATION_ERROR",
        ),
        (
            "réévaluation importée en manual",
            json!({ "debit": "5", "foreign": { "amount": "0", "rate": "0.9", "rateSource": "manual" } }),
            "FX_VALUE_OUT_OF_RANGE",
        ),
    ];
    for (libelle, ligne, attendu) in refus {
        let (s, b) = post(&ctx, &corps(&ctx, ctx.usd, ligne, "9.12")).await;
        assert_eq!((s, code(&b)), (400, attendu), "{libelle} : {b}");
    }
    // `side` avec deux colonnes non nulles : jugé par l'exclusivité, non par `side`.
    let (s, b) = post(
        &ctx,
        &corps(&ctx, ctx.usd, json!({ "debit": "9", "credit": "0.12", "foreign": { "amount": "10", "rate": "0.912", "side": "credit" } }), "8.88"),
    )
    .await;
    assert_eq!(s, 400, "{b}");
    assert!(
        b["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("exclusif"),
        "exclusivité : {b}"
    );
}

/// Ordre des refus de l'analyse (CR6-P2-4, CR6-P3-4).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ordre_des_refus_de_l_analyse(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Ligne 1 à `rate` incohérent, ligne 2 sans `rate` au CHF 0 → la ligne 2.
    let body = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": "Ordre",
        "lines": [
            { "accountId": ctx.usd, "debit": "9.13", "credit": "0", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } },
            { "accountId": ctx.usd, "credit": "0", "foreign": { "amount": "10", "chf": "0", "side": "credit" } },
        ],
    });
    let (s, b) = post(&ctx, &body).await;
    assert_eq!((s, code(&b)), (400, "FX_VALUE_OUT_OF_RANGE"), "{b}");
    assert_eq!(b["error"]["details"]["line"], 2);
    // Ligne CHF négative et libellé vide → libellé (inchangé) ; même corps avec
    // une ligne `foreign` négative → montant négatif d'abord.
    let chf = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": " ",
        "lines": [
            { "accountId": ctx.s.accounts["1000"], "debit": "-1", "credit": "0" },
            { "accountId": ctx.s.accounts["3000"], "debit": "0", "credit": "1" },
        ],
    });
    let (_, b) = post(&ctx, &chf).await;
    assert!(
        b["error"]["message"].as_str().unwrap().contains("libellé"),
        "{b}"
    );
    let fxneg = json!({
        "entryDate": "2026-03-15", "journal": "OD", "description": " ",
        "lines": [
            { "accountId": ctx.usd, "debit": "-1", "credit": "0", "foreign": { "amount": "1", "rate": "1" } },
            { "accountId": ctx.s.accounts["3000"], "debit": "0", "credit": "1" },
        ],
    });
    let (_, b) = post(&ctx, &fxneg).await;
    assert!(
        b["error"]["message"].as_str().unwrap().contains("négatif"),
        "{b}"
    );
}

// ============================================================
// AC-7 — `PUT`
// ============================================================

/// Un `PUT` renvoyé tel que lu, sans `rateSource`, après la reprise d'un cours
/// corrigé → sans effet (même version) ; avec une provenance changée → écrit.
/// Changer le compte d'une ligne (U2) : USD → EUR passe, la réponse porte EUR.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_d_une_ecriture_en_devise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    cours(&pool, &ctx, "USD", "0.91193", "afc").await;
    let ligne = json!({ "debit": "1125.83", "credit": "0", "foreign": { "amount": "1234.56", "rate": "0.91193" } });
    let (s, b) = post(&ctx, &corps(&ctx, ctx.usd, ligne.clone(), "1125.83")).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "afc");
    let id = b["id"].as_i64().unwrap();
    let version = b["version"].as_i64().unwrap();
    cours(&pool, &ctx, "USD", "0.91194", "afc").await;
    let mut body = corps(&ctx, ctx.usd, ligne.clone(), "1125.83");
    body["version"] = json!(version);
    let (s, b) = put(&ctx, id, &body).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["version"].as_i64(), Some(version), "sans effet");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "afc");
    // Changer le compte : USD → EUR, `foreign` en EUR (manual).
    let mut eur = corps(
        &ctx,
        ctx.eur,
        json!({ "debit": "1125.83", "credit": "0", "foreign": { "currency": "EUR", "amount": "1234.56", "rate": "0.91193", "rateSource": "manual" } }),
        "1125.83",
    );
    eur["version"] = json!(version);
    let (s, b) = put(&ctx, id, &eur).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["lines"][0]["foreign"]["currency"], "EUR");
    assert_eq!(b["version"].as_i64(), Some(version + 1));
}

/// Revue de code P1 — `FX_AMOUNT_MISMATCH` rend les montants **sans arrondi** :
/// un CHF fourni à quatre décimales (`9.1201`) ne s'affiche pas « 9.12 », égal
/// au CHF calculé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ecart_chf_rendu_sans_arrondi(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "debit": "9.1201", "credit": "0", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } }),
            "9.1201",
        ),
    )
    .await;
    assert_eq!(s, 400, "{b}");
    assert_eq!(code(&b), "FX_AMOUNT_MISMATCH", "{b}");
    assert_eq!(b["error"]["details"]["expected"], "9.12", "{b}");
    assert_eq!(b["error"]["details"]["provided"], "9.1201", "{b}");
    assert!(
        b["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("9.1201"),
        "{b}"
    );
}

/// Revue de code P1 — CR-6 au travers de HTTP : le marqueur « cours déduit »,
/// posé par l'analyse de la route, traverse le `PUT`. Une ligne `{ amount, chf,
/// side }` créée au cours AFC retenu (A32) puis renvoyée telle quelle après la
/// reprise d'un cours corrigé reste `afc` au cours stocké, sans effet ; copiée
/// deux fois, une seule copie s'apparie, l'autre est qualifiée au cours neuf
/// (`manual`, le cours AFC neuf ne redonnant pas son CHF).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn put_cours_deduit_au_travers_de_http(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    cours(&pool, &ctx, "USD", "0.91193", "afc").await;
    let ligne = json!({ "foreign": { "amount": "1234.56", "chf": "1125.83", "side": "debit" } });
    let (s, b) = post(&ctx, &corps(&ctx, ctx.usd, ligne.clone(), "1125.83")).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "afc", "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rate"], "0.91193", "{b}");
    let id = b["id"].as_i64().unwrap();
    let version = b["version"].as_i64().unwrap();
    cours(&pool, &ctx, "USD", "0.91194", "afc").await;
    let mut meme = corps(&ctx, ctx.usd, ligne.clone(), "1125.83");
    meme["version"] = json!(version);
    let (s, b) = put(&ctx, id, &meme).await;
    assert_eq!(s, 200, "{b}");
    assert_eq!(b["version"].as_i64(), Some(version), "sans effet : {b}");
    assert_eq!(b["lines"][0]["foreign"]["rateSource"], "afc", "{b}");
    assert_eq!(b["lines"][0]["foreign"]["rate"], "0.91193", "{b}");
    // Copie double : une seule s'apparie.
    let mut l2 = ligne.clone();
    l2["accountId"] = json!(ctx.usd);
    let mut double = corps(&ctx, ctx.usd, ligne, "2251.66");
    double["lines"].as_array_mut().unwrap().insert(1, l2);
    double["version"] = json!(version);
    let (s, b) = put(&ctx, id, &double).await;
    assert_eq!(s, 200, "{b}");
    let mut sources: Vec<(String, String)> = b["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|l| {
            let f = l.get("foreign")?;
            Some((
                f["rateSource"].as_str()?.to_string(),
                f["rate"].as_str()?.to_string(),
            ))
        })
        .collect();
    sources.sort();
    assert_eq!(
        sources,
        vec![
            ("afc".to_string(), "0.91193".to_string()),
            ("manual".to_string(), "0.9119281363".to_string()),
        ],
        "{b}"
    );
}

/// Revue de code P1 — 409 `ENTRY_IS_FX_REVALUATION` au `PUT` d'une écriture
/// qui porte une ligne de réévaluation (posée en SQL : aucune route de cette
/// story n'en crée) ; sa suppression reste permise.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn reevaluation_non_modifiable_par_http(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (s, b) = post(
        &ctx,
        &corps(
            &ctx,
            ctx.usd,
            json!({ "debit": "9.12", "credit": "0", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } }),
            "9.12",
        ),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    let id = b["id"].as_i64().unwrap();
    let version = b["version"].as_i64().unwrap();
    sqlx::query(
        "UPDATE journal_entry_lines SET amount_currency = 0, rate_source = 'revaluation' \
         WHERE entry_id = ? AND line_order = 1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let mut body = corps(
        &ctx,
        ctx.usd,
        json!({ "debit": "9.12", "credit": "0", "foreign": { "amount": "10", "rate": "0.912", "rateSource": "manual" } }),
        "9.12",
    );
    body["description"] = json!("Modifiée");
    body["version"] = json!(version);
    let (s, b) = put(&ctx, id, &body).await;
    assert_eq!((s, code(&b)), (409, "ENTRY_IS_FX_REVALUATION"), "{b}");
    let r = ctx
        .app
        .client
        .delete(format!("{}/api/v1/journal-entries/{id}", ctx.app.base_url))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "suppression : {}", r.status());
}

/// Revue de code P1 — conversion EUR → USD en deux écritures d'une seule
/// devise chacune, par un compte de conversion en CHF (R7) : la seconde porte
/// la ligne d'écart ; le compte de conversion revient à 0.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_eur_usd_en_deux_ecritures(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let conversion = compte(&pool, ctx.s.company_id, "1090", None).await;
    let ecart = ctx.s.accounts["4000"];
    // 1) EUR → conversion : 1000 EUR à 0.95 = 950 CHF.
    let (s, b) = post(
        &ctx,
        &json!({
            "entryDate": "2026-03-15", "journal": "OD", "description": "Conversion 1/2",
            "lines": [
                { "accountId": conversion, "debit": "950", "credit": "0" },
                { "accountId": ctx.eur, "debit": "0", "credit": "950",
                  "foreign": { "amount": "1000", "rate": "0.95", "rateSource": "manual" } },
            ],
        }),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    // 2) conversion → USD : 1050 USD à 0.9 = 945 CHF, écart 5 CHF en charge.
    let (s, b) = post(
        &ctx,
        &json!({
            "entryDate": "2026-03-15", "journal": "OD", "description": "Conversion 2/2",
            "lines": [
                { "accountId": ctx.usd, "debit": "945", "credit": "0",
                  "foreign": { "amount": "1050", "rate": "0.9", "rateSource": "manual" } },
                { "accountId": ecart, "debit": "5", "credit": "0" },
                { "accountId": conversion, "debit": "0", "credit": "950" },
            ],
        }),
    )
    .await;
    assert_eq!(s, 201, "{b}");
    let solde: rust_decimal::Decimal = sqlx::query_scalar(
        "SELECT COALESCE(SUM(debit - credit), 0) FROM journal_entry_lines WHERE account_id = ?",
    )
    .bind(conversion)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(solde.is_zero(), "compte de conversion soldé : {solde}");
    // Les deux devises dans une seule écriture restent refusées.
    let (s, b) = post(
        &ctx,
        &json!({
            "entryDate": "2026-03-15", "journal": "OD", "description": "Conversion directe",
            "lines": [
                { "accountId": ctx.usd, "debit": "945", "credit": "0",
                  "foreign": { "amount": "1050", "rate": "0.9", "rateSource": "manual" } },
                { "accountId": ctx.eur, "debit": "0", "credit": "945",
                  "foreign": { "amount": "994.7368", "rate": "0.95", "rateSource": "manual" } },
            ],
        }),
    )
    .await;
    assert_eq!(code(&b), "FX_MULTIPLE_CURRENCIES", "{s} {b}");
}
