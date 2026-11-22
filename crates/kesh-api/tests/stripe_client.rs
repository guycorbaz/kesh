//! Story 40-8a (Epic 40, phase 2) — le client de l'API Stripe, contre un
//! **serveur Stripe simulé** lancé dans le processus (AC-5, socle S5 : un seul
//! schéma de test, aucun appel au vrai Stripe ni à `api.stripe.com`).
//!
//! Le serveur simulé (`axum` sur `127.0.0.1:0`) sert les réponses **fictives**
//! de `tests/fixtures/stripe-api/` — versements sur deux pages (curseur), un
//! versement manuel, les transactions de trois versements, construites depuis
//! la fixture CSV de 40-7a — et, sur demande, des erreurs (401, 400, 403, 429,
//! 5xx, corps illisible). La clé porte le marqueur du socle S9.
//!
//! Le serveur note chaque requête : le test exige que toutes soient des `GET`,
//! que `Authorization` soit exactement `Bearer <clé>` et que `Stripe-Version`
//! soit l'épinglée. L'attente des pauses est notée et ne dort pas. Aucune base
//! de données.

mod common;

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::extract::{Query, State};
use axum::http::{HeaderMap, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use kesh_api::config::ConfigError;
use kesh_api::errors::AppError;
use kesh_api::provider_download::{ProviderGate, Wait, Window, error_body};
use kesh_api::provider_env::{ProviderApiConfig, ProviderSeries, read};
use kesh_api::stripe::{
    KeyMode, MAX_BODY_BYTES, MAX_PAGES, RESTRICTED_KEY_EXPECTED, SECRET_KEY_REFUSED, STRIPE,
    STRIPE_KEY, STRIPE_VERSION, StripeClient, StripeFetched, fetch_rows,
};
use kesh_core::horloge::debut_du_jour_suisse;
use kesh_import::parse_stripe_payout_csv;
use serde_json::{Value, json};

use common::capture_trace::CaptureTrace;
use common::secrets_prestataire::{MARQUEUR_SECRET, aucun_secret, secret};

/// Le mot que portent les messages des corps d'erreur fictifs : il ne doit
/// sortir nulle part (le texte de Stripe ne sort jamais).
const TEXTE_DE_STRIPE: &str = "FUITE";

fn jour(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

/// « Maintenant » : après toutes les périodes lues.
fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 11, 20, 12, 0, 0).unwrap()
}

/// La période des fixtures : février 2020.
fn fevrier() -> Window {
    Window::new(jour(2020, 2, 1), jour(2020, 2, 29), maintenant())
}

