//! Story 37-5 — réévaluation de fin d'exercice par l'API :
//! `POST /api/v1/fiscal-years/{id}/fx-revaluation/preview` et
//! `POST /api/v1/fiscal-years/{id}/fx-revaluation` ; le refus de la clôture
//! (409 `FOREIGN_ACCOUNTS_NOT_REVALUED`).
//!
//! Couvre : forme de l'aperçu (cours, lignes, totaux, réévaluations déjà
//! passées, `blockers[]`), genre et source des cours (défaut journalier, repli
//! mensuel, surcharge, source brute), présélection tronquée à 498, refus de
//! forme et ordre des refus d'avant la transaction, RBAC (Consultation 403,
//! clé en lecture seule 403), clé d'API à l'audit (`for_actor`), passation et
//! rejeu, corps du 409 de la clôture. Aucun appel réseau (cours posés par SQL).

use std::net::SocketAddr;
use std::path::Path;
use std::sync::Arc;

use chrono::{TimeDelta, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::auth::password::hash_password;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::entities::{NewUser, Role};
use kesh_db::repositories::users;
use kesh_db::test_fixtures::{SeededCompany, seed_accounting_company};
use serde_json::{Value, json};
use sqlx::MySqlPool;

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

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
    fy: i64,
    gain: i64,
    perte: i64,
    usd: i64,
    eur: i64,
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    typ: &str,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) \
         VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(typ)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

async fn cours(
    pool: &MySqlPool,
    company_id: i64,
    devise: &str,
    genre: &str,
    date: &str,
    taux: &str,
    source: &str,
) {
    sqlx::query(
        "INSERT INTO exchange_rates (company_id, currency, kind, rate_date, rate, source) \
         VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(devise)
    .bind(genre)
    .bind(date)
    .bind(taux)
    .bind(source)
    .execute(pool)
    .await
    .unwrap();
}

/// Société et exercice 2024 multi-devises ; comptes de différence 6950/6940 ;
/// USD 1000 tenus 880 CHF (1020), EUR 200 tenus 190 CHF (1030), au 15.03.2024
/// (lignes posées par l'API des écritures, cours saisi).
async fn monter(pool: &MySqlPool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    sqlx::query(
        "UPDATE fiscal_years SET name = 'Exercice 2024', start_date = '2024-01-01', \
         end_date = '2024-12-31', multi_currency = TRUE WHERE id = ?",
    )
    .bind(s.fiscal_year_id)
    .execute(pool)
    .await
    .unwrap();
    let gain = compte(pool, s.company_id, "6950", "Revenue", None).await;
    let perte = compte(pool, s.company_id, "6940", "Expense", None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(s.company_id)
    .execute(pool)
    .await
    .unwrap();
    let usd = compte(pool, s.company_id, "1020", "Asset", Some("USD")).await;
    let eur = compte(pool, s.company_id, "1030", "Asset", Some("EUR")).await;
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    let ctx = Ctx {
        app,
        jwt,
        fy: s.fiscal_year_id,
        s,
        gain,
        perte,
        usd,
        eur,
    };
    ecriture(&ctx, ctx.usd, "1000", "880.00", "0.88").await;
    ecriture(&ctx, ctx.eur, "200", "190.00", "0.95").await;
    ctx
}

/// Une écriture au 15.03.2024 : `montant` en devise au débit de `compte`, à
/// `cours` (saisi), contre la caisse.
async fn ecriture(ctx: &Ctx, compte: i64, montant: &str, chf: &str, cours: &str) {
    let body = json!({
        "entryDate": "2024-03-15",
        "journal": "OD",
        "description": "Achat de devises",
        "lines": [
            { "accountId": compte, "debit": chf, "credit": "0",
              "foreign": { "amount": montant, "rate": cours, "rateSource": "manual" } },
            { "accountId": ctx.s.accounts["1000"], "debit": "0", "credit": chf },
        ],
    });
    let r = ctx
        .app
        .client
        .post(format!("{}/api/v1/journal-entries", ctx.app.base_url))
        .bearer_auth(&ctx.jwt)
        .json(&body)
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 201, "{:?}", r.text().await);
}

async fn appel(ctx: &Ctx, jeton: &str, chemin: &str, body: &Value) -> (u16, Value) {
    let r = ctx
        .app
        .client
        .post(format!(
            "{}/api/v1/fiscal-years/{}/{chemin}",
            ctx.app.base_url, ctx.fy
        ))
        .bearer_auth(jeton)
        .json(body)
        .send()
        .await
        .unwrap();
    let status = r.status().as_u16();
    (status, r.json().await.unwrap_or(Value::Null))
}

async fn apercu(ctx: &Ctx, body: &Value) -> (u16, Value) {
    appel(ctx, &ctx.jwt, "fx-revaluation/preview", body).await
}

async fn passer(ctx: &Ctx, body: &Value) -> (u16, Value) {
    appel(ctx, &ctx.jwt, "fx-revaluation", body).await
}

fn ligne(v: &Value, compte: i64) -> &Value {
    v["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["accountId"] == compte)
        .unwrap_or_else(|| panic!("ligne {compte} absente : {v}"))
}

fn cours_de<'a>(v: &'a Value, devise: &str) -> &'a Value {
    v["rates"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == devise)
        .unwrap_or_else(|| panic!("cours {devise} absent : {v}"))
}

