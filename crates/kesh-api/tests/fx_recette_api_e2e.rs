//! Story 37-5 (AC-7, V7, V8, V12 ; CR-6) — la recette multi-devises **par l'API
//! seule**, de l'activation aux rapports, sans écran ni cookie : chaque appel
//! en `Authorization: Bearer` d'une session (`POST /api/v1/auth/login`,
//! `accessToken`), l'Administrateur pour l'activation et la création du
//! Comptable, le Comptable pour le reste ; source de cours factice de la 37-2
//! (`AppState::new_for_tests`, aucun appel sortant).
//!
//! Étapes, chacune contrôlée par sa réponse : comptes de différence (6950,
//! 6940) et de conversion (1090) ; activation ; comptes USD (à comptes de
//! différence propres 6841/6842) et EUR ; exercice 2024 multi-devises (créé
//! après l'activation) ; récupération des cours de mars 2024 et du 31.12.2024 ;
//! écritures en devise (au cours retenu sans `rateSource` → `afc` ; par `chf`
//! égal au produit → `afc` ; par `chf` différent → `manual`, cours déduit ;
//! par `side`) ; réévaluation au 31.12.2024 avec les cours des livres
//! d'origine (V8) ; rapports ; lecture MCP de l'écriture de réévaluation (V12).
//!
//! ⚠️ **Écart écrit** : l'ouverture d'un compte en devise par
//! `POST /opening-balances` (`{ amount, chf, side }`) et la conversion
//! `POST /journal-entries/conversions` (`completeDifference`) sont livrées par
//! la 37-3b, absente de la base de la 37-5 : leurs deux étapes s'ajouteront à
//! ce test à la fusion de la 37-3b.
//!
//! Tous les montants CHF envoyés et attendus sont des **littéraux** (données
//! des livres d'origine, ou fixés d'après les cours que la source factice rend
//! — mensuel USD 0.8800, EUR 0.9300 ; journalier USD 0.89120, EUR 0.94450),
//! jamais des produits calculés par le test.

mod common;

use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

use chrono::TimeDelta;
use kesh_api::config::Config;
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::seed_accounting_company;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::mcp_pieges::{banc, structure};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";

struct App {
    base: String,
    client: reqwest::Client,
}

impl App {
    async fn envoyer(
        &self,
        methode: reqwest::Method,
        jeton: &str,
        chemin: &str,
        corps: Option<&Value>,
    ) -> (u16, Value) {
        let mut r = self
            .client
            .request(methode, format!("{}{chemin}", self.base))
            .bearer_auth(jeton);
        if let Some(c) = corps {
            r = r.json(c);
        }
        let r = r.send().await.unwrap();
        let st = r.status().as_u16();
        (st, r.json().await.unwrap_or(Value::Null))
    }
    async fn post(&self, jeton: &str, chemin: &str, corps: &Value) -> (u16, Value) {
        self.envoyer(reqwest::Method::POST, jeton, chemin, Some(corps)).await
    }
    async fn put(&self, jeton: &str, chemin: &str, corps: &Value) -> (u16, Value) {
        self.envoyer(reqwest::Method::PUT, jeton, chemin, Some(corps)).await
    }
    async fn get(&self, jeton: &str, chemin: &str) -> (u16, Value) {
        self.envoyer(reqwest::Method::GET, jeton, chemin, None).await
    }
    async fn login(&self, nom: &str, mot: &str) -> String {
        let r: Value = self
            .client
            .post(format!("{}/api/v1/auth/login", self.base))
            .json(&json!({ "username": nom, "password": mot }))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        r["accessToken"]
            .as_str()
            .unwrap_or_else(|| panic!("connexion de {nom} : {r}"))
            .to_string()
    }
}

async fn spawn(pool: &MySqlPool) -> App {
    let config = Config::from_fields_for_test(
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
    );
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
    let state = AppState::new_for_tests(pool.clone(), Arc::new(config), Arc::new(rate_limiter), i18n);
    let app = build_router(state, "nonexistent-static-dir".to_string());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
            .await
            .unwrap();
    });
    App {
        base: format!("http://{addr}"),
        client: reqwest::Client::new(),
    }
}

