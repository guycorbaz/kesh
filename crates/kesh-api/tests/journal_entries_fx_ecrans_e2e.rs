//! Story 37-3b — les routes serveur des écrans en devise :
//!
//! - AC-1 : soldes d'ouverture (`POST /opening-balances`) et leur complément
//!   (`POST /opening-balances/complete`) avec des lignes en devise ;
//! - AC-2 bis : la conversion entre devises,
//!   `POST /journal-entries/conversions` (deux écritures par le compte de
//!   conversion, ligne de différence calculée sur demande ou contrôlée).

mod common;

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
// Harnais (patron journal_entries_foreign_currency_e2e.rs)
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

/// Comptes de la fixture.
struct Ctx {
    app: TestApp,
    jwt: String,
    s: SeededCompany,
    /// Compte EUR (actif) et compte USD (actif) aux comptes de la société.
    eur: i64,
    usd: i64,
    /// Second compte USD (actif), à comptes de différence **propres**.
    usd_own: i64,
    /// Second compte USD sans comptes propres (deux comptes de même devise).
    usd_bis: i64,
    /// Compte de conversion (CHF, actif).
    transit: i64,
    /// Comptes de gains (produit) et de pertes (charge) de la société,
    /// distincts.
    gain: i64,
    loss: i64,
    /// Comptes propres de `usd_own`.
    own_gain: i64,
    own_loss: i64,
    /// Comptes de charge en devise : 6911 *Frais PayPal USD*, et un EUR.
    fee_usd: i64,
    fee_eur: i64,
    /// Compte de charge CHF.
    fee_chf: i64,
    /// Compte de report à nouveau (rôle `RetainedEarnings`).
    retained: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
    role: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency, role) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .bind(role)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Société multi-devises (SQL), exercice 2020-2030 multi-devises, comptes de
/// gains, de pertes et de conversion désignés.
async fn monter(pool: &MySqlPool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    let c = s.company_id;
    let gain = compte(pool, c, "6840", "Revenue", None, None).await;
    let loss = compte(pool, c, "6940", "Expense", None, None).await;
    let transit = compte(pool, c, "1090", "Asset", None, None).await;
    let own_gain = compte(pool, c, "6841", "Revenue", None, None).await;
    let own_loss = compte(pool, c, "6941", "Expense", None, None).await;
    let retained = compte(pool, c, "2970", "Liability", None, Some("RetainedEarnings")).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(loss)
    .bind(transit)
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("UPDATE fiscal_years SET multi_currency = TRUE WHERE id = ?")
        .bind(s.fiscal_year_id)
        .execute(pool)
        .await
        .unwrap();
    let eur = compte(pool, c, "1021", "Asset", Some("EUR"), None).await;
    let usd = compte(pool, c, "1022", "Asset", Some("USD"), None).await;
    let usd_own = compte(pool, c, "1023", "Asset", Some("USD"), None).await;
    let usd_bis = compte(pool, c, "1024", "Asset", Some("USD"), None).await;
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(own_gain)
        .bind(own_loss)
        .bind(usd_own)
        .execute(pool)
        .await
        .unwrap();
    let fee_usd = compte(pool, c, "6911", "Expense", Some("USD"), None).await;
    let fee_eur = compte(pool, c, "6912", "Expense", Some("EUR"), None).await;
    let fee_chf = compte(pool, c, "6900", "Expense", None, None).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", c);
    Ctx {
        app,
        jwt,
        s,
        eur,
        usd,
        usd_own,
        usd_bis,
        transit,
        gain,
        loss,
        own_gain,
        own_loss,
        fee_usd,
        fee_eur,
        fee_chf,
        retained,
    }
}

async fn envoyer(ctx: &Ctx, chemin: &str, corps: &Value) -> (u16, Value) {
    envoyer_avec(ctx, &ctx.jwt, chemin, corps).await
}

