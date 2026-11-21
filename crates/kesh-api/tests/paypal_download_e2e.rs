//! Story 38-12a (Epic 38, phase 2) — le téléchargement par l'API PayPal, contre
//! un **serveur PayPal simulé** lancé dans le processus (AC-10, socle S5 : un
//! seul schéma de test, pas de source simulée).
//!
//! Le serveur simulé (`axum` sur `127.0.0.1:0`) joue le jeton OAuth, des pages
//! à taille réglable, 429 (avec et sans `Retry-After`, et un `Retry-After` de
//! 3600 borné à 120), `400 RESULTSET_TOO_LARGE`, 401, 403, une réponse qui ne
//! vient pas, et des transactions de chaque statut. Données **fictives**
//! (identifiants `0TEST…`, adresses `@example.invalid`) : les transactions de la
//! fixture CSV de 38-11a, traduites dans la forme de l'API, pour qu'un CSV puis
//! un téléchargement des mêmes transactions se comparent. Aucun appel au vrai
//! PayPal.
//!
//! L'attente des pauses est notée et ne dort pas ; l'horloge est arrêtée au
//! 2020-06-30 (la fixture date de 2020).

mod common;

use std::collections::VecDeque;
use std::net::SocketAddr;
use std::path::Path;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use chrono::{DateTime, TimeDelta, TimeZone, Utc};
use jsonwebtoken::{Algorithm, EncodingKey, Header};
use kesh_api::auth::jwt::Claims;
use kesh_api::config::Config;
use kesh_api::provider_download::{Downloads, Wait};
use kesh_api::provider_env::{PAYPAL, ProviderApiConfig, ProviderSeries, ProvidersConfig};
use kesh_api::{AppState, build_router};
use kesh_db::test_fixtures::{
    SeededCompany, disable_rounding_to_5_centimes, mark_onboarding_complete,
    seed_accounting_company,
};
use serde_json::{Value, json};
use sqlx::MySqlPool;

use common::secrets_prestataire::{aucun_secret, secret};

const TEST_JWT_SECRET: &[u8] = b"test-secret-32-bytes-minimum-test-secret-padding";
const FIXTURE: &[u8] =
    include_bytes!("../../kesh-import/tests/fixtures/paypal-balance-affecting-fictif.csv");
const COMPTE: &str = "PayPal www";
/// Un second compte PayPal de Kesh, sans série d'identifiants.
const COMPTE_SANS_API: &str = "PayPal hébergement";

// ---------------------------------------------------------------------------
// Serveur PayPal simulé
// ---------------------------------------------------------------------------

