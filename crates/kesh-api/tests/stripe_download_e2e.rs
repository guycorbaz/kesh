//! Story 40-8b (Epic 40, phase 2) — le téléchargement par l'API Stripe, de la
//! route à la base, contre un **serveur Stripe simulé** lancé dans le processus
//! (socle S5 : aucun appel au vrai Stripe ni à `api.stripe.com`).
//!
//! Le serveur simulé (`axum` sur `127.0.0.1:0`) sert les réponses **fictives**
//! de `tests/fixtures/stripe-api/` (40-8a) : quatre versements de février 2020,
//! dont un manuel, **filtrés** par `arrival_date[gte]`/`[lt]` et paginés par
//! deux (curseur `starting_after`), et les transactions de solde de chaque
//! versement automatique ; sur demande, des réponses forcées (429, 401, 403),
//! des données modifiées (montant d'un versement, identifiant trop long) et des
//! requêtes retenues (tâche « en cours »). La clé porte le marqueur du socle S9.
//!
//! L'attente des pauses est notée et ne dort pas ; l'horloge est arrêtée au
//! 2020-06-30 (les fixtures datent de février 2020).

mod common;

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::provider_download::{Downloads, Wait};
use kesh_api::provider_env::{ProviderApiConfig, ProviderSeries, ProvidersConfig};
use kesh_api::stripe::{STRIPE, STRIPE_KEY};
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use reqwest::multipart;
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::secrets_prestataire::{aucun_secret, secret};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const CSV: &[u8] =
    include_bytes!("../../kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv");
/// Le compte Stripe dont la clé est dans l'environnement.
const COMPTE: &str = "Stripe boutique";
/// Un second compte Stripe de Kesh, sans clé.
const COMPTE_SANS_API: &str = "Stripe atelier";
/// Février 2020, la période des fixtures.
const DU: &str = "2020-02-01";
const AU: &str = "2020-02-29";

// ---------------------------------------------------------------------------
// Serveur Stripe simulé
// ---------------------------------------------------------------------------

fn fixture(nom: &str) -> Value {
    let p = format!(
        "{}/tests/fixtures/stripe-api/{nom}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&p).expect(&p)).expect(&p)
}

/// Une réponse forcée : statut, `Retry-After`, corps.
#[derive(Clone, Debug)]
struct Forcee(u16, Option<&'static str>, &'static str);

/// Une requête reçue.
#[derive(Clone, Debug)]
struct Req {
    path: String,
    query: HashMap<String, String>,
    authorization: String,
}

type Regle = Arc<dyn Fn(&Req) -> Option<Forcee> + Send + Sync>;

#[derive(Clone)]
struct Sim {
    /// Les versements, du plus récent au plus ancien (ordre de Stripe).
    payouts: Arc<Mutex<Vec<Value>>>,
    /// Les transactions de solde, par versement.
    transactions: Arc<Mutex<HashMap<String, Value>>>,
    forcees: Arc<Mutex<VecDeque<Forcee>>>,
    regle: Arc<Mutex<Option<Regle>>>,
    requetes: Arc<Mutex<Vec<Req>>>,
    /// Retient les requêtes de la liste des versements tant qu'il est vidé.
    verrou_liste: Arc<tokio::sync::Semaphore>,
    /// Retient les requêtes des transactions tant qu'il est vidé.
    verrou_transactions: Arc<tokio::sync::Semaphore>,
    /// Requêtes de transactions de solde reçues et pas encore répondues.
    en_vol: Arc<std::sync::atomic::AtomicUsize>,
    /// Le plus grand nombre de ces requêtes vu en même temps.
    max_en_vol: Arc<std::sync::atomic::AtomicUsize>,
}

/// Compte une requête de transactions « en vol » le temps de sa vie.
struct EnVol(Arc<std::sync::atomic::AtomicUsize>);

impl Drop for EnVol {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }
}

impl Sim {
    fn compte(&self, path: &str) -> usize {
        self.requetes
            .lock()
            .unwrap()
            .iter()
            .filter(|r| r.path == path)
            .count()
    }
}

/// Versements par page : deux, pour que le curseur serve.
const PAGE: usize = 2;