async fn envoyer_avec(ctx: &Ctx, jeton: &str, chemin: &str, corps: &Value) -> (u16, Value) {
    let r = ctx
        .app
        .client
        .post(format!("{}{chemin}", ctx.app.base_url))
        .bearer_auth(jeton)
        .json(corps)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

fn code(body: &Value) -> &str {
    body["error"]["code"].as_str().unwrap_or("")
}

async fn nb_ecritures(pool: &MySqlPool, company_id: i64) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entries WHERE company_id = ?")
        .bind(company_id)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// Solde (débit − crédit) d'un compte, toutes écritures confondues.
async fn solde(pool: &MySqlPool, account_id: i64) -> String {
    let s: Option<rust_decimal::Decimal> = sqlx::query_scalar(
        "SELECT SUM(debit - credit) FROM journal_entry_lines WHERE account_id = ?",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap();
    s.unwrap_or_default().normalize().to_string()
}

/// Ligne en devise du corps (`manual`, côté donné par la colonne CHF).
fn fx(account: i64, cote: &str, chf: &str, devise: &str, montant: &str, cours: &str) -> Value {
    let (debit, credit) = if cote == "debit" {
        (chf, "0")
    } else {
        ("0", chf)
    };
    json!({
        "accountId": account,
        "debit": debit,
        "credit": credit,
        "foreign": { "currency": devise, "amount": montant, "rate": cours, "rateSource": "manual" },
    })
}

/// Ligne CHF du corps.
fn chf(account: i64, debit: &str, credit: &str) -> Value {
    json!({ "accountId": account, "debit": debit, "credit": credit })
}

// ============================================================
// AC-1 — soldes d'ouverture
// ============================================================

/// Ouverture à lignes EUR et USD (exemption de `FX_MULTIPLE_CURRENCIES`),
/// contrepartie CHF ; la réponse porte la devise du compte ; le `PUT` du seul
/// libellé de l'écriture ainsi créée → 200.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_a_deux_devises_puis_put_du_libelle(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let corps = json!({ "lines": [
        fx(ctx.eur, "debit", "947.00", "EUR", "1000", "0.947"),
        fx(ctx.usd, "debit", "88.46", "USD", "100", "0.8846"),
        chf(ctx.s.accounts["2000"], "0", "1035.46"),
    ]});
    let (st, body) = envoyer(&ctx, "/api/v1/opening-balances", &corps).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["lines"][0]["foreign"]["currency"], "EUR");
    assert_eq!(body["lines"][0]["foreign"]["rateSource"], "manual");
    assert_eq!(body["lines"][1]["foreign"]["currency"], "USD");
    assert!(body["lines"][2].get("foreign").is_none(), "{body}");

    // `PUT` du libellé, lignes inchangées.
    let lignes: Vec<Value> = body["lines"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            let mut v = json!({
                "accountId": l["accountId"], "debit": l["debit"], "credit": l["credit"],
            });
            if let Some(f) = l.get("foreign") {
                v["foreign"] = f.clone();
            }
            v
        })
        .collect();
    let put = json!({
        "entryDate": body["entryDate"], "journal": body["journal"],
        "description": "Ouverture (libellé corrigé)", "lines": lignes,
        "version": body["version"],
    });
    let r = ctx
        .app
        .client
        .put(format!(
            "{}/api/v1/journal-entries/{}",
            ctx.app.base_url, body["id"]
        ))
        .bearer_auth(&ctx.jwt)
        .json(&put)
        .send()
        .await
        .unwrap();
    let st = r.status().as_u16();
    let b: Value = r.json().await.unwrap();
    assert_eq!(st, 200, "{b}");
}

/// V1/V2 : ligne `{ amount, chf, side }` sans `rate`, sans `debit`/`credit`,
/// sans `rateSource` → 201, cours déduit (`rate_from_chf`), provenance
/// `manual` (aucun cours `afc` retenu pour le mois) ; T3 : sans `currency`, la
/// ligne prend celle du compte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_sans_cours_ni_colonnes_ni_provenance(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let corps = json!({ "lines": [
        { "accountId": ctx.usd, "foreign": { "amount": "100", "chf": "91.19", "side": "debit" } },
        chf(ctx.s.accounts["2000"], "0", "91.19"),
    ]});
    let (st, body) = envoyer(&ctx, "/api/v1/opening-balances", &corps).await;
    assert_eq!(st, 201, "{body}");
    let f = &body["lines"][0]["foreign"];
    assert_eq!(f["currency"], "USD", "{body}");
    assert_eq!(f["rate"], "0.9119", "{body}");
    assert_eq!(f["rateSource"], "manual", "{body}");
    assert_eq!(body["lines"][0]["debit"], "91.1900", "{body}");
}

/// A32 (IF1) : même ligne, le mois de la date d'ouverture (janvier 2020) ayant
/// un cours `afc` retenu qui redonne le CHF → provenance `afc`, cours retenu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_au_cours_deduit_qui_redonne_le_cours_afc(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, 'USD', 'month', '2020-01-01', 0.91193, 'afc')",
    )
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let corps = json!({ "lines": [
        { "accountId": ctx.usd, "foreign": { "amount": "100", "chf": "91.19", "side": "debit" } },
        chf(ctx.s.accounts["2000"], "0", "91.19"),
    ]});
    let (st, body) = envoyer(&ctx, "/api/v1/opening-balances", &corps).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["lines"][0]["foreign"]["rateSource"], "afc", "{body}");
    assert_eq!(body["lines"][0]["foreign"]["rate"], "0.91193", "{body}");
}