async fn nb_reevaluations(pool: &MySqlPool) -> i64 {
    sqlx::query_scalar("SELECT COUNT(*) FROM journal_entry_lines WHERE rate_source = 'revaluation'")
        .fetch_one(pool)
        .await
        .unwrap()
}

// ---------------------------------------------------------------------------
// Aperçu
// ---------------------------------------------------------------------------

/// AC-1 : la forme de l'aperçu au cours par défaut (journalier pour USD,
/// repli mensuel pour EUR), sans rien écrire.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn apercu_forme_et_cours_par_defaut(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.91193", "afc").await;
    cours(&pool, c, "USD", "month", "2024-12-01", "0.8846", "afc").await;
    cours(&pool, c, "EUR", "month", "2024-12-01", "0.9470", "afc").await;
    let (st, v) = apercu(&ctx, &json!({})).await;
    assert_eq!(st, 200, "{v}");
    assert_eq!(v["closingRateKind"], "day");
    assert_eq!(v["endDate"], "2024-12-31");
    let usd = cours_de(&v, "USD");
    assert_eq!(usd["rate"], "0.9119300000");
    assert_eq!(usd["rateKind"], "day");
    assert_eq!(usd["rateDate"], "2024-12-31");
    assert_eq!(usd["defaultSource"], "afc");
    assert_eq!(usd["overridden"], false);
    let eur = cours_de(&v, "EUR");
    assert_eq!(eur["rateKind"], "month", "repli mensuel au genre day");
    assert_eq!(eur["defaultKind"], "month");
    assert_eq!(usd["defaultKind"], "day");
    assert_eq!(eur["rateDate"], "2024-12-01");
    assert_eq!(eur["sourceDate"], Value::Null);

    let l = ligne(&v, ctx.usd);
    assert_eq!(l["accountNumber"], "1020");
    assert_eq!(l["accountName"], "Compte 1020");
    assert_eq!(l["currency"], "USD");
    assert_eq!(l["selected"], true);
    assert_eq!(l["selectable"], true);
    assert_eq!(l["deviseNet"], "1000.0000");
    assert_eq!(l["chfNet"], "880.0000");
    assert_eq!(l["targetChf"], "911.93");
    assert_eq!(l["delta"], "31.93");
    assert_eq!(l["counterpartAccountId"], ctx.gain);
    assert_eq!(l["counterpartAccountNumber"], "6950");
    assert_eq!(l["fxAccountsOrigin"], "company");
    assert_eq!(
        l["label"],
        "Gain de change Compte 1020 USD 0.91193 au 31.12.2024 (non réalisé)"
    );
    assert_eq!(l["previousRevaluationTotal"], "0.00");
    assert_eq!(l["previousEntryIds"], json!([]));
    assert_eq!(l["revaluationTotalAfter"], "31.93");
    // EUR : 200 × 0.947 = 189.40 − 190 = −0.60, au compte de pertes.
    let e = ligne(&v, ctx.eur);
    assert_eq!(e["delta"], "-0.60");
    assert_eq!(e["counterpartAccountId"], ctx.perte);
    assert_eq!(v["totals"]["gains"], "31.93");
    assert_eq!(v["totals"]["losses"], "0.60");
    assert_eq!(v["preselectionTruncated"], false);
    assert_eq!(v["blockers"], json!([]));
    assert_eq!(nb_reevaluations(&pool).await, 0, "l'aperçu n'écrit rien");

    // Le libellé de l'aperçu est celui de l'écriture passée (DRY).
    let (st, p) = passer(&ctx, &json!({ "accountIds": [ctx.usd] })).await;
    assert_eq!(st, 200, "{p}");
    let id = p["entryIds"][0].as_i64().unwrap();
    let desc: String = sqlx::query_scalar("SELECT description FROM journal_entries WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(desc, l["label"].as_str().unwrap());

    // A39 : la réévaluation déjà passée, et la nouvelle valeur cumulée.
    ecriture(&ctx, ctx.usd, "100", "90.00", "0.90").await;
    let (_, v) = apercu(&ctx, &json!({})).await;
    let l = ligne(&v, ctx.usd);
    assert_eq!(l["previousRevaluationTotal"], "31.93");
    assert_eq!(l["previousEntryIds"], json!([id]));
    // 1100 × 0.91193 = 1003.12 ; 880 + 31.93 + 90 = 1001.93 → +1.19.
    assert_eq!(l["delta"], "1.19");
    assert_eq!(l["revaluationTotalAfter"], "33.12");
}