fn fixture(nom: &str) -> Value {
    let p = format!(
        "{}/tests/fixtures/stripe-api/{nom}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(&p).expect(&p)).expect(&p)
}

fn texte_fixture(nom: &str) -> String {
    fixture(nom).to_string()
}

// ---------------------------------------------------------------------------
// Serveur Stripe simulé
// ---------------------------------------------------------------------------

/// Une réponse du serveur simulé.
#[derive(Clone, Debug)]
enum Rep {
    /// 200 et ce corps JSON.
    Json(Value),
    /// Statut, `Retry-After`, `Stripe-Rate-Limited-Reason`, corps.
    Statut(u16, Option<&'static str>, Option<&'static str>, String),
}

/// Une requête reçue.
#[derive(Clone, Debug)]
struct Req {
    method: String,
    path: String,
    query: HashMap<String, String>,
    authorization: String,
    version: String,
}

type Repondeur = Arc<dyn Fn(&Req) -> Rep + Send + Sync>;

#[derive(Clone)]
struct Sim {
    forcees: Arc<Mutex<VecDeque<Rep>>>,
    repondeur: Arc<Mutex<Repondeur>>,
    requetes: Arc<Mutex<Vec<Req>>>,
    /// Attente réelle avant chaque réponse à `/v1/balance_transactions`
    /// (mesure du parallélisme, revue P1 L1).
    delai_transactions: Arc<Mutex<Option<Duration>>>,
    /// Requêtes `/v1/balance_transactions` en cours, et leur pic.
    en_vol: Arc<AtomicUsize>,
    pic: Arc<AtomicUsize>,
}

impl Sim {
    /// Réponses forcées, consommées une à une avant le cours normal.
    fn forcer(&self, r: Rep) {
        self.forcees.lock().unwrap().push_back(r);
    }
    fn repondre(&self, f: impl Fn(&Req) -> Rep + Send + Sync + 'static) {
        *self.repondeur.lock().unwrap() = Arc::new(f);
    }
    fn requetes(&self) -> Vec<Req> {
        self.requetes.lock().unwrap().clone()
    }
    fn compte(&self, path: &str) -> usize {
        self.requetes().iter().filter(|r| r.path == path).count()
    }
}

/// Le cours normal : les fixtures. Versements : page 1 sans curseur, page 2
/// après `po_TEST0002` ; transactions : par `payout`.
fn cours_normal(req: &Req) -> Rep {
    match req.path.as_str() {
        "/v1/payouts" => match req.query.get("starting_after").map(String::as_str) {
            None => Rep::Json(fixture("payouts-p1.json")),
            Some("po_TEST0002") => Rep::Json(fixture("payouts-p2.json")),
            Some(other) => panic!("curseur inattendu : {other}"),
        },
        "/v1/balance_transactions" => {
            let po = req.query.get("payout").expect("payout");
            Rep::Json(fixture(&format!("bt-{po}.json")))
        }
        other => panic!("chemin inattendu : {other}"),
    }
}

async fn gerer(
    State(sim): State<Sim>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    Query(query): Query<HashMap<String, String>>,
) -> Response {
    let entete = |n: &str| {
        headers
            .get(n)
            .and_then(|v| v.to_str().ok())
            .unwrap_or_default()
            .to_string()
    };
    let req = Req {
        method: method.to_string(),
        path: uri.path().to_string(),
        query,
        authorization: entete("authorization"),
        version: entete("stripe-version"),
    };
    sim.requetes.lock().unwrap().push(req.clone());
    let delai = *sim.delai_transactions.lock().unwrap();
    if let (Some(d), "/v1/balance_transactions") = (delai, req.path.as_str()) {
        let n = sim.en_vol.fetch_add(1, Ordering::SeqCst) + 1;
        sim.pic.fetch_max(n, Ordering::SeqCst);
        tokio::time::sleep(d).await;
        sim.en_vol.fetch_sub(1, Ordering::SeqCst);
    }
    let forcee = sim.forcees.lock().unwrap().pop_front();
    let rep = match forcee {
        Some(r) => r,
        None => {
            let f = sim.repondeur.lock().unwrap().clone();
            f(&req)
        }
    };
    match rep {
        Rep::Json(v) => (StatusCode::OK, v.to_string()).into_response(),
        Rep::Statut(code, retry, motif, corps) => {
            let mut r = (StatusCode::from_u16(code).unwrap(), corps).into_response();
            if let Some(v) = retry {
                r.headers_mut().insert("retry-after", v.parse().unwrap());
            }
            if let Some(v) = motif {
                r.headers_mut()
                    .insert("stripe-rate-limited-reason", v.parse().unwrap());
            }
            r
        }
    }
}

async fn simuler() -> (String, Sim) {
    let sim = Sim {
        forcees: Arc::default(),
        repondeur: Arc::new(Mutex::new(Arc::new(cours_normal))),
        requetes: Arc::default(),
        delai_transactions: Arc::default(),
        en_vol: Arc::default(),
        pic: Arc::default(),
    };
    let app = Router::new().fallback(gerer).with_state(sim.clone());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: SocketAddr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });
    (format!("http://{addr}"), sim)
}

/// Attente notée, qui ne dort pas.
#[derive(Default)]
struct Notee(Mutex<Vec<Duration>>);

impl Wait for Notee {
    fn wait(&self, d: Duration) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        self.0.lock().unwrap().push(d);
        Box::pin(async {})
    }
}

impl Notee {
    fn secondes(&self) -> Vec<u64> {
        self.0
            .lock()
            .unwrap()
            .iter()
            .map(|d| d.as_secs_f64().round() as u64)
            .collect()
    }
}

struct Banc {
    client: Arc<StripeClient>,
    sim: Sim,
    attente: Arc<Notee>,
    cle: String,
}