/// Refus de la génération : statuts et codes (jamais un 422 d'axum).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn ouverture_refus_par_cause(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let contre = |c: &str| chf(ctx.s.accounts["2000"], "0", c);
    let cas: Vec<(Value, u16, &str)> = vec![
        // CHF fourni ≠ round(montant × cours, 2).
        (
            json!({ "lines": [fx(ctx.usd, "debit", "88.47", "USD", "100", "0.8846"), contre("88.47")] }),
            400,
            "FX_AMOUNT_MISMATCH",
        ),
        // Devise annoncée ≠ devise du compte.
        (
            json!({ "lines": [fx(ctx.usd, "debit", "88.46", "EUR", "100", "0.8846"), contre("88.46")] }),
            400,
            "FX_CURRENCY_MISMATCH",
        ),
        // `foreign` sur un compte CHF.
        (
            json!({ "lines": [fx(ctx.s.accounts["1000"], "debit", "88.46", "USD", "100", "0.8846"), contre("88.46")] }),
            400,
            "FX_ACCOUNT_IN_CHF",
        ),
        // `rateSource: "revaluation"` : 400, non le 422 d'axum.
        (
            json!({ "lines": [
                { "accountId": ctx.usd, "debit": "88.46", "credit": "0",
                  "foreign": { "amount": "100", "rate": "0.8846", "rateSource": "revaluation" } },
                contre("88.46"),
            ]}),
            400,
            "VALIDATION_ERROR",
        ),
        // Ligne CHF sans `credit` : 400, non plus 422.
        (
            json!({ "lines": [chf(ctx.s.accounts["1000"], "10", "0"), { "accountId": ctx.s.accounts["2000"], "debit": "0" }] }),
            400,
            "VALIDATION_ERROR",
        ),
        // `debit`/`credit` absents sans `side`.
        (
            json!({ "lines": [{ "accountId": ctx.usd, "foreign": { "amount": "100", "rate": "0.8846" } }, contre("88.46")] }),
            400,
            "VALIDATION_ERROR",
        ),
        // Ni cours ni CHF (ni `chf`, ni `debit`/`credit` : rien d'où déduire le cours).
        (
            json!({ "lines": [{ "accountId": ctx.usd, "foreign": { "amount": "100", "side": "debit" } }, contre("88.46")] }),
            400,
            "VALIDATION_ERROR",
        ),
        // `chf` ≠ `debit` + `credit`.
        (
            json!({ "lines": [
                { "accountId": ctx.usd, "debit": "88.46", "credit": "0",
                  "foreign": { "amount": "100", "chf": "88.40", "rate": "0.8846" } },
                contre("88.46"),
            ]}),
            400,
            "FX_AMOUNT_MISMATCH",
        ),
    ];
    for (corps, statut, attendu) in cas {
        let (st, body) = envoyer(&ctx, "/api/v1/opening-balances", &corps).await;
        assert_eq!((st, code(&body)), (statut, attendu), "{corps} → {body}");
    }
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// Exercice mono-devise (clos avant l'activation, puis rouvert ; fabriqué par
/// les routes d'activation et de réouverture) : une ligne sur un compte en
/// devise est refusée par le serveur, 409 `MULTI_CURRENCY_DISABLED` avec
/// `details.fiscalYearId`, à la génération comme au complément.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn exercice_mono_devise_refuse_l_ouverture_en_devise(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let c = s.company_id;
    // Exercice 2019 clos, antérieur à l'exercice du seed : il devient le
    // premier exercice.
    let clos: i64 = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status) \
         VALUES (?, 'Exercice 2019', '2019-01-01', '2019-12-31', 'Closed')",
    )
    .bind(c)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let gain = compte(&pool, c, "6840", "Revenue", None, None).await;
    let loss = compte(&pool, c, "6940", "Expense", None, None).await;
    let transit = compte(&pool, c, "1090", "Asset", None, None).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", c);
    let version: i64 = sqlx::query_scalar("SELECT version FROM companies WHERE id = ?")
        .bind(c)
        .fetch_one(&pool)
        .await
        .unwrap();
    let r = app
        .client
        .put(format!(
            "{}/api/v1/companies/current/currency-settings",
            app.base_url
        ))
        .bearer_auth(&jwt)
        .json(&json!({
            "multiCurrencyEnabled": true, "fxGainAccountId": gain, "fxLossAccountId": loss,
            "fxTransitAccountId": transit, "version": version,
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "activation");
    let r = app
        .client
        .post(format!(
            "{}/api/v1/fiscal-years/{clos}/reopen",
            app.base_url
        ))
        .bearer_auth(&jwt)
        .json(&json!({ "motif": "Correction" }))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 200, "réouverture");
    let usd = compte(&pool, c, "1022", "Asset", Some("USD"), None).await;
    let corps = json!({ "lines": [
        fx(usd, "debit", "88.46", "USD", "100", "0.8846"),
        chf(s.accounts["2000"], "0", "88.46"),
    ]});
    let r = app
        .client
        .post(format!("{}/api/v1/opening-balances", app.base_url))
        .bearer_auth(&jwt)
        .json(&corps)
        .send()
        .await
        .unwrap();
    let st = r.status().as_u16();
    let body: Value = r.json().await.unwrap();
    assert_eq!(
        (st, code(&body)),
        (409, "MULTI_CURRENCY_DISABLED"),
        "{body}"
    );
    assert_eq!(body["error"]["details"]["fiscalYearId"], clos, "{body}");
}