/// Un cours reçu égal au défaut garde son genre ; différent → `request`,
/// `overridden` ; une devise sans cours est signalée, ses comptes exclus.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn apercu_surcharge_et_devise_sans_cours(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    cours(
        &pool,
        ctx.s.company_id,
        "USD",
        "day",
        "2024-12-31",
        "0.91193",
        "afc",
    )
    .await;
    let (st, v) = apercu(&ctx, &json!({ "rates": { "USD": "0.911930" } })).await;
    assert_eq!(st, 200, "{v}");
    assert_eq!(cours_de(&v, "USD")["rateKind"], "day");
    assert_eq!(cours_de(&v, "USD")["overridden"], false);
    let eur = cours_de(&v, "EUR");
    assert_eq!(eur["rate"], Value::Null);
    assert_eq!(eur["defaultRate"], Value::Null);
    let e = ligne(&v, ctx.eur);
    assert_eq!(e["reason"], "RATE_UNAVAILABLE");
    assert_eq!(e["selected"], false);
    assert_eq!(e["selectable"], false);

    // Un cours saisi ici pour EUR : la ligne revient, genre `request`.
    let (st, v) = apercu(&ctx, &json!({ "rates": { "USD": "0.95", "EUR": "0.96" } })).await;
    assert_eq!(st, 200, "{v}");
    assert_eq!(cours_de(&v, "USD")["rateKind"], "request");
    assert_eq!(cours_de(&v, "USD")["overridden"], true);
    assert_eq!(cours_de(&v, "USD")["defaultRate"], "0.9119300000");
    assert_eq!(ligne(&v, ctx.eur)["selected"], true);
    assert_eq!(ligne(&v, ctx.eur)["delta"], "2.00");
}