fn dec_de(v: &Value) -> Decimal {
    Decimal::from_str(v.as_str().unwrap_or_else(|| panic!("décimal attendu : {v}"))).unwrap()
}

#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn recette_multi_devises_par_l_api(pool: MySqlPool) {
    // Montage : la société du seed, sans exercice (l'exercice 2024 se crée par
    // l'API, après l'activation). Aucune écriture n'existe encore.
    let s = seed_accounting_company(&pool).await.unwrap();
    sqlx::query("DELETE FROM fiscal_years WHERE company_id = ?")
        .bind(s.company_id)
        .execute(&pool)
        .await
        .unwrap();
    let app = spawn(&pool).await;
    let admin = app.login("admin", "admin123").await;

    // Le Comptable, créé par l'Administrateur.
    let (st, v) = app
        .post(
            &admin,
            "/api/v1/users",
            &json!({ "username": "comptable", "password": "comptable-mot-de-passe-12", "role": "Comptable" }),
        )
        .await;
    assert_eq!(st, 201, "{v}");
    let compta = app.login("comptable", "comptable-mot-de-passe-12").await;

    // Comptes de différence et de conversion (en CHF).
    let creer = |numero: &'static str, nom: &'static str, typ: &'static str, extra: Value| {
        let app = &app;
        let compta = compta.clone();
        async move {
            let mut corps = json!({ "number": numero, "name": nom, "accountType": typ, "postable": true });
            for (k, v) in extra.as_object().unwrap() {
                corps[k] = v.clone();
            }
            let (st, v) = app.post(&compta, "/api/v1/accounts", &corps).await;
            assert_eq!(st, 201, "compte {numero} : {v}");
            v["id"].as_i64().unwrap()
        }
    };
    let gain = creer("6950", "Gains de change", "Revenue", json!({})).await;
    let perte = creer("6940", "Pertes de change", "Expense", json!({})).await;
    let conversion = creer("1090", "Conversion entre devises", "Asset", json!({})).await;
    let gain_pp = creer("6841", "Gains de change PayPal", "Revenue", json!({})).await;
    let perte_pp = creer("6842", "Pertes de change PayPal", "Expense", json!({})).await;

    // Activation (Administrateur), à la version lue.
    let (_, societe) = app.get(&admin, "/api/v1/companies/current").await;
    let version = societe["company"]["version"].as_i64().unwrap();
    let (st, v) = app
        .put(
            &admin,
            "/api/v1/companies/current/currency-settings",
            &json!({
                "multiCurrencyEnabled": true,
                "fxGainAccountId": gain,
                "fxLossAccountId": perte,
                "fxTransitAccountId": conversion,
                "version": version,
            }),
        )
        .await;
    assert_eq!(st, 200, "activation : {v}");

    // Comptes de bilan en devise : PayPal USD à comptes propres, Wise EUR.
    let usd = creer(
        "1020",
        "PayPal USD",
        "Asset",
        json!({ "currency": "USD", "fxGainAccountId": gain_pp, "fxLossAccountId": perte_pp }),
    )
    .await;
    let eur = creer("1030", "Wise EUR", "Asset", json!({ "currency": "EUR" })).await;

    // Exercice 2024, multi-devises parce que créé après l'activation.
    let (st, v) = app
        .post(
            &compta,
            "/api/v1/fiscal-years",
            &json!({ "name": "Exercice 2024", "startDate": "2024-01-01", "endDate": "2024-12-31" }),
        )
        .await;
    assert_eq!(st, 201, "{v}");
    assert_eq!(v["multiCurrency"], true);
    let fy = v["id"].as_i64().unwrap();

    // Récupération des cours (source factice) : mars 2024 et le 31.12.2024.
    for corps in [json!({ "year": 2024, "month": 3 }), json!({ "date": "2024-12-31" })] {
        let (st, v) = app.post(&compta, "/api/v1/exchange-rates/fetch", &corps).await;
        assert_eq!(st, 200, "récupération {corps} : {v}");
    }

    // Écritures du 15.03.2024, contre la caisse (1000).
    let caisse = s.accounts["1000"];
    let ecrire = |ligne: Value, contre: Value| {
        let app = &app;
        let compta = compta.clone();
        async move {
            let (st, v) = app
                .post(
                    &compta,
                    "/api/v1/journal-entries",
                    &json!({
                        "entryDate": "2024-03-15",
                        "journal": "OD",
                        "description": "Livres d'origine",
                        "lines": [ligne, contre],
                    }),
                )
                .await;
            assert_eq!(st, 201, "{v}");
            v
        }
    };
    // (1) Au cours retenu (0.88), sans `rateSource` : relue `afc`.
    let e1 = ecrire(
        json!({ "accountId": usd, "debit": "880.00", "foreign": { "amount": "1000", "rate": "0.88" } }),
        json!({ "accountId": caisse, "credit": "880.00" }),
    )
    .await;
    assert_eq!(e1["lines"][0]["foreign"]["rateSource"], "afc", "{e1}");
    // (2) Par `chf` égal à round(200 × 0.93) = 186.00 (littéral) : `afc`, au
    // cours retenu exact (A32).
    let e2 = ecrire(
        json!({ "accountId": eur, "debit": "186.00", "foreign": { "amount": "200", "chf": "186.00" } }),
        json!({ "accountId": caisse, "credit": "186.00" }),
    )
    .await;
    assert_eq!(e2["lines"][0]["foreign"]["rateSource"], "afc", "{e2}");
    assert_eq!(dec_de(&e2["lines"][0]["foreign"]["rate"]), dec!(0.93));
    // (3) Par `chf` différent du produit (100 × 0.88 = 88.00) : 90.00, relue
    // `manual`, au cours déduit 0.9.
    let e3 = ecrire(
        json!({ "accountId": usd, "debit": "90.00", "foreign": { "amount": "100", "chf": "90.00" } }),
        json!({ "accountId": caisse, "credit": "90.00" }),
    )
    .await;
    assert_eq!(e3["lines"][0]["foreign"]["rateSource"], "manual", "{e3}");
    assert_eq!(dec_de(&e3["lines"][0]["foreign"]["rate"]), dec!(0.9));
    // (4) Sans débit ni crédit, par `side` : 50 USD au crédit, 44.00 CHF.
    let e4 = ecrire(
        json!({ "accountId": usd, "foreign": { "amount": "50", "rate": "0.88", "side": "credit" } }),
        json!({ "accountId": caisse, "debit": "44.00" }),
    )
    .await;
    assert_eq!(dec_de(&e4["lines"][0]["credit"]), dec!(44.00), "{e4}");

    // Réévaluation au 31.12.2024 (V8) : les cours des livres d'origine.
    let rates = json!({ "USD": "0.91193", "EUR": "0.94947" });
    let (st, apercu) = app
        .post(
            &compta,
            &format!("/api/v1/fiscal-years/{fy}/fx-revaluation/preview"),
            &json!({ "rates": rates, "accountIds": [usd, eur] }),
        )
        .await;
    assert_eq!(st, 200, "{apercu}");
    let ligne_usd = apercu["lines"]
        .as_array()
        .unwrap()
        .iter()
        .find(|l| l["accountId"] == usd)
        .unwrap();
    // USD : 1000 + 100 − 50 = 1050 ; CHF 880 + 90 − 44 = 926.00 ;
    // 1050 × 0.91193 = 957.53 → écart +31.53, sur le compte propre 6841.
    assert_eq!(dec_de(&ligne_usd["deviseNet"]), dec!(1050));
    assert_eq!(dec_de(&ligne_usd["chfNet"]), dec!(926.00));
    assert_eq!(ligne_usd["delta"], "31.53");
    assert_eq!(ligne_usd["counterpartAccountId"], gain_pp);
    assert_eq!(ligne_usd["fxAccountsOrigin"], "own");
    let (st, passe) = app
        .post(
            &compta,
            &format!("/api/v1/fiscal-years/{fy}/fx-revaluation"),
            &json!({ "rates": rates, "accountIds": [usd, eur] }),
        )
        .await;
    assert_eq!(st, 200, "{passe}");
    let ids: Vec<i64> = passe["entryIds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_i64().unwrap())
        .collect();
    assert_eq!(ids.len(), 2, "une écriture par compte (USD, EUR)");
    // Genre `request` et source `manual` pour chaque devise : les deux cours
    // diffèrent des cours journaliers factices (0.89120, 0.94450).
    let (details,): (Value,) = sqlx::query_as(
        "SELECT details_json FROM audit_log WHERE action = 'fx_revaluation.posted'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    for devise in ["USD", "EUR"] {
        assert_eq!(details["rateKinds"][devise], "request", "{details}");
        assert_eq!(details["sources"][devise], "manual", "{details}");
    }

    // Rapports : solde CHF au 31.12.2024 = round(solde en devise × cours).
    let (st, balance) = app
        .get(&compta, &format!("/api/v1/reports/trial-balance?fiscalYearId={fy}"))
        .await;
    assert_eq!(st, 200, "{balance}");
    let ligne = |id: i64| {
        balance["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["accountId"] == id)
            .unwrap_or_else(|| panic!("compte {id} absent de la balance : {balance}"))
            .clone()
    };
    assert_eq!(dec_de(&ligne(usd)["closingBalance"]), dec!(957.53));
    assert_eq!(dec_de(&ligne(usd)["closingCurrency"]), dec!(1050));
    // EUR : 200 × 0.94947 = 189.89.
    assert_eq!(dec_de(&ligne(eur)["closingBalance"]), dec!(189.89));
    assert_eq!(dec_de(&ligne(eur)["closingCurrency"]), dec!(200));
    for rapport in ["balance-sheet", "general-ledger"] {
        let (st, v) = app
            .get(&compta, &format!("/api/v1/reports/{rapport}?fiscalYearId={fy}"))
            .await;
        assert_eq!(st, 200, "{rapport} : {v}");
    }

    // Lecture MCP (V12) : serveur activé par l'Administrateur, filtre
    // d'adresse admettant le pair du banc, clé `books:read` dispensée de
    // validation ; `kesh_ecritures` puis `kesh_ecriture` rendent les données.
    let (_, reglages) = app.get(&admin, "/api/v1/admin/mcp/settings").await;
    let (st, v) = app
        .put(
            &admin,
            "/api/v1/admin/mcp/settings",
            &json!({
                "enabled": true,
                "addressFilterEnabled": true,
                "allowedNetworks": ["127.0.0.1/32"],
                "trustedProxies": [],
                "allowedOrigins": [],
                "version": reglages["version"],
            }),
        )
        .await;
    assert_eq!(st, 200, "réglages MCP : {v}");
    let (st, cle) = app
        .post(
            &admin,
            "/api/v1/admin/mcp/keys",
            &json!({ "name": "recette", "scopes": ["books:read"], "sendValidation": "exempt" }),
        )
        .await;
    assert_eq!(st, 201, "clé MCP : {cle}");
    let jeton = cle["key"].as_str().unwrap().to_string();
    let b = banc(&pool);
    let liste = structure(
        &b.appeler(&jeton, "kesh_ecritures", json!({ "accountId": usd })).await,
        "kesh_ecritures",
    );
    assert!(liste["ecritures"].as_array().is_some_and(|e| !e.is_empty()), "{liste}");
    let ecriture = structure(
        &b.appeler(&jeton, "kesh_ecriture", json!({ "id": ids[0] })).await,
        "kesh_ecriture",
    );
    let lignes = ecriture["ecriture"]["lignes"].as_array().unwrap();
    let reev = lignes
        .iter()
        .find(|l| l["devise"] == "USD")
        .unwrap_or_else(|| panic!("ligne USD absente : {ecriture}"));
    assert_eq!(reev["montantDevise"], "0.00", "{ecriture}");
    assert_eq!(reev["sourceCours"], "revaluation", "{ecriture}");
}