async fn generer_ouverture_chf(ctx: &Ctx) {
    let corps = json!({ "lines": [
        chf(ctx.s.accounts["1000"], "100", "0"),
        chf(ctx.s.accounts["2000"], "0", "100"),
    ]});
    let (st, body) = envoyer(ctx, "/api/v1/opening-balances", &corps).await;
    assert_eq!(st, 201, "{body}");
}

/// Complément à lignes EUR et USD accepté par sa route (exemption de
/// `FX_MULTIPLE_CURRENCIES`), contrepartie CHF sur le compte de report ; le
/// statut expose la devise des comptes complétables.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complement_a_deux_devises_contrepartie_au_report(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    generer_ouverture_chf(&ctx).await;
    let r = ctx
        .app
        .client
        .get(format!(
            "{}/api/v1/opening-balances/status",
            ctx.app.base_url
        ))
        .bearer_auth(&ctx.jwt)
        .send()
        .await
        .unwrap();
    let statut: Value = r.json().await.unwrap();
    let completables = statut["completableAccounts"].as_array().unwrap();
    let eur = completables
        .iter()
        .find(|a| a["id"] == ctx.eur)
        .expect("compte EUR complétable");
    assert_eq!(eur["currency"], "EUR", "{statut}");
    let chf_compte = completables
        .iter()
        .find(|a| a["id"] == ctx.transit)
        .expect("compte CHF complétable");
    assert!(chf_compte.get("currency").is_none(), "{statut}");

    let corps = json!({ "lines": [
        fx(ctx.eur, "debit", "947.00", "EUR", "1000", "0.947"),
        { "accountId": ctx.usd, "foreign": { "currency": "USD", "amount": "100", "chf": "88.46", "side": "credit", "rateSource": "manual" } },
    ]});
    let (st, body) = envoyer(&ctx, "/api/v1/opening-balances/complete", &corps).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["lines"][0]["foreign"]["currency"], "EUR");
    assert_eq!(body["lines"][1]["foreign"]["rate"], "0.8846");
    assert_eq!(body["lines"][1]["credit"], "88.4600");
    // Contrepartie : 947.00 − 88.46 = 858.54 au crédit du report.
    assert_eq!(body["lines"][2]["accountId"], ctx.retained, "{body}");
    assert_eq!(body["lines"][2]["credit"], "858.5400", "{body}");
    assert!(body["lines"][2].get("foreign").is_none());
}

/// Chaque refus FX du complément rend son code en 400, contrôlé avant la
/// transaction — jamais le 500 du filet `opening-complement:unbalanced`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complement_refus_fx_en_400(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    generer_ouverture_chf(&ctx).await;
    let avant = nb_ecritures(&pool, ctx.s.company_id).await;
    let cas: Vec<(Value, &str)> = vec![
        (
            json!({ "lines": [fx(ctx.usd, "debit", "88.47", "USD", "100", "0.8846")] }),
            "FX_AMOUNT_MISMATCH",
        ),
        (
            // 0.001 × 1 = 0.001 → 0.00 CHF.
            json!({ "lines": [{ "accountId": ctx.usd, "foreign": { "amount": "0.001", "rate": "1", "side": "debit" } }] }),
            "FX_LINE_ROUNDS_TO_ZERO",
        ),
        (
            json!({ "lines": [{ "accountId": ctx.usd, "foreign": { "amount": "1.00001", "rate": "1", "side": "debit" } }] }),
            "FX_VALUE_OUT_OF_RANGE",
        ),
        (
            json!({ "lines": [fx(ctx.usd, "debit", "88.46", "EUR", "100", "0.8846")] }),
            "FX_CURRENCY_MISMATCH",
        ),
        (
            json!({ "lines": [{ "accountId": ctx.usd, "debit": "88.46",
                "foreign": { "amount": "100", "rate": "0.8846", "rateSource": "revaluation" } }] }),
            "VALIDATION_ERROR",
        ),
    ];
    for (corps, attendu) in cas {
        let (st, body) = envoyer(&ctx, "/api/v1/opening-balances/complete", &corps).await;
        assert_eq!((st, code(&body)), (400, attendu), "{corps} → {body}");
        if attendu.starts_with("FX_") && attendu != "FX_CURRENCY_MISMATCH" {
            assert_eq!(body["error"]["details"]["line"], 1, "{body}");
        }
    }
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, avant);
}

/// A32 (IF1) au complément : ligne sans `rateSource`, au CHF qui redonne le
/// cours `afc` retenu du mois de la date du complément → `afc`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn complement_au_cours_deduit_qui_redonne_le_cours_afc(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    generer_ouverture_chf(&ctx).await;
    // Date du complément : premier jour du premier exercice (exercice ouvert,
    // aucun verrou) — janvier 2020.
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, 'USD', 'month', '2020-01-01', 0.91193, 'afc')",
    )
    .bind(ctx.s.company_id)
    .execute(&pool)
    .await
    .unwrap();
    let corps = json!({ "lines": [
        { "accountId": ctx.usd, "foreign": { "amount": "100", "chf": "91.19", "side": "debit" } },
    ]});
    let (st, body) = envoyer(&ctx, "/api/v1/opening-balances/complete", &corps).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entryDate"], "2020-01-01", "{body}");
    assert_eq!(body["lines"][0]["foreign"]["rateSource"], "afc", "{body}");
    assert_eq!(body["lines"][0]["foreign"]["rate"], "0.91193", "{body}");
}