/// `blockers[]` : `FX_ACCOUNT_INVALID` sur les comptes sélectionnés seulement,
/// avec le message du serveur ; décoché, plus de blocker.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn apercu_blockers_sur_la_selection(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.92", "afc").await;
    cours(&pool, c, "EUR", "day", "2024-12-31", "0.95", "afc").await;
    let g = compte(&pool, c, "6841", "Revenue", None).await;
    let p = compte(&pool, c, "6842", "Expense", None).await;
    sqlx::query("UPDATE accounts SET fx_gain_account_id = ?, fx_loss_account_id = ? WHERE id = ?")
        .bind(g)
        .bind(p)
        .bind(ctx.usd)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(p)
        .execute(&pool)
        .await
        .unwrap();
    let (st, v) = apercu(&ctx, &json!({})).await;
    assert_eq!(st, 200, "{v}");
    let b = &v["blockers"][0];
    assert_eq!(b["code"], "FX_ACCOUNT_INVALID");
    assert_eq!(b["details"]["holderAccountId"], ctx.usd);
    assert!(!b["message"].as_str().unwrap().is_empty());
    assert_eq!(ligne(&v, ctx.usd)["fxAccountsOrigin"], "own");
    let (st, v) = apercu(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    assert_eq!(st, 200, "{v}");
    assert_eq!(v["blockers"], json!([]));
    assert_eq!(ligne(&v, ctx.usd)["selected"], false);
    // La passation : refusée avec, admise sans.
    let (st, e) = passer(&ctx, &json!({ "accountIds": [ctx.usd, ctx.eur] })).await;
    assert_eq!(
        (st, e["error"]["code"].as_str()),
        (400, Some("FX_ACCOUNT_INVALID"))
    );
    let (st, _) = passer(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    assert_eq!(st, 200);

    // Livres verrouillés à la date de fin, exercice antérieur ouvert : signalés.
    sqlx::query("UPDATE companies SET books_locked_through = '2024-12-31' WHERE id = ?")
        .bind(c)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2023', '2023-01-01', '2023-12-31', 'Open', TRUE)",
    )
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    let (_, v) = apercu(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    let codes: Vec<&str> = v["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| b["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, vec!["EARLIER_FISCAL_YEAR_OPEN", "PERIOD_LOCKED"]);
}

/// 499 comptes admissibles : l'aperçu en présélectionne 498 et le signale ;
/// la passation accepte 498 comptes et refuse 499 (`tooMany`).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn preselection_tronquee_et_bornes_des_comptes(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.92", "afc").await;
    cours(&pool, c, "EUR", "day", "2024-12-31", "0.95", "afc").await;
    let mut ids = vec![ctx.usd];
    for i in 0..497 {
        ids.push(compte(&pool, c, &format!("11{i:03}"), "Asset", Some("USD")).await);
    }
    // 498 USD + 1 EUR = 499 admissibles.
    let (st, v) = apercu(&ctx, &json!({})).await;
    assert_eq!(st, 200);
    assert_eq!(v["preselectionTruncated"], true);
    let coches = v["lines"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|l| l["selected"] == true)
        .count();
    assert_eq!(coches, 498);
    let (st, _) = passer(&ctx, &json!({ "accountIds": ids })).await;
    assert_eq!(st, 200, "498 comptes acceptés");
    let mut trop = ids.clone();
    trop.push(ctx.eur);
    let (st, e) = passer(&ctx, &json!({ "accountIds": trop })).await;
    assert_eq!(st, 400);
    assert_eq!(e["error"]["code"], "FX_REVALUATION_ACCOUNT_INVALID");
    assert_eq!(e["error"]["details"]["reason"], "tooMany");
}