/// Une réponse forcée du serveur simulé.
#[derive(Clone, Debug)]
enum Forcee {
    /// Statut, `Retry-After`, corps.
    Statut(u16, Option<&'static str>, String),
    /// La réponse ne vient pas (au-delà du délai du client).
    Lente,
}

/// Une demande de page reçue.
#[derive(Clone, Debug)]
struct Demande {
    start: String,
    end: String,
    page: u32,
    authorization: String,
    enforce_iso: bool,
}

/// Règle persistante : rend une réponse forcée pour une demande, ou rien.
type Regle = Arc<dyn Fn(&Demande) -> Option<Forcee> + Send + Sync>;

#[derive(Clone)]
struct Sim {
    transactions: Arc<Mutex<Vec<Value>>>,
    page_size: usize,
    /// Réponses forcées des pages, consommées une à une avant le cours normal.
    file_pages: Arc<Mutex<VecDeque<Forcee>>>,
    /// Réponses forcées du jeton, consommées une à une.
    file_jeton: Arc<Mutex<VecDeque<Forcee>>>,
    regle: Arc<Mutex<Option<Regle>>>,
    demandes: Arc<Mutex<Vec<Demande>>>,
    jetons: Arc<AtomicUsize>,
    basic_attendu: String,
    jeton: String,
    /// Retient chaque demande de page tant qu'il est fermé (tâche « en cours »).
    verrou: Arc<tokio::sync::Semaphore>,
}

/// Base64 standard (en-tête Basic attendu) — sans dépendance.
fn base64(entree: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for bloc in entree.chunks(3) {
        let b = [
            bloc[0],
            *bloc.get(1).unwrap_or(&0),
            *bloc.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= bloc.len() {
                out.push(T[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

fn forcee(f: Forcee) -> Response {
    match f {
        Forcee::Statut(code, retry, corps) => {
            let mut r = (StatusCode::from_u16(code).unwrap(), corps).into_response();
            if let Some(v) = retry {
                r.headers_mut().insert("retry-after", v.parse().unwrap());
            }
            r
        }
        Forcee::Lente => unreachable!("traitée par l'appelant"),
    }
}

async fn route_jeton(State(sim): State<Sim>, headers: HeaderMap, corps: String) -> Response {
    sim.jetons.fetch_add(1, Ordering::SeqCst);
    let premiere = sim.file_jeton.lock().unwrap().pop_front();
    if let Some(f) = premiere {
        if let Forcee::Lente = f {
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
        return forcee(f);
    }
    let auth = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if auth != sim.basic_attendu || corps != "grant_type=client_credentials" {
        return (
            StatusCode::UNAUTHORIZED,
            r#"{"error":"invalid_client"}"#.to_string(),
        )
            .into_response();
    }
    axum::Json(json!({
        "scope": "https://uri.paypal.com/services/reporting/search/read",
        "access_token": sim.jeton,
        "token_type": "Bearer",
        "expires_in": 32400,
    }))
    .into_response()
}

async fn route_pages(
    State(sim): State<Sim>,
    headers: HeaderMap,
    Query(q): Query<std::collections::HashMap<String, String>>,
) -> Response {
    let _place = sim.verrou.acquire().await.unwrap();
    let demande = Demande {
        start: q.get("start_date").cloned().unwrap_or_default(),
        end: q.get("end_date").cloned().unwrap_or_default(),
        page: q.get("page").and_then(|p| p.parse().ok()).unwrap_or(1),
        authorization: headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string(),
        enforce_iso: headers
            .get("paypal-enforce-iso8601-format")
            .is_some_and(|v| v == "true"),
    };
    sim.demandes.lock().unwrap().push(demande.clone());
    assert_eq!(q.get("fields").map(String::as_str), Some("all"));
    assert_eq!(q.get("page_size").map(String::as_str), Some("500"));
    assert_eq!(
        q.get("balance_affecting_records_only").map(String::as_str),
        Some("Y")
    );
    let regle = sim.regle.lock().unwrap().clone();
    let force = sim
        .file_pages
        .lock()
        .unwrap()
        .pop_front()
        .or_else(|| regle.and_then(|r| r(&demande)));
    if let Some(f) = force {
        if let Forcee::Lente = f {
            tokio::time::sleep(Duration::from_secs(5)).await;
            return StatusCode::OK.into_response();
        }
        return forcee(f);
    }
    if demande.authorization != format!("Bearer {}", sim.jeton) {
        return (
            StatusCode::UNAUTHORIZED,
            r#"{"name":"AUTHENTICATION_FAILURE"}"#,
        )
            .into_response();
    }
    let debut = DateTime::parse_from_rfc3339(&demande.start).unwrap();
    let fin = DateTime::parse_from_rfc3339(&demande.end).unwrap();
    let dans: Vec<Value> = sim
        .transactions
        .lock()
        .unwrap()
        .iter()
        .filter(|t| {
            let d = DateTime::parse_from_rfc3339(
                t["transaction_info"]["transaction_initiation_date"]
                    .as_str()
                    .unwrap(),
            )
            .unwrap();
            d >= debut && d <= fin
        })
        .cloned()
        .collect();
    let total_pages = dans.len().div_ceil(sim.page_size);
    let page: Vec<Value> = dans
        .into_iter()
        .skip((demande.page as usize - 1) * sim.page_size)
        .take(sim.page_size)
        .collect();
    let mut corps = json!({
        "start_date": demande.start,
        "end_date": demande.end,
        "page": demande.page,
        "total_pages": total_pages,
        // Le plus ancien rafraîchissement est celui des pages 2.
        "last_refreshed_datetime": if demande.page == 2 { "2020-06-30T07:00:00+00:00" } else { "2020-06-30T08:00:00+00:00" },
    });
    if !page.is_empty() {
        corps["transaction_details"] = Value::Array(page);
    }
    axum::Json(corps).into_response()
}

/// Lance le serveur simulé ; rend son URL et son état.
async fn simuler(transactions: Vec<Value>, page_size: usize) -> (String, Sim) {
    let id = secret("client");
    let secret_ = secret("secret");
    let sim = Sim {
        transactions: Arc::new(Mutex::new(transactions)),
        page_size,
        file_pages: Arc::default(),
        file_jeton: Arc::default(),
        regle: Arc::default(),
        demandes: Arc::default(),
        jetons: Arc::default(),
        basic_attendu: format!("Basic {}", base64(format!("{id}:{secret_}").as_bytes())),
        jeton: secret("jeton"),
        verrou: Arc::new(tokio::sync::Semaphore::new(1000)),
    };
    let app = Router::new()
        .route("/v1/oauth2/token", post(route_jeton))
        .route("/v1/reporting/transactions", get(route_pages))
        .with_state(sim.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), sim)
}

/// Les transactions de la fixture CSV de 38-11a, dans la forme de l'API : les
/// lignes qui affectent le solde (pas de `Memo`), datées à 10:00 UTC (11:00 à
/// Zurich, même jour), statut en lettre, code d'événement par la table.
fn transactions_de_la_fixture() -> Vec<Value> {
    let csv = kesh_import::paypal::parse_paypal_csv(FIXTURE).unwrap();
    csv.rows
        .iter()
        .filter(|r| r.balance_impact != kesh_import::paypal::BalanceImpact::Memo)
        .map(|r| {
            let code = kesh_import::paypal_api::PAYPAL_EVENT_TYPES
                .iter()
                .find(|(_, l)| *l == r.kind)
                .map(|(c, _)| *c)
                .unwrap_or_else(|| panic!("type sans code : {}", r.kind));
            let statut = match r.status.as_str() {
                "Completed" => "S",
                "Pending" => "P",
                "Denied" => "D",
                autre => panic!("statut {autre}"),
            };
            let mut info = json!({
                "transaction_id": r.transaction_id,
                "transaction_event_code": code,
                "transaction_initiation_date": format!("{}T10:00:00+00:00", r.date),
                "transaction_amount": { "currency_code": r.currency, "value": r.gross.to_string() },
                "transaction_status": statut,
            });
            if !r.fee.is_zero() {
                info["fee_amount"] =
                    json!({ "currency_code": r.currency, "value": r.fee.to_string() });
            }
            if let Some(reference) = &r.reference_txn_id {
                info["paypal_reference_id"] = json!(reference);
            }
            let email = if r.gross.is_sign_negative() {
                r.to_email.clone()
            } else {
                r.from_email.clone()
            };
            json!({
                "transaction_info": info,
                "payer_info": {
                    "email_address": email,
                    "payer_name": { "alternate_full_name": r.name },
                },
            })
        })
        .collect()
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

struct TestApp {
    base_url: String,
    client: reqwest::Client,
    attente: Arc<Notee>,
}

fn config(sim_url: &str, parallel: usize) -> Config {
    let serie = ProviderSeries::for_tests(
        1,
        COMPTE,
        &[
            ("CLIENT_ID", secret("client").as_str()),
            ("SECRET", secret("secret").as_str()),
        ],
    );
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
            PAYPAL,
            sim_url,
            parallel,
            vec![serie],
        )),
    )
}

/// L'instant des tests : la fixture date de 2020.
fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2020, 6, 30, 12, 0, 0).unwrap()
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

struct Ctx {
    pool: MySqlPool,
    app: TestApp,
    jwt: String,
    seeded: SeededCompany,
}

async fn compte(pool: &MySqlPool, c: i64, numero: &str, devise: Option<&str>, type_: &str) -> i64 {
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

/// Une société multi-devises et ses soldes PayPal : EUR, USD et, si
/// `avec_chf`, CHF ; un compte « sans API » en EUR.
async fn monter(pool: &MySqlPool, sim_url: &str, parallel: usize, avec_chf: bool) -> Ctx {
    let seeded = seed_accounting_company(pool).await.expect("seed");
    mark_onboarding_complete(pool).await.expect("onboarding");
    disable_rounding_to_5_centimes(pool, seeded.company_id)
        .await
        .expect("arrondi");
    let c = seeded.company_id;
    sqlx::query("UPDATE companies SET multi_currency_enabled = TRUE WHERE id = ?")
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
    let frais = compte(pool, c, "6941", Some("EUR"), "Expense").await;
    let eur = compte(pool, c, "1031", Some("EUR"), "Asset").await;
    let usd = compte(pool, c, "1032", Some("USD"), "Asset").await;
    let eur_bis = compte(pool, c, "1033", Some("EUR"), "Asset").await;
    ctx.solde(eur, Some(frais), COMPTE).await;
    ctx.solde(usd, None, COMPTE).await;
    if avec_chf {
        let chf = compte(pool, c, "1030", None, "Asset").await;
        ctx.solde(chf, None, COMPTE).await;
    }
    ctx.solde(eur_bis, None, COMPTE_SANS_API).await;
    ctx
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

    async fn solde(&self, ledger: i64, fee: Option<i64>, libelle: &str) {
        let (s, b) = self
            .post(
                "/api/v1/bank-accounts",
                json!({
                    "bankName": "PayPal",
                    "journalAccountId": ledger,
                    "paypal": { "account": libelle, "feeAccountId": fee },
                }),
            )
            .await;
        assert_eq!(s, 201, "solde PayPal : {b}");
    }

    /// Lance un téléchargement ; rend le statut et le corps du `POST`.
    async fn lancer(&self, compte: &str, du: &str, au: &str) -> (u16, Value) {
        self.post(
            "/api/v1/paypal-downloads",
            json!({ "account": compte, "from": du, "to": au }),
        )
        .await
    }

    /// Suit une tâche jusqu'à sa fin ; rend sa dernière vue.
    async fn suivre(&self, job: &Value) -> Value {
        let chemin = format!("/api/v1/paypal-downloads/{}", job["jobId"]);
        for _ in 0..1000 {
            let (s, v) = self
                .req_avec(&self.jwt, reqwest::Method::GET, &chemin, None)
                .await;
            assert_eq!(s, 200, "{v}");
            if v["state"] != "running" {
                return v;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("tâche restée en cours");
    }

    /// Lance et suit jusqu'à la fin.
    async fn telecharger(&self, du: &str, au: &str) -> Value {
        let (s, b) = self.lancer(COMPTE, du, au).await;
        assert_eq!(s, 202, "{b}");
        assert!(b["jobId"].is_u64(), "{b}");
        self.suivre(&b).await
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
                "SELECT COUNT(*) FROM audit_log WHERE action IN ('bank_import.created', 'paypal_import.created', 'paypal_download.created')",
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

fn par_devise(resultat: &Value, devise: &str) -> Value {
    resultat["currencies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["currency"] == devise)
        .cloned()
        .unwrap_or_else(|| panic!("devise {devise} absente : {resultat}"))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// AC-4, AC-5, AC-7, AC-8 — quatre mois : cinq fenêtres de 30 jours de Zurich,
/// deux pages dans une fenêtre, un doublon identique entre deux pages, un
/// statut `V` ; ventilation de 38-11b par devise ; traces sous l'acteur de la
/// demande.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn quatre_mois_ventiles_par_devise(pool: MySqlPool) {
    let mut txs = transactions_de_la_fixture();
    // Doublon identique de la première vente, glissé juste après elle (page
    // suivante : taille de page 2).
    txs.insert(2, txs[0].clone());
    // Une transaction annulée (`V`) : écartée comme non `Completed`.
    let mut annulee = txs[0].clone();
    annulee["transaction_info"]["transaction_id"] = json!("0TEST0000000000VV");
    annulee["transaction_info"]["transaction_status"] = json!("V");
    annulee["transaction_info"]["transaction_initiation_date"] = json!("2020-04-25T10:00:00+00:00");
    txs.push(annulee);
    let (url, sim) = simuler(txs, 2).await;
    let ctx = monter(&pool, &url, 1, true).await;

    let v = ctx.telecharger("2020-01-01", "2020-04-30").await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(v["steps"], json!({ "done": 5, "total": 5 }));
    assert_eq!(v["error"], Value::Null);
    let r = &v["result"];
    assert_eq!(r["account"], COMPTE);
    for (devise, insere) in [("EUR", 9), ("USD", 3), ("CHF", 3)] {
        let d = par_devise(r, devise);
        assert_eq!(d["inserted"], insere, "{devise} : {r}");
        assert_eq!(d["duplicates"], 0, "{devise}");
    }
    assert_eq!(r["skipped"]["neutralPairs"], 1);
    assert_eq!(r["skipped"]["memo"], 0);
    let ecartees: Vec<&str> = r["skipped"]["notCompleted"]
        .as_array()
        .unwrap()
        .iter()
        .map(|l| l["status"].as_str().unwrap())
        .collect();
    assert_eq!(ecartees, ["Pending", "Denied", "Reversed"]);
    assert_eq!(r["lastRefreshed"], "2020-06-30T07:00:00+00:00");

    // Fenêtres demandées : premières pages, bornes de Zurich en UTC, à la
    // seconde ; l'en-tête ISO 8601 et le jeton envoyés.
    let demandes = sim.demandes.lock().unwrap().clone();
    let premieres: Vec<(String, String)> = demandes
        .iter()
        .filter(|d| d.page == 1)
        .map(|d| (d.start.clone(), d.end.clone()))
        .collect();
    assert_eq!(
        premieres,
        [
            // Recouvrement d'une seconde : la fin d'une fenêtre est le début
            // de la suivante (revue P1, F3).
            ("2019-12-31T23:00:00Z", "2020-01-30T23:00:00Z"),
            ("2020-01-30T23:00:00Z", "2020-02-29T23:00:00Z"),
            ("2020-02-29T23:00:00Z", "2020-03-30T22:00:00Z"),
            ("2020-03-30T22:00:00Z", "2020-04-29T22:00:00Z"),
            ("2020-04-29T22:00:00Z", "2020-04-30T22:00:00Z"),
        ]
        .map(|(a, b)| (a.to_string(), b.to_string()))
    );
    assert!(demandes.iter().any(|d| d.page == 2), "deux pages lues");
    assert!(demandes.iter().all(|d| d.enforce_iso));
    assert_eq!(sim.jetons.load(Ordering::SeqCst), 1, "un jeton par tâche");

    // Écrit : trois imports, quinze transactions, une trace de
    // téléchargement, trois `bank_import.created`, aucune trace d'import.
    assert_eq!(ctx.ecrit().await, (3, 15, 4));
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'paypal_import.created'")
            .await,
        0
    );
    let (acteur, cle, details): (String, Option<i64>, Value) = sqlx::query_as(
        "SELECT actor_type, actor_api_key_id, details_json FROM audit_log WHERE action = 'paypal_download.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((acteur.as_str(), cle), ("user", None));
    assert_eq!(details["provider_account"], COMPTE);
    assert_eq!(
        details["period"],
        json!({ "from": "2020-01-01", "to": "2020-04-30" })
    );
    assert_eq!(details["windows"], 5);
    assert_eq!(details["currencies"].as_array().unwrap().len(), 3);
    let noms: Vec<String> = sqlx::query_scalar("SELECT DISTINCT filename FROM bank_imports")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(noms, ["PayPal API 2020-01-01…2020-04-30"]);
    // Frais positif du remboursement, en `:FEE`.
    assert_eq!(
        ctx.count(
            "SELECT COUNT(*) FROM bank_transactions WHERE transaction_id = '0TEST0000000000BL:FEE' AND amount = 0.96"
        )
        .await,
        1
    );
}

/// AC-7 — un CSV puis le téléchargement des mêmes transactions, puis un second
/// téléchargement qui chevauche : rien de doublé, aucun refus d'empreinte.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn csv_puis_telechargement_rien_de_double(pool: MySqlPool) {
    let (url, _sim) = simuler(transactions_de_la_fixture(), 500).await;
    let ctx = monter(&pool, &url, 1, true).await;
    let form = reqwest::multipart::Form::new()
        .part(
            "file",
            reqwest::multipart::Part::bytes(FIXTURE.to_vec()).file_name("Download.CSV"),
        )
        .text("account", COMPTE);
    let resp = ctx
        .app
        .client
        .post(format!("{}/api/v1/paypal-imports", ctx.app.base_url))
        .bearer_auth(&ctx.jwt)
        .multipart(form)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let avant = ctx.transactions().await;
    assert_eq!(avant.len(), 15);

    for (du, au) in [("2020-01-01", "2020-04-30"), ("2020-02-01", "2020-06-30")] {
        let v = ctx.telecharger(du, au).await;
        assert_eq!(v["state"], "done", "{v}");
        for devise in ["EUR", "USD", "CHF"] {
            let d = par_devise(&v["result"], devise);
            assert_eq!(d["inserted"], 0, "{devise}");
            assert_eq!(d["importId"], Value::Null, "{devise}");
        }
    }
    assert_eq!(
        par_devise(
            &ctx.telecharger("2020-01-01", "2020-04-30").await["result"],
            "EUR"
        )["duplicates"],
        9
    );
    assert_eq!(ctx.transactions().await, avant, "rien de doublé");
    assert_eq!(ctx.count("SELECT COUNT(*) FROM bank_imports").await, 3);
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'paypal_download.created'")
            .await,
        3
    );
}

/// AC-6 — 429 sans `Retry-After` (5 s), puis avec un `Retry-After` de 3600,
/// borné à 120 s, puis la même requête réussit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn limitation_de_debit_puis_reprise(pool: MySqlPool) {
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    {
        let mut f = sim.file_pages.lock().unwrap();
        f.push_back(Forcee::Statut(
            429,
            None,
            r#"{"name":"RATE_LIMIT_REACHED"}"#.into(),
        ));
        f.push_back(Forcee::Statut(
            429,
            Some("3600"),
            r#"{"name":"RATE_LIMIT_REACHED"}"#.into(),
        ));
    }
    let ctx = monter(&pool, &url, 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["state"], "done", "{v}");
    let attentes: Vec<u64> = ctx
        .app
        .attente
        .0
        .lock()
        .unwrap()
        .iter()
        .map(|d| d.as_secs_f64().round() as u64)
        .collect();
    assert_eq!(attentes, [5, 120]);
    let demandes = sim.demandes.lock().unwrap().clone();
    assert_eq!(demandes.len(), 3, "la même requête reprise");
    assert!(
        demandes
            .iter()
            .all(|d| d.start == demandes[0].start && d.page == 1)
    );
}

/// AC-6 — parallélisme 3 : les mêmes lignes qu'à 1.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn parallelisme_trois_memes_lignes_qu_a_un(pool: MySqlPool) {
    let (url, _sim) = simuler(transactions_de_la_fixture(), 1).await;
    let ctx = monter(&pool, &url, 1, true).await;
    assert_eq!(
        ctx.telecharger("2020-01-01", "2020-04-30").await["state"],
        "done"
    );
    let a_un = ctx.transactions().await;
    let ordre_un: Vec<String> =
        sqlx::query_scalar("SELECT transaction_id FROM bank_transactions ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    sqlx::query("DELETE FROM bank_transactions")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM bank_imports")
        .execute(&pool)
        .await
        .unwrap();

    let app3 = spawn_app(pool.clone(), &url, 3).await;
    let ctx3 = Ctx {
        pool: pool.clone(),
        app: app3,
        jwt: ctx.jwt.clone(),
        seeded: ctx.seeded.clone(),
    };
    assert_eq!(
        ctx3.telecharger("2020-01-01", "2020-04-30").await["state"],
        "done"
    );
    assert_eq!(ctx3.transactions().await, a_un);
    let ordre_trois: Vec<String> =
        sqlx::query_scalar("SELECT transaction_id FROM bank_transactions ORDER BY id")
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(ordre_trois, ordre_un, "rangées par fenêtre puis par page");
}

/// AC-6, AC-7 — la fenêtre 3 ne répond qu'en 500 : six essais, échec
/// `PROVIDER_UNAVAILABLE` nommant la fenêtre et la page, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn echec_a_la_fenetre_trois_rien_d_ecrit(pool: MySqlPool) {
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    *sim.regle.lock().unwrap() = Some(Arc::new(|d: &Demande| {
        (d.start == "2020-02-29T23:00:00Z")
            .then(|| Forcee::Statut(503, None, r#"{"name":"INTERNAL_SERVICE_ERROR"}"#.into()))
    }));
    let ctx = monter(&pool, &url, 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-04-30").await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_UNAVAILABLE");
    assert_eq!(v["error"]["details"]["provider"], "paypal");
    assert_eq!(v["error"]["details"]["window"], "2020-03-01…2020-03-30");
    assert_eq!(v["error"]["details"]["page"], 1);
    assert_eq!(v["result"], Value::Null);
    assert_eq!(v["steps"]["done"], 2);
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
    let essais = sim
        .demandes
        .lock()
        .unwrap()
        .iter()
        .filter(|d| d.start == "2020-02-29T23:00:00Z")
        .count();
    assert_eq!(essais, 6);
}

/// AC-4, AC-6 — `RESULTSET_TOO_LARGE` coupe la fenêtre en deux (une étape de
/// plus), au plancher d'un jour échec ; 403 ; 401 renouvelé une fois ; une
/// réponse qui ne vient pas ; un identifiant répété avec d'autres montants.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn fenetre_coupee_et_refus_du_prestataire(pool: MySqlPool) {
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    let trop = || {
        Forcee::Statut(
            400,
            None,
            r#"{"name":"RESULTSET_TOO_LARGE","message":"x"}"#.into(),
        )
    };
    // La première fenêtre entière est trop chargée ; ses moitiés passent.
    *sim.regle.lock().unwrap() = Some(Arc::new(move |d: &Demande| {
        (d.start == "2019-12-31T23:00:00Z" && d.end == "2020-01-30T23:00:00Z").then(trop)
    }));
    let ctx = monter(&pool, &url, 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-02-15").await;
    assert_eq!(v["state"], "done", "{v}");
    assert_eq!(v["steps"], json!({ "done": 3, "total": 3 }));
    assert_eq!(par_devise(&v["result"], "EUR")["inserted"], 5, "{v}");

    // Plancher d'un jour : échec nommant la fenêtre.
    *sim.regle.lock().unwrap() = Some(Arc::new(move |_d: &Demande| Some(trop())));
    let v = ctx.telecharger("2020-03-01", "2020-03-01").await;
    assert_eq!(v["error"]["code"], "PROVIDER_API_ERROR", "{v}");
    assert_eq!(v["error"]["details"]["window"], "2020-03-01…2020-03-01");

    // 403 : l'application n'a pas la fonction.
    *sim.regle.lock().unwrap() = Some(Arc::new(|_d: &Demande| {
        Some(Forcee::Statut(
            403,
            None,
            r#"{"name":"NOT_AUTHORIZED"}"#.into(),
        ))
    }));
    let v = ctx.telecharger("2020-03-01", "2020-03-02").await;
    assert_eq!(v["error"]["code"], "PROVIDER_PERMISSION_DENIED", "{v}");

    // 401 sur les pages : renouvelé une fois, puis échec d'authentification.
    let jetons_avant = sim.jetons.load(Ordering::SeqCst);
    *sim.regle.lock().unwrap() = Some(Arc::new(|_d: &Demande| {
        Some(Forcee::Statut(
            401,
            None,
            r#"{"name":"AUTHENTICATION_FAILURE"}"#.into(),
        ))
    }));
    let v = ctx.telecharger("2020-03-01", "2020-03-02").await;
    assert_eq!(v["error"]["code"], "PROVIDER_AUTH_FAILED", "{v}");
    assert_eq!(
        sim.jetons.load(Ordering::SeqCst) - jetons_avant,
        2,
        "un jeton, un renouvellement"
    );

    // Le jeton refusé d'emblée.
    *sim.regle.lock().unwrap() = None;
    sim.file_jeton.lock().unwrap().push_back(Forcee::Statut(
        401,
        None,
        r#"{"error":"invalid_client"}"#.into(),
    ));
    let v = ctx.telecharger("2020-03-01", "2020-03-02").await;
    assert_eq!(v["error"]["code"], "PROVIDER_AUTH_FAILED", "{v}");

    // La réponse ne vient pas : six essais, puis indisponible.
    *sim.regle.lock().unwrap() = Some(Arc::new(|_d: &Demande| Some(Forcee::Lente)));
    let v = ctx.telecharger("2020-03-01", "2020-03-02").await;
    assert_eq!(v["error"]["code"], "PROVIDER_UNAVAILABLE", "{v}");

    // Un identifiant répété avec d'autres montants : refus, l'identifiant nommé.
    *sim.regle.lock().unwrap() = None;
    {
        let mut txs = sim.transactions.lock().unwrap();
        let mut autre = txs[0].clone();
        autre["transaction_info"]["transaction_amount"]["value"] = json!("99.00");
        txs.push(autre);
    }
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["error"]["code"], "PROVIDER_RESPONSE_INVALID", "{v}");
    assert_eq!(v["error"]["details"]["transactionId"], "0TEST00000000001A");

    // Rien n'a été écrit par les échecs : seul le premier téléchargement.
    assert_eq!(
        ctx.count("SELECT COUNT(*) FROM audit_log WHERE action = 'paypal_download.created'")
            .await,
        1
    );
}

/// Revue P1, F1 — une `to` future est acceptée et ramenée au jour du jour :
/// aucune fenêtre inversée n'est envoyée, aucune ne commence après l'instant,
/// leur nombre s'arrête à aujourd'hui ; le nom de l'import et la trace disent
/// la période lue. Au parallélisme 1 puis 3.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_fin_future_est_ramenee_au_jour_du_jour(pool: MySqlPool) {
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    let ctx = monter(&pool, &url, 1, true).await;
    let v = ctx.telecharger("2020-03-01", "9999-12-31").await;
    assert_eq!(v["state"], "done", "{v}");
    // 2020-03-01 → 2020-06-30 : cinq fenêtres de 30 jours au plus, la
    // dernière de deux jours, finie à l'instant (les ventes de mars s'écrivent).
    assert_eq!(v["steps"], json!({ "done": 5, "total": 5 }));
    let demandes = sim.demandes.lock().unwrap().clone();
    assert_eq!(demandes.len(), 5, "{demandes:?}");
    for d in &demandes {
        let (debut, fin) = (
            DateTime::parse_from_rfc3339(&d.start).unwrap(),
            DateTime::parse_from_rfc3339(&d.end).unwrap(),
        );
        assert!(debut <= fin, "fenêtre inversée : {d:?}");
        assert!(
            fin.with_timezone(&Utc) <= maintenant(),
            "après l'instant : {d:?}"
        );
    }
    assert_eq!(demandes.last().unwrap().end, "2020-06-30T12:00:00Z");
    let noms: Vec<String> = sqlx::query_scalar("SELECT DISTINCT filename FROM bank_imports")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(noms, ["PayPal API 2020-03-01…2020-06-30"]);
    let (details,): (Value,) = sqlx::query_as(
        "SELECT details_json FROM audit_log WHERE action = 'paypal_download.created'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(
        details["period"],
        json!({ "from": "2020-03-01", "to": "2020-06-30" })
    );

    // Parallélisme 3, une période de trois ans jusqu'en 9999 : autant de
    // fenêtres que de tranches de 30 jours jusqu'au jour du jour, aucune
    // inversée.
    sim.demandes.lock().unwrap().clear();
    let app3 = spawn_app(pool.clone(), &url, 3).await;
    let ctx3 = Ctx {
        pool: pool.clone(),
        app: app3,
        jwt: ctx.jwt.clone(),
        seeded: ctx.seeded.clone(),
    };
    let v = ctx3.telecharger("2017-07-01", "9999-12-31").await;
    assert_eq!(v["state"], "done", "{v}");
    let jours = (chrono::NaiveDate::from_ymd_opt(2020, 6, 30).unwrap()
        - chrono::NaiveDate::from_ymd_opt(2017, 7, 1).unwrap())
    .num_days()
        + 1;
    assert_eq!(v["steps"]["total"], (jours + 29) / 30);
    let demandes = sim.demandes.lock().unwrap().clone();
    assert_eq!(demandes.len() as i64, (jours + 29) / 30);
    assert!(demandes.iter().all(|d| {
        DateTime::parse_from_rfc3339(&d.start).unwrap()
            <= DateTime::parse_from_rfc3339(&d.end).unwrap()
    }));
}

/// Revue P1, F2 — une erreur de transport définitive (ici, TLS parlé à un
/// serveur en clair, comme un certificat refusé) n'est pas reprise : échec
/// `PROVIDER_API_ERROR` au premier essai, sans pause.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn une_erreur_de_transport_definitive_n_est_pas_reprise(pool: MySqlPool) {
    // Un serveur qui répond en clair à toute connexion, compte les connexions.
    let clair = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = clair.local_addr().unwrap();
    let connexions = Arc::new(AtomicUsize::new(0));
    let compte = connexions.clone();
    tokio::spawn(async move {
        while let Ok((mut flux, _)) = clair.accept().await {
            compte.fetch_add(1, Ordering::SeqCst);
            use tokio::io::AsyncWriteExt;
            let _ = flux
                .write_all(b"HTTP/1.1 400 Bad Request\r\ncontent-length: 0\r\n\r\n")
                .await;
        }
    });
    let ctx = monter(&pool, &format!("https://{adresse}"), 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_API_ERROR", "{v}");
    assert!(ctx.app.attente.0.lock().unwrap().is_empty(), "aucune pause");
    assert_eq!(connexions.load(Ordering::SeqCst), 1, "un seul essai");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// Revue P2, G1 — un serveur qui lit la requête puis ferme sans répondre est
/// une panne passagère : six essais, cinq pauses, `PROVIDER_UNAVAILABLE`.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn connexion_fermee_apres_la_requete_est_reprise(pool: MySqlPool) {
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = ecoute.local_addr().unwrap();
    let connexions = Arc::new(AtomicUsize::new(0));
    let compte = connexions.clone();
    tokio::spawn(async move {
        while let Ok((mut flux, _)) = ecoute.accept().await {
            compte.fetch_add(1, Ordering::SeqCst);
            use tokio::io::AsyncReadExt;
            let mut tampon = [0u8; 4096];
            let _ = flux.read(&mut tampon).await;
            drop(flux);
        }
    });
    let ctx = monter(&pool, &format!("http://{adresse}"), 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_UNAVAILABLE", "{v}");
    assert_eq!(ctx.app.attente.0.lock().unwrap().len(), 5, "cinq pauses");
    assert_eq!(connexions.load(Ordering::SeqCst), 6, "six essais");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// Revue P3, P3-3 — le jeton est servi, seul le `GET` de téléchargement est
/// coupé après la requête : six essais du `GET`, cinq pauses.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn get_coupe_apres_la_requete_est_repris(pool: MySqlPool) {
    let ecoute = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let adresse = ecoute.local_addr().unwrap();
    let gets = Arc::new(AtomicUsize::new(0));
    let compte = gets.clone();
    tokio::spawn(async move {
        while let Ok((mut flux, _)) = ecoute.accept().await {
            let compte = compte.clone();
            tokio::spawn(async move {
                use tokio::io::{AsyncReadExt, AsyncWriteExt};
                let mut tampon = [0u8; 4096];
                let n = flux.read(&mut tampon).await.unwrap_or(0);
                if tampon[..n].starts_with(b"POST") {
                    let corps = r#"{"access_token":"t","expires_in":3600}"#;
                    let _ = flux
                        .write_all(
                            format!(
                                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{corps}",
                                corps.len()
                            )
                            .as_bytes(),
                        )
                        .await;
                } else {
                    compte.fetch_add(1, Ordering::SeqCst);
                }
                drop(flux);
            });
        }
    });
    let ctx = monter(&pool, &format!("http://{adresse}"), 1, true).await;
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_UNAVAILABLE", "{v}");
    assert_eq!(gets.load(Ordering::SeqCst), 6, "six essais du GET");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-7, AC-8 — une devise sans compte bancaire : `failed`
/// `PROVIDER_CURRENCY_ACCOUNT_MISSING`, rien d'écrit.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn devise_sans_compte_rien_d_ecrit(pool: MySqlPool) {
    let (url, _sim) = simuler(transactions_de_la_fixture(), 500).await;
    let ctx = monter(&pool, &url, 1, false).await;
    let v = ctx.telecharger("2020-01-01", "2020-04-30").await;
    assert_eq!(v["state"], "failed", "{v}");
    assert_eq!(v["error"]["code"], "PROVIDER_CURRENCY_ACCOUNT_MISSING");
    assert_eq!(v["error"]["details"]["currencies"], json!(["CHF"]));
    assert_eq!(v["error"]["details"]["provider"], "paypal");
    assert_eq!(ctx.ecrit().await, (0, 0, 0));
}

/// AC-8 — l'ordre des refus du `POST`, la tâche unique par compte (409 avec
/// `jobId`), les clés d'API refusées sur les deux routes, la tâche d'une autre
/// société ou inconnue en 404.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn refus_tache_unique_et_cles_d_api(pool: MySqlPool) {
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    let ctx = monter(&pool, &url, 1, true).await;

    let (_, ecriture) = common::create_key_via_http(
        &ctx.app.client,
        &ctx.app.base_url,
        &ctx.jwt,
        "rw",
        "read-write",
    )
    .await;
    let corps = json!({ "account": COMPTE, "from": "2020-01-01", "to": "2020-01-30" });
    let (s, b) = ctx
        .req_avec(
            &ecriture,
            reqwest::Method::POST,
            "/api/v1/paypal-downloads",
            Some(corps.clone()),
        )
        .await;
    assert_eq!(
        (s, b["error"]["code"].as_str()),
        (403, Some("API_KEY_OPERATION_FORBIDDEN"))
    );
    let (s, _) = ctx
        .req_avec(
            &ecriture,
            reqwest::Method::GET,
            "/api/v1/paypal-downloads/1",
            None,
        )
        .await;
    assert_eq!(s, 403);
    let consultation = forge_jwt(
        ctx.seeded.admin_user_id,
        "Consultation",
        ctx.seeded.company_id,
    );
    let (s, _) = ctx
        .req_avec(
            &consultation,
            reqwest::Method::POST,
            "/api/v1/paypal-downloads",
            Some(corps),
        )
        .await;
    assert_eq!(s, 403);

    let code = |b: &Value| b["error"]["code"].as_str().unwrap_or("").to_string();
    let (s, b) = ctx
        .post("/api/v1/paypal-downloads", json!({ "account": COMPTE }))
        .await;
    assert_eq!((s, code(&b)), (400, "VALIDATION_ERROR".into()));
    let (s, b) = ctx.lancer("Inconnu", "2020-01-01", "2020-01-30").await;
    assert_eq!((s, code(&b)), (404, "PROVIDER_ACCOUNT_NOT_FOUND".into()));
    let (s, b) = ctx
        .lancer(COMPTE_SANS_API, "2020-01-01", "2020-01-30")
        .await;
    assert_eq!((s, code(&b)), (422, "PROVIDER_API_NOT_CONFIGURED".into()));
    for (du, au, motif) in [
        ("2020-02-01", "2020-01-01", "from_after_to"),
        ("2020-07-01", "2020-07-02", "future"),
        ("2017-06-29", "2017-07-30", "too_old"),
    ] {
        let (s, b) = ctx.lancer(COMPTE, du, au).await;
        assert_eq!(
            (s, code(&b)),
            (400, "PROVIDER_PERIOD_INVALID".into()),
            "{b}"
        );
        assert_eq!(b["error"]["details"]["reason"], motif);
        assert_eq!(b["error"]["details"]["provider"], "paypal");
    }
    assert_eq!(sim.jetons.load(Ordering::SeqCst), 0, "rien lancé");

    // Le serveur retient les pages : la tâche reste en cours.
    let place = sim.verrou.acquire_many(1000).await.unwrap();
    let (s, premier) = ctx.lancer(COMPTE, "2020-01-01", "2020-01-30").await;
    assert_eq!(s, 202);
    let (s, b) = ctx.lancer("paypal WWW", "2020-02-01", "2020-02-02").await;
    assert_eq!((s, code(&b)), (409, "PROVIDER_DOWNLOAD_RUNNING".into()));
    assert_eq!(b["error"]["details"]["jobId"], premier["jobId"]);
    let (s, v) = ctx
        .req_avec(
            &ctx.jwt,
            reqwest::Method::GET,
            &format!("/api/v1/paypal-downloads/{}", premier["jobId"]),
            None,
        )
        .await;
    assert_eq!((s, v["state"].as_str()), (200, Some("running")));
    drop(place);
    assert_eq!(ctx.suivre(&premier).await["state"], "done");
    // Finie, elle ne bloque plus la suivante.
    let (s, _) = ctx.lancer(COMPTE, "2020-02-01", "2020-02-02").await;
    assert_eq!(s, 202);

    let (s, _) = ctx
        .req_avec(
            &ctx.jwt,
            reqwest::Method::GET,
            "/api/v1/paypal-downloads/999999",
            None,
        )
        .await;
    assert_eq!(s, 404);
    let autre = forge_jwt(
        ctx.seeded.admin_user_id,
        "Admin",
        ctx.seeded.company_id + 1000,
    );
    let (s, _) = ctx
        .req_avec(
            &autre,
            reqwest::Method::GET,
            &format!("/api/v1/paypal-downloads/{}", premier["jobId"]),
            None,
        )
        .await;
    assert!(s == 404 || s == 401 || s == 403, "autre société : {s}");
}

/// AC-9 (socle S9) — un échec d'authentification et un échec de page : aucun
/// identifiant, secret ni jeton dans les réponses ni dans les journaux captés.
#[sqlx::test(migrations = "../kesh-db/test-schema")]
async fn aucun_secret_dans_les_reponses_ni_les_journaux(pool: MySqlPool) {
    let capture = common::capture_trace::CaptureTrace::installer();
    let (url, sim) = simuler(transactions_de_la_fixture(), 500).await;
    let ctx = monter(&pool, &url, 1, true).await;
    let mut corps: Vec<String> = Vec::new();

    sim.file_jeton.lock().unwrap().push_back(Forcee::Statut(
        401,
        None,
        r#"{"error":"invalid_client"}"#.into(),
    ));
    let (s, b) = ctx.lancer(COMPTE, "2020-01-01", "2020-01-30").await;
    corps.push(b.to_string());
    assert_eq!(s, 202);
    let v = ctx.suivre(&b).await;
    assert_eq!(v["error"]["code"], "PROVIDER_AUTH_FAILED");
    corps.push(v.to_string());

    *sim.regle.lock().unwrap() = Some(Arc::new(|_d: &Demande| {
        Some(Forcee::Statut(
            403,
            None,
            r#"{"name":"NOT_AUTHORIZED"}"#.into(),
        ))
    }));
    let v = ctx.telecharger("2020-01-01", "2020-01-30").await;
    assert_eq!(v["error"]["code"], "PROVIDER_PERMISSION_DENIED");
    corps.push(v.to_string());

    // Témoin : la capture voit bien les évènements de la tâche.
    capture.exiger("API : droits insuffisants");
    let evenements = capture.evenements();
    aucun_secret(corps.iter().map(String::as_str));
    aucun_secret(evenements.iter().map(String::as_str));
    // L'hôte de l'API n'est jamais renvoyé.
    assert!(corps.iter().all(|c| !c.contains(&url)), "{corps:?}");
}