// ============================================================
// AC-2 bis — conversion entre devises
// ============================================================

const ROUTE: &str = "/api/v1/journal-entries/conversions";

fn ecriture(date: &str, libelle: &str, lignes: Vec<Value>) -> Value {
    json!({ "entryDate": date, "journal": "OD", "description": libelle, "lines": lignes })
}

/// Écriture 1 de la maquette : EUR 1000 × 0.9470 = 947.00 au crédit du
/// compte EUR, débit du compte de conversion.
fn ecriture_1(ctx: &Ctx) -> Value {
    ecriture(
        "2026-03-05",
        "Conversion EUR → USD (1/2)",
        vec![
            chf(ctx.transit, "947.00", "0"),
            fx(ctx.eur, "credit", "947.00", "EUR", "1000", "0.947"),
        ],
    )
}

/// Écriture 2 sans ligne de différence : USD 1078.40 × 0.8846 = 953.95 au
/// débit de `usd`, crédit du compte de conversion 947.00 (déséquilibrée de
/// 6.95 : un gain).
fn ecriture_2(usd: i64, ctx: &Ctx) -> Value {
    ecriture(
        "2026-03-06",
        "Conversion EUR → USD (2/2)",
        vec![
            fx(usd, "debit", "953.95", "USD", "1078.40", "0.8846"),
            chf(ctx.transit, "0", "947.00"),
        ],
    )
}

fn conversion(e1: Value, e2: Value, complete: Option<bool>) -> Value {
    let mut c = json!({ "entries": [e1, e2] });
    if let Some(b) = complete {
        c["completeDifference"] = json!(b);
    }
    c
}

/// Sans frais, `completeDifference: true` : la ligne de gain (6.95) est
/// ajoutée en dernière ligne de l'écriture à compléter, au crédit du compte de
/// gains de la société ; réponse dans l'ordre du corps ; audit ×2 ; compte de
/// conversion à 0.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_sans_frais_gain_calcule(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), Some(true));
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!(st, 201, "{body}");
    let entries = body["entries"].as_array().unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["description"], "Conversion EUR → USD (1/2)");
    let lignes = entries[1]["lines"].as_array().unwrap();
    assert_eq!(lignes.len(), 3, "{body}");
    assert_eq!(lignes[2]["accountId"], ctx.gain);
    assert_eq!(lignes[2]["credit"], "6.9500");
    assert_eq!(solde(&pool, ctx.transit).await, "0");
    for e in entries {
        let id = e["id"].as_i64().unwrap();
        let actions = common::audit_actions(&pool, "journal_entry", id).await;
        assert_eq!(actions, vec!["journal_entry.created".to_string()], "{id}");
    }
    // La ligne ajoutée figure à l'audit.
    let details = common::audit_details(
        &pool,
        "journal_entry",
        entries[1]["id"].as_i64().unwrap(),
        "journal_entry.created",
    )
    .await
    .unwrap();
    assert!(details.to_string().contains("6.95"), "{details}");
}

/// Sans le champ (ou à `false`), la même conversion déséquilibrée rend
/// `ENTRY_UNBALANCED` de l'écriture à compléter (`details.entry`), rien de créé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_sans_demande_rend_entry_unbalanced(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    for complete in [None, Some(false)] {
        let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), complete);
        let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
        assert_eq!((st, code(&body)), (400, "ENTRY_UNBALANCED"), "{body}");
        assert_eq!(body["error"]["details"]["entry"], 2, "{body}");
    }
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// La ligne de différence portée par le corps (celle de l'assistant), sur la
/// paire résolue, sans le champ → 201 telle quelle ; portée hors de la paire →
/// 400 `Validation` (le contrôle ne dépend pas du champ) ; gain porté au
/// débit → 400 ; avec le champ → 400 (A31).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_ligne_de_difference_portee(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let avec = |ligne: Value| {
        let mut e2 = ecriture_2(ctx.usd, &ctx);
        e2["lines"].as_array_mut().unwrap().push(ligne);
        e2
    };
    // Hors de la paire : le gain d'un autre compte (compte propre de `usd_own`).
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), avec(chf(ctx.own_gain, "0", "6.95")), None),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2, "{body}");
    assert_eq!(body["error"]["details"]["line"], 3, "{body}");
    // Gain au débit (comptes de gains et de pertes distincts).
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), avec(chf(ctx.gain, "6.95", "0")), None),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    // A31 : ligne portée et demande.
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(
            ecriture_1(&ctx),
            avec(chf(ctx.gain, "0", "6.95")),
            Some(true),
        ),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2, "{body}");
    assert_eq!(body["error"]["details"]["line"], 3, "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
    // Sur la paire, sans le champ.
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), avec(chf(ctx.gain, "0", "6.95")), None),
    )
    .await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entries"][1]["lines"].as_array().unwrap().len(), 3);
}