/// La clé de test : restreinte, porteuse du marqueur S9.
fn cle_de_test() -> String {
    format!("rk_test_{}", secret("cle").replace('-', "_"))
}

async fn banc() -> Banc {
    let (url, sim) = simuler().await;
    banc_sur(&url, sim)
}

/// Le banc d'un client dirigé vers `url` (le serveur simulé `sim` ne reçoit
/// alors rien si `url` vise un autre serveur).
fn banc_sur(url: &str, sim: Sim) -> Banc {
    let cle = cle_de_test();
    let serie = ProviderSeries::for_tests(1, "Boutique", &[(STRIPE_KEY, cle.as_str())]);
    let config = ProviderApiConfig::for_tests(STRIPE, url, 2, vec![serie.clone()]);
    let attente = Arc::new(Notee::default());
    let client = StripeClient::new(
        &config,
        &serie,
        Arc::new(ProviderGate::new(2)),
        attente.clone(),
        None,
    )
    .unwrap();
    Banc {
        client: Arc::new(client),
        sim,
        attente,
        cle,
    }
}

impl Banc {
    async fn lire(&self) -> Result<StripeFetched, AppError> {
        fetch_rows(self.client.clone(), fevrier(), 2).await
    }

    /// Contrôles communs à toute requête reçue : verbe, clé, version.
    fn controles_communs(&self) {
        let reqs = self.sim.requetes();
        assert!(!reqs.is_empty());
        for r in reqs {
            assert_eq!(r.method, "GET", "verbe interdit : {r:?}");
            assert_eq!(r.authorization, format!("Bearer {}", self.cle));
            assert_eq!(r.version, STRIPE_VERSION);
        }
    }
}

/// Le corps d'erreur d'une tâche, sans clé ni texte de Stripe.
async fn corps_sans_fuite(e: AppError) -> Value {
    let body = error_body(e, "stripe").await;
    let texte = body.to_string();
    aucun_secret([texte.as_str()]);
    assert!(!texte.contains(TEXTE_DE_STRIPE), "{texte}");
    body
}

fn journal_sans_fuite(capture: &CaptureTrace) {
    let evenements = capture.evenements();
    aucun_secret(evenements.iter().map(String::as_str));
    capture.exiger_absent(TEXTE_DE_STRIPE);
}

// ---------------------------------------------------------------------------
// Lecture nominale
// ---------------------------------------------------------------------------

