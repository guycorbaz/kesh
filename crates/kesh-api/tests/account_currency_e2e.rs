//! Story 37-1 (AC-4, AC-4 bis) — la devise d'un compte et ses comptes de
//! différence propres, par l'API : normalisation (`CHF`/`chf` en mono-devise),
//! absence = inchangée, JSON d'un compte CHF sans les clés, chaque code neuf
//! (`MULTI_CURRENCY_DISABLED`, `CURRENCY_UNKNOWN`, `ACCOUNT_CURRENCY_LOCKED`,
//! `ACCOUNT_CURRENCY_IN_USE`, `ACCOUNT_IN_FOREIGN_CURRENCY`,
//! `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`) et la forme `missing` des comptes
//! propres, et les gardes inverses des routes bancaires, des articles et des
//! règles de réconciliation.

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
// Harnais (patron currency_settings_e2e.rs)
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
}

async fn monter(pool: &MySqlPool) -> Ctx {
    let s = seed_accounting_company(pool).await.unwrap();
    let app = spawn_app(pool.clone()).await;
    let jwt = forge_jwt(s.admin_user_id, "Admin", s.company_id);
    Ctx { app, jwt, s }
}

async fn compte(
    pool: &MySqlPool,
    company_id: i64,
    numero: &str,
    type_: &str,
    devise: Option<&str>,
) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, currency) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(company_id)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

/// Active le multi-devises par SQL : `(gain, perte, conversion)`.
async fn multi(pool: &MySqlPool, ctx: &Ctx) -> (i64, i64, i64) {
    let c = ctx.s.company_id;
    let gain = compte(pool, c, "6950", "Revenue", None).await;
    let perte = compte(pool, c, "6960", "Expense", None).await;
    let transit = compte(pool, c, "1090", "Asset", None).await;
    sqlx::query(
        "UPDATE companies SET multi_currency_enabled = TRUE, fx_gain_account_id = ?, \
         fx_loss_account_id = ?, fx_transit_account_id = ? WHERE id = ?",
    )
    .bind(gain)
    .bind(perte)
    .bind(transit)
    .bind(c)
    .execute(pool)
    .await
    .unwrap();
    (gain, perte, transit)
}

async fn envoyer(ctx: &Ctx, methode: reqwest::Method, chemin: &str, corps: Value) -> (u16, Value) {
    let resp = ctx
        .app
        .client
        .request(methode, format!("{}{}", ctx.app.base_url, chemin))
        .bearer_auth(&ctx.jwt)
        .json(&corps)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    (status, resp.json().await.unwrap_or(Value::Null))
}

async fn creer(ctx: &Ctx, corps: Value) -> (u16, Value) {
    envoyer(ctx, reqwest::Method::POST, "/api/v1/accounts", corps).await
}

/// `PUT /accounts/{id}` : nom, type, rôle, postabilité et version relus en base,
/// complétés par `extra`.
async fn modifier(pool: &MySqlPool, ctx: &Ctx, id: i64, extra: Value) -> (u16, Value) {
    let (name, account_type, role, postable, version): (String, String, Option<String>, bool, i32) =
        sqlx::query_as(
            "SELECT name, account_type, role, postable, version FROM accounts WHERE id = ?",
        )
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap();
    let mut corps = json!({
        "name": name, "accountType": account_type, "role": role,
        "postable": postable, "version": version,
    });
    for (k, v) in extra.as_object().unwrap() {
        corps[k] = v.clone();
    }
    envoyer(
        ctx,
        reqwest::Method::PUT,
        &format!("/api/v1/accounts/{id}"),
        corps,
    )
    .await
}

fn code(corps: &Value) -> &str {
    corps["error"]["code"].as_str().unwrap_or("")
}

const CLES_DEVISE: [&str; 4] = [
    "currency",
    "currencyLocked",
    "fxGainAccountId",
    "fxLossAccountId",
];