fn page_de_versements(sim: &Sim, req: &Req) -> Value {
    let gte: i64 = req.query["arrival_date[gte]"].parse().unwrap();
    let lt: i64 = req.query["arrival_date[lt]"].parse().unwrap();
    assert_eq!(req.query.get("status").map(String::as_str), Some("paid"));
    let dans: Vec<Value> = sim
        .payouts
        .lock()
        .unwrap()
        .iter()
        .filter(|p| {
            let a = p["arrival_date"].as_i64().unwrap();
            a >= gte && a < lt
        })
        .cloned()
        .collect();
    let debut = match req.query.get("starting_after") {
        None => 0,
        Some(c) => dans.iter().position(|p| p["id"] == c.as_str()).unwrap() + 1,
    };
    let page: Vec<Value> = dans.iter().skip(debut).take(PAGE).cloned().collect();
    json!({ "object": "list", "data": page, "has_more": debut + PAGE < dans.len() })
}

async fn gerer(
    State(sim): State<Sim>,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let req = Req {
        path: uri.path().to_string(),
        query,
        authorization: headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string(),
    };
    sim.requetes.lock().unwrap().push(req.clone());
    let verrou = if req.path == "/v1/payouts" {
        sim.verrou_liste.clone()
    } else {
        sim.verrou_transactions.clone()
    };
    // Compté avant la retenue : une requête retenue est une requête en vol.
    let _vol = (req.path != "/v1/payouts").then(|| {
        use std::sync::atomic::Ordering::SeqCst;
        let n = sim.en_vol.fetch_add(1, SeqCst) + 1;
        sim.max_en_vol.fetch_max(n, SeqCst);
        EnVol(sim.en_vol.clone())
    });
    let _place = verrou.acquire().await.unwrap();
    let regle = sim.regle.lock().unwrap().clone();
    let forcee = sim
        .forcees
        .lock()
        .unwrap()
        .pop_front()
        .or_else(|| regle.and_then(|r| r(&req)));
    if let Some(Forcee(code, retry, corps)) = forcee {
        let mut r = (StatusCode::from_u16(code).unwrap(), corps.to_string()).into_response();
        if let Some(v) = retry {
            r.headers_mut().insert("retry-after", v.parse().unwrap());
        }
        return r;
    }
    let corps = match req.path.as_str() {
        "/v1/payouts" => page_de_versements(&sim, &req),
        "/v1/balance_transactions" => {
            let po = &req.query["payout"];
            sim.transactions.lock().unwrap()[po].clone()
        }
        autre => panic!("chemin inattendu : {autre}"),
    };
    (StatusCode::OK, corps.to_string()).into_response()
}

async fn simuler() -> (String, Sim) {
    let mut payouts: Vec<Value> = Vec::new();
    for f in ["payouts-p1.json", "payouts-p2.json"] {
        payouts.extend(fixture(f)["data"].as_array().unwrap().iter().cloned());
    }
    let transactions = ["po_TEST0001", "po_TEST0002", "po_TEST0003"]
        .into_iter()
        .map(|po| (po.to_string(), fixture(&format!("bt-{po}.json"))))
        .collect();
    let sim = Sim {
        payouts: Arc::new(Mutex::new(payouts)),
        transactions: Arc::new(Mutex::new(transactions)),
        forcees: Arc::default(),
        regle: Arc::default(),
        requetes: Arc::default(),
        verrou_liste: Arc::new(tokio::sync::Semaphore::new(1000)),
        verrou_transactions: Arc::new(tokio::sync::Semaphore::new(1000)),
        en_vol: Arc::default(),
        max_en_vol: Arc::default(),
    };
    let app = Router::new().fallback(gerer).with_state(sim.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), sim)
}

// ---------------------------------------------------------------------------
// Kesh
// ---------------------------------------------------------------------------

/// Attente notée, qui ne dort pas.
#[derive(Default)]
struct Notee(Mutex<Vec<Duration>>);

impl Wait for Notee {
    fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.0.lock().unwrap().push(d);
        Box::pin(async {})
    }
}

/// La clé de test : restreinte, porteuse du marqueur S9.
fn cle() -> String {
    format!("rk_test_{}", secret("cle").replace('-', "_"))
}

fn config(sim_url: &str, parallel: usize) -> Config {
    let serie = ProviderSeries::for_tests(1, COMPTE, &[(STRIPE_KEY, cle().as_str())]);
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
    .with_providers(
        ProvidersConfig::default().with(ProviderApiConfig::for_tests(
            STRIPE,
            sim_url,
            parallel,
            vec![serie],
        )),
    )
}

fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2020, 6, 30, 12, 0, 0).unwrap()
}

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    attente: Arc<Notee>,
}

async fn spawn_app(pool: MySqlPool, sim_url: &str, parallel: usize) -> TestApp {
    let config = config(sim_url, parallel);
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
    let attente = Arc::new(Notee::default());
    let mut state = AppState::new_for_tests(pool, Arc::new(config), Arc::new(rate_limiter), i18n);
    state = common::horloge::avec_horloge(state, maintenant());
    state.provider_downloads = Arc::new(Downloads::new(
        attente.clone(),
        Some((Duration::from_millis(500), Duration::from_millis(1500))),
    ));
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
        attente,
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

async fn compte(pool: &MySqlPool, c: i64, numero: &str, type_: &str, devise: Option<&str>) -> i64 {
    sqlx::query(
        "INSERT INTO accounts (company_id, number, name, account_type, active, postable, currency) \
         VALUES (?, ?, ?, ?, 1, 1, ?)",
    )
    .bind(c)
    .bind(numero)
    .bind(format!("Compte {numero}"))
    .bind(type_)
    .bind(devise)
    .execute(pool)
    .await
    .unwrap()
    .last_insert_id() as i64
}

struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
}

/// Une société multi-devises et ses soldes Stripe CHF, EUR, USD ; un compte
/// « sans API » en CHF.
async fn monter(pool: &MySqlPool, sim_url: &str, parallel: usize) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let c = seeded.company_id;
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
    let ctx = Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone(), sim_url, parallel).await,
        jwt: forge_jwt(seeded.admin_user_id, "Admin", c),
        seeded,
    };
    for (numero, devise, libelle) in [
        ("1030", None, COMPTE),
        ("1031", Some("EUR"), COMPTE),
        ("1032", Some("USD"), COMPTE),
        ("1034", None, COMPTE_SANS_API),
    ] {
        let ledger = compte(pool, c, numero, "Asset", devise).await;
        let (s, b) = ctx
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "Stripe",
                    "journalAccountId": ledger,
                    "stripe": { "account": libelle, "feeAccountId": null, "payoutBankAccountId": null },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde Stripe : {b}");
    }
    ctx
}

fn code(b: &Value) -> String {
    b["error"]["code"].as_str().unwrap_or("").to_string()
}

impl Ctx {
    async fn req_avec(
        &self,
        jeton: &str,
        methode: reqwest::Method,
        chemin: &str,
        corps: Option<Value>,
    ) -> (u16, Value) {
        let mut r = self
            .app
            .client
            .request(methode, format!("{}{chemin}", self.app.base_url))
            .bearer_auth(jeton);
        if let Some(c) = corps {
            r = r.json(&c);
        }
        let resp = r.send().await.unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, chemin: &str, corps: Value) -> (u16, Value) {
        self.req_avec(&self.jwt, reqwest::Method::POST, chemin, Some(corps))
            .await
    }

    async fn lancer(&self, compte: &str, du: &str, au: &str) -> (u16, Value) {
        self.post(
            "/api/v1/stripe-downloads",
            json!({ "account": compte, "from": du, "to": au }),
        )
        .await
    }

    async fn vue(&self, job: &Value) -> Value {
        let chemin = format!("/api/v1/stripe-downloads/{}", job["jobId"]);
        let (s, v) = self
            .req_avec(&self.jwt, reqwest::Method::GET, &chemin, None)
            .await;
        assert_eq!(s, 200, "{v}");
        v
    }