/// AC-2, AC-3, AC-5 — deux pages de versements, filtre `arrival_date` aux
/// bornes des jours de Zurich, `status=paid`, 100 par page ; un versement
/// manuel écarté et rendu au compte rendu ; les lignes égalent celles de la
/// fixture CSV de 40-7a pour les mêmes versements, champ par champ.
#[tokio::test]
async fn lit_deux_pages_et_rend_les_lignes_du_rapport_csv() {
    let b = banc().await;
    let lu = b.lire().await.expect("lecture");
    b.controles_communs();

    let reqs = b.sim.requetes();
    let pages: Vec<&Req> = reqs.iter().filter(|r| r.path == "/v1/payouts").collect();
    assert_eq!(pages.len(), 2);
    let gte = debut_du_jour_suisse(jour(2020, 2, 1))
        .timestamp()
        .to_string();
    let lt = debut_du_jour_suisse(jour(2020, 3, 1))
        .timestamp()
        .to_string();
    for p in &pages {
        assert_eq!(p.query.get("arrival_date[gte]"), Some(&gte));
        assert_eq!(p.query.get("arrival_date[lt]"), Some(&lt));
        assert_eq!(p.query.get("status").map(String::as_str), Some("paid"));
        assert_eq!(p.query.get("limit").map(String::as_str), Some("100"));
        assert!(!p.query.contains_key("created[gte]"));
    }
    assert_eq!(pages[0].query.get("starting_after"), None);
    assert_eq!(
        pages[1].query.get("starting_after").map(String::as_str),
        Some("po_TEST0002")
    );
    // Les trois versements automatiques, pas le manuel.
    let mut lus: Vec<&str> = reqs
        .iter()
        .filter(|r| r.path == "/v1/balance_transactions")
        .map(|r| {
            assert_eq!(r.query.get("limit").map(String::as_str), Some("100"));
            r.query["payout"].as_str()
        })
        .collect();
    lus.sort();
    assert_eq!(lus, ["po_TEST0001", "po_TEST0002", "po_TEST0003"]);
    assert_eq!(lu.manual_payouts, ["po_TEST0005"]);

    // Égalité avec le CSV de 40-7a (sans `po_TEST0004`, absent des réponses).
    let csv = std::fs::read(format!(
        "{}/../kesh-import/tests/fixtures/stripe-itemized-payout-fictif.csv",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    let attendues: Vec<_> = parse_stripe_payout_csv(&csv)
        .unwrap()
        .rows
        .into_iter()
        .filter(|r| r.payout_id != "po_TEST0004")
        .collect();
    assert_eq!(attendues.len(), 8);
    // Ce que le CSV porte et l'API pas : la charge liée.
    let attendues: Vec<_> = attendues
        .into_iter()
        .map(|mut r| {
            r.charge_id = None;
            r
        })
        .collect();
    assert_eq!(lu.rows, attendues);
    // Témoin de Zurich : la transaction créée le 31 janvier à 23:00 UTC est
    // du 1er février.
    let frais = lu
        .rows
        .iter()
        .find(|r| r.balance_transaction_id == "txn_TEST0004")
        .unwrap();
    assert_eq!(frais.date, jour(2020, 2, 1));
}

/// AC-2 — parallélisme : avec `parallel = 1`, les versements se lisent l'un
/// après l'autre et rendent les mêmes lignes.
#[tokio::test]
async fn un_seul_fil_rend_les_memes_lignes() {
    let b = banc().await;
    let un = fetch_rows(b.client.clone(), fevrier(), 1).await.unwrap();
    let deux = b.lire().await.unwrap();
    assert_eq!(un, deux);
}

/// Revue P1, L1 — `fetch_rows` branche bien `parallel` sur la lecture des
/// transactions : avec trois versements automatiques et une réponse retardée
/// de 100 ms, le pic des requêtes `balance_transactions` en vol vaut
/// exactement `parallel` (ni tout en série, ni tout à la fois ; le sémaphore
/// du prestataire, à 2, ne borne pas en dessous).
#[tokio::test]
async fn parallel_borne_les_transactions_en_vol() {
    for parallel in [1, 2] {
        let b = banc().await;
        *b.sim.delai_transactions.lock().unwrap() = Some(Duration::from_millis(100));
        let lu = fetch_rows(b.client.clone(), fevrier(), parallel)
            .await
            .unwrap();
        assert_eq!(lu.rows.len(), 8);
        assert_eq!(
            b.sim.pic.load(Ordering::SeqCst),
            parallel,
            "pic des requêtes en vol, parallel = {parallel}"
        );
    }
}

/// Revue P1, L2 — une période qui finit après « maintenant » : la fin de la
/// fenêtre est ramenée à maintenant ([`Window::new`]) et c'est elle que porte
/// `arrival_date[lt]`, le début restant le premier jour à minuit à Zurich.
#[tokio::test]
async fn periode_jusqu_a_aujourd_hui_finit_maintenant() {
    let b = banc().await;
    let w = Window::new(jour(2026, 11, 1), jour(2026, 11, 30), maintenant());
    b.client.payouts(&w).await.unwrap();
    let pages: Vec<Req> = b
        .sim
        .requetes()
        .into_iter()
        .filter(|r| r.path == "/v1/payouts")
        .collect();
    assert!(!pages.is_empty());
    let gte = debut_du_jour_suisse(jour(2026, 11, 1))
        .timestamp()
        .to_string();
    let lt = maintenant().timestamp().to_string();
    for p in &pages {
        assert_eq!(p.query.get("arrival_date[gte]"), Some(&gte));
        assert_eq!(p.query.get("arrival_date[lt]"), Some(&lt));
    }
}

// ---------------------------------------------------------------------------
// Erreurs
// ---------------------------------------------------------------------------

/// AC-4 — 401 : `PROVIDER_AUTH_FAILED`, sans reprise ; ni la clé ni le texte de
/// Stripe (qui cite un fragment de la clé) dans le corps ni au journal.
#[tokio::test]
async fn cle_refusee() {
    let capture = CaptureTrace::installer();
    let b = banc().await;
    b.sim.forcer(Rep::Statut(
        401,
        None,
        None,
        texte_fixture("erreur-401.json"),
    ));
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(e, AppError::ProviderAuthFailed { provider: "stripe" }),
        "{e:?}"
    );
    assert_eq!(b.sim.compte("/v1/payouts"), 1, "aucune reprise sur 401");
    let body = corps_sans_fuite(e).await;
    assert_eq!(body["code"], "PROVIDER_AUTH_FAILED");
    capture.exiger_ensemble(&["clé refusée", "payouts"]);
    journal_sans_fuite(&capture);
}

/// AC-4 — 400 : `PROVIDER_API_ERROR` nommant la période et la page ; seuls
/// `error.code` et `error.param` vont au journal, jamais `error.message`.
#[tokio::test]
async fn requete_refusee() {
    let capture = CaptureTrace::installer();
    let b = banc().await;
    b.sim.repondre(|req| {
        if req.query.contains_key("starting_after") {
            Rep::Statut(400, None, None, texte_fixture("erreur-400.json"))
        } else {
            cours_normal(req)
        }
    });
    let e = b.lire().await.unwrap_err();
    match &e {
        AppError::ProviderApiError {
            provider: "stripe",
            window,
            page,
        } => {
            assert_eq!(window.as_deref(), Some("2020-02-01…2020-02-29"));
            assert_eq!(*page, Some(2));
        }
        other => panic!("PROVIDER_API_ERROR attendu : {other:?}"),
    }
    let body = corps_sans_fuite(e).await;
    assert_eq!(body["code"], "PROVIDER_API_ERROR");
    assert_eq!(body["details"]["page"], 2);
    capture.exiger_ensemble(&["parameter_unknown", "arrival_date[gtx]", "statut=400"]);
    journal_sans_fuite(&capture);
}

/// AC-4 — 403 sur les transactions : `PROVIDER_PERMISSION_DENIED`, la
/// ressource au journal.
#[tokio::test]
async fn droits_insuffisants_sur_les_transactions() {
    let capture = CaptureTrace::installer();
    let b = banc().await;
    b.sim.repondre(|req| {
        if req.path == "/v1/balance_transactions" {
            Rep::Statut(403, None, None, texte_fixture("erreur-403.json"))
        } else {
            cours_normal(req)
        }
    });
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(e, AppError::ProviderPermissionDenied { .. }),
        "{e:?}"
    );
    let body = corps_sans_fuite(e).await;
    assert_eq!(body["code"], "PROVIDER_PERMISSION_DENIED");
    capture.exiger_ensemble(&["droits insuffisants", "balance_transactions"]);
    journal_sans_fuite(&capture);
}