/// AC-4 : refus de forme et d'état, avant la transaction.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_de_forme_et_d_etat(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.92", "afc").await;
    let code = |v: &Value| v["error"]["code"].as_str().unwrap_or_default().to_string();

    // Cours invalides : 0, négatif, 11 décimales, > 10⁶, devise sans compte.
    for taux in ["0", "-1", "0.12345678901", "1000000.5"] {
        let (st, v) = passer(
            &ctx,
            &json!({ "rates": { "USD": taux }, "accountIds": [ctx.usd] }),
        )
        .await;
        assert_eq!((st, code(&v).as_str()), (400, "FX_RATE_INVALID"), "{taux}");
        assert_eq!(v["error"]["details"]["currency"], "USD");
    }
    let (st, v) = passer(
        &ctx,
        &json!({ "rates": { "GBP": "1.1" }, "accountIds": [ctx.usd] }),
    )
    .await;
    assert_eq!((st, code(&v).as_str()), (400, "FX_RATE_INVALID"));
    assert_eq!(v["error"]["details"]["currency"], "GBP");
    let cent_une: serde_json::Map<String, Value> =
        (0..101).map(|i| (format!("C{i:02}"), json!("1"))).collect();
    let (st, v) = apercu(&ctx, &json!({ "rates": cent_une })).await;
    assert_eq!((st, code(&v).as_str()), (400, "FX_RATE_INVALID"));
    assert_eq!(v["error"]["details"]["reason"], "tooMany");

    // accountIds : vide, en double ; étranger et inexistant → réponses identiques.
    let (st, v) = passer(&ctx, &json!({ "accountIds": [] })).await;
    assert_eq!(
        (st, v["error"]["details"]["reason"].as_str()),
        (400, Some("empty"))
    );
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.usd, ctx.usd] })).await;
    assert_eq!(
        (st, v["error"]["details"]["reason"].as_str()),
        (400, Some("duplicate"))
    );
    let autre = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let etranger = compte(&pool, autre, "1020", "Asset", Some("USD")).await;
    let (st1, v1) = apercu(&ctx, &json!({ "accountIds": [etranger] })).await;
    let (st2, v2) = apercu(&ctx, &json!({ "accountIds": [987654] })).await;
    assert_eq!((st1, st2), (400, 400));
    assert_eq!(v1["error"]["details"]["reason"], "notEligible");
    assert_eq!(v1["error"]["message"], v2["error"]["message"]);
    assert_eq!(v1["error"]["code"], v2["error"]["code"]);
    // Compte de résultat et compte CHF : notEligible.
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.gain] })).await;
    assert_eq!(
        (st, v["error"]["details"]["reason"].as_str()),
        (400, Some("notEligible"))
    );

    // Cours par défaut manquant (EUR) : 404 RATE_UNAVAILABLE, même avec un
    // exercice antérieur ouvert (refus d'avant la transaction d'abord).
    sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Exercice 2023', '2023-01-01', '2023-12-31', 'Open', TRUE)",
    )
    .bind(c)
    .execute(&pool)
    .await
    .unwrap();
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    assert_eq!((st, code(&v).as_str()), (404, "RATE_UNAVAILABLE"));
    assert_eq!(v["error"]["details"]["month"], "2024-12-01");
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.usd] })).await;
    assert_eq!((st, code(&v).as_str()), (409, "EARLIER_FISCAL_YEAR_OPEN"));

    // Exercice clos ET cours manquant → FISCAL_YEAR_CLOSED ; mono-devise → 409.
    sqlx::query("UPDATE fiscal_years SET status = 'Closed' WHERE id = ?")
        .bind(ctx.fy)
        .execute(&pool)
        .await
        .unwrap();
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    assert_eq!((st, code(&v).as_str()), (400, "FISCAL_YEAR_CLOSED"));
    let (st, v) = apercu(&ctx, &json!({})).await;
    assert_eq!((st, code(&v).as_str()), (400, "FISCAL_YEAR_CLOSED"));
    sqlx::query("UPDATE fiscal_years SET status = 'Open', multi_currency = FALSE WHERE id = ?")
        .bind(ctx.fy)
        .execute(&pool)
        .await
        .unwrap();
    for chemin in ["fx-revaluation/preview", "fx-revaluation"] {
        let (st, v) = appel(&ctx, &ctx.jwt, chemin, &json!({ "accountIds": [ctx.usd] })).await;
        assert_eq!(
            (st, code(&v).as_str()),
            (409, "MULTI_CURRENCY_DISABLED"),
            "{chemin}"
        );
        assert_eq!(v["error"]["details"]["fiscalYearId"], ctx.fy);
    }
    assert_eq!(nb_reevaluations(&pool).await, 0);
}