    /// Suit une tâche jusqu'à sa fin ; rend sa dernière vue.
    async fn suivre(&self, job: &Value) -> Value {
        for _ in 0..1000 {
            let v = self.vue(job).await;
            if v["state"] != "running" {
                return v;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("tâche restée en cours");
    }

    /// Lance février 2020 et suit jusqu'à la fin.
    async fn telecharger(&self) -> Value {
        let (s, b) = self.lancer(COMPTE, DU, AU).await;
        assert_eq!(s, 202, "{b}");
        assert!(b["jobId"].is_u64(), "{b}");
        self.suivre(&b).await
    }

    async fn importer_csv(&self, fichier: &[u8]) -> (u16, Value) {
        let form = multipart::Form::new()
            .part(
                "file",
                multipart::Part::bytes(fichier.to_vec())
                    .file_name("payouts.csv")
                    .mime_str("text/csv")
                    .unwrap(),
            )
            .text("account", COMPTE.to_string());
        let resp = self
            .app
            .client
            .post(format!("{}/api/v1/stripe-imports", self.app.base_url))
            .bearer_auth(&self.jwt)
            .multipart(form)
            .send()
            .await
            .unwrap();
        let status = resp.status().as_u16();
        (status, resp.json().await.unwrap_or(Value::Null))
    }

    async fn count(&self, sql: &str) -> i64 {
        sqlx::query_scalar(sql).fetch_one(&self.pool).await.unwrap()
    }

    /// (imports, transactions, traces d'import et de téléchargement).
    async fn ecrit(&self) -> (i64, i64, i64) {
        (
            self.count("SELECT COUNT(*) FROM bank_imports").await,
            self.count("SELECT COUNT(*) FROM bank_transactions").await,
            self.count(
                "SELECT COUNT(*) FROM audit_log WHERE action IN ('bank_import.created', 'stripe_import.created', 'stripe_download.created')",
            )
            .await,
        )
    }

    /// Les transactions écrites : (devise, identifiant, montant, date), triées.
    async fn transactions(&self) -> Vec<(String, String, String, String)> {
        sqlx::query_as::<_, (String, Option<String>, rust_decimal::Decimal, chrono::NaiveDate)>(
            "SELECT currency, transaction_id, amount, booking_date FROM bank_transactions ORDER BY transaction_id",
        )
        .fetch_all(&self.pool)
        .await
        .unwrap()
        .into_iter()
        .map(|(c, t, a, d)| (c, t.unwrap_or_default(), a.to_string(), d.to_string()))
        .collect()
    }
}

/// `(insérées, doublons)` d'une devise du compte rendu.
fn par_devise(resultat: &Value, devise: &str) -> (i64, i64) {
    let c = resultat["currencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["currency"] == devise)
        .unwrap_or_else(|| panic!("devise {devise} absente : {resultat}"));
    (
        c["inserted"].as_i64().unwrap(),
        c["duplicates"].as_i64().unwrap(),
    )
}

fn total_insere(resultat: &Value) -> i64 {
    resultat["currencies"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["inserted"].as_i64().unwrap())
        .sum()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// AC-1, AC-3 — février 2020 : trois versements automatiques lus (le manuel au
/// compte rendu), ventilés par devise, une entrée `bank_imports` par devise au
/// nom de la période lue, la trace `stripe_download.created` ; la progression
/// finit à « 3/3 » ; relancer la même période n'insère rien.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn telechargement_ventile_par_devise_puis_relance_sans_doublon(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;

    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(v["steps"], json!({ "done": 3, "total": 3 }));
    assert!(v["error"].is_null());
    let r = &v["result"];
    assert_eq!(r["account"], COMPTE);
    assert_eq!(r["manualPayouts"], json!(["po_TEST0005"]));
    let mut lus: Vec<&str> = r["payouts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["payoutId"].as_str().unwrap())
        .collect();
    lus.sort();
    assert_eq!(lus, ["po_TEST0001", "po_TEST0002", "po_TEST0003"]);
    let insere = total_insere(r);
    assert!(insere > 0);
    for devise in ["CHF", "EUR", "USD"] {
        assert!(par_devise(r, devise).0 > 0, "{devise} : {r}");
    }
    // Deux pages de versements (curseur), une liste par versement automatique.
    assert_eq!(sim.compte("/v1/payouts"), 2);
    assert_eq!(sim.compte("/v1/balance_transactions"), 3);
    assert!(
        sim.requetes
            .lock()
            .unwrap()
            .iter()
            .all(|q| q.authorization == format!("Bearer {}", cle()))
    );

    let (imports, transactions, traces) = ctx.ecrit().await;
    assert_eq!((imports, transactions), (3, insere));
    // Une trace par devise et une du téléchargement.
    assert_eq!(traces, 4);
    let noms: Vec<String> = sqlx::query_scalar("SELECT DISTINCT filename FROM bank_imports")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(noms, ["Stripe API 2020-02-01…2020-02-29"]);
    let details: Value = sqlx::query_scalar(
        "SELECT details_json FROM audit_log WHERE action = 'stripe_download.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(details["provider"], "stripe");
    assert_eq!(details["provider_account"], COMPTE);
    assert_eq!(
        details["period"],
        json!({ "from": "2020-02-01", "to": "2020-02-29" })
    );
    assert_eq!(details["manual_payouts"], json!(["po_TEST0005"]));

    // Relance : rien de neuf, aucun refus d'empreinte.
    let avant = ctx.transactions().await;
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(total_insere(&v["result"]), 0);
    for devise in ["CHF", "EUR", "USD"] {
        assert_eq!(
            par_devise(&v["result"], devise),
            (0, par_devise(r, devise).0)
        );
    }
    assert_eq!(ctx.transactions().await, avant);
}

/// AC-3 — le CSV de 40-7a puis le téléchargement des mêmes versements : rien
/// d'inséré ; dans l'autre sens, le CSV n'ajoute que `po_TEST0004`, absent de
/// l'API (une ligne et son versement, en USD).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn csv_et_telechargement_se_melangent_sans_doublon(pool: MySqlPool) {
    let (url, _sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let (s, b) = ctx.importer_csv(CSV).await;
    assert_eq!(s, 201, "{b}");
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(total_insere(&v["result"]), 0, "{v}");
}

/// AC-3 — dans l'autre sens : le téléchargement d'abord, puis le CSV, qui
/// n'insère que ce que l'API n'a pas rendu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn telechargement_puis_csv_n_ajoute_que_le_reste(pool: MySqlPool) {
    let (url, _sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    let (s, b) = ctx.importer_csv(CSV).await;
    assert_eq!(s, 201, "{b}");
    assert_eq!(par_devise(&b, "USD").0, 2, "{b}");
    assert_eq!(par_devise(&b, "CHF").0, 0, "{b}");
    assert_eq!(par_devise(&b, "EUR").0, 0, "{b}");
}

/// AC-2 — `KESH_STRIPE_PARALLEL = 3` rend les mêmes transactions que 1.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn trois_versements_a_la_fois_rendent_les_memes_lignes(pool: MySqlPool) {
    let (url, _sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    let un = ctx.transactions().await;
    sqlx::query("DELETE FROM bank_transactions")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM bank_imports")
        .execute(&pool)
        .await
        .unwrap();
    let trois = Ctx {
        pool: pool.clone(),
        app: spawn_app(pool.clone(), &url, 3).await,
        jwt: ctx.jwt.clone(),
        seeded: ctx.seeded.clone(),
    };
    let v = trois.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(v["steps"], json!({ "done": 3, "total": 3 }));
    assert_eq!(trois.transactions().await, un);
}

/// AC-2 / FR-40-8b-3 — `KESH_STRIPE_PARALLEL` borne les requêtes HTTP de
/// transactions en vol : le serveur les retient, et leur nombre atteint la
/// valeur réglée sans la dépasser (trois versements à lire). La borne est tenue
/// par le `ProviderGate` ; le test ne voit pas le nombre de tâches de lecture
/// (`Run.parallel`), qui n'ajoute aucune requête en vol de plus (revue P2, N1).
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn requetes_http_en_vol_egal_le_reglage(pool: MySqlPool) {
    use std::sync::atomic::Ordering::SeqCst;
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 2).await;
    let retenue = sim.verrou_transactions.acquire_many(1000).await.unwrap();
    let (s, job) = ctx.lancer(COMPTE, DU, AU).await;
    assert_eq!(s, 202, "{job}");
    for _ in 0..500 {
        if sim.en_vol.load(SeqCst) >= 2 {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    // Laisse le temps à un troisième versement de partir, s'il le pouvait.
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(
        sim.en_vol.load(SeqCst),
        2,
        "KESH_STRIPE_PARALLEL=2 : deux requêtes en vol, ni une ni trois"
    );
    drop(retenue);
    let v = ctx.suivre(&job).await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(sim.max_en_vol.load(SeqCst), 2, "jamais plus que le réglage");
}

/// AC-1 — la progression : `total` nul pendant la liste des versements, puis
/// le nombre de versements automatiques et une étape par versement lu.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn progression_par_versement(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let liste = sim.verrou_liste.acquire_many(1000).await.unwrap();
    let transactions = sim.verrou_transactions.acquire_many(1000).await.unwrap();
    let (s, job) = ctx.lancer(COMPTE, DU, AU).await;
    assert_eq!(s, 202, "{job}");
    let v = ctx.vue(&job).await;
    assert_eq!(v["state"], "running");
    assert_eq!(v["steps"], json!({ "done": 0, "total": null }));
    drop(liste);
    // La liste lue, les transactions retenues : le total est connu.
    let mut vu = Value::Null;
    for _ in 0..500 {
        vu = ctx.vue(&job).await;
        if !vu["steps"]["total"].is_null() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    assert_eq!(vu["steps"], json!({ "done": 0, "total": 3 }), "{vu}");
    drop(transactions);
    let v = ctx.suivre(&job).await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(v["steps"], json!({ "done": 3, "total": 3 }));
}

/// AC-2 — un 429 repris par la politique commune (`Retry-After` lu) ; au-delà
/// de six essais, `PROVIDER_RATE_LIMITED` et rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn limitation_de_debit_reprise_puis_epuisee(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    sim.forcees.lock().unwrap().push_back(Forcee(
        429,
        Some("2"),
        r#"{"error":{"type":"invalid_request_error","code":"rate_limit"}}"#,
    ));
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "done", "{v}");
    // La pause est commune : la requête reprise l'attend, et les suivantes
    // attendent ce qu'il en reste (l'attente notée ne dort pas, la pause ne
    // s'écoule donc pas pendant le test).
    let attentes = ctx.app.attente.0.lock().unwrap().clone();
    assert_eq!(attentes[0].as_secs_f64().round() as u64, 2, "{attentes:?}");
    assert!(
        attentes.iter().all(|d| *d <= Duration::from_secs(2)),
        "{attentes:?}"
    );
    // La première page reprise une fois, puis la seconde.
    assert_eq!(sim.compte("/v1/payouts"), 3);

    sqlx::query("DELETE FROM bank_transactions")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM bank_imports")
        .execute(&pool)
        .await
        .unwrap();
    let avant = ctx.ecrit().await;
    *sim.regle.lock().unwrap() = Some(Arc::new(|r: &Req| {
        (r.path == "/v1/balance_transactions").then_some(Forcee(
            429,
            None,
            r#"{"error":{"code":"rate_limit"}}"#,
        ))
    }));
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_RATE_LIMITED");
    assert_eq!(v["error"]["details"]["provider"], "stripe");
    assert_eq!(ctx.ecrit().await, avant, "rien d'écrit");
}

/// AC-2 — 401 et 403 : arrêt immédiat, code de 40-8a, rien d'écrit, aucun
/// secret dans les réponses.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn cle_refusee_et_droits_insuffisants(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let mut corps = Vec::new();
    for (statut, attendu) in [
        (401, "PROVIDER_AUTH_FAILED"),
        (403, "PROVIDER_PERMISSION_DENIED"),
    ] {
        let avant = sim.compte("/v1/payouts");
        sim.forcees.lock().unwrap().push_back(Forcee(
            statut,
            None,
            r#"{"error":{"code":"secret_key_required","message":"FUITE rk_test_x"}}"#,
        ));
        let v = ctx.telecharger().await;
        assert_eq!(v["state"], "failed", "{v}");
        assert_eq!(v["error"]["code"], attendu);
        assert_eq!(sim.compte("/v1/payouts"), avant + 1, "pas de reprise");
        assert!(!v.to_string().contains("FUITE"), "{v}");
        corps.push(v.to_string());
    }
    aucun_secret(corps.iter().map(String::as_str));
    assert!(corps.iter().all(|c| !c.contains(&url)));
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-3 (40-8a AC-3) — un versement dont la somme des nets diffère de son
/// montant : `PROVIDER_RESPONSE_INVALID`, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn versement_au_total_faux_rien_d_ecrit(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    for p in sim.payouts.lock().unwrap().iter_mut() {
        if p["id"] == "po_TEST0002" {
            p["amount"] = json!(p["amount"].as_i64().unwrap() + 1);
        }
    }
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_RESPONSE_INVALID");
    assert_eq!(v["error"]["details"]["payoutId"], "po_TEST0002");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// Revue P1 de la 40-8a, L5 — un identifiant de 300 caractères (forme admise
/// par le client) est refusé par la borne de l'import du CSV avant toute
/// écriture : `PROVIDER_RESPONSE_INVALID`, erreur de ligne `ID`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn identifiant_trop_long_refuse_avant_l_ecriture(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let long = format!("txn_{}", "a".repeat(296));
    {
        let mut t = sim.transactions.lock().unwrap();
        let bt = t.get_mut("po_TEST0001").unwrap();
        let ligne = bt["data"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|x| x["type"] != "payout")
            .unwrap();
        ligne["id"] = json!(long);
    }
    let v = ctx.telecharger().await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_RESPONSE_INVALID", "{v}");
    let ligne = &v["error"]["details"]["lines"][0];
    assert_eq!(ligne["code"], "ID", "{v}");
    assert_eq!(ligne["field"], "balance_transaction_id", "{v}");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-1 — refus dans l'ordre, rien lancé : clé d'API (`POST` et `GET`),
/// Consultation, corps, compte inconnu, compte sans clé, période ; puis une
/// tâche à la fois par compte (`409` avec la tâche et sa période lue), le `GET`
/// limité à la société.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_tache_unique_et_cles_d_api(pool: MySqlPool) {
    let (url, sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;

    let (_, ecriture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let corps = json!({ "account": COMPTE, "from": DU, "to": AU });
    let (s, b) = ctx
        .req_avec(
            &ecriture,
            reqwest::Method::POST,
            "/api/v1/stripe-downloads",
            Some(corps.clone()),
        )
        .await;
    assert_eq!((s, code(&b)), (403, "API_KEY_OPERATION_FORBIDDEN".into()));
    let (s, b) = ctx
        .req_avec(
            &ecriture,
            reqwest::Method::GET,
            "/api/v1/stripe-downloads/1",
            None,
        )
        .await;
    assert_eq!((s, code(&b)), (403, "API_KEY_OPERATION_FORBIDDEN".into()));
    let consultation = forge_jwt(
        ctx.seeded.admin_user_id,
        "Consultation",
        ctx.seeded.company_id,
    );
    let (s, _) = ctx
        .req_avec(
            &consultation,
            reqwest::Method::POST,
            "/api/v1/stripe-downloads",
            Some(corps),
        )
        .await;
    assert_eq!(s, 403);

    let (s, b) = ctx
        .post("/api/v1/stripe-downloads", json!({ "account": COMPTE }))
        .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR".into()));
    let (s, b) = ctx.lancer("Inconnu", DU, AU).await;
    assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND".into()));
    let (s, b) = ctx.lancer(COMPTE_SANS_API, DU, AU).await;
    assert_eq!((s, code(&b)), (422, "PROVIDER_API_NOT_CONFIGURED".into()));
    for (du, au, motif) in [
        ("2020-02-29", "2020-02-01", "from_after_to"),
        ("2020-07-01", "2020-07-02", "future"),
        ("1990-06-29", "1990-07-30", "too_old"),
    ] {
        let (s, b) = ctx.lancer(COMPTE, du, au).await;
        assert_eq!(
            (s, code(&b)),
            (400, "PROVIDER_PERIOD_INVALID".into()),
            "{b}"
        );
        assert_eq!(b["error"]["details"]["reason"], motif);
        assert_eq!(b["error"]["details"]["provider"], "stripe");
        // La profondeur d'historique que dit le texte du refus `too_old`.
        assert_eq!(b["error"]["details"]["historyYears"], 30);
    }
    // Aucune requête n'est partie vers Stripe.
    assert!(sim.requetes.lock().unwrap().is_empty(), "rien lancé");

    // Le serveur retient la liste : la tâche reste en cours. Dix ans d'un
    // coup (aucun plafond de durée, une seule fenêtre) et une fin future
    // (l'horloge est au 2020-06-30) : la tâche garde la fin LUE.
    let place = sim.verrou_liste.acquire_many(1000).await.unwrap();
    let (s, premier) = ctx.lancer(COMPTE, "2010-07-01", "2020-07-15").await;
    assert_eq!(s, 202, "{premier}");
    let (s, b) = ctx.lancer("stripe BOUTIQUE", DU, AU).await;
    assert_eq!((s, code(&b)), (409, "PROVIDER_DOWNLOAD_RUNNING".into()));
    assert_eq!(b["error"]["details"]["jobId"], premier["jobId"]);
    assert_eq!(b["error"]["details"]["provider"], "stripe");
    assert_eq!(
        (
            b["error"]["details"]["from"].as_str(),
            b["error"]["details"]["to"].as_str()
        ),
        (Some("2010-07-01"), Some("2020-06-30"))
    );
    drop(place);
    let v = ctx.suivre(&premier).await;
    assert_eq!(v["state"], "done", "{v}");
    let noms: Vec<String> = sqlx::query_scalar("SELECT DISTINCT filename FROM bank_imports")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(noms, ["Stripe API 2010-07-01…2020-06-30"]);
    // Finie, elle ne bloque plus la suivante.
    let (s, _) = ctx.lancer(COMPTE, DU, AU).await;
    assert_eq!(s, 202);

    let (s, _) = ctx
        .req_avec(
            &ctx.jwt,
            reqwest::Method::GET,
            "/api/v1/stripe-downloads/999999",
            None,
        )
        .await;
    assert_eq!(s, 404);
    // Une tâche d'une autre société est introuvable (404), non refusée : le
    // numéro de tâche ne dit pas qu'elle existe.
    let autre = kesh_db::repositories::companies::create(
        &pool,
        kesh_db::entities::NewCompany {
            name: "Autre SA".into(),
            first_name: None,
            last_name: None,
            address_structured: kesh_db::entities::address::StructuredAddress {
                street: "Rue Test".into(),
                building: "1".into(),
                postal_code: "1000".into(),
                city: "Lausanne".into(),
                country: "CH".into(),
            },
            ide_number: None,
            org_type: kesh_db::entities::OrgType::Independant,
            accounting_language: kesh_db::entities::Language::Fr,
            instance_language: kesh_db::entities::Language::Fr,
        },
    )
    .await
    .unwrap()
    .id;
    let autre_admin = kesh_db::repositories::users::create(
        &pool,
        kesh_db::entities::NewUser {
            username: "autre-admin".into(),
            password_hash: kesh_api::auth::password::hash_password("password123").unwrap(),
            role: kesh_db::entities::Role::Admin,
            active: true,
            company_id: autre,
            email: None,
        },
    )
    .await
    .unwrap()
    .id;
    let jeton_autre = forge_jwt(autre_admin, "Admin", autre);
    let chemin = format!("/api/v1/stripe-downloads/{}", premier["jobId"]);
    let (s, _) = ctx
        .req_avec(&ctx.jwt, reqwest::Method::GET, &chemin, None)
        .await;
    assert_eq!(s, 200, "la société de la tâche la lit");
    let (s, _) = ctx
        .req_avec(&jeton_autre, reqwest::Method::GET, &chemin, None)
        .await;
    assert_eq!(s, 404, "une autre société ne lit pas la tâche");
    // Une tâche Stripe ne se lit pas par la route de PayPal.
    let (s, _) = ctx
        .req_avec(
            &ctx.jwt,
            reqwest::Method::GET,
            &format!("/api/v1/paypal-downloads/{}", premier["jobId"]),
            None,
        )
        .await;
    assert_eq!(s, 404);
}

/// AC-4 — la liste des comptes bancaires dit, pour un compte Stripe, si son
/// libellé a une clé, et le mode lu à la clé (`rk_test_` → `sandbox`), même
/// vers le serveur simulé ; jamais la clé ni l'hôte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn la_liste_des_comptes_dit_le_telechargement_et_son_mode(pool: MySqlPool) {
    let (url, _sim) = simuler().await;
    let ctx = monter(&pool, &url, 1).await;
    let (s, liste) = ctx
        .req_avec(
            &ctx.jwt,
            reqwest::Method::GET,
            "/api/v1/bank-accounts",
            None,
        )
        .await;
    assert_eq!(s, 200, "{liste}");
    let mut vus = (0, 0);
    for c in liste.as_array().unwrap() {
        match c["stripe"]["account"].as_str() {
            Some(COMPTE) => {
                assert_eq!(
                    c["providerApi"],
                    json!({ "configured": true, "mode": "sandbox" }),
                    "{c}"
                );
                vus.0 += 1;
            }
            Some(COMPTE_SANS_API) => {
                assert_eq!(c["providerApi"], json!({ "configured": false }), "{c}");
                vus.1 += 1;
            }
            _ => {}
        }
    }
    assert_eq!(vus, (3, 1), "{liste}");
    let texte = liste.to_string();
    aucun_secret([texte.as_str()]);
    assert!(!texte.contains("127.0.0.1"), "{texte}");
}