/// AC-4 — 429 puis réussite : la même requête reprise après la pause de
/// `Retry-After` (notée, sans dormir) ; le motif de Stripe au journal.
#[tokio::test]
async fn debit_limite_puis_reprise() {
    let capture = CaptureTrace::installer();
    let b = banc().await;
    b.sim.forcer(Rep::Statut(
        429,
        Some("3"),
        Some("global-rate"),
        texte_fixture("erreur-429.json"),
    ));
    let lu = b.lire().await.expect("reprise");
    assert_eq!(lu.rows.len(), 8);
    assert_eq!(b.sim.compte("/v1/payouts"), 3, "page 1 reprise une fois");
    // La pause de `Retry-After` (3 s), commune : l'attente notée ne dort pas,
    // si bien que les requêtes suivantes, partant dans les 3 s réelles,
    // attendent le reste de la même pause — jamais une pause de plus.
    let pauses = b.attente.secondes();
    assert_eq!(pauses.first(), Some(&3), "{pauses:?}");
    assert!(pauses.iter().all(|s| *s <= 3), "{pauses:?}");
    capture.exiger_ensemble(&["débit limité", "global-rate"]);
    journal_sans_fuite(&capture);
}

/// AC-4 — six 429 : `PROVIDER_RATE_LIMITED` ; six 5xx : `PROVIDER_UNAVAILABLE`.
#[tokio::test]
async fn reprise_epuisee() {
    let b = banc().await;
    for _ in 0..6 {
        b.sim.forcer(Rep::Statut(
            429,
            None,
            None,
            texte_fixture("erreur-429.json"),
        ));
    }
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(&e, AppError::ProviderRateLimited { page: Some(1), .. }),
        "{e:?}"
    );
    assert_eq!(b.sim.compte("/v1/payouts"), 6);
    assert_eq!(b.attente.secondes(), [5, 10, 20, 40, 60]);
    corps_sans_fuite(e).await;

    let b = banc().await;
    for _ in 0..6 {
        b.sim
            .forcer(Rep::Statut(503, None, None, "<html>FUITE</html>".into()));
    }
    let e = b.lire().await.unwrap_err();
    assert!(matches!(&e, AppError::ProviderUnavailable { .. }), "{e:?}");
    corps_sans_fuite(e).await;
}