/// Exercice d'une autre société : 404 ; Consultation : 403, aperçu compris ;
/// clé d'API en lecture seule : 403 ; clé en écriture : passe, et l'audit
/// `fx_revaluation.posted` porte la clé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn rbac_et_cle_d_api(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    cours(
        &pool,
        ctx.s.company_id,
        "USD",
        "day",
        "2024-12-31",
        "0.92",
        "afc",
    )
    .await;
    // Exercice d'une autre société.
    let autre = sqlx::query(
        "INSERT INTO companies (name, address, org_type, accounting_language, instance_language) \
         VALUES ('Autre société', 'Rue 1\n1000 Lausanne', 'Independant', 'FR', 'FR')",
    )
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let fy_autre = sqlx::query(
        "INSERT INTO fiscal_years (company_id, name, start_date, end_date, status, multi_currency) \
         VALUES (?, 'Autre', '2024-01-01', '2024-12-31', 'Open', TRUE)",
    )
    .bind(autre)
    .execute(&pool)
    .await
    .unwrap()
    .last_insert_id() as i64;
    let r = ctx
        .app
        .client
        .post(format!(
            "{}/api/v1/fiscal-years/{fy_autre}/fx-revaluation/preview",
            ctx.app.base_url
        ))
        .bearer_auth(&ctx.jwt)
        .json(&json!({}))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status().as_u16(), 404);

    let creer = |nom: &'static str, role: Role| {
        let pool = pool.clone();
        let c = ctx.s.company_id;
        async move {
            users::create(
                &pool,
                NewUser {
                    username: nom.into(),
                    password_hash: hash_password("password123").unwrap(),
                    role,
                    active: true,
                    company_id: c,
                    email: None,
                },
            )
            .await
            .unwrap()
            .id
        }
    };
    let lecteur = creer("lecteur", Role::Consultation).await;
    let jeton = forge_jwt(lecteur, "Consultation", ctx.s.company_id);
    for chemin in ["fx-revaluation/preview", "fx-revaluation"] {
        let (st, _) = appel(&ctx, &jeton, chemin, &json!({ "accountIds": [ctx.usd] })).await;
        assert_eq!(st, 403, "Consultation : {chemin}");
    }

    let cle = |scope: &'static str| {
        let ctx = &ctx;
        async move {
            let r = ctx
                .app
                .client
                .post(format!("{}/api/v1/settings/api-keys", ctx.app.base_url))
                .bearer_auth(&ctx.jwt)
                .json(&json!({ "name": format!("clé {scope}"), "scope": scope }))
                .send()
                .await
                .unwrap();
            assert_eq!(r.status().as_u16(), 201);
            let b: Value = r.json().await.unwrap();
            (
                b["key"].as_str().unwrap().to_string(),
                b["id"].as_i64().unwrap(),
            )
        }
    };
    let (lecture, _) = cle("read").await;
    for chemin in ["fx-revaluation/preview", "fx-revaluation"] {
        let (st, _) = appel(&ctx, &lecture, chemin, &json!({ "accountIds": [ctx.usd] })).await;
        assert_eq!(st, 403, "clé en lecture seule : {chemin}");
    }
    let (ecriture_cle, cle_id) = cle("read-write").await;
    let (st, v) = appel(&ctx, &ecriture_cle, "fx-revaluation/preview", &json!({})).await;
    assert_eq!(st, 200, "{v}");
    // Le cours prérempli renvoyé tel quel : source `afc`, genre `day`.
    let (st, v) = appel(
        &ctx,
        &ecriture_cle,
        "fx-revaluation",
        &json!({ "rates": { "USD": "0.9200000000" }, "accountIds": [ctx.usd] }),
    )
    .await;
    assert_eq!(st, 200, "{v}");
    let (actor_type, actor_key, details): (String, Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, details_json FROM audit_log \
         WHERE action = 'fx_revaluation.posted'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((actor_type.as_str(), actor_key), ("api_key", Some(cle_id)));
    assert_eq!(details["sources"]["USD"], "afc");
    assert_eq!(details["rateKinds"]["USD"], "day");
    assert_eq!(details["entries"][0]["entryId"], v["entryIds"][0]);
    // Rejeu : rien à passer, `{ entryIds: [] }`, aucune seconde trace.
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.usd] })).await;
    assert_eq!((st, v["entryIds"].clone()), (200, json!([])));
    let n: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM audit_log WHERE action = 'fx_revaluation.posted'")
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(n, 1);
}