// ============================================================
// Normalisation, absence, sérialisation
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn chf_est_normalise_et_une_devise_refusee_en_mono_devise(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    for (numero, devise) in [("1020", "CHF"), ("1021", " chf ")] {
        let (st, a) = creer(
            &ctx,
            json!({ "number": numero, "name": "Banque", "accountType": "Asset", "currency": devise }),
        )
        .await;
        assert_eq!(st, 201, "{a}");
        for cle in CLES_DEVISE {
            assert!(a.get(cle).is_none(), "{cle} absente d'un compte CHF : {a}");
        }
    }
    let (st, e) = creer(
        &ctx,
        json!({ "number": "1022", "name": "Banque", "accountType": "Asset", "currency": "usd" }),
    )
    .await;
    assert_eq!((st, code(&e)), (409, "MULTI_CURRENCY_DISABLED"), "{e}");
    assert_eq!(e["error"]["details"], json!({}));

    // `BGN`, `HRK` : monnaies retirées, admises pour un cours de change
    // seulement (correctif `fix-cours-afc-bgn`) — jamais pour un compte.
    for inconnu in ["XYZ", "US", "éur", "", "BGN", "HRK"] {
        let (st, e) = creer(
            &ctx,
            json!({ "number": "1023", "name": "Banque", "accountType": "Asset", "currency": inconnu }),
        )
        .await;
        assert_eq!((st, code(&e)), (400, "CURRENCY_UNKNOWN"), "{inconnu} : {e}");
        assert_eq!(e["error"]["details"]["value"], inconnu);
    }

    // PUT : `CHF` renvoyé sur un compte CHF en mono-devise — no-op, pas un refus.
    let id = ctx.s.accounts["1100"];
    let (st, a) = modifier(&pool, &ctx, id, json!({ "currency": "chf" })).await;
    assert_eq!(st, 200, "{a}");
    assert_eq!(a["version"], 1);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn devise_absente_inchangee_null_rend_chf(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let (st, a) = creer(
        &ctx,
        json!({ "number": "1020", "name": "Banque USD", "accountType": "Asset", "currency": "usd" }),
    )
    .await;
    assert_eq!(st, 201, "{a}");
    assert_eq!(a["currency"], "USD");
    let id = a["id"].as_i64().unwrap();

    // Absente = inchangée (un client qui ne connaît pas le champ n'efface rien).
    let (st, a) = modifier(&pool, &ctx, id, json!({ "name": "Banque dollars" })).await;
    assert_eq!(st, 200, "{a}");
    assert_eq!(a["currency"], "USD");
    // `null` = CHF ; la réponse n'a plus aucune des clés.
    let (st, a) = modifier(&pool, &ctx, id, json!({ "currency": null })).await;
    assert_eq!(st, 200, "{a}");
    for cle in CLES_DEVISE {
        assert!(a.get(cle).is_none(), "{cle} : {a}");
    }
}

/// Correctif `fix-cours-afc-bgn` (revue P1-2) : une monnaie retirée (`BGN`,
/// `HRK`), admise pour un cours de change, ne se pose pas non plus par
/// `PUT /accounts/{id}` — ni sur un compte en CHF, ni sur un compte en USD
/// sans écriture —, en multi-devises : `400 CURRENCY_UNKNOWN`, devise et
/// version inchangées en base.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn monnaie_retiree_refusee_a_la_modification_d_un_compte(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let chf = compte(&pool, ctx.s.company_id, "1021", "Asset", None).await;
    let usd = compte(&pool, ctx.s.company_id, "1022", "Asset", Some("USD")).await;
    for (id, avant) in [(chf, None), (usd, Some("USD"))] {
        for retiree in ["BGN", "HRK", "bgn"] {
            let (version_avant,): (i32,) =
                sqlx::query_as("SELECT version FROM accounts WHERE id = ?")
                    .bind(id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let (st, e) = modifier(&pool, &ctx, id, json!({ "currency": retiree })).await;
            assert_eq!(
                (st, code(&e)),
                (400, "CURRENCY_UNKNOWN"),
                "{id} → {retiree} : {e}"
            );
            assert_eq!(e["error"]["details"]["value"], retiree);
            let (devise, version): (Option<String>, i32) =
                sqlx::query_as("SELECT currency, version FROM accounts WHERE id = ?")
                    .bind(id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(devise.as_deref(), avant, "{id} → {retiree}");
            assert_eq!(version, version_avant, "{id} → {retiree}");
        }
    }
    // Sonde positive : la même route pose une monnaie active.
    let (st, a) = modifier(&pool, &ctx, chf, json!({ "currency": "EUR" })).await;
    assert_eq!(st, 200, "{a}");
    assert_eq!(a["currency"], "EUR");
}

// ============================================================
// Codes de refus
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn devise_verrouillee_apres_une_ligne(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let id = compte(&pool, ctx.s.company_id, "1020", "Asset", None).await;
    // La ligne se passe sur le compte encore CHF.
    let je: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, 1, '2026-06-01', 'Banque', 'test') RETURNING id",
    )
    .bind(ctx.s.company_id)
    .bind(ctx.s.fiscal_year_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    for (ordre, compte_ligne, debit, credit) in [(1, id, 10, 0), (2, ctx.s.accounts["1000"], 0, 10)]
    {
        sqlx::query(
            "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(je)
        .bind(compte_ligne)
        .bind(ordre)
        .bind(debit)
        .bind(credit)
        .execute(&pool)
        .await
        .unwrap();
    }
    let (st, e) = modifier(&pool, &ctx, id, json!({ "currency": "USD" })).await;
    assert_eq!((st, code(&e)), (409, "ACCOUNT_CURRENCY_LOCKED"), "{e}");
}

/// 37-5-CR-P3-1 — un compte de bilan en devise qui porte une ligne ne reçoit
/// pas de sous-compte : 409, le parent nommé, rien n'est créé.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn sous_compte_refuse_sous_un_compte_en_devise_mouvemente(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let id = compte(&pool, ctx.s.company_id, "1020", "Asset", Some("USD")).await;
    let je: i64 = sqlx::query_scalar(
        "INSERT INTO journal_entries (company_id, fiscal_year_id, entry_number, entry_date, journal, description) \
         VALUES (?, ?, 1, '2026-06-01', 'Banque', 'test') RETURNING id",
    )
    .bind(ctx.s.company_id)
    .bind(ctx.s.fiscal_year_id)
    .fetch_one(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO journal_entry_lines (entry_id, account_id, line_order, debit, credit) \
         VALUES (?, ?, 1, 10, 0)",
    )
    .bind(je)
    .bind(id)
    .execute(&pool)
    .await
    .unwrap();
    let (st, e) = creer(
        &ctx,
        json!({ "number": "1021", "name": "Détail", "accountType": "Asset", "currency": "USD", "parentId": id }),
    )
    .await;
    assert_eq!(
        (st, code(&e)),
        (409, "ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES"),
        "{e}"
    );
    assert_eq!(
        e["error"]["details"],
        json!({ "parentAccountId": id, "parentNumber": "1020", "currency": "USD" })
    );
    assert!(
        e["error"]["message"].as_str().unwrap().contains("1020"),
        "{e}"
    );
    let crees: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM accounts WHERE number = '1021'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(crees, 0);
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn devise_refusee_sur_un_compte_employe_en_chf(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    // 3000 est le compte de produit par défaut des réglages de facturation.
    let (st, e) = modifier(
        &pool,
        &ctx,
        ctx.s.accounts["3000"],
        json!({ "currency": "EUR" }),
    )
    .await;
    assert_eq!((st, code(&e)), (409, "ACCOUNT_CURRENCY_IN_USE"), "{e}");
    assert_eq!(
        e["error"]["details"],
        json!({ "site": "invoiceSettings", "setting": "revenue" })
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn compte_de_difference_garde_par_l_api(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    let (gain, _perte, transit) = multi(&pool, &ctx).await;

    // Archivage sans recours, même confirmé.
    let (st, e) = envoyer(
        &ctx,
        reqwest::Method::PUT,
        &format!("/api/v1/accounts/{gain}/archive"),
        json!({ "version": 1, "confirmDesignated": true }),
    )
    .await;
    assert_eq!(
        (st, code(&e)),
        (409, "ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT"),
        "{e}"
    );
    assert_eq!(e["error"]["details"], json!({ "designation": "gain" }));

    // Conversion : sous-compte refusé.
    let (st, e) = creer(
        &ctx,
        json!({ "number": "10901", "name": "Sous", "accountType": "Asset", "parentId": transit }),
    )
    .await;
    assert_eq!(
        (st, code(&e)),
        (409, "ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT"),
        "{e}"
    );
    assert_eq!(e["error"]["details"]["designation"], "transit");
    // Revue de code P2 (37-1-CR-P2-5) : le message nomme le compte de conversion.
    let msg = e["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("compte de conversion"), "{msg}");

    // Désignation propre : le détenteur est nommé.
    let propre = compte(&pool, ctx.s.company_id, "6970", "Expense", None).await;
    let (st, h) = creer(
        &ctx,
        json!({ "number": "1030", "name": "PayPal", "accountType": "Asset", "currency": "USD",
                "fxGainAccountId": propre, "fxLossAccountId": propre }),
    )
    .await;
    assert_eq!(st, 201, "{h}");
    assert_eq!(h["fxGainAccountId"], propre);
    let h_id = h["id"].as_i64().unwrap();
    let (st, e) = modifier(&pool, &ctx, propre, json!({ "accountType": "Asset" })).await;
    assert_eq!(
        (st, code(&e)),
        (409, "ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT"),
        "{e}"
    );
    assert_eq!(
        e["error"]["details"],
        json!({ "designation": "gain", "holderAccountId": h_id, "holderNumber": "1030" })
    );
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn comptes_propres_forme_et_retour_au_defaut(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let propre = compte(&pool, ctx.s.company_id, "6970", "Expense", None).await;

    // L'un sans l'autre : `missing` sur le champ manquant, avant la transaction.
    let (st, e) = creer(
        &ctx,
        json!({ "number": "1030", "name": "PayPal", "accountType": "Asset", "currency": "USD",
                "fxGainAccountId": propre }),
    )
    .await;
    assert_eq!((st, code(&e)), (400, "FX_ACCOUNT_INVALID"), "{e}");
    assert_eq!(
        e["error"]["details"],
        json!({ "field": "fxLossAccountId", "reason": "missing" })
    );

    let (st, a) = creer(
        &ctx,
        json!({ "number": "1030", "name": "PayPal", "accountType": "Asset", "currency": "USD" }),
    )
    .await;
    assert_eq!(st, 201, "{a}");
    let id = a["id"].as_i64().unwrap();
    for (corps, champ) in [
        (json!({ "fxGainAccountId": propre }), "fxLossAccountId"),
        (json!({ "fxLossAccountId": null }), "fxGainAccountId"),
        (
            json!({ "fxGainAccountId": null, "fxLossAccountId": propre }),
            "fxGainAccountId",
        ),
    ] {
        let (st, e) = modifier(&pool, &ctx, id, corps.clone()).await;
        assert_eq!((st, code(&e)), (400, "FX_ACCOUNT_INVALID"), "{corps} : {e}");
        assert_eq!(e["error"]["details"]["field"], champ, "{corps}");
        assert_eq!(e["error"]["details"]["reason"], "missing");
        assert_eq!(e["error"]["details"]["holderAccountId"], id);
    }

    // Posés, puis rendus au défaut : les clés disparaissent.
    let (st, a) = modifier(
        &pool,
        &ctx,
        id,
        json!({ "fxGainAccountId": propre, "fxLossAccountId": propre }),
    )
    .await;
    assert_eq!(st, 200, "{a}");
    assert_eq!(a["fxLossAccountId"], propre);
    let (st, a) = modifier(
        &pool,
        &ctx,
        id,
        json!({ "fxGainAccountId": null, "fxLossAccountId": null }),
    )
    .await;
    assert_eq!(st, 200, "{a}");
    assert!(a.get("fxGainAccountId").is_none(), "{a}");
    assert_eq!(a["currency"], "USD");

    // `unexpected` sur un compte CHF.
    let (st, e) = modifier(
        &pool,
        &ctx,
        ctx.s.accounts["1100"],
        json!({ "fxGainAccountId": propre, "fxLossAccountId": propre }),
    )
    .await;
    assert_eq!((st, code(&e)), (400, "FX_ACCOUNT_INVALID"), "{e}");
    assert_eq!(e["error"]["details"]["reason"], "unexpected");
    // Revue de code P2 (37-1-CR-P2-3) : la société est multi-devises, le
    // message parle du compte en devise, non du mode.
    let msg = e["error"]["message"].as_str().unwrap_or("");
    assert!(msg.contains("compte en devise"), "{msg}");
    assert!(!msg.contains("mode multi-devises"), "{msg}");
}

// ============================================================
// Gardes inverses des routes de configuration
// ============================================================

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn un_compte_en_devise_n_est_designable_nulle_part(pool: MySqlPool) {
    let ctx = monter(&pool).await;
    multi(&pool, &ctx).await;
    let c = ctx.s.company_id;
    let banque_usd = compte(&pool, c, "1020", "Asset", Some("USD")).await;
    let produit_usd = compte(&pool, c, "3990", "Revenue", Some("USD")).await;
    let charge_usd = compte(&pool, c, "4990", "Expense", Some("USD")).await;

    // Compte bancaire (pré-vol partagé avec l'étape bancaire de l'assistant) —
    // les routes bancaires exigent l'assistant terminé.
    sqlx::query(
        "INSERT INTO onboarding_state (singleton, step_completed, is_demo, ui_mode) \
         VALUES (TRUE, 7, FALSE, 'guided') \
         ON DUPLICATE KEY UPDATE step_completed = 7, version = version + 1",
    )
    .execute(&pool)
    .await
    .unwrap();
    let (st, e) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/bank-accounts",
        json!({ "bankName": "PostFinance", "iban": "CH3908704016075473007",
                "isPrimary": false, "journalAccountId": banque_usd }),
    )
    .await;
    assert_eq!((st, code(&e)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{e}");
    assert_eq!(
        e["error"]["details"],
        json!({ "accountId": banque_usd, "currency": "USD" })
    );

    // Article.
    let (st, e) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/products",
        json!({ "name": "Article", "unitPrice": "100.00", "vatRate": "8.10",
                "defaultRevenueAccountId": produit_usd }),
    )
    .await;
    assert_eq!((st, code(&e)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{e}");

    // Règle de réconciliation.
    let (st, e) = envoyer(
        &ctx,
        reqwest::Method::POST,
        "/api/v1/reconciliation/rules",
        json!({ "label": "R", "matchType": "counterparty_contains", "matchValue": "Swisscom",
                "counterpartyAccountId": charge_usd, "priority": 100 }),
    )
    .await;
    assert_eq!((st, code(&e)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{e}");

    // Rôle posé sur un compte en devise.
    let (st, e) = modifier(&pool, &ctx, banque_usd, json!({ "role": "Receivable" })).await;
    assert_eq!((st, code(&e)), (400, "ACCOUNT_IN_FOREIGN_CURRENCY"), "{e}");
}