/// AC-4 — corps illisible, montant non entier : `PROVIDER_RESPONSE_INVALID`,
/// sans la réponse.
#[tokio::test]
async fn reponse_illisible() {
    let b = banc().await;
    b.sim
        .forcer(Rep::Statut(200, None, None, "<html>FUITE</html>".into()));
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(&e, AppError::ProviderResponseInvalid { .. }),
        "{e:?}"
    );
    let body = corps_sans_fuite(e).await;
    assert_eq!(body["details"]["reason"], "page");

    let b = banc().await;
    b.sim.repondre(|req| {
        if req.path == "/v1/balance_transactions" {
            let mut v = fixture("bt-po_TEST0001.json");
            v["data"][1]["amount"] = json!(10.5);
            Rep::Json(v)
        } else {
            cours_normal(req)
        }
    });
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(&e, AppError::ProviderResponseInvalid { .. }),
        "{e:?}"
    );
}

/// AC-2 — une page `has_more` vide ne fait pas tourner sans fin.
#[tokio::test]
async fn curseur_sans_objet() {
    let b = banc().await;
    b.sim.forcer(Rep::Json(
        json!({"object":"list","data":[],"has_more":true,"url":"/v1/payouts"}),
    ));
    let e = b.lire().await.unwrap_err();
    match e {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["reason"], "cursor");
        }
        other => panic!("refus attendu : {other:?}"),
    }
}

/// AC-2 — un curseur qui ne bouge pas (la même page rendue deux fois avec
/// `has_more`) ne fait pas tourner sans fin.
#[tokio::test]
async fn curseur_immobile() {
    let b = banc().await;
    b.sim.repondre(|req| {
        if req.path == "/v1/payouts" {
            Rep::Json(fixture("payouts-p1.json"))
        } else {
            cours_normal(req)
        }
    });
    match b.lire().await.unwrap_err() {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["reason"], "cursor");
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(b.sim.compte("/v1/payouts"), 2);
}

/// Une page d'un seul versement automatique d'identifiant `id`, `has_more`.
fn page_pleine(id: &str) -> Value {
    let mut v = fixture("payouts-p1.json");
    let mut po = v["data"][0].clone();
    po["id"] = json!(id);
    v["data"] = json!([po]);
    v["has_more"] = json!(true);
    v
}

/// Revue P1, L3 — une liste qui ne finit jamais (curseur neuf à chaque page) :
/// `PROVIDER_RESPONSE_INVALID`, `too_many_pages`, après exactement
/// [`MAX_PAGES`] pages.
#[tokio::test]
async fn trop_de_pages() {
    let b = banc().await;
    b.sim.repondre(|req| {
        let n: u32 = match req.query.get("starting_after") {
            None => 0,
            Some(c) => c.trim_start_matches("po_GEN").parse::<u32>().unwrap() + 1,
        };
        Rep::Json(page_pleine(&format!("po_GEN{n}")))
    });
    match b.lire().await.unwrap_err() {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["reason"], "too_many_pages");
            assert_eq!(details["resource"], "payouts");
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(b.sim.compte("/v1/payouts"), MAX_PAGES as usize);
}