/// AC-9 : la clôture refuse en 409 `FOREIGN_ACCOUNTS_NOT_REVALUED`, avec les
/// `details` de K9 et un message ; passe après la passation au cours par
/// défaut. Le panneau rend K9 en attention puis `ok`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cloture_refusee_par_k9(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.92", "afc").await;
    cours(&pool, c, "EUR", "day", "2024-12-31", "0.95", "afc").await;
    let fermer = || async {
        let r = ctx
            .app
            .client
            .post(format!(
                "{}/api/v1/fiscal-years/{}/close",
                ctx.app.base_url, ctx.fy
            ))
            .bearer_auth(&ctx.jwt)
            .json(&json!({}))
            .send()
            .await
            .unwrap();
        let st = r.status().as_u16();
        (st, r.json::<Value>().await.unwrap_or(Value::Null))
    };
    let controles = || async {
        let r = ctx
            .app
            .client
            .get(format!(
                "{}/api/v1/fiscal-years/{}/closing-checks",
                ctx.app.base_url, ctx.fy
            ))
            .bearer_auth(&ctx.jwt)
            .send()
            .await
            .unwrap();
        r.json::<Value>().await.unwrap()
    };
    let k = controles().await;
    assert_eq!(k["checks"][8]["code"], "FOREIGN_ACCOUNTS_NOT_REVALUED");
    assert_eq!(k["checks"][8]["status"], "attention");
    assert_eq!(k["checks"][8]["count"], 1);
    let (st, v) = fermer().await;
    assert_eq!(st, 409, "{v}");
    assert_eq!(v["error"]["code"], "FOREIGN_ACCOUNTS_NOT_REVALUED");
    assert_eq!(
        v["error"]["details"], k["checks"][8]["details"],
        "mêmes détails que K9"
    );
    assert_eq!(v["error"]["details"]["accounts"][0]["accountId"], ctx.usd);
    assert_eq!(v["error"]["details"]["accounts"][0]["delta"], "40.00");
    assert!(
        v["error"]["message"]
            .as_str()
            .unwrap()
            .contains("Réévaluez")
    );

    let (st, _) = passer(&ctx, &json!({ "accountIds": [ctx.usd, ctx.eur] })).await;
    assert_eq!(st, 200);
    assert_eq!(controles().await["checks"][8]["status"], "ok");
    let (st, v) = fermer().await;
    assert_eq!(st, 200, "{v}");
}

/// 37-5-CR-P1-5 : un compte sélectionné par `accountIds` dont la devise n'a
/// pas de cours est signalé dans `blockers[]` (`RATE_UNAVAILABLE`, forme
/// `month`), comme la passation le refuserait ; un compte non admissible sans
/// cours reste `notEligible` à la passation, non `RATE_UNAVAILABLE` ; une clé
/// de cours hors de la sélection se refuse avant `notEligible`, aux deux
/// routes.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_sans_cours_et_ordre_des_refus_aux_deux_routes(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let c = ctx.s.company_id;
    cours(&pool, c, "USD", "day", "2024-12-31", "0.92", "afc").await;
    let code = |v: &Value| v["error"]["code"].as_str().unwrap_or_default().to_string();

    let corps = json!({ "accountIds": [ctx.usd, ctx.eur] });
    let (st, v) = apercu(&ctx, &corps).await;
    assert_eq!(st, 200, "{v}");
    let e = ligne(&v, ctx.eur);
    assert_eq!(
        (e["selected"].clone(), e["selectable"].clone()),
        (json!(true), json!(false))
    );
    let b: Vec<&Value> = v["blockers"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|b| b["code"] == "RATE_UNAVAILABLE")
        .collect();
    assert_eq!(b.len(), 1, "{v}");
    assert_eq!(b[0]["details"]["currency"], "EUR");
    assert_eq!(b[0]["details"]["month"], "2024-12-01");
    assert!(!b[0]["message"].as_str().unwrap().is_empty());
    let (st, v) = passer(&ctx, &corps).await;
    assert_eq!((st, code(&v).as_str()), (404, "RATE_UNAVAILABLE"));
    // Sans le compte EUR : aucun blocker de cours.
    let (_, v) = apercu(&ctx, &json!({ "accountIds": [ctx.usd] })).await;
    assert!(
        v["blockers"]
            .as_array()
            .unwrap()
            .iter()
            .all(|b| b["code"] != "RATE_UNAVAILABLE"),
        "{v}"
    );

    // Clé hors sélection ET compte non admissible : FX_RATE_INVALID aux deux.
    let corps = json!({ "rates": { "GBP": "1.1" }, "accountIds": [ctx.gain] });
    for chemin in ["fx-revaluation/preview", "fx-revaluation"] {
        let (st, v) = appel(&ctx, &ctx.jwt, chemin, &corps).await;
        assert_eq!(
            (st, code(&v).as_str()),
            (400, "FX_RATE_INVALID"),
            "{chemin} : {v}"
        );
    }

    // Compte EUR archivé, sans cours EUR : notEligible, non RATE_UNAVAILABLE.
    sqlx::query("UPDATE accounts SET active = FALSE WHERE id = ?")
        .bind(ctx.eur)
        .execute(&pool)
        .await
        .unwrap();
    let (st, v) = passer(&ctx, &json!({ "accountIds": [ctx.eur] })).await;
    assert_eq!(
        (st, v["error"]["details"]["reason"].as_str()),
        (400, Some("notEligible")),
        "{v}"
    );
    assert_eq!(nb_reevaluations(&pool).await, 0);
}