/// Compte USD à comptes propres → la ligne calculée va sur ses comptes ; la
/// ligne de différence portée sur le compte de gains de la société → 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_compte_a_comptes_propres(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let mut e2 = ecriture_2(ctx.usd_own, &ctx);
    e2["lines"]
        .as_array_mut()
        .unwrap()
        .push(chf(ctx.gain, "0", "6.95"));
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, None)).await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["line"], 3, "{body}");

    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd_own, &ctx), Some(true));
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entries"][1]["lines"][2]["accountId"], ctx.own_gain);

    // Perte (USD 1000 × 0.9 = 900 contre 947) : au débit du compte de pertes propre.
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.usd_own, "debit", "900.00", "USD", "1000", "0.9"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entries"][1]["lines"][2]["accountId"], ctx.own_loss);
    assert_eq!(body["entries"][1]["lines"][2]["debit"], "47.0000");
}

/// Frais aux deux écritures, perte : l'écriture 1 porte un frais CHF au débit,
/// l'écriture 2 un frais 6911 USD (A33) ; la différence se calcule après les
/// frais (perte au débit du compte de pertes) ; frais tels quels.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_avec_frais_perte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e1 = ecriture(
        "2026-03-05",
        "Conversion (1/2)",
        vec![
            chf(ctx.transit, "942.00", "0"),
            chf(ctx.fee_chf, "5.00", "0"),
            fx(ctx.eur, "credit", "947.00", "EUR", "1000", "0.947"),
        ],
    );
    // USD 1000 × 0.9 = 900.00 ; frais USD 10 × 0.9 = 9.00 ; transit 942.00 :
    // débits 909.00 < crédits 942.00 → perte 33.00 au débit.
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.usd, "debit", "900.00", "USD", "1000", "0.9"),
            fx(ctx.fee_usd, "debit", "9.00", "USD", "10", "0.9"),
            chf(ctx.transit, "0", "942.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(e1, e2, Some(true))).await;
    assert_eq!(st, 201, "{body}");
    let l2 = body["entries"][1]["lines"].as_array().unwrap();
    assert_eq!(l2[1]["accountId"], ctx.fee_usd);
    assert_eq!(l2[1]["debit"], "9.0000");
    assert_eq!(l2[3]["accountId"], ctx.loss, "{body}");
    assert_eq!(l2[3]["debit"], "33.0000", "{body}");
    assert_eq!(solde(&pool, ctx.transit).await, "0");
}

/// Différence nulle : aucune ligne ajoutée, même avec le champ.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_difference_nulle(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.usd, "debit", "947.00", "USD", "1000", "0.947"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entries"][1]["lines"].as_array().unwrap().len(), 2);
}

/// V1/V2 : écritures passées sans `rate` ni `debit`/`credit`
/// (`{ amount, chf, side }`) → 201.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_sans_calcul_du_client(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e1 = ecriture(
        "2026-03-05",
        "Conversion (1/2)",
        vec![
            chf(ctx.transit, "947.00", "0"),
            json!({ "accountId": ctx.eur, "foreign": { "amount": "1000", "chf": "947.00", "side": "credit" } }),
        ],
    );
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            json!({ "accountId": ctx.usd, "foreign": { "amount": "1078.40", "chf": "953.95", "side": "debit" } }),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(e1, e2, Some(true))).await;
    assert_eq!(st, 201, "{body}");
    assert_eq!(body["entries"][1]["lines"][2]["credit"], "6.9500");
}

/// Refus qui visent la conversion entière (sans `details.entry`) : compte de
/// conversion déséquilibré, compte de conversion non désigné ou invalide.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_refus_du_compte_de_conversion(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Transit : 947 au débit, 900 au crédit.
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.usd, "debit", "900.00", "USD", "1000", "0.9"),
            chf(ctx.transit, "0", "900.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, None)).await;
    assert_eq!(
        (st, code(&body)),
        (400, "FX_CONVERSION_TRANSIT_UNBALANCED"),
        "{body}"
    );
    assert!(body["error"]["details"].get("entry").is_none(), "{body}");
    assert_eq!(body["error"]["details"]["entry1"], "947.00", "{body}");
    assert_eq!(body["error"]["details"]["entry2"], "-900.00", "{body}");

    // Compte de conversion archivé.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.transit)
        .execute(&pool)
        .await
        .unwrap();
    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), Some(true));
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!((st, code(&body)), (400, "FX_ACCOUNT_INVALID"), "{body}");
    assert_eq!(body["error"]["details"]["field"], "fxTransitAccountId");
    assert!(body["error"]["details"].get("entry").is_none(), "{body}");

    // Compte de conversion non désigné.
    sqlx::query("UPDATE companies SET fx_transit_account_id = NULL WHERE id = ?")
        .bind(ctx.s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!((st, code(&body)), (400, "FX_ACCOUNT_INVALID"), "{body}");
    assert_eq!(body["error"]["details"]["reason"], "missing", "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// Comptes de différence invalides (compte propre archivé) : refus à l'usage
/// (`FX_ACCOUNT_INVALID`, `holderAccountId`), rien de créé ; écriture à
/// compléter équilibrée → 201 (paire non lue).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_paire_lue_a_l_usage_seulement(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.own_gain)
        .execute(&pool)
        .await
        .unwrap();
    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd_own, &ctx), Some(true));
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!((st, code(&body)), (400, "FX_ACCOUNT_INVALID"), "{body}");
    assert_eq!(body["error"]["details"]["holderAccountId"], ctx.usd_own);
    assert_eq!(body["error"]["details"]["entry"], 2, "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);

    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.usd_own, "debit", "947.00", "USD", "1000", "0.947"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!(st, 201, "{body}");
}