/// Revue P1, L4 — un cycle de curseur de longueur 2 (A, B, A, B…) n'est pas
/// reconnu comme un curseur immobile, mais il est **borné** : la lecture
/// s'arrête sur `too_many_pages` après [`MAX_PAGES`] pages, sans boucle sans
/// fin.
#[tokio::test]
async fn cycle_de_curseur_borne_par_max_pages() {
    let b = banc().await;
    b.sim.repondre(|req| {
        let id = match req.query.get("starting_after").map(String::as_str) {
            None | Some("po_CYCB") => "po_CYCA",
            Some("po_CYCA") => "po_CYCB",
            Some(other) => panic!("curseur inattendu : {other}"),
        };
        Rep::Json(page_pleine(id))
    });
    match b.lire().await.unwrap_err() {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["reason"], "too_many_pages");
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(b.sim.compte("/v1/payouts"), MAX_PAGES as usize);
}

/// Revue P1, L3 — un corps plus grand que [`MAX_BODY_BYTES`] :
/// `PROVIDER_RESPONSE_INVALID`, `too_large`, sans reprise et sans la réponse.
#[tokio::test]
async fn corps_trop_grand() {
    let b = banc().await;
    b.sim
        .forcer(Rep::Statut(200, None, None, "x".repeat(MAX_BODY_BYTES + 1)));
    let e = b.lire().await.unwrap_err();
    match &e {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["reason"], "too_large");
            assert_eq!(details["resource"], "payouts");
        }
        other => panic!("refus attendu : {other:?}"),
    }
    assert_eq!(b.sim.compte("/v1/payouts"), 1, "aucune reprise");
    corps_sans_fuite(e).await;
}

/// Revue P1, L3 — une erreur de transport définitive (TLS parlé à un serveur
/// en clair, comme un certificat refusé) : `PROVIDER_API_ERROR` nommant la
/// période et la page, au premier essai, sans pause.
#[tokio::test]
async fn transport_definitif_sans_reprise() {
    let capture = CaptureTrace::installer();
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
    let (_, sim) = simuler().await;
    let b = banc_sur(&format!("https://{adresse}"), sim);
    let e = b.lire().await.unwrap_err();
    match &e {
        AppError::ProviderApiError {
            provider: "stripe",
            window,
            page,
        } => {
            assert_eq!(window.as_deref(), Some("2020-02-01…2020-02-29"));
            assert_eq!(*page, Some(1));
        }
        other => panic!("PROVIDER_API_ERROR attendu : {other:?}"),
    }
    assert!(b.attente.secondes().is_empty(), "aucune pause");
    assert_eq!(connexions.load(Ordering::SeqCst), 1, "un seul essai");
    corps_sans_fuite(e).await;
    capture.exiger_ensemble(&["erreur de transport définitive"]);
    journal_sans_fuite(&capture);
}

/// Revue P1, L3 — 408 (délai dépassé côté serveur) est passager comme un
/// 5xx : six essais, cinq pauses, puis `PROVIDER_UNAVAILABLE`.
#[tokio::test]
async fn delai_serveur_408_repris_puis_indisponible() {
    let b = banc().await;
    for _ in 0..6 {
        b.sim.forcer(Rep::Statut(408, None, None, "FUITE".into()));
    }
    let e = b.lire().await.unwrap_err();
    assert!(
        matches!(&e, AppError::ProviderUnavailable { page: Some(1), .. }),
        "{e:?}"
    );
    assert_eq!(b.sim.compte("/v1/payouts"), 6);
    assert_eq!(b.attente.secondes().len(), 5);
    corps_sans_fuite(e).await;
}