/// AC-7 (R3) : l'exercice mono-devise sous une société multi-devises, état
/// posé **par l'API** — exercice clos avant l'activation, activation,
/// réouverture : aperçu et passation → 409 `MULTI_CURRENCY_DISABLED`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn mono_devise_pose_par_l_api(pool: MySqlPool) {
    let s = seed_accounting_company(&pool).await.unwrap();
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    let envoyer = |methode: reqwest::Method, chemin: String, body: Value| {
        let (app, jwt) = (&app, &jwt);
        async move {
            let r = app
                .client
                .request(methode, format!("{}{chemin}", app.base_url))
                .bearer_auth(jwt)
                .json(&body)
                .send()
                .await
                .unwrap();
            let st = r.status().as_u16();
            (st, r.json::<Value>().await.unwrap_or(Value::Null))
        }
    };
    let fy = s.fiscal_year_id;
    let (st, v) = envoyer(
        reqwest::Method::POST,
        format!("/api/v1/fiscal-years/{fy}/close"),
        json!({}),
    )
    .await;
    assert_eq!(st, 200, "clôture avant l'activation : {v}");
    assert_eq!(v["multiCurrency"], false);
    let gain = compte(&pool, s.company_id, "6950", "Revenue", None).await;
    let perte = compte(&pool, s.company_id, "6940", "Expense", None).await;
    let transit = compte(&pool, s.company_id, "1090", "Asset", None).await;
    let version: i64 = sqlx::query_scalar("SELECT version FROM companies WHERE id = ?")
        .bind(s.company_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    let (st, v) = envoyer(
        reqwest::Method::PUT,
        "/api/v1/companies/current/currency-settings".into(),
        json!({
            "multiCurrencyEnabled": true,
            "fxGainAccountId": gain,
            "fxLossAccountId": perte,
            "fxTransitAccountId": transit,
            "version": version,
        }),
    )
    .await;
    assert_eq!(st, 200, "activation : {v}");
    let (st, v) = envoyer(
        reqwest::Method::POST,
        format!("/api/v1/fiscal-years/{fy}/reopen"),
        json!({ "motif": "Correction" }),
    )
    .await;
    assert_eq!(st, 200, "réouverture : {v}");
    assert_eq!(v["multiCurrency"], false, "rouvert, il reste mono-devise");
    let usd = compte(&pool, s.company_id, "1020", "Asset", Some("USD")).await;
    for chemin in ["fx-revaluation/preview", "fx-revaluation"] {
        let (st, v) = envoyer(
            reqwest::Method::POST,
            format!("/api/v1/fiscal-years/{fy}/{chemin}"),
            json!({ "accountIds": [usd] }),
        )
        .await;
        assert_eq!(st, 409, "{chemin} : {v}");
        assert_eq!(v["error"]["code"], "MULTI_CURRENCY_DISABLED");
        assert_eq!(v["error"]["details"]["fiscalYearId"], fy);
    }
}