/// Formes non complétées : l'écriture à compléter sans compte en devise, au
/// crédit net, ou à deux comptes de la même devise → `ENTRY_UNBALANCED` ;
/// deux comptes de même devise avec une ligne de différence portée → 400.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_formes_non_completees(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Compte en devise au crédit net (`side: "credit"` par erreur).
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            chf(ctx.fee_chf, "1000.00", "0"),
            fx(ctx.usd, "credit", "40.00", "USD", "50", "0.8"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!((st, code(&body)), (400, "ENTRY_UNBALANCED"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2);

    // Deux comptes USD distincts hors comptes de charge.
    let deux = |extra: Option<Value>| {
        let mut lignes = vec![
            fx(ctx.usd, "debit", "900.00", "USD", "1000", "0.9"),
            fx(ctx.usd_bis, "debit", "45.00", "USD", "50", "0.9"),
            chf(ctx.transit, "0", "947.00"),
        ];
        if let Some(l) = extra {
            lignes.push(l);
        }
        ecriture("2026-03-06", "Conversion (2/2)", lignes)
    };
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), deux(None), Some(true)),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "ENTRY_UNBALANCED"), "{body}");
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(
            ecriture_1(&ctx),
            deux(Some(chf(ctx.loss, "2.00", "0"))),
            None,
        ),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["line"], 4, "{body}");

    // Seul compte en devise : un compte de charge (6911) → jamais complétée.
    let e2 = ecriture(
        "2026-03-06",
        "Conversion (2/2)",
        vec![
            fx(ctx.fee_usd, "debit", "9.00", "USD", "10", "0.9"),
            chf(ctx.fee_chf, "900.00", "0"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!((st, code(&body)), (400, "ENTRY_UNBALANCED"), "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// Deux écritures déséquilibrées : le refus est celui de l'écriture qui n'est
/// pas à compléter, même placée en second dans le corps et datée après ;
/// l'écriture 1 déséquilibrée n'est jamais complétée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_l_autre_ecriture_doit_s_equilibrer_seule(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Écriture à compléter (transit au crédit) en tête, sans compte en devise,
    // déséquilibrée ; l'autre (transit au débit) en second, datée après,
    // déséquilibrée elle aussi.
    let a_completer = ecriture(
        "2026-03-05",
        "A",
        vec![
            chf(ctx.fee_chf, "100.00", "0"),
            chf(ctx.transit, "0", "947.00"),
        ],
    );
    let autre = ecriture(
        "2026-03-07",
        "B",
        vec![
            chf(ctx.transit, "947.00", "0"),
            fx(ctx.eur, "credit", "900.00", "EUR", "1000", "0.9"),
        ],
    );
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(a_completer, autre, Some(true))).await;
    assert_eq!((st, code(&body)), (400, "ENTRY_UNBALANCED"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2, "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// A33 : frais sur 6911 USD au crédit → 400 (`details.line` = le frais) ;
/// frais sur un compte de charge EUR dans l'écriture USD →
/// `FX_MULTIPLE_CURRENCIES` (avec le champ, et avec une ligne portée sans
/// lui) ; frais CHF + frais 6911 USD → 201, différence après eux.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_frais_a33(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let e2 = |frais: Value, extra: Option<Value>| {
        let mut lignes = vec![
            fx(ctx.usd, "debit", "900.00", "USD", "1000", "0.9"),
            frais,
            chf(ctx.transit, "0", "947.00"),
        ];
        if let Some(l) = extra {
            lignes.push(l);
        }
        ecriture("2026-03-06", "Conversion (2/2)", lignes)
    };
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(
            ecriture_1(&ctx),
            e2(fx(ctx.fee_usd, "credit", "9.00", "USD", "10", "0.9"), None),
            Some(true),
        ),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2);
    assert_eq!(body["error"]["details"]["line"], 2);

    let frais_eur = fx(ctx.fee_eur, "debit", "9.00", "EUR", "10", "0.9");
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), e2(frais_eur.clone(), None), Some(true)),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "FX_MULTIPLE_CURRENCIES"), "{body}");
    assert_eq!(
        body["error"]["details"]["currencies"],
        json!(["EUR", "USD"])
    );
    assert_eq!(body["error"]["details"]["entry"], 2);
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(
            ecriture_1(&ctx),
            e2(frais_eur, Some(chf(ctx.gain, "0", "38.00"))),
            None,
        ),
    )
    .await;
    assert_eq!((st, code(&body)), (400, "FX_MULTIPLE_CURRENCIES"), "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);

    // Frais CHF + frais 6911 : 900 + 9 + 2 = 911 < 947 → perte 36.00.
    let mut corps2 = e2(
        fx(ctx.fee_usd, "debit", "9.00", "USD", "10", "0.9"),
        Some(chf(ctx.fee_chf, "2.00", "0")),
    );
    let _ = &mut corps2;
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(ecriture_1(&ctx), corps2, Some(true)),
    )
    .await;
    assert_eq!(st, 201, "{body}");
    let l = body["entries"][1]["lines"].as_array().unwrap();
    assert_eq!(l.len(), 5, "{body}");
    assert_eq!(l[4]["accountId"], ctx.loss);
    assert_eq!(l[4]["debit"], "36.0000");
}