/// AC-3 — une transaction manquante : Σ net ≠ montant du versement,
/// `PROVIDER_RESPONSE_INVALID` nommant le versement ; rien n'est rendu.
#[tokio::test]
async fn transaction_manquante_echoue() {
    let b = banc().await;
    b.sim.repondre(|req| {
        if req.query.get("payout").map(String::as_str) == Some("po_TEST0001") {
            let mut v = fixture("bt-po_TEST0001.json");
            v["data"].as_array_mut().unwrap().pop();
            Rep::Json(v)
        } else {
            cours_normal(req)
        }
    });
    match b.lire().await.unwrap_err() {
        AppError::ProviderResponseInvalid { details, .. } => {
            assert_eq!(details["payoutId"], "po_TEST0001");
            assert_eq!(details["expected"], "125.15");
            assert_eq!(details["found"], "28.35");
        }
        other => panic!("refus attendu : {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Configuration (AC-1)
// ---------------------------------------------------------------------------

fn lire_env(pairs: &[(&str, &str)]) -> Result<ProviderApiConfig, ConfigError> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let names: Vec<String> = map.keys().cloned().collect();
    read(
        STRIPE,
        |k| {
            map.get(k)
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
        },
        &names,
    )
}

fn refus(pairs: &[(&str, &str)]) -> (String, &'static str, String) {
    match lire_env(pairs) {
        Err(e @ ConfigError::InvalidProviderVar { .. }) => {
            let msg = e.to_string();
            let ConfigError::InvalidProviderVar { var, reason } = e else {
                unreachable!()
            };
            (var, reason, msg)
        }
        other => panic!("refus attendu : {other:?}"),
    }
}

#[test]
fn configuration_de_la_cle() {
    // Sans variable Stripe, rien ; une série vide vaut absente.
    assert_eq!(lire_env(&[]).unwrap().series_count(), 0);
    let c = lire_env(&[("KESH_STRIPE_1_ACCOUNT", " "), ("KESH_STRIPE_1_KEY", "")]).unwrap();
    assert_eq!(c.series_count(), 0);
    assert_eq!(c.base_url, "https://api.stripe.com");

    let cle = cle_de_test();
    let c = lire_env(&[
        ("KESH_STRIPE_2_ACCOUNT", "Boutique"),
        ("KESH_STRIPE_2_KEY", cle.as_str()),
        ("KESH_STRIPE_BASE_URL", "http://127.0.0.1:4242"),
        ("KESH_STRIPE_PARALLEL", "3"),
    ])
    .unwrap();
    assert_eq!(
        c.series_for("boutique").unwrap().field(STRIPE_KEY),
        Some(cle.as_str())
    );
    assert_eq!(
        (c.base_url.as_str(), c.parallel),
        ("http://127.0.0.1:4242", 3)
    );
    let dbg = format!("{c:?}");
    assert!(!dbg.contains(MARQUEUR_SECRET), "{dbg}");

    // Moitié de paire : refus nommant la variable manquante.
    assert_eq!(
        refus(&[("KESH_STRIPE_1_KEY", cle.as_str())]).0,
        "KESH_STRIPE_1_ACCOUNT"
    );
    assert_eq!(
        refus(&[("KESH_STRIPE_1_ACCOUNT", "B")]).0,
        "KESH_STRIPE_1_KEY"
    );

    // Clé secrète, publiable, gabarit : refus ; le message ne cite pas la valeur.
    let secrete = format!("sk_live_{MARQUEUR_SECRET}");
    let (var, reason, msg) = refus(&[
        ("KESH_STRIPE_1_ACCOUNT", "B"),
        ("KESH_STRIPE_1_KEY", secrete.as_str()),
    ]);
    assert_eq!(
        (var.as_str(), reason),
        ("KESH_STRIPE_1_KEY", SECRET_KEY_REFUSED)
    );
    assert!(
        msg.contains("KESH_STRIPE_1_KEY") && !msg.contains(MARQUEUR_SECRET),
        "{msg}"
    );
    let (_, reason, _) = refus(&[
        ("KESH_STRIPE_1_ACCOUNT", "B"),
        ("KESH_STRIPE_1_KEY", "pk_live_abc"),
    ]);
    assert_eq!(reason, RESTRICTED_KEY_EXPECTED);
    let (var, reason, _) = refus(&[
        ("KESH_STRIPE_1_ACCOUNT", "B"),
        ("KESH_STRIPE_1_KEY", "<rk_live_…>"),
    ]);
    assert_eq!(
        (var.as_str(), reason),
        ("KESH_STRIPE_1_KEY", "valeur de gabarit <…>")
    );
    assert_eq!(
        refus(&[("KESH_STRIPE_BASE_URL", "http://api.stripe.com")]).0,
        "KESH_STRIPE_BASE_URL"
    );
}

/// Le mode de la clé se lit au préfixe (montré par 40-8b).
#[tokio::test]
async fn mode_de_la_cle() {
    let b = banc().await;
    assert_eq!(b.client.mode(), Some(KeyMode::Test));
}