/// Refus de la seconde écriture **exécutée** (la plus tardive, placée en tête
/// du corps) : rien de créé, `details.entry` = son rang dans le corps.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_refus_de_la_seconde_creation(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    // Le compte EUR archivé : l'écriture 1 (datée après, en tête du corps)
    // est créée en second et refusée.
    let e1 = ecriture(
        "2026-03-09",
        "Conversion (1/2)",
        vec![
            chf(ctx.transit, "947.00", "0"),
            fx(ctx.eur, "credit", "947.00", "EUR", "1000", "0.947"),
        ],
    );
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.eur)
        .execute(&pool)
        .await
        .unwrap();
    let (st, body) = envoyer(
        &ctx,
        ROUTE,
        &conversion(e1, ecriture_2(ctx.usd, &ctx), Some(true)),
    )
    .await;
    assert_eq!(st, 400, "{body}");
    assert_eq!(body["error"]["details"]["entry"], 1, "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// Forme du corps et rôles : `entries` de longueur 0, 1 ou 3 → 400 ; champ
/// non booléen → 400 ; trop de lignes au corps complété → 400 ; Consultation
/// → 403 ; clé d'API en lecture seule → refusée.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_forme_et_roles(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    for n in [0usize, 1, 3] {
        let entries: Vec<Value> = (0..n).map(|_| ecriture_1(&ctx)).collect();
        let (st, body) = envoyer(&ctx, ROUTE, &json!({ "entries": entries })).await;
        assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{n} : {body}");
    }
    let mut corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), None);
    corps["completeDifference"] = json!("oui");
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");

    // 500 lignes dans l'écriture à compléter : la ligne calculée ne s'ajoute pas.
    let mut lignes = vec![
        fx(ctx.usd, "debit", "900.00", "USD", "1000", "0.9"),
        chf(ctx.transit, "0", "947.00"),
    ];
    for _ in 0..498 {
        lignes.push(chf(ctx.fee_chf, "0.01", "0"));
    }
    let e2 = ecriture("2026-03-06", "Conversion (2/2)", lignes);
    let (st, body) = envoyer(&ctx, ROUTE, &conversion(ecriture_1(&ctx), e2, Some(true))).await;
    assert_eq!((st, code(&body)), (400, "VALIDATION_ERROR"), "{body}");
    assert_eq!(body["error"]["details"]["entry"], 2, "{body}");

    let consult = forge_jwt(ctx.s.admin_user_id, "Consultation", ctx.s.company_id);
    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), Some(true));
    let (st, _) = envoyer_avec(&ctx, &consult, ROUTE, &corps).await;
    assert_eq!(st, 403);
    let (_, cle) = common::audit_traces::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "lecture",
        "read",
    )
    .await;
    let (st, body) = envoyer_avec(&ctx, &cle, ROUTE, &corps).await;
    assert_eq!(st, 403, "{body}");
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}

/// Date dans un exercice mono-devise (exercice multi-devises rendu
/// mono-devise par SQL, comme après une réouverture d'un exercice clos avant
/// l'activation) : `MULTI_CURRENCY_DISABLED`, `details.entry`, rien de créé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn conversion_dans_un_exercice_mono_devise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    sqlx::query("UPDATE fiscal_years SET multi_currency = FALSE WHERE id = ?")
        .bind(ctx.s.fiscal_year_id)
        .execute(&pool)
        .await
        .unwrap();
    let corps = conversion(ecriture_1(&ctx), ecriture_2(ctx.usd, &ctx), Some(true));
    let (st, body) = envoyer(&ctx, ROUTE, &corps).await;
    assert_eq!(
        (st, code(&body)),
        (409, "MULTI_CURRENCY_DISABLED"),
        "{body}"
    );
    assert_eq!(body["error"]["details"]["entry"], 1, "{body}");
    assert_eq!(
        body["error"]["details"]["fiscalYearId"],
        ctx.s.fiscal_year_id
    );
    assert_eq!(nb_ecritures(&pool, ctx.s.company_id).await, 0);
}
